//! `change rank` 的 remote 臂（change-plan「change rank 動詞寫入看板順序鍵」的
//! remote 情境；verb-contract「模式分岔的單點宣告」的「change rank 於 remote 模式走
//! server」）：remote 模式把同一個請求送到 server，stdout、stderr 與 exit code 與本機
//! 相同，且不碰本機 store。
//!
//! 順序鍵落進 board resource、metadata 逐位元不變、缺順序鍵的欄一次提交補章、守門
//! 失敗零寫入、board resource 被同時改寫時的版本衝突，這些 server 端的事實由
//! speclink-server 的 `api::plan_api` 與 speclink-host 的 `port_rank` 釘住；這裡用
//! remote_plan 的 capturing mock server 釘 CLI 這一端：發出的請求、印出的結果，以及
//! server 回來的拒絕與衝突怎麼呈現。

use crate::remote_plan::{mock_server, stderr_of, stdout_of, TempProject};

const ANCHOR: &str = "schema: spec-driven\ncreated: 2026-09-01\nboard_rank: n\n";
const UNRANKED: &str = "schema: spec-driven\ncreated: 2026-09-02\n";

/// fs 對照組：a 已有順序鍵 n、c 沒有，兩者同為提案中。
fn fs_pair(tag: &str, c_meta: &str) -> TempProject {
    let p = TempProject::at(tag);
    p.put_change("a", ANCHOR);
    p.put_change("c", c_meta);
    p
}

#[test]
fn remote_change_rank_posts_one_request_and_prints_the_fs_line() {
    // Scenario「remote 模式寫入 board resource」「change rank 於 remote 模式走 server」：
    // 一次 POST /changes/c/rank（補章也由 server 在同一次提交完成，CLI 不另發請求），
    // 成功行與 fs 模式逐位元相同，本機沒有任何規格檔被建立或改寫。
    let fs = fs_pair("rank-fs", UNRANKED);
    let fs_out = fs.run(&["change", "rank", "c", "--before", "a", "--no-color"]);
    assert!(fs_out.status.success(), "fs stderr: {}", stderr_of(&fs_out));

    let mock = mock_server(vec![(
        "POST",
        "/changes/c/rank".into(),
        200,
        r#"{"change":"c","anchor":"a","position":"before"}"#.into(),
    )]);
    let remote = TempProject::remote("rank-remote", &mock.base);
    let out = remote.run(&["change", "rank", "c", "--before", "a", "--no-color"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "✓ c ranked before a\n");
    assert_eq!(stdout_of(&out), stdout_of(&fs_out), "the same line as fs mode");
    assert_eq!(stderr_of(&out), stderr_of(&fs_out));
    assert_eq!(mock.verb_calls(), ["POST /changes/c/rank"], "one request");
    assert_eq!(
        mock.body_of("POST", "/changes/c/rank"),
        r#"{"anchor":"a","position":"before","force":false}"#
    );
    assert!(!remote.dir.join("speclink").exists(), "the local store is never touched");
}

#[test]
fn remote_change_rank_json_is_the_fs_shape() {
    // Scenario「remote 模式的 --json 形狀與本機相同」：三個欄位、不含順序鍵的值；
    // --force 在請求裡帶 true。
    let fs = fs_pair("rank-json-fs", "schema: spec-driven\ncreated: 2026-09-02\nboard_rank: t\n");
    let fs_out = fs.run(&["change", "rank", "c", "--after", "a", "--force", "--json"]);
    assert!(fs_out.status.success(), "fs stderr: {}", stderr_of(&fs_out));

    let mock = mock_server(vec![(
        "POST",
        "/changes/c/rank".into(),
        200,
        r#"{"change":"c","anchor":"a","position":"after"}"#.into(),
    )]);
    let remote = TempProject::remote("rank-json-remote", &mock.base);
    let out = remote.run(&["change", "rank", "c", "--after", "a", "--force", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), stdout_of(&fs_out), "bit-for-bit the fs JSON");
    let value: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("JSON stdout");
    assert_eq!(value, serde_json::json!({ "change": "c", "anchor": "a", "position": "after" }));
    assert_eq!(
        mock.body_of("POST", "/changes/c/rank"),
        r#"{"anchor":"a","position":"after","force":true}"#
    );
}

#[test]
fn remote_guard_refusal_prints_the_engine_line_and_writes_nothing() {
    // Scenario「remote 模式守門失敗零寫入」：server 以 refused 409 帶回引擎原文，CLI
    // 印出與 fs 模式相同的那一行、非 0 結束，stdout 為空。
    let fs = fs_pair("rank-refused-fs", "schema: spec-driven\ncreated: 2026-09-02\nboard_rank: t\n");
    let fs_out = fs.run(&["change", "rank", "c", "--after", "a"]);
    assert!(!fs_out.status.success(), "fs refuses");

    let line = "'c' already has a board rank; pass --force to move it anyway";
    let mock = mock_server(vec![(
        "POST",
        "/changes/c/rank".into(),
        409,
        format!(r#"{{"status":409,"reason":"refused","message":"{line}"}}"#),
    )]);
    let remote = TempProject::remote("rank-refused-remote", &mock.base);
    let out = remote.run(&["change", "rank", "c", "--after", "a"]);
    assert!(!out.status.success(), "a refusal exits non-zero");
    let err = stderr_of(&out);
    assert!(err.contains("already has a board rank") && err.contains("--force"), "stderr: {err}");
    assert_eq!(err, stderr_of(&fs_out), "the same line as fs mode");
    assert!(stdout_of(&out).is_empty(), "stdout stays empty");
    assert_eq!(mock.verb_calls(), ["POST /changes/c/rank"]);
}

#[test]
fn remote_concurrent_board_write_reports_a_revision_conflict() {
    // Scenario「remote 模式 board resource 被同時改寫」：server 回 revision_conflict，
    // CLI 顯示既有的衝突訊息、非 0 結束。
    let mock = mock_server(vec![(
        "POST",
        "/changes/c/rank".into(),
        409,
        r#"{"status":409,"reason":"revision_conflict","message":"conflict on board/order: expected version 3, found version 4"}"#
            .into(),
    )]);
    let remote = TempProject::remote("rank-conflict-remote", &mock.base);
    let out = remote.run(&["change", "rank", "c", "--before", "a"]);
    assert!(!out.status.success(), "a conflict exits non-zero");
    assert!(
        stderr_of(&out).contains("content changed since you read it"),
        "stderr: {}",
        stderr_of(&out)
    );
    assert!(stdout_of(&out).is_empty());
}

#[test]
fn an_old_server_without_the_route_is_not_found() {
    // 相容性：舊版 Server 沒有這條路由，回沒有內容的 404；CLI 顯示既有的找不到訊息。
    let mock = mock_server(vec![("POST", "/changes/c/rank".into(), 404, String::new())]);
    let remote = TempProject::remote("rank-old-server", &mock.base);
    let out = remote.run(&["change", "rank", "c", "--before", "a"]);
    assert!(!out.status.success(), "a missing route exits non-zero");
    assert!(stderr_of(&out).contains("resource not found"), "stderr: {}", stderr_of(&out));
}
