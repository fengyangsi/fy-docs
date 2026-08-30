//! Axum router: serves the generated `docs/target/` directory and pushes
//! live-reload notifications over the `/events` SSE stream.

use crate::state::AppState;
use axum::Router;
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::routing::get;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::wrappers::WatchStream;
use tokio_stream::{StreamExt, iter};
use tower_http::services::ServeDir;

pub fn router(state: &Arc<AppState>) -> Router {
    let events = state.subscribe();
    Router::new()
        .route(
            "/events",
            get(move || {
                // Seed the stream with the current id so a freshly opened
                // page records its baseline before the next rebuild arrives.
                let current = *events.borrow();
                let seed = iter(vec![Ok::<_, Infallible>(
                    SseEvent::default().data(current.to_string()),
                )]);
                let live = WatchStream::new(events.clone())
                    .map(|id| Ok::<_, Infallible>(SseEvent::default().data(id.to_string())));
                async move { Sse::new(seed.chain(live)).keep_alive(KeepAlive::default()) }
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
    async fn events_endpoint_streams_the_current_build_id() {
        let temp = std::env::temp_dir().join(format!("fy-docs-server-test-{}", std::process::id()));
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
        let state = Arc::new(AppState::new(project));
        state.bump_build();
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
        // The seed frame carries the current build id.
        let mut body = response.into_body();
        let frame = body.frame().await.unwrap().unwrap();
        let data = frame.into_data().unwrap();
        assert!(std::str::from_utf8(&data).unwrap().contains("data: 2"));

        let _ = std::fs::remove_dir_all(temp);
    }

    #[tokio::test]
    async fn fallback_serves_target_directory() {
        let temp =
            std::env::temp_dir().join(format!("fy-docs-server-static-{}", std::process::id()));
        std::fs::create_dir_all(&temp).unwrap();
        std::fs::write(temp.join("index.html"), "<p>hello static</p>").unwrap();

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

        let _ = std::fs::remove_dir_all(temp);
    }
}
