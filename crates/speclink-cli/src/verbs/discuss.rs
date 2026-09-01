//! The discuss family: every discussion subcommand, both arms.
//!
//! Dual at the family level — the subcommand enum is matched exhaustively with
//! no catch-all in either arm, so a new subcommand fails to build until both
//! modes answer for it.

use anyhow::Result;
use clap::{Args, Subcommand};
use speclink_core as core;
use speclink_protocol::query as protocol_query;

use crate::color;
use crate::common::{open_project, print_json, read_stdin_content, run_command};
use crate::remote_base::RemoteCtx;
use core::store::Store;

#[derive(Args)]
pub(crate) struct DiscussArgs {
    #[command(subcommand)]
    command: DiscussCommands,
}
#[derive(Subcommand)]
enum DiscussCommands {
    /// Create a new discussion document
    New {
        topic: String,
        /// Override the record slug (ASCII kebab-case); the topic stays verbatim
        #[arg(long)]
        slug: Option<String>,
        /// Mark the record's type (only: improve)
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// List discussions
    List {
        /// Show archived discussions instead of live ones
        #[arg(long)]
        archived: bool,
        #[arg(long)]
        json: bool,
    },
    /// Show a discussion document
    Show { slug: String, #[arg(long)] json: bool },
    /// Set the discussion's Context section (content from stdin)
    Context { slug: String, #[arg(long)] stdin: bool, #[arg(long)] json: bool },
    /// Append a round to a discussion (content from stdin)
    #[command(name = "add-round")]
    AddRound { slug: String, #[arg(long, default_value = "interview")] mode: String, #[arg(long)] stdin: bool, #[arg(long)] json: bool },
    /// Conclude a discussion (content from stdin)
    Conclude { slug: String, #[arg(long)] stdin: bool, #[arg(long)] json: bool },
    /// Archive a discussion (move to discussions/archive/<created>-<slug>.md)
    Archive { slug: String, #[arg(long)] json: bool },
    /// Discard a live discussion (delete the file; --force required once rounds exist)
    Discard {
        slug: String,
        /// Delete even when the discussion has recorded rounds
        #[arg(long)]
        force: bool,
        #[arg(long)]
        json: bool,
    },
    /// Promote a discussion into a change scaffold (proposal prefilled from the conclusion)
    Promote {
        slug: String,
        /// Change name (defaults to the discussion slug)
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Link a discussion to an existing change (forges the from_discussion chain)
    Link {
        slug: String,
        /// Existing change name to link the discussion to
        change: String,
        #[arg(long)]
        json: bool,
    },
    /// Seal a discussion→change reflection: mark the discussion promoted once content has landed
    Seal {
        slug: String,
        /// Change whose from_discussion already includes this discussion
        change: String,
        #[arg(long)]
        json: bool,
    },
}
pub(crate) fn cmd_discuss(a: DiscussArgs) -> Result<()> {
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    match a.command {
        DiscussCommands::New { topic, slug, kind, json } => {
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::DiscussNew { topic, slug, kind },
            )?;
            let core::command::CommandOutcome::DiscussNew(info) = outcome else {
                unreachable!("discuss new yields a discussion info");
            };
            if json {
                return print_json(&info);
            }
            render_discuss_new_human(NewDiscussionLines {
                slug: &info.slug,
                topic: &info.topic,
                path: &info.path,
            });
        }
        DiscussCommands::List { archived, json } => {
            let outcome =
                run_command(store, Some(&ws), core::command::Command::DiscussList { archived })?;
            let core::command::CommandOutcome::DiscussList(items) = outcome else {
                unreachable!("discuss list yields a discussion list");
            };
            render_discuss_list(&items, archived, json)?;
        }
        DiscussCommands::Show { slug, json } => {
            let outcome =
                run_command(store, Some(&ws), core::command::Command::DiscussShow { slug })?;
            let core::command::CommandOutcome::DiscussShow(show) = outcome else {
                unreachable!("discuss show yields a discussion document");
            };
            render_discuss_show(&show, json)?;
        }
        DiscussCommands::Context { slug, stdin, json } => {
            let content = read_stdin_content(stdin);
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::DiscussContext { slug, content },
            )?;
            let core::command::CommandOutcome::DiscussContext(o) = outcome else {
                unreachable!("discuss context yields a subject outcome");
            };
            render_discuss_context(&o.slug, json)?;
        }
        DiscussCommands::AddRound { slug, mode, stdin, json } => {
            let content = read_stdin_content(stdin);
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::DiscussAddRound { slug, mode, content },
            )?;
            let core::command::CommandOutcome::DiscussAddRound(o) = outcome else {
                unreachable!("discuss add-round yields a round outcome");
            };
            render_discuss_add_round(&o.slug, o.round, &o.mode, json)?;
        }
        DiscussCommands::Conclude { slug, stdin, json } => {
            let content = read_stdin_content(stdin);
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::DiscussConclude { slug, content },
            )?;
            let core::command::CommandOutcome::DiscussConclude(o) = outcome else {
                unreachable!("discuss conclude yields a conclude outcome");
            };
            render_discuss_conclude(&o.slug, &o.restale_flagged, o.auto_archived, json)?;
            // 閉環封存步失敗：結論與 restale 已落盤（上面照常呈現），這裡以非零
            // exit code 收場、stderr 說明原因；重跑 discuss archive 即可收尾。
            if let Some(reason) = &o.closing_error {
                anyhow::bail!("conclude closing step failed to archive the record: {reason}");
            }
        }
        DiscussCommands::Archive { slug, json } => {
            let outcome =
                run_command(store, Some(&ws), core::command::Command::DiscussArchive { slug })?;
            let core::command::CommandOutcome::DiscussArchive(o) = outcome else {
                unreachable!("discuss archive yields an archive outcome");
            };
            render_discuss_archive(
                &o.slug,
                &format!("discussions/archive/{}", o.archived_file),
                json,
            )?;
        }
        DiscussCommands::Discard { slug, force, json } => {
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::DiscussDiscard { slug, force },
            )?;
            let core::command::CommandOutcome::DiscussDiscard(o) = outcome else {
                unreachable!("discuss discard yields a subject outcome");
            };
            render_discuss_discard(&o.slug, json)?;
        }
        DiscussCommands::Promote { slug, name, json } => {
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::DiscussPromote { slug, name },
            )?;
            let core::command::CommandOutcome::DiscussPromote(o) = outcome else {
                unreachable!("discuss promote yields a promote outcome");
            };
            let shown = o.path.to_string_lossy();
            let wire = core::util::to_slash(&o.path);
            render_discuss_promote(
                &o.slug,
                &o.change,
                Some(PromotedPath { shown: &shown, wire: &wire }),
                json,
            )?;
        }
        DiscussCommands::Link { slug, change, json } => {
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::DiscussLink { slug, change },
            )?;
            let core::command::CommandOutcome::DiscussLink(o) = outcome else {
                unreachable!("discuss link yields a bind outcome");
            };
            render_discuss_bind(&o.slug, &o.change, DiscussBind::Link, json)?;
        }
        DiscussCommands::Seal { slug, change, json } => {
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::DiscussSeal { slug, change },
            )?;
            let core::command::CommandOutcome::DiscussSeal(o) = outcome else {
                unreachable!("discuss seal yields a bind outcome");
            };
            render_discuss_bind(&o.slug, &o.change, DiscussBind::Seal, json)?;
        }
    }
    Ok(())
}
/// `link` 與 `seal` 的輸出只差動詞與尾註，共用一支渲染。
#[derive(Clone, Copy)]
enum DiscussBind {
    Link,
    Seal,
}
impl DiscussBind {
    /// 一次給齊三段文字：--json 的 status、成功行動詞、尾註——單一 match。
    fn parts(self) -> (&'static str, &'static str, &'static str) {
        match self {
            DiscussBind::Link => ("linked", "Linked", ""),
            DiscussBind::Seal => ("sealed", "Sealed", " (marked promoted)"),
        }
    }
}
/// 人眼三行是兩模式共用的文本；`--json` 不進這裡——兩模式各有既定形狀
/// （fs 印完整 DiscussionInfo、remote 印 wire 回應原樣），組另一邊的型別
/// 會捏造來源沒說的欄位。三行具名綁定，杜絕相鄰同型參數寫反。
struct NewDiscussionLines<'a> {
    slug: &'a str,
    topic: &'a str,
    path: &'a str,
}
fn render_discuss_new_human(lines: NewDiscussionLines<'_>) {
    println!("{} Created discussion: {}", color::green("✓"), lines.slug);
    println!("  Topic: {}", lines.topic);
    println!("  Path: {}", lines.path);
}
fn render_discuss_list(
    items: &[core::discuss::DiscussionInfo],
    archived: bool,
    json: bool,
) -> Result<()> {
    if json {
        return print_json(&serde_json::json!({ "discussions": items }));
    }
    if items.is_empty() {
        let what = if archived { "archived discussions" } else { "discussions" };
        println!("No {what} found.");
        return Ok(());
    }
    let heading = if archived { "Archived discussions:" } else { "Discussions:" };
    println!("{heading}");
    for d in items {
        println!("  • {} [{}] ({} rounds) — {}", d.slug, d.status, d.rounds, d.topic);
    }
    Ok(())
}
fn render_discuss_show(show: &core::command::DiscussShowOutcome, json: bool) -> Result<()> {
    if json {
        return print_json(&serde_json::json!({ "info": show.info, "content": show.content }));
    }
    print!("{}", show.content);
    Ok(())
}
fn render_discuss_context(slug: &str, json: bool) -> Result<()> {
    if json {
        return print_json(&serde_json::json!({ "slug": slug, "context": "set" }));
    }
    println!("{} Set context for discussion '{slug}'", color::green("✓"));
    Ok(())
}
fn render_discuss_add_round(slug: &str, round: usize, mode: &str, json: bool) -> Result<()> {
    if json {
        return print_json(&serde_json::json!({ "slug": slug, "round": round, "mode": mode }));
    }
    println!("{} Recorded round {round} ({mode}) to discussion '{slug}'", color::green("✓"));
    Ok(())
}
fn render_discuss_conclude(
    slug: &str,
    flagged: &[String],
    auto_archived: bool,
    json: bool,
) -> Result<()> {
    if json {
        // Byte-identical to before when nothing was flagged (promoted_to empty);
        // the array appears only when a re-conclude actually staled changes, and
        // autoArchived only when the closing step archived the record.
        let mut payload = serde_json::json!({ "slug": slug, "status": "concluded" });
        if !flagged.is_empty() {
            payload["restaleFlagged"] = serde_json::json!(flagged);
        }
        if auto_archived {
            payload["autoArchived"] = serde_json::json!(true);
        }
        return print_json(&payload);
    }
    println!("{} Concluded discussion '{slug}'", color::green("✓"));
    if !flagged.is_empty() {
        println!("  Flagged {} change(s) for re-ingest: {}", flagged.len(), flagged.join(", "));
    }
    if auto_archived {
        println!("  Auto-archived the record (all promoted changes are archived)");
    }
    Ok(())
}
fn render_discuss_archive(slug: &str, archived_to: &str, json: bool) -> Result<()> {
    if json {
        return print_json(&serde_json::json!({ "slug": slug, "archived_to": archived_to }));
    }
    println!("{} Archived discussion: {slug} → {archived_to}", color::green("✓"));
    Ok(())
}
fn render_discuss_discard(slug: &str, json: bool) -> Result<()> {
    if json {
        return print_json(&serde_json::json!({ "slug": slug, "status": "discarded" }));
    }
    println!("{} Discarded discussion: {slug}", color::green("✓"));
    Ok(())
}
/// The new change's directory in both spellings: the human line keeps the
/// platform separators the fs path was built with, while `--json` (and the
/// wire) is always slashed. On Unix the two are identical; on Windows they
/// are not, and the frozen fs output depends on the difference.
struct PromotedPath<'a> {
    shown: &'a str,
    wire: &'a str,
}
/// `path` absent is the remote mode's declared divergence (design D5 #5): the
/// new change's directory is a store-side location with no meaning on the
/// caller's machine, so the Path line AND the follow-up hint after it are both
/// dropped together — the two lines share one fate.
fn render_discuss_promote(
    slug: &str,
    change: &str,
    path: Option<PromotedPath<'_>>,
    json: bool,
) -> Result<()> {
    if json {
        let mut payload = serde_json::json!({
            "change": change,
            "slug": slug,
            "status": "promoted",
        });
        if let Some(path) = &path {
            payload["path"] = serde_json::json!(path.wire);
        }
        return print_json(&payload);
    }
    println!("{} Promoted discussion '{slug}' → change '{change}'", color::green("✓"));
    if let Some(path) = &path {
        println!("  Path: {}", path.shown);
        println!("  Proposal prefilled from the conclusion — run /speclink-propose to complete the artifacts");
    }
    Ok(())
}
fn render_discuss_bind(slug: &str, change: &str, bind: DiscussBind, json: bool) -> Result<()> {
    let (status, verb, suffix) = bind.parts();
    if json {
        return print_json(&serde_json::json!({
            "change": change,
            "slug": slug,
            "status": status,
        }));
    }
    println!("{} {verb} discussion '{slug}' → change '{change}'{suffix}", color::green("✓"));
    Ok(())
}
pub(crate) fn remote_discuss(ctx: &RemoteCtx, a: DiscussArgs) -> Result<()> {
    match a.command {
        DiscussCommands::List { archived, json } => {
            let items: Vec<_> = ctx
                .client
                .list_discussions(archived)?
                .discussions
                .iter()
                .map(to_discussion_info)
                .collect();
            render_discuss_list(&items, archived, json)
        }
        DiscussCommands::Show { slug, json } => {
            let payload = ctx.client.show_discussion(&slug)?;
            render_discuss_show(
                &core::command::DiscussShowOutcome {
                    info: Some(to_discussion_info(&payload.info)),
                    content: payload.content,
                },
                json,
            )
        }
        DiscussCommands::New { topic, slug, kind, json } => {
            // --slug 與 --kind 隨請求上 wire；驗證的單一事實來源在引擎（server 端），
            // CLI 不預驗（design D1）。
            let resp = ctx.client.new_discussion(&topic, slug.as_deref(), kind.as_deref())?;
            if json {
                // remote 的 --json 契約是 wire 回應原樣（slug／topic／path）——
                // 組 core 型別會捏造 server 沒說的欄位，形狀凍結不允許。
                return print_json(&resp);
            }
            render_discuss_new_human(NewDiscussionLines {
                slug: &resp.slug,
                topic: &resp.topic,
                path: &resp.path,
            });
            Ok(())
        }
        DiscussCommands::Context { slug, stdin, json } => {
            let content = read_stdin_content(stdin);
            ctx.client.discussion_context(&slug, &content)?;
            render_discuss_context(&slug, json)
        }
        DiscussCommands::AddRound { slug, mode, stdin, json } => {
            let content = read_stdin_content(stdin);
            let round = ctx.client.discussion_add_round(&slug, &mode, &content)?.round;
            render_discuss_add_round(&slug, round as usize, &mode, json)
        }
        DiscussCommands::Conclude { slug, stdin, json } => {
            let content = read_stdin_content(stdin);
            let resp = ctx.client.discussion_conclude(&slug, &content)?;
            render_discuss_conclude(&slug, &resp.restale_flagged, resp.auto_archived, json)
        }
        DiscussCommands::Archive { slug, json } => {
            let archived_to = ctx.client.discussion_archive(&slug)?.archived_to;
            render_discuss_archive(&slug, &archived_to, json)
        }
        DiscussCommands::Promote { slug, name, json } => {
            let change = ctx.client.discussion_promote(&slug, name.as_deref())?.change;
            // 明文分歧（design D5）：新變更目錄是 store 端位置，remote 不印，
            // 與 `new change` 的 Path 行同一條裁定。
            render_discuss_promote(&slug, &change, None, json)
        }
        DiscussCommands::Discard { slug, force, json } => {
            let slug = ctx.client.discard_discussion(&slug, force)?.slug;
            render_discuss_discard(&slug, json)
        }
        DiscussCommands::Link { slug, change, json } => {
            let bound = ctx.client.link_discussion(&slug, &change)?;
            render_discuss_bind(&bound.slug, &bound.change, DiscussBind::Link, json)
        }
        DiscussCommands::Seal { slug, change, json } => {
            let bound = ctx.client.seal_discussion(&slug, &change)?;
            render_discuss_bind(&bound.slug, &bound.change, DiscussBind::Seal, json)
        }
    }
}
/// The wire discussion summary reshaped into the engine's info type, so both
/// modes render (and serialize) it through one path.
fn to_discussion_info(d: &protocol_query::DiscussionInfo) -> core::discuss::DiscussionInfo {
    core::discuss::DiscussionInfo {
        slug: d.slug.clone(),
        topic: d.topic.clone(),
        status: d.status.clone(),
        rounds: d.rounds,
        created: d.created.clone(),
        created_by: d.created_by.clone(),
        kind: d.kind.clone(),
        path: d.path.clone(),
        archived: d.archived,
    }
}
