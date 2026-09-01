//! Language identity: the per-language compile target and the BCP 47
//! toolkit that normalizes, formats, and labels language tags.

use crate::project::template_args::main_typ_lang;
use std::path::PathBuf;

/// A target language document within a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LanguageTarget {
    /// Language identifier, e.g. "zh-CN", "en", "zh-TW", or empty string for default.
    pub(crate) lang: String,
    /// This target's document language as a normalized BCP 47 tag. Resolved
    /// from declarations alone — the language directory, else the entry's
    /// `lang:` argument, else `en` — and used for the page's `<html lang>` and
    /// its toolbar labels.
    pub(crate) content_lang: String,
    /// Display name in language switcher, e.g. "简体中文", "English".
    pub(crate) display_name: String,
    /// Path to entry `main.typ` for this language.
    pub(crate) entry: PathBuf,
    /// Output html filename, e.g. "index.html" or "index_zh-CN.html".
    pub(crate) html_file_name: String,
    /// Release PDF filename.
    pub(crate) pdf_file_name: String,
}

impl LanguageTarget {
    /// Builds a target from its declarations: the language directory name
    /// (empty for the default target), the switcher label, and the entry
    /// path. The package name and resolved version shape both output file
    /// names, so the naming rules live in exactly this one place.
    pub(crate) fn new(
        lang: &str,
        display_name: &str,
        entry: PathBuf,
        pkg_name: &str,
        version: &str,
    ) -> Self {
        let html_file_name = if lang.is_empty() {
            "index.html".to_owned()
        } else {
            format!("index_{lang}.html")
        };
        let pdf_file_name = if lang.is_empty() {
            format!("{pkg_name}_v{version}_specification.pdf")
        } else {
            format!("{pkg_name}_v{version}_{lang}_specification.pdf")
        };
        Self {
            lang: lang.to_owned(),
            content_lang: resolve_content_lang(lang, &entry),
            display_name: display_name.to_owned(),
            entry,
            html_file_name,
            pdf_file_name,
        }
    }
}

/// Normalizes a language tag for comparison: trimmed, lowercase, and with `_`
/// interchangeable with `-`, so `zh_CN` and `ZH-cn` both mean `zh-CN`.
pub(crate) fn normalize_lang(lang: &str) -> String {
    lang.trim().to_lowercase().replace('_', "-")
}

/// Maps language codes to user-friendly native display labels.
pub(crate) fn lang_display_name(lang: &str) -> String {
    match normalize_lang(lang).as_str() {
        "zh" | "zh-cn" | "zh-hans" => "简体中文".to_owned(),
        "zh-tw" | "zh-hk" | "zh-hant" => "繁體中文".to_owned(),
        "en" | "en-us" | "en-gb" => "English".to_owned(),
        "ja" | "ja-jp" => "日本語".to_owned(),
        "de" | "de-de" => "Deutsch".to_owned(),
        "fr" | "fr-fr" => "Français".to_owned(),
        "ru" | "ru-ru" => "Русский".to_owned(),
        "es" | "es-es" => "Español".to_owned(),
        other => format_lang(other),
    }
}

/// Rewrites an unregistered tag into BCP 47 display shape: the base subtag
/// stays lowercase and every following subtag is cased by length. Guessing a
/// name would be worse than showing a correct code.
pub(crate) fn format_lang(tag: &str) -> String {
    let mut subtags = tag.split('-');
    let base = subtags.next().unwrap_or_default().to_owned();
    let rest: Vec<String> = subtags.map(format_subtag).collect();
    if rest.is_empty() {
        base
    } else {
        format!("{base}-{}", rest.join("-"))
    }
}

/// Two-character subtags are regions (`CN`); longer ones are scripts or
/// variants and take title case (`Hans`, `Latn`).
fn format_subtag(subtag: &str) -> String {
    if subtag.chars().count() == 2 {
        return subtag.to_uppercase();
    }
    let mut chars = subtag.chars();
    match chars.next() {
        Some(first) => {
            let capitalized: String = first.to_uppercase().collect();
            capitalized + chars.as_str()
        }
        None => String::new(),
    }
}

/// Resolves a target's content language from declarations: its language
/// directory when it has one, otherwise the `lang:` its entry declares,
/// otherwise `en` — the same default the fy-spec template declares.
fn resolve_content_lang(dir_name: &str, entry: &std::path::Path) -> String {
    let declared = if dir_name.is_empty() {
        main_typ_lang(entry).unwrap_or_default()
    } else {
        dir_name.to_owned()
    };
    let normalized = normalize_lang(&declared);
    if normalized.is_empty() {
        return "en".to_owned();
    }
    format_lang(&normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_lang_folds_case_and_separators() {
        assert_eq!(normalize_lang("ZH_CN"), "zh-cn");
        assert_eq!(normalize_lang(" en "), "en");
        assert_eq!(normalize_lang(""), "");
    }

    #[test]
    fn registered_languages_keep_their_endonyms() {
        for (tag, label) in [
            ("zh-CN", "简体中文"),
            ("zh_CN", "简体中文"),
            ("EN", "English"),
            ("ja-JP", "日本語"),
            ("zh-Hant", "繁體中文"),
        ] {
            assert_eq!(lang_display_name(tag), label, "{tag}");
        }
    }

    #[test]
    fn unregistered_tags_display_a_well_formed_code() {
        // Shape only: fy-docs cannot know what language a bare code means, so
        // it must not invent a name for it either.
        assert_eq!(lang_display_name("pt_BR"), "pt-BR");
        assert_eq!(lang_display_name("fil"), "fil");
        assert_eq!(lang_display_name("zh_hant_tw"), "zh-Hant-TW");
        assert_eq!(lang_display_name("  sr_latn_rs "), "sr-Latn-RS");
    }
}
