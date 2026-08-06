//! Change-diff scope resolution (change-diff-scope spec).
//!
//! The Host owns the review station's Git and hunk semantics: the Apply
//! baseline sidecar (design D1), the baseCommit→worktree candidate resolver
//! (D2), ambiguity adjudication with hash-pinned selection (D3), and the
//! frozen review snapshots (D4). Local and remote CLI entry points both feed
//! host-local touched records, the Workspace, and selection parameters into
//! this single module — no second Git algorithm exists anywhere else.
//!
//! Everything here is host-local work data under `.speclink/review-scopes/`:
//! never a touched record, never change metadata, never a TeamStore document.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use speclink_core::workspace::Workspace;
use std::path::PathBuf;

/// Baseline sidecar format version.
pub const BASELINE_VERSION: u32 = 1;

/// How trustworthy the recorded baseline is for automatic hunk attribution.
/// Only `Initial` lets the resolver claim change hunks automatically; `Late`
/// and `Unavailable` are diagnostics that force an explicit fixed point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    Initial,
    Late,
    Unavailable,
}

/// The Apply baseline sidecar (`.speclink/review-scopes/<change>/baseline.json`).
/// Serde fields are camelCase by contract (change-diff-scope spec).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Baseline {
    pub version: u32,
    pub change: String,
    /// Full git HEAD SHA at capture; None when no git checkout exists.
    pub base_commit: Option<String>,
    /// Repo-root relative, forward-slashed, sorted, deduped dirty paths.
    pub dirty_files_at_start: Vec<String>,
    /// UTC RFC3339 capture time.
    pub captured_at: String,
    pub confidence: Confidence,
}

/// What `review prepare` did — the CLI maps `Late`/`Unavailable` to a stderr
/// warning (still exit 0) and everything else to silence.
#[derive(Debug)]
pub enum PrepareOutcome {
    /// Fresh (or replaced un-started) initial capture — silent.
    Captured(Baseline),
    /// Change already started and a baseline exists — first baseline kept.
    KeptExisting(Baseline),
    /// Started but no baseline on disk — recorded as diagnostic-only `late`.
    Late(Baseline),
    /// No usable git checkout — recorded as `unavailable`, baseCommit null.
    Unavailable(Baseline),
}

/// Sidecar directory for one change's review scopes.
pub fn scope_dir(ws: &Workspace, change: &str) -> PathBuf {
    ws.review_scopes_dir().join(change)
}

/// The baseline sidecar path for a change.
pub fn baseline_path(ws: &Workspace, change: &str) -> PathBuf {
    scope_dir(ws, change).join("baseline.json")
}

/// Load the baseline sidecar, if present and parseable.
pub fn load_baseline(ws: &Workspace, change: &str) -> Option<Baseline> {
    let text = std::fs::read_to_string(baseline_path(ws, change)).ok()?;
    serde_json::from_str(&text).ok()
}

/// Capture (or keep) the Apply baseline for a change (design D1).
///
/// `started` = the change already carries `started_*` metadata. Not started:
/// atomically replace any previous baseline with the current state
/// (confidence `initial`). Started with an existing baseline: keep the first
/// baseline untouched. Started without one: record `late`. No git checkout:
/// record `unavailable` with a null baseCommit. Writes go through a same-dir
/// temp file + rename; a write failure is an error (the caller must not
/// proceed to `in-progress add`).
pub fn prepare(ws: &Workspace, change: &str, started: bool) -> Result<PrepareOutcome> {
    if started {
        if let Some(existing) = load_baseline(ws, change) {
            return Ok(PrepareOutcome::KeptExisting(existing));
        }
    }
    // HEAD is guarded on the workspace root's own `.git` (the git_changed_files
    // precedent): a project nested inside an ancestor repo has no usable fixed
    // point of its own. A repo with no commits yet has no fixed point either.
    let head = if ws.root.join(".git").exists() {
        speclink_core::util::git(&ws.root, &["rev-parse", "HEAD"]).filter(|s| !s.is_empty())
    } else {
        None
    };
    let (base_commit, dirty, confidence) = match head {
        None => (None, Vec::new(), Confidence::Unavailable),
        Some(sha) => {
            let mut dirty = speclink_core::tasks::git_changed_files(ws);
            dirty.sort();
            dirty.dedup();
            let confidence = if started { Confidence::Late } else { Confidence::Initial };
            (Some(sha), dirty, confidence)
        }
    };
    let baseline = Baseline {
        version: BASELINE_VERSION,
        change: change.to_string(),
        base_commit,
        dirty_files_at_start: dirty,
        captured_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        confidence,
    };
    write_baseline(ws, &baseline)?;
    Ok(match confidence {
        Confidence::Initial => PrepareOutcome::Captured(baseline),
        Confidence::Late => PrepareOutcome::Late(baseline),
        Confidence::Unavailable => PrepareOutcome::Unavailable(baseline),
    })
}

// --- D2: the single Git-backed change-diff resolver ---

/// File delta kind (change-diff-scope spec: modified／added／deleted／
/// renamed／binary).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeltaKind {
    Modified,
    Added,
    Deleted,
    Renamed,
    Binary,
}

impl DeltaKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeltaKind::Modified => "modified",
            DeltaKind::Added => "added",
            DeltaKind::Deleted => "deleted",
            DeltaKind::Renamed => "renamed",
            DeltaKind::Binary => "binary",
        }
    }
}

/// One `@@` hunk of a text file delta. `id` is the sha256 hex over the path
/// identity, the four ranges, and the hunk body — the stable handle
/// `--include-hunk` selections use. Additions carry oldStart=0／oldLines=0,
/// deletions newStart=0／newLines=0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hunk {
    pub id: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
}

/// Where a validation round's file delta came from (design D4). Discovery has
/// no previous round to attribute against, so its deltas carry None.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Attribution {
    /// Named by the previous round's unresolved findings.
    Finding,
    /// A candidate file the remediation moved without a finding naming it.
    Adjacent,
    /// First became dirty after the previous round's capture.
    New,
}

/// One file's delta between the base commit tree and the current worktree.
/// `before_hash`／`after_hash` are `sha256:<hex>` over the raw bytes of each
/// side; the missing side of an addition／deletion is None. Binary deltas
/// carry hashes only (no hunks).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDelta {
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub kind: DeltaKind,
    pub before_hash: Option<String>,
    pub after_hash: Option<String>,
    pub hunks: Vec<Hunk>,
    /// Validation-only origin marker — absent from discovery payloads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribution: Option<Attribution>,
}

/// The resolved discovery candidate: the canonical patch from `base_commit`
/// to the current worktree over the touched paths, its identity hash, and
/// the parsed per-file deltas.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub base_commit: String,
    /// `sha256:<hex>` over the canonical patch bytes.
    pub candidate_hash: String,
    /// Canonical unified patch text.
    pub patch: String,
    pub files: Vec<FileDelta>,
    /// Per-file canonical segment parts (same order as `files`) — the raw
    /// material hash-pinned selections rebuild a narrowed patch from.
    parts: Vec<SegmentPart>,
}

/// One file's canonical segment split for patch reconstruction: everything
/// before the first `@@` line, then each hunk's text (header included) in
/// the same order as the parsed [`FileDelta::hunks`].
#[derive(Debug, Clone)]
struct SegmentPart {
    header: String,
    hunk_texts: Vec<String>,
}

/// Resolve the discovery candidate (design D2): a two-endpoint comparison of
/// the `base_commit` tree against the current worktree — staged AND unstaged
/// included, never `<base>...HEAD` — limited to `touched_paths`. Touched
/// untracked files become whole-file additions. `openspec/` artifacts and
/// `.speclink/` work data are never review targets.
pub fn resolve_candidate(
    ws: &Workspace,
    base_commit: &str,
    touched_paths: &[String],
) -> Result<Candidate> {
    let targets = normalize_targets(ws, touched_paths);
    if targets.is_empty() {
        let base = verify_base(ws, base_commit)?;
        return Ok(Candidate {
            base_commit: base,
            candidate_hash: sha256_prefixed(b""),
            patch: String::new(),
            files: Vec::new(),
            parts: Vec::new(),
        });
    }
    resolve_candidate_pathspec(ws, base_commit, &targets)
}

/// `\`→`/` normalized, sorted, deduped touched paths with `openspec/` and
/// `.speclink/` excluded — never review targets.
fn normalize_targets(ws: &Workspace, touched_paths: &[String]) -> Vec<String> {
    let spec_prefix = format!("{}/", ws.spec_dir_name);
    let mut targets: Vec<String> = touched_paths
        .iter()
        .map(|p| p.replace('\\', "/"))
        .filter(|p| !p.starts_with(&spec_prefix) && !p.starts_with(".speclink/"))
        .collect();
    targets.sort();
    targets.dedup();
    targets
}

/// The whole-worktree candidate for an explicit `--base` on an empty touched
/// set (design D3): the entire diff — spec artifacts and work data still
/// excluded — offered as a needsInput candidate only.
fn resolve_worktree_candidate(ws: &Workspace, base_commit: &str) -> Result<Candidate> {
    let pathspec = vec![
        ".".to_string(),
        format!(":(exclude){}", ws.spec_dir_name),
        ":(exclude).speclink".to_string(),
    ];
    resolve_candidate_pathspec(ws, base_commit, &pathspec)
}

fn verify_base(ws: &Workspace, base_commit: &str) -> Result<String> {
    Ok(String::from_utf8_lossy(&git_bytes(
        &ws.root,
        &["rev-parse", "--verify", &format!("{base_commit}^{{commit}}")],
    )
    .map_err(|e| anyhow::anyhow!("cannot resolve base commit '{base_commit}': {e}"))?)
    .trim()
    .to_string())
}

fn resolve_candidate_pathspec(
    ws: &Workspace,
    base_commit: &str,
    pathspec: &[String],
) -> Result<Candidate> {
    let base = verify_base(ws, base_commit)?;
    // (sort key, canonical segment text, parsed delta, parts)
    let mut entries: Vec<(String, String, FileDelta, SegmentPart)> = Vec::new();
    // The one Git comparison (design D2): base commit tree → current
    // worktree, staged AND unstaged, renames and deletes included.
    // Never `<base>...HEAD` — that compares commit graphs only.
    let mut args: Vec<&str> = vec![
        "-c",
        "core.quotepath=false",
        "diff",
        "--no-color",
        "--no-ext-diff",
        "--find-renames",
        "--full-index",
        &base,
        "--",
    ];
    args.extend(pathspec.iter().map(String::as_str));
    let diff = String::from_utf8_lossy(&git_bytes(&ws.root, &args)?).to_string();
    for seg in split_file_segments(&diff) {
        let delta = parse_segment(&seg, &ws.root, &base)?;
        let part = split_segment_part(&seg);
        let key = delta.new_path.clone().or_else(|| delta.old_path.clone()).unwrap_or_default();
        entries.push((key, seg, delta, part));
    }
    // Touched-but-untracked files never appear in `git diff <base>`:
    // they become whole-file additions with a synthesized canonical patch.
    let mut ls: Vec<&str> =
        vec!["-c", "core.quotepath=false", "ls-files", "--others", "--exclude-standard", "-z", "--"];
    ls.extend(pathspec.iter().map(String::as_str));
    let raw = git_bytes(&ws.root, &ls)?;
    for path in String::from_utf8_lossy(&raw).split('\0').filter(|p| !p.is_empty()) {
        let (seg, delta) = synthesize_addition(&ws.root, path)?;
        let part = split_segment_part(&seg);
        entries.push((path.to_string(), seg, delta, part));
    }
    // A gitignored touched file shows up in neither of the two commands above,
    // so without this it would drop out of the frozen patch silently. Only
    // literal pathspec entries qualify: the whole-worktree candidate's `.` and
    // `:(exclude)` magic name no file of their own.
    for path in ignored_scope_additions(ws, pathspec, &entries)? {
        let (seg, delta) = synthesize_addition(&ws.root, &path)?;
        let part = split_segment_part(&seg);
        entries.push((path, seg, delta, part));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let patch: String = entries.iter().map(|(_, seg, _, _)| seg.as_str()).collect();
    let (files, parts) = entries.into_iter().map(|(_, _, d, p)| (d, p)).unzip();
    Ok(Candidate {
        base_commit: base,
        candidate_hash: sha256_prefixed(patch.as_bytes()),
        patch,
        files,
        parts,
    })
}

/// Literal pathspec entries that exist on disk, carry no delta yet, and are
/// untracked — i.e. touched files git ignores. A tracked-but-unchanged path is
/// excluded here, so an unchanged file is never faked into a whole-file
/// addition.
fn ignored_scope_additions(
    ws: &Workspace,
    pathspec: &[String],
    entries: &[(String, String, FileDelta, SegmentPart)],
) -> Result<Vec<String>> {
    let literal: Vec<&String> =
        pathspec.iter().filter(|p| !p.starts_with(':') && p.as_str() != ".").collect();
    if literal.is_empty() {
        return Ok(Vec::new());
    }
    let covered: std::collections::HashSet<&str> =
        entries.iter().map(|(k, _, _, _)| k.as_str()).collect();
    let mut args: Vec<&str> = vec!["ls-files", "-z", "--"];
    args.extend(literal.iter().map(|p| p.as_str()));
    let raw = git_bytes(&ws.root, &args)?;
    let tracked: std::collections::HashSet<String> = String::from_utf8_lossy(&raw)
        .split('\0')
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect();
    Ok(literal
        .into_iter()
        .filter(|p| !covered.contains(p.as_str()) && !tracked.contains(*p))
        .filter(|p| ws.root.join(p).is_file())
        .cloned()
        .collect())
}

/// Split one canonical segment into its header (everything before the first
/// `@@`) and per-hunk texts, in file order.
fn split_segment_part(seg: &str) -> SegmentPart {
    let mut header = String::new();
    let mut hunk_texts: Vec<String> = Vec::new();
    for line in seg.split_inclusive('\n') {
        if line.starts_with("@@ ") {
            hunk_texts.push(line.to_string());
        } else if let Some(last) = hunk_texts.last_mut() {
            last.push_str(line);
        } else {
            header.push_str(line);
        }
    }
    SegmentPart { header, hunk_texts }
}

// --- D3: ambiguity adjudication and hash-pinned selection ---

/// Which review phase a resolved scope belongs to: first round (no ticket)
/// is discovery, follow-up rounds are remediation validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Discovery,
    Validation,
}

impl Phase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Phase::Discovery => "discovery",
            Phase::Validation => "validation",
        }
    }
}

/// Which quality station a scope resolution belongs to (design D8). The two
/// stations share the Apply baseline and this resolver but keep separate
/// remediation snapshot namespaces: either can stamp or discard on its own,
/// and a shared namespace would let one station's cleanup break the other's
/// follow-up validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StationNs {
    #[default]
    Review,
    Verify,
}

impl StationNs {
    /// Sidecar sub-directory under `.speclink/review-scopes/<change>/`.
    /// Review keeps the original `snapshots` name — renaming it would orphan
    /// every in-flight review ticket on upgrade.
    pub fn dir_name(&self) -> &'static str {
        match self {
            StationNs::Review => "snapshots",
            StationNs::Verify => "verify-snapshots",
        }
    }
}

/// Another active change's host-local touched claim (assembled by the CLI
/// adapter — local from fs records, remote from the same local checkout).
#[derive(Debug, Clone)]
pub struct ActiveClaim {
    pub change: String,
    pub paths: Vec<String>,
}

/// The review ticket facts scope resolution needs (assembled by the caller):
/// present ⇒ the next scope is a validation pass.
#[derive(Debug, Clone)]
pub struct TicketBinding {
    /// Every round's frozen patch identity, newest first — the chain a
    /// validation pass walks back to rebuild a candidate file's frozen state.
    /// Empty when the last round is legacy (carries no patch hash); the
    /// validation pass then fails closed instead of guessing.
    pub patch_hash_chain: Vec<String>,
    /// Paths of the last round's unresolved findings.
    pub finding_paths: Vec<String>,
}

/// One scope resolution request. Selection parameters (`--base`,
/// `--candidate-hash`, `--include-hunk`) arrive verbatim from the CLI.
#[derive(Debug, Clone, Default)]
pub struct ScopeRequest {
    pub change: String,
    /// This change's own touched-file union (host-local records).
    pub touched_paths: Vec<String>,
    /// Other active changes' touched claims (overlap guard).
    pub other_claims: Vec<ActiveClaim>,
    /// Present when the change already carries a review ticket.
    pub ticket: Option<TicketBinding>,
    /// Trusted fixed point overriding the Apply baseline.
    pub base_override: Option<String>,
    /// The candidate identity a hash-pinned selection is anchored to.
    pub candidate_hash: Option<String>,
    /// Selected hunk ids (each must exist; duplicates rejected).
    pub include_hunks: Vec<String>,
    /// Which station's snapshot namespace this resolution reads and writes.
    pub station: StationNs,
}

/// Why the scope is ambiguous. The CLI renders these on stderr together with
/// the three disposals (trusted `--base`, hash-pinned `--include-hunk`, an
/// isolated worktree).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AmbiguityReason {
    BaselineMissing,
    BaselineLate,
    BaselineUnavailable,
    /// The recorded or provided base commit does not resolve in this repo.
    BaseUnresolvable(String),
    /// Touched paths already dirty when Apply started.
    DirtyAtStart(Vec<String>),
    /// Another active change's touched record claims the same paths.
    ActiveOverlap { change: String, paths: Vec<String> },
    /// touchedFiles missing or empty — never auto-review the whole worktree.
    EmptyTouched,
}

/// The fail-closed half of scope resolution: ambiguous, nothing frozen.
#[derive(Debug, Clone)]
pub struct ScopeNeedsInput {
    pub change: String,
    pub phase: Phase,
    pub reasons: Vec<AmbiguityReason>,
    /// Present when a candidate could still be computed (selection anchor).
    pub candidate_hash: Option<String>,
    pub ambiguous_paths: Vec<String>,
    /// Candidate file deltas with selectable hunk ids (empty when no
    /// candidate could be computed).
    pub files: Vec<FileDelta>,
}

/// The resolved half: a frozen, hashable review patch.
#[derive(Debug, Clone)]
pub struct ResolvedScope {
    pub change: String,
    pub phase: Phase,
    pub base_commit: String,
    /// Identity of the full candidate the resolution ran against.
    pub candidate_hash: String,
    /// Identity of the frozen patch (equals candidate_hash unless a hunk
    /// selection narrowed it).
    pub patch_hash: String,
    pub paths: Vec<String>,
    pub files: Vec<FileDelta>,
    pub patch: String,
    /// Candidate files that moved but no round ever captured — annotated for
    /// the user, deliberately outside the review face (design D3). Always
    /// empty in discovery.
    pub out_of_scope_changed: Vec<String>,
}

/// Outcome of scope resolution: frozen or fail-closed.
#[derive(Debug, Clone)]
pub enum ScopeOutcome {
    Resolved(ResolvedScope),
    NeedsInput(ScopeNeedsInput),
}

/// Resolve a change's review scope (design D3).
///
/// Ambiguity — missing/late/unavailable baseline without a trusted
/// `base_override`, touched paths dirty at start, another active change's
/// overlapping claim, or empty touched — yields `NeedsInput` with zero
/// snapshot effects. A hash-pinned selection must carry the previous
/// candidate hash; the resolver recomputes the candidate and rejects drift,
/// unknown/duplicate hunk ids, and empty selections loudly (an `Err`, never
/// a silent fallback). A successful selection freezes only the chosen text
/// hunks while file before/after hashes stay anchored to the real files.
pub fn resolve_scope(ws: &Workspace, req: &ScopeRequest) -> Result<ScopeOutcome> {
    let phase = if req.ticket.is_some() { Phase::Validation } else { Phase::Discovery };
    if let Some(ticket) = &req.ticket {
        return resolve_validation_scope(ws, req, ticket);
    }
    let touched = normalize_targets(ws, &req.touched_paths);

    let mut reasons: Vec<AmbiguityReason> = Vec::new();
    let baseline = load_baseline(ws, &req.change);
    // A trusted fixed point: the user's --base assertion, else an initial
    // baseline. Late/unavailable/missing baselines are diagnostics only.
    let trusted_base = match (&req.base_override, &baseline) {
        (Some(b), _) => Some(b.clone()),
        (None, Some(b)) if b.confidence == Confidence::Initial => b.base_commit.clone(),
        (None, Some(b)) if b.confidence == Confidence::Late => {
            reasons.push(AmbiguityReason::BaselineLate);
            None
        }
        (None, Some(_)) => {
            reasons.push(AmbiguityReason::BaselineUnavailable);
            None
        }
        (None, None) => {
            reasons.push(AmbiguityReason::BaselineMissing);
            None
        }
    };

    // Dirty-at-start adjudication runs only against an initial baseline's
    // list — a late capture cannot testify about the start state.
    let mut ambiguous: Vec<String> = Vec::new();
    if let Some(b) = baseline.as_ref().filter(|b| b.confidence == Confidence::Initial) {
        let overlap: Vec<String> =
            touched.iter().filter(|p| b.dirty_files_at_start.contains(p)).cloned().collect();
        if !overlap.is_empty() {
            ambiguous.extend(overlap.iter().cloned());
            reasons.push(AmbiguityReason::DirtyAtStart(overlap));
        }
    }
    for claim in &req.other_claims {
        let claimed: Vec<String> = claim.paths.iter().map(|p| p.replace('\\', "/")).collect();
        let overlap: Vec<String> =
            touched.iter().filter(|p| claimed.contains(p)).cloned().collect();
        if !overlap.is_empty() {
            ambiguous.extend(overlap.iter().cloned());
            reasons.push(AmbiguityReason::ActiveOverlap {
                change: claim.change.clone(),
                paths: overlap,
            });
        }
    }
    if touched.is_empty() {
        reasons.push(AmbiguityReason::EmptyTouched);
    }

    // Compute the candidate wherever a trusted fixed point exists — it either
    // resolves cleanly or anchors a hash-pinned selection.
    let candidate = match &trusted_base {
        None => None,
        Some(base) => {
            let computed = if touched.is_empty() {
                resolve_worktree_candidate(ws, base)
            } else {
                resolve_candidate(ws, base, &touched)
            };
            match computed {
                Ok(c) => Some(c),
                Err(e) => {
                    reasons.push(AmbiguityReason::BaseUnresolvable(format!("{e:#}")));
                    None
                }
            }
        }
    };

    // An explicit hash-pinned selection adjudicates the ambiguity itself.
    if req.candidate_hash.is_some() || !req.include_hunks.is_empty() {
        let (Some(anchor), false) = (&req.candidate_hash, req.include_hunks.is_empty()) else {
            anyhow::bail!(
                "a hunk selection needs both --candidate-hash and at least one --include-hunk"
            );
        };
        let Some(candidate) = candidate else {
            anyhow::bail!(
                "cannot verify the selection: no trusted fixed point — pass a trusted --base"
            );
        };
        let resolved = apply_selection(&candidate, anchor, &req.include_hunks, req, phase)?;
        let texts = discovery_texts(ws, &resolved.base_commit, &resolved.files)?;
        write_scope_snapshot(ws, &resolved, req.station, texts, true)?;
        return Ok(ScopeOutcome::Resolved(resolved));
    }

    if reasons.is_empty() {
        let candidate = candidate.expect("no reasons implies a trusted base and a candidate");
        let resolved = ResolvedScope {
            change: req.change.clone(),
            phase,
            base_commit: candidate.base_commit,
            candidate_hash: candidate.candidate_hash.clone(),
            patch_hash: candidate.candidate_hash,
            paths: delta_paths(&candidate.files),
            files: candidate.files,
            patch: candidate.patch,
            out_of_scope_changed: Vec::new(),
        };
        let texts = discovery_texts(ws, &resolved.base_commit, &resolved.files)?;
        write_scope_snapshot(ws, &resolved, req.station, texts, true)?;
        return Ok(ScopeOutcome::Resolved(resolved));
    }
    ambiguous.sort();
    ambiguous.dedup();
    Ok(ScopeOutcome::NeedsInput(ScopeNeedsInput {
        change: req.change.clone(),
        phase,
        reasons,
        candidate_hash: candidate.as_ref().map(|c| c.candidate_hash.clone()),
        ambiguous_paths: ambiguous,
        files: candidate.map(|c| c.files).unwrap_or_default(),
    }))
}

/// Validation scope (design D1–D3): a follow-up round rebuilds the remediation
/// delta from the frozen snapshots — never from touched files or the current
/// worktree. Attribution is by *content movement*, not by which files the
/// findings named: every candidate file whose content moved since the last
/// capture enters the patch, rebuilt against the most recent round that
/// captured it. Files no round ever captured are annotated and let through;
/// paths that first became dirty after the capture join as `new`.
fn resolve_validation_scope(
    ws: &Workspace,
    req: &ScopeRequest,
    ticket: &TicketBinding,
) -> Result<ScopeOutcome> {
    if req.candidate_hash.is_some() || !req.include_hunks.is_empty() {
        anyhow::bail!("hunk selection applies to discovery scope only — a validation pass is bound to the frozen snapshot");
    }
    let Some(patch_hash) = ticket.patch_hash_chain.first() else {
        anyhow::bail!(
            "the ticket's last round carries no patch hash — the remediation delta cannot \
             be reconstructed precisely; discard the ticket and re-run discovery explicitly"
        );
    };
    let Some(snap) = load_snapshot(ws, &req.change, req.station, patch_hash) else {
        anyhow::bail!(
            "frozen snapshot for {patch_hash} is missing — cannot validate precisely and \
             will not fall back to discovery; keep the ticket and stop, or discard it and \
             re-run discovery explicitly"
        );
    };

    let finding_paths = normalize_targets(ws, &ticket.finding_paths);
    // (sort key, segment text, delta, texts entry)
    let mut entries: Vec<(String, String, FileDelta, Option<SnapshotText>)> = Vec::new();
    // Unchanged unresolved finding paths produce no patch segment, but their
    // frozen text must travel into the new snapshot — the next round still
    // validates them against this exact state.
    let mut carried_texts: Vec<SnapshotText> = Vec::new();
    for path in &finding_paths {
        match remediation_segment(ws, &snap, path)? {
            Some((seg, delta, text)) => {
                entries.push((path.clone(), seg, attributed(delta, Attribution::Finding), text))
            }
            None => {
                if let Some(t) = snap.texts.iter().find(|t| &t.path == path) {
                    carried_texts.push(SnapshotText {
                        path: path.clone(),
                        before_text: t.after_text.clone(),
                        after_text: t.after_text.clone(),
                    });
                }
            }
        }
    }
    // Every other captured candidate file whose content moved: the remediation
    // touched a neighbour the findings never named. The last round's snapshot
    // rebases most of them; the rest need an older round walked back to.
    let mut pending: Vec<String> = Vec::new();
    for entry in &snap.dirty_files_at_capture {
        if finding_paths.contains(&entry.path) {
            continue;
        }
        let now = std::fs::read(ws.root.join(&entry.path)).ok().map(|b| sha256_prefixed(&b));
        if now.as_deref() == Some(entry.hash.as_str()) {
            continue; // content did not move — nothing to review
        }
        if snapshot_preserves(&snap, &entry.path) {
            push_adjacent(ws, &snap, &entry.path, Some(&entry.hash), &mut entries)?;
        } else {
            pending.push(entry.path.clone());
        }
    }
    // Walk the ticket's rounds newest→oldest, loading each snapshot at most
    // once, until every pending path found the round that last captured it.
    for older_hash in ticket.patch_hash_chain.iter().skip(1) {
        if pending.is_empty() {
            break;
        }
        let Some(older) = load_snapshot(ws, &req.change, req.station, older_hash) else {
            anyhow::bail!(
                "frozen snapshot for {older_hash} is missing — the ticket's round chain \
                 cannot be walked back to rebuild {}; keep the ticket and stop, or discard \
                 it and re-run discovery explicitly",
                pending.join(", ")
            );
        };
        let mut still_pending = Vec::new();
        for path in pending {
            if snapshot_preserves(&older, &path) {
                push_adjacent(ws, &older, &path, None, &mut entries)?;
            } else {
                still_pending.push(path);
            }
        }
        pending = still_pending;
    }
    // What survives the walk was never captured by any round — the user
    // excluded it at discovery, and a re-review does not relitigate that.
    let out_of_scope_changed = pending;
    // Paths that first became dirty after the capture join the validation
    // patch via the same D2 resolver, anchored at the frozen base commit.
    // A dirty-deleted path never enters the capture ledger (no bytes to
    // hash), so `git status` would re-offer it every round — but when the
    // last snapshot already froze its deletion and it is still absent, its
    // content has not moved and it must not re-enter the patch. The frozen
    // "absent" state travels forward as a carried text (after None) so the
    // suppression survives its own round's snapshot.
    let captured: Vec<&str> = snap.dirty_files_at_capture.iter().map(|e| e.path.as_str()).collect();
    let deletion_frozen = |p: &str| {
        (snap.files.iter().any(|f| f.old_path.as_deref() == Some(p) && f.new_path.is_none())
            || snap.texts.iter().any(|t| t.path == p && t.after_text.is_none()))
            && !ws.root.join(p).exists()
    };
    let mut new_dirty: Vec<String> = Vec::new();
    for p in speclink_core::tasks::git_changed_files(ws) {
        if captured.contains(&p.as_str()) || finding_paths.contains(&p) {
            continue;
        }
        if deletion_frozen(&p) {
            carried_texts.push(SnapshotText { path: p, before_text: None, after_text: None });
        } else {
            new_dirty.push(p);
        }
    }
    if !new_dirty.is_empty() {
        let candidate = resolve_candidate(ws, &snap.base_commit, &new_dirty)?;
        let texts = discovery_texts(ws, &candidate.base_commit, &candidate.files)?;
        for (delta, part) in candidate.files.iter().zip(&candidate.parts) {
            let key =
                delta.new_path.clone().or_else(|| delta.old_path.clone()).unwrap_or_default();
            let seg: String = std::iter::once(part.header.as_str())
                .chain(part.hunk_texts.iter().map(String::as_str))
                .collect();
            let text = texts.iter().find(|t| Some(&t.path) == delta.new_path.as_ref()).cloned();
            entries.push((key, seg, attributed(delta.clone(), Attribution::New), text));
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let patch: String = entries.iter().map(|(_, seg, _, _)| seg.as_str()).collect();
    let patch_hash = sha256_prefixed(patch.as_bytes());
    let mut texts: Vec<SnapshotText> =
        entries.iter().filter_map(|(_, _, _, t)| t.clone()).collect();
    texts.extend(carried_texts);
    texts.sort_by(|a, b| a.path.cmp(&b.path));
    let files: Vec<FileDelta> = entries.into_iter().map(|(_, _, d, _)| d).collect();
    let resolved = ResolvedScope {
        change: req.change.clone(),
        phase: Phase::Validation,
        base_commit: snap.base_commit.clone(),
        candidate_hash: patch_hash.clone(),
        patch_hash,
        paths: delta_paths(&files),
        files,
        patch,
        out_of_scope_changed,
    };
    // The discovery snapshot stays (the ticket references it); only stamp／
    // discard clear snapshots.
    write_scope_snapshot(ws, &resolved, req.station, texts, false)?;
    Ok(ScopeOutcome::Resolved(resolved))
}

fn attributed(delta: FileDelta, attribution: Attribution) -> FileDelta {
    FileDelta { attribution: Some(attribution), ..delta }
}

/// Whether a snapshot can rebase `path` precisely — it carries the path's
/// frozen text, or a file delta whose AFTER side is the path (its after_hash
/// is the rebuild anchor). An old_path-only match (a rename source) has no
/// frozen `before` for the path and must not claim otherwise.
fn snapshot_preserves(snap: &Snapshot, path: &str) -> bool {
    snap.texts.iter().any(|t| t.path == path)
        || snap.files.iter().any(|f| f.new_path.as_deref() == Some(path))
}

/// Append one adjacent segment rebuilt from `origin` — the most recent round
/// that captured the path. A None segment means the content came back to that
/// round's frozen state; there is nothing to review.
///
/// `expect_before` pins the rebuilt `before` against the ledger hash the SAME
/// round recorded for the path — they were read from one file at one freeze,
/// so a disagreement means a parallel writer slipped between the two reads and
/// the snapshot now contradicts itself. Walked-back rounds pass None: an older
/// round's frozen state is supposed to differ from the last round's ledger.
fn push_adjacent(
    ws: &Workspace,
    origin: &Snapshot,
    path: &str,
    expect_before: Option<&str>,
    entries: &mut Vec<(String, String, FileDelta, Option<SnapshotText>)>,
) -> Result<()> {
    if let Some((seg, delta, text)) = remediation_segment(ws, origin, path)? {
        if let Some(expected) = expect_before {
            if delta.before_hash.as_deref() != Some(expected) {
                anyhow::bail!(
                    "the frozen snapshot contradicts its own dirty-file ledger for '{path}' \
                     (rebuilt {}, recorded {expected}) — the remediation delta cannot be \
                     rebuilt precisely; discard the ticket and re-run discovery explicitly",
                    delta.before_hash.as_deref().unwrap_or("nothing")
                );
            }
        }
        entries.push((path.to_string(), seg, attributed(delta, Attribution::Adjacent), text));
    }
    Ok(())
}

/// Build one scope path's remediation segment: frozen afterText → current
/// content. Returns None when the path is unchanged since the capture.
fn remediation_segment(
    ws: &Workspace,
    snap: &Snapshot,
    path: &str,
) -> Result<Option<(String, FileDelta, Option<SnapshotText>)>> {
    let frozen_text = snap.texts.iter().find(|t| t.path == path).and_then(|t| t.after_text.clone());
    let frozen_delta = snap
        .files
        .iter()
        .find(|f| f.new_path.as_deref() == Some(path) || f.old_path.as_deref() == Some(path));
    let current = std::fs::read(ws.root.join(path)).ok();
    let Some(frozen_text) = frozen_text else {
        // No frozen text: the path was binary (or absent) in the frozen scope.
        let Some(frozen) = frozen_delta else {
            anyhow::bail!(
                "scope path '{path}' is not part of the frozen snapshot — the ticket and \
                 the snapshot disagree; discard the ticket and re-run discovery explicitly"
            );
        };
        let before_hash = frozen.after_hash.clone();
        let after_hash = current.as_deref().map(sha256_prefixed);
        if before_hash == after_hash {
            return Ok(None);
        }
        let seg = format!(
            "diff --git a/{path} b/{path}\nBinary files a/{path} and b/{path} differ\n"
        );
        let delta = FileDelta {
            old_path: Some(path.to_string()),
            new_path: current.is_some().then(|| path.to_string()),
            kind: DeltaKind::Binary,
            before_hash,
            after_hash,
            hunks: Vec::new(),
            attribution: None,
        };
        return Ok(Some((seg, delta, None)));
    };
    let before_hash = Some(sha256_prefixed(frozen_text.as_bytes()));
    match current {
        None => {
            // Remediation deleted the finding file: synthesize the deletion
            // from the frozen text (no git involved — `--no-index` against a
            // null device is not portable).
            let (seg, delta) = synthesize_deletion_from_text(path, &frozen_text);
            let text = SnapshotText {
                path: path.to_string(),
                before_text: Some(frozen_text),
                after_text: None,
            };
            Ok(Some((seg, delta, Some(text))))
        }
        Some(bytes) => {
            let after_hash = Some(sha256_prefixed(&bytes));
            if after_hash == before_hash {
                return Ok(None);
            }
            let hunk_src = no_index_hunks(ws, path, frozen_text.as_bytes())?;
            let mut seg = format!("diff --git a/{path} b/{path}\n--- a/{path}\n+++ b/{path}\n");
            seg.push_str(&hunk_src);
            let some_path = Some(path.to_string());
            let hunks = parse_hunks(&seg, &some_path, &some_path);
            let delta = FileDelta {
                old_path: some_path.clone(),
                new_path: some_path,
                kind: DeltaKind::Modified,
                before_hash,
                after_hash,
                hunks,
                attribution: None,
            };
            let text = SnapshotText {
                path: path.to_string(),
                before_text: Some(frozen_text),
                after_text: String::from_utf8(bytes).ok(),
            };
            Ok(Some((seg, delta, Some(text))))
        }
    }
}

/// Hunk lines of `frozen bytes → worktree path` via `git diff --no-index`
/// (exit 1 = differences — success, the classic sharp edge). Headers are
/// discarded; the caller writes canonical ones.
fn no_index_hunks(ws: &Workspace, path: &str, frozen: &[u8]) -> Result<String> {
    let dir = ws.review_scopes_dir().join(format!(".no-index-{}", std::process::id()));
    std::fs::create_dir_all(&dir).context("cannot create no-index temp dir")?;
    let tmp = dir.join("before.tmp");
    std::fs::write(&tmp, frozen).context("cannot write no-index temp file")?;
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(&ws.root)
        .args(["-c", "core.quotepath=false", "diff", "--no-color", "--no-ext-diff", "--no-index", "--"])
        .arg(&tmp)
        .arg(path)
        .output()
        .context("cannot run git diff --no-index")?;
    let _ = std::fs::remove_dir_all(&dir);
    // --no-index exits 1 when the files differ; only >1 is a real failure.
    if out.status.code().map_or(true, |c| c > 1) {
        anyhow::bail!(
            "git diff --no-index failed for '{path}': {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(text
        .split_inclusive('\n')
        .skip_while(|l| !l.starts_with("@@ "))
        .collect())
}

/// Synthesize a whole-file deletion segment from frozen text (the mirror of
/// [`synthesize_addition`]).
fn synthesize_deletion_from_text(path: &str, text: &str) -> (String, FileDelta) {
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let count = lines.len() as u32;
    let mut seg = format!("diff --git a/{path} b/{path}\ndeleted file mode 100644\n");
    let mut hunks = Vec::new();
    if count > 0 {
        seg.push_str(&format!("--- a/{path}\n+++ /dev/null\n"));
        let header = if count == 1 {
            "@@ -1 +0,0 @@\n".to_string()
        } else {
            format!("@@ -1,{count} +0,0 @@\n")
        };
        seg.push_str(&header);
        let mut body = String::new();
        for line in &lines {
            body.push('-');
            body.push_str(line.strip_suffix('\n').unwrap_or(line));
            body.push('\n');
            if !line.ends_with('\n') {
                body.push_str("\\ No newline at end of file\n");
            }
        }
        seg.push_str(&body);
        hunks.push(build_hunk(path, "", (1, count, 0, 0), &body));
    }
    let delta = FileDelta {
        old_path: Some(path.to_string()),
        new_path: None,
        kind: DeltaKind::Deleted,
        before_hash: Some(sha256_prefixed(text.as_bytes())),
        after_hash: None,
        hunks,
        attribution: None,
    };
    (seg, delta)
}

/// Frozen texts for a discovery-shaped scope: before from the base blob,
/// after from the worktree; only UTF-8 sides are preserved.
fn discovery_texts(
    ws: &Workspace,
    base: &str,
    files: &[FileDelta],
) -> Result<Vec<SnapshotText>> {
    let mut texts = Vec::new();
    for f in files {
        if f.kind == DeltaKind::Binary {
            continue;
        }
        let before_text = match &f.old_path {
            Some(p) => String::from_utf8(git_bytes(&ws.root, &["show", &format!("{base}:{p}")])?).ok(),
            None => None,
        };
        let after_text = match &f.new_path {
            Some(p) => std::fs::read(ws.root.join(p)).ok().and_then(|b| String::from_utf8(b).ok()),
            None => None,
        };
        if before_text.is_none() && after_text.is_none() {
            continue;
        }
        let path = f.new_path.clone().or_else(|| f.old_path.clone()).unwrap_or_default();
        texts.push(SnapshotText { path, before_text, after_text });
    }
    Ok(texts)
}

/// Atomically freeze a resolved scope's snapshot. `clear_first` removes
/// orphans of a ticketless (discovery) scope before the new write.
fn write_scope_snapshot(
    ws: &Workspace,
    resolved: &ResolvedScope,
    ns: StationNs,
    texts: Vec<SnapshotText>,
    clear_first: bool,
) -> Result<()> {
    if clear_first {
        clear_snapshots(ws, &resolved.change, ns)
            .context("cannot clear orphan snapshots")?;
    }
    let mut dirty_files_at_capture = Vec::new();
    for path in speclink_core::tasks::git_changed_files(ws) {
        // A dirty-deleted file has no bytes to hash; it re-enters review as a
        // fresh path if it ever reappears.
        if let Ok(bytes) = std::fs::read(ws.root.join(&path)) {
            dirty_files_at_capture.push(PathHash { path, hash: sha256_prefixed(&bytes) });
        }
    }
    dirty_files_at_capture.sort_by(|a, b| a.path.cmp(&b.path));
    let snapshot = Snapshot {
        version: SNAPSHOT_VERSION,
        change: resolved.change.clone(),
        phase: resolved.phase,
        candidate_hash: resolved.candidate_hash.clone(),
        patch_hash: resolved.patch_hash.clone(),
        base_commit: resolved.base_commit.clone(),
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        dirty_files_at_capture,
        patch: resolved.patch.clone(),
        files: resolved.files.clone(),
        texts,
    };
    let dir = snapshots_dir(ws, &resolved.change, ns);
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("cannot create snapshot dir {}", dir.display()))?;
    let json = serde_json::to_string_pretty(&snapshot)?;
    let tmp = sidecar_tmp_path(&dir, "snapshot.json");
    std::fs::write(&tmp, &json).context("cannot write snapshot temp file")?;
    std::fs::rename(&tmp, snapshot_path(ws, &resolved.change, ns, &resolved.patch_hash))
        .context("cannot finalize the snapshot")?;
    Ok(())
}

/// Sorted unique repo paths of a delta set (new side, else old side).
fn delta_paths(files: &[FileDelta]) -> Vec<String> {
    let mut paths: Vec<String> = files
        .iter()
        .filter_map(|f| f.new_path.clone().or_else(|| f.old_path.clone()))
        .collect();
    paths.sort();
    paths.dedup();
    paths
}

/// Apply a hash-pinned hunk selection (design D3): recompute-and-compare the
/// candidate identity, validate every id, then freeze only the chosen text
/// hunks. File before/after hashes stay anchored to the real files.
fn apply_selection(
    candidate: &Candidate,
    anchor: &str,
    include_hunks: &[String],
    req: &ScopeRequest,
    phase: Phase,
) -> Result<ResolvedScope> {
    if candidate.candidate_hash != anchor {
        anyhow::bail!(
            "the candidate has drifted since the selection was made (current {}, selected \
             against {anchor}) — re-run review scope and select again",
            candidate.candidate_hash
        );
    }
    let mut seen = std::collections::HashSet::new();
    for id in include_hunks {
        if !seen.insert(id.as_str()) {
            anyhow::bail!("duplicate --include-hunk id: {id}");
        }
        if !candidate.files.iter().any(|f| f.hunks.iter().any(|h| &h.id == id)) {
            anyhow::bail!(
                "unknown --include-hunk id: {id} — binary deltas have no selectable hunks; \
                 list the current candidate with review scope --json"
            );
        }
    }
    let mut patch = String::new();
    let mut files = Vec::new();
    for (delta, part) in candidate.files.iter().zip(&candidate.parts) {
        let chosen: Vec<usize> = delta
            .hunks
            .iter()
            .enumerate()
            .filter(|(_, h)| seen.contains(h.id.as_str()))
            .map(|(i, _)| i)
            .collect();
        if chosen.is_empty() {
            continue;
        }
        patch.push_str(&part.header);
        let mut narrowed = delta.clone();
        narrowed.hunks = chosen
            .iter()
            .map(|&i| {
                patch.push_str(&part.hunk_texts[i]);
                delta.hunks[i].clone()
            })
            .collect();
        files.push(narrowed);
    }
    Ok(ResolvedScope {
        change: req.change.clone(),
        phase,
        base_commit: candidate.base_commit.clone(),
        candidate_hash: candidate.candidate_hash.clone(),
        patch_hash: sha256_prefixed(patch.as_bytes()),
        paths: delta_paths(&files),
        files,
        patch,
        out_of_scope_changed: Vec::new(),
    })
}

// --- D4: frozen review snapshots ---

/// Snapshot sidecar format version.
pub const SNAPSHOT_VERSION: u32 = 1;

/// A (repo path, content sha256) pair — the dirty-at-capture ledger entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathHash {
    pub path: String,
    pub hash: String,
}

/// Frozen text contents of one scope file. Only UTF-8 sides are preserved;
/// binary／non-UTF-8 files keep hashes only (in [`FileDelta`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotText {
    pub path: String,
    pub before_text: Option<String>,
    pub after_text: Option<String>,
}

/// The frozen review snapshot (design D4): everything a follow-up validation
/// needs to rebuild the remediation delta precisely. Host-local work data —
/// never a touched record, never uploaded.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub version: u32,
    pub change: String,
    pub phase: Phase,
    pub candidate_hash: String,
    pub patch_hash: String,
    pub base_commit: String,
    /// UTC RFC3339 capture time.
    pub created_at: String,
    /// Every dirty worktree path at capture with its content hash.
    pub dirty_files_at_capture: Vec<PathHash>,
    /// The canonical frozen patch.
    pub patch: String,
    pub files: Vec<FileDelta>,
    pub texts: Vec<SnapshotText>,
}

/// Snapshot directory for one change and one quality station.
pub fn snapshots_dir(ws: &Workspace, change: &str, ns: StationNs) -> PathBuf {
    scope_dir(ws, change).join(ns.dir_name())
}

/// Digest-only, Windows-safe snapshot filename: the hex digest without the
/// `sha256:` prefix (a `:` is illegal in NTFS filenames).
pub fn snapshot_path(ws: &Workspace, change: &str, ns: StationNs, patch_hash: &str) -> PathBuf {
    let digest = patch_hash.strip_prefix("sha256:").unwrap_or(patch_hash);
    snapshots_dir(ws, change, ns).join(format!("{digest}.json"))
}

/// Load the snapshot frozen for `patch_hash`, if present and parseable.
pub fn load_snapshot(
    ws: &Workspace,
    change: &str,
    ns: StationNs,
    patch_hash: &str,
) -> Option<Snapshot> {
    let text = std::fs::read_to_string(snapshot_path(ws, change, ns, patch_hash)).ok()?;
    serde_json::from_str(&text).ok()
}

/// Remove one station's snapshots of a change, keeping the Apply baseline and
/// the other station's snapshots — the stamp／discard cleanup (design D4／D8).
/// The stations resolve and stamp independently, so either one clearing the
/// other's snapshots would break its follow-up validation.
pub fn clear_snapshots(ws: &Workspace, change: &str, ns: StationNs) -> std::io::Result<()> {
    match std::fs::remove_dir_all(snapshots_dir(ws, change, ns)) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

/// Run git in `root`, returning raw stdout bytes; a non-zero exit is an error
/// carrying git's stderr. Bytes rather than String: a lossy conversion would
/// corrupt the content hashes taken from `git show` output.
///
/// This is the contract, not a census of call sites — anything needing a
/// different one (a tolerated non-zero exit, or core's HEAD and dirty-listing
/// helpers) necessarily invokes git for itself.
fn git_bytes(root: &std::path::Path, args: &[&str]) -> Result<Vec<u8>> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .context("cannot run git")?;
    if !out.status.success() {
        anyhow::bail!(
            "git {} failed: {}",
            args.first().copied().unwrap_or(""),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(out.stdout)
}

fn sha256_prefixed(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("sha256:{:x}", Sha256::digest(bytes))
}

/// Split a unified diff into per-file segments (each starting `diff --git `).
fn split_file_segments(patch: &str) -> Vec<String> {
    let mut segs = Vec::new();
    let mut cur = String::new();
    for line in patch.split_inclusive('\n') {
        if line.starts_with("diff --git ") && !cur.is_empty() {
            segs.push(std::mem::take(&mut cur));
        }
        cur.push_str(line);
    }
    if !cur.trim().is_empty() {
        segs.push(cur);
    }
    segs
}

/// Parse one file segment into a [`FileDelta`], filling before／after hashes
/// from the base blob and the worktree bytes.
fn parse_segment(seg: &str, root: &std::path::Path, base: &str) -> Result<FileDelta> {
    let mut old_path: Option<String> = None;
    let mut new_path: Option<String> = None;
    let (mut renamed, mut added, mut deleted, mut binary) = (false, false, false, false);
    // Header lines only: past the first `@@` every line is hunk body, where a
    // deleted `-- x` reads as `--- x` and an added `++ x` as `+++ x`. Scanning
    // on would let file content overwrite the parsed paths.
    for line in seg.lines().take_while(|l| !l.starts_with("@@ ")) {
        if let Some(rest) = line.strip_prefix("rename from ") {
            old_path = Some(rest.to_string());
            renamed = true;
        } else if let Some(rest) = line.strip_prefix("rename to ") {
            new_path = Some(rest.to_string());
            renamed = true;
        } else if let Some(rest) = line.strip_prefix("--- ") {
            if rest != "/dev/null" {
                old_path = Some(strip_diff_prefix(rest, "a/"));
            }
        } else if let Some(rest) = line.strip_prefix("+++ ") {
            if rest != "/dev/null" {
                new_path = Some(strip_diff_prefix(rest, "b/"));
            }
        } else if line.starts_with("new file mode") {
            added = true;
        } else if line.starts_with("deleted file mode") {
            deleted = true;
        } else if line.starts_with("Binary files ") || line.starts_with("GIT binary patch") {
            binary = true;
        }
    }
    // Binary and mode-only segments carry no ---/+++ lines: fall back to the
    // `diff --git a/X b/X` header (halves are equal outside a rename).
    if old_path.is_none() && new_path.is_none() {
        if let Some(p) = equal_halves_path(seg) {
            if !added {
                old_path = Some(p.clone());
            }
            if !deleted {
                new_path = Some(p);
            }
        }
    }
    if added {
        old_path = None;
    }
    if deleted {
        new_path = None;
    }
    let kind = if binary {
        DeltaKind::Binary
    } else if added {
        DeltaKind::Added
    } else if deleted {
        DeltaKind::Deleted
    } else if renamed {
        DeltaKind::Renamed
    } else {
        DeltaKind::Modified
    };
    let hunks = if binary { Vec::new() } else { parse_hunks(seg, &old_path, &new_path) };
    let before_hash = match &old_path {
        Some(p) => Some(sha256_prefixed(&git_bytes(root, &["show", &format!("{base}:{p}")])?)),
        None => None,
    };
    let after_hash = match &new_path {
        Some(p) => {
            let bytes = std::fs::read(root.join(p))
                .with_context(|| format!("cannot read worktree file {p}"))?;
            Some(sha256_prefixed(&bytes))
        }
        None => None,
    };
    Ok(FileDelta { old_path, new_path, kind, before_hash, after_hash, hunks, attribution: None })
}

/// Strip the diff prefix (`a/`／`b/`) and a possible trailing tab git adds
/// for paths carrying spaces.
fn strip_diff_prefix(rest: &str, prefix: &str) -> String {
    let rest = rest.strip_suffix('\t').unwrap_or(rest);
    rest.strip_prefix(prefix).unwrap_or(rest).to_string()
}

/// Recover the path from `diff --git a/P b/P` when both halves are equal
/// (every non-rename segment). Returns None when the halves cannot match.
fn equal_halves_path(seg: &str) -> Option<String> {
    let header = seg.lines().next()?.strip_prefix("diff --git ")?;
    // "a/P b/P" → len = 2 + P + 1 + 2 + P
    let len = header.len().checked_sub(5)?;
    if len % 2 != 0 {
        return None;
    }
    let p = len / 2;
    let (left, right) = (header.get(2..2 + p)?, header.get(header.len() - p..)?);
    (left == right && header.get(..2) == Some("a/")).then(|| left.to_string())
}

/// The stable selection handle for one hunk: sha256 hex over the path
/// identity, the four ranges, and the hunk body (design D2).
fn hunk_id(old_path: &str, new_path: &str, ranges: (u32, u32, u32, u32), body: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(old_path.as_bytes());
    hasher.update([0]);
    hasher.update(new_path.as_bytes());
    hasher.update([0]);
    hasher.update(format!("{},{},{},{}", ranges.0, ranges.1, ranges.2, ranges.3).as_bytes());
    hasher.update([0]);
    hasher.update(body.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn build_hunk(old_path: &str, new_path: &str, ranges: (u32, u32, u32, u32), body: &str) -> Hunk {
    Hunk {
        id: hunk_id(old_path, new_path, ranges, body),
        old_start: ranges.0,
        old_lines: ranges.1,
        new_start: ranges.2,
        new_lines: ranges.3,
    }
}

/// Parse the `@@` hunks of a text segment.
fn parse_hunks(seg: &str, old_path: &Option<String>, new_path: &Option<String>) -> Vec<Hunk> {
    let (old, new) = (old_path.as_deref().unwrap_or(""), new_path.as_deref().unwrap_or(""));
    let mut hunks = Vec::new();
    let mut current: Option<((u32, u32, u32, u32), String)> = None;
    for line in seg.lines() {
        if let Some(ranges) = parse_hunk_header(line) {
            if let Some((r, body)) = current.take() {
                hunks.push(build_hunk(old, new, r, &body));
            }
            current = Some((ranges, String::new()));
        } else if let Some((_, body)) = current.as_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    if let Some((r, body)) = current.take() {
        hunks.push(build_hunk(old, new, r, &body));
    }
    hunks
}

/// Parse `@@ -a[,b] +c[,d] @@ ...` into (a, b, c, d); a missing count is 1.
fn parse_hunk_header(line: &str) -> Option<(u32, u32, u32, u32)> {
    let inner = line.strip_prefix("@@ ")?.split(" @@").next()?;
    let (old, new) = inner.split_once(' ')?;
    let parse = |s: &str, sign: char| -> Option<(u32, u32)> {
        let s = s.strip_prefix(sign)?;
        match s.split_once(',') {
            Some((a, b)) => Some((a.parse().ok()?, b.parse().ok()?)),
            None => Some((s.parse().ok()?, 1)),
        }
    };
    let (os, ol) = parse(old, '-')?;
    let (ns, nl) = parse(new, '+')?;
    Some((os, ol, ns, nl))
}

/// Synthesize the canonical whole-file-addition segment for a touched
/// untracked file (design D2) — `git diff <base>` cannot see it.
fn synthesize_addition(root: &std::path::Path, path: &str) -> Result<(String, FileDelta)> {
    let bytes = std::fs::read(root.join(path))
        .with_context(|| format!("cannot read untracked touched file {path}"))?;
    let blob = String::from_utf8_lossy(&git_bytes(root, &["hash-object", "--", path])?)
        .trim()
        .to_string();
    let mode = file_mode(root, path);
    let mut seg = format!(
        "diff --git a/{path} b/{path}\nnew file mode {mode}\nindex {}..{blob}\n",
        "0".repeat(blob.len())
    );
    let binary = bytes.iter().take(8000).any(|b| *b == 0);
    let after_hash = Some(sha256_prefixed(&bytes));
    if binary {
        seg.push_str(&format!("Binary files /dev/null and b/{path} differ\n"));
        let delta = FileDelta {
            old_path: None,
            new_path: Some(path.to_string()),
            kind: DeltaKind::Binary,
            before_hash: None,
            after_hash,
            hunks: Vec::new(),
            attribution: None,
        };
        return Ok((seg, delta));
    }
    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let count = lines.len() as u32;
    let mut hunks = Vec::new();
    if count > 0 {
        seg.push_str(&format!("--- /dev/null\n+++ b/{path}\n"));
        let header = if count == 1 {
            "@@ -0,0 +1 @@\n".to_string()
        } else {
            format!("@@ -0,0 +1,{count} @@\n")
        };
        seg.push_str(&header);
        let mut body = String::new();
        for line in &lines {
            body.push('+');
            body.push_str(line.strip_suffix('\n').unwrap_or(line));
            body.push('\n');
            if !line.ends_with('\n') {
                body.push_str("\\ No newline at end of file\n");
            }
        }
        seg.push_str(&body);
        hunks.push(build_hunk("", path, (0, 0, 1, count), &body));
    }
    let delta = FileDelta {
        old_path: None,
        new_path: Some(path.to_string()),
        kind: DeltaKind::Added,
        before_hash: None,
        after_hash,
        hunks,
        attribution: None,
    };
    Ok((seg, delta))
}

/// 100755 for executable files on unix, 100644 otherwise.
fn file_mode(root: &std::path::Path, path: &str) -> &'static str {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(root.join(path)) {
            if meta.permissions().mode() & 0o111 != 0 {
                return "100755";
            }
        }
    }
    let _ = (root, path);
    "100644"
}

/// A temp path in the target's own directory, unique per writer: a fixed name
/// would let two concurrent writers (local and remote scope on the same
/// change) share one temp file and rename half-written bytes into place.
fn sidecar_tmp_path(dir: &std::path::Path, stem: &str) -> PathBuf {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    dir.join(format!("{stem}.{}.{n}.tmp", std::process::id()))
}

/// Atomic sidecar write: same-dir temp file then rename, so a crash never
/// leaves a half-written baseline behind.
fn write_baseline(ws: &Workspace, baseline: &Baseline) -> Result<()> {
    let dir = scope_dir(ws, &baseline.change);
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("cannot create review-scope dir {}", dir.display()))?;
    let json = serde_json::to_string_pretty(baseline)?;
    let tmp = sidecar_tmp_path(&dir, "baseline.json");
    std::fs::write(&tmp, &json)
        .with_context(|| format!("cannot write baseline temp file {}", tmp.display()))?;
    std::fs::rename(&tmp, dir.join("baseline.json"))
        .with_context(|| format!("cannot finalize baseline in {}", dir.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::process::Command;

    /// Throwaway repo fixture: git-initialized with one commit unless built
    /// via `no_git`; removed on drop.
    struct TempRepo {
        root: PathBuf,
    }

    impl TempRepo {
        fn no_git(tag: &str) -> TempRepo {
            let root = std::env::temp_dir()
                .join(format!("speclink-host-changediff-{tag}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&root);
            std::fs::create_dir_all(root.join("src")).unwrap();
            std::fs::write(root.join("src").join("lib.rs"), "fn demo() {}\n").unwrap();
            TempRepo { root }
        }

        fn new(tag: &str) -> TempRepo {
            let repo = TempRepo::no_git(tag);
            repo.git(&["init", "-q"]);
            repo.git(&["config", "user.name", "Sandbox Tester"]);
            repo.git(&["config", "user.email", "sandbox@example.com"]);
            repo.git(&["add", "-A"]);
            repo.git(&["commit", "-q", "-m", "init"]);
            repo
        }

        fn git(&self, args: &[&str]) {
            let ok = Command::new("git")
                .args(args)
                .current_dir(&self.root)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            assert!(ok, "git {args:?} failed");
        }

        fn head(&self) -> String {
            let out = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&self.root)
                .output()
                .expect("git rev-parse");
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }

        fn ws(&self) -> Workspace {
            Workspace { root: self.root.clone(), spec_dir_name: "openspec".to_string() }
        }

        fn write(&self, rel: &str, content: &str) {
            let p = self.root.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, content).unwrap();
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    fn baseline_json(ws: &Workspace, change: &str) -> serde_json::Value {
        let text = std::fs::read_to_string(baseline_path(ws, change)).expect("baseline exists");
        serde_json::from_str(&text).expect("baseline is valid JSON")
    }

    fn assert_utc_rfc3339(s: &str) {
        let parsed = chrono::DateTime::parse_from_rfc3339(s)
            .unwrap_or_else(|e| panic!("capturedAt must be RFC3339: {s} ({e})"));
        assert_eq!(parsed.offset().local_minus_utc(), 0, "capturedAt must be UTC: {s}");
    }

    // --- spec「Apply 開始前記錄 host-local baseline」---

    #[test]
    fn initial_baseline_records_head_dirty_paths_and_utc() {
        // spec Scenario「首次 Apply 記錄乾淨 baseline」：HEAD 為 40 字元 SHA、
        // worktree 有 notes/local.txt 一個既存髒檔 → baseCommit 為該 SHA、
        // dirtyFilesAtStart 為 ["notes/local.txt"]、confidence 為 initial，
        // touched 記錄不存在。
        let repo = TempRepo::new("initial");
        repo.write("notes/local.txt", "scratch\n");
        let ws = repo.ws();
        let outcome = prepare(&ws, "demo", false).expect("initial capture");
        let PrepareOutcome::Captured(b) = outcome else {
            panic!("un-started change must capture an initial baseline, got {outcome:?}");
        };
        assert_eq!(b.base_commit.as_deref(), Some(repo.head().as_str()));
        assert_eq!(b.base_commit.unwrap().len(), 40, "full SHA, not abbreviated");
        assert_eq!(b.dirty_files_at_start, vec!["notes/local.txt".to_string()]);
        assert_eq!(b.confidence, Confidence::Initial);
        assert_eq!(b.version, BASELINE_VERSION);
        assert_utc_rfc3339(&b.captured_at);
        assert!(
            !ws.touched_dir().join("demo.json").exists(),
            "prepare must not create a touched record"
        );
    }

    #[test]
    fn prepare_replaces_an_unstarted_baseline_with_current_state() {
        // D1：change 尚未 started 時，每次 prepare 都以當下狀態原子取代舊 baseline。
        let repo = TempRepo::new("replace");
        repo.write("notes/local.txt", "scratch\n");
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("first capture");
        repo.write("notes/more.txt", "more\n");
        let outcome = prepare(&ws, "demo", false).expect("second capture");
        let PrepareOutcome::Captured(b) = outcome else {
            panic!("un-started re-prepare must recapture, got {outcome:?}");
        };
        assert_eq!(
            b.dirty_files_at_start,
            vec!["notes/local.txt".to_string(), "notes/more.txt".to_string()],
            "sorted, deduped current dirty set"
        );
        // 原子寫入不留同目錄暫存檔。
        let leftovers: Vec<_> = std::fs::read_dir(scope_dir(&ws, "demo"))
            .expect("scope dir exists")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n != "baseline.json")
            .collect();
        assert!(leftovers.is_empty(), "no temp files left behind: {leftovers:?}");
    }

    #[test]
    fn started_change_with_existing_baseline_keeps_the_first_capture() {
        // D1：change 已 started 且 baseline 存在時保持 first baseline。
        let repo = TempRepo::new("keep");
        repo.write("notes/local.txt", "scratch\n");
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("initial capture");
        let first = std::fs::read_to_string(baseline_path(&ws, "demo")).unwrap();
        repo.write("notes/more.txt", "more\n");
        let outcome = prepare(&ws, "demo", true).expect("prepare on a started change");
        assert!(
            matches!(outcome, PrepareOutcome::KeptExisting(_)),
            "started + baseline present must keep, got {outcome:?}"
        );
        let after = std::fs::read_to_string(baseline_path(&ws, "demo")).unwrap();
        assert_eq!(after, first, "first baseline must stay byte-identical");
    }

    #[test]
    fn started_change_without_baseline_records_late() {
        // spec Scenario「已開始但 baseline 缺失」：confidence=late，僅供診斷。
        let repo = TempRepo::new("late");
        let ws = repo.ws();
        let outcome = prepare(&ws, "demo", true).expect("late capture");
        let PrepareOutcome::Late(b) = outcome else {
            panic!("started without baseline must record late, got {outcome:?}");
        };
        assert_eq!(b.confidence, Confidence::Late);
        let on_disk = load_baseline(&ws, "demo").expect("late baseline persisted");
        assert_eq!(on_disk.confidence, Confidence::Late);
    }

    #[test]
    fn missing_git_checkout_records_unavailable_with_null_base() {
        // spec：無 Git checkout 時 confidence=unavailable 且 baseCommit=null。
        let repo = TempRepo::no_git("nogit");
        let ws = repo.ws();
        let outcome = prepare(&ws, "demo", false).expect("unavailable capture");
        let PrepareOutcome::Unavailable(b) = outcome else {
            panic!("no git must record unavailable, got {outcome:?}");
        };
        assert_eq!(b.confidence, Confidence::Unavailable);
        assert_eq!(b.base_commit, None);
        let raw = baseline_json(&ws, "demo");
        assert!(raw["baseCommit"].is_null(), "JSON null, not omitted: {raw}");
    }

    #[test]
    fn baseline_json_uses_camel_case_fields() {
        // spec：baseline JSON 欄位 SHALL 為 camelCase。
        let repo = TempRepo::new("camel");
        repo.write("notes/local.txt", "scratch\n");
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("capture");
        let raw = baseline_json(&ws, "demo");
        for key in ["version", "change", "baseCommit", "dirtyFilesAtStart", "capturedAt", "confidence"]
        {
            assert!(!raw[key].is_null() || key == "baseCommit", "camelCase key {key} present: {raw}");
            assert!(raw.get(key).is_some(), "camelCase key {key} present: {raw}");
        }
        assert_eq!(raw["change"], "demo");
        assert_eq!(raw["confidence"], "initial");
    }

    #[test]
    fn touched_record_stays_byte_identical_across_prepare() {
        // spec：touched v1／v2 記錄 SHALL NOT 因 baseline 而增欄。
        let repo = TempRepo::new("touched");
        let ws = repo.ws();
        let touched = ws.touched_dir().join("demo.json");
        std::fs::create_dir_all(ws.touched_dir()).unwrap();
        let fixture = r#"{"version":2,"change":"demo","touched":[],"entries":[]}"#;
        std::fs::write(&touched, fixture).unwrap();
        prepare(&ws, "demo", false).expect("capture");
        assert_eq!(
            std::fs::read_to_string(&touched).unwrap(),
            fixture,
            "touched fixture must stay byte-identical"
        );
    }

    // --- spec「Git-backed discovery scope 解析完整 worktree patch」---

    fn sha256_of(bytes: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        format!("sha256:{:x}", Sha256::digest(bytes))
    }

    fn paths(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    /// 從 canonical patch 撈出全部 `@@` header 的四個 range。
    fn header_ranges(patch: &str) -> Vec<(u32, u32, u32, u32)> {
        patch
            .lines()
            .filter(|l| l.starts_with("@@ "))
            .map(|l| {
                let inner = l.trim_start_matches("@@ ").split(" @@").next().unwrap();
                let (old, new) = inner.split_once(' ').unwrap();
                let parse = |s: &str| -> (u32, u32) {
                    let s = &s[1..]; // strip -/+
                    match s.split_once(',') {
                        Some((a, b)) => (a.parse().unwrap(), b.parse().unwrap()),
                        None => (s.parse().unwrap(), 1),
                    }
                };
                let (os, ol) = parse(old);
                let (ns, nl) = parse(new);
                (os, ol, ns, nl)
            })
            .collect()
    }

    #[test]
    fn discovery_scope_covers_staged_and_unstaged_where_three_dot_is_empty() {
        // spec Scenario「未提交 staged 與 unstaged 內容都在 scope」：同檔同時有
        // staged 與 unstaged 修改，即使 `git diff <base>...HEAD` 為空，patch 仍
        // 同時含兩部分。
        let repo = TempRepo::new("staged-unstaged");
        let base = repo.head();
        repo.write("src/lib.rs", "fn demo() {}\nfn staged() {}\n");
        repo.git(&["add", "src/lib.rs"]);
        repo.write("src/lib.rs", "fn demo() {}\nfn staged() {}\nfn unstaged() {}\n");
        // 釘住 three-dot 語意：merge-base(base, HEAD)→HEAD 對未提交修改為空。
        let three_dot = Command::new("git")
            .args(["diff", &format!("{base}...HEAD")])
            .current_dir(&repo.root)
            .output()
            .expect("git diff three-dot");
        assert!(
            three_dot.stdout.is_empty(),
            "fixture invariant: three-dot must be empty for uncommitted work"
        );
        let ws = repo.ws();
        let candidate =
            resolve_candidate(&ws, &base, &paths(&["src/lib.rs"])).expect("candidate resolves");
        assert!(candidate.patch.contains("+fn staged() {}"), "staged half: {}", candidate.patch);
        assert!(
            candidate.patch.contains("+fn unstaged() {}"),
            "unstaged half: {}",
            candidate.patch
        );
        assert_eq!(candidate.files.len(), 1);
        let f = &candidate.files[0];
        assert_eq!(f.kind, DeltaKind::Modified);
        assert_eq!(f.old_path.as_deref(), Some("src/lib.rs"));
        assert_eq!(f.new_path.as_deref(), Some("src/lib.rs"));
        assert_eq!(f.before_hash.as_deref(), Some(sha256_of(b"fn demo() {}\n").as_str()));
        assert_eq!(
            f.after_hash.as_deref(),
            Some(sha256_of(b"fn demo() {}\nfn staged() {}\nfn unstaged() {}\n").as_str())
        );
        assert_eq!(candidate.base_commit, base);
        assert_eq!(
            candidate.candidate_hash,
            sha256_of(candidate.patch.as_bytes()),
            "candidateHash pins the canonical patch bytes"
        );
    }

    #[test]
    fn discovery_scope_untracked_touched_file_is_a_whole_file_addition() {
        // spec Scenario「untracked touched file 是整檔 addition」：kind=added、
        // beforeHash=null、首 hunk oldStart=0／oldLines=0。
        let repo = TempRepo::new("untracked");
        let base = repo.head();
        repo.write("src/new_helper.rs", "pub fn helper() {}\npub fn other() {}\n");
        let ws = repo.ws();
        let candidate = resolve_candidate(&ws, &base, &paths(&["src/new_helper.rs"]))
            .expect("candidate resolves");
        assert_eq!(candidate.files.len(), 1);
        let f = &candidate.files[0];
        assert_eq!(f.kind, DeltaKind::Added);
        assert_eq!(f.old_path, None);
        assert_eq!(f.new_path.as_deref(), Some("src/new_helper.rs"));
        assert_eq!(f.before_hash, None);
        assert_eq!(
            f.after_hash.as_deref(),
            Some(sha256_of(b"pub fn helper() {}\npub fn other() {}\n").as_str())
        );
        assert_eq!(f.hunks.len(), 1);
        assert_eq!((f.hunks[0].old_start, f.hunks[0].old_lines), (0, 0), "addition ranges");
        assert_eq!((f.hunks[0].new_start, f.hunks[0].new_lines), (1, 2));
        assert!(candidate.patch.contains("+pub fn helper() {}"), "{}", candidate.patch);
    }

    #[test]
    fn discovery_scope_delete_and_rename_keep_both_sides() {
        // spec Scenario「delete 與 rename 保留雙端語意」：delete 的 afterHash 為
        // null 且 newLines=0；rename 同時輸出 oldPath 與 newPath。
        let repo = TempRepo::new("del-rename");
        repo.write("src/gone.rs", "fn gone() {}\n");
        repo.write("src/before.rs", "fn stable_content_one() {}\nfn stable_content_two() {}\n");
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", "seed"]);
        let base = repo.head();
        std::fs::remove_file(repo.root.join("src/gone.rs")).unwrap();
        repo.git(&["mv", "src/before.rs", "src/after.rs"]);
        let ws = repo.ws();
        let candidate = resolve_candidate(
            &ws,
            &base,
            &paths(&["src/gone.rs", "src/before.rs", "src/after.rs"]),
        )
        .expect("candidate resolves");
        let deleted = candidate
            .files
            .iter()
            .find(|f| f.kind == DeltaKind::Deleted)
            .expect("deleted delta present");
        assert_eq!(deleted.old_path.as_deref(), Some("src/gone.rs"));
        assert_eq!(deleted.new_path, None);
        assert_eq!(deleted.after_hash, None);
        assert_eq!(deleted.before_hash.as_deref(), Some(sha256_of(b"fn gone() {}\n").as_str()));
        assert_eq!(deleted.hunks.len(), 1);
        assert_eq!(
            (deleted.hunks[0].new_start, deleted.hunks[0].new_lines),
            (0, 0),
            "deletion ranges"
        );
        let renamed = candidate
            .files
            .iter()
            .find(|f| f.kind == DeltaKind::Renamed)
            .expect("renamed delta present");
        assert_eq!(renamed.old_path.as_deref(), Some("src/before.rs"));
        assert_eq!(renamed.new_path.as_deref(), Some("src/after.rs"));
    }

    #[test]
    fn discovery_scope_multi_hunk_ranges_match_the_patch_headers() {
        // spec：多段修改 SHALL 保留多筆 hunk；oldStart／oldLines／newStart／
        // newLines 與 canonical patch 的 `@@` header 逐欄一致。
        let repo = TempRepo::new("multihunk");
        let body: String = (1..=20).map(|i| format!("line {i}\n")).collect();
        repo.write("src/wide.rs", &body);
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", "seed"]);
        let base = repo.head();
        let edited = body.replace("line 2\n", "line 2 edited\n").replace("line 18\n", "line 18 edited\n");
        repo.write("src/wide.rs", &edited);
        let ws = repo.ws();
        let candidate =
            resolve_candidate(&ws, &base, &paths(&["src/wide.rs"])).expect("candidate resolves");
        let f = &candidate.files[0];
        assert_eq!(f.hunks.len(), 2, "two spread edits → two hunks: {}", candidate.patch);
        let expected = header_ranges(&candidate.patch);
        let actual: Vec<(u32, u32, u32, u32)> = f
            .hunks
            .iter()
            .map(|h| (h.old_start, h.old_lines, h.new_start, h.new_lines))
            .collect();
        assert_eq!(actual, expected, "parsed ranges mirror the @@ headers");
        // hunk id 穩定且互異（selection 的 handle）。
        assert_ne!(f.hunks[0].id, f.hunks[1].id);
        let again =
            resolve_candidate(&ws, &base, &paths(&["src/wide.rs"])).expect("re-resolve");
        assert_eq!(again.files[0].hunks[0].id, f.hunks[0].id, "ids deterministic");
        assert_eq!(again.candidate_hash, candidate.candidate_hash);
    }

    #[test]
    fn discovery_scope_binary_delta_reports_hashes_without_hunks() {
        // spec：binary SHALL 回報 file hashes 與 kind=binary，hunks 為空。
        let repo = TempRepo::new("binary");
        let before: &[u8] = &[0u8, 159, 146, 150, 1, 2, 3];
        let after: &[u8] = &[0u8, 159, 146, 150, 9, 9, 9, 9];
        std::fs::create_dir_all(repo.root.join("assets")).unwrap();
        std::fs::write(repo.root.join("assets/logo.bin"), before).unwrap();
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", "seed"]);
        let base = repo.head();
        std::fs::write(repo.root.join("assets/logo.bin"), after).unwrap();
        let ws = repo.ws();
        let candidate = resolve_candidate(&ws, &base, &paths(&["assets/logo.bin"]))
            .expect("candidate resolves");
        assert_eq!(candidate.files.len(), 1);
        let f = &candidate.files[0];
        assert_eq!(f.kind, DeltaKind::Binary);
        assert!(f.hunks.is_empty(), "binary delta has no text ranges");
        assert_eq!(f.before_hash.as_deref(), Some(sha256_of(before).as_str()));
        assert_eq!(f.after_hash.as_deref(), Some(sha256_of(after).as_str()));
    }

    #[test]
    fn discovery_scope_excludes_openspec_and_speclink_targets() {
        // spec：openspec artifacts 與 `.speclink` work data SHALL NOT 成為 review
        // target——即使 touched 清單夾帶也不得出現在 candidate。
        let repo = TempRepo::new("excludes");
        let base = repo.head();
        repo.write("src/lib.rs", "fn demo() { changed(); }\n");
        repo.write("openspec/changes/demo/tasks.md", "- [ ] 1.1 x\n");
        repo.write(".speclink/touched/demo.json", "{}");
        let ws = repo.ws();
        let candidate = resolve_candidate(
            &ws,
            &base,
            &paths(&["src/lib.rs", "openspec/changes/demo/tasks.md", ".speclink/touched/demo.json"]),
        )
        .expect("candidate resolves");
        assert_eq!(candidate.files.len(), 1, "only the code path survives");
        assert_eq!(candidate.files[0].new_path.as_deref(), Some("src/lib.rs"));
        assert!(!candidate.patch.contains("openspec"), "{}", candidate.patch);
        assert!(!candidate.patch.contains(".speclink"), "{}", candidate.patch);
    }

    #[test]
    fn discovery_scope_content_lines_never_hijack_the_file_header() {
        // 刪除一行內容為 `-- x` 時 unified diff 的 body 行就是 `--- x`，與檔頭
        // 同形；標頭解析必須止於第一個 `@@`，否則 old_path 會被 hunk 內容覆寫。
        let repo = TempRepo::new("header-hijack");
        repo.write("src/lib.rs", "fn demo() {}\n-- comment line\n++ other line\n");
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", "seed"]);
        let base = repo.head();
        repo.write("src/lib.rs", "fn demo() {}\n");
        let ws = repo.ws();
        let candidate =
            resolve_candidate(&ws, &base, &paths(&["src/lib.rs"])).expect("candidate resolves");
        assert!(
            candidate.patch.contains("\n--- comment line"),
            "fixture invariant: the deleted body line must look like a `---` header: {}",
            candidate.patch
        );
        assert_eq!(candidate.files.len(), 1);
        let f = &candidate.files[0];
        assert_eq!(f.old_path.as_deref(), Some("src/lib.rs"));
        assert_eq!(f.new_path.as_deref(), Some("src/lib.rs"));
        assert_eq!(
            f.before_hash.as_deref(),
            Some(sha256_of(b"fn demo() {}\n-- comment line\n++ other line\n").as_str()),
            "beforeHash must anchor the real file, not a path parsed out of the hunk body"
        );
    }

    #[test]
    fn discovery_scope_gitignored_touched_file_is_still_reviewed() {
        // gitignored 的 touched 檔既不在 `git diff <base>` 也不在
        // `ls-files --others --exclude-standard`；不補齊就會從 frozen patch
        // 靜默消失，而 scope 的界線是 touched 清單本身。
        let repo = TempRepo::new("ignored-touched");
        repo.write(".gitignore", "generated/\n");
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", "ignore"]);
        let base = repo.head();
        repo.write("generated/out.txt", "produced\n");
        let ws = repo.ws();
        let candidate = resolve_candidate(&ws, &base, &paths(&["generated/out.txt"]))
            .expect("candidate resolves");
        assert_eq!(candidate.files.len(), 1, "the ignored touched file stays in scope");
        let f = &candidate.files[0];
        assert_eq!(f.kind, DeltaKind::Added);
        assert_eq!(f.new_path.as_deref(), Some("generated/out.txt"));
        assert!(f.before_hash.is_none());
    }

    #[test]
    fn discovery_scope_unchanged_touched_file_is_not_faked_into_an_addition() {
        // 上一條的反面守門：追蹤中但自 base 起未變的 touched 檔沒有 delta，
        // 不得被當成 untracked 而合成整檔 addition。
        let repo = TempRepo::new("unchanged-touched");
        let base = repo.head();
        let ws = repo.ws();
        let candidate =
            resolve_candidate(&ws, &base, &paths(&["src/lib.rs"])).expect("candidate resolves");
        assert!(candidate.files.is_empty(), "no delta means no file: {}", candidate.patch);
        assert!(candidate.patch.is_empty());
    }

    #[test]
    fn sidecar_temp_names_do_not_collide_between_writers() {
        // temp+rename 的原子性靠「暫存檔專屬於這次寫入」：固定檔名在同一
        // change 的並行 scope（local＋remote）下會互相覆寫、rename 出半寫內容。
        let a = sidecar_tmp_path(Path::new("/tmp/scope"), "snapshot.json");
        let b = sidecar_tmp_path(Path::new("/tmp/scope"), "snapshot.json");
        assert_ne!(a, b, "each write needs its own temp file");
        assert_eq!(a.parent(), Some(Path::new("/tmp/scope")), "temp stays in the target dir");
        assert!(
            a.file_name().unwrap().to_string_lossy().starts_with("snapshot.json."),
            "temp name keeps its stem for recognisable debris: {a:?}"
        );
    }

    // --- spec「歧義 scope 必須 fail closed 並以 hash-pinned selection 解鎖」---

    /// snapshots 目錄零效果：不存在或為空。
    fn assert_zero_snapshot_effects(ws: &Workspace, change: &str) {
        let dir = scope_dir(ws, change).join("snapshots");
        let count = std::fs::read_dir(&dir).map(|it| it.count()).unwrap_or(0);
        assert_eq!(count, 0, "needsInput/rejection must leave zero snapshot effects");
    }

    fn scope_req(touched: &[&str]) -> ScopeRequest {
        ScopeRequest {
            change: "demo".to_string(),
            touched_paths: paths(touched),
            ..Default::default()
        }
    }

    #[test]
    fn review_scope_auto_resolves_a_clean_touched_candidate() {
        // 乾淨 baseline＋開始後才修改的 touched path → 自動凍結 discovery，
        // patchHash 等於 candidateHash。
        let repo = TempRepo::new("scope-clean");
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("clean baseline");
        repo.write("src/lib.rs", "fn demo() { changed(); }\n");
        let outcome = resolve_scope(&ws, &scope_req(&["src/lib.rs"])).expect("scope resolves");
        let ScopeOutcome::Resolved(r) = outcome else {
            panic!("clean candidate must resolve, got {outcome:?}");
        };
        assert_eq!(r.phase, Phase::Discovery);
        assert_eq!(r.patch_hash, r.candidate_hash);
        assert!(r.patch_hash.starts_with("sha256:"));
        assert_eq!(r.paths, vec!["src/lib.rs".to_string()]);
        assert!(r.patch.contains("+fn demo() { changed(); }"), "{}", r.patch);
    }

    #[test]
    fn review_scope_dirty_at_start_touched_path_needs_input() {
        // spec Scenario「開始前已髒的 touched file 不被靜默認領」：非零收場的
        // 核心語意——NeedsInput、ambiguousPaths 含該檔、零 snapshot effects。
        let repo = TempRepo::new("scope-dirty");
        repo.write("src/lib.rs", "fn demo() { dirty_before_start(); }\n");
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("baseline records the dirty file");
        repo.write("src/lib.rs", "fn demo() { dirty_before_start(); more(); }\n");
        let outcome = resolve_scope(&ws, &scope_req(&["src/lib.rs"])).expect("adjudication runs");
        let ScopeOutcome::NeedsInput(n) = outcome else {
            panic!("dirty-at-start must fail closed, got {outcome:?}");
        };
        assert!(n.ambiguous_paths.contains(&"src/lib.rs".to_string()), "{:?}", n.ambiguous_paths);
        assert!(n.candidate_hash.is_some(), "candidate anchor lets the user pin hunks");
        assert!(!n.files.is_empty(), "candidate files carry selectable hunk ids");
        assert_zero_snapshot_effects(&ws, "demo");
    }

    #[test]
    fn review_scope_active_overlap_needs_input() {
        // spec：另一 active change 的 touched record 認領同一路徑 → needsInput。
        let repo = TempRepo::new("scope-overlap");
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("clean baseline");
        repo.write("src/lib.rs", "fn demo() { changed(); }\n");
        let mut req = scope_req(&["src/lib.rs"]);
        req.other_claims = vec![ActiveClaim {
            change: "other".to_string(),
            paths: vec!["src/lib.rs".to_string()],
        }];
        let outcome = resolve_scope(&ws, &req).expect("adjudication runs");
        let ScopeOutcome::NeedsInput(n) = outcome else {
            panic!("active overlap must fail closed, got {outcome:?}");
        };
        assert!(n.ambiguous_paths.contains(&"src/lib.rs".to_string()));
        assert!(
            n.reasons.iter().any(|r| matches!(r, AmbiguityReason::ActiveOverlap { change, .. } if change == "other")),
            "reason names the claiming change: {:?}",
            n.reasons
        );
        assert_zero_snapshot_effects(&ws, "demo");
    }

    #[test]
    fn review_scope_late_baseline_needs_input_until_a_trusted_base() {
        // D3：late baseline 不可自動認領；使用者明示 --base 補可信 fixed point
        // 後（無其他歧義）即可解。
        let repo = TempRepo::new("scope-late");
        let ws = repo.ws();
        prepare(&ws, "demo", true).expect("late baseline");
        repo.write("src/lib.rs", "fn demo() { changed(); }\n");
        let outcome = resolve_scope(&ws, &scope_req(&["src/lib.rs"])).expect("adjudication runs");
        assert!(
            matches!(outcome, ScopeOutcome::NeedsInput(_)),
            "late baseline must fail closed, got {outcome:?}"
        );
        let mut req = scope_req(&["src/lib.rs"]);
        req.base_override = Some(repo.head());
        let outcome = resolve_scope(&ws, &req).expect("trusted base resolves");
        assert!(
            matches!(outcome, ScopeOutcome::Resolved(_)),
            "an explicit trusted base lifts the baseline ambiguity, got {outcome:?}"
        );
    }

    #[test]
    fn review_scope_empty_touched_never_reviews_the_whole_worktree() {
        // spec：touchedFiles 缺失或為空 SHALL NOT 自動審查全 worktree；--base 後
        // 整個 diff 只是 needsInput candidate，仍須 hash-pinned selection。
        let repo = TempRepo::new("scope-empty");
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("clean baseline");
        repo.write("src/lib.rs", "fn demo() { changed(); }\n");
        let outcome = resolve_scope(&ws, &scope_req(&[])).expect("adjudication runs");
        let ScopeOutcome::NeedsInput(n) = outcome else {
            panic!("empty touched must fail closed, got {outcome:?}");
        };
        assert!(n.reasons.contains(&AmbiguityReason::EmptyTouched), "{:?}", n.reasons);
        let mut req = scope_req(&[]);
        req.base_override = Some(repo.head());
        let outcome = resolve_scope(&ws, &req).expect("adjudication runs");
        let ScopeOutcome::NeedsInput(n) = outcome else {
            panic!("--base alone must not freeze the whole worktree, got {outcome:?}");
        };
        assert!(n.candidate_hash.is_some(), "whole diff becomes the needsInput candidate");
        assert!(
            n.files.iter().any(|f| f.new_path.as_deref() == Some("src/lib.rs")),
            "candidate lists the worktree diff: {:?}",
            n.files
        );
        assert_zero_snapshot_effects(&ws, "demo");
    }

    #[test]
    fn review_scope_candidate_drift_rejects_the_stale_selection() {
        // spec Scenario「candidate 漂移拒絕舊選擇」：worktree 又改變後帶舊
        // candidateHash 重試 → 非零拒絕、不建立 snapshot。
        let repo = TempRepo::new("scope-drift");
        repo.write("src/lib.rs", "fn demo() { dirty_before_start(); }\n");
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("baseline with dirty file");
        repo.write("src/lib.rs", "fn demo() { dirty_before_start(); more(); }\n");
        let ScopeOutcome::NeedsInput(n) = resolve_scope(&ws, &scope_req(&["src/lib.rs"])).unwrap()
        else {
            panic!("fixture must be ambiguous");
        };
        let stale = n.candidate_hash.expect("candidate anchor");
        let hunk = n.files[0].hunks[0].id.clone();
        repo.write("src/lib.rs", "fn demo() { dirty_before_start(); more(); drifted(); }\n");
        let mut req = scope_req(&["src/lib.rs"]);
        req.candidate_hash = Some(stale);
        req.include_hunks = vec![hunk];
        let err = resolve_scope(&ws, &req).expect_err("drifted candidate must be rejected");
        assert!(format!("{err:#}").contains("drift"), "error names the drift: {err:#}");
        assert_zero_snapshot_effects(&ws, "demo");
    }

    #[test]
    fn review_scope_hash_pinned_selection_freezes_only_the_selected_hunks() {
        // spec Scenario「hash-pinned hunk selection 成功」：選定 hunks 之外不入
        // frozen patch；files 的 before／after hashes 仍錨定實際整檔內容。
        let repo = TempRepo::new("scope-select");
        let body: String = (1..=20).map(|i| format!("line {i}\n")).collect();
        repo.write("src/wide.rs", &body);
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", "seed"]);
        let dirty = body.replace("line 2\n", "line 2 dirty-before-start\n");
        repo.write("src/wide.rs", &dirty);
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("baseline records the dirty file");
        let current = dirty.replace("line 18\n", "line 18 the-change\n");
        repo.write("src/wide.rs", &current);
        let ScopeOutcome::NeedsInput(n) = resolve_scope(&ws, &scope_req(&["src/wide.rs"])).unwrap()
        else {
            panic!("dirty-at-start fixture must be ambiguous");
        };
        let f = &n.files[0];
        assert_eq!(f.hunks.len(), 2, "two separable hunks");
        // 第二個 hunk 是開始後的修改（行號較大）。
        let chosen = f.hunks.iter().max_by_key(|h| h.new_start).unwrap().id.clone();
        let mut req = scope_req(&["src/wide.rs"]);
        req.candidate_hash = n.candidate_hash.clone();
        req.include_hunks = vec![chosen];
        let ScopeOutcome::Resolved(r) = resolve_scope(&ws, &req).expect("selection resolves")
        else {
            panic!("valid selection must resolve");
        };
        assert!(r.patch.contains("line 18 the-change"), "selected hunk present: {}", r.patch);
        assert!(
            !r.patch.contains("line 2 dirty-before-start"),
            "unselected hunk excluded: {}",
            r.patch
        );
        assert_ne!(r.patch_hash, r.candidate_hash, "narrowed patch has its own identity");
        assert_eq!(r.files[0].hunks.len(), 1, "resolved files carry only the chosen hunks");
        assert_eq!(
            r.files[0].after_hash.as_deref(),
            Some(sha256_of(current.as_bytes()).as_str()),
            "afterHash anchors the real whole file"
        );
        assert_eq!(
            r.files[0].before_hash.as_deref(),
            Some(sha256_of(body.as_bytes()).as_str()),
            "beforeHash anchors the base blob"
        );
    }

    #[test]
    fn review_scope_invalid_duplicate_or_empty_selection_rejects() {
        // spec：hunk ID 不存在、重複 ID、空選擇 SHALL 非零拒絕且零 snapshot
        // effects；binary-only candidate 沒有可選 hunk，任何選擇都被拒。
        let repo = TempRepo::new("scope-badsel");
        repo.write("src/lib.rs", "fn demo() { dirty_before_start(); }\n");
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("baseline with dirty file");
        repo.write("src/lib.rs", "fn demo() { dirty_before_start(); more(); }\n");
        let ScopeOutcome::NeedsInput(n) = resolve_scope(&ws, &scope_req(&["src/lib.rs"])).unwrap()
        else {
            panic!("fixture must be ambiguous");
        };
        let anchor = n.candidate_hash.clone().expect("candidate anchor");
        let valid = n.files[0].hunks[0].id.clone();
        let unknown = "f".repeat(64);
        let cases: Vec<Vec<String>> = vec![
            vec![unknown.clone()],
            vec![valid.clone(), valid.clone()],
            vec![],
        ];
        for include in cases {
            let mut req = scope_req(&["src/lib.rs"]);
            req.candidate_hash = Some(anchor.clone());
            req.include_hunks = include.clone();
            assert!(
                resolve_scope(&ws, &req).is_err(),
                "selection {include:?} must be rejected"
            );
        }
        assert_zero_snapshot_effects(&ws, "demo");
        // binary-only candidate：無可選 hunk。
        let repo2 = TempRepo::new("scope-badsel-bin");
        std::fs::create_dir_all(repo2.root.join("assets")).unwrap();
        std::fs::write(repo2.root.join("assets/logo.bin"), [0u8, 1, 2]).unwrap();
        let ws2 = repo2.ws();
        prepare(&ws2, "demo", false).expect("baseline records the dirty binary");
        std::fs::write(repo2.root.join("assets/logo.bin"), [0u8, 9, 9, 9]).unwrap();
        let ScopeOutcome::NeedsInput(n2) =
            resolve_scope(&ws2, &scope_req(&["assets/logo.bin"])).unwrap()
        else {
            panic!("dirty binary must be ambiguous");
        };
        assert!(
            n2.files.iter().all(|f| f.hunks.is_empty()),
            "binary candidate exposes no selectable hunks: {:?}",
            n2.files
        );
        let mut req = scope_req(&["assets/logo.bin"]);
        req.candidate_hash = n2.candidate_hash.clone();
        req.include_hunks = vec![valid];
        assert!(resolve_scope(&ws2, &req).is_err(), "binary can never be hunk-selected");
        assert_zero_snapshot_effects(&ws2, "demo");
    }

    // --- spec「frozen snapshot 綁定 discovery 與 validation patch」---

    fn resolved(ws: &Workspace, req: &ScopeRequest) -> ResolvedScope {
        match resolve_scope(ws, req).expect("scope resolves") {
            ScopeOutcome::Resolved(r) => r,
            other => panic!("expected a resolved scope, got {other:?}"),
        }
    }

    fn attribution_of(r: &ResolvedScope, path: &str) -> Option<Attribution> {
        r.files.iter().find(|f| f.new_path.as_deref() == Some(path)).and_then(|f| f.attribution)
    }

    fn snapshot_files(ws: &Workspace, change: &str) -> Vec<String> {
        std::fs::read_dir(snapshots_dir(ws, change, StationNs::Review))
            .map(|it| {
                it.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_string()).collect()
            })
            .unwrap_or_default()
    }

    #[test]
    fn review_snapshot_written_with_digest_only_windows_safe_filename() {
        // spec：resolved scope SHALL 原子建立 snapshot；檔名只取 patchHash 的
        // hex digest（不含 `sha256:` 冒號——Windows 檔名安全）。
        let repo = TempRepo::new("snap-name");
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("clean baseline");
        repo.write("src/lib.rs", "fn demo() { changed(); }\n");
        let r = resolved(&ws, &scope_req(&["src/lib.rs"]));
        let names = snapshot_files(&ws, "demo");
        let digest = r.patch_hash.strip_prefix("sha256:").unwrap();
        assert_eq!(names, vec![format!("{digest}.json")], "digest-only filename");
        assert!(!names[0].contains(':'), "no colon in the filename");
        let snap = load_snapshot(&ws, "demo", StationNs::Review, &r.patch_hash).expect("snapshot parses");
        assert_eq!(snap.version, SNAPSHOT_VERSION);
        assert_eq!(snap.change, "demo");
        assert_eq!(snap.phase, Phase::Discovery);
        assert_eq!(snap.patch_hash, r.patch_hash);
        assert_eq!(snap.candidate_hash, r.candidate_hash);
        assert_eq!(snap.base_commit, r.base_commit);
        assert_eq!(snap.patch, r.patch);
        assert_utc_rfc3339(&snap.created_at);
        // camelCase JSON 對外契約。
        let raw: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(snapshot_path(&ws, "demo", StationNs::Review, &r.patch_hash)).unwrap(),
        )
        .unwrap();
        for key in ["candidateHash", "patchHash", "baseCommit", "createdAt", "dirtyFilesAtCapture", "texts"] {
            assert!(raw.get(key).is_some(), "camelCase key {key}: {raw}");
        }
        // 髒檔帳本記錄了 scope 檔自身。
        assert!(
            snap.dirty_files_at_capture.iter().any(|e| e.path == "src/lib.rs"),
            "{:?}",
            snap.dirty_files_at_capture
        );
    }

    #[test]
    fn review_snapshot_freezes_before_and_after_text_but_binary_hashes_only() {
        // spec：UTF-8 scope 檔凍結 beforeText／afterText；binary 僅保留 hashes。
        let repo = TempRepo::new("snap-text");
        std::fs::create_dir_all(repo.root.join("assets")).unwrap();
        std::fs::write(repo.root.join("assets/logo.bin"), [0u8, 1, 2]).unwrap();
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", "seed"]);
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("clean baseline");
        repo.write("src/lib.rs", "fn demo() { changed(); }\n");
        std::fs::write(repo.root.join("assets/logo.bin"), [0u8, 9, 9]).unwrap();
        let r = resolved(&ws, &scope_req(&["src/lib.rs", "assets/logo.bin"]));
        let snap = load_snapshot(&ws, "demo", StationNs::Review, &r.patch_hash).expect("snapshot parses");
        let lib = snap
            .texts
            .iter()
            .find(|t| t.path == "src/lib.rs")
            .expect("text entry for the UTF-8 file");
        assert_eq!(lib.before_text.as_deref(), Some("fn demo() {}\n"));
        assert_eq!(lib.after_text.as_deref(), Some("fn demo() { changed(); }\n"));
        assert!(
            !snap.texts.iter().any(|t| t.path == "assets/logo.bin"),
            "binary keeps hashes only: {:?}",
            snap.texts
        );
        let bin = snap.files.iter().find(|f| f.kind == DeltaKind::Binary).expect("binary delta");
        assert!(bin.before_hash.is_some() && bin.after_hash.is_some());
    }

    #[test]
    fn review_snapshot_validation_patch_covers_findings_and_new_dirty_only() {
        // spec Scenario「follow-up 只輸出 remediation patch」：修正只改 A 並新增
        // 先前乾淨的 C → validation patch 只含 A 自 frozen afterText 起的差異與
        // C 的新差異，不重新輸出未修改的 B。
        let repo = TempRepo::new("snap-validate");
        repo.write("src/a.rs", "alpha\n");
        repo.write("src/b.rs", "beta\n");
        repo.write("src/c.rs", "gamma\n");
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", "seed"]);
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("clean baseline");
        repo.write("src/a.rs", "alpha\nround-one-change\n");
        repo.write("src/b.rs", "beta\nround-one-change\n");
        let r1 = resolved(&ws, &scope_req(&["src/a.rs", "src/b.rs"]));
        // 修正：A 加一行、C 首次變髒；B 不動。
        repo.write("src/a.rs", "alpha\nround-one-change\nthe-fix\n");
        repo.write("src/c.rs", "gamma\nc-fix\n");
        let mut req = scope_req(&["src/a.rs", "src/b.rs"]);
        req.ticket = Some(TicketBinding {
            patch_hash_chain: vec![r1.patch_hash.clone()],
            finding_paths: vec!["src/a.rs".to_string()],
        });
        let r2 = resolved(&ws, &req);
        assert_eq!(r2.phase, Phase::Validation);
        assert!(r2.patch.contains("+the-fix"), "A 的新差異在場: {}", r2.patch);
        assert!(
            !r2.patch.contains("+round-one-change"),
            "A 的差異自 frozen afterText 起算，不重播 Round 1: {}",
            r2.patch
        );
        assert!(r2.patch.contains("+c-fix"), "新髒檔 C 的差異在場: {}", r2.patch);
        assert!(!r2.patch.contains("src/b.rs"), "未修改的 B 不重新輸出: {}", r2.patch);
        assert_eq!(attribution_of(&r2, "src/a.rs"), Some(Attribution::Finding));
        assert_eq!(attribution_of(&r2, "src/c.rs"), Some(Attribution::New));
        assert_ne!(r2.patch_hash, r1.patch_hash);
        // validation snapshot 也凍結，discovery snapshot 因工單引用而保留。
        let names = snapshot_files(&ws, "demo");
        assert_eq!(names.len(), 2, "both rounds' snapshots present: {names:?}");
    }

    #[test]
    fn review_snapshot_validation_emits_unnamed_candidate_movement_as_adjacent() {
        // spec Scenario「未點名候選檔的修復以 adjacent 段進驗證 patch」：Round 1
        // 收錄 A、B、C，findings 只點名 A，修復同時動 A 與 B → Round 2 resolved，
        // patch 含 A（finding）與 B（adjacent，自 B 的凍結後狀態起算）；未動的
        // 候選檔 C 不進 patch。
        let repo = TempRepo::new("snap-adjacent");
        repo.write("src/a.rs", "alpha\n");
        repo.write("src/b.rs", "beta\n");
        repo.write("src/c.rs", "gamma\n");
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", "seed"]);
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("clean baseline");
        repo.write("src/a.rs", "alpha\nround-one-a\n");
        repo.write("src/b.rs", "beta\nround-one-b\n");
        repo.write("src/c.rs", "gamma\nround-one-c\n");
        let touched = ["src/a.rs", "src/b.rs", "src/c.rs"];
        let r1 = resolved(&ws, &scope_req(&touched));
        // 修復：findings 點名的 A 之外，鄰居 B 也被動到；C 原封不動。
        repo.write("src/a.rs", "alpha\nround-one-a\nthe-fix\n");
        repo.write("src/b.rs", "beta\nround-one-b\nneighbour-fix\n");
        let mut req = scope_req(&touched);
        req.ticket = Some(TicketBinding {
            patch_hash_chain: vec![r1.patch_hash.clone()],
            finding_paths: vec!["src/a.rs".to_string()],
        });
        let r2 = resolved(&ws, &req);
        assert_eq!(r2.phase, Phase::Validation);
        assert!(r2.patch.contains("+the-fix"), "findings 檔的修復在場: {}", r2.patch);
        assert!(r2.patch.contains("+neighbour-fix"), "鄰居檔的修復也在場: {}", r2.patch);
        assert!(
            !r2.patch.contains("+round-one-b"),
            "B 自凍結後狀態起算，不重播 Round 1: {}",
            r2.patch
        );
        assert!(!r2.patch.contains("src/c.rs"), "內容未動的候選檔不進 patch: {}", r2.patch);
        assert_eq!(attribution_of(&r2, "src/a.rs"), Some(Attribution::Finding));
        assert_eq!(attribution_of(&r2, "src/b.rs"), Some(Attribution::Adjacent));
        assert!(r2.out_of_scope_changed.is_empty(), "{:?}", r2.out_of_scope_changed);
    }

    #[test]
    fn review_snapshot_validation_walks_the_hash_chain_to_the_latest_capture() {
        // spec Scenario「連續多輪修復未點名檔沿雜湊鏈回走」：B 在 Round 2 沒被
        // 動到（不進 Round 2 快照的保存面），Round 2 的修復才動 B → Round 3 沿
        // patchHash 鏈回走取最近收錄 B 的快照重建凍結後狀態，正常 resolved。
        let repo = TempRepo::new("snap-chain");
        repo.write("src/a.rs", "alpha\n");
        repo.write("src/b.rs", "beta\n");
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", "seed"]);
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("clean baseline");
        repo.write("src/a.rs", "alpha\nround-one-a\n");
        repo.write("src/b.rs", "beta\nround-one-b\n");
        let touched = ["src/a.rs", "src/b.rs"];
        let r1 = resolved(&ws, &scope_req(&touched));
        // Round 2：只修 A —— B 不動，因而不進 Round 2 快照的保存面。
        repo.write("src/a.rs", "alpha\nround-one-a\nfix-one\n");
        let mut req = scope_req(&touched);
        req.ticket = Some(TicketBinding {
            patch_hash_chain: vec![r1.patch_hash.clone()],
            finding_paths: vec!["src/a.rs".to_string()],
        });
        let r2 = resolved(&ws, &req);
        assert!(!r2.patch.contains("src/b.rs"), "B 未動，不進 Round 2 patch: {}", r2.patch);
        // Round 3：這回動到 B —— 上輪快照沒有它，須沿鏈回走 Round 1。
        repo.write("src/b.rs", "beta\nround-one-b\nlate-neighbour-fix\n");
        req.ticket = Some(TicketBinding {
            patch_hash_chain: vec![r2.patch_hash.clone(), r1.patch_hash.clone()],
            finding_paths: vec!["src/a.rs".to_string()],
        });
        let r3 = resolved(&ws, &req);
        assert!(
            r3.patch.contains("+late-neighbour-fix"),
            "回走重建後 B 的新差異在場: {}",
            r3.patch
        );
        assert!(
            !r3.patch.contains("+round-one-b"),
            "自最近收錄輪的凍結後狀態起算: {}",
            r3.patch
        );
        assert_eq!(attribution_of(&r3, "src/b.rs"), Some(Attribution::Adjacent));
        assert!(r3.out_of_scope_changed.is_empty(), "回走成功不得誤判為範圍外: {:?}", r3.out_of_scope_changed);
    }

    #[test]
    fn review_snapshot_never_captured_dirty_file_is_annotated_not_blocked() {
        // spec Scenario「範圍外變動註記不擋凍結」：discovery 時未被收錄的髒檔於
        // 驗證期間變動 → 照常 resolved、列入 outOfScopeChanged、不進 patch。
        let repo = TempRepo::new("snap-outofscope");
        repo.write("src/a.rs", "alpha\n");
        repo.write("notes/d.txt", "scratch\n");
        repo.git(&["add", "src/a.rs"]);
        repo.git(&["commit", "-q", "-m", "seed"]);
        // D 在 Apply 開始前就髒（untracked），且不屬 touched scope。
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("baseline");
        repo.write("src/a.rs", "alpha\nround-one-change\n");
        let r1 = resolved(&ws, &scope_req(&["src/a.rs"]));
        // 修正 A 的同時 D 又變了——D 從未進任何快照的保存面。
        repo.write("src/a.rs", "alpha\nround-one-change\nthe-fix\n");
        repo.write("notes/d.txt", "scratch changed\n");
        let mut req = scope_req(&["src/a.rs"]);
        req.ticket = Some(TicketBinding {
            patch_hash_chain: vec![r1.patch_hash.clone()],
            finding_paths: vec!["src/a.rs".to_string()],
        });
        let r2 = resolved(&ws, &req);
        assert_eq!(r2.out_of_scope_changed, vec!["notes/d.txt".to_string()]);
        assert!(!r2.patch.contains("notes/d.txt"), "範圍外的檔不進 patch: {}", r2.patch);
        assert!(r2.patch.contains("+the-fix"), "凍結照常進行: {}", r2.patch);
        assert_eq!(snapshot_files(&ws, "demo").len(), 2, "validation snapshot 照常寫入");
    }

    #[test]
    fn snapshot_preserves_only_rebuildable_paths_never_rename_sources() {
        // Round 1 C1：preserved 判定必須與 remediation_segment 的重建能力一致
        // ——texts 精確命中或 delta 的 new_path 命中。rename 來源只在 old_path
        // 出現，texts 的鍵是 new_path，重建必然拿錯 before，不得聲稱保存。
        let snap = Snapshot {
            version: SNAPSHOT_VERSION,
            change: "demo".to_string(),
            phase: Phase::Discovery,
            candidate_hash: "sha256:c".to_string(),
            patch_hash: "sha256:p".to_string(),
            base_commit: "b".to_string(),
            created_at: "2026-08-05T00:00:00Z".to_string(),
            dirty_files_at_capture: Vec::new(),
            patch: String::new(),
            files: vec![FileDelta {
                old_path: Some("src/old.rs".to_string()),
                new_path: Some("src/new.rs".to_string()),
                kind: DeltaKind::Renamed,
                before_hash: Some("sha256:x".to_string()),
                after_hash: Some("sha256:y".to_string()),
                hunks: Vec::new(),
                attribution: None,
            }],
            texts: vec![SnapshotText {
                path: "src/new.rs".to_string(),
                before_text: Some("a\n".to_string()),
                after_text: Some("b\n".to_string()),
            }],
        };
        assert!(snapshot_preserves(&snap, "src/new.rs"), "rename 目標可精確重建");
        assert!(
            !snapshot_preserves(&snap, "src/old.rs"),
            "rename 來源無 before 可重建，不得聲稱保存"
        );
    }

    #[test]
    fn review_snapshot_frozen_deletion_is_not_reemitted_every_round() {
        // Round 1 C2：修復期間刪除的候選檔，其刪除凍結進驗證快照後，之後的輪
        // 次不得再被 git_changed_files 撿回當「首次變髒」重播整份刪除——內容
        // 沒動（仍然是刪除狀態）就不進 patch。
        let repo = TempRepo::new("snap-del-once");
        repo.write("src/a.rs", "alpha\n");
        repo.write("src/b.rs", "beta\n");
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", "seed"]);
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("clean baseline");
        repo.write("src/a.rs", "alpha\nround-one-a\n");
        repo.write("src/b.rs", "beta\nround-one-b\n");
        let touched = ["src/a.rs", "src/b.rs"];
        let r1 = resolved(&ws, &scope_req(&touched));
        // 修復判定 B 該整個刪掉。
        std::fs::remove_file(repo.root.join("src/b.rs")).unwrap();
        let mut req = scope_req(&touched);
        req.ticket = Some(TicketBinding {
            patch_hash_chain: vec![r1.patch_hash.clone()],
            finding_paths: vec!["src/b.rs".to_string()],
        });
        let r2 = resolved(&ws, &req);
        assert!(r2.patch.contains("deleted file"), "刪除進 Round 2 驗證面: {}", r2.patch);
        // Round 3：什麼都沒再動 → 驗證 patch 必須為空，B 不得重播。
        req.ticket = Some(TicketBinding {
            patch_hash_chain: vec![r2.patch_hash.clone(), r1.patch_hash.clone()],
            finding_paths: vec![],
        });
        let r3 = resolved(&ws, &req);
        assert_eq!(r3.patch, "", "凍結過的刪除不重播");
        assert!(r3.out_of_scope_changed.is_empty(), "{:?}", r3.out_of_scope_changed);
        // Round 4（Round 2 驗證輪抓到的殘留）：抑制輪自己的快照必須繼續攜帶
        // 「凍結狀態＝已刪除」的紀錄，否則隔一輪 deletion_frozen 失去依據、
        // 又把整份刪除當首次變髒重播。
        req.ticket = Some(TicketBinding {
            patch_hash_chain: vec![
                r3.patch_hash.clone(),
                r2.patch_hash.clone(),
                r1.patch_hash.clone(),
            ],
            finding_paths: vec![],
        });
        let r4 = resolved(&ws, &req);
        assert_eq!(r4.patch, "", "抑制紀錄逐輪傳遞，隔輪也不重播");
        assert!(r4.out_of_scope_changed.is_empty(), "{:?}", r4.out_of_scope_changed);
    }

    #[test]
    fn review_snapshot_contradicting_its_own_dirty_ledger_fails_closed() {
        // design 失敗模式：重建後雜湊與 dirtyFilesAtCapture 記錄不符 → 硬錯誤。
        // 現實觸發路徑＝凍結當下另一個 session 寫了同一個候選檔，兩次讀之間內容
        // 換了；快照自相矛盾時 before 無從信任，不得靜默產出錯誤的 diff。
        let repo = TempRepo::new("snap-ledger-skew");
        repo.write("src/a.rs", "alpha\n");
        repo.write("src/b.rs", "beta\n");
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", "seed"]);
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("clean baseline");
        repo.write("src/a.rs", "alpha\nround-one-a\n");
        repo.write("src/b.rs", "beta\nround-one-b\n");
        let r1 = resolved(&ws, &scope_req(&["src/a.rs", "src/b.rs"]));
        // 竄改帳本：B 的記錄雜湊與快照凍結的 afterText 對不上。
        let path = snapshot_path(&ws, "demo", StationNs::Review, &r1.patch_hash);
        let mut snap: Snapshot = serde_json::from_str(&std::fs::read_to_string(&path).unwrap())
            .expect("snapshot parses");
        let entry = snap
            .dirty_files_at_capture
            .iter_mut()
            .find(|e| e.path == "src/b.rs")
            .expect("B 在帳本裡");
        entry.hash = format!("sha256:{}", "e".repeat(64));
        std::fs::write(&path, serde_json::to_string_pretty(&snap).unwrap()).unwrap();
        repo.write("src/b.rs", "beta\nround-one-b\nneighbour-fix\n");
        let mut req = scope_req(&["src/a.rs", "src/b.rs"]);
        req.ticket = Some(TicketBinding {
            patch_hash_chain: vec![r1.patch_hash.clone()],
            finding_paths: vec!["src/a.rs".to_string()],
        });
        let err = resolve_scope(&ws, &req).expect_err("a self-contradicting snapshot must bail");
        let msg = format!("{err:#}");
        assert!(msg.contains("src/b.rs"), "訊息點名對不上的路徑: {msg}");
        assert!(msg.contains("discard"), "指出明示的出路: {msg}");
    }

    #[test]
    fn review_snapshot_missing_link_in_the_hash_chain_fails_closed() {
        // spec：回走鏈中任一輪快照缺失 SHALL 非零硬錯誤且訊息點名該 patchHash
        //（不得退回 discovery，也不得當成範圍外變動放行）。
        let repo = TempRepo::new("snap-chain-gap");
        repo.write("src/a.rs", "alpha\n");
        repo.write("src/b.rs", "beta\n");
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", "seed"]);
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("clean baseline");
        repo.write("src/a.rs", "alpha\nround-one-a\n");
        repo.write("src/b.rs", "beta\nround-one-b\n");
        let touched = ["src/a.rs", "src/b.rs"];
        let r1 = resolved(&ws, &scope_req(&touched));
        repo.write("src/a.rs", "alpha\nround-one-a\nfix-one\n");
        let mut req = scope_req(&touched);
        req.ticket = Some(TicketBinding {
            patch_hash_chain: vec![r1.patch_hash.clone()],
            finding_paths: vec!["src/a.rs".to_string()],
        });
        let r2 = resolved(&ws, &req);
        // Round 1 的快照被移除後，B 的凍結後狀態再也無從重建。
        std::fs::remove_file(snapshot_path(&ws, "demo", StationNs::Review, &r1.patch_hash)).unwrap();
        repo.write("src/b.rs", "beta\nround-one-b\nlate-neighbour-fix\n");
        req.ticket = Some(TicketBinding {
            patch_hash_chain: vec![r2.patch_hash.clone(), r1.patch_hash.clone()],
            finding_paths: vec!["src/a.rs".to_string()],
        });
        let err = resolve_scope(&ws, &req).expect_err("a gap in the chain must fail closed");
        let msg = format!("{err:#}");
        assert!(msg.contains(&r1.patch_hash), "訊息點名缺失的 patchHash: {msg}");
        assert!(msg.contains("discard"), "指出明示的出路: {msg}");
    }

    #[test]
    fn review_snapshot_missing_does_not_fall_back_to_discovery() {
        // spec Scenario「snapshot 缺失不退回 discovery」：ticket 有 patchHash 但
        // snapshot 已被移除 → 非零、不得用 touched 整檔或 worktree 重新 discovery。
        let repo = TempRepo::new("snap-missing");
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("baseline");
        repo.write("src/lib.rs", "fn demo() { changed(); }\n");
        let mut req = scope_req(&["src/lib.rs"]);
        req.ticket = Some(TicketBinding {
            patch_hash_chain: vec![format!("sha256:{}", "d".repeat(64))],
            finding_paths: vec!["src/lib.rs".to_string()],
        });
        let err = resolve_scope(&ws, &req).expect_err("missing snapshot must fail closed");
        let msg = format!("{err:#}");
        assert!(msg.contains("snapshot"), "error names the missing snapshot: {msg}");
        assert!(!msg.contains("falling back"), "no fallback happens: {msg}");
        // legacy 工單（無 patchHash）同樣不得假裝精確驗收。
        req.ticket = Some(TicketBinding { patch_hash_chain: vec![], finding_paths: vec![] });
        let err = resolve_scope(&ws, &req).expect_err("legacy ticket must fail closed");
        assert!(format!("{err:#}").contains("discard"), "points at the explicit way out: {err:#}");
    }

    #[test]
    fn review_snapshot_orphans_cleared_before_a_ticketless_scope() {
        // spec：add-round 失敗留下的 orphan SHALL 在下一次無對應工單的 scope 前
        // 清除——重算後只剩新 snapshot。
        let repo = TempRepo::new("snap-orphan");
        let ws = repo.ws();
        prepare(&ws, "demo", false).expect("baseline");
        repo.write("src/lib.rs", "fn demo() { changed(); }\n");
        let first = resolved(&ws, &scope_req(&["src/lib.rs"]));
        repo.write("src/lib.rs", "fn demo() { changed(); more(); }\n");
        let second = resolved(&ws, &scope_req(&["src/lib.rs"]));
        assert_ne!(first.patch_hash, second.patch_hash, "fixture drifts between scopes");
        let names = snapshot_files(&ws, "demo");
        let new_digest = second.patch_hash.strip_prefix("sha256:").unwrap();
        assert_eq!(names, vec![format!("{new_digest}.json")], "orphan replaced by the new snapshot");
    }

    #[test]
    fn unwritable_sidecar_location_errors_instead_of_capturing() {
        // spec Scenario「baseline 寫入失敗停止 Apply 起點」：寫入失敗 → 非零收場
        //（CLI 面）；host 面＝prepare 回錯誤，呼叫端不得接著 in-progress add。
        let repo = TempRepo::new("unwritable");
        let ws = repo.ws();
        // 讓 `.speclink/review-scopes` 成為檔案：其下目錄無從建立。
        std::fs::create_dir_all(ws.work_dir()).unwrap();
        std::fs::write(ws.work_dir().join("review-scopes"), "not a dir").unwrap();
        let err = prepare(&ws, "demo", false).expect_err("write failure must error");
        let msg = format!("{err:#}");
        assert!(!msg.is_empty(), "error carries a message");
        assert!(
            !Path::new(&baseline_path(&ws, "demo")).exists(),
            "no baseline may appear on the failure path"
        );
    }
}
