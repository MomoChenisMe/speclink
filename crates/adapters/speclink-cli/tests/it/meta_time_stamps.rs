//! Integration tests: change metadata time fields are written as RFC 3339 stamps,
//! but every output keeps emitting the date part (spec change-lifecycle「change
//! metadata 時間欄位寫入時戳、對外輸出日期」; process-cost-metrics design D2).

use std::path::PathBuf;
use std::process::{Command, Output};

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// Project skeleton with one change named `name` carrying `meta`.
    fn new(tag: &str, name: &str, meta: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-meta-time-stamps-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("speclink").join("specs")).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), "tools:\n  - claude\n").unwrap();
        let change = dir.join("speclink").join("changes").join(name);
        std::fs::create_dir_all(&change).unwrap();
        std::fs::write(change.join(".meta.yaml"), meta).unwrap();
        std::fs::write(change.join("proposal.md"), "## Why\n\nDemo.\n").unwrap();
        std::fs::write(change.join("tasks.md"), "- [ ] 1.1 Do the thing\n").unwrap();
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

    fn json(&self, args: &[&str]) -> serde_json::Value {
        let out = self.run(args);
        assert!(out.status.success(), "{args:?} failed: {}", String::from_utf8_lossy(&out.stderr));
        serde_json::from_slice(&out.stdout).expect("stdout is JSON")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn show_outputs_only_the_date_part_of_a_stamped_created() {
    // spec scenario「對外輸出只給日期部分」。
    let p = TempProject::new(
        "show",
        "demo",
        "schema: spec-driven\ncreated: 2026-10-21T14:05:00+08:00\n",
    );
    let out = p.run(&["show", "demo"]);
    assert!(out.status.success(), "show failed: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.lines().any(|l| l == "Created: 2026-10-21"),
        "human output carries the date part only: {stdout}"
    );
    let json = p.json(&["show", "demo", "--json"]);
    assert_eq!(json["created"], "2026-10-21");
}

#[test]
fn plain_date_created_keeps_the_existing_outputs() {
    // spec scenario「純日期記錄的輸出不變」：show --json 與 list --json 與本變更之前
    // 的輸出相同，沒有警告。
    let p = TempProject::new("plain", "demo", "schema: spec-driven\ncreated: 2026-07-29\n");
    let show = p.run(&["show", "demo", "--json"]);
    assert!(show.status.success() && show.stderr.is_empty(), "{}", String::from_utf8_lossy(&show.stderr));
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&show.stdout).unwrap(),
        serde_json::json!({
            "created": "2026-07-29",
            "deltaSpecs": [],
            "design": null,
            "fromDiscussions": [],
            "name": "demo",
            "proposal": "## Why\n\nDemo.\n",
            "restaleFrom": [],
            "schema": "spec-driven",
            "tasks": "- [ ] 1.1 Do the thing\n"
        })
    );
    let list = p.run(&["list", "--json"]);
    assert!(list.status.success() && list.stderr.is_empty(), "{}", String::from_utf8_lossy(&list.stderr));
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&list.stdout).unwrap(),
        serde_json::json!({ "changes": [{
            "completedTasks": 0,
            "name": "demo",
            "status": "in-progress",
            "summary": "Demo.",
            "totalTasks": 1
        }] })
    );
}

#[test]
fn new_change_writes_created_as_an_rfc3339_stamp() {
    // spec scenario「新變更寫入時戳」：經 CLI 建立的 change，檔內 created 是 RFC 3339。
    let p = TempProject::new("new", "existing", "schema: spec-driven\ncreated: 2026-07-29\n");
    let out = p.run(&["new", "change", "fresh"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let meta = std::fs::read_to_string(
        p.dir.join("speclink").join("changes").join("fresh").join(".meta.yaml"),
    )
    .unwrap();
    let created = meta.lines().find_map(|l| l.strip_prefix("created: ")).expect("created line");
    assert!(
        created.len() == 25 && chrono::DateTime::parse_from_rfc3339(created).is_ok(),
        "created is an offset stamp at second precision: {meta}"
    );
}

#[test]
fn staleness_of_a_stamped_created_counts_from_its_date_part() {
    // 依建立日期計算的天數取日期部分：時戳不會讓過期判定解析失敗。
    let p = TempProject::new(
        "stale",
        "demo",
        "schema: spec-driven\ncreated: 2026-09-01T23:30:00-05:00\n",
    );
    // 指令前後各取一次「今天」：跑在午夜兩側時兩個天數都算對。
    let created = chrono::NaiveDate::from_ymd_opt(2026, 9, 1).unwrap();
    let days_old = || (chrono::Local::now().date_naive() - created).num_days();
    let before = days_old();
    let json = p.json(&["instructions", "apply", "--change", "demo", "--json"]);
    let after = days_old();
    let actual = json["preflight"]["staleness"]["daysOld"].as_i64();
    assert!(actual == Some(before) || actual == Some(after), "preflight: {json}");
}

#[test]
fn unrecognised_created_is_output_verbatim_without_staleness() {
    // spec scenario「無法辨識的時間值維持現行行為」。
    let p = TempProject::new("odd", "odd", "schema: spec-driven\ncreated: someday\n");
    let out = p.run(&["show", "odd"]);
    assert!(out.status.success(), "show failed: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.lines().any(|l| l == "Created: someday"), "verbatim value: {stdout}");
    let json = p.json(&["instructions", "apply", "--change", "odd", "--json"]);
    assert!(
        json["preflight"].get("staleness").is_none(),
        "no staleness for an unrecognised created: {json}"
    );
}
