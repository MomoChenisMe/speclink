//! speclink-core: spec-driven development engine (Spectra-compatible behavior).

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod analyzer;
pub mod archive;
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
pub mod schema;
pub mod skills;
pub mod status;
pub mod store;
pub mod tasks;
#[cfg(test)]
pub(crate) mod teststore;
pub mod util;
pub mod validate;
pub mod workspace;
