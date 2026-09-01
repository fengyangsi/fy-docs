//! Shapes the stderr a *successful* typst compile wrote into the lines
//! fy-docs forwards.

use crate::term::strip_verbatim;

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
pub(crate) fn format_warnings(stderr: &[u8]) -> Option<String> {
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
    Some(strip_verbatim(&note))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_warnings_folds_repeated_font_warnings() {
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
        let note = format_warnings(stderr.as_bytes()).unwrap();
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
    fn format_warnings_edge_cases() {
        assert!(format_warnings(b"").is_none());
        assert!(format_warnings(b"  \n\t ").is_none());

        // Without blank-line separators the `warning:` prefix still splits blocks.
        let note =
            format_warnings(b"warning: unknown font family: a\nwarning: unknown font family: b\n")
                .unwrap();
        assert!(note.contains("fallback applied: a, b"), "{note}");

        // An unrecognized block is forwarded as written, never dropped.
        let note = format_warnings(b"something unexpected from typst\n").unwrap();
        assert_eq!(
            note,
            "[fy-docs] typst reported warnings:\nsomething unexpected from typst"
        );
    }
}
