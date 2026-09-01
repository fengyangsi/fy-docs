//! The shared template-argument parser: reads named arguments (`version:`,
//! `lang:`) out of an entry `main.typ`'s live code.

use std::path::Path;

/// Falls back to the `version: "..."` argument of the document template.
pub(crate) fn main_typ_version(entry: &Path) -> Option<String> {
    template_argument(entry, "version")
}

/// Reads the `lang: "..."` argument of the document template: the entry's own
/// declaration of its content language.
pub(crate) fn main_typ_lang(entry: &Path) -> Option<String> {
    template_argument(entry, "lang")
}

fn template_argument(entry: &Path, key: &str) -> Option<String> {
    parse_template_argument(&std::fs::read_to_string(entry).ok()?, key)
}

/// Reads a named template argument (`version:`, `lang:`) from live code only.
/// A mention inside a comment must not become the value, an occurrence without
/// a quoted value is skipped rather than ending the search, and a name that
/// merely ends in the key (`sub-lang:`) is a different argument.
pub(crate) fn parse_template_argument(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        let code = match line.find("//") {
            Some(cut) => &line[..cut],
            None => line,
        };
        let mut offset = 0;
        while let Some(at) = code[offset..].find(key) {
            let at = offset + at;
            offset = at + key.len();
            let boundary_before = at == 0 || {
                let previous = code.as_bytes()[at - 1];
                !previous.is_ascii_alphanumeric() && previous != b'_' && previous != b'-'
            };
            let rest = &code[at + key.len()..];
            if !boundary_before || !rest.starts_with(':') {
                continue;
            }
            let rest = &rest[1..];
            let Some(open) = rest.find('"') else {
                continue;
            };
            let from = open + 1;
            let Some(close) = rest[from..].find('"') else {
                continue;
            };
            return Some(rest[from..from + close].to_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_argument_reads_live_code_only() {
        let text = "// version: \"9.9.9\"\n#show: project_book.with(\n  title: \"x\", // version: \"8.8.8\"\n  version: none,\n  version: \"1.2.3\",\n)\n";
        assert_eq!(
            parse_template_argument(text, "version").as_deref(),
            Some("1.2.3")
        );
        assert_eq!(
            parse_template_argument("// version: \"1.0.0\"", "version"),
            None
        );
        assert_eq!(parse_template_argument("no version here", "version"), None);

        // An argument whose name merely ends in the key is a different argument.
        assert_eq!(parse_template_argument("sub-lang: \"de\"", "lang"), None);
        assert_eq!(
            parse_template_argument("lang: \"de\"", "lang").as_deref(),
            Some("de")
        );
        // The comment after a value cannot steal it.
        assert_eq!(
            parse_template_argument(r#"lang: "en", // try "zh-CN""#, "lang").as_deref(),
            Some("en")
        );
    }
}
