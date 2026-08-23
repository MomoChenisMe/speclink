//! The two quality stations: review and verify.
//!
//! Both are Dual, declared at the family level (`station_dual`) after the clap
//! subcommands are normalized into one `StationVerb`, so the two stations share
//! a single ticket flow instead of duplicating it.

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use speclink_core as core;

use crate::color;
use crate::common::{open_project, print_json, read_stdin_content, require_workspace};
use crate::dual;
use crate::remote_base::RemoteCtx;
use core::store::Store;

#[derive(Args)]
pub(crate) struct ReviewArgs {
    #[command(subcommand)]
    command: ReviewCommands,
}
#[derive(Subcommand)]
enum ReviewCommands {
    /// Capture the Apply baseline sidecar before the change is marked in-progress
    Prepare { change: String },
    /// Resolve and freeze the review scope (--json for the structured payload)
    Scope {
        change: String,
        #[arg(long)]
        json: bool,
        /// Trusted fixed point overriding the Apply baseline
        #[arg(long)]
        base: Option<String>,
        /// Candidate identity a hash-pinned hunk selection is anchored to
        #[arg(long = "candidate-hash")]
        candidate_hash: Option<String>,
        /// Hunk id to include (repeatable; requires --candidate-hash)
        #[arg(long = "include-hunk")]
        include_hunk: Vec<String>,
    },
    /// Append a review round to the change's ticket (content from stdin; creates the ticket on the first round)
    #[command(name = "add-round")]
    AddRound {
        change: String,
        #[arg(long)]
        stdin: bool,
    },
    /// Print the review ticket (--json for the structured payload)
    Show {
        change: String,
        #[arg(long)]
        json: bool,
    },
    /// Stamp the review: requires all tasks done and an empty must-fix set in the last round (SUGGESTION never blocks)
    Stamp {
        change: String,
        /// Stamp despite outstanding must-fix (CRITICAL/WARNING) findings in the last round
        #[arg(long)]
        accept: bool,
        /// Tool identity recorded as reviewed_with (mirrors `new change --agent`)
        #[arg(long)]
        agent: Option<String>,
    },
    /// Discard the review ticket without stamping
    Discard { change: String },
}
#[derive(Args)]
pub(crate) struct VerifyArgs {
    #[command(subcommand)]
    command: VerifyCommands,
}
/// 驗證站沒有 `prepare`：Apply baseline 由 apply 流程一次錄下、兩站共用
/// （design D8），第二個 prepare 只會覆蓋同一份 sidecar。
#[derive(Subcommand)]
enum VerifyCommands {
    /// Resolve and freeze the verify scope (--json for the structured payload)
    Scope {
        change: String,
        #[arg(long)]
        json: bool,
        /// Trusted fixed point overriding the Apply baseline
        #[arg(long)]
        base: Option<String>,
        /// Candidate identity a hash-pinned hunk selection is anchored to
        #[arg(long = "candidate-hash")]
        candidate_hash: Option<String>,
        /// Hunk id to include (repeatable; requires --candidate-hash)
        #[arg(long = "include-hunk")]
        include_hunk: Vec<String>,
    },
    /// Append a verify round to the change's ticket (content from stdin; requires every task done)
    #[command(name = "add-round")]
    AddRound {
        change: String,
        #[arg(long)]
        stdin: bool,
    },
    /// Print the verify ticket (--json for the structured payload)
    Show {
        change: String,
        #[arg(long)]
        json: bool,
    },
    /// Stamp the verification: requires all tasks done and an empty must-fix set in the last round (SUGGESTION never blocks)
    Stamp {
        change: String,
        /// Stamp despite outstanding must-fix (CRITICAL/WARNING) findings in the last round
        #[arg(long)]
        accept: bool,
        /// Tool identity recorded as verified_with (mirrors `new change --agent`)
        #[arg(long)]
        agent: Option<String>,
    },
    /// Discard the verify ticket without stamping
    Discard { change: String },
}
/// 兩個品質站共用的動詞形狀（design D1 的 CLI 面）：clap 的兩個子命令 enum
/// 各自保留自己的說明文字與旗標可用性（verify 無 `prepare`），在此正規化為
/// 同一組動詞，往下只有一份實作。
enum StationVerb {
    Scope {
        change: String,
        json: bool,
        base: Option<String>,
        candidate_hash: Option<String>,
        include_hunk: Vec<String>,
    },
    AddRound {
        change: String,
        stdin: bool,
    },
    Show {
        change: String,
        json: bool,
    },
    Stamp {
        change: String,
        accept: bool,
        agent: Option<String>,
    },
    Discard {
        change: String,
    },
}
/// 一個品質站在 CLI 這層的全部站別差異：引擎常數組（工單檔名、meta 前綴、
/// 訊息用詞）與 host-local snapshot namespace。
struct StationCli {
    pub station: &'static core::station::Station,
    pub ns: speclink_host::change_diff::StationNs,
}
const REVIEW_CLI: StationCli = StationCli {
    station: &core::review::STATION,
    ns: speclink_host::change_diff::StationNs::Review,
};
const VERIFY_CLI: StationCli = StationCli {
    station: &core::verify::STATION,
    ns: speclink_host::change_diff::StationNs::Verify,
};
pub(crate) fn cmd_review(a: ReviewArgs) -> Result<()> {
    let verb = match a.command {
        // prepare：Apply baseline 的兩站共用入口（verify 無此子指令），自成雙臂。
        ReviewCommands::Prepare { change } => {
            return dual(change, review_prepare_fs, |ctx, c| remote_review_prepare(ctx, c));
        }
        ReviewCommands::Scope { change, json, base, candidate_hash, include_hunk } => {
            StationVerb::Scope { change, json, base, candidate_hash, include_hunk }
        }
        ReviewCommands::AddRound { change, stdin } => StationVerb::AddRound { change, stdin },
        ReviewCommands::Show { change, json } => StationVerb::Show { change, json },
        ReviewCommands::Stamp { change, accept, agent } => {
            StationVerb::Stamp { change, accept, agent }
        }
        ReviewCommands::Discard { change } => StationVerb::Discard { change },
    };
    station_dual(&REVIEW_CLI, verb)
}
pub(crate) fn cmd_verify(a: VerifyArgs) -> Result<()> {
    let verb = match a.command {
        VerifyCommands::Scope { change, json, base, candidate_hash, include_hunk } => {
            StationVerb::Scope { change, json, base, candidate_hash, include_hunk }
        }
        VerifyCommands::AddRound { change, stdin } => StationVerb::AddRound { change, stdin },
        VerifyCommands::Show { change, json } => StationVerb::Show { change, json },
        VerifyCommands::Stamp { change, accept, agent } => {
            StationVerb::Stamp { change, accept, agent }
        }
        VerifyCommands::Discard { change } => StationVerb::Discard { change },
    };
    station_dual(&VERIFY_CLI, verb)
}
/// 兩站共用的 Dual 宣告：正規化後的 StationVerb 派給本機／remote 站臂——
/// 站別差異只剩 StationCli 常數組，正規化邏輯不進臂內。
fn station_dual(cli: &StationCli, verb: StationVerb) -> Result<()> {
    dual(verb, |v| station_fs(cli, v), |ctx, v| remote_station(ctx, cli, v))
}
/// `review prepare` 的本機臂（驗證站無此動詞：Apply baseline 兩站共用）。
fn review_prepare_fs(change: String) -> Result<()> {
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    if !store.change_exists(&change) {
        bail!("change not found: {change}");
    }
    // started 判讀走 fail-closed 解析：壞 meta 不得被讀作「未開始」。
    let raw_meta = store.read_change_meta(&change);
    let meta = core::model::ChangeMeta::from_text(raw_meta.as_deref())
        .map_err(|e| anyhow::anyhow!(e))?;
    run_review_prepare(&ws, &change, meta.started_at.is_some())
}
fn station_fs(cli: &StationCli, verb: StationVerb) -> Result<()> {
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    let st = cli.station;
    match verb {
        StationVerb::Scope { change, json, base, candidate_hash, include_hunk } => {
            if !store.change_exists(&change) {
                bail!("change not found: {change}");
            }
            let ticket = store
                .artifact_exists(&change, st.doc)
                .then(|| core::station::show(st, store, &change))
                .transpose()?
                .map(|t| speclink_host::change_diff::TicketBinding {
                    patch_hash_chain: patch_hash_chain(
                        t.rounds.iter().map(|r| r.patch_hash.as_deref()),
                    ),
                    finding_paths: t
                        .last_round()
                        .findings
                        .iter()
                        .map(|f| f.path.clone())
                        .collect(),
                });
            let names = store.list_changes().into_iter().map(|c| c.name).collect();
            let req = build_scope_request(
                store,
                change,
                names,
                ticket,
                base,
                candidate_hash,
                include_hunk,
                cli.ns,
            );
            run_station_scope(&ws, st, &req, json)?;
        }
        StationVerb::AddRound { change, stdin } => {
            let content = read_stdin_content(stdin);
            let round = core::station::add_round(st, store, &change, &content)?;
            render_station_action(st, StationAction::AddRound(round), &change);
        }
        StationVerb::Show { change, json } => {
            // `--json` 只要解析後的工單，不碰原文——與導入共用渲染前的行為一致
            // （多讀一次是多一個失敗面）。
            if json {
                let ticket = core::station::show(st, store, &change)?;
                return render_station_show(st, &change, &ticket, None, true);
            }
            // 人眼路徑印工單原文（show 已驗證存在與格式）。
            let (ticket, doc) = core::station::show_with_content(st, store, &change)?;
            let doc = doc.expect("show verified the ticket exists");
            return render_station_show(st, &change, &ticket, Some(&doc), false);
        }
        StationVerb::Stamp { change, accept, agent } => {
            let actor = speclink_host::context::git_identity(&ws.root);
            let read_file = |p: &str| core::util::read_bytes_opt(&ws.root.join(p));
            let file_exists = |p: &str| ws.root.join(p).is_file();
            core::station::stamp(
                st,
                store,
                &change,
                accept,
                actor.as_deref(),
                agent.as_deref(),
                &read_file,
                &file_exists,
            )?;
            render_station_action(st, StationAction::Stamp, &change);
            clear_snapshots_warning(&ws, st, cli.ns, &change);
        }
        StationVerb::Discard { change } => {
            core::station::discard(st, store, &change)?;
            render_station_action(st, StationAction::Discard, &change);
            clear_snapshots_warning(&ws, st, cli.ns, &change);
        }
    }
    Ok(())
}
/// 兩站三個「動作完成」動詞的成功行——只差站名片語，共用一支渲染。
fn render_station_action(st: &core::station::Station, action: StationAction, change: &str) {
    // 動作片語不同、外框相同——三臂共用同一行輸出。
    let did = match action {
        StationAction::AddRound(round) => format!("Recorded {} Round {round}", st.noun_phrase),
        StationAction::Stamp => format!("Stamped {}", st.noun_phrase),
        StationAction::Discard => format!("Discarded {} ticket", st.noun_phrase),
    };
    println!("{} {did} for change '{change}'", color::green("✓"));
}
enum StationAction {
    AddRound(usize),
    Stamp,
    Discard,
}
/// 工單閱讀的共用渲染。`--json` 一律走 `ticket_json`——payload 的欄位集合是
/// 對外契約，工單原文不屬於它，所以原文只走人眼路徑。`doc` 缺席是 remote 對
/// 舊 server 的退化：印結構化摘要，而不是拿結構化欄位反推一份假原文。
fn render_station_show(
    st: &core::station::Station,
    change: &str,
    ticket: &core::station::Ticket,
    doc: Option<&str>,
    json: bool,
) -> Result<()> {
    if json {
        return print_json(&ticket_json(change, ticket));
    }
    if let Some(doc) = doc {
        print!("{doc}");
        return Ok(());
    }
    println!("{} — {change}", st.title);
    for r in &ticket.rounds {
        println!("\nRound {}", r.index);
        if let Some(phase) = &r.phase {
            println!("  Phase: {}", phase.as_str());
        }
        if let Some(hash) = &r.patch_hash {
            println!("  Patch: {hash}");
        }
        if !r.scope.is_empty() {
            println!("  Scope: {}", r.scope.join(", "));
        }
        for f in &r.findings {
            println!("  - [{}] {} — {}", f.severity.as_str(), f.path, f.text);
        }
    }
    Ok(())
}
/// 工單的 `--json` payload（local／remote 之外，兩站也共用同一份組裝——欄位集合
/// 與 null 語意是對外契約）。
fn ticket_json(change: &str, ticket: &core::station::Ticket) -> serde_json::Value {
    let round_json = |r: &core::station::Round| {
        serde_json::json!({
            "index": r.index,
            "phase": r.phase.map(|p| p.as_str()),
            "patchHash": r.patch_hash,
            "scope": r.scope,
            "findings": r
                .findings
                .iter()
                .map(|f| serde_json::json!({
                    "severity": f.severity.as_str(),
                    "path": f.path,
                    "text": f.text,
                }))
                .collect::<Vec<_>>(),
        })
    };
    serde_json::json!({
        "change": change,
        "rounds": ticket.rounds.iter().map(round_json).collect::<Vec<_>>(),
        "lastRound": round_json(ticket.last_round()),
    })
}
/// `review prepare` 的唯一實作（local／remote 共用）：sidecar 全在本地
/// checkout，只有「是否已開工」這件事實由各自的 store 提供。
///
/// initial／kept 靜默（spec：stdout 為空）；late／unavailable 以 stderr 警告但
/// 仍 exit 0，讓 apply 可以繼續。
fn run_review_prepare(
    ws: &core::workspace::Workspace,
    change: &str,
    started: bool,
) -> Result<()> {
    match speclink_host::change_diff::prepare(ws, change, started)? {
        speclink_host::change_diff::PrepareOutcome::Captured(_)
        | speclink_host::change_diff::PrepareOutcome::KeptExisting(_) => {}
        speclink_host::change_diff::PrepareOutcome::Late(_) => eprintln!(
            "Warning: baseline for '{change}' was captured late (the change already started) \
             — review scope will need an explicit trusted --base fixed point"
        ),
        speclink_host::change_diff::PrepareOutcome::Unavailable(_) => eprintln!(
            "Warning: no git checkout found — baseline recorded as unavailable; review scope \
             will need an explicit trusted --base fixed point"
        ),
    }
    Ok(())
}
/// 工單各輪的 patchHash 鏈（新→舊）——validation 回走重建 adjacent 段的依據。
/// 末輪沒有 patchHash（legacy 輪）時回空鏈：驗證輪據此 fail closed，不拿更早
/// 一輪的快照冒充末輪。
fn patch_hash_chain<'a>(
    rounds: impl DoubleEndedIterator<Item = Option<&'a str>>,
) -> Vec<String> {
    let mut newest_first = rounds.rev();
    match newest_first.next() {
        Some(Some(last)) => {
            std::iter::once(last).chain(newest_first.flatten()).map(str::to_string).collect()
        }
        _ => Vec::new(),
    }
}
/// `review scope` 的請求組裝（local／remote 共用）：touched 記錄與重疊認領都是
/// host-local 事實，只有 change 清單與工單由各自的 store 提供。
#[allow(clippy::too_many_arguments)]
fn build_scope_request(
    store: &dyn Store,
    change: String,
    other_change_names: Vec<String>,
    ticket: Option<speclink_host::change_diff::TicketBinding>,
    base: Option<String>,
    candidate_hash: Option<String>,
    include_hunks: Vec<String>,
    station: speclink_host::change_diff::StationNs,
) -> speclink_host::change_diff::ScopeRequest {
    let touched_paths = core::tasks::TouchedRecord::load(store, &change).all_files();
    // 其他 active change 的 host-local touched 認領（overlap 守門）。
    let other_claims = other_change_names
        .into_iter()
        .filter(|name| *name != change)
        .filter_map(|name| {
            let paths = core::tasks::TouchedRecord::load(store, &name).all_files();
            (!paths.is_empty())
                .then_some(speclink_host::change_diff::ActiveClaim { change: name, paths })
        })
        .collect();
    speclink_host::change_diff::ScopeRequest {
        change,
        touched_paths,
        other_claims,
        ticket,
        base_override: base,
        candidate_hash,
        include_hunks,
        station,
    }
}
/// scope 的解析與呈現（local／remote／兩站共用——resolved payload 逐位元同形的
/// 唯一保證）。needsInput 印 JSON（--json 時）後以非零收場。
fn run_station_scope(
    ws: &core::workspace::Workspace,
    st: &core::station::Station,
    req: &speclink_host::change_diff::ScopeRequest,
    json: bool,
) -> Result<()> {
    match speclink_host::change_diff::resolve_scope(ws, req)? {
        speclink_host::change_diff::ScopeOutcome::Resolved(r) => {
            if json {
                return print_json(&serde_json::json!({
                    "change": r.change,
                    "phase": r.phase.as_str(),
                    "state": "resolved",
                    "baseCommit": r.base_commit,
                    "candidateHash": r.candidate_hash,
                    "patchHash": r.patch_hash,
                    "paths": r.paths,
                    "files": r.files,
                    "patch": r.patch,
                    "outOfScopeChanged": r.out_of_scope_changed,
                }));
            }
            let hunk_count: usize = r.files.iter().map(|f| f.hunks.len()).sum();
            println!(
                "{} Frozen {} scope for change '{}'",
                color::green("✓"),
                r.phase.as_str(),
                r.change
            );
            println!("  Patch: {}", r.patch_hash);
            println!(
                "  Scope: {} file(s), {} hunk(s){}",
                r.paths.len(),
                hunk_count,
                attribution_breakdown(&r.files)
            );
            // 範圍外變動＝從未進本站檢查面的候選檔又動了：轉知使用者，不入檢查面。
            if !r.out_of_scope_changed.is_empty() {
                println!(
                    "  Changed outside the {} scope: {}",
                    st.noun_phrase,
                    r.out_of_scope_changed.join(", ")
                );
            }
            Ok(())
        }
        speclink_host::change_diff::ScopeOutcome::NeedsInput(n) => {
            if json {
                print_json(&serde_json::json!({
                    "change": n.change,
                    "phase": n.phase.as_str(),
                    "state": "needsInput",
                    "candidateHash": n.candidate_hash,
                    "ambiguousPaths": n.ambiguous_paths,
                    "files": n.files,
                }))?;
            }
            bail!("{}", scope_needs_input_message(st, &n));
        }
    }
}
/// 驗證輪計數行的三類出身補述（design D4）：discovery 沒有上輪可歸因，回空字串。
fn attribution_breakdown(files: &[speclink_host::change_diff::FileDelta]) -> String {
    use speclink_host::change_diff::Attribution;
    let count = |a: Attribution| files.iter().filter(|f| f.attribution == Some(a)).count();
    let (finding, adjacent, new) =
        (count(Attribution::Finding), count(Attribution::Adjacent), count(Attribution::New));
    if finding + adjacent + new == 0 {
        return String::new();
    }
    format!(" — {finding} finding, {adjacent} adjacent, {new} new")
}
/// stamp／discard 後清本站的 host-local snapshots（Apply baseline 與另一站的
/// snapshots 保留——design D8）。清除失敗僅警告——canonical 工單／metadata
/// mutation 已完成，不回滾。
fn clear_snapshots_warning(
    ws: &core::workspace::Workspace,
    st: &core::station::Station,
    ns: speclink_host::change_diff::StationNs,
    change: &str,
) {
    if let Err(e) = speclink_host::change_diff::clear_snapshots(ws, change, ns) {
        eprintln!("Warning: could not clear {} snapshots for '{change}': {e}", st.noun_phrase);
    }
}
/// needsInput 的 stderr 說明：原因、ambiguous paths 與三種處置（可信 --base、
/// hash-pinned --include-hunk、隔離 worktree）。
fn scope_needs_input_message(
    st: &core::station::Station,
    n: &speclink_host::change_diff::ScopeNeedsInput,
) -> String {
    use speclink_host::change_diff::AmbiguityReason;
    let mut lines = vec![format!("{} scope for '{}' needs input:", st.noun, n.change)];
    for reason in &n.reasons {
        lines.push(match reason {
            AmbiguityReason::BaselineMissing => {
                "  - no Apply baseline was captured for this change".to_string()
            }
            AmbiguityReason::BaselineLate => {
                "  - the baseline was captured late (change already started)".to_string()
            }
            AmbiguityReason::BaselineUnavailable => {
                "  - the baseline has no usable git fixed point".to_string()
            }
            AmbiguityReason::BaseUnresolvable(e) => format!("  - {e}"),
            AmbiguityReason::DirtyAtStart(paths) => {
                format!("  - touched paths were already dirty at start: {}", paths.join(", "))
            }
            AmbiguityReason::ActiveOverlap { change, paths } => format!(
                "  - active change '{change}' also claims: {}",
                paths.join(", ")
            ),
            AmbiguityReason::EmptyTouched => {
                "  - no touched files recorded — the whole worktree is never auto-reviewed"
                    .to_string()
            }
        });
    }
    if !n.ambiguous_paths.is_empty() {
        lines.push(format!("  ambiguous paths: {}", n.ambiguous_paths.join(", ")));
    }
    lines.push("resolve it explicitly by one of:".to_string());
    // Only discovery ever needs input: a validation round resolves its scope by
    // content movement against the frozen snapshot chain, so all three
    // disposals here address a discovery fixed point.
    lines.push("  1. pass a trusted fixed point with --base <rev>".to_string());
    lines.push(
        "  2. pin hunks with --candidate-hash <sha256> and --include-hunk <id> (repeatable)"
            .to_string(),
    );
    lines.push("  3. redo the work in an isolated worktree".to_string());
    lines.join("\n")
}
#[cfg(test)]
mod patch_hash_chain_tests {
    //! Ticket→TicketBinding 的 patchHash 鏈組裝（spec「frozen snapshot 綁定
    //! discovery 與 validation patch」的回走鏈輸入）：新→舊、legacy 空洞跳過、
    //! 末輪 legacy＝空鏈（驗證輪據此 fail closed，不拿舊輪冒充末輪）。
    use super::patch_hash_chain;

    #[test]
    fn newest_first_and_legacy_gaps_skipped() {
        let rounds = [Some("sha256:r1"), None, Some("sha256:r3")];
        assert_eq!(
            patch_hash_chain(rounds.iter().map(|h| *h)),
            vec!["sha256:r3".to_string(), "sha256:r1".to_string()],
            "鏈序新→舊，中段 legacy 輪跳過"
        );
    }

    #[test]
    fn legacy_last_round_yields_an_empty_chain_even_with_older_hashes() {
        let rounds = [Some("sha256:r1"), Some("sha256:r2"), None];
        assert_eq!(
            patch_hash_chain(rounds.iter().map(|h| *h)),
            Vec::<String>::new(),
            "末輪無 hash 時不得拿舊輪快照冒充末輪"
        );
        assert_eq!(patch_hash_chain([None].iter().map(|h| *h)), Vec::<String>::new());
        assert_eq!(
            patch_hash_chain(std::iter::empty::<Option<&str>>()),
            Vec::<String>::new(),
            "空工單＝空鏈"
        );
    }
}
/// `review prepare` 的 remote 面：sidecar 仍在本地 checkout，先做 remote read
/// （存在＋startedAt），失敗即零 sidecar effects。listing 的 status 只由任務
/// 完成度推導，不能當「已開工」讀。
fn remote_review_prepare(ctx: &RemoteCtx, change: String) -> Result<()> {
    let summary = ctx
        .client
        .list_changes()?
        .changes
        .into_iter()
        .find(|c| c.name == change)
        .ok_or_else(|| anyhow::anyhow!("change not found: {change}"))?;
    let ws = require_workspace()?;
    run_review_prepare(&ws, &change, summary.started_at.is_some())
}
/// 兩個品質站的 remote 動詞（唯一實作落點）：工單經 typed client 的站別端點
/// 讀寫，scope 仍由本地 checkout 的 Host resolver 解析——server 不收 patch、
/// 不收 snapshot，也沒有 Git endpoint。
fn remote_station(ctx: &RemoteCtx, cli: &StationCli, verb: StationVerb) -> Result<()> {
    let st = cli.station;
    let noun = st.noun;
    match verb {
        StationVerb::Scope { change, json, base, candidate_hash, include_hunk } => {
            // 同一 Host resolver：remote 只提供 active changes 與 ticket 事實，
            // Git、baseline、touched、snapshot 全在本地 checkout。
            let changes = ctx.client.list_changes()?.changes;
            if !changes.iter().any(|c| c.name == change) {
                anyhow::bail!("change not found: {change}");
            }
            let ws = require_workspace()?;
            let ticket = ctx.client.station_ticket_if_any(noun, &change)?.map(|t| {
                speclink_host::change_diff::TicketBinding {
                    patch_hash_chain: patch_hash_chain(
                        t.rounds.iter().map(|r| r.patch_hash.as_deref()),
                    ),
                    finding_paths: t
                        .last_round
                        .findings
                        .iter()
                        .map(|f| f.path.clone())
                        .collect(),
                }
            });
            let names = changes.into_iter().map(|c| c.name).collect();
            // touched 認領同樣來自本地 checkout：remote 模式下 change 文件在
            // server，但 scope 解析讀的是這台機器上的 evidence 記錄——沒有就是
            // 空認領，與這條路徑一直以來的行為相同。
            let local = speclink_fs::FsStore::new(&ws.root, &ws.spec_dir_name);
            let req = build_scope_request(
                &local,
                change,
                names,
                ticket,
                base,
                candidate_hash,
                include_hunk,
                cli.ns,
            );
            run_station_scope(&ws, st, &req, json)
        }
        StationVerb::AddRound { change, stdin } => {
            let content = read_stdin_content(stdin);
            let round = ctx.client.station_add_round(noun, &change, &content)?.round;
            // u64→usize：支援平台皆 64-bit，無損（不為不可能的情境設防）。
            render_station_action(st, StationAction::AddRound(round as usize), &change);
            Ok(())
        }
        StationVerb::Show { change, json } => {
            let resp = ctx.client.station_ticket(noun, &change)?;
            // 人眼＋有原文＝純轉印，不碰結構化欄位——這正是原文上 wire 的目的：
            // server 詞彙比 CLI 新（未知 token、新形狀）也不影響印出工單本文。
            // 解析（與其 fail-loud）只留給真正讀 token 的兩條路：--json 與退化摘要。
            if !json {
                if let Some(doc) = &resp.content {
                    print!("{doc}");
                    return Ok(());
                }
            }
            let ticket = to_station_ticket(resp)?;
            render_station_show(st, &change, &ticket, None, json)
        }
        StationVerb::Stamp { change, accept, agent } => {
            // 指紋歸屬（design D4a）：工作樹持有者是這裡——先取工單算 Scope
            // 聯集（鏡射引擎的正規化：`\`→`/`、去重、排序），逐檔讀 checkout
            // 內容算雜湊，隨請求上 wire；server 驗集合相等、不重算。
            let ticket = ctx.client.station_ticket(noun, &change)?;
            let Some(ws) = core::workspace::Workspace::discover_cwd()? else {
                anyhow::bail!(
                    "{noun} stamp needs a workspace checkout to fingerprint scope files"
                );
            };
            let paths = core::station::scope_union(
                ticket.rounds.iter().flat_map(|r| r.scope.iter().map(String::as_str)),
            );
            // 修正可能刪除／改名早輪檢查過的檔：仍存在者算雜湊，已消失者以
            // missing 明示宣告——server 無工作樹，分割由這裡的存在性判定。
            let (present, missing): (Vec<String>, Vec<String>) =
                paths.into_iter().partition(|p| ws.root.join(p).is_file());
            let read_file = |p: &str| core::util::read_bytes_opt(&ws.root.join(p));
            let scope: Vec<_> = core::station::fingerprint_scope(&present, &read_file)?
                .into_iter()
                .map(|(path, hash)| speclink_protocol::command::ReviewScopeEntryDto { path, hash })
                .collect();
            ctx.client.station_stamp(noun, &change, accept, agent.as_deref(), &scope, &missing)?;
            render_station_action(st, StationAction::Stamp, &change);
            Ok(())
        }
        StationVerb::Discard { change } => {
            ctx.client.station_discard(noun, &change)?;
            render_station_action(st, StationAction::Discard, &change);
            Ok(())
        }
    }
}
/// The wire ticket reshaped into the engine's ticket, so `--json` goes through
/// the one payload assembly whose field set is the public contract (and the
/// document body, which is not part of that contract, cannot leak into it).
///
/// An unrecognized phase token is an error, not a silent `None`: it means the
/// server speaks a round vocabulary this CLI does not, and rendering it as a
/// legacy round would state something false about the ticket.
fn to_station_ticket(
    resp: speclink_protocol::command::ReviewTicketResponse,
) -> Result<core::station::Ticket> {
    // 引擎工單的不變量是至少一輪；wire 形狀上允許空陣列，所以這裡是明確
    // 錯誤，不是留給 last_round() 的 expect 去炸。
    if resp.rounds.is_empty() {
        anyhow::bail!(
            "the server returned a ticket with no rounds for change '{}' — cannot render it",
            resp.change
        );
    }
    let rounds = resp
        .rounds
        .into_iter()
        .map(to_station_round)
        .collect::<Result<Vec<_>>>()?;
    Ok(core::station::Ticket { rounds })
}
/// server 端詞彙比這支 CLI 新——靜默吞掉會渲染出錯誤事實，一律報錯。
fn unknown_ticket_token(kind: &str, token: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "unknown {kind} '{token}' from the server — this CLI is older than the server's ticket format"
    )
}
fn to_station_round(
    r: speclink_protocol::command::ReviewRoundDto,
) -> Result<core::station::Round> {
    let phase = match r.phase.as_deref() {
        None => None,
        Some(token) => Some(
            core::station::RoundPhase::parse(token)
                .ok_or_else(|| unknown_ticket_token("round phase", token))?,
        ),
    };
    let findings = r
        .findings
        .into_iter()
        .map(|f| {
            let severity = core::station::Severity::parse(&f.severity)
                .ok_or_else(|| unknown_ticket_token("finding severity", &f.severity))?;
            Ok(core::station::Finding { severity, path: f.path, text: f.text })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(core::station::Round {
        index: r.index as usize,
        phase,
        patch_hash: r.patch_hash,
        scope: r.scope,
        findings,
    })
}
