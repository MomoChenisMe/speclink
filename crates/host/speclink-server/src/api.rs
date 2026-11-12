//! HTTP 對外面（本檔只宣告子模組，無自身程式碼）：轉交宿主套件 HTTP 入口的處理
//! 函式、留在 server 的匯入與 SSE 事件路由，以及 wire 錯誤信封。子模組以
//! `lib.rs` 的根層 `pub use` 掛回舊名，`speclink_server::routes` 這類路徑不變。

pub mod error;
pub mod events;
pub mod forward;
pub mod routes;
