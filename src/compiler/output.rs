//! Artifact writing: atomic writes, the root `index.html` decision, error
//! pages, temporary sweeping, and multi-language style merging.

use super::{INDEX_FILE, LIVE_JS_FILE, STYLE_FILE, TYPST_CSS_FILE, VIEWER_JS_FILE};
use crate::project::{LanguageTarget, Project};
use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::Path;

/// Sweeps the `_temp_*.html` intermediates a compile killed mid-way left in
/// the output directory: a target that finishes removes its own intermediate
/// on success and on failure alike, so anything still there is orphaned.
pub(crate) fn clean_compile_temporaries(target: &Path) -> Result<()> {
    for entry in fs::read_dir(target)?.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(super::TEMP_PREFIX) && name.ends_with(".html") {
            let _ = fs::remove_file(entry.path());
        }
    }
    Ok(())
}

/// Writes a file atomically: the content lands in a sibling temp file that
/// is renamed over the destination, so a concurrent HTTP read from the dev
/// server never observes a half-written page.
pub(crate) fn write_atomic(path: &Path, contents: &str) -> Result<()> {
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

/// Extends the combined stylesheet with one export's CSS: the first non-empty
/// export seeds it, an export whose styles are already contained is a no-op,
/// and a genuinely different one appends under a marker comment. Containment
/// decides equality, so two languages exporting the same sheet keep one copy.
pub(crate) fn merge_styles(combined: &mut String, styles: &str) {
    if styles.is_empty() {
        return;
    }
    if combined.is_empty() {
        combined.push_str(styles);
    } else if !combined.contains(styles) {
        combined.push_str("\n/* additional language styles */\n");
        combined.push_str(styles);
    }
}

/// Guarantees a root `index.html` after a build that wrote no pages. A pure
/// i18n project owns `index.html` through no single target, so without it the
/// dev server has no route for `/`. An existing landing page is left alone.
pub(crate) fn ensure_landing_page(project: &Project, target: &Path) -> Result<()> {
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
pub(crate) fn write_error_pages(project: &Project, targets: &[&LanguageTarget], raw_error: &str) {
    for target in targets {
        if let Err(err) = write_error_page(project, raw_error, target) {
            crate::term::log(&format!(
                "[fy-docs] could not write error page {}: {err}",
                target.html_file_name
            ));
        }
    }
}

pub(crate) fn write_error_page(
    project: &Project,
    raw_error: &str,
    target: &LanguageTarget,
) -> Result<()> {
    let dir = &project.target_dir;
    fs::create_dir_all(dir)?;

    let ui = crate::assets::ui_text(&target.content_lang);
    let escaped_error = crate::assets::escape(raw_error);
    let body = format!(
        r#"<div class="fy-error">
  <h1>{}</h1>
  <p>{}</p>
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
        target,
        &project.targets,
    );

    write_atomic(&dir.join(&target.html_file_name), &page)?;
    write_atomic(&dir.join(STYLE_FILE), crate::assets::BASE_CSS)?;
    write_atomic(&dir.join(VIEWER_JS_FILE), crate::assets::VIEWER_JS)?;
    // Only seed typst.css when absent: in a partial failure the combined
    // styles of the successful targets are already on disk and must survive.
    let typst_css = dir.join(TYPST_CSS_FILE);
    if !typst_css.exists() {
        write_atomic(&typst_css, "")?;
    }
    write_atomic(&dir.join(LIVE_JS_FILE), crate::assets::LIVE_JS)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Project;
    use std::path::PathBuf;

    fn lang_target(lang: &str) -> LanguageTarget {
        LanguageTarget {
            lang: lang.to_owned(),
            content_lang: lang.to_owned(),
            display_name: lang.to_owned(),
            entry: PathBuf::from(format!("docs/{lang}/main.typ")),
            html_file_name: format!("index_{lang}.html"),
            pdf_file_name: format!("test_{lang}.pdf"),
        }
    }

    #[test]
    fn error_page_targets_the_given_file() {
        let temp = tempfile::tempdir().unwrap();
        let docs = temp.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        let project = Project::for_tests(docs);

        write_error_page(&project, "boom <b>detail</b>", &lang_target("zh-CN")).unwrap();
        let page = fs::read_to_string(project.target_dir.join("index_zh-CN.html")).unwrap();
        assert!(page.contains("fy-error"));
        assert!(page.contains("boom &lt;b&gt;detail&lt;/b&gt;"));
        // The zh-CN target gets a localized error page.
        assert!(page.contains("<html lang=\"zh-CN\">"));
        // The landing page must stay untouched by error output.
        assert!(!project.target_dir.join("index.html").is_file());
    }

    #[test]
    fn error_page_preserves_combined_typst_styles() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("docs/target")).unwrap();
        let project = Project::for_tests(temp.path().join("docs"));

        // A partial failure: the successful targets already wrote their
        // combined styles; the error page must not blank them out.
        fs::write(project.target_dir.join("typst.css"), ".combined{color:teal}").unwrap();
        write_error_page(&project, "boom", &lang_target("en")).unwrap();
        assert_eq!(
            fs::read_to_string(project.target_dir.join("typst.css")).unwrap(),
            ".combined{color:teal}"
        );

        // With no styles on disk yet (everything failed) the error page
        // seeds an empty typst.css so the shell still loads.
        fs::remove_file(project.target_dir.join("typst.css")).unwrap();
        write_error_page(&project, "boom", &lang_target("en")).unwrap();
        assert!(project.target_dir.join("typst.css").is_file());
    }

    #[test]
    fn clean_compile_temporaries_sweeps_leaked_files() {
        let temp = tempfile::tempdir().unwrap();
        for name in ["_temp_zh-CN_4242.html", "index.html", "typst.css"] {
            fs::write(temp.path().join(name), "x").unwrap();
        }

        clean_compile_temporaries(temp.path()).unwrap();

        assert!(
            !temp.path().join("_temp_zh-CN_4242.html").exists(),
            "a killed compile's intermediate must go"
        );
        // Output the current build owns stays put.
        assert!(temp.path().join("index.html").is_file());
        assert!(temp.path().join("typst.css").is_file());

        // Sweeping an already clean directory is a no-op.
        clean_compile_temporaries(temp.path()).unwrap();
    }

    #[test]
    fn merge_styles_seeds_deduplicates_and_appends() {
        let mut combined = String::new();
        // An empty export never seeds the sheet.
        merge_styles(&mut combined, "");
        assert_eq!(combined, "");

        // The first non-empty export seeds the sheet verbatim.
        merge_styles(&mut combined, "a{}");
        assert_eq!(combined, "a{}");

        // An identical later export (a repeated language) is a no-op.
        merge_styles(&mut combined, "a{}");
        assert_eq!(combined, "a{}");

        // A genuinely different export appends under the marker comment.
        merge_styles(&mut combined, "b{}");
        assert_eq!(combined, "a{}\n/* additional language styles */\nb{}");
    }

    #[test]
    fn landing_page_survives_a_build_with_no_pages() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("docs/target")).unwrap();
        let docs = temp.path().join("docs");
        let mut project = Project::for_tests(docs.clone());

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
        let mut default_project = Project::for_tests(docs.clone());
        default_project.targets = vec![LanguageTarget {
            lang: String::new(),
            content_lang: "en".to_owned(),
            display_name: "Default".to_owned(),
            entry: docs.join("main.typ"),
            html_file_name: INDEX_FILE.to_owned(),
            pdf_file_name: "test.pdf".to_owned(),
        }];
        ensure_landing_page(&default_project, &default_project.target_dir).unwrap();
        assert!(!docs.join("target/index.html").exists());
    }
}
