//! Axum router: serves the generated `docs/target/` directory and injects the
//! live-reload poll script when running as a server (the static `build`
//! output ships a no-op stub instead).

use crate::assets;
use crate::state::AppState;
use axum::Router;
use axum::http::header;
use axum::routing::get;
use std::sync::Arc;
use tower_http::services::ServeDir;

pub fn router(state: &Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/_poll.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript")],
                    assets::POLL_REAL,
                )
            }),
        )
        .fallback_service(ServeDir::new(&state.project.target_dir))
}
