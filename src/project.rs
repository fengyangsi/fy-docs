//! Detects the documentation project in the current directory: entry file,
//! package name/version, typst compile root, and watch targets.

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

/// A documentation project: `<cwd>/docs/main.typ` plus everything needed to
/// compile and serve it.
#[derive(Debug, Clone)]
pub struct Project {
    /// Package name (Cargo.toml `name`, else the directory name).
    pub name: String,
    /// Package version (Cargo.toml, else `version:` in main.typ, else 0.1.0).
    pub version: String,
    /// GitHub repository URL from Cargo.toml, when supplied.
    pub repository: Option<String>,
    /// Path to `docs/main.typ`.
    pub entry: PathBuf,
    /// The `docs/` source directory.
    pub docs_dir: PathBuf,
    /// Typst compile root: the ancestor directory that satisfies every
    /// absolute `#import "/..."` used by the sources.
    pub root: PathBuf,
    /// Generated HTML output: `docs/target/`.
    pub target_dir: PathBuf,
    /// Versioned PDF directory: `docs/release/`.
    pub release_dir: PathBuf,
    /// Extra directories to watch for changes (imported local packages).
    pub watch_dirs: Vec<PathBuf>,
}

impl Project {
    /// File name of the release PDF: `{name}_v{version}_specification.pdf`.
    pub fn pdf_file_name(&self) -> String {
        format!("{}_v{}_specification.pdf", self.name, self.version)
    }
}

pub fn detect(cwd: &Path, root_override: Option<&Path>) -> Result<Project> {
    let docs_dir = cwd.join("docs");
    let entry = docs_dir.join("main.typ");
    if !entry.is_file() {
        bail!(
            "`{}` has no docs/main.typ — cargo fy-docs runs inside a project directory (like cargo doc runs inside a crate)",
            crate::state::display_path(cwd)
        );
    }

    let (cargo_name, cargo_version, repository) = cargo_package_info(cwd);
    let name = cargo_name.unwrap_or_else(|| {
        cwd.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "project".to_owned())
    });
    let version = cargo_version
        .unwrap_or_else(|| main_typ_version(&entry).unwrap_or_else(|| "0.1.0".to_owned()));

    let imports = absolute_imports(&docs_dir);
    let root = match root_override {
        Some(root) => root.canonicalize().context("--root path does not exist")?,
        None => detect_root(cwd, &imports).context(
            "could not locate the typst root that satisfies the absolute imports; pass --root",
        )?,
    };

    // Watch the docs sources plus every local directory the imports pull in
    // (e.g. a sibling shared template package).
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
        repository,
        entry,
        docs_dir,
        root,
        target_dir,
        release_dir,
        watch_dirs,
    })
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
            if path.file_name().is_some_and(|n| n == "target") {
                continue; // generated output, never imported
            }
            collect_imports(&path, imports);
        } else if path.extension().is_some_and(|ext| ext == "typ")
            && let Ok(text) = std::fs::read_to_string(&path)
        {
            for line in text.lines() {
                if !line.contains("import") {
                    continue;
                }
                for quoted in line.split('"').skip(1).step_by(2) {
                    if let Some(rel) = quoted.strip_prefix('/')
                        && rel.ends_with(".typ")
                    {
                        imports.push(rel.to_owned());
                    }
                }
            }
        }
    }
}

/// Reads `[package] name/version` from Cargo.toml, if present.
fn cargo_package_info(cwd: &Path) -> (Option<String>, Option<String>, Option<String>) {
    let Ok(text) = std::fs::read_to_string(cwd.join("Cargo.toml")) else {
        return (None, None, None);
    };
    let Ok(manifest) = text.parse::<toml::Table>() else {
        return (None, None, None);
    };
    let Some(package) = manifest.get("package").and_then(toml::Value::as_table) else {
        return (None, None, None);
    };
    let value = |key| package_value(cwd, package, key);
    let repository = value("repository").filter(|url| is_github_repository(url));
    (value("name"), value("version"), repository)
}

fn is_github_repository(url: &str) -> bool {
    url.strip_prefix("https://github.com/")
        .is_some_and(|path| !path.trim_matches('/').is_empty())
}

/// Reads either a direct package field or Cargo's \`field.workspace = true\`
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
    let text = std::fs::read_to_string(entry).ok()?;
    let pos = text.find("version:")?;
    let rest = &text[pos + "version:".len()..];
    let from = rest.find('"')? + 1;
    let to = rest[from..].find('"')? + from;
    Some(rest[from..to].to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_pdf_as_a_document_release() {
        let project = Project {
            name: "fy-docs".to_owned(),
            version: "0.1.0".to_owned(),
            repository: None,
            entry: PathBuf::new(),
            docs_dir: PathBuf::new(),
            root: PathBuf::new(),
            target_dir: PathBuf::new(),
            release_dir: PathBuf::new(),
            watch_dirs: Vec::new(),
        };
        assert_eq!(project.pdf_file_name(), "fy-docs_v0.1.0_specification.pdf");
    }

    #[test]
    fn reads_package_metadata_from_valid_toml() {
        let temp = std::env::temp_dir().join(format!("fy-docs-test-{}", std::process::id()));
        std::fs::create_dir_all(&temp).unwrap();
        std::fs::write(
            temp.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"1.2.3\"\n",
        )
        .unwrap();
        assert_eq!(
            cargo_package_info(&temp),
            (Some("demo".to_owned()), Some("1.2.3".to_owned()), None)
        );
        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn reads_workspace_inherited_package_metadata() {
        let temp =
            std::env::temp_dir().join(format!("fy-docs-workspace-test-{}", std::process::id()));
        let member = temp.join("member");
        std::fs::create_dir_all(&member).unwrap();
        std::fs::write(
            temp.join("Cargo.toml"),
            "[workspace]\nmembers = [\"member\"]\n[workspace.package]\nname = \"workspace-demo\"\nversion = \"2.0.0\"\n",
        )
        .unwrap();
        std::fs::write(
            member.join("Cargo.toml"),
            "[package]\nname.workspace = true\nversion.workspace = true\n",
        )
        .unwrap();
        assert_eq!(
            cargo_package_info(&member),
            (
                Some("workspace-demo".to_owned()),
                Some("2.0.0".to_owned()),
                None
            )
        );
        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn accepts_only_github_repository_urls() {
        assert!(is_github_repository(
            "https://github.com/fengyangsi/fy-docs"
        ));
        assert!(!is_github_repository(
            "https://codeberg.org/fengyangsi/fy-docs"
        ));
    }

    #[test]
    fn detect_fails_when_main_typ_missing() {
        let temp = std::env::temp_dir().join(format!("fy-docs-no-main-{}", std::process::id()));
        std::fs::create_dir_all(&temp).unwrap();
        assert!(detect(&temp, None).is_err());
        let _ = std::fs::remove_dir_all(temp);
    }

    #[test]
    fn detect_succeeds_with_minimal_project() {
        let temp = std::env::temp_dir().join(format!("fy-docs-detect-{}", std::process::id()));
        let docs = temp.join("docs");
        std::fs::create_dir_all(&docs).unwrap();
        std::fs::write(
            docs.join("main.typ"),
            "#import \"fy-spec/lib.typ\": *\n#show: project_book.with(title: \"Doc\", version: \"0.3.1\")\n",
        )
        .unwrap();

        let proj = detect(&temp, None).unwrap();
        assert_eq!(proj.name, temp.file_name().unwrap().to_str().unwrap());
        assert_eq!(proj.version, "0.3.1");
        assert_eq!(proj.root, temp);

        // With explicit root
        let explicit_root = temp.join("custom_root");
        std::fs::create_dir_all(&explicit_root).unwrap();
        let proj2 = detect(&temp, Some(&explicit_root)).unwrap();
        assert_eq!(proj2.root, explicit_root.canonicalize().unwrap());

        let _ = std::fs::remove_dir_all(temp);
    }

    #[test]
    fn main_typ_version_fallback() {
        let temp = std::env::temp_dir().join(format!("fy-docs-ver-{}", std::process::id()));
        std::fs::create_dir_all(&temp).unwrap();
        let entry = temp.join("main.typ");
        std::fs::write(&entry, "version: \"1.9.4\"").unwrap();
        assert_eq!(main_typ_version(&entry), Some("1.9.4".to_owned()));

        std::fs::write(&entry, "no version here").unwrap();
        assert_eq!(main_typ_version(&entry), None);
        let _ = std::fs::remove_dir_all(temp);
    }

    #[test]
    fn detects_absolute_imports_and_resolves_root() {
        let temp = std::env::temp_dir().join(format!("fy-docs-import-{}", std::process::id()));
        let docs = temp.join("docs");
        std::fs::create_dir_all(&docs).unwrap();
        std::fs::write(
            docs.join("main.typ"),
            "// comment with \"/fake.typ\"\n#import \"/shared/lib.typ\": *\n",
        )
        .unwrap();

        let imports = absolute_imports(&docs);
        assert_eq!(imports, vec!["shared/lib.typ".to_owned()]);

        // When directory doesn't exist on disk, detect_root returns Err
        assert!(detect_root(&temp, &imports).is_err());

        // When shared exists, detect_root resolves closest root
        let shared = temp.join("shared");
        std::fs::create_dir_all(&shared).unwrap();
        std::fs::write(shared.join("lib.typ"), "content").unwrap();
        let root2 = detect_root(&temp, &imports).unwrap();
        assert_eq!(root2, temp);

        let _ = std::fs::remove_dir_all(temp);
    }
}
