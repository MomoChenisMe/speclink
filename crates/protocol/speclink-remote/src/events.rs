//! Client-side SSE consumption for a project's event stream (blueprint
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

use crate::{translate_protocol_error, RemoteError};
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
/// recovery. A stream whose heartbeat is not shorter than this limit only
/// costs a periodic lossless resubscribe (Last-Event-ID continues the
/// sequence); the demonstration server refuses such a setting at startup.
const STALL_LIMIT: Duration = Duration::from_secs(speclink_protocol::events::STALL_LIMIT_SECS);

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

/// An open event-stream subscription. [`next`](EventStream::next) blocks until a
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

/// A project's event-stream URL as [`stream_url`] resolved it: it has the
/// project's origin, so the access token [`subscribe`] sends goes to the
/// project's Server only. [`stream_url`] is the only way to make one. The
/// text is the parsed URL in its normal form (host in lower case, no default
/// port, no `.` or `..` segment) — the very parse result the origin check saw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamUrl(String);

impl StreamUrl {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The full URL of a project's event stream, from the url the handshake
/// declares for its sse transport. `project_base` is the project's API
/// address (`{origin}/api/speclink/v1/projects/{project}`).
///
/// - A url that starts with `/` is a path under the project's API address,
///   as the polling url is: `/events` gives `{project_base}/events`.
/// - A full `http://` or `https://` url is used only when its origin (scheme,
///   host, port) is the project's.
/// - Any other url — empty, a path without the leading `/`, another scheme,
///   or one that does not parse — gives `None`: the client then polls.
///
/// Sharp edges, as audited:
///
/// - The access token goes to the Server's origin only. The check compares
///   the parsed scheme, host and port (the `url` crate, the parser ureq uses
///   for the request), never the text: in `https://specs.example.com@evil.example`
///   the host is evil.example, and a path url is checked after it is joined.
///   The result is that parsed URL, not the declared text: the host the check
///   saw is the host [`subscribe`] connects to, whichever parser the HTTP
///   client uses. ureq drops `Authorization` on a redirect (its default), so
///   a redirect does not carry the token away either.
/// - A declaration that does not parse is `None`, never a guess. Swapped
///   arguments give a URL of the same origin or `None`, never another host.
/// - [`subscribe`] takes a [`StreamUrl`], which only this function makes: a
///   caller cannot subscribe to a declared url that skipped the check.
///
/// What the events themselves carry is audited at `event_payload` in
/// `speclink-host`.
pub fn stream_url(project_base: &str, declared: &str) -> Option<StreamUrl> {
    let base = url::Url::parse(project_base).ok()?;
    let stream = if declared.starts_with('/') {
        format!("{}{declared}", project_base.trim_end_matches('/'))
    } else {
        declared.to_string()
    };
    let parsed = url::Url::parse(&stream).ok()?;
    let web = |url: &url::Url| matches!(url.scheme(), "http" | "https");
    (web(&base) && web(&parsed) && parsed.origin() == base.origin()).then(|| StreamUrl(parsed.into()))
}

/// Open a subscription to the event stream at `stream`, the full URL
/// [`stream_url`] resolves. `token` and `repo` are what
/// [`crate::client::Client::new`] takes; a `last_event_id` resumes after the
/// given outbox sequence. Fails closed on any non-2xx precondition (same
/// translation as every request).
pub fn subscribe(
    stream: &StreamUrl,
    token: &str,
    repo: Option<&str>,
    last_event_id: Option<u64>,
) -> Result<EventStream, RemoteError> {
    let agent = crate::http::agent_builder()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(ABORT_POLL)
        .build();
    let mut req = agent
        .get(stream.as_str())
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
        Err(ureq::Error::Transport(error)) => Err(crate::http::translate_transport_error(&error)),
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
                            evidence: None,
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
    let agent = crate::http::agent_builder()
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
        Err(ureq::Error::Transport(error)) => Err(crate::http::translate_transport_error(&error)),
    }
}

/// The stream ended without an abort: the server closed it or the connection
/// broke. One semantic line, mirroring the crate's translation rule.
fn disconnected() -> RemoteError {
    RemoteError {
        message: "event stream disconnected — resubscribe and converge via sync-state".into(),
        reason: None,
        status: None,
        evidence: None,
    }
}

/// The stream stayed silent past [`STALL_LIMIT`]: the connection is presumed
/// half-open and dead. Same recovery path as a clean disconnect.
fn stalled() -> RemoteError {
    RemoteError {
        message: "event stream stalled — no heartbeat within the stall limit; resubscribe and converge via sync-state".into(),
        reason: None,
        status: None,
        evidence: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{EventStream, STALL_LIMIT};
    use std::io::{Error, ErrorKind, Read};
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    const BASE: &str = "https://specs.example.com/api/speclink/v1/projects/acme";

    /// A connection that sends nothing: one read timeout, then the peer closes.
    struct Silent {
        timed_out: bool,
    }

    impl Read for Silent {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            if std::mem::replace(&mut self.timed_out, true) {
                return Ok(0);
            }
            Err(Error::from(ErrorKind::TimedOut))
        }
    }

    /// A stream that got its last byte `silent_for` ago and gets nothing more.
    fn silent_stream(silent_for: Duration) -> EventStream {
        EventStream {
            reader: Box::new(Silent { timed_out: false }),
            aborted: Arc::new(AtomicBool::new(false)),
            pending: Vec::new(),
            event: None,
            data: Vec::new(),
            have_fields: false,
            last_activity: Instant::now().checked_sub(silent_for).expect("a clock past the stall limit"),
        }
    }

    #[test]
    fn a_stream_silent_for_45_seconds_is_treated_as_broken() {
        // Spec server-event-stream「超過 45 秒沒有資料時用戶端重連」：45 秒是文件寫明
        // 的對外數字，改它要連文件一起改。
        assert_eq!(STALL_LIMIT, Duration::from_secs(45));
        // One source for the number: a stream's heartbeat is checked against the same constant.
        assert_eq!(STALL_LIMIT, Duration::from_secs(speclink_protocol::events::STALL_LIMIT_SECS));

        let err = silent_stream(STALL_LIMIT).next().expect_err("a stalled stream is an error");
        assert!(err.message.starts_with("event stream stalled"), "{}", err.message);

        // Under the limit a read timeout is only the abort poll: the stream
        // goes on reading, and here meets the peer's close.
        let err = silent_stream(STALL_LIMIT - Duration::from_secs(5)).next().expect_err("the peer closed");
        assert!(err.message.starts_with("event stream disconnected"), "{}", err.message);
    }

    /// [`super::stream_url`]'s answer as text.
    fn resolved(project_base: &str, declared: &str) -> Option<String> {
        super::stream_url(project_base, declared).map(|url| url.as_str().to_string())
    }

    #[test]
    fn a_declared_url_resolves_against_the_project_or_is_refused() {
        // Spec remote-workspace-data Example「宣告的 url 與實際訂閱的網址」的網址列
        // （「只宣告 websocket」那列沒有 url，由 Desktop 的握手測試涵蓋）。
        let rows: [(&str, Option<&str>); 7] = [
            ("/events", Some("https://specs.example.com/api/speclink/v1/projects/acme/events")),
            ("/stream/changes", Some("https://specs.example.com/api/speclink/v1/projects/acme/stream/changes")),
            ("https://specs.example.com/push/acme", Some("https://specs.example.com/push/acme")),
            ("https://SPECS.Example.com:443/push/acme", Some("https://specs.example.com/push/acme")),
            ("https://push.example.com/acme", None),
            ("http://specs.example.com/push/acme", None),
            ("https://specs.example.com:8443/push", None),
        ];
        for (declared, expected) in rows {
            assert_eq!(resolved(BASE, declared).as_deref(), expected, "{declared}");
        }
    }

    #[test]
    fn the_url_to_connect_to_is_the_parsed_one_in_its_normal_form() {
        // The check and the connection use one parse result: what `subscribe`
        // gets is that result, never the declared text parsed a second time.
        // A trailing slash on the project's API address adds no empty segment.
        assert_eq!(
            resolved(&format!("{BASE}/"), "/events").as_deref(),
            Some("https://specs.example.com/api/speclink/v1/projects/acme/events")
        );
        // The default port written or not is the same origin, both ways, and
        // the normal form leaves it out.
        assert_eq!(resolved(BASE, "https://specs.example.com:443/push").as_deref(), Some("https://specs.example.com/push"));
        assert_eq!(
            resolved("https://specs.example.com:443/api/speclink/v1/projects/acme", "https://specs.example.com/push")
                .as_deref(),
            Some("https://specs.example.com/push")
        );
        // A host that differs only in case is the same origin, in lower case.
        assert_eq!(
            resolved(BASE, "https://SPECS.Example.com/push/acme").as_deref(),
            Some("https://specs.example.com/push/acme")
        );
        // `.` and `..` in a path are resolved.
        assert_eq!(
            resolved(BASE, "/a/../events").as_deref(),
            Some("https://specs.example.com/api/speclink/v1/projects/acme/events")
        );
        // The text before `@` is a user name: the host is evil.example.
        assert_eq!(resolved(BASE, "https://specs.example.com@evil.example/x"), None);
    }

    #[test]
    fn an_empty_or_unrooted_url_is_refused() {
        for declared in ["", "events", "stream/changes", "ftp://specs.example.com/events", "https://"] {
            assert_eq!(resolved(BASE, declared), None, "{declared:?}");
        }
    }
}
