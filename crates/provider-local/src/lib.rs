//! SpecLink LocalProvider 實作：將 artifact 寫入 `.speclink/`、state 寫入
//! `<git-common-dir>/speclink/`。

#![allow(clippy::doc_markdown)]

pub mod link_yaml;
pub mod state_db;
pub mod store;

pub use state_db::{MIGRATIONS, StateDb, StateDbError};
pub use store::LocalProjectStore;
