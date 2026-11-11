//! The engine-over-TeamStore execution bridge.
//!
//! The bridge wraps a TeamStore scope as a store port
//! ([`TeamStorePort`]) and runs the command through the same entry as every
//! other store ([`port_exec::execute`]). There is no second store view: the
//! reads, the single commit and the events all go through that one path.

use crate::context::SpeclinkExecutionContext;
use crate::port_exec::{self, ExecError};
use crate::teamstore_port::TeamStorePort;
use speclink_core::command::{Command, CommandOutcome, DomainEvent};
use speclink_store::{Revision, Scope, TeamStore};
use speclink_store_port::{KeyPrefix, StorePort};

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
    run(&port, ctx, cmd)
}

fn run(
    port: &TeamStorePort,
    ctx: &SpeclinkExecutionContext,
    cmd: Command,
) -> Result<BridgeExecution, BridgeError> {
    let done = port_exec::execute(port, port.scope_id(), ctx, cmd)?;
    Ok(BridgeExecution {
        outcome: done.outcome,
        events: done.events,
        revision: done.scope_version.map(|version| Revision(version.get())),
    })
}

/// What [`execute_at_revision`] did.
#[derive(Debug)]
pub enum AtRevision {
    /// The scope was at the expected revision and the command ran.
    Done(BridgeExecution),
    /// The scope is at another revision; nothing ran.
    Moved { actual: Revision },
}

/// [`execute`], but only when the scope is at `expected` — for a write whose
/// caller states the scope revision it last saw. The comparison and the
/// command's reads use one load of the scope, so a writer that lands after
/// the comparison meets the commit's own version check: the command never
/// reads that writer's content and writes over it.
pub fn execute_at_revision(
    store: &(dyn TeamStore + Sync),
    ctx: &SpeclinkExecutionContext,
    cmd: Command,
    expected: Revision,
) -> Result<AtRevision, BridgeError> {
    let scope = Scope::new(ctx.project.clone(), ctx.repo.clone());
    let port = TeamStorePort::new(store, scope).map_err(BridgeError::Store)?;
    // Any list loads the scope; the command then reads from that same load.
    let everything = KeyPrefix::new("").expect("the empty prefix is a prefix");
    let actual = port.list(port.scope_id(), &everything).map_err(BridgeError::Store)?.version.get();
    if actual != expected.0 {
        return Ok(AtRevision::Moved { actual: Revision(actual) });
    }
    run(&port, ctx, cmd).map(AtRevision::Done)
}
