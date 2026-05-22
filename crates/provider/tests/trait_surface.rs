//! Compile-only surface 測試：證明 `StateMachineStore` trait 簽章存在，且
//! `ChangeStore` trait 表面沒有任何 `change.state` / `change.version` setter。
//!
//! 對應 spec requirement「The `change-store` capability SHALL NOT expose any
//! direct setter for `change.state`」「The `change-store` capability SHALL NOT
//! mutate `version` directly」。任何違反此契約的修改會在 compile 階段被拒絕。

use speclink_provider::{
    Actor, ChangeRow, ChangeState, ChangeStateView, ChangeStore, ProviderError, StateMachineStore,
    StateTransitionReason, TransitionRequest,
};

#[allow(dead_code)]
struct SurfaceStateMachineStore;

#[async_trait::async_trait]
impl StateMachineStore for SurfaceStateMachineStore {
    async fn get_change_state(&self, _name: &str) -> Result<ChangeStateView, ProviderError> {
        Ok(ChangeStateView {
            change_id: "cid".into(),
            state: ChangeState::Proposing,
            version: 1,
            actor: None,
            all_tasks_done: false,
        })
    }
    async fn transition_state(
        &self,
        _name: &str,
        _expected_version: u64,
        _request: TransitionRequest,
    ) -> Result<ChangeStateView, ProviderError> {
        Err(ProviderError::Internal("stub".into()))
    }
    async fn set_actor(
        &self,
        _name: &str,
        _expected_version: u64,
        _actor: Option<Actor>,
    ) -> Result<ChangeStateView, ProviderError> {
        Err(ProviderError::Internal("stub".into()))
    }
    async fn set_all_tasks_done(
        &self,
        _name: &str,
        _expected_version: u64,
        _done: bool,
    ) -> Result<ChangeStateView, ProviderError> {
        Err(ProviderError::Internal("stub".into()))
    }
}

#[allow(dead_code)]
struct SurfaceChangeStore;

#[async_trait::async_trait]
impl ChangeStore for SurfaceChangeStore {
    async fn create_change(
        &self,
        _name: &str,
        _schema_id: &str,
    ) -> Result<ChangeRow, ProviderError> {
        Err(ProviderError::Internal("stub".into()))
    }
    async fn list_changes(&self) -> Result<Vec<ChangeRow>, ProviderError> {
        Ok(vec![])
    }
    async fn get_change(&self, _name: &str) -> Result<ChangeRow, ProviderError> {
        Err(ProviderError::Internal("stub".into()))
    }
    async fn delete_change(&self, _name: &str) -> Result<(), ProviderError> {
        Ok(())
    }
}

#[test]
fn state_machine_store_trait_is_object_safe_and_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Box<dyn StateMachineStore>>();
}

#[test]
fn transition_request_constructs_with_three_actor_semantics() {
    // 純 type check：confirm TransitionRequest 接受 3 種 actor 語意（None/Some(Some)/Some(None)）。
    let _none = TransitionRequest {
        to_state: ChangeState::InProgress,
        actor: None,
        reason: StateTransitionReason::ApplyStart,
    };
    let _assign = TransitionRequest {
        to_state: ChangeState::InProgress,
        actor: Some(Some(Actor {
            agent_host: "cli".into(),
            os_user: "alice".into(),
            host_id: "h".into(),
        })),
        reason: StateTransitionReason::ApplyStart,
    };
    let _clear = TransitionRequest {
        to_state: ChangeState::Ready,
        actor: Some(None),
        reason: StateTransitionReason::ApplyPause,
    };
}

#[test]
fn change_store_trait_does_not_expose_state_or_version_setter() {
    // Compile-time proof：所有 `ChangeStore` method 都不接受 state / version 作為輸入。
    // 若未來有人加 `update_state` / `set_version` method 進 ChangeStore，本檔會 build fail。
    fn type_check_change_store<T: ChangeStore>() {}
    type_check_change_store::<SurfaceChangeStore>();
}
