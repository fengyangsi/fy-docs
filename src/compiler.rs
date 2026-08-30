//! Invokes the typst CLI and renders the self-contained static page(s) into
//! `docs/target/`, plus the print-edition PDF(s) into `docs/release/`.

use crate::project::{LanguageTarget, Project};
use crate::state::AppState;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub const INDEX_FILE: &str = "index.html";
pub const SKIN_FILE: &str = "fy-docs.css";
pub const TYPST_CSS_FILE: &str = "typst.css";
pub const VIEWER_JS_FILE: &str = "fy-docs.js";
pub const POLL_STUB_FILE: &str = "_poll.js";
pub const BUILD_FILE: &str = "_build";

/// Generates HTML page(s) into `docs/target/`, optionally with PDF(s).
/// Failures become visible error pages; this never returns `Err`.
pub fn generate_into(state: &AppState, with_pdf: bool, lang_filter: Option<&str>) {
    crate::state::log(&format!(
        "[fy-docs] compiling `{}` (v{})...",
        state.project.name, state.project.version
    ));
    let outcome = generate(&state.project, with_pdf, lang_filter);
    match outcome {
        Ok(()) => crate::state::log(" ok"),
        Err(err) => {
            crate::state::log(" FAILED");
            if let Err(write_err) = write_error_page(&state.project, &format!("{err:#}")) {
                crate::state::log(&format!(" could not write error page: {write_err}"));
            }
        }
    }
}

fn generate(project: &Project, with_pdf: bool, lang_filter: Option<&str>) -> Result<()> {
    let target = &project.target_dir;
    fs::create_dir_all(target)?;

    let selected_targets = project.selected_targets(lang_filter);
    if selected_targets.is_empty() {
        bail!(
            "no matching documentation language targets found for filter {:?}",
            lang_filter
        );
    }

    if with_pdf {
        compile_pdf(project, lang_filter)?;
    }

    // Parallel compilation of all language targets for HTML export
    let results: Vec<Result<(&LanguageTarget, String, String, String)>> = std::thread::scope(|s| {
        let mut handles = Vec::new();
        for lang_target in &selected_targets {
            handles.push(s.spawn(move || {
                let temp_html = target.join(format!(
                    "_temp_{}_{}.html",
                    if lang_target.lang.is_empty() {
                        "root"
                    } else {
                        &lang_target.lang
                    },
                    std::process::id()
                ));
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
                .context(format!(
                    "typst HTML export failed for [{}] {}",
                    if lang_target.lang.is_empty() {
                        "default"
                    } else {
                        &lang_target.lang
                    },
                    lang_target.entry.display()
                ))?;

                let html = fs::read_to_string(&temp_html)?;
                let title = extract_between(&html, "<title>", "</title>")
                    .unwrap_or_else(|| project.name.clone());
                let styles = extract_all_styles(&html);
                let body = extract_between(&html, "<body>", "</body>")
                    .context("typst HTML export contains no <body>")?;
                let _ = fs::remove_file(&temp_html);

                Ok((*lang_target, title, styles, body))
            }));
        }
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    let mut combined_styles = String::new();
    let mut rendered_pages: Vec<(&LanguageTarget, String, String)> = Vec::new();

    for res in results {
        let (lang_target, title, styles, body) = res?;
        if combined_styles.is_empty() {
            combined_styles = styles;
        } else if !styles.is_empty() && !combined_styles.contains(&styles) {
            combined_styles.push_str("\n/* additional language styles */\n");
            combined_styles.push_str(&styles);
        }
        rendered_pages.push((lang_target, title, body));
    }

    fs::write(target.join(TYPST_CSS_FILE), combined_styles)?;
    fs::write(target.join(SKIN_FILE), crate::assets::BASE_CSS)?;
    fs::write(target.join(VIEWER_JS_FILE), crate::assets::VIEWER_JS)?;
    fs::write(target.join(POLL_STUB_FILE), crate::assets::POLL_STUB)?;

    let mut has_index = false;
    for (lang_target, title, body) in &rendered_pages {
        let page_html = crate::assets::doc_page(
            title,
            &project.name,
            project.repository.as_deref(),
            body.trim(),
            Some(lang_target),
            &project.targets,
        );

        fs::write(target.join(&lang_target.html_file_name), &page_html)?;
        if lang_target.html_file_name == INDEX_FILE {
            has_index = true;
        }
    }

    // If no index.html was directly generated from a root main.typ:
    if !has_index && !rendered_pages.is_empty() {
        if rendered_pages.len() == 1 {
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
            fs::write(target.join(INDEX_FILE), default_page)?;
        } else {
            // Multi-language: write lightweight (~500B) client-side redirect landing page
            let landing = crate::assets::redirect_page(&project.targets);
            fs::write(target.join(INDEX_FILE), landing)?;
        }
    }

    ensure_gitignore_ignores(project, ["/docs/target/"]);
    Ok(())
}

/// Compiles the print-edition PDF 2.0 specifications into `docs/release/` in parallel.
pub fn compile_pdf(project: &Project, lang_filter: Option<&str>) -> Result<Vec<PathBuf>> {
    fs::create_dir_all(&project.release_dir)?;
    let selected_targets = project.selected_targets(lang_filter);
    if selected_targets.is_empty() {
        bail!(
            "no matching documentation language targets found for filter {:?}",
            lang_filter
        );
    }

    let results: Vec<Result<PathBuf>> = std::thread::scope(|s| {
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
                .context(format!(
                    "typst PDF export failed for [{}] {}",
                    if lang_target.lang.is_empty() {
                        "default"
                    } else {
                        &lang_target.lang
                    },
                    lang_target.entry.display()
                ))?;
                Ok(release_path)
            }));
        }
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    let mut generated_paths = Vec::new();
    for res in results {
        generated_paths.push(res?);
    }

    ensure_gitignore_ignores(project, ["/docs/release/"]);
    Ok(generated_paths)
}

fn run(cmd: &mut Command) -> Result<()> {
    let output = cmd
        .output()
        .context("failed to spawn `typst` (is it on PATH?)")?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stderr.trim().is_empty() {
        bail!("{stdout}")
    } else {
        bail!("{stderr}")
    }
}

fn extract_between(haystack: &str, start: &str, end: &str) -> Option<String> {
    let from = haystack.find(start)? + start.len();
    let to = haystack[from..].find(end)? + from;
    Some(haystack[from..to].to_owned())
}

fn extract_all_styles(html: &str) -> String {
    let mut styles = String::new();
    let mut rest = html;
    while let Some(pos) = rest.find("<style>") {
        let after = &rest[pos + "<style>".len()..];
        let Some(end) = after.find("</style>") else {
            break;
        };
        styles.push_str(&after[..end]);
        styles.push('\n');
        rest = &after[end + "</style>".len()..];
    }
    styles
}

/// Makes sure the generated directories stay ignored; creates the .gitignore
/// entry when missing so `git status` stays clean without manual setup.
fn ensure_gitignore_ignores(project: &Project, entries: impl IntoIterator<Item = &'static str>) {
    let gitignore = project
        .docs_dir
        .parent()
        .unwrap_or(&project.docs_dir)
        .join(".gitignore");
    let mut content = fs::read_to_string(&gitignore).unwrap_or_default();
    let mut changed = false;
    for entry in entries {
        if !content.lines().any(|line| line.trim() == entry) {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(entry);
            content.push('\n');
            changed = true;
        }
    }
    if changed {
        let _ = fs::write(&gitignore, content);
    }
}

fn write_error_page(project: &Project, raw_error: &str) -> Result<()> {
    let target = &project.target_dir;
    fs::create_dir_all(target)?;

    let ui = crate::assets::ui_text(&project.name, raw_error);
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

    fs::write(target.join(INDEX_FILE), page)?;
    fs::write(target.join(SKIN_FILE), crate::assets::BASE_CSS)?;
    fs::write(target.join(VIEWER_JS_FILE), crate::assets::VIEWER_JS)?;
    fs::write(target.join(TYPST_CSS_FILE), "")?;
    fs::write(target.join(POLL_STUB_FILE), crate::assets::POLL_STUB)?;
    Ok(())
}
