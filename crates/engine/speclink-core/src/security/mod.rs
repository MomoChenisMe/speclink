//! Offline preliminary security review. No process, network, or filesystem access.

mod boundary;
pub mod data;
pub mod input;
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
