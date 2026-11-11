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

/// How long one connection may take to send its request. A browser's idle
/// preconnect must not stall the wait for the real callback for long.
const REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(2);
/// A callback request is a request line and a few headers.
const MAX_REQUEST_BYTES: usize = 8 * 1024;

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
    stream.set_read_timeout(Some(REQUEST_READ_TIMEOUT)).ok()?;
    stream.set_write_timeout(Some(REQUEST_READ_TIMEOUT)).ok()?;
    let head = read_head(&mut stream)?;
    let mut line = head.lines().next().unwrap_or_default().split(' ');
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
        let code: String = error.chars().filter(|c| c.is_ascii_graphic()).take(64).collect();
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

/// The request line and headers, up to the blank line. A connection that sends
/// nothing in time, or too much, is dropped.
fn read_head(stream: &mut TcpStream) -> Option<String> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 1024];
    while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
        let n = stream.read(&mut chunk).ok()?;
        if n == 0 || buf.len() + n > MAX_REQUEST_BYTES {
            return None;
        }
        buf.extend_from_slice(&chunk[..n]);
    }
    Some(String::from_utf8_lossy(&buf).into_owned())
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

    #[test]
    fn state_comparison_needs_an_exact_match() {
        assert!(same("abc", "abc"));
        assert!(!same("abc", "abd"));
        assert!(!same("abc", "ab"));
    }
}
