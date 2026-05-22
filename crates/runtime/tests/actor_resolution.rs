//! Actor 推導 cross-platform 行為測試。
//!
//! 對應 spec requirement「Actor SHALL be resolved by fallback chain when `--actor`
//! flag is omitted」。`os_user` / `host_id` 跨平台靠 `whoami` crate，本測只斷言
//! fallback 字串永遠非空、不會 panic。
//!
//! 注意：cargo test 預設 parallel；本檔測試共用 `SPECLINK_AGENT_HOST` env var，
//! 因此用 `std::sync::Mutex` 序列化避免 race condition。

use std::sync::Mutex;

use speclink_runtime::resolve_actor;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn with_env<F: FnOnce()>(env_value: Option<&str>, body: F) {
    let _guard = ENV_LOCK.lock().expect("lock");
    let prev = std::env::var("SPECLINK_AGENT_HOST").ok();
    unsafe {
        match env_value {
            Some(v) => std::env::set_var("SPECLINK_AGENT_HOST", v),
            None => std::env::remove_var("SPECLINK_AGENT_HOST"),
        }
    }
    body();
    unsafe {
        match prev {
            Some(v) => std::env::set_var("SPECLINK_AGENT_HOST", v),
            None => std::env::remove_var("SPECLINK_AGENT_HOST"),
        }
    }
}

#[test]
fn explicit_agent_host_overrides_env() {
    with_env(Some("from-env"), || {
        let actor = resolve_actor(Some("from-arg"));
        assert_eq!(actor.agent_host, "from-arg");
    });
}

#[test]
fn env_used_when_no_arg() {
    with_env(Some("claude-code"), || {
        let actor = resolve_actor(None);
        assert_eq!(actor.agent_host, "claude-code");
    });
}

#[test]
fn fallback_to_cli_when_no_env_no_arg() {
    with_env(None, || {
        let actor = resolve_actor(None);
        assert_eq!(actor.agent_host, "cli");
    });
}

#[test]
fn empty_arg_treated_as_no_arg() {
    with_env(None, || {
        let actor = resolve_actor(Some("   "));
        assert_eq!(actor.agent_host, "cli");
    });
}

#[test]
fn os_user_and_host_id_are_always_populated_with_fallback() {
    let actor = resolve_actor(Some("any"));
    assert!(
        !actor.os_user.is_empty(),
        "os_user SHALL fall back to literal"
    );
    assert!(
        !actor.host_id.is_empty(),
        "host_id SHALL fall back to literal"
    );
}
