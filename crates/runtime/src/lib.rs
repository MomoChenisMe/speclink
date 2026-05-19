//! Runtime crate — propose 等 AI workflow 的純編排層。
//!
//! 此 crate 不直接持有 filesystem / SQLite 等資源，所有 I/O 由 `Provider` trait 抽象。

pub mod propose;
