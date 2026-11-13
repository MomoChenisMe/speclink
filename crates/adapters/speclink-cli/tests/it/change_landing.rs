//! 分支、落地與蓋章 commit 的 CLI 契約（change station-status-and-landing；specs
//! change-landing 三條需求、review-station「蓋章記錄程式碼的 commit」、verify-station
//! 「驗證蓋章記錄程式碼的 commit」、client-protocol「CLI 的 list --json 不帶新欄位」）。
//!
//! CLI 在這裡的工作是收集 checkout 的事實（HEAD、目前的分支）交給引擎或 Server：
//! 兩種模式都收集，引擎與 Server 都不讀 git。

use crate::remote_plan::{mock_server, stderr_of, stdout_of, TempProject as RemoteProject};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const META: &str = "schema: spec-driven\ncreated: 2026-11-01\n";
const SHA: &str = "0123456789abcdef0123456789abcdef01234567";
const TASKS_DONE: &str = "- [x] 1.1 first\n";
const TICKET_ROUND: &str = "## Round 1\n\n**Scope**: src/lib.rs\n";

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git").args(args).current_dir(dir).output().expect("run git");
    assert!(out.status.success(), "git {args:?} failed: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Make `dir` a git checkout on branch `main` with one commit; returns HEAD.
fn git_checkout(dir: &Path) -> String {
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.name", "Sandbox Tester"]);
    git(dir, &["config", "user.email", "sandbox@example.com"]);
    git(dir, &["config", "commit.gpgsign", "false"]);
    std::fs::write(dir.join("README.md"), "demo\n").unwrap();
    git(dir, &["add", "README.md"]);
    git(dir, &["commit", "-q", "-m", "init"]);
    git(dir, &["rev-parse", "HEAD"])
}

/// A local project with change `demo`. It is a git checkout only after
/// [`Project::git`].
struct Project {
    dir: PathBuf,
}

impl Project {
    fn new(tag: &str) -> Project {
        let dir = std::env::temp_dir().join(format!("speclink-cli-landing-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let change = dir.join("speclink").join("changes").join("demo");
        std::fs::create_dir_all(&change).unwrap();
        std::fs::write(change.join(".meta.yaml"), META).unwrap();
        Project { dir }
    }

    /// A project that is a git checkout; returns it with its HEAD.
    fn git(tag: &str) -> (Project, String) {
        let p = Project::new(tag);
        let head = git_checkout(&p.dir);
        (p, head)
    }

    fn change_file(&self, name: &str) -> PathBuf {
        self.dir.join("speclink").join("changes").join("demo").join(name)
    }

    fn meta(&self) -> String {
        std::fs::read_to_string(self.change_file(".meta.yaml")).unwrap()
    }

    /// The value of the top-level `key` in the change's metadata, unquoted.
    fn meta_value(&self, key: &str) -> Option<String> {
        let prefix = format!("{key}: ");
        self.meta().lines().find_map(|l| l.strip_prefix(&prefix)).map(|v| v.trim_matches('"').to_string())
    }

    /// Everything a stamp of `station` needs: done tasks, the scope file and a
    /// one-round ticket.
    fn ready_to_stamp(&self, station: &str) {
        std::fs::write(self.change_file("tasks.md"), TASKS_DONE).unwrap();
        std::fs::create_dir_all(self.dir.join("src")).unwrap();
        std::fs::write(self.dir.join("src").join("lib.rs"), "fn demo() {}\n").unwrap();
        let title = if station == "review" { "Review" } else { "Verify" };
        std::fs::write(self.change_file(&format!("{station}.md")), format!("# {title} — demo\n\n{TICKET_ROUND}"))
            .unwrap();
    }

    fn run(&self, args: &[&str]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(args).current_dir(&self.dir);
        for key in ["SPECLINK_STORE_URL", "SPECLINK_TOKEN", "SPECLINK_WORKTREE", "NO_COLOR", "CLICOLOR", "CLICOLOR_FORCE"] {
            cmd.env_remove(key);
        }
        cmd.output().expect("run speclink binary")
    }

    fn ok(&self, args: &[&str]) -> Output {
        let out = self.run(args);
        assert!(out.status.success(), "{args:?} failed: {}", stderr_of(&out));
        out
    }

    fn json(&self, args: &[&str]) -> Value {
        serde_json::from_slice(&self.ok(args).stdout).expect("JSON stdout")
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn parsed(out: &Output) -> Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| panic!("JSON stdout ({e}): {}", stdout_of(out)))
}

// --- change landed 動詞回報落地的 commit ---

#[test]
fn landed_without_a_commit_reports_the_checkouts_head() {
    // Scenario「以目前的 HEAD 回報」。
    let (p, head) = Project::git("head");
    let out = p.ok(&["change", "landed", "demo", "--no-color"]);
    assert_eq!(stdout_of(&out), format!("✓ demo landed at {}\n", &head[..12]));
    assert!(stderr_of(&out).is_empty());
    assert_eq!(p.meta_value("landed_commit").as_deref(), Some(head.as_str()), "the whole commit");
    assert_eq!(p.meta_value("landed_by").as_deref(), Some("Sandbox Tester <sandbox@example.com>"));
    let at = p.meta_value("landed_at").expect("a report time");
    assert!(chrono::DateTime::parse_from_rfc3339(&at).is_ok(), "RFC 3339: {at}");
    assert!(p.meta_value("branch").is_none(), "no --branch: the checkout's branch is not taken");
    assert!(p.meta().starts_with(META), "the fields that were there are kept");
}

#[test]
fn landed_with_a_commit_and_a_branch_prints_them_as_json() {
    // Scenario「指定 commit 與分支」。
    let p = Project::new("flags");
    let out = p.ok(&["change", "landed", "demo", "--commit", SHA, "--branch", "speclink/demo", "--json"]);
    assert_eq!(parsed(&out), json!({ "change": "demo", "commit": SHA, "branch": "speclink/demo" }));
    assert_eq!(p.meta_value("branch").as_deref(), Some("speclink/demo"));
    assert_eq!(p.meta_value("landed_commit").as_deref(), Some(SHA));
}

#[test]
fn a_repeated_report_overwrites_the_earlier_one() {
    // Scenario「重複回報覆寫」。
    let p = Project::new("repeat");
    p.ok(&["change", "landed", "demo", "--commit", SHA]);
    p.ok(&["change", "landed", "demo", "--commit", "89abcdef0123"]);
    let meta = p.meta();
    assert_eq!(meta.matches("landed_commit: ").count(), 1, "{meta}");
    assert_eq!(meta.matches("landed_at: ").count(), 1, "{meta}");
    assert_eq!(p.meta_value("landed_commit").as_deref(), Some("89abcdef0123"));
}

#[test]
fn landed_outside_a_git_checkout_without_a_commit_is_refused() {
    // Scenario「沒有 commit 可用」。
    let p = Project::new("no-git");
    let out = p.run(&["change", "landed", "demo"]);
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("--commit"), "stderr says how to give one: {}", stderr_of(&out));
    assert!(stdout_of(&out).is_empty());
    assert_eq!(p.meta(), META, "the metadata is byte-identical");
}

#[test]
fn a_malformed_commit_and_an_unknown_change_are_refused() {
    // Scenarios「commit 的形狀不合」「變更不存在」。
    let p = Project::new("refused");
    let out = p.run(&["change", "landed", "demo", "--commit", "not-a-sha"]);
    assert!(!out.status.success());
    assert_eq!(p.meta(), META, "the metadata is byte-identical");

    let out = p.run(&["change", "landed", "ghost", "--commit", "0123456"]);
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("ghost"), "stderr names the change: {}", stderr_of(&out));
}

#[test]
fn a_proposed_change_can_report_and_stays_proposed() {
    // Scenario「提案中的變更也能回報」。
    let p = Project::new("proposed");
    std::fs::write(p.change_file("tasks.md"), "- [ ] 1.1 open\n").unwrap();
    let stage = |p: &Project| p.json(&["plan", "--json"])["changes"][0]["stage"].clone();
    assert_eq!(stage(&p), "proposed");
    p.ok(&["change", "landed", "demo", "--commit", "0123456"]);
    assert_eq!(stage(&p), "proposed");
    assert!(p.meta_value("started_at").is_none());
}

#[test]
fn remote_landed_posts_the_report_and_prints_the_fs_json() {
    // Scenario「remote 模式與本機同形」。
    let fs = Project::new("parity-fs");
    let fs_out = fs.ok(&["change", "landed", "demo", "--commit", "0123456789ab", "--json"]);

    let mock = mock_server(vec![(
        "POST",
        "/changes/demo/landed".into(),
        200,
        r#"{"change":"demo","commit":"0123456789ab"}"#.into(),
    )]);
    let remote = RemoteProject::remote("landing-parity-remote", &mock.base);
    let out = remote.run(&["change", "landed", "demo", "--commit", "0123456789ab", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), stdout_of(&fs_out), "bit-for-bit the fs JSON");
    assert_eq!(mock.verb_calls(), ["POST /changes/demo/landed"], "one request");
    assert_eq!(mock.body_of("POST", "/changes/demo/landed"), r#"{"commit":"0123456789ab"}"#);
    assert!(!remote.dir.join("speclink").exists(), "the local store is never touched");
}

#[test]
fn remote_landed_takes_the_checkouts_head_and_sends_the_branch_flag() {
    let mock = mock_server(vec![("POST", "/changes/demo/landed".into(), 200, format!(r#"{{"change":"demo","commit":"{SHA}"}}"#))]);
    let remote = RemoteProject::remote("landing-head-remote", &mock.base);
    let head = git_checkout(&remote.dir);
    let out = remote.run(&["change", "landed", "demo", "--branch", "speclink/demo", "--no-color"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "✓ demo landed at 0123456789ab\n", "the line shows what the server recorded");
    assert_eq!(
        serde_json::from_str::<Value>(&mock.body_of("POST", "/changes/demo/landed")).unwrap(),
        json!({ "commit": head, "branch": "speclink/demo" })
    );
}

#[test]
fn remote_landed_without_any_commit_sends_nothing() {
    let mock = mock_server(vec![]);
    let remote = RemoteProject::remote("landing-none-remote", &mock.base);
    let out = remote.run(&["change", "landed", "demo"]);
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("--commit"), "stderr: {}", stderr_of(&out));
    assert!(mock.verb_calls().is_empty(), "no request without a commit: {:?}", mock.verb_calls());
}

#[test]
fn a_server_without_the_landing_route_makes_the_verb_fail() {
    // 相容性：舊版 Server 沒有這條路由，回 404；CLI 非 0 結束。
    let mock = mock_server(vec![("POST", "/changes/demo/landed".into(), 404, String::new())]);
    let remote = RemoteProject::remote("landing-old-server", &mock.base);
    let out = remote.run(&["change", "landed", "demo", "--commit", "0123456"]);
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("resource not found"), "stderr: {}", stderr_of(&out));
    assert!(stdout_of(&out).is_empty());
}

// --- 開工時記下分支 ---

#[test]
fn in_progress_add_records_the_checkouts_branch() {
    // Scenarios「在功能分支上開工」「已開工時不改分支」「退回提案中移除分支」。
    let (p, _) = Project::git("branch");
    git(&p.dir, &["checkout", "-q", "-b", "feature/x"]);
    let out = p.ok(&["in-progress", "add", "demo"]);
    assert!(stdout_of(&out).is_empty() && stderr_of(&out).is_empty(), "the verb stays silent");
    assert!(p.meta().contains("\nbranch: feature/x\n"), "{}", p.meta());
    assert!(p.meta_value("started_at").is_some());

    git(&p.dir, &["checkout", "-q", "-b", "other"]);
    p.ok(&["in-progress", "add", "demo"]);
    assert_eq!(p.meta_value("branch").as_deref(), Some("feature/x"), "the first branch stays");

    p.ok(&["in-progress", "remove", "demo"]);
    assert_eq!(p.meta(), META, "the marker and the branch are gone");
}

#[test]
fn in_progress_add_on_a_detached_head_or_outside_git_records_no_branch() {
    // Scenario「detached HEAD 不記分支」。
    let (detached, head) = Project::git("detached");
    git(&detached.dir, &["checkout", "-q", "--detach", &head]);
    detached.ok(&["in-progress", "add", "demo"]);
    assert!(detached.meta_value("started_at").is_some());
    assert!(detached.meta_value("branch").is_none(), "{}", detached.meta());

    let plain = Project::new("no-git-branch");
    plain.ok(&["in-progress", "add", "demo"]);
    assert!(plain.meta_value("started_at").is_some() && plain.meta_value("branch").is_none());
}

#[test]
fn a_project_inside_another_repo_records_no_branch_and_no_commit() {
    // 專案資料夾自己不是 git repo、但位於別的 repo 底下：外層 repo 的分支與 HEAD 不是
    // 這個專案的事實，開工不記分支，蓋章不記 commit。
    let outer = std::env::temp_dir().join(format!("speclink-cli-landing-outer-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&outer);
    std::fs::create_dir_all(&outer).unwrap();
    git_checkout(&outer);
    git(&outer, &["checkout", "-q", "-b", "outer/branch"]);
    let inner = Project { dir: outer.join("nested") };
    let change = inner.dir.join("speclink").join("changes").join("demo");
    std::fs::create_dir_all(&change).unwrap();
    std::fs::write(change.join(".meta.yaml"), META).unwrap();

    inner.ok(&["in-progress", "add", "demo"]);
    assert!(inner.meta_value("started_at").is_some());
    assert!(inner.meta_value("branch").is_none(), "{}", inner.meta());
    inner.ready_to_stamp("review");
    inner.ok(&["review", "stamp", "demo"]);
    assert!(inner.meta_value("reviewed_commit").is_none(), "{}", inner.meta());
    drop(inner);
    let _ = std::fs::remove_dir_all(&outer);
}

#[test]
fn a_branch_with_a_slash_and_cjk_reads_back_through_show() {
    // Scenario「含斜線與中文的分支名稱」。
    let (p, _) = Project::git("cjk");
    git(&p.dir, &["checkout", "-q", "-b", "功能/登入"]);
    p.ok(&["in-progress", "add", "demo"]);
    assert_eq!(p.json(&["show", "demo", "--json"])["branch"], "功能/登入");
}

#[test]
fn remote_in_progress_add_sends_the_checkouts_branch() {
    let mock = mock_server(vec![("POST", "/changes/demo/in-progress".into(), 200, "{}".into())]);
    let remote = RemoteProject::remote("branch-remote", &mock.base);
    git_checkout(&remote.dir);
    git(&remote.dir, &["checkout", "-q", "-b", "feature/x"]);
    let out = remote.run(&["in-progress", "add", "demo"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(mock.body_of("POST", "/changes/demo/in-progress"), r#"{"branch":"feature/x"}"#);

    // Not a git checkout: the body is the bare object, as before.
    let mock = mock_server(vec![("POST", "/changes/demo/in-progress".into(), 200, "{}".into())]);
    let bare = RemoteProject::remote("branch-remote-bare", &mock.base);
    assert!(bare.run(&["in-progress", "add", "demo"]).status.success());
    assert_eq!(mock.body_of("POST", "/changes/demo/in-progress"), "{}");
}

// --- 蓋章記錄程式碼的 commit ---

#[test]
fn a_stamp_records_the_checkouts_head_and_prints_what_it_printed_before() {
    // Scenarios「蓋章記下 HEAD」「不是 git checkout 時照常蓋章」，兩個關卡。
    for (station, key) in [("review", "reviewed_commit"), ("verify", "verified_commit")] {
        let (p, head) = Project::git(&format!("stamp-{station}"));
        p.ready_to_stamp(station);
        let stamped = p.ok(&[station, "stamp", "demo", "--no-color"]);
        assert_eq!(p.meta_value(key).as_deref(), Some(head.as_str()), "{station}");
        assert!(!p.change_file(&format!("{station}.md")).exists(), "{station}: the ticket is gone");

        let plain = Project::new(&format!("stamp-{station}-no-git"));
        plain.ready_to_stamp(station);
        let unstamped_commit = plain.ok(&[station, "stamp", "demo", "--no-color"]);
        assert!(plain.meta_value(key).is_none(), "{station}: {}", plain.meta());
        assert!(plain.meta().contains("_tasks_total: 1"), "{station}: the stamp landed all the same");
        assert_eq!(stdout_of(&stamped), stdout_of(&unstamped_commit), "{station}: the same line with or without a commit");
        assert!(!stdout_of(&stamped).contains(&head[..7]), "{station}: the line does not show the commit");
    }
}

#[test]
fn remote_stamp_sends_the_checkouts_head() {
    for station in ["review", "verify"] {
        let ticket = r#"{"change":"demo","rounds":[{"index":1,"phase":null,"patchHash":null,"scope":["src/lib.rs"],"findings":[]}],"lastRound":{"index":1,"phase":null,"patchHash":null,"scope":["src/lib.rs"],"findings":[]}}"#;
        let mock = mock_server(vec![
            ("GET", format!("/changes/demo/{station}"), 200, ticket.into()),
            ("POST", format!("/changes/demo/{station}/stamp"), 200, r#"{"change":"demo"}"#.into()),
        ]);
        let remote = RemoteProject::remote(&format!("stamp-remote-{station}"), &mock.base);
        let head = git_checkout(&remote.dir);
        std::fs::create_dir_all(remote.dir.join("src")).unwrap();
        std::fs::write(remote.dir.join("src").join("lib.rs"), "fn demo() {}\n").unwrap();
        let out = remote.run(&[station, "stamp", "demo"]);
        assert!(out.status.success(), "{station} stderr: {}", stderr_of(&out));
        let body: Value = serde_json::from_str(&mock.body_of("POST", &format!("/changes/demo/{station}/stamp"))).unwrap();
        assert_eq!(body["headCommit"], json!(head), "{station}: {body}");
        assert_eq!(body["scope"][0]["path"], "src/lib.rs", "{station}");
    }
}

// --- show --json 的四個欄位，以及不變的 list --json ---

#[test]
fn show_json_carries_the_stations_the_branch_and_the_landing() {
    // Scenarios「show 帶出落地記錄」「show --json 帶出關卡狀態」「沒有記錄時不輸出」。
    let (p, head) = Project::git("show");
    git(&p.dir, &["checkout", "-q", "-b", "feature/x"]);
    let bare = p.json(&["show", "demo", "--json"]);
    for key in ["review", "verify", "branch", "landed"] {
        assert!(bare.get(key).is_none(), "nothing recorded, no {key} key: {bare}");
    }
    let human_before = stdout_of(&p.ok(&["show", "demo", "--no-color"]));

    p.ok(&["in-progress", "add", "demo"]);
    p.ready_to_stamp("review");
    p.ok(&["review", "stamp", "demo", "--agent", "claude"]);
    p.ready_to_stamp("verify");
    p.ok(&["change", "landed", "demo"]);

    let shown = p.json(&["show", "demo", "--json"]);
    let review = &shown["review"];
    assert_eq!(review["state"], "stamped");
    assert_eq!(review["commit"], json!(head));
    assert_eq!((&review["rounds"], &review["tasksTotal"]), (&json!(1), &json!(1)));
    assert_eq!(review["stampedBy"], "Sandbox Tester <sandbox@example.com>");
    assert_eq!(review["stampedWith"], "claude");
    assert!(review["stampedAt"].is_string() && review.get("scope").is_none(), "{review}");
    assert_eq!(shown["verify"], json!({ "state": "inProgress" }));
    assert_eq!(shown["branch"], "feature/x");
    assert_eq!(shown["landed"]["commit"], json!(head));
    assert_eq!(shown["landed"]["by"], "Sandbox Tester <sandbox@example.com>");
    assert!(shown["landed"]["at"].as_str().is_some_and(|at| chrono::DateTime::parse_from_rfc3339(at).is_ok()));

    let human_after = stdout_of(&p.ok(&["show", "demo", "--no-color"]));
    assert_eq!(human_after, human_before, "the human output shows none of it");
}

#[test]
fn show_json_reports_a_verify_stamp_with_its_commit() {
    // verify-station Scenario「show --json 帶出關卡狀態」。
    let (p, head) = Project::git("show-verify");
    p.ready_to_stamp("verify");
    p.ok(&["verify", "stamp", "demo"]);
    let verify = p.json(&["show", "demo", "--json"])["verify"].clone();
    assert_eq!(verify["state"], "stamped");
    assert_eq!(verify["commit"], json!(head));
    assert_eq!((&verify["rounds"], &verify["tasksTotal"]), (&json!(1), &json!(1)));
    assert!(verify["stampedAt"].is_string() && verify.get("scope").is_none(), "{verify}");
}

#[test]
fn remote_show_json_builds_the_same_fields_from_the_single_change_response() {
    let status = r#"{"changeName":"demo","schemaName":"spec-driven","isComplete":false,"applyRequires":[],"artifacts":[],
        "created":"2026-11-01","branch":"feature/x",
        "review":{"state":"stamped","stampedAt":"2026-11-02","stampedBy":"Rev","rounds":3,"tasksTotal":12,
                  "commit":"0123456789ab","scope":[{"path":"src/a.rs","hash":"aa"}]},
        "verify":{"state":"inProgress"},
        "landedCommit":"89abcdef0123","landedAt":"2026-11-04T09:00:00+08:00","landedBy":"Ann"}"#;
    let mock = mock_server(vec![
        ("GET", "/specs".into(), 200, r#"{"specs":[]}"#.into()),
        ("GET", "/changes/demo".into(), 200, status.into()),
        ("GET", "/changes".into(), 200, r#"{"changes":[{"name":"demo"}]}"#.into()),
    ]);
    let remote = RemoteProject::remote("show-remote", &mock.base);
    let out = remote.run(&["show", "demo", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let shown = parsed(&out);
    assert_eq!(
        shown["review"],
        json!({
            "state": "stamped",
            "stampedAt": "2026-11-02",
            "stampedBy": "Rev",
            "rounds": 3,
            "tasksTotal": 12,
            "commit": "0123456789ab",
        }),
        "the fs shape: no scope"
    );
    assert_eq!(shown["verify"], json!({ "state": "inProgress" }));
    assert_eq!(shown["branch"], "feature/x");
    assert_eq!(shown["landed"], json!({ "commit": "89abcdef0123", "at": "2026-11-04T09:00:00+08:00", "by": "Ann" }));
}

#[test]
fn remote_show_json_from_a_server_without_the_fields_has_none_of_them() {
    // 新版用戶端連到舊版 Server：沒有新欄位，其餘照舊。
    let status = r#"{"changeName":"demo","schemaName":"spec-driven","isComplete":false,"applyRequires":[],"artifacts":[]}"#;
    let mock = mock_server(vec![
        ("GET", "/specs".into(), 200, r#"{"specs":[]}"#.into()),
        ("GET", "/changes/demo".into(), 200, status.into()),
        ("GET", "/changes".into(), 200, r#"{"changes":[{"name":"demo"}]}"#.into()),
    ]);
    let remote = RemoteProject::remote("show-remote-old", &mock.base);
    let shown = parsed(&remote.run(&["show", "demo", "--json"]));
    for key in ["review", "verify", "branch", "landed"] {
        assert!(shown.get(key).is_none(), "{key}: {shown}");
    }
    assert_eq!(shown["name"], "demo");
}

/// The keys of the first item of `list --json`, sorted.
fn list_keys(listed: &Value) -> Vec<String> {
    let mut keys: Vec<String> = listed["changes"][0].as_object().expect("a change item").keys().cloned().collect();
    keys.sort();
    keys
}

#[test]
fn list_json_keeps_its_shape_through_a_stamp_and_a_landing() {
    // Scenarios「list 不變」「list --json 不變」：欄位集合相同；回報落地前後除了
    // `modifiedAt`（文件確實被改寫了）以外逐位元相同。
    let (p, _) = Project::git("list");
    p.ready_to_stamp("review");
    p.ok(&["in-progress", "add", "demo"]);
    let before = p.json(&["list", "--json"]);

    p.ok(&["review", "stamp", "demo"]);
    let stamped = p.json(&["list", "--json"]);
    assert_eq!(list_keys(&stamped), list_keys(&before), "a stamp adds no key");

    p.ok(&["change", "landed", "demo", "--branch", "speclink/demo"]);
    let mut landed = p.json(&["list", "--json"]);
    assert_eq!(list_keys(&landed), list_keys(&before), "a landing adds no key");
    let mut stamped = stamped;
    for listed in [&mut stamped, &mut landed] {
        listed["changes"][0].as_object_mut().unwrap().remove("modifiedAt");
    }
    assert_eq!(landed, stamped);
}

#[test]
fn remote_list_json_drops_the_station_branch_and_landing_fields() {
    // Scenario「CLI 的 list --json 不帶新欄位」。
    let list = r#"{"changes":[{"name":"demo","summary":"s","status":"in-progress","completedTasks":1,"totalTasks":2,
        "created":"2026-11-01","review":{"state":"stamped","stampedAt":"2026-11-02","rounds":3,"tasksTotal":12},
        "verify":{"state":"inProgress"},"branch":"feature/x","landedCommit":"0123456789ab"}]}"#;
    let mock = mock_server(vec![("GET", "/changes".into(), 200, list.into())]);
    let remote = RemoteProject::remote("list-remote", &mock.base);
    let out = remote.run(&["list", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let listed = parsed(&out);
    for key in ["review", "verify", "branch", "landedCommit"] {
        assert!(listed["changes"][0].get(key).is_none(), "{key}: {listed}");
    }

    let fs = Project::new("list-shape-fs");
    std::fs::write(fs.change_file("tasks.md"), "- [x] 1.1 a\n- [ ] 1.2 b\n").unwrap();
    std::fs::write(fs.change_file("proposal.md"), "## Why\n\nDemo.\n").unwrap();
    let fs_keys = list_keys(&fs.json(&["list", "--json"]));
    let remote_keys = list_keys(&listed);
    assert!(remote_keys.iter().all(|key| fs_keys.contains(key)), "remote {remote_keys:?} within fs {fs_keys:?}");
}
