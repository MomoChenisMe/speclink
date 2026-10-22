use speclink_core::{
    config::ResolvedPolicy,
    security::{
        data::{IntegrityKind, StandardRepository, ValidatedStandard},
        report::{DeepReportInput, ReportRepository},
        standards::{standard_source, StandardId},
    },
};
use speclink_host::{
    context::{Actor, ActorSource, ExecutionMode, SpeclinkExecutionContext},
    policy::EffectiveWorkflowPolicy,
    security::{
        self,
        remote::{Operation, Outcome},
    },
};
use speclink_protocol::security::*;
use speclink_store::{
    memory::MemoryStore, CommandContext, DocumentId, ProjectId, RepoId, Scope, TeamStore,
};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    sync::{atomic::AtomicBool, Arc},
};

#[test]
fn paired_remote_publication_is_atomic_scoped_fresh_and_cas_guarded() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("checkout");
    std::fs::create_dir_all(root.join("openspec/changes/demo")).unwrap();
    let policy="security_verification:\n  enabled: true\n  required_for_acceptance: true\n  standards:\n    owasp_top10: {enabled: true, version: '2025'}\n    cwe: {enabled: true, version: '4.20'}\n  reviews: {spec: false, code: true, api_boundary: false}\n";
    std::fs::write(root.join("openspec/config.yaml"), policy).unwrap();
    std::fs::write(root.join("app.ts"), "export const value = 1;\n").unwrap();
    let data = speclink_fs::security::FsSecurityData::new(&temp.path().join("data")).unwrap();
    let mut source = standard_source(StandardId::OwaspTop10, "2025").unwrap();
    let mut files = BTreeMap::new();
    for f in &mut source.files {
        f.sha256 = None;
        files.insert(
            f.name.clone(),
            format!("# {} fixture\n", f.item_id.as_deref().unwrap_or("license")).into_bytes(),
        );
    }
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(include_bytes!(
        "../../../engine/speclink-fs/tests/fixtures/security-guidance.zip"
    )))
    .unwrap();
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).unwrap();
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes).unwrap();
        files.insert(f.name().into(), bytes);
    }
    data.begin_install(StandardId::OwaspTop10, "2025")
        .unwrap()
        .commit(ValidatedStandard::new(source, files.clone(), IntegrityKind::Imported, &|| false).unwrap())
        .unwrap();
    // Model the official download: retain the original ZIP as well as expanded XML.
    let cwe = standard_source(StandardId::Cwe, "4.20").unwrap();
    let mut cwe_files = BTreeMap::from([("LICENSE".into(), b"CWE fixture terms".to_vec())]);
    let mut archive = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    archive.start_file("cwec_v4.20.xml", zip::write::SimpleFileOptions::default()).unwrap();
    archive.write_all(br#"<Weakness_Catalog xmlns="http://cwe.mitre.org/cwe-7" Version="4.20"><Weaknesses><Weakness ID="79" Name="Synthetic"><Description>Synthetic fixture</Description></Weakness></Weaknesses></Weakness_Catalog>"#).unwrap();
    let cwe_zip = archive.finish().unwrap().into_inner();
    cwe_files.insert("raw/cwec_v4.20.xml.zip".into(), cwe_zip.clone());
    for guidance in speclink_core::security::knowledge::registry().unwrap().files() {
        cwe_files.insert(guidance.name.clone(), files[&guidance.name].clone());
    }
    speclink_fs::security::expand_cwe(&cwe, &mut cwe_files, &|| false).unwrap();
    data.begin_install(StandardId::Cwe, "4.20").unwrap()
        .commit(ValidatedStandard::new(cwe, cwe_files, IntegrityKind::SourceTls, &|| false).unwrap()).unwrap();
    let key = security::remote_project_key("https://server.example", "demo", "backend").unwrap();
    let reader = speclink_fs::security::scan::AuthorizedProject::open(&root).unwrap();
    let mut target =
        security::ReviewTarget::local(reader.root(), "openspec", Some("demo".into())).unwrap();
    target.proposal_project_key = Some(key.clone());
    speclink_fs::security::knowledge::prepare(&root, &key, policy, &data, &|| false).unwrap();
    let device =
        speclink_fs::security::reports::FsSecurityReports::new(&temp.path().join("device"))
            .unwrap();
    let prepared =
        speclink_fs::security::prepared::FsPreparedReviews::new(&temp.path().join("prepared"))
            .unwrap();
    let budget = security::ReviewBudget::new(Arc::new(AtomicBool::new(false)));
    let observed = std::cell::RefCell::new(Vec::new());
    let basic = security::run_review(
        &target,
        Some(&reader),
        &data,
        &prepared,
        &device,
        &budget,
        &|p| {
            observed.borrow_mut().push(p.project_key);
        },
    )
    .unwrap()
    .report;
    assert!(
        observed
            .borrow()
            .iter()
            .all(|k| k == &target.binding.project_key()),
        "job routing remains bound to the device checkout"
    );
    let local = speclink_fs::FsStore::new(&root, "openspec");
    speclink_core::security::report::save_change_report(
        &local,
        "demo",
        &speclink_core::security::report::VersionedReport::Builtin(basic.clone()),
    )
    .unwrap();
    let (context, builtin_id) =
        security::submission_context(&target, &reader, &data, &local, None, &budget).unwrap();
    let zip_path = reader.root().join(".security_kb/standards/cwe/4.20/raw/cwec_v4.20.xml.zip");
    assert_eq!(std::fs::read(&zip_path).unwrap(), cwe_zip);
    std::fs::write(&zip_path, b"tampered ZIP").unwrap();
    assert!(security::submission_context(&target, &reader, &data, &local, None, &budget)
        .err().unwrap().to_string().contains("integrity_error"));
    std::fs::write(&zip_path, &cwe_zip).unwrap();
    assert_eq!(builtin_id.as_deref(), Some(basic.report_id()));
    let frame = security::source_attestation(&context, &data).unwrap();
    let submission = security::Submission::from(&context);
    let input:DeepReportInput=serde_json::from_value(serde_json::json!({"formatVersion":2,"changeName":"demo","policyDigest":submission.policy_digest,"knowledgeDigest":submission.knowledge_digest,"scopeDigest":submission.scope_digest,"builtinReportId":builtin_id,"complete":true,"targets":submission.targets,"checks":submission.required_checks.iter().map(|id|serde_json::json!({"checkId":id,"status":"PASS","reason":"人工追蹤已確認","evidencePaths":submission.targets,"resolvesCheckIds":submission.unverified_checks})).collect::<Vec<_>>(),"findings":[{"findingId":"H1","title":"跨帳戶授權缺失","severity":"high","confidence":"high","confidenceReason":"程式缺少控制；無可信對照","references":[],"evidence":[{"path":"app.ts","line":1,"endLine":1,"function":"handler"}],"codeSnippet":"export const value = 1;","dataFlow":"input → handler → store","impact":"程式缺失已確認；外部防護未確認","remediation":"SHALL 依本機授權指南核對資源擁有者。","acceptance":"帳戶 A 存取 B 應回 403。"}]})).unwrap();
    let request = SecurityRoundRequest {
        project_key: key.clone(),
        expected_revision: 0,
        report: SecurityRoundPayload::Deep(
            serde_json::from_value(serde_json::to_value(input).unwrap()).unwrap(),
        ),
        builtin_report: Some(
            serde_json::from_value(serde_json::to_value(&basic).unwrap()).unwrap(),
        ),
        security_source_attestation: Some(frame.clone()),
    };
    let server = MemoryStore::new();
    let scope = Scope::new(ProjectId::new("demo"), RepoId::new("backend"));
    let ctx = SpeclinkExecutionContext {
        actor: Actor::Identified {
            display: "Reviewer".into(),
            source: ActorSource::Explicit,
        },
        project: scope.project.clone(),
        repo: scope.repo.clone(),
        mode: ExecutionMode::SharedStore,
        policy: EffectiveWorkflowPolicy::new(
            ResolvedPolicy {
                locale: "English".into(),
                spec_locale: None,
                tdd: false,
                audit: false,
                worktree: false,
            },
            "",
        ),
    };
    let mut seed = server
        .begin_unit_of_work(
            &scope,
            CommandContext {
                command: "seed".into(),
                actor: "seed".into(),
            },
        )
        .unwrap();
    seed.create(DocumentId::WorkflowConfig, policy);
    seed.create(
        DocumentId::ChangeArtifact {
            change: "demo".into(),
            artifact: "tasks.md".into(),
        },
        "- [x] 1.1 Implemented\n",
    );
    seed.create(
        DocumentId::ChangeMeta {
            change: "demo".into(),
        },
        "schema: spec-driven\n",
    );
    server.commit(seed, vec![]).unwrap();
    let before = server.snapshot(&scope).unwrap().revision();
    let mut invalid = request.clone();
    if let SecurityRoundPayload::Deep(input) = &mut invalid.report {
        input.targets.push("../other.ts".into());
    }
    assert!(security::remote::execute(
        &server,
        &ctx,
        &key,
        "demo",
        &data,
        Operation::Submit(invalid)
    )
    .is_err());
    assert_eq!(
        server.snapshot(&scope).unwrap().revision(),
        before,
        "failed deep validation must not publish the paired basic report"
    );
    let Outcome::Submitted(receipt) = security::remote::execute(
        &server,
        &ctx,
        &key,
        "demo",
        &data,
        Operation::Submit(request.clone()),
    )
    .unwrap() else {
        panic!()
    };
    assert!(receipt.complete);
    let saved = security::remote::report(&server, &ctx, "demo", basic.report_id()).unwrap();
    assert_eq!(serde_json::to_value(saved).unwrap()["reportId"], basic.report_id());
    assert!(
        device.get(basic.report_id()).unwrap().is_some(),
        "device draft survives publication"
    );
    let Outcome::View(view) = security::remote::execute(
        &server,
        &ctx,
        &key,
        "demo",
        &data,
        Operation::Show(Some(frame.clone())),
    )
    .unwrap() else {
        panic!()
    };
    assert_eq!(view.reports.len(), 2);
    assert_eq!(view.acceptance.status, SecurityAcceptanceStatus::Blocked);
    let accept = SecurityAcceptRequest {
        expected_revision: view.expected_revision.unwrap(),
        report_id: receipt.report_id.clone(),
        finding_id: "H1".into(),
        reason: "業務負責人接受；api_key = 'unique-remote-decision-secret'".into(),
        security_source_attestation: frame.clone(),
    };
    let Outcome::Accepted(_) = security::remote::execute(
        &server,
        &ctx,
        &key,
        "demo",
        &data,
        Operation::Accept(accept.clone()),
    )
    .unwrap() else {
        panic!()
    };
    assert!(security::remote::execute(
        &server,
        &ctx,
        &key,
        "demo",
        &data,
        Operation::Accept(accept)
    )
    .unwrap_err()
    .to_string()
    .starts_with("revision_conflict"));
    let Outcome::View(accepted) = security::remote::execute(
        &server,
        &ctx,
        &key,
        "demo",
        &data,
        Operation::Show(Some(frame.clone())),
    )
    .unwrap() else {
        panic!()
    };
    assert_eq!(accepted.acceptance.status, SecurityAcceptanceStatus::Passed);
    assert!(!serde_json::to_string(&accepted)
        .unwrap()
        .contains("unique-remote-decision-secret"));
    assert!(accepted.expected_revision.unwrap() > view.expected_revision.unwrap());
    assert_eq!(
        view.acceptance.source_freshness,
        SecuritySourceFreshness::ClientAttested
    );
    assert!(security::remote::execute(
        &server,
        &ctx,
        &key,
        "demo",
        &data,
        Operation::Submit(request)
    )
    .unwrap_err()
    .to_string()
    .starts_with("revision_conflict"));
    let Outcome::View(unverified) =
        security::remote::execute(&server, &ctx, &key, "demo", &data, Operation::Show(None))
            .unwrap()
    else {
        panic!()
    };
    assert_ne!(
        unverified.acceptance.status,
        SecurityAcceptanceStatus::Passed
    );
    use speclink_core::command::Command;
    speclink_host::bridge::execute(
        &server,
        &ctx,
        Command::VerifyAddRound {
            change: "demo".into(),
            content: "**Scope**: app.ts\n".into(),
        },
    )
    .unwrap();
    let stamp = Command::VerifyStamp {
        change: "demo".into(),
        accept: true,
        tool: None,
        scope: vec![speclink_core::model::ReviewedScopeEntry {
            path: "app.ts".into(),
            hash: speclink_core::security::data::sha256(b"export const value = 1;\n"),
        }],
        missing: vec![],
    };
    assert!(
        speclink_host::bridge::execute(&server, &ctx, stamp.clone()).is_err(),
        "ordinary --accept cannot replace current source attestation"
    );
    let gate = SecurityGateRequest {
        expected_revision: accepted.expected_revision,
        security_source_attestation: Some(frame.clone()),
    };
    speclink_host::bridge::execute_with_security(&server, &ctx, stamp, &key, &gate, &data).unwrap();
    let mut expired = frame.clone();
    expired.checked_at = "2020-01-01T00:00:00Z".into();
    assert!(security::remote::execute(
        &server,
        &ctx,
        &key,
        "demo",
        &data,
        Operation::Show(Some(expired))
    )
    .is_err());
    let config_doc = server
        .snapshot(&scope)
        .unwrap()
        .read(&DocumentId::WorkflowConfig)
        .unwrap()
        .unwrap();
    let mut update = server
        .begin_unit_of_work(
            &scope,
            CommandContext {
                command: "archive-policy".into(),
                actor: "Reviewer".into(),
            },
        )
        .unwrap();
    update.update(DocumentId::WorkflowConfig,format!("{policy}check_records:\n  archive: {{review: false, verify: false, security: false}}\n"),config_doc.revision);
    server.commit(update, vec![]).unwrap();
    let Outcome::View(current) = security::remote::execute(
        &server,
        &ctx,
        &key,
        "demo",
        &data,
        Operation::Show(Some(frame.clone())),
    )
    .unwrap() else {
        panic!()
    };
    let archive = Command::Archive {
        change: Some("demo".into()),
        skip_specs: true,
        no_validate: true,
        mark_tasks_complete: false,
        carry_review: false,
        carry_verify: false,
    };
    let stale = SecurityGateRequest {
        expected_revision: accepted.expected_revision,
        security_source_attestation: Some(frame.clone()),
    };
    assert!(
        speclink_host::bridge::execute_with_security(
            &server,
            &ctx,
            archive.clone(),
            &key,
            &stale,
            &data
        )
        .is_err(),
        "archive must CAS the latest index"
    );
    let gate = SecurityGateRequest {
        expected_revision: current.expected_revision,
        security_source_attestation: Some(frame),
    };
    let result =
        speclink_host::bridge::execute_with_security(&server, &ctx, archive, &key, &gate, &data)
            .unwrap();
    let speclink_core::command::CommandOutcome::Archive(result) = result.outcome else {
        panic!()
    };
    assert_eq!(result.check_records.security.omitted, 2);
    assert!(result.check_records.acceptance_receipt);
    let snapshot = server.snapshot(&scope).unwrap();
    let saved = snapshot
        .read(&DocumentId::ArchivedChange {
            change: result.dated_name.clone(),
            doc: "checks/acceptance.json".into(),
        })
        .unwrap()
        .unwrap();
    let mut receipt: speclink_core::history::AcceptanceReceipt =
        serde_json::from_str(&saved.content).unwrap();
    receipt.validate().unwrap();
    assert_eq!(receipt.reports.len(), 2);
    assert_eq!(receipt.decisions.len(), 1);
    assert!(!saved.content.contains("unique-remote-decision-secret"));
    assert!(!saved.content.contains("codeSnippet"));
    let index = snapshot
        .read(&DocumentId::ArchivedChange {
            change: result.dated_name.clone(),
            doc: "checks/index.json".into(),
        })
        .unwrap()
        .unwrap();
    let omitted = serde_json::from_str::<speclink_core::history::CheckIndex>(&index.content).unwrap().entries;
    assert_eq!(omitted.len(), 3);
    assert_eq!(omitted.iter().filter(|entry| entry.kind == speclink_core::history::CheckKind::Security).count(), 2);
    assert!(omitted.iter().all(|entry| entry.outcome == "omitted"));
    assert!(snapshot
        .read(&DocumentId::ArchivedChange {
            change: result.dated_name.clone(),
            doc: format!("checks/security/{}/report.json", basic.report_id())
        })
        .unwrap()
        .is_none());
    let speclink_protocol::query::CheckHistoryResponse::List(history) =
        speclink_host::history::query_remote(
            &server,
            &scope,
            &result.dated_name,
            speclink_core::history::CheckKind::Security,
            None,
        )
        .unwrap()
    else {
        panic!()
    };
    let value = serde_json::to_value(history).unwrap();
    assert!(value["acceptanceReceipt"]["content"]
        .as_str()
        .unwrap()
        .contains("不作為新一輪驗收的證據"));
    let Outcome::View(archived) = security::remote::execute(
        &server,
        &ctx,
        &key,
        &result.dated_name,
        &data,
        Operation::Show(None),
    )
    .unwrap() else {
        panic!()
    };
    assert_ne!(
        archived.acceptance.status,
        SecurityAcceptanceStatus::Passed,
        "a receipt cannot replace current source evidence"
    );
}
