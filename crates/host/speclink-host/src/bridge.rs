//! The engine-over-TeamStore execution bridge.
//!
//! The bridge wraps a TeamStore scope as a store port
//! ([`TeamStorePort`]) and runs the command through the same entry as every
//! other store ([`port_exec::execute`]). There is no second store view: the
//! reads, the single commit and the events all go through that one path.

use crate::context::SpeclinkExecutionContext;
use crate::port_exec::{self, ExecError};
use crate::teamstore_port::TeamStorePort;
use speclink_core::command::{Command, CommandError, CommandOutcome, DomainEvent};
use speclink_store::{Revision, Scope, TeamStore};
use speclink_store_port::PortError;

/// A bridge execution failure, keeping the two error vocabularies distinct so
/// the server maps each at exactly one place: the Engine command layer's five
/// typed codes, or the store's failures — a conflict keeps its key and its
/// expected and actual versions.
#[derive(Debug)]
pub enum BridgeError {
    /// The Engine command layer refused before or during the flow.
    Command(CommandError),
    /// The store failed during a read, or rejected the commit.
    Store(PortError),
}

impl From<ExecError> for BridgeError {
    fn from(e: ExecError) -> BridgeError {
        match e {
            ExecError::Command(c) => BridgeError::Command(c),
            ExecError::Store(s) => BridgeError::Store(s),
        }
    }
}

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

/// Run one Engine command over `store`'s scope (resolved from `ctx`). A
/// mutating verb's writes commit atomically with its domain events, in one
/// unit of work. Queries never open a unit of work.
pub fn execute(
    store: &(dyn TeamStore + Sync),
    ctx: &SpeclinkExecutionContext,
    cmd: Command,
) -> Result<BridgeExecution, BridgeError> {
    let scope = Scope::new(ctx.project.clone(), ctx.repo.clone());
    let port = TeamStorePort::new(store, scope).map_err(BridgeError::Store)?;
    let done = port_exec::execute(&port, port.scope_id(), ctx, cmd)?;
    Ok(BridgeExecution {
        outcome: done.outcome,
        events: done.events,
        revision: done.scope_version.map(|version| Revision(version.get())),
    })
}
