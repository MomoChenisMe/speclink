//! Completed-record selection is planned before archive changes any document.
use super::*;
use crate::{
    config::ArchiveRecords,
    security::{
        acceptance::{AcceptanceProof, RiskDecision, SourceFreshness},
        report::{redact_text, VersionedReport},
    },
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecordCount {
    pub retained: usize,
    pub omitted: usize,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveRecordCounts {
    pub review: RecordCount,
    pub verify: RecordCount,
    pub security: RecordCount,
    pub acceptance_receipt: bool,
}
impl ArchiveRecordCounts {
    pub fn has_records(&self) -> bool {
        self.review.retained
            + self.review.omitted
            + self.verify.retained
            + self.verify.omitted
            + self.security.retained
            + self.security.omitted
            > 0
            || self.acceptance_receipt
    }
}
#[derive(Debug, Clone)]
pub struct ArchiveChecksPlan {
    /// Canonical writes travel with the archive so the backend validates and locks first.
    pub canonical_specs: BTreeMap<String, String>,
    pub retention: ArchiveRecords,
    pub expected_index: Option<String>,
    pub expected_meta: Option<String>,
    pub index: Option<String>,
    pub receipt: Option<String>,
    pub archived_meta: Option<String>,
    pub counts: ArchiveRecordCounts,
}
impl ArchiveChecksPlan {
    pub fn write_canonical_specs<S: Store + ?Sized>(&self, store: &S) -> Result<()> {
        for (capability, content) in &self.canonical_specs {
            store.write_canonical_spec(capability, content)
                .map_err(|error| anyhow::anyhow!("Failed to write canonical spec for '{capability}': {error}"))?;
        }
        Ok(())
    }
    pub fn retains(&self, path: &str) -> bool {
        for (kind, keep) in [
            ("review", self.retention.review),
            ("verify", self.retention.verify),
            ("security", self.retention.security),
        ] {
            if path.starts_with(&format!("checks/{kind}/")) {
                return keep;
            }
        }
        true
    }
    pub fn modifies_checks(&self) -> bool {
        self.retention != ArchiveRecords::default()
            || self.receipt.is_some()
            || self.index.is_some()
    }
    pub fn validate_base<S: Store + ?Sized>(&self, store: &S, change: &str) -> Result<()> {
        ensure!(
            store
                .read_artifact(change, INDEX)
                .map(|s| sha256(s.as_bytes()))
                == self.expected_index,
            "revision_conflict: check index changed"
        );
        ensure!(
            store.read_change_meta(change).map(|s| sha256(s.as_bytes())) == self.expected_meta,
            "revision_conflict: change metadata changed"
        );
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReceiptReport {
    pub report_id: String,
    pub digest: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcceptanceReceipt {
    pub format_version: u8,
    pub change_name: String,
    pub project_key: String,
    pub created_at: String,
    pub actor: Option<String>,
    pub status: String,
    pub policy_digest: String,
    pub knowledge_digest: String,
    pub scope_digest: String,
    pub source_freshness: SourceFreshness,
    pub reports: Vec<ReceiptReport>,
    pub decisions: Vec<RiskDecision>,
}
impl AcceptanceReceipt {
    pub fn validate(&mut self) -> Result<()> {
        ensure!(
            self.format_version == 1
                && self.status == "passed"
                && crate::security::report::valid_project_key(&self.project_key)
                && valid_digest(&self.policy_digest)
                && valid_digest(&self.knowledge_digest)
                && valid_digest(&self.scope_digest)
                && self.source_freshness != SourceFreshness::Unverified,
            "invalid_acceptance_receipt"
        );
        crate::security::prepared::validate_change(Some(&self.change_name))?;
        ensure!(
            chrono::DateTime::parse_from_rfc3339(&self.created_at).is_ok()
                && self.reports.len() == 2,
            "invalid_acceptance_receipt"
        );
        for r in &self.reports {
            validate_id(&r.report_id)?;
            ensure!(valid_digest(&r.digest), "invalid_acceptance_receipt");
        }
        self.actor = self.actor.as_deref().map(redact_text);
        for d in &mut self.decisions {
            d.validate()?;
            ensure!(
                d.policy_digest == self.policy_digest
                    && d.scope_digest == self.scope_digest
                    && self.reports.iter().any(|r| r.report_id == d.report_id),
                "invalid_acceptance_receipt"
            );
            d.reason = redact_text(&d.reason);
            d.actor = redact_text(&d.actor);
        }
        Ok(())
    }
    pub fn markdown(&self) -> String {
        let mut s=format!("# 安全驗收摘要\n\n- 提案：{}\n- 結果：{}\n- 時間：{}\n- 來源核對：{:?}\n- 政策指紋：{}\n- 知識庫指紋：{}\n- 來源指紋：{}\n\n此摘要保存當時裁決與接受理由，不包含完整審查證據或程式片段，也不作為新一輪驗收的證據。\n",self.change_name,self.status,self.created_at,self.source_freshness,self.policy_digest,self.knowledge_digest,self.scope_digest);
        for r in &self.reports {
            s.push_str(&format!(
                "\n- 報告：{}；SHA-256：{}\n",
                r.report_id, r.digest
            ));
        }
        for d in &self.decisions {
            s.push_str(&format!(
                "\n## 已接受風險 {} / {}\n\n- 接受者：{}\n- 接受時間：{}\n- 理由：{}\n",
                d.report_id,
                d.finding_id,
                redact_text(&d.actor),
                d.accepted_at,
                redact_text(&d.reason)
            ));
        }
        s
    }
}
pub fn acceptance_receipt(store: &dyn Store, change: &str) -> Result<Option<AcceptanceReceipt>> {
    crate::security::prepared::validate_change(Some(change))?;
    store.ensure_checks_readable(change)?;
    let Some(text) = super::read(store, change, "checks/acceptance.json") else {
        return Ok(None);
    };
    ensure!(
        text.len() <= 64 * 1024 * 1024,
        "size_limit: acceptance receipt"
    );
    let mut receipt: AcceptanceReceipt =
        serde_json::from_str(&text).map_err(|_| anyhow::anyhow!("invalid_acceptance_receipt"))?;
    receipt.validate()?;
    ensure!(
        receipt.change_name == change
            || store.archived_change_exists(change)
                && change.ends_with(&format!("-{}", receipt.change_name)),
        "binding_mismatch"
    );
    Ok(Some(receipt))
}
pub fn prepare_archive_checks(
    store: &dyn Store,
    change: &str,
    actor: Option<&str>,
    date: &str,
    stamp: &str,
    proof: Option<&AcceptanceProof>,
) -> Result<ArchiveChecksPlan> {
    let config = crate::config::WorkflowConfig::from_text(store.read_workflow_config().as_deref())?;
    let retention = config.check_records().archive;
    let original = store.read_artifact(change, INDEX);
    let mut index = index(store, change)?;
    let mut counts = ArchiveRecordCounts::default();
    for e in &index.entries {
        let (keep, count) = match e.kind {
            CheckKind::Review => (retention.review, &mut counts.review),
            CheckKind::Verify => (retention.verify, &mut counts.verify),
            CheckKind::Security => (retention.security, &mut counts.security),
        };
        if keep {
            count.retained += 1;
            if e.kind == CheckKind::Security {
                let json = store
                    .read_artifact(change, &e.relative_path.replace("report.md", "report.json"))
                    .ok_or_else(|| anyhow::anyhow!("missing_check_record"))?;
                let md = store
                    .read_artifact(change, &e.relative_path)
                    .ok_or_else(|| anyhow::anyhow!("missing_check_record"))?;
                crate::security::report::validate_report_pair(&e.record_id, &json, &md)?;
            }
        } else {
            count.omitted += 1;
        }
    }
    for entry in &mut index.entries {
        let keep = match entry.kind {
            CheckKind::Review => retention.review,
            CheckKind::Verify => retention.verify,
            CheckKind::Security => retention.security,
        };
        if !keep { entry.outcome = "omitted".into(); }
    }
    let original_meta = store.read_change_meta(change);
    let mut archived_meta = original_meta.clone();
    if let Some(meta) = &mut archived_meta {
        if !meta.ends_with('\n') {
            meta.push('\n');
        }
        if let Some(actor) = actor {
            meta.push_str(&format!("archived_by: {actor}\n"));
        }
        meta.push_str(&format!("archived_at: {date}\n"));
    }
    let policy = config.security();
    let receipt = if policy.enabled && policy.required_for_acceptance {
        crate::security::require_acceptance(store, change, proof)?;
        let proof = proof.ok_or_else(|| anyhow::anyhow!("missing_security_review"))?;
        let fresh = proof
            .fresh
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("source_freshness_unverified"))?;
        let models = proof
            .builtin
            .iter()
            .cloned()
            .map(VersionedReport::Builtin)
            .chain(proof.deep.iter().cloned().map(VersionedReport::Deep))
            .collect::<Vec<_>>();
        let reports = models
            .iter()
            .map(|r| {
                Ok(ReceiptReport {
                    report_id: r.report_id().into(),
                    digest: sha256(r.json()?.as_bytes()),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let mut effective = BTreeMap::new();
        for d in &proof.decisions {
            if d.policy_digest == fresh.policy_digest
                && d.scope_digest == fresh.scope_digest
                && reports.iter().any(|r| r.report_id == d.report_id)
            {
                effective.insert((d.report_id.clone(), d.finding_id.clone()), d.clone());
            }
        }
        let mut receipt = AcceptanceReceipt {
            format_version: 1,
            change_name: change.into(),
            project_key: proof.project_key.clone(),
            created_at: stamp.into(),
            actor: actor.map(str::to_owned),
            status: "passed".into(),
            policy_digest: fresh.policy_digest.clone(),
            knowledge_digest: fresh.knowledge_digest.clone(),
            scope_digest: fresh.scope_digest.clone(),
            source_freshness: fresh.source_freshness,
            reports,
            decisions: effective.into_values().collect(),
        };
        receipt.validate()?;
        counts.acceptance_receipt = true;
        let text = serde_json::to_string_pretty(&receipt)?;
        ensure!(
            text.len() <= 64 * 1024 * 1024,
            "storage_limit: acceptance receipt"
        );
        Some(text)
    } else {
        None
    };
    Ok(ArchiveChecksPlan {
        canonical_specs: BTreeMap::new(),
        retention: retention.clone(),
        expected_index: original.as_deref().map(|s| sha256(s.as_bytes())),
        expected_meta: original_meta.as_deref().map(|s| sha256(s.as_bytes())),
        index: if retention == ArchiveRecords::default() {
            None
        } else {
            Some(serde_json::to_string_pretty(&index)?)
        },
        receipt,
        archived_meta,
        counts,
    })
}
