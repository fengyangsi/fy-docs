//! Shared state: the project plus a monotonically increasing build id.

use crate::project::Project;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Options captured at startup; dev-mode rebuilds reuse them so a filtered
/// session (for example `dev --lang zh-CN`) stays filtered after the first
/// save instead of silently rebuilding every language.
#[derive(Debug, Clone, Default)]
pub struct GenerateOptions {
    /// Also compile the print-edition PDF on every rebuild.
    pub with_pdf: bool,
    /// Restrict generation to one language target, e.g. `Some("zh-CN")`.
    pub lang_filter: Option<String>,
}

pub struct AppState {
    pub project: Project,
    pub build_id: AtomicU64,
    /// Startup options reused by the watcher on every rebuild.
    pub generate: GenerateOptions,
}

impl AppState {
    /// Plain constructor for contexts without CLI options (tests, tooling).
    #[allow(dead_code)]
    pub fn new(project: Project) -> Self {
        Self::with_generate(project, GenerateOptions::default())
    }

    /// Captures the CLI options so later rebuilds repeat the same generation.
    pub fn with_generate(project: Project, generate: GenerateOptions) -> Self {
        Self {
            project,
            build_id: AtomicU64::new(1),
            generate,
        }
    }

    pub fn current_build_id(&self) -> u64 {
        self.build_id.load(Ordering::SeqCst)
    }

    /// Bumps the build id and persists it into `docs/target/_build`; served
    /// pages poll this file and reload themselves when it changes.
    pub fn bump_build(&self) {
        let id = self.build_id.fetch_add(1, Ordering::SeqCst) + 1;
        let path = self.project.target_dir.join(crate::compiler::BUILD_FILE);
        if let Err(err) = std::fs::write(&path, id.to_string()) {
            crate::state::log(&format!(
                "[fy-docs] could not update {}: {err}",
                path.display()
            ));
        }
    }

    /// Writes the initial build id after the first successful generation.
    pub fn write_build(&self) {
        let path = self.project.target_dir.join(crate::compiler::BUILD_FILE);
        let _ = std::fs::write(&path, self.current_build_id().to_string());
    }
}

/// Writes a progress line to stderr. A closed output pipe (e.g. the server is
/// piped into `head`) must not panic the watcher thread, so write errors are
/// deliberately ignored.
pub fn log(message: &str) {
    use std::io::Write as _;
    let _ = writeln!(std::io::stderr().lock(), "{message}");
}

/// Formats a path for people instead of exposing Windows' internal verbatim
/// path prefix (for example `\\?\D:\...`) in terminal output.
pub fn display_path(path: &Path) -> String {
    let raw = path.display().to_string();
    #[cfg(windows)]
    {
        if let Some(unc) = raw.strip_prefix(r"\\?\UNC\") {
            return format!(r"\\{unc}");
        }
        raw.strip_prefix(r"\\?\").unwrap_or(&raw).to_owned()
    }
    #[cfg(not(windows))]
    raw
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn displays_normal_windows_paths_without_verbatim_prefix() {
        #[cfg(windows)]
        assert_eq!(display_path(Path::new(r"\\?\D:\Code\fy")), r"D:\Code\fy");
    }

    #[cfg(not(windows))]
    #[test]
    fn preserves_native_unix_paths() {
        assert_eq!(display_path(Path::new("/tmp/fy-docs")), "/tmp/fy-docs");
    }

    #[test]
    fn app_state_tracks_and_bumps_build_id() {
        let temp = std::env::temp_dir().join(format!("fy-docs-state-test-{}", std::process::id()));
        std::fs::create_dir_all(&temp).unwrap();
        let project = Project {
            name: "test".to_owned(),
            version: "0.1.0".to_owned(),
            repository: None,
            entry: temp.join("main.typ"),
            targets: Vec::new(),
            docs_dir: temp.clone(),
            root: temp.clone(),
            target_dir: temp.clone(),
            release_dir: temp.clone(),
            watch_dirs: Vec::new(),
        };
        let state = AppState::new(project);
        assert_eq!(state.current_build_id(), 1);

        state.write_build();
        let build_file = temp.join(crate::compiler::BUILD_FILE);
        assert_eq!(std::fs::read_to_string(&build_file).unwrap(), "1");

        state.bump_build();
        assert_eq!(state.current_build_id(), 2);
        assert_eq!(std::fs::read_to_string(&build_file).unwrap(), "2");

        let _ = std::fs::remove_dir_all(temp);
    }

    #[test]
    fn log_executes_safely() {
        log("test log output");
    }

    #[test]
    fn with_generate_captures_startup_options() {
        let state = AppState::with_generate(
            Project {
                name: "test".to_owned(),
                version: "0.1.0".to_owned(),
                repository: None,
                entry: PathBuf::new(),
                targets: Vec::new(),
                docs_dir: PathBuf::new(),
                root: PathBuf::new(),
                target_dir: PathBuf::new(),
                release_dir: PathBuf::new(),
                watch_dirs: Vec::new(),
            },
            GenerateOptions {
                with_pdf: true,
                lang_filter: Some("zh-CN".to_owned()),
            },
        );
        assert!(state.generate.with_pdf);
        assert_eq!(state.generate.lang_filter.as_deref(), Some("zh-CN"));

        // The plain constructor keeps the dev defaults.
        let plain = AppState::new(Project {
            name: "test".to_owned(),
            version: "0.1.0".to_owned(),
            repository: None,
            entry: PathBuf::new(),
            targets: Vec::new(),
            docs_dir: PathBuf::new(),
            root: PathBuf::new(),
            target_dir: PathBuf::new(),
            release_dir: PathBuf::new(),
            watch_dirs: Vec::new(),
        });
        assert!(!plain.generate.with_pdf);
        assert!(plain.generate.lang_filter.is_none());
    }
}
