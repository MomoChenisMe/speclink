//! Read-only check verbs: validate, analyze, drift.
//!
//! All three are Dual and share one output function per verb (speclink-verbs'
//! `render::checks`): the remote arm converts the wire payload back into the
//! core report type and feeds the same function the fs arm does, so stdout
//! stays byte-identical across modes.

use anyhow::{bail, Result};
use speclink_core as core;

use crate::common::{
    emit, engine_command, info_if_no_changes, open_project, plain_command, render_options, run,
};
use crate::remote_base::{remote_resolve_change, RemoteCtx};
use speclink_verbs::families::checks::{ChangeArg, ValidateArgs};
use speclink_verbs::render::checks as render;
use speclink_verbs::VerbInputs;

pub(crate) fn cmd_validate(a: ValidateArgs) -> Result<()> {
    let (ws, store) = open_project()?;
    let cmd = plain_command(&a)?;
    let v: core::command::ValidateOutcome = run(&store, Some(&ws), cmd)?;
    emit_validate(&v.results, a.json)
}
/// fs 與 remote 共用的 validate 收尾：印出共用的輸出後，任一 invalid 以
/// 「Validation failed.」非零收尾（--json 也一樣）。
fn emit_validate(results: &[core::validate::ValidationResult], json: bool) -> Result<()> {
    emit(render::validate(results, render_options(json)));
    if results.iter().any(|r| !r.valid) {
        bail!("Validation failed.");
    }
    Ok(())
}
pub(crate) fn cmd_analyze(a: ChangeArg) -> Result<()> {
    let (ws, store) = open_project()?;
    if info_if_no_changes(&store, a.change.as_deref()) {
        return Ok(());
    }
    let cmd = engine_command(a.analyze_command(VerbInputs::default()))?;
    let report: core::analyzer::AnalyzeReport = run(&store, Some(&ws), cmd)?;
    emit(render::analyze(&report, render_options(a.json)));
    Ok(())
}
pub(crate) fn cmd_drift(a: ChangeArg) -> Result<()> {
    let (ws, store) = open_project()?;
    if info_if_no_changes(&store, a.change.as_deref()) {
        return Ok(());
    }
    // Client/server drift split, orchestrated locally: the Host collects the
    // workspace facts and fixes the basis; the Engine computes the spec side and
    // the workspace side; the single merger assembles the frozen report shape.
    let change = core::command::resolve_guarded_change(&store, a.change.as_deref())
        .map_err(anyhow::Error::new)?;
    let binding = speclink_host::binding::local_default_binding();
    let bundle = speclink_host::drift::produce_drift_bundle(&store, &change, &binding);
    let facts = speclink_host::drift::collect_workspace_facts(&ws, &store, &change);
    let spec = core::drift::compute_spec_drift(&store, &change);
    let workspace = core::drift::compute_workspace_drift(&store, &change, Some(&facts));
    let basis = core::drift::DriftBasis {
        expected: bundle.basis_digests.clone(),
        current: core::tasks::current_basis_digests(&store, &change.name),
    };
    let report = core::drift::merge_drift_reports(&change, spec, workspace, Some(&basis));
    emit(render::drift(&report, render_options(a.json)));
    Ok(())
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
        // 正式規格沒有 server 端驗證端點，也不新開一個：以既有的規格讀取動詞
        // 取回內容，本地跑 fs 模式同一支驗證器——輸出因此同形。
        let mut caps: Vec<String> =
            ctx.client.list_specs()?.specs.into_iter().map(|s| s.id).collect();
        caps.sort();
        for cap in &caps {
            let content = ctx.client.spec_document(cap)?.content;
            results.push(core::validate::validate_canonical_spec(cap, &content, a.strict));
        }
    }
    emit_validate(&results, a.json)
}
pub(crate) fn remote_analyze(ctx: &RemoteCtx, a: &ChangeArg) -> Result<()> {
    let Some(name) = remote_resolve_change(ctx, a.change.as_deref(), "Specify one:")? else {
        return Ok(());
    };
    let report = speclink_remote::convert::analyze_report(ctx.client.analyze_change(&name)?);
    emit(render::analyze(&report, render_options(a.json)));
    Ok(())
}
/// Remote drift: the Server supplies the spec side, the basis it was computed
/// against, and the change's store-side inputs; the workspace side is collected
/// and computed here off the local checkout; the Engine's one merger assembles
/// the report and the output function fs mode uses renders it — so there is no
/// second merge and no second output shape.
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
        response.change.evidence,
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
    let facts = core::util::git_available(&ctx.ws.root)
        .then(|| speclink_host::drift::collect_workspace_facts(&ctx.ws, &docs, &change));
    let workspace = core::drift::compute_workspace_drift(&docs, &change, facts.as_ref());

    // The spec side and its basis come from one server snapshot, so expected
    // and current are the same fixed point and the report is never stale —
    // matching fs mode, where the bundle and the current digests are read
    // back-to-back off one store.
    let digests = speclink_host::drift::basis_from_wire(&response.basis);
    let basis = core::drift::DriftBasis { expected: digests.clone(), current: digests };
    let report = core::drift::merge_drift_reports(&change, spec, workspace, Some(&basis));
    emit(render::drift(&report, render_options(a.json)));
    Ok(())
}
