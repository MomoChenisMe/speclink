//! The project route the demo server keeps for itself: the event stream.
//! Every other project route is the host kit's HTTP door (`forward`).

use crate::auth::Binding;
use crate::error::ApiError;
use crate::events::{EventSettings, Subscription};
use crate::identity::IdentityStore;
use crate::state::AppState;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use speclink_host::scope::{scope_id, ProjectId, RepoId};
use speclink_protocol::events::InvalidationEvent;
use speclink_store_port::{PortError, ScopeId};
use std::convert::Infallible;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::MissedTickBehavior;

/// The store scope a binding addresses.
fn scope_of(binding: &Binding) -> Result<ScopeId, PortError> {
    scope_id(&ProjectId::new(binding.project.key.clone()), &RepoId::new(binding.repo.clone()))
}

/// `GET /events` — the project-scoped SSE invalidation stream (server-event-stream
/// spec). The `Binding` extractor has already run the same bearer/binding
/// precondition as every route, so an unauthenticated or non-member request
/// never reaches here. A `Last-Event-ID` below the cleaned floor, or above the
/// newest sequence, gets a reset frame first; a resumable one backfills the
/// gap; an idle stream is kept alive by comment heartbeats. Each hint carries
/// only the event sequence, scope, resource id, and revision — never document
/// content.
///
/// The precondition ran once, when the stream opened. So once per heartbeat
/// interval the stream asks the identity store again whether its subscriber
/// is still an active account and still a member of the project, and ends
/// when it is not: the client then subscribes again and gets the refusal the
/// precondition gives. The check does not look at the access token's expiry —
/// a token that ages out is replaced by the refresh flow, and the stream
/// stays.
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
    let scope = scope_of(&binding)?;
    let (user_id, project_key) = (binding.actor.id, binding.project.key);
    let Subscription { mut rx, plan } =
        state.blocking(move |state| state.events.subscribe(&scope, last_event_id)).await?;

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
        // The first check is one interval after the stream opened.
        let every = recheck_period(heartbeat);
        let mut recheck = tokio::time::interval_at(tokio::time::Instant::now() + every, every);
        recheck.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            // `None` is the tick of the check.
            let received = tokio::select! {
                received = rx.recv() => Some(received),
                _ = recheck.tick() => None,
            };
            match received {
                Some(Ok(item)) => {
                    if item.seq > last {
                        last = item.seq;
                        yield Ok(invalidation_event(&item.event));
                    }
                }
                // A slow consumer that overflowed its buffer is dropped; it
                // reconnects with Last-Event-ID and resumes.
                Some(Err(RecvError::Lagged(_))) | Some(Err(RecvError::Closed)) => break,
                None => {
                    let (user_id, project_key) = (user_id.clone(), project_key.clone());
                    let keeps = state
                        .blocking(move |state| subscriber_still_allowed(state.identity.as_ref(), &user_id, &project_key))
                        .await;
                    if !keeps {
                        break;
                    }
                }
            }
        }
    };

    Ok(Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(heartbeat).text("heartbeat"))
        .into_response())
}

/// How often a stream checks its subscriber: once per heartbeat interval. A
/// heartbeat of zero gives no interval to run at (and a timer refuses a zero
/// period), so the check then runs at the default heartbeat interval.
fn recheck_period(heartbeat: std::time::Duration) -> std::time::Duration {
    if heartbeat.is_zero() {
        EventSettings::default().heartbeat
    } else {
        heartbeat
    }
}

/// Ask the identity store whether `user_id` is still an active account and
/// still a member of `project_key`.
fn subscriber_still_allowed(identity: &dyn IdentityStore, user_id: &str, project_key: &str) -> bool {
    let active = identity.get_user(user_id).map(|user| user.is_some_and(|user| user.active));
    let member = identity.is_member(user_id, project_key);
    subscriber_keeps_the_stream(active, member)
}

/// The decision of the check: the stream ends only when the identity store
/// says "no" to one of the two questions. A store that cannot be read ends
/// nothing — a short storage problem must not make every stream reconnect —
/// and the next interval asks again.
fn subscriber_keeps_the_stream<E>(active: Result<bool, E>, member: Result<bool, E>) -> bool {
    active.unwrap_or(true) && member.unwrap_or(true)
}

/// One invalidation hint as an SSE event: `id` is the event sequence (so a
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
/// cannot resume; it converges by re-reading through Query + ETag.
fn reset_event() -> Event {
    Event::default()
        .event("reset")
        .data("cursor expired; re-read via query")
}

#[cfg(test)]
mod tests {
    use super::subscriber_keeps_the_stream;

    /// spec server-event-stream「使用者失去權限時關閉串流」: the stream ends only
    /// on a "no". An identity store that cannot answer is not a "no".
    #[test]
    fn only_a_no_from_the_identity_store_ends_the_stream() {
        let unreadable = || Err::<bool, &str>("identity store unavailable");

        assert!(subscriber_keeps_the_stream(Ok::<_, &str>(true), Ok(true)), "an active member");
        assert!(!subscriber_keeps_the_stream(Ok::<_, &str>(false), Ok(true)), "a suspended account");
        assert!(!subscriber_keeps_the_stream(Ok::<_, &str>(true), Ok(false)), "no longer a member");

        assert!(subscriber_keeps_the_stream(unreadable(), unreadable()), "the store answers nothing");
        assert!(subscriber_keeps_the_stream(unreadable(), Ok(true)));
        assert!(subscriber_keeps_the_stream(Ok(true), unreadable()));
        // One readable "no" is enough, whatever the other read did.
        assert!(!subscriber_keeps_the_stream(unreadable(), Ok(false)));
        assert!(!subscriber_keeps_the_stream(Ok(false), unreadable()));
    }
}
