//! Secret redaction：在寫入 tracing 輸出前替換已知 secret 欄位與 `Bearer xxx` token。
//!
//! 此實作以字串掃描替代 regex 依賴：先逐個 secret key 處理 `key="..."` / `key=...`
//! 形式的鍵值對，再處理 `Bearer <token>` 樣式。

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

/// 替換為的占位文字。
pub const REDACTION_PLACEHOLDER: &str = "[REDACTED]";

/// 列為 secret 的欄位名稱（依長度由長到短，避免 `access_token` 被 `token` 規則先 partial-match）。
const SECRET_KEYS: &[&str] = &[
    "access_token",
    "refresh_token",
    "api_key",
    "password",
    "secret",
    "token",
];

/// 對輸入字串套用 secret redaction，回傳新字串。
pub fn redact(input: &str) -> String {
    let mut s = input.to_string();
    for key in SECRET_KEYS {
        s = redact_kv(&s, key);
    }
    s = redact_bearer(&s);
    s
}

fn redact_kv(input: &str, key: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    loop {
        let Some(pos) = rest.find(key) else {
            out.push_str(rest);
            return out;
        };
        // 左邊界：必須是字串起頭，或前一個字元非 `[a-zA-Z0-9_]`。
        let prefix_ok = match rest[..pos].chars().last() {
            None => true,
            Some(c) => !(c.is_ascii_alphanumeric() || c == '_'),
        };
        let after_key = &rest[pos + key.len()..];
        if !prefix_ok || !after_key.starts_with('=') {
            // 不是 key=value 形式；複製到 key 結尾後繼續找下一個。
            out.push_str(&rest[..pos + key.len()]);
            rest = after_key;
            continue;
        }
        // 寫出 `key=`
        out.push_str(&rest[..pos + key.len() + 1]);
        let val_start = &after_key[1..];
        if let Some(stripped) = val_start.strip_prefix('"') {
            // quoted value
            out.push('"');
            let end_quote = stripped.find('"').unwrap_or(stripped.len());
            out.push_str(REDACTION_PLACEHOLDER);
            if end_quote < stripped.len() {
                out.push('"');
                rest = &stripped[end_quote + 1..];
            } else {
                rest = "";
            }
        } else {
            // unquoted；到第一個空白或 `,` 為止
            let end = val_start
                .find(|c: char| c.is_whitespace() || c == ',')
                .unwrap_or(val_start.len());
            out.push_str(REDACTION_PLACEHOLDER);
            rest = &val_start[end..];
        }
    }
}

fn redact_bearer(input: &str) -> String {
    const BEARER: &str = "Bearer ";
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    loop {
        let Some(pos) = rest.find(BEARER) else {
            out.push_str(rest);
            return out;
        };
        out.push_str(&rest[..pos + BEARER.len()]);
        let after = &rest[pos + BEARER.len()..];
        let end = after
            .find(|c: char| c.is_whitespace() || c == '"')
            .unwrap_or(after.len());
        if end == 0 {
            // 沒有後續 token，不視為 Bearer match
            rest = after;
            continue;
        }
        out.push_str(REDACTION_PLACEHOLDER);
        rest = &after[end..];
    }
}

/// 寫入時自動套用 [`redact`] 的 `io::Write` wrapper。
pub struct RedactingWriter<W: Write> {
    inner: W,
}

impl<W: Write> RedactingWriter<W> {
    /// 建立新 writer。
    pub fn new(inner: W) -> Self {
        Self { inner }
    }
}

impl<W: Write> Write for RedactingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = String::from_utf8_lossy(buf);
        let redacted = redact(&s);
        self.inner.write_all(redacted.as_bytes())?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// 共享緩衝區的 `MakeWriter`，每次寫入經 [`redact`] 處理後寫入鎖內 buffer。
#[derive(Clone)]
pub struct SharedRedactingWriter {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl SharedRedactingWriter {
    /// 建立新 shared writer。
    pub fn new(buf: Arc<Mutex<Vec<u8>>>) -> Self {
        Self { buf }
    }
}

impl Write for SharedRedactingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = String::from_utf8_lossy(buf);
        let redacted = redact(&s);
        let Ok(mut g) = self.buf.lock() else {
            return Err(io::Error::other("redacting writer mutex poisoned"));
        };
        g.extend_from_slice(redacted.as_bytes());
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for SharedRedactingWriter {
    type Writer = SharedRedactingWriter;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::tracing_layer::redact;

    #[test]
    fn redacts_token_field_value() {
        let input = r#"event token="abc123""#;
        let out = redact(input);
        assert!(!out.contains("abc123"));
        assert!(out.contains("[REDACTED]"));
    }

    #[test]
    fn redacts_token_kv_no_quotes() {
        let input = "event token=abc123 more";
        let out = redact(input);
        assert!(!out.contains("abc123"), "got: {out}");
        assert!(out.contains("[REDACTED]"));
    }

    #[test]
    fn redacts_bearer_value() {
        let input = "authorization=\"Bearer abc123\"";
        let out = redact(input);
        assert!(!out.contains("abc123"), "got: {out}");
        assert!(out.contains("[REDACTED]"));
    }

    #[test]
    fn redacts_all_known_secret_keys() {
        let keys = [
            "token",
            "access_token",
            "refresh_token",
            "api_key",
            "password",
            "secret",
        ];
        for k in keys {
            let input = format!("{k}=\"hunter2\"");
            let out = redact(&input);
            assert!(!out.contains("hunter2"), "key '{k}' did not redact: {out}");
        }
    }

    #[test]
    fn leaves_non_secret_text_unchanged() {
        let input = "just a normal log line about foo and bar";
        let out = redact(input);
        assert_eq!(out, input);
    }

    #[test]
    fn tracing_subscriber_with_shared_writer_captures_redacted() {
        use crate::tracing_layer::SharedRedactingWriter;
        use std::sync::{Arc, Mutex};
        use tracing_subscriber::fmt::SubscriberBuilder;

        let buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
        let writer = SharedRedactingWriter::new(buf.clone());
        let subscriber = SubscriberBuilder::default()
            .with_writer(writer)
            .with_ansi(false)
            .with_target(false)
            .with_level(false)
            .without_time()
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(token = "abc123", "doing work");
            tracing::info!(authorization = "Bearer abc123", "called api");
        });
        let captured = {
            let g = buf.lock().unwrap();
            String::from_utf8(g.clone()).unwrap()
        };
        assert!(
            !captured.contains("abc123"),
            "expected secret redacted, got: {captured}"
        );
        assert!(captured.contains("[REDACTED]"), "captured: {captured}");
    }
}
