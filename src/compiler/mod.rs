//! Invokes the typst CLI and renders the self-contained static page(s) into
//! `docs/target/`, plus the print-edition PDF(s) into `docs/release/`.

mod extract;
mod output;
mod typst;
mod warnings;

pub(crate) use typst::{compile_pdf, precheck};

use crate::project::{LanguageTarget, Project};
use crate::state::AppState;
use anyhow::{Result, anyhow, bail};
use std::fs;

pub(crate) const INDEX_FILE: &str = "index.html";
pub(crate) const STYLE_FILE: &str = "fy-docs.css";
pub(crate) const TYPST_CSS_FILE: &str = "typst.css";
pub(crate) const VIEWER_JS_FILE: &str = "fy-docs.js";
pub(crate) const LIVE_JS_FILE: &str = "live.js";

/// Prefix of the throwaway HTML file each language target compiles through.
pub(crate) const TEMP_PREFIX: &str = "_temp_";

/// Runs one full generation of the captured options: HTML page(s) into
/// `docs/target/`, optionally with PDF(s). Failures become visible error
/// pages at the affected targets so browser reloads never fall back to stale
/// content; the `Err` return lets batch commands exit non-zero while the dev
/// server keeps running.
pub(crate) fn generate(state: &AppState) -> Result<()> {
    crate::term::log(&format!(
        "[fy-docs] compiling `{}` (v{})...",
        state.project.name, state.project.version
    ));
    let result = generate_pages(
        &state.project,
        state.generate.with_pdf,
        state.generate.lang_filter.as_deref(),
    );
    match &result {
        Ok(()) => crate::term::log(" ok"),
        Err(err) => {
            crate::term::log(" FAILED");
            crate::term::log(&format!("[fy-docs] {err:#}"));
        }
    }
    result
}

fn generate_pages(project: &Project, with_pdf: bool, lang_filter: Option<&str>) -> Result<()> {
    let target = &project.target_dir;
    fs::create_dir_all(target)?;
    output::clean_compile_temporaries(target)?;

    let selected = select_targets(project, lang_filter)?;

    if with_pdf && let Err(err) = typst::compile_pdf(project, lang_filter) {
        output::write_error_pages(project, &selected, &format!("{err:#}"));
        output::ensure_landing_page(project, target)?;
        return Err(err);
    }

    // Parallel compilation of all language targets for HTML export. Every
    // thread reports its target even on failure, so only the affected pages
    // degrade to error output while successful targets keep fresh pages.
    let (results, panic_note) = std::thread::scope(|s| {
        let mut handles = Vec::new();
        for lang_target in &selected {
            handles.push(s.spawn(move || {
                (
                    lang_target,
                    typst::compile_html_target(project, lang_target, target),
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
        output::write_error_pages(project, &selected, &note);
        bail!("compile thread panicked: {note}");
    }

    let mut combined_styles = String::new();
    let mut rendered_pages: Vec<(&LanguageTarget, extract::ExtractedPage)> = Vec::new();
    let mut failures: Vec<(&LanguageTarget, String)> = Vec::new();

    for (lang_target, page) in results {
        match page {
            Ok(extracted) => {
                output::merge_styles(&mut combined_styles, &extracted.styles);
                rendered_pages.push((lang_target, extracted));
            }
            Err(err) => failures.push((lang_target, format!("{err:#}"))),
        }
    }

    output::write_atomic(&target.join(TYPST_CSS_FILE), &combined_styles)?;
    output::write_atomic(&target.join(STYLE_FILE), crate::assets::BASE_CSS)?;
    output::write_atomic(&target.join(VIEWER_JS_FILE), crate::assets::VIEWER_JS)?;
    output::write_atomic(&target.join(LIVE_JS_FILE), crate::assets::LIVE_JS)?;

    for (lang_target, page) in &rendered_pages {
        let page_html = crate::assets::doc_page(
            &page.title,
            &project.name,
            project.repository.as_deref(),
            page.body.trim(),
            lang_target,
            &project.targets,
        );
        output::write_atomic(&target.join(&lang_target.html_file_name), &page_html)?;
    }

    for (lang_target, err) in &failures {
        if let Err(write_err) = output::write_error_page(project, err, lang_target) {
            crate::term::log(&format!(
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
        .any(|(t, _)| t.html_file_name == INDEX_FILE)
        || failures.iter().any(|(t, _)| t.html_file_name == INDEX_FILE);
    if !has_index {
        // The project is single-language only when *all* of its targets
        // collapse to one; a multi-language project with one surviving page
        // must still get the landing page so language routing survives.
        if project.targets.len() == 1 && rendered_pages.len() == 1 {
            // Single-language subfolder: copy page directly to index.html
            let (first_target, first_page) = &rendered_pages[0];
            let default_page = crate::assets::doc_page(
                &first_page.title,
                &project.name,
                project.repository.as_deref(),
                first_page.body.trim(),
                first_target,
                &project.targets,
            );
            output::write_atomic(&target.join(INDEX_FILE), &default_page)?;
        } else {
            // Multi-language: write lightweight (~500B) client-side redirect landing page
            let landing = crate::assets::redirect_page(&project.targets);
            output::write_atomic(&target.join(INDEX_FILE), &landing)?;
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

/// The display label of a target's language: its tag, or `default` for a
/// root `main.typ` target.
fn lang_label(target: &LanguageTarget) -> &str {
    if target.lang.is_empty() {
        "default"
    } else {
        &target.lang
    }
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

/// Makes sure the generated directories stay ignored (see
/// [`crate::project::ensure_gitignore`]).
fn ensure_gitignore(project: &Project, entries: &[&str]) {
    let root = project.docs_dir.parent().unwrap_or(&project.docs_dir);
    crate::project::ensure_gitignore(root, entries);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_panic_payloads() {
        let text: Box<dyn std::any::Any + Send> = Box::new("boom");
        assert_eq!(panic_message(&text), "boom");
        let owned: Box<dyn std::any::Any + Send> = Box::new("boom".to_owned());
        assert_eq!(panic_message(&owned), "boom");
        let odd: Box<dyn std::any::Any + Send> = Box::new(42u8);
        assert_eq!(panic_message(&odd), "unknown panic payload");
    }

    #[test]
    fn select_targets_rejects_an_unknown_language() {
        let temp = tempfile::tempdir().unwrap();
        let mut project = Project::for_tests(temp.path().to_path_buf());
        let selected_target = |lang: &str| LanguageTarget {
            lang: lang.to_owned(),
            content_lang: lang.to_owned(),
            display_name: lang.to_owned(),
            entry: temp.path().join(format!("{lang}/main.typ")),
            html_file_name: format!("index_{lang}.html"),
            pdf_file_name: format!("test_{lang}.pdf"),
        };
        project.targets = vec![selected_target("zh-CN"), selected_target("en")];

        let err = select_targets(&project, Some("zz"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("`zz`"), "{err}");
        assert!(err.contains("zh-CN") && err.contains("en"), "{err}");

        // Normalized variants of a real language still resolve.
        let selected = select_targets(&project, Some("ZH_CN")).unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].lang, "zh-CN");
    }
}
