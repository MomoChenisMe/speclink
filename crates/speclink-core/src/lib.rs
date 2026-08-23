//! speclink-core: spec-driven development engine.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod analyzer;
pub mod archive;
pub mod capname;
pub mod command;
pub mod config;
pub mod demo;
pub mod discard;
pub mod discuss;
pub mod drift;
pub mod init;
pub mod inprogress;
pub mod instructions;
pub mod listing;
pub mod model;
pub mod newcmd;
pub mod preflight;
pub mod review;
pub mod schema;
pub mod skills;
pub mod station;
pub mod status;
pub mod store;
pub mod tasks;
/// Test-support helpers for this workspace's test suites — compiled only for
/// this crate's own tests or when a sibling crate's dev-dependency opts in via
/// the `testkit` feature; never part of a production build.
#[cfg(any(test, feature = "testkit"))]
#[doc(hidden)]
pub mod testkit;
pub mod trace;
#[cfg(test)]
pub(crate) mod teststore;
pub mod util;
pub mod validate;
pub mod verify;
pub mod workspace;
