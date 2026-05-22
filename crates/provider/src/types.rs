//! 共用資料型別：`LinkYaml`、`ProjectInfo`、`ProjectStatus`、`InitOptions`、
//! `Etag`、`Versioned<T>`、`ExpectedEtag`、`ArtifactKind`、`ChangeRow` 與 `validate_kebab_id`。
//!
//! 這些型別是 SpecLink 各 provider 實作之間的 stable contract。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// `.speclink/link.yaml` v1 schema。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LinkYaml {
    pub version: u32,
    pub project_id: String,
    pub instance_id: String,
    pub provider: String,
    pub created_at: String,
    pub working_dir_fingerprint: String,
}

/// `speclink status` 回傳的專案狀態。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectStatus {
    pub project_id: String,
    pub provider: String,
    pub artifact_root: String,
    pub state_root: String,
    pub git_head: String,
    pub requires_git: bool,
}

/// `init` / `link` 等命令成功後回傳的精簡資訊。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectInfo {
    pub project_id: String,
    pub artifact_root: String,
    pub state_root: String,
}

/// `init` 的輸入旗標。
#[derive(Debug, Clone)]
pub struct InitOptions {
    pub working_dir: PathBuf,
    pub force: bool,
}

/// Artifact / change row Etag。永遠以 `sha256:<64 lowercase hex>` 形式存在。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Etag(String);

/// `Etag` 解析錯誤。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EtagError {
    #[error("Etag must start with `sha256:` prefix")]
    MissingPrefix,
    #[error("Etag hex digest must be exactly 64 lowercase hex chars, got {0} chars")]
    BadHexLength(usize),
    #[error("Etag hex digest must contain only [0-9a-f] characters")]
    BadHexChars,
}

impl Etag {
    /// 從原始 byte 串建立 Etag（`sha256:<hex>`）。
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        use sha2::{Digest, Sha256};
        let digest = Sha256::digest(bytes);
        Self(format!("sha256:{}", hex::encode(digest)))
    }

    /// 取得內部字串（含 `sha256:` prefix）。
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 取得 hex digest（不含 prefix）。
    #[must_use]
    pub fn hex(&self) -> &str {
        // 安全：constructor 保證 prefix 存在
        &self.0["sha256:".len()..]
    }
}

impl std::fmt::Display for Etag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for Etag {
    type Err = EtagError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rest = s.strip_prefix("sha256:").ok_or(EtagError::MissingPrefix)?;
        if rest.len() != 64 {
            return Err(EtagError::BadHexLength(rest.len()));
        }
        if !rest.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
            return Err(EtagError::BadHexChars);
        }
        Ok(Self(s.to_string()))
    }
}

/// 帶 Etag 的值。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Versioned<T> {
    pub value: T,
    pub etag: Etag,
}

/// `artifact.write` 並發控制旗標。
///
/// - `None` 表示「新建專用」：檔案必須不存在才會寫入。
/// - `Some(etag)` 表示「覆寫專用」：檔案必須存在且 sha256 與 `etag` 相符才會寫入。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpectedEtag {
    None,
    Some(Etag),
}

/// Artifact kind 白名單。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactKind {
    Proposal,
    Design,
    Tasks,
    Spec,
}

impl ArtifactKind {
    /// 從字串 parse；不在白名單回 `None`。
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "proposal" => Some(Self::Proposal),
            "design" => Some(Self::Design),
            "tasks" => Some(Self::Tasks),
            "spec" => Some(Self::Spec),
            _ => None,
        }
    }

    /// 對應的 stable string identifier。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposal => "proposal",
            Self::Design => "design",
            Self::Tasks => "tasks",
            Self::Spec => "spec",
        }
    }

    /// `--capability` 是否必填（僅 spec 需要）。
    #[must_use]
    pub fn requires_capability(&self) -> bool {
        matches!(self, Self::Spec)
    }
}

impl std::fmt::Display for ArtifactKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// state.db `change` 表 row。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeRow {
    pub change_id: String,
    pub name: String,
    pub state: String,
    pub schema_id: String,
    pub version: u32,
    pub created_at: String,
    pub updated_at: String,
}

/// kebab-case identifier 驗證錯誤。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum IdError {
    #[error("identifier MUST be 1-64 bytes (UTF-8), got {0} bytes")]
    BadLength(usize),
    #[error("identifier MUST match `^[a-z][a-z0-9]*(-[a-z0-9]+)*$`")]
    BadGrammar,
}

/// 驗證 kebab-case identifier：`^[a-z][a-z0-9]*(-[a-z0-9]+)*$`、長度 1-64 byte。
///
/// 此 grammar 被 change name 與 capability id 兩處共用。
///
/// # Errors
/// 長度不符回 [`IdError::BadLength`]；grammar 不符回 [`IdError::BadGrammar`]。
pub fn validate_kebab_id(s: &str) -> Result<(), IdError> {
    let len = s.len();
    if !(1..=64).contains(&len) {
        return Err(IdError::BadLength(len));
    }
    let bytes = s.as_bytes();
    if !bytes[0].is_ascii_lowercase() {
        return Err(IdError::BadGrammar);
    }
    let mut prev_hyphen = false;
    for &b in &bytes[1..] {
        if b.is_ascii_lowercase() || b.is_ascii_digit() {
            prev_hyphen = false;
        } else if b == b'-' {
            if prev_hyphen {
                return Err(IdError::BadGrammar);
            }
            prev_hyphen = true;
        } else {
            return Err(IdError::BadGrammar);
        }
    }
    if prev_hyphen {
        return Err(IdError::BadGrammar);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn etag_from_bytes_uses_sha256_prefix_and_lowercase_hex() {
        let etag = Etag::from_bytes(b"hello\n");
        assert_eq!(
            etag.as_str(),
            "sha256:5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"
        );
        assert_eq!(etag.hex().len(), 64);
    }

    #[test]
    fn etag_parse_accepts_valid_sha256_string() {
        let s = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
        let e = Etag::from_str(s).expect("parse");
        assert_eq!(e.as_str(), s);
    }

    #[test]
    fn etag_parse_rejects_missing_prefix() {
        let s = "0000000000000000000000000000000000000000000000000000000000000000";
        assert_eq!(Etag::from_str(s).unwrap_err(), EtagError::MissingPrefix);
    }

    #[test]
    fn etag_parse_rejects_wrong_hex_length() {
        let short = "sha256:000";
        assert!(matches!(
            Etag::from_str(short).unwrap_err(),
            EtagError::BadHexLength(3)
        ));
    }

    #[test]
    fn etag_parse_rejects_non_hex_chars() {
        let bad = "sha256:zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz";
        assert_eq!(Etag::from_str(bad).unwrap_err(), EtagError::BadHexChars);
    }

    #[test]
    fn versioned_serde_roundtrip() {
        let v = Versioned {
            value: "hello".to_string(),
            etag: Etag::from_bytes(b"hello\n"),
        };
        let json = serde_json::to_string(&v).expect("serialize");
        let back: Versioned<String> = serde_json::from_str(&json).expect("parse");
        assert_eq!(back, v);
    }

    #[test]
    fn expected_etag_has_two_variants() {
        let _none = ExpectedEtag::None;
        let _some = ExpectedEtag::Some(Etag::from_bytes(b""));
    }

    #[test]
    fn artifact_kind_parse_and_serde() {
        for k in [
            ArtifactKind::Proposal,
            ArtifactKind::Design,
            ArtifactKind::Tasks,
            ArtifactKind::Spec,
        ] {
            assert_eq!(ArtifactKind::parse(k.as_str()), Some(k));
        }
        assert_eq!(ArtifactKind::parse("summary"), None);
        assert!(ArtifactKind::Spec.requires_capability());
        for k in [
            ArtifactKind::Proposal,
            ArtifactKind::Design,
            ArtifactKind::Tasks,
        ] {
            assert!(!k.requires_capability());
        }
        let json = serde_json::to_string(&ArtifactKind::Proposal).expect("serialize");
        assert_eq!(json, "\"proposal\"");
    }

    #[test]
    fn change_row_serde_uses_camel_case() {
        let row = ChangeRow {
            change_id: "550e8400-e29b-41d4-a716-446655440000".into(),
            name: "billing-system".into(),
            state: "proposing".into(),
            schema_id: "spec-driven".into(),
            version: 1,
            created_at: "2026-05-22T10:30:00Z".into(),
            updated_at: "2026-05-22T10:30:00Z".into(),
        };
        let json = serde_json::to_string(&row).expect("serialize");
        for needle in [
            "changeId",
            "name",
            "state",
            "schemaId",
            "version",
            "createdAt",
            "updatedAt",
        ] {
            assert!(
                json.contains(needle),
                "missing {needle} in serialized ChangeRow: {json}"
            );
        }
        let back: ChangeRow = serde_json::from_str(&json).expect("parse");
        assert_eq!(back, row);
    }

    #[test]
    fn validate_kebab_id_accepts_valid_examples_from_spec_table() {
        // Spec table 中標 yes 的兩列。
        validate_kebab_id("billing-system").expect("billing-system valid");
        validate_kebab_id("add-2fa").expect("add-2fa valid");
        // Plus boundary length cases.
        validate_kebab_id("a").expect("1-byte valid");
        let max = "a".repeat(64);
        validate_kebab_id(&max).expect("64-byte valid");
    }

    #[test]
    fn validate_kebab_id_rejects_invalid_examples_from_spec_table() {
        // Spec table 中標 no 的所有列。
        assert!(matches!(
            validate_kebab_id("BillingSystem").unwrap_err(),
            IdError::BadGrammar
        ));
        assert!(matches!(
            validate_kebab_id("billing_system").unwrap_err(),
            IdError::BadGrammar
        ));
        assert!(matches!(
            validate_kebab_id("-billing").unwrap_err(),
            IdError::BadGrammar
        ));
        assert!(matches!(
            validate_kebab_id("billing-").unwrap_err(),
            IdError::BadGrammar
        ));
        assert!(matches!(
            validate_kebab_id("billing--system").unwrap_err(),
            IdError::BadGrammar
        ));
        assert!(matches!(
            validate_kebab_id("2fa-feature").unwrap_err(),
            IdError::BadGrammar
        ));
        assert!(matches!(
            validate_kebab_id("").unwrap_err(),
            IdError::BadLength(0)
        ));
        let too_long = "a".repeat(65);
        assert!(matches!(
            validate_kebab_id(&too_long).unwrap_err(),
            IdError::BadLength(65)
        ));
    }
}
