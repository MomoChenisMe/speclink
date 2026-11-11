//! The discuss family: every discussion subcommand, both arms.
//!
//! Dual at the family level — the subcommand enum is matched exhaustively with
//! no catch-all in either arm, so a new subcommand fails to build until both
//! modes answer for it.

use anyhow::Result;
use speclink_core as core;
use speclink_protocol::query as protocol_query;

use crate::common::{
    emit, engine_command, open_project, read_stdin_content, render_options, run, stdin_for,
};
use crate::remote_base::RemoteCtx;
use speclink_verbs::families::discuss::{DiscussArgs, DiscussCommands};
use speclink_verbs::render::discuss::{
    self as render, Concluded, DiscussBind, NewDiscussionLines, Promoted, PromotedPath, RecordedRound,
};
use speclink_verbs::{ToCommand, VerbInputs};

pub(crate) fn cmd_discuss(a: DiscussArgs) -> Result<()> {
    let (ws, store) = open_project()?;
    let stdin = stdin_for(a.stdin_policy());
    let cmd = engine_command(a.to_command(VerbInputs { stdin, ..Default::default() }))?;
    match a.command {
        DiscussCommands::New { json, .. } => {
            let info: core::discuss::DiscussionInfo = run(&store, Some(&ws), cmd)?;
            emit(render::new(&info, render_options(json)));
        }
        DiscussCommands::List { archived, json } => {
            let items: Vec<core::discuss::DiscussionInfo> = run(&store, Some(&ws), cmd)?;
            emit(render::list(&items, archived, render_options(json)));
        }
        DiscussCommands::Show { json, .. } => {
            let show: core::command::DiscussShowOutcome = run(&store, Some(&ws), cmd)?;
            emit(render::show(&show, render_options(json)));
        }
        DiscussCommands::Search { terms, json } => {
            let hits: Vec<core::discuss::DiscussionHit> = run(&store, Some(&ws), cmd)?;
            emit(render::search(&terms, &hits, render_options(json)));
        }
        DiscussCommands::Context { json, .. } => {
            let o: core::command::DiscussSubjectOutcome = run(&store, Some(&ws), cmd)?;
            emit(render::context(&o.slug, render_options(json)));
        }
        DiscussCommands::AddRound { json, .. } => {
            let o: core::command::DiscussRoundOutcome = run(&store, Some(&ws), cmd)?;
            emit(render::add_round(
                &RecordedRound { slug: &o.slug, round: o.round, mode: &o.mode },
                render_options(json),
            ));
        }
        DiscussCommands::Conclude { json, .. } => {
            let o: core::command::DiscussConcludeOutcome = run(&store, Some(&ws), cmd)?;
            emit(render::conclude(
                &Concluded {
                    slug: &o.slug,
                    restale_flagged: &o.restale_flagged,
                    auto_archived: o.auto_archived,
                    held: o.held,
                },
                render_options(json),
            ));
            // 閉環封存步失敗：結論與 restale 已落盤（上面照常呈現），這裡以非零
            // exit code 收場、stderr 說明原因；重跑 discuss archive 即可收尾。
            if let Some(reason) = &o.closing_error {
                anyhow::bail!("{}", render::closing_error_message(reason));
            }
        }
        DiscussCommands::Archive { json, .. } => {
            let o: core::command::DiscussArchiveOutcome = run(&store, Some(&ws), cmd)?;
            emit(render::archive(
                &o.slug,
                &render::archived_to(&o.archived_file),
                render_options(json),
            ));
        }
        DiscussCommands::Discard { json, .. } => {
            let o: core::command::DiscussSubjectOutcome = run(&store, Some(&ws), cmd)?;
            emit(render::discard(&o.slug, render_options(json)));
        }
        DiscussCommands::Promote { json, .. } => {
            let o: core::command::DiscussPromoteOutcome = run(&store, Some(&ws), cmd)?;
            let shown = o.path.to_string_lossy();
            let wire = core::util::to_slash(&o.path);
            emit(render::promote(
                &Promoted {
                    slug: &o.slug,
                    change: &o.change,
                    path: Some(PromotedPath { shown: &shown, wire: &wire }),
                },
                render_options(json),
            ));
        }
        DiscussCommands::Link { json, .. } => {
            let o: core::command::DiscussBindOutcome = run(&store, Some(&ws), cmd)?;
            emit(render::bind(DiscussBind::Link, &o.slug, &o.change, render_options(json)));
        }
        DiscussCommands::Seal { json, .. } => {
            let o: core::command::DiscussBindOutcome = run(&store, Some(&ws), cmd)?;
            emit(render::bind(DiscussBind::Seal, &o.slug, &o.change, render_options(json)));
        }
    }
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
            emit(render::list(&items, archived, render_options(json)));
        }
        DiscussCommands::Search { terms, json } => {
            // 同一支輸出函式：wire hit 轉回引擎型別（資訊欄位走既有 to_discussion_info，
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
            emit(render::search(&terms, &hits, render_options(json)));
        }
        DiscussCommands::Show { slug, json } => {
            let payload = ctx.client.show_discussion(&slug)?;
            emit(render::show(
                &core::command::DiscussShowOutcome {
                    info: Some(to_discussion_info(&payload.info)),
                    content: payload.content,
                },
                render_options(json),
            ));
        }
        DiscussCommands::New { topic, slug, kind, json } => {
            // --slug 與 --kind 隨請求上 wire；驗證的單一事實來源在引擎（server 端），
            // CLI 不預驗（design D1）。
            let resp = ctx.client.new_discussion(&topic, slug.as_deref(), kind.as_deref())?;
            // remote 的 --json 契約是 wire 回應原樣（slug／topic／path）——
            // 組 core 型別會捏造 server 沒說的欄位，形狀凍結不允許。
            emit(render::new_from(
                &resp,
                NewDiscussionLines { slug: &resp.slug, topic: &resp.topic, path: &resp.path },
                render_options(json),
            ));
        }
        DiscussCommands::Context { slug, stdin, json } => {
            let content = read_stdin_content(stdin);
            ctx.client.discussion_context(&slug, &content)?;
            emit(render::context(&slug, render_options(json)));
        }
        DiscussCommands::AddRound { slug, mode, stdin, json } => {
            let content = read_stdin_content(stdin);
            let round = ctx.client.discussion_add_round(&slug, &mode, &content)?.round;
            emit(render::add_round(
                &RecordedRound { slug: &slug, round: round as usize, mode: &mode },
                render_options(json),
            ));
        }
        DiscussCommands::Conclude { slug, stdin, hold, json } => {
            let content = read_stdin_content(stdin);
            let resp = ctx.client.discussion_conclude(&slug, &content, hold)?;
            emit(render::conclude(
                &Concluded {
                    slug: &slug,
                    restale_flagged: &resp.restale_flagged,
                    auto_archived: resp.auto_archived,
                    held: resp.held,
                },
                render_options(json),
            ));
        }
        DiscussCommands::Archive { slug, json } => {
            let archived_to = ctx.client.discussion_archive(&slug)?.archived_to;
            emit(render::archive(&slug, &archived_to, render_options(json)));
        }
        DiscussCommands::Promote { slug, name, last, json } => {
            let change = ctx.client.discussion_promote(&slug, name.as_deref(), last)?.change;
            // 明文分歧（design D5）：新變更目錄是 store 端位置，remote 不印，
            // 與 `new change` 的 Path 行同一條裁定。
            emit(render::promote(
                &Promoted { slug: &slug, change: &change, path: None },
                render_options(json),
            ));
        }
        DiscussCommands::Discard { slug, force, json } => {
            let slug = ctx.client.discard_discussion(&slug, force)?.slug;
            emit(render::discard(&slug, render_options(json)));
        }
        DiscussCommands::Link { slug, change, json } => {
            let bound = ctx.client.link_discussion(&slug, &change)?;
            emit(render::bind(DiscussBind::Link, &bound.slug, &bound.change, render_options(json)));
        }
        DiscussCommands::Seal { slug, change, last, json } => {
            let bound = ctx.client.seal_discussion(&slug, &change, last)?;
            emit(render::bind(DiscussBind::Seal, &bound.slug, &bound.change, render_options(json)));
        }
    }
    Ok(())
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
