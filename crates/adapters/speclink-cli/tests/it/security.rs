use std::{
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "speclink-security-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        std::fs::create_dir_all(path.with_extension("home")).unwrap();
        Self(path)
    }
    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_speclink"));
        command.args(args)
            .current_dir(&self.0)
            .env("HOME", self.0.with_extension("home"))
            .env("USERPROFILE", self.0.with_extension("home"))
            .env("XDG_CONFIG_HOME", self.0.with_extension("home").join("config"))
            .env("XDG_DATA_HOME", self.0.with_extension("home").join("data"))
            .env("LOCALAPPDATA", self.0.with_extension("home").join("data"))
            .env("SPECLINK_TOKEN", "fixture-token");
        command
    }
    fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }
    fn input(&self, args: &[&str], input: &str) -> Output {
        use std::io::Write;
        use std::process::Stdio;
        let mut child = self.command(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
        child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
        child.wait_with_output().unwrap()
    }
}

#[test]
fn security_data_custom_root_preserves_outputs_and_disconnection_never_falls_back() {
    let fixture=Fixture::new();
    let baseline=fixture.run(&["security","data","list","--json","--no-color"]);
    assert!(baseline.status.success());
    let home=fixture.0.with_extension("home");
    let config=if cfg!(windows) {home.join("AppData/Roaming/speclink")} else if cfg!(target_os="macos") {home.join("Library/Application Support/speclink")} else {home.join("config/speclink")};
    std::fs::create_dir_all(&config).unwrap();let shared=fixture.0.join("shared");std::fs::create_dir(&shared).unwrap();
    std::fs::write(config.join("security-data-preferences.json"),serde_json::to_vec(&serde_json::json!({"formatVersion":1,"standardsPath":shared})).unwrap()).unwrap();
    let out=fixture.run(&["security","data","list","--json","--no-color"]);
    assert!(out.status.success(),"{}",String::from_utf8_lossy(&out.stderr));
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&out.stdout).unwrap(),serde_json::from_slice::<serde_json::Value>(&baseline.stdout).unwrap());
    let human=fixture.run(&["security","data","list","--no-color"]);assert!(human.status.success());assert!(!human.stdout.contains(&27));
    std::fs::rename(&shared,fixture.0.join("disconnected")).unwrap();
    let failed=fixture.run(&["security","data","list","--json"]);
    assert_eq!(failed.status.code(),Some(1));assert!(failed.stdout.is_empty());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("data_directory_unavailable"));
}

#[test]
fn disconnected_standards_do_not_block_existing_private_reports() {
    let fixture = Fixture::new();
    std::fs::create_dir_all(fixture.0.join("openspec")).unwrap();
    std::fs::write(fixture.0.join(".speclink.yaml"), "tools: [codex]\n").unwrap();
    let output = fixture.run(&["security", "run", "--json"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let id = report["reportId"].as_str().unwrap();
    let home = fixture.0.with_extension("home");
    let config = if cfg!(windows) { home.join("AppData/Roaming/speclink") } else if cfg!(target_os="macos") { home.join("Library/Application Support/speclink") } else { home.join("config/speclink") };
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(config.join("security-data-preferences.json"), serde_json::to_vec(&serde_json::json!({"formatVersion":1,"standardsPath":fixture.0.join("disconnected")})).unwrap()).unwrap();
    for format in ["json", "markdown"] {
        let output = fixture.run(&["security", "report", id, "--format", format, "--no-color"]);
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(!output.stdout.is_empty());
    }
    let output = fixture.run(&["security", "data", "list", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
}

#[test]
fn security_acceptance_and_record_policy_cli_contract() {
    let fixture = Fixture::new();
    std::fs::create_dir(fixture.0.join("openspec")).unwrap();
    let path = fixture.0.join("openspec/config.yaml");
    let original = "schema: spec-driven\n# keep\nfuture: untouched\n";
    std::fs::write(&path, original).unwrap();
    let args = ["workflow-config", "check-records", "--stdin", "--no-color"];
    let dry = fixture.input(&["workflow-config", "check-records", "--stdin", "--dry-run", "--no-color"], "archive: {security: false}");
    assert!(dry.status.success(), "{}", String::from_utf8_lossy(&dry.stderr));
    assert!(!dry.stdout.contains(&27));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    let out = fixture.input(&args, "archive: {security: false}");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("check_records updated"));
    let out = fixture.input(&["workflow-config", "security", "--stdin"], "required_for_acceptance: true");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let out = fixture.run(&["workflow-config", "show", "--json", "--no-color"]);
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["checkRecords"]["archive"], serde_json::json!({"review":true,"verify":true,"security":false}));
    assert_eq!(json["securityVerification"]["requiredForAcceptance"], true);
    assert!(!out.stdout.contains(&27));
    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(saved.contains("# keep\nfuture: untouched\n"));
    for input in ["", "archive: {typo: true}", "archive: {security: 'false'}"] {
        let out = fixture.input(&args, input);
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stdout.is_empty());
        assert!(!out.stderr.is_empty());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), saved);
    }
    assert_eq!(fixture.run(&["workflow-config", "check-records"]).status.code(), Some(1));
    assert_eq!(fixture.run(&["workflow-config", "check-records", "--json"]).status.code(), Some(2));
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
        let _ = std::fs::remove_dir_all(self.0.with_extension("home"));
    }
}

#[test]
fn security_kb_prepare_is_offline_and_missing_data_does_not_create_snapshot() {
    let fixture = Fixture::new();
    std::fs::create_dir(fixture.0.join("openspec")).unwrap();
    std::fs::write(fixture.0.join("openspec/config.yaml"), "security_verification:\n  enabled: true\n  standards:\n    owasp_top10: {enabled: true, version: '2025'}\n").unwrap();
    let output = fixture.run(&["security", "kb", "prepare", "--json", "--no-color"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing_knowledge"));
    assert!(!fixture.0.join(".security_kb").exists());
    let with_change = fixture.run(&["security", "kb", "prepare", "--change", "demo", "--json", "--no-color"]);
    assert_eq!(with_change.status.code(), Some(1), "--change must resolve the selected preparation, then report missing installed guidance");
    assert!(String::from_utf8_lossy(&with_change.stderr).contains("missing_knowledge"));
    assert_eq!(fixture.run(&["security", "kb", "prepare", "--stdin"]).status.code(), Some(2));
}

#[test]
fn security_data_list_is_mode_free_offline_json_and_no_color() {
    let fixture = Fixture::new();
    // A remote binding must not cause a handshake for device-local data.
    std::fs::write(
        fixture.0.join(".speclink.yaml"),
        "mode: remote\nremote:\n  url: http://127.0.0.1:1\n  project: never-contact\n",
    )
    .unwrap();
    let output = fixture.run(&["security", "data", "list", "--json", "--no-color"]);
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stderr), String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.stderr.is_empty());
    let data: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(data["standards"].as_array().unwrap().len(), 4);
    for standard in data["standards"].as_array().unwrap() {
        assert!(standard["standardId"].is_string());
        for version in standard["versions"].as_array().unwrap() {
            for key in [
                "version",
                "status",
                "sourceUrl",
                "licenseName",
                "licenseUrl",
                "attribution",
                "sourceFormat",
            ] {
                assert!(version[key].is_string(), "{key}");
            }
            assert_eq!(version["status"], "missing");
        }
    }
    assert!(!output.stdout.contains(&27));
    assert!(!fixture.0.join("security").exists());
    assert!(!fixture.0.join("openspec").exists());
}

#[test]
fn security_data_invalid_inputs_never_emit_success_payload() {
    let fixture = Fixture::new();
    for args in [
        vec!["security", "data", "import", "missing.zip", "--json"],
        vec![
            "security",
            "data",
            "install",
            "owasp_top10",
            "--version",
            "2099",
            "--json",
        ],
        vec![
            "security",
            "data",
            "remove",
            "cwe",
            "--version",
            "../../escape",
            "--json",
        ],
    ] {
        let output = fixture.run(&args);
        assert_eq!(
            output.status.code(),
            Some(1),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
    let syntax = fixture.run(&["security", "data", "install", "owasp_top10"]);
    assert_eq!(syntax.status.code(), Some(2));
}

#[test]
fn security_run_rules_prepare_and_report_are_offline_and_redacted() {
    let fixture = Fixture::new();
    std::fs::create_dir(fixture.0.join("openspec")).unwrap();
    std::fs::write(fixture.0.join(".speclink.yaml"), "tools: []\n").unwrap();
    std::fs::write(fixture.0.join("openspec/config.yaml"),"locale: tw\ntdd: true\naudit: true\nsecurity_verification:\n  enabled: true\n  reviews: {spec: false, api_boundary: false}\n").unwrap();
    std::fs::write(
        fixture.0.join("app.ts"),
        "const token='private-cli-security-sentinel-9595';",
    )
    .unwrap();
    let rules = fixture.run(&["security", "rules", "--json", "--no-color"]);
    assert!(rules.status.success());
    let rules: serde_json::Value = serde_json::from_slice(&rules.stdout).unwrap();
    assert_eq!(rules["rules"].as_array().unwrap().len(), 19);
    assert!(rules["ruleVersion"].is_string());
    let output = fixture.run(&["security", "run", "--json", "--no-color"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["counts"]["findings"], 1);
    assert_eq!(report["configurationState"], "unconfigured");
    assert_eq!(report["executionMode"], "builtin-offline");
    assert!(report["complete"].as_bool().unwrap());
    assert!(!output.stdout.contains(&27));
    let id = report["reportId"].as_str().unwrap();
    for args in [
        vec!["security", "report", id],
        vec![
            "security",
            "report",
            id,
            "--format",
            "markdown",
            "--no-color",
        ],
    ] {
        let exported = fixture.run(&args);
        assert!(exported.status.success());
        assert!(!String::from_utf8_lossy(&exported.stdout)
            .contains("private-cli-security-sentinel-9595"));
        assert!(!exported.stdout.contains(&27));
    }
    let prepared = fixture.run(&["security", "prepare", "--json"]);
    assert!(
        prepared.status.success(),
        "{}",
        String::from_utf8_lossy(&prepared.stderr)
    );
    let data: serde_json::Value = serde_json::from_slice(&prepared.stdout).unwrap();
    assert!(data["projectKey"].is_string());
    assert_eq!(data["policyRevision"], serde_json::Value::Null);
    assert!(data["documentCount"].is_number());
    assert_eq!(data["checkoutAvailable"], true);
    let bad = fixture.run(&["security", "report", "../../secret"]);
    assert_eq!(bad.status.code(), Some(1));
    assert!(bad.stdout.is_empty());
    assert!(!fixture.0.join("openspec/changes").exists());
}

#[test]
fn security_run_missing_data_is_incomplete_and_disabled_legacy_reads_no_sources() {
    let fixture = Fixture::new();
    std::fs::create_dir(fixture.0.join("openspec")).unwrap();
    std::fs::write(fixture.0.join(".speclink.yaml"), "tools: []\n").unwrap();
    std::fs::write(
        fixture.0.join("openspec/config.yaml"),
        "tdd: true\naudit: true\n",
    )
    .unwrap();
    std::fs::write(
        fixture.0.join("app.ts"),
        "const token='never-read-disabled-sentinel';",
    )
    .unwrap();
    let output = fixture.run(&["security", "run", "--json"]);
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["configurationState"], "disabled");
    assert_eq!(report["coverage"]["bytesRead"], 0);
    assert_eq!(report["counts"]["findings"], 0);
    std::fs::write(fixture.0.join("openspec/config.yaml"),"security_verification:\n  enabled: true\n  reviews: {spec: false, api_boundary: false}\n  standards:\n    owasp_top10: {enabled: true, version: '2025'}\n").unwrap();
    let output = fixture.run(&["security", "run", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["complete"], false);
    assert!(String::from_utf8_lossy(&output.stdout).contains("missing_data"));
    let unknown = fixture.run(&["security", "run", "--change", "unknown", "--json"]);
    assert_eq!(unknown.status.code(), Some(1));
    assert!(unknown.stdout.is_empty());
    let syntax = fixture.run(&["security", "report"]);
    assert_eq!(syntax.status.code(), Some(2));
}

#[cfg(unix)]
#[test]
fn security_run_ctrl_c_returns_130_with_a_saved_partial_report() {
    use std::{
        io::{BufRead, BufReader},
        process::Stdio,
        time::{Duration, Instant},
    };
    let fixture = Fixture::new();
    std::fs::create_dir(fixture.0.join("openspec")).unwrap();
    std::fs::write(fixture.0.join(".speclink.yaml"), "tools: []\n").unwrap();
    std::fs::write(
        fixture.0.join("openspec/config.yaml"),
        "security_verification:\n  enabled: true\n  reviews: {spec: false, api_boundary: false}\n",
    )
    .unwrap();
    let source = "const value=1;\n".repeat(60_000);
    for index in 0..50 {
        std::fs::write(fixture.0.join(format!("source-{index}.ts")), &source).unwrap();
    }
    let mut child = Command::new(env!("CARGO_BIN_EXE_speclink"))
        .args(["security", "run", "--json"])
        .current_dir(&fixture.0)
        .env("HOME", fixture.0.with_extension("home"))
        .env(
            "XDG_DATA_HOME",
            fixture.0.with_extension("home").join("data"),
        )
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stderr = BufReader::new(child.stderr.take().unwrap());
    let mut line = String::new();
    assert!(stderr.read_line(&mut line).unwrap() > 0);
    assert!(line.contains("reading"), "{line}");
    let started = Instant::now();
    assert!(Command::new("/bin/kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .unwrap()
        .success());
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(130));
    assert!(started.elapsed() < Duration::from_secs(2));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["complete"], false);
    let id = report["reportId"].as_str().unwrap();
    assert!(fixture.run(&["security", "report", id]).status.success());
}

#[test]
fn security_prepare_remote_is_explicit_atomic_and_run_makes_zero_requests() {
    use std::{
        sync::{atomic::AtomicBool, Arc, Mutex},
        time::Duration,
    };
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").unwrap());
    let base = format!(
        "http://{}/api/speclink/v1/projects/demo",
        server.server_addr()
    );
    let stop = Arc::new(AtomicBool::new(false));
    let fail = Arc::new(AtomicBool::new(false));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let (service, done, failed, captured) =
        (server.clone(), stop.clone(), fail.clone(), requests.clone());
    let policy="locale: tw\nsecurity_verification:\n  enabled: true\n  reviews: {spec: false, api_boundary: false}\n";
    let documents = vec![
        speclink_protocol::context::ContextDocument {
            path: "openspec/config.yaml".into(),
            content: policy.into(),
            revision: Some(7),
            digest: format!(
                "sha256:{}",
                speclink_core::security::data::sha256(policy.as_bytes())
            ),
        },
        speclink_protocol::context::ContextDocument {
            path: "openspec/specs/auth/spec.md".into(),
            content: "API authentication requirements".into(),
            revision: Some(4),
            digest: format!(
                "sha256:{}",
                speclink_core::security::data::sha256(b"API authentication requirements")
            ),
        },
    ];
    let snapshot = serde_json::to_string(&speclink_protocol::context::ContextSnapshot {
        snapshot_id: "snapshot-test".into(),
        policy_revision: Some(7),
        digest: "sha256:fixture".into(),
        documents,
    })
    .unwrap();
    let worker = std::thread::spawn(move || {
        while !done.load(Ordering::Relaxed) {
            let Some(mut request) = service.recv_timeout(Duration::from_millis(50)).unwrap() else {
                continue;
            };
            let mut body = String::new();
            request.as_reader().read_to_string(&mut body).unwrap();
            let path = request.url().to_owned();
            captured.lock().unwrap().push((path.clone(), body));
            let (status, body) = if failed.load(Ordering::Relaxed) {
                (401,r#"{"status":401,"reason":"unauthenticated","message":"private-server-error-body-sentinel"}"#.to_owned())
            } else if path.ends_with("/binding") {
                (
                    200,
                    format!(
                        r#"{{"actor":{{"id":"u","name":"Test"}},"project":{{"id":"p","key":"demo","name":"Demo"}},"repo":{{"id":"r","key":"backend","name":"Backend"}},"apiVersion":"1","engineVersion":"{}","capabilities":{{"contextSnapshots":true,"securityReview":true}}}}"#,
                        env!("CARGO_PKG_VERSION")
                    ),
                )
            } else if path.ends_with("/context") {
                (200, snapshot.clone())
            } else {
                (404, "{}".into())
            };
            let _ =
                request.respond(tiny_http::Response::from_string(body).with_status_code(status));
        }
    });
    let fixture = Fixture::new();
    std::fs::write(
        fixture.0.join(".speclink.yaml"),
        format!("remote:\n  url: {base}\n  repo: backend\n"),
    )
    .unwrap();
    std::fs::write(
        fixture.0.join("app.ts"),
        "const token='unique-remote-source-sentinel';",
    )
    .unwrap();
    let prepared = fixture.run(&["security", "prepare", "--json"]);
    assert!(
        prepared.status.success(),
        "{}",
        String::from_utf8_lossy(&prepared.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&prepared.stdout).unwrap()["documentCount"],
        1
    );
    fail.store(true, Ordering::Relaxed);
    let rejected = fixture.run(&["security", "prepare", "--json"]);
    assert_eq!(rejected.status.code(), Some(1));
    assert!(rejected.stdout.is_empty());
    assert!(
        !String::from_utf8_lossy(&rejected.stderr).contains("private-server-error-body-sentinel")
    );
    let before = requests.lock().unwrap().len();
    let run = fixture.run(&["security", "run", "--json"]);
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(requests.lock().unwrap().len(), before);
    let report: serde_json::Value = serde_json::from_slice(&run.stdout).unwrap();
    assert_eq!(report["policyRevision"], 7);
    assert!(report["preparedAt"].is_string());
    assert_eq!(report["counts"]["findings"], 1);
    assert!(!fixture.0.join(".speclink/context").exists());
    assert!(requests
        .lock()
        .unwrap()
        .iter()
        .all(|(_, body)| !body.contains("unique-remote-source-sentinel")));
    stop.store(true, Ordering::Relaxed);
    worker.join().unwrap();
}
#[test]
fn security_run_invalid_policy_errors_do_not_echo_values() {
    let fixture = Fixture::new();
    std::fs::create_dir(fixture.0.join("openspec")).unwrap();
    std::fs::write(
        fixture.0.join("openspec/config.yaml"),
        "security_verification: {enabled: private-policy-credential-sentinel}\n",
    )
    .unwrap();
    let output = fixture.run(&["security", "run", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("private-policy-credential-sentinel"));
}

#[test]
fn security_top10_2025_only_installed_then_offline_without_ai_or_archive_gate() {
    use speclink_core::security::{
        data::{IntegrityKind, StandardRepository, ValidatedStandard},
        standards::{standard_source, StandardId},
    };
    use std::collections::BTreeMap;
    let fixture = Fixture::new();
    let home = fixture.0.with_extension("home");
    #[cfg(target_os = "macos")]
    let app_data = home.join("Library/Application Support/speclink");
    #[cfg(not(target_os = "macos"))]
    let app_data = home.join("data/speclink");
    let repository = speclink_fs::security::FsSecurityData::new(&app_data).unwrap();
    let mut source = standard_source(StandardId::OwaspTop10, "2025").unwrap();
    // Synthetic licensed data exercises the real installer/parser. This is not an
    // official download and deliberately carries Imported rather than pinned trust.
    let mut files = BTreeMap::new();
    for file in &mut source.files {
        file.sha256 = None;
        let bytes = if let Some(id) = &file.item_id {
            format!("# {id} Synthetic fixture\n\n## List of Mapped CWEs\n[CWE-798](https://cwe.mitre.org/data/definitions/798.html)\n").into_bytes()
        } else {
            b"Synthetic test data; not official standard prose".to_vec()
        };
        files.insert(file.name.clone(), bytes);
    }
    {
        use std::io::Read;
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(include_bytes!("../../../../engine/speclink-fs/tests/fixtures/security-guidance.zip"))).unwrap();
        for index in 0..archive.len() {
            let mut file = archive.by_index(index).unwrap();
            let mut bytes = Vec::new(); file.read_to_end(&mut bytes).unwrap();
            files.insert(file.name().into(), bytes);
        }
    }
    repository
        .begin_install(StandardId::OwaspTop10, "2025")
        .unwrap()
        .commit(ValidatedStandard::new(source, files, IntegrityKind::Imported, &|| false).unwrap())
        .unwrap();
    std::fs::create_dir_all(fixture.0.join("openspec/changes/demo")).unwrap();
    std::fs::write(fixture.0.join(".speclink.yaml"), "tools: [codex]\n").unwrap();
    std::fs::write(
        fixture.0.join("openspec/changes/demo/.openspec.yaml"),
        "schema: spec-driven\ncreated: 2026-10-17\n",
    )
    .unwrap();
    std::fs::write(
        fixture.0.join("openspec/changes/demo/proposal.md"),
        "## Why\n\nFixture.\n",
    )
    .unwrap();
    std::fs::write(
        fixture.0.join("openspec/changes/demo/tasks.md"),
        "- [x] 1.1 Done\n",
    )
    .unwrap();
    std::fs::write(fixture.0.join("openspec/config.yaml"),"audit: false\nsecurity_verification:\n  enabled: true\n  scope: web-api\n  standards:\n    owasp_top10: {enabled: true, version: '2025'}\n  reviews: {spec: false, code: true, api_boundary: false}\n").unwrap();
    let kb = fixture.run(&["security", "kb", "prepare", "--json", "--no-color"]);
    assert!(kb.status.success(), "{}", String::from_utf8_lossy(&kb.stderr));
    assert!(kb.stderr.is_empty());
    assert!(!kb.stdout.contains(&27));
    let snapshot: serde_json::Value = serde_json::from_slice(&kb.stdout).unwrap();
    for key in ["projectKey", "path", "policyDigest", "knowledgeDigest"] { assert!(snapshot[key].is_string()); }
    assert!(snapshot["files"].is_u64()); assert!(snapshot["bytes"].is_u64());
    assert_eq!(snapshot["standards"].as_array().unwrap().len(), 1);
    assert_eq!(snapshot["standards"][0]["version"], "2025");
    assert!(fixture.0.join(".security_kb/standards/owasp_top10/2025").is_dir());
    for absent in ["2021", "cwe", "owasp_asvs", "owasp_api_top10"] { assert!(!fixture.0.join(".security_kb/standards").join(absent).exists()); }
    assert!(!fixture.0.join(".security_kb/mappings/owasp_2021_cwe.json").exists());
    let human = fixture.run(&["security", "kb", "prepare", "--no-color"]);
    assert!(human.status.success());
    let human_text=String::from_utf8_lossy(&human.stdout);
    assert!(human_text.contains("Prepared knowledge at"));
    for key in ["projectKey", "policyDigest"] {assert!(human_text.contains(snapshot[key].as_str().unwrap()), "{key}: {human_text}");}
    for key in ["standardId", "version", "dataSha256"] {assert!(human_text.contains(snapshot["standards"][0][key].as_str().unwrap()), "{key}: {human_text}");}
    std::fs::write(
        fixture.0.join("app.ts"),
        "const api_key = 'offline-only-private-sentinel';\n",
    )
    .unwrap();
    // A proxy trap observes zero HTTP requests while the review runs with no AI.
    let trap = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let proxy = format!("http://{}", trap.server_addr());
    let output = Command::new(env!("CARGO_BIN_EXE_speclink"))
        .args(["security", "run", "--change", "demo", "--json"])
        .current_dir(&fixture.0)
        .env("HOME", &home)
        .env("XDG_DATA_HOME", home.join("data"))
        .env("LOCALAPPDATA", home.join("data"))
        .env("HTTP_PROXY", &proxy)
        .env("HTTPS_PROXY", &proxy)
        .env_remove("SPECLINK_TOKEN")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stderr), String::from_utf8_lossy(&output.stdout)
    );
    assert!(trap.try_recv().unwrap().is_none());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["counts"]["findings"], 1);
    assert_eq!(report["executionMode"], "builtin-offline");
    let selected = report["standards"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["status"] != "DISABLED")
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0]["version"], "2025");
    assert_eq!(selected[0]["installedStatus"], "installed");
    let reference = &report["findings"][0]["references"][0];
    assert_eq!(reference["standardId"], "owasp_top10");
    assert_eq!(reference["version"], "2025");
    let id = report["reportId"].as_str().unwrap();
    for format in ["json", "markdown"] {
        let exported = fixture.run(&["security", "report", id, "--change", "demo", "--format", format]);
        assert!(exported.status.success());
        assert!(
            !String::from_utf8_lossy(&exported.stdout).contains("offline-only-private-sentinel")
        );
    }
    let shown = fixture.run(&["security", "review", "show", "demo", "--json", "--no-color"]);
    assert!(shown.status.success(), "{}", String::from_utf8_lossy(&shown.stderr));
    let shown: serde_json::Value = serde_json::from_slice(&shown.stdout).unwrap();
    let submission = &shown["submission"];
    assert_eq!(submission["builtinReportId"], id, "{shown}");
    let checks = submission["requiredChecks"].as_array().unwrap().iter().map(|check| serde_json::json!({"checkId":check,"status":"PASS","reason":"已追蹤完整資料流並核對本機控制項","evidencePaths":submission["targets"],"resolvesCheckIds":submission["unverifiedChecks"]})).collect::<Vec<_>>();
    let input = serde_json::json!({"formatVersion":2,"changeName":"demo","policyDigest":submission["policyDigest"],"knowledgeDigest":submission["knowledgeDigest"],"scopeDigest":submission["scopeDigest"],"builtinReportId":id,"complete":true,"targets":submission["targets"],"checks":checks,"findings":[]});
    let add = fixture.input(&["security", "review", "add-round", "demo", "--stdin", "--json", "--no-color"], &input.to_string());
    assert!(add.status.success(), "{}", String::from_utf8_lossy(&add.stderr));
    let added: serde_json::Value = serde_json::from_slice(&add.stdout).unwrap();
    assert!(added["complete"].is_boolean()); assert_eq!(added["changeName"], "demo");
    assert!(!add.stdout.contains(&27));
    let deep_id = added["reportId"].as_str().unwrap();
    for format in ["json", "markdown"] {
        let out = fixture.run(&["security", "report", deep_id, "--change", "demo", "--format", format]);
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        assert!(!String::from_utf8_lossy(&out.stdout).contains("offline-only-private-sentinel"));
    }
    for (field, value) in [("formatVersion", serde_json::json!(3)), ("scopeDigest", serde_json::json!("0".repeat(64))), ("targets", serde_json::json!(["../outside"]))] {
        let mut invalid = input.clone(); invalid[field] = value;
        let out = fixture.input(&["security", "review", "add-round", "demo", "--stdin", "--json"], &invalid.to_string());
        assert_eq!(out.status.code(), Some(1)); assert!(out.stdout.is_empty());
    }
    assert_eq!(fixture.run(&["security", "review", "add-round", "demo"]).status.code(), Some(2));
    let unknown = fixture.input(&["security", "review", "add-round", "missing", "--stdin"], &input.to_string());
    assert_eq!(unknown.status.code(), Some(1));
    let too_large = fixture.input(&["security", "review", "add-round", "demo", "--stdin"], &" ".repeat(4*1024*1024+1));
    assert_eq!(too_large.status.code(), Some(1)); assert!(String::from_utf8_lossy(&too_large.stderr).contains("size_limit"));
    let exact_limit = fixture.input(&["security", "review", "add-round", "demo", "--stdin"], &format!("{}{}", " ".repeat(4*1024*1024-input.to_string().len()), input));
    assert!(exact_limit.status.success(), "{}", String::from_utf8_lossy(&exact_limit.stderr));
    let mut duplicate = input.clone();
    duplicate["checks"].as_array_mut().unwrap().push(input["checks"][0].clone());
    assert_eq!(fixture.input(&["security","review","add-round","demo","--stdin"], &duplicate.to_string()).status.code(),Some(1));
    // Pre-existing device reports require explicit import and retain a legacy marker.
    use speclink_core::security::report::{Report,ReportRepository};
    let legacy_id = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
    let mut legacy = report.clone(); legacy["reportId"] = legacy_id.into();
    for finding in legacy["findings"].as_array_mut().unwrap() { finding["findingId"] = format!("{legacy_id}:00001").into(); }
    let device = speclink_fs::security::reports::FsSecurityReports::new(&app_data).unwrap();
    device.save(&Report::from_json(&serde_json::to_vec(&legacy).unwrap()).unwrap()).unwrap();
    let legacy_input = serde_json::json!({"legacyImportReportId":legacy_id}).to_string();
    let imported = fixture.input(&["security","review","add-round","demo","--stdin","--json"], &legacy_input);
    assert!(imported.status.success(), "{}", String::from_utf8_lossy(&imported.stderr));
    let index:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(fixture.0.join("openspec/changes/demo/checks/index.json")).unwrap()).unwrap();
    assert_eq!(index["entries"].as_array().unwrap().iter().find(|e|e["recordId"]==legacy_id).unwrap()["outcome"],"legacy");
    assert_eq!(fixture.input(&["security","review","add-round","demo","--stdin"], &legacy_input).status.code(),Some(1));
    let required=fixture.input(&["workflow-config","security","--stdin"],"required_for_acceptance: true");assert!(required.status.success());
    let risk=serde_json::json!({"reportId":id,"findingId":report["findings"][0]["findingId"],"reason":"本次已完成金鑰輪替，接受仍被列出的殘餘風險"});
    let args=["security","review","accept","demo","--stdin","--json","--no-color"];
    let no_identity=fixture.input(&args,&risk.to_string());assert_eq!(no_identity.status.code(),Some(1));assert!(String::from_utf8_lossy(&no_identity.stderr).contains("identity_required"));
    for reason in [" ".to_owned(),"字".repeat(2001)] {
        let mut invalid=risk.clone();invalid["reason"]=reason.into();let out=fixture.input(&args,&invalid.to_string());assert_eq!(out.status.code(),Some(1));assert!(out.stdout.is_empty());
    }
    let oversized=fixture.input(&args,&" ".repeat(16*1024+1));assert_eq!(oversized.status.code(),Some(1));assert!(String::from_utf8_lossy(&oversized.stderr).contains("size_limit"));
    for arguments in [vec!["init"],vec!["config","user.name","Fixture Owner"],vec!["config","user.email","owner@example.test"]] {
        let out=Command::new("git").args(arguments).current_dir(&fixture.0).output().unwrap();assert!(out.status.success());
    }
    let path=fixture.0.join(format!("openspec/changes/demo/checks/security/{id}/report.json"));let original=std::fs::read(&path).unwrap();
    let accepted=fixture.input(&args,&risk.to_string());assert!(accepted.status.success(),"{}",String::from_utf8_lossy(&accepted.stderr));
    let accepted:serde_json::Value=serde_json::from_slice(&accepted.stdout).unwrap();assert!(accepted["decisionId"].is_string());assert!(accepted["findingId"].is_string());assert_eq!(accepted["reportId"],id);assert_eq!(accepted["status"],"accepted");
    assert_eq!(std::fs::read(&path).unwrap(),original);
    let current=fixture.run(&["security","review","show","demo","--json"]);let current:serde_json::Value=serde_json::from_slice(&current.stdout).unwrap();assert_eq!(current["acceptance"]["status"],"passed");
    for field in ["decisionId","action","reportId","findingId","reason","actor","acceptedAt","policyDigest","scopeDigest"] {assert!(current["decisions"][0][field].is_string(),"{field}");}
    let human=fixture.run(&["security","review","show","demo","--no-color"]);
    assert!(human.status.success(),"{}",String::from_utf8_lossy(&human.stderr));
    let text=String::from_utf8_lossy(&human.stdout);
    for field in ["decisionId","action","reportId","findingId","reason","actor","acceptedAt","policyDigest","scopeDigest"] {
        assert!(text.contains(current["decisions"][0][field].as_str().unwrap()),"missing {field}: {text}");
    }
    for summary in ["required: true", "blocking findings: 0", "not verified: 0"] {assert!(text.contains(summary),"missing {summary}: {text}");}

    let mut old=risk.clone();old["reportId"]=legacy_id.into();assert_eq!(fixture.input(&args,&old.to_string()).status.code(),Some(1));
    let reference=fixture.input(&["workflow-config","security","--stdin"],"required_for_acceptance: false");assert!(reference.status.success());
    let required=fixture.input(&["workflow-config","security","--stdin"],"required_for_acceptance: true");assert!(required.status.success());
    let verify=fixture.input(&["verify","add-round","demo","--stdin"],"**Scope**: app.ts\n");assert!(verify.status.success(),"{}",String::from_utf8_lossy(&verify.stderr));
    let stamp=fixture.run(&["verify","stamp","demo"]);assert!(stamp.status.success(),"{}",String::from_utf8_lossy(&stamp.stderr));
    let change = fixture.0.join("openspec/changes/demo");
    assert!(!change.join("verify.md").exists());
    assert!(!change.join("review.md").exists());
    let archive = fixture.run(&["archive", "demo", "--skip-specs", "--no-validate"]);
    assert!(
        archive.status.success(),
        "{}",
        String::from_utf8_lossy(&archive.stderr)
    );
    let archived=std::fs::read_dir(fixture.0.join("openspec/changes/archive")).unwrap().flatten().find(|e|e.file_name().to_string_lossy().ends_with("-demo")).unwrap().file_name().to_string_lossy().into_owned();
    for args in [vec!["security","report",deep_id,"--change",archived.as_str()],vec!["security","review","show",archived.as_str(),"--json"]] {
        let out=fixture.run(&args); assert!(out.status.success(),"{}",String::from_utf8_lossy(&out.stderr));
    }
}

fn cwe_package(fixture: &Fixture) -> PathBuf {
    use speclink_core::security::{
        data::{ExportSource, IntegrityKind, ValidatedStandard},
        standards::{standard_source, StandardId},
    };
    let files=std::collections::BTreeMap::from([
        ("raw/cwec_v4.20.xml".into(),br#"<Weakness_Catalog xmlns="http://cwe.mitre.org/cwe-7" Version="4.20"><Weaknesses><Weakness ID="79" Name="Synthetic"><Description>Fixture</Description></Weakness></Weaknesses></Weakness_Catalog>"#.to_vec()),
        ("LICENSE".into(),b"MITRE CWE Terms of Use fixture".to_vec())]);
    let data = ValidatedStandard::new(
        standard_source(StandardId::Cwe, "4.20").unwrap(),
        files,
        IntegrityKind::Imported,
        &|| false,
    )
    .unwrap();
    let bytes = speclink_fs::security_packages::encode_single(
        ExportSource {
            metadata: data.metadata("test".into(), "now".into()).unwrap(),
            files: data.files().clone(),
        },
        &|| false,
    )
    .unwrap();
    let path = fixture.0.join("input.zip");
    std::fs::write(&path, bytes).unwrap();
    path
}
#[test]
fn security_data_export_round_trip_json_human_clap_and_existing_file() {
    let fixture = Fixture::new();
    let input = cwe_package(&fixture);
    let import = fixture.run(&[
        "security",
        "data",
        "import",
        input.to_str().unwrap(),
        "--json",
    ]);
    assert!(
        import.status.success(),
        "{}",
        String::from_utf8_lossy(&import.stderr)
    );
    let single = fixture.0.join("single.zip");
    let exported = fixture.run(&[
        "security",
        "data",
        "export",
        "cwe",
        "--version",
        "4.20",
        "--output",
        single.to_str().unwrap(),
        "--json",
        "--no-color",
    ]);
    assert!(
        exported.status.success(),
        "{}",
        String::from_utf8_lossy(&exported.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&exported.stdout).unwrap();
    assert_eq!(value.as_object().unwrap().len(), 5);
    assert_eq!(value["formatVersion"], 1);
    assert!(value["bytes"].is_u64());
    assert_eq!(value["path"], single.to_str().unwrap());
    assert_eq!(value["packages"][0]["standardId"], "cwe");
    assert_eq!(value["packages"][0]["version"], "4.20");
    assert_eq!(value["packages"][0]["sha256"].as_str().unwrap().len(), 64);
    assert_eq!(value["skipped"], serde_json::json!([]));
    let repeat = fixture.run(&[
        "security",
        "data",
        "import",
        single.to_str().unwrap(),
        "--json",
    ]);
    assert!(repeat.status.success());
    let before: serde_json::Value = serde_json::from_slice(&import.stdout).unwrap();
    let after: serde_json::Value = serde_json::from_slice(&repeat.stdout).unwrap();
    assert_eq!(
        before.as_object().unwrap().keys().collect::<Vec<_>>(),
        after.as_object().unwrap().keys().collect::<Vec<_>>()
    );
    assert_eq!(before["sha256"], after["sha256"]);
    assert_eq!(after["integrityKind"], "imported");
    let saved = std::fs::read(&single).unwrap();
    let exists = fixture.run(&[
        "security",
        "data",
        "export",
        "--all",
        "--output",
        single.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(exists.status.code(), Some(1));
    assert!(exists.stdout.is_empty());
    assert!(String::from_utf8_lossy(&exists.stderr).contains("file_exists"));
    assert_eq!(std::fs::read(&single).unwrap(), saved);
    let all = fixture.0.join("all.zip");
    let human = fixture.run(&[
        "security",
        "data",
        "export",
        "--all",
        "--output",
        all.to_str().unwrap(),
        "--no-color",
    ]);
    assert!(
        human.status.success(),
        "{}",
        String::from_utf8_lossy(&human.stderr)
    );
    let text = String::from_utf8(human.stdout).unwrap();
    assert!(text.contains("cwe 4.20"));
    assert!(text.contains(all.to_str().unwrap()));
    assert!(!text.contains('\x1b'));
    let batch = fixture.run(&[
        "security",
        "data",
        "import",
        all.to_str().unwrap(),
        "--json",
    ]);
    assert!(
        batch.status.success(),
        "{}",
        String::from_utf8_lossy(&batch.stderr)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&batch.stdout).unwrap();
    assert_eq!(receipt["formatVersion"], 2);
    assert_eq!(receipt["status"], "installed");
    assert_eq!(receipt["results"][0]["status"], "installed");
    assert!(receipt["results"][0]["errorCode"].is_null());
    let human_batch = fixture.run(&[
        "security",
        "data",
        "import",
        all.to_str().unwrap(),
        "--no-color",
    ]);
    assert!(human_batch.status.success());
    let text = String::from_utf8(human_batch.stdout).unwrap();
    assert!(text.contains("cwe 4.20: installed"));
    assert!(text.contains(receipt["results"][0]["sha256"].as_str().unwrap()));
    assert!(!text.contains('\x1b'));
    for args in [
        vec![
            "security",
            "data",
            "export",
            "cwe",
            "--all",
            "--version",
            "4.20",
            "--output",
            "x",
        ],
        vec![
            "security",
            "data",
            "export",
            "--all",
            "--version",
            "4.20",
            "--output",
            "x",
        ],
        vec!["security", "data", "export", "cwe", "--output", "x"],
        vec!["security", "data", "export", "--output", "x"],
    ] {
        let out = fixture.run(&args);
        assert_eq!(out.status.code(), Some(2), "args: {args:?}");
        assert!(out.stdout.is_empty());
    }
}
