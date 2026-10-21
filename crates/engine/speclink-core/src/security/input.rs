//! Transient source text and serializable coverage; source text is never serialized.
use super::ReviewStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Json,
    Yaml,
    Markdown,
    Unsupported,
}
impl Language {
    pub fn from_path(path: &str) -> Self {
        match path
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str()
        {
            "rs" => Self::Rust,
            "ts" | "tsx" | "mts" | "cts" => Self::TypeScript,
            "js" | "jsx" | "mjs" | "cjs" => Self::JavaScript,
            "json" => Self::Json,
            "yaml" | "yml" => Self::Yaml,
            "md" => Self::Markdown,
            _ => Self::Unsupported,
        }
    }
}
#[derive(Clone)]
pub struct SourceDocument {
    pub path: String,
    pub text: std::sync::Arc<str>,
    pub language: Language,
    pub content_digest: String,
}
impl SourceDocument {
    pub fn new(path: String, text: String) -> Self {
        Self {
            language: Language::from_path(&path),
            content_digest: super::data::sha256(text.as_bytes()),
            path,
            text: text.into(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageEntry {
    pub path: String,
    pub status: ReviewStatus,
    pub reason: Option<String>,
    pub bytes: u64,
}
pub struct ScanInputs {
    pub visited_files: usize,
    pub documents: Vec<SourceDocument>,
    pub coverage: Vec<CoverageEntry>,
    pub remaining_targets: Vec<String>,
    pub total_bytes: u64,
    pub complete: bool,
    pub stop_reason: Option<String>,
}
#[derive(Debug)]
pub struct InputReadFailure {
    pub code: &'static str,
    pub bytes_read: usize,
}
impl std::fmt::Display for InputReadFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code)
    }
}
impl std::error::Error for InputReadFailure {}
pub fn is_excluded(path: &std::path::Path, extra: &[String]) -> bool {
    path.components().any(|c| {
        matches!(
            c.as_os_str().to_str(),
            Some(".git" | ".gitignore" | "node_modules" | "target" | "dist" | "build" | ".security_kb" | ".security_kb.lock" | ".checks.lock" | ".checks-transaction.json")
        )
    }) || path.components().any(|component| component.as_os_str().to_str().is_some_and(|name| name.starts_with(".security_kb-stage-") || name.starts_with(".security_kb-backup-") || name.starts_with(".checks-stage-") || name.starts_with(".checks-backup-")))
        || {
            let parts: Vec<_> = path.components().filter_map(|c|c.as_os_str().to_str()).collect();
            parts.windows(3).any(|w|w[0]=="changes" && w[2]=="checks") || parts.windows(3).any(|w|w[0]=="archive" && w[2]=="checks")
        }
        || path.starts_with(".speclink/context")
        || extra.iter().any(|e| path.starts_with(e))
}
#[derive(Clone)]
pub struct ScanLimits {
    pub file_bytes: usize,
    pub total_bytes: usize,
    pub files: usize,
    pub depth: usize,
    pub seconds: u64,
}
#[derive(Clone, Copy)]
pub enum InputKind {
    Code,
    Api,
    Spec,
    /// Source evidence for the skill; built-in rule language support is unchanged.
    Deep,
}
pub trait ProjectInput: Send + Sync {
    fn root(&self) -> &std::path::Path;
    /// Bounded, link-safe raw data for knowledge fingerprints, including retained archives.
    fn read_optional_bytes(
        &self,
        path: &str,
        limit: usize,
        stop: &dyn Fn() -> Option<&'static str>,
    ) -> anyhow::Result<Option<Vec<u8>>>;
    fn read_optional_text(
        &self,
        path: &str,
        limit: usize,
        stop: &dyn Fn() -> Option<&'static str>,
    ) -> anyhow::Result<Option<String>>;
    fn directory_exists(&self, path: &str) -> anyhow::Result<bool>;
    fn scan_selected(
        &self,
        paths: &[String],
        kind: InputKind,
        exclude: &[String],
        limits: &ScanLimits,
        stop: &dyn Fn() -> Option<&'static str>,
    ) -> anyhow::Result<ScanInputs>;
}
impl Default for ScanLimits {
    fn default() -> Self {
        Self {
            file_bytes: 1_048_576,
            total_bytes: 104_857_600,
            files: 10_000,
            depth: 32,
            seconds: 60,
        }
    }
}
