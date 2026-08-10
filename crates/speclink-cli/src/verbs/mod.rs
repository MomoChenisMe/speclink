//! One file per verb family (design D1/D5).
//!
//! A family file carries that family's whole story: clap parameter types, the
//! fs arm, the remote arm, rendering, wire→core conversion and its own tests.
//! Family files never import each other — anything two families share is
//! lifted into `crate::common` or `crate::remote_base` (design D4). Upward
//! dependencies are limited to main.rs's declaration layer (design D3):
//! family-level Dual arms (station, workflow-config) call `crate::dual`, and
//! toolchain's completion generation walks the `crate::Cli` clap tree.

pub(crate) mod checks;
pub(crate) mod config;
pub(crate) mod connection;
pub(crate) mod discuss;
pub(crate) mod documents;
pub(crate) mod init;
pub(crate) mod instructions;
pub(crate) mod lifecycle;
pub(crate) mod new;
pub(crate) mod progress;
pub(crate) mod query;
pub(crate) mod station;
pub(crate) mod toolchain;
