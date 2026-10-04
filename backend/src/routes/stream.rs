use std::convert::Infallible;
use std::time::Duration;

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::stream::Stream;
use tokio::sync::broadcast::error::RecvError;

use crate::app_state::{AppState, UserRole};
use crate::errors::{ApiError, ApiResult};

/// `GET /api/sessions/{sessionId}/events?role=<role>` — the live update signal.
///
/// The event body is only a revision number; each client re-reads `GET /state` for
/// its own role, so redaction happens in exactly one place on the server
/// (contracts/realtime.md).
pub async fn events(
    State(state): State<AppState>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
    axum::extract::Query(query): axum::extract::Query<EventsQuery>,
) -> ApiResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    // The role is validated here so a bad role fails before the stream opens; the
    // redaction it implies happens later, in GET /state.
    UserRole::parse(&query.role)
        .ok_or_else(|| ApiError::not_found(format!("Unknown role '{}'.", query.role)))?;
    state.require_session(&session_id)?;

    let mut receiver = state.hub.subscribe();
    let session = session_id.clone();
    let app = state.clone();
    let start_revision = state.current_revision(&session_id);

    let signal_stream = async_stream::stream! {
        // The first frame lets a client that connected late settle on the current
        // revision before it starts listening for changes.
        yield Ok(signal_event("state", start_revision));
        let mut last_revision: i64 = start_revision;
        loop {
            match receiver.recv().await {
                Ok(update) if update.session_id == session => {
                    if last_revision != update.revision {
                        last_revision = update.revision;
                        yield Ok(signal_event("state", update.revision));
                    }
                }
                Ok(_) => {}
                Err(RecvError::Lagged(_)) => {
                    // A slow client missed updates; it will refetch on the next
                    // signal, so ask it to read state now.
                    last_revision = app.current_revision(&session);
                    yield Ok(signal_event("state", last_revision));
                }
                Err(RecvError::Closed) => break,
            }
            // Every frame carries a revision, heartbeats included, so a client can
            // skip the refetch whenever nothing actually moved (realtime.md).
            yield Ok(signal_event("heartbeat", last_revision));
        }
    };

    // The keep-alive writes an SSE comment; it keeps proxies from closing an idle
    // stream without putting a second, unversioned frame on the wire.
    Ok(Sse::new(signal_stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}

fn signal_event(name: &str, revision: i64) -> Event {
    Event::default()
        .event(name)
        .data(format!("{{\"revision\":{revision}}}"))
}

#[derive(serde::Deserialize)]
pub struct EventsQuery {
    pub role: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::apply_schema;
    use crate::routes::api::router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use futures_util::StreamExt;
    use rusqlite::Connection;
    use std::path::PathBuf;
    use std::pin::Pin;
    use tower::ServiceExt;

    fn test_state() -> AppState {
        let conn = Connection::open_in_memory().unwrap();
        apply_schema(&conn).unwrap();
        AppState::new(conn, PathBuf::from("does-not-exist"))
    }

    #[tokio::test]
    async fn an_unknown_session_is_refused_before_the_stream_opens() {
        let response = router(test_state())
            .oneshot(
                Request::get("/api/sessions/nope/events?role=p1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn an_unknown_role_is_refused_before_the_stream_opens() {
        let state = test_state();
        state.with_db(|conn| crate::db::insert_test_session(conn, "s1", 3));

        let response = router(state)
            .oneshot(
                Request::get("/api/sessions/s1/events?role=admin")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn the_first_frame_carries_the_revision_the_database_holds() {
        let state = test_state();
        state.with_db(|conn| crate::db::insert_test_session(conn, "s1", 12));

        let mut frames = open_frames(state).await;

        assert_eq!(
            frames.next().await.unwrap(),
            "event: state\ndata: {\"revision\":12}\n\n"
        );
    }

    #[tokio::test]
    async fn a_bump_shows_up_on_an_open_stream() {
        let state = test_state();
        state.with_db(|conn| crate::db::insert_test_session(conn, "s1", 1));

        let mut frames = open_frames(state.clone()).await;
        // The settled frame first, so the client knows where it stands.
        assert_eq!(
            frames.next().await.unwrap(),
            "event: state\ndata: {\"revision\":1}\n\n"
        );

        assert_eq!(state.bump_revision("s1").unwrap(), 2);

        let update = tokio::time::timeout(Duration::from_secs(2), frames.next())
            .await
            .expect("no update frame arrived")
            .expect("stream ended");
        assert_eq!(update, "event: state\ndata: {\"revision\":2}\n\n");
    }

    /// Open the stream through the router and hand back one `String` per SSE
    /// frame, so a test sees exactly what a browser would read.
    async fn open_frames(state: AppState) -> Pin<Box<dyn Stream<Item = String> + Send>> {
        use futures_util::StreamExt;

        let response = router(state)
            .oneshot(
                Request::get("/api/sessions/s1/events?role=p1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let mut bytes = response.into_body().into_data_stream();

        Box::pin(async_stream::stream! {
            let mut pending = String::new();
            while let Some(chunk) = bytes.next().await {
                let Ok(chunk) = chunk else { break };
                pending.push_str(&String::from_utf8_lossy(&chunk));
                while let Some(pos) = pending.find("\n\n") {
                    let frame: String = pending.drain(..pos + 2).collect();
                    yield frame;
                }
            }
        })
    }
}
