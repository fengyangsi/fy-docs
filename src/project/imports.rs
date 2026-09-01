//! The typst compile root: the absolute-import scan and the ancestor walk
//! that finds the directory satisfying every import.

use super::GENERATED_DOC_DIRS;
use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

/// Walks up from `start` looking for the closest ancestor under which every
/// absolute import target exists. No imports → the project directory itself.
pub(crate) fn detect_root(start: &Path, imports: &[String]) -> Result<PathBuf> {
    if imports.is_empty() {
        return Ok(start.to_path_buf());
    }
    let mut candidate = Some(start.to_path_buf());
    while let Some(dir) = candidate {
        let satisfied = imports.iter().all(|import| dir.join(import).is_file());
        if satisfied {
            return Ok(dir);
        }
        candidate = dir.parent().map(Path::to_path_buf);
    }
    bail!(
        "absolute imports not satisfied by any ancestor of {} (imports: {})",
        start.display(),
        imports.join(", ")
    )
}

/// Collects absolute import paths (`#import "/pkg/lib.typ"`) from every
/// `.typ` file under `docs/`, as root-relative (leading `/` stripped) paths.
pub(crate) fn absolute_imports(docs_dir: &Path) -> Vec<String> {
    let mut imports = Vec::new();
    collect_imports(docs_dir, &mut imports);
    imports.sort();
    imports.dedup();
    imports
}

fn collect_imports(dir: &Path, imports: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if GENERATED_DOC_DIRS.contains(&name.as_str()) {
                continue; // generated output, never imported
            }
            collect_imports(&path, imports);
        } else if path.extension().is_some_and(|ext| ext == "typ")
            && let Ok(text) = std::fs::read_to_string(&path)
        {
            for line in text.lines() {
                if line.contains("import") {
                    imports.extend(quoted_absolute_typs(line));
                }
            }
        }
    }
}

/// Extracts the root-relative `.typ` paths quoted on a single source line.
/// Both quote styles are recognized and everything after `//` is a comment, so
/// a commented-out `#import` cannot drag root detection to the wrong ancestor.
fn quoted_absolute_typs(line: &str) -> Vec<String> {
    let chars: Vec<char> = line.chars().collect();
    let mut found = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '/' if chars.get(i + 1) == Some(&'/') => break,
            quote @ ('"' | '\'') => {
                let open = i + 1;
                // An unterminated literal makes the rest of the line
                // untrustworthy; typst would reject the file anyway.
                let Some(len) = chars[open..].iter().position(|c| *c == quote) else {
                    break;
                };
                let text: String = chars[open..open + len].iter().collect();
                if let Some(rel) = text.strip_prefix('/')
                    && rel.ends_with(".typ")
                {
                    found.push(rel.to_owned());
                }
                i = open + len + 1;
            }
            _ => i += 1,
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_scan_handles_quotes_and_comments() {
        assert_eq!(
            quoted_absolute_typs(r#"#import "/pkg/lib.typ": *"#),
            vec!["pkg/lib.typ".to_owned()]
        );
        assert_eq!(
            quoted_absolute_typs(r#"#import '/pkg/lib.typ': *"#),
            vec!["pkg/lib.typ".to_owned()]
        );
        assert!(quoted_absolute_typs(r#"// #import "/stale/lib.typ": *"#).is_empty());
        assert_eq!(
            quoted_absolute_typs(r#"#import "/a.typ": x // "/b.typ""#),
            vec!["a.typ".to_owned()]
        );
        assert!(quoted_absolute_typs(r#"#import "relative/lib.typ": *"#).is_empty());
    }
}
