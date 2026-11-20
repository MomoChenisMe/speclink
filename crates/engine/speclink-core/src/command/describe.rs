//! Each engine command's stable name and serializable params (spec
//! command-runtime: 引擎命令有穩定的名稱與可序列化的參數). Hosts see both in
//! their hooks and the store's commit record carries the name, so the two are a
//! contract: `tests/golden/command-describe.json` pins them per variant.

use super::Command;

/// Every engine command's name ([`Command::name`]), once each, in the order of
/// its match arms (spec command-runtime: 引擎提供全部命令名稱的清單). The order
/// is no contract. The command-describe test pins this list to the commands:
/// a new command whose name is not added here turns it red.
pub const COMMAND_NAMES: &[&str] = &[
    "list",
    "show",
    "status",
    "instructions",
    "validate",
    "analyze",
    "trace",
    "plan",
    "plan-strict",
    "artifact-cat",
    "evidence-show",
    "language-show",
    "language-replace",
    "discuss-list",
    "discuss-show",
    "discuss-search",
    "new-change",
    "new-artifact",
    "task-done",
    "task-undone",
    "task-move",
    "claim",
    "claim-release",
    "in-progress-add",
    "in-progress-remove",
    "change-depends",
    "change-schema",
    "change-rank",
    "change-landed",
    "change-fields-edit",
    "archive",
    "discard",
    "discuss-new",
    "discuss-context",
    "discuss-add-round",
    "discuss-conclude",
    "discuss-promote",
    "discuss-link",
    "discuss-seal",
    "discuss-archive",
    "discuss-restore",
    "discuss-discard",
    "discuss-fields-edit",
    "review-add-round",
    "review-show",
    "review-stamp",
    "review-discard",
    "verify-add-round",
    "verify-show",
    "verify-stamp",
    "verify-discard",
    "workflow-config-show",
    "workflow-config-edit",
    "workflow-config-replace",
];

impl Command {
    /// The command's kebab-case name: the variant name, fixed per kind and
    /// independent of the params. Matched with no wildcard arm, so a new
    /// variant does not compile until it is named here.
    pub fn name(&self) -> &'static str {
        match self {
            Command::List { .. } => "list",
            Command::Show { .. } => "show",
            Command::Status { .. } => "status",
            Command::Instructions { .. } => "instructions",
            Command::Validate { .. } => "validate",
            Command::Analyze { .. } => "analyze",
            Command::Trace { .. } => "trace",
            Command::Plan { .. } => "plan",
            Command::PlanStrict => "plan-strict",
            Command::ArtifactCat { .. } => "artifact-cat",
            Command::EvidenceShow { .. } => "evidence-show",
            Command::LanguageShow => "language-show",
            Command::LanguageReplace { .. } => "language-replace",
            Command::DiscussList { .. } => "discuss-list",
            Command::DiscussShow { .. } => "discuss-show",
            Command::DiscussSearch { .. } => "discuss-search",
            Command::NewChange { .. } => "new-change",
            Command::NewArtifact { .. } => "new-artifact",
            Command::TaskDone { .. } => "task-done",
            Command::TaskUndone { .. } => "task-undone",
            Command::TaskMove { .. } => "task-move",
            Command::Claim { .. } => "claim",
            Command::ClaimRelease { .. } => "claim-release",
            Command::InProgressAdd { .. } => "in-progress-add",
            Command::InProgressRemove { .. } => "in-progress-remove",
            Command::ChangeDepends { .. } => "change-depends",
            Command::ChangeSchema { .. } => "change-schema",
            Command::ChangeRank { .. } => "change-rank",
            Command::ChangeLanded { .. } => "change-landed",
            Command::ChangeFieldsEdit { .. } => "change-fields-edit",
            Command::Archive { .. } => "archive",
            Command::Discard { .. } => "discard",
            Command::DiscussNew { .. } => "discuss-new",
            Command::DiscussContext { .. } => "discuss-context",
            Command::DiscussAddRound { .. } => "discuss-add-round",
            Command::DiscussConclude { .. } => "discuss-conclude",
            Command::DiscussPromote { .. } => "discuss-promote",
            Command::DiscussLink { .. } => "discuss-link",
            Command::DiscussSeal { .. } => "discuss-seal",
            Command::DiscussArchive { .. } => "discuss-archive",
            Command::DiscussRestore { .. } => "discuss-restore",
            Command::DiscussDiscard { .. } => "discuss-discard",
            Command::DiscussFieldsEdit { .. } => "discuss-fields-edit",
            Command::ReviewAddRound { .. } => "review-add-round",
            Command::ReviewShow { .. } => "review-show",
            Command::ReviewStamp { .. } => "review-stamp",
            Command::ReviewDiscard { .. } => "review-discard",
            Command::VerifyAddRound { .. } => "verify-add-round",
            Command::VerifyShow { .. } => "verify-show",
            Command::VerifyStamp { .. } => "verify-stamp",
            Command::VerifyDiscard { .. } => "verify-discard",
            Command::WorkflowConfigShow => "workflow-config-show",
            Command::WorkflowConfigEdit { .. } => "workflow-config-edit",
            Command::WorkflowConfigReplace { .. } => "workflow-config-replace",
        }
    }

    /// The command's params as a JSON object: one camelCase key per field,
    /// valued as the field serializes. A variant with no fields gives `{}`.
    pub fn params(&self) -> serde_json::Value {
        // Externally tagged: a struct variant is `{ "<Variant>": { fields } }`,
        // a unit variant its bare name.
        match serde_json::to_value(self).expect("command fields serialize to JSON") {
            serde_json::Value::Object(tagged) => {
                tagged.into_values().next().expect("a struct variant serializes as one tagged entry")
            }
            _ => serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}
