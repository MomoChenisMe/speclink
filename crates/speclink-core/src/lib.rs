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
pub mod trace;
#[cfg(test)]
pub(crate) mod teststore;
pub mod util;
pub mod validate;
pub mod verify;
pub mod workspace;
