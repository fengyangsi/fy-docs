//! Tolerant decomposition of a typst HTML export: the pieces the page shell
//! needs plus the document's own language. Every extractor scans for the
//! shapes a real export produces instead of requiring a strict parse, so a
//! typst export format drift cannot break page assembly.

use crate::project::{LanguageTarget, normalize_lang};

/// Page pieces extracted from one typst HTML export.
pub(crate) struct ExtractedPage {
    pub(crate) title: String,
    pub(crate) styles: String,
    pub(crate) body: String,
}

/// Reads the `lang` attribute of the root `<html>` start tag of a typst export.
/// Only that tag is scanned and an attribute name must start on a whitespace
/// boundary, so a `lang` deeper in the body and `xml:lang` are both ignored.
pub(crate) fn extract_root_lang(html: &str) -> Option<String> {
    let tag_start = html.find("<html")? + "<html".len();
    let attributes = &html[tag_start..tag_start + html[tag_start..].find('>')?];
    let mut offset = 0;
    while let Some(found) = attributes[offset..].find("lang") {
        let at = offset + found;
        offset = at + "lang".len();
        if !attributes[..at].ends_with(char::is_whitespace) {
            continue;
        }
        let Some(value) = attributes[at + "lang".len()..].strip_prefix('=') else {
            continue;
        };
        let Some(value) = value.trim_start().strip_prefix('"') else {
            continue;
        };
        let Some(end) = value.find('"') else {
            continue;
        };
        return Some(value[..end].to_owned());
    }
    None
}

/// Describes a target whose exported language disagrees with the one fy-docs
/// resolved from declarations. The two share a single root — the `lang:` the
/// template sets and the directory name — so a divergence means one of them is
/// stale. Case and separator variants name the same tag and stay quiet, and an
/// export without a root language offers nothing to compare.
pub(crate) fn language_drift(target: &LanguageTarget, exported: Option<&str>) -> Option<String> {
    let exported = exported?;
    if normalize_lang(exported) == normalize_lang(&target.content_lang) {
        return None;
    }
    Some(format!(
        "[fy-docs] language mismatch for [{}]: typst typesets `{exported}` but fy-docs reports \
         `{}` — the entry's `lang:` and its language directory disagree: {}",
        super::lang_label(target),
        target.content_lang,
        target.entry.display()
    ))
}

pub(crate) fn extract_between(haystack: &str, start: &str, end: &str) -> Option<String> {
    let from = haystack.find(start)? + start.len();
    let to = haystack[from..].find(end)? + from;
    Some(haystack[from..to].to_owned())
}

/// Extracts the `<body>` inner HTML, tolerating attributes on the opening tag
/// (`<body lang="en">`) so a typst export format drift cannot break page
/// assembly.
pub(crate) fn extract_body(html: &str) -> Option<String> {
    let tag = html.find("<body")?;
    let from = html[tag..].find('>')? + tag + 1;
    let to = html[from..].find("</body>")? + from;
    Some(html[from..to].to_owned())
}

/// Concatenates every `<style>` block's CSS from a typst HTML export. The
/// opening tag may carry attributes (`<style media="print">`), and a missed
/// block would silently drop a real stylesheet, so the scan tolerates them the
/// way [`extract_body`] tolerates them on `<body>`.
pub(crate) fn extract_all_styles(html: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn root_lang_comes_from_the_opening_tag_only() {
        assert_eq!(
            extract_root_lang(
                r#"<!DOCTYPE html><html lang="zh-CN"><head><title>T</title></head><body><p>文</p></body></html>"#
            )
            .as_deref(),
            Some("zh-CN")
        );
        // A lang attribute in the body is never the document language.
        assert_eq!(
            extract_root_lang(r#"<html lang="en"><body><p lang="zh">x</p></body></html>"#)
                .as_deref(),
            Some("en")
        );
        // `xml:lang` ends in the same name but is a different attribute.
        assert_eq!(
            extract_root_lang(r#"<html xml:lang="de" lang="fr">"#).as_deref(),
            Some("fr")
        );
        assert_eq!(extract_root_lang("<html><body>x</body></html>"), None);
        assert_eq!(extract_root_lang("<div>no root tag</div>"), None);
    }

    fn lang_target(lang: &str) -> LanguageTarget {
        LanguageTarget {
            lang: lang.to_owned(),
            content_lang: lang.to_owned(),
            display_name: lang.to_owned(),
            entry: std::path::PathBuf::from(format!("docs/{lang}/main.typ")),
            html_file_name: format!("index_{lang}.html"),
            pdf_file_name: format!("test_{lang}.pdf"),
        }
    }

    #[test]
    fn drift_is_reported_only_for_a_real_divergence() {
        let zh = lang_target("zh-CN");
        let note = language_drift(&zh, Some("en")).expect("a divergence must warn");
        assert!(
            note.contains("typst typesets `en`") && note.contains("reports `zh-CN`"),
            "{note}"
        );

        // Separator and case variants name the same tag, so they stay quiet.
        assert!(language_drift(&zh, Some("ZH_CN")).is_none());
        assert!(language_drift(&zh, Some("zh-CN")).is_none());
        // An export without a root language offers nothing to compare.
        assert!(language_drift(&zh, None).is_none());
    }
}
