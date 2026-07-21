//! Tray 復原 action 的原生 fallback 邊界：直接 retry 不取得主視窗焦點；
//! 顯式詳情、設定與重新登入才允許喚起主視窗。未知 action 必須 fail-closed。

use speclink_desktop_lib::tray::TrayRecoveryAction;

#[test]
fn recovery_action_focus_policy_is_explicit_and_fail_closed() {
    assert!(!TrayRecoveryAction::Retry.focuses_main_window());
    assert!(TrayRecoveryAction::OpenRecovery.focuses_main_window());
    assert!(TrayRecoveryAction::OpenSettings.focuses_main_window());
    assert!(TrayRecoveryAction::Reauthenticate.focuses_main_window());

    let unknown = serde_json::from_str::<TrayRecoveryAction>(r#""future-action""#);
    assert!(unknown.is_err(), "未知 tray action 不得降階成會執行的既有動作");
}

#[test]
fn recovery_action_serialization_uses_closed_kebab_case_names() {
    assert_eq!(
        serde_json::to_string(&TrayRecoveryAction::Retry).unwrap(),
        r#""retry""#,
    );
    assert_eq!(
        serde_json::to_string(&TrayRecoveryAction::OpenRecovery).unwrap(),
        r#""open-recovery""#,
    );
    assert_eq!(
        serde_json::to_string(&TrayRecoveryAction::OpenSettings).unwrap(),
        r#""open-settings""#,
    );
    assert_eq!(
        serde_json::to_string(&TrayRecoveryAction::Reauthenticate).unwrap(),
        r#""reauthenticate""#,
    );
}
