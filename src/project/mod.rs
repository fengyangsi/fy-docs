//! Detects the documentation project in the current directory: entry file,
//! package name/version, typst compile root, i18n language targets, and watch targets.

mod cargo_meta;
mod imports;
mod lang;
mod template_args;

pub(crate) use cargo_meta::cargo_package_meta;
pub(crate) use lang::{LanguageTarget, lang_display_name, normalize_lang};

use anyhow::{Context, Result, bail};
use imports::{absolute_imports, detect_root};
use std::path::{Path, PathBuf};
use template_args::main_typ_version;

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

#[cfg(test)]
impl Project {
    /// A minimal project rooted at `docs_dir` for tests: no targets, the
    /// docs directory itself under watch. Fields stay public within the
    /// crate, so a test shapes only what it exercises.
    pub(crate) fn for_tests(docs_dir: PathBuf) -> Self {
        Self {
            name: "test".to_owned(),
            version: "0.1.0".to_owned(),
            repository: None,
            targets: Vec::new(),
            docs_dir: docs_dir.clone(),
            root: docs_dir.clone(),
            target_dir: docs_dir.join("target"),
            release_dir: docs_dir.join("release"),
            watch_dirs: vec![docs_dir],
        }
    }
}

/// Directories under `docs/` that hold generated output rather than sources, so
/// they can never be language targets.
const GENERATED_DOC_DIRS: &[&str] = &["target", "release"];

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
            crate::term::display_path(cwd)
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

/// Resolves a target's version in one place: the manifest hint, else the
/// entry's `version:` argument, else `0.1.0`.
fn resolve_version(version_hint: Option<&str>, entry: &Path) -> String {
    version_hint
        .map(str::to_owned)
        .or_else(|| main_typ_version(entry))
        .unwrap_or_else(|| "0.1.0".to_owned())
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
            if !path.is_dir() {
                continue;
            }
            let dir_name = entry.file_name().to_string_lossy().into_owned();
            // Generated output is the only thing that can never be a
            // language: every other directory qualifies through its own
            // main.typ, so shared folders like fy-spec/ need no denylist.
            if GENERATED_DOC_DIRS.contains(&dir_name.as_str()) {
                continue;
            }
            let sub_main = path.join("main.typ");
            if sub_main.is_file() {
                let version = resolve_version(version_hint, &sub_main);
                let display = lang_display_name(&dir_name);
                sub_targets.push(LanguageTarget::new(
                    &dir_name, &display, sub_main, pkg_name, &version,
                ));
            }
        }
    }

    // Sort sub-targets deterministically by language code
    sub_targets.sort_by(|a, b| a.lang.cmp(&b.lang));

    // A root main.typ registers the default target on top of the languages.
    let register_default = |targets: &mut Vec<LanguageTarget>| -> PathBuf {
        let version = resolve_version(version_hint, &root_main);
        targets.push(LanguageTarget::new(
            "",
            "Default",
            root_main.clone(),
            pkg_name,
            &version,
        ));
        root_main.clone()
    };

    let primary_entry;
    if !sub_targets.is_empty() {
        if root_main.is_file() {
            primary_entry = register_default(&mut targets);
        } else {
            primary_entry = sub_targets[0].entry.clone();
        }
        targets.extend(sub_targets);
    } else if root_main.is_file() {
        primary_entry = register_default(&mut targets);
    } else {
        bail!(
            "`{}` has no main.typ or language subdirectory (e.g. docs/zh-CN/main.typ)",
            docs_dir.display()
        );
    }

    Ok((targets, primary_entry))
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
        crate::term::log(&format!(
            "[fy-docs] could not update {}: {err}",
            gitignore.display()
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::lang::format_lang;

    #[test]
    fn gitignore_entries_are_added_once() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join(".gitignore"), "/existing\n").unwrap();

        ensure_gitignore(temp.path(), &["/docs/target/"]);
        ensure_gitignore(temp.path(), &["/docs/target/"]);
        let content = std::fs::read_to_string(temp.path().join(".gitignore")).unwrap();
        assert_eq!(content, "/existing\n/docs/target/\n");
    }

    fn project_with_langs(langs: &[&str]) -> Project {
        let target = |lang: &str| LanguageTarget {
            lang: lang.to_owned(),
            content_lang: if lang.is_empty() {
                "en".to_owned()
            } else {
                format_lang(&normalize_lang(lang))
            },
            display_name: lang_display_name(lang),
            entry: PathBuf::new(),
            html_file_name: if lang.is_empty() {
                "index.html".to_owned()
            } else {
                format!("index_{lang}.html")
            },
            pdf_file_name: format!("demo_{lang}.pdf"),
        };
        let mut project = Project::for_tests(PathBuf::new());
        project.name = "demo".to_owned();
        project.targets = langs.iter().map(|lang| target(lang)).collect();
        project
    }

    fn selected_langs(project: &Project, filter: Option<&str>) -> Vec<String> {
        project
            .selected_targets(filter)
            .into_iter()
            .map(|t| t.lang.clone())
            .collect()
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

    /// Writes a `docs/` tree into a fresh temp project and detects it. The
    /// returned `TempDir` must outlive the `Project`.
    fn detect_docs(files: &[(&str, &str)]) -> (tempfile::TempDir, Project) {
        let temp = tempfile::tempdir().unwrap();
        for (name, content) in files {
            let path = temp.path().join("docs").join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, content).unwrap();
        }
        let project = detect(temp.path(), None).unwrap();
        (temp, project)
    }

    fn book_with(lang: Option<&str>, body: &str) -> String {
        let declared = lang.map_or(String::new(), |lang| format!("  lang: \"{lang}\",\n"));
        format!("#show: project_book.with(\n  title: \"T\",\n{declared})\n\n{body}\n")
    }

    #[test]
    fn the_language_directory_decides_content_lang() {
        // The entry disagrees with its folder; the folder is what the page,
        // the file name and the switcher are all built from.
        let (_temp, project) = detect_docs(&[("pt_BR/main.typ", &book_with(Some("en"), "Ola"))]);
        let target = project.targets.first().unwrap();
        assert_eq!(target.lang, "pt_BR");
        assert_eq!(target.content_lang, "pt-BR");
    }

    #[test]
    fn a_default_target_inherits_its_entry_declaration() {
        let (_temp, project) = detect_docs(&[("main.typ", &book_with(Some("zh-CN"), "内容"))]);
        let target = project.targets.first().unwrap();
        assert!(target.lang.is_empty());
        assert_eq!(target.content_lang, "zh-CN");

        let (_temp, project) = detect_docs(&[("main.typ", &book_with(Some("ZH_TW"), "text"))]);
        assert_eq!(project.targets.first().unwrap().content_lang, "zh-TW");
    }

    #[test]
    fn an_undeclared_default_target_is_english_even_with_chinese_text() {
        // Nothing is guessed from glyphs: an entry that declares no language
        // means the template's own default, which is English.
        let (_temp, project) = detect_docs(&[(
            "main.typ",
            &book_with(None, "#show: heading[中文规格说明]\n\n正文内容"),
        )]);
        let target = project.targets.first().unwrap();
        assert_eq!(target.lang, "");
        assert_eq!(target.content_lang, "en");
    }
}
