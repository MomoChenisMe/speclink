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

/// Change lifecycle 的 6 個合法 state 值（slice A3 落實 `state-machine` capability）。
///
/// 序列化為 dot-separated snake_case 字串以對齊 `state.db.change.state` 欄位的內容。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ChangeState {
    /// 新建 change 的初始狀態；artifact DAG 未齊全。
    Proposing,
    /// `require_artifact_review=true` 時，DAG 齊全暫停等 reviewer approve。
    Reviewing,
    /// `apply` 待開始；可前進至 `in_progress`。
    Ready,
    /// `apply.start` 後的工作中狀態。
    InProgress,
    /// `require_code_review=true` 時，所有 task done 後等 reviewer approve。
    CodeReviewing,
    /// 終態：change archive 後保留 row 但不再 mutate。
    Archived,
}

impl ChangeState {
    /// 列舉全部 6 個合法 variant（順序與 design §6.2 表一致）。
    #[must_use]
    pub const fn all() -> [ChangeState; 6] {
        [
            ChangeState::Proposing,
            ChangeState::Reviewing,
            ChangeState::Ready,
            ChangeState::InProgress,
            ChangeState::CodeReviewing,
            ChangeState::Archived,
        ]
    }

    /// 對應的 stable string identifier；與 `Display` / serde 結果一致。
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            ChangeState::Proposing => "proposing",
            ChangeState::Reviewing => "reviewing",
            ChangeState::Ready => "ready",
            ChangeState::InProgress => "in_progress",
            ChangeState::CodeReviewing => "code_reviewing",
            ChangeState::Archived => "archived",
        }
    }
}

/// `ChangeState::from_str` 失敗時的錯誤型別。對應 `state.invalid_value` 錯誤路徑。
#[derive(Debug, Error, PartialEq, Eq)]
#[error(
    "invalid change.state value `{value}`: expected one of proposing/reviewing/ready/in_progress/code_reviewing/archived"
)]
pub struct ChangeStateParseError {
    /// 觸發錯誤的原始字串。
    pub value: String,
}

impl std::fmt::Display for ChangeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ChangeState {
    type Err = ChangeStateParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "proposing" => Ok(ChangeState::Proposing),
            "reviewing" => Ok(ChangeState::Reviewing),
            "ready" => Ok(ChangeState::Ready),
            "in_progress" => Ok(ChangeState::InProgress),
            "code_reviewing" => Ok(ChangeState::CodeReviewing),
            "archived" => Ok(ChangeState::Archived),
            other => Err(ChangeStateParseError {
                value: other.to_string(),
            }),
        }
    }
}

/// `state_transition.reason` 欄位的列舉值；對應 design §6.2 transition table 的觸發來源。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum StateTransitionReason {
    /// `apply start` 把 `ready` 推進到 `in_progress`。
    ApplyStart,
    /// `apply pause` 把 `in_progress` 退回 `ready`。
    ApplyPause,
    /// `task done` 完成最後一個 task；walking-skeleton 下設 `all_tasks_done=1`，
    /// `require_code_review=true` 下推進到 `code_reviewing`。
    TaskDoneAuto,
    /// `task undo` 在 `code_reviewing` state 下退回 `in_progress`。
    TaskUndoRevert,
    /// `artifact.write` 後 DAG evaluator 把 `proposing` 推進到 `ready` / `reviewing`。
    ArtifactDagComplete,
    /// 預留：future review slice 把 `reviewing` 推進到 `ready`。
    ReviewApprovedArtifact,
    /// 預留：future review slice 把 `code_reviewing` 退回 `in_progress`。
    ReviewRejectedCode,
    /// `archive.run` 把 `in_progress`（walking-skeleton）或 `code_reviewing`（review slice 後）
    /// 推進到 `archived`。
    ArchiveRun,
    /// `archive.run` 在 SQLite commit 後 filesystem rename 失敗時的 best-effort revert：
    /// 把剛 commit 的 `archived` 退回 `in_progress`、清空 `archived_at`。
    /// 只由 `LocalArchiveStore::archive_change` 在 rename 失敗 fallback path 寫入。
    ArchiveRunRevert,
}

impl StateTransitionReason {
    /// 對應的 stable string identifier；與 serde 結果一致。
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            StateTransitionReason::ApplyStart => "apply_start",
            StateTransitionReason::ApplyPause => "apply_pause",
            StateTransitionReason::TaskDoneAuto => "task_done_auto",
            StateTransitionReason::TaskUndoRevert => "task_undo_revert",
            StateTransitionReason::ArtifactDagComplete => "artifact_dag_complete",
            StateTransitionReason::ReviewApprovedArtifact => "review_approved_artifact",
            StateTransitionReason::ReviewRejectedCode => "review_rejected_code",
            StateTransitionReason::ArchiveRun => "archive_run",
            StateTransitionReason::ArchiveRunRevert => "archive_run_revert",
        }
    }
}

impl std::fmt::Display for StateTransitionReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// `apply.start` 把 change 綁定到的執行者；對應 `change.actor_json` 欄位 schema。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Actor {
    /// AI agent host 識別碼（例如 `claude-code` / `cursor` / `cli`）。
    pub agent_host: String,
    /// 作業系統使用者名稱；由 `whoami` 跨平台 lookup 取得。
    pub os_user: String,
    /// 主機識別碼；由 cross-platform hostname lookup 取得。
    pub host_id: String,
}

/// `StateMachineStore::transition_state` 的輸入請求。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionRequest {
    /// 目標 state（合法 transition 由 runtime 層的 state machine 表決定）。
    pub to_state: ChangeState,
    /// 此次 transition 對 actor 的影響：`Some(Some(actor))` 代表寫入新 actor、
    /// `Some(None)` 代表清空 actor、`None` 代表不動 actor。
    pub actor: Option<Option<Actor>>,
    /// Audit log reason code；寫入 `state_transition.reason` 欄位。
    pub reason: StateTransitionReason,
}

/// `StateMachineStore` 對外回傳的 change state 快照。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeStateView {
    pub change_id: String,
    pub state: ChangeState,
    pub version: u64,
    pub actor: Option<Actor>,
    pub all_tasks_done: bool,
}

/// `ArchiveStore::archive_change` 的輸入請求。
///
/// 對應 CLI 表面：`speclink archive <change-id> [--skip-specs] [--yes] [--no-validate] [--json]`。
/// 本 slice 內 `no_validate` 與 `yes` 為 no-op flag（接受並回傳但不改變 runtime 行為）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveRequest {
    /// 目標 change 識別碼（即 `change.name`）。
    pub change_id: String,
    /// 跳過 spec delta merge、僅 state transition + 目錄搬遷；emergency 用。
    pub skip_specs: bool,
    /// CLI 表面相容旗標：本 slice 為 no-op；`add-analyze` slice 之後接 validation。
    pub no_validate: bool,
    /// CLI 表面相容旗標：archive 本 slice 不 prompt；旗標僅為對齊 catalogue。
    pub yes: bool,
}

/// `archive.run` 對單一 capability 的 spec delta merge 結果。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MergedSpec {
    /// Capability 識別碼（即 `.speclink/specs/<capability>/` 目錄名）。
    pub capability: String,
    /// 寫入後新檔案的 line count。
    pub lines_added: u64,
    /// 寫入前目標檔案的 line count；新 capability dir 為 0。
    pub lines_removed: u64,
}

/// `ArchiveStore::archive_change` 的輸出結果。
///
/// 對應 `archive.run` op 的 JSON envelope `data` 欄位 schema（design.md「JSON envelope shape」）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveResult {
    /// Archive 完成的 change 識別碼。
    pub change_id: String,
    /// 結束狀態；成功 `archive.run` 永遠為 [`ChangeState::Archived`]。
    pub state: ChangeState,
    /// 本次 merge 的 capability spec 列表；`--skip-specs` 路徑為空陣列。
    pub merged_specs: Vec<MergedSpec>,
    /// UTC ISO-8601 timestamp，對應 `change.archived_at` 欄位。
    pub archived_at: String,
    /// Working-tree-relative 路徑指向新 archive 目錄
    /// （`.speclink/changes/archive/<YYYY-MM-DD>-<change-id>[-N]`）。
    pub archive_dir: String,
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
    fn change_state_all_enumerates_six_variants_in_design_order() {
        let v = ChangeState::all();
        assert_eq!(v.len(), 6);
        assert_eq!(v[0], ChangeState::Proposing);
        assert_eq!(v[1], ChangeState::Reviewing);
        assert_eq!(v[2], ChangeState::Ready);
        assert_eq!(v[3], ChangeState::InProgress);
        assert_eq!(v[4], ChangeState::CodeReviewing);
        assert_eq!(v[5], ChangeState::Archived);
    }

    #[test]
    fn change_state_display_and_as_str_match_serde_snake_case() {
        for s in ChangeState::all() {
            let json = serde_json::to_string(&s).expect("serialize");
            let stripped = json.trim_matches('"');
            assert_eq!(s.as_str(), stripped, "as_str vs serde for {s:?}");
            assert_eq!(format!("{s}"), stripped, "Display vs serde for {s:?}");
        }
        // Pin exact strings (spec normative — case-sensitive)
        assert_eq!(ChangeState::Proposing.as_str(), "proposing");
        assert_eq!(ChangeState::Reviewing.as_str(), "reviewing");
        assert_eq!(ChangeState::Ready.as_str(), "ready");
        assert_eq!(ChangeState::InProgress.as_str(), "in_progress");
        assert_eq!(ChangeState::CodeReviewing.as_str(), "code_reviewing");
        assert_eq!(ChangeState::Archived.as_str(), "archived");
    }

    #[test]
    fn change_state_from_str_round_trips_all_six() {
        for s in ChangeState::all() {
            let parsed: ChangeState = s.as_str().parse().expect("parse");
            assert_eq!(parsed, s);
        }
    }

    #[test]
    fn change_state_from_str_rejects_illegal_values() {
        // Case-sensitive: uppercase rejected (per state-machine spec example).
        assert!("Proposing".parse::<ChangeState>().is_err());
        // Not in enum.
        assert!("done".parse::<ChangeState>().is_err());
        // Empty string.
        assert!("".parse::<ChangeState>().is_err());
        let err = "garbage".parse::<ChangeState>().unwrap_err();
        assert_eq!(err.value, "garbage");
    }

    #[test]
    fn change_state_serde_roundtrip_via_json() {
        for s in ChangeState::all() {
            let json = serde_json::to_string(&s).expect("serialize");
            let back: ChangeState = serde_json::from_str(&json).expect("parse");
            assert_eq!(back, s);
        }
    }

    #[test]
    fn state_transition_reason_serde_uses_snake_case() {
        let pairs = [
            (StateTransitionReason::ApplyStart, "apply_start"),
            (StateTransitionReason::ApplyPause, "apply_pause"),
            (StateTransitionReason::TaskDoneAuto, "task_done_auto"),
            (StateTransitionReason::TaskUndoRevert, "task_undo_revert"),
            (
                StateTransitionReason::ArtifactDagComplete,
                "artifact_dag_complete",
            ),
            (
                StateTransitionReason::ReviewApprovedArtifact,
                "review_approved_artifact",
            ),
            (
                StateTransitionReason::ReviewRejectedCode,
                "review_rejected_code",
            ),
            (StateTransitionReason::ArchiveRun, "archive_run"),
            (
                StateTransitionReason::ArchiveRunRevert,
                "archive_run_revert",
            ),
        ];
        for (variant, expected) in pairs {
            let json = serde_json::to_string(&variant).expect("serialize");
            assert_eq!(json, format!("\"{expected}\""), "serde for {variant:?}");
            let back: StateTransitionReason = serde_json::from_str(&json).expect("parse");
            assert_eq!(back, variant);
            assert_eq!(variant.as_str(), expected);
            assert_eq!(format!("{variant}"), expected);
        }
    }

    #[test]
    fn archive_request_default_flags_are_false() {
        // ArchiveRequest 預設 flag 全 false：clap derive 預設行為對齊。
        let req = ArchiveRequest {
            change_id: "demo".into(),
            skip_specs: false,
            no_validate: false,
            yes: false,
        };
        assert_eq!(req.change_id, "demo");
        assert!(!req.skip_specs);
        assert!(!req.no_validate);
        assert!(!req.yes);
    }

    #[test]
    fn archive_request_flags_can_be_toggled_independently() {
        let req = ArchiveRequest {
            change_id: "demo".into(),
            skip_specs: true,
            no_validate: false,
            yes: true,
        };
        assert!(req.skip_specs);
        assert!(!req.no_validate);
        assert!(req.yes);
    }

    #[test]
    fn archive_result_serde_uses_camel_case() {
        let result = ArchiveResult {
            change_id: "demo".into(),
            state: ChangeState::Archived,
            merged_specs: vec![MergedSpec {
                capability: "user-auth".into(),
                lines_added: 142,
                lines_removed: 0,
            }],
            archived_at: "2026-05-22T18:00:00Z".into(),
            archive_dir: ".speclink/changes/archive/2026-05-22-demo".into(),
        };
        let json = serde_json::to_string(&result).expect("serialize");
        for needle in [
            "changeId",
            "state",
            "mergedSpecs",
            "archivedAt",
            "archiveDir",
            "archived",
            "user-auth",
        ] {
            assert!(json.contains(needle), "missing {needle} in {json}");
        }
        let back: ArchiveResult = serde_json::from_str(&json).expect("parse");
        assert_eq!(back, result);
    }

    #[test]
    fn archive_result_state_is_always_archived() {
        // archive.run 成功 state 必為 archived；compile-time check 不可能、
        // 但 runtime serde roundtrip 至少保證序列化形態。
        let result = ArchiveResult {
            change_id: "demo".into(),
            state: ChangeState::Archived,
            merged_specs: vec![],
            archived_at: "2026-05-22T18:00:00Z".into(),
            archive_dir: ".speclink/changes/archive/2026-05-22-demo".into(),
        };
        assert_eq!(result.state, ChangeState::Archived);
    }

    #[test]
    fn merged_spec_serde_uses_camel_case() {
        let spec = MergedSpec {
            capability: "audit-log".into(),
            lines_added: 87,
            lines_removed: 64,
        };
        let json = serde_json::to_string(&spec).expect("serialize");
        for needle in ["capability", "linesAdded", "linesRemoved", "audit-log"] {
            assert!(json.contains(needle), "missing {needle} in {json}");
        }
        let back: MergedSpec = serde_json::from_str(&json).expect("parse");
        assert_eq!(back, spec);
    }

    #[test]
    fn merged_spec_lines_removed_zero_for_new_capability() {
        let spec = MergedSpec {
            capability: "new-cap".into(),
            lines_added: 50,
            lines_removed: 0,
        };
        assert_eq!(spec.lines_removed, 0);
    }

    #[test]
    fn archive_result_empty_merged_specs_for_skip_specs_path() {
        let result = ArchiveResult {
            change_id: "demo".into(),
            state: ChangeState::Archived,
            merged_specs: vec![],
            archived_at: "2026-05-22T18:00:00Z".into(),
            archive_dir: ".speclink/changes/archive/2026-05-22-demo".into(),
        };
        assert!(result.merged_specs.is_empty());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(json.contains("\"mergedSpecs\":[]"), "got: {json}");
    }

    #[test]
    fn state_transition_reason_archive_run_revert_is_separate_variant() {
        // ArchiveRun 與 ArchiveRunRevert 必須是兩個獨立 variant，
        // 對應「best-effort revert」與正常 archive 在 audit log 上的區分。
        assert_ne!(
            StateTransitionReason::ArchiveRun,
            StateTransitionReason::ArchiveRunRevert
        );
        assert_eq!(StateTransitionReason::ArchiveRun.as_str(), "archive_run");
        assert_eq!(
            StateTransitionReason::ArchiveRunRevert.as_str(),
            "archive_run_revert"
        );
    }

    #[test]
    fn actor_serde_round_trips_all_three_fields() {
        let actor = Actor {
            agent_host: "claude-code".to_string(),
            os_user: "alice".to_string(),
            host_id: "macbook-alice".to_string(),
        };
        let json = serde_json::to_string(&actor).expect("serialize");
        for needle in ["agent_host", "os_user", "host_id"] {
            assert!(json.contains(needle), "missing {needle} in {json}");
        }
        let back: Actor = serde_json::from_str(&json).expect("parse");
        assert_eq!(back, actor);
    }

    #[test]
    fn change_state_view_serde_uses_camel_case() {
        let view = ChangeStateView {
            change_id: "cid-1".to_string(),
            state: ChangeState::InProgress,
            version: 3,
            actor: Some(Actor {
                agent_host: "cli".into(),
                os_user: "bob".into(),
                host_id: "linux-box".into(),
            }),
            all_tasks_done: false,
        };
        let json = serde_json::to_string(&view).expect("serialize");
        for needle in ["changeId", "version", "allTasksDone", "in_progress"] {
            assert!(json.contains(needle), "missing {needle} in {json}");
        }
        let back: ChangeStateView = serde_json::from_str(&json).expect("parse");
        assert_eq!(back, view);
    }

    #[test]
    fn transition_request_actor_field_has_three_distinct_semantics() {
        // None → 不動 actor
        let req = TransitionRequest {
            to_state: ChangeState::InProgress,
            actor: None,
            reason: StateTransitionReason::ApplyStart,
        };
        assert!(req.actor.is_none());
        // Some(Some(actor)) → assign new actor
        let req = TransitionRequest {
            to_state: ChangeState::InProgress,
            actor: Some(Some(Actor {
                agent_host: "cli".into(),
                os_user: "alice".into(),
                host_id: "h".into(),
            })),
            reason: StateTransitionReason::ApplyStart,
        };
        assert!(matches!(req.actor, Some(Some(_))));
        // Some(None) → clear actor
        let req = TransitionRequest {
            to_state: ChangeState::Ready,
            actor: Some(None),
            reason: StateTransitionReason::ApplyPause,
        };
        assert!(matches!(req.actor, Some(None)));
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
