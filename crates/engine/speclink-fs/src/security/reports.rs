//! Atomic local JSON/Markdown report pairs. Reports never mutate SDD tickets.
use super::{
    atomic_replace, check_directory, open_lock, open_regular, owned_directory, read_limited,
    sync_directory, write_new, FileLock,
};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use speclink_core::security::{
    data::sha256,
    report::{valid_project_key, valid_report_id, Report, ReportReceipt, ReportRepository},
};
use std::{
    fs,
    path::{Path, PathBuf},
};
const MAX_BYTES: u64 = 200 * 1024 * 1024;

pub struct FsSecurityReports {
    application_data: PathBuf,
    root: PathBuf,
    #[cfg(test)]
    fault: Option<ReportFault>,
    #[cfg(test)]
    max_reports: usize,
}
#[cfg(test)]
#[derive(Clone, Copy, PartialEq)]
enum ReportFault {
    BeforePublish,
    Latest,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Manifest {
    format_version: u8,
    report_id: String,
    project_key: String,
    json_sha256: String,
    markdown_sha256: String,
}
impl FsSecurityReports {
    pub fn new(application_data: &Path) -> Result<Self> {
        ensure!(application_data.is_absolute(), "data_directory_unavailable");
        Ok(Self {
            application_data: application_data.into(),
            root: application_data.join("security/reports"),
            #[cfg(test)]
            fault: None,
            #[cfg(test)]
            max_reports: 1000,
        })
    }
    /// A device copy remains a cache; only the server index is authoritative.
    pub fn record_publication(
        &self,
        report: &Report,
        change: &str,
        deep_report_id: &str,
    ) -> Result<()> {
        ensure!(
            valid_report_id(deep_report_id) && report.change_name() == Some(change),
            "binding_mismatch"
        );
        let Some(local) = self.get(report.report_id())? else {
            return Ok(());
        };
        ensure!(
            local.project_key() == report.project_key(),
            "binding_mismatch"
        );
        let directory = self
            .root
            .join(report.project_key())
            .join(report.report_id());
        check_directory(&directory)?;
        let temp = directory.join(format!(".publication-{}", ulid::Ulid::new()));
        let text = serde_json::to_vec(
            &serde_json::json!({"formatVersion":1,"projectKey":report.project_key(),"changeName":change,"builtinReportId":report.report_id(),"deepReportId":deep_report_id,"authority":"server-index"}),
        )?;
        write_new(&temp, &text, &|| false)?;
        atomic_replace(&temp, &directory.join("publication.json"))?;
        sync_directory(&directory)
    }
    fn directories(&self) -> Result<Vec<(String, String, PathBuf)>> {
        if !owned_directory(&self.root, &self.application_data, false)? {
            return Ok(Vec::new());
        }
        let mut directories = Vec::new();
        let mut projects = 0;
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            let key = entry.file_name().to_string_lossy().into_owned();
            if key == "staging" || key.starts_with('.') {
                continue;
            }
            ensure!(
                valid_project_key(&key),
                "invalid_report: unexpected project key"
            );
            check_directory(&entry.path())?;
            projects += 1;
            ensure!(projects <= 1000, "storage_limit: report project count");
            for report in fs::read_dir(entry.path())? {
                let report = report?;
                let id = report.file_name().to_string_lossy().into_owned();
                if id == "latest.json" || id.starts_with('.') {
                    continue;
                }
                ensure!(valid_report_id(&id), "invalid_report: unexpected report ID");
                check_directory(&report.path())?;
                directories.push((key.clone(), id, report.path()));
                ensure!(directories.len() <= 1000, "storage_limit: report count");
            }
        }
        Ok(directories)
    }
    fn read_at(&self, key: &str, id: &str, path: &Path) -> Result<Report> {
        let manifest: Manifest =
            serde_json::from_slice(&read_limited(&path.join("manifest.json"), 4096)?)
                .context("invalid_report: manifest")?;
        ensure!(
            manifest.format_version == 1 && manifest.project_key == key && manifest.report_id == id,
            "invalid_report: manifest identity"
        );
        let json = read_limited(&path.join("report.json"), MAX_BYTES as usize)?;
        ensure!(
            sha256(&json) == manifest.json_sha256,
            "integrity_error: report JSON"
        );
        let markdown = read_limited(&path.join("report.md"), MAX_BYTES as usize)?;
        ensure!(
            sha256(&markdown) == manifest.markdown_sha256,
            "integrity_error: report Markdown"
        );
        let report = Report::from_json(&json)?;
        ensure!(
            report.project_key() == key && report.report_id() == id,
            "invalid_report: report identity"
        );
        ensure!(
            report.markdown().as_bytes() == markdown,
            "integrity_error: report representations differ"
        );
        Ok(report)
    }
}
impl ReportRepository for FsSecurityReports {
    fn save(&self, report: &Report) -> Result<ReportReceipt> {
        owned_directory(&self.root, &self.application_data, true)?;
        let lock = open_lock(&self.root.join(".lock"))?;
        lock.try_lock()
            .map_err(|_| anyhow::anyhow!("busy: report storage is being modified"))?;
        let _lock = FileLock(lock);
        let directories = self.directories()?;
        #[cfg(test)]
        let max_reports = self.max_reports;
        #[cfg(not(test))]
        let max_reports = 1000;
        ensure!(
            directories.len() < max_reports,
            "storage_limit: 1000 reports; remove or export old reports manually"
        );
        ensure!(
            !directories
                .iter()
                .any(|(_, id, _)| id == report.report_id()),
            "report_exists"
        );
        let mut existing = 0u64;
        for (_, _, path) in &directories {
            for name in ["report.json", "report.md", "manifest.json"] {
                existing = existing
                    .checked_add(open_regular(&path.join(name), false)?.metadata()?.len())
                    .ok_or_else(|| anyhow::anyhow!("storage_limit"))?;
            }
        }
        let json = serde_json::to_vec_pretty(report)?;
        let markdown = report.markdown().into_bytes();
        let manifest = serde_json::to_vec(&Manifest {
            format_version: 1,
            project_key: report.project_key().into(),
            report_id: report.report_id().into(),
            json_sha256: sha256(&json),
            markdown_sha256: sha256(&markdown),
        })?;
        let needed = json.len() as u64 + markdown.len() as u64 + manifest.len() as u64;
        ensure!(
            existing.saturating_add(needed) <= MAX_BYTES,
            "storage_limit: 200 MiB report quota"
        );
        let project = self.root.join(report.project_key());
        owned_directory(&project, &self.application_data, true)?;
        let staging_root = self.root.join("staging");
        owned_directory(&staging_root, &self.application_data, true)?;
        let stage = staging_root.join(ulid::Ulid::new().to_string());
        fs::create_dir(&stage)?;
        struct Stage(Option<PathBuf>);
        impl Drop for Stage {
            fn drop(&mut self) {
                if let Some(path) = &self.0 {
                    let _ = fs::remove_dir_all(path);
                }
            }
        }
        let mut cleanup = Stage(Some(stage.clone()));
        write_new(&stage.join("report.json"), &json, &|| false)?;
        write_new(&stage.join("report.md"), &markdown, &|| false)?;
        write_new(&stage.join("manifest.json"), &manifest, &|| false)?;
        sync_directory(&stage)?;
        #[cfg(test)]
        if self.fault == Some(ReportFault::BeforePublish) {
            anyhow::bail!("injected_report_write_failure");
        }
        let destination = project.join(report.report_id());
        ensure!(!destination.try_exists()?, "report_exists");
        fs::rename(&stage, &destination)?;
        cleanup.0 = None;
        let mut warnings = Vec::new();
        if sync_directory(&project).is_err() {
            warnings.push("report_directory_sync_unavailable".into());
        }
        let latest_temp = project.join(format!(".latest-{}.tmp", ulid::Ulid::new()));
        let latest = (|| -> Result<()> {
            #[cfg(test)]
            if self.fault == Some(ReportFault::Latest) {
                anyhow::bail!("injected_latest_failure");
            }
            write_new(
                &latest_temp,
                &serde_json::to_vec(
                    &serde_json::json!({"formatVersion":1,"reportId":report.report_id()}),
                )?,
                &|| false,
            )?;
            atomic_replace(&latest_temp, &project.join("latest.json"))?;
            sync_directory(&project)
        })();
        if latest.is_err() {
            let _ = fs::remove_file(&latest_temp);
            warnings.push("latest_index_unavailable; report remains available by ID".into());
        }
        Ok(ReportReceipt {
            report_id: report.report_id().into(),
            project_key: report.project_key().into(),
            warnings,
        })
    }
    fn get(&self, report_id: &str) -> Result<Option<Report>> {
        ensure!(valid_report_id(report_id), "invalid_report_id");
        let directories = self.directories()?;
        let matching = directories
            .iter()
            .filter(|(_, id, _)| id == report_id)
            .collect::<Vec<_>>();
        ensure!(matching.len() <= 1, "invalid_report: ambiguous report ID");
        matching
            .first()
            .map(|(key, id, path)| self.read_at(key, id, path))
            .transpose()
    }
    fn latest(&self, project_key: &str, change: Option<&str>) -> Result<Option<Report>> {
        ensure!(valid_project_key(project_key), "invalid_project_key");
        // The index is an optimization hint, never the source of truth. Walking
        // the bounded complete generations also recovers a failed/stale index.
        let mut directories = self
            .directories()?
            .into_iter()
            .filter(|(key, _, _)| key == project_key)
            .collect::<Vec<_>>();
        directories.sort_by(|a, b| b.1.cmp(&a.1));
        for (key, id, path) in directories {
            let report = self.read_at(&key, &id, &path)?;
            if report.change_name() == change {
                return Ok(Some(report));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_core::security::{
        input::ScanInputs,
        policy::SecurityScope,
        report::{ReportContext, ReportRepository},
    };
    #[test]
    fn proposal_security_reports_keep_v1_and_redacted_v2_in_one_index() {
        use speclink_core::{
            security::report::{read_change_report, save_change_report, VersionedReport},
            store::Store,
        };
        let directory = tempfile::tempdir().unwrap();
        let store = crate::FsStore::new(directory.path(), "openspec");
        store
            .create_change("demo", "schema: spec-driven\n")
            .unwrap();
        let basic = report("01ARZ3NDEKTSV4RRFFQ69G5FAV");
        let mut value = serde_json::to_value(&basic).unwrap();
        assert!(matches!(
            VersionedReport::from_json(&serde_json::to_vec(&value).unwrap()).unwrap(),
            VersionedReport::Builtin(_)
        ));
        save_change_report(&store, "demo", &VersionedReport::Builtin(basic)).unwrap();
        value["formatVersion"] = 2.into();
        value["reportId"] = "01ARZ3NDEKTSV4RRFFQ69G5FAX".into();
        value["executionMode"] = "deep-skill".into();
        value["knowledgeDigest"] = "d".repeat(64).into();
        value["scopeDigest"] = "c".repeat(64).into();
        value["actor"] = "Reviewer".into();
        value["builtinReportId"] = "01ARZ3NDEKTSV4RRFFQ69G5FAV".into();
        value["targets"] = serde_json::json!(["app.ts"]);
        value["checks"] = serde_json::json!([]);
        value["configurationState"] = "configured".into();
        value["findings"] = serde_json::json!([{"findingId":"H1","domain":"機密與通訊","title":"金鑰寫死","severity":"high","confidence":"high","confidenceReason":"程式已證明；無可信對照","references":[],"evidence":[{"path":"app.ts","line":1,"endLine":null,"function":null}],"codeSnippet":"api_key='unique-fs-deep-secret'","dataFlow":"input → secret → service","impact":"unique-fs-deep-secret 必須省略","remediation":"SHALL 使用本機機密管理指南修正。","acceptance":"測試不得輸出機密。"}]);
        let deep = VersionedReport::from_json(&serde_json::to_vec(&value).unwrap()).unwrap();
        save_change_report(&store, "demo", &deep).unwrap();
        let index: serde_json::Value =
            serde_json::from_str(&store.read_artifact("demo", "checks/index.json").unwrap())
                .unwrap();
        assert_eq!(index["entries"].as_array().unwrap().len(), 2);
        let saved = read_change_report(&store, "demo", deep.report_id())
            .unwrap()
            .unwrap();
        assert!(!saved.json().unwrap().contains("unique-fs-deep-secret"));
        assert!(!saved.markdown().contains("unique-fs-deep-secret"));
        let path = store
            .layout()
            .change_dir("demo")
            .join(format!("checks/security/{}/report.json", deep.report_id()));
        let original = std::fs::read(&path).unwrap();
        let mut changed: serde_json::Value = serde_json::from_slice(&original).unwrap();
        changed["policyDigest"] = "f".repeat(64).into();
        std::fs::write(&path, serde_json::to_vec_pretty(&changed).unwrap()).unwrap();
        assert!(read_change_report(&store, "demo", deep.report_id()).is_err());
        std::fs::write(path, original).unwrap();
        assert!(read_change_report(&store, "demo", "../../outside").is_err());
        assert!(
            read_change_report(&store, "demo", "01ARZ3NDEKTSV4RRFFQ69G5FAY")
                .unwrap()
                .is_none()
        );
        store.archive_change("demo", "2026-10-20-demo").unwrap();
        assert!(
            read_change_report(&store, "2026-10-20-demo", deep.report_id())
                .unwrap()
                .is_some()
        );
    }
    fn report(id: &str) -> Report {
        Report::new(
            ReportContext {
                report_id: id.into(),
                project_key: "a".repeat(64),
                project_name: "Synthetic project".into(),
                change_name: Some("demo".into()),
                reviewed_at: "2026-10-17T00:00:00Z".into(),
                commit_sha: None,
                policy_digest: "b".repeat(64),
                policy_revision: None,
                prepared_at: None,
                rule_version: "1.0.0".into(),
                scope: SecurityScope::Off,
                locale: "en".into(),
                review_methods: Vec::new(),
            },
            "disabled",
            true,
            Vec::new(),
            Vec::new(),
            &ScanInputs {
                visited_files: 0,
                documents: Vec::new(),
                coverage: Vec::new(),
                remaining_targets: Vec::new(),
                total_bytes: 0,
                complete: true,
                stop_reason: None,
            },
        )
        .unwrap()
    }
    #[test]
    fn security_report_atomic_pair_index_failure_quota_and_invalid_identity() {
        struct Directory(std::path::PathBuf);
        impl Drop for Directory {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let directory =
            Directory(std::env::temp_dir().join(format!("speclink-reports-{}", ulid::Ulid::new())));
        std::fs::create_dir_all(&directory.0).unwrap();
        let mut repository = FsSecurityReports::new(&directory.0).unwrap();
        let first = report("01ARZ3NDEKTSV4RRFFQ69G5FAV");
        repository.save(&first).unwrap();
        assert_eq!(
            repository
                .get(first.report_id())
                .unwrap()
                .unwrap()
                .markdown(),
            first.markdown()
        );
        let second = report("01ARZ3NDEKTSV4RRFFQ69G5FAW");
        repository.fault = Some(ReportFault::BeforePublish);
        assert!(repository.save(&second).is_err());
        assert!(repository.get(second.report_id()).unwrap().is_none());
        assert_eq!(
            repository
                .latest(first.project_key(), Some("demo"))
                .unwrap()
                .unwrap()
                .report_id(),
            first.report_id()
        );
        repository.fault = Some(ReportFault::Latest);
        assert!(!repository.save(&second).unwrap().warnings.is_empty());
        assert!(repository.get(second.report_id()).unwrap().is_some());
        repository.fault = None;
        repository.max_reports = 2;
        assert!(repository
            .save(&report("01ARZ3NDEKTSV4RRFFQ69G5FAX"))
            .unwrap_err()
            .to_string()
            .contains("storage_limit"));
        repository.max_reports = 1000;
        let json_path = directory
            .0
            .join("security/reports")
            .join(first.project_key())
            .join(first.report_id())
            .join("report.json");
        let original = std::fs::read(&json_path).unwrap();
        std::fs::OpenOptions::new()
            .write(true)
            .open(&json_path)
            .unwrap()
            .set_len(MAX_BYTES)
            .unwrap();
        assert!(repository
            .save(&report("01ARZ3NDEKTSV4RRFFQ69G5FAX"))
            .unwrap_err()
            .to_string()
            .contains("storage_limit"));
        std::fs::write(json_path, original).unwrap();
        for invalid in ["../../secret", "lowercase", "01ARZ3NDEKTSV4RRFFQ69G5FA/"] {
            assert!(repository.get(invalid).is_err());
        }
        let file = directory
            .0
            .join("security/reports")
            .join(first.project_key())
            .join(first.report_id())
            .join("report.md");
        std::fs::write(file, "tampered-secret-not-exported").unwrap();
        assert!(repository.get(first.report_id()).is_err());
        assert!(!directory.0.join("openspec").exists());
    }
}

/// The caller supplies an explicit save-dialog path and already redacted bytes.
pub fn export_to_path(path: &Path, bytes: &[u8]) -> Result<()> {
    super::scan::export_new_file(path, bytes)
}
