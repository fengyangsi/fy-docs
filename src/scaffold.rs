//! Scaffolds a new documentation project: `cargo fy-docs init` creates a
//! `docs/` directory with a starter `main.typ` and the bundled fy-spec
//! template, ready for `cargo fy-docs` to build and preview.

use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;

/// The fy-spec template library, embedded at compile time.
const TEMPLATE_LIB: &str = include_str!("../docs/fy-spec/lib.typ");

/// Starter `main.typ` with `{{NAME}}` and `{{VERSION}}` placeholders.
const STARTER_MAIN: &str = r#"#import "fy-spec/lib.typ": *

#show: project_book.with(
  title: "{{NAME}} Specification",
  subtitle: none,
  version: "{{VERSION}}",
  author: "{{AUTHOR}}",
  date: datetime.today().display("[year]-[month]-[day]"),
  lang: "en", // change to "zh" or "zh-CN" for Chinese documents
)

= Overview

This document was initialized by `cargo fy-docs init`. Add your module specifications in `docs/modules/` and `#include` them here.

// #include "modules/example.typ"
"#;

/// Scaffolds `docs/` inside `cwd`.
///
/// Refuses to overwrite an existing `docs/main.typ` — the user must remove it
/// explicitly before re-initializing.
pub fn init(cwd: &Path) -> Result<()> {
    let docs_dir = cwd.join("docs");
    let entry = docs_dir.join("main.typ");

    if entry.is_file() {
        bail!(
            "`{}` already exists — remove it first if you want to re-initialize",
            crate::state::display_path(&entry)
        );
    }

    let (name, version, author) = read_cargo_meta(cwd);

    // Create directory structure.
    fs::create_dir_all(docs_dir.join("modules")).context("could not create docs/modules/")?;
    fs::create_dir_all(docs_dir.join("fy-spec")).context("could not create docs/fy-spec/")?;

    // Write the fy-spec template.
    fs::write(docs_dir.join("fy-spec").join("lib.typ"), TEMPLATE_LIB)
        .context("could not write docs/fy-spec/lib.typ")?;

    // Write the starter main.typ.
    let main_content = STARTER_MAIN
        .replace("{{NAME}}", &name)
        .replace("{{VERSION}}", &version)
        .replace("{{AUTHOR}}", &author);
    fs::write(&entry, main_content).context("could not write docs/main.typ")?;

    // Ensure .gitignore has the generated directories.
    ensure_gitignore(cwd, &["/docs/target/", "/docs/release/"]);

    crate::state::log(&format!(
        "[fy-docs] initialized {}",
        crate::state::display_path(&docs_dir)
    ));
    crate::state::log("[fy-docs] created docs/main.typ");
    crate::state::log("[fy-docs] created docs/fy-spec/lib.typ");
    crate::state::log("[fy-docs] created docs/modules/");
    crate::state::log("[fy-docs] → run `cargo fy-docs` to preview");
    Ok(())
}

/// Reads `[package] name`, `version` and the first `authors` entry from
/// `Cargo.toml`, falling back to the directory name, `"0.1.0"` and `"TODO"`.
fn read_cargo_meta(cwd: &Path) -> (String, String, String) {
    let fallback_name = cwd
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "project".to_owned());

    let Ok(text) = fs::read_to_string(cwd.join("Cargo.toml")) else {
        return (fallback_name, "0.1.0".to_owned(), "TODO".to_owned());
    };
    let Ok(manifest) = text.parse::<toml::Table>() else {
        return (fallback_name, "0.1.0".to_owned(), "TODO".to_owned());
    };
    let package = manifest.get("package").and_then(toml::Value::as_table);
    let name = package
        .and_then(|p| p.get("name"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .unwrap_or(fallback_name);
    let version = package
        .and_then(|p| p.get("version"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| "0.1.0".to_owned());
    let author = package
        .and_then(|p| p.get("authors"))
        .and_then(toml::Value::as_array)
        .and_then(|authors| authors.first())
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| "TODO".to_owned());
    (name, version, author)
}

/// Appends entries to `.gitignore` if they are not already present.
fn ensure_gitignore(cwd: &Path, entries: &[&str]) {
    let gitignore = cwd.join(".gitignore");
    let mut content = fs::read_to_string(&gitignore).unwrap_or_default();
    let mut changed = false;
    for &entry in entries {
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
        if let Err(err) = fs::write(&gitignore, content) {
            crate::state::log(&format!(
                "[fy-docs] could not update {}: {err}",
                gitignore.display()
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffolds_docs_directory() {
        let temp = std::env::temp_dir().join(format!("fy-docs-init-{}", std::process::id()));
        std::fs::create_dir_all(&temp).unwrap();
        std::fs::write(
            temp.join("Cargo.toml"),
            "[package]\nname = \"test-project\"\nversion = \"1.0.0\"\nauthors = [\"Tester <t@example.com>\"]\n",
        )
        .unwrap();

        init(&temp).unwrap();

        assert!(temp.join("docs/main.typ").is_file());
        assert!(temp.join("docs/fy-spec/lib.typ").is_file());
        assert!(temp.join("docs/modules").is_dir());

        let main = std::fs::read_to_string(temp.join("docs/main.typ")).unwrap();
        assert!(main.contains("test-project"));
        assert!(main.contains("1.0.0"));
        assert!(main.contains("Tester"));
        assert!(!main.contains("fengyangsi"));

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn refuses_to_overwrite_existing_main() {
        let temp = std::env::temp_dir().join(format!("fy-docs-init-dup-{}", std::process::id()));
        std::fs::create_dir_all(temp.join("docs")).unwrap();
        std::fs::write(temp.join("docs/main.typ"), "existing").unwrap();

        let result = init(&temp);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn falls_back_without_cargo_toml() {
        let temp =
            std::env::temp_dir().join(format!("fy-docs-init-no-cargo-{}", std::process::id()));
        std::fs::create_dir_all(&temp).unwrap();

        init(&temp).unwrap();

        let main = std::fs::read_to_string(temp.join("docs/main.typ")).unwrap();
        assert!(main.contains("0.1.0"));
        assert!(main.contains(r#"author: "TODO""#));

        std::fs::remove_dir_all(temp).unwrap();
    }
}
