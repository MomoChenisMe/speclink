//! Runtime crate — propose / artifact / status 等 AI workflow 的純編排層。
//!
//! 此 crate 不直接持有 filesystem / SQLite 等資源，所有 I/O 由 `Provider` trait 抽象。

pub mod archive;
pub mod artifact;
pub mod instructions;
pub mod propose;
pub mod spec_delta;
pub mod status;
pub mod task;
pub mod tasks_parser;
