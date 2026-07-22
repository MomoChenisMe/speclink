//! Client-side SSE consumption for a project's `/events` stream (blueprint
//! §9.1/§9.2): a blocking, abortable subscription that yields typed events —
//! invalidation hints and the reset signal. Push is a pointer, never a
//! payload: consumers re-read the canon through Query + ETag, so this module
//! carries no document content and no caching.
//!
//! The stream deliberately does not reuse [`crate::client::Client`]'s agent:
//! that agent carries an overall request timeout, which would sever a
//! long-lived stream. This agent has no overall timeout; instead a short
//! socket read timeout doubles as the abort poll — a blocked read wakes up
//! within [`ABORT_POLL`] to observe the abort flag.

use crate::{translate_protocol_error, translate_transport, RemoteError};
use speclink_protocol::events::InvalidationEvent;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// How long a blocked read waits before waking to check the abort flag. Also
/// the socket read timeout, so a dead connection is only silent this long
/// between heartbeats.
const ABORT_POLL: Duration = Duration::from_secs(2);

/// How long a stream may stay completely silent before it is treated as dead.
/// The server sends comment heartbeats at a fixed interval (default 15s), so
/// a healthy connection is never silent this long — but a half-open socket
/// whose FIN/RST never arrived would otherwise block [`EventStream::next`]
/// forever (observed on macOS loopback under CPU starvation). Three missed
/// default heartbeats: generous against scheduler pauses, still a bounded
/// recovery. A server configured with a heartbeat above this limit only costs
/// a periodic lossless resubscribe (Last-Event-ID continues the sequence).
const STALL_LIMIT: Duration = Duration::from_secs(45);

/// A typed event off the stream. Unknown event names are skipped (a newer
/// server never breaks an older client); convergence is guaranteed by Query +
/// ETag regardless.
#[derive(Debug)]
pub enum RemoteEvent {
    /// A commit invalidation hint — re-read the named resource via Query.
    Invalidate(InvalidationEvent),
    /// The resume cursor expired server-side — do a full re-read, then keep
    /// consuming from the new position.
    Reset,
}

/// Ends a stream's blocking read from another thread: after [`abort`], the
/// next wakeup of [`EventStream::next`] returns `Ok(None)`.
///
/// [`abort`]: AbortHandle::abort
#[derive(Clone)]
pub struct AbortHandle(Arc<AtomicBool>);

impl AbortHandle {
    pub fn abort(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

/// An open `/events` subscription. [`next`](EventStream::next) blocks until a
/// typed event arrives, the stream ends, or the abort handle fires.
pub struct EventStream {
    reader: Box<dyn Read + Send + Sync + 'static>,
    aborted: Arc<AtomicBool>,
    /// Bytes received but not yet split into lines.
    pending: Vec<u8>,
    /// The accumulating event's `event:` name and `data:` lines.
    event: Option<String>,
    data: Vec<String>,
    have_fields: bool,
    /// When the last byte (heartbeats included) arrived — the stall clock.
    last_activity: std::time::Instant,
}

/// Open a subscription to `{base_url}/events`. `base_url`, `token`, and
/// `repo` are the same triple [`crate::client::Client::new`] takes; a
/// `last_event_id` resumes after the given outbox sequence. Fails closed on
/// any non-2xx precondition (same translation as every request).
pub fn subscribe(
    base_url: &str,
    token: &str,
    repo: Option<&str>,
    last_event_id: Option<u64>,
) -> Result<EventStream, RemoteError> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(ABORT_POLL)
        .build();
    let mut req = agent
        .get(&format!("{}/events", base_url.trim_end_matches('/')))
        .set("Authorization", &format!("Bearer {token}"))
        .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
        .set("Accept", "text/event-stream");
    if let Some(repo) = repo {
        req = req.set("X-Speclink-Repo", repo);
    }
    if let Some(id) = last_event_id {
        req = req.set("Last-Event-ID", &id.to_string());
    }
    match req.call() {
        Ok(resp) => Ok(EventStream {
            reader: resp.into_reader(),
            aborted: Arc::new(AtomicBool::new(false)),
            pending: Vec::new(),
            event: None,
            data: Vec::new(),
            have_fields: false,
            last_activity: std::time::Instant::now(),
        }),
        Err(ureq::Error::Status(status, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            Err(translate_protocol_error(status, &body))
        }
        Err(ureq::Error::Transport(_)) => Err(translate_transport()),
    }
}

impl EventStream {
    /// A handle that unblocks [`next`](EventStream::next) from another thread.
    pub fn abort_handle(&self) -> AbortHandle {
        AbortHandle(Arc::clone(&self.aborted))
    }

    /// Block until the next typed event. `Ok(None)` means the abort handle
    /// fired; a server-side close or a broken connection is an `Err` — the
    /// caller resubscribes with `Last-Event-ID` and converges via Query.
    pub fn next(&mut self) -> Result<Option<RemoteEvent>, RemoteError> {
        let mut chunk = [0u8; 2048];
        loop {
            if self.aborted.load(Ordering::Relaxed) {
                return Ok(None);
            }
            while let Some(line) = self.take_line() {
                if let Some(event) = self.consume_line(&line)? {
                    return Ok(Some(event));
                }
            }
            match self.reader.read(&mut chunk) {
                Ok(0) => {
                    return if self.aborted.load(Ordering::Relaxed) {
                        Ok(None)
                    } else {
                        Err(disconnected())
                    };
                }
                Ok(n) => {
                    self.last_activity = std::time::Instant::now();
                    self.pending.extend_from_slice(&chunk[..n]);
                }
                // A read timeout is the abort poll tick, not a failure — but a
                // stream silent past the stall limit is a dead connection whose
                // teardown never reached us: hand it to the resubscribe path.
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    if self.last_activity.elapsed() >= STALL_LIMIT {
                        return Err(stalled());
                    }
                }
                Err(_) => {
                    return if self.aborted.load(Ordering::Relaxed) {
                        Ok(None)
                    } else {
                        Err(disconnected())
                    };
                }
            }
        }
    }

    /// The next complete line out of `pending`, without its terminator.
    fn take_line(&mut self) -> Option<String> {
        let nl = self.pending.iter().position(|&b| b == b'\n')?;
        let mut line: Vec<u8> = self.pending.drain(..=nl).collect();
        line.pop(); // the \n
        if line.last() == Some(&b'\r') {
            line.pop();
        }
        Some(String::from_utf8_lossy(&line).into_owned())
    }

    /// Feed one SSE line into the accumulator; a blank line dispatches the
    /// accumulated event. Comments (heartbeats) and the `id:`/`retry:` fields
    /// are consumed silently — the invalidation DTO's `eventId` mirrors the
    /// SSE id, so the typed event already carries the resume cursor.
    fn consume_line(&mut self, line: &str) -> Result<Option<RemoteEvent>, RemoteError> {
        if line.is_empty() {
            if !self.have_fields {
                return Ok(None);
            }
            let name = self.event.take();
            let data = self.data.join("\n");
            self.data.clear();
            self.have_fields = false;
            return match name.as_deref() {
                Some("invalidate") => {
                    let hint: InvalidationEvent =
                        serde_json::from_str(&data).map_err(|_| RemoteError {
                            message: "unexpected server response — the event stream carried an unreadable invalidation".into(),
                            reason: None,
                            status: None,
                        })?;
                    Ok(Some(RemoteEvent::Invalidate(hint)))
                }
                Some("reset") => Ok(Some(RemoteEvent::Reset)),
                // Unknown event names are over-read tolerance, not failures.
                _ => Ok(None),
            };
        }
        if line.starts_with(':') {
            return Ok(None); // heartbeat comment
        }
        let (key, value) = match line.split_once(':') {
            Some((k, v)) => (k, v.strip_prefix(' ').unwrap_or(v)),
            None => (line, ""),
        };
        self.have_fields = true;
        match key {
            "event" => self.event = Some(value.to_string()),
            "data" => self.data.push(value.to_string()),
            _ => {}
        }
        Ok(None)
    }
}

/// Read the scope's current `/sync-state` ETag — the polling convergence
/// bedrock (§9.2): after a missed or dropped push, a differing ETag is the
/// signal to re-read the canon via Query. Same triple as [`subscribe`].
pub fn sync_state(
    base_url: &str,
    token: &str,
    repo: Option<&str>,
) -> Result<String, RemoteError> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(10))
        .build();
    let mut req = agent
        .get(&format!("{}/sync-state", base_url.trim_end_matches('/')))
        .set("Authorization", &format!("Bearer {token}"))
        .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION);
    if let Some(repo) = repo {
        req = req.set("X-Speclink-Repo", repo);
    }
    match req.call() {
        Ok(resp) => Ok(resp.header("etag").unwrap_or_default().to_string()),
        Err(ureq::Error::Status(status, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            Err(translate_protocol_error(status, &body))
        }
        Err(ureq::Error::Transport(_)) => Err(translate_transport()),
    }
}

/// The stream ended without an abort: the server closed it or the connection
/// broke. One semantic line, mirroring the crate's translation rule.
fn disconnected() -> RemoteError {
    RemoteError {
        message: "event stream disconnected — resubscribe and converge via sync-state".into(),
        reason: None,
        status: None,
    }
}

/// The stream stayed silent past [`STALL_LIMIT`]: the connection is presumed
/// half-open and dead. Same recovery path as a clean disconnect.
fn stalled() -> RemoteError {
    RemoteError {
        message: "event stream stalled — no heartbeat within the stall limit; resubscribe and converge via sync-state".into(),
        reason: None,
        status: None,
    }
}
