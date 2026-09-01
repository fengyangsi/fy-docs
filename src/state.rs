//! Shared state: the project, generation options, and the build-id channel.

use crate::project::Project;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::watch;

/// Options captured at startup; dev-mode rebuilds reuse them so a filtered
/// session (for example `dev --lang zh-CN`) stays filtered after the first
/// save instead of silently rebuilding every language.
#[derive(Debug, Clone, Default)]
pub(crate) struct GenerateOptions {
    /// Also compile the print-edition PDF on every rebuild.
    pub(crate) with_pdf: bool,
    /// Restrict generation to one language target, e.g. `Some("zh-CN")`.
    pub(crate) lang_filter: Option<String>,
}

pub(crate) struct AppState {
    pub(crate) project: Project,
    pub(crate) build_id: AtomicU64,
    /// Startup options reused by the watcher on every rebuild.
    pub(crate) generate: GenerateOptions,
    /// Broadcasts the current build id to `/events` subscribers so open
    /// pages reload themselves after each rebuild.
    events: watch::Sender<u64>,
    /// Holds a receiver for the process lifetime: a watch channel with no
    /// receiver is closed, and `send` would silently drop every update.
    _events_anchor: watch::Receiver<u64>,
}

impl AppState {
    /// Plain constructor with default generation options. Only the tests use
    /// it; production always captures the real CLI options via
    /// [`Self::with_generate`].
    #[cfg(test)]
    pub(crate) fn new(project: Project) -> Self {
        Self::with_generate(project, GenerateOptions::default())
    }

    /// Captures the CLI options so later rebuilds repeat the same generation.
    pub(crate) fn with_generate(project: Project, generate: GenerateOptions) -> Self {
        let (events, _events_anchor) = watch::channel(1);
        Self {
            project,
            build_id: AtomicU64::new(1),
            generate,
            events,
            _events_anchor,
        }
    }

    /// Subscribes to the build-id broadcast (fed into the `/events` stream).
    pub(crate) fn subscribe(&self) -> watch::Receiver<u64> {
        self.events.subscribe()
    }

    pub(crate) fn current_build_id(&self) -> u64 {
        self.build_id.load(Ordering::SeqCst)
    }

    /// Bumps the build id and notifies every `/events` subscriber so open
    /// pages reload themselves.
    pub(crate) fn bump_build(&self) {
        let id = self.build_id.fetch_add(1, Ordering::SeqCst) + 1;
        let _ = self.events.send(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn app_state_tracks_and_broadcasts_build_id() {
        let state = AppState::new(Project::for_tests(PathBuf::new()));
        let events = state.subscribe();
        assert_eq!(state.current_build_id(), 1);
        assert_eq!(*events.borrow(), 1);

        state.bump_build();
        assert_eq!(state.current_build_id(), 2);
        assert_eq!(*events.borrow(), 2);
    }

    #[test]
    fn with_generate_captures_startup_options() {
        let state = AppState::with_generate(
            Project::for_tests(PathBuf::new()),
            GenerateOptions {
                with_pdf: true,
                lang_filter: Some("zh-CN".to_owned()),
            },
        );
        assert!(state.generate.with_pdf);
        assert_eq!(state.generate.lang_filter.as_deref(), Some("zh-CN"));

        // The plain constructor keeps the dev defaults.
        let plain = AppState::new(Project::for_tests(PathBuf::new()));
        assert!(!plain.generate.with_pdf);
        assert!(plain.generate.lang_filter.is_none());
    }
}
