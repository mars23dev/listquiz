use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use axum::Json;
use axum::Router;
use axum::http::{StatusCode, Uri};
use axum::response::Redirect;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde::Serialize;
use tower_http::services::{ServeDir, ServeFile};

use crate::app_state::AppState;
use crate::errors::ApiError;

#[derive(Serialize)]
struct Health {
    status: &'static str,
    version: &'static str,
}

async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn api_not_found(uri: Uri) -> Response {
    ApiError::not_found(format!("No API route matches {}.", uri.path())).into_response()
}

/// `GET /{sessionId}` — a bare role link means "player one", so the server
/// sends the phone to the player view instead of leaving the reader guessing.
async fn session_redirect(
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> Redirect {
    Redirect::temporary(&format!("/{session_id}/p1"))
}

/// Assemble the whole application: `/api` first, then the built frontend, then the
/// history-API fallback so a deep link like `/abc123/p1` serves `index.html`.
pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(health))
        .route(
            "/sessions/{session_id}/events",
            get(crate::routes::stream::events),
        )
        .fallback(api_not_found);
    let api = crate::routes::sessions::mount(api);
    let api = crate::routes::folders::mount(api);
    let api = crate::routes::lists::mount(api);
    let api = api.with_state(state.clone());

    let index = state.dist_dir.join("index.html");
    let frontend = ServeDir::new(state.dist_dir.clone())
        .not_found_service(ServeFile::new(index.clone()))
        .fallback(ServeFile::new(index));

    Router::new()
        .nest("/api", api)
        .route("/{session_id}", get(session_redirect))
        .fallback_service(frontend)
        .with_state(state)
}

pub async fn serve(
    state: AppState,
    addr: SocketAddr,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router(state))
        .with_graceful_shutdown(shutdown)
        .await
}

/// True when the frontend has been built into the backend's expected location.
pub fn frontend_built(dist_dir: &Path) -> bool {
    dist_dir.join("index.html").is_file()
}

/// Message shown at startup when the frontend is missing, with the command that
/// fixes it.
pub fn missing_frontend_notice(dist_dir: &Path) -> String {
    format!(
        "Frontend not found at {}.\nRun: npm --prefix frontend install && npm --prefix frontend run build",
        dist_dir.display()
    )
}

pub fn not_ready_response() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "error": { "code": "frontend_missing", "message": missing_frontend_notice(&default_dist_dir()) }
        })),
    )
        .into_response()
}

/// The backend owns the built frontend. In development it is beside the source;
/// in a release zip it is beside the executable.
pub fn default_dist_dir() -> PathBuf {
    crate::paths::default_dist_dir()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use rusqlite::Connection;
    use tower::ServiceExt;

    fn test_state() -> AppState {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::apply_schema(&conn).unwrap();
        AppState::new(conn, PathBuf::from("does-not-exist"))
    }

    #[tokio::test]
    async fn health_reports_ok() {
        let response = router(test_state())
            .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn an_unknown_api_route_answers_json_not_found() {
        let response = router(test_state())
            .oneshot(Request::get("/api/nope").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn a_bare_session_link_redirects_to_the_first_player() {
        let response = router(test_state())
            .oneshot(Request::get("/abc123").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(response.headers()["location"], "/abc123/p1");
    }

    #[test]
    fn the_expected_dist_directory_is_inside_the_project() {
        let dist = default_dist_dir();

        assert!(dist.ends_with("frontend/dist"));
        assert!(
            !dist.to_string_lossy().to_lowercase().contains("appdata"),
            "state must never land under %APPDATA%"
        );
    }
}
