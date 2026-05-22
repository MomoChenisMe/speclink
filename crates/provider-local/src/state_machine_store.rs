//! `LocalStateMachineStore` — `StateMachineStore` trait 的 SQLite-backed 實作。
//!
//! 對齊 design.md 「Provider trait interface」決策：所有寫入 method 透過
//! `crates/provider-local/src/state_db.rs` 的 CAS helper 在單一 SQLite tx 內完成
//! `change` row update + `state_transition` audit insert；CAS 衝突映射為
//! [`ProviderError::StateVersionConflict`]。

#![allow(clippy::doc_markdown)]

use std::fs;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use speclink_provider::{
    Actor, ChangeState, ChangeStateView, ProviderError, StateMachineStore, TransitionRequest,
};
use uuid::Uuid;

use crate::state_db::{ActorUpdate, ChangeStateRow, StateDb, StateDbError, StateTransitionRow};

/// LocalProvider 的 `StateMachineStore` 實作。
pub struct LocalStateMachineStore {
    state_root: PathBuf,
}

impl LocalStateMachineStore {
    /// 建立 store handle；不接觸磁碟。
    #[must_use]
    pub fn new(state_root: PathBuf) -> Self {
        Self { state_root }
    }

    /// State root 路徑（state.db 所在目錄）。
    #[must_use]
    pub fn state_root(&self) -> &Path {
        &self.state_root
    }

    fn open_db(&self) -> Result<StateDb, ProviderError> {
        fs::create_dir_all(&self.state_root)
            .map_err(|e| ProviderError::Internal(format!("create state root: {e}")))?;
        let path = self.state_root.join("state.db");
        let db = StateDb::open(&path).map_err(|e| map_state_db_error(e, "open state.db"))?;
        db.migrate(4)
            .map_err(|e| map_state_db_error(e, "migrate state.db"))?;
        Ok(db)
    }
}

#[async_trait]
impl StateMachineStore for LocalStateMachineStore {
    async fn get_change_state(&self, name: &str) -> Result<ChangeStateView, ProviderError> {
        let db = self.open_db()?;
        let row = db
            .read_change_state_row(name)
            .map_err(|e| map_state_db_error(e, "read change state"))?;
        row_to_view(&row)
    }

    async fn transition_state(
        &self,
        name: &str,
        expected_version: u64,
        request: TransitionRequest,
    ) -> Result<ChangeStateView, ProviderError> {
        let db = self.open_db()?;
        let row = db
            .read_change_state_row(name)
            .map_err(|e| map_state_db_error(e, "read change state"))?;
        let change_id = row.change_id.clone();
        let from_state = row.state.clone();
        let actor_json_owned = match &request.actor {
            Some(Some(a)) => Some(actor_to_json(a)?),
            _ => None,
        };
        let actor_update = match &request.actor {
            None => ActorUpdate::Keep,
            Some(None) => ActorUpdate::Set(None),
            Some(Some(_)) => ActorUpdate::Set(actor_json_owned.as_deref()),
        };
        let to_state_str = request.to_state.as_str();
        let now = now_rfc3339();
        let transition_id = Uuid::new_v4().to_string();
        let audit = StateTransitionRow {
            transition_id: &transition_id,
            from_state: &from_state,
            to_state: to_state_str,
            actor_json: actor_json_owned.as_deref(),
            transitioned_at: &now,
            reason: request.reason.as_str(),
        };
        db.update_change_state_cas(
            &change_id,
            expected_version,
            to_state_str,
            actor_update,
            &audit,
            &now,
        )
        .map_err(|e| map_state_db_error(e, "transition state"))?;
        // re-read for canonical view
        let new_row = db
            .read_change_state_row(name)
            .map_err(|e| map_state_db_error(e, "re-read change state"))?;
        row_to_view(&new_row)
    }

    async fn set_actor(
        &self,
        name: &str,
        expected_version: u64,
        actor: Option<Actor>,
    ) -> Result<ChangeStateView, ProviderError> {
        let db = self.open_db()?;
        let row = db
            .read_change_state_row(name)
            .map_err(|e| map_state_db_error(e, "read change state"))?;
        let change_id = row.change_id.clone();
        let json_owned = match &actor {
            Some(a) => Some(actor_to_json(a)?),
            None => None,
        };
        let now = now_rfc3339();
        db.cas_set_actor(&change_id, expected_version, json_owned.as_deref(), &now)
            .map_err(|e| map_state_db_error(e, "set actor"))?;
        let new_row = db
            .read_change_state_row(name)
            .map_err(|e| map_state_db_error(e, "re-read change state"))?;
        row_to_view(&new_row)
    }

    async fn set_all_tasks_done(
        &self,
        name: &str,
        expected_version: u64,
        done: bool,
    ) -> Result<ChangeStateView, ProviderError> {
        let db = self.open_db()?;
        let row = db
            .read_change_state_row(name)
            .map_err(|e| map_state_db_error(e, "read change state"))?;
        let change_id = row.change_id.clone();
        let now = now_rfc3339();
        db.cas_set_all_tasks_done(&change_id, expected_version, done, &now)
            .map_err(|e| map_state_db_error(e, "set all_tasks_done"))?;
        let new_row = db
            .read_change_state_row(name)
            .map_err(|e| map_state_db_error(e, "re-read change state"))?;
        row_to_view(&new_row)
    }
}

fn row_to_view(row: &ChangeStateRow) -> Result<ChangeStateView, ProviderError> {
    let state: ChangeState = row
        .state
        .parse()
        .map_err(|_| ProviderError::StateInvalidValue {
            value: row.state.clone(),
        })?;
    let actor = match row.actor_json.as_deref() {
        None => None,
        Some(json) => Some(actor_from_json(json)?),
    };
    Ok(ChangeStateView {
        change_id: row.change_id.clone(),
        state,
        version: row.version,
        actor,
        all_tasks_done: row.all_tasks_done,
    })
}

fn actor_to_json(actor: &Actor) -> Result<String, ProviderError> {
    serde_json::to_string(actor).map_err(|e| ProviderError::Internal(format!("encode actor: {e}")))
}

fn actor_from_json(json: &str) -> Result<Actor, ProviderError> {
    serde_json::from_str(json).map_err(|e| ProviderError::Internal(format!("decode actor: {e}")))
}

fn map_state_db_error(e: StateDbError, ctx: &str) -> ProviderError {
    match e {
        StateDbError::CasConflict { current_version } => {
            ProviderError::StateVersionConflict { current_version }
        }
        StateDbError::ChangeRowNotFound { change_id } => {
            ProviderError::ChangeNotFound { name: change_id }
        }
        StateDbError::SchemaVersion { expected, found } => ProviderError::StateDbSchemaInvalid {
            found,
            supported: expected,
        },
        other => ProviderError::Internal(format!("{ctx}: {other}")),
    }
}

fn now_rfc3339() -> String {
    use time::OffsetDateTime;
    use time::format_description::well_known::Rfc3339;
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| String::from("1970-01-01T00:00:00Z"))
}
