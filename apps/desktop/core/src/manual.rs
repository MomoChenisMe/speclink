//! 手冊頁查詢（change desktop-manual-page design D1〜D3）：讀 `openspec/manual/`
//! 各頁的 frontmatter、推導閱讀序、依 manual-pages 契約計算「可能過期」與
//! 「生成後新增且未入冊」。純本機路徑、不經 store 抽象（v1 只有本機；remote
//! 投影未定）；Tauri 殼各一行委派。
//!
//! 寬容降級：缺欄位的頁照常列出（缺 title 用檔名、缺 section 為 null、缺或非整數
//! order 為 null 並置分區末）；frontmatter 無法解析的頁列入 `pages` 且進 `malformed`，
//! 不報錯、不改檔。

use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

use chrono::NaiveDate;
use regex::Regex;
use serde_json::{json, Value};
use speclink_core::store::Store;

use crate::init_core_context;
use crate::query::is_safe_path_param;

/// 一頁 frontmatter 解析後的索引資料；缺欄位以 `None`／空陣列表示。
struct Page {
    slug: String,
    title: String,
    section: Option<String>,
    order: Option<i64>,
    keywords: Vec<String>,
    sources: Vec<String>,
    /// frontmatter 原字串（輸出用）。
    generated: Option<String>,
    /// 可解析為日期的 generated（過期比對用）。
    generated_date: Option<NaiveDate>,
    malformed: bool,
}

fn empty_index(reason: Value) -> Value {
    json!({
        "present": false,
        "reason": reason,
        "pages": [],
        "uncoveredNew": [],
        "malformed": []
    })
}

// 正典 spec 的 `@trace` 註解區塊與其中的 `updated:` 行（與前端 trace.ts 同一種讀法）。
static TRACE_BLOCK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)<!--\s*@trace\b(.*?)-->").expect("static regex"));
static TRACE_UPDATED_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*updated:\s*(\d{4}-\d{2}-\d{2})\s*$").expect("static regex"));

/// 對應 Tauri command `list_manual_pages`：`{ present, reason, pages, uncoveredNew,
/// malformed }`（欄位 camelCase）。目錄不存在、無 `.md`、不可讀或非專案時
/// `present` 為 false（錯誤只記日誌）。
pub fn list_manual_pages_at(root: &Path) -> Value {
    let Some(ctx) = init_core_context(root) else {
        return empty_index(Value::Null);
    };
    // `openspec/manual/` 跟 spec 目錄名走（`.speclink.yaml` 的 spec_dir）。
    let dir = ctx.workspace.spec_dir().join("manual");
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                eprintln!("manual: cannot read {}: {e}", dir.display());
            }
            return empty_index(Value::Null);
        }
    };
    let mut pages: Vec<Page> = entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("md") {
                return None;
            }
            let slug = path.file_stem()?.to_str()?.to_string();
            Some(match std::fs::read_to_string(&path) {
                Ok(text) => parse_page(slug, &text),
                Err(e) => {
                    eprintln!("manual: cannot read {}: {e}", path.display());
                    malformed_page(slug)
                }
            })
        })
        .collect();
    if pages.is_empty() {
        return empty_index(Value::Null);
    }
    sort_reading_order(&mut pages);

    // 每個 capability 的 @trace updated 範圍只讀一次（stale 與 uncoveredNew 共用）。
    let store: &dyn Store = &ctx.store;
    let mut ranges: HashMap<String, Option<(NaiveDate, NaiveDate)>> = HashMap::new();
    let mut range_of = |cap: &str| -> Option<(NaiveDate, NaiveDate)> {
        *ranges
            .entry(cap.to_string())
            .or_insert_with(|| store.read_canonical_spec(cap).and_then(|doc| trace_updated_range(&doc)))
    };

    let items: Vec<Value> = pages
        .iter()
        .map(|p| {
            let stale = p.generated_date.is_some_and(|gen| {
                p.sources
                    .iter()
                    .any(|cap| range_of(cap).is_some_and(|(_, max)| max > gen))
            });
            json!({
                "slug": p.slug,
                "title": p.title,
                "section": p.section,
                "order": p.order,
                "keywords": p.keywords,
                "sources": p.sources,
                "generated": p.generated,
                "stale": stale,
            })
        })
        .collect();

    // 未入冊：正典 capability 的最小 updated 晚於全手冊最大 generated，且不在任何頁的 sources。
    let max_generated = pages.iter().filter_map(|p| p.generated_date).max();
    let mut uncovered: Vec<String> = match max_generated {
        None => Vec::new(),
        Some(max_gen) => store
            .list_canonical_capabilities()
            .into_iter()
            .filter(|cap| !pages.iter().any(|p| p.sources.iter().any(|s| s == cap)))
            .filter(|cap| range_of(cap).is_some_and(|(min, _)| min > max_gen))
            .collect(),
    };
    uncovered.sort_unstable();

    let malformed: Vec<&str> = pages.iter().filter(|p| p.malformed).map(|p| p.slug.as_str()).collect();
    json!({
        "present": true,
        "reason": Value::Null,
        "pages": items,
        "uncoveredNew": uncovered,
        "malformed": malformed,
    })
}

/// 對應 Tauri command `get_manual_page`：去掉 frontmatter 的內文；frontmatter 壞掉
/// 的頁回全文；頁不存在（或 slug 不安全）回 `None`。
pub fn manual_page_at(root: &Path, slug: &str) -> Option<String> {
    // slug 是單一檔名段：拒絕路徑分隔與 `..`（與其他 document 讀取同一守門）。
    if !is_safe_path_param(slug) || slug.contains(['/', '\\']) {
        return None;
    }
    let ctx = init_core_context(root)?;
    let path = ctx.workspace.spec_dir().join("manual").join(format!("{slug}.md"));
    let text = std::fs::read_to_string(path).ok()?;
    Some(match split_frontmatter(&text) {
        Some((yaml, body)) if parse_frontmatter_yaml(yaml).is_some() => body.to_string(),
        _ => text,
    })
}

/// 切出 frontmatter：首行為 `---`、到下一個 `---` 行為止（皆容忍 `\r`）。回傳
/// （YAML 原文、收尾行之後的內文）；不以 `---` 開頭或沒有收尾行時 `None`。
fn split_frontmatter(text: &str) -> Option<(&str, &str)> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut lines = text.split_inclusive('\n');
    let first = lines.next()?;
    if first.trim_end_matches(['\r', '\n']) != "---" {
        return None;
    }
    let yaml_start = first.len();
    let mut pos = yaml_start;
    for line in lines {
        if line.trim_end_matches(['\r', '\n']) == "---" {
            return Some((&text[yaml_start..pos], &text[pos + line.len()..]));
        }
        pos += line.len();
    }
    None
}

/// frontmatter YAML → 映射（空 frontmatter 視為空映射）；解析失敗或非映射回 `None`。
fn parse_frontmatter_yaml(yaml: &str) -> Option<serde_yaml::Mapping> {
    let cleaned = yaml.replace('\r', "");
    match serde_yaml::from_str::<serde_yaml::Value>(&cleaned).ok()? {
        serde_yaml::Value::Mapping(map) => Some(map),
        serde_yaml::Value::Null => Some(serde_yaml::Mapping::new()),
        _ => None,
    }
}

fn malformed_page(slug: String) -> Page {
    Page {
        title: slug.clone(),
        slug,
        section: None,
        order: None,
        keywords: Vec::new(),
        sources: Vec::new(),
        generated: None,
        generated_date: None,
        malformed: true,
    }
}

fn parse_page(slug: String, text: &str) -> Page {
    let Some(map) = split_frontmatter(text).and_then(|(yaml, _)| parse_frontmatter_yaml(yaml)) else {
        return malformed_page(slug);
    };
    let string_field = |key: &str| map.get(key).and_then(|v| v.as_str()).map(str::to_string);
    let list_field = |key: &str| -> Vec<String> {
        map.get(key)
            .and_then(|v| v.as_sequence())
            .map(|seq| seq.iter().filter_map(scalar_to_string).collect())
            .unwrap_or_default()
    };
    let generated = string_field("generated");
    let generated_date = generated
        .as_deref()
        .and_then(|s| NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok());
    Page {
        title: string_field("title").unwrap_or_else(|| slug.clone()),
        slug,
        section: string_field("section"),
        order: map.get("order").and_then(|v| v.as_i64()),
        keywords: list_field("keywords"),
        sources: list_field("sources"),
        generated,
        generated_date,
        malformed: false,
    }
}

fn scalar_to_string(v: &serde_yaml::Value) -> Option<String> {
    match v {
        serde_yaml::Value::String(s) => Some(s.clone()),
        serde_yaml::Value::Number(n) => Some(n.to_string()),
        serde_yaml::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// 閱讀序（manual-pages 契約「頁的排序僅由 order 決定；分區順序由分區內最小
/// order 決定」）：分區依（最小 order、該 order 的最小檔名）排、分區內依
/// （order、檔名）排；缺或非整數 order 以 i64::MAX 沉底。
fn sort_reading_order(pages: &mut [Page]) {
    let order_key = |p: &Page| (p.order.unwrap_or(i64::MAX), p.slug.clone());
    let mut section_rank: HashMap<Option<String>, (i64, String)> = HashMap::new();
    for p in pages.iter() {
        let key = order_key(p);
        section_rank
            .entry(p.section.clone())
            .and_modify(|best| {
                if key < *best {
                    *best = key.clone();
                }
            })
            .or_insert(key);
    }
    pages.sort_by_cached_key(|p| (section_rank[&p.section].clone(), order_key(p)));
}

/// 正典 spec 全文 `@trace` 註解區塊內 `updated:` 日期的（最小、最大）；沒有可解析
/// 的日期時 `None`。
fn trace_updated_range(doc: &str) -> Option<(NaiveDate, NaiveDate)> {
    let mut range: Option<(NaiveDate, NaiveDate)> = None;
    for block in TRACE_BLOCK_RE.captures_iter(doc) {
        for cap in TRACE_UPDATED_RE.captures_iter(&block[1]) {
            let Ok(date) = NaiveDate::parse_from_str(&cap[1], "%Y-%m-%d") else { continue };
            range = Some(match range {
                None => (date, date),
                Some((min, max)) => (min.min(date), max.max(date)),
            });
        }
    }
    range
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testfixture::FixtureRoot;

    fn page(fx: &FixtureRoot, slug: &str, frontmatter: &str, body: &str) {
        fx.write(
            &format!("openspec/manual/{slug}.md"),
            &format!("---\n{frontmatter}\n---\n\n{body}\n"),
        );
    }

    fn spec_with_trace(fx: &FixtureRoot, cap: &str, updated: &[&str]) {
        let blocks: String = updated
            .iter()
            .map(|d| format!("<!-- @trace\nsource: some-change\nupdated: {d}\n-->\n"))
            .collect();
        fx.write(
            &format!("openspec/specs/{cap}/spec.md"),
            &format!("# {cap} Specification\n\n## Purpose\n\nx\n\n{blocks}"),
        );
    }

    fn slugs(v: &Value) -> Vec<String> {
        v["pages"]
            .as_array()
            .expect("pages array")
            .iter()
            .map(|p| p["slug"].as_str().unwrap().to_string())
            .collect()
    }

    fn page_of<'a>(v: &'a Value, slug: &str) -> &'a Value {
        v["pages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["slug"] == slug)
            .unwrap_or_else(|| panic!("page {slug} listed"))
    }

    #[test]
    fn index_shape_and_reading_order_follow_the_spec_example() {
        // spec「依 order 排序並依 section 分組」Example：四頁 10/20/30/40。
        let fx = FixtureRoot::new("manual-shape");
        page(&fx, "editor", "title: 編輯器\nsection: 文件協作\norder: 30\nsources: []\ngenerated: 2026-09-02", "e");
        page(&fx, "about", "title: 本手冊的來源\nsection: 附錄\norder: 40\nsources: []\ngenerated: 2026-09-02", "a");
        page(
            &fx,
            "first-login",
            "title: 第一次登入\nsection: 開始使用\norder: 20\nkeywords: [登入, github, 審核]\nsources: [github-oauth, user-pending-blocked-pages]\ngenerated: 2026-09-02",
            "f",
        );
        page(&fx, "index", "title: 手冊\nsection: 開始使用\norder: 10\nsources: []\ngenerated: 2026-09-02", "i");
        let v = list_manual_pages_at(fx.root());
        assert_eq!(v["present"], true);
        assert_eq!(v["reason"], Value::Null);
        assert_eq!(slugs(&v), ["index", "first-login", "editor", "about"]);
        assert_eq!(v["malformed"], serde_json::json!([]));
        assert_eq!(v["uncoveredNew"], serde_json::json!([]));
        let fl = page_of(&v, "first-login");
        assert_eq!(fl["title"], "第一次登入");
        assert_eq!(fl["section"], "開始使用");
        assert_eq!(fl["order"], 20);
        assert_eq!(fl["keywords"], serde_json::json!(["登入", "github", "審核"]));
        assert_eq!(fl["sources"], serde_json::json!(["github-oauth", "user-pending-blocked-pages"]));
        assert_eq!(fl["generated"], "2026-09-02");
        assert_eq!(fl["stale"], false);
        // 欄位名是對外契約：每頁恰好這八個 camelCase 鍵。
        let mut keys: Vec<&str> = fl.as_object().unwrap().keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, ["generated", "keywords", "order", "section", "slug", "sources", "stale", "title"]);
    }

    #[test]
    fn ties_break_by_slug_and_sections_rank_by_their_smallest_order() {
        // 同 order 以檔名決斷；分區順序為分區內最小 order（交錯的 A10/B20/A30 →
        // A 區整組在前）。
        let fx = FixtureRoot::new("manual-order");
        page(&fx, "b-second", "section: B\norder: 20", "");
        page(&fx, "a-late", "section: A\norder: 30", "");
        page(&fx, "a-first", "section: A\norder: 10", "");
        page(&fx, "z-tie", "section: A\norder: 30", "");
        page(&fx, "m-tie", "section: A\norder: 30", "");
        let v = list_manual_pages_at(fx.root());
        assert_eq!(slugs(&v), ["a-first", "a-late", "m-tie", "z-tie", "b-second"]);
    }

    #[test]
    fn missing_fields_degrade_without_error() {
        // spec「缺欄位的頁寬容降級」：缺 title 用檔名、缺 section 為 null、非整數
        // order 為 null 且置該分區末、缺 keywords/sources 為空陣列、缺 generated
        // 為 null 且不判過期。
        let fx = FixtureRoot::new("manual-degrade");
        page(&fx, "index", "title: 手冊\nsection: 開始使用\norder: 10", "");
        page(&fx, "orphan", "order: abc", "");
        page(&fx, "no-order", "section: 開始使用", "");
        page(&fx, "float-order", "section: 開始使用\norder: 15.5", "");
        let v = list_manual_pages_at(fx.root());
        assert_eq!(slugs(&v), ["index", "float-order", "no-order", "orphan"]);
        let orphan = page_of(&v, "orphan");
        assert_eq!(orphan["title"], "orphan");
        assert_eq!(orphan["section"], Value::Null);
        assert_eq!(orphan["order"], Value::Null);
        assert_eq!(orphan["keywords"], serde_json::json!([]));
        assert_eq!(orphan["sources"], serde_json::json!([]));
        assert_eq!(orphan["generated"], Value::Null);
        assert_eq!(orphan["stale"], false);
        assert_eq!(page_of(&v, "float-order")["order"], Value::Null);
        assert_eq!(v["malformed"], serde_json::json!([]));
    }

    #[test]
    fn malformed_frontmatter_pages_are_listed_and_flagged() {
        // spec「frontmatter 壞掉的頁仍可開」：不以 --- 開頭、YAML 壞掉、沒有收尾
        // --- 的頁都以檔名為標題列出並進 malformed；其他頁順序與內容不受影響。
        let fx = FixtureRoot::new("manual-malformed");
        page(&fx, "index", "title: 手冊\nsection: 開始使用\norder: 10", "");
        fx.write("openspec/manual/plain.md", "# 沒有 frontmatter\n\n內文。\n");
        fx.write("openspec/manual/broken.md", "---\ntitle: [unclosed\norder: 5\n---\n\n內文。\n");
        fx.write("openspec/manual/unterminated.md", "---\ntitle: 永遠不收尾\n\n內文。\n");
        let v = list_manual_pages_at(fx.root());
        assert_eq!(v["present"], true);
        assert_eq!(slugs(&v), ["index", "broken", "plain", "unterminated"]);
        for slug in ["plain", "broken", "unterminated"] {
            let p = page_of(&v, slug);
            assert_eq!(p["title"], slug);
            assert_eq!(p["section"], Value::Null);
            assert_eq!(p["order"], Value::Null);
        }
        assert_eq!(v["malformed"], serde_json::json!(["broken", "plain", "unterminated"]));
        assert_eq!(page_of(&v, "index")["title"], "手冊");
        // 壞頁仍可開：回全文。
        assert_eq!(
            manual_page_at(fx.root(), "broken").as_deref(),
            Some("---\ntitle: [unclosed\norder: 5\n---\n\n內文。\n")
        );
        assert_eq!(manual_page_at(fx.root(), "plain").as_deref(), Some("# 沒有 frontmatter\n\n內文。\n"));
    }

    #[test]
    fn stale_follows_the_max_trace_updated_of_the_sources() {
        // spec「可能過期與未入冊的標示」Example 判定表。
        let fx = FixtureRoot::new("manual-stale");
        spec_with_trace(&fx, "x", &["2026-08-01", "2026-09-05"]);
        spec_with_trace(&fx, "y", &["2026-08-20"]);
        page(&fx, "px", "order: 10\nsources: [x]\ngenerated: 2026-09-01", "");
        page(&fx, "py", "order: 20\nsources: [y]\ngenerated: 2026-09-01", "");
        page(&fx, "pxy", "order: 25\nsources: [y, x]\ngenerated: 2026-09-01", "");
        page(&fx, "p-empty", "order: 30\nsources: []\ngenerated: 2026-09-01", "");
        page(&fx, "p-nogen", "order: 40\nsources: [x]", "");
        page(&fx, "p-missing-spec", "order: 50\nsources: [nope]\ngenerated: 2026-09-01", "");
        page(&fx, "p-same-day", "order: 60\nsources: [x]\ngenerated: 2026-09-05", "");
        let v = list_manual_pages_at(fx.root());
        assert_eq!(page_of(&v, "px")["stale"], true);
        assert_eq!(page_of(&v, "py")["stale"], false);
        assert_eq!(page_of(&v, "pxy")["stale"], true, "任一來源更新即過期");
        assert_eq!(page_of(&v, "p-empty")["stale"], false);
        assert_eq!(page_of(&v, "p-nogen")["stale"], false);
        assert_eq!(page_of(&v, "p-missing-spec")["stale"], false);
        assert_eq!(page_of(&v, "p-same-day")["stale"], false, "同日不算晚於");
    }

    #[test]
    fn uncovered_new_lists_specs_born_after_the_manual_and_absent_from_every_sources() {
        // spec「生成後新增的規格計入未入冊」：min updated 晚於全手冊最大 generated
        // 且不在任何 sources；加進某頁 sources 後消失。
        let fx = FixtureRoot::new("manual-uncovered");
        spec_with_trace(&fx, "z", &["2026-09-03", "2026-09-04"]);
        spec_with_trace(&fx, "w", &["2026-09-03"]);
        spec_with_trace(&fx, "v", &["2026-08-01", "2026-09-04"]);
        fx.write("openspec/specs/no-trace/spec.md", "# no-trace\n");
        page(&fx, "index", "order: 10\nsources: []\ngenerated: 2026-08-15", "");
        page(&fx, "pw", "order: 20\nsources: [w]\ngenerated: 2026-09-01", "");
        let v = list_manual_pages_at(fx.root());
        assert_eq!(v["uncoveredNew"], serde_json::json!(["z"]));
        page(&fx, "pz", "order: 30\nsources: [z]\ngenerated: 2026-09-01", "");
        let v = list_manual_pages_at(fx.root());
        assert_eq!(v["uncoveredNew"], serde_json::json!([]));
    }

    #[test]
    fn uncovered_new_is_empty_when_no_page_carries_generated() {
        let fx = FixtureRoot::new("manual-uncovered-nogen");
        spec_with_trace(&fx, "z", &["2026-09-03"]);
        page(&fx, "index", "order: 10", "");
        let v = list_manual_pages_at(fx.root());
        assert_eq!(v["uncoveredNew"], serde_json::json!([]));
    }

    #[test]
    fn absent_or_empty_manual_dir_reports_present_false() {
        // spec「無手冊目錄」：目錄不存在、只有非 .md 檔、非 speclink 專案。
        let fx = FixtureRoot::new("manual-absent");
        let v = list_manual_pages_at(fx.root());
        assert_eq!(
            v,
            serde_json::json!({
                "present": false,
                "reason": null,
                "pages": [],
                "uncoveredNew": [],
                "malformed": []
            })
        );
        fx.write("openspec/manual/notes.txt", "not a page");
        let v = list_manual_pages_at(fx.root());
        assert_eq!(v["present"], false);
        assert_eq!(v["pages"], serde_json::json!([]));
        assert_eq!(manual_page_at(fx.root(), "index"), None);

        let non_project = crate::testfixture::fresh_non_project_dir("manual");
        assert_eq!(list_manual_pages_at(&non_project)["present"], false);
        assert_eq!(manual_page_at(&non_project, "index"), None);
    }

    #[test]
    fn crlf_content_parses_and_body_keeps_its_line_endings() {
        // design Risks：Windows 換行——frontmatter 以行為單位剝 \r；內文原樣回傳。
        let fx = FixtureRoot::new("manual-crlf");
        fx.write(
            "openspec/manual/index.md",
            "---\r\ntitle: 手冊\r\nsection: 開始使用\r\norder: 10\r\nkeywords: [a, b]\r\nsources: []\r\ngenerated: 2026-09-02\r\n---\r\n\r\n# 標題\r\n\r\n內文。\r\n",
        );
        let v = list_manual_pages_at(fx.root());
        let p = page_of(&v, "index");
        assert_eq!(p["title"], "手冊");
        assert_eq!(p["section"], "開始使用");
        assert_eq!(p["order"], 10);
        assert_eq!(p["keywords"], serde_json::json!(["a", "b"]));
        assert_eq!(p["generated"], "2026-09-02");
        assert_eq!(v["malformed"], serde_json::json!([]));
        assert_eq!(manual_page_at(fx.root(), "index").as_deref(), Some("\r\n# 標題\r\n\r\n內文。\r\n"));
    }

    #[test]
    fn manual_page_strips_frontmatter_and_refuses_escapes() {
        let fx = FixtureRoot::new("manual-page");
        page(&fx, "index", "title: 手冊\norder: 10", "# 手冊\n\n內文。");
        fx.write("openspec/manual/empty-fm.md", "---\n---\nbody only\n");
        fx.write("openspec/secret.md", "not a manual page");
        assert_eq!(manual_page_at(fx.root(), "index").as_deref(), Some("\n# 手冊\n\n內文。\n"));
        assert_eq!(manual_page_at(fx.root(), "empty-fm").as_deref(), Some("body only\n"));
        assert_eq!(manual_page_at(fx.root(), "missing"), None);
        assert_eq!(manual_page_at(fx.root(), "../secret"), None);
        assert_eq!(manual_page_at(fx.root(), "sub/index"), None);
        assert_eq!(manual_page_at(fx.root(), ""), None);
    }

    #[test]
    fn keywords_and_sources_accept_scalars_and_ignore_non_lists() {
        let fx = FixtureRoot::new("manual-lists");
        page(&fx, "index", "order: 10\nkeywords: [登入, 42, true]\nsources: github-oauth", "");
        let v = list_manual_pages_at(fx.root());
        let p = page_of(&v, "index");
        assert_eq!(p["keywords"], serde_json::json!(["登入", "42", "true"]));
        assert_eq!(p["sources"], serde_json::json!([]), "非陣列的 sources 視為缺席");
    }
}
