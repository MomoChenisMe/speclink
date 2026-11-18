//! speclink-host: the canonical Rust Host application-service layer.
//!
//! The Host is the Engine's only application-layer boundary: it resolves
//! the SpeclinkExecutionContext (actor, project/repo binding, mode,
//! effective workflow policy) exactly once at the entry point, adjudicates
//! lifecycle-gate transitions, and runs the Engine's commands on a store
//! port (one verb is one commit, with its events).
//! The Engine consumes the resolved context and never reads process env or
//! git identity itself; callers and models cannot override identity through
//! command parameters.

pub mod binding;
pub mod change_diff;
pub mod commit;
pub mod context;
pub mod documents;
pub mod drift;
pub mod gate;
pub mod http;
pub mod kit;
pub mod policy;
pub mod port_exec;
pub(crate) mod port_view;
pub mod projection;
pub mod read_only;
pub mod scope;
pub mod transfer;
pub mod worktree;
