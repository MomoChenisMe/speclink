//! Offline preliminary security review. No process, network, or filesystem access.

mod boundary;
pub mod acceptance;
pub mod data;
pub mod input;
pub mod knowledge;
pub mod policy;
pub mod prepared;
pub mod report;
pub mod rules;
pub mod standards;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReviewStatus {
    Pass,
    Finding,
    NotVerified,
    NotApplicable,
    Disabled,
}

/// Every required verify/archive path consumes Host evidence at this boundary.
pub fn require_acceptance(store:&dyn crate::store::Store,change:&str,proof:Option<&acceptance::AcceptanceProof>) -> anyhow::Result<()> {
    let policy=crate::config::WorkflowConfig::from_text(store.read_workflow_config().as_deref())?.security();
    if !(policy.enabled&&policy.required_for_acceptance) {return Ok(());}
    let fail=|code,reasons| anyhow::Error::new(acceptance::GateFailure{code,reasons});
    let proof=proof.ok_or_else(||fail("missing_security_review",vec!["source_freshness_unverified".into()]))?;
    let index=store.read_artifact(change,crate::history::INDEX).map(|s|data::sha256(s.as_bytes()));
    if proof.change_name!=change||!report::valid_project_key(&proof.project_key)||proof.index_digest!=index {
        return Err(fail("security_report_stale",vec!["evidence_changed".into()]));
    }
    let result=proof.outcome(&policy);
    if result.required&&result.status=="passed"&&result.source_freshness!=acceptance::SourceFreshness::Unverified {return Ok(());}
    let code=if result.status=="stale" {"security_report_stale"}else if result.reasons.iter().any(|r|r=="missing_security_review"||r=="missing_builtin_review") {"missing_security_review"}else{"security_acceptance_blocked"};
    Err(fail(code,result.reasons))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobProgress {
    pub job_id: String,
    pub project_key: String,
    pub phase: String,
    pub completed_bytes: u64,
    pub total_bytes: Option<u64>,
    pub completed_files: usize,
    pub message: String,
}
