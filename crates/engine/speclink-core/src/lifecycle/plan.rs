//! Change-level execution order (change-plan design D1): a total basis order
//! (stage → board rank → created → name), then a greedy topological pass that
//! places every active change into a wave. Declared prerequisites
//! (`depends_on`) are edges; delta-capability overlap is mutual exclusion
//! decided by the basis order, never an edge — so a cycle can only come from
//! `depends_on`. Corrupt metadata is reported in `skipped` and takes no part
//! in placement or overlap. Read-only: nothing here writes.

use crate::model::{self, Change, Stage};
use crate::store::Store;
use serde::Serialize;
use std::collections::BTreeMap;

/// One wave: the changes that may run in parallel, in placement order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Wave {
    /// 1-based.
    pub index: usize,
    pub changes: Vec<String>,
}

/// One change's row of the plan (design D3 field order).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanChange {
    pub name: String,
    pub wave: usize,
    pub stage: Stage,
    /// The meta's declaration verbatim — archived or unknown names included.
    pub depends_on: Vec<String>,
    /// Every other placed change sharing a delta capability, in placement order.
    pub overlaps: Vec<Overlap>,
    /// Active declared prerequisites ∪ overlapping changes placed earlier,
    /// deduplicated, in placement order.
    pub blocked_by: Vec<String>,
    /// `blocked_by` is empty.
    pub ready: bool,
}

/// A delta-capability overlap with one other change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Overlap {
    pub change: String,
    /// Shared capability names, ascending.
    pub capabilities: Vec<String>,
}

/// A change left out of the plan because its metadata cannot be parsed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Skipped {
    pub change: String,
    pub reason: String,
}

/// The `speclink plan --json` payload (design D3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Plan {
    pub waves: Vec<Wave>,
    /// Every placed change in placement order.
    pub changes: Vec<PlanChange>,
    /// The first proposed change with nothing blocking it.
    pub next: Option<String>,
    pub skipped: Vec<Skipped>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    /// The `depends_on` graph has a cycle; the path starts and ends on the
    /// same change (`a -> b -> a`).
    Cycle(Vec<String>),
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanError::Cycle(path) => write!(f, "dependency cycle: {}", path.join(" -> ")),
        }
    }
}

impl std::error::Error for PlanError {}

/// Plan the active changes with each change's own `board_rank` as its rank.
pub fn compute(store: &dyn Store) -> Result<Plan, PlanError> {
    let changes = model::list_changes(store);
    let ranks = meta_ranks(&changes);
    compute_from(store, changes, &ranks)
}

/// Each change's own `board_rank`, keyed by name; unranked changes are absent.
fn meta_ranks(changes: &[Change]) -> BTreeMap<String, String> {
    changes
        .iter()
        .filter_map(|c| c.meta.board_rank.clone().map(|r| (c.name.clone(), r)))
        .collect()
}

/// Plan the active changes with an external rank table (a caller whose ranks
/// live outside the change meta); a change absent from `ranks` is unranked.
pub fn compute_with_ranks(
    store: &dyn Store,
    ranks: &BTreeMap<String, String>,
) -> Result<Plan, PlanError> {
    compute_from(store, model::list_changes(store), ranks)
}

/// The `depends_on` list of a change after a `set_depends` write, plus whether
/// this call changed the document (false for the idempotent pass — no event).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependsWrite {
    pub depends_on: Vec<String>,
    pub changed: bool,
}

/// A name that cannot be a single directory entry under changes/ — never an
/// active change, and never allowed near the raw meta read/write pair.
fn unsafe_name(name: &str) -> bool {
    name.trim().is_empty() || name.contains(['/', '\\', ':']) || name.contains("..")
}

/// Declare (or with `remove`, undeclare) `on` as prerequisites of `name`
/// (change-plan design D4). Guards in order — the target must be an active
/// change; when adding, every prerequisite must be active (an archived or
/// unknown name is told apart), while a remove may clear a leftover that
/// points at an archived or unknown name; a change cannot depend on itself;
/// and, when adding, no cycle may run through the edited change (a cycle
/// elsewhere is `plan`'s to report). Names are matched exactly against the
/// active roster — the store's existence probes follow the file system's case
/// rules, the plan's placement does not. Any refusal writes nothing. The write
/// is one [`crate::model::edit_meta`] call that pushes or removes every name
/// in turn, so the document changes once or not at all; an edge already
/// present (or already absent) is an idempotent success.
pub fn set_depends(
    store: &dyn Store,
    name: &str,
    on: &[String],
    remove: bool,
) -> anyhow::Result<DependsWrite> {
    let changes = model::list_changes(store);
    let active = |n: &str| changes.iter().any(|c| c.name == n);
    if unsafe_name(name) || !active(name) {
        anyhow::bail!("Change '{name}' not found.");
    }
    let refuse = |msg: String| anyhow::Error::from(crate::command::Refusal(msg));
    for other in on {
        if unsafe_name(other) || (!remove && !active(other)) {
            let archived = store
                .list_archived_changes()
                .iter()
                .any(|dated| crate::util::strip_date_prefix(dated) == other);
            return Err(refuse(if archived {
                format!("cannot depend on '{other}': it is already archived")
            } else {
                format!("cannot depend on '{other}': no active change with that name")
            }));
        }
        if other == name {
            return Err(refuse(format!("'{name}' cannot depend on itself")));
        }
    }
    if !remove {
        let ranks = meta_ranks(&changes);
        let (mut entries, _) = entries_of(store, changes, &ranks);
        // A corrupt target is no entry: nothing to walk, and `edit_meta`
        // below refuses it fail-closed.
        if let Some(target) = entries.iter().position(|e| e.name == name) {
            entries[target].depends_on.extend(on.iter().cloned());
            let deps = deps_of(&entries);
            if let Some(path) = cycle_through(&entries, &deps, target) {
                return Err(refuse(PlanError::Cycle(path).to_string()));
            }
        }
    }
    let write = model::edit_meta(store, name, |m| {
        let before = m.list("depends_on");
        for other in on {
            if remove {
                m.remove_list("depends_on", other)?;
            } else {
                m.push_list("depends_on", other)?;
            }
        }
        let depends_on = m.list("depends_on");
        Ok(DependsWrite {
            changed: depends_on != before,
            depends_on,
        })
    })?;
    // The roster listed a directory, so a missing document is a change with no
    // `.openspec.yaml` at all — nothing to append the line to.
    write.ok_or_else(|| {
        anyhow::anyhow!(
            "cannot write depends_on for '{name}': openspec/changes/{name}/.openspec.yaml is missing"
        )
    })
}

/// The cycle `start` sits on, if any: depth-first along the declared
/// prerequisites in declaration order until the walk comes back to `start`.
/// The path reads from the edited change (`c -> a -> c`); a cycle that does
/// not pass through `start` is not found here.
fn cycle_through(entries: &[Entry], deps: &[Vec<usize>], start: usize) -> Option<Vec<String>> {
    fn walk(
        here: usize,
        start: usize,
        deps: &[Vec<usize>],
        path: &mut Vec<usize>,
        seen: &mut [bool],
    ) -> bool {
        for &d in &deps[here] {
            if d == start {
                return true;
            }
            if !seen[d] {
                seen[d] = true;
                path.push(d);
                if walk(d, start, deps, path, seen) {
                    return true;
                }
                path.pop();
            }
        }
        false
    }
    let mut path = vec![start];
    let mut seen = vec![false; entries.len()];
    seen[start] = true;
    walk(start, start, deps, &mut path, &mut seen).then(|| {
        path.push(start);
        path.iter().map(|&i| entries[i].name.clone()).collect()
    })
}

/// A placeable change with everything the placement needs, resolved once.
struct Entry {
    name: String,
    stage: Stage,
    rank: Option<String>,
    created: Option<String>,
    depends_on: Vec<String>,
    capabilities: Vec<String>,
}

impl Entry {
    /// The basis order key (design D1): stage; ranked before unranked, ranks by
    /// bytes; unranked by created with missing created last; name decides ties.
    fn key(&self) -> (Stage, u8, &str, u8, &str, &str) {
        match (&self.rank, &self.created) {
            (Some(rank), _) => (self.stage, 0, rank, 0, "", &self.name),
            (None, Some(created)) => (self.stage, 1, "", 0, created, &self.name),
            (None, None) => (self.stage, 1, "", 1, "", &self.name),
        }
    }
}

/// The placeable entries in basis order plus the skipped (corrupt-meta) ones.
fn entries_of(
    store: &dyn Store,
    changes: Vec<Change>,
    ranks: &BTreeMap<String, String>,
) -> (Vec<Entry>, Vec<Skipped>) {
    let mut skipped = Vec::new();
    let mut entries = Vec::new();
    for c in changes {
        if let Some(reason) = &c.meta_error {
            skipped.push(Skipped {
                change: c.name.clone(),
                reason: reason.clone(),
            });
            continue;
        }
        entries.push(Entry {
            stage: model::stage(store, &c),
            rank: ranks.get(&c.name).cloned(),
            created: c.meta.created.clone(),
            depends_on: c.meta.depends_on(),
            capabilities: store.delta_capabilities(&c.name),
            name: c.name,
        });
    }
    entries.sort_by(|a, b| a.key().cmp(&b.key()));
    (entries, skipped)
}

/// Edges by entry index: declared prerequisites that are themselves placeable.
/// Anything else — archived, unknown, corrupt, or the change itself — is
/// satisfied, and a repeated name is one edge.
fn deps_of(entries: &[Entry]) -> Vec<Vec<usize>> {
    let position: BTreeMap<&str, usize> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| (e.name.as_str(), i))
        .collect();
    entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let mut out: Vec<usize> = Vec::new();
            for d in &e.depends_on {
                if let Some(&j) = position.get(d.as_str()) {
                    if j != i && !out.contains(&j) {
                        out.push(j);
                    }
                }
            }
            out
        })
        .collect()
}

fn shared(a: &Entry, b: &Entry) -> Vec<String> {
    a.capabilities
        .iter()
        .filter(|cap| b.capabilities.contains(cap))
        .cloned()
        .collect()
}

/// Greedy placement along the basis order (design D1): repeatedly take the
/// first entry whose prerequisites are all placed. Returns the placement
/// order and each entry's wave, or the cycle that stalled it.
fn place(entries: &[Entry], deps: &[Vec<usize>]) -> Result<(Vec<usize>, Vec<usize>), PlanError> {
    let mut wave_of: Vec<Option<usize>> = vec![None; entries.len()];
    let mut order: Vec<usize> = Vec::new();
    while order.len() < entries.len() {
        let pick = (0..entries.len())
            .find(|&i| wave_of[i].is_none() && deps[i].iter().all(|&d| wave_of[d].is_some()));
        let Some(i) = pick else {
            return Err(PlanError::Cycle(cycle(entries, deps, &wave_of)));
        };
        let mut wave = 1;
        for &d in &deps[i] {
            wave = wave.max(wave_of[d].expect("placed") + 1);
        }
        for &j in &order {
            if !shared(&entries[i], &entries[j]).is_empty() {
                wave = wave.max(wave_of[j].expect("placed") + 1);
            }
        }
        wave_of[i] = Some(wave);
        order.push(i);
    }
    let waves = wave_of
        .into_iter()
        .map(|w| w.expect("every entry placed"))
        .collect();
    Ok((order, waves))
}

fn compute_from(
    store: &dyn Store,
    changes: Vec<Change>,
    ranks: &BTreeMap<String, String>,
) -> Result<Plan, PlanError> {
    let (entries, skipped) = entries_of(store, changes, ranks);
    let deps = deps_of(&entries);
    let (order, wave_of) = place(&entries, &deps)?;

    let placed_at = |i: usize| order.iter().position(|&o| o == i).expect("placed");
    let mut plan_changes = Vec::new();
    for &i in &order {
        let overlaps: Vec<Overlap> = order
            .iter()
            .filter(|&&j| j != i)
            .filter_map(|&j| {
                let capabilities = shared(&entries[i], &entries[j]);
                (!capabilities.is_empty()).then(|| Overlap {
                    change: entries[j].name.clone(),
                    capabilities,
                })
            })
            .collect();
        let mut blockers: Vec<usize> = deps[i].clone();
        for &j in order.iter().take(placed_at(i)) {
            if !blockers.contains(&j) && !shared(&entries[i], &entries[j]).is_empty() {
                blockers.push(j);
            }
        }
        blockers.sort_by_key(|&j| placed_at(j));
        let blocked_by: Vec<String> = blockers.iter().map(|&j| entries[j].name.clone()).collect();
        plan_changes.push(PlanChange {
            name: entries[i].name.clone(),
            wave: wave_of[i],
            stage: entries[i].stage,
            depends_on: entries[i].depends_on.clone(),
            overlaps,
            ready: blocked_by.is_empty(),
            blocked_by,
        });
    }
    let last_wave = plan_changes.iter().map(|c| c.wave).max().unwrap_or(0);
    let waves = (1..=last_wave)
        .map(|index| Wave {
            index,
            changes: plan_changes
                .iter()
                .filter(|c| c.wave == index)
                .map(|c| c.name.clone())
                .collect(),
        })
        .collect();
    let next = plan_changes
        .iter()
        .find(|c| c.stage == Stage::Proposed && c.ready)
        .map(|c| c.name.clone());
    Ok(Plan {
        waves,
        changes: plan_changes,
        next,
        skipped,
    })
}

/// Every (dependent, prerequisite) pair of `depends_on` whose dependent sits
/// before its prerequisite in `sequence` (design D5). Only names present in
/// the sequence take part: a prerequisite in another stage or the archive
/// cannot be "crossed" here. Pure — callers whose ranks live elsewhere hand in
/// their own sequence.
pub fn violations(
    sequence: &[&str],
    depends_on: &BTreeMap<String, Vec<String>>,
) -> Vec<(String, String)> {
    let position = |name: &str| sequence.iter().position(|s| *s == name);
    let mut out = Vec::new();
    for (i, dependent) in sequence.iter().enumerate() {
        let Some(prereqs) = depends_on.get(*dependent) else {
            continue;
        };
        for prereq in prereqs {
            if position(prereq).is_some_and(|p| p > i) {
                out.push((dependent.to_string(), prereq.clone()));
            }
        }
    }
    out
}

/// A rank move that would cross a declared dependency (design D5): the moved
/// change would sit before a prerequisite (`must_follow`) or after a change
/// that depends on it (`must_precede`). Display is the one-line human message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankMoveBlocked {
    pub change: String,
    pub must_follow: Vec<String>,
    pub must_precede: Vec<String>,
}

impl std::fmt::Display for RankMoveBlocked {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cannot move '{}' there:", self.change)?;
        if !self.must_follow.is_empty() {
            write!(f, " it depends on {}", self.must_follow.join(", "))?;
        }
        if !self.must_follow.is_empty() && !self.must_precede.is_empty() {
            write!(f, ";")?;
        }
        if !self.must_precede.is_empty() {
            write!(f, " {} depend on it", self.must_precede.join(", "))?;
        }
        Ok(())
    }
}

impl std::error::Error for RankMoveBlocked {}

/// Would writing `rank` as `name`'s board rank cross a declared dependency?
/// Re-sorts the change's own stage by the basis order (the same [`entries_of`]
/// the plan uses) with the new rank in place and asks [`violations`]. Reads
/// only — the write (and the rank and name validation that comes with it)
/// stays with `set_board_rank`; an unknown or corrupt `name` has nothing to
/// cross and passes.
pub fn check_rank_move(store: &dyn Store, name: &str, rank: &str) -> Result<(), RankMoveBlocked> {
    let changes = model::list_changes(store);
    let mut ranks = meta_ranks(&changes);
    ranks.insert(name.to_string(), rank.to_string());
    let (entries, _) = entries_of(store, changes, &ranks);
    let Some(moved) = entries.iter().find(|e| e.name == name) else {
        return Ok(());
    };
    let column: Vec<&Entry> = entries.iter().filter(|e| e.stage == moved.stage).collect();
    let sequence: Vec<&str> = column.iter().map(|e| e.name.as_str()).collect();
    let depends_on: BTreeMap<String, Vec<String>> = column
        .iter()
        .map(|e| (e.name.clone(), e.depends_on.clone()))
        .collect();
    let mut must_follow = Vec::new();
    let mut must_precede = Vec::new();
    for (dependent, prereq) in violations(&sequence, &depends_on) {
        if dependent == name && !must_follow.contains(&prereq) {
            must_follow.push(prereq);
        } else if prereq == name && !must_precede.contains(&dependent) {
            must_precede.push(dependent);
        }
    }
    if must_follow.is_empty() && must_precede.is_empty() {
        return Ok(());
    }
    Err(RankMoveBlocked {
        change: name.to_string(),
        must_follow,
        must_precede,
    })
}

/// The cycle the stalled placement ran into: from the first unplaced change,
/// follow the first unplaced prerequisite until a change repeats — every
/// unplaced change has one, or the placement would not have stalled.
fn cycle(entries: &[Entry], deps: &[Vec<usize>], wave_of: &[Option<usize>]) -> Vec<String> {
    let start = (0..entries.len())
        .find(|&i| wave_of[i].is_none())
        .expect("stalled");
    let mut path = vec![start];
    loop {
        let here = *path.last().expect("non-empty");
        let next = deps[here]
            .iter()
            .copied()
            .find(|&d| wave_of[d].is_none())
            .expect("an unplaced change has an unplaced prerequisite");
        if let Some(first) = path.iter().position(|&p| p == next) {
            let mut names: Vec<String> = path[first..]
                .iter()
                .map(|&p| entries[p].name.clone())
                .collect();
            names.push(entries[next].name.clone());
            return names;
        }
        path.push(next);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::teststore::TestStore;

    /// 多 change 的 store：每項 (名稱, meta 原文)。
    fn store_of(changes: &[(&str, &str)]) -> TestStore {
        let store = TestStore::default();
        for (name, meta) in changes {
            store
                .metas
                .borrow_mut()
                .insert(name.to_string(), meta.to_string());
        }
        store
    }

    /// 給 change 一份指定 capability 的 delta spec。
    fn delta(store: &TestStore, change: &str, cap: &str) {
        store.put_artifact(
            change,
            &format!("specs/{cap}/spec.md"),
            "## ADDED Requirements\n",
        );
    }

    fn names(plan: &Plan) -> Vec<&str> {
        plan.changes.iter().map(|c| c.name.as_str()).collect()
    }

    fn waves(plan: &Plan) -> Vec<Vec<&str>> {
        plan.waves
            .iter()
            .map(|w| w.changes.iter().map(String::as_str).collect())
            .collect()
    }

    fn entry<'a>(plan: &'a Plan, name: &str) -> &'a PlanChange {
        plan.changes
            .iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("{name} missing from plan: {:?}", names(plan)))
    }

    const PROPOSED: &str = "schema: spec-driven\ncreated: 2026-09-01\n";

    // --- 基底順序三鍵（spec「執行順序的基底與拓樸修正」）---

    #[test]
    fn basis_order_puts_ready_before_in_progress_before_proposed() {
        // 階段優先於一切：created 刻意反向（提案中最早、已就緒最晚）。
        let store = store_of(&[
            ("proposed", "schema: spec-driven\ncreated: 2026-09-01\n"),
            (
                "started",
                "schema: spec-driven\ncreated: 2026-09-05\nstarted_at: 2026-09-06\n",
            ),
            ("ready", "schema: spec-driven\ncreated: 2026-09-09\n"),
        ]);
        store.put_artifact("ready", "tasks.md", "- [x] 1.1 a\n");
        let plan = compute(&store).unwrap();
        assert_eq!(names(&plan), ["ready", "started", "proposed"]);
        assert_eq!(entry(&plan, "ready").stage, Stage::Ready);
        assert_eq!(entry(&plan, "started").stage, Stage::InProgress);
        assert_eq!(entry(&plan, "proposed").stage, Stage::Proposed);
    }

    #[test]
    fn basis_order_ranked_changes_precede_unranked_and_sort_by_rank_bytes() {
        // 同階段：有 rank 的在前、rank 位元組字典序升冪；缺 rank 者即使 created 最早也排後。
        let store = store_of(&[
            (
                "rank-n",
                "schema: spec-driven\ncreated: 2026-09-01\nboard_rank: n\n",
            ),
            (
                "rank-f",
                "schema: spec-driven\ncreated: 2026-09-02\nboard_rank: f\n",
            ),
            ("no-rank", "schema: spec-driven\ncreated: 2026-08-01\n"),
        ]);
        let plan = compute(&store).unwrap();
        assert_eq!(names(&plan), ["rank-f", "rank-n", "no-rank"]);
    }

    #[test]
    fn basis_order_unranked_changes_sort_by_created_then_missing_created_last() {
        let store = store_of(&[
            ("late", "schema: spec-driven\ncreated: 2026-09-10\n"),
            ("early", "schema: spec-driven\ncreated: 2026-09-02\n"),
            ("undated", "schema: spec-driven\n"),
        ]);
        let plan = compute(&store).unwrap();
        assert_eq!(names(&plan), ["early", "late", "undated"]);
    }

    #[test]
    fn basis_order_breaks_ties_by_name() {
        // 同 rank 與同 created 各自以名稱決斷，順序為全序。
        let store = store_of(&[
            ("b-ranked", "schema: spec-driven\nboard_rank: n\n"),
            ("a-ranked", "schema: spec-driven\nboard_rank: n\n"),
            ("d-dated", "schema: spec-driven\ncreated: 2026-09-01\n"),
            ("c-dated", "schema: spec-driven\ncreated: 2026-09-01\n"),
        ]);
        let plan = compute(&store).unwrap();
        assert_eq!(names(&plan), ["a-ranked", "b-ranked", "c-dated", "d-dated"]);
    }

    // --- 拓樸修正 ---

    #[test]
    fn no_deps_no_overlap_all_land_in_the_first_wave() {
        // Scenario 無依賴無重疊時全部同波。
        let store = store_of(&[("a", PROPOSED), ("b", PROPOSED), ("c", PROPOSED)]);
        delta(&store, "a", "cap-a");
        delta(&store, "b", "cap-b");
        delta(&store, "c", "cap-c");
        let plan = compute(&store).unwrap();
        assert_eq!(waves(&plan), [vec!["a", "b", "c"]]);
        assert_eq!(plan.waves[0].index, 1);
        assert!(plan
            .changes
            .iter()
            .all(|c| c.wave == 1 && c.ready && c.blocked_by.is_empty()));
    }

    #[test]
    fn declared_dependency_pushes_the_dependent_a_wave_later() {
        // Scenario 宣告依賴推後波次。
        let store = store_of(&[
            ("a", PROPOSED),
            (
                "c",
                "schema: spec-driven\ncreated: 2026-09-01\ndepends_on: a\n",
            ),
        ]);
        let plan = compute(&store).unwrap();
        let a = entry(&plan, "a");
        let c = entry(&plan, "c");
        assert_eq!(c.wave, a.wave + 1);
        assert_eq!(c.blocked_by, ["a"]);
        assert_eq!(c.depends_on, ["a"]);
        assert!(!c.ready);
        assert_eq!(waves(&plan), [vec!["a"], vec!["c"]]);
    }

    #[test]
    fn overlap_is_mutual_exclusion_ordered_by_basis() {
        // Scenario 重疊互斥依基底順序定先後：a 與 b 皆動 desktop-app，a 基底在前。
        let store = store_of(&[
            ("a", "schema: spec-driven\ncreated: 2026-09-01\n"),
            ("b", "schema: spec-driven\ncreated: 2026-09-02\n"),
        ]);
        delta(&store, "a", "desktop-app");
        delta(&store, "b", "desktop-app");
        delta(&store, "b", "tray-status-menu");
        let plan = compute(&store).unwrap();
        let a = entry(&plan, "a");
        let b = entry(&plan, "b");
        assert_eq!((a.wave, b.wave), (1, 2));
        assert_eq!(b.blocked_by, ["a"]);
        assert!(a.blocked_by.is_empty());
        assert!(a.ready && !b.ready);
        let overlap = |c: &PlanChange| {
            c.overlaps
                .iter()
                .map(|o| (o.change.clone(), o.capabilities.clone()))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            overlap(a),
            [("b".to_string(), vec!["desktop-app".to_string()])]
        );
        assert_eq!(
            overlap(b),
            [("a".to_string(), vec!["desktop-app".to_string()])]
        );
    }

    #[test]
    fn dependency_on_an_archived_change_counts_as_satisfied() {
        // Scenario 指向已封存的依賴視為滿足：dependsOn 仍原文列出。
        let store = store_of(&[(
            "c",
            "schema: spec-driven\ncreated: 2026-09-01\ndepends_on: old-change\n",
        )]);
        store
            .archived_metas
            .borrow_mut()
            .insert("2026-08-01-old-change".to_string(), PROPOSED.to_string());
        let plan = compute(&store).unwrap();
        let c = entry(&plan, "c");
        assert_eq!(c.wave, 1);
        assert!(c.blocked_by.is_empty() && c.ready);
        assert_eq!(c.depends_on, ["old-change"]);
    }

    #[test]
    fn a_started_change_places_before_an_unstarted_one_created_earlier() {
        // Scenario 已開工者排在未開工者之前：b 較早建立，但 a 已開工。
        let store = store_of(&[
            (
                "a",
                "schema: spec-driven\ncreated: 2026-09-05\nstarted_at: 2026-09-06\n",
            ),
            ("b", "schema: spec-driven\ncreated: 2026-09-01\n"),
        ]);
        delta(&store, "a", "desktop-app");
        delta(&store, "b", "desktop-app");
        let plan = compute(&store).unwrap();
        assert_eq!(names(&plan), ["a", "b"]);
        assert_eq!(entry(&plan, "b").blocked_by, ["a"]);
    }

    #[test]
    fn spec_example_five_changes() {
        // Example「五個 change 的基底順序與波次」逐列。
        let store = store_of(&[
            ("r1", "schema: spec-driven\ncreated: 2026-09-01\n"),
            (
                "p2",
                "schema: spec-driven\ncreated: 2026-09-03\nboard_rank: n\nstarted_at: 2026-09-04\n",
            ),
            (
                "n5",
                "schema: spec-driven\ncreated: 2026-09-02\nboard_rank: f\n",
            ),
            ("n3", "schema: spec-driven\ncreated: 2026-09-10\n"),
            (
                "n4",
                "schema: spec-driven\ncreated: 2026-09-12\ndepends_on: n3\n",
            ),
        ]);
        store.put_artifact("r1", "tasks.md", "- [x] 1.1 a\n- [x] 1.2 b\n");
        delta(&store, "r1", "tray-status-menu");
        delta(&store, "p2", "desktop-app");
        delta(&store, "n5", "archive-skill");
        delta(&store, "n3", "desktop-app");
        delta(&store, "n4", "discuss-skill");
        let plan = compute(&store).unwrap();
        assert_eq!(names(&plan), ["r1", "p2", "n5", "n3", "n4"], "基底序");
        assert_eq!(
            waves(&plan),
            [vec!["r1", "p2", "n5"], vec!["n3"], vec!["n4"]]
        );
        assert_eq!(plan.next.as_deref(), Some("n5"));
        let rows: Vec<(&str, usize, Vec<&str>)> = plan
            .changes
            .iter()
            .map(|c| {
                (
                    c.name.as_str(),
                    c.wave,
                    c.blocked_by.iter().map(String::as_str).collect(),
                )
            })
            .collect();
        assert_eq!(
            rows,
            [
                ("r1", 1, vec![]),
                ("p2", 1, vec![]),
                ("n5", 1, vec![]),
                ("n3", 2, vec!["p2"]),
                ("n4", 3, vec!["n3"]),
            ]
        );
        assert_eq!(entry(&plan, "r1").stage, Stage::Ready);
        assert_eq!(entry(&plan, "p2").stage, Stage::InProgress);
        assert_eq!(entry(&plan, "n5").stage, Stage::Proposed);
    }

    #[test]
    fn next_is_none_when_no_proposed_change_is_ready() {
        // 已就緒與進行中不算 next；唯一的提案中 change 被擋住 → None。
        let store = store_of(&[
            ("a", "schema: spec-driven\nstarted_at: 2026-09-06\n"),
            ("c", "schema: spec-driven\ndepends_on: a\n"),
        ]);
        let plan = compute(&store).unwrap();
        assert_eq!(plan.next, None);
    }

    #[test]
    fn an_empty_store_plans_to_nothing() {
        let plan = compute(&TestStore::default()).unwrap();
        assert!(plan.waves.is_empty());
        assert!(plan.changes.is_empty());
        assert_eq!(plan.next, None);
        assert!(plan.skipped.is_empty());
    }

    #[test]
    fn a_dependency_cycle_is_an_error_naming_the_cycle() {
        // Scenario 依賴成環：a→b→a。
        let store = store_of(&[
            ("a", "schema: spec-driven\ndepends_on: b\n"),
            ("b", "schema: spec-driven\ndepends_on: a\n"),
        ]);
        let err = compute(&store).unwrap_err();
        assert_eq!(
            err,
            PlanError::Cycle(vec!["a".into(), "b".into(), "a".into()])
        );
        assert_eq!(err.to_string(), "dependency cycle: a -> b -> a");
    }

    #[test]
    fn a_cycle_behind_a_tail_is_reported_from_the_cycle_itself() {
        // x 依賴 a、a↔b 成環：報的是環（a -> b -> a），不是走進環的尾巴。
        let store = store_of(&[
            (
                "a",
                "schema: spec-driven\ncreated: 2026-09-02\ndepends_on: b\n",
            ),
            (
                "b",
                "schema: spec-driven\ncreated: 2026-09-03\ndepends_on: a\n",
            ),
            (
                "x",
                "schema: spec-driven\ncreated: 2026-09-01\ndepends_on: a\n",
            ),
        ]);
        let err = compute(&store).unwrap_err();
        assert_eq!(err.to_string(), "dependency cycle: a -> b -> a");
    }

    #[test]
    fn corrupt_meta_is_skipped_and_excluded_from_waves_and_overlaps() {
        // Scenario 壞 metadata 列入 skipped：不參與配置，也不擋重疊的鄰居。
        let store = store_of(&[
            ("bad", ": : :\n\t bad yaml [unclosed\n"),
            ("good", PROPOSED),
        ]);
        delta(&store, "bad", "desktop-app");
        delta(&store, "good", "desktop-app");
        let plan = compute(&store).unwrap();
        assert_eq!(names(&plan), ["good"]);
        assert_eq!(waves(&plan), [vec!["good"]]);
        let good = entry(&plan, "good");
        assert!(good.blocked_by.is_empty() && good.overlaps.is_empty() && good.ready);
        assert_eq!(plan.skipped.len(), 1);
        assert_eq!(plan.skipped[0].change, "bad");
        assert!(
            !plan.skipped[0].reason.is_empty(),
            "the parse reason travels"
        );
        assert_eq!(plan.next.as_deref(), Some("good"));
    }

    #[test]
    fn compute_with_ranks_takes_the_given_ranks_over_the_meta_ones() {
        // compute 以 meta 的 board_rank 當 ranks（b 的 f 排在 a 的 n 前）；外部
        // ranks 整份取代——b 不在表內即缺 rank、排到 a 之後。
        let store = store_of(&[
            ("a", "schema: spec-driven\nboard_rank: n\n"),
            ("b", "schema: spec-driven\nboard_rank: f\n"),
        ]);
        assert_eq!(names(&compute(&store).unwrap()), ["b", "a"]);
        let ranks = BTreeMap::from([("a".to_string(), "a".to_string())]);
        assert_eq!(
            names(&compute_with_ranks(&store, &ranks).unwrap()),
            ["a", "b"]
        );
    }

    #[test]
    fn plan_serializes_camel_case_with_empty_skipped_as_an_array() {
        // design D3 的 payload 形狀：四鍵、七鍵皆 camelCase，skipped 空時為 []。
        let store = store_of(&[
            ("a", "schema: spec-driven\ncreated: 2026-09-01\n"),
            (
                "b",
                "schema: spec-driven\ncreated: 2026-09-02\ndepends_on: a\n",
            ),
        ]);
        delta(&store, "a", "zeta");
        delta(&store, "a", "alpha");
        delta(&store, "b", "zeta");
        delta(&store, "b", "alpha");
        let json = serde_json::to_value(compute(&store).unwrap()).unwrap();
        let keys =
            |v: &serde_json::Value| v.as_object().unwrap().keys().cloned().collect::<Vec<_>>();
        assert_eq!(keys(&json), ["changes", "next", "skipped", "waves"]);
        assert_eq!(json["skipped"], serde_json::json!([]));
        assert_eq!(json["next"], "a");
        assert_eq!(
            json["waves"],
            serde_json::json!([
                { "index": 1, "changes": ["a"] },
                { "index": 2, "changes": ["b"] },
            ])
        );
        let b = &json["changes"][1];
        assert_eq!(
            keys(b),
            [
                "blockedBy",
                "dependsOn",
                "name",
                "overlaps",
                "ready",
                "stage",
                "wave"
            ]
        );
        assert_eq!(b["stage"], "proposed");
        assert_eq!(b["wave"], 2);
        assert_eq!(b["ready"], false);
        assert_eq!(b["dependsOn"], serde_json::json!(["a"]));
        assert_eq!(b["blockedBy"], serde_json::json!(["a"]));
        assert_eq!(
            b["overlaps"],
            serde_json::json!([{ "change": "a", "capabilities": ["alpha", "zeta"] }]),
            "capabilities ascending"
        );
    }

    // --- rank 移動的依賴檢查（design D5；spec「rank 移動的依賴檢查」）---

    fn deps_map(pairs: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
            .collect()
    }

    #[test]
    fn violations_lists_dependents_placed_before_their_prerequisites() {
        // 純序列：c 依賴 a 與 b 卻排最前 → 兩對違規；b 依賴 a 且在 a 之後 → 合法。
        let deps = deps_map(&[("c", &["a", "b"]), ("b", &["a"])]);
        assert_eq!(
            violations(&["c", "a", "b"], &deps),
            [
                ("c".to_string(), "a".to_string()),
                ("c".to_string(), "b".to_string())
            ]
        );
        assert!(violations(&["a", "b", "c"], &deps).is_empty());
        // 前置不在序列內（別的階段、已封存）不構成違規。
        let outside = deps_map(&[("a", &["zzz"])]);
        assert!(violations(&["a", "b"], &outside).is_empty());
    }

    #[test]
    fn check_rank_move_refuses_dragging_a_change_before_its_prerequisite() {
        // Scenario 拖到宣告前置之前被拒：c 依賴 a，擬把 c 的 rank 排到 a 之前。
        let store = store_of(&[
            ("a", "schema: spec-driven\nboard_rank: n\n"),
            ("c", "schema: spec-driven\nboard_rank: p\ndepends_on: a\n"),
        ]);
        let err = check_rank_move(&store, "c", "a").unwrap_err();
        assert_eq!(err.change, "c");
        assert_eq!(err.must_follow, ["a"]);
        assert!(err.must_precede.is_empty());
        assert!(
            err.to_string().contains("'c'") && err.to_string().contains("a"),
            "{err}"
        );
        // 排到 a 之後則允許。
        assert_eq!(check_rank_move(&store, "c", "z"), Ok(()));
    }

    #[test]
    fn check_rank_move_refuses_dragging_a_change_behind_its_dependent() {
        // 宣告依賴它的 change 會排到它前面 → must_precede。
        let store = store_of(&[
            ("a", "schema: spec-driven\nboard_rank: a\n"),
            ("c", "schema: spec-driven\nboard_rank: n\ndepends_on: a\n"),
        ]);
        let err = check_rank_move(&store, "a", "z").unwrap_err();
        assert_eq!(err.change, "a");
        assert!(err.must_follow.is_empty());
        assert_eq!(err.must_precede, ["c"]);
    }

    #[test]
    fn check_rank_move_allows_flipping_overlap_partners() {
        // Scenario 重疊夥伴翻轉允許：只因 delta 重疊而依序，翻轉是合法選擇。
        let store = store_of(&[
            ("a", "schema: spec-driven\nboard_rank: n\n"),
            ("b", "schema: spec-driven\nboard_rank: z\n"),
        ]);
        delta(&store, "a", "desktop-app");
        delta(&store, "b", "desktop-app");
        assert_eq!(check_rank_move(&store, "b", "f"), Ok(()));
    }

    #[test]
    fn check_rank_move_only_looks_at_the_same_stage() {
        // 前置在另一欄（進行中）時，提案中的 c 在自己欄內怎麼排都不違規。
        let store = store_of(&[
            (
                "a",
                "schema: spec-driven\nboard_rank: n\nstarted_at: 2026-09-06\n",
            ),
            ("c", "schema: spec-driven\nboard_rank: p\ndepends_on: a\n"),
        ]);
        assert_eq!(check_rank_move(&store, "c", "a"), Ok(()));
    }

    #[test]
    fn check_rank_move_never_writes() {
        let store = store_of(&[
            ("a", "schema: spec-driven\nboard_rank: n\n"),
            ("c", "schema: spec-driven\nboard_rank: p\ndepends_on: a\n"),
        ]);
        assert!(check_rank_move(&store, "c", "a").is_err());
        assert_eq!(check_rank_move(&store, "c", "z"), Ok(()));
        assert_eq!(*store.meta_writes.borrow(), 0, "the check writes nothing");
        assert_eq!(
            store.meta("c"),
            "schema: spec-driven\nboard_rank: p\ndepends_on: a\n"
        );
    }

    // --- sharp edges（tasks 2.4：殘留值不得讓 compute 崩潰或成環誤報）---

    #[test]
    fn residue_blank_entries_are_ignored() {
        // 中間與尾端的空項（手改留下的 `a, , b,`）由讀取器丟掉。
        let store = store_of(&[
            ("a", PROPOSED),
            ("b", PROPOSED),
            ("c", "schema: spec-driven\ndepends_on: a, , b,\n"),
        ]);
        let plan = compute(&store).unwrap();
        let c = entry(&plan, "c");
        assert_eq!(c.depends_on, ["a", "b"]);
        assert_eq!(c.blocked_by, ["a", "b"]);
    }

    #[test]
    fn residue_duplicate_entries_block_once() {
        // dependsOn 原文照列（含重複），blockedBy 去重、波次不重複累加。
        let store = store_of(&[
            ("a", PROPOSED),
            ("c", "schema: spec-driven\ndepends_on: a, a\n"),
        ]);
        let plan = compute(&store).unwrap();
        let c = entry(&plan, "c");
        assert_eq!(c.depends_on, ["a", "a"]);
        assert_eq!(c.blocked_by, ["a"]);
        assert_eq!(c.wave, 2);
    }

    #[test]
    fn residue_self_reference_is_not_a_cycle() {
        let store = store_of(&[("a", "schema: spec-driven\ndepends_on: a\n")]);
        let plan = compute(&store).unwrap();
        let a = entry(&plan, "a");
        assert_eq!(a.depends_on, ["a"], "verbatim");
        assert!(a.blocked_by.is_empty() && a.ready && a.wave == 1);
        assert_eq!(plan.next.as_deref(), Some("a"));
        assert_eq!(check_rank_move(&store, "a", "n"), Ok(()));
    }

    #[test]
    fn residue_path_like_names_are_satisfied_and_listed_verbatim() {
        // 含路徑分隔符的殘留值：不匹配任何作用中 change → 視為滿足；compute 只
        // 以 change 名比對，從不拿這些值去讀 store。
        let store = store_of(&[(
            "a",
            "schema: spec-driven\ndepends_on: ../evil, x/y, c:d, ..\n",
        )]);
        let plan = compute(&store).unwrap();
        let a = entry(&plan, "a");
        assert_eq!(a.depends_on, ["../evil", "x/y", "c:d", ".."]);
        assert!(a.blocked_by.is_empty() && a.ready);
    }

    #[test]
    fn dependency_on_a_skipped_change_is_not_a_blocker() {
        // 壞 meta 的 change 排除於配置之外：對它的依賴若算數會永遠配置不了，
        // 所以視為已滿足；skipped 仍把它列出來給人看。
        let store = store_of(&[
            ("bad", ": : :\n\t bad yaml [unclosed\n"),
            ("c", "schema: spec-driven\ndepends_on: bad\n"),
        ]);
        let plan = compute(&store).unwrap();
        let c = entry(&plan, "c");
        assert_eq!(c.depends_on, ["bad"]);
        assert!(c.blocked_by.is_empty() && c.ready && c.wave == 1);
        assert_eq!(plan.skipped[0].change, "bad");
    }

    #[test]
    fn check_rank_move_tolerates_unknown_names_and_odd_ranks() {
        // 未知名稱無可交叉 → Ok（寫入端才報 not found）；非法 rank 只當位元組
        // 排序、不 panic（合法性由 set_board_rank 守）。
        let store = store_of(&[
            ("a", "schema: spec-driven\nboard_rank: n\n"),
            ("c", "schema: spec-driven\nboard_rank: p\ndepends_on: a\n"),
        ]);
        assert_eq!(check_rank_move(&store, "ghost", "n"), Ok(()));
        // 'Z' (0x5a) 排在 'n' (0x6e) 前：答案確定為「會跨過前置 a」，而非 panic。
        let err = check_rank_move(&store, "c", "ZZ9").unwrap_err();
        assert_eq!(err.must_follow, ["a"]);
    }

    #[test]
    fn stage_order_is_ready_then_in_progress_then_proposed() {
        // 基底排序的第一鍵倚賴 Stage 的宣告順序；重排 enum 會靜默翻轉整個 plan。
        assert!(Stage::Ready < Stage::InProgress);
        assert!(Stage::InProgress < Stage::Proposed);
    }

    // --- set_depends（design D4；spec「change depends 動詞寫入宣告依賴」）---

    #[test]
    fn set_depends_writes_once_and_reports_the_list_after_the_write() {
        let store = store_of(&[("a", PROPOSED), ("b", PROPOSED), ("c", PROPOSED)]);
        let write = set_depends(&store, "c", &["a".into(), "b".into()], false).unwrap();
        assert_eq!(
            write,
            DependsWrite {
                depends_on: vec!["a".into(), "b".into()],
                changed: true
            }
        );
        assert_eq!(store.meta("c"), format!("{PROPOSED}depends_on: a, b\n"));
        assert_eq!(*store.meta_writes.borrow(), 1, "several --on, one write");
        // 冪等：已存在的邊不改檔、changed 為 false。
        let again = set_depends(&store, "c", &["a".into()], false).unwrap();
        assert_eq!(
            again,
            DependsWrite {
                depends_on: vec!["a".into(), "b".into()],
                changed: false
            }
        );
        assert_eq!(*store.meta_writes.borrow(), 1);
        // 移除到空：整行移除。
        let gone = set_depends(&store, "c", &["a".into(), "b".into()], true).unwrap();
        assert_eq!(
            gone,
            DependsWrite {
                depends_on: vec![],
                changed: true
            }
        );
        assert_eq!(store.meta("c"), PROPOSED);
    }

    #[test]
    fn set_depends_guards_refuse_in_order_with_zero_writes() {
        let store = store_of(&[
            ("a", "schema: spec-driven\ndepends_on: c\n"),
            ("c", PROPOSED),
        ]);
        store
            .archived_metas
            .borrow_mut()
            .insert("2026-08-01-old".to_string(), PROPOSED.to_string());
        let refuse = |name: &str, on: &[&str], needle: &str| {
            let on: Vec<String> = on.iter().map(|s| s.to_string()).collect();
            let err = set_depends(&store, name, &on, false)
                .unwrap_err()
                .to_string();
            assert!(err.contains(needle), "{name} --on {on:?}: {err}");
        };
        refuse("ghost", &["a"], "Change 'ghost' not found.");
        refuse("../evil", &["a"], "not found");
        refuse("c", &["ghost"], "no active change with that name");
        refuse("c", &["old"], "already archived");
        refuse("c", &["../evil"], "no active change");
        refuse("c", &["c"], "cannot depend on itself");
        refuse("c", &["a"], "dependency cycle: c -> a -> c");
        // 環從被編輯的 change 起讀：即使基底順序 a 在前（a 有 created、c 沒有）。
        store.metas.borrow_mut().insert(
            "a".to_string(),
            "schema: spec-driven\ncreated: 2026-01-01\ndepends_on: c\n".to_string(),
        );
        store
            .metas
            .borrow_mut()
            .insert("c".to_string(), "schema: spec-driven\n".to_string());
        refuse("c", &["a"], "dependency cycle: c -> a -> c");
        store
            .metas
            .borrow_mut()
            .insert("c".to_string(), PROPOSED.to_string());
        // 守門順序：不存在的前置先於「自己」報錯（第一個 --on 就擋）。
        refuse("c", &["ghost", "c"], "no active change");
        assert_eq!(
            *store.meta_writes.borrow(),
            0,
            "every refusal is a zero-write"
        );
        assert_eq!(store.meta("c"), PROPOSED);
    }

    #[test]
    fn set_depends_remove_skips_the_cycle_check_and_tolerates_absent_edges() {
        // --remove 不可能造環，也不需要邊存在：移除不存在的邊是冪等成功。
        let store = store_of(&[
            ("a", "schema: spec-driven\ndepends_on: c\n"),
            ("c", PROPOSED),
        ]);
        let write = set_depends(&store, "c", &["a".into()], true).unwrap();
        assert_eq!(
            write,
            DependsWrite {
                depends_on: vec![],
                changed: false
            }
        );
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn set_depends_remove_clears_entries_that_point_nowhere() {
        // --remove 清殘留：前置已封存或已不存在的名稱照樣移除（加邊時才要求作用中）。
        let store = store_of(&[("c", &format!("{PROPOSED}depends_on: old, ghost, a\n"))]);
        store
            .archived_metas
            .borrow_mut()
            .insert("2026-08-01-old".to_string(), PROPOSED.to_string());
        let write = set_depends(&store, "c", &["old".into(), "ghost".into()], true).unwrap();
        assert_eq!(
            write,
            DependsWrite {
                depends_on: vec!["a".into()],
                changed: true
            }
        );
        assert_eq!(store.meta("c"), format!("{PROPOSED}depends_on: a\n"));
        assert_eq!(*store.meta_writes.borrow(), 1);
        // 加邊仍照守門：同一批名稱加回去被拒、零寫入。
        let err = set_depends(&store, "c", &["old".into()], false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("already archived"), "{err}");
        assert_eq!(*store.meta_writes.borrow(), 1);
        // --remove 只擋不合法名稱與自己：空白名稱與自我引用不碰檔。
        for bad in ["", "  ", "../evil", "c"] {
            let err = set_depends(&store, "c", &[bad.into()], true).unwrap_err();
            assert!(
                err.downcast_ref::<crate::command::Refusal>().is_some(),
                "{bad:?}: {err}"
            );
        }
        assert_eq!(*store.meta_writes.borrow(), 1);
    }

    #[test]
    fn set_depends_only_refuses_a_cycle_that_runs_through_the_edited_change() {
        // 別處既有的環（a ↔ b）不擋 c 的加邊；plan 自己會報那個環。
        let store = store_of(&[
            ("a", "schema: spec-driven\ndepends_on: b\n"),
            ("b", "schema: spec-driven\ndepends_on: a\n"),
            ("c", PROPOSED),
            ("d", PROPOSED),
        ]);
        let write = set_depends(&store, "c", &["d".into()], false).unwrap();
        assert_eq!(write.depends_on, ["d"]);
        assert_eq!(*store.meta_writes.borrow(), 1);
        // 經過被編輯 change 的環才拒絕，路徑從它起讀、沿宣告順序走：c -> d -> e -> c。
        store.metas.borrow_mut().insert(
            "d".to_string(),
            "schema: spec-driven\ndepends_on: e\n".to_string(),
        );
        store.metas.borrow_mut().insert(
            "e".to_string(),
            "schema: spec-driven\ndepends_on: c\n".to_string(),
        );
        let err = set_depends(&store, "c", &["d".into()], false)
            .unwrap_err()
            .to_string();
        assert_eq!(err, "dependency cycle: c -> d -> e -> c");
        assert_eq!(*store.meta_writes.borrow(), 1);
    }

    #[test]
    fn set_depends_refuses_corrupt_target_meta_without_writing() {
        // 壞 meta 的 `<name>` 由 edit_meta fail-closed 拒絕（MetaError 可 downcast）。
        const BAD: &str = ": : :\n\t bad yaml [unclosed\n";
        let store = store_of(&[("a", PROPOSED), ("bad", BAD)]);
        let err = set_depends(&store, "bad", &["a".into()], false).unwrap_err();
        assert!(
            err.downcast_ref::<crate::model::MetaError>().is_some(),
            "{err}"
        );
        assert_eq!(store.meta("bad"), BAD);
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn compute_never_writes() {
        let store = store_of(&[
            ("a", PROPOSED),
            ("b", "schema: spec-driven\ndepends_on: a\n"),
        ]);
        compute(&store).unwrap();
        assert_eq!(*store.meta_writes.borrow(), 0);
        assert_eq!(*store.artifact_writes.borrow(), 0);
    }
}
