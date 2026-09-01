//! Watches `.typ` sources (project docs plus imported local packages) and
//! regenerates the page with a debounce.

use crate::compiler;
use crate::project::Project;
use crate::state::AppState;
use anyhow::Result;
use notify::{Event, RecursiveMode, Watcher};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

/// Spawns the background watcher thread; it lives until the process exits.
pub(crate) fn spawn(state: Arc<AppState>) -> Result<()> {
    let (tx, rx) = channel::<notify::Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    for dir in &state.project.watch_dirs {
        watcher.watch(dir, RecursiveMode::Recursive)?;
    }
    std::thread::spawn(move || {
        // Hold the watcher: dropping it unregisters the watch.
        let _watcher = watcher;
        while wait_for_source_change(&rx, &state.project) {
            // generate reads the captured startup options itself, so
            // `dev --lang zh-CN` keeps building only the filtered language
            // after the first save. A failing rebuild leaves the server on
            // its error pages instead of killing the watcher.
            let _ = compiler::generate(&state);
            state.bump_build();
            crate::term::log(&format!(
                "[fy-docs] build #{} finished",
                state.current_build_id()
            ));
        }
    });
    Ok(())
}

/// Quiet period that folds an editor's save burst into a single rebuild.
const DEBOUNCE_QUIET: Duration = Duration::from_millis(500);

/// Hard cap on that fold, measured from the first change. Sliding the quiet
/// window alone would let a process writing sources continuously postpone the
/// rebuild forever.
const DEBOUNCE_MAX: Duration = Duration::from_millis(2_000);

/// Drains the channel until it has been quiet for [`DEBOUNCE_QUIET`], never
/// waiting past [`DEBOUNCE_MAX`].
fn debounce(rx: &Receiver<notify::Result<Event>>) {
    let started = Instant::now();
    let mut quiet_until = started + DEBOUNCE_QUIET;
    loop {
        let deadline = quiet_until.min(started + DEBOUNCE_MAX);
        match rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
            Ok(Ok(_)) => quiet_until = Instant::now() + DEBOUNCE_QUIET,
            // A timeout means the burst settled; a closed channel ends the loop.
            _ => return,
        }
    }
}

/// Blocks until a `.typ` source change arrives, folding the editor's save
/// burst into one notification. Returns `false` only when the channel closed.
fn wait_for_source_change(rx: &Receiver<notify::Result<Event>>, project: &Project) -> bool {
    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if event.paths.iter().any(|path| is_typ_source(path, project)) {
                    debounce(rx);
                    return true;
                }
            }
            Ok(Err(_)) => continue,
            Err(_) => return false,
        }
    }
}

/// A `.typ` edit counts as a source change unless it lands inside the
/// generated `target/` or `release/` directories. Excluding those by path —
/// not only by extension — keeps the watcher from reacting to its own output
/// even if a `.typ`-suffixed artifact ever appears there.
fn is_typ_source(path: &std::path::Path, project: &Project) -> bool {
    path.extension().is_some_and(|ext| ext == "typ")
        && !is_under(path, &project.target_dir)
        && !is_under(path, &project.release_dir)
}

fn is_under(path: &std::path::Path, dir: &std::path::Path) -> bool {
    path.strip_prefix(dir).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Project;
    use std::path::Path;

    #[test]
    fn is_typ_source_identifies_typst_files() {
        let project = Project::for_tests(std::path::PathBuf::from("docs"));
        assert!(is_typ_source(Path::new("main.typ"), &project));
        assert!(is_typ_source(Path::new("docs/modules/truth.typ"), &project));
        assert!(!is_typ_source(Path::new("index.html"), &project));
        assert!(!is_typ_source(Path::new("Cargo.toml"), &project));
        assert!(!is_typ_source(Path::new("no_extension"), &project));
    }

    #[test]
    fn generated_directories_are_excluded_by_path() {
        let project = Project::for_tests(std::path::PathBuf::from("docs"));
        // Even a .typ artifact inside target/ or release/ must not trigger a
        // rebuild (path-level exclusion, not just extension filtering).
        assert!(!is_typ_source(
            &project.target_dir.join("dump.typ"),
            &project
        ));
        assert!(!is_typ_source(
            &project.release_dir.join("spec.typ"),
            &project
        ));
        assert!(is_typ_source(&project.docs_dir.join("main.typ"), &project));
    }

    #[test]
    fn debounce_never_waits_past_its_cap() {
        let (tx, rx) = channel::<notify::Result<Event>>();
        let feeder = std::thread::spawn(move || {
            let event = notify::Event::new(notify::EventKind::Any);
            for _ in 0..10 {
                if tx.send(Ok(event.clone())).is_err() {
                    break;
                }
                // Arrive faster than the quiet window, so only the cap can end it.
                std::thread::sleep(Duration::from_millis(260));
            }
        });

        let started = Instant::now();
        debounce(&rx);
        let elapsed = started.elapsed();
        assert!(
            elapsed < Duration::from_millis(3_000),
            "a continuous stream must not postpone the rebuild past the cap (waited {elapsed:?})"
        );
        assert!(
            elapsed >= Duration::from_millis(800),
            "the burst should be folded, not abandoned immediately (waited {elapsed:?})"
        );
        feeder.join().unwrap();
    }

    #[test]
    fn watcher_spawn_initiates_successfully() {
        let temp = tempfile::tempdir().unwrap();
        let project = Project::for_tests(temp.path().join("docs"));
        std::fs::create_dir_all(&project.docs_dir).unwrap();
        let state = Arc::new(AppState::new(project));
        let res = spawn(state);
        assert!(res.is_ok());
    }
}
