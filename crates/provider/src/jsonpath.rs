//! JSONPath subset for config keys.
//!
//! Grammar：`segment ( '.' segment | '[' index ']' )*`，其中
//! `segment = [a-zA-Z_][a-zA-Z0-9_-]*`、`index = [0-9]+`。Wildcards、filters、
//! recursive-descent SHALL be rejected。對應 `config-rw` capability requirement
//! 「The JSONPath subset grammar SHALL be `segment ( '.' segment | '[' index ']' )*`
//! ...」與 design decision「JSONPath subset grammar 與 CLI value 解析規則」。
//!
//! A5 此 module 只承載 `JsonPath` newtype 與 `JsonPathSegment` enum；實際 parser
//! 與 grammar 拒絕路徑由 task 7.2（`crates/cli/src/commands/config.rs`）接通並可
//! 把 parsing helper 放在這裡或就近 CLI module，視 6.3 / 7.2 實作落地。

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// JSONPath subset 的單段：dot-separated field 或 bracketed index。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum JsonPathSegment {
    /// 對應 `rules` / `require_code_review` 這類 field 名稱。
    Field(String),
    /// 對應 `[0]` / `[42]` 這類陣列索引。
    Index(usize),
}

/// `JsonPath::parse` 失敗時的錯誤型別。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum JsonPathParseError {
    /// 路徑包含 wildcard（`*`）；對應 spec scenario「Reject unsupported JSONPath syntax」。
    #[error("JSONPath wildcards (`*`) are not supported; use a literal segment")]
    UnsupportedWildcard,
    /// 路徑包含 filter（`?`）或 recursive descent（`..`）。
    #[error("JSONPath filters and recursive-descent are not supported")]
    UnsupportedSyntax,
    /// 路徑為空、或 segment grammar 不符（`[a-zA-Z_][a-zA-Z0-9_-]*`）。
    #[error("invalid JSONPath segment grammar at position {pos}: {detail}")]
    BadSegment { pos: usize, detail: String },
    /// `[index]` 的 index 不是十進位數字。
    #[error("invalid JSONPath array index at position {pos}: {detail}")]
    BadIndex { pos: usize, detail: String },
}

/// 解析後的 JSONPath 序列；A5 newtype，不暴露內部 `Vec`。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct JsonPath(Vec<JsonPathSegment>);

impl JsonPath {
    /// 由 caller 已驗證過的 segment vec 建構；parser 失敗路徑不在此 method。
    #[must_use]
    pub fn from_segments(segments: Vec<JsonPathSegment>) -> Self {
        Self(segments)
    }

    /// 取得內部 segment slice。
    #[must_use]
    pub fn segments(&self) -> &[JsonPathSegment] {
        &self.0
    }

    /// 是否為空路徑（無 segment）。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// 解析 dot/bracket-form JSONPath subset 字串。
    ///
    /// Grammar：`segment ( '.' segment | '[' index ']' )*`，`segment` = `[a-zA-Z_][a-zA-Z0-9_-]*`、
    /// `index` = `[0-9]+`。Wildcards、filters、recursive-descent 均拒絕。
    ///
    /// # Errors
    /// 不符 grammar 或包含不支援語法時回 [`JsonPathParseError`]。
    pub fn parse(input: &str) -> Result<Self, JsonPathParseError> {
        if input.contains('*') {
            return Err(JsonPathParseError::UnsupportedWildcard);
        }
        if input.contains('?') || input.contains("..") {
            return Err(JsonPathParseError::UnsupportedSyntax);
        }
        let bytes = input.as_bytes();
        let mut segs = Vec::new();
        let mut i = 0;
        if bytes.is_empty() {
            return Err(JsonPathParseError::BadSegment {
                pos: 0,
                detail: "empty path".to_string(),
            });
        }
        // 路徑必須以 segment 起頭（不允許 leading `.` 或 `[`）。
        if matches!(bytes[0], b'.' | b'[') {
            return Err(JsonPathParseError::BadSegment {
                pos: 0,
                detail: format!(
                    "path SHALL start with a segment, got `{}`",
                    bytes[0] as char
                ),
            });
        }
        while i < bytes.len() {
            match bytes[i] {
                b'.' => {
                    i += 1;
                    if i == bytes.len() {
                        return Err(JsonPathParseError::BadSegment {
                            pos: i,
                            detail: "trailing `.` without segment".to_string(),
                        });
                    }
                    // 進入 segment 解析（fallthrough by re-loop）。
                    let (seg, consumed) = parse_segment(&bytes[i..], i)?;
                    segs.push(JsonPathSegment::Field(seg));
                    i += consumed;
                }
                b'[' => {
                    let close = bytes[i..].iter().position(|&b| b == b']').ok_or(
                        JsonPathParseError::BadIndex {
                            pos: i,
                            detail: "missing `]`".to_string(),
                        },
                    )?;
                    let inner = &input[i + 1..i + close];
                    let idx = inner
                        .parse::<usize>()
                        .map_err(|e| JsonPathParseError::BadIndex {
                            pos: i + 1,
                            detail: format!("not a base-10 integer: {e}"),
                        })?;
                    segs.push(JsonPathSegment::Index(idx));
                    i += close + 1;
                }
                _ => {
                    let (seg, consumed) = parse_segment(&bytes[i..], i)?;
                    segs.push(JsonPathSegment::Field(seg));
                    i += consumed;
                }
            }
        }
        Ok(Self(segs))
    }
}

fn parse_segment(bytes: &[u8], offset: usize) -> Result<(String, usize), JsonPathParseError> {
    if bytes.is_empty() {
        return Err(JsonPathParseError::BadSegment {
            pos: offset,
            detail: "expected segment, got end-of-input".to_string(),
        });
    }
    let first = bytes[0];
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return Err(JsonPathParseError::BadSegment {
            pos: offset,
            detail: format!("segment must start with [a-zA-Z_], got `{}`", first as char),
        });
    }
    let mut end = 1;
    while end < bytes.len() {
        let c = bytes[end];
        if c.is_ascii_alphanumeric() || c == b'_' || c == b'-' {
            end += 1;
        } else {
            break;
        }
    }
    let seg = std::str::from_utf8(&bytes[..end])
        .map_err(|_| JsonPathParseError::BadSegment {
            pos: offset,
            detail: "non-UTF-8 byte in segment".to_string(),
        })?
        .to_string();
    Ok((seg, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_dotted_path() {
        let p = JsonPath::parse("rules.require_code_review").expect("parse ok");
        assert_eq!(
            p.segments(),
            &[
                JsonPathSegment::Field("rules".into()),
                JsonPathSegment::Field("require_code_review".into()),
            ]
        );
    }

    #[test]
    fn parse_path_with_index() {
        let p = JsonPath::parse("roles[0].name").expect("parse ok");
        assert_eq!(
            p.segments(),
            &[
                JsonPathSegment::Field("roles".into()),
                JsonPathSegment::Index(0),
                JsonPathSegment::Field("name".into()),
            ]
        );
    }

    #[test]
    fn reject_wildcard() {
        assert_eq!(
            JsonPath::parse("rules.*").unwrap_err(),
            JsonPathParseError::UnsupportedWildcard
        );
    }

    #[test]
    fn reject_recursive_descent() {
        assert_eq!(
            JsonPath::parse("a..b").unwrap_err(),
            JsonPathParseError::UnsupportedSyntax
        );
    }

    #[test]
    fn reject_leading_dot() {
        assert!(matches!(
            JsonPath::parse(".rules").unwrap_err(),
            JsonPathParseError::BadSegment { .. }
        ));
    }

    #[test]
    fn reject_empty() {
        assert!(matches!(
            JsonPath::parse("").unwrap_err(),
            JsonPathParseError::BadSegment { .. }
        ));
    }
}
