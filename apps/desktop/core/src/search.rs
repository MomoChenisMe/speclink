//! workspace 全文查詢（desktop-ux-polish design D6）：遍歷 active 變更的
//! artifacts（proposal／design／tasks／delta 規格）與 active 討論記錄，
//! 不分大小寫子字串比對，回傳命中卡片與 snippet——看板全文搜尋的單一資料源。
//! 每張卡取首個命中 artifact 的首個命中處；比對走 char 層，snippet 裁切
//! 不會落在多位元組字元中間（內容多為中文）。

use std::path::Path;

use serde_json::{json, Value};
use speclink_core::store::Store;

use crate::init_core_context;

/// snippet 於命中前後各保留的字元數（char，非 byte）。
const SNIPPET_CONTEXT: usize = 30;

/// 對看板全文搜尋：`{ "hits": [{ kind, id, artifact, snippet }] }`。
/// 空（或僅空白）query 與非專案目錄一律回傳空命中，不視為錯誤。
pub fn search_workspace_at(root: &Path, query: &str) -> Value {
    let q = query.trim();
    if q.is_empty() {
        return json!({ "hits": [] });
    }
    let Some(ctx) = init_core_context(root) else {
        return json!({ "hits": [] });
    };
    let store: &dyn Store = &ctx.store;
    let mut hits: Vec<Value> = Vec::new();

    // active 變更：artifacts 依固定序掃描，首個命中即代表該卡。
    for change in speclink_core::model::list_changes(store) {
        let mut artifacts = vec![
            "proposal.md".to_string(),
            "design.md".to_string(),
            "tasks.md".to_string(),
        ];
        for cap in store.delta_capabilities(&change.name) {
            artifacts.push(format!("specs/{cap}/spec.md"));
        }
        for artifact in artifacts {
            let Some(text) = store.read_artifact(&change.name, &artifact) else { continue };
            if let Some(snippet) = find_snippet(&text, q) {
                hits.push(json!({
                    "kind": "change",
                    "id": change.name,
                    "artifact": artifact,
                    "snippet": snippet,
                }));
                break;
            }
        }
    }

    // active 討論記錄全文。
    for doc in store.list_live_discussions() {
        if let Some(snippet) = find_snippet(&doc.text, q) {
            hits.push(json!({
                "kind": "discussion",
                "id": doc.slug,
                "artifact": doc
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                "snippet": snippet,
            }));
        }
    }

    json!({ "hits": hits })
}

/// char 層不分大小寫子字串搜尋：命中時回傳前後各 [`SNIPPET_CONTEXT`] 字元的
/// 裁切（含命中原文；截斷端補 …），未命中回 None。以 char 為單位比對與裁切，
/// 避開 to_lowercase 位移與多位元組邊界問題。
fn find_snippet(text: &str, query: &str) -> Option<String> {
    let hay: Vec<char> = text.chars().collect();
    let hay_lower: Vec<char> = hay.iter().map(lower_first).collect();
    let needle: Vec<char> = query.chars().map(|c| lower_first(&c)).collect();
    if needle.is_empty() || hay_lower.len() < needle.len() {
        return None;
    }
    let start = (0..=hay_lower.len() - needle.len())
        .find(|&i| hay_lower[i..i + needle.len()] == needle[..])?;
    let end = start + needle.len();
    let from = start.saturating_sub(SNIPPET_CONTEXT);
    let to = (end + SNIPPET_CONTEXT).min(hay.len());
    let mut snippet = String::new();
    if from > 0 {
        snippet.push('…');
    }
    // 換行壓成空白：snippet 是卡片上的單行呈現。
    snippet.extend(hay[from..to].iter().map(|c| if *c == '\n' || *c == '\r' { ' ' } else { *c }));
    if to < hay.len() {
        snippet.push('…');
    }
    Some(snippet)
}

/// char 層小寫化：取 to_lowercase 的首字元（ASCII 與 CJK 皆 1:1；罕見多字元
/// 展開取首字元即可——本查詢是輔助路徑，不追求 Unicode 完備摺疊）。
fn lower_first(c: &char) -> char {
    c.to_lowercase().next().unwrap_or(*c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testfixture::FixtureRoot;

    const META: &str = "schema: spec-driven\ncreated: 2026-07-01\ncreated_by: momo\ncreated_with: claude\n";

    fn write_discussion(fx: &FixtureRoot, slug: &str, body: &str) {
        fx.write(
            &format!("openspec/discussions/{slug}.md"),
            &format!("---\ntopic: t\nslug: {slug}\nstatus: open\ncreated: 2026-07-01\n---\n\n{body}\n"),
        );
    }

    #[test]
    fn hits_change_artifacts_case_insensitively_with_snippet() {
        let fx = FixtureRoot::new("s-change");
        fx.add_change("demo", META);
        fx.write(
            "openspec/changes/demo/design.md",
            "## Context\n\n唯一 Command Runtime 的 Dispatch 相容層設計。\n",
        );
        let v = search_workspace_at(fx.root(), "dispatch");
        let hits = v["hits"].as_array().expect("hits array");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0]["kind"], "change");
        assert_eq!(hits[0]["id"], "demo");
        assert_eq!(hits[0]["artifact"], "design.md");
        let snippet = hits[0]["snippet"].as_str().unwrap();
        assert!(snippet.contains("Dispatch"), "snippet 保留命中原文大小寫: {snippet}");
        assert!(snippet.contains("相容層"), "snippet 含命中後文: {snippet}");
    }

    #[test]
    fn first_matching_artifact_wins_one_hit_per_card() {
        // 每卡一命中：proposal 先於 design 掃描，兩者皆含關鍵字時取 proposal。
        let fx = FixtureRoot::new("s-first");
        fx.add_change("demo", META);
        fx.write("openspec/changes/demo/proposal.md", "## Why\n\nkeyword here\n");
        fx.write("openspec/changes/demo/design.md", "## Context\n\nkeyword again\n");
        let v = search_workspace_at(fx.root(), "keyword");
        let hits = v["hits"].as_array().unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0]["artifact"], "proposal.md");
    }

    #[test]
    fn hits_delta_specs_and_live_discussions() {
        let fx = FixtureRoot::new("s-spec-disc");
        fx.add_change("demo", META);
        fx.write(
            "openspec/changes/demo/specs/some-cap/spec.md",
            "## ADDED Requirements\n\n### Requirement: 浮層落點\n\n內容 zebra-token 在此。\n",
        );
        write_discussion(&fx, "talk", "## Rounds\n\n關於 zebra-token 的討論。");
        let v = search_workspace_at(fx.root(), "ZEBRA-TOKEN");
        let hits = v["hits"].as_array().unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0]["kind"], "change");
        assert_eq!(hits[0]["artifact"], "specs/some-cap/spec.md");
        assert_eq!(hits[1]["kind"], "discussion");
        assert_eq!(hits[1]["id"], "talk");
    }

    #[test]
    fn no_match_empty_query_and_non_project_all_return_empty_hits() {
        let fx = FixtureRoot::new("s-empty");
        fx.add_change("demo", META);
        assert!(search_workspace_at(fx.root(), "nonexistent-zzz")["hits"].as_array().unwrap().is_empty());
        assert!(search_workspace_at(fx.root(), "   ")["hits"].as_array().unwrap().is_empty());
        let non_project = std::env::temp_dir().join("speclink-search-nonproj");
        let _ = std::fs::create_dir_all(&non_project);
        assert!(search_workspace_at(&non_project, "x")["hits"].as_array().unwrap().is_empty());
    }

    #[test]
    fn snippet_truncates_around_cjk_match_without_panicking() {
        // 中文（多位元組）內容的長文命中：char 層裁切、兩端補 …，不 panic。
        let fx = FixtureRoot::new("s-cjk");
        fx.add_change("demo", META);
        let long = format!("{}目標詞{}", "前".repeat(80), "後".repeat(80));
        fx.write("openspec/changes/demo/design.md", &format!("## Context\n\n{long}\n"));
        let v = search_workspace_at(fx.root(), "目標詞");
        let snippet = v["hits"][0]["snippet"].as_str().unwrap().to_string();
        assert!(snippet.starts_with('…') && snippet.ends_with('…'), "兩端截斷補 …: {snippet}");
        assert!(snippet.contains("目標詞"));
        // 前後各約 30 字元＋命中 3 字＋兩個 …。
        assert!(snippet.chars().count() <= 30 + 3 + 30 + 2);
    }
}
