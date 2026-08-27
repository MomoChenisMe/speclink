//! remote workspace 的 handshake 與資料面三類矩陣（規格「handshake 成功後才
//! 建立 remote session」「capability 驅動停用且不偽造缺口」；design「決策 1：
//! RemoteDataSource 的三類覆蓋矩陣」「決策 2：capability 描述隨 session 建立
//! 產生」「決策 6：極簡開啟入口與 handshake fail-closed」）。
//!
//! in-process speclink-server：open_workspace 以 project[/repo] 識別
//! handshake——成功回 project/repo 顯示名與 capability 描述、403/404/多義
//! 原樣回錯不建 runtime；直達類逐方法回 server 真值；組合類 set_tasks 中途
//! 失敗中止並回報筆數；不支援類回拒絕錯誤。

use crate::common;

use chrono::{Duration, Utc};
use crate::common::Harness;
use speclink_remote::credentials::{CredentialKind, CredentialStore, MemoryCredentialStore};
use speclink_desktop_lib::remote::{self, RemoteWorkspace, TokenManager};
use speclink_remote::RemoteError;
use speclink_protocol::query::ChangeSummary;
use speclink_server::identity::{IdentityStore, NewInvitation};
use speclink_store::{CommandContext, DocumentId, TeamStore};
use std::sync::Arc;

const FOUR_TASKS: &str = "- [ ] 1.1 First\n- [ ] 1.2 Second\n- [ ] 1.3 Third\n- [ ] 1.4 Fourth\n";

/// 一條 PAT 憑證入 in-memory store，回傳 (credentials, manager)。
fn runtime(h: &Harness) -> (MemoryCredentialStore, Arc<TokenManager>) {
    let store = MemoryCredentialStore::new();
    store
        .set(&h.origin, CredentialKind::Pat, &common::pat_of(h))
        .expect("set pat");
    let manager = Arc::new(TokenManager::new(&h.origin));
    (store, manager)
}

/// handshake 成功開出 workspace。
fn open(
    h: &Harness,
    credentials: &MemoryCredentialStore,
    manager: &Arc<TokenManager>,
) -> RemoteWorkspace {
    let (workspace, _info) =
        remote::open_workspace(&h.origin, "demo", manager, credentials).expect("open workspace");
    workspace
}

/// 種一個討論文件（concluded fixture 格式）進 demo/backend scope。
fn seed_discussion(store: &dyn TeamStore, slug: &str, topic: &str) {
    let doc = format!(
        "---\ntopic: {topic}\nslug: {slug}\nstatus: concluded\ncreated: 2026-01-02\n---\n\n\
         # Discussion: {topic}\n\n\
         ## Context\n\nFixture context.\n\n\
         ## Rounds\n\n### Round 1 — assumptions (2026-01-02)\n\n**Focus**: scope\n\n\
         ## Conclusion\n\n**Decision**: do it\n"
    );
    let mut uow = store
        .begin_unit_of_work(
            &common::scope(),
            CommandContext {
                command: "seed".into(),
                actor: "seed".into(),
            },
        )
        .expect("begin uow");
    uow.create(
        DocumentId::Discussion {
            slug: slug.into(),
            archived: false,
        },
        &doc,
    );
    store.commit(uow, Vec::new()).expect("seed discussion");
}

// --- 看板欄位由生命週期標記驅動（spec desktop-app）---

#[test]
fn change_stage_matrix_follows_the_lifecycle_markers() {
    // spec「欄位判定矩陣」Example：0=提案中、1=進行中、2=已就緒。
    fn summary(started_at: Option<&str>, completed: usize, total: usize) -> ChangeSummary {
        ChangeSummary {
            name: "demo".into(),
            summary: String::new(),
            status: "in-progress".into(),
            completed_tasks: completed,
            total_tasks: total,
            restale_from: Vec::new(),
            meta_error: None,
            repo: None,
            lifecycle: None,
            claimed_by: None,
            started_at: started_at.map(str::to_string),
            created_by: None,
            created: None,
            from_discussions: Vec::new(),
        }
    }
    let cases: [(Option<&str>, usize, usize, u8); 7] = [
        (None, 0, 0, 0),
        (None, 0, 28, 0),
        (None, 3, 28, 1),
        (Some("2026-07-30"), 0, 28, 1),
        (Some("2026-07-30"), 13, 28, 1),
        (None, 28, 28, 2),
        (Some("2026-07-30"), 28, 28, 2),
    ];
    for (started, completed, total, want) in cases {
        assert_eq!(
            remote::change_stage(&summary(started, completed, total)),
            want,
            "started={started:?} progress={completed}/{total}"
        );
    }
}

#[test]
fn started_at_rides_the_remote_change_list_payload() {
    // in-progress 蓋章後 startedAt 隨清單 payload 進桌面（系統匣與看板同源）。
    let h = common::harness();
    common::seed_change(h.store.as_ref(), FOUR_TASKS);
    {
        let mut uow = h
            .store
            .begin_unit_of_work(
                &common::scope(),
                CommandContext {
                    command: "seed".into(),
                    actor: "seed".into(),
                },
            )
            .expect("begin uow");
        uow.create(
            DocumentId::ChangeMeta {
                change: "started-zero".into(),
            },
            "schema: spec-driven\ncreated: 2026-07-29\nstarted_at: 2026-07-30\nstarted_by: Momo <m@example.com>\n",
        );
        h.store.commit(uow, Vec::new()).expect("seed started change");
    }
    let (credentials, manager) = runtime(&h);
    let ws = open(&h, &credentials, &manager);

    let changes = ws.list_changes(&credentials).expect("list changes").changes;
    let started = changes
        .iter()
        .find(|c| c.name == "started-zero")
        .expect("started change listed");
    assert_eq!(
        started.started_at.as_deref(),
        Some("2026-07-30"),
        "startedAt comes from the server meta"
    );
    assert_eq!(
        remote::change_stage(started),
        1,
        "帶 startedAt 且完成數 0 判為進行中"
    );
    let unstarted = changes.iter().find(|c| c.name == "demo").expect("demo listed");
    assert_eq!(unstarted.started_at, None, "未開工 change 不帶 startedAt");
    assert_eq!(remote::change_stage(unstarted), 0, "未開工零進度停在提案中");
}

// --- 開啟入口：handshake fail-closed ---

#[test]
fn open_handshakes_and_returns_identity_and_capability_description() {
    let h = common::harness();
    common::seed_change(h.store.as_ref(), FOUR_TASKS);
    let (credentials, manager) = runtime(&h);

    // 單 repo 的 project：省略 repo 由 server 自動綁定。
    let (_, info) =
        remote::open_workspace(&h.origin, "demo", &manager, &credentials).expect("open");
    assert_eq!(info.project_key, "demo");
    assert_eq!(info.project_name, "Demo", "handshake 回的 project 顯示名");
    assert_eq!(info.repo_key, "backend", "單 repo 自動綁定");
    assert_eq!(info.repo_name, "backend");

    // 顯式 project/repo 形式同樣成立。
    let (_, info2) =
        remote::open_workspace(&h.origin, "demo/backend", &manager, &credentials).expect("open");
    assert_eq!(info2.repo_key, "backend");

    // capability 描述：直達／組合類為真、不支援類為假（決策 1 矩陣）。
    let caps = &info.capabilities;
    for (name, on) in [
        ("listChanges", caps.list_changes),
        ("listSpecs", caps.list_specs),
        ("status", caps.status),
        ("getDocument", caps.get_document),
        ("setTaskDone", caps.set_task_done),
        ("setAllTasks", caps.set_all_tasks),
        ("archive", caps.archive),
        ("listDiscussions", caps.list_discussions),
        ("getDiscussionDocument", caps.get_discussion_document),
        ("promoteDiscussion", caps.promote_discussion),
        ("archiveDiscussion", caps.archive_discussion),
        ("listArchived", caps.list_archived),
        ("getArchivedDocument", caps.get_archived_document),
        ("archivedCapabilities", caps.archived_capabilities),
        ("searchWorkspace", caps.search_workspace),
        ("getSpecDocument", caps.get_spec_document),
        // remote-verb-parity：動詞端點直達，editor handshake 四欄全真。
        ("validate", caps.validate),
        ("analyze", caps.analyze),
        ("deleteChange", caps.delete_change),
        ("moveTask", caps.move_task),
        // remote-board-order：看板拖排直達 board resource，editor 為真。
        ("reorderCard", caps.reorder_card),
        // remote-read-parity：change 詮釋資料與 capability 清單以 status
        // payload 既有欄位映射——資料已在 wire 上，capability 為真。
        ("changeMeta", caps.change_meta),
        ("changeCapabilities", caps.change_capabilities),
        // remote-claim-ownership：editor handshake 的認領面為真。
        ("claim", caps.claim),
    ] {
        assert!(on, "{name} 是直達／組合／payload 映射類，capability 應為真");
    }
    assert!(
        caps.live_updates,
        "handshake 宣告了 SSE 與 polling——事件能力為真"
    );
}

#[test]
fn open_fails_closed_on_403_404_and_ambiguity() {
    let h = common::harness();
    common::seed_change(h.store.as_ref(), FOUR_TASKS);

    // 403：有效 token、非成員——server 拒絕原樣呈現。
    let stranger_invite = h
        .identity
        .create_invitation(NewInvitation {
            email: "stranger@example.com".to_string(),
            display: "Stranger".to_string(),
            memberships: vec![],
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    let stranger_id = h
        .identity
        .accept_invitation(&stranger_invite, "pw-stranger")
        .expect("accept");
    let (_, stranger_pat) = h
        .identity
        .create_pat(&stranger_id, "test", None)
        .expect("pat");
    let credentials = MemoryCredentialStore::new();
    credentials
        .set(&h.origin, CredentialKind::Pat, &stranger_pat)
        .expect("set");
    let manager = Arc::new(TokenManager::new(&h.origin));
    let err = remote::open_workspace(&h.origin, "demo", &manager, &credentials)
        .expect_err("非成員 fail-closed");
    assert_eq!(err.status, Some(403), "403 原樣回錯：{}", err.message);

    // 404：不存在的 project。
    let (credentials, manager) = runtime(&h);
    let err = remote::open_workspace(&h.origin, "nope", &manager, &credentials)
        .expect_err("不存在的 project fail-closed");
    assert_eq!(err.status, Some(404));
    assert!(
        err.message.contains("nope"),
        "server 錯誤原樣呈現：{}",
        err.message
    );

    // 多義：multi 有兩個 repo、未指定——server 的 refused 原樣呈現。
    let err = remote::open_workspace(&h.origin, "multi", &manager, &credentials)
        .expect_err("多 repo 未指定 fail-closed");
    assert_eq!(err.status, Some(409));
    assert!(
        err.message.contains("web") && err.message.contains("api"),
        "多義拒絕帶候選清單：{}",
        err.message
    );
}

#[test]
fn remote_open_failure_serializes_machine_readable_fields_without_credentials() {
    let cases = [
        ("transport", None, None),
        ("unauthorized", Some("permission_denied"), Some(401)),
        ("forbidden", Some("permission_denied"), Some(403)),
        ("missing", Some("not_found"), Some(404)),
        ("unknown", Some("future_reason"), Some(599)),
    ];

    for (message, reason, status) in cases {
        let failure = remote::RemoteOpenFailure::from(RemoteError {
            message: message.to_string(),
            reason: reason.map(str::to_string),
            status,
            evidence: None,
        });
        let value = serde_json::to_value(failure).expect("serialize remote_open failure");
        let object = value.as_object().expect("failure is an object");

        assert_eq!(object.len(), 3, "failure payload 只允許三個公開欄位");
        assert_eq!(object.get("message").and_then(|v| v.as_str()), Some(message));
        assert_eq!(object.get("reason").and_then(|v| v.as_str()), reason);
        assert_eq!(object.get("status").and_then(|v| v.as_u64()), status.map(u64::from));
        for forbidden in [
            "token",
            "accessToken",
            "refreshCredential",
            "pat",
            "authorization",
            "keychain",
        ] {
            assert!(!object.contains_key(forbidden), "不得序列化 {forbidden}");
        }
    }
}

// --- 直達類：逐方法回 server 真值 ---

#[test]
fn changes_specs_and_artifacts_read_server_truth() {
    let h = common::harness();
    common::seed_change(h.store.as_ref(), FOUR_TASKS);
    {
        let mut uow = h
            .store
            .begin_unit_of_work(
                &common::scope(),
                CommandContext {
                    command: "seed".into(),
                    actor: "seed".into(),
                },
            )
            .expect("begin uow");
        uow.create(
            DocumentId::CanonicalSpec {
                capability: "auth".into(),
            },
            "## Purpose\n\nAuth canon.\n",
        );
        h.store.commit(uow, Vec::new()).expect("seed spec");
    }
    let (credentials, manager) = runtime(&h);
    let ws = open(&h, &credentials, &manager);

    let changes = ws.list_changes(&credentials).expect("list changes");
    assert_eq!(changes.changes.len(), 1);
    assert_eq!(changes.changes[0].name, "demo");
    assert_eq!(changes.changes[0].total_tasks, 4, "任務數是 server 真值");

    let specs = ws.list_specs(&credentials).expect("list specs");
    assert!(
        specs.specs.iter().any(|s| s.id == "auth"),
        "正典 spec 清單：{:?}",
        specs.specs
    );

    let status = ws.change_status(&credentials, "demo").expect("status");
    assert_eq!(status.change_name, "demo");
    assert_eq!(status.schema_name, "spec-driven");
    assert!(
        status.artifacts.iter().any(|a| a.id == "tasks"),
        "artifact 狀態來自 server"
    );

    let doc = ws
        .document(&credentials, "demo", "tasks")
        .expect("document");
    assert!(
        doc.content.contains("- [ ] 1.1 First"),
        "artifact 內文是 server 真值"
    );

    // SpeclinkDataSource 的定址是檔名（proposal.md、tasks.md、specs/{cap}/spec.md）
    // ——server 端點吃 artifact id；runtime 單點正規化，UI 檔名不得漏成 400。
    let doc = ws
        .document(&credentials, "demo", "tasks.md")
        .expect("檔名定址同樣成立");
    assert!(doc.content.contains("- [ ] 1.1 First"));
    let err = ws
        .document(&credentials, "demo", "proposal.md")
        .expect_err("不存在的 artifact 是 404");
    assert_eq!(
        err.status,
        Some(404),
        "檔名正規化後缺席是 not_found、不是 invalid_argument"
    );
}

#[test]
fn archived_spec_document_and_search_reads_reach_server_truth() {
    let h = common::harness();
    common::seed_change(h.store.as_ref(), FOUR_TASKS);
    seed_discussion(h.store.as_ref(), "search-talk", "RemoteNeedle discussion");
    let dated_name = "2026-07-19-old-feature";
    let mut uow = h
        .store
        .begin_unit_of_work(
            &common::scope(),
            CommandContext {
                command: "seed-read-surfaces".into(),
                actor: "seed".into(),
            },
        )
        .expect("begin uow");
    for (document, content) in [
        (
            DocumentId::CanonicalSpec {
                capability: "auth".into(),
            },
            "# auth Specification\n\nRemote canonical truth.\n",
        ),
        (
            DocumentId::ChangeArtifact {
                change: "demo".into(),
                artifact: "proposal.md".into(),
            },
            "## Why\n\nRemoteNeedle in a change.\n",
        ),
        (
            DocumentId::ArchivedChange {
                change: dated_name.into(),
                doc: ".openspec.yaml".into(),
            },
            "schema: spec-driven\ncreated_by: Creator\nfrom_discussion: source-talk\n",
        ),
        (
            DocumentId::ArchivedChange {
                change: dated_name.into(),
                doc: "proposal.md".into(),
            },
            "## Why\n\nRemote archived truth.\n",
        ),
        (
            DocumentId::ArchivedChange {
                change: dated_name.into(),
                doc: "tasks.md".into(),
            },
            "- [x] 1.1 Done\n",
        ),
        (
            DocumentId::ArchivedChange {
                change: dated_name.into(),
                doc: "specs/auth/spec.md".into(),
            },
            "## ADDED Requirements\n\n### Requirement: Login\n",
        ),
    ] {
        uow.create(document, content);
    }
    h.store.commit(uow, Vec::new()).expect("seed read surfaces");

    let (credentials, manager) = runtime(&h);
    let ws = open(&h, &credentials, &manager);
    assert_eq!(
        ws.spec_document(&credentials, "auth")
            .expect("spec document")
            .content,
        "# auth Specification\n\nRemote canonical truth.\n"
    );

    let archived = ws.list_archived(&credentials).expect("archived list");
    let item = archived
        .archived
        .iter()
        .find(|item| item.dated_name == dated_name)
        .expect("archived item");
    assert_eq!(item.tasks_done, Some(1));
    assert_eq!(item.spec_count, 1);
    assert_eq!(item.created_by.as_deref(), Some("Creator"));
    assert_eq!(item.from_discussions, ["source-talk"]);
    assert_eq!(
        ws.archived_document(&credentials, dated_name, "proposal.md")
            .expect("archived document")
            .content,
        "## Why\n\nRemote archived truth.\n"
    );
    assert_eq!(
        ws.archived_capabilities(&credentials, dated_name)
            .expect("archived capabilities"),
        ["auth"]
    );

    let search = ws
        .search_workspace(&credentials, "remoteneedle")
        .expect("search");
    assert_eq!(search.hits.len(), 2);
    assert!(search.hits.iter().any(|hit| hit.kind == "change"));
    assert!(search.hits.iter().any(|hit| hit.kind == "discussion"));
}

#[test]
fn task_flips_claim_and_archive_write_through() {
    let h = common::harness();
    common::seed_change(h.store.as_ref(), FOUR_TASKS);
    {
        // 一個 delta spec 讓 archive 有 capability 可折入正典。
        let mut uow = h
            .store
            .begin_unit_of_work(
                &common::scope(),
                CommandContext {
                    command: "seed".into(),
                    actor: "seed".into(),
                },
            )
            .expect("begin uow");
        uow.create(
            DocumentId::ChangeArtifact {
                change: "demo".into(),
                artifact: "specs/auth/spec.md".into(),
            },
            // 新開 capability 的 delta 自帶合格 Purpose，否則封存被 Purpose 守門擋下
            // （spec archive-merge「新 capability 的 Purpose 自 delta 帶入」）。
            "## Purpose\n\n本 capability 負責使用者登入與登出的可觀察行為，涵蓋工作階段的建立、續期與撤銷三段流程。\n\n\
             ## ADDED Requirements\n\n### Requirement: Login\n\nUsers SHALL log in.\n\n\
             #### Scenario: ok\n\n- **WHEN** login\n- **THEN** ok\n",
        );
        h.store.commit(uow, Vec::new()).expect("seed delta spec");
    }
    let (credentials, manager) = runtime(&h);
    let ws = open(&h, &credentials, &manager);

    ws.set_task_done(&credentials, "demo", "1", true)
        .expect("task done");
    let doc = ws
        .document(&credentials, "demo", "tasks")
        .expect("document");
    assert!(
        doc.content.contains("- [x] 1.1 First"),
        "勾選寫穿 server：{}",
        doc.content
    );

    ws.set_task_done(&credentials, "demo", "1", false)
        .expect("task undone");
    let doc = ws
        .document(&credentials, "demo", "tasks")
        .expect("document");
    assert!(
        doc.content.contains("- [ ] 1.1 First"),
        "取消勾選寫穿 server"
    );

    let claim = ws.claim(&credentials, "demo").expect("claim");
    assert_eq!(
        claim.claimed_by.as_deref(),
        Some(common::ACTOR_IDENTITY),
        "claim 回實際 actor"
    );

    // 封存前補完任務——單筆封存的任務完成度守門（archive-readiness-gating）
    // 一體適用 server 通道，未完成 change 會被拒絕。
    for task in ["1", "2", "3", "4"] {
        ws.set_task_done(&credentials, "demo", task, true)
            .expect("complete before archive");
    }

    let archived = ws.archive(&credentials, "demo").expect("archive");
    assert_eq!(archived.specs.len(), 1, "delta spec 折入正典");
    assert_eq!(archived.specs[0].capability, "auth");
    let changes = ws.list_changes(&credentials).expect("list changes");
    assert!(changes.changes.is_empty(), "archive 後不在 active 清單");
}

#[test]
fn board_reorder_writes_the_board_resource_only_and_meta_is_untouched() {
    // 規格「board resource 為 scope 單文件且 server 不解析」Scenario「拖排不動
    // 卡片文件」＋「看板卡片順序以 board_rank 欄位為真相」修訂後的 remote 不寫
    // meta 斷言：拖排後只有 board resource 產生新 revision。
    let h = common::harness();
    common::seed_named_change(h.store.as_ref(), "alpha", FOUR_TASKS);
    common::seed_named_change(h.store.as_ref(), "bravo", FOUR_TASKS);
    common::seed_named_change(h.store.as_ref(), "carol", FOUR_TASKS);
    let (credentials, manager) = runtime(&h);
    let ws = open(&h, &credentials, &manager);

    let read_meta = |name: &str| {
        let snap = h.store.snapshot(&common::scope()).expect("snapshot");
        snap.read(&DocumentId::ChangeMeta {
            change: name.into(),
        })
        .expect("read meta")
        .expect("meta exists")
    };
    let before: Vec<_> = ["alpha", "bravo", "carol"].map(read_meta).into();

    ws.reorder_card(&credentials, "change", "carol", None, Some("alpha"))
        .expect("reorder lands");

    let snap = h.store.snapshot(&common::scope()).expect("snapshot");
    snap.read(&DocumentId::BoardOrder)
        .expect("read board resource")
        .expect("board resource created by the first reorder");
    for (name, was) in ["alpha", "bravo", "carol"].iter().zip(before) {
        let now = read_meta(name);
        assert_eq!(now.content, was.content, "{name} 的 meta 內容不變");
        assert_eq!(now.revision, was.revision, "{name} 的 meta revision 不變");
    }

    // 清單 overlay 反映新序：carol 移到欄頂（alpha 之前），跨讀取持久。
    let changes = ws.list_changes(&credentials).expect("list changes");
    let names: Vec<&str> = changes.changes.iter().map(|c| c.name.as_str()).collect();
    let pos = |n: &str| names.iter().position(|x| *x == n).expect("card present");
    assert!(pos("carol") < pos("alpha"), "拖排後 carol 在 alpha 前：{names:?}");
}

#[test]
fn discussion_flow_reads_and_writes_through() {
    let h = common::harness();
    common::seed_change(h.store.as_ref(), FOUR_TASKS);
    seed_discussion(h.store.as_ref(), "rate-limiting", "Rate limiting");
    seed_discussion(h.store.as_ref(), "old-topic", "Old topic");
    let (credentials, manager) = runtime(&h);
    let ws = open(&h, &credentials, &manager);

    let lists = ws.list_discussions(&credentials).expect("list discussions");
    assert!(
        lists.active.iter().any(|d| d.slug == "rate-limiting"),
        "active 清單是 server 真值"
    );
    assert!(lists.archived.is_empty(), "尚無 archived 討論");

    let shown = ws
        .discussion_document(&credentials, "rate-limiting")
        .expect("show");
    assert_eq!(shown.info.topic, "Rate limiting");
    assert!(
        shown.content.contains("**Decision**: do it"),
        "討論內文是 server 真值"
    );

    let promoted = ws
        .promote_discussion(&credentials, "rate-limiting", None)
        .expect("promote");
    assert!(!promoted.change.is_empty(), "promote 回新 change 名");
    let changes = ws.list_changes(&credentials).expect("list changes");
    assert!(
        changes.changes.iter().any(|c| c.name == promoted.change),
        "promote 產生的 change 出現在清單"
    );

    let archived = ws
        .archive_discussion(&credentials, "old-topic")
        .expect("archive discussion");
    assert!(
        archived.archived_to.contains("old-topic"),
        "封存路徑回真值：{}",
        archived.archived_to
    );
    let lists = ws.list_discussions(&credentials).expect("list discussions");
    assert!(
        !lists.active.iter().any(|d| d.slug == "old-topic"),
        "封存後離開 active"
    );
    assert!(
        lists.archived.iter().any(|d| d.slug == "old-topic"),
        "出現在 archived"
    );
}

// --- 組合類：set_tasks 中途失敗中止並回報筆數 ---

#[test]
fn composed_set_tasks_aborts_midway_and_reports_the_completed_count() {
    let h = common::harness();
    common::seed_change(h.store.as_ref(), FOUR_TASKS);
    let (credentials, manager) = runtime(&h);
    let ws = open(&h, &credentials, &manager);

    // 「99」不存在：前兩筆成功、第三筆失敗即中止，第四筆不得執行。
    let ids: Vec<String> = ["1", "2", "99", "4"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let failure = ws
        .set_tasks(&credentials, "demo", &ids, true)
        .expect_err("中途失敗即中止");
    assert_eq!(failure.completed, 2, "回報已完成筆數");

    let doc = ws
        .document(&credentials, "demo", "tasks")
        .expect("document");
    assert_eq!(
        doc.content.matches("- [x]").count(),
        2,
        "恰好前兩筆落地：{}",
        doc.content
    );
    assert!(doc.content.contains("- [ ] 1.4 Fourth"), "中止後不再前進");

    // set_all_tasks 走 server 的任務清單組合——把剩餘任務補完。
    let completed = ws
        .set_all_tasks(&credentials, "demo", true)
        .expect("set all");
    assert_eq!(completed, 2, "剩餘兩筆由組合實作補完");
    let doc = ws
        .document(&credentials, "demo", "tasks")
        .expect("document");
    assert_eq!(
        doc.content.matches("- [x]").count(),
        4,
        "全數完成：{}",
        doc.content
    );
}

// --- 不支援類：回拒絕錯誤 ---

#[test]
fn unsupported_operations_are_refused_with_a_zh_tw_message() {
    let err = remote::unsupported("全文搜尋");
    assert_eq!(
        err.reason.as_deref(),
        Some("unsupported"),
        "機讀 reason 固定"
    );
    assert!(
        err.message.contains("全文搜尋"),
        "訊息點名操作：{}",
        err.message
    );
    assert!(
        err.message.contains("尚未提供"),
        "繁中說明缺口來自 server：{}",
        err.message
    );
}

// --- 認領：capability 位依 role（remote-workspace-data「認領操作與認領人呈現」）---

#[test]
fn claim_capability_follows_the_membership_role() {
    let h = common::harness();
    common::seed_change(h.store.as_ref(), FOUR_TASKS);

    let (credentials, manager) = runtime(&h);
    let (_, editor) =
        remote::open_workspace(&h.origin, "demo", &manager, &credentials).expect("editor opens");
    assert!(editor.capabilities.claim, "editor 可認領");

    let invite = h
        .identity
        .create_invitation(NewInvitation {
            email: "reader@example.com".to_string(),
            display: "Reader".to_string(),
            memberships: vec!["demo".to_string()],
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite reader");
    let reader_id = h
        .identity
        .accept_invitation(&invite, "pw-reader")
        .expect("accept");
    h.identity
        .admin_set_membership(
            &speclink_server::audit::AuditActor::system_cli(),
            &reader_id,
            "demo",
            speclink_server::identity::MembershipRole::Reader,
            true,
        )
        .expect("demote to reader");
    let (_, reader_pat) = h.identity.create_pat(&reader_id, "test", None).expect("pat");
    let reader_credentials = MemoryCredentialStore::new();
    reader_credentials
        .set(&h.origin, CredentialKind::Pat, &reader_pat)
        .expect("set reader pat");
    let reader_manager = Arc::new(TokenManager::new(&h.origin));
    let (_, reader) =
        remote::open_workspace(&h.origin, "demo", &reader_manager, &reader_credentials)
            .expect("reader opens");
    assert!(!reader.capabilities.claim, "reader 的認領面停用");
}
