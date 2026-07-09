//! Change discovery, metadata, and artifact status.

use crate::schema::{Artifact, Schema};
use crate::store::Store;
use serde::Deserialize;
use std::path::PathBuf;

/// `.openspec.yaml` — per-change metadata.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ChangeMeta {
    pub schema: Option<String>,
    pub created: Option<String>,
    pub created_by: Option<String>,
    #[serde(default)]
    pub created_with: Option<String>,
    /// Slug of the discussion this change was promoted from (speclink extension).
    #[serde(default)]
    pub from_discussion: Option<String>,
    /// Discussions this change reflected (sealed) that were later re-concluded, so the
    /// change is stale relative to the new conclusion and needs re-ingest (speclink
    /// extension). Comma-separated slug accumulator: written by `discuss conclude` when
    /// it re-concludes an already-reflected discussion, cleared per-slug by `discuss
    /// seal`. Absent reads as empty — nothing pending.
    #[serde(default)]
    pub restale_from: Option<String>,
    /// The "started" lifecycle station (stamped by `in-progress add`). Absent
    /// on pre-migration metadata — such a change simply reads as not started.
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub started_by: Option<String>,
    #[serde(default)]
    pub started_with: Option<String>,
    /// 看板欄內排序鍵（speclink 桌面延伸；desktop-card-reorder）。缺席＝未排序，
    /// 卡片置欄頂、回退現行排序。ChangeMeta 僅 Deserialize，CLI 輸出不受影響。
    #[serde(default)]
    pub board_rank: Option<String>,
}

impl ChangeMeta {
    /// Parse a raw metadata document. A missing document or a parse error
    /// yields the defaults (a corrupt `.openspec.yaml` never breaks listing).
    pub fn from_text(text: Option<&str>) -> ChangeMeta {
        match text {
            Some(s) => serde_yaml::from_str(s).unwrap_or_default(),
            None => ChangeMeta::default(),
        }
    }
    pub fn schema_name(&self) -> String {
        self.schema
            .clone()
            .unwrap_or_else(|| "spec-driven".to_string())
    }

    /// The discussions this change was promoted from or linked to. `from_discussion`
    /// is a comma-separated accumulator (mirroring the discussion side's `promoted_to`)
    /// so a change can carry several source discussions — the first entry is its
    /// originating discussion. A single value is the degenerate one-element list;
    /// absent reads as empty.
    pub fn from_discussions(&self) -> Vec<String> {
        self.from_discussion
            .as_deref()
            .map(|v| {
                v.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The discussions this change reflected then went stale against (re-concluded after
    /// seal), read from the `restale_from` comma accumulator. Mirrors [`from_discussions`]:
    /// a single value is the one-element list, absent reads as empty.
    pub fn restale_from(&self) -> Vec<String> {
        self.restale_from
            .as_deref()
            .map(|v| {
                v.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// A discovered change.
#[derive(Debug, Clone)]
pub struct Change {
    pub name: String,
    /// Display location of the change's documents (rendered in payloads and
    /// human output; content access goes through the [`Store`]).
    pub dir: PathBuf,
    pub meta: ChangeMeta,
}

/// List active changes, sorted by name.
pub fn list_changes(store: &dyn Store) -> Vec<Change> {
    store.list_changes()
}

/// 寫入（或原位更新）一個 change 的看板排序鍵 `board_rank`。
/// 沿 started_* 的文字手術機制（read → 行代換或 append → write，永不重新序列化），
/// 其餘欄位逐位元組保留。非法 rank、非單一路徑段名稱、change 不存在皆回明確錯誤。
pub fn set_board_rank(store: &dyn Store, name: &str, rank: &str) -> anyhow::Result<()> {
    if !crate::util::is_valid_board_rank(rank) {
        anyhow::bail!("invalid board rank '{rank}' — lowercase ASCII letters only");
    }
    // 同 in-progress add 的防護：名稱必須是單一路徑段，否則可能經 raw 讀寫對
    // 觸及 changes/ 外的 metadata 文件。
    if name.contains(['/', '\\', ':']) || name.contains("..") {
        anyhow::bail!("invalid change name: {name}");
    }
    let Some(meta) = store.read_change_meta(name) else {
        anyhow::bail!("change not found: {name}");
    };
    let line = format!("board_rank: {rank}\n");
    let mut out = String::with_capacity(meta.len() + line.len());
    let mut replaced = false;
    for l in meta.split_inclusive('\n') {
        // 頂層鍵在第 0 欄；縮排行（巢狀值）不會誤中。
        if !replaced && l.starts_with("board_rank:") {
            out.push_str(&line);
            replaced = true;
        } else {
            out.push_str(l);
        }
    }
    if !replaced {
        if !out.ends_with('\n') && !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&line);
    }
    store.write_change_meta(name, &out)?;
    Ok(())
}

/// Find an active change by name.
pub fn find_change(store: &dyn Store, name: &str) -> Option<Change> {
    store.find_change(name)
}

/// Whether an artifact's output exists and has content.
pub fn artifact_done(store: &dyn Store, change: &str, artifact: &Artifact) -> bool {
    // Done-ness is EXISTS-based — an empty file counts (matches Spectra). A glob-style output
    // (e.g. "specs/**/*.md") is done when any matching delta spec exists.
    if artifact.output_path.contains("**") {
        return !store.delta_capabilities(change).is_empty();
    }
    store.artifact_exists(change, &artifact.output_path)
}

/// The artifact identifier of a capability's delta spec inside a change.
pub fn delta_spec_artifact(cap: &str) -> String {
    format!("specs/{cap}/spec.md")
}

/// Number of `### Requirement:` declarations under an ADDED/MODIFIED/REMOVED section. A RENAMED
/// section (FROM:/TO:) and empty operation headers contribute zero — matching Spectra's rule that
/// a delta spec must contain at least one applied operation.
pub fn op_requirement_count(text: &str) -> usize {
    let mut op = "";
    let mut count = 0;
    for line in text.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("## ") {
            if rest.trim_end().ends_with("Requirements") {
                op = rest.split_whitespace().next().unwrap_or("");
            }
        } else if t.starts_with("### Requirement:")
            && matches!(op, "ADDED" | "MODIFIED" | "REMOVED")
        {
            count += 1;
        }
    }
    count
}

/// A line-start `### Requirement:` that is not under an ADDED/MODIFIED/REMOVED section (a malformed
/// delta — requirement declared with no operation heading).
pub fn has_orphan_requirement(text: &str) -> bool {
    let mut op = "";
    for line in text.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("## ") {
            if rest.trim_end().ends_with("Requirements") {
                op = rest.split_whitespace().next().unwrap_or("");
            }
        } else if t.starts_with("### Requirement:") && !matches!(op, "ADDED" | "MODIFIED" | "REMOVED") {
            return true;
        }
    }
    false
}

/// Whether a delta spec body has an applicable operation (ADDED/MODIFIED/REMOVED with a requirement).
pub fn has_delta_operation(text: &str) -> bool {
    // Speclink divergence #4: a RENAMED section with at least one valid FROM/TO pair
    // counts as an operation, so a pure-rename delta validates and archives. (Spectra
    // documents RENAMED but treats rename-only deltas as invalid and never applies
    // renames at all.)
    op_requirement_count(text) > 0 || !rename_pairs(text).is_empty()
}

/// Rename pairs from `## RENAMED Requirements` sections (speclink divergence #4 —
/// Spectra parses but never applies renames). Both documented syntaxes:
/// - bullet form: `- FROM: `### Requirement: Old`` / `- TO: `### Requirement: New``
///   (bold markers and bare names accepted)
/// - header form: `### Requirement: Old` followed by a `TO: New` line
pub fn rename_pairs(text: &str) -> Vec<(String, String)> {
    fn req_name(raw: &str) -> String {
        let s = raw.trim().trim_matches('`').trim();
        s.strip_prefix("### Requirement:").map(str::trim).unwrap_or(s).to_string()
    }
    let mut out = Vec::new();
    let mut in_renamed = false;
    let mut from: Option<String> = None;
    for line in text.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("## ") {
            if rest.trim_end().ends_with("Requirements") {
                in_renamed = rest.split_whitespace().next() == Some("RENAMED");
                from = None;
                continue;
            }
        }
        if !in_renamed {
            continue;
        }
        if let Some(name) = t.strip_prefix("### Requirement:") {
            from = Some(name.trim().to_string());
            continue;
        }
        let norm = t.trim().trim_start_matches("- ").replace("**", "");
        if let Some(v) = norm.strip_prefix("FROM:") {
            let v = req_name(v);
            if !v.is_empty() {
                from = Some(v);
            }
        } else if let Some(v) = norm.strip_prefix("TO:") {
            let v = req_name(v);
            if let (Some(f), false) = (from.take(), v.is_empty()) {
                out.push((f, v));
            }
        }
    }
    out
}

/// DAG status of a single artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactStatus {
    Done,
    Ready,
    Blocked,
}

impl ArtifactStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArtifactStatus::Done => "done",
            ArtifactStatus::Ready => "ready",
            ArtifactStatus::Blocked => "blocked",
        }
    }
}

/// Compute the status of every artifact in the schema for a change.
pub fn artifact_statuses(schema: &Schema, store: &dyn Store, change: &str) -> Vec<(String, ArtifactStatus)> {
    // First pass: done-ness.
    let mut done: std::collections::HashMap<&str, bool> = std::collections::HashMap::new();
    for a in &schema.artifacts {
        done.insert(a.id.as_str(), artifact_done(store, change, a));
    }
    // Second pass: ready/blocked based on requires.
    let mut out = Vec::new();
    for a in &schema.artifacts {
        let status = if *done.get(a.id.as_str()).unwrap_or(&false) {
            ArtifactStatus::Done
        } else if a.requires.iter().all(|r| *done.get(r.as_str()).unwrap_or(&false)) {
            ArtifactStatus::Ready
        } else {
            ArtifactStatus::Blocked
        };
        out.push((a.id.clone(), status));
    }
    out
}

/// Which artifact ids block a given artifact (unmet requires).
pub fn blocked_by(schema: &Schema, store: &dyn Store, change: &str, id: &str) -> Vec<String> {
    let Some(a) = schema.artifact(id) else {
        return Vec::new();
    };
    a.requires
        .iter()
        .filter(|r| {
            schema
                .artifact(r)
                .map(|ra| !artifact_done(store, change, ra))
                .unwrap_or(false)
        })
        .map(|s| s.to_string())
        .collect()
}

/// Whether EVERY artifact in the schema is done (matches Spectra — an absent optional artifact
/// such as `design` keeps the change incomplete).
pub fn is_complete(schema: &Schema, store: &dyn Store, change: &str) -> bool {
    schema
        .artifacts
        .iter()
        .all(|a| artifact_done(store, change, a))
}

/// Artifacts in the schema that are not yet done (used for analyze "Missing" reporting).
pub fn missing_artifacts(schema: &Schema, store: &dyn Store, change: &str) -> Vec<String> {
    schema
        .artifacts
        .iter()
        .filter(|a| !artifact_done(store, change, a))
        .map(|a| a.id.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::ChangeMeta;
    use crate::teststore::TestStore;

    #[test]
    fn meta_parses_board_rank_and_absent_reads_as_none() {
        // 看板排序欄位（speclink 桌面延伸）：含 board_rank 的 meta 正常解析，
        // 舊 meta（無此欄位）讀為 None——排序回退現行規則。
        let meta = ChangeMeta::from_text(Some("schema: spec-driven\nboard_rank: n\n"));
        assert_eq!(meta.board_rank.as_deref(), Some("n"));
        let old = ChangeMeta::from_text(Some("schema: spec-driven\ncreated: 2026-07-01\n"));
        assert!(old.board_rank.is_none());
    }

    const STAMPED_META: &str = "schema: spec-driven\ncreated: 2026-07-01\ncreated_by: Base Line <base@example.com>\ncreated_with: claude\nstarted_at: 2026-07-03\nstarted_by: Worker <w@example.com>\nstarted_with: claude\n";

    #[test]
    fn set_board_rank_appends_and_preserves_existing_fields_verbatim() {
        // spec「meta 寫入路徑對 board_rank 互不破壞」：寫回除 board_rank 外
        // 逐位元組保留（沿 started_* 的 read → append → write 機制）。
        let store = TestStore::with_meta("demo", STAMPED_META);
        super::set_board_rank(&store, "demo", "n").unwrap();
        let meta = store.meta("demo");
        assert!(
            meta.starts_with(STAMPED_META),
            "existing fields must be preserved byte-for-byte, got: {meta}"
        );
        assert_eq!(&meta[STAMPED_META.len()..], "board_rank: n\n");
        assert_eq!(
            ChangeMeta::from_text(Some(&meta)).board_rank.as_deref(),
            Some("n")
        );
    }

    #[test]
    fn set_board_rank_replaces_existing_line_in_place() {
        // 更新既有 rank：原行原位代換（欄位順序保留），其餘逐位元組不變。
        let store = TestStore::with_meta(
            "demo",
            "schema: spec-driven\nboard_rank: b\ncreated: 2026-07-01\n",
        );
        super::set_board_rank(&store, "demo", "abn").unwrap();
        assert_eq!(
            store.meta("demo"),
            "schema: spec-driven\nboard_rank: abn\ncreated: 2026-07-01\n"
        );
    }

    #[test]
    fn set_board_rank_handles_meta_missing_trailing_newline() {
        let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01");
        super::set_board_rank(&store, "demo", "n").unwrap();
        assert_eq!(
            store.meta("demo"),
            "schema: spec-driven\ncreated: 2026-07-01\nboard_rank: n\n"
        );
    }

    #[test]
    fn set_board_rank_rejects_invalid_values_without_writing() {
        // 系統邊界驗證（sharp edge）：rank 僅小寫英文字母——空值、大寫、數字、
        // 空白與換行注入一律拒絕且零寫入（換行會注入任意 YAML 欄位）。
        let store = TestStore::with_meta("demo", "schema: spec-driven\n");
        for bad in ["", "N", "a1", "a b", "a\nstarted_at: forged", "ranké"] {
            assert!(
                super::set_board_rank(&store, "demo", bad).is_err(),
                "invalid rank {bad:?} must be rejected"
            );
        }
        assert_eq!(*store.meta_writes.borrow(), 0, "invalid ranks must not write");
    }

    #[test]
    fn set_board_rank_errors_on_missing_change_and_unsafe_names() {
        // 不存在的 change 回明確錯誤（桌面以單行錯誤呈現，不靜默）；
        // 非單一路徑段名稱拒絕（沿 in-progress add 的同款防護）。
        let store = TestStore::with_meta("demo", "schema: spec-driven\n");
        assert!(super::set_board_rank(&store, "ghost", "n").is_err());
        assert!(super::set_board_rank(&store, "../evil", "n").is_err());
        assert!(super::set_board_rank(&store, "a/b", "n").is_err());
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn meta_without_started_fields_parses_as_not_started() {
        // Backward compatibility: pre-migration documents (no started_*) keep
        // parsing without warnings and read as "not started".
        let old = "schema: spec-driven\ncreated: 2026-07-01\ncreated_by: Base Line <base@example.com>\n";
        let meta = ChangeMeta::from_text(Some(old));
        assert_eq!(meta.schema.as_deref(), Some("spec-driven"));
        assert!(meta.started_at.is_none());
        assert!(meta.started_by.is_none());
        assert!(meta.started_with.is_none());
    }

    #[test]
    fn meta_with_started_fields_parses_all_three_stations() {
        let text = "schema: spec-driven\ncreated: 2026-07-01\nstarted_at: 2026-07-03\nstarted_by: Worker <w@example.com>\nstarted_with: claude\n";
        let meta = ChangeMeta::from_text(Some(text));
        assert_eq!(meta.started_at.as_deref(), Some("2026-07-03"));
        assert_eq!(meta.started_by.as_deref(), Some("Worker <w@example.com>"));
        assert_eq!(meta.started_with.as_deref(), Some("claude"));
    }

    // --- from_discussion 累積器讀取（design D1；M↔N 關係）---

    #[test]
    fn from_discussions_absent_yields_empty() {
        let meta = ChangeMeta::from_text(Some("schema: spec-driven\ncreated: 2026-07-01\n"));
        assert!(meta.from_discussions().is_empty());
    }

    #[test]
    fn from_discussions_single_value() {
        let meta = ChangeMeta::from_text(Some("schema: spec-driven\nfrom_discussion: alpha-search\n"));
        assert_eq!(meta.from_discussions(), vec!["alpha-search".to_string()]);
    }

    #[test]
    fn from_discussions_comma_accumulated_values() {
        // 逗號清單依 meta 順序切分、項目前後空白修剪（沿 promoted_to 同款）。
        let meta = ChangeMeta::from_text(Some(
            "schema: spec-driven\nfrom_discussion: alpha-search, beta-cache\n",
        ));
        assert_eq!(
            meta.from_discussions(),
            vec!["alpha-search".to_string(), "beta-cache".to_string()]
        );
    }

    // --- restale_from 讀取（design D3/D6；spec change-lifecycle）---

    #[test]
    fn restale_from_absent_yields_empty() {
        let meta = ChangeMeta::from_text(Some("schema: spec-driven\ncreated: 2026-07-01\n"));
        assert!(meta.restale_from().is_empty());
    }

    #[test]
    fn restale_from_single_value() {
        let meta = ChangeMeta::from_text(Some("schema: spec-driven\nrestale_from: alpha-search\n"));
        assert_eq!(meta.restale_from(), vec!["alpha-search".to_string()]);
    }

    #[test]
    fn restale_from_comma_accumulated_values() {
        // spec change-lifecycle Example：逗號多值 trim 後分割。
        let meta = ChangeMeta::from_text(Some(
            "schema: spec-driven\nrestale_from: alpha-search, beta-cache\n",
        ));
        assert_eq!(
            meta.restale_from(),
            vec!["alpha-search".to_string(), "beta-cache".to_string()]
        );
    }
}
