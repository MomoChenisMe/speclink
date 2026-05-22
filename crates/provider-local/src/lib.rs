//! SpecLink LocalProvider 實作：將 artifact 寫入 `.speclink/`、state 寫入
//! `<git-common-dir>/speclink/`。

#![allow(clippy::doc_markdown)]

pub mod artifact_store;
pub mod change_store;
pub mod link_yaml;
pub mod paths;
pub mod state_db;
pub mod store;

pub use artifact_store::LocalArtifactStore;
pub use change_store::LocalChangeStore;
pub use state_db::{MIGRATIONS, StateDb, StateDbError};
pub use store::LocalProjectStore;
