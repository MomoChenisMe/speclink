//! Explicitly prepared policy/spec data; never contains a source-code checkout.
use super::{data::sha256, report::valid_project_key};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewBinding {
    pub origin: Option<String>,
    pub project: Option<String>,
    pub repo: Option<String>,
    pub checkout: Option<String>,
}
impl ReviewBinding {
    pub fn project_key(&self) -> String {
        sha256(&serde_json::to_vec(self).expect("serializable binding"))
    }
    pub fn is_remote(&self) -> bool {
        self.origin.is_some()
    }
    pub fn validate(&self) -> Result<()> {
        for value in [&self.origin, &self.project, &self.repo, &self.checkout]
            .into_iter()
            .flatten()
        {
            ensure!(
                !value.is_empty() && value.len() <= 4096 && !value.chars().any(char::is_control),
                "binding_mismatch: invalid target identity"
            );
        }
        ensure!(
            (self.is_remote() && self.project.is_some() && self.repo.is_some())
                || (!self.is_remote()
                    && self.project.is_none()
                    && self.repo.is_none()
                    && self.checkout.is_some()),
            "binding_mismatch: incomplete target identity"
        );
        if let Some(root) = &self.checkout {
            ensure!(
                std::path::Path::new(root).is_absolute(),
                "unauthorized_root"
            );
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreparedDocument {
    pub path: String,
    pub text: String,
    pub revision: Option<u64>,
    pub content_digest: String,
}
impl PreparedDocument {
    pub fn new(path: String, text: String, revision: Option<u64>) -> Self {
        let content_digest = sha256(text.as_bytes());
        Self {
            path,
            text,
            revision,
            content_digest,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreparedSnapshot {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub proposal_project_key:Option<String>,
    pub format_version: u8,
    pub project_key: String,
    pub binding: ReviewBinding,
    pub change_name: Option<String>,
    pub policy_text: String,
    pub policy_digest: String,
    pub policy_revision: Option<u64>,
    pub prepared_at: String,
    pub documents: Vec<PreparedDocument>,
    pub documents_digest: String,
    pub document_count: usize,
    pub document_bytes: usize,
    pub locale: String,
}
impl PreparedSnapshot {
    pub fn validate(&self) -> Result<()> {
        self.validate_header()?;
        ensure!(
            self.document_count == self.documents.len()
                && self.document_bytes
                    == self.documents.iter().map(|d| d.text.len()).sum::<usize>(),
            "integrity_error: prepared document counts"
        );
        ensure!(
            sha256(&serde_json::to_vec(&self.documents)?) == self.documents_digest,
            "integrity_error: prepared documents"
        );
        ensure!(
            self.documents.len() <= 10_000
                && self.documents.iter().map(|d| d.text.len()).sum::<usize>() <= 104_857_600,
            "size_limit: prepared specifications"
        );
        let mut paths = std::collections::BTreeSet::new();
        for document in &self.documents {
            ensure!(
                valid_relative(&document.path)
                    && document.path.ends_with(".md")
                    && paths.insert(&document.path)
                    && document.text.len() <= 1_048_576
                    && sha256(document.text.as_bytes()) == document.content_digest,
                "invalid_preparation: specification identity or size"
            );
        }
        Ok(())
    }
    pub fn validate_header(&self) -> Result<()> {
        ensure!(self.proposal_project_key.as_deref().is_none_or(valid_project_key),"binding_mismatch: proposal project key");
        self.binding.validate()?;
        validate_change(self.change_name.as_deref())?;
        ensure!(
            self.format_version == 1
                && valid_project_key(&self.project_key)
                && self.project_key == self.binding.project_key(),
            "binding_mismatch"
        );
        ensure!(
            self.policy_text.len() <= 64 * 1024
                && sha256(self.policy_text.as_bytes()) == self.policy_digest,
            "invalid_preparation: policy fingerprint"
        );
        crate::config::WorkflowConfig::from_text(Some(&self.policy_text)).map_err(|_| {
            anyhow::anyhow!("invalid_config: prepared workflow policy could not be parsed")
        })?;
        ensure!(
            self.documents_digest.len() == 64
                && self.documents_digest.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid_preparation: document fingerprint"
        );
        ensure!(
            self.document_count <= 10_000 && self.document_bytes <= 104_857_600,
            "size_limit: prepared documents"
        );
        ensure!(
            matches!(self.locale.as_str(), "tw" | "ja" | "en"),
            "invalid_preparation: locale"
        );
        Ok(())
    }
    pub fn validate_binding(&self, binding: &ReviewBinding, change: Option<&str>) -> Result<()> {
        self.validate()?;
        ensure!(
            &self.binding == binding && self.change_name.as_deref() == change,
            "binding_mismatch"
        );
        Ok(())
    }
}
pub fn validate_change(change: Option<&str>) -> Result<()> {
    if let Some(change) = change {
        ensure!(
            !change.is_empty()
                && change.len() <= 200
                && change
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_')),
            "invalid_change"
        );
    }
    Ok(())
}
pub fn valid_relative(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', ':'])
        && !path.chars().any(char::is_control)
        && !std::path::Path::new(path).is_absolute()
        && path.split('/').all(|part| !matches!(part, "" | "." | ".."))
}
pub trait PreparedRepository: Send + Sync {
    fn find_binding(
        &self,
        origin: &str,
        project: &str,
        repo: Option<&str>,
        checkout: Option<&str>,
        change: Option<&str>,
    ) -> Result<Option<ReviewBinding>>;
    fn save(&self, snapshot: &PreparedSnapshot) -> Result<()>;
    fn load(&self, project_key: &str, change: Option<&str>) -> Result<Option<PreparedSnapshot>>;
    fn load_policy(
        &self,
        project_key: &str,
        change: Option<&str>,
    ) -> Result<Option<PreparedSnapshot>>;
}
