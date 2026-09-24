//! speclink-core: spec-driven development engine.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// 三個資料夾模組本身是私有的：分組只是給讀者看的，不是公開 API。底下的模組一律
// 由本檔的 `pub use` 掛回 crate 根，`speclink_core::<模組>` 的既有路徑一字不改。
mod lifecycle;
mod quality;
// `workspace/` 資料夾裡有一個同名的 `workspace.rs` 子模組，資料夾模組不能也叫
// `workspace`（同一層兩個同名模組是 E0255），所以給它一個私有別名。
#[path = "workspace/mod.rs"]
mod workspace_group;

pub mod command;
pub mod demo;
pub mod keylines;
pub mod store;
/// Test-support helpers for this workspace's test suites — compiled only for
/// this crate's own tests or when a sibling crate's dev-dependency opts in via
/// the `testkit` feature; never part of a production build.
#[cfg(any(test, feature = "testkit"))]
#[doc(hidden)]
pub mod testkit;
#[cfg(test)]
pub(crate) mod teststore;
pub mod util;

pub use lifecycle::{
    archive, capname, discard, discuss, inprogress, listing, model, newcmd, plan, preflight,
    rank, status, tasks, trace,
};
pub use quality::{analyzer, drift, station, validate};
pub use workspace_group::{config, init, instructions, schema, skills, workspace};
