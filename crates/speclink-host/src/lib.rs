//! speclink-host: the canonical Rust Host application-service layer.
//!
//! The Host is the Engine's only application-layer boundary (platform
//! architecture §4.2): it resolves the SpeclinkExecutionContext (actor,
//! project/repo binding, mode, effective workflow policy) exactly once at
//! the entry point, adjudicates lifecycle-gate transitions, and composes
//! the Engine with the TeamStore contract (unit of work + event commit).
//! The Engine consumes the resolved context and never reads process env or
//! git identity itself; callers and models cannot override identity through
//! command parameters.

pub mod binding;
pub mod commit;
pub mod context;
pub mod evidence;
pub mod gate;
pub mod policy;
