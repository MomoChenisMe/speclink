//! The loopback callback of a browser login (RFC 8252 §7.3; client-browser-login
//! D3 steps 4, 8 and 9).
//!
//! The listener binds `127.0.0.1` on a port the operating system picks — never
//! `localhost`, which can resolve elsewhere, and never a fixed port, which
//! another program may hold. It accepts one thing: a GET to `/callback` that
//! carries the state this login sent. Anything else is answered and ignored, so
//! another program on the machine cannot end the login with a forged request.
//!
//! The pages it serves say whether the login finished and nothing more: no
//! code, no token, no expected state.

use crate::login::LoginIo;
use std::io::{Read as _, Write as _};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

/// How long one connection may take to send its request, start to finish. A
/// browser's idle preconnect, or a program trickling bytes, must not stall the
/// wait for the real callback for long.
const REQUEST_DEADLINE: Duration = Duration::from_secs(2);
/// How much of a request is read. Only the request line matters; the headers
/// can run long, because cookies set by any server on 127.0.0.1 come along.
const MAX_REQUEST_BYTES: usize = 64 * 1024;

const PAGE_DONE: &str = "<!doctype html><meta charset=\"utf-8\"><title>speclink</title><p>登入完成，可以關閉這個視窗。</p><p>Sign-in complete. You can close this window.</p>";
const PAGE_FAILED: &str = "<!doctype html><meta charset=\"utf-8\"><title>speclink</title><p>登入沒有完成，請回到 speclink 查看原因。</p><p>Sign-in did not complete. Return to speclink for details.</p>";
const PAGE_NOT_OURS: &str = "<!doctype html><meta charset=\"utf-8\"><title>speclink</title><p>這不是 speclink 正在等待的登入回應。</p><p>This is not the sign-in response speclink is waiting for.</p>";
const PAGE_NOT_FOUND: &str = "<!doctype html><meta charset=\"utf-8\"><title>speclink</title><p>Not found.</p>";

/// The listening socket of one browser login. Dropping it closes the port.
pub(crate) struct CallbackListener {
    listener: TcpListener,
}

/// How the wait for the callback ended.
pub(crate) enum Callback {
    /// The login service sent the user back with a code. The browser is still
    /// waiting on `reply` to learn whether the login finished.
    Code { code: String, reply: Reply },
    Denied,
    Failed(String),
    Expired,
    Cancelled,
}

/// The browser's open connection, answered once the login has finished.
pub(crate) struct Reply(TcpStream);

impl Reply {
    pub fn finish(mut self, done: bool) {
        respond(&mut self.0, 200, if done { PAGE_DONE } else { PAGE_FAILED });
    }
}

impl CallbackListener {
    pub fn bind() -> std::io::Result<CallbackListener> {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        listener.set_nonblocking(true)?;
        Ok(CallbackListener { listener })
    }

    pub fn redirect_uri(&self) -> std::io::Result<String> {
        Ok(format!("http://127.0.0.1:{}/callback", self.listener.local_addr()?.port()))
    }

    /// Wait for the callback carrying `state`, for `window_secs` ticks of the
    /// injected clock (`io.sleep_secs(1)` each). Cancellation is checked every
    /// tick.
    pub fn wait(&self, state: &str, window_secs: u64, io: &dyn LoginIo) -> Callback {
        let mut waited = 0;
        loop {
            if io.cancelled() {
                return Callback::Cancelled;
            }
            match self.listener.accept() {
                Ok((stream, _)) => {
                    if let Some(callback) = handle(stream, state) {
                        return callback;
                    }
                }
                // No connection waiting (or a transient accept failure): one
                // tick of the window passes.
                Err(_) => {
                    if waited >= window_secs {
                        return Callback::Expired;
                    }
                    io.sleep_secs(1);
                    waited += 1;
                }
            }
        }
    }
}

/// Read one request and decide what it is. `None` means "not the callback —
/// keep waiting".
fn handle(mut stream: TcpStream, state: &str) -> Option<Callback> {
    // Accepted sockets inherit non-blocking mode on some platforms.
    stream.set_nonblocking(false).ok()?;
    stream.set_write_timeout(Some(REQUEST_DEADLINE)).ok()?;
    let request_line = read_request_line(&mut stream)?;
    let mut line = request_line.split(' ');
    let (method, target) = (line.next().unwrap_or_default(), line.next().unwrap_or_default());
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    if method != "GET" || path != "/callback" {
        respond(&mut stream, 404, PAGE_NOT_FOUND);
        return None;
    }

    let params = parse_query(query);
    let param = |name: &str| params.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str());
    // Every callback — an error one too — must carry our state; one that does
    // not is someone else's, and must not end this login.
    if !param("state").is_some_and(|got| same(got, state)) {
        respond(&mut stream, 400, PAGE_NOT_OURS);
        return None;
    }
    if let Some(error) = param("error") {
        respond(&mut stream, 200, PAGE_FAILED);
        if error == "access_denied" {
            return Some(Callback::Denied);
        }
        let code = crate::oauth::error_code(error);
        return Some(Callback::Failed(format!("the login service returned an error: {code}")));
    }
    match param("code").filter(|code| !code.is_empty()) {
        Some(code) => Some(Callback::Code { code: code.to_string(), reply: Reply(stream) }),
        None => {
            respond(&mut stream, 400, PAGE_FAILED);
            Some(Callback::Failed("the login service returned no authorization code".to_string()))
        }
    }
}

/// The request line of one request. The whole request is read — up to the
/// blank line, the size cap or the deadline, whichever comes first — so the
/// answer does not race the browser's own sending; then only the first line is
/// kept. A connection that has not sent a full request line by the deadline is
/// dropped.
fn read_request_line(stream: &mut TcpStream) -> Option<String> {
    let deadline = std::time::Instant::now() + REQUEST_DEADLINE;
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    while buf.len() < MAX_REQUEST_BYTES && !buf.windows(4).any(|w| w == b"\r\n\r\n") {
        let Some(left) = deadline.checked_duration_since(std::time::Instant::now()).filter(|d| !d.is_zero()) else {
            break;
        };
        stream.set_read_timeout(Some(left)).ok()?;
        match stream.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
        }
    }
    let end = buf.windows(2).position(|w| w == b"\r\n")?;
    Some(String::from_utf8_lossy(&buf[..end]).into_owned())
}

fn respond(stream: &mut TcpStream, status: u16, page: &str) {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        _ => "Not Found",
    };
    let _ = write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{page}",
        page.len()
    );
    let _ = stream.flush();
}

/// Compare without stopping at the first differing byte.
fn same(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn parse_query(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            (percent_decode(k), percent_decode(v))
        })
        .collect()
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                match std::str::from_utf8(&bytes[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok()) {
                    Some(byte) => {
                        out.push(byte);
                        i += 2;
                    }
                    None => out.push(b'%'),
                }
            }
            byte => out.push(byte),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_values_are_percent_decoded() {
        assert_eq!(
            parse_query("code=a%2Fb+c&state=x%ZZ&flag"),
            vec![
                ("code".to_string(), "a/b c".to_string()),
                ("state".to_string(), "x%ZZ".to_string()),
                ("flag".to_string(), String::new()),
            ]
        );
    }

    /// Accept one connection that `client` writes to, and hand it to `handle`.
    /// Returns the verdict and how long `handle` took to reach it.
    fn handled(client: impl FnOnce(TcpStream) + Send + 'static) -> (Option<Callback>, Duration) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let addr = listener.local_addr().unwrap();
        let writer = std::thread::spawn(move || client(TcpStream::connect(addr).unwrap()));
        let (stream, _) = listener.accept().unwrap();
        let started = std::time::Instant::now();
        let callback = handle(stream, "the-state");
        let took = started.elapsed();
        writer.join().unwrap();
        (callback, took)
    }

    /// Cookies from other servers on 127.0.0.1 ride along with the callback;
    /// headers far past a few kilobytes must not lose it.
    #[test]
    fn a_callback_with_very_long_headers_is_still_accepted() {
        // The client only sends: the answer comes after `handle` returns, and
        // `handled` joins the client before that.
        let (callback, _) = handled(|mut stream| {
            let cookie = "c=".to_string() + &"x".repeat(20_000);
            let _ = write!(stream, "GET /callback?code=abc&state=the-state HTTP/1.1\r\nCookie: {cookie}\r\n\r\n");
        });
        match callback {
            Some(Callback::Code { code, reply }) => {
                assert_eq!(code, "abc");
                reply.finish(true);
            }
            _ => panic!("the callback must be accepted"),
        }
    }

    /// A connection trickling bytes cannot hold the wait past the deadline.
    #[test]
    fn a_connection_that_never_finishes_its_request_line_is_dropped_at_the_deadline() {
        let (callback, took) = handled(|mut stream| {
            for _ in 0..8 {
                if stream.write_all(b"G").is_err() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(500));
            }
        });
        assert!(callback.is_none());
        assert!(took < REQUEST_DEADLINE + Duration::from_millis(1000), "{took:?}");
    }

    #[test]
    fn state_comparison_needs_an_exact_match() {
        assert!(same("abc", "abc"));
        assert!(!same("abc", "abd"));
        assert!(!same("abc", "ab"));
    }
}
