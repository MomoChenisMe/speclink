//! The engine-over-TeamStore execution bridge.
//!
//! The bridge wraps a TeamStore scope as a store port
//! ([`TeamStorePort`]) and runs the command through the same entry as every
//! other store ([`port_exec::execute`]). There is no second store view: the
//! reads, the single commit and the events all go through that one path.

use crate::context::SpeclinkExecutionContext;
use crate::port_exec::{self, ExecError, PortExecution};
use crate::teamstore_port::{scope_id, TeamStorePort};
use speclink_core::command::{Command, CommandOutcome, DomainEvent};
use speclink_store::{Revision, Scope, TeamStore};

/// A bridge execution failure. It is the failure of the store-port entry the
/// bridge runs on, under the name the server already uses: the Engine command
/// layer's typed codes, or the store's failures — a conflict keeps its key and
/// its expected and actual versions.
pub type BridgeError = ExecError;

/// The result of running one command over a TeamStore scope: the typed
/// outcome, the domain events the execution produced (empty for queries), and
/// — for a command that committed — the resulting project revision (`None`
/// when nothing was committed).
#[derive(Debug)]
pub struct BridgeExecution {
    pub outcome: CommandOutcome,
    pub events: Vec<DomainEvent>,
    pub revision: Option<Revision>,
}

impl From<PortExecution> for BridgeExecution {
    fn from(done: PortExecution) -> Self {
        BridgeExecution {
            outcome: done.outcome,
            events: done.events,
            revision: done.scope_version.map(|version| Revision(version.get())),
        }
    }
}

/// Run one Engine command over `store`'s scope (resolved from `ctx`). A
/// mutating verb's writes commit atomically with its domain events, in one
/// unit of work. Queries never open a unit of work.
pub fn execute(
    store: &(dyn TeamStore + Sync),
    ctx: &SpeclinkExecutionContext,
    cmd: Command,
) -> Result<BridgeExecution, BridgeError> {
    let scope = scope_id(&Scope::new(ctx.project.clone(), ctx.repo.clone()))
        .map_err(BridgeError::Store)?;
    port_exec::execute(&TeamStorePort::new(store), &scope, ctx, cmd).map(BridgeExecution::from)
}
