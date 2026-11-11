//! The discuss family: every discussion subcommand, both arms.
//!
//! Dual at the family level — the subcommand enum is matched exhaustively with
//! no catch-all in either arm, so a new subcommand fails to build until both
//! modes answer for it.

use anyhow::Result;
use speclink_core as core;
use speclink_protocol::query as protocol_query;

use crate::color;
use crate::common::{engine_command, open_project, print_json, read_stdin_content, run, stdin_for};
use crate::remote_base::RemoteCtx;
use speclink_verbs::families::discuss::{DiscussArgs, DiscussCommands};
use speclink_verbs::{ToCommand, VerbInputs};

pub(crate) fn cmd_discuss(a: DiscussArgs) -> Result<()> {
    let (ws, store) = open_project()?;
    let stdin = stdin_for(a.stdin_policy());
    let cmd = engine_command(a.to_command(VerbInputs { stdin, ..Default::default() }))?;
    match a.command {
        DiscussCommands::New { json, .. } => {
            let info: core::discuss::DiscussionInfo = run(&store, Some(&ws), cmd)?;
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
            let items: Vec<core::discuss::DiscussionInfo> = run(&store, Some(&ws), cmd)?;
            render_discuss_list(&items, archived, json)?;
        }
        DiscussCommands::Show { json, .. } => {
            let show: core::command::DiscussShowOutcome = run(&store, Some(&ws), cmd)?;
            render_discuss_show(&show, json)?;
        }
        DiscussCommands::Search { terms, json } => {
            let hits: Vec<core::discuss::DiscussionHit> = run(&store, Some(&ws), cmd)?;
            render_discuss_search(&terms, &hits, json)?;
        }
        DiscussCommands::Context { json, .. } => {
            let o: core::command::DiscussSubjectOutcome = run(&store, Some(&ws), cmd)?;
            render_discuss_context(&o.slug, json)?;
        }
        DiscussCommands::AddRound { json, .. } => {
            let o: core::command::DiscussRoundOutcome = run(&store, Some(&ws), cmd)?;
            render_discuss_add_round(&o.slug, o.round, &o.mode, json)?;
        }
        DiscussCommands::Conclude { json, .. } => {
            let o: core::command::DiscussConcludeOutcome = run(&store, Some(&ws), cmd)?;
            render_discuss_conclude(&o.slug, &o.restale_flagged, o.auto_archived, o.held, json)?;
            // 閉環封存步失敗：結論與 restale 已落盤（上面照常呈現），這裡以非零
            // exit code 收場、stderr 說明原因；重跑 discuss archive 即可收尾。
            if let Some(reason) = &o.closing_error {
                anyhow::bail!("conclude closing step failed to archive the record: {reason}");
            }
        }
        DiscussCommands::Archive { json, .. } => {
            let o: core::command::DiscussArchiveOutcome = run(&store, Some(&ws), cmd)?;
            render_discuss_archive(
                &o.slug,
                &format!("discussions/archive/{}", o.archived_file),
                json,
            )?;
        }
        DiscussCommands::Discard { json, .. } => {
            let o: core::command::DiscussSubjectOutcome = run(&store, Some(&ws), cmd)?;
            render_discuss_discard(&o.slug, json)?;
        }
        DiscussCommands::Promote { json, .. } => {
            let o: core::command::DiscussPromoteOutcome = run(&store, Some(&ws), cmd)?;
            let shown = o.path.to_string_lossy();
            let wire = core::util::to_slash(&o.path);
            render_discuss_promote(
                &o.slug,
                &o.change,
                Some(PromotedPath { shown: &shown, wire: &wire }),
                json,
            )?;
        }
        DiscussCommands::Link { json, .. } => {
            let o: core::command::DiscussBindOutcome = run(&store, Some(&ws), cmd)?;
            render_discuss_bind(&o.slug, &o.change, DiscussBind::Link, json)?;
        }
        DiscussCommands::Seal { json, .. } => {
            let o: core::command::DiscussBindOutcome = run(&store, Some(&ws), cmd)?;
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
/// `discuss search` in both modes: one line per hit in the list's idiom, then
/// one indented line per match (`<where> <kind>: <text>`). Zero hits is a
/// success with a one-line notice; `--json` is `{ "hits": [...] }` either way.
fn render_discuss_search(
    terms: &[String],
    hits: &[core::discuss::DiscussionHit],
    json: bool,
) -> Result<()> {
    if json {
        return print_json(&serde_json::json!({ "hits": hits }));
    }
    let query = terms.join(" ");
    if hits.is_empty() {
        println!("No discussions match \"{query}\".");
        return Ok(());
    }
    println!("Discussions matching \"{query}\":");
    for hit in hits {
        let d = &hit.info;
        let place = if d.archived { "archived" } else { "live" };
        println!("  • {} [{}, {place}] ({}) — {}", d.slug, d.status, d.created, d.topic);
        for m in &hit.matches {
            println!("      {} {}: {}", color::dim(&m.where_), m.kind, m.text);
        }
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
    held: bool,
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
        if held {
            payload["held"] = serde_json::json!(true);
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
    if held {
        println!("  Held live (a later spin-out is planned)");
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
        DiscussCommands::Search { terms, json } => {
            // 同一個 renderer：wire hit 轉回引擎型別（資訊欄位走既有 to_discussion_info，
            // promotedTo／concluded 沿 list 的既定分歧落下）。
            let hits: Vec<core::discuss::DiscussionHit> = ctx
                .client
                .search_discussions(&terms)?
                .hits
                .iter()
                .map(|h| core::discuss::DiscussionHit {
                    info: to_discussion_info(&h.info),
                    matches: h
                        .matches
                        .iter()
                        .map(|m| core::discuss::DiscussionMatch {
                            kind: m.kind.clone(),
                            where_: m.where_.clone(),
                            text: m.text.clone(),
                        })
                        .collect(),
                })
                .collect();
            render_discuss_search(&terms, &hits, json)
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
        DiscussCommands::Conclude { slug, stdin, hold, json } => {
            let content = read_stdin_content(stdin);
            let resp = ctx.client.discussion_conclude(&slug, &content, hold)?;
            render_discuss_conclude(
                &slug,
                &resp.restale_flagged,
                resp.auto_archived,
                resp.held,
                json,
            )
        }
        DiscussCommands::Archive { slug, json } => {
            let archived_to = ctx.client.discussion_archive(&slug)?.archived_to;
            render_discuss_archive(&slug, &archived_to, json)
        }
        DiscussCommands::Promote { slug, name, last, json } => {
            let change = ctx.client.discussion_promote(&slug, name.as_deref(), last)?.change;
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
        DiscussCommands::Seal { slug, change, last, json } => {
            let bound = ctx.client.seal_discussion(&slug, &change, last)?;
            render_discuss_bind(&bound.slug, &bound.change, DiscussBind::Seal, json)
        }
    }
}
/// The wire discussion summary reshaped into the engine's info type, so both
/// modes render (and serialize) it through one path.
fn to_discussion_info(d: &protocol_query::DiscussionInfo) -> core::discuss::DiscussionInfo {
    // wire DTO 帶 promotedTo／concluded／hold（缺席＝舊 server，視同未保留）；
    // board_rank 不上鏈，取預設。CLI 不顯示 hold，映過來只為 head 與 fs 模式同形。
    let mut head = core::discuss::DiscussionHead::default();
    head.promoted_to = d.promoted_to.clone();
    head.hold = d.hold.unwrap_or(false);
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
        concluded: d.concluded.unwrap_or(false),
        head,
    }
}
