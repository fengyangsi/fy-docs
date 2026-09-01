//! Axum router: serves the generated `docs/target/` directory and pushes
//! live-reload notifications over the `/events` SSE stream.

use crate::state::AppState;
use axum::Router;
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::routing::get;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::WatchStream;
use tower_http::services::ServeDir;

pub(crate) fn router(state: &Arc<AppState>) -> Router {
    let events = state.subscribe();
    Router::new()
        .route(
            "/events",
            get(move || {
                // WatchStream::new emits the channel's current value once when
                // a page subscribes (that frame is live.js's baseline) and then
                // every subsequent bump, so the first save after opening a page
                // reloads it immediately. from_changes would skip that baseline
                // and leave the page one rebuild behind.
                let live = WatchStream::new(events.clone())
                    .map(|id| Ok::<_, Infallible>(SseEvent::default().data(id.to_string())));
                async move { Sse::new(live).keep_alive(KeepAlive::default()) }
            }),
        )
        .fallback_service(ServeDir::new(&state.project.target_dir))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Project;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn events_endpoint_streams_baseline_then_rebuilds() {
        let temp = tempfile::tempdir().unwrap();
        let project = Project::for_tests(temp.path().join("docs"));
        let state = Arc::new(AppState::new(project));
        let app = router(&state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/events")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::CONTENT_TYPE)
                .unwrap(),
            "text/event-stream"
        );
        // WatchStream::new sends the channel's current value as the first
        // frame: that baseline is what live.js compares every later rebuild
        // against, so the very first save triggers a reload.
        let mut body = response.into_body();
        let baseline = body.frame().await.unwrap().unwrap().into_data().unwrap();
        assert!(
            std::str::from_utf8(&baseline).unwrap().contains("data: 1"),
            "the first frame must carry the current build id as baseline"
        );

        // A rebuild pushes the incremented id; live.js reloads on the change.
        state.bump_build();
        let frame = body.frame().await.unwrap().unwrap().into_data().unwrap();
        assert!(std::str::from_utf8(&frame).unwrap().contains("data: 2"));
    }

    #[tokio::test]
    async fn fallback_serves_target_directory() {
        let temp = tempfile::tempdir().unwrap();
        let project = Project::for_tests(temp.path().join("docs"));
        std::fs::create_dir_all(&project.target_dir).unwrap();
        std::fs::write(project.target_dir.join("index.html"), "<p>hello static</p>").unwrap();

        let state = Arc::new(AppState::new(project));
        let app = router(&state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/index.html")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(std::str::from_utf8(&bytes).unwrap(), "<p>hello static</p>");
    }
}
