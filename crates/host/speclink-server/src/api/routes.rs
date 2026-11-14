//! The project route the demo server keeps for itself: the event stream.
//! Every other project route is the host kit's HTTP door (`forward`).

use crate::auth::Binding;
use crate::error::ApiError;
use crate::events::Subscription;
use crate::state::AppState;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use speclink_protocol::events::InvalidationEvent;
use speclink_store::{ProjectId, RepoId, Scope};
use std::convert::Infallible;
use tokio::sync::broadcast::error::RecvError;

/// The store scope a binding addresses.
fn scope_of(binding: &Binding) -> Scope {
    Scope::new(
        ProjectId::new(binding.project.key.clone()),
        RepoId::new(binding.repo.clone()),
    )
}

/// `GET /events` — the project-scoped SSE invalidation stream (server-event-stream
/// spec). The `Binding` extractor has already run the same bearer/binding
/// precondition as every route, so an unauthenticated or non-member request
/// never reaches here. A `Last-Event-ID` below the cleaned floor gets a reset
/// frame first; a resumable one backfills the gap; an idle stream is kept alive
/// by comment heartbeats. Each hint carries only the outbox sequence, scope,
/// resource id, and revision — never document content.
pub async fn events(
    State(state): State<AppState>,
    binding: Binding,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let last_event_id = headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse::<u64>().ok());
    let heartbeat = state.events.settings().heartbeat;
    let scope = scope_of(&binding);
    let Subscription { mut rx, plan } = state.events.subscribe(&scope, last_event_id)?;

    let stream = async_stream::stream! {
        if plan.reset {
            yield Ok::<Event, Infallible>(reset_event());
        }
        // Everything at or below `last` is already delivered (backfill or
        // history), so the live tail skips it — no gap, no repeat.
        let mut last = plan.cursor;
        for hint in plan.backfill {
            if let Ok(seq) = hint.event_id.parse::<u64>() {
                last = seq;
            }
            yield Ok(invalidation_event(&hint));
        }
        loop {
            match rx.recv().await {
                Ok(item) => {
                    if item.seq > last {
                        last = item.seq;
                        yield Ok(invalidation_event(&item.event));
                    }
                }
                // A slow consumer that overflowed its buffer is dropped; it
                // reconnects with Last-Event-ID and resumes.
                Err(RecvError::Lagged(_)) | Err(RecvError::Closed) => break,
            }
        }
    };

    Ok(Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(heartbeat).text("heartbeat"))
        .into_response())
}

/// One invalidation hint as an SSE event: `id` is the outbox sequence (so a
/// client's `Last-Event-ID` resumes from it), the event type is `invalidate`,
/// and the data is the DTO JSON.
fn invalidation_event(hint: &InvalidationEvent) -> Event {
    Event::default()
        .id(hint.event_id.clone())
        .event("invalidate")
        .json_data(hint)
        .unwrap_or_else(|_| Event::default().comment("invalidation serialize failed"))
}

/// The reset signal — a distinct SSE event type telling the client its cursor
/// is cleaned; it converges by re-reading through Query + ETag.
fn reset_event() -> Event {
    Event::default()
        .event("reset")
        .data("cursor expired; re-read via query")
}
