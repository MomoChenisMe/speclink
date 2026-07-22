//! credential 進出的唯一介面（design 決策 2）：唯一落點 OS Keychain，Rust 側
//! 進出，TS 永遠看不到 secret。
//!
//! 鍵＝server origin＋credential 種類（refresh／pat）。生產實作走 keyring
//! （macOS Keychain／Windows Credential Manager，service 名固定
//! [`SERVICE`]）；CI 無 headless Keychain 可用，trait 注入是唯一可測形狀——
//! 測試與編排注入 [`MemoryCredentialStore`]。錯誤訊息絕不夾帶 secret 內容。

use std::collections::HashMap;
use std::sync::Mutex;

/// Keychain entry 的 service 名（手動驗收以
/// `security find-generic-password -s speclink-desktop` 對照）。
pub const SERVICE: &str = "speclink-desktop";

/// credential 種類：device 流程存 refresh credential、PAT 流程存 PAT。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CredentialKind {
    Refresh,
    Pat,
}

impl CredentialKind {
    fn slug(self) -> &'static str {
        match self {
            CredentialKind::Refresh => "refresh",
            CredentialKind::Pat => "pat",
        }
    }
}

/// 逐 origin＋種類的 credential 存取。實作必須冪等刪除：登出時 entry 已不在
/// 不是錯誤（本機清理不可被阻擋的語意一環）。
pub trait CredentialStore: Send + Sync {
    fn get(&self, origin: &str, kind: CredentialKind) -> Result<Option<String>, String>;
    fn set(&self, origin: &str, kind: CredentialKind, secret: &str) -> Result<(), String>;
    fn delete(&self, origin: &str, kind: CredentialKind) -> Result<(), String>;
}

/// in-memory 實作：測試與編排測試注入用。
#[derive(Default)]
pub struct MemoryCredentialStore {
    slots: Mutex<HashMap<(String, CredentialKind), String>>,
}

impl MemoryCredentialStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CredentialStore for MemoryCredentialStore {
    fn get(&self, origin: &str, kind: CredentialKind) -> Result<Option<String>, String> {
        Ok(self
            .slots
            .lock()
            .expect("credential lock")
            .get(&(origin.to_string(), kind))
            .cloned())
    }

    fn set(&self, origin: &str, kind: CredentialKind, secret: &str) -> Result<(), String> {
        self.slots
            .lock()
            .expect("credential lock")
            .insert((origin.to_string(), kind), secret.to_string());
        Ok(())
    }

    fn delete(&self, origin: &str, kind: CredentialKind) -> Result<(), String> {
        self.slots
            .lock()
            .expect("credential lock")
            .remove(&(origin.to_string(), kind));
        Ok(())
    }
}

/// keyring 生產實作：service 固定 [`SERVICE`]、account＝`<kind>:<origin>`。
pub struct KeyringCredentialStore;

fn entry(origin: &str, kind: CredentialKind) -> Result<keyring::Entry, String> {
    keyring::Entry::new(SERVICE, &format!("{}:{origin}", kind.slug()))
        .map_err(|e| format!("無法存取系統 Keychain：{e}"))
}

impl CredentialStore for KeyringCredentialStore {
    fn get(&self, origin: &str, kind: CredentialKind) -> Result<Option<String>, String> {
        match entry(origin, kind)?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(format!("無法讀取系統 Keychain：{e}")),
        }
    }

    fn set(&self, origin: &str, kind: CredentialKind, secret: &str) -> Result<(), String> {
        entry(origin, kind)?
            .set_password(secret)
            .map_err(|e| format!("無法寫入系統 Keychain：{e}"))
    }

    fn delete(&self, origin: &str, kind: CredentialKind) -> Result<(), String> {
        match entry(origin, kind)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(format!("無法刪除系統 Keychain entry：{e}")),
        }
    }
}
