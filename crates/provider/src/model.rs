//! SpecLink 共用資料模型：[`Project`]、[`ProjectId`]、[`ChangeId`]、[`Change`]、[`NewChange`]、
//! [`Artifact`]、[`NewArtifact`]、[`State`]。
//!
//! 所有型別以 serde 採 camelCase 序列化，作為 CLI、runtime、provider 之間
//! 跨層交換的穩定格式，並對應 `metadata.json` / JSON envelope schema。
//!
//! `ProjectId` 與 `ChangeId` 以 transparent newtype 包裝 [`String`]，避免裸 [`String`]
//! 跨界帶來的型別混淆；由於兩者已是獨立名義型別，不需要 `PhantomData`。

use serde::{Deserialize, Serialize};
use std::fmt;

/// SpecLink 專案識別碼。
///
/// 以 transparent newtype 包裝 [`String`]，序列化結果為純字串。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProjectId(String);

impl ProjectId {
    /// 取得內部字串切片。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ProjectId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for ProjectId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ProjectId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Change（變更提案）識別碼。
///
/// 必須是 kebab-case；驗證規則由 `provider-local` crate 的
/// `is_valid_change_id` 實作。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ChangeId(String);

impl ChangeId {
    /// 取得內部字串切片。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ChangeId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for ChangeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ChangeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Display for ChangeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// SpecLink 專案。MVP 僅保留識別碼與顯示名稱兩個欄位。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// 專案識別碼。
    pub id: ProjectId,
    /// 顯示名稱。
    pub name: String,
}

/// Change 在 SpecLink lifecycle 中的狀態。
///
/// 本 change 僅實作 `draft → proposed` 一步；其他狀態保留供後續 change 使用。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    /// 草稿；尚未呼叫 `propose create`。
    Draft,
    /// 已建立 proposal。
    Proposed,
}

/// 建立者中繼資訊。對應 `metadata.json` 中的 `createdBy` 欄位。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedBy {
    /// 建立者類別：`"agent"` 或 `"user"`。
    #[serde(rename = "type")]
    pub kind: String,
    /// 建立者顯示名稱；MVP 一律為空字串。
    pub name: String,
}

/// `propose create` 對 provider 提交的新 change 輸入。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewChange {
    /// Change 識別碼。
    pub change_id: ChangeId,
    /// Change 一行摘要，會寫入 `proposal.md` 的 `## Why` 區塊。
    pub summary: String,
}

/// 已建立的 change，對應 `metadata.json` 內容。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    /// Change 識別碼。
    pub change_id: ChangeId,
    /// 當前 lifecycle 狀態。
    pub state: State,
    /// 建立時間，ISO 8601 UTC 字串（秒精度，例如 `2026-05-19T12:34:56Z`）。
    pub created_at: String,
    /// 建立者資訊。
    pub created_by: CreatedBy,
}

/// Artifact 種類。MVP 僅 `Proposal`，其他種類保留供後續 change 使用。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactKind {
    /// `proposal.md`。
    Proposal,
    /// `design.md`。
    Design,
    /// `tasks.md`。
    Tasks,
    /// `specs/**/*.md`。
    Spec,
}

/// 對 provider 提交的新 artifact 內容。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewArtifact {
    /// Artifact 種類。
    pub kind: ArtifactKind,
    /// 文字內容（已序列化的 markdown）。
    pub content: String,
}

/// 已寫入的 artifact。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    /// Artifact 種類。
    pub kind: ArtifactKind,
    /// 相對於專案根目錄的 POSIX 風格路徑。
    pub path: String,
    /// 內容雜湊（格式由 provider 自行定義，例如 `sha256:...`）。
    pub content_hash: String,
}

#[cfg(test)]
mod tests {
    /// Round-trip helper：`value → JSON 字串 → 再 deserialize`，必須與原值相等。
    fn assert_round_trip<T>(value: T)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let json = serde_json::to_string(&value).expect("serialize");
        let back: T = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(value, back, "round-trip mismatch; json = {json}");
    }

    #[test]
    fn project_id_round_trip() {
        use crate::model::ProjectId;
        let id = ProjectId::from("project-acme");
        assert_round_trip(id);
    }

    #[test]
    fn change_id_round_trip() {
        use crate::model::ChangeId;
        let id = ChangeId::from("add-order-export");
        assert_round_trip(id);
    }

    #[test]
    fn state_round_trip_proposed() {
        use crate::model::State;
        assert_round_trip(State::Proposed);
        // 序列化採 lowercase，例如 `"proposed"`
        let json = serde_json::to_string(&State::Proposed).unwrap();
        assert_eq!(json, "\"proposed\"");
    }

    #[test]
    fn project_round_trip() {
        use crate::model::{Project, ProjectId};
        let project = Project {
            id: ProjectId::from("project-acme"),
            name: "Acme".to_string(),
        };
        assert_round_trip(project);
    }

    #[test]
    fn new_change_round_trip() {
        use crate::model::{ChangeId, NewChange};
        let new_change = NewChange {
            change_id: ChangeId::from("demo"),
            summary: "test summary".to_string(),
        };
        assert_round_trip(new_change);
    }

    #[test]
    fn change_round_trip() {
        use crate::model::{Change, ChangeId, CreatedBy, State};
        let change = Change {
            change_id: ChangeId::from("demo"),
            state: State::Proposed,
            created_at: "2026-05-19T12:34:56Z".to_string(),
            created_by: CreatedBy {
                kind: "agent".to_string(),
                name: String::new(),
            },
        };
        let json = serde_json::to_string(&change).unwrap();
        assert!(
            json.contains("\"changeId\":\"demo\""),
            "expected camelCase changeId field; got {json}"
        );
        assert!(
            json.contains("\"createdAt\":\"2026-05-19T12:34:56Z\""),
            "expected camelCase createdAt field; got {json}"
        );
        assert_round_trip(change);
    }

    #[test]
    fn new_artifact_round_trip() {
        use crate::model::{ArtifactKind, NewArtifact};
        let na = NewArtifact {
            kind: ArtifactKind::Proposal,
            content: "## Why\n\ntest\n".to_string(),
        };
        assert_round_trip(na);
    }

    #[test]
    fn artifact_round_trip() {
        use crate::model::{Artifact, ArtifactKind};
        let a = Artifact {
            kind: ArtifactKind::Proposal,
            path: ".speclink/changes/demo/proposal.md".to_string(),
            content_hash: "sha256:deadbeef".to_string(),
        };
        let json = serde_json::to_string(&a).unwrap();
        assert!(
            json.contains("\"contentHash\":\"sha256:deadbeef\""),
            "expected camelCase contentHash field; got {json}"
        );
        assert_round_trip(a);
    }
}
