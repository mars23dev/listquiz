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
/// How often an idle stream emits a named `heartbeat` frame. The contract wants a
/// heartbeat event, not only an SSE comment, so a client can prove its connection
/// is alive and confirm the revision it is holding.
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);

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

    let heartbeat = state.heartbeat_interval;
    let signal_stream = signal_stream(state.clone(), session_id, state.hub.subscribe(), heartbeat);

    // The keep-alive writes an SSE comment as well, for proxies that close a
    // connection with no bytes on it at all.
    Ok(Sse::new(signal_stream).keep_alive(KeepAlive::new().interval(heartbeat)))
}

fn signal_stream(
    state: AppState,
    session_id: String,
    mut receiver: tokio::sync::broadcast::Receiver<crate::app_state::Update>,
    heartbeat: Duration,
) -> impl Stream<Item = Result<Event, Infallible>> {
    async_stream::stream! {
        // The first frame lets a client that connected late settle on the current
        // revision before it starts listening for changes.
        let mut last_revision: i64 = state.current_revision(&session_id);
        yield Ok(signal_event("state", last_revision));

        let mut ticker = tokio::time::interval(heartbeat);
        // The first tick is immediate; the opening `state` frame already covers it.
        ticker.tick().await;

        loop {
            tokio::select! {
                received = receiver.recv() => {
                    match received {
                        Ok(update) if update.session_id == session_id => {
                            if last_revision != update.revision {
                                last_revision = update.revision;
                                yield Ok(signal_event("state", update.revision));
                            }
                        }
                        Ok(_) => {}
                        Err(RecvError::Lagged(_)) => {
                            // A slow client missed updates; it refetches on the next
                            // signal, so ask it to read state now.
                            last_revision = state.current_revision(&session_id);
                            yield Ok(signal_event("state", last_revision));
                        }
                        Err(RecvError::Closed) => break,
                    }
                }
                _ = ticker.tick() => {
                    // Every frame carries a revision, heartbeats included, so a client
                    // can skip the refetch whenever nothing moved (realtime.md).
                    yield Ok(signal_event("heartbeat", last_revision));
                }
            }
        }
    }
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
            .with_heartbeat_interval(Duration::from_millis(20))
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

    #[tokio::test]
    async fn an_idle_stream_still_sends_named_heartbeats() {
        let state = test_state();
        state.with_db(|conn| crate::db::insert_test_session(conn, "s1", 7));

        let mut frames = open_frames(state.clone()).await;

        assert_eq!(
            frames.next().await.unwrap(),
            "event: state\ndata: {\"revision\":7}\n\n"
        );
        for _ in 0..2 {
            let frame = tokio::time::timeout(Duration::from_secs(2), frames.next())
                .await
                .expect("no heartbeat arrived")
                .expect("stream ended");
            assert_eq!(frame, "event: heartbeat\ndata: {\"revision\":7}\n\n");
        }
    }

    #[tokio::test]
    async fn a_heartbeat_repeats_the_revision_a_client_already_holds() {
        let state = test_state();
        state.with_db(|conn| crate::db::insert_test_session(conn, "s1", 3));

        let mut frames = open_frames(state.clone()).await;
        assert_eq!(
            frames.next().await.unwrap(),
            "event: state\ndata: {\"revision\":3}\n\n"
        );

        assert_eq!(state.bump_revision("s1").unwrap(), 4);
        assert_eq!(
            frames.next().await.unwrap(),
            "event: state\ndata: {\"revision\":4}\n\n"
        );
        assert_eq!(
            frames.next().await.unwrap(),
            "event: heartbeat\ndata: {\"revision\":4}\n\n"
        );
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
