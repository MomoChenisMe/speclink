//! Free fields from the CLI (change host-free-fields; specs free-fields
//! 「自由欄位的寫入動詞」「自由欄位的讀取輸出」, client-protocol「舊 Server 不保存
//! 欄位時警告」).
//!
//! Each scenario runs twice on one set of documents: as files of a local
//! project, and through a real server — the host kit's HTTP door on the
//! in-memory store port. The verb's stdout and exit code must be the same in
//! both. The remote project holds only its connection file.

use crate::remote_plan::mock_server;
use crate::remote_read_verbs::{json_of, pair, stderr_of, stdout_of, Pair, TempProject};
use serde_json::{json, Value};

const META: &str = "schema: spec-driven\ncreated: 2026-11-01\n";
const RECORD: &str = "---\ntopic: Board search\nslug: board-search\nstatus: open\ncreated: 2026-11-01\n---\n\n\
                      # Discussion: Board search\n\n## Context\n\n## Rounds\n\n## Conclusion\n";

/// A project with change `demo` (a proposal and one task), change `other`,
/// and discussion `board-search`; none of them has a free field.
const DOCS: &[(&str, &str)] = &[
    ("changes/demo/.meta.yaml", META),
    ("changes/demo/proposal.md", "## Why\n\nBecause.\n"),
    ("changes/demo/tasks.md", "- [ ] 1.1 first\n"),
    ("changes/other/.meta.yaml", META),
    ("discussions/board-search.md", RECORD),
];

// --- what these scenarios ask of the two projects (remote_read_verbs has the projects) ---

/// The file of store key `key` in a local project, or `None` when it has none.
fn read(p: &TempProject, key: &str) -> Option<String> {
    std::fs::read_to_string(p.spec_file(key)).ok()
}

/// A run in one project that must succeed.
fn ok_in(p: &TempProject, args: &[&str]) -> std::process::Output {
    let out = p.run(args);
    assert!(out.status.success(), "{args:?} failed: {}", stderr_of(&out));
    out
}

fn json_in(p: &TempProject, args: &[&str]) -> Value {
    json_of(&ok_in(p, args))
}

fn projects(b: &Pair) -> [(&'static str, &TempProject); 2] {
    [("fs", &b.fs), ("remote", &b.remote)]
}

/// Run `args` in both projects: the same stdout, stderr and exit code, and
/// a success. Returns the stdout.
fn ok(b: &Pair, args: &[&str]) -> String {
    stdout_of(&b.same_ok(args))
}

/// Run `args` in both projects; both must fail and print nothing to stdout.
/// Returns each stderr (the two modes word some refusals differently).
fn fails(b: &Pair, args: &[&str]) -> [String; 2] {
    projects(b).map(|(mode, p)| {
        let out = p.run(args);
        assert!(!out.status.success(), "{mode}: {args:?} must fail, printed {}", stdout_of(&out));
        assert!(stdout_of(&out).is_empty(), "{mode}: {args:?} prints nothing to stdout");
        stderr_of(&out)
    })
}

/// The `fields` of `show <change> --json` in both projects (the same).
fn fields_of(b: &Pair, change: &str) -> Value {
    let [fs, remote] = projects(b).map(|(_, p)| json_in(p, &["show", change, "--json"])["fields"].clone());
    assert_eq!(remote, fs, "the same fields in both modes");
    fs
}

/// The `info.fields` of `discuss show <slug> --json` in both projects.
fn discussion_fields_of(b: &Pair, slug: &str) -> Value {
    let [fs, remote] =
        projects(b).map(|(_, p)| json_in(p, &["discuss", "show", slug, "--json"])["info"]["fields"].clone());
    assert_eq!(remote, fs, "the same fields in both modes");
    fs
}

// === 規格「自由欄位的寫入動詞」：四個動詞，兩種模式同形 ===

#[test]
fn set_prints_the_changed_keys_in_byte_order_in_both_modes() {
    // Scenario「設定兩個欄位」。
    let b = pair("fields-set", DOCS);
    assert_eq!(ok(&b, &["change", "field", "set", "demo", "ticket=PROJ-1", "source=portal", "--no-color"]), "✓ demo: set source, ticket\n");
    assert_eq!(fields_of(&b, "demo"), json!({ "source": "portal", "ticket": "PROJ-1" }));
    // 存放的樣子：一個區塊，鍵依位元組順序，值加雙引號。
    assert_eq!(
        read(&b.fs, "changes/demo/.meta.yaml").unwrap(),
        format!("{META}fields:\n  source: \"portal\"\n  ticket: \"PROJ-1\"\n")
    );
}

#[test]
fn set_json_is_the_change_and_every_field_after_the_write() {
    // Scenario「--json 回傳全部欄位」「remote 模式與本機同形」。
    let b = pair("fields-set-json", DOCS);
    ok(&b, &["change", "field", "set", "demo", "source=portal"]);
    let stdout = ok(&b, &["change", "field", "set", "demo", "ticket=PROJ-1", "--json"]);
    let value: Value = serde_json::from_str(&stdout).expect("JSON stdout");
    assert_eq!(value, json!({ "change": "demo", "fields": { "source": "portal", "ticket": "PROJ-1" } }));
    assert!(value["change"].is_string() && value["fields"].is_object());
}

#[test]
fn a_value_keeps_everything_after_the_first_equals_sign_and_reads_back_verbatim() {
    // Scenario「值含等號」與「值讀回逐字相同」。
    let b = pair("fields-values", DOCS);
    let values = [("query", "a=b"), ("n", "123"), ("flag", "true"), ("nothing", "null"), ("note", "a: b # c"), ("quote", "say \"hi\" \\ there"), ("zh", "一段中文"), ("empty", "")];
    for (key, value) in values {
        ok(&b, &["change", "field", "set", "demo", &format!("{key}={value}")]);
    }
    let fields = fields_of(&b, "demo");
    for (key, value) in values {
        assert_eq!(fields[key], json!(value), "{key}");
    }
    // 其餘的欄位照常讀得到。
    assert_eq!(json_in(&b.fs, &["show", "demo", "--json"])["schema"], "spec-driven");
}

#[test]
fn unset_names_only_the_keys_it_removed_and_a_no_op_says_no_change() {
    // Scenario「移除欄位」「沒有變動」。
    let b = pair("fields-unset", DOCS);
    ok(&b, &["change", "field", "set", "demo", "ticket=PROJ-1", "source=portal"]);
    assert_eq!(ok(&b, &["change", "field", "unset", "demo", "ticket", "ghost", "--no-color"]), "✓ demo: unset ticket\n");
    assert_eq!(fields_of(&b, "demo"), json!({ "source": "portal" }));

    let before = read(&b.fs, "changes/demo/.meta.yaml").unwrap();
    assert_eq!(ok(&b, &["change", "field", "unset", "demo", "ghost", "--no-color"]), "✓ demo: no change\n");
    assert_eq!(ok(&b, &["change", "field", "set", "demo", "source=portal", "--no-color"]), "✓ demo: no change\n");
    assert_eq!(read(&b.fs, "changes/demo/.meta.yaml").unwrap(), before, "the file stays byte for byte");

    // 最後一個鍵移除後，區塊消失。
    ok(&b, &["change", "field", "unset", "demo", "source"]);
    assert_eq!(read(&b.fs, "changes/demo/.meta.yaml").unwrap(), META);
    assert_eq!(fields_of(&b, "demo"), Value::Null);
}

#[test]
fn one_bad_pair_writes_nothing() {
    // Scenario「全部寫入或全部不寫」：ok=1 Bad=2。
    let b = pair("fields-all-or-nothing", DOCS);
    let errors = fails(&b, &["change", "field", "set", "demo", "ok=1", "Bad=2"]);
    for stderr in &errors {
        assert!(stderr.contains("Bad"), "{stderr}");
    }
    assert_eq!(fields_of(&b, "demo"), Value::Null, "ok was not written");
    assert_eq!(read(&b.fs, "changes/demo/.meta.yaml").unwrap(), META);

    // 值太長、含換行，以及第 33 個鍵。
    let too_long = format!("note={}", "x".repeat(1025));
    fails(&b, &["change", "field", "set", "demo", &too_long]);
    fails(&b, &["change", "field", "set", "demo", "note=a\nb"]);
    let pairs: Vec<String> = (0..33).map(|i| format!("k{i:02}=v")).collect();
    let mut args = vec!["change", "field", "set", "demo"];
    args.extend(pairs.iter().map(String::as_str));
    fails(&b, &args);
    assert_eq!(fields_of(&b, "demo"), Value::Null);
}

#[test]
fn an_argument_without_an_equals_sign_or_a_repeated_key_is_refused_before_anything_runs() {
    let b = pair("fields-usage", DOCS);
    for args in [
        &["change", "field", "set", "demo", "ticket"][..],
        &["change", "field", "set", "demo", "a=1", "a=2"],
        &["change", "field", "unset", "demo", "a", "a"],
        &["discuss", "field", "set", "board-search", "owner"],
        &["new", "change", "fresh", "--field", "ticket"],
        &["new", "change", "fresh", "--field", "a=1", "--field", "a=2"],
        &["discuss", "new", "Fresh topic", "--field", "owner"],
    ] {
        fails(&b, args);
    }
    assert_eq!(fields_of(&b, "demo"), Value::Null);
    assert!(read(&b.fs, "changes/fresh/.meta.yaml").is_none(), "nothing was created");
    // 缺名稱或缺欄位：動詞文字本身就不成立。
    for args in [&["change", "field", "set", "demo"][..], &["change", "field", "unset"], &["discuss", "field", "set"]] {
        fails(&b, args);
    }
}

#[test]
fn a_change_that_is_not_there_is_reported_as_not_found() {
    let b = pair("fields-missing", DOCS);
    for stderr in fails(&b, &["change", "field", "set", "ghost", "a=1"]) {
        assert!(stderr.contains("not found"), "{stderr}");
    }
    for stderr in fails(&b, &["change", "field", "unset", "ghost", "a"]) {
        assert!(stderr.contains("not found"), "{stderr}");
    }
}

#[test]
fn the_discussion_verbs_write_the_frontmatter_and_leave_the_rounds() {
    // Scenario「討論的欄位」。
    let b = pair("fields-discuss", DOCS);
    let stdout = ok(&b, &["discuss", "field", "set", "board-search", "owner=alice", "--json"]);
    let value: Value = serde_json::from_str(&stdout).expect("JSON stdout");
    assert_eq!(value, json!({ "discussion": "board-search", "fields": { "owner": "alice" } }));
    assert_eq!(discussion_fields_of(&b, "board-search"), json!({ "owner": "alice" }));
    let (head, body) = RECORD.split_once("---\n\n").unwrap();
    assert_eq!(
        read(&b.fs, "discussions/board-search.md").unwrap(),
        format!("{head}fields:\n  owner: \"alice\"\n---\n\n{body}"),
        "only the frontmatter changed"
    );

    assert_eq!(ok(&b, &["discuss", "field", "set", "board-search", "team=core", "--no-color"]), "✓ board-search: set team\n");
    assert_eq!(ok(&b, &["discuss", "field", "unset", "board-search", "owner", "team", "--no-color"]), "✓ board-search: unset owner, team\n");
    assert_eq!(read(&b.fs, "discussions/board-search.md").unwrap(), RECORD);
    assert_eq!(discussion_fields_of(&b, "board-search"), Value::Null);
}

#[test]
fn an_archived_or_missing_discussion_is_reported_as_not_found() {
    // Scenario「已封存的討論」。
    let archived = RECORD.replace("board-search", "old-talk");
    let docs = [DOCS, &[("archive/discussions/2026-10-01-old-talk.md", archived.as_str())]].concat();
    let b = pair("fields-discuss-missing", &docs);
    for slug in ["old-talk", "ghost"] {
        for stderr in fails(&b, &["discuss", "field", "set", slug, "owner=alice"]) {
            assert!(stderr.contains("not found"), "{slug}: {stderr}");
        }
    }
    assert_eq!(read(&b.fs, "archive/discussions/2026-10-01-old-talk.md").unwrap(), archived);
}

// === 規格「建立時附帶」 ===

#[test]
fn new_change_with_field_flags_creates_the_change_with_its_fields() {
    let b = pair("fields-new-change", DOCS);
    for (mode, p) in projects(&b) {
        let out = ok_in(p, &["new", "change", "fresh", "--field", "ticket=PROJ-1", "--field", "source=portal", "--no-color"]);
        assert!(stdout_of(&out).starts_with("✓ Created change: fresh\n"), "{mode}: {}", stdout_of(&out));
        assert!(stderr_of(&out).is_empty(), "{mode}: no warning from a server that keeps fields: {}", stderr_of(&out));
    }
    assert_eq!(fields_of(&b, "fresh"), json!({ "source": "portal", "ticket": "PROJ-1" }));
    // 建立的那一份文件就有欄位。
    let meta = read(&b.fs, "changes/fresh/.meta.yaml").unwrap();
    assert!(meta.ends_with("fields:\n  source: \"portal\"\n  ticket: \"PROJ-1\"\n"), "{meta}");
}

#[test]
fn new_change_with_a_field_outside_the_rules_creates_nothing() {
    // Scenario「建立時欄位不合規則」。
    let b = pair("fields-new-change-bad", DOCS);
    fails(&b, &["new", "change", "fresh", "--field", "Bad=1"]);
    assert!(read(&b.fs, "changes/fresh/.meta.yaml").is_none());
    for (mode, p) in projects(&b) {
        let names: Vec<String> =
            json_in(p, &["list", "--json"])["changes"].as_array().unwrap().iter().map(|c| c["name"].as_str().unwrap().to_string()).collect();
        assert!(!names.contains(&"fresh".to_string()), "{mode}: {names:?}");
    }
}

#[test]
fn discuss_new_with_field_flags_creates_the_record_with_its_fields() {
    let b = pair("fields-discuss-new", DOCS);
    for (mode, p) in projects(&b) {
        let out = ok_in(p, &["discuss", "new", "Fresh topic", "--slug", "fresh-topic", "--field", "owner=alice", "--field", "n=123", "--json"]);
        assert!(stderr_of(&out).is_empty(), "{mode}: {}", stderr_of(&out));
    }
    assert_eq!(discussion_fields_of(&b, "fresh-topic"), json!({ "n": "123", "owner": "alice" }));
    let record = read(&b.fs, "discussions/fresh-topic.md").unwrap();
    assert!(record.contains("\nfields:\n  n: \"123\"\n  owner: \"alice\"\n---\n"), "{record}");

    fails(&b, &["discuss", "new", "Bad topic", "--slug", "bad-topic", "--field", "Bad=1"]);
    assert!(read(&b.fs, "discussions/bad-topic.md").is_none());
}

// === 規格「自由欄位的讀取輸出」 ===

#[test]
fn the_read_verbs_carry_the_fields_in_json_and_never_in_the_human_form() {
    // Scenario「清單帶出欄位」「人眼輸出不變」「討論資訊帶出欄位」。
    let b = pair("fields-reads", DOCS);
    let human_before = projects(&b).map(|(_, p)| [stdout_of(&ok_in(p, &["show", "demo", "--no-color"])), stdout_of(&ok_in(p, &["list", "--no-color"])), stdout_of(&ok_in(p, &["discuss", "list", "--no-color"]))]);
    ok(&b, &["change", "field", "set", "demo", "ticket=PROJ-1", "source=portal"]);
    ok(&b, &["discuss", "field", "set", "board-search", "owner=alice"]);

    for (mode, p) in projects(&b) {
        let list = json_in(p, &["list", "--json", "--sort", "name"]);
        let item = |name: &str| list["changes"].as_array().unwrap().iter().find(|c| c["name"] == name).unwrap().clone();
        assert_eq!(item("demo")["fields"], json!({ "source": "portal", "ticket": "PROJ-1" }), "{mode}");
        assert!(item("other").get("fields").is_none(), "{mode}: {}", item("other"));
        assert_eq!(json_in(p, &["show", "demo", "--json"])["fields"], json!({ "source": "portal", "ticket": "PROJ-1" }), "{mode}");
        assert!(json_in(p, &["show", "other", "--json"]).get("fields").is_none(), "{mode}");
        assert_eq!(json_in(p, &["discuss", "list", "--json"])["discussions"][0]["fields"], json!({ "owner": "alice" }), "{mode}");
        assert_eq!(json_in(p, &["discuss", "show", "board-search", "--json"])["info"]["fields"], json!({ "owner": "alice" }), "{mode}");
        assert_eq!(json_in(p, &["discuss", "search", "board", "--json"])["hits"][0]["fields"], json!({ "owner": "alice" }), "{mode}");
    }
    let human_after = projects(&b).map(|(_, p)| [stdout_of(&ok_in(p, &["show", "demo", "--no-color"])), stdout_of(&ok_in(p, &["list", "--no-color"])), stdout_of(&ok_in(p, &["discuss", "list", "--no-color"]))]);
    assert_eq!(human_after, human_before, "the human forms do not change");
    // 兩種模式的 --json 同形（討論的 path 是既有的分歧：本機是檔案路徑，remote 是儲存的鍵）。
    ok(&b, &["show", "demo", "--json"]);
}

#[test]
fn a_project_with_no_free_field_prints_what_it_printed_before_this_capability() {
    // Scenario「沒有欄位的專案輸出不變」。底下三段文字是加上這個能力之前的 CLI 對同一批
    // 文件印出的內容，逐位元照抄（清單的 modifiedAt 是檔案時間，先換成固定的字再比）。
    let p = TempProject::local("fields-unchanged", DOCS);
    let show = "{\n  \"created\": \"2026-11-01\",\n  \"deltaSpecs\": [],\n  \"design\": null,\n  \"fromDiscussions\": [],\n  \"name\": \"demo\",\n  \"proposal\": \"## Why\\n\\nBecause.\\n\",\n  \"restaleFrom\": [],\n  \"schema\": \"spec-driven\",\n  \"tasks\": \"- [ ] 1.1 first\\n\"\n}\n";
    assert_eq!(stdout_of(&ok_in(&p, &["show", "demo", "--json"])), show);

    let path = p.spec_file("discussions/board-search.md").to_string_lossy().replace('\\', "/");
    let path = serde_json::to_string(&path).unwrap();
    let discussions = format!(
        "{{\n  \"discussions\": [\n    {{\n      \"archived\": false,\n      \"created\": \"2026-11-01\",\n      \"path\": {path},\n      \"rounds\": 0,\n      \"slug\": \"board-search\",\n      \"status\": \"open\",\n      \"topic\": \"Board search\"\n    }}\n  ]\n}}\n"
    );
    let printed: Value = serde_json::from_slice(&ok_in(&p, &["discuss", "list", "--json"]).stdout).unwrap();
    let expected: Value = serde_json::from_str(&discussions).unwrap();
    // 路徑的寫法隨平台的暫存目錄而異（符號連結、磁碟代號），所以路徑以外逐位元比。
    let without_path = |text: &str, value: &Value| text.replace(&serde_json::to_string(&value["discussions"][0]["path"]).unwrap(), "\"<path>\"");
    assert_eq!(
        without_path(&stdout_of(&ok_in(&p, &["discuss", "list", "--json"])), &printed),
        without_path(&discussions, &expected)
    );

    let list = "{\n  \"changes\": [\n    {\n      \"completedTasks\": 0,\n      \"created\": \"2026-11-01\",\n      \"modifiedAt\": \"<time>\",\n      \"name\": \"demo\",\n      \"status\": \"in-progress\",\n      \"summary\": \"Because.\",\n      \"totalTasks\": 1\n    },\n    {\n      \"completedTasks\": 0,\n      \"created\": \"2026-11-01\",\n      \"modifiedAt\": \"<time>\",\n      \"name\": \"other\",\n      \"status\": \"in-progress\",\n      \"totalTasks\": 0\n    }\n  ]\n}\n";
    let printed = stdout_of(&ok_in(&p, &["list", "--json", "--sort", "name"]));
    let value: Value = serde_json::from_str(&printed).unwrap();
    let mut normalized = printed.clone();
    for change in value["changes"].as_array().unwrap() {
        normalized = normalized.replace(change["modifiedAt"].as_str().expect("a file time"), "<time>");
    }
    assert_eq!(normalized, list);
}

// === client-protocol「舊 Server 不保存欄位時警告」 ===

#[test]
fn a_server_that_does_not_declare_fields_gets_the_create_and_the_user_gets_one_warning() {
    // remote_plan 的 mock server 的握手沒有 `capabilities.fields`：它就是一台舊 Server。
    let created = r#"{"name":"demo","schema":"spec-driven"}"#.to_string();
    let mock = mock_server(vec![("POST", "/changes".into(), 200, created)]);
    // The projects of this file and of remote_read_verbs share one directory
    // prefix: a tag must not be one that file uses.
    let p = TempProject::remote("fields-old-server", &mock.base);
    let out = p.run(&["new", "change", "demo", "--field", "ticket=PROJ-1", "--no-color"]);
    assert!(out.status.success(), "the create itself succeeds: {}", stderr_of(&out));
    assert!(stdout_of(&out).starts_with("✓ Created change: demo\n"), "{}", stdout_of(&out));
    let stderr = stderr_of(&out);
    assert_eq!(stderr.lines().count(), 1, "one line: {stderr}");
    assert!(stderr.contains("did not keep") && stderr.contains("field"), "{stderr}");
    assert!(!stderr.contains("PROJ-1"), "the warning never prints a value: {stderr}");
    // 欄位照樣送出：要不要保存是 Server 的事。
    assert_eq!(mock.body_of("POST", "/changes"), r#"{"name":"demo","fields":{"ticket":"PROJ-1"}}"#);

    // 沒有帶欄位就沒有警告。
    let out = p.run(&["new", "change", "demo", "--no-color"]);
    assert!(out.status.success() && stderr_of(&out).is_empty(), "{}", stderr_of(&out));
}

#[test]
fn the_same_warning_follows_a_new_discussion_on_such_a_server() {
    let created = r#"{"slug":"board-search","topic":"Board search","path":"discussions/board-search.md"}"#.to_string();
    let mock = mock_server(vec![("POST", "/discussions".into(), 200, created)]);
    let p = TempProject::remote("fields-old-server-discuss", &mock.base);
    let out = p.run(&["discuss", "new", "Board search", "--field", "owner=alice", "--no-color"]);
    assert!(out.status.success(), "{}", stderr_of(&out));
    let stderr = stderr_of(&out);
    assert_eq!(stderr.lines().count(), 1, "one line: {stderr}");
    assert!(stderr.contains("did not keep"), "{stderr}");
    let out = p.run(&["discuss", "new", "Board search", "--no-color"]);
    assert!(out.status.success() && stderr_of(&out).is_empty(), "{}", stderr_of(&out));
}

#[test]
fn the_field_verbs_on_a_server_without_the_route_fail_and_say_so() {
    // 相容性：新版 CLI 連到舊版 Server，設定欄位的路由不存在，回 404。
    let mock = mock_server(vec![]);
    let p = TempProject::remote("fields-old-server-put", &mock.base);
    let out = p.run(&["change", "field", "set", "demo", "ticket=PROJ-1"]);
    assert!(!out.status.success());
    assert!(stdout_of(&out).is_empty());
    assert_eq!(mock.verb_calls(), ["PUT /changes/demo/fields"]);
}
