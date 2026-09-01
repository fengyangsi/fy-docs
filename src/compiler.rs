//! Invokes the typst CLI and renders the self-contained static page(s) into
//! `docs/target/`, plus the print-edition PDF(s) into `docs/release/`.

use crate::project::{LanguageTarget, Project};
use crate::state::AppState;
use anyhow::{Context, Result, anyhow, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) const INDEX_FILE: &str = "index.html";
pub(crate) const SKIN_FILE: &str = "fy-docs.css";
pub(crate) const TYPST_CSS_FILE: &str = "typst.css";
pub(crate) const VIEWER_JS_FILE: &str = "fy-docs.js";
pub(crate) const LIVE_JS_FILE: &str = "live.js";

/// Prefix of the throwaway HTML file each language target compiles through.
const TEMP_PREFIX: &str = "_temp_";

/// The oldest typst release that accepts every flag fy-docs passes:
/// `--pdf-standard 2.0` first shipped in Typst 0.14.
const MINIMUM_TYPST: (u64, u64, u64) = (0, 14, 0);

/// Verifies that the `typst` CLI exists and is new enough for the flags
/// fy-docs passes (`--features html`, `--pdf-standard 2.0`).
pub(crate) fn precheck() -> Result<()> {
    let output = Command::new("typst").arg("--version").output().context(
        "typst was not found on PATH — install Typst 0.14 or later \
             (https://github.com/typst/typst/releases)",
    )?;
    let banner = String::from_utf8_lossy(&output.stdout);
    match typst_banner_version(&banner) {
        Some(version) if version < MINIMUM_TYPST => bail!(
            "typst {}.{}.{} is too old — fy-docs requires Typst 0.14 or later \
             for `--pdf-standard 2.0`; upgrade at \
             https://github.com/typst/typst/releases",
            version.0,
            version.1,
            version.2
        ),
        // An unrecognized banner must not block a working install; the
        // compile step surfaces real errors anyway.
        _ => Ok(()),
    }
}

/// Parses the `typst 0.15.1 (<hash>)` version banner into (major, minor, patch).
fn typst_banner_version(banner: &str) -> Option<(u64, u64, u64)> {
    let version = banner.split_whitespace().nth(1)?;
    let mut parts = version.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    Some((major, minor, patch))
}

/// Generates HTML page(s) into `docs/target/`, optionally with PDF(s).
/// Failures become visible error pages at the affected targets; returns
/// `false` when anything failed so batch commands can exit non-zero.
pub(crate) fn generate_into(state: &AppState, with_pdf: bool, lang_filter: Option<&str>) -> bool {
    crate::state::log(&format!(
        "[fy-docs] compiling `{}` (v{})...",
        state.project.name, state.project.version
    ));
    match generate(&state.project, with_pdf, lang_filter) {
        Ok(()) => {
            crate::state::log(" ok");
            true
        }
        Err(err) => {
            crate::state::log(" FAILED");
            crate::state::log(&format!("[fy-docs] {err:#}"));
            false
        }
    }
}

/// Page pieces extracted from one typst HTML export: title, styles, body.
type HtmlParts = (String, String, String);

fn generate(project: &Project, with_pdf: bool, lang_filter: Option<&str>) -> Result<()> {
    let target = &project.target_dir;
    fs::create_dir_all(target)?;
    clean_compile_temporaries(target)?;

    let selected_targets = select_targets(project, lang_filter)?;

    if with_pdf && let Err(err) = compile_pdf(project, lang_filter) {
        write_error_pages(project, &selected_targets, &format!("{err:#}"));
        ensure_landing_page(project, target)?;
        return Err(err);
    }

    // Parallel compilation of all language targets for HTML export. Every
    // thread reports its target even on failure, so only the affected pages
    // degrade to error output while successful targets keep fresh pages.
    let (results, panic_note) = std::thread::scope(|s| {
        let mut handles = Vec::new();
        for lang_target in &selected_targets {
            handles.push(s.spawn(move || {
                (
                    lang_target,
                    compile_html_target(project, lang_target, target),
                )
            }));
        }
        let mut results = Vec::new();
        let mut panic_note = None;
        for handle in handles {
            match handle.join() {
                Ok(pair) => results.push(pair),
                Err(payload) => panic_note = Some(panic_message(&payload)),
            }
        }
        (results, panic_note)
    });
    if let Some(note) = panic_note {
        write_error_pages(project, &selected_targets, &note);
        bail!("compile thread panicked: {note}");
    }

    let mut combined_styles = String::new();
    let mut rendered_pages: Vec<(&LanguageTarget, String, String)> = Vec::new();
    let mut failures: Vec<(&LanguageTarget, String)> = Vec::new();

    for (lang_target, parts) in results {
        match parts {
            Ok((title, styles, body)) => {
                if combined_styles.is_empty() {
                    combined_styles = styles;
                } else if !styles.is_empty() && !combined_styles.contains(&styles) {
                    combined_styles.push_str("\n/* additional language styles */\n");
                    combined_styles.push_str(&styles);
                }
                rendered_pages.push((lang_target, title, body));
            }
            Err(err) => failures.push((lang_target, format!("{err:#}"))),
        }
    }

    write_atomic(&target.join(TYPST_CSS_FILE), &combined_styles)?;
    write_atomic(&target.join(SKIN_FILE), crate::assets::BASE_CSS)?;
    write_atomic(&target.join(VIEWER_JS_FILE), crate::assets::VIEWER_JS)?;
    write_atomic(&target.join(LIVE_JS_FILE), crate::assets::LIVE_JS)?;

    for (lang_target, title, body) in &rendered_pages {
        let page_html = crate::assets::doc_page(
            title,
            &project.name,
            project.repository.as_deref(),
            body.trim(),
            Some(lang_target),
            &project.targets,
        );
        write_atomic(&target.join(&lang_target.html_file_name), &page_html)?;
    }

    for (lang_target, err) in &failures {
        if let Err(write_err) =
            write_error_page(project, err, &lang_target.html_file_name, &lang_target.lang)
        {
            crate::state::log(&format!(
                "[fy-docs] could not write error page for [{}]: {write_err}",
                lang_label(lang_target)
            ));
        }
    }

    // Always leave an index.html behind: either a target wrote it directly,
    // the page of a single-language project is copied, or multi-language
    // builds (including partial failures) keep their routing landing page.
    let has_index = rendered_pages
        .iter()
        .any(|(t, ..)| t.html_file_name == INDEX_FILE)
        || failures.iter().any(|(t, _)| t.html_file_name == INDEX_FILE);
    if !has_index {
        // The project is single-language only when *all* of its targets
        // collapse to one; a multi-language project with one surviving page
        // must still get the landing page so language routing survives.
        if project.targets.len() == 1 && rendered_pages.len() == 1 {
            // Single-language subfolder: copy page directly to index.html
            let (first_target, title, body) = &rendered_pages[0];
            let default_page = crate::assets::doc_page(
                title,
                &project.name,
                project.repository.as_deref(),
                body.trim(),
                Some(first_target),
                &project.targets,
            );
            write_atomic(&target.join(INDEX_FILE), &default_page)?;
        } else {
            // Multi-language: write lightweight (~500B) client-side redirect landing page
            let landing = crate::assets::redirect_page(&project.targets);
            write_atomic(&target.join(INDEX_FILE), &landing)?;
        }
    }

    ensure_gitignore(project, &["/docs/target/"]);

    if failures.is_empty() {
        Ok(())
    } else {
        let summary = failures
            .iter()
            .map(|(t, err)| format!("[{}] {err}", lang_label(t)))
            .collect::<Vec<_>>()
            .join("\n");
        Err(anyhow!("{summary}"))
    }
}

/// Sweeps the `_temp_*.html` intermediates a compile killed mid-way left in
/// the output directory: a target that finishes removes its own intermediate
/// on success and on failure alike, so anything still there is orphaned.
fn clean_compile_temporaries(target: &Path) -> Result<()> {
    for entry in fs::read_dir(target)?.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(TEMP_PREFIX) && name.ends_with(".html") {
            let _ = fs::remove_file(entry.path());
        }
    }
    Ok(())
}

/// Writes a file atomically: the content lands in a sibling temp file that
/// is renamed over the destination, so a concurrent HTTP read from the dev
/// server never observes a half-written page.
fn write_atomic(path: &Path, contents: &str) -> Result<()> {
    use std::io::Write as _;
    let parent = path
        .parent()
        .context("output path has no parent directory")?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)
        .with_context(|| format!("could not create a temp file next to {}", path.display()))?;
    temp.write_all(contents.as_bytes())?;
    temp.persist(path)
        .map_err(|err| anyhow!("could not replace {}: {err}", path.display()))?;
    Ok(())
}

/// The display label of a target's language: its tag, or `default` for a
/// root `main.typ` target.
fn lang_label(target: &LanguageTarget) -> &str {
    if target.lang.is_empty() {
        "default"
    } else {
        &target.lang
    }
}

/// Compiles one language target to HTML in a temp file and extracts the page
/// pieces. The temp file is removed on every path, including failures.
fn compile_html_target(
    project: &Project,
    lang_target: &LanguageTarget,
    target_dir: &Path,
) -> Result<HtmlParts> {
    let temp_html = target_dir.join(format!(
        "{TEMP_PREFIX}{}_{}.html",
        if lang_target.lang.is_empty() {
            "root"
        } else {
            &lang_target.lang
        },
        std::process::id()
    ));
    let parts = (|| -> Result<HtmlParts> {
        run(Command::new("typst")
            .args([
                "compile",
                "--features",
                "html",
                "--format",
                "html",
                "--root",
            ])
            .arg(&project.root)
            .arg(&lang_target.entry)
            .arg(&temp_html))
        .with_context(|| {
            format!(
                "typst HTML export failed for [{}] {}",
                lang_label(lang_target),
                lang_target.entry.display()
            )
        })?;

        let html = fs::read_to_string(&temp_html)?;
        let title =
            extract_between(&html, "<title>", "</title>").unwrap_or_else(|| project.name.clone());
        let styles = extract_all_styles(&html);
        let body = extract_body(&html).context("typst HTML export contains no <body>")?;
        Ok((title, styles, body))
    })();
    let _ = fs::remove_file(&temp_html);
    parts
}

/// Compiles the print-edition PDF 2.0 specifications into `docs/release/` in parallel.
pub(crate) fn compile_pdf(project: &Project, lang_filter: Option<&str>) -> Result<Vec<PathBuf>> {
    fs::create_dir_all(&project.release_dir)?;
    let selected_targets = select_targets(project, lang_filter)?;

    let (results, panic_note) = std::thread::scope(|s| {
        let mut handles = Vec::new();
        for lang_target in selected_targets {
            handles.push(s.spawn(move || {
                let release_path = project.release_dir.join(&lang_target.pdf_file_name);
                run(Command::new("typst")
                    .args(["compile", "--root"])
                    .arg(&project.root)
                    .args(["--pdf-standard", "2.0"])
                    .arg(&lang_target.entry)
                    .arg(&release_path))
                .with_context(|| {
                    format!(
                        "typst PDF export failed for [{}] {}",
                        lang_label(lang_target),
                        lang_target.entry.display()
                    )
                })
                .map(|()| release_path)
            }));
        }
        let mut results = Vec::new();
        let mut panic_note = None;
        for handle in handles {
            match handle.join() {
                Ok(res) => results.push(res),
                Err(payload) => panic_note = Some(panic_message(&payload)),
            }
        }
        (results, panic_note)
    });
    if let Some(note) = panic_note {
        bail!("compile thread panicked: {note}");
    }

    let mut generated_paths = Vec::new();
    for res in results {
        generated_paths.push(res?);
    }

    ensure_gitignore(project, &["/docs/release/"]);
    Ok(generated_paths)
}

/// Makes sure the generated directories stay ignored (see
/// [`crate::project::ensure_gitignore`]).
fn ensure_gitignore(project: &Project, entries: &[&str]) {
    let root = project.docs_dir.parent().unwrap_or(&project.docs_dir);
    crate::project::ensure_gitignore(root, entries);
}

/// Resolves the language filter into targets, refusing a filter that matches
/// no language so a typo cannot quietly build something else instead.
fn select_targets<'a>(
    project: &'a Project,
    lang_filter: Option<&str>,
) -> Result<Vec<&'a LanguageTarget>> {
    let selected = project.selected_targets(lang_filter);
    if selected.is_empty() {
        let available = project
            .targets
            .iter()
            .map(lang_label)
            .collect::<Vec<_>>()
            .join(", ");
        match lang_filter {
            Some(filter) => bail!(
                "no documentation language target matches `{filter}`; \
                 this project provides: {available}"
            ),
            None => bail!("this documentation project declares no language targets"),
        }
    }
    Ok(selected)
}

fn run(cmd: &mut Command) -> Result<()> {
    let output = cmd
        .output()
        .context("failed to spawn `typst` (is it on PATH?)")?;
    if output.status.success() {
        if let Some(note) = warnings_note(&output.stderr) {
            crate::state::log(&note);
        }
        return Ok(());
    }
    let stderr = crate::state::strip_verbatim(&String::from_utf8_lossy(&output.stderr));
    let stdout = crate::state::strip_verbatim(&String::from_utf8_lossy(&output.stdout));
    if stderr.trim().is_empty() {
        bail!("{stdout}")
    } else {
        bail!("{stderr}")
    }
}

/// Builds the line forwarding typst's stderr from a *successful* compile.
/// Warnings change the artifact without failing it (substituted fonts,
/// directives dropped by HTML export), so dropping them would hide real
/// regressions behind a green build.
///
/// Absent font families are the one exception: a fallback chain deliberately
/// lists candidates for several operating systems, and typst re-reports every
/// unavailable one at each style site, so the repeats add nothing beyond the
/// distinct names. They collapse into a single line; everything else is
/// forwarded verbatim.
fn warnings_note(stderr: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(stderr).trim().to_owned();
    let mut missing_fonts: Vec<String> = Vec::new();
    let mut kept: Vec<String> = Vec::new();
    for block in split_warning_blocks(&text) {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }
        // An unrecognized block shape stays as typst wrote it: losing a warning
        // to a parser assumption would be worse than the noise.
        if let Some(family) = unmatched_font(block) {
            if !missing_fonts.contains(&family) {
                missing_fonts.push(family);
            }
        } else {
            kept.push(block.to_owned());
        }
    }
    if kept.is_empty() && missing_fonts.is_empty() {
        return None;
    }
    let mut note = String::from("[fy-docs] typst reported warnings:");
    for block in kept {
        note.push('\n');
        note.push_str(&block);
    }
    if !missing_fonts.is_empty() {
        note.push_str(&format!(
            "\nwarning: font families unavailable, fallback applied: {}",
            missing_fonts.join(", ")
        ));
    }
    Some(crate::state::strip_verbatim(&note))
}

/// Splits diagnostics into blocks at each line that starts a new warning,
/// independent of the blank-line separators typst happens to emit.
fn split_warning_blocks(text: &str) -> Vec<String> {
    let mut blocks: Vec<String> = Vec::new();
    for line in text.lines() {
        if blocks.is_empty() || line.starts_with("warning:") {
            blocks.push(line.to_owned());
        } else if let Some(last) = blocks.last_mut() {
            last.push('\n');
            last.push_str(line);
        }
    }
    blocks
}

/// Extracts the family name from a typst "unknown font family" warning.
fn unmatched_font(block: &str) -> Option<String> {
    let line = block.lines().next()?.trim();
    let message = line.strip_prefix("warning:")?.trim();
    let family = message.strip_prefix("unknown font family:")?.trim();
    (!family.is_empty()).then(|| family.to_lowercase())
}

/// Formats a panicked thread's payload for an error summary.
fn panic_message(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(text) = payload.downcast_ref::<&str>() {
        (*text).to_owned()
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text.clone()
    } else {
        "unknown panic payload".to_owned()
    }
}

fn extract_between(haystack: &str, start: &str, end: &str) -> Option<String> {
    let from = haystack.find(start)? + start.len();
    let to = haystack[from..].find(end)? + from;
    Some(haystack[from..to].to_owned())
}

/// Extracts the `<body>` inner HTML, tolerating attributes on the opening tag
/// (`<body lang="en">`) so a typst export format drift cannot break page
/// assembly.
fn extract_body(html: &str) -> Option<String> {
    let tag = html.find("<body")?;
    let from = html[tag..].find('>')? + tag + 1;
    let to = html[from..].find("</body>")? + from;
    Some(html[from..to].to_owned())
}

/// Concatenates every `<style>` block's CSS from a typst HTML export. The
/// opening tag may carry attributes (`<style media="print">`), and a missed
/// block would silently drop a real stylesheet, so the scan tolerates them the
/// way [`extract_body`] tolerates them on `<body>`.
fn extract_all_styles(html: &str) -> String {
    const OPEN: &str = "<style";
    let mut styles = String::new();
    let mut rest = html;
    while let Some(pos) = rest.find(OPEN) {
        let mut tail = &rest[pos + OPEN.len()..];
        // Only a real tag: a bare end or the start of an attribute list.
        if !tail.starts_with(['>', ' ', '\t', '\n', '\r']) {
            rest = tail;
            continue;
        }
        let Some(open_end) = tail.find('>') else {
            break;
        };
        tail = &tail[open_end + 1..];
        let Some(end) = tail.find("</style>") else {
            break;
        };
        styles.push_str(&tail[..end]);
        styles.push('\n');
        rest = &tail[end + "</style>".len()..];
    }
    styles
}

/// Guarantees a root `index.html` after a build that wrote no pages. A pure
/// i18n project owns `index.html` through no single target, so without it the
/// dev server has no route for `/`. An existing landing page is left alone.
fn ensure_landing_page(project: &Project, target: &Path) -> Result<()> {
    let index = target.join(INDEX_FILE);
    let owned_by_target = project
        .targets
        .iter()
        .any(|t| t.html_file_name == INDEX_FILE);
    if owned_by_target || index.is_file() {
        return Ok(());
    }
    write_atomic(&index, &crate::assets::redirect_page(&project.targets))
}

/// Renders the compile failure as an error page at every given target so
/// browser reloads never fall back to stale content.
fn write_error_pages(project: &Project, targets: &[&LanguageTarget], raw_error: &str) {
    for target in targets {
        if let Err(err) = write_error_page(project, raw_error, &target.html_file_name, &target.lang)
        {
            crate::state::log(&format!(
                "[fy-docs] could not write error page {}: {err}",
                target.html_file_name
            ));
        }
    }
}

fn write_error_page(
    project: &Project,
    raw_error: &str,
    file_name: &str,
    lang_hint: &str,
) -> Result<()> {
    let target = &project.target_dir;
    fs::create_dir_all(target)?;

    let ui = crate::assets::ui_text(Some(lang_hint), raw_error);
    let escaped_error = crate::assets::escape(raw_error);
    let body = format!(
        r#"<div class="fy-error">
  <h1>{}</h1>
  <p class="fy-error-lead">{}</p>
  <pre><code>{}</code></pre>
  <p class="fy-error-hint">{}</p>
</div>"#,
        ui.compile_failed, ui.compile_failed_detail, escaped_error, ui.compile_failed_hint
    );

    let page = crate::assets::doc_page(
        &project.name,
        &project.name,
        project.repository.as_deref(),
        &body,
        None,
        &project.targets,
    );

    write_atomic(&target.join(file_name), &page)?;
    write_atomic(&target.join(SKIN_FILE), crate::assets::BASE_CSS)?;
    write_atomic(&target.join(VIEWER_JS_FILE), crate::assets::VIEWER_JS)?;
    // Only seed typst.css when absent: in a partial failure the combined
    // styles of the successful targets are already on disk and must survive.
    let typst_css = target.join(TYPST_CSS_FILE);
    if !typst_css.exists() {
        write_atomic(&typst_css, "")?;
    }
    write_atomic(&target.join(LIVE_JS_FILE), crate::assets::LIVE_JS)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typst_version_banners() {
        assert_eq!(
            typst_banner_version("typst 0.15.1 (abcdef1)"),
            Some((0, 15, 1))
        );
        assert_eq!(typst_banner_version("typst 0.14.0"), Some((0, 14, 0)));
        assert_eq!(typst_banner_version("typst 1.2.3 (x)"), Some((1, 2, 3)));
        // Unknown formats resolve to None and must not block compiles.
        assert_eq!(typst_banner_version(""), None);
        assert_eq!(typst_banner_version("something else entirely"), None);
    }

    #[test]
    fn minimum_version_gate_rejects_old_typst() {
        assert!((0, 13, 9) < MINIMUM_TYPST);
        assert!((0, 14, 0) >= MINIMUM_TYPST);
    }

    #[test]
    fn extract_between_finds_enclosed_text() {
        assert_eq!(
            extract_between("<title>Hello</title>", "<title>", "</title>"),
            Some("Hello".to_owned())
        );
        assert_eq!(extract_between("no tags", "<title>", "</title>"), None);
    }

    #[test]
    fn extract_body_tolerates_tag_attributes() {
        assert_eq!(
            extract_body(r#"<html><body lang="en"><p>hi</p></body></html>"#),
            Some("<p>hi</p>".to_owned())
        );
        assert_eq!(
            extract_body("<body><p>plain</p></body>"),
            Some("<p>plain</p>".to_owned())
        );
        assert_eq!(extract_body("<div>no body</div>"), None);
    }

    #[test]
    fn extract_all_styles_concatenates_blocks() {
        let html = "<style>a{}</style><p>x</p><style>b{}</style>";
        assert_eq!(extract_all_styles(html), "a{}\nb{}\n");
        assert_eq!(extract_all_styles("<style>unclosed"), "");
    }

    #[test]
    fn extract_all_styles_reads_attributed_tags_only() {
        let html = r#"<style media="print">a{}</style><styleset>x</styleset><style>b{}</style>"#;
        assert_eq!(extract_all_styles(html), "a{}\nb{}\n");
    }

    #[test]
    fn formats_panic_payloads() {
        let text: Box<dyn std::any::Any + Send> = Box::new("boom");
        assert_eq!(panic_message(&text), "boom");
        let owned: Box<dyn std::any::Any + Send> = Box::new("boom".to_owned());
        assert_eq!(panic_message(&owned), "boom");
        let odd: Box<dyn std::any::Any + Send> = Box::new(42u8);
        assert_eq!(panic_message(&odd), "unknown panic payload");
    }

    fn test_project(docs_dir: &Path) -> Project {
        Project {
            name: "test".to_owned(),
            version: "0.1.0".to_owned(),
            repository: None,
            targets: Vec::new(),
            docs_dir: docs_dir.to_path_buf(),
            root: docs_dir.to_path_buf(),
            target_dir: docs_dir.join("target"),
            release_dir: docs_dir.join("release"),
            watch_dirs: Vec::new(),
        }
    }

    #[test]
    fn error_page_targets_the_given_file() {
        let temp = std::env::temp_dir().join(format!("fy-docs-errpage-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(temp.join("docs")).unwrap();
        let project = test_project(&temp.join("docs"));

        write_error_page(&project, "boom <b>detail</b>", "index_zh-CN.html", "zh-CN").unwrap();
        let page = fs::read_to_string(temp.join("docs/target/index_zh-CN.html")).unwrap();
        assert!(page.contains("fy-error"));
        assert!(page.contains("boom &lt;b&gt;detail&lt;/b&gt;"));
        // The zh-CN target gets a localized error page.
        assert!(page.contains("<html lang=\"zh-CN\">"));
        // The landing page must stay untouched by error output.
        assert!(!temp.join("docs/target/index.html").is_file());

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn error_page_preserves_combined_typst_styles() {
        let temp = std::env::temp_dir().join(format!("fy-docs-errcss-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(temp.join("docs/target")).unwrap();
        let project = test_project(&temp.join("docs"));

        // A partial failure: the successful targets already wrote their
        // combined styles; the error page must not blank them out.
        fs::write(temp.join("docs/target/typst.css"), ".combined{color:teal}").unwrap();
        write_error_page(&project, "boom", "index_en.html", "en").unwrap();
        assert_eq!(
            fs::read_to_string(temp.join("docs/target/typst.css")).unwrap(),
            ".combined{color:teal}"
        );

        // With no styles on disk yet (everything failed) the error page
        // seeds an empty typst.css so the shell still loads.
        fs::remove_file(temp.join("docs/target/typst.css")).unwrap();
        write_error_page(&project, "boom", "index_en.html", "en").unwrap();
        assert!(temp.join("docs/target/typst.css").is_file());

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn warnings_note_folds_repeated_font_warnings() {
        let stderr = concat!(
            "warning: unknown font family: Noto Serif CJK SC\n",
            "   ┌─ docs/fy-spec/lib.typ:297:10\n",
            "297 │     font: active-fonts.serif,\n",
            "\n",
            "warning: unknown font family: noto serif cjk sc\n",
            "   ┌─ docs/fy-spec/lib.typ:383:51\n",
            "\n",
            "warning: pagebreak was ignored during HTML export\n",
            "   ┌─ docs/fy-spec/lib.typ:430:2\n",
        );
        let note = warnings_note(stderr.as_bytes()).unwrap();
        assert!(note.starts_with("[fy-docs] typst reported warnings:"));
        assert!(
            note.contains("font families unavailable, fallback applied: noto serif cjk sc"),
            "{note}"
        );
        assert_eq!(note.matches("noto serif cjk sc").count(), 1, "{note}");
        // The unrelated warning must survive untouched.
        assert!(
            note.contains("warning: pagebreak was ignored during HTML export"),
            "{note}"
        );
    }

    #[test]
    fn warnings_note_edge_cases() {
        assert!(warnings_note(b"").is_none());
        assert!(warnings_note(b"  \n\t ").is_none());

        // Without blank-line separators the `warning:` prefix still splits blocks.
        let note =
            warnings_note(b"warning: unknown font family: a\nwarning: unknown font family: b\n")
                .unwrap();
        assert!(note.contains("fallback applied: a, b"), "{note}");

        // An unrecognized block is forwarded as written, never dropped.
        let note = warnings_note(b"something unexpected from typst\n").unwrap();
        assert_eq!(
            note,
            "[fy-docs] typst reported warnings:\nsomething unexpected from typst"
        );
    }

    #[test]
    fn select_targets_rejects_an_unknown_language() {
        let temp = std::env::temp_dir().join(format!("fy-docs-select-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let mut project = test_project(&temp);
        let target = |lang: &str| LanguageTarget {
            lang: lang.to_owned(),
            display_name: lang.to_owned(),
            entry: temp.join(format!("{lang}/main.typ")),
            html_file_name: format!("index_{lang}.html"),
            pdf_file_name: format!("test_{lang}.pdf"),
        };
        project.targets = vec![target("zh-CN"), target("en")];

        let err = select_targets(&project, Some("zz"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("`zz`"), "{err}");
        assert!(err.contains("zh-CN") && err.contains("en"), "{err}");

        // Normalized variants of a real language still resolve.
        let selected = select_targets(&project, Some("ZH_CN")).unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].lang, "zh-CN");

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn clean_compile_temporaries_sweeps_leaked_files() {
        let temp = std::env::temp_dir().join(format!("fy-docs-clean-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        for name in ["_temp_zh-CN_4242.html", "index.html", "typst.css"] {
            fs::write(temp.join(name), "x").unwrap();
        }

        clean_compile_temporaries(&temp).unwrap();

        assert!(
            !temp.join("_temp_zh-CN_4242.html").exists(),
            "a killed compile's intermediate must go"
        );
        // Output the current build owns stays put.
        assert!(temp.join("index.html").is_file());
        assert!(temp.join("typst.css").is_file());

        // Sweeping an already clean directory is a no-op.
        clean_compile_temporaries(&temp).unwrap();
        let _ = fs::remove_dir_all(temp);
    }

    fn lang_target(lang: &str) -> LanguageTarget {
        LanguageTarget {
            lang: lang.to_owned(),
            display_name: lang.to_owned(),
            entry: PathBuf::from(format!("docs/{lang}/main.typ")),
            html_file_name: format!("index_{lang}.html"),
            pdf_file_name: format!("test_{lang}.pdf"),
        }
    }

    #[test]
    fn landing_page_survives_a_build_with_no_pages() {
        let temp = std::env::temp_dir().join(format!("fy-docs-landing-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(temp.join("docs/target")).unwrap();
        let docs = temp.join("docs");
        let mut project = test_project(&docs);

        // A pure i18n project: index.html belongs to no single target, so the
        // routing landing page must be written for it.
        project.targets = vec![lang_target("en"), lang_target("zh-CN")];
        ensure_landing_page(&project, &project.target_dir).unwrap();
        let page = fs::read_to_string(docs.join("target/index.html")).unwrap();
        assert!(page.contains("index_en.html"), "{page}");

        // An existing landing page is never clobbered.
        fs::write(docs.join("target/index.html"), "existing").unwrap();
        ensure_landing_page(&project, &project.target_dir).unwrap();
        assert_eq!(
            fs::read_to_string(docs.join("target/index.html")).unwrap(),
            "existing"
        );

        // A project with a default target writes index.html itself: nothing to seed.
        fs::remove_file(docs.join("target/index.html")).unwrap();
        let mut default_project = test_project(&docs);
        default_project.targets = vec![LanguageTarget {
            lang: String::new(),
            display_name: "Default".to_owned(),
            entry: docs.join("main.typ"),
            html_file_name: INDEX_FILE.to_owned(),
            pdf_file_name: "test.pdf".to_owned(),
        }];
        ensure_landing_page(&default_project, &default_project.target_dir).unwrap();
        assert!(!docs.join("target/index.html").exists());

        let _ = fs::remove_dir_all(temp);
    }
}
