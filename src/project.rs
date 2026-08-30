//! Detects the documentation project in the current directory: entry file,
//! package name/version, typst compile root, i18n language targets, and watch targets.

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

/// A target language document within a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LanguageTarget {
    /// Language identifier, e.g. "zh-CN", "en", "zh-TW", or empty string for default.
    pub(crate) lang: String,
    /// Display name in language switcher, e.g. "简体中文", "English".
    pub(crate) display_name: String,
    /// Path to entry `main.typ` for this language.
    pub(crate) entry: PathBuf,
    /// Output html filename, e.g. "index.html" or "index_zh-CN.html".
    pub(crate) html_file_name: String,
    /// Release PDF filename.
    pub(crate) pdf_file_name: String,
}

/// A documentation project: `<cwd>/docs/` plus everything needed to compile,
/// build, and serve single or multi-language specifications.
#[derive(Debug, Clone)]
pub(crate) struct Project {
    /// Package name (Cargo.toml `name`, else the directory name).
    pub(crate) name: String,
    /// Package version (Cargo.toml, else `version:` in main.typ, else 0.1.0).
    pub(crate) version: String,
    /// GitHub repository URL from Cargo.toml, when supplied.
    pub(crate) repository: Option<String>,
    /// All detected language targets (single-language or i18n).
    pub(crate) targets: Vec<LanguageTarget>,
    /// The `docs/` source directory.
    pub(crate) docs_dir: PathBuf,
    /// Typst compile root: the ancestor directory that satisfies every
    /// absolute `#import "/..."` used by the sources.
    pub(crate) root: PathBuf,
    /// Generated HTML output: `docs/target/`.
    pub(crate) target_dir: PathBuf,
    /// Versioned PDF directory: `docs/release/`.
    pub(crate) release_dir: PathBuf,
    /// Extra directories to watch for changes (imported local packages).
    pub(crate) watch_dirs: Vec<PathBuf>,
}

/// Directories under `docs/` that hold generated output rather than sources, so
/// they can never be language targets.
const GENERATED_DOC_DIRS: &[&str] = &["target", "release"];

/// Normalizes a language tag for comparison: trimmed, lowercase, and with `_`
/// interchangeable with `-`, so `zh_CN` and `ZH-cn` both mean `zh-CN`.
pub(crate) fn normalize_lang(lang: &str) -> String {
    lang.trim().to_lowercase().replace('_', "-")
}

impl Project {
    /// Selects targets matching an optional language filter.
    ///
    /// The `default` target (a root `main.typ`) rides along only when the
    /// filter matched at least one named language: a filter that matches
    /// nothing selects nothing, so the caller surfaces the typo instead of
    /// quietly building the default page.
    pub(crate) fn selected_targets(&self, lang_filter: Option<&str>) -> Vec<&LanguageTarget> {
        let Some(filter) = lang_filter else {
            return self.targets.iter().collect();
        };
        let filter = normalize_lang(filter);
        let matched = self
            .targets
            .iter()
            .any(|t| !t.lang.is_empty() && normalize_lang(&t.lang) == filter);
        if !matched {
            return Vec::new();
        }
        self.targets
            .iter()
            .filter(|t| t.lang.is_empty() || normalize_lang(&t.lang) == filter)
            .collect()
    }
}

/// Maps language codes to user-friendly native display labels.
pub(crate) fn lang_display_name(lang: &str) -> String {
    match lang.to_lowercase().replace('_', "-").as_str() {
        "zh" | "zh-cn" | "zh-hans" => "简体中文".to_owned(),
        "zh-tw" | "zh-hk" | "zh-hant" => "繁體中文".to_owned(),
        "en" | "en-us" | "en-gb" => "English".to_owned(),
        "ja" | "ja-jp" => "日本語".to_owned(),
        "de" | "de-de" => "Deutsch".to_owned(),
        "fr" | "fr-fr" => "Français".to_owned(),
        "ru" | "ru-ru" => "Русский".to_owned(),
        "es" | "es-es" => "Español".to_owned(),
        _ => lang.to_owned(),
    }
}

pub(crate) fn clean_canonicalize(path: &Path) -> Result<PathBuf> {
    let p = path.canonicalize().context("path does not exist")?;
    #[cfg(windows)]
    {
        let raw = p.to_string_lossy();
        if let Some(rest) = raw.strip_prefix(r"\\?\") {
            return Ok(PathBuf::from(rest));
        }
    }
    Ok(p)
}

pub(crate) fn detect(cwd: &Path, root_override: Option<&Path>) -> Result<Project> {
    let docs_dir = cwd.join("docs");
    if !docs_dir.is_dir() {
        bail!(
            "`{}` has no docs/ directory — cargo fy-docs runs inside a project directory (like cargo doc runs inside a crate)",
            crate::state::display_path(cwd)
        );
    }

    let meta = cargo_package_meta(cwd);
    let name = meta.name.unwrap_or_else(|| {
        cwd.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "project".to_owned())
    });

    // Detect language targets within docs/
    let (targets, primary_entry) =
        detect_language_targets(&docs_dir, &name, meta.version.as_deref())?;

    let version = meta
        .version
        .unwrap_or_else(|| main_typ_version(&primary_entry).unwrap_or_else(|| "0.1.0".to_owned()));

    let imports = absolute_imports(&docs_dir);
    let root = match root_override {
        Some(root) => clean_canonicalize(root)?,
        None => detect_root(cwd, &imports).context(
            "could not locate the typst root that satisfies the absolute imports; pass --root",
        )?,
    };

    // Watch the docs sources plus every local directory the imports pull in.
    let mut watch_dirs = vec![docs_dir.clone()];
    for import in &imports {
        if let Some(top) = import.split('/').next() {
            let dir = root.join(top);
            if dir.is_dir() && !watch_dirs.contains(&dir) {
                watch_dirs.push(dir);
            }
        }
    }

    let target_dir = docs_dir.join("target");
    let release_dir = docs_dir.join("release");
    Ok(Project {
        name,
        version,
        repository: meta.repository,
        targets,
        docs_dir,
        root,
        target_dir,
        release_dir,
        watch_dirs,
    })
}

fn detect_language_targets(
    docs_dir: &Path,
    pkg_name: &str,
    version_hint: Option<&str>,
) -> Result<(Vec<LanguageTarget>, PathBuf)> {
    let mut targets = Vec::new();
    let root_main = docs_dir.join("main.typ");

    // Check for language subdirectories: docs/<lang>/main.typ
    let mut sub_targets = Vec::new();
    if let Ok(entries) = std::fs::read_dir(docs_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = entry.file_name().to_string_lossy().into_owned();
                // Generated output is the only thing that can never be a
                // language: every other directory qualifies through its own
                // main.typ, so shared folders like fy-spec/ need no denylist.
                if GENERATED_DOC_DIRS.contains(&dir_name.as_str()) {
                    continue;
                }
                let sub_main = path.join("main.typ");
                if sub_main.is_file() {
                    let version = version_hint
                        .map(str::to_owned)
                        .or_else(|| main_typ_version(&sub_main))
                        .unwrap_or_else(|| "0.1.0".to_owned());
                    sub_targets.push(LanguageTarget {
                        lang: dir_name.clone(),
                        display_name: lang_display_name(&dir_name),
                        entry: sub_main,
                        html_file_name: format!("index_{dir_name}.html"),
                        pdf_file_name: format!(
                            "{pkg_name}_v{version}_{dir_name}_specification.pdf"
                        ),
                    });
                }
            }
        }
    }

    // Sort sub-targets deterministically by language code
    sub_targets.sort_by(|a, b| a.lang.cmp(&b.lang));

    let primary_entry;
    if !sub_targets.is_empty() {
        if root_main.is_file() {
            primary_entry = root_main.clone();
            let version = version_hint
                .map(str::to_owned)
                .or_else(|| main_typ_version(&root_main))
                .unwrap_or_else(|| "0.1.0".to_owned());
            targets.push(LanguageTarget {
                lang: "".to_owned(),
                display_name: "Default".to_owned(),
                entry: root_main,
                html_file_name: "index.html".to_owned(),
                pdf_file_name: format!("{pkg_name}_v{version}_specification.pdf"),
            });
        } else {
            primary_entry = sub_targets[0].entry.clone();
        }
        targets.extend(sub_targets);
    } else if root_main.is_file() {
        primary_entry = root_main.clone();
        let version = version_hint
            .map(str::to_owned)
            .or_else(|| main_typ_version(&root_main))
            .unwrap_or_else(|| "0.1.0".to_owned());
        targets.push(LanguageTarget {
            lang: "".to_owned(),
            display_name: "Default".to_owned(),
            entry: root_main,
            html_file_name: "index.html".to_owned(),
            pdf_file_name: format!("{pkg_name}_v{version}_specification.pdf"),
        });
    } else {
        bail!(
            "`{}` has no main.typ or language subdirectory (e.g. docs/zh-CN/main.typ)",
            docs_dir.display()
        );
    }

    Ok((targets, primary_entry))
}

/// Walks up from `start` looking for the closest ancestor under which every
/// absolute import target exists. No imports → the project directory itself.
fn detect_root(start: &Path, imports: &[String]) -> Result<PathBuf> {
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
fn absolute_imports(docs_dir: &Path) -> Vec<String> {
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

/// Package metadata read from `Cargo.toml`, honoring `workspace = true`
/// inheritance for scalar fields.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct CargoMeta {
    pub(crate) name: Option<String>,
    pub(crate) version: Option<String>,
    /// A GitHub repository URL, when the manifest declares one.
    pub(crate) repository: Option<String>,
    /// The first `authors` entry, when declared directly in the manifest.
    pub(crate) author: Option<String>,
}

/// Reads `[package]` metadata from `Cargo.toml`, if present.
pub(crate) fn cargo_package_meta(cwd: &Path) -> CargoMeta {
    let Ok(text) = std::fs::read_to_string(cwd.join("Cargo.toml")) else {
        return CargoMeta::default();
    };
    let Ok(manifest) = text.parse::<toml::Table>() else {
        return CargoMeta::default();
    };
    let Some(package) = manifest.get("package").and_then(toml::Value::as_table) else {
        return CargoMeta::default();
    };
    let value = |key| package_value(cwd, package, key);
    let repository = value("repository").filter(|url| is_github_repository(url));
    CargoMeta {
        name: value("name"),
        version: value("version"),
        repository,
        author: package
            .get("authors")
            .and_then(toml::Value::as_array)
            .and_then(|authors| authors.first())
            .and_then(toml::Value::as_str)
            .map(str::to_owned),
    }
}

/// Appends entries to the project's `.gitignore` when they are missing so
/// generated directories stay untracked; write failures are logged, never
/// fatal.
pub(crate) fn ensure_gitignore(root: &Path, entries: &[&str]) {
    let gitignore = root.join(".gitignore");
    let mut content = std::fs::read_to_string(&gitignore).unwrap_or_default();
    let mut changed = false;
    for entry in entries {
        if !content.lines().any(|line| line.trim() == *entry) {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(entry);
            content.push('\n');
            changed = true;
        }
    }
    if changed && let Err(err) = std::fs::write(&gitignore, content) {
        crate::state::log(&format!(
            "[fy-docs] could not update {}: {err}",
            gitignore.display()
        ));
    }
}

fn is_github_repository(url: &str) -> bool {
    url.strip_prefix("https://github.com/")
        .is_some_and(|path| !path.trim_matches('/').is_empty())
}

/// Reads either a direct package field or Cargo's `field.workspace = true`
/// inheritance from the nearest workspace manifest.
fn package_value(
    cwd: &Path,
    package: &toml::map::Map<String, toml::Value>,
    key: &str,
) -> Option<String> {
    if let Some(value) = package.get(key).and_then(toml::Value::as_str) {
        return Some(value.to_owned());
    }
    let inherited = package
        .get(key)
        .and_then(toml::Value::as_table)
        .and_then(|field| field.get("workspace"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    inherited
        .then(|| workspace_package_value(cwd, key))
        .flatten()
}

fn workspace_package_value(start: &Path, key: &str) -> Option<String> {
    for directory in start.ancestors() {
        let Ok(text) = std::fs::read_to_string(directory.join("Cargo.toml")) else {
            continue;
        };
        let Ok(manifest) = text.parse::<toml::Table>() else {
            continue;
        };
        let value = manifest
            .get("workspace")
            .and_then(toml::Value::as_table)
            .and_then(|workspace| workspace.get("package"))
            .and_then(toml::Value::as_table)
            .and_then(|package| package.get(key))
            .and_then(toml::Value::as_str);
        if let Some(value) = value {
            return Some(value.to_owned());
        }
    }
    None
}

/// Falls back to the `version: "..."` argument of the document template.
fn main_typ_version(entry: &Path) -> Option<String> {
    parse_main_typ_version(&std::fs::read_to_string(entry).ok()?)
}

/// Reads the template's `version:` argument from live code only. A mention
/// inside a comment must not become the project version, and an occurrence
/// without a quoted value is skipped rather than ending the search.
fn parse_main_typ_version(text: &str) -> Option<String> {
    for line in text.lines() {
        let code = match line.find("//") {
            Some(cut) => &line[..cut],
            None => line,
        };
        let Some(at) = code.find("version:") else {
            continue;
        };
        let rest = &code[at + "version:".len()..];
        let Some(open) = rest.find('"') else {
            continue;
        };
        let from = open + 1;
        let Some(close) = rest[from..].find('"') else {
            continue;
        };
        return Some(rest[from..from + close].to_owned());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_package_metadata_from_valid_toml() {
        let temp = std::env::temp_dir().join(format!("fy-docs-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).unwrap();
        std::fs::write(
            temp.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"1.2.3\"\nrepository = \"https://github.com/org/repo\"\nauthors = [\"Tester <t@example.com>\"]\n",
        )
        .unwrap();

        let meta = cargo_package_meta(&temp);
        assert_eq!(meta.name.as_deref(), Some("demo"));
        assert_eq!(meta.version.as_deref(), Some("1.2.3"));
        assert_eq!(
            meta.repository.as_deref(),
            Some("https://github.com/org/repo")
        );
        assert_eq!(meta.author.as_deref(), Some("Tester <t@example.com>"));
        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn gitignore_entries_are_added_once() {
        let temp = std::env::temp_dir().join(format!("fy-docs-ign-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).unwrap();
        std::fs::write(temp.join(".gitignore"), "/existing\n").unwrap();

        ensure_gitignore(&temp, &["/docs/target/"]);
        ensure_gitignore(&temp, &["/docs/target/"]);
        let content = std::fs::read_to_string(temp.join(".gitignore")).unwrap();
        assert_eq!(content, "/existing\n/docs/target/\n");

        let _ = std::fs::remove_dir_all(&temp);
    }

    fn project_with_langs(langs: &[&str]) -> Project {
        let target = |lang: &str| LanguageTarget {
            lang: lang.to_owned(),
            display_name: lang_display_name(lang),
            entry: PathBuf::new(),
            html_file_name: if lang.is_empty() {
                "index.html".to_owned()
            } else {
                format!("index_{lang}.html")
            },
            pdf_file_name: format!("demo_{lang}.pdf"),
        };
        Project {
            name: "demo".to_owned(),
            version: "0.1.0".to_owned(),
            repository: None,
            targets: langs.iter().map(|lang| target(lang)).collect(),
            docs_dir: PathBuf::new(),
            root: PathBuf::new(),
            target_dir: PathBuf::new(),
            release_dir: PathBuf::new(),
            watch_dirs: Vec::new(),
        }
    }

    fn selected_langs(project: &Project, filter: Option<&str>) -> Vec<String> {
        project
            .selected_targets(filter)
            .into_iter()
            .map(|t| t.lang.clone())
            .collect()
    }

    #[test]
    fn normalize_lang_folds_case_and_separators() {
        assert_eq!(normalize_lang("ZH_CN"), "zh-cn");
        assert_eq!(normalize_lang(" en "), "en");
        assert_eq!(normalize_lang(""), "");
    }

    #[test]
    fn lang_filter_accepts_case_and_separator_variants() {
        let project = project_with_langs(&["", "zh-CN", "en"]);
        for variant in ["zh-CN", "zh-cn", "ZH_CN", "zh_CN", " Zh-Cn "] {
            assert_eq!(
                selected_langs(&project, Some(variant)),
                vec![String::new(), "zh-CN".to_owned()],
                "`{variant}` must select the zh-CN target alongside the default"
            );
        }
        assert_eq!(
            selected_langs(&project, Some("en")),
            vec![String::new(), "en".to_owned()]
        );
        assert_eq!(selected_langs(&project, None).len(), 3);
    }

    #[test]
    fn unmatched_lang_filter_selects_nothing() {
        // A root main.typ must not mask a typo: falling back to "default only"
        // would exit 0 having built something nobody asked for.
        let project = project_with_langs(&["", "zh-CN", "en"]);
        assert!(project.selected_targets(Some("zz")).is_empty());
        assert!(project.selected_targets(Some("zh")).is_empty());

        let single_language = project_with_langs(&[""]);
        assert!(single_language.selected_targets(Some("en")).is_empty());
    }

    #[test]
    fn version_fallback_reads_live_code_only() {
        let text = "// version: \"9.9.9\"\n#show: project_book.with(\n  title: \"x\", // version: \"8.8.8\"\n  version: none,\n  version: \"1.2.3\",\n)\n";
        assert_eq!(parse_main_typ_version(text).as_deref(), Some("1.2.3"));
        assert_eq!(parse_main_typ_version("// version: \"1.0.0\""), None);
        assert_eq!(parse_main_typ_version("no version here"), None);
    }

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
