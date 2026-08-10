//! Read verbs: list, show, status.
//!
//! Each verb renders through exactly one function that eats the core outcome
//! type; the remote arm converts the wire payload first, so both modes print
//! the same bytes.

use anyhow::{bail, Result};
use clap::Args;
use speclink_core as core;
use speclink_protocol::query as protocol_query;

use crate::color;
use crate::common::{info_if_no_changes, open_project, print_json, run_command};
use crate::remote_base::{remote_resolve_change, RemoteCtx};
use core::listing::ListChangeJson;
use core::store::Store;

#[derive(Args)]
pub(crate) struct ListArgs {
    /// Show only specs
    #[arg(long)]
    specs: bool,
    /// Show only changes
    #[arg(long)]
    changes: bool,
    /// Sort by: name, modified, created
    #[arg(long, default_value = "modified")]
    sort: String,
    /// Output as JSON
    #[arg(long)]
    json: bool,
}
#[derive(Args)]
pub(crate) struct ShowArgs {
    /// Item to show (change or spec name)
    item: Option<String>,
    /// Output as JSON
    #[arg(long)]
    json: bool,
    /// Item type: change, spec
    #[arg(long = "item-type", value_name = "type")]
    item_type: Option<String>,
    /// Show only delta specs
    #[arg(long = "deltas-only")]
    deltas_only: bool,
    /// Show requirements
    #[arg(short = 'r', long)]
    requirements: bool,
}
#[derive(Args)]
pub(crate) struct StatusArgs {
    /// Change name
    #[arg(long)]
    change: Option<String>,
    /// Schema name
    #[arg(long)]
    schema: Option<String>,
    /// Output as JSON
    #[arg(long)]
    json: bool,
}
pub(crate) fn cmd_list(a: ListArgs) -> Result<()> {
    let (ws, fs_store) = open_project()?;
    // Worktree overlay (D3): only a local MAIN checkout with the policy on gets
    // here — everywhere else `facts` is empty, git is never spawned, and both
    // the store and the payload are exactly what they were before this feature.
    let facts = speclink_host::worktree::observed_facts(&ws, &fs_store, |key| std::env::var(key).ok());
    let overlaid = speclink_host::worktree::WorktreeOverlay::new(
        &fs_store,
        facts
            .iter()
            .map(|(name, e)| {
                let store: Box<dyn Store> =
                    Box::new(speclink_fs::FsStore::new(&e.path, &ws.spec_dir_name));
                (name.clone(), store)
            })
            .collect(),
    );
    let store: &dyn Store = if facts.is_empty() { &fs_store } else { &overlaid };
    let outcome = run_command(
        store,
        Some(&ws),
        core::command::Command::List {
            sort: a.sort.clone(),
            specs: a.specs,
            changes: a.changes,
            worktrees: speclink_host::worktree::payload_objects(&facts),
        },
    )?;
    let core::command::CommandOutcome::List(list) = outcome else {
        unreachable!("list command yields a list outcome");
    };
    render_list(&list, a.json)
}
/// fs 與 remote 共用的 list 渲染：兩模式各自組出同一個 ListOutcome，輸出逐位元
/// 一致。模式差異只活在上游的資料組裝——remote 恆無 worktree 事實，其餘欄位同形。
fn render_list(list: &core::command::ListOutcome, json: bool) -> Result<()> {
    // --specs alone omits the changes section (outcome.changes is None).
    let Some(items) = &list.changes else {
        return render_specs_section(list.specs.as_ref().expect("specs requested"), json);
    };
    if json {
        if let Some(specs) = &list.specs {
            let mut payload = serde_json::Map::new();
            payload.insert("changes".into(), serde_json::to_value(items)?);
            payload.insert("specs".into(), specs.clone());
            return print_json(&serde_json::Value::Object(payload));
        }
        return print_json(&serde_json::json!({ "changes": items }));
    }
    if items.is_empty() {
        println!("No active changes.");
        if let Some(specs) = &list.specs {
            println!();
            render_specs_section(specs, false)?;
        }
        return Ok(());
    }
    println!("{}", color::bold("Changes:"));
    for c in items {
        // The progress marker is omitted entirely for changes with zero tasks.
        let marker = if c.total_tasks > 0 {
            format!(" [{}/{}]", c.completed_tasks, c.total_tasks)
        } else {
            String::new()
        };
        // The dim wrapper always prints — an empty summary yields the empty
        // \x1b[2m\x1b[0m pair in color mode and nothing in plain mode.
        let suffix = if c.summary.is_empty() {
            String::new()
        } else {
            format!(" — {}", c.summary)
        };
        // Fail-closed diagnostic (frozen marker text): only corrupt-metadata
        // lines gain the trailing marker; valid lines stay byte-identical.
        let invalid = if c.meta_error.is_some() {
            format!(" {}", color::red("(invalid .openspec.yaml)"))
        } else {
            String::new()
        };
        // Fixed literal, no color: `--no-color` must read exactly the same.
        let worktree = if c.worktree.is_some() { " [worktree]" } else { "" };
        println!(
            "  {} {}{marker}{}{invalid}{worktree}",
            color::cyan("•"),
            c.name,
            color::dim(&suffix)
        );
    }
    if let Some(specs) = &list.specs {
        println!();
        render_specs_section(specs, false)?;
    }
    Ok(())
}
/// The remote list's engine-shaped outcome, assembled from typed wire pieces.
/// The assembly lives on this side of the include because the remote intercept
/// layer speaks protocol DTOs only (tests/it/no_raw_wire_json.rs), while the
/// engine's `ListOutcome` carries its specs section as an opaque payload.
fn remote_list_outcome(
    changes: Option<Vec<ListChangeJson>>,
    specs: Option<Vec<protocol_query::SpecSummary>>,
) -> Result<core::command::ListOutcome> {
    let specs = specs.map(serde_json::to_value).transpose()?;
    Ok(core::command::ListOutcome { changes, specs })
}
/// Render the specs section from the outcome's `{id, path}` items.
fn render_specs_section(specs: &serde_json::Value, json: bool) -> Result<()> {
    if json {
        return print_json(&serde_json::json!({ "specs": specs }));
    }
    let items = specs.as_array().map(Vec::as_slice).unwrap_or_default();
    if items.is_empty() {
        println!("No specs.");
        return Ok(());
    }
    println!("{}", color::bold("Specs:"));
    for s in items {
        println!("  {} {}", color::cyan("•"), s["id"].as_str().unwrap_or_default());
    }
    Ok(())
}
pub(crate) fn cmd_show(a: ShowArgs) -> Result<()> {
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    let outcome = run_command(
        store,
        Some(&ws),
        core::command::Command::Show {
            item: a.item.clone(),
            item_type: a.item_type.clone(),
        },
    )?;
    let core::command::CommandOutcome::Show(show) = outcome else {
        unreachable!("show command yields a show outcome");
    };
    render_show(show, a.json)
}
/// fs 與 remote 共用的 show 渲染：兩模式餵進同一個 ShowOutcome，輸出逐位元一致。
fn render_show(show: core::command::ShowOutcome, json: bool) -> Result<()> {
    match show {
        core::command::ShowOutcome::Spec { name, content } => {
            if json {
                return print_json(&serde_json::json!({
                    "files": [{ "content": content, "name": "spec.md" }],
                    "name": name,
                }));
            }
            println!("{}: {name}", color::bold("Spec"));
            println!();
            println!("{}", color::dim("--- spec.md ---"));
            print!("{content}");
            println!(); // A trailing newline is always emitted (an extra blank when content ends with \n)
        }
        core::command::ShowOutcome::Change(c) => {
            let caps: Vec<String> = c
                .delta_capabilities
                .iter()
                .map(|cap| format!("{cap}/spec.md"))
                .collect();
            if json {
                return print_json(&serde_json::json!({
                    "name": c.name,
                    "schema": c.schema,
                    "created": c.created,
                    "proposal": c.proposal,
                    "design": c.design,
                    "tasks": c.tasks,
                    "deltaSpecs": caps,
                    "fromDiscussions": c.from_discussions,
                    "restaleFrom": c.restale_from,
                }));
            }
            println!("{}: {}", color::bold("Change"), c.name);
            if let Some(schema_name) = &c.schema {
                println!("{}: {schema_name}", color::bold("Schema"));
            }
            if let Some(created) = &c.created {
                println!("{}: {created}", color::bold("Created"));
            }
            // The header's trailing blank line prints only when a section follows.
            if c.proposal.is_some() || !caps.is_empty() {
                println!();
            }
            // The Proposal section renders whenever the FILE exists (even empty) — frozen behavior.
            if let Some(proposal) = &c.proposal {
                println!("{}", color::dim("--- Proposal ---"));
                print!("{proposal}");
                if !caps.is_empty() {
                    // The proposal's own trailing newline determines the blank-line count before the header.
                    print!("\n\n{}\n", color::dim("--- Delta Specs ---"));
                    for cap in &caps {
                        println!("  {cap}");
                    }
                } else {
                    // A newline is always appended after the proposal body (an extra blank
                    // line when the file already ends with one), same as the spec branch above.
                    println!();
                }
            } else if !caps.is_empty() {
                // No proposal, but delta specs exist — still render the section.
                println!("{}", color::dim("--- Delta Specs ---"));
                for cap in &caps {
                    println!("  {cap}");
                }
            }
        }
    }
    Ok(())
}
pub(crate) fn cmd_status(a: StatusArgs) -> Result<()> {
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    if info_if_no_changes(store, a.change.as_deref()) {
        return Ok(());
    }
    let outcome = run_command(
        store,
        Some(&ws),
        core::command::Command::Status {
            change: a.change.clone(),
            schema: a.schema.clone(),
        },
    )?;
    let core::command::CommandOutcome::Status(report) = outcome else {
        unreachable!("status command yields a status outcome");
    };
    if a.json {
        return print_json(&report);
    }
    render_status_human(&report);
    Ok(())
}
/// Human status rendering — shared by the fs and remote paths so both modes
/// stay byte-identical.
fn render_status_human(report: &core::status::StatusReport) {
    println!("{}: {}", color::bold("Change"), report.change_name);
    println!("{}: {}", color::bold("Schema"), report.schema_name);
    println!();
    for art in &report.artifacts {
        let sym = match art.status.as_str() {
            "done" => color::green("✓"),
            "ready" => color::yellow("○"),
            _ => color::red("✗"),
        };
        println!(
            "  {sym} {} ({})",
            color::bold(&art.id),
            color::dim(&art.output_path)
        );
        if art.status == "blocked" && !art.blocked_by.is_empty() {
            println!("    blocked by: {}", art.blocked_by.join(", "));
        }
    }
    println!();
    if report.is_complete {
        println!("  {} All artifacts complete", color::green("✓"));
    }
}
/// The wire summary reshaped into the core listing item — the same rendering
/// path then keeps stdout byte-identical to fs mode.
fn to_list_change_json(c: &protocol_query::ChangeSummary) -> ListChangeJson {
    ListChangeJson {
        completed_tasks: c.completed_tasks,
        name: c.name.clone(),
        status: c.status.clone(),
        summary: c.summary.clone(),
        total_tasks: c.total_tasks,
        restale_from: c.restale_from.clone(),
        meta_error: c.meta_error.clone(),
        // remote 恆缺席：worktree 是本機主 checkout 的觀察面，
        // server 端沒有這回事（spec scenario remote list 恆無 worktree 欄位）。
        worktree: None,
    }
}
pub(crate) fn remote_list(ctx: &RemoteCtx, a: &ListArgs) -> Result<()> {
    // 組出 fs 模式同一個 ListOutcome 再交給共用渲染：`--specs` 單獨給定時
    // changes 為 None，與引擎的 List outcome 同構。
    let specs_only = a.specs && !a.changes;
    let changes = if specs_only {
        None
    } else {
        let mut items: Vec<ListChangeJson> =
            ctx.client.list_changes()?.changes.iter().map(to_list_change_json).collect();
        // `--sort name` 在本地重排對齊 fs；`created`／`modified` 的排序鍵
        // （meta created、store mtime）不在 wire 上，維持 server 回傳序。
        if a.sort == "name" {
            items.sort_by(|x, y| x.name.cmp(&y.name));
        }
        Some(items)
    };
    let specs = if a.specs { Some(ctx.client.list_specs()?.specs) } else { None };
    render_list(&remote_list_outcome(changes, specs)?, a.json)
}
/// The wire status reshaped into the fs report type — the same rendering
/// path keeps stdout byte-identical to fs mode; server extras (repo,
/// lifecycle, versions) simply don't cross this boundary.
fn to_status_report(status: protocol_query::ChangeStatus) -> core::status::StatusReport {
    core::status::StatusReport {
        change_name: status.change_name,
        schema_name: status.schema_name,
        is_complete: status.is_complete,
        apply_requires: status.apply_requires,
        artifacts: status
            .artifacts
            .into_iter()
            .map(|a| core::status::ArtifactStatusJson {
                id: a.id,
                output_path: a.output_path,
                status: a.status,
                blocked_by: a.missing_deps,
            })
            .collect(),
    }
}
pub(crate) fn remote_status(ctx: &RemoteCtx, a: &StatusArgs) -> Result<()> {
    if a.schema.is_some() {
        bail!("--schema is not supported in remote mode — the server's workflow config decides the schema");
    }
    let Some(name) =
        remote_resolve_change(ctx, a.change.as_deref(), "Use --change to specify one:")?
    else {
        return Ok(());
    };
    let report = to_status_report(ctx.client.get_change(&name)?);
    if a.json {
        return print_json(&report);
    }
    render_status_human(&report);
    Ok(())
}
/// remote show 的讀取組合（design D4）：以既有讀 API 組出與 fs 模式相同的
/// ShowOutcome，交給共用的 render_show 渲染。item 與 --item-type 的判別序
/// 對齊引擎 run_show：type==spec 直取規格；否則 change 優先、缺席時規格
/// 遞補（type==change 不遞補）；錯誤訊息沿引擎凍結文本。
fn remote_show_outcome(
    ctx: &RemoteCtx,
    item: Option<&str>,
    item_type: Option<&str>,
) -> Result<core::command::ShowOutcome> {
    let Some(item) = item else {
        bail!("Please specify an item name.");
    };
    if let Some(t) = item_type {
        if t != "change" && t != "spec" {
            bail!("Unknown type: {t}. Use 'change' or 'spec'.");
        }
    }
    let is_spec = ctx.client.list_specs()?.specs.iter().any(|s| s.id == item);
    let status = if item_type == Some("spec") {
        None
    } else {
        match ctx.client.get_change(item) {
            Ok(status) => Some(status),
            Err(e) if e.status == Some(404) => None,
            Err(e) => return Err(e.into()),
        }
    };
    let show_spec =
        item_type == Some("spec") || (item_type != Some("change") && status.is_none() && is_spec);
    if show_spec {
        if !is_spec {
            bail!("Spec '{item}' not found.");
        }
        let content = ctx.client.spec_document(item)?.content;
        return Ok(core::command::ShowOutcome::Spec { name: item.to_string(), content });
    }
    let Some(status) = status else {
        if item_type == Some("change") {
            bail!("Change '{item}' not found.");
        }
        bail!("Item '{item}' not found as a change or spec.");
    };
    // 成對規則由 server 套用：created 出現即代表 meta 的 schema+created 成對。
    let (schema, created) = match status.created {
        Some(created) => (Some(status.schema_name.clone()), Some(created)),
        None => (None, None),
    };
    let read_artifact = |artifact: &str| -> Result<Option<String>> {
        match ctx.client.get_artifact(item, artifact) {
            Ok(content) => Ok(Some(content.content)),
            Err(e) if e.status == Some(404) => Ok(None),
            Err(e) => Err(e.into()),
        }
    };
    // restaleFrom 走清單摘要（設計上 show 組合的「兩份清單」之一）。
    let restale_from = ctx
        .client
        .list_changes()?
        .changes
        .into_iter()
        .find(|c| c.name == item)
        .map(|c| c.restale_from)
        .unwrap_or_default();
    Ok(core::command::ShowOutcome::Change(core::command::ShowChange {
        name: status.change_name,
        schema,
        created,
        proposal: read_artifact("proposal")?,
        design: read_artifact("design")?,
        tasks: read_artifact("tasks")?,
        delta_capabilities: status.delta_capabilities,
        from_discussions: status.from_discussions,
        restale_from,
    }))
}

/// show 的 remote 家族臂：wire → ShowOutcome → 與 fs 同一支渲染。臂在族內
/// 具名，`render_show` 與 `remote_show_outcome` 因此不必離開本檔。
pub(crate) fn remote_show(ctx: &RemoteCtx, a: ShowArgs) -> Result<()> {
    render_show(remote_show_outcome(ctx, a.item.as_deref(), a.item_type.as_deref())?, a.json)
}
