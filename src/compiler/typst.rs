//! The process boundary to the `typst` CLI: the version precheck, the HTML
//! and PDF compile invocations, and stderr handling.

use super::extract::{
    ExtractedPage, extract_all_styles, extract_between, extract_body, extract_root_lang,
    language_drift,
};
use super::warnings::format_warnings;
use super::{TEMP_PREFIX, ensure_gitignore, lang_label, panic_message, select_targets};
use crate::project::{LanguageTarget, Project};
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

/// Compiles one language target to HTML in a temp file and extracts the page
/// pieces. The temp file is removed on every path, including failures.
pub(crate) fn compile_html_target(
    project: &Project,
    lang_target: &LanguageTarget,
    target_dir: &Path,
) -> Result<ExtractedPage> {
    let temp_html = target_dir.join(format!(
        "{TEMP_PREFIX}{}_{}.html",
        if lang_target.lang.is_empty() {
            "root"
        } else {
            &lang_target.lang
        },
        std::process::id()
    ));
    let parts = (|| -> Result<ExtractedPage> {
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
        if let Some(note) = language_drift(lang_target, extract_root_lang(&html).as_deref()) {
            crate::term::log(&note);
        }
        let title =
            extract_between(&html, "<title>", "</title>").unwrap_or_else(|| project.name.clone());
        let styles = extract_all_styles(&html);
        let body = extract_body(&html).context("typst HTML export contains no <body>")?;
        Ok(ExtractedPage { title, styles, body })
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

fn run(cmd: &mut Command) -> Result<()> {
    let output = cmd
        .output()
        .context("failed to spawn `typst` (is it on PATH?)")?;
    if output.status.success() {
        if let Some(note) = format_warnings(&output.stderr) {
            crate::term::log(&note);
        }
        return Ok(());
    }
    let stderr = crate::term::strip_verbatim(&String::from_utf8_lossy(&output.stderr));
    let stdout = crate::term::strip_verbatim(&String::from_utf8_lossy(&output.stdout));
    if stderr.trim().is_empty() {
        bail!("{stdout}")
    } else {
        bail!("{stderr}")
    }
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
}
