//! Separate immutable policy/spec files: disabled runs never read spec content.
use super::{
    atomic_replace, check_directory, cleanup_stale, open_lock, open_regular, owned_directory,
    read_limited, references_lock, sync_directory, valid_generation, write_new, FileLock,
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use speclink_core::security::{
    data::sha256,
    prepared::{validate_change, PreparedRepository, PreparedSnapshot, ReviewBinding},
    report::valid_project_key,
    standards::StandardId,
};
use std::{
    fs,
    path::{Path, PathBuf},
};
pub struct FsPreparedReviews {
    application_data: PathBuf,
    root: PathBuf,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Current {
    generation: String,
    policy_sha256: String,
    snapshot_sha256: String,
}
impl FsPreparedReviews {
    pub fn new(application_data: &Path) -> Result<Self> {
        ensure!(application_data.is_absolute(), "data_directory_unavailable");
        Ok(Self {
            application_data: application_data.into(),
            root: application_data.join("security/prepared"),
        })
    }
    fn directory(&self, key: &str, change: Option<&str>) -> Result<PathBuf> {
        ensure!(valid_project_key(key), "invalid_project_key");
        validate_change(change)?;
        Ok(self.root.join(key).join(
            change
                .map(|s| format!("change-{s}"))
                .unwrap_or_else(|| "project".into()),
        ))
    }
    fn read(
        &self,
        key: &str,
        change: Option<&str>,
        policy_only: bool,
    ) -> Result<Option<PreparedSnapshot>> {
        let directory = self.directory(key, change)?;
        if !owned_directory(&directory, &self.application_data, false)?
            || !directory.join("current.json").try_exists()?
        {
            return Ok(None);
        }
        let current: Current =
            serde_json::from_slice(&read_limited(&directory.join("current.json"), 4096)?)?;
        ensure!(
            valid_generation(&current.generation),
            "invalid_preparation: generation"
        );
        let generation = directory.join("generations").join(current.generation);
        owned_directory(&generation, &self.application_data, false)?;
        let pin = open_regular(&generation.join(".pin"), false)?;
        pin.try_lock_shared()
            .map_err(|_| anyhow::anyhow!("busy: prepared snapshot cleanup"))?;
        let _pin = FileLock(pin);
        let (name, limit, expected) = if policy_only {
            ("policy.json", 128 * 1024, current.policy_sha256)
        } else {
            ("snapshot.json", 200 * 1024 * 1024, current.snapshot_sha256)
        };
        let bytes = read_limited(&generation.join(name), limit)?;
        ensure!(
            sha256(&bytes) == expected,
            "integrity_error: prepared snapshot"
        );
        let snapshot: PreparedSnapshot = serde_json::from_slice(&bytes)?;
        if policy_only {
            snapshot.validate_header()?;
        } else {
            snapshot.validate()?;
        }
        ensure!(
            snapshot.project_key == key && snapshot.change_name.as_deref() == change,
            "binding_mismatch"
        );
        Ok(Some(snapshot))
    }
}
impl PreparedRepository for FsPreparedReviews {
    fn find_binding(
        &self,
        origin: &str,
        project: &str,
        repo: Option<&str>,
        checkout: Option<&str>,
        change: Option<&str>,
    ) -> Result<Option<ReviewBinding>> {
        validate_change(change)?;
        if !owned_directory(&self.root, &self.application_data, false)? {
            return Ok(None);
        }
        let mut matches = Vec::new();
        let mut count = 0;
        for entry in fs::read_dir(&self.root)? {
            count += 1;
            ensure!(count <= 10_000, "size_limit: prepared bindings");
            let entry = entry?;
            check_directory(&entry.path())?;
            let key = entry.file_name().to_string_lossy().into_owned();
            ensure!(valid_project_key(&key), "invalid_preparation: project key");
            if let Some(snapshot) = self.load_policy(&key, change)? {
                let binding = snapshot.binding;
                if binding.origin.as_deref() == Some(origin)
                    && binding.project.as_deref() == Some(project)
                    && binding.checkout.as_deref() == checkout
                    && repo.is_none_or(|r| binding.repo.as_deref() == Some(r))
                {
                    matches.push(binding);
                }
            }
        }
        ensure!(
            matches.len() <= 1,
            "ambiguous_preparation: declare remote.repo and prepare the intended scope"
        );
        Ok(matches.pop())
    }
    fn save(&self, snapshot: &PreparedSnapshot) -> Result<()> {
        snapshot.validate()?;
        let refs = self.application_data.join("security/references");
        owned_directory(&refs, &self.application_data, true)?;
        let _references = references_lock(&refs)?;
        let directory = self.directory(&snapshot.project_key, snapshot.change_name.as_deref())?;
        owned_directory(&directory, &self.application_data, true)?;
        let lock = open_lock(&directory.join(".lock"))?;
        lock.try_lock()
            .map_err(|_| anyhow::anyhow!("busy: preparation in progress"))?;
        let _lock = FileLock(lock);
        let bytes = serde_json::to_vec(snapshot)?;
        ensure!(
            bytes.len() <= 200 * 1024 * 1024,
            "size_limit: prepared snapshot"
        );
        let mut header = snapshot.clone();
        header.documents.clear();
        let policy = serde_json::to_vec(&header)?;
        ensure!(policy.len() <= 128 * 1024, "size_limit: prepared policy");
        let id = ulid::Ulid::new().to_string();
        let stage = directory.join("staging").join(&id);
        owned_directory(&stage, &self.application_data, true)?;
        let generations = directory.join("generations");
        owned_directory(&generations, &self.application_data, true)?;
        let pointer = directory.join(format!(".current-{id}.tmp"));
        let result = (|| -> Result<()> {
            write_new(&stage.join("snapshot.json"), &bytes, &|| false)?;
            write_new(&stage.join("policy.json"), &policy, &|| false)?;
            write_new(&stage.join(".pin"), b"", &|| false)?;
            sync_directory(&stage)?;
            fs::rename(&stage, generations.join(&id))?;
            sync_directory(&generations)?;
            write_new(
                &pointer,
                &serde_json::to_vec(&Current {
                    generation: id.clone(),
                    policy_sha256: sha256(&policy),
                    snapshot_sha256: sha256(&bytes),
                })?,
                &|| false,
            )?;
            atomic_replace(&pointer, &directory.join("current.json"))?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_dir_all(stage);
            let _ = fs::remove_file(pointer);
        } else {
            let _ = sync_directory(&directory);
            let _ = cleanup_stale(&directory, &id);
        }
        result
    }
    fn load(&self, key: &str, change: Option<&str>) -> Result<Option<PreparedSnapshot>> {
        self.read(key, change, false)
    }
    fn load_policy(&self, key: &str, change: Option<&str>) -> Result<Option<PreparedSnapshot>> {
        self.read(key, change, true)
    }
}
pub(super) fn references_version(
    application_data: &Path,
    id: StandardId,
    version: &str,
) -> Result<bool> {
    let repository = FsPreparedReviews::new(application_data)?;
    if !owned_directory(&repository.root, application_data, false)? {
        return Ok(false);
    }
    let mut count = 0;
    for project in fs::read_dir(&repository.root)? {
        let project = project?;
        check_directory(&project.path())?;
        let key = project.file_name().to_string_lossy().into_owned();
        ensure!(valid_project_key(&key), "invalid_preparation: project key");
        for scope in fs::read_dir(project.path())? {
            count += 1;
            ensure!(count <= 10_000, "size_limit: known preparations");
            let scope = scope?;
            check_directory(&scope.path())?;
            let name = scope.file_name().to_string_lossy().into_owned();
            let change = if name == "project" {
                None
            } else {
                Some(
                    name.strip_prefix("change-")
                        .ok_or_else(|| anyhow::anyhow!("invalid_preparation: scope"))?,
                )
            };
            if let Some(snapshot) = repository.load_policy(&key, change)? {
                let policy =
                    speclink_core::config::WorkflowConfig::from_text(Some(&snapshot.policy_text))?
                        .security();
                if policy.enabled
                    && policy
                        .standards
                        .selected()
                        .iter()
                        .any(|(s, v, _)| *s == id.as_str() && *v == version)
                {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_core::security::{
        data::StandardRepository,
        prepared::{PreparedDocument, ReviewBinding},
    };
    #[test]
    fn security_prepare_policy_is_separate_atomic_and_protects_known_references() {
        struct Directory(PathBuf);
        impl Drop for Directory {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let directory = Directory(
            std::env::temp_dir().join(format!("speclink-prepared-{}", ulid::Ulid::new())),
        );
        fs::create_dir_all(&directory.0).unwrap();
        let repository = FsPreparedReviews::new(&directory.0).unwrap();
        let binding = ReviewBinding {
            origin: Some("https://example.invalid".into()),
            project: Some("project".into()),
            repo: Some("repo".into()),
            checkout: None,
        };
        let documents = vec![PreparedDocument::new(
            "design.md".into(),
            "API specification with private-document-sentinel".into(),
            Some(2),
        )];
        let policy_text="security_verification:\n  enabled: true\n  standards:\n    owasp_top10: {enabled: true, version: '2025'}\n".to_owned();
        let mut snapshot = PreparedSnapshot {
            format_version: 1,
            project_key: binding.project_key(),
            binding,
            change_name: Some("demo".into()),
            policy_digest: sha256(policy_text.as_bytes()),
            policy_text,
            policy_revision: Some(3),
            prepared_at: "2026-10-17T00:00:00Z".into(),
            documents_digest: sha256(&serde_json::to_vec(&documents).unwrap()),
            document_count: 1,
            document_bytes: documents[0].text.len(),
            documents,
            locale: "en".into(),
        };
        repository.save(&snapshot).unwrap();
        let policy = repository
            .load_policy(&snapshot.project_key, Some("demo"))
            .unwrap()
            .unwrap();
        assert!(policy.documents.is_empty());
        assert_eq!(policy.document_count, 1);
        assert_eq!(
            repository
                .load(&snapshot.project_key, Some("demo"))
                .unwrap()
                .unwrap()
                .documents[0]
                .revision,
            Some(2)
        );
        let data = super::super::FsSecurityData::new(&directory.0).unwrap();
        assert!(data
            .remove(StandardId::OwaspTop10, "2025")
            .unwrap_err()
            .to_string()
            .contains("in_use"));
        snapshot.documents[0].path = "../outside.md".into();
        assert!(repository.save(&snapshot).is_err());
        assert_eq!(
            repository
                .load(&snapshot.project_key, Some("demo"))
                .unwrap()
                .unwrap()
                .documents[0]
                .path,
            "design.md"
        );
        let base = repository
            .directory(&snapshot.project_key, Some("demo"))
            .unwrap();
        let pointer: Current =
            serde_json::from_slice(&fs::read(base.join("current.json")).unwrap()).unwrap();
        fs::write(
            base.join("generations")
                .join(pointer.generation)
                .join("snapshot.json"),
            "corrupted source document body",
        )
        .unwrap();
        assert!(repository
            .load(&snapshot.project_key, Some("demo"))
            .is_err());
        assert!(repository
            .load_policy(&snapshot.project_key, Some("demo"))
            .unwrap()
            .unwrap()
            .documents
            .is_empty());
    }
}
