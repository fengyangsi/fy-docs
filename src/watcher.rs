//! Watches `.typ` sources (project docs plus imported local packages) and
//! regenerates the page with a debounce.

use crate::compiler;
use crate::state::AppState;
use anyhow::Result;
use notify::{Event, RecursiveMode, Watcher};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

/// Spawns the background watcher thread; it lives until the process exits.
pub fn spawn(state: Arc<AppState>) -> Result<()> {
    let (tx, rx) = channel::<notify::Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    for dir in &state.project.watch_dirs {
        watcher.watch(dir, RecursiveMode::Recursive)?;
    }
    std::thread::spawn(move || {
        // Hold the watcher: dropping it unregisters the watch.
        let _watcher = watcher;
        while wait_for_source_change(&rx) {
            // Reuse the startup options so `dev --lang zh-CN` keeps building
            // only the filtered language after the first save.
            let generate = &state.generate;
            compiler::generate_into(&state, generate.with_pdf, generate.lang_filter.as_deref());
            state.bump_build();
            crate::state::log(&format!(
                "[fy-docs] build #{} finished",
                state.current_build_id()
            ));
        }
    });
    Ok(())
}

/// Blocks until a `.typ` source change arrives, folding the editor's save
/// burst into one notification. Returns `false` only when the channel closed.
fn wait_for_source_change(rx: &Receiver<notify::Result<Event>>) -> bool {
    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if event.paths.iter().any(|path| is_typ_source(path)) {
                    let deadline = Instant::now() + Duration::from_millis(500);
                    while let Ok(Ok(_)) =
                        rx.recv_timeout(deadline.saturating_duration_since(Instant::now()))
                    {
                    }
                    return true;
                }
            }
            Ok(Err(_)) => continue,
            Err(_) => return false,
        }
    }
}

/// Only `.typ` edits count; generated files under `docs/target/` carry other
/// extensions, so the watcher never triggers itself.
fn is_typ_source(path: &std::path::Path) -> bool {
    path.extension().is_some_and(|ext| ext == "typ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn is_typ_source_identifies_typst_files() {
        assert!(is_typ_source(Path::new("main.typ")));
        assert!(is_typ_source(Path::new("docs/modules/truth.typ")));
        assert!(!is_typ_source(Path::new("index.html")));
        assert!(!is_typ_source(Path::new("fy-docs.css")));
        assert!(!is_typ_source(Path::new("Cargo.toml")));
        assert!(!is_typ_source(Path::new("no_extension")));
    }

    #[test]
    fn watcher_spawn_initiates_successfully() {
        use crate::project::Project;

        let temp = std::env::temp_dir().join(format!("fy-docs-watch-{}", std::process::id()));
        let docs = temp.join("docs");
        std::fs::create_dir_all(&docs).unwrap();
        let project = Project {
            name: "watch_test".to_owned(),
            version: "0.1.0".to_owned(),
            repository: None,
            entry: docs.join("main.typ"),
            targets: Vec::new(),
            docs_dir: docs.clone(),
            root: temp.clone(),
            target_dir: docs.join("target"),
            release_dir: docs.join("release"),
            watch_dirs: vec![docs.clone()],
        };
        let state = Arc::new(AppState::new(project));
        let res = spawn(state);
        assert!(res.is_ok());
        let _ = std::fs::remove_dir_all(temp);
    }
}
