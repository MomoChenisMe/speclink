//! Recording SSE subscriber for chain scenarios (phase2-acceptance spec
//! 「event recovery 伴隨劇本收斂」): accompanies a workflow, records the
//! deduplicated event stream across connections, can be force-disconnected,
//! and reconnects with `Last-Event-ID` (resume) or from the newest sequence
//! (after a reset). The frame parser matches the sse_events knife's proven
//! shape; this type adds the cross-connection accumulation the chain needs.

use std::io::BufRead;
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// One parsed SSE frame: a comment (heartbeat) or a dispatched event.
#[derive(Debug, Clone)]
pub enum Frame {
    Comment(String),
    Event { id: Option<String>, event: Option<String>, data: String },
}

/// One invalidation hint as the recorder keeps it: the outbox sequence (the
/// dedup identity), the resource category/id, and the commit revision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedEvent {
    pub seq: u64,
    pub scope: String,
    pub resource: String,
    pub revision: u64,
}

/// A live SSE connection: a reader thread parses frames into a channel the
/// test pulls with a timeout. Dropping the connection force-disconnects — the
/// reader thread (which owns the socket) exits on the next frame it cannot
/// deliver; `thread` lets the recorder wait for that teardown to complete.
struct Conn {
    rx: mpsc::Receiver<Frame>,
    thread: std::thread::JoinHandle<()>,
}

impl Conn {
    fn open(events_url: &str, token: &str, repo: &str, last_event_id: Option<&str>) -> Result<Conn, u16> {
        let mut req = ureq::get(events_url)
            .set("Authorization", &format!("Bearer {token}"))
            .set("X-Speclink-Api-Version", "1")
            .set("X-Speclink-Repo", repo)
            .set("Accept", "text/event-stream");
        if let Some(id) = last_event_id {
            req = req.set("Last-Event-ID", id);
        }
        match req.call() {
            Ok(resp) => {
                let reader = std::io::BufReader::new(resp.into_reader());
                let (tx, rx) = mpsc::channel();
                let thread = std::thread::spawn(move || parse_frames(reader, tx));
                Ok(Conn { rx, thread })
            }
            Err(ureq::Error::Status(code, _)) => Err(code),
            Err(e) => panic!("SSE transport error: {e}"),
        }
    }

    /// The next dispatched event within `timeout`, skipping heartbeat comments.
    fn next_event(&self, timeout: Duration) -> Option<Frame> {
        let deadline = Instant::now() + timeout;
        while let Ok(frame) = self.rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
            if let Frame::Event { .. } = frame {
                return Some(frame);
            }
        }
        None
    }
}

/// Parse the SSE body line by line, sending each comment and each blank-line
/// terminated event to `tx`. Ends on EOF, a read error, or a dropped receiver.
fn parse_frames<R: BufRead>(mut reader: R, tx: mpsc::Sender<Frame>) {
    let mut id = None;
    let mut event = None;
    let mut data: Vec<String> = Vec::new();
    let mut have_fields = false;
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let text = line.trim_end_matches(['\r', '\n']);
        if text.is_empty() {
            if have_fields {
                let frame = Frame::Event { id: id.take(), event: event.take(), data: data.join("\n") };
                data.clear();
                have_fields = false;
                if tx.send(frame).is_err() {
                    break;
                }
            }
            continue;
        }
        if let Some(comment) = text.strip_prefix(':') {
            if tx.send(Frame::Comment(comment.trim_start().to_string())).is_err() {
                break;
            }
            continue;
        }
        let (key, value) = match text.split_once(':') {
            Some((k, v)) => (k, v.strip_prefix(' ').unwrap_or(v)),
            None => (text, ""),
        };
        have_fields = true;
        match key {
            "id" => id = Some(value.to_string()),
            "event" => event = Some(value.to_string()),
            "data" => data.push(value.to_string()),
            _ => {}
        }
    }
}

/// The accompanying subscriber: accumulates the deduplicated invalidation
/// stream across connections and knows how to resume or start fresh.
pub struct Recorder {
    events_url: String,
    token: String,
    repo: String,
    conn: Option<Conn>,
    /// A dropped connection whose reader thread may still hold the socket
    /// (it exits on the first frame it cannot deliver).
    closing: Option<std::thread::JoinHandle<()>>,
    /// Deduplicated (by outbox seq) events in arrival order.
    pub events: Vec<RecordedEvent>,
}

impl Recorder {
    /// Subscribe fresh (no `Last-Event-ID`): the stream starts at the newest
    /// sequence and pushes only new writes.
    pub fn connect(events_url: &str, token: &str, repo: &str) -> Recorder {
        let conn = Conn::open(events_url, token, repo, None).expect("subscribe /events");
        Recorder {
            events_url: events_url.to_string(),
            token: token.to_string(),
            repo: repo.to_string(),
            conn: Some(conn),
            closing: None,
            events: Vec::new(),
        }
    }

    /// Force-disconnect: the receiving side drops mid-stream, exactly like a
    /// lost client. Recorded history is kept. The reader thread owning the
    /// socket exits on the next frame it cannot deliver — call
    /// [`Recorder::await_teardown`] to wait for the socket to actually close.
    pub fn disconnect(&mut self) {
        if let Some(conn) = self.conn.take() {
            drop(conn.rx);
            self.closing = Some(conn.thread);
        }
    }

    /// Wait until the dropped connection's reader thread has exited (i.e. the
    /// socket is really closed). The exit is triggered by the next frame the
    /// server pushes, so a write must happen (or a heartbeat fire) within
    /// `timeout`.
    pub fn await_teardown(&mut self, timeout: Duration) {
        let Some(thread) = self.closing.take() else { return };
        let deadline = Instant::now() + timeout;
        while !thread.is_finished() {
            assert!(Instant::now() < deadline, "the dropped subscriber socket closes within {timeout:?}");
            std::thread::sleep(Duration::from_millis(20));
        }
        thread.join().expect("reader thread exits cleanly");
    }

    /// Reconnect with `Last-Event-ID` = the last recorded sequence. Returns
    /// true when the server answers with a `reset` first frame (the cursor was
    /// cleaned), false when the resume proceeds with backfill.
    pub fn reconnect_from_last(&mut self) -> bool {
        let last = self.last_seq().expect("resume needs at least one recorded event").to_string();
        let conn = Conn::open(&self.events_url, &self.token, &self.repo, Some(&last))
            .expect("resubscribe /events");
        // A reset, when owed, is the first frame; a backfilled event otherwise.
        // Peek one frame to classify, keeping it when it is a regular event.
        let reset = match conn.next_event(Duration::from_secs(3)) {
            Some(Frame::Event { event, id, data }) => {
                if event.as_deref() == Some("reset") {
                    true
                } else {
                    self.record(Frame::Event { event, id, data });
                    false
                }
            }
            Some(Frame::Comment(_)) => unreachable!("next_event skips comments"),
            None => false,
        };
        self.conn = Some(conn);
        reset
    }

    /// Drop the current connection (if any) and subscribe fresh from the
    /// newest sequence — the post-reset re-subscription.
    pub fn resubscribe_fresh(&mut self) {
        self.conn = Some(
            Conn::open(&self.events_url, &self.token, &self.repo, None).expect("fresh resubscribe"),
        );
    }

    /// Pull events until `quiet` passes with none arriving, deduplicating by
    /// sequence into the record. Returns how many new events were recorded.
    pub fn drain(&mut self, quiet: Duration) -> usize {
        let mut frames = Vec::new();
        {
            let conn = self.conn.as_ref().expect("drain needs a live connection");
            while let Some(frame) = conn.next_event(quiet) {
                frames.push(frame);
            }
        }
        frames.into_iter().filter(|frame| self.record(frame.clone())).count()
    }

    /// Wait until an event for `resource` arrives (recording everything seen
    /// along the way); panics after `timeout`.
    pub fn await_resource(&mut self, resource: &str, timeout: Duration) -> RecordedEvent {
        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let frame = self
                .conn
                .as_ref()
                .expect("await needs a live connection")
                .next_event(remaining)
                .unwrap_or_else(|| panic!("no event for '{resource}' within {timeout:?}"));
            let is_match = matches!(&frame, Frame::Event { data, .. }
                if parse_event(None, data).map(|e| e.resource == resource).unwrap_or(false));
            self.record(frame);
            if is_match {
                return self.events.last().expect("just recorded").clone();
            }
            if Instant::now() >= deadline {
                panic!("no event for '{resource}' within {timeout:?}");
            }
        }
    }

    /// The last recorded outbox sequence.
    pub fn last_seq(&self) -> Option<u64> {
        self.events.last().map(|e| e.seq)
    }

    /// The recorded sequences in arrival order — the gap/duplicate assertion
    /// surface.
    pub fn seqs(&self) -> Vec<u64> {
        self.events.iter().map(|e| e.seq).collect()
    }

    /// Record one frame, deduplicating by sequence. Returns whether it was new.
    fn record(&mut self, frame: Frame) -> bool {
        let Frame::Event { id, event, data } = frame else { return false };
        if event.as_deref() == Some("reset") {
            return false;
        }
        let Some(parsed) = parse_event(id.as_deref(), &data) else { return false };
        if self.events.iter().any(|e| e.seq == parsed.seq) {
            return false;
        }
        self.events.push(parsed);
        true
    }
}

/// Parse one invalidation frame into a [`RecordedEvent`]; the SSE id and the
/// DTO's eventId are the same outbox sequence, either identifies it.
fn parse_event(id: Option<&str>, data: &str) -> Option<RecordedEvent> {
    let dto: serde_json::Value = serde_json::from_str(data).ok()?;
    let seq: u64 = id
        .map(str::to_string)
        .or_else(|| dto.get("eventId").and_then(|v| v.as_str()).map(str::to_string))?
        .parse()
        .ok()?;
    Some(RecordedEvent {
        seq,
        scope: dto.get("scope").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        resource: dto.get("resourceId").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        revision: dto.get("revision").and_then(|v| v.as_u64()).unwrap_or_default(),
    })
}
