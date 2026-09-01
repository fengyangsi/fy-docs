//! Package metadata read from `Cargo.toml`, honoring Cargo's
//! `field.workspace = true` inheritance for scalar fields.

use std::path::Path;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_package_metadata_from_valid_toml() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"1.2.3\"\nrepository = \"https://github.com/org/repo\"\nauthors = [\"Tester <t@example.com>\"]\n",
        )
        .unwrap();

        let meta = cargo_package_meta(temp.path());
        assert_eq!(meta.name.as_deref(), Some("demo"));
        assert_eq!(meta.version.as_deref(), Some("1.2.3"));
        assert_eq!(
            meta.repository.as_deref(),
            Some("https://github.com/org/repo")
        );
        assert_eq!(meta.author.as_deref(), Some("Tester <t@example.com>"));
    }
}
