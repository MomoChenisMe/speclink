//! Integration tests: `speclink list --json` names each change's delta
//! capabilities in `deltaCapabilities`, omits the key for a change without
//! delta specs, and leaves the human listing untouched
//! (spec「list --json 曝變更的 delta capability」).

use std::path::PathBuf;
use std::process::{Command, Output};

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// `with-deltas` carries two delta specs, written in reverse order;
    /// `no-deltas` carries none.
    fn new(tag: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-list-delta-caps-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), "tools:\n  - claude\n").unwrap();
        let changes = dir.join("openspec").join("changes");
        for name in ["with-deltas", "no-deltas"] {
            let change = changes.join(name);
            std::fs::create_dir_all(&change).unwrap();
            std::fs::write(change.join(".openspec.yaml"), "schema: spec-driven\ncreated: 2026-09-16\n")
                .unwrap();
            std::fs::write(change.join("proposal.md"), "## Why\n\nDemo.\n").unwrap();
            std::fs::write(change.join("tasks.md"), "- [ ] 1.1 Do the thing\n").unwrap();
        }
        for cap in ["server-verb-api", "client-protocol"] {
            let spec = changes.join("with-deltas").join("specs").join(cap);
            std::fs::create_dir_all(&spec).unwrap();
            std::fs::write(spec.join("spec.md"), "## ADDED Requirements\n").unwrap();
        }
        TempProject { dir }
    }

    fn run(&self, args: &[&str]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(args).current_dir(&self.dir);
        for key in [
            "SPECLINK_LOCALE",
            "SPECLINK_SPEC_LOCALE",
            "SPECLINK_TDD",
            "SPECLINK_AUDIT",
            "SPECLINK_STORE_URL",
        ] {
            cmd.env_remove(key);
        }
        cmd.output().expect("run speclink binary")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn list_json_names_delta_capabilities_and_omits_the_key_without_deltas() {
    let p = TempProject::new("json");
    let out = p.run(&["list", "--sort", "name", "--json"]);
    assert!(out.status.success(), "list failed: {}", String::from_utf8_lossy(&out.stderr));
    let payload: serde_json::Value = serde_json::from_slice(&out.stdout).expect("stdout is JSON");
    let changes = payload["changes"].as_array().expect("changes array");

    let with = changes.iter().find(|c| c["name"] == "with-deltas").expect("with-deltas listed");
    assert_eq!(
        with["deltaCapabilities"],
        serde_json::json!(["client-protocol", "server-verb-api"]),
        "capability names, ascending: {with}"
    );

    let without = changes.iter().find(|c| c["name"] == "no-deltas").expect("no-deltas listed");
    let keys: Vec<&str> = without.as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        vec!["completedTasks", "name", "status", "summary", "totalTasks"],
        "no delta specs → field shape unchanged (no deltaCapabilities key)"
    );
}

#[test]
fn human_list_does_not_show_delta_capabilities() {
    let p = TempProject::new("human");
    let out = p.run(&["list", "--sort", "name", "--no-color"]);
    assert!(out.status.success(), "list failed: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("deltaCapabilities") && !stdout.contains("client-protocol"),
        "human listing stays free of delta capabilities: {stdout}"
    );
}
