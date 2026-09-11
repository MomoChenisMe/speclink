//! HTTP 對外面（本檔只宣告子模組，無自身程式碼）：Command／Query 路由、唯讀
//! 查詢、動詞橋接、SSE 事件、Context 投影與 wire 錯誤信封。六個子模組以
//! `lib.rs` 的根層 `pub use` 掛回舊名，`speclink_server::routes` 這類路徑不變。

pub mod context;
pub mod error;
pub mod events;
pub mod read_api;
pub mod routes;
pub mod verb;
