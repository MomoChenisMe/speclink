//! Change-level execution order (change-plan design D1): a total basis order
//! (stage → board rank → for the unranked: dependents, task total, created →
//! name), then a greedy topological pass that places every active change into
//! a wave. Declared prerequisites (`depends_on`) are the only edges: they alone
//! push a wave, so a cycle can only come from them. Delta overlap never delays
//! a start — a requirement two changes both touch only orders their archiving
//! (`archive_after`). Corrupt metadata is reported in `skipped` and takes no
//! part in placement or overlap.

use crate::model::{self, Change, Stage};
use crate::store::Store;
use serde::Serialize;
use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeMap, BTreeSet, HashMap};

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
    /// Every other placed change sharing a delta capability, in placement
    /// order — the directory-level view, kept as it was for its readers.
    pub overlaps: Vec<Overlap>,
    /// Active declared prerequisites, deduplicated, in placement order.
    pub blocked_by: Vec<String>,
    /// `blocked_by` is empty.
    pub ready: bool,
    /// Every requirement shared with another placed change, by that change's
    /// placement order, then capability, then requirement.
    pub requirement_overlap: Vec<RequirementOverlap>,
    /// The changes this one should archive after, in placement order.
    pub archive_after: Vec<String>,
}

/// A delta-capability overlap with one other change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Overlap {
    pub change: String,
    /// Shared capability names, ascending.
    pub capabilities: Vec<String>,
}

/// A requirement this change and `change` both touch in the same capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementOverlap {
    pub change: String,
    pub capability: String,
    pub requirement: String,
    pub own_operation: DeltaOp,
    pub other_operation: DeltaOp,
    /// Both sides bring the name in (ADDED, or the new name of a rename), or
    /// both take it away (REMOVED, or the old name of a rename): the archive
    /// refuses whichever lands second — the name is already there, or already
    /// gone — so no order helps; one side has to change.
    pub conflict: bool,
}

/// The delta section a requirement is touched by. RENAMED covers both ends of
/// a rename pair — the old name and the new one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DeltaOp {
    Added,
    Modified,
    Removed,
    Renamed,
}

impl DeltaOp {
    /// The JSON value (`ADDED` …) — what the wire carries as a string.
    pub fn as_str(self) -> &'static str {
        match self {
            DeltaOp::Added => "ADDED",
            DeltaOp::Modified => "MODIFIED",
            DeltaOp::Removed => "REMOVED",
            DeltaOp::Renamed => "RENAMED",
        }
    }

    /// The inverse of [`DeltaOp::as_str`]; anything else is no operation.
    pub fn parse(s: &str) -> Option<DeltaOp> {
        [DeltaOp::Added, DeltaOp::Modified, DeltaOp::Removed, DeltaOp::Renamed]
            .into_iter()
            .find(|op| op.as_str() == s)
    }
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
    compute_from(store, changes, &ranks, false)
}

/// `plan --strict-overlap` — the plan as it was before requirement-level
/// overlap: a delta capability shared with a change placed earlier pushes the
/// wave and joins `blocked_by`, and the unranked sort by created alone. The
/// CLI's local plan is its one caller.
pub(crate) fn compute_strict(store: &dyn Store) -> Result<Plan, PlanError> {
    let changes = model::list_changes(store);
    let ranks = meta_ranks(&changes);
    compute_from(store, changes, &ranks, true)
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
    compute_from(store, model::list_changes(store), ranks, false)
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

/// Why `name` is no active change: archived, or no change by that name.
fn inactive_reason(store: &dyn Store, name: &str) -> &'static str {
    let archived = store
        .list_archived_changes()
        .iter()
        .any(|dated| crate::util::strip_date_prefix(dated) == name);
    if archived {
        "it is already archived"
    } else {
        "no active change with that name"
    }
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
            return Err(refuse(format!(
                "cannot depend on '{other}': {}",
                inactive_reason(store, other)
            )));
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
    /// How many other placeable changes declare this one as a prerequisite.
    dependents: usize,
    /// Tasks in tasks.md; `None` when there is no task to apply yet — no
    /// tasks.md, or one that lists no task.
    task_total: Option<usize>,
}

impl Entry {
    /// The basis order (design D1): stage; ranked before unranked, ranks by
    /// bytes; unranked by most dependents, then fewest tasks (no task to
    /// apply last) — `strict` (`plan --strict-overlap`) skips these two, as the order
    /// before them did — then created, missing created last; name decides
    /// ties. A rank alone orders the ranked.
    fn basis_cmp(&self, other: &Entry, strict: bool) -> Ordering {
        let size = |e: &Entry| (Reverse(e.dependents), e.task_total.unwrap_or(usize::MAX));
        self.stage
            .cmp(&other.stage)
            .then_with(|| match (&self.rank, &other.rank) {
                (Some(a), Some(b)) => a.cmp(b),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => (if strict {
                    Ordering::Equal
                } else {
                    size(self).cmp(&size(other))
                })
                .then_with(|| self.created.is_none().cmp(&other.created.is_none()))
                .then_with(|| self.created.cmp(&other.created)),
            })
            .then_with(|| self.name.cmp(&other.name))
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
            dependents: 0,
            task_total: store
                .read_artifact(&c.name, "tasks.md")
                .map(|md| crate::tasks::parse(&md).len())
                .filter(|&total| total > 0),
            name: c.name,
        });
    }
    // The same edges the placement follows: a dependent counts once however
    // often it names the prerequisite, and only a placeable one counts.
    for prereqs in deps_of(&entries) {
        for j in prereqs {
            entries[j].dependents += 1;
        }
    }
    entries.sort_by(|a, b| a.basis_cmp(b, false));
    (entries, skipped)
}

/// The placeable changes' names in basis order, before declared dependencies
/// correct it — the order a caller falls back to when the plan cannot be
/// computed (a dependency cycle), so both come from one key.
pub fn basis_order(store: &dyn Store) -> Vec<String> {
    let changes = model::list_changes(store);
    let ranks = meta_ranks(&changes);
    entries_of(store, changes, &ranks)
        .0
        .into_iter()
        .map(|e| e.name)
        .collect()
}

/// The placeable changes' names in the order the board shows them (design
/// D1): the plan's placement, or the basis order when a dependency cycle
/// stalls it — the one order the board lists and a board move is placed in.
pub fn board_order(store: &dyn Store) -> Vec<String> {
    let changes = model::list_changes(store);
    let ranks = meta_ranks(&changes);
    let (entries, _) = entries_of(store, changes, &ranks);
    shown_order(&entries)
        .into_iter()
        .map(|i| entries[i].name.clone())
        .collect()
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

/// How a delta touches a requirement name. A rename touches two names — the
/// one it drops and the one it brings in — and the output tells neither end
/// apart ([`DeltaOp::Renamed`]), but the pairing does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Touch {
    Added,
    Modified,
    Removed,
    RenamedFrom,
    RenamedTo,
}

/// What a touch needs from the name and leaves behind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    /// MODIFIED: needs the name there and leaves it there.
    Uses,
    /// REMOVED, or the old name of a rename: needs the name there and leaves
    /// it gone.
    TakesAway,
    /// ADDED, or the new name of a rename: needs the name gone and leaves it
    /// there.
    BringsIn,
}

impl Touch {
    fn op(self) -> DeltaOp {
        match self {
            Touch::Added => DeltaOp::Added,
            Touch::Modified => DeltaOp::Modified,
            Touch::Removed => DeltaOp::Removed,
            Touch::RenamedFrom | Touch::RenamedTo => DeltaOp::Renamed,
        }
    }

    fn role(self) -> Role {
        match self {
            Touch::Modified => Role::Uses,
            Touch::Removed | Touch::RenamedFrom => Role::TakesAway,
            Touch::Added | Touch::RenamedTo => Role::BringsIn,
        }
    }
}

/// Where `role` falls in the order the archives on one name must run, given
/// whether the canonical spec carries the name now: each archive needs the
/// name the way the one before it leaves it. A name the canon carries is
/// used, then taken away, then brought back in; a name it lacks is brought
/// in, then used, then taken away.
fn step(role: Role, in_canon: bool) -> u8 {
    match (role, in_canon) {
        (Role::Uses, true) | (Role::BringsIn, false) => 0,
        (Role::TakesAway, true) | (Role::Uses, false) => 1,
        (Role::BringsIn, true) | (Role::TakesAway, false) => 2,
    }
}

/// Every (capability, touch, requirement) a change's deltas touch, read with
/// the archive merge gate's own reading ([`crate::archive::delta_operations`])
/// so the plan and the gate agree on what a delta names: ADDED, MODIFIED and
/// REMOVED from the operation sections, both ends of each rename pair. An
/// unreadable or blank delta touches nothing.
fn delta_touches(store: &dyn Store, change: &str) -> Vec<(String, Touch, String)> {
    let mut out = Vec::new();
    for cap in store.delta_capabilities(change) {
        let text = store
            .read_artifact(change, &model::delta_spec_artifact(&cap))
            .unwrap_or_default();
        let (reqs, renames) = crate::archive::delta_operations(&text);
        for r in reqs {
            let touch = match DeltaOp::parse(&r.operation) {
                Some(DeltaOp::Added) => Touch::Added,
                Some(DeltaOp::Modified) => Touch::Modified,
                Some(DeltaOp::Removed) => Touch::Removed,
                // Rename ends come from the pairs below; other sections touch
                // nothing.
                Some(DeltaOp::Renamed) | None => continue,
            };
            out.push((cap.clone(), touch, r.name));
        }
        for (from, to) in renames {
            out.push((cap.clone(), Touch::RenamedFrom, from));
            out.push((cap.clone(), Touch::RenamedTo, to));
        }
    }
    out
}

/// One requirement a change touches: how, and at which [`step`] on the name.
#[derive(Debug, Clone, Copy)]
struct Touched {
    touch: Touch,
    step: u8,
}

/// A change's touches keyed by (capability, requirement), so iteration runs
/// in the order the plan lists overlaps.
type Touches = BTreeMap<(String, String), Touched>;

/// A name touched twice keeps its first touch — only a delta the merge gate
/// refuses anyway can do that. `in_canon` tells whether a capability's
/// canonical spec carries a requirement now.
fn touch_map(
    touches: Vec<(String, Touch, String)>,
    in_canon: impl Fn(&str, &str) -> bool,
) -> Touches {
    let mut map = Touches::new();
    for (cap, touch, req) in touches {
        let step = step(touch.role(), in_canon(&cap, &req));
        map.entry((cap, req)).or_insert(Touched { touch, step });
    }
    map
}

/// Every requirement `own` and `theirs` both touch, capability then
/// requirement ascending, as seen from the owner of `own`.
fn overlaps_with(own: &Touches, other: &str, theirs: &Touches) -> Vec<RequirementOverlap> {
    own.iter()
        .filter_map(|(key, mine)| {
            let their = theirs.get(key)?;
            let role = mine.touch.role();
            Some(RequirementOverlap {
                change: other.to_string(),
                capability: key.0.clone(),
                requirement: key.1.clone(),
                own_operation: mine.touch.op(),
                other_operation: their.touch.op(),
                conflict: role != Role::Uses && role == their.touch.role(),
            })
        })
        .collect()
}

/// Does `own` touch a name at a later [`step`] than `theirs` does? Then the
/// owner of `own` waits for the other: its archive needs the name the way the
/// other's archive leaves it, whatever the placement.
fn waits_for(own: &Touches, theirs: &Touches) -> bool {
    own.iter()
        .any(|(key, mine)| theirs.get(key).is_some_and(|their| mine.step > their.step))
}

/// Does the owner of `own` archive after the owner of `theirs`? Waiting on a
/// shared name decides it for the whole pair; so does the other waiting on
/// one. Only when neither waits does a name both sides use follow
/// `partner_first` — the other comes earlier in the archive order. Both sides
/// bringing a name in, or both taking it away, is a conflict, not an order;
/// waiting both ways leaves each side on the other's list — a real deadlock.
fn archives_after(own: &Touches, theirs: &Touches, partner_first: bool) -> bool {
    if waits_for(own, theirs) {
        return true;
    }
    if waits_for(theirs, own) {
        return false;
    }
    partner_first
        && own.iter().any(|(key, mine)| {
            mine.touch.role() == Role::Uses
                && theirs
                    .get(key)
                    .is_some_and(|their| their.touch.role() == Role::Uses)
        })
}

/// Each entry's position in the order the plan advises archiving in: the
/// placement order, except that a change comes after every change it waits
/// for ([`waits_for`]) — a stable topological pass that takes the
/// earliest-placed change whose waits are all behind it. Overlaps no name
/// orders then follow this one order, so the advice never runs in a circle
/// through them. Waiting both ways stalls the pass; it breaks the stall by
/// placement.
fn archive_positions(order: &[usize], touches: &[Touches]) -> Vec<usize> {
    let waits: Vec<Vec<bool>> = (0..touches.len())
        .map(|i| {
            (0..touches.len())
                .map(|j| i != j && waits_for(&touches[i], &touches[j]))
                .collect()
        })
        .collect();
    let mut position = vec![usize::MAX; touches.len()];
    for at in 0..order.len() {
        let open = |i: usize| position[i] == usize::MAX;
        let pick = order
            .iter()
            .copied()
            .find(|&i| open(i) && order.iter().all(|&j| !open(j) || !waits[i][j]))
            .or_else(|| order.iter().copied().find(|&i| open(i)))
            .expect("an entry is left to take");
        position[pick] = at;
    }
    position
}

/// Greedy placement along the basis order (design D1): repeatedly take the
/// first entry whose prerequisites are all placed; its wave is one past its
/// latest prerequisite's — and, under `strict`, past every capability-sharing
/// change already placed. Returns the placement order and each entry's wave,
/// or the cycle that stalled it.
fn place(
    entries: &[Entry],
    deps: &[Vec<usize>],
    strict: bool,
) -> Result<(Vec<usize>, Vec<usize>), PlanError> {
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
        if strict {
            for &j in &order {
                if !shared(&entries[i], &entries[j]).is_empty() {
                    wave = wave.max(wave_of[j].expect("placed") + 1);
                }
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
    strict: bool,
) -> Result<Plan, PlanError> {
    let (mut entries, skipped) = entries_of(store, changes, ranks);
    if strict {
        entries.sort_by(|a, b| a.basis_cmp(b, true));
    }
    let deps = deps_of(&entries);
    let (order, wave_of) = place(&entries, &deps, strict)?;
    let canon: BTreeMap<&str, BTreeSet<String>> = entries
        .iter()
        .flat_map(|e| e.capabilities.iter().map(String::as_str))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|cap| (cap, crate::archive::canonical_names(store, cap)))
        .collect();
    let touches: Vec<Touches> = entries
        .iter()
        .map(|e| {
            touch_map(delta_touches(store, &e.name), |cap, req| {
                canon.get(cap).is_some_and(|names| names.contains(req))
            })
        })
        .collect();
    let archived_at = archive_positions(&order, &touches);

    let placed_at = |i: usize| order.iter().position(|&o| o == i).expect("placed");
    let mut plan_changes = Vec::new();
    for (at, &i) in order.iter().enumerate() {
        let mut overlaps = Vec::new();
        let mut requirement_overlap = Vec::new();
        let mut archive_after = Vec::new();
        for &j in &order {
            if j == i {
                continue;
            }
            let capabilities = shared(&entries[i], &entries[j]);
            if !capabilities.is_empty() {
                overlaps.push(Overlap {
                    change: entries[j].name.clone(),
                    capabilities,
                });
            }
            if archives_after(&touches[i], &touches[j], archived_at[j] < archived_at[i]) {
                archive_after.push(entries[j].name.clone());
            }
            requirement_overlap.extend(overlaps_with(&touches[i], &entries[j].name, &touches[j]));
        }
        let mut blockers: Vec<usize> = deps[i].clone();
        if strict {
            for &j in order.iter().take(at) {
                if !blockers.contains(&j) && !shared(&entries[i], &entries[j]).is_empty() {
                    blockers.push(j);
                }
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
            requirement_overlap,
            archive_after,
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

/// Would the rank table `ranks` put `name` across a declared dependency
/// (design D5)? Re-sorts the change's own stage by the basis order (the same
/// [`entries_of`] the plan uses) under those ranks and asks [`blocked_move`].
/// Reads only; [`move_rank`] checks the stamps and the moved change's new rank
/// together this way before any write. An unknown or corrupt `name` has
/// nothing to cross and passes.
fn check_ranks(
    store: &dyn Store,
    changes: Vec<Change>,
    ranks: &BTreeMap<String, String>,
    name: &str,
) -> Result<(), RankMoveBlocked> {
    let (entries, _) = entries_of(store, changes, ranks);
    let Some(moved) = entries.iter().find(|e| e.name == name) else {
        return Ok(());
    };
    let column: Vec<&Entry> = entries.iter().filter(|e| e.stage == moved.stage).collect();
    let sequence: Vec<&str> = column.iter().map(|e| e.name.as_str()).collect();
    let depends_on: BTreeMap<String, Vec<String>> = column
        .iter()
        .map(|e| (e.name.clone(), e.depends_on.clone()))
        .collect();
    blocked_move(&sequence, &depends_on, name)
}

/// Which side of its anchor `change rank` puts a change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Before,
    After,
}

impl Side {
    pub fn as_str(self) -> &'static str {
        match self {
            Side::Before => "before",
            Side::After => "after",
        }
    }
}

/// `change rank` (spec「change rank 動詞寫入看板順序鍵」): rewrite `name`'s
/// board rank so it sits right on `side` of `anchor` in their stage's column.
/// Guards in order — `name`, then `anchor`, active with parseable metadata;
/// the two distinct; the same stage (a rank only orders one column); `name`
/// unranked unless `force`. The move is then [`move_rank`]'s — the same
/// start ([`move_start`]) and the same write — between the anchor and its
/// neighbor on that side as the board shows the column.
pub fn set_rank(
    store: &dyn Store,
    write: &dyn Fn(&str, &str) -> anyhow::Result<()>,
    name: &str,
    anchor: &str,
    side: Side,
    force: bool,
) -> anyhow::Result<()> {
    let (ranks, entries) = move_start(store, name)?;
    let refuse = |msg: String| anyhow::Error::from(crate::command::Refusal(msg));
    let changes = model::list_changes(store);
    let Some(anchored) = find_active(&changes, anchor) else {
        return Err(refuse(format!(
            "cannot rank against '{anchor}': {}",
            inactive_reason(store, anchor)
        )));
    };
    model::require_valid_meta(anchored)?;
    if name == anchor {
        return Err(refuse(format!("'{name}' cannot be ranked against itself")));
    }
    let (stage, anchor_stage) = (stage_of(&entries, name), stage_of(&entries, anchor));
    if stage != anchor_stage {
        return Err(refuse(format!(
            "cannot rank '{name}' against '{anchor}': '{name}' is {} but '{anchor}' is {} \
             — a board rank only orders changes within one stage",
            stage.as_str(),
            anchor_stage.as_str()
        )));
    }
    if ranks.contains_key(name) && !force {
        return Err(refuse(format!(
            "'{name}' already has a board rank; pass --force to move it anyway"
        )));
    }
    let column = board_column(&entries, stage, name);
    let at = column
        .iter()
        .position(|n| *n == anchor)
        .expect("the anchor sits in its own column");
    let (prev, next) = match side {
        Side::Before => (at.checked_sub(1).map(|i| column[i]), Some(anchor)),
        Side::After => (Some(anchor), column.get(at + 1).copied()),
    };
    write_between(store, write, &ranks, &column, name, prev, next)
}

/// Put `name` between `prev` and `next` in its stage's column — the one step
/// the board drag and `change rank` share (spec「欄內拖排以中點 rank 單檔寫回」).
/// A neighbor missing from the column (gone since the drop) is an open end.
/// `store` is the view the board reads; `write` puts one change's board rank
/// into that change's own copy — the one the view reads it from (a worktree
/// copy for a change checked out in one), which a view that redirects reads
/// only (the worktree overlay) cannot do for itself.
pub fn move_rank(
    store: &dyn Store,
    write: &dyn Fn(&str, &str) -> anyhow::Result<()>,
    name: &str,
    prev: Option<&str>,
    next: Option<&str>,
) -> anyhow::Result<()> {
    let (ranks, entries) = move_start(store, name)?;
    let column = board_column(&entries, stage_of(&entries, name), name);
    write_between(store, write, &ranks, &column, name, prev, next)
}

/// Where a move of `name` starts: the rank table the metas hold and the
/// placeable entries in basis order. `name` must be an active change with
/// parseable metadata.
fn move_start(
    store: &dyn Store,
    name: &str,
) -> anyhow::Result<(BTreeMap<String, String>, Vec<Entry>)> {
    let changes = model::list_changes(store);
    let Some(moved) = find_active(&changes, name) else {
        anyhow::bail!("Change '{name}' not found.");
    };
    model::require_valid_meta(moved)?;
    let ranks = meta_ranks(&changes);
    let (entries, _) = entries_of(store, changes, &ranks);
    Ok((ranks, entries))
}

/// The active change called `name`; a name that could leave the changes
/// directory is none.
fn find_active<'a>(changes: &'a [Change], name: &str) -> Option<&'a Change> {
    (!unsafe_name(name))
        .then(|| changes.iter().find(|c| c.name == name))
        .flatten()
}

fn stage_of(entries: &[Entry], name: &str) -> Stage {
    entries
        .iter()
        .find(|e| e.name == name)
        .expect("valid metadata is placeable")
        .stage
}

/// `stage`'s column as the board shows it ([`shown_order`]) with `name` left
/// out: its old place does not count.
fn board_column<'a>(entries: &'a [Entry], stage: Stage, name: &str) -> Vec<&'a str> {
    shown_order(entries)
        .into_iter()
        .map(|i| &entries[i])
        .filter(|e| e.stage == stage && e.name != name)
        .map(|e| e.name.as_str())
        .collect()
}

/// `entries` by index in the order the board shows them: the plan's
/// placement, or the basis order when a dependency cycle stalls it.
fn shown_order(entries: &[Entry]) -> Vec<usize> {
    place(entries, &deps_of(entries), false)
        .map_or_else(|_| (0..entries.len()).collect(), |(order, _)| order)
}

/// Write `name`'s new rank between `prev` and `next` in `column`. When a rank
/// in the column is missing, out of its order, or not in the generated shape
/// (a hand-edited key has no midpoint), the whole column is stamped first by
/// the board's rule, so the new key has real neighbors on both sides.
/// Every checked thing comes before any write, so a refusal writes nothing: a
/// change to write needs its `.openspec.yaml`, and the stamps plus the new key
/// must cross no declared dependency. Writes go stamps first, `name` last; a
/// failed write stops the run and keeps the stamps already written — a partly
/// stamped column is still a valid order, and running the move again resumes
/// from it.
fn write_between(
    store: &dyn Store,
    write: &dyn Fn(&str, &str) -> anyhow::Result<()>,
    ranks: &BTreeMap<String, String>,
    column: &[&str],
    name: &str,
    prev: Option<&str>,
    next: Option<&str>,
) -> anyhow::Result<()> {
    let refuse = |msg: String| anyhow::Error::from(crate::command::Refusal(msg));
    let current: Vec<Option<&str>> = column
        .iter()
        .map(|n| ranks.get(*n).map(String::as_str))
        .collect();
    let usable = crate::rank::ranked_in_order(&current)
        && current.iter().flatten().all(|key| crate::rank::generated_shape(key));
    let stamps: Vec<(&str, String)> = if usable {
        Vec::new()
    } else {
        column
            .iter()
            .copied()
            .zip(crate::rank::spread(column.len()))
            .collect()
    };
    let mut keys: HashMap<&str, String> = column
        .iter()
        .filter_map(|n| ranks.get(*n).map(|r| (*n, r.clone())))
        .collect();
    keys.extend(stamps.iter().map(|(n, key)| (*n, key.clone())));
    let key = crate::rank::neighbor_midpoint(&keys, prev, next);

    // A change directory without its .openspec.yaml has no line to take the
    // rank: refuse before the first write, not halfway through the column.
    for target in stamps.iter().map(|(n, _)| *n).chain([name]) {
        if store.read_change_meta(target).is_none() {
            return Err(refuse(format!(
                "cannot write a board rank for '{target}': openspec/changes/{target}/.openspec.yaml is missing"
            )));
        }
    }
    let mut after = ranks.clone();
    after.extend(stamps.iter().map(|(n, k)| (n.to_string(), k.clone())));
    after.insert(name.to_string(), key.clone());
    check_ranks(store, model::list_changes(store), &after, name)
        .map_err(|blocked| refuse(blocked.to_string()))?;
    for (stamped, stamp) in &stamps {
        write(stamped, stamp)?;
    }
    write(name, &key)
}

/// Does `name`, where `sequence` places it, cross a declared dependency? Only
/// the [`violations`] pairs that involve `name` count — an older violation
/// between two other changes is not this move's doing. The one judgement
/// behind [`move_rank`]'s check and callers whose ranks live outside the change
/// metas (the remote board resource).
pub fn blocked_move(
    sequence: &[&str],
    depends_on: &BTreeMap<String, Vec<String>>,
    name: &str,
) -> Result<(), RankMoveBlocked> {
    let mut must_follow = Vec::new();
    let mut must_precede = Vec::new();
    for (dependent, prereq) in violations(sequence, depends_on) {
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

    /// 在 change 的 capability delta 尾端追加一個 requirement 操作（`op` 為區段名）。
    fn req(store: &TestStore, change: &str, cap: &str, op: &str, name: &str) {
        let artifact = format!("specs/{cap}/spec.md");
        let before = store.read_artifact(change, &artifact).unwrap_or_default();
        store.put_artifact(
            change,
            &artifact,
            &format!("{before}## {op} Requirements\n\n### Requirement: {name}\n\nbody\n\n"),
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

    /// `n` 個未勾選任務的 tasks.md（不勾任何一項，階段維持提案中）。
    fn open_tasks(store: &TestStore, change: &str, n: usize) {
        let md: String = (1..=n).map(|i| format!("- [ ] 1.{i} t\n")).collect();
        store.put_artifact(change, "tasks.md", &md);
    }

    #[test]
    fn unranked_sort_by_dependents_then_task_total_then_created() {
        // Scenario 缺 rank 者依被依賴數與 task 數排：x 被 d1、d2 依賴，y 12 個
        // task、z 5 個 task → x、z、y。d1、d2 與 z 同為 5 個 task，依 created 排在
        // z 之後。
        let store = store_of(&[
            ("x", "schema: spec-driven\ncreated: 2026-09-20\n"),
            ("y", "schema: spec-driven\ncreated: 2026-09-01\n"),
            ("z", "schema: spec-driven\ncreated: 2026-09-02\n"),
            (
                "d1",
                "schema: spec-driven\ncreated: 2026-09-03\ndepends_on: x\n",
            ),
            (
                "d2",
                "schema: spec-driven\ncreated: 2026-09-04\ndepends_on: x\n",
            ),
        ]);
        open_tasks(&store, "y", 12);
        for change in ["z", "d1", "d2"] {
            open_tasks(&store, change, 5);
        }
        assert_eq!(basis_order(&store), ["x", "z", "d1", "d2", "y"]);
        let plan = compute(&store).unwrap();
        assert_eq!(names(&plan), ["x", "z", "d1", "d2", "y"]);
    }

    #[test]
    fn dependents_count_each_dependent_once() {
        // 被依賴數是「有幾個 change 依賴它」：d 把 p 宣告兩次仍只算一個，e、f 各
        // 依賴 q 一次算兩個 → q 在 p 前，即使 p 建立得早。s 的自我引用不算。
        let store = store_of(&[
            ("p", "schema: spec-driven\ncreated: 2026-09-01\n"),
            ("q", "schema: spec-driven\ncreated: 2026-09-02\n"),
            (
                "d",
                "schema: spec-driven\ncreated: 2026-09-03\ndepends_on: p, p\n",
            ),
            (
                "e",
                "schema: spec-driven\ncreated: 2026-09-04\ndepends_on: q\n",
            ),
            (
                "f",
                "schema: spec-driven\ncreated: 2026-09-05\ndepends_on: q\n",
            ),
            (
                "s",
                "schema: spec-driven\ncreated: 2026-08-01\ndepends_on: s\n",
            ),
        ]);
        assert_eq!(&basis_order(&store)[..2], ["q", "p"]);
    }

    #[test]
    fn missing_tasks_sorts_last_among_unranked() {
        // 沒有 tasks.md 的還不能 apply：同層殿後，建立得再早也排在 30 個 task 的後面。
        let store = store_of(&[
            ("no-tasks", "schema: spec-driven\ncreated: 2026-08-01\n"),
            ("many-tasks", "schema: spec-driven\ncreated: 2026-09-01\n"),
        ]);
        open_tasks(&store, "many-tasks", 30);
        assert_eq!(basis_order(&store), ["many-tasks", "no-tasks"]);
    }

    #[test]
    fn board_order_is_the_placement_or_the_basis_order_on_a_cycle() {
        // 看板顯示序：照 plan 的配置順序（c 的 rank 在前，但它依賴 a，配置時排在 a
        // 之後）；依賴成環、plan 算不出來時整份退回基底序。
        let store = store_of(&[
            ("a", "schema: spec-driven\nboard_rank: n\n"),
            ("c", "schema: spec-driven\nboard_rank: b\ndepends_on: a\n"),
        ]);
        assert_eq!(basis_order(&store), ["c", "a"]);
        assert_eq!(board_order(&store), ["a", "c"]);
        store.metas.borrow_mut().insert(
            "a".to_string(),
            "schema: spec-driven\nboard_rank: n\ndepends_on: c\n".to_string(),
        );
        assert!(compute(&store).is_err());
        assert_eq!(board_order(&store), ["c", "a"]);
    }

    #[test]
    fn tasks_md_without_a_task_sorts_last_like_a_missing_one() {
        // tasks.md 只有標題、一個 task 都沒有：跟沒有 tasks.md 一樣還不能 apply，
        // 同層殿後，也不會成為 next。
        let store = store_of(&[
            ("empty", "schema: spec-driven\ncreated: 2026-08-01\n"),
            ("some", "schema: spec-driven\ncreated: 2026-09-01\n"),
        ]);
        store.put_artifact("empty", "tasks.md", "## 1. 準備\n");
        open_tasks(&store, "some", 3);
        assert_eq!(basis_order(&store), ["some", "empty"]);
        assert_eq!(compute(&store).unwrap().next.as_deref(), Some("some"));
    }

    #[test]
    fn ranked_changes_ignore_new_keys() {
        // 有 rank 者只看 rank 與名稱：同 rank 的 r-a、r-b 依名稱，不管 r-b 被依賴、
        // task 又少；有 rank 者一律在缺 rank 者前，即使後者也被依賴。
        let store = store_of(&[
            ("r-b", "schema: spec-driven\nboard_rank: n\n"),
            ("r-a", "schema: spec-driven\nboard_rank: n\n"),
            ("u", "schema: spec-driven\ncreated: 2026-09-01\n"),
            (
                "dep",
                "schema: spec-driven\ncreated: 2026-09-02\ndepends_on: r-b, u\n",
            ),
        ]);
        open_tasks(&store, "r-a", 40);
        open_tasks(&store, "r-b", 1);
        assert_eq!(basis_order(&store), ["r-a", "r-b", "u", "dep"]);
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
    fn no_depends_means_wave_one_regardless_of_overlap() {
        // 無 depends_on 一律第 1 波：三者共用 desktop-app，其中兩者還動到同一個
        // requirement——重疊只排封存順序，不推波次、不進 blockedBy。
        let store = store_of(&[
            ("a", "schema: spec-driven\ncreated: 2026-09-01\n"),
            ("b", "schema: spec-driven\ncreated: 2026-09-02\n"),
            ("c", "schema: spec-driven\ncreated: 2026-09-03\n"),
        ]);
        req(&store, "a", "desktop-app", "MODIFIED", "看板與任務");
        req(&store, "b", "desktop-app", "MODIFIED", "看板與任務");
        req(&store, "c", "desktop-app", "MODIFIED", "Conversation pin");
        let plan = compute(&store).unwrap();
        assert_eq!(waves(&plan), [vec!["a", "b", "c"]]);
        assert!(plan
            .changes
            .iter()
            .all(|c| c.wave == 1 && c.blocked_by.is_empty() && c.ready));
        assert_eq!(plan.next.as_deref(), Some("a"));
    }

    #[test]
    fn requirement_overlap_sets_archive_after_not_wave() {
        // Scenario 重疊互斥依基底順序定先後：a 與 b 各自 MODIFIED desktop-app 的
        // 「看板與任務」，a 基底在前——同波、互不阻擋，b 排在 a 之後封存。
        let store = store_of(&[
            ("a", "schema: spec-driven\ncreated: 2026-09-01\n"),
            ("b", "schema: spec-driven\ncreated: 2026-09-02\n"),
        ]);
        req(&store, "a", "desktop-app", "MODIFIED", "看板與任務");
        req(&store, "b", "desktop-app", "MODIFIED", "看板與任務");
        delta(&store, "b", "tray-status-menu");
        let plan = compute(&store).unwrap();
        let a = entry(&plan, "a");
        let b = entry(&plan, "b");
        assert_eq!((a.wave, b.wave), (1, 1));
        assert!(a.blocked_by.is_empty() && b.blocked_by.is_empty());
        assert!(a.ready && b.ready);
        assert_eq!(b.archive_after, ["a"]);
        assert!(a.archive_after.is_empty());
        let both = |other: &str| {
            vec![req_overlap(
                other,
                "desktop-app",
                "看板與任務",
                DeltaOp::Modified,
                DeltaOp::Modified,
                false,
            )]
        };
        assert_eq!(a.requirement_overlap, both("b"));
        assert_eq!(b.requirement_overlap, both("a"));
        // 目錄級 overlaps 原樣保留（第二刀前的桌面與 remote 讀者照舊）。
        let dirs = |c: &PlanChange| {
            c.overlaps
                .iter()
                .map(|o| (o.change.clone(), o.capabilities.clone()))
                .collect::<Vec<_>>()
        };
        assert_eq!(dirs(a), [("b".to_string(), vec!["desktop-app".to_string()])]);
        assert_eq!(dirs(b), [("a".to_string(), vec!["desktop-app".to_string()])]);
    }

    #[test]
    fn same_capability_different_requirement_plans_no_overlap() {
        // Scenario 同 capability 不同 requirement 不算重疊。
        let store = store_of(&[
            ("a", "schema: spec-driven\ncreated: 2026-09-01\n"),
            ("b", "schema: spec-driven\ncreated: 2026-09-02\n"),
        ]);
        req(&store, "a", "desktop-app", "MODIFIED", "Run rewind point");
        req(&store, "b", "desktop-app", "MODIFIED", "Conversation pin");
        let plan = compute(&store).unwrap();
        assert_eq!(waves(&plan), [vec!["a", "b"]]);
        for c in &plan.changes {
            assert!(c.requirement_overlap.is_empty() && c.archive_after.is_empty(), "{c:?}");
        }
    }

    #[test]
    fn added_side_archives_first_regardless_of_basis() {
        // Scenario ADDED 先於動到同名的 MODIFIED：基底序 b 在 a 前，仍是 b 排在 a 之後封存。
        let store = store_of(&[
            ("a", "schema: spec-driven\ncreated: 2026-09-02\n"),
            ("b", "schema: spec-driven\ncreated: 2026-09-01\n"),
        ]);
        req(&store, "a", "export", "ADDED", "匯出");
        req(&store, "b", "export", "MODIFIED", "匯出");
        let plan = compute(&store).unwrap();
        assert_eq!(names(&plan), ["b", "a"], "基底序");
        let a = entry(&plan, "a");
        let b = entry(&plan, "b");
        assert_eq!(b.archive_after, ["a"]);
        assert!(a.archive_after.is_empty());
        assert_eq!((a.wave, b.wave), (1, 1));
    }

    #[test]
    fn conflict_not_in_archive_after() {
        // Scenario 兩個 ADDED 同名標衝突：雙方各列對方為 conflict，archiveAfter 皆空。
        let store = store_of(&[
            ("a", "schema: spec-driven\ncreated: 2026-09-01\n"),
            ("b", "schema: spec-driven\ncreated: 2026-09-02\n"),
        ]);
        req(&store, "a", "export", "ADDED", "匯出");
        req(&store, "b", "export", "ADDED", "匯出");
        let plan = compute(&store).unwrap();
        for (me, other) in [("a", "b"), ("b", "a")] {
            let c = entry(&plan, me);
            assert_eq!(
                c.requirement_overlap,
                [req_overlap(other, "export", "匯出", DeltaOp::Added, DeltaOp::Added, true)]
            );
            assert!(c.archive_after.is_empty() && c.blocked_by.is_empty());
        }
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
    fn in_progress_overlap_partner_archives_first() {
        // Scenario 已開工者排在未開工者之前：b 較早建立，但 a 已開工——a 配置在前，
        // 兩者 MODIFIED 同一個 requirement，b 排在 a 之後封存，兩者同為第 1 波。
        let store = store_of(&[
            (
                "a",
                "schema: spec-driven\ncreated: 2026-09-05\nstarted_at: 2026-09-06\n",
            ),
            ("b", "schema: spec-driven\ncreated: 2026-09-01\n"),
        ]);
        req(&store, "a", "desktop-app", "MODIFIED", "看板與任務");
        req(&store, "b", "desktop-app", "MODIFIED", "看板與任務");
        let plan = compute(&store).unwrap();
        assert_eq!(names(&plan), ["a", "b"]);
        assert_eq!(entry(&plan, "b").archive_after, ["a"]);
        assert!(entry(&plan, "a").archive_after.is_empty());
        assert!(entry(&plan, "b").blocked_by.is_empty());
        assert_eq!(waves(&plan), [vec!["a", "b"]]);
    }

    /// Example「五個 change 的基底順序與波次」的五列（delta 觸碰依表設定）。
    fn five_change_example() -> TestStore {
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
        req(&store, "r1", "tray-status-menu", "MODIFIED", "列首波次");
        req(&store, "p2", "desktop-app", "MODIFIED", "看板與任務");
        req(&store, "n5", "archive-skill", "ADDED", "候選清單");
        req(&store, "n3", "desktop-app", "MODIFIED", "看板與任務");
        req(&store, "n4", "discuss-skill", "MODIFIED", "輪次");
        store
    }

    #[test]
    fn spec_example_five_changes() {
        // Example「五個 change 的基底順序與波次」逐列。
        let plan = compute(&five_change_example()).unwrap();
        assert_eq!(names(&plan), ["r1", "p2", "n5", "n3", "n4"], "基底序");
        assert_eq!(waves(&plan), [vec!["r1", "p2", "n5", "n3"], vec!["n4"]]);
        assert_eq!(plan.next.as_deref(), Some("n5"));
        let rows: Vec<(&str, usize, Vec<&str>, Vec<&str>)> = plan
            .changes
            .iter()
            .map(|c| {
                (
                    c.name.as_str(),
                    c.wave,
                    c.blocked_by.iter().map(String::as_str).collect(),
                    c.archive_after.iter().map(String::as_str).collect(),
                )
            })
            .collect();
        assert_eq!(
            rows,
            [
                ("r1", 1, vec![], vec![]),
                ("p2", 1, vec![], vec![]),
                ("n5", 1, vec![], vec![]),
                ("n3", 1, vec![], vec!["p2"]),
                ("n4", 2, vec!["n3"], vec![]),
            ]
        );
        assert_eq!(entry(&plan, "r1").stage, Stage::Ready);
        assert_eq!(entry(&plan, "p2").stage, Stage::InProgress);
        assert_eq!(entry(&plan, "n5").stage, Stage::Proposed);
    }

    #[test]
    fn strict_overlap_reproduces_capability_level_waves() {
        // 同一份五 change 資料在 strict 下回到本變更前的輸出：n3 與 p2 共用
        // desktop-app 被推到第 2 波並被 p2 擋住；requirementOverlap 與
        // archiveAfter 照常計算。
        let plan = compute_strict(&five_change_example()).unwrap();
        assert_eq!(
            waves(&plan),
            [vec!["r1", "p2", "n5"], vec!["n3"], vec!["n4"]]
        );
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
        assert_eq!(plan.next.as_deref(), Some("n5"));
        assert_eq!(entry(&plan, "n3").archive_after, ["p2"]);

        // Scenario strict-overlap 退回目錄級推波：同 capability、不同 requirement。
        let store = store_of(&[
            ("a", "schema: spec-driven\ncreated: 2026-09-01\n"),
            ("b", "schema: spec-driven\ncreated: 2026-09-02\n"),
        ]);
        req(&store, "a", "desktop-app", "MODIFIED", "Run rewind point");
        req(&store, "b", "desktop-app", "MODIFIED", "Conversation pin");
        let plan = compute_strict(&store).unwrap();
        let (a, b) = (entry(&plan, "a"), entry(&plan, "b"));
        assert_eq!((a.wave, b.wave), (1, 2));
        assert_eq!(b.blocked_by, ["a"]);
        assert!(a.ready && !b.ready);
        assert!(b.archive_after.is_empty() && b.requirement_overlap.is_empty());
        assert_eq!(waves(&compute(&store).unwrap()), [vec!["a", "b"]]);
    }

    #[test]
    fn strict_overlap_keeps_the_created_order_for_the_unranked() {
        // `--strict-overlap` 是執行期回退：缺 rank 者只依 created 排（本 change 之前
        // 的舊鍵），輸出回到舊版。a 有 12 個 task、較早建立；b 只有 5 個 task。
        let store = store_of(&[
            ("a", "schema: spec-driven\ncreated: 2026-09-01\n"),
            ("b", "schema: spec-driven\ncreated: 2026-09-02\n"),
        ]);
        open_tasks(&store, "a", 12);
        open_tasks(&store, "b", 5);
        req(&store, "a", "desktop-app", "MODIFIED", "Run rewind point");
        req(&store, "b", "desktop-app", "MODIFIED", "Conversation pin");
        let strict = compute_strict(&store).unwrap();
        assert_eq!(waves(&strict), [vec!["a"], vec!["b"]]);
        assert_eq!(entry(&strict, "b").blocked_by, ["a"]);
        // 預設模式照新 tie-break：task 少的 b 在前，兩者同波。
        assert_eq!(waves(&compute(&store).unwrap()), [vec!["b", "a"]]);
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
        // design D3 的 payload 形狀：四鍵、九鍵皆 camelCase，skipped 空時為 []。
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
                "archiveAfter",
                "blockedBy",
                "dependsOn",
                "name",
                "overlaps",
                "ready",
                "requirementOverlap",
                "stage",
                "wave"
            ]
        );
        assert_eq!(b["archiveAfter"], serde_json::json!([]));
        assert_eq!(b["requirementOverlap"], serde_json::json!([]));
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

    /// The rank-move check for `name` taking `rank`: [`check_ranks`] over the
    /// metas' ranks with that one replaced — what a drag or `change rank` asks.
    fn check_move(store: &TestStore, name: &str, rank: &str) -> Result<(), RankMoveBlocked> {
        let changes = model::list_changes(store);
        let mut ranks = meta_ranks(&changes);
        ranks.insert(name.to_string(), rank.to_string());
        check_ranks(store, changes, &ranks, name)
    }

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
    fn blocked_move_counts_only_the_pairs_that_involve_the_moved_change() {
        // add-change-plan-remote D6：rank 不在 meta 的呼叫端（remote board resource）
        // 以自己的拖放後序列呼叫同一判定、得到同一句訊息。
        let deps = deps_map(&[("c", &["a"]), ("y", &["x"])]);
        let err = blocked_move(&["c", "a", "y", "x"], &deps, "c").unwrap_err();
        assert_eq!(err.change, "c");
        assert_eq!(err.must_follow, ["a"]);
        assert!(err.must_precede.is_empty(), "y 與 x 的違規與 c 無關");
        assert_eq!(err.to_string(), "cannot move 'c' there: it depends on a");
        let err = blocked_move(&["c", "a"], &deps, "a").unwrap_err();
        assert!(err.must_follow.is_empty());
        assert_eq!(err.must_precede, ["c"]);
        assert_eq!(blocked_move(&["a", "c", "y", "x"], &deps, "c"), Ok(()));
    }

    #[test]
    fn rank_move_check_refuses_dragging_a_change_before_its_prerequisite() {
        // Scenario 拖到宣告前置之前被拒：c 依賴 a，擬把 c 的 rank 排到 a 之前。
        let store = store_of(&[
            ("a", "schema: spec-driven\nboard_rank: n\n"),
            ("c", "schema: spec-driven\nboard_rank: p\ndepends_on: a\n"),
        ]);
        let err = check_move(&store, "c", "a").unwrap_err();
        assert_eq!(err.change, "c");
        assert_eq!(err.must_follow, ["a"]);
        assert!(err.must_precede.is_empty());
        assert!(
            err.to_string().contains("'c'") && err.to_string().contains("a"),
            "{err}"
        );
        // 排到 a 之後則允許。
        assert_eq!(check_move(&store, "c", "z"), Ok(()));
    }

    #[test]
    fn rank_move_check_refuses_dragging_a_change_behind_its_dependent() {
        // 宣告依賴它的 change 會排到它前面 → must_precede。
        let store = store_of(&[
            ("a", "schema: spec-driven\nboard_rank: a\n"),
            ("c", "schema: spec-driven\nboard_rank: n\ndepends_on: a\n"),
        ]);
        let err = check_move(&store, "a", "z").unwrap_err();
        assert_eq!(err.change, "a");
        assert!(err.must_follow.is_empty());
        assert_eq!(err.must_precede, ["c"]);
    }

    #[test]
    fn rank_move_check_allows_flipping_overlap_partners() {
        // Scenario 重疊夥伴翻轉允許：只因 delta 重疊而依序，翻轉是合法選擇。
        let store = store_of(&[
            ("a", "schema: spec-driven\nboard_rank: n\n"),
            ("b", "schema: spec-driven\nboard_rank: z\n"),
        ]);
        delta(&store, "a", "desktop-app");
        delta(&store, "b", "desktop-app");
        assert_eq!(check_move(&store, "b", "f"), Ok(()));
    }

    #[test]
    fn rank_move_check_only_looks_at_the_same_stage() {
        // 前置在另一欄（進行中）時，提案中的 c 在自己欄內怎麼排都不違規。
        let store = store_of(&[
            (
                "a",
                "schema: spec-driven\nboard_rank: n\nstarted_at: 2026-09-06\n",
            ),
            ("c", "schema: spec-driven\nboard_rank: p\ndepends_on: a\n"),
        ]);
        assert_eq!(check_move(&store, "c", "a"), Ok(()));
    }

    #[test]
    fn rank_move_check_never_writes() {
        let store = store_of(&[
            ("a", "schema: spec-driven\nboard_rank: n\n"),
            ("c", "schema: spec-driven\nboard_rank: p\ndepends_on: a\n"),
        ]);
        assert!(check_move(&store, "c", "a").is_err());
        assert_eq!(check_move(&store, "c", "z"), Ok(()));
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
        assert_eq!(check_move(&store, "a", "n"), Ok(()));
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
    fn rank_move_check_tolerates_unknown_names_and_odd_ranks() {
        // 未知名稱無可交叉 → Ok（寫入端才報 not found）；非法 rank 只當位元組
        // 排序、不 panic（合法性由 set_board_rank 守）。
        let store = store_of(&[
            ("a", "schema: spec-driven\nboard_rank: n\n"),
            ("c", "schema: spec-driven\nboard_rank: p\ndepends_on: a\n"),
        ]);
        assert_eq!(check_move(&store, "ghost", "n"), Ok(()));
        // 'Z' (0x5a) 排在 'n' (0x6e) 前：答案確定為「會跨過前置 a」，而非 panic。
        let err = check_move(&store, "c", "ZZ9").unwrap_err();
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

    // --- requirement 級觸碰（design「requirement 級重疊自 delta 內文解析」）---

    #[test]
    fn delta_touches_lists_every_operation_including_rename_from_and_to() {
        // 四種區段各一，RENAMED 的 FROM 與 TO 皆計；CRLF 的 delta 與 LF 讀出同名。
        let store = store_of(&[("a", PROPOSED)]);
        store.put_artifact(
            "a",
            "specs/desktop-app/spec.md",
            "## ADDED Requirements\n\n### Requirement: 匯出\n\nbody\n\n\
             ## MODIFIED Requirements\n\n### Requirement: 看板與任務\n\nbody\n\n\
             ## REMOVED Requirements\n\n### Requirement: 舊功能\n\n\
             ## RENAMED Requirements\n\n- FROM: `### Requirement: 舊名`\n- TO: `### Requirement: 新名`\n",
        );
        store.put_artifact(
            "a",
            "specs/tray-status-menu/spec.md",
            "## MODIFIED Requirements\r\n\r\n### Requirement: 列首波次\r\n\r\nbody\r\n",
        );
        let touch = |cap: &str, t: Touch, req: &str| (cap.to_string(), t, req.to_string());
        assert_eq!(
            delta_touches(&store, "a"),
            [
                touch("desktop-app", Touch::Added, "匯出"),
                touch("desktop-app", Touch::Modified, "看板與任務"),
                touch("desktop-app", Touch::Removed, "舊功能"),
                touch("desktop-app", Touch::RenamedFrom, "舊名"),
                touch("desktop-app", Touch::RenamedTo, "新名"),
                touch("tray-status-menu", Touch::Modified, "列首波次"),
            ]
        );
        // 輸出只有四種操作：改名的兩端都寫 RENAMED；配對時舊名是「拿走」、新名是「帶進」。
        assert_eq!(Touch::RenamedFrom.op(), DeltaOp::Renamed);
        assert_eq!(Touch::RenamedTo.op(), DeltaOp::Renamed);
        assert_eq!(
            [
                Touch::Added,
                Touch::Modified,
                Touch::Removed,
                Touch::RenamedFrom,
                Touch::RenamedTo,
            ]
            .map(Touch::role),
            [
                Role::BringsIn,
                Role::Uses,
                Role::TakesAway,
                Role::TakesAway,
                Role::BringsIn,
            ]
        );
    }

    #[test]
    fn delta_touches_empty_delta_is_zero_touch() {
        // 空檔、只有空白、只有區段標頭、根本沒有 delta：一律零觸碰，不報錯。
        let store = store_of(&[("a", PROPOSED), ("b", PROPOSED), ("c", PROPOSED), ("d", PROPOSED)]);
        store.put_artifact("a", "specs/desktop-app/spec.md", "");
        store.put_artifact("b", "specs/desktop-app/spec.md", "  \n\n");
        delta(&store, "c", "desktop-app");
        for change in ["a", "b", "c", "d"] {
            assert!(delta_touches(&store, change).is_empty(), "{change}");
        }
    }

    // --- requirement 級配對規則（spec「執行順序的基底與拓樸修正」）---

    /// 配對規則測試用的觸碰：正式規格一個名稱都沒有。
    fn touches(rows: &[(&str, Touch, &str)]) -> Touches {
        touches_with(false, rows)
    }

    /// 正式規格有全部這些名稱（`in_canon`），或一個都沒有。
    fn touches_with(in_canon: bool, rows: &[(&str, Touch, &str)]) -> Touches {
        touch_map(
            rows.iter()
                .map(|(cap, t, req)| (cap.to_string(), *t, req.to_string()))
                .collect(),
            |_, _| in_canon,
        )
    }

    fn req_overlap(
        change: &str,
        cap: &str,
        req: &str,
        own: DeltaOp,
        other: DeltaOp,
        conflict: bool,
    ) -> RequirementOverlap {
        RequirementOverlap {
            change: change.to_string(),
            capability: cap.to_string(),
            requirement: req.to_string(),
            own_operation: own,
            other_operation: other,
            conflict,
        }
    }

    #[test]
    fn requirement_overlap_pairs_modified_with_modified() {
        // 兩邊都 MODIFIED 同名是重疊；同一對方依 capability 再依 requirement 升冪
        // 列出。先後照封存序：對方在前，我方排在它之後封存。
        let a = touches(&[
            ("desktop-app", Touch::Modified, "看板與任務"),
            ("alpha", Touch::Modified, "舊名"),
        ]);
        let b = touches(&[
            ("desktop-app", Touch::Modified, "看板與任務"),
            ("alpha", Touch::Modified, "舊名"),
        ]);
        let got = overlaps_with(&a, "b", &b);
        assert_eq!(
            got,
            [
                req_overlap("b", "alpha", "舊名", DeltaOp::Modified, DeltaOp::Modified, false),
                req_overlap("b", "desktop-app", "看板與任務", DeltaOp::Modified, DeltaOp::Modified, false),
            ]
        );
        assert!(archives_after(&a, &b, true));
        assert!(!archives_after(&a, &b, false));
    }

    #[test]
    fn a_take_away_pairs_by_the_role_order() {
        // 拿走（REMOVED、改名的舊名）照角色先後配對：兩邊都拿走是衝突、誰都不排；
        // 對上修改時修改方先；對上帶進時，正式規格有這個名稱就拿走方先（名稱空出來
        // 才新增得了），沒有就帶進方先（名稱有了才拿走得了）。
        let removed = |c| touches_with(c, &[("cap", Touch::Removed, "X")]);
        let renamed_from = |c| touches_with(c, &[("cap", Touch::RenamedFrom, "X")]);
        let modified = |c| touches_with(c, &[("cap", Touch::Modified, "X")]);
        let added = |c| touches_with(c, &[("cap", Touch::Added, "X")]);
        for in_canon in [true, false] {
            assert_eq!(
                overlaps_with(&removed(in_canon), "b", &renamed_from(in_canon)),
                [req_overlap("b", "cap", "X", DeltaOp::Removed, DeltaOp::Renamed, true)]
            );
            assert!(!overlaps_with(&removed(in_canon), "b", &modified(in_canon))[0].conflict);
            assert!(!overlaps_with(&removed(in_canon), "b", &added(in_canon))[0].conflict);
            for partner_first in [true, false] {
                assert!(!archives_after(&removed(in_canon), &renamed_from(in_canon), partner_first));
                assert!(!archives_after(&renamed_from(in_canon), &removed(in_canon), partner_first));
                assert!(archives_after(&removed(in_canon), &modified(in_canon), partner_first));
                assert!(!archives_after(&modified(in_canon), &removed(in_canon), partner_first));
                assert_eq!(archives_after(&added(in_canon), &removed(in_canon), partner_first), in_canon);
                assert_eq!(archives_after(&removed(in_canon), &added(in_canon), partner_first), !in_canon);
            }
        }
    }

    #[test]
    fn added_pairs_with_modified_as_overlap() {
        // ADDED 方一律先封存：不論配置先後，MODIFIED 方排在 ADDED 方之後、ADDED 方不排。
        let added = touches(&[("export", Touch::Added, "匯出")]);
        let modified = touches(&[("export", Touch::Modified, "匯出")]);
        let from_added = overlaps_with(&added, "m", &modified);
        let from_modified = overlaps_with(&modified, "a", &added);
        assert_eq!(
            from_added,
            [req_overlap("m", "export", "匯出", DeltaOp::Added, DeltaOp::Modified, false)]
        );
        assert_eq!(
            from_modified,
            [req_overlap("a", "export", "匯出", DeltaOp::Modified, DeltaOp::Added, false)]
        );
        for partner_first in [true, false] {
            assert!(!archives_after(&added, &modified, partner_first));
            assert!(archives_after(&modified, &added, partner_first));
        }
    }

    #[test]
    fn added_with_added_is_conflict() {
        // 兩邊 ADDED 同名是改名問題不是順序問題：標 conflict，誰都不排在誰之後。
        let a = touches(&[("export", Touch::Added, "匯出")]);
        let b = touches(&[("export", Touch::Added, "匯出")]);
        let got = overlaps_with(&a, "b", &b);
        assert_eq!(
            got,
            [req_overlap("b", "export", "匯出", DeltaOp::Added, DeltaOp::Added, true)]
        );
        for partner_first in [true, false] {
            assert!(!archives_after(&a, &b, partner_first));
        }
        // payload 形狀：六鍵 camelCase，操作為大寫字串。
        assert_eq!(
            serde_json::to_value(&got[0]).unwrap(),
            serde_json::json!({
                "change": "b",
                "capability": "export",
                "requirement": "匯出",
                "ownOperation": "ADDED",
                "otherOperation": "ADDED",
                "conflict": true,
            })
        );
    }

    #[test]
    fn a_rename_target_pairs_like_an_added_requirement() {
        // 改名的新名在落地前不存在，配對時與 ADDED 同一套：對上 ADDED 同名是衝突；
        // 對上 MODIFIED 時改名方先封存、不論配置先後。改名的舊名與 REMOVED 同一套：
        // 對上 MODIFIED 時修改方先。
        let renamed_to = touches(&[("export", Touch::RenamedTo, "匯出")]);
        let added = touches(&[("export", Touch::Added, "匯出")]);
        let modified = touches(&[("export", Touch::Modified, "匯出")]);
        let renamed_from = touches(&[("export", Touch::RenamedFrom, "匯出")]);

        assert_eq!(
            overlaps_with(&renamed_to, "b", &added),
            [req_overlap("b", "export", "匯出", DeltaOp::Renamed, DeltaOp::Added, true)]
        );
        for partner_first in [true, false] {
            assert!(!archives_after(&renamed_to, &added, partner_first));
            assert!(!archives_after(&added, &renamed_to, partner_first));
            assert!(!archives_after(&renamed_to, &modified, partner_first));
            assert!(archives_after(&modified, &renamed_to, partner_first));
        }
        assert!(!overlaps_with(&renamed_from, "m", &modified)[0].conflict);
        for partner_first in [true, false] {
            assert!(archives_after(&renamed_from, &modified, partner_first));
            assert!(!archives_after(&modified, &renamed_from, partner_first));
        }
    }

    #[test]
    fn a_name_brought_in_orders_the_pair_over_the_placement() {
        // 一對 change 有多個同名 requirement 時只取一個方向：bb 較早建立先配置，
        // R2 兩邊 MODIFIED 本該 aa 在後；但 R1 是 aa 新增、bb 修改，bb 必須等
        // aa 落地——這個方向優先，aa 不再列 bb（否則兩邊互等）。
        let store = store_of(&[
            ("aa", "schema: spec-driven\ncreated: 2026-09-02\n"),
            ("bb", "schema: spec-driven\ncreated: 2026-09-01\n"),
        ]);
        req(&store, "aa", "cap", "ADDED", "R1");
        req(&store, "aa", "cap", "MODIFIED", "R2");
        req(&store, "bb", "cap", "MODIFIED", "R1");
        req(&store, "bb", "cap", "MODIFIED", "R2");
        let plan = compute(&store).unwrap();
        assert_eq!(names(&plan), ["bb", "aa"]);
        assert_eq!(entry(&plan, "bb").archive_after, ["aa"]);
        assert!(entry(&plan, "aa").archive_after.is_empty());
        assert_eq!(entry(&plan, "aa").requirement_overlap.len(), 2);
    }

    #[test]
    fn archive_order_never_runs_in_a_circle_through_placement_ordered_pairs() {
        // 三個 change 兩兩重疊：c 新增 R、a 修改 R，所以 a 一定等 c；a 與 b、b 與 c
        // 只照順序。照配置序（a、b、c）決定後兩對會繞成 a→c→b→a 的環。改照封存序
        // ——配置序經「帶進名稱」修正後為 b、c、a——就沒有環：b 先、c 次、a 最後。
        let store = store_of(&[
            ("a", "schema: spec-driven\ncreated: 2026-09-01\n"),
            ("b", "schema: spec-driven\ncreated: 2026-09-01\n"),
            ("c", "schema: spec-driven\ncreated: 2026-09-01\n"),
        ]);
        open_tasks(&store, "a", 3);
        open_tasks(&store, "b", 10);
        open_tasks(&store, "c", 20);
        req(&store, "a", "cap", "MODIFIED", "R");
        req(&store, "a", "cap", "MODIFIED", "S");
        req(&store, "b", "cap", "MODIFIED", "S");
        req(&store, "b", "cap", "MODIFIED", "T");
        req(&store, "c", "cap", "ADDED", "R");
        req(&store, "c", "cap", "MODIFIED", "T");
        let plan = compute(&store).unwrap();
        assert_eq!(names(&plan), ["a", "b", "c"]);
        assert_eq!(entry(&plan, "a").archive_after, ["b", "c"]);
        assert!(entry(&plan, "b").archive_after.is_empty());
        assert_eq!(entry(&plan, "c").archive_after, ["b"]);
    }

    #[test]
    fn a_conflict_leaves_the_order_of_another_shared_name_to_the_placement() {
        // 兩邊都新增 R1（衝突，要改名）又都修改 R2：衝突不決定方向，R2 照順序——
        // 先配置的 a 在前，b 列 a；a 不列 b。
        let store = store_of(&[
            ("a", "schema: spec-driven\ncreated: 2026-09-01\n"),
            ("b", "schema: spec-driven\ncreated: 2026-09-02\n"),
        ]);
        for change in ["a", "b"] {
            req(&store, change, "cap", "ADDED", "R1");
            req(&store, change, "cap", "MODIFIED", "R2");
        }
        let plan = compute(&store).unwrap();
        assert_eq!(entry(&plan, "b").archive_after, ["a"]);
        assert!(entry(&plan, "a").archive_after.is_empty());
        assert!(entry(&plan, "b").requirement_overlap.iter().any(|o| o.conflict && o.requirement == "R1"));
    }

    /// 讓 `cap` 的正式規格現在就有這些 requirement。
    fn canon(store: &TestStore, cap: &str, names: &[&str]) {
        let body: String = names
            .iter()
            .map(|n| format!("### Requirement: {n}\n\nbody\n\n"))
            .collect();
        store
            .canonical
            .borrow_mut()
            .insert(cap.to_string(), format!("## Requirements\n\n{body}"));
    }

    #[test]
    fn a_name_the_canon_carries_is_taken_away_before_it_is_brought_back() {
        // 正式規格已有 Y：c 移除 Y、d 新增 Y。d 要等 Y 被拿走才新增得了，所以 c 先
        // 封存，不論配置先後（d 較早建立、配置在前）。把 X 改名掉的 e 對上新增 X 的
        // f 也一樣：e 先。
        let store = store_of(&[
            ("c", "schema: spec-driven\ncreated: 2026-09-02\n"),
            ("d", "schema: spec-driven\ncreated: 2026-09-01\n"),
            ("e", "schema: spec-driven\ncreated: 2026-09-04\n"),
            ("f", "schema: spec-driven\ncreated: 2026-09-03\n"),
        ]);
        canon(&store, "cap", &["X", "Y"]);
        req(&store, "c", "cap", "REMOVED", "Y");
        req(&store, "d", "cap", "ADDED", "Y");
        store.put_artifact(
            "e",
            "specs/cap/spec.md",
            "## RENAMED Requirements\n\n- FROM: `### Requirement: X`\n- TO: `### Requirement: Z`\n",
        );
        req(&store, "f", "cap", "ADDED", "X");
        let plan = compute(&store).unwrap();
        assert_eq!(names(&plan), ["d", "c", "f", "e"]);
        assert_eq!(entry(&plan, "d").archive_after, ["c"]);
        assert!(entry(&plan, "c").archive_after.is_empty());
        assert_eq!(entry(&plan, "f").archive_after, ["e"]);
        assert!(entry(&plan, "e").archive_after.is_empty());
        for change in ["c", "d", "e", "f"] {
            assert!(
                entry(&plan, change).requirement_overlap.iter().all(|o| !o.conflict),
                "{change}"
            );
        }
    }

    #[test]
    fn a_use_archives_before_a_take_away() {
        // 正式規格有 X：a 移除 X、b 修改 X。a 先封存的話 b 就找不到 X，所以 b 先，
        // 不論配置先後（a 較早建立、配置在前）。
        let store = store_of(&[
            ("a", "schema: spec-driven\ncreated: 2026-09-01\n"),
            ("b", "schema: spec-driven\ncreated: 2026-09-02\n"),
        ]);
        canon(&store, "cap", &["X"]);
        req(&store, "a", "cap", "REMOVED", "X");
        req(&store, "b", "cap", "MODIFIED", "X");
        let plan = compute(&store).unwrap();
        assert_eq!(names(&plan), ["a", "b"]);
        assert_eq!(entry(&plan, "a").archive_after, ["b"]);
        assert!(entry(&plan, "b").archive_after.is_empty());
    }

    #[test]
    fn taking_the_same_name_away_twice_is_a_conflict() {
        // 正式規格有 X：a 移除 X、b 把 X 改名掉。後封存的那一個會因 X 已不存在而失敗，
        // 怎麼排都一樣：標 conflict，誰都不排在誰之後。
        let store = store_of(&[
            ("a", "schema: spec-driven\ncreated: 2026-09-01\n"),
            ("b", "schema: spec-driven\ncreated: 2026-09-02\n"),
        ]);
        canon(&store, "cap", &["X"]);
        req(&store, "a", "cap", "REMOVED", "X");
        store.put_artifact(
            "b",
            "specs/cap/spec.md",
            "## RENAMED Requirements\n\n- FROM: `### Requirement: X`\n- TO: `### Requirement: W`\n",
        );
        let plan = compute(&store).unwrap();
        for (change, other) in [("a", "b"), ("b", "a")] {
            let e = entry(&plan, change);
            assert!(e.archive_after.is_empty(), "{change}");
            assert!(
                e.requirement_overlap
                    .iter()
                    .any(|o| o.change == other && o.requirement == "X" && o.conflict),
                "{change}"
            );
        }
    }

    #[test]
    fn names_brought_in_both_ways_list_each_other() {
        // 兩個方向都由「帶進來的名字」推出：誰先封存，另一邊的 MODIFIED 都找不到
        // 目標——真的互卡，兩邊都列對方，要人調整拆分。
        let a = touches(&[("cap", Touch::Added, "R1"), ("cap", Touch::Modified, "R3")]);
        let b = touches(&[("cap", Touch::Modified, "R1"), ("cap", Touch::Added, "R3")]);
        for partner_first in [true, false] {
            assert!(archives_after(&a, &b, partner_first));
            assert!(archives_after(&b, &a, partner_first));
        }
    }

    #[test]
    fn same_capability_different_requirement_is_not_overlap() {
        let a = touches(&[("desktop-app", Touch::Modified, "Run rewind point")]);
        let b = touches(&[("desktop-app", Touch::Modified, "Conversation pin")]);
        assert!(overlaps_with(&a, "b", &b).is_empty());
        assert!(!archives_after(&a, &b, true));
        // 同名但不同 capability 也不算。
        let c = touches(&[("tray-status-menu", Touch::Modified, "Run rewind point")]);
        assert!(overlaps_with(&a, "c", &c).is_empty());
    }

    #[test]
    fn delta_op_strings_match_the_json_values() {
        // wire 以字串搬運操作：as_str／parse 與 serde 的大寫值同一份字面。
        for op in [DeltaOp::Added, DeltaOp::Modified, DeltaOp::Removed, DeltaOp::Renamed] {
            assert_eq!(serde_json::to_value(op).unwrap(), op.as_str());
            assert_eq!(DeltaOp::parse(op.as_str()), Some(op));
        }
        assert_eq!(DeltaOp::parse("added"), None);
        assert_eq!(DeltaOp::parse("MOVED"), None);
    }

    // --- set_rank（spec「change rank 動詞寫入看板順序鍵」）---

    /// The single-store writer: every rank lands in `store` itself.
    fn rank_writer(store: &TestStore) -> impl Fn(&str, &str) -> anyhow::Result<()> + '_ {
        move |change, key| model::set_board_rank(store, change, key)
    }

    fn rank_of(store: &TestStore, name: &str) -> Option<String> {
        model::ChangeMeta::from_text(Some(&store.meta(name)))
            .expect("meta parses")
            .board_rank
    }

    #[test]
    fn rank_before_ranked_anchor_writes_only_the_moved_change() {
        // Scenario 無 rank 的 change 排到另一個之前：欄內只有 a（rank n）與 c →
        // 只寫 c 一檔、rank 小於 n，隨後 plan 的順序 c 在 a 前。
        let store = store_of(&[
            ("a", "schema: spec-driven\nboard_rank: n\n"),
            ("c", "schema: spec-driven\ncreated: 2026-09-01\n"),
        ]);
        set_rank(&store, &rank_writer(&store), "c", "a", Side::Before, false).unwrap();
        assert!(rank_of(&store, "c").unwrap().as_str() < "n");
        assert_eq!(*store.meta_write_log.borrow(), ["c"]);
        assert_eq!(names(&compute(&store).unwrap()), ["c", "a"]);
    }

    #[test]
    fn rank_before_unranked_anchor_stamps_column_then_writes() {
        // Scenario 缺 rank 的欄先整欄補章：a、b、c 皆無 rank，c --before b → a、b
        // 依顯示序補章（a 小於 b），c 落在兩者之間，plan 為 a、c、b。寫入順序是
        // 補章逐一、最後寫 c；壞 meta 的卡不補章、逐位元不變。
        const BAD: &str = ": : :\n\t bad yaml [unclosed\n";
        let store = store_of(&[
            ("a", "schema: spec-driven\ncreated: 2026-09-01\n"),
            ("b", "schema: spec-driven\ncreated: 2026-09-02\n"),
            ("c", "schema: spec-driven\ncreated: 2026-09-03\n"),
            ("bad", BAD),
        ]);
        set_rank(&store, &rank_writer(&store), "c", "b", Side::Before, false).unwrap();
        let rank = |name: &str| rank_of(&store, name).unwrap();
        assert!(rank("a") < rank("c") && rank("c") < rank("b"));
        assert_eq!(*store.meta_write_log.borrow(), ["a", "b", "c"]);
        assert_eq!(store.meta("bad"), BAD);
        assert_eq!(names(&compute(&store).unwrap()), ["a", "c", "b"]);
    }

    #[test]
    fn rank_restamps_a_column_whose_ranks_disagree_with_the_display_order() {
        // 看板拖排同一判定：c 的 rank 在 a 前但 c 依賴 a，看板顯示 a、c。rank 序與
        // 顯示序不一致時整欄依顯示序重派，x 才落得進 a 與 c 之間。
        let store = store_of(&[
            ("a", "schema: spec-driven\nboard_rank: f\n"),
            ("c", "schema: spec-driven\nboard_rank: b\ndepends_on: a\n"),
            ("x", "schema: spec-driven\ncreated: 2026-09-01\n"),
        ]);
        assert_eq!(names(&compute(&store).unwrap()), ["a", "c", "x"]);
        set_rank(&store, &rank_writer(&store), "x", "c", Side::Before, false).unwrap();
        assert_eq!(*store.meta_write_log.borrow(), ["a", "c", "x"]);
        assert_eq!(names(&compute(&store).unwrap()), ["a", "x", "c"]);
    }

    #[test]
    fn rank_stops_at_a_failed_stamp_and_keeps_the_earlier_ones() {
        // 補章中途失敗：停下回報，已寫的補章保留（部分補章的欄仍是合法順序）。
        let store = store_of(&[
            ("a", "schema: spec-driven\ncreated: 2026-09-01\n"),
            ("b", "schema: spec-driven\ncreated: 2026-09-02\n"),
            ("c", "schema: spec-driven\ncreated: 2026-09-03\n"),
        ]);
        *store.fail_meta_write.borrow_mut() = Some("b".to_string());
        assert!(set_rank(&store, &rank_writer(&store), "c", "b", Side::Before, false).is_err());
        assert_eq!(*store.meta_write_log.borrow(), ["a"]);
        assert!(rank_of(&store, "a").is_some());
        assert!(rank_of(&store, "b").is_none() && rank_of(&store, "c").is_none());
    }

    #[test]
    fn rank_refuses_when_already_ranked_without_force() {
        // Scenario 已有 rank 未帶 --force 被拒：訊息指名 --force，零寫入。
        let store = store_of(&[
            ("a", "schema: spec-driven\nboard_rank: n\n"),
            ("c", "schema: spec-driven\nboard_rank: t\n"),
        ]);
        let err = set_rank(&store, &rank_writer(&store), "c", "a", Side::After, false)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("already has a board rank") && err.contains("--force"),
            "{err}"
        );
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn rank_force_overwrites() {
        // Scenario 帶 --force 覆寫既有 rank：c（rank b）--after a（rank n，欄尾）→
        // c 的 rank 大於 n。
        let store = store_of(&[
            ("a", "schema: spec-driven\nboard_rank: n\n"),
            ("c", "schema: spec-driven\nboard_rank: b\n"),
        ]);
        set_rank(&store, &rank_writer(&store), "c", "a", Side::After, true).unwrap();
        assert!(rank_of(&store, "c").unwrap().as_str() > "n");
        assert_eq!(names(&compute(&store).unwrap()), ["a", "c"]);
    }

    #[test]
    fn rank_refuses_across_declared_dependency() {
        // Scenario 跨宣告依賴被拒：c 依賴 a，c --before a → must_follow 訊息；守門
        // 先於任何寫入，連本該先補章的 a 也不寫。
        let store = store_of(&[
            ("a", "schema: spec-driven\ncreated: 2026-09-01\n"),
            (
                "c",
                "schema: spec-driven\ncreated: 2026-09-02\ndepends_on: a\n",
            ),
        ]);
        let err = set_rank(&store, &rank_writer(&store), "c", "a", Side::Before, false)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("cannot move 'c' there") && err.contains("it depends on a"),
            "{err}"
        );
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn rank_refuses_different_stage() {
        // Scenario 不同階段被拒：a 進行中、c 提案中。
        let store = store_of(&[
            ("a", "schema: spec-driven\nstarted_at: 2026-09-06\n"),
            ("c", PROPOSED),
        ]);
        let err = set_rank(&store, &rank_writer(&store), "c", "a", Side::Before, false)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("proposed") && err.contains("in-progress"),
            "{err}"
        );
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn rank_refuses_self_anchor() {
        let store = store_of(&[("c", PROPOSED)]);
        let err = set_rank(&store, &rank_writer(&store), "c", "c", Side::Before, false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("itself"), "{err}");
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn rank_refuses_unknown_archived_or_corrupt_names() {
        const BAD: &str = ": : :\n\t bad yaml [unclosed\n";
        let store = store_of(&[("c", PROPOSED), ("bad", BAD)]);
        store
            .archived_metas
            .borrow_mut()
            .insert("2026-08-01-old".to_string(), PROPOSED.to_string());
        let refuse = |name: &str, anchor: &str, needle: &str| {
            let err = set_rank(&store, &rank_writer(&store), name, anchor, Side::Before, false)
                .unwrap_err()
                .to_string();
            assert!(err.contains(needle), "{name} --before {anchor}: {err}");
        };
        refuse("ghost", "c", "Change 'ghost' not found.");
        refuse("../evil", "c", "not found");
        refuse("c", "ghost", "no active change with that name");
        refuse("c", "old", "already archived");
        refuse("c", "bad", "invalid openspec/changes/bad/.openspec.yaml");
        refuse("bad", "c", "invalid openspec/changes/bad/.openspec.yaml");
        assert_eq!(*store.meta_writes.borrow(), 0);
        assert_eq!(store.meta("bad"), BAD);
    }

    #[test]
    fn a_hand_edited_rank_gets_the_column_restamped_before_the_move() {
        // 手改出生成格式以外的 rank：大寫 N 當前鄰居算中點會溢位，只有 a 的鍵當
        // 後鄰居時中點延長不完。兩者都當成缺 rank：整欄先補章再放新鍵。
        for (bad, side) in [("N", Side::After), ("a", Side::Before)] {
            let meta = format!("schema: spec-driven\nboard_rank: {bad}\n");
            let store = store_of(&[("m1", meta.as_str()), ("m3", PROPOSED)]);
            set_rank(&store, &rank_writer(&store), "m3", "m1", side, false).unwrap();
            let (m1, m3) = (rank_of(&store, "m1").unwrap(), rank_of(&store, "m3").unwrap());
            assert_ne!(m1, bad);
            match side {
                Side::After => assert!(m1 < m3, "{m1} < {m3}"),
                Side::Before => assert!(m3 < m1, "{m3} < {m1}"),
            }
        }
    }

    // --- move_rank（看板拖排與 change rank 共用的一步）---

    #[test]
    fn move_rank_puts_the_change_between_the_given_neighbors() {
        // 看板拖排：欄內 a（b）、c（n）有 rank 且與顯示序一致，x 拖到兩者之間 → 只寫 x。
        let store = store_of(&[
            ("a", "schema: spec-driven\nboard_rank: b\n"),
            ("c", "schema: spec-driven\nboard_rank: n\n"),
            ("x", "schema: spec-driven\nboard_rank: t\n"),
        ]);
        move_rank(&store, &rank_writer(&store), "x", Some("a"), Some("c")).unwrap();
        assert_eq!(*store.meta_write_log.borrow(), ["x"]);
        assert_eq!(names(&compute(&store).unwrap()), ["a", "x", "c"]);
    }

    #[test]
    fn move_rank_leaves_the_moved_change_out_of_the_stamp_decision() {
        // 只有被移動的卡缺 rank：欄內其餘已排好，不補章，只寫它一檔。
        let store = store_of(&[
            ("a", "schema: spec-driven\nboard_rank: f\n"),
            ("b", "schema: spec-driven\nboard_rank: t\n"),
            ("x", PROPOSED),
        ]);
        move_rank(&store, &rank_writer(&store), "x", Some("a"), Some("b")).unwrap();
        assert_eq!(*store.meta_write_log.borrow(), ["x"]);
        assert_eq!(names(&compute(&store).unwrap()), ["a", "x", "b"]);
    }

    #[test]
    fn move_rank_refusal_writes_nothing_even_when_the_column_needs_stamps() {
        // 依賴檢查在任何寫入之前：c 依賴 a、兩者都缺 rank，把 c 拖到 a 之前 →
        // 拒絕，連 a 的補章也不寫。
        let store = store_of(&[
            ("a", "schema: spec-driven\ncreated: 2026-09-01\n"),
            ("c", "schema: spec-driven\ncreated: 2026-09-02\ndepends_on: a\n"),
        ]);
        let err = move_rank(&store, &rank_writer(&store), "c", None, Some("a")).unwrap_err().to_string();
        assert!(
            err.contains("cannot move 'c' there") && err.contains("it depends on a"),
            "{err}"
        );
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn move_rank_refuses_an_unknown_or_corrupt_change() {
        const BAD: &str = ": : :\n\t bad yaml [unclosed\n";
        let store = store_of(&[("a", PROPOSED), ("bad", BAD)]);
        let err = move_rank(&store, &rank_writer(&store), "ghost", None, None).unwrap_err().to_string();
        assert!(err.contains("Change 'ghost' not found."), "{err}");
        let err = move_rank(&store, &rank_writer(&store), "bad", Some("a"), None).unwrap_err().to_string();
        assert!(err.contains("invalid openspec/changes/bad/.openspec.yaml"), "{err}");
        assert_eq!(*store.meta_writes.borrow(), 0);
        assert_eq!(store.meta("bad"), BAD);
    }
}
