//! 唯讀查詢：桌面 command 的資料來源。
//!
//! 每個函式吃專案 `root`，經內嵌 core 回傳與對應 CLI `--json` 同形狀的 payload。
//! 非 speclink 專案目錄時回傳明確的空狀態（空清單／None），不 panic。

use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use speclink_core::store::Store;

use crate::init_core_context;

/// 對應 `speclink list --json`：`{ "changes": [ … ] }`。非專案回傳 `{ "changes": [] }`。
pub fn list_changes_at(root: &Path) -> Value {
    let Some(ctx) = init_core_context(root) else {
        return json!({ "changes": [] });
    };
    // Worktree 觀察面：與 CLI list 同一組閘門與 payload 落點（design D6）。
    // 政策關、非主 checkout、git 不可用 → facts 為空，讀寫與 payload 與此功能
    // 出現前完全相同。
    let facts = crate::facts_for(&ctx);
    let overlaid = overlay_store(&ctx, &facts);
    let store: &dyn Store = if facts.is_empty() { &ctx.store } else { &overlaid };
    let changes = board_sorted_changes(store);
    // 桌面 payload 在 CLI 同形項上疊加生命週期標記欄位（parity 紅線：CLI 的
    // changes_json 本身不動）。meta 類欄位取自 list_changes 已解析的 meta；
    // whyExcerpt 例外——另讀各 change 的 proposal.md 首段（描述列資料源）。
    let items: Vec<Value> = speclink_core::listing::changes_json_with(
        store,
        &changes,
        &speclink_host::worktree::payload_objects(&facts),
    )
        .iter()
        .zip(changes.iter())
        .map(|(item, c)| {
            let mut v = serde_json::to_value(item).unwrap_or_else(|_| json!({}));
            v["startedAt"] = json!(c.meta.started_at);
            v["startedBy"] = json!(c.meta.started_by);
            v["startedWith"] = json!(c.meta.started_with);
            v["createdBy"] = json!(c.meta.created_by);
            v["created"] = json!(c.meta.created);
            v["fromDiscussions"] = json!(c.meta.from_discussions());
            // 「待重新反映」徽章的資料源：恆存在（空陣列＝無旗標），供看板卡片渲染。
            v["restaleFrom"] = json!(c.meta.restale_from());
            // 變更卡描述列的資料源：恆存在 key（缺件為 null），前端對 null 隱藏描述列。
            v["whyExcerpt"] =
                json!(store.read_artifact(&c.name, "proposal.md").as_deref().and_then(why_excerpt));
            // 審查狀態（spec client-protocol「變更清單的審查狀態欄位」；design D6）：
            // 工單存在 → inReview；否則以 core 失效純函式重算凍結度（有工作樹的
            // client 端）——Fresh → reviewed、Stale → reviewedStale、Unknown（無章）
            // → none。章存在時附 reviewedAt／reviewedBy。
            // 內容指紋錨讀的是 repo 任意程式檔、不是 artifact——overlay 幫不上，
            // 逐 change 解析讀取根（design D2）：有 worktree 映射讀該副本，
            // 否則讀主 checkout。任務錨已走 overlay，兩錨自此同源；解析含
            // 存在性回退（worktree_root_for），與 overlay 的 of() 同步。
            let scope_root = crate::worktree_root_for(&ctx, &facts, &c.name)
                .unwrap_or(ctx.workspace.root.as_path());
            // 兩站的失效判定共用這一個閉包，同一 scope 檔會被問兩次——快取
            // 單次讀取，避免大型 binary 資產在列表這條熱路徑被重複整檔讀入。
            let read_cache =
                std::cell::RefCell::new(std::collections::HashMap::<String, Option<Vec<u8>>>::new());
            let read_file = |p: &str| {
                read_cache
                    .borrow_mut()
                    .entry(p.to_string())
                    .or_insert_with(|| speclink_core::util::read_bytes_opt(&scope_root.join(p)))
                    .clone()
            };
            // 失效判定吃兩組計數（寫碼任務錨）——`[M]` 手測任務的勾選不打黃卡片。
            let counts = speclink_core::tasks::counts_for(store, &c.name);
            // 寫碼進度三欄（spec client-protocol「變更清單的寫碼進度欄位」）：
            // 「待手測」章的資料源，與失效判定共用這一份雙組計數——呈現層不另
            // 行過濾 `[M]`。CLI 的 changes_json 不含這三欄。
            v["codeTotal"] = json!(counts.code_total);
            v["codeComplete"] = json!(counts.code_complete);
            v["codeRemaining"] = json!(counts.code_remaining);
            let review = if store.artifact_exists(&c.name, speclink_core::review::REVIEW_DOC) {
                "inReview"
            } else {
                match speclink_core::review::freshness(&c.meta, &counts, &read_file) {
                    speclink_core::station::Freshness::Fresh => "reviewed",
                    speclink_core::station::Freshness::Stale => "reviewedStale",
                    speclink_core::station::Freshness::Unknown => "none",
                }
            };
            v["reviewStatus"] = json!(review);
            if matches!(review, "reviewed" | "reviewedStale") {
                v["reviewedAt"] = json!(c.meta.reviewed_at);
                v["reviewedBy"] = json!(c.meta.reviewed_by);
            }
            // 驗證狀態（spec client-protocol「變更清單的驗證狀態欄位」；design
            // D5）：與審查狀態同構且互不遮蔽——兩站可各自獨立進行與蓋章，故
            // 各判各的，共用同一份失效純函式與同一個讀取根。
            let verify = if store.artifact_exists(&c.name, speclink_core::verify::VERIFY_DOC) {
                "inVerify"
            } else {
                match speclink_core::verify::freshness(&c.meta, &counts, &read_file) {
                    speclink_core::station::Freshness::Fresh => "verified",
                    speclink_core::station::Freshness::Stale => "verifiedStale",
                    speclink_core::station::Freshness::Unknown => "none",
                }
            };
            v["verifyStatus"] = json!(verify);
            if matches!(verify, "verified" | "verifiedStale") {
                v["verifiedAt"] = json!(c.meta.verified_at);
                v["verifiedBy"] = json!(c.meta.verified_by);
            }
            v
        })
        .collect();
    json!({ "changes": items })
}

/// 批次入口的讀取 store（design D2）：每個有映射的 change，其 artifact 讀取轉向
/// 該 worktree 副本，其餘讀取與全部寫入直通主 store。一次處理全部 change 的入口
/// （看板清單、全文搜尋）共用這一組裝，不逐 change 重建 context——那會對每個
/// change 各 spawn 一次 git，刷新代價過高。
pub(crate) fn overlay_store<'a>(
    ctx: &'a crate::ProjectContext,
    facts: &speclink_host::worktree::WorktreeFacts,
) -> speclink_host::worktree::WorktreeOverlay<'a> {
    speclink_host::worktree::WorktreeOverlay::new(
        &ctx.store,
        facts
            .iter()
            .map(|(name, e)| {
                let store: Box<dyn Store> =
                    Box::new(speclink_fs::FsStore::new(&e.path, &ctx.workspace.spec_dir_name));
                (name.clone(), store)
            })
            .collect(),
    )
}

/// worktree 掛著時，這個 change 的破壞性動詞要擋下來（design D7）。
///
/// 封存或退回會動主 checkout 的 change 目錄，但真正的工作在 worktree 副本裡——
/// 擋下比事後對帳便宜。唯讀面（抽屜、檢視、diff）不經此關。政策關閉、非主
/// checkout 或 git 不可用時無映射，一律放行。
///
/// 守門有意採 facts 原值、不跟隨 `worktree_root_for` 的存在性回退：副本目錄
/// 剛消失的競態窗口內，對破壞性動詞寧可多擋一次，也不放行。
pub(crate) fn refuse_if_worktree_is_open(
    ctx: &crate::ProjectContext,
    change: &str,
) -> Result<(), String> {
    let facts = crate::facts_for(ctx);
    match facts.get(change) {
        None => Ok(()),
        Some(entry) => Err(format!(
            "{change} 正在 worktree（{}）中進行，請先執行 speclink-worktree-merge 收尾再操作。",
            entry.branch
        )),
    }
}

/// 看板要跟著動的所有路徑，第一項恆為 spec 目錄本身（既有監看目標）。
///
/// worktree 流程下，進度寫在別的資料夾：各 worktree 副本的 change 目錄。再加上
/// 主 repo 的 `.git/worktrees/`，worktree 的增減本身也會觸發刷新，卡片標示才會
/// 隨 merge 收尾退場。登記簿要到第一個 worktree 建立才存在，此前以 `.git`
/// 目錄本身代位（監看層對它非遞迴掛載）——等登記簿出生的事件驅動重掛。
/// 非專案或非 git repo 時只回 spec 目錄一項——監看範圍與此功能出現前相同。
pub fn watch_targets_at(root: &Path) -> Vec<PathBuf> {
    let Some(ctx) = init_core_context(root) else {
        return Vec::new();
    };
    let mut targets = vec![ctx.workspace.spec_dir()];
    let facts = crate::facts_for(&ctx);
    for (change, entry) in &facts {
        targets.push(
            entry
                .path
                .join(&ctx.workspace.spec_dir_name)
                .join("changes")
                .join(change),
        );
    }
    let git_dir = ctx.workspace.root.join(".git");
    let registry = git_dir.join("worktrees");
    if registry.is_dir() {
        targets.push(registry);
    } else if git_dir.is_dir() {
        targets.push(git_dir);
    }
    targets
}

/// 看板顯示序的變更清單（design D2）：先取 CLI 預設 modified 序當回退，再以穩定
/// 排序疊上 board_rank 複合鍵——缺值置頂維持回退序、具值依 rank 升冪、同值以
/// 名稱決斷。CLI 的 `speclink list --json` 排序不經此路徑，逐位元不變。
pub(crate) fn board_sorted_changes(store: &dyn Store) -> Vec<speclink_core::model::Change> {
    let mut changes = speclink_core::model::list_changes(store);
    speclink_core::listing::sort_changes(store, &mut changes, "modified");
    changes.sort_by(|x, y| match (&x.meta.board_rank, &y.meta.board_rank) {
        (None, None) => std::cmp::Ordering::Equal, // 穩定排序保留回退序
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (Some(a), Some(b)) => a.cmp(b).then_with(|| x.name.cmp(&y.name)),
    });
    changes
}

/// 對應 `speclink list --specs --json`：`{ "specs": … }`。非專案回傳 `{ "specs": [] }`。
pub fn list_specs_at(root: &Path) -> Value {
    let Some(ctx) = init_core_context(root) else {
        return json!({ "specs": [] });
    };
    let store: &dyn Store = &ctx.store;
    // 桌面 payload 在 CLI 同形項上疊加呈現層輔助欄位（design D2、spec-archive-drawer
    // design D4；parity 紅線：CLI 的 specs_json_items 本身不動）。mtime 不可得時不插
    // key；規格卡欄位恆存在（不可讀／缺席檔案為 0／null，清單照常回傳）。
    let mut specs = speclink_core::listing::specs_json_items(store);
    if let Value::Array(items) = &mut specs {
        for item in items {
            let Some(id) = item["id"].as_str().map(str::to_string) else { continue };
            if let Some(date) = modified_date(&store.canonical_spec_path(&id)) {
                item["modifiedAt"] = json!(date);
            }
            let doc = store.read_canonical_spec(&id);
            let purpose = doc.as_deref().and_then(purpose_excerpt);
            item["requirementCount"] = json!(doc.as_deref().map_or(0, requirement_count));
            // 佔位前綴的單一定義在 speclink-core（design D5）：產生器與偵測共用
            // 同一常數，兩處字串再也不會各自漂移。
            item["purposeTbd"] = json!(purpose
                .as_deref()
                .is_some_and(|p| p.starts_with(speclink_core::model::PURPOSE_TBD_PREFIX)));
            item["purposeExcerpt"] = json!(purpose);
            item["traceCount"] = json!(doc.as_deref().map_or(0, trace_count));
        }
    }
    json!({ "specs": specs })
}

/// 正典 spec 的 `### Requirement:` 標題數。
fn requirement_count(doc: &str) -> usize {
    doc.lines().filter(|l| l.trim().starts_with("### Requirement:")).count()
}

/// 正典 spec `## Purpose` 區段首個非空行原文；區段缺席或無內容時 `None`。
fn purpose_excerpt(doc: &str) -> Option<String> {
    let mut in_purpose = false;
    for line in doc.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("## ") {
            in_purpose = heading.trim() == "Purpose";
            continue;
        }
        if in_purpose && !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

/// 全文 `@trace` 註解區塊內 `source:` 值的去重數（同一變更多次溯源計一次）。
fn trace_count(doc: &str) -> usize {
    let mut sources = std::collections::HashSet::new();
    let mut in_trace = false;
    for line in doc.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<!-- @trace") {
            in_trace = true;
            continue;
        }
        if in_trace {
            if let Some(src) = trimmed.strip_prefix("source:") {
                sources.insert(src.trim().to_string());
            }
            if trimmed.ends_with("-->") {
                in_trace = false;
            }
        }
    }
    sources.len()
}

/// proposal.md 的 `## Why` 區段首個非空行原文（board-card-anatomy design D2）；
/// 區段缺席或區段內無內容時 `None`。
pub(crate) fn why_excerpt(doc: &str) -> Option<String> {
    let mut in_why = false;
    for line in doc.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("## ") {
            in_why = heading.trim() == "Why";
            continue;
        }
        if in_why && !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

/// 檔案 mtime 衍生的本地日期（YYYY-MM-DD）；metadata 或 mtime 不可得時 `None`。
pub(crate) fn modified_date(path: &Path) -> Option<String> {
    let mtime = std::fs::metadata(path).ok()?.modified().ok()?;
    Some(chrono::DateTime::<chrono::Local>::from(mtime).format("%Y-%m-%d").to_string())
}

/// 對應 `speclink status --change <name> --json`（`StatusReport` 序列化）。
/// change 不存在或非專案時回傳 `Err` 附訊息。
pub fn status_at(root: &Path, change: &str) -> Result<Value, String> {
    let ctx = crate::context_for_change(root, change)
        .ok_or_else(|| format!("not a speclink project: {}", root.display()))?;
    let store: &dyn Store = &ctx.store;
    let change = store
        .find_change(change)
        .ok_or_else(|| format!("change not found: {change}"))?;
    let schema = resolve_schema(&ctx.workspace, &change.meta.schema_name())?;
    let report = speclink_core::status::build(store, &change, &schema);
    serde_json::to_value(&report).map_err(|e| e.to_string())
}

/// 讀取一個 change 的 artifact markdown。`artifact` 為 output path（如 `proposal.md`、
/// `design.md`、`specs/<cap>/spec.md`，同 status 的 `outputPath`）。無則 `None`。
///
/// 邊界防護：`change`／`artifact` 含路徑穿越（`..`）或絕對路徑時視為無效回傳 `None`，
/// 不讓拼接後的路徑逃出 change 目錄讀取任意檔案。
pub fn document_at(root: &Path, change: &str, artifact: &str) -> Option<String> {
    if !is_safe_path_param(change) || !is_safe_path_param(artifact) {
        return None;
    }
    let ctx = crate::context_for_change(root, change)?;
    ctx.store.read_artifact(change, artifact)
}

/// 讀取一個 capability 的正典 spec.md。無則 `None`。含路徑穿越的 `capability` 回傳 `None`。
pub fn spec_document_at(root: &Path, capability: &str) -> Option<String> {
    if !is_safe_path_param(capability) {
        return None;
    }
    let ctx = init_core_context(root)?;
    ctx.store.read_canonical_spec(capability)
}

/// 列出一個 change 的 delta capability 名（供規格分頁載入 specs/<cap>/spec.md）。
pub fn change_capabilities_at(root: &Path, change: &str) -> Vec<String> {
    if !is_safe_path_param(change) {
        return Vec::new();
    }
    match crate::context_for_change(root, change) {
        Some(ctx) => ctx.store.delta_capabilities(change),
        None => Vec::new(),
    }
}

/// 讀取一個已封存 change 的 artifact 原文（dated_name 如 `2026-07-04-old`，
/// artifact 為 output path）。缺件或不存在回 `None`；路徑穿越參數一律拒絕。
pub fn archived_document_at(root: &Path, dated_name: &str, artifact: &str) -> Option<String> {
    if !is_safe_path_param(dated_name) || !is_safe_path_param(artifact) {
        return None;
    }
    let ctx = init_core_context(root)?;
    ctx.store.read_archived_artifact(dated_name, artifact)
}

/// 列出一個已封存 change 的 delta capability 名（供唯讀規格分頁載入）。
pub fn archived_capabilities_at(root: &Path, dated_name: &str) -> Vec<String> {
    if !is_safe_path_param(dated_name) {
        return Vec::new();
    }
    match init_core_context(root) {
        Some(ctx) => ctx.store.archived_delta_capabilities(dated_name),
        None => Vec::new(),
    }
}

/// 拒絕會逃出目標目錄的路徑參數：`..` 段、絕對路徑、Windows 磁碟前綴。
pub(crate) fn is_safe_path_param(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    if s.starts_with('/') || s.starts_with('\\') || s.contains(':') {
        return false;
    }
    !s.split(['/', '\\']).any(|seg| seg == "..")
}

fn resolve_schema(
    ws: &speclink_core::workspace::Workspace,
    name: &str,
) -> Result<speclink_core::schema::Schema, String> {
    match speclink_core::schema::resolve_with(Some(ws), Some(&speclink_host::context::global_config_dir()), name) {
        Some(Ok(s)) => Ok(s),
        Some(Err(e)) => Err(e),
        None => Err(speclink_core::schema::not_found_msg(name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testfixture::FixtureRoot;

    const OLD_META: &str = "schema: spec-driven\ncreated: 2026-07-01\ncreated_by: momo\ncreated_with: claude\n";
    const STARTED_META: &str = "schema: spec-driven\ncreated: 2026-07-01\ncreated_by: momo\ncreated_with: claude\nstarted_at: 2026-07-06\nstarted_by: Worker <w@example.com>\nstarted_with: claude\n";

    fn fresh_non_project_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("speclink-query-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn list_changes_shape_matches_cli_and_includes_active_change() {
        let fx = FixtureRoot::new("q-list");
        fx.add_change("demo", OLD_META);
        let v = list_changes_at(fx.root());
        let arr = v["changes"].as_array().expect("changes array");
        assert_eq!(arr.len(), 1);
        let item = &arr[0];
        for key in ["name", "status", "totalTasks", "completedTasks", "summary"] {
            assert!(item.get(key).is_some(), "change item missing camelCase key {key}");
        }
        assert_eq!(item["name"], "demo");
        assert_eq!(item["totalTasks"], 2);
        assert_eq!(item["completedTasks"], 1);
    }

    // --- spec client-protocol「變更清單的寫碼進度欄位」---

    #[test]
    fn list_changes_carries_code_progress_beside_the_full_counts() {
        // Scenario「清單項帶寫碼進度」：九個已勾寫碼任務＋一個未勾 `[M]`。
        let fx = FixtureRoot::new("q-code-progress");
        fx.add_change("demo", OLD_META);
        let mut md = String::from("## 1. Group\n\n");
        for i in 1..=9 {
            md.push_str(&format!("- [x] 1.{i} task\n"));
        }
        md.push_str("- [ ] [M] 手動驗證\n");
        fx.write("openspec/changes/demo/tasks.md", &md);
        let v = list_changes_at(fx.root());
        let item = &v["changes"][0];
        assert_eq!(item["codeTotal"], 9, "{item}");
        assert_eq!(item["codeComplete"], 9, "{item}");
        assert_eq!(item["codeRemaining"], 0, "{item}");
        // 既有欄位不變（`[M]` 仍計入全量）。
        assert_eq!(item["totalTasks"], 10);
        assert_eq!(item["completedTasks"], 9);
    }

    #[test]
    fn code_progress_mirrors_full_counts_without_manual_tasks() {
        let fx = FixtureRoot::new("q-code-progress-plain");
        fx.add_change("demo", OLD_META);
        let item = &list_changes_at(fx.root())["changes"][0];
        assert_eq!(item["codeTotal"], 2, "{item}");
        assert_eq!(item["codeComplete"], 1, "{item}");
        assert_eq!(item["codeRemaining"], 1, "{item}");
    }

    #[test]
    fn list_changes_carries_the_worktree_object_for_a_mapped_change() {
        // spec worktree-overlay「desktop 看板的 worktree 呈現」Scenario「卡片標示
        // 與抽屜資訊」：payload 帶 camelCase 的 path 與 branch（皆為字串）。
        let fx = FixtureRoot::new("q-list-wt");
        fx.add_change("add-auth", OLD_META);
        let wt = fx.attach_worktree("add-auth");

        let v = list_changes_at(fx.root());
        let item = &v["changes"].as_array().expect("changes")[0];

        let worktree = item.get("worktree").expect("mapped change carries a worktree object");
        assert_eq!(worktree["branch"], "speclink/add-auth");
        assert_eq!(worktree["path"], wt.path().to_string_lossy().to_string());
        assert!(worktree["branch"].is_string() && worktree["path"].is_string());
    }

    #[test]
    fn watch_targets_cover_each_worktree_copy_and_the_worktree_registry() {
        // 進度寫在 worktree 副本裡，worktree 的增減寫在 .git/worktrees/——兩者都
        // 要監看，否則看板不會自動更新。
        let fx = FixtureRoot::new("q-watch-wt");
        fx.add_change("add-auth", OLD_META);
        let wt = fx.attach_worktree("add-auth");

        let targets = watch_targets_at(fx.root());

        assert_eq!(targets[0], fx.root().join("openspec"), "第一項恆為 spec 目錄");
        let want_copy = wt.change_dir("add-auth");
        assert!(targets.contains(&want_copy), "須監看 worktree 副本的 change 目錄: {targets:?}");
        let want_registry = fx.root().join(".git").join("worktrees");
        assert!(targets.contains(&want_registry), "須監看 worktree 登記簿: {targets:?}");
        assert!(
            !targets.contains(&fx.root().join(".git")),
            "登記簿已存在時不需 .git 哨兵（避免多吞 git 雜訊）: {targets:?}"
        );
    }

    #[test]
    fn watch_targets_include_the_git_sentinel_before_the_first_worktree() {
        // 登記簿 .git/worktrees/ 要到第一個 worktree 建立才存在，而監看層會濾掉
        // 不存在的目錄——推導須以 .git（非遞迴哨兵）代位，等登記簿出生的事件
        // 驅動重掛，否則第一個 worktree 的新增永遠不會觸發看板刷新。
        let fx = FixtureRoot::new("q-watch-git-sentinel");
        fx.add_change("demo", OLD_META);
        let out = std::process::Command::new("git")
            .current_dir(fx.root())
            .args(["init", "-q", "-b", "main"])
            .output()
            .expect("run git");
        assert!(out.status.success(), "git init: {}", String::from_utf8_lossy(&out.stderr));

        let targets = watch_targets_at(fx.root());

        assert_eq!(targets[0], fx.root().join("openspec"), "第一項恆為 spec 目錄");
        assert!(
            targets.contains(&fx.root().join(".git")),
            "git repo 無 worktree 時須含 .git 哨兵: {targets:?}"
        );
        assert!(
            !targets.contains(&fx.root().join(".git").join("worktrees")),
            "登記簿尚不存在，不推導死路徑: {targets:?}"
        );
    }

    #[test]
    fn watch_targets_are_just_the_spec_dir_without_worktrees() {
        let fx = FixtureRoot::new("q-watch-plain");
        fx.add_change("demo", OLD_META);
        assert_eq!(watch_targets_at(fx.root()), vec![fx.root().join("openspec")]);
    }

    #[test]
    fn watch_targets_are_empty_outside_a_project() {
        assert!(watch_targets_at(&fresh_non_project_dir()).is_empty());
    }

    #[test]
    fn drawer_documents_come_from_the_worktree_copy() {
        // spec worktree-overlay「desktop 看板的 worktree 呈現」：抽屜各分頁的文件
        // 原文與計數同源——計數已走 overlay，原文也必須來自同一份副本。
        let fx = FixtureRoot::new("q-doc-wt");
        fx.add_change("add-auth", OLD_META);
        let wt = fx.attach_worktree("add-auth");
        let wt_change = wt.change_dir("add-auth");
        std::fs::write(wt_change.join("tasks.md"), TASKS_ALL_DONE).unwrap();
        std::fs::write(wt_change.join("design.md"), "## Context\n\nworktree 版設計。\n").unwrap();
        std::fs::create_dir_all(wt_change.join("specs").join("cap-wt")).unwrap();
        std::fs::write(
            wt_change.join("specs").join("cap-wt").join("spec.md"),
            "## ADDED Requirements\n",
        )
        .unwrap();

        let tasks = document_at(fx.root(), "add-auth", "tasks.md").expect("tasks.md 可讀");
        assert!(tasks.contains("- [x] 1.1"), "任務分頁原文須取 worktree 現值: {tasks}");
        assert_eq!(
            document_at(fx.root(), "add-auth", "design.md").as_deref(),
            Some("## Context\n\nworktree 版設計。\n"),
            "只存在於 worktree 的文件也要讀得到"
        );
        assert_eq!(
            change_capabilities_at(fx.root(), "add-auth"),
            vec!["cap-wt"],
            "規格分頁清單同源 worktree"
        );
    }

    #[test]
    fn worktree_context_is_built_directly_without_discovery() {
        // 審查 finding（Round 1）：定根不得走 discovery——向上探索在極端情境會走出
        // worktree、落到祖先目錄的別的專案；副本的 .speclink.yaml 損壞時也不得靜默
        // 回讀主 checkout。映射條件只看「change 目錄可讀」，成立就以 worktree 路徑
        // ＋主 checkout 的 spec 目錄名直接建構（與 overlay 的組裝同款）。
        let fx = FixtureRoot::new("q-wt-direct");
        fx.add_change("add-auth", OLD_META);
        let wt = fx.attach_worktree("add-auth");
        std::fs::write(wt.path().join(".speclink.yaml"), ": : broken [yaml\n").unwrap();
        std::fs::write(
            wt.change_dir("add-auth").join("tasks.md"),
            TASKS_ALL_DONE,
        )
        .unwrap();

        let doc = document_at(fx.root(), "add-auth", "tasks.md").expect("tasks.md 可讀");
        assert!(doc.contains("- [x] 1.1"), "直接組裝、不受副本 app config 影響: {doc}");
    }

    #[test]
    fn status_report_comes_from_the_worktree_copy() {
        // 同一需求的狀態報告面：design.md 只存在於 worktree 副本時，該 artifact
        // 的狀態為 done（讀主 checkout 會停在未完成）。
        let fx = FixtureRoot::new("q-status-wt");
        fx.add_change("add-auth", OLD_META);
        let wt = fx.attach_worktree("add-auth");
        std::fs::write(
            wt.change_dir("add-auth").join("design.md"),
            "## Context\n\nworktree 版設計。\n",
        )
        .unwrap();

        let report = status_at(fx.root(), "add-auth").expect("status ok");
        let design = report["artifacts"]
            .as_array()
            .expect("artifacts array")
            .iter()
            .find(|a| a["id"] == "design")
            .expect("design artifact listed")
            .clone();
        assert_eq!(design["status"], "done", "狀態報告須反映 worktree 副本: {report}");
        assert!(
            !fx.root().join("openspec/changes/add-auth/design.md").exists(),
            "主 checkout 確實沒有這份文件（測試前提）"
        );
    }

    #[test]
    fn every_read_entry_falls_back_to_the_main_checkout_when_the_policy_is_off() {
        // design D4 紅線：三道閘門任一不成立時 facts 為空——這裡關掉 worktree 政策，
        // worktree 副本明明有更新的內容，全部讀取面仍須讀主 checkout（＝此功能出現
        // 前的行為）。
        let fx = FixtureRoot::new("q-policy-off");
        fx.add_change("add-auth", OLD_META);
        let wt = fx.attach_worktree("add-auth");
        fx.write("openspec/config.yaml", "schema: spec-driven\nworktree: false\n");
        let wt_change = wt.change_dir("add-auth");
        std::fs::write(wt_change.join("tasks.md"), TASKS_ALL_DONE).unwrap();
        std::fs::write(wt_change.join("design.md"), "## Context\n\n只在 worktree。\n").unwrap();

        let v = list_changes_at(fx.root());
        let item = &v["changes"].as_array().expect("changes")[0];
        assert_eq!(item["completedTasks"], 1, "計數回到主 checkout: {item}");
        assert!(item.get("worktree").is_none_or(Value::is_null), "政策關則無標示: {item}");
        assert!(
            document_at(fx.root(), "add-auth", "tasks.md").unwrap().contains("- [ ] 1.1"),
            "文件回到主 checkout"
        );
        assert!(document_at(fx.root(), "add-auth", "design.md").is_none(), "副本的新檔不外露");
        let report = status_at(fx.root(), "add-auth").expect("status ok");
        let design = report["artifacts"]
            .as_array()
            .expect("artifacts")
            .iter()
            .find(|a| a["id"] == "design")
            .expect("design listed")
            .clone();
        assert_ne!(design["status"], "done", "狀態報告回到主 checkout: {report}");
        assert!(
            crate::search::search_workspace_at(fx.root(), "只在 worktree")["hits"]
                .as_array()
                .expect("hits")
                .is_empty(),
            "搜尋不掃副本"
        );
    }

    #[test]
    fn reads_fall_back_to_the_main_checkout_once_the_worktree_is_removed() {
        // design D4：facts 每次現取、不快取——worktree 移除後的下一次呼叫即回讀
        // 主 checkout，沒有 stale 視窗。
        let fx = FixtureRoot::new("q-wt-removed");
        fx.add_change("add-auth", OLD_META);
        let wt = fx.attach_worktree("add-auth");
        std::fs::write(
            wt.change_dir("add-auth").join("tasks.md"),
            TASKS_ALL_DONE,
        )
        .unwrap();
        assert!(
            document_at(fx.root(), "add-auth", "tasks.md").unwrap().contains("- [x] 1.1"),
            "移除前讀副本"
        );

        let out = std::process::Command::new("git")
            .args(["worktree", "remove", "--force", "wt"])
            .current_dir(fx.root())
            .output()
            .expect("run git");
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));

        assert!(
            document_at(fx.root(), "add-auth", "tasks.md").unwrap().contains("- [ ] 1.1"),
            "移除後的下一次呼叫即回讀主 checkout"
        );
    }

    #[test]
    fn list_changes_omits_the_worktree_object_without_a_mapping() {
        let fx = FixtureRoot::new("q-list-no-wt");
        fx.add_change("demo", OLD_META);
        let v = list_changes_at(fx.root());
        let item = &v["changes"].as_array().expect("changes")[0];
        assert!(item.get("worktree").is_none_or(Value::is_null), "無映射時不帶 worktree: {item}");
    }

    #[test]
    fn list_changes_counts_tasks_from_the_worktree_copy() {
        // Scenario「worktree 內進度即時反映」：勾選發生在 worktree 副本，主看板
        // 的計數要跟著動（讀經 overlay，而非主 checkout 的舊值）。
        let fx = FixtureRoot::new("q-list-wt-progress");
        fx.add_change("add-auth", OLD_META);
        let wt = fx.attach_worktree("add-auth");
        let tasks = wt.change_dir("add-auth").join("tasks.md");
        std::fs::write(&tasks, "## 1. Group\n\n- [x] 1.1 First task\n- [x] 1.2 Second task\n")
            .unwrap();

        let v = list_changes_at(fx.root());
        let item = &v["changes"].as_array().expect("changes")[0];

        assert_eq!(item["completedTasks"], 2, "計數須來自 worktree 副本: {item}");
        assert_eq!(item["totalTasks"], 2);
    }

    #[test]
    fn list_changes_overlays_lifecycle_marker_fields() {
        // D2：桌面 payload 在 CLI 同形欄位之上疊加 startedAt/startedBy/startedWith
        // （camelCase；未開工為 null）。資料來自已解析 meta，CLI 輸出不受影響。
        let fx = FixtureRoot::new("q-overlay");
        fx.add_change("not-started", OLD_META);
        fx.add_change("underway", STARTED_META);
        let v = list_changes_at(fx.root());
        let arr = v["changes"].as_array().expect("changes array");
        assert_eq!(arr.len(), 2);
        for item in arr {
            for key in ["startedAt", "startedBy", "startedWith"] {
                assert!(item.get(key).is_some(), "missing overlay key {key} on {item}");
            }
        }
        let by_name = |name: &str| arr.iter().find(|c| c["name"] == name).unwrap().clone();
        let idle = by_name("not-started");
        assert!(idle["startedAt"].is_null());
        assert!(idle["startedBy"].is_null());
        assert!(idle["startedWith"].is_null());
        let underway = by_name("underway");
        assert_eq!(underway["startedAt"], "2026-07-06");
        assert_eq!(underway["startedBy"], "Worker <w@example.com>");
        assert_eq!(underway["startedWith"], "claude");
        // createdBy（camelCase）由 meta 蓋章曝露、snake_case 不外洩（審計：邊界無型別混淆）。
        assert_eq!(underway["createdBy"], "momo");
        assert!(underway.get("created_by").is_none(), "camelCase only");
        // 既有欄位形狀不變。
        assert_eq!(underway["totalTasks"], 2);
        assert_eq!(underway["completedTasks"], 1);
    }

    #[test]
    fn list_changes_overlays_created_date() {
        // 建立時間篩選的資料源（desktop-ux-polish design D5）：created（YYYY-MM-DD）
        // 由已解析 meta 曝露；CLI 輸出不經此路徑、逐位元不變。
        let fx = FixtureRoot::new("q-created");
        fx.add_change("demo", OLD_META);
        let v = list_changes_at(fx.root());
        let arr = v["changes"].as_array().expect("changes array");
        assert_eq!(arr[0]["created"], "2026-07-01");
    }

    #[test]
    fn list_changes_overlays_from_discussions_as_array() {
        // 同源連結（多值化）：來自討論的 change 於清單項帶 fromDiscussions 陣列，
        // 依 meta 順序；多來源全列、單來源一元、非討論而來為空陣列。舊單值鍵 fromDiscussion 不再出現。
        let fx = FixtureRoot::new("q-fromdisc");
        fx.add_change(
            "multi",
            "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: alpha-search, beta-cache\n",
        );
        fx.add_change(
            "single",
            "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: alpha-search\n",
        );
        fx.add_change("plain", OLD_META);
        let v = list_changes_at(fx.root());
        let arr = v["changes"].as_array().expect("changes array");
        let by_name = |name: &str| arr.iter().find(|c| c["name"] == name).unwrap().clone();
        assert_eq!(
            by_name("multi")["fromDiscussions"],
            serde_json::json!(["alpha-search", "beta-cache"])
        );
        assert_eq!(by_name("single")["fromDiscussions"], serde_json::json!(["alpha-search"]));
        assert_eq!(by_name("plain")["fromDiscussions"], serde_json::json!([]));
        assert!(by_name("multi").get("fromDiscussion").is_none(), "old single-value key gone");
    }

    const TICKET: &str = "# Review — x\n\n## Round 1\n\n**Scope**: src/lib.rs\n\n- [WARNING] src/lib.rs — possible smell\n";
    const TASKS_ALL_DONE: &str = "## 1. Group\n\n- [x] 1.1 First task\n- [x] 1.2 Second task\n";

    fn reviewed_meta(scope_path: &str, hash: &str) -> String {
        format!(
            "schema: spec-driven\ncreated: 2026-07-01\nreviewed_at: 2026-08-01\nreviewed_by: Rev <r@example.com>\nreviewed_with: claude\nreviewed_tasks_total: 2\nreviewed_scope:\n  - path: {scope_path}\n    hash: {hash}\n"
        )
    }

    #[test]
    fn list_changes_overlays_review_status_four_states() {
        // spec client-protocol「變更清單的審查狀態欄位」：工單存在 → inReview；
        // 章在且雙錨符 → reviewed；章在錨不符 → reviewedStale；皆無 → none。
        // 章存在時附 reviewedAt／reviewedBy；凍結度於有工作樹的 client 端重算。
        let fx = FixtureRoot::new("q-review");
        fx.add_change("plain", OLD_META);
        fx.add_change("underway", OLD_META);
        fx.write("openspec/changes/underway/review.md", TICKET);
        // reviewed：任務錨（2/2 全完成、蓋章總數 2）與內容錨（現值指紋相符）皆符。
        let content = "fn keep() {}\n";
        fx.write("src/lib.rs", content);
        let hash = speclink_core::review::content_fingerprint(content);
        fx.add_change("stamped", &reviewed_meta("src/lib.rs", &hash));
        fx.write("openspec/changes/stamped/tasks.md", TASKS_ALL_DONE);
        // reviewedStale（spec Example「章在但指紋不符」）：scope 檔其後追加了一行。
        let old_hash = speclink_core::review::content_fingerprint("fn keep() {}\n");
        fx.write("src/stale.rs", "fn keep() {}\nfn extra() {}\n");
        fx.add_change("gone-stale", &reviewed_meta("src/stale.rs", &old_hash));
        fx.write("openspec/changes/gone-stale/tasks.md", TASKS_ALL_DONE);

        let v = list_changes_at(fx.root());
        let arr = v["changes"].as_array().expect("changes array");
        let by_name = |name: &str| arr.iter().find(|c| c["name"] == name).unwrap().clone();

        assert_eq!(by_name("plain")["reviewStatus"], "none");
        assert!(by_name("plain").get("reviewedAt").is_none(), "no anchors without a stamp");
        assert_eq!(by_name("underway")["reviewStatus"], "inReview");
        assert!(by_name("underway").get("reviewedAt").is_none(), "in-review carries no stamp");
        let stamped = by_name("stamped");
        assert_eq!(stamped["reviewStatus"], "reviewed");
        assert_eq!(stamped["reviewedAt"], "2026-08-01");
        assert_eq!(stamped["reviewedBy"], "Rev <r@example.com>");
        let stale = by_name("gone-stale");
        assert_eq!(stale["reviewStatus"], "reviewedStale");
        assert_eq!(stale["reviewedAt"], "2026-08-01", "anchors survive the downgrade");
        // camelCase 邊界：snake_case 不外洩；既有 CLI 形狀欄位不動。
        assert!(stamped.get("review_status").is_none() && stamped.get("reviewed_at").is_none());
        for key in ["name", "status", "totalTasks", "completedTasks", "summary"] {
            assert!(stamped.get(key).is_some(), "existing key {key} intact");
        }
    }

    #[test]
    fn review_freshness_handles_non_utf8_scope_bytes() {
        // design 契約「desktop 的 reviewStatus 重算同語意」：非 UTF-8 scope 檔
        // 位元組未變 → reviewed、變動 → reviewedStale。改動前這類檔讀不成文字、
        // 章恆判 stale——指紋分流後行為翻轉，這裡把翻轉後的語意釘住。
        let fx = FixtureRoot::new("q-review-binary");
        let bin: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0xFF, 0x00];
        std::fs::write(fx.root().join("logo.png"), bin).unwrap();
        let hash = speclink_core::review::content_fingerprint_bytes(bin);
        fx.add_change("bin-scope", &reviewed_meta("logo.png", &hash));
        fx.write("openspec/changes/bin-scope/tasks.md", TASKS_ALL_DONE);

        let v = list_changes_at(fx.root());
        assert_eq!(v["changes"][0]["reviewStatus"], "reviewed", "位元組未變 → reviewed: {v}");

        std::fs::write(fx.root().join("logo.png"), [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0xFF, 0x01])
            .unwrap();
        let v = list_changes_at(fx.root());
        assert_eq!(v["changes"][0]["reviewStatus"], "reviewedStale", "位元組變動 → stale: {v}");
    }

    /// 帶完整驗證章的 meta（任務錨 2、內容錨 scope_path＋hash）。
    fn verified_meta(scope_path: &str, hash: &str) -> String {
        format!(
            "schema: spec-driven\ncreated: 2026-07-01\nverified_at: 2026-08-02\nverified_by: Ver <v@example.com>\nverified_with: claude\nverified_tasks_total: 2\nverified_scope:\n  - path: {scope_path}\n    hash: {hash}\n"
        )
    }

    #[test]
    fn list_changes_overlays_verify_status_four_states() {
        // spec client-protocol「變更清單的驗證狀態欄位」：verify.md 存在 →
        // inVerify；章在且雙錨符 → verified；章在錨不符 → verifiedStale；
        // 皆無 → none。章存在時附 verifiedAt／verifiedBy。
        let fx = FixtureRoot::new("q-verify");
        fx.add_change("plain", OLD_META);
        fx.add_change("underway", OLD_META);
        fx.write("openspec/changes/underway/verify.md", TICKET);
        let content = "fn keep() {}\n";
        fx.write("src/lib.rs", content);
        let hash = speclink_core::verify::content_fingerprint(content);
        fx.add_change("stamped", &verified_meta("src/lib.rs", &hash));
        fx.write("openspec/changes/stamped/tasks.md", TASKS_ALL_DONE);
        let old_hash = speclink_core::verify::content_fingerprint("fn keep() {}\n");
        fx.write("src/stale.rs", "fn keep() {}\nfn extra() {}\n");
        fx.add_change("gone-stale", &verified_meta("src/stale.rs", &old_hash));
        fx.write("openspec/changes/gone-stale/tasks.md", TASKS_ALL_DONE);

        let v = list_changes_at(fx.root());
        let arr = v["changes"].as_array().expect("changes array");
        let by_name = |name: &str| arr.iter().find(|c| c["name"] == name).unwrap().clone();

        assert_eq!(by_name("plain")["verifyStatus"], "none");
        assert!(by_name("plain").get("verifiedAt").is_none(), "no anchors without a stamp");
        assert_eq!(by_name("underway")["verifyStatus"], "inVerify");
        assert!(by_name("underway").get("verifiedAt").is_none(), "in-verify carries no stamp");
        let stamped = by_name("stamped");
        assert_eq!(stamped["verifyStatus"], "verified");
        assert_eq!(stamped["verifiedAt"], "2026-08-02");
        assert_eq!(stamped["verifiedBy"], "Ver <v@example.com>");
        let stale = by_name("gone-stale");
        assert_eq!(stale["verifyStatus"], "verifiedStale");
        assert_eq!(stale["verifiedAt"], "2026-08-02", "anchors survive the downgrade");
        assert!(stamped.get("verify_status").is_none() && stamped.get("verified_at").is_none());
    }

    #[test]
    fn the_two_stations_are_judged_independently() {
        // 討論 code-review-stage 定案：兩站互不遮蔽。審查章＋未結驗證工單並存
        // 時，reviewStatus 仍是 reviewed、verifyStatus 是 inVerify——任一站的
        // 狀態不得被另一站蓋掉。
        let fx = FixtureRoot::new("q-both");
        let content = "fn keep() {}\n";
        fx.write("src/lib.rs", content);
        let hash = speclink_core::review::content_fingerprint(content);
        fx.add_change("mixed", &reviewed_meta("src/lib.rs", &hash));
        fx.write("openspec/changes/mixed/tasks.md", TASKS_ALL_DONE);
        fx.write("openspec/changes/mixed/verify.md", TICKET);

        let v = list_changes_at(fx.root());
        let item = v["changes"].as_array().unwrap()[0].clone();
        assert_eq!(item["reviewStatus"], "reviewed", "{item}");
        assert_eq!(item["verifyStatus"], "inVerify", "{item}");
        assert_eq!(item["reviewedBy"], "Rev <r@example.com>");
        assert!(item.get("verifiedAt").is_none(), "an open verify ticket carries no stamp");
    }

    /// 蓋章時 scope 檔的內容——指紋錨記的就是它的雜湊。
    const STAMPED_SCOPE: &str = "fn keep() {}\n";

    /// 蓋章 change ＋ worktree 映射：主 checkout 與 worktree 副本的 scope 檔內容
    /// 分別指定，兩份相異才看得出凍結度到底讀了哪一份。
    fn stamped_change_with_worktree(tag: &str, main: &str, worktree: &str) -> FixtureRoot {
        let fx = FixtureRoot::new(tag);
        let hash = speclink_core::review::content_fingerprint(STAMPED_SCOPE);
        fx.write("src/auth.rs", main);
        fx.add_change("fix-auth", &reviewed_meta("src/auth.rs", &hash));
        fx.write("openspec/changes/fix-auth/tasks.md", TASKS_ALL_DONE);
        let wt = fx.attach_worktree("fix-auth");
        std::fs::write(wt.path().join("src").join("auth.rs"), worktree).unwrap();
        fx
    }

    #[test]
    fn review_freshness_reads_the_scope_file_from_the_worktree_copy() {
        // spec client-protocol Scenario「worktree 中蓋章的凍結度以 worktree 現值
        // 判定」＋ Example 第一列：worktree 副本與蓋章時一致、主 checkout 仍是
        // 未實作的舊內容 → reviewed。
        let fx = stamped_change_with_worktree(
            "q-review-wt-fresh",
            "fn unimplemented_yet() {}\n",
            STAMPED_SCOPE,
        );
        let v = list_changes_at(fx.root());
        assert_eq!(v["changes"][0]["reviewStatus"], "reviewed", "指紋錨須讀 worktree 現值: {v}");
        assert_eq!(v["changes"][0]["reviewedAt"], "2026-08-01", "章的欄位照常附帶");
    }

    #[test]
    fn review_freshness_turns_stale_when_the_worktree_copy_drifts() {
        // 同 Example 第二列：worktree 內於蓋章後再改該檔 → reviewedStale。主
        // checkout 這次反而相符——讀錯一份就會停在 reviewed。
        let fx = stamped_change_with_worktree(
            "q-review-wt-stale",
            STAMPED_SCOPE,
            "fn keep() {}\nfn drifted() {}\n",
        );
        let v = list_changes_at(fx.root());
        assert_eq!(
            v["changes"][0]["reviewStatus"], "reviewedStale",
            "worktree 內變動後須轉 stale: {v}"
        );
    }

    #[test]
    fn list_changes_overlays_why_excerpt_from_proposal() {
        // 變更卡描述列資料源（board-card-anatomy design D2）：whyExcerpt 為 proposal.md
        // `## Why` 區段首個非空行原文（camelCase、恆存在 key）；Why 前有其他區段不影響。
        let fx = FixtureRoot::new("q-why");
        fx.add_change("demo", OLD_META);
        fx.add_change("prefixed", OLD_META);
        fx.write(
            "openspec/changes/prefixed/proposal.md",
            "## Summary\n\nOne liner.\n\n## Why\n\nReal reason here.\nSecond line ignored.\n",
        );
        let v = list_changes_at(fx.root());
        let arr = v["changes"].as_array().expect("changes array");
        let by_name = |name: &str| arr.iter().find(|c| c["name"] == name).unwrap().clone();
        assert_eq!(by_name("demo")["whyExcerpt"], "Demo change.");
        assert_eq!(by_name("prefixed")["whyExcerpt"], "Real reason here.");
        assert!(by_name("demo").get("why_excerpt").is_none(), "camelCase only");
    }

    #[test]
    fn why_excerpt_is_null_when_proposal_or_why_missing() {
        // 缺件容錯（board-card-anatomy design D2）：無 proposal.md、Why 區段缺席或
        // 區段為空時 whyExcerpt 為 null，清單照常回傳（描述列由前端缺席處理）。
        let fx = FixtureRoot::new("q-why-missing");
        fx.write("openspec/changes/no-proposal/.openspec.yaml", OLD_META);
        fx.write("openspec/changes/no-proposal/tasks.md", "- [ ] 1.1 t\n");
        fx.add_change("empty-why", OLD_META);
        fx.write(
            "openspec/changes/empty-why/proposal.md",
            "## Why\n\n## What Changes\n\n- something\n",
        );
        let v = list_changes_at(fx.root());
        let arr = v["changes"].as_array().expect("changes array");
        let by_name = |name: &str| arr.iter().find(|c| c["name"] == name).unwrap().clone();
        assert!(by_name("no-proposal")["whyExcerpt"].is_null(), "no proposal.md → null");
        assert!(by_name("empty-why")["whyExcerpt"].is_null(), "empty Why section → null");
    }

    #[test]
    fn list_changes_overlays_restale_from_as_array() {
        // 「待重新反映」徽章資料源：restale_from 非空的 change 於清單項帶 restaleFrom 陣列
        // （依 meta 順序），無旗標為空陣列。
        let fx = FixtureRoot::new("q-restale");
        fx.add_change(
            "stale",
            "schema: spec-driven\ncreated: 2026-07-01\nrestale_from: alpha, beta\n",
        );
        fx.add_change("fresh", OLD_META);
        let v = list_changes_at(fx.root());
        let arr = v["changes"].as_array().expect("changes array");
        let by_name = |name: &str| arr.iter().find(|c| c["name"] == name).unwrap().clone();
        assert_eq!(by_name("stale")["restaleFrom"], serde_json::json!(["alpha", "beta"]));
        assert_eq!(by_name("fresh")["restaleFrom"], serde_json::json!([]));
    }

    #[test]
    fn list_changes_marks_invalid_card_and_keeps_the_board_open() {
        // spec「看板照常開啟並標記損壞卡」：壞 metadata 卡帶 metaError 診斷、
        // 有效卡不帶，看板照常列出全部卡片。
        let fx = FixtureRoot::new("q-invalid-meta");
        fx.add_change("good", OLD_META);
        fx.add_change("broken", ": : :\n\t bad yaml [unclosed\n");
        let v = list_changes_at(fx.root());
        let arr = v["changes"].as_array().expect("changes array");
        assert_eq!(arr.len(), 2, "one corrupt card must not break the board");
        let by_name = |name: &str| arr.iter().find(|c| c["name"] == name).unwrap().clone();
        let broken = by_name("broken");
        assert!(broken["metaError"].is_string(), "invalid card carries the diagnostic: {broken}");
        assert!(!broken["metaError"].as_str().unwrap().is_empty());
        assert!(by_name("good").get("metaError").is_none(), "valid card carries no diagnostic");
    }

    #[test]
    fn list_changes_sorts_by_board_rank_with_unranked_on_top() {
        // spec「看板卡片順序以 board_rank 欄位為真相」＋ design D2：缺值卡置頂
        // 維持回退序（mtime 平手時名稱升冪），具值卡依 rank 字典序升冪殿後。
        let fx = FixtureRoot::new("q-rank-sort");
        fx.add_change("ranked-n", &format!("{OLD_META}board_rank: n\n"));
        fx.add_change("ranked-b", &format!("{OLD_META}board_rank: b\n"));
        fx.add_change("unranked-y", OLD_META);
        fx.add_change("unranked-x", OLD_META);
        let v = list_changes_at(fx.root());
        let names: Vec<String> = v["changes"]
            .as_array()
            .expect("changes array")
            .iter()
            .map(|c| c["name"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(
            names,
            ["unranked-x", "unranked-y", "ranked-b", "ranked-n"],
            "unranked on top (fallback order), ranked ascending by key"
        );
    }

    #[test]
    fn list_changes_breaks_equal_rank_ties_by_name() {
        // design D2 同值決斷：rank 相同以名稱字典序，跨機器確定。
        let fx = FixtureRoot::new("q-rank-tie");
        fx.add_change("beta", &format!("{OLD_META}board_rank: n\n"));
        fx.add_change("alpha", &format!("{OLD_META}board_rank: n\n"));
        let v = list_changes_at(fx.root());
        let names: Vec<String> = v["changes"]
            .as_array()
            .expect("changes array")
            .iter()
            .map(|c| c["name"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(names, ["alpha", "beta"]);
    }

    #[test]
    fn list_specs_shape_matches_cli() {
        let fx = FixtureRoot::new("q-specs");
        fx.write("openspec/specs/cap-x/spec.md", "# cap-x Specification\n\n## Requirements\n");
        let v = list_specs_at(fx.root());
        let arr = v["specs"].as_array().expect("specs array");
        let ids: Vec<&str> = arr.iter().filter_map(|s| s["id"].as_str()).collect();
        assert!(ids.contains(&"cap-x"), "canonical spec present: {ids:?}");
    }

    #[test]
    fn list_specs_includes_card_info_fields() {
        // 規格卡收合資訊資料源（spec-archive-drawer design D4）：requirementCount 為
        // `### Requirement:` 標題數、purposeExcerpt 為 Purpose 區段首個非空行原文、
        // traceCount 為全文 @trace 標記的 source 去重數；欄位 camelCase。
        let fx = FixtureRoot::new("q-spec-card");
        fx.write(
            "openspec/specs/cap-x/spec.md",
            concat!(
                "# cap-x Specification\n\n",
                "## Purpose\n\nSearch behavior for the desktop app.\nSecond line ignored.\n\n",
                "## Requirements\n\n",
                "### Requirement: Alpha\n\nIt SHALL alpha.\n\n",
                "<!-- @trace\nsource: change-one\nupdated: 2026-07-01\ncode:\n  - a.rs\n-->\n\n---\n",
                "### Requirement: Beta\n\nIt SHALL beta.\n\n",
                "<!-- @trace\nsource: change-two\nupdated: 2026-07-02\ncode:\n  - b.rs\n-->\n\n---\n",
                "### Requirement: Gamma\n\nIt SHALL gamma.\n\n",
                "<!-- @trace\nsource: change-one\nupdated: 2026-07-03\ncode:\n  - c.rs\n-->",
            ),
        );
        let v = list_specs_at(fx.root());
        let arr = v["specs"].as_array().expect("specs array");
        let item = arr.iter().find(|s| s["id"] == "cap-x").expect("cap-x listed");
        assert_eq!(item["requirementCount"], 3);
        assert_eq!(item["purposeExcerpt"], "Search behavior for the desktop app.");
        assert_eq!(item["purposeTbd"], false);
        assert_eq!(item["traceCount"], 2, "source 去重：change-one 出現兩次計一次");
        for key in ["requirement_count", "purpose_excerpt", "purpose_tbd", "trace_count"] {
            assert!(item.get(key).is_none(), "camelCase only: {key}");
        }
    }

    #[test]
    fn list_specs_purpose_tbd_flags_archive_placeholder() {
        // 佔位偵測釘的是 speclink-core 的公開常數（design D5）：產生器與偵測共用
        // 同一份字串，所以殘留佔位的正典規格必被標為 purposeTbd。核心常數若變動，
        // 這份 fixture 隨之變動，偵測不需要跟著改。
        let fx = FixtureRoot::new("q-spec-tbd");
        fx.write(
            "openspec/specs/cap-new/spec.md",
            &format!(
                "# cap-new Specification\n\n## Purpose\n\n{} change 'demo'. Update Purpose after archive.\n\n## Requirements\n\n### Requirement: Fresh works\n\nIt SHALL work.\n",
                speclink_core::model::PURPOSE_TBD_PREFIX
            ),
        );
        let v = list_specs_at(fx.root());
        let arr = v["specs"].as_array().expect("specs array");
        let item = arr.iter().find(|s| s["id"] == "cap-new").expect("cap-new listed");
        assert_eq!(item["purposeTbd"], true);
        assert_eq!(item["requirementCount"], 1);
    }

    #[test]
    fn spec_card_fields_tolerate_minimal_content() {
        // 容錯（Implementation Contract 失敗模式）：無 Purpose 區段、無 Requirement、
        // 無 @trace 的規格計數欄位為 0、excerpt 為 null，清單照常回傳。
        let fx = FixtureRoot::new("q-spec-card-bare");
        fx.write("openspec/specs/bare/spec.md", "# bare Specification\n");
        let v = list_specs_at(fx.root());
        let arr = v["specs"].as_array().expect("specs array");
        let item = arr.iter().find(|s| s["id"] == "bare").expect("bare listed");
        assert_eq!(item["requirementCount"], 0);
        assert!(item["purposeExcerpt"].is_null());
        assert_eq!(item["purposeTbd"], false);
        assert_eq!(item["traceCount"], 0);
    }

    #[cfg(unix)]
    #[test]
    fn spec_card_fields_tolerate_unreadable_file() {
        // 不可讀檔案（列出後權限被拒）：該筆欄位 0／null，清單照常回傳且其他規格不受影響，
        // 不因單筆壞檔讓整頁失敗。
        use std::os::unix::fs::PermissionsExt;
        let fx = FixtureRoot::new("q-spec-card-unreadable");
        fx.write("openspec/specs/ok/spec.md", "## Purpose\n\nFine.\n\n### Requirement: A\n\nx\n");
        fx.write("openspec/specs/locked/spec.md", "## Purpose\n\nHidden.\n");
        let locked = fx.root().join("openspec/specs/locked/spec.md");
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();
        let v = list_specs_at(fx.root());
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o644)).unwrap();
        let arr = v["specs"].as_array().expect("specs array");
        let bad = arr.iter().find(|s| s["id"] == "locked").expect("locked still listed");
        assert_eq!(bad["requirementCount"], 0);
        assert!(bad["purposeExcerpt"].is_null());
        assert_eq!(bad["purposeTbd"], false);
        assert_eq!(bad["traceCount"], 0);
        let good = arr.iter().find(|s| s["id"] == "ok").expect("ok listed");
        assert_eq!(good["requirementCount"], 1);
        assert_eq!(good["purposeExcerpt"], "Fine.");
    }

    #[test]
    fn list_specs_includes_modified_at_from_mtime() {
        // 呈現層輔助欄位（design D2）：規格清單查詢對每個 spec 帶 modifiedAt——
        // spec.md 檔案系統 mtime 的本地日期（YYYY-MM-DD）。剛寫入的檔案即今天。
        let fx = FixtureRoot::new("q-specs-mtime");
        fx.write("openspec/specs/cap-x/spec.md", "# cap-x Specification\n");
        let v = list_specs_at(fx.root());
        let arr = v["specs"].as_array().expect("specs array");
        let item = arr.iter().find(|s| s["id"] == "cap-x").expect("cap-x listed");
        assert_eq!(item["modifiedAt"], speclink_core::util::today().as_str());
    }

    #[test]
    fn modified_date_is_absent_when_mtime_unavailable() {
        // mtime 不可得時欄位缺席（衍生為 None、overlay 不插 key，而非塞 null）。
        let fx = FixtureRoot::new("q-specs-no-mtime");
        let ghost = fx.root().join("openspec/specs/ghost/spec.md");
        assert!(modified_date(&ghost).is_none());
    }

    #[test]
    fn status_shape_matches_cli() {
        let fx = FixtureRoot::new("q-status");
        fx.add_change("demo", OLD_META);
        let v = status_at(fx.root(), "demo").expect("status ok");
        for key in ["changeName", "schemaName", "isComplete", "applyRequires", "artifacts"] {
            assert!(v.get(key).is_some(), "status missing camelCase key {key}");
        }
        assert_eq!(v["changeName"], "demo");
    }

    #[test]
    fn evidence_record_never_shows_up_as_a_change_artifact() {
        // design Risk「.evidence.json 意外出現在桌面 artifact 清單」：證據記錄是
        // 機器寫入的 dot 檔，與 `.openspec.yaml` 同待遇——不進 artifact 清單、
        // 不進規格分頁的 capability 清單、不成為搜尋命中的 artifact 名。
        // 三個面在固定 schema 清單下對 dot 檔永真，所以再放一個非 dot 的 stray
        // 檔當判別器：哪天清單改成掃目錄，stray 檔會現形、此測試才會真的失敗。
        let fx = FixtureRoot::new("q-evidence-hidden");
        fx.add_change("demo", OLD_META);
        fx.write(
            "openspec/changes/demo/.evidence.json",
            r#"{"version":2,"change":"demo","entries":[{"taskId":"1","taskDesc":"1.1 a","touchedFiles":["src/searchable-token.rs"],"basisDigests":{"spec":"sha256:0","tasks":"sha256:0","policy":"sha256:0"},"recordedAt":"2026-07-13T00:00:00Z"}]}"#,
        );
        fx.write("openspec/changes/demo/stray-notes.md", "searchable-token stray notes\n");

        let v = status_at(fx.root(), "demo").expect("status ok");
        let paths: Vec<&str> = v["artifacts"]
            .as_array()
            .expect("artifacts array")
            .iter()
            .filter_map(|a| a["outputPath"].as_str())
            .collect();
        assert!(!paths.is_empty(), "the schema's own artifacts are still listed: {paths:?}");
        for p in &paths {
            assert!(
                !p.starts_with('.'),
                "machine-written dot files stay out of the artifact list: {paths:?}"
            );
        }
        assert!(
            !paths.contains(&"stray-notes.md"),
            "the artifact list comes from the schema, not a directory scan: {paths:?}"
        );

        assert!(
            !change_capabilities_at(fx.root(), "demo").iter().any(|c| c.starts_with('.')),
            "the record is not a capability"
        );
        // 搜尋掃的是固定 artifact 序，證據記錄的內容不會成為命中來源。
        let hits = crate::search::search_workspace_at(fx.root(), "searchable-token");
        assert_eq!(
            hits["hits"].as_array().map(|a| a.len()),
            Some(0),
            "evidence content is not searchable change text: {hits}"
        );
    }

    #[test]
    fn status_unknown_change_errors() {
        let fx = FixtureRoot::new("q-status-unknown");
        fx.add_change("demo", OLD_META);
        assert!(status_at(fx.root(), "no-such-change-xyz").is_err());
    }

    #[test]
    fn document_reads_change_artifact() {
        // artifact 以 output path 定址（與 status 的 outputPath 一致）。
        let fx = FixtureRoot::new("q-doc");
        fx.add_change("demo", OLD_META);
        let doc = document_at(fx.root(), "demo", "proposal.md").expect("proposal exists");
        assert!(doc.contains("## Why"));
    }

    #[test]
    fn spec_document_reads_canonical() {
        let fx = FixtureRoot::new("q-canonical");
        fx.write("openspec/specs/cap-x/spec.md", "# cap-x Specification\n");
        let doc = spec_document_at(fx.root(), "cap-x").expect("spec exists");
        assert!(!doc.trim().is_empty());
    }

    #[test]
    fn document_rejects_path_traversal() {
        // 沒有防護時 `../../../secret.txt` 會逃出 change 目錄讀到專案根的檔案。
        let fx = FixtureRoot::new("q-traversal");
        fx.add_change("demo", OLD_META);
        fx.write("secret.txt", "top secret");
        assert!(
            document_at(fx.root(), "demo", "../../../secret.txt").is_none(),
            "traversal in artifact must be rejected"
        );
        assert!(
            document_at(fx.root(), "../..", "secret.txt").is_none(),
            "traversal in change name must be rejected"
        );
        assert!(
            spec_document_at(fx.root(), "../../secret").is_none(),
            "traversal in capability must be rejected"
        );
    }

    #[test]
    fn archived_document_reads_content_and_lists_capabilities() {
        let fx = FixtureRoot::new("q-archived");
        fx.write(
            "openspec/changes/archive/2026-07-04-old/.openspec.yaml",
            "schema: spec-driven\ncreated: 2026-07-01\narchived_at: 2026-07-04\n",
        );
        fx.write("openspec/changes/archive/2026-07-04-old/proposal.md", "## Why\n\nOld body.\n");
        fx.write("openspec/changes/archive/2026-07-04-old/tasks.md", "- [x] 1.1 done\n");
        fx.write(
            "openspec/changes/archive/2026-07-04-old/specs/cap-x/spec.md",
            "## ADDED Requirements\n",
        );

        assert_eq!(
            archived_document_at(fx.root(), "2026-07-04-old", "proposal.md").unwrap(),
            "## Why\n\nOld body.\n"
        );
        assert_eq!(archived_capabilities_at(fx.root(), "2026-07-04-old"), vec!["cap-x"]);
        // 缺件文件回 None（前端顯示空狀態，不是錯誤）。
        assert!(archived_document_at(fx.root(), "2026-07-04-old", "design.md").is_none());
        assert!(archived_document_at(fx.root(), "2026-01-01-ghost", "proposal.md").is_none());
    }

    #[test]
    fn archived_document_rejects_path_traversal() {
        let fx = FixtureRoot::new("q-archived-traversal");
        fx.write("openspec/changes/archive/2026-07-04-old/proposal.md", "## Why\n");
        fx.write("secret.txt", "top secret");

        assert!(
            archived_document_at(fx.root(), "2026-07-04-old", "../../../../secret.txt").is_none(),
            "traversal in artifact must be rejected"
        );
        assert!(
            archived_document_at(fx.root(), "../..", "secret.txt").is_none(),
            "traversal in dated name must be rejected"
        );
        assert!(
            archived_document_at(fx.root(), "2026-07-04-old", "C:\\evil.txt").is_none(),
            "drive-prefixed artifact must be rejected"
        );
        assert!(archived_capabilities_at(fx.root(), "../..").is_empty());
    }

    #[test]
    fn non_project_yields_empty_state_not_panic() {
        let root = fresh_non_project_dir();
        assert_eq!(list_changes_at(&root), json!({ "changes": [] }));
        assert_eq!(list_specs_at(&root), json!({ "specs": [] }));
        assert!(status_at(&root, "anything").is_err());
        assert!(document_at(&root, "anything", "proposal").is_none());
    }
}
