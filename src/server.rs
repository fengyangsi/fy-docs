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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Project;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn poll_endpoint_serves_script() {
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
        let app = router(&state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/_poll.js")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/javascript"
        );
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(std::str::from_utf8(&bytes).unwrap(), assets::POLL_REAL);

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
