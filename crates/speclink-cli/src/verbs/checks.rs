//! Read-only check verbs: validate, analyze, drift.
//!
//! All three are Dual and share one renderer per verb: the remote arm converts
//! the wire payload back into the core report type and feeds the same function
//! the fs arm does, so stdout stays byte-identical across modes.

use anyhow::{bail, Result};
use clap::Args;
use speclink_core as core;

use crate::color;
use crate::common::{info_if_no_changes, open_project, print_json, run_command};
use crate::remote_base::{remote_resolve_change, RemoteCtx};
use core::store::Store;

#[derive(Args)]
pub(crate) struct ValidateArgs {
    /// Item to validate
    item: Option<String>,
    /// Validate all items
    #[arg(long)]
    all: bool,
    /// Validate only changes
    #[arg(long)]
    changes: bool,
    /// Validate only specs
    #[arg(long)]
    specs: bool,
    /// Strict mode
    #[arg(long)]
    strict: bool,
    /// Output as JSON
    #[arg(long)]
    json: bool,
}
#[derive(Args)]
pub(crate) struct ChangeArg {
    /// Change name (auto-detects if only one exists)
    change: Option<String>,
    /// Output as JSON
    #[arg(long)]
    json: bool,
}
pub(crate) fn cmd_validate(a: ValidateArgs) -> Result<()> {
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    let outcome = run_command(
        store,
        Some(&ws),
        core::command::Command::Validate {
            item: a.item.clone(),
            all: a.all,
            changes: a.changes,
            specs: a.specs,
            strict: a.strict,
        },
    )?;
    let core::command::CommandOutcome::Validate(v) = outcome else {
        unreachable!("validate command yields a validate outcome");
    };
    render_validate_results(&v.results, a.json)
}
/// fs 與 remote 共用的 validate 渲染：--json 印完整結果陣列、人眼逐 change
/// 列 error/warn；任一 invalid 以「Validation failed.」非零收尾。
fn render_validate_results(results: &[core::validate::ValidationResult], json: bool) -> Result<()> {
    let any_invalid = results.iter().any(|r| !r.valid);
    if json {
        print_json(&results)?;
        if any_invalid {
            bail!("Validation failed.");
        }
        return Ok(());
    }
    for r in results {
        if r.valid {
            println!("{} {} — valid", color::green("✓"), r.change);
        } else {
            println!("{} {} — invalid", color::red("✗"), r.change);
            for e in &r.errors {
                println!("  {} {e}", color::red("error:"));
            }
        }
        for w in &r.warnings {
            println!("  {} {w}", color::yellow("warn:"));
        }
    }
    if any_invalid {
        bail!("Validation failed.");
    }
    Ok(())
}
pub(crate) fn cmd_analyze(a: ChangeArg) -> Result<()> {
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    if info_if_no_changes(store, a.change.as_deref()) {
        return Ok(());
    }
    let outcome = run_command(
        store,
        Some(&ws),
        core::command::Command::Analyze { change: a.change.clone() },
    )?;
    let core::command::CommandOutcome::Analyze(report) = outcome else {
        unreachable!("analyze command yields an analyze outcome");
    };
    if a.json {
        return print_json(&report);
    }
    render_analyze(&report);
    Ok(())
}
fn render_analyze(report: &core::analyzer::AnalyzeReport) {
    println!("{}: {}", color::bold("Change"), report.change_id);
    println!();
    for d in &report.dimensions {
        let sym = if d.finding_count == 0 {
            color::green("✓")
        } else {
            color::yellow("●")
        };
        // The bold span covers a 14-wide padded name; the 15th column separator
        // space stays outside it (plain bytes are identical to the old {:<15} form).
        println!(
            "  {sym} {} {} ({} findings)",
            color::bold(&format!("{:<14}", d.dimension)),
            color::dim(&d.status),
            d.finding_count
        );
    }
    // The blank separator is tied to the "Analyzed:" line; an empty change (nothing analyzed)
    // prints "Missing:" directly after the dimensions (frozen output shape).
    if !report.artifacts_analyzed.is_empty() {
        println!();
        println!("  {} {}", color::dim("Analyzed:"), report.artifacts_analyzed.join(", "));
    }
    if !report.artifacts_missing.is_empty() {
        println!("  {} {}", color::yellow("Missing:"), report.artifacts_missing.join(", "));
    }
    if report.findings.is_empty() {
        println!();
        println!("  {} No issues found", color::green("✓"));
        return;
    }
    println!();
    println!("  {} ({}):", color::bold("Findings"), report.findings.len());
    println!();
    for f in &report.findings {
        let tag = match f.severity.as_str() {
            "Critical" => color::bold_red("CRITICAL"),
            "Warning" => color::yellow("WARNING"),
            _ => color::dim("SUGGEST"),
        };
        println!("  [{tag}] {}", f.summary);
        println!("    {} {}", color::dim("at:"), color::dim(&f.location));
        println!("    {} {}", color::dim("→"), color::dim(&f.recommendation));
    }
}
pub(crate) fn cmd_drift(a: ChangeArg) -> Result<()> {
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    if info_if_no_changes(store, a.change.as_deref()) {
        return Ok(());
    }
    // Client/server drift split, orchestrated locally: the Host collects the
    // workspace facts and fixes the basis; the Engine computes the spec side and
    // the workspace side; the single merger assembles the frozen report shape.
    let change = core::command::resolve_guarded_change(store, a.change.as_deref())
        .map_err(anyhow::Error::new)?;
    let binding = speclink_host::binding::local_default_binding();
    let bundle = speclink_host::drift::produce_drift_bundle(store, &ws, &change, &binding);
    let facts = speclink_host::drift::collect_workspace_facts(&ws, store, &change);
    let spec = core::drift::compute_spec_drift(store, &change);
    let workspace = core::drift::compute_workspace_drift(store, &change, Some(&facts));
    let basis = core::drift::DriftBasis {
        expected: bundle.basis_digests.clone(),
        current: core::tasks::current_basis_digests(store, &change.name),
    };
    let report = core::drift::merge_drift_reports(&change, spec, workspace, Some(&basis));
    if a.json {
        return print_json(&report);
    }
    render_drift(&report.report);
    Ok(())
}
fn render_drift(report: &core::drift::DriftReport) {
    println!("{}: {}", color::bold("Drift Report"), report.change_id);
    if let Some(created) = &report.created {
        println!("  Created: {created}");
    }
    println!();
    // Bold spans cover 11/35-wide padded cells with the separator spaces outside, plus
    // a leading-space " Score" cell — plain bytes are identical to the old format.
    println!(
        "  {} {} {}",
        color::bold(&format!("{:<11}", "Dimension")),
        color::bold(&format!("{:<35}", "Status")),
        color::bold(" Score")
    );
    for d in &report.dimensions {
        let score = if d.contributes_to_total {
            format!("+{}", d.score)
        } else {
            "—".to_string()
        };
        println!("  {:<11} {:<36} {:>5}", d.kind, d.status, score);
    }
    println!(
        "  {} {} {}",
        color::bold(&format!("{:<11}", "Total")),
        " ".repeat(35),
        color::bold(&format!("{:>6}", report.total_score))
    );
    if !report.broken_anchors.is_empty() {
        println!();
        println!("Broken anchors");
        for b in &report.broken_anchors {
            println!("  - {} ({}) — {}", b.anchor, b.category, b.reason);
        }
    }
    if !report.spec_assumptions.is_empty() {
        println!();
        println!("Stale delta assumptions");
        for a in &report.spec_assumptions {
            println!(
                "  - {} '{}' in {} — {}",
                a.operation, a.requirement, a.capability, a.reason
            );
        }
    }
    if !report.tasks_maybe_resolved.is_empty() {
        println!();
        println!("Tasks maybe already done");
        for t in &report.tasks_maybe_resolved {
            println!("  - {t}");
        }
    }
    if !report.tasks_blocked_external.is_empty() {
        println!();
        println!("Tasks blocked by external changes");
        for t in &report.tasks_blocked_external {
            println!("  - {t}");
        }
    }
    println!();
    let sev = report.severity.to_uppercase();
    let sev_colored = match sev.as_str() {
        "LIGHT" => color::bold_green(&sev),
        "MEDIUM" => color::bold_yellow(&sev),
        _ => color::bold_red(&sev),
    };
    println!("{}: {} drift", color::bold("Severity"), sev_colored);
    println!("> {}", color::bold_cyan(&report.primary_recommendation));
}
/// 聚合語意（無參數／--all／--changes）由 client 組合：先 list 再逐 change 打
/// 單 change 端點（design 決策 2）；DTO 轉回本地型別後走 fs 同一渲染（決策 6）。
pub(crate) fn remote_validate(ctx: &RemoteCtx, a: &ValidateArgs) -> Result<()> {
    // 目標集的旗標語意由引擎單一定義，fs 與 remote 兩條路徑共用（design D4）；
    // --specs 與 item 同傳的拒絕措辭也因此兩模式同字。
    let targets = core::validate::validate_targets(a.item.as_deref(), a.all, a.changes, a.specs)
        .map_err(anyhow::Error::msg)?;
    let mut results: Vec<core::validate::ValidationResult> = Vec::new();
    if targets.changes {
        let names: Vec<String> = if let Some(item) = &a.item {
            vec![item.clone()]
        } else {
            // 無參數與 --all/--changes 的目標集在 remote 皆為 scope 的全部
            // changes（fs 的「恰一個 active change 單驗」是同集合的特例）。
            ctx.client.list_changes()?.changes.into_iter().map(|c| c.name).collect()
        };
        for n in &names {
            results.push(speclink_remote::convert::validation_result(
                ctx.client.validate_change(n)?,
            ));
        }
    }
    if targets.specs {
        // 正典規格沒有 server 端驗證端點，也不新開一個：以既有的規格讀取動詞
        // 取回內容，本地跑 fs 模式同一支驗證器——輸出因此同形。
        let mut caps: Vec<String> =
            ctx.client.list_specs()?.specs.into_iter().map(|s| s.id).collect();
        caps.sort();
        for cap in &caps {
            let content = ctx.client.spec_document(cap)?.content;
            results.push(core::validate::validate_canonical_spec(cap, &content, a.strict));
        }
    }
    render_validate_results(&results, a.json)
}
pub(crate) fn remote_analyze(ctx: &RemoteCtx, a: &ChangeArg) -> Result<()> {
    let Some(name) = remote_resolve_change(ctx, a.change.as_deref(), "Specify one:")? else {
        return Ok(());
    };
    let report = speclink_remote::convert::analyze_report(ctx.client.analyze_change(&name)?);
    if a.json {
        return print_json(&report);
    }
    render_analyze(&report);
    Ok(())
}
/// Remote drift: the Server supplies the spec side, the basis it was computed
/// against, and the change's store-side inputs; the workspace side is collected
/// and computed here off the local checkout; the Engine's one merger assembles
/// the report and the renderer fs mode uses prints it — so there is no second
/// merge and no second output shape.
pub(crate) fn remote_drift(ctx: &RemoteCtx, a: &ChangeArg) -> Result<()> {
    // Positional-style wording, matching the fs verb's auto-detect error.
    let Some(name) = remote_resolve_change(ctx, a.change.as_deref(), "Specify one:")? else {
        return Ok(());
    };
    // A server failure stops here: no report is rendered from half the facts.
    let response = ctx.client.spec_drift(&name)?;

    let docs = speclink_host::drift::RemoteDriftStore::new(
        &name,
        response.change.created,
        response.change.design,
        response.change.tasks,
    );
    let change = docs.change();
    let spec = speclink_host::drift::spec_drift_from_wire(&response.spec_drift);

    // The workspace side needs a checkout. Remote mode always has a workspace
    // (the .speclink.yaml the mode was resolved from), so git availability is
    // what actually distinguishes "there is code here" from "there is not".
    // Without it the facts are absent — which the Engine reports as
    // unavailable. Collecting anyway would stat the anchors against a
    // codeless directory and report every one of them broken: an absent
    // checkout would read as deleted code.
    let ws = core::workspace::Workspace::discover_cwd()?
        .filter(|w| core::util::git_available(&w.root));
    let facts = ws
        .as_ref()
        .map(|w| speclink_host::drift::collect_workspace_facts(w, &docs, &change));
    let workspace = core::drift::compute_workspace_drift(&docs, &change, facts.as_ref());

    // The spec side and its basis come from one server snapshot, so expected
    // and current are the same fixed point and the report is never stale —
    // matching fs mode, where the bundle and the current digests are read
    // back-to-back off one store.
    let digests = speclink_host::drift::basis_from_wire(&response.basis);
    let basis = core::drift::DriftBasis { expected: digests.clone(), current: digests };
    let report = core::drift::merge_drift_reports(&change, spec, workspace, Some(&basis));
    if a.json {
        return print_json(&report);
    }
    render_drift(&report.report);
    Ok(())
}
