//! Invokes the typst CLI and renders the self-contained static page into
//! `docs/target/`, plus the print-edition PDF into `docs/release/`.

use crate::project::Project;
use crate::state::AppState;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const INDEX_FILE: &str = "index.html";
pub const SKIN_FILE: &str = "fy-docs.css";
pub const TYPST_CSS_FILE: &str = "typst.css";
pub const VIEWER_JS_FILE: &str = "fy-docs.js";
pub const POLL_STUB_FILE: &str = "_poll.js";
pub const BUILD_FILE: &str = "_build";

/// Generates the HTML page into `docs/target/`, optionally with the PDF.
/// Failures become the visible error page; this never returns `Err`.
pub fn generate_into(state: &AppState, with_pdf: bool) {
    crate::state::log(&format!(
        "[fy-docs] compiling `{}` (v{})...",
        state.project.name, state.project.version
    ));
    let outcome = generate(&state.project, with_pdf);
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

fn generate(project: &Project, with_pdf: bool) -> Result<()> {
    let target = &project.target_dir;
    fs::create_dir_all(target)?;

    if with_pdf {
        compile_pdf(project)?;
    }

    let full_html = target.join("full.html");
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
        .arg(&project.entry)
        .arg(&full_html))
    .context("typst HTML export failed")?;

    let html = fs::read_to_string(&full_html)?;
    let title =
        extract_between(&html, "<title>", "</title>").unwrap_or_else(|| project.name.clone());
    let styles = extract_all_styles(&html);
    let body = extract_between(&html, "<body>", "</body>")
        .context("typst HTML export contains no <body>")?;
    let _ = fs::remove_file(&full_html);

    fs::write(target.join(TYPST_CSS_FILE), styles)?;
    fs::write(target.join(SKIN_FILE), crate::assets::BASE_CSS)?;
    fs::write(target.join(VIEWER_JS_FILE), crate::assets::VIEWER_JS)?;

    fs::write(
        target.join(INDEX_FILE),
        crate::assets::doc_page(
            &title,
            &project.name,
            project.repository.as_deref(),
            body.trim(),
        ),
    )?;

    fs::write(target.join(POLL_STUB_FILE), crate::assets::POLL_STUB)?;
    ensure_gitignore_ignores(project, ["/docs/target/"]);
    Ok(())
}

/// Compiles the print-edition PDF into `docs/release/`.
pub fn compile_pdf(project: &Project) -> Result<PathBuf> {
    fs::create_dir_all(&project.release_dir)?;

    let pdf_file = project.pdf_file_name();
    run(Command::new("typst")
        .args(["compile", "--root"])
        .arg(&project.root)
        .arg(&project.entry)
        .arg(project.release_dir.join(&pdf_file)))
    .context("typst PDF export failed")?;

    let release_path = project.release_dir.join(&pdf_file);
    ensure_gitignore_ignores(project, ["/docs/release/"]);
    Ok(release_path)
}

fn run(cmd: &mut Command) -> Result<()> {
    let output = cmd
        .output()
        .context("failed to spawn `typst` (is it on PATH?)")?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut tail: Vec<&str> = stderr.lines().rev().take(14).collect();
    tail.reverse();
    bail!(tail.join("\n"))
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
        .unwrap_or(Path::new("."))
        .join(".gitignore");
    let mut content = fs::read_to_string(&gitignore).unwrap_or_default();
    let mut changed = false;
    for entry in entries {
        let already = content.lines().any(|line| line.trim() == entry);
        if !already {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(entry);
            content.push('\n');
            changed = true;
        }
    }
    if changed {
        if let Err(err) = fs::write(&gitignore, content) {
            crate::state::log(&format!(
                "[fy-docs] could not update {}: {err}",
                gitignore.display()
            ));
        } else {
            crate::state::log(&format!(
                "[fy-docs] added ignore entries to {}",
                gitignore.display()
            ));
        }
    }
}

fn write_error_page(project: &Project, error: &str) -> Result<()> {
    let ui = crate::assets::ui_text(&project.name, error);
    let target = &project.target_dir;
    fs::create_dir_all(target)?;
    fs::write(target.join(SKIN_FILE), crate::assets::BASE_CSS)?;
    fs::write(target.join(VIEWER_JS_FILE), crate::assets::VIEWER_JS)?;
    let escaped = crate::assets::escape(error);
    let body = format!(
        "<div class=\"fy-error\"><h1>{}</h1>\
         <p>{}</p><pre>{escaped}</pre>\
         <p class=\"fy-error-hint\">{}</p></div>",
        ui.compile_failed, ui.compile_failed_detail, ui.compile_failed_hint,
    );
    fs::write(
        target.join(INDEX_FILE),
        crate::assets::doc_page(
            ui.compile_failed,
            &project.name,
            project.repository.as_deref(),
            &body,
        ),
    )?;
    fs::write(target.join(POLL_STUB_FILE), crate::assets::POLL_STUB)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_title_body_and_styles() {
        let html = "<!DOCTYPE html><html><head><title>T</title><style>a{}</style>\
                    <style>b{}</style></head><body><nav></nav><h1>H</h1></body></html>";
        assert_eq!(
            extract_between(html, "<title>", "</title>"),
            Some("T".to_owned())
        );
        assert_eq!(extract_all_styles(html), "a{}\nb{}\n");
        let body = extract_between(html, "<body>", "</body>").unwrap();
        assert!(body.contains("<h1>H</h1>"));
    }

    #[test]
    fn missing_markers_yield_none() {
        assert_eq!(extract_between("<p>x</p>", "<title>", "</title>"), None);
        assert_eq!(extract_all_styles("<p>x</p>"), "");
    }

    #[test]
    fn escapes_error_markup() {
        assert_eq!(
            crate::assets::escape("<unknown> & <vars>"),
            "&lt;unknown&gt; &amp; &lt;vars&gt;"
        );
    }
}
