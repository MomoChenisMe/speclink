//! Rebuildable local knowledge snapshot. This module has no network capability.
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use speclink_core::security::{
    data::{sha256, StandardRepository},
    knowledge::registry,
    standards::StandardId,
};
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeStandard {
    pub standard_id: StandardId,
    pub version: String,
    pub data_sha256: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareReceipt {
    pub project_key: String,
    pub path: String,
    pub policy_digest: String,
    pub files: usize,
    pub bytes: usize,
    pub standards: Vec<KnowledgeStandard>,
    pub knowledge_digest: String,
}
pub use speclink_core::security::knowledge::KnowledgeManifest;
fn check(cancelled: &dyn Fn() -> bool) -> Result<()> {
    ensure!(!cancelled(), "cancelled");
    Ok(())
}

pub fn prepare(
    root: &Path,
    project_key: &str,
    policy_text: &str,
    repository: &dyn StandardRepository,
    cancelled: &dyn Fn() -> bool,
) -> Result<PrepareReceipt> {
    check(cancelled)?;
    ensure!(
        speclink_core::security::report::valid_project_key(project_key),
        "invalid_project_key"
    );
    let authorized = super::scan::AuthorizedProject::open(root)?;
    let root = authorized.root();
    let workflow = speclink_core::config::WorkflowConfig::from_text(Some(policy_text))
        .map_err(|_| anyhow::anyhow!("invalid_config: workflow policy could not be parsed"))?;
    let policy = workflow.security();
    ensure!(policy.enabled, "security_disabled");
    let selected = policy.standards.selected();
    ensure!(
        !selected.is_empty(),
        "unconfigured: select a standard before preparing knowledge"
    );
    let destination = root.join(".security_kb");
    if destination.symlink_metadata().is_ok() {
        validate_existing(&destination, project_key)?;
    }
    let ignore = root.join(".gitignore");
    if ignore.symlink_metadata().is_ok() {
        let _ = super::read_limited(&ignore, 1024 * 1024)?;
    }
    let lock = super::open_lock(&root.join(".security_kb.lock"))?;
    lock.try_lock()
        .map_err(|_| anyhow::anyhow!("busy: knowledge preparation"))?;
    let _lock = super::FileLock(lock);
    let registry = registry()?;
    let mut files = BTreeMap::new();
    let mut standards = Vec::new();
    let mut generations = Vec::new();
    let mut snapshots = Vec::new();
    let mut pins = Vec::new();
    for (id, version, _) in selected {
        check(cancelled)?;
        let id: StandardId = id.parse()?;
        pins.push(
            repository
                .pin(id, version)
                .context("missing_knowledge: selected standard not installed")?,
        );
        let source = repository
            .read_export_source(id, version)?
            .ok_or_else(|| anyhow::anyhow!("missing_knowledge: selected standard not installed"))?;
        ensure!(
            source.metadata.standard_id == id && source.metadata.version == version,
            "binding_mismatch"
        );
        ensure!(
            registry.status(&source.files)? == "ready",
            "missing_knowledge: complete guidance is not installed"
        );
        let installed = repository
            .read(id, version)?
            .ok_or_else(|| anyhow::anyhow!("missing_knowledge"))?;
        for (name, bytes) in source.files {
            check(cancelled)?;
            ensure!(
                source.metadata.files.get(&name) == Some(&sha256(&bytes)),
                "integrity_error: knowledge source"
            );
            let path =
                speclink_core::security::knowledge::snapshot_path(&registry, id, version, &name);
            if let Some(existing) = files.insert(path, bytes.clone()) {
                ensure!(
                    existing == bytes,
                    "integrity_error: inconsistent guidance generations"
                );
            }
        }
        standards.push(KnowledgeStandard {
            standard_id: id,
            version: version.into(),
            data_sha256: source.metadata.sha256.clone(),
        });
        generations.push(source.metadata);
        snapshots.push(installed.snapshot);
    }
    files.extend(speclink_core::security::knowledge::snapshot_mappings(
        &snapshots,
    )?);
    let policy_digest = policy.review_digest()?;
    let hashes: BTreeMap<_, _> = files
        .iter()
        .map(|(name, bytes)| (name.clone(), sha256(bytes)))
        .collect();
    ensure!(
        hashes == speclink_core::security::knowledge::snapshot_hashes(&generations, &snapshots)?,
        "integrity_error: snapshot assembly"
    );
    let knowledge_digest = speclink_core::security::knowledge::snapshot_digest(
        project_key,
        &policy_digest,
        &hashes,
        &generations,
    )?;
    let manifest = KnowledgeManifest {
        format_version: 1,
        generated_by: "SpecLink".into(),
        project_key: project_key.into(),
        policy_digest: policy_digest.clone(),
        knowledge_digest: knowledge_digest.clone(),
        files: hashes,
        data_generations: generations,
    };
    let bytes = files.values().map(Vec::len).sum();
    let count = files.len();
    let stage = tempfile::Builder::new()
        .prefix(".security_kb-stage-")
        .tempdir_in(root)?;
    for (path, bytes) in files {
        check(cancelled)?;
        let path = stage.path().join(path);
        fs::create_dir_all(path.parent().unwrap())?;
        super::write_new(&path, &bytes, cancelled)?;
    }
    super::write_new(
        &stage.path().join("manifest.json"),
        &serde_json::to_vec_pretty(&manifest)?,
        cancelled,
    )?;
    super::sync_directory(stage.path())?;
    check(cancelled)?;
    // Preserve the entire previous directory until publication succeeds.
    let backup = root.join(format!(".security_kb-backup-{}", ulid::Ulid::new()));
    let had_previous = destination.symlink_metadata().is_ok();
    if had_previous {
        validate_existing(&destination, project_key)?;
        fs::rename(&destination, &backup)?;
    }
    if let Err(error) = fs::rename(stage.path(), &destination) {
        if had_previous {
            fs::rename(&backup, &destination)
                .context("knowledge_recovery_required: old snapshot retained in backup")?;
        }
        return Err(error.into());
    }
    if let Err(error) = ensure_ignored(root) {
        let _ = fs::rename(&destination, stage.path());
        if had_previous {
            fs::rename(&backup, &destination).context("knowledge_recovery_required")?;
        }
        return Err(error);
    }
    super::sync_directory(root)?;
    if had_previous {
        let _ = fs::remove_dir_all(backup);
    }
    Ok(PrepareReceipt {
        project_key: project_key.into(),
        path: destination.to_string_lossy().into_owned(),
        policy_digest,
        files: count,
        bytes,
        standards,
        knowledge_digest,
    })
}

pub fn validate_existing(path: &Path, project_key: &str) -> Result<KnowledgeManifest> {
    super::check_directory(path)?;
    let manifest: KnowledgeManifest = serde_json::from_slice(&super::read_limited(
        &path.join("manifest.json"),
        1024 * 1024,
    )?)
    .context("foreign_knowledge: not a managed snapshot")?;
    ensure!(
        manifest.format_version == 1
            && manifest.generated_by == "SpecLink"
            && manifest.project_key == project_key,
        "foreign_knowledge: snapshot belongs to another project"
    );
    fn walk(root: &Path, directory: &Path, expected: &BTreeMap<String, String>) -> Result<()> {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            ensure!(!kind.is_symlink(), "skipped_link");
            if kind.is_dir() {
                walk(root, &entry.path(), expected)?;
            } else {
                let relative = entry
                    .path()
                    .strip_prefix(root)?
                    .to_string_lossy()
                    .replace('\\', "/");
                ensure!(
                    relative == "manifest.json" || expected.contains_key(&relative),
                    "foreign_knowledge: unlisted file"
                );
            }
        }
        Ok(())
    }
    walk(path, path, &manifest.files)?;
    for (relative, digest) in &manifest.files {
        ensure!(
            speclink_core::security::prepared::valid_relative(relative),
            "unauthorized_path"
        );
        let bytes = super::read_limited(
            &path.join(relative),
            speclink_core::security::standards::MAX_XML_BYTES,
        )?;
        ensure!(
            sha256(&bytes) == *digest,
            "integrity_error: knowledge snapshot"
        );
    }
    Ok(manifest)
}
fn ensure_ignored(root: &Path) -> Result<()> {
    let path = root.join(".gitignore");
    let mut text = if path.symlink_metadata().is_ok() {
        String::from_utf8(super::read_limited(&path, 1024 * 1024)?)?
    } else {
        String::new()
    };
    if text.lines().last() == Some("/.security_kb/") {
        return Ok(());
    }
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str(
        "/.security_kb.lock\n/.security_kb-stage-*/\n/.security_kb-backup-*/\n/.security_kb/\n",
    );
    let temp = root.join(format!(".security-kb-ignore-{}", ulid::Ulid::new()));
    super::write_new(&temp, text.as_bytes(), &|| false)?;
    super::atomic_replace(&temp, &path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_core::security::data::StandardRepository;
    fn package(repository: &super::super::FsSecurityData) {
        use speclink_core::security::{
            data::{IntegrityKind, ValidatedStandard},
            standards::{standard_source, StandardId},
        };
        use std::{
            collections::BTreeMap,
            io::{Cursor, Read},
        };
        let mut files = BTreeMap::from([
            ("raw/cwec_v4.20.xml".into(), br#"<Weakness_Catalog xmlns="http://cwe.mitre.org/cwe-7" Version="4.20"><Weaknesses><Weakness ID="79" Name="Synthetic"><Description>Fixture</Description></Weakness></Weaknesses></Weakness_Catalog>"#.to_vec()),
            ("LICENSE".into(), b"Synthetic MITRE terms fixture".to_vec()),
        ]);
        let mut archive = zip::ZipArchive::new(Cursor::new(include_bytes!(
            "../../tests/fixtures/security-guidance.zip"
        )))
        .unwrap();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            files.insert(file.name().into(), bytes);
        }
        let data = ValidatedStandard::new(
            standard_source(StandardId::Cwe, "4.20").unwrap(),
            files,
            IntegrityKind::Imported,
            &|| false,
        )
        .unwrap();
        repository
            .begin_install(StandardId::Cwe, "4.20")
            .unwrap()
            .commit(data)
            .unwrap();
    }
    const POLICY: &str = "security_verification:\n  enabled: true\n  standards:\n    cwe: {enabled: true, version: '4.20'}\n";
    #[test]
    fn knowledge_content_digest_survives_installation_on_another_device() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        let app_a = tempfile::tempdir().unwrap();
        let app_b = tempfile::tempdir().unwrap();
        let repo_a = super::super::FsSecurityData::new(app_a.path()).unwrap();
        let repo_b = super::super::FsSecurityData::new(app_b.path()).unwrap();
        package(&repo_a);
        package(&repo_b);
        let ma = repo_a
            .read(StandardId::Cwe, "4.20")
            .unwrap()
            .unwrap()
            .metadata;
        let mb = repo_b
            .read(StandardId::Cwe, "4.20")
            .unwrap()
            .unwrap()
            .metadata;
        assert_ne!(ma.generation, mb.generation);
        assert_eq!(ma.sha256, mb.sha256);
        let ra = prepare(a.path(), &"a".repeat(64), POLICY, &repo_a, &|| false).unwrap();
        let rb = prepare(b.path(), &"a".repeat(64), POLICY, &repo_b, &|| false).unwrap();
        assert_eq!(ra.knowledge_digest, rb.knowledge_digest);
    }
    #[test]
    fn knowledge_prepare_is_local_complete_and_preserves_foreign_or_missing_data() {
        let root = tempfile::tempdir().unwrap();
        let app = tempfile::tempdir().unwrap();
        let repo = super::super::FsSecurityData::new(app.path()).unwrap();
        let key = "a".repeat(64);
        assert!(prepare(root.path(), &key, POLICY, &repo, &|| false)
            .unwrap_err()
            .to_string()
            .contains("missing_knowledge"));
        assert!(!root.path().join(".security_kb").exists());
        package(&repo);
        let receipt = prepare(root.path(), &key, POLICY, &repo, &|| false).unwrap();
        assert!(receipt.files > 18);
        assert_eq!(receipt.standards.len(), 1);
        assert_eq!(receipt.standards[0].standard_id.as_str(), "cwe");
        let manifest_path = root.path().join(".security_kb/manifest.json");
        let original = std::fs::read(&manifest_path).unwrap();
        assert!(std::fs::read_to_string(root.path().join(".gitignore"))
            .unwrap()
            .contains("/.security_kb/"));
        assert!(prepare(root.path(), &key, POLICY, &repo, &|| true).is_err());
        assert_eq!(std::fs::read(&manifest_path).unwrap(), original);
        let guide = root
            .path()
            .join(".security_kb/cheatsheets/Authentication_Cheat_Sheet.md");
        let content = std::fs::read(&guide).unwrap();
        std::fs::write(&guide, "modified").unwrap();
        assert!(prepare(root.path(), &key, POLICY, &repo, &|| false)
            .unwrap_err()
            .to_string()
            .contains("integrity_error"));
        assert_eq!(std::fs::read_to_string(&guide).unwrap(), "modified");
        std::fs::write(&guide, content).unwrap();
        assert!(prepare(root.path(), &"b".repeat(64), POLICY, &repo, &|| false).is_err());
        assert_eq!(std::fs::read(&manifest_path).unwrap(), original);
        std::fs::write(&manifest_path, "foreign").unwrap();
        assert!(prepare(root.path(), &key, POLICY, &repo, &|| false).is_err());
        assert_eq!(std::fs::read_to_string(&manifest_path).unwrap(), "foreign");
    }
    #[cfg(unix)]
    #[test]
    fn knowledge_prepare_rejects_symlink_destination_and_gitignore() {
        let root = tempfile::tempdir().unwrap();
        let app = tempfile::tempdir().unwrap();
        let repo = super::super::FsSecurityData::new(app.path()).unwrap();
        package(&repo);
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join(".security_kb")).unwrap();
        assert!(prepare(root.path(), &"a".repeat(64), POLICY, &repo, &|| false).is_err());
        assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 0);
        std::fs::remove_file(root.path().join(".security_kb")).unwrap();
        let target = outside.path().join("ignore");
        std::fs::write(&target, "keep").unwrap();
        std::os::unix::fs::symlink(&target, root.path().join(".gitignore")).unwrap();
        assert!(prepare(root.path(), &"a".repeat(64), POLICY, &repo, &|| false).is_err());
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "keep");
    }
}
