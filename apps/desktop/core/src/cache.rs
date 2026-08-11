//! 歸檔清單的 SQLite 衍生快取。
//!
//! 快取限縮於歸檔（archived）change 清單——歸檔量隨時間無上限成長，每次全量重解析浪費。
//! active change／spec 清單不經此快取（見 [`crate::query`]），一律即時經 core 讀取檔案。
//!
//! 快取為衍生資料：可刪除後由歸檔目錄重建、帶 schema 版本標記；版本不符時重建。
//! 檔案系統（`changes/archive/`）恆為真相，快取每次讀取都對照目錄收斂。

use std::path::Path;

use rusqlite::Connection;
use serde_json::{json, Value};
use speclink_core::store::Store;

use crate::init_core_context;

/// 快取 schema 版本。改動快取表結構時遞增——舊版本會被丟棄重建。
/// v2：archived_changes 加 tasks_total／tasks_done（清單徽章，首次收斂後零解析）。
/// v3：加 spec_count／created_by／from_discussions（封存卡收合資訊，spec-archive-drawer design D5）。
/// v4：加 review_status（封存時的審查結局，spec client-protocol「已封存清單的審查結局欄位」）。
/// v5：加 verify_status（封存時的驗證結局，spec client-protocol「已封存清單的驗證結局欄位」）。
/// v6：加 why_excerpt／created（封存卡描述列與抽屜出身列，spec client-protocol
///     「已封存清單的呈現輔助欄位」）。
const CACHE_VERSION: i64 = 6;

/// 回傳歸檔 change 清單：`{ "archived": [ { datedName, date, name } ] }`，按 datedName 排序。
/// 非專案回傳 `{ "archived": [] }`。
pub fn archived_changes_at(root: &Path) -> Value {
    let Some(ctx) = init_core_context(root) else {
        return json!({ "archived": [] });
    };
    let archive_dir = ctx.workspace.spec_dir().join("changes").join("archive");
    let mut names = read_archive_dir_names(&archive_dir);
    names.sort();
    let db_path = ctx.workspace.work_dir().join("desktop-cache.db");
    // 快取失敗（如磁碟/鎖問題）時退回直接以目錄資料回應——真相恆為檔案。
    let items = match reconcile(&db_path, &ctx.store, &names) {
        Ok(items) => items,
        Err(_) => names.iter().map(|n| item_for(n)).collect(),
    };
    json!({ "archived": items })
}

/// 列出歸檔目錄下的子目錄名（各為一個 dated_name）。目錄不存在時回傳空。
fn read_archive_dir_names(archive_dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(archive_dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect()
}

/// 以目前歸檔目錄為準收斂快取，回傳排序後的清單項。
fn reconcile(db_path: &Path, store: &dyn Store, names: &[String]) -> rusqlite::Result<Vec<Value>> {
    if let Some(parent) = db_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let conn = Connection::open(db_path)?;
    ensure_schema(&conn)?;

    // 只為尚未快取的名稱讀 meta 與解析任務計數——這是快取相對「每次全量重解析」
    // 的省力點；之後清單讀取零解析。
    for name in names {
        let cached: bool = conn.query_row(
            "SELECT 1 FROM archived_changes WHERE dated_name = ?1",
            [name],
            |_| Ok(true),
        )
        .unwrap_or(false);
        if !cached {
            let meta = store.read_archived_meta(name).unwrap_or_default();
            let counts = store.read_archived_artifact(name, "tasks.md").map(|text| {
                let tasks = speclink_core::tasks::parse(&text);
                let (total, complete, _) = speclink_core::tasks::progress(&tasks);
                (total as i64, complete as i64)
            });
            // 封存卡收合資訊（design D4/D5）：入庫使清單讀取零解析。from_discussions
            // 以逗號串存放（與 meta 累積器同格式），讀出時同語意拆分。
            // 歸檔 metadata 的壞檔語意不在 fail-closed 範圍（僅活躍 change）——
            // 歷史紀錄照舊寬鬆讀取。
            let parsed = speclink_core::model::ChangeMeta::from_text(Some(&meta)).unwrap_or_default();
            let spec_count = store.archived_delta_capabilities(name).len() as i64;
            // 審查結局（封存即定格，不重算凍結度）：含章 → reviewed；含化石工單
            // 而無章 → reviewedNotPassed；皆無 → none。入庫後清單讀取零解析。
            let review_status = if parsed.reviewed_at.is_some() {
                "reviewed"
            } else if store
                .read_archived_artifact(name, speclink_core::review::REVIEW_DOC)
                .is_some()
            {
                "reviewedNotPassed"
            } else {
                "none"
            };
            // 驗證結局同構且獨立判定：同一項可以是「審查通過」卻「曾驗證未通過」。
            let verify_status = if parsed.verified_at.is_some() {
                "verified"
            } else if store
                .read_archived_artifact(name, speclink_core::verify::VERIFY_DOC)
                .is_some()
            {
                "verifiedNotPassed"
            } else {
                "none"
            };
            // 呈現輔助欄位（desktop-archived-parity D2）：封存卡描述列的 Why 首句與
            // 抽屜出身列的建立日期。來源不可讀／缺席時存 NULL，讀出時不插 key。
            let why_excerpt = store
                .read_archived_artifact(name, "proposal.md")
                .as_deref()
                .and_then(crate::query::why_excerpt);
            conn.execute(
                "INSERT OR REPLACE INTO archived_changes (dated_name, meta, tasks_total, tasks_done, spec_count, created_by, from_discussions, review_status, verify_status, why_excerpt, created) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![
                    name,
                    meta,
                    counts.map(|c| c.0),
                    counts.map(|c| c.1),
                    spec_count,
                    parsed.created_by,
                    parsed.from_discussions().join(","),
                    review_status,
                    verify_status,
                    why_excerpt,
                    parsed.created,
                ],
            )?;
        }
    }

    // 移除已不在歸檔目錄中的殘留列（真相是目錄）。
    let keep: std::collections::HashSet<&str> = names.iter().map(|s| s.as_str()).collect();
    let mut stmt = conn.prepare("SELECT dated_name FROM archived_changes")?;
    let existing: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .flatten()
        .collect();
    drop(stmt);
    for name in &existing {
        if !keep.contains(name.as_str()) {
            conn.execute("DELETE FROM archived_changes WHERE dated_name = ?1", [name])?;
        }
    }

    let mut stmt = conn.prepare(
        "SELECT dated_name, tasks_total, tasks_done, spec_count, created_by, from_discussions, review_status, verify_status, why_excerpt, created FROM archived_changes ORDER BY dated_name",
    )?;
    let items: Vec<Value> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<i64>>(1)?,
                r.get::<_, Option<i64>>(2)?,
                r.get::<_, Option<i64>>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, Option<String>>(6)?,
                r.get::<_, Option<String>>(7)?,
                r.get::<_, Option<String>>(8)?,
                r.get::<_, Option<String>>(9)?,
            ))
        })?
        .flatten()
        .map(|row| {
            let (
                n,
                total,
                done,
                spec_count,
                created_by,
                from_discussions,
                review_status,
                verify_status,
                why_excerpt,
                created,
            ) = row;
            let mut item = item_for(&n);
            // 無 tasks.md 的封存項徽章欄位缺席（前端據此不顯示徽章）。
            if let (Some(total), Some(done)) = (total, done) {
                item["tasksTotal"] = json!(total);
                item["tasksDone"] = json!(done);
            }
            // 封存卡收合資訊恆存在 key：缺席語意為 0／null／空陣列（design D4）。
            item["specCount"] = json!(spec_count.unwrap_or(0));
            item["createdBy"] = json!(created_by);
            let discussions: Vec<&str> = from_discussions
                .as_deref()
                .map(|v| v.split(',').map(str::trim).filter(|s| !s.is_empty()).collect())
                .unwrap_or_default();
            item["fromDiscussions"] = json!(discussions);
            item["reviewStatus"] = json!(review_status.as_deref().unwrap_or("none"));
            item["verifyStatus"] = json!(verify_status.as_deref().unwrap_or("none"));
            // 呈現輔助欄位缺席時不插 key（spec：不以空字串或 null 佔位）。
            if let Some(excerpt) = why_excerpt {
                item["whyExcerpt"] = json!(excerpt);
            }
            if let Some(created) = created {
                item["created"] = json!(created);
            }
            item
        })
        .collect();
    Ok(items)
}

/// 確保快取 schema 存在且版本相符；版本不符或缺失時丟棄重建。
fn ensure_schema(conn: &Connection) -> rusqlite::Result<()> {
    let version: Option<i64> = conn
        .query_row(
            "SELECT version FROM schema_version LIMIT 1",
            [],
            |r| r.get(0),
        )
        .ok();
    if version != Some(CACHE_VERSION) {
        conn.execute_batch(
            "DROP TABLE IF EXISTS archived_changes;
             DROP TABLE IF EXISTS schema_version;
             CREATE TABLE schema_version (version INTEGER);
             CREATE TABLE archived_changes (dated_name TEXT PRIMARY KEY, meta TEXT, tasks_total INTEGER, tasks_done INTEGER, spec_count INTEGER, created_by TEXT, from_discussions TEXT, review_status TEXT, verify_status TEXT, why_excerpt TEXT, created TEXT);",
        )?;
        conn.execute(
            "INSERT INTO schema_version (version) VALUES (?1)",
            [CACHE_VERSION],
        )?;
    }
    Ok(())
}

/// 由 dated_name（`YYYY-MM-DD-<name>`）廉價拆出顯示欄位，不讀檔。
fn item_for(dated_name: &str) -> Value {
    let (date, name) = split_dated_name(dated_name);
    json!({ "datedName": dated_name, "date": date, "name": name })
}

/// 拆 `YYYY-MM-DD-<name>`：前 10 字為日期、第 12 字起為名稱；不符格式時 date 空、name 為全名。
fn split_dated_name(dated: &str) -> (&str, &str) {
    let bytes = dated.as_bytes();
    let looks_dated = bytes.len() > 11
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'-'
        && bytes[..10].iter().enumerate().all(|(i, b)| {
            if i == 4 || i == 7 {
                *b == b'-'
            } else {
                b.is_ascii_digit()
            }
        });
    if looks_dated {
        (&dated[..10], &dated[11..])
    } else {
        ("", dated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    /// 建立一個含歸檔 change 的暫存 fixture 專案（openspec/ 使 discover 成功）。
    fn fixture_project(tag: &str, archived: &[&str]) -> PathBuf {
        let root = std::env::temp_dir().join(format!("speclink-cache-{}-{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let archive = root.join("openspec").join("changes").join("archive");
        for name in archived {
            let dir = archive.join(name);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("proposal.md"), "## Why\narchived\n").unwrap();
        }
        fs::create_dir_all(root.join("openspec").join("specs")).unwrap();
        root
    }

    fn dated_names(v: &Value) -> Vec<String> {
        v["archived"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["datedName"].as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn archived_list_carries_review_outcome_three_states() {
        // spec client-protocol「已封存清單的審查結局欄位」：封存目錄含章 → reviewed；
        // 含（化石）工單而無章 → reviewedNotPassed；皆無 → none。已封存側不重算
        // 凍結度（封存即定格）——此處的假 hash 不得使 reviewed 降級。
        let root = fixture_project(
            "review",
            &["2026-01-01-plain", "2026-01-02-passed", "2026-01-03-carried"],
        );
        let archive = root.join("openspec").join("changes").join("archive");
        fs::write(
            archive.join("2026-01-02-passed").join(".openspec.yaml"),
            "schema: spec-driven\ncreated: 2026-01-01\nreviewed_at: 2026-01-02\nreviewed_by: Rev <r@example.com>\nreviewed_with: claude\nreviewed_tasks_total: 1\nreviewed_scope:\n  - path: src/lib.rs\n    hash: dead\n",
        )
        .unwrap();
        fs::write(
            archive.join("2026-01-03-carried").join("review.md"),
            "# Review — carried\n\n## Round 1\n\n**Scope**: src/lib.rs\n\n- [CRITICAL] src/lib.rs — broken\n",
        )
        .unwrap();
        let v = archived_changes_at(&root);
        let by_name = |n: &str| {
            v["archived"]
                .as_array()
                .unwrap()
                .iter()
                .find(|i| i["datedName"] == n)
                .unwrap()
                .clone()
        };
        assert_eq!(by_name("2026-01-01-plain")["reviewStatus"], "none");
        assert_eq!(by_name("2026-01-02-passed")["reviewStatus"], "reviewed");
        assert_eq!(by_name("2026-01-03-carried")["reviewStatus"], "reviewedNotPassed");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn archived_list_carries_verify_outcome_three_states() {
        // spec client-protocol「已封存清單的驗證結局欄位」：封存目錄含章 →
        // verified；含（化石）工單而無章 → verifiedNotPassed；皆無 → none。
        // 已封存側不重算凍結度（封存即定格）——假 hash 不得使 verified 降級。
        // 兩站在同一項上獨立判定：末項同時是「審查通過」與「曾驗證未通過」。
        let root = fixture_project(
            "verify",
            &["2026-02-01-plain", "2026-02-02-passed", "2026-02-03-carried"],
        );
        let archive = root.join("openspec").join("changes").join("archive");
        fs::write(
            archive.join("2026-02-02-passed").join(".openspec.yaml"),
            "schema: spec-driven\ncreated: 2026-02-01\nverified_at: 2026-02-02\nverified_by: Ver <v@example.com>\nverified_with: claude\nverified_tasks_total: 1\nverified_scope:\n  - path: src/lib.rs\n    hash: dead\n",
        )
        .unwrap();
        fs::write(
            archive.join("2026-02-03-carried").join(".openspec.yaml"),
            "schema: spec-driven\ncreated: 2026-02-01\nreviewed_at: 2026-02-02\nreviewed_by: Rev <r@example.com>\nreviewed_tasks_total: 1\nreviewed_scope:\n  - path: src/lib.rs\n    hash: dead\n",
        )
        .unwrap();
        fs::write(
            archive.join("2026-02-03-carried").join("verify.md"),
            "# Verify — carried\n\n## Round 1\n\n**Scope**: src/lib.rs\n\n- [CRITICAL] src/lib.rs — requirement R2 has no implementation\n",
        )
        .unwrap();
        let v = archived_changes_at(&root);
        let by_name = |n: &str| {
            v["archived"]
                .as_array()
                .unwrap()
                .iter()
                .find(|i| i["datedName"] == n)
                .unwrap()
                .clone()
        };
        assert_eq!(by_name("2026-02-01-plain")["verifyStatus"], "none");
        assert_eq!(by_name("2026-02-02-passed")["verifyStatus"], "verified");
        let carried = by_name("2026-02-03-carried");
        assert_eq!(carried["verifyStatus"], "verifiedNotPassed");
        assert_eq!(carried["reviewStatus"], "reviewed", "the two outcomes stay independent");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn archived_list_matches_archive_dir() {
        let root = fixture_project("match", &["2026-01-02-beta", "2026-01-01-alpha"]);
        let got = dated_names(&archived_changes_at(&root));
        assert_eq!(got, vec!["2026-01-01-alpha", "2026-01-02-beta"]);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rebuilds_when_cache_file_missing() {
        let root = fixture_project("missing", &["2026-03-03-gamma"]);
        // 第一次讀建立快取
        let _ = archived_changes_at(&root);
        // 刪掉快取檔，仍應由歸檔目錄重建
        let _ = fs::remove_file(root.join(".speclink").join("desktop-cache.db"));
        let got = dated_names(&archived_changes_at(&root));
        assert_eq!(got, vec!["2026-03-03-gamma"]);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rebuilds_on_schema_version_mismatch() {
        let root = fixture_project("ver", &["2026-04-04-delta"]);
        let db_dir = root.join(".speclink");
        fs::create_dir_all(&db_dir).unwrap();
        // 預先寫入一個版本不符的快取
        let conn = Connection::open(db_dir.join("desktop-cache.db")).unwrap();
        conn.execute("CREATE TABLE schema_version (version INTEGER)", []).unwrap();
        conn.execute("INSERT INTO schema_version (version) VALUES (999)", []).unwrap();
        drop(conn);
        let got = dated_names(&archived_changes_at(&root));
        assert_eq!(got, vec!["2026-04-04-delta"], "stale-version cache is rebuilt");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn active_list_does_not_create_cache_db() {
        let root = fixture_project("active", &["2026-05-05-epsilon"]);
        let _ = crate::query::list_changes_at(&root);
        let _ = crate::query::list_specs_at(&root);
        assert!(
            !root.join(".speclink").join("desktop-cache.db").exists(),
            "active listing must not touch the archived cache"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn task_counts_converge_once_and_read_zero_parse_afterwards() {
        let root = fixture_project("counts", &["2026-06-06-zeta"]);
        let dir = root.join("openspec").join("changes").join("archive").join("2026-06-06-zeta");
        fs::write(dir.join("tasks.md"), "## 1. G\n\n- [x] 1.1 a\n- [x] 1.2 b\n- [ ] 1.3 c\n").unwrap();

        let v = archived_changes_at(&root);
        let item = &v["archived"].as_array().unwrap()[0];
        assert_eq!(item["tasksTotal"], 3, "first read converges counts into the cache: {item}");
        assert_eq!(item["tasksDone"], 2);

        // 首次收斂後清單讀取零解析：改動檔案不影響已快取的計數
        //（封存內容本就不再變動，這證明讀取路徑不再碰 tasks.md）。
        fs::write(dir.join("tasks.md"), "- [ ] only one\n").unwrap();
        let v2 = archived_changes_at(&root);
        let item2 = &v2["archived"].as_array().unwrap()[0];
        assert_eq!(item2["tasksTotal"], 3, "cached counts must be served without re-parsing");
        assert_eq!(item2["tasksDone"], 2);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn archived_change_without_tasks_md_has_no_badge_fields() {
        let root = fixture_project("nocounts", &["2026-06-07-eta"]);
        let v = archived_changes_at(&root);
        let item = &v["archived"].as_array().unwrap()[0];
        assert!(item.get("tasksTotal").is_none(), "no tasks.md → badge fields absent: {item}");
        assert!(item.get("tasksDone").is_none());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn v1_cache_is_dropped_and_rebuilt_with_counts() {
        let root = fixture_project("v1", &["2026-06-08-theta"]);
        let dir = root.join("openspec").join("changes").join("archive").join("2026-06-08-theta");
        fs::write(dir.join("tasks.md"), "- [x] 1.1 done\n").unwrap();
        // 預寫一個 v1 快取（舊表結構、已含該列）——版本升級須整表重建。
        let db_dir = root.join(".speclink");
        fs::create_dir_all(&db_dir).unwrap();
        let conn = Connection::open(db_dir.join("desktop-cache.db")).unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_version (version INTEGER);
             INSERT INTO schema_version (version) VALUES (1);
             CREATE TABLE archived_changes (dated_name TEXT PRIMARY KEY, meta TEXT);
             INSERT INTO archived_changes (dated_name, meta) VALUES ('2026-06-08-theta', '');",
        )
        .unwrap();
        drop(conn);

        let v = archived_changes_at(&root);
        let item = &v["archived"].as_array().unwrap()[0];
        assert_eq!(item["tasksTotal"], 1, "v1 cache must be rebuilt so counts converge: {item}");
        assert_eq!(item["tasksDone"], 1);
        let conn = Connection::open(db_dir.join("desktop-cache.db")).unwrap();
        let version: i64 = conn
            .query_row("SELECT version FROM schema_version LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, CACHE_VERSION, "schema version stamped to current after rebuild");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn archived_items_carry_card_info_fields() {
        // 封存變更卡收合資訊（spec-archive-drawer design D4）：specCount 為 specs/ 下
        // capability 目錄數、createdBy 取 .openspec.yaml 的 created_by（缺席 null）、
        // fromDiscussions 為來源討論 slug 陣列（缺席空陣列）；欄位入快取，首次收斂後零解析。
        let root = fixture_project("cardinfo", &["2026-06-10-kappa", "2026-06-11-lambda"]);
        let dir = root.join("openspec").join("changes").join("archive").join("2026-06-10-kappa");
        fs::write(
            dir.join(".openspec.yaml"),
            "schema: spec-driven\ncreated: 2026-06-01\ncreated_by: momo\nfrom_discussion: alpha-search, beta-cache\narchived_at: 2026-06-10\n",
        )
        .unwrap();
        fs::create_dir_all(dir.join("specs").join("cap-a")).unwrap();
        fs::write(dir.join("specs").join("cap-a").join("spec.md"), "## ADDED Requirements\n").unwrap();
        fs::create_dir_all(dir.join("specs").join("cap-b")).unwrap();
        fs::write(dir.join("specs").join("cap-b").join("spec.md"), "## ADDED Requirements\n").unwrap();

        let v = archived_changes_at(&root);
        let arr = v["archived"].as_array().unwrap();
        let by_name = |n: &str| arr.iter().find(|i| i["datedName"] == n).unwrap().clone();
        let kappa = by_name("2026-06-10-kappa");
        assert_eq!(kappa["specCount"], 2, "specs/ 下兩個 capability 目錄: {kappa}");
        assert_eq!(kappa["createdBy"], "momo");
        assert_eq!(kappa["fromDiscussions"], json!(["alpha-search", "beta-cache"]));
        // meta 與 specs 全缺席的封存項：null／空陣列，清單照常回傳。
        let lambda = by_name("2026-06-11-lambda");
        assert_eq!(lambda["specCount"], 0);
        assert!(lambda["createdBy"].is_null());
        assert_eq!(lambda["fromDiscussions"], json!([]));

        // 首次收斂後零解析：改動 meta 不影響已快取的欄位（封存內容本就不再變動）。
        fs::write(dir.join(".openspec.yaml"), "created_by: someone-else\n").unwrap();
        let v2 = archived_changes_at(&root);
        let item2 = v2["archived"].as_array().unwrap().iter().find(|i| i["datedName"] == "2026-06-10-kappa").unwrap().clone();
        assert_eq!(item2["createdBy"], "momo", "cached card info served without re-parsing");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn v2_cache_is_dropped_and_rebuilt_with_card_info() {
        // 版本遞升（spec-archive-drawer design D5）：新欄位入庫使 CACHE_VERSION 2→3，
        // 舊 v2 快取整表重建、新欄位補齊、版本蓋 3 章。
        let root = fixture_project("v2", &["2026-06-12-mu"]);
        let dir = root.join("openspec").join("changes").join("archive").join("2026-06-12-mu");
        fs::write(dir.join(".openspec.yaml"), "created: 2026-06-01\ncreated_by: momo\n").unwrap();
        fs::create_dir_all(dir.join("specs").join("cap-a")).unwrap();
        fs::write(dir.join("specs").join("cap-a").join("spec.md"), "## ADDED Requirements\n").unwrap();
        fs::write(dir.join("tasks.md"), "- [x] 1.1 done\n").unwrap();
        // 預寫一個 v2 快取（v2 表結構、已含該列）——版本升級須整表重建。
        let db_dir = root.join(".speclink");
        fs::create_dir_all(&db_dir).unwrap();
        let conn = Connection::open(db_dir.join("desktop-cache.db")).unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_version (version INTEGER);
             INSERT INTO schema_version (version) VALUES (2);
             CREATE TABLE archived_changes (dated_name TEXT PRIMARY KEY, meta TEXT, tasks_total INTEGER, tasks_done INTEGER);
             INSERT INTO archived_changes (dated_name, meta, tasks_total, tasks_done) VALUES ('2026-06-12-mu', '', 1, 1);",
        )
        .unwrap();
        drop(conn);

        let v = archived_changes_at(&root);
        let item = &v["archived"].as_array().unwrap()[0];
        assert_eq!(item["specCount"], 1, "v2 cache must be rebuilt so card info converges: {item}");
        assert_eq!(item["createdBy"], "momo");
        assert_eq!(item["tasksTotal"], 1);
        let conn = Connection::open(db_dir.join("desktop-cache.db")).unwrap();
        let version: i64 = conn
            .query_row("SELECT version FROM schema_version LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, CACHE_VERSION, "schema version stamped to current after rebuild");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn archived_items_carry_why_excerpt_and_created() {
        // spec client-protocol「已封存清單的呈現輔助欄位」：whyExcerpt 為封存
        // proposal.md 的 Why 區段首個非空行、created 取 metadata 的建立日期；
        // 來源不可讀或缺席時該欄位不插 key（不以空字串或 null 佔位），
        // 清單其餘欄位照常回傳。
        let root = fixture_project(
            "whyexcerpt",
            &["2026-07-01-nu", "2026-07-02-xi", "2026-07-03-omicron"],
        );
        let archive = root.join("openspec").join("changes").join("archive");
        // 兩欄位俱在。
        fs::write(
            archive.join("2026-07-01-nu").join("proposal.md"),
            "## Why\n\n看板搜尋列缺席，跨卡找東西只能逐欄翻。\n\n## What Changes\n\n- 補搜尋列\n",
        )
        .unwrap();
        fs::write(
            archive.join("2026-07-01-nu").join(".openspec.yaml"),
            "schema: spec-driven\ncreated: 2026-06-20\narchived_at: 2026-07-01\n",
        )
        .unwrap();
        // proposal 缺席（fixture 預設會寫一份，此處移除）——metadata 仍可得。
        fs::remove_file(archive.join("2026-07-02-xi").join("proposal.md")).unwrap();
        fs::write(
            archive.join("2026-07-02-xi").join(".openspec.yaml"),
            "schema: spec-driven\ncreated: 2026-06-21\n",
        )
        .unwrap();
        // proposal 在但無 Why 區段，且 metadata 全缺席。
        fs::write(
            archive.join("2026-07-03-omicron").join("proposal.md"),
            "## What Changes\n\n只有這段。\n",
        )
        .unwrap();

        let v = archived_changes_at(&root);
        let arr = v["archived"].as_array().unwrap();
        let by_name = |n: &str| arr.iter().find(|i| i["datedName"] == n).unwrap().clone();

        let nu = by_name("2026-07-01-nu");
        assert_eq!(nu["whyExcerpt"], "看板搜尋列缺席，跨卡找東西只能逐欄翻。");
        assert_eq!(nu["created"], "2026-06-20");

        let xi = by_name("2026-07-02-xi");
        assert!(xi.get("whyExcerpt").is_none(), "proposal 缺席 → 不插 key: {xi}");
        assert_eq!(xi["created"], "2026-06-21", "另一欄位照常");
        assert_eq!(xi["datedName"], "2026-07-02-xi", "既有欄位照常回傳");

        let omicron = by_name("2026-07-03-omicron");
        assert!(omicron.get("whyExcerpt").is_none(), "無 Why 區段 → 不插 key: {omicron}");
        assert!(omicron.get("created").is_none(), "metadata 無建立日期 → 不插 key");

        // 首次收斂後零解析：改動 proposal 不影響已快取的欄位。
        fs::write(
            archive.join("2026-07-01-nu").join("proposal.md"),
            "## Why\n\n改過的內容。\n",
        )
        .unwrap();
        let v2 = archived_changes_at(&root);
        let nu2 = v2["archived"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["datedName"] == "2026-07-01-nu")
            .unwrap()
            .clone();
        assert_eq!(
            nu2["whyExcerpt"], "看板搜尋列缺席，跨卡找東西只能逐欄翻。",
            "cached excerpt served without re-parsing"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn cache_failure_falls_back_to_directory_listing_without_badges() {
        let root = fixture_project("fallback", &["2026-06-09-iota"]);
        // 讓快取檔位置變成目錄——Connection::open 失敗，走目錄直讀退回路徑。
        fs::create_dir_all(root.join(".speclink").join("desktop-cache.db")).unwrap();
        let v = archived_changes_at(&root);
        let got = dated_names(&v);
        assert_eq!(got, vec!["2026-06-09-iota"], "fallback still serves the directory truth");
        let item = &v["archived"].as_array().unwrap()[0];
        assert!(item.get("tasksTotal").is_none(), "fallback items carry no badge fields");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn non_project_yields_empty_archived() {
        let root = std::env::temp_dir().join(format!("speclink-cache-none-{}", std::process::id()));
        let _ = fs::create_dir_all(&root);
        assert_eq!(archived_changes_at(&root), json!({ "archived": [] }));
        let _ = fs::remove_dir_all(&root);
    }
}
