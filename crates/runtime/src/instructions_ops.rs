//! `instructions.get` op — instruction body / template / dependency resolver.
//!
//! P1-3 slice `add-instructions-get` 落實 op id 32 (`instructions.get`)。
//! 從 `crates/runtime/src/embedded/` bundle 取 schema-static template /
//! instruction body；從 `ConfigStore::read` 取 best-effort `context` /
//! `rules.<kind>` / `locale`（fallback null）；提供 `--change` 時驗證
//! existence 並回 schema_id。
//!
//! 設計參考：
//! - `doc/speclink-design.md` §7 — Artifact DAG（spec-driven）
//! - `doc/speclink-design.md` §11.7 — Rules 注入機制
//! - `doc/speclink-design.md` §18.4 — Phase 1 P1-3 slice
//! - `doc/protocol/operations.md` §`instructions.get` — 11-field envelope

#![allow(clippy::doc_markdown)]

use std::path::Path;

use serde::Serialize;
use speclink_provider::{ChangeStore, Config, ConfigStore, ConfigWarning, ProviderError};
use speclink_provider_local::{LocalChangeStore, LocalConfigStore};

use crate::embedded::{instruction_for, template_for};
use crate::error::{RuntimeError, RuntimeWarning};
use crate::git::GitProbe;
use crate::paths::resolve_state_root;

/// Active schema id（MVP 階段 `instructions.get` 永遠回 `spec-driven`）。
///
/// Phase 2 `add-schema-ops` 引入 user schema fork 後，從 active schema state
/// 取代此 const。
pub const ACTIVE_SCHEMA_ID: &str = "spec-driven";

/// 支援 kind 的完整名單字串（給 error envelope `hint` 用）。
const SUPPORTED_KIND_HINT: &str =
    "Supported kinds: proposal, spec, design, tasks, apply, ingest, archive, commit";

/// 支援的 8 個 kind — 4 artifact kinds + 4 workflow phase kinds。
///
/// `discuss` / 其他字串在 `Kind::from_str` 解析階段以 `UnknownKind` 拒絕，
/// 對應 catalogue 第 32 條 input schema 內的 `discuss` enum 值是 Phase 2
/// `add-discuss-ops` 才實作；本 slice 一律 unknown_kind。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Artifact: proposal.md — establishes WHY。
    Proposal,
    /// Artifact: spec.md — normative WHAT。
    Spec,
    /// Artifact: design.md — HOW。
    Design,
    /// Artifact: tasks.md — implementation checklist。
    Tasks,
    /// Workflow phase: apply skill。
    Apply,
    /// Workflow phase: ingest skill。
    Ingest,
    /// Workflow phase: archive skill。
    Archive,
    /// Workflow phase: commit sub-flow。
    Commit,
}

/// `Kind::from_str` 對非支援字串回此 error；上層 dispatch 轉成
/// `instructions.unknown_kind` error code。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownKind {
    /// User 傳進來的原始 kind 字串（保留以填 error message）。
    pub kind: String,
}

/// Artifact DAG 邊：runtime 用此型別組 `dependencies[]` wire payload。
///
/// `capability` 在本 slice 一律 `None`（multi-instance spec capability
/// 解析屬 Phase 2 `add-spec-canonical-read`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Dependency {
    /// 依賴 kind（如 `proposal` / `spec` / `design`）。
    pub kind: &'static str,
    /// 對 multi-instance artifact 用；本 slice 永遠 `None`。
    pub capability: Option<&'static str>,
    /// 依賴 artifact 的 output_path（如 `proposal.md`）。
    pub path: &'static str,
}

/// `instructions.get` 的 input。
///
/// CLI parse 階段填入；`role` / `discussion_id` 在 P1-3 接受但忽略（reserved
/// for Phase 2 `add-discuss-ops`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    /// 必填。任何 8 個支援 kind 之外的字串走 `Error::UnknownKind`（含 `discuss`）。
    pub kind: String,
    /// 可選 change context；提供時 op 走 change existence check（Group 5）。
    pub change_id: Option<String>,
    /// 接受但忽略（Phase 2）。
    pub role: Option<String>,
    /// 接受但忽略（Phase 2）。
    pub discussion_id: Option<String>,
}

/// `instructions.get` 的 output — 對齊 `operations.md` 內 11-field stable envelope。
///
/// 序列化用 `snake_case`（per ops envelope 慣例）。Wire-format 只走 serialize
/// 方向（read-only op、無 deserialize round-trip 需求）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Output {
    pub kind: String,
    pub schema_id: String,
    pub instruction: String,
    /// Artifact kinds 為 `Some(template body)`；workflow phase kinds 為 `None`。
    pub template: Option<String>,
    /// `Config.context`；config 缺失或未設定為 `None`。
    pub context: Option<String>,
    /// `Config.instructions[kind]`；缺 key 為 `None`、empty Vec 保留為 `Some(vec![])`。
    pub rules: Option<Vec<String>>,
    /// 對應 §7 artifact DAG static table；`capability` 永遠 None（Phase 2）。
    pub dependencies: Vec<Dependency>,
    /// Artifact kinds 為 `Some("<kind>.md")`；workflow phase kinds 為 `None`。
    pub output_path: Option<String>,
    /// `Config.locale`；config 未設定為 `None`。
    pub locale: Option<String>,
    /// 永遠為 `None`（Phase 2 `add-discuss-ops` 才實作）。
    pub available_roles: Option<Vec<serde_json::Value>>,
    /// 永遠為 `None`（Phase 2 `add-discuss-ops` 才實作）。
    pub linked_changes_context: Option<Vec<serde_json::Value>>,
}

/// `instructions.get` 對外 error variants。
///
/// Wire-format error code：`UnknownKind` → `instructions.unknown_kind`、
/// `ChangeNotFound` → `change.not_found`。對應 §17 exit code 2 範疇。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// `<kind>` 不屬於當前 schema 支援集合（含 `discuss`、typo）。
    #[error("unknown kind: {kind}")]
    UnknownKind {
        /// User 傳進來的原始 kind 字串。
        kind: String,
        /// 給 error envelope `hint` 用的支援 kind 名單。
        hint: String,
    },
    /// 提供 `--change <id>` 但 change row 不存在（Group 5 觸發）。
    #[error("change not found: {change_id}")]
    ChangeNotFound {
        /// Caller 傳的 change id。
        change_id: String,
    },
}

impl Error {
    /// Wire-format error code（對應 `doc/speclink-design.md` §17 reference table）。
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownKind { .. } => "instructions.unknown_kind",
            Self::ChangeNotFound { .. } => "change.not_found",
        }
    }
}

/// `instructions.get` 主 dispatch — 對齊 spec Requirement「`speclink instructions
/// <kind>` SHALL return an 11-field envelope」。
///
/// Group 5 階段：kind dispatch + template/instruction lookup + dependencies +
/// config 三欄 hydration（透過 `ConfigStore::read_config`）+ A5 fallback warnings
/// forward + change context existence check（`--change <id>` 提供時走
/// `ChangeStore::get_change`）。
///
/// # Errors
///
/// - `Error::UnknownKind` 當 `input.kind` 不屬於 8 個支援 kind。
/// - `Error::ChangeNotFound` 當 `input.change_id` 提供但 change row 不存在。
pub async fn run(
    input: Input,
    config_store: &dyn ConfigStore,
    change_store: &dyn ChangeStore,
) -> Result<(Output, Vec<RuntimeWarning>), Error> {
    let kind = Kind::from_str(&input.kind).map_err(|e| Error::UnknownKind {
        kind: e.kind,
        hint: SUPPORTED_KIND_HINT.to_string(),
    })?;

    let schema_id = verify_change_context(input.change_id.as_deref(), change_store).await?;

    let (context, rules, locale, warnings) = hydrate_config_fields(kind, config_store);

    let instruction = instruction_for(kind.as_str())
        .expect("embedded bundle covers all 8 kinds (smoke test guards)")
        .to_string();

    let template = if kind.is_artifact_kind() {
        template_for(kind.as_str()).map(ToString::to_string)
    } else {
        None
    };

    let output_path = kind.output_path().map(ToString::to_string);

    Ok((
        Output {
            kind: kind.as_str().to_string(),
            schema_id,
            instruction,
            template,
            context,
            rules,
            locale,
            dependencies: kind.dependencies().to_vec(),
            output_path,
            // Phase 2 才實作
            available_roles: None,
            linked_changes_context: None,
        },
        warnings,
    ))
}

/// Change context existence check + schema_id resolution。
///
/// 對應 spec Requirement「verify change existence when `--change <id>` is provided
/// and reject missing changes with `change.not_found`」與 Decision「Change context
/// 插值範圍限縮在『change exists check』+ output payload meta echo」。
///
/// - `None` → 不查 store，回 `ACTIVE_SCHEMA_ID`。
/// - `Some(id)` 且 store 找到 → 回 change row 的 `schema_id`（MVP 永遠 spec-driven）。
/// - `Some(id)` 且 store `ChangeNotFound` → `Err(Error::ChangeNotFound)`。
///
/// 其他 `ProviderError` 走 fallback：回 `ACTIVE_SCHEMA_ID`，不阻斷請求（`instructions.get`
/// 是 read-only meta op，不該因 state.db transient 失敗 fail；非 not_found 情境屬
/// 系統錯誤，由其他 op 自然踩到）。
async fn verify_change_context(
    change_id: Option<&str>,
    change_store: &dyn ChangeStore,
) -> Result<String, Error> {
    let Some(id) = change_id else {
        return Ok(ACTIVE_SCHEMA_ID.to_string());
    };
    match change_store.get_change(id).await {
        Ok(row) => Ok(row.schema_id),
        Err(ProviderError::ChangeNotFound { .. }) => Err(Error::ChangeNotFound {
            change_id: id.to_string(),
        }),
        Err(_) => Ok(ACTIVE_SCHEMA_ID.to_string()),
    }
}

/// 從 `ConfigStore::read_config` 收集 `context` / `instructions[kind]` / `locale`
/// 三欄，同時 forward A5 的 fallback warnings。
///
/// A5 `LocalConfigStore` 對 config 缺失 / malformed 已內建 fallback to defaults +
/// `config.malformed_using_defaults` warning（不 raise error）。`Provider` trait 的
/// `read_config` 仍宣告 `Result<_, ProviderError>` — 若真實作回 `Err`，本函式
/// 採 defensive fallback 到 `Config::default()`、三欄皆 None，**不** propagate
/// error（`instructions.get` 是 read-only meta op，永遠不該因 config 不完整 fail）。
fn hydrate_config_fields(
    kind: Kind,
    config_store: &dyn ConfigStore,
) -> (
    Option<String>,
    Option<Vec<String>>,
    Option<String>,
    Vec<RuntimeWarning>,
) {
    let config = match config_store.read_config() {
        Ok(v) => v.value,
        Err(_) => Config::default(),
    };
    let warnings = convert_config_warnings(config_store.take_warnings());
    let context = config.context;
    let locale = config.locale;
    let rules = config.instructions.get(kind.as_str()).cloned();
    (context, rules, locale, warnings)
}

/// CLI-facing wrapper：建立 LocalConfigStore + LocalChangeStore、呼 `run`、
/// 把 `instructions_ops::Error` 翻成 `RuntimeError`。
///
/// 對齊既有 `ChangeOperations` / `ConfigOperations` pattern（struct + GitProbe
/// 泛型 + build_*_store helper）。
pub struct InstructionsOperations<G: GitProbe> {
    git: G,
}

impl<G: GitProbe> InstructionsOperations<G> {
    /// 建立 handle。不接觸 disk。
    pub fn new(git: G) -> Self {
        Self { git }
    }

    fn build_stores(
        &self,
        working_dir: &Path,
    ) -> Result<(LocalConfigStore, LocalChangeStore), RuntimeError> {
        let state_root = resolve_state_root::<G>(&self.git, working_dir)?;
        Ok((
            LocalConfigStore::new(working_dir.to_path_buf(), state_root.clone()),
            LocalChangeStore::new(working_dir.to_path_buf(), state_root),
        ))
    }

    /// 執行 `instructions.get` op：建立 stores、呼 `run`、回 `(Output, warnings)`
    /// 或 `RuntimeError`。
    ///
    /// # Errors
    /// `InstructionsUnknownKind` / `ChangeNotFound` / `RequiresGit` / `Internal`。
    pub async fn get_instructions(
        &self,
        working_dir: &Path,
        input: Input,
    ) -> Result<(Output, Vec<RuntimeWarning>), RuntimeError> {
        let (config_store, change_store) = self.build_stores(working_dir)?;
        run(input, &config_store, &change_store)
            .await
            .map_err(map_instructions_error)
    }
}

/// 把 op-internal `Error` 翻成 `RuntimeError`。CLI 層只看 `RuntimeError`。
fn map_instructions_error(err: Error) -> RuntimeError {
    match err {
        Error::UnknownKind { kind, .. } => RuntimeError::InstructionsUnknownKind { kind },
        Error::ChangeNotFound { change_id } => RuntimeError::ChangeNotFound { name: change_id },
    }
}

fn convert_config_warnings(ws: Vec<ConfigWarning>) -> Vec<RuntimeWarning> {
    ws.into_iter()
        .map(|w| RuntimeWarning {
            code: w.code.to_string(),
            message: w.message,
            details: None,
        })
        .collect()
}

impl Kind {
    /// Parse kebab-case string 成 `Kind`。
    ///
    /// # Errors
    /// 非 8 個支援 kind（含 `discuss`、typo）回 `UnknownKind`。
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, UnknownKind> {
        match s {
            "proposal" => Ok(Self::Proposal),
            "spec" => Ok(Self::Spec),
            "design" => Ok(Self::Design),
            "tasks" => Ok(Self::Tasks),
            "apply" => Ok(Self::Apply),
            "ingest" => Ok(Self::Ingest),
            "archive" => Ok(Self::Archive),
            "commit" => Ok(Self::Commit),
            other => Err(UnknownKind {
                kind: other.to_string(),
            }),
        }
    }

    /// Wire-format kebab-case 字串。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposal => "proposal",
            Self::Spec => "spec",
            Self::Design => "design",
            Self::Tasks => "tasks",
            Self::Apply => "apply",
            Self::Ingest => "ingest",
            Self::Archive => "archive",
            Self::Commit => "commit",
        }
    }

    /// `true` 對 4 個 artifact kinds、`false` 對 4 個 workflow phase kinds。
    ///
    /// Artifact kinds 會回傳 template + output_path；phase kinds 兩者皆 None。
    #[must_use]
    pub fn is_artifact_kind(&self) -> bool {
        matches!(
            self,
            Self::Proposal | Self::Spec | Self::Design | Self::Tasks
        )
    }

    /// Artifact 寫入路徑（相對 change directory）；workflow phase kinds 回 `None`。
    #[must_use]
    pub fn output_path(&self) -> Option<&'static str> {
        match self {
            Self::Proposal => Some("proposal.md"),
            Self::Spec => Some("spec.md"),
            Self::Design => Some("design.md"),
            Self::Tasks => Some("tasks.md"),
            Self::Apply | Self::Ingest | Self::Archive | Self::Commit => None,
        }
    }

    /// 靜態硬表：對應 `doc/speclink-design.md` §7 spec-driven artifact DAG。
    ///
    /// `crates/runtime/src/embedded/schemas/spec-driven/schema.yaml` 載有同樣
    /// 的 DAG；`hardcoded_table_matches_embedded_schema_yaml` test 守同步。
    #[must_use]
    pub fn dependencies(&self) -> &'static [Dependency] {
        const PROPOSAL: Dependency = Dependency {
            kind: "proposal",
            capability: None,
            path: "proposal.md",
        };
        const SPEC: Dependency = Dependency {
            kind: "spec",
            capability: None,
            path: "spec.md",
        };
        const DESIGN: Dependency = Dependency {
            kind: "design",
            capability: None,
            path: "design.md",
        };
        const TASKS: Dependency = Dependency {
            kind: "tasks",
            capability: None,
            path: "tasks.md",
        };

        match self {
            Self::Proposal | Self::Commit => &[],
            Self::Spec => &[PROPOSAL],
            Self::Design => &[PROPOSAL, SPEC],
            Self::Tasks => &[PROPOSAL, SPEC, DESIGN],
            Self::Apply | Self::Ingest => &[PROPOSAL, SPEC, TASKS],
            Self::Archive => &[SPEC, TASKS],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----- Task 2.1: kind dependency table matches spec -----

    #[test]
    fn kind_dependency_table_matches_spec() {
        // Spec §「`instructions.get` SHALL derive `dependencies[]` from a static
        // artifact DAG table」表格：proposal=0, spec=1, design=2, tasks=3,
        // apply=3, ingest=3, archive=2, commit=0。
        assert_eq!(Kind::Proposal.dependencies().len(), 0);
        assert_eq!(Kind::Spec.dependencies().len(), 1);
        assert_eq!(Kind::Design.dependencies().len(), 2);
        assert_eq!(Kind::Tasks.dependencies().len(), 3);
        assert_eq!(Kind::Apply.dependencies().len(), 3);
        assert_eq!(Kind::Ingest.dependencies().len(), 3);
        assert_eq!(Kind::Archive.dependencies().len(), 2);
        assert_eq!(Kind::Commit.dependencies().len(), 0);
    }

    #[test]
    fn kind_dependencies_have_expected_kinds_in_order() {
        // 細粒度驗證：spec 表內每個 entry 的 kind 順序與內容。
        assert_eq!(
            Kind::Tasks
                .dependencies()
                .iter()
                .map(|d| d.kind)
                .collect::<Vec<_>>(),
            vec!["proposal", "spec", "design"]
        );
        assert_eq!(
            Kind::Tasks
                .dependencies()
                .iter()
                .map(|d| d.path)
                .collect::<Vec<_>>(),
            vec!["proposal.md", "spec.md", "design.md"]
        );
        assert_eq!(
            Kind::Archive
                .dependencies()
                .iter()
                .map(|d| d.kind)
                .collect::<Vec<_>>(),
            vec!["spec", "tasks"]
        );
    }

    #[test]
    fn dependencies_capability_is_always_none() {
        for k in [
            Kind::Proposal,
            Kind::Spec,
            Kind::Design,
            Kind::Tasks,
            Kind::Apply,
            Kind::Ingest,
            Kind::Archive,
            Kind::Commit,
        ] {
            for d in k.dependencies() {
                assert!(
                    d.capability.is_none(),
                    "kind {:?} has non-None capability {:?}",
                    k,
                    d.capability
                );
            }
        }
    }

    // ----- Task 2.2 supplements: as_str / is_artifact_kind / output_path -----

    #[test]
    fn as_str_roundtrip_with_from_str() {
        for k in [
            Kind::Proposal,
            Kind::Spec,
            Kind::Design,
            Kind::Tasks,
            Kind::Apply,
            Kind::Ingest,
            Kind::Archive,
            Kind::Commit,
        ] {
            assert_eq!(Kind::from_str(k.as_str()).unwrap(), k);
        }
    }

    #[test]
    fn is_artifact_kind_true_for_four_kinds() {
        assert!(Kind::Proposal.is_artifact_kind());
        assert!(Kind::Spec.is_artifact_kind());
        assert!(Kind::Design.is_artifact_kind());
        assert!(Kind::Tasks.is_artifact_kind());
        assert!(!Kind::Apply.is_artifact_kind());
        assert!(!Kind::Ingest.is_artifact_kind());
        assert!(!Kind::Archive.is_artifact_kind());
        assert!(!Kind::Commit.is_artifact_kind());
    }

    #[test]
    fn output_path_matches_artifact_kind_membership() {
        assert_eq!(Kind::Proposal.output_path(), Some("proposal.md"));
        assert_eq!(Kind::Spec.output_path(), Some("spec.md"));
        assert_eq!(Kind::Design.output_path(), Some("design.md"));
        assert_eq!(Kind::Tasks.output_path(), Some("tasks.md"));
        for phase in [Kind::Apply, Kind::Ingest, Kind::Archive, Kind::Commit] {
            assert_eq!(phase.output_path(), None);
        }
    }

    // ----- Task 3.5 prep: unknown kind rejection in parser -----

    #[test]
    fn from_str_rejects_discuss_kind() {
        // 對應 Requirement: `instructions.get` SHALL reject unknown kinds with
        // `instructions.unknown_kind` and exit 2 — discuss 在 P1-3 屬 unknown。
        let err = Kind::from_str("discuss").unwrap_err();
        assert_eq!(err.kind, "discuss");
    }

    #[test]
    fn from_str_rejects_arbitrary_string() {
        let err = Kind::from_str("xyz_random_typo").unwrap_err();
        assert_eq!(err.kind, "xyz_random_typo");
        // empty string 也屬 unknown
        assert!(Kind::from_str("").is_err());
    }

    // ----- Task 4.1 + 5.1: MockConfigStore + MockChangeStore + 共用 test helpers -----

    use speclink_provider::{ChangeRow, Etag, ProviderError, Versioned, WriteConfigRequest};
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Test double — caller 透過 builder 設定 read_config / take_warnings 行為。
    /// 不實作 write_config（本 op 只走 read path；誤呼直接 panic）。
    ///
    /// `Mutex` 滿足 `ConfigStore: Send + Sync`、避免 `unsafe impl Sync`。
    struct MockConfigStore {
        config: Config,
        warnings: Mutex<Vec<ConfigWarning>>,
        read_invocations: Mutex<u32>,
    }

    impl MockConfigStore {
        fn new(config: Config, warnings: Vec<ConfigWarning>) -> Self {
            Self {
                config,
                warnings: Mutex::new(warnings),
                read_invocations: Mutex::new(0),
            }
        }
        fn default_empty() -> Self {
            Self::new(Config::default(), vec![])
        }
        fn read_count(&self) -> u32 {
            *self.read_invocations.lock().expect("MockConfigStore mutex")
        }
    }

    impl ConfigStore for MockConfigStore {
        fn read_config(&self) -> Result<Versioned<Config>, ProviderError> {
            *self.read_invocations.lock().expect("MockConfigStore mutex") += 1;
            Ok(Versioned {
                value: self.config.clone(),
                etag: Etag::from_literal("v1.deadbeefdead".to_string()),
            })
        }
        fn write_config(
            &self,
            _request: WriteConfigRequest,
        ) -> Result<Versioned<Config>, ProviderError> {
            unreachable!("instructions_ops should never call ConfigStore::write_config")
        }
        fn read_defaults(&self) -> Config {
            Config::default()
        }
        fn take_warnings(&self) -> Vec<ConfigWarning> {
            std::mem::take(&mut self.warnings.lock().expect("MockConfigStore mutex"))
        }
    }

    /// MockChangeStore — only `get_change` 實作；其他 method panic。
    /// 用 `changes` HashMap 模擬「change 存在 / 不存在」、`get_invocations` 計數。
    ///
    /// `Mutex` 滿足 `ChangeStore: Send + Sync`、避免 `unsafe impl Sync`。
    struct MockChangeStore {
        changes: HashMap<String, ChangeRow>,
        get_invocations: Mutex<u32>,
    }

    impl MockChangeStore {
        fn empty() -> Self {
            Self {
                changes: HashMap::new(),
                get_invocations: Mutex::new(0),
            }
        }
        fn with_change(name: &str, schema_id: &str) -> Self {
            let mut changes = HashMap::new();
            changes.insert(
                name.to_string(),
                ChangeRow {
                    change_id: name.to_string(),
                    name: name.to_string(),
                    state: "ready".to_string(),
                    schema_id: schema_id.to_string(),
                    version: 1,
                    created_at: "2026-05-23T00:00:00Z".to_string(),
                    updated_at: "2026-05-23T00:00:00Z".to_string(),
                },
            );
            Self {
                changes,
                get_invocations: Mutex::new(0),
            }
        }
        fn get_count(&self) -> u32 {
            *self.get_invocations.lock().expect("MockChangeStore mutex")
        }
    }

    #[async_trait::async_trait]
    impl ChangeStore for MockChangeStore {
        async fn create_change(
            &self,
            _name: &str,
            _schema_id: &str,
        ) -> Result<ChangeRow, ProviderError> {
            unreachable!("instructions_ops should never call ChangeStore::create_change")
        }
        async fn list_changes(&self) -> Result<Vec<ChangeRow>, ProviderError> {
            unreachable!("instructions_ops should never call ChangeStore::list_changes")
        }
        async fn get_change(&self, name: &str) -> Result<ChangeRow, ProviderError> {
            *self.get_invocations.lock().expect("MockChangeStore mutex") += 1;
            self.changes
                .get(name)
                .cloned()
                .ok_or_else(|| ProviderError::ChangeNotFound {
                    name: name.to_string(),
                })
        }
        async fn delete_change(&self, _name: &str) -> Result<(), ProviderError> {
            unreachable!("instructions_ops should never call ChangeStore::delete_change")
        }
    }

    // ----- Task 3.1-3.5: instructions_ops::run dispatch -----

    fn make_input(kind: &str) -> Input {
        Input {
            kind: kind.to_string(),
            change_id: None,
            role: None,
            discussion_id: None,
        }
    }

    /// Call run() with default empty config store + empty change store.
    /// Returns Output only — drops warnings (for tests that don't care).
    async fn run_default(input: Input) -> Result<Output, Error> {
        let config = MockConfigStore::default_empty();
        let change = MockChangeStore::empty();
        run(input, &config, &change).await.map(|(out, _)| out)
    }

    #[tokio::test]
    async fn run_returns_artifact_kind_envelope() {
        // 對應 Requirement「11-field envelope for supported artifact and workflow
        // phase kinds」+ scenario「Get proposal instructions (artifact kind)」。
        for (kind, expected_path) in [
            ("proposal", "proposal.md"),
            ("spec", "spec.md"),
            ("design", "design.md"),
            ("tasks", "tasks.md"),
        ] {
            let out = run_default(make_input(kind)).await.unwrap();
            assert_eq!(out.kind, kind);
            assert_eq!(out.schema_id, "spec-driven");
            assert!(!out.instruction.is_empty(), "instruction empty for {kind}");
            assert!(
                out.template.as_ref().is_some_and(|t| !t.is_empty()),
                "template missing or empty for {kind}"
            );
            assert_eq!(out.output_path.as_deref(), Some(expected_path));
            assert!(out.available_roles.is_none());
            assert!(out.linked_changes_context.is_none());
            // Group 4 之前 context/rules/locale 為 None
            assert!(out.context.is_none());
            assert!(out.rules.is_none());
            assert!(out.locale.is_none());
        }
    }

    #[tokio::test]
    async fn run_returns_phase_kind_envelope_with_null_template() {
        // 對應 scenario「Get apply instructions (workflow phase kind)」。
        for (kind, expected_dep_count) in
            [("apply", 3), ("ingest", 3), ("archive", 2), ("commit", 0)]
        {
            let out = run_default(make_input(kind)).await.unwrap();
            assert_eq!(out.kind, kind);
            assert!(
                out.template.is_none(),
                "phase {kind} should have null template"
            );
            assert!(
                out.output_path.is_none(),
                "phase {kind} should have null output_path"
            );
            assert!(!out.instruction.is_empty(), "instruction empty for {kind}");
            assert_eq!(
                out.dependencies.len(),
                expected_dep_count,
                "phase {kind} dep count mismatch"
            );
        }
    }

    #[tokio::test]
    async fn run_artifact_dependencies_match_dag_table() {
        // 對應 spec Requirement「derive `dependencies[]` from a static artifact
        // DAG table」+ scenario「Tasks dependencies include all three predecessors」。
        let tasks = run_default(make_input("tasks")).await.unwrap();
        assert_eq!(tasks.dependencies.len(), 3);
        let kinds: Vec<&str> = tasks.dependencies.iter().map(|d| d.kind).collect();
        assert_eq!(kinds, vec!["proposal", "spec", "design"]);
        let paths: Vec<&str> = tasks.dependencies.iter().map(|d| d.path).collect();
        assert_eq!(paths, vec!["proposal.md", "spec.md", "design.md"]);
        for d in &tasks.dependencies {
            assert!(d.capability.is_none());
        }
    }

    #[tokio::test]
    async fn run_returns_unknown_kind_error_for_discuss_and_typo() {
        // 對應 Requirement「reject unknown kinds with `instructions.unknown_kind`
        // and exit 2」+ scenarios「Reject `discuss` kind」/「Reject arbitrary string」。
        for bad in ["discuss", "random_kind_xyz", "", "Proposal", "ARCHIVE"] {
            let err = run_default(make_input(bad)).await.unwrap_err();
            match err {
                Error::UnknownKind { ref kind, ref hint } => {
                    assert_eq!(kind, bad, "kind echo mismatch for {bad}");
                    assert!(
                        hint.contains("proposal")
                            && hint.contains("spec")
                            && hint.contains("design")
                            && hint.contains("tasks")
                            && hint.contains("apply")
                            && hint.contains("ingest")
                            && hint.contains("archive")
                            && hint.contains("commit"),
                        "hint missing supported kinds for {bad}: {hint}"
                    );
                }
                _ => panic!("expected UnknownKind for {bad}, got {err:?}"),
            }
            assert_eq!(err.code(), "instructions.unknown_kind");
        }
    }

    #[tokio::test]
    async fn run_change_id_without_existing_change_returns_not_found() {
        // Group 5 後 change_id 不再是 no-op：empty MockChangeStore + change_id 提供
        // → 預期 Err(ChangeNotFound)。
        let mut input = make_input("proposal");
        input.change_id = Some("anything".to_string());
        let err = run_default(input).await.unwrap_err();
        match err {
            Error::ChangeNotFound { change_id } => assert_eq!(change_id, "anything"),
            _ => panic!("expected ChangeNotFound, got {err:?}"),
        }
    }

    #[tokio::test]
    async fn run_role_and_discussion_id_accepted_but_ignored() {
        // 對應 Requirement「`--role` and `--discussion` flags SHALL be accepted
        // by the CLI surface but ignored by the dispatcher」+ scenario「--role is
        // accepted but ignored」。
        let input = Input {
            kind: "proposal".to_string(),
            change_id: None,
            role: Some("pm".to_string()),
            discussion_id: Some("abc-123".to_string()),
        };
        let out = run_default(input).await.unwrap();
        // 不報 error、available_roles / linked_changes_context 仍恆 None
        assert!(out.available_roles.is_none());
        assert!(out.linked_changes_context.is_none());
    }

    // ----- Task 4.2-4.5: config hydration -----

    #[tokio::test]
    async fn run_config_file_missing_returns_all_null() {
        // 對應 spec scenario「Config file does not exist」+ Decision: Config 三欄
        // 採 best-effort read + fallback null。
        // A5 LocalConfigStore 對 file missing 回 Config::default() — 我們直接用
        // default_empty 模擬該行為。
        let cfg_store = MockConfigStore::default_empty();
        let change_store = MockChangeStore::empty();
        let (out, warnings) = run(make_input("proposal"), &cfg_store, &change_store)
            .await
            .unwrap();
        assert!(
            out.context.is_none(),
            "context should be None for missing config"
        );
        assert!(
            out.rules.is_none(),
            "rules should be None for missing config"
        );
        assert!(
            out.locale.is_none(),
            "locale should be None for missing config"
        );
        assert!(warnings.is_empty(), "no warnings on clean default config");
    }

    #[tokio::test]
    async fn run_config_partial_keys_each_field_independent() {
        // 對應 scenario「Config exists with partial keys」: 只設 locale。
        let cfg = Config {
            locale: Some("Traditional Chinese (繁體中文)".to_string()),
            context: None,
            instructions: HashMap::new(),
            ..Config::default()
        };
        let cfg_store = MockConfigStore::new(cfg, vec![]);
        let change_store = MockChangeStore::empty();
        let (out, _) = run(make_input("proposal"), &cfg_store, &change_store)
            .await
            .unwrap();
        assert_eq!(
            out.locale.as_deref(),
            Some("Traditional Chinese (繁體中文)")
        );
        assert!(out.context.is_none());
        assert!(out.rules.is_none());
    }

    #[tokio::test]
    async fn run_instructions_empty_array_vs_null() {
        // 對應 scenario 表格: explicit empty vec → Some(vec![]); missing key → None。
        let change_store = MockChangeStore::empty();

        let mut instructions_map = HashMap::new();
        instructions_map.insert("proposal".to_string(), Vec::<String>::new());
        let cfg_with_empty = Config {
            instructions: instructions_map,
            ..Config::default()
        };
        let store_empty = MockConfigStore::new(cfg_with_empty, vec![]);
        let (out_empty, _) = run(make_input("proposal"), &store_empty, &change_store)
            .await
            .unwrap();
        assert_eq!(
            out_empty.rules,
            Some(vec![]),
            "explicit empty array should be Some(vec![])"
        );

        let cfg_missing = Config {
            instructions: HashMap::new(),
            ..Config::default()
        };
        let store_missing = MockConfigStore::new(cfg_missing, vec![]);
        let (out_missing, _) = run(make_input("proposal"), &store_missing, &change_store)
            .await
            .unwrap();
        assert!(out_missing.rules.is_none(), "missing key should be None");
    }

    #[tokio::test]
    async fn run_per_kind_rules_isolated_to_requested_kind() {
        // 額外驗證: 不同 kind 各自查自己的 entry。
        let mut instructions_map = HashMap::new();
        instructions_map.insert("proposal".to_string(), vec!["Don't use emoji".to_string()]);
        instructions_map.insert(
            "tasks".to_string(),
            vec!["Verification target required".to_string()],
        );
        let cfg = Config {
            instructions: instructions_map,
            ..Config::default()
        };
        let cfg_store = MockConfigStore::new(cfg, vec![]);
        let change_store = MockChangeStore::empty();

        let (out_proposal, _) = run(make_input("proposal"), &cfg_store, &change_store)
            .await
            .unwrap();
        assert_eq!(
            out_proposal.rules,
            Some(vec!["Don't use emoji".to_string()])
        );

        let (out_spec, _) = run(make_input("spec"), &cfg_store, &change_store)
            .await
            .unwrap();
        assert!(out_spec.rules.is_none(), "spec has no entry");

        let (out_tasks, _) = run(make_input("tasks"), &cfg_store, &change_store)
            .await
            .unwrap();
        assert_eq!(
            out_tasks.rules,
            Some(vec!["Verification target required".to_string()])
        );
    }

    #[tokio::test]
    async fn run_config_malformed_forwards_warning() {
        // 對應 scenario「Config malformed forwards A5 warning」: A5 對 malformed
        // 已 fallback to defaults + emit `config.malformed_using_defaults` warning（不
        // raise error）；我們 forward 該 warning 到 envelope。
        let warnings = vec![ConfigWarning {
            code: "config.malformed_using_defaults",
            message: "config.yaml YAML parse failed; using defaults".to_string(),
        }];
        let cfg_store = MockConfigStore::new(Config::default(), warnings);
        let change_store = MockChangeStore::empty();
        let (out, runtime_warnings) = run(make_input("proposal"), &cfg_store, &change_store)
            .await
            .unwrap();

        // 三欄 fallback null
        assert!(out.context.is_none());
        assert!(out.rules.is_none());
        assert!(out.locale.is_none());

        // warning forwarded
        assert_eq!(runtime_warnings.len(), 1);
        assert_eq!(runtime_warnings[0].code, "config.malformed_using_defaults");
        assert!(
            runtime_warnings[0].message.contains("YAML parse failed"),
            "warning message preserved"
        );
    }

    #[tokio::test]
    async fn run_invokes_read_config_once() {
        // sanity: hydrate_config_fields 只呼一次 read_config。
        let cfg_store = MockConfigStore::default_empty();
        let change_store = MockChangeStore::empty();
        let _ = run(make_input("proposal"), &cfg_store, &change_store)
            .await
            .unwrap();
        assert_eq!(cfg_store.read_count(), 1);
    }

    // ----- Task 5.1-5.4: change context existence check -----

    #[tokio::test]
    async fn run_with_change_id_calls_change_store_and_passes_on_existence() {
        // 對應 Requirement「verify change existence when `--change <id>` is provided」
        // + Decision「Change context 插值範圍限縮在『change exists check』+ output
        // payload meta echo」。
        let cfg_store = MockConfigStore::default_empty();
        let change_store = MockChangeStore::with_change("my-feature", "spec-driven");

        let mut input = make_input("proposal");
        input.change_id = Some("my-feature".to_string());

        let (out, _) = run(input, &cfg_store, &change_store).await.unwrap();
        assert_eq!(out.schema_id, "spec-driven");
        assert_eq!(change_store.get_count(), 1);
    }

    #[tokio::test]
    async fn run_without_change_id_does_not_invoke_change_store() {
        // 對應 scenario「No --change flag does not invoke ChangeStore」。
        let cfg_store = MockConfigStore::default_empty();
        let change_store = MockChangeStore::empty();

        let input = make_input("proposal"); // change_id: None
        let (out, _) = run(input, &cfg_store, &change_store).await.unwrap();

        assert_eq!(out.schema_id, "spec-driven");
        assert_eq!(
            change_store.get_count(),
            0,
            "change_store should NOT be invoked when change_id is None"
        );
    }

    #[tokio::test]
    async fn run_with_missing_change_id_returns_change_not_found() {
        // 對應 scenario「Change does not exist」 + Requirement「reject missing
        // changes with `change.not_found`」。
        let cfg_store = MockConfigStore::default_empty();
        let change_store = MockChangeStore::empty(); // no changes registered

        let mut input = make_input("proposal");
        input.change_id = Some("nonexistent-change".to_string());

        let err = run(input, &cfg_store, &change_store).await.unwrap_err();
        match err {
            Error::ChangeNotFound { ref change_id } => {
                assert_eq!(change_id, "nonexistent-change");
            }
            _ => panic!("expected ChangeNotFound, got {err:?}"),
        }
        assert_eq!(err.code(), "change.not_found");
    }

    #[tokio::test]
    async fn run_change_id_existence_check_runs_before_config_or_unknown_kind() {
        // 順序 sanity: kind 驗證在 change 之前（design choice — input validation
        // first），所以 unknown_kind + change_id=Some(missing) → unknown_kind 勝出。
        let cfg_store = MockConfigStore::default_empty();
        let change_store = MockChangeStore::empty();

        let input = Input {
            kind: "bogus".to_string(),
            change_id: Some("nonexistent".to_string()),
            role: None,
            discussion_id: None,
        };
        let err = run(input, &cfg_store, &change_store).await.unwrap_err();
        assert!(matches!(err, Error::UnknownKind { .. }));
    }

    // ----- Task 2.3: hardcoded table matches embedded schema.yaml DAG -----

    #[test]
    fn hardcoded_table_matches_embedded_schema_yaml() {
        // Spec scenario「Hardcoded dependency table matches schema.yaml DAG」:
        // 解析 EMBEDDED_SCHEMA_YAML、對 8 kind 逐一比對 dependency edges。
        use crate::embedded::EMBEDDED_SCHEMA_YAML;
        use std::collections::HashMap;

        #[derive(serde::Deserialize)]
        struct SchemaYaml {
            artifacts: Vec<SchemaEntry>,
            phases: Vec<SchemaEntry>,
        }
        #[derive(serde::Deserialize)]
        struct SchemaEntry {
            id: String,
            dependencies: Vec<String>,
        }

        let yaml: SchemaYaml =
            serde_yaml::from_str(EMBEDDED_SCHEMA_YAML).expect("schema.yaml parses");

        let mut yaml_deps: HashMap<String, Vec<String>> = HashMap::new();
        for entry in yaml.artifacts.into_iter().chain(yaml.phases) {
            yaml_deps.insert(entry.id, entry.dependencies);
        }

        for k in [
            Kind::Proposal,
            Kind::Spec,
            Kind::Design,
            Kind::Tasks,
            Kind::Apply,
            Kind::Ingest,
            Kind::Archive,
            Kind::Commit,
        ] {
            let yaml_edges = yaml_deps
                .get(k.as_str())
                .unwrap_or_else(|| panic!("schema.yaml missing entry for {}", k.as_str()));
            let hardcode_edges: Vec<&str> = k.dependencies().iter().map(|d| d.kind).collect();
            assert_eq!(
                yaml_edges,
                &hardcode_edges,
                "schema.yaml vs Kind::dependencies() mismatch for {}",
                k.as_str()
            );
        }
    }
}
