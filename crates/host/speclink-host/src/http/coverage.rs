//! Where every Engine command is on the wire (design D6). The match has no
//! catch-all arm: a new command kind does not compile until it gets a route
//! or a reason to stay off the wire. The door's tests hold this map and the
//! route table to each other in both directions.

use super::Method;
use speclink_core::command::Command;

/// A command's place on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireCoverage {
    /// The route that runs it; the pattern is under
    /// `/api/speclink/v1/projects/{key}`.
    Route { method: Method, pattern: &'static str },
    /// It has no route, for this reason.
    NotOnWire { reason: &'static str },
}

const fn route(method: Method, pattern: &'static str) -> WireCoverage {
    WireCoverage::Route { method, pattern }
}

const fn off(reason: &'static str) -> WireCoverage {
    WireCoverage::NotOnWire { reason }
}

/// The wire place of `command`.
pub fn wire_coverage(command: &Command) -> WireCoverage {
    use Method::{Delete, Get, Post, Put};
    match command {
        Command::List { specs: true, .. } => route(Get, "/specs"),
        Command::List { .. } => route(Get, "/changes"),
        Command::Show { .. } => off(
            "the client builds `show` from the change, spec and document routes; the Engine's own text is not sent",
        ),
        Command::Status { .. } => route(Get, "/changes/{name}"),
        Command::Instructions { .. } => route(Get, "/changes/{name}/instructions/{*artifact}"),
        Command::Validate { .. } => route(Get, "/changes/{name}/validate"),
        Command::Analyze { .. } => route(Get, "/changes/{name}/analyze"),
        Command::Trace { .. } => route(Get, "/trace/{capability}"),
        // One route runs both plans: `strictOverlap=true` in the query picks this one.
        Command::Plan { .. } | Command::PlanStrict => route(Get, "/plan"),
        Command::ArtifactCat { .. } => route(Get, "/changes/{name}/artifacts/{*artifact}"),
        Command::EvidenceShow { .. } => route(Get, "/changes/{name}/evidence"),
        Command::LanguageShow => route(Get, "/language"),
        Command::DiscussList { .. } => route(Get, "/discussions"),
        Command::DiscussShow { .. } => route(Get, "/discussions/{slug}"),
        Command::DiscussSearch { .. } => route(Get, "/discussions/search"),
        Command::NewChange { .. } => route(Post, "/changes"),
        Command::NewArtifact { .. } => route(Put, "/changes/{name}/artifacts/{*artifact}"),
        Command::TaskDone { .. } => route(Post, "/changes/{name}/tasks/{task_id}/done"),
        Command::TaskUndone { .. } => route(Post, "/changes/{name}/tasks/{task_id}/undone"),
        Command::TaskMove { .. } => route(Post, "/changes/{name}/tasks/move"),
        Command::Claim { .. } => route(Post, "/changes/{name}/claim"),
        Command::InProgressAdd { .. } => route(Post, "/changes/{name}/in-progress"),
        Command::InProgressRemove { .. } => route(Delete, "/changes/{name}/in-progress"),
        Command::ChangeDepends { .. } => route(Post, "/changes/{name}/depends"),
        Command::ChangeSchema { .. } => route(Post, "/changes/{name}/schema"),
        Command::ChangeRank { .. } => route(Post, "/changes/{name}/rank"),
        Command::Archive { .. } => route(Post, "/changes/{name}/archive"),
        Command::Discard { .. } => route(Delete, "/changes/{name}"),
        Command::DiscussNew { .. } => route(Post, "/discussions"),
        Command::DiscussContext { .. } => route(Put, "/discussions/{slug}/context"),
        Command::DiscussAddRound { .. } => route(Post, "/discussions/{slug}/rounds"),
        Command::DiscussConclude { .. } => route(Post, "/discussions/{slug}/conclude"),
        Command::DiscussPromote { .. } => route(Post, "/discussions/{slug}/promote"),
        Command::DiscussLink { .. } => route(Post, "/discussions/{slug}/link"),
        Command::DiscussSeal { .. } => route(Post, "/discussions/{slug}/seal"),
        Command::DiscussArchive { .. } => route(Post, "/discussions/{slug}/archive"),
        Command::DiscussDiscard { .. } => route(Delete, "/discussions/{slug}"),
        Command::ReviewAddRound { .. } => route(Post, "/changes/{name}/review/rounds"),
        Command::ReviewShow { .. } => route(Get, "/changes/{name}/review"),
        Command::ReviewStamp { .. } => route(Post, "/changes/{name}/review/stamp"),
        Command::ReviewDiscard { .. } => route(Delete, "/changes/{name}/review"),
        Command::VerifyAddRound { .. } => route(Post, "/changes/{name}/verify/rounds"),
        Command::VerifyShow { .. } => route(Get, "/changes/{name}/verify"),
        Command::VerifyStamp { .. } => route(Post, "/changes/{name}/verify/stamp"),
        Command::VerifyDiscard { .. } => route(Delete, "/changes/{name}/verify"),
        Command::WorkflowConfigShow => off("the client reads the config's text from GET /config and shows it itself"),
        Command::WorkflowConfigEdit { .. } => off(
            "the client computes the edit on the text from GET /config and writes the whole document with PUT /config",
        ),
        Command::WorkflowConfigReplace { .. } => route(Put, "/config"),
    }
}
