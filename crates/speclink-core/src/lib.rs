//! speclink-core: spec-driven development engine (Spectra-compatible behavior).

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod analyzer;
pub mod archive;
pub mod config;
pub mod demo;
pub mod discuss;
pub mod drift;
pub mod init;
pub mod inprogress;
pub mod instructions;
pub mod model;
pub mod newcmd;
pub mod preflight;
pub mod schema;
pub mod skills;
pub mod status;
pub mod store;
pub mod tasks;
pub mod util;
pub mod validate;
pub mod workspace;
