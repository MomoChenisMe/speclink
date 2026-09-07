//! Discussion documents — a speclink extension: discussions are durable records.
//!
//! Each discussion is a single append-only document (stored by the Store as a
//! live discussion under its slug) so an iterative conversation accumulates a
//! durable record that `propose` can later consume. Archived discussions are
//! renamed by the store with a `<created>-` date prefix — like archived
//! changes — so a slug can be reused by a later discussion.

use crate::store::{DiscussionDoc, Store};
use crate::util;
use anyhow::{bail, Result};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct DiscussionInfo {
    pub slug: String,
    pub topic: String,
    pub status: String,
    pub rounds: usize,
    pub created: String,
    /// 建立者（"Name <email>"），discuss new 由 git 身分蓋章；缺席時省略。
    #[serde(rename = "createdBy", skip_serializing_if = "Option::is_none", default)]
    pub created_by: Option<String>,
    /// 討論型別（目前唯一合法值 `improve`）；一般討論缺席時省略。
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub kind: Option<String>,
    pub path: String,
    pub archived: bool,
}

/// `discuss new --kind` 的白名單——驗證與拒絕訊息的單一事實來源。
/// CLI `--kind` 的 help 字面（clap 靜態字串）另行點名合法值，擴充時同步。
pub const DISCUSSION_KINDS: &[&str] = &["improve"];

fn frontmatter_value(text: &str, key: &str) -> Option<String> {
    let mut in_fm = false;
    for (i, line) in text.lines().enumerate() {
        if i == 0 && line.trim() == "---" {
            in_fm = true;
            continue;
        }
        if in_fm {
            if line.trim() == "---" {
                break;
            }
            if let Some(rest) = line.strip_prefix(&format!("{key}:")) {
                return Some(rest.trim().to_string());
            }
        }
    }
    None
}

/// 輪標題前綴：scaffold 版面（level-3）與 pre-scaffold 容忍（level-2）。
/// 辨識端（計數、跳脫、區段邊界、discard 保護）與產生端（add_round）共用。
const SCAFFOLD_ROUND_PREFIX: &str = "### Round ";
const PRE_SCAFFOLD_ROUND_PREFIX: &str = "## Round ";

/// Fenced code block 圍欄行（``` 或 ~~~ 開頭；容忍前導空白）。圍欄內的行不是
/// 結構——跳脫、計數與區段解析一致跳過。簡化：不比對圍欄長度與縮排細則，
/// 討論記錄引用版面的用途以此為足。
fn is_fence_line(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("```") || t.starts_with("~~~")
}

/// 合法輪標題形狀（scaffold 版面）：`### Round <編號> — <mode> (<日期>)`，
/// 與 UI splitRounds 的判準同形。跳脫後的內文行（行首帶反斜線）與
/// 缺編號、缺 mode、缺日期括號的撞名行都不是輪。
fn is_scaffold_round_heading(line: &str) -> bool {
    let Some(rest) = line.strip_prefix(SCAFFOLD_ROUND_PREFIX) else {
        return false;
    };
    let digits = rest.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 {
        return false;
    }
    let Some(tail) = rest[digits..].strip_prefix(" — ") else {
        return false;
    };
    // `<mode> (<date>)`：mode 與日期都非空。
    let t = tail.trim_end();
    match t.rfind(" (") {
        Some(i) => i > 0 && t.ends_with(')') && i + 2 < t.len() - 1,
        None => false,
    }
}

fn count_rounds(text: &str) -> usize {
    // `## Round ` tolerates pre-scaffold documents.
    let mut in_fence = false;
    text.lines()
        .filter(|l| {
            if is_fence_line(l) {
                in_fence = !in_fence;
                return false;
            }
            !in_fence
                && (is_scaffold_round_heading(l) || l.starts_with(PRE_SCAFFOLD_ROUND_PREFIX))
        })
        .count()
}

/// discard 的保護偵測：對形狀寬鬆（任何輪標題前綴都算，含手改壞形狀），
/// 與 [`count_rounds`] 的嚴格計數刻意分離——保護面誤拒比誤刪安全。
/// 圍欄內的引用不算：寫入端保證圍欄成對，圍欄內容確定不是輪。
fn round_traces(text: &str) -> usize {
    let mut in_fence = false;
    text.lines()
        .filter(|l| {
            if is_fence_line(l) {
                in_fence = !in_fence;
                return false;
            }
            !in_fence
                && (l.starts_with(SCAFFOLD_ROUND_PREFIX)
                    || l.starts_with(PRE_SCAFFOLD_ROUND_PREFIX))
        })
        .count()
}

/// 結構標題白名單——只有這三個整行標題是討論文件的區段邊界；
/// 輪內文的其他「## 」行不是結構、不得截斷區段。
const STRUCTURAL_HEADERS: &[&str] = &["## Context", "## Rounds", "## Conclusion"];

/// 區段邊界：結構標題白名單，加 pre-scaffold 輪標題（`## Round ` 前綴）的容忍——
/// 內文的同形行經寫入端跳脫必帶反斜線，故未跳脫者必為結構。
fn is_section_boundary(line: &str) -> bool {
    STRUCTURAL_HEADERS.contains(&line) || line.starts_with(PRE_SCAFFOLD_ROUND_PREFIX)
}

/// Byte range of a structural section's body: after the `## <name>` line, up to the next
/// section boundary ([`is_section_boundary`]) or EOF. Content lines that merely start
/// with `## `, and any line inside a fenced code block, do not terminate the section.
/// `None` when the header is absent.
fn section_body_range(text: &str, name: &str) -> Option<(usize, usize)> {
    let header = format!("## {name}");
    let mut offset = 0;
    let mut start: Option<usize> = None;
    let mut in_fence = false;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_end();
        if is_fence_line(trimmed) {
            in_fence = !in_fence;
        } else if !in_fence {
            if let Some(s) = start {
                if is_section_boundary(trimmed) {
                    return Some((s, offset));
                }
            } else if trimmed == header {
                start = Some(offset + line.len());
            }
        }
        offset += line.len();
    }
    start.map(|s| (s, text.len()))
}

/// 討論內容寫入動詞共用的落盤前跳脫：撞名內容行（整行為結構標題，或行首為
/// 輪標題前綴）加 markdown 反斜線，使內容不可能被解讀為文件結構。
/// 成對 fenced code block 內的行照原樣保留（markdown 在圍欄內不解跳脫）；
/// 圍欄行為奇數時，最後一個落單的圍欄行一併跳脫——落盤內容因此永遠成對，
/// 全文件的圍欄解析（section_body_range／count_rounds）得以保持健全。
/// 其他「# 」開頭行維持原樣（最小改動，非全面跳脫）。
fn escape_colliding_lines(content: &str) -> String {
    let fence_lines: Vec<usize> = content
        .split('\n')
        .enumerate()
        .filter(|(_, l)| is_fence_line(l.trim_end()))
        .map(|(i, _)| i)
        .collect();
    let dangling = (fence_lines.len() % 2 == 1).then(|| *fence_lines.last().unwrap());
    let mut in_fence = false;
    content
        .split('\n')
        .enumerate()
        .map(|(i, l)| {
            let t = l.trim_end();
            if is_fence_line(t) {
                if Some(i) == dangling {
                    return format!("\\{l}");
                }
                in_fence = !in_fence;
                return l.to_string();
            }
            if !in_fence
                && (STRUCTURAL_HEADERS.contains(&t)
                    || t.starts_with(SCAFFOLD_ROUND_PREFIX)
                    || t.starts_with(PRE_SCAFFOLD_ROUND_PREFIX))
            {
                format!("\\{l}")
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Replace a level-2 section's body, keeping its header. `None` when the section is absent.
fn replace_section(text: &str, name: &str, body: &str) -> Option<String> {
    let (s, e) = section_body_range(text, name)?;
    let tail = &text[e..];
    let mid = if tail.is_empty() {
        format!("\n{}\n", body.trim_end())
    } else {
        format!("\n{}\n\n", body.trim_end())
    };
    Some(format!("{}{}{}", &text[..s], mid, tail))
}

fn strip_html_comments(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find("<!--") {
        out.push_str(&rest[..i]);
        match rest[i..].find("-->") {
            Some(j) => rest = &rest[i + j + 3..],
            None => rest = "",
        }
    }
    out.push_str(rest);
    out
}

fn info_from_doc(doc: &DiscussionDoc) -> DiscussionInfo {
    DiscussionInfo {
        slug: frontmatter_value(&doc.text, "slug").unwrap_or_else(|| doc.slug.clone()),
        topic: frontmatter_value(&doc.text, "topic").unwrap_or_else(|| doc.slug.clone()),
        status: frontmatter_value(&doc.text, "status").unwrap_or_else(|| "open".to_string()),
        rounds: count_rounds(&doc.text),
        created: frontmatter_value(&doc.text, "created").unwrap_or_default(),
        created_by: frontmatter_value(&doc.text, "created_by"),
        // 空值 `kind:`（手改記錄）正規化為缺席，維持「缺席即省略」的 payload 形狀。
        kind: frontmatter_value(&doc.text, "kind").filter(|v| !v.is_empty()),
        path: util::to_slash(&doc.path),
        archived: doc.archived,
    }
}

/// Load a live discussion for mutation; a helpful error distinguishes "archived" from "missing".
fn load_live(store: &dyn Store, slug: &str) -> Result<String> {
    match store.read_live_discussion(slug) {
        Some(t) => Ok(t),
        None => {
            if store.archived_discussion_exists(slug) {
                bail!("discussion '{slug}' is archived — move it out of discussions/archive/ to continue it");
            }
            bail!("discussion '{slug}' not found — run `speclink discuss new` first")
        }
    }
}

/// Kebab-case gate for the slug override: lowercase ASCII letters/digits in
/// single-hyphen-separated runs. Deliberately stricter than the topic-derived
/// fallback (which keeps CJK) — the override exists to produce English names.
fn is_valid_slug_override(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('-')
        && !s.ends_with('-')
        && !s.contains("--")
        && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Create a new discussion document. Errors if a live one already exists.
/// `slug_override` names the record file directly (validated ASCII kebab-case);
/// without it the slug falls back to deriving from the topic. `kind` marks the
/// record's type (whitelist [`DISCUSSION_KINDS`]); absent means a plain discussion.
pub fn new_discussion(
    store: &dyn Store,
    topic: &str,
    slug_override: Option<&str>,
    created_by: Option<&str>,
    kind: Option<&str>,
) -> Result<DiscussionInfo> {
    if let Some(k) = kind {
        if !DISCUSSION_KINDS.contains(&k) {
            bail!(
                "invalid kind '{k}' — --kind accepts only: {}",
                DISCUSSION_KINDS.join(", ")
            );
        }
    }
    // topic 逐字寫入 frontmatter，夾帶換行可注入偽造的 kind:/status: 行——
    // 在系統邊界一次擋下整類注入。
    if topic.contains(['\n', '\r']) {
        bail!("invalid topic '{}' — must be a single line", topic.escape_debug());
    }
    let slug = match slug_override {
        Some(s) => {
            if !is_valid_slug_override(s) {
                bail!(
                    "invalid slug '{s}' — must be ASCII kebab-case: lowercase letters/digits \
                     separated by single hyphens (e.g. board-search-bar)"
                );
            }
            s.to_string()
        }
        None => util::slugify(topic),
    };
    if slug.is_empty() {
        bail!("could not derive a slug from topic '{topic}'");
    }
    if store.live_discussion_exists(&slug) {
        bail!(
            "discussion '{slug}' already exists at {}",
            util::to_slash(&store.live_discussion_path(&slug))
        );
    }
    let created = util::today();
    // 建立者章（比照 change 的 newcmd）：有 git 身分才蓋，無身分省略該行。
    let created_by_line = created_by
        .map(|id| format!("created_by: {id}\n"))
        .unwrap_or_default();
    // kind 已過白名單，寫出的必是常數字串（無 YAML 跳脫顧慮）；缺席時整行不存在。
    let kind_line = kind.map(|k| format!("kind: {k}\n")).unwrap_or_default();
    let content = format!(
        "---\n\
         topic: {topic}\n\
         slug: {slug}\n\
         status: open\n\
         created: {created}\n\
         {created_by_line}\
         {kind_line}\
         ---\n\
         \n\
         # Discussion: {topic}\n\
         \n\
         <!--\n\
         Document rules:\n\
         - Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.\n\
         \x20 A changed position gets a new round that names what changed and why.\n\
         - Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.\n\
         - The conclusion must resolve or explicitly defer every open question left by the rounds.\n\
         -->\n\
         \n\
         ## Context\n\
         \n\
         <!-- What prompted this discussion, whether a grill stage was needed and why,\n\
         and the related changes/specs. Set once via `speclink discuss context <slug> --stdin`. -->\n\
         \n\
         ## Rounds\n\
         \n\
         <!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->\n\
         \n\
         ## Conclusion\n\
         \n\
         <!-- Written by `speclink discuss conclude`:\n\
         **Decision** / **Rationale** / **Rejected alternatives** / **Deferred** / **Capture to** / **Next** -->\n"
    );
    let path = store.write_live_discussion(&slug, &content)?;
    Ok(DiscussionInfo {
        slug,
        topic: topic.to_string(),
        status: "open".to_string(),
        rounds: 0,
        created,
        created_by: created_by.map(str::to_string),
        kind: kind.map(str::to_string),
        path: util::to_slash(&path),
        archived: false,
    })
}

/// List live discussions (sorted by slug).
pub fn list_discussions(store: &dyn Store) -> Vec<DiscussionInfo> {
    let mut out: Vec<DiscussionInfo> = store
        .list_live_discussions()
        .iter()
        .map(info_from_doc)
        .collect();
    out.sort_by(|a, b| a.slug.cmp(&b.slug));
    out
}

/// List archived discussions (sorted by archived file name, i.e. by archive date).
pub fn list_archived(store: &dyn Store) -> Vec<DiscussionInfo> {
    let mut out: Vec<DiscussionInfo> = store
        .list_archived_discussions()
        .iter()
        .map(info_from_doc)
        .collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

pub fn show_discussion(store: &dyn Store, slug: &str) -> Option<String> {
    store.read_discussion(slug).map(|d| d.text)
}

pub fn info(store: &dyn Store, slug: &str) -> Option<DiscussionInfo> {
    store.read_discussion(slug).map(|d| info_from_doc(&d))
}

/// One keyword hit inside a discussion record (`discuss search`, design D3):
/// `kind` names what matched (topic / slug / ruled-out / decision / rejected /
/// deferred), `where_` names where (frontmatter / round-N / conclusion) and
/// `text` is the matched line, outer whitespace trimmed. Serialize-only: the
/// remote CLI rebuilds it from the wire type by hand.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiscussionMatch {
    pub kind: String,
    #[serde(rename = "where")]
    pub where_: String,
    pub text: String,
}

/// One record that `discuss search` matched: the listing fields plus every
/// match in document order. `info` flattens so `--json` reads as the list
/// payload's item with a `matches` array appended.
#[derive(Debug, Clone, Serialize)]
pub struct DiscussionHit {
    #[serde(flatten)]
    pub info: DiscussionInfo,
    pub matches: Vec<DiscussionMatch>,
}

/// Decision-line markers (design D2): the round-side one and the three
/// conclusion-side ones, each with the `kind` it reports.
const RULED_OUT_MARKER: &str = "**Ruled out**:";
const CONCLUSION_MARKERS: &[(&str, &str)] = &[
    ("**Decision**:", "decision"),
    ("**Rejected alternatives**:", "rejected"),
    ("**Deferred**:", "deferred"),
];

/// Search live and archived discussions for any of `terms` (case-insensitive
/// substring; any term hitting counts). Every term is split on whitespace
/// first — the server's `q` arrives that way and the CLI's quoted argument
/// must mean the same thing — so a keyword can never contain a space. Only
/// the topic, the slug and the decision lines take part: each round's
/// `**Ruled out**:` line and the Conclusion's `**Decision**:` /
/// `**Rejected alternatives**:` / `**Deferred**:` line, each together with
/// the list-item lines that directly continue it (records habitually put
/// the marker on its own line and the verdicts as bullets under it).
/// Evidence, Focus, Position, Open and prose never match. A record without
/// round headings or a Conclusion still matches by topic and slug. Hits sort
/// with topic/slug hits first, then created newest first, then slug.
pub fn search(store: &dyn Store, terms: &[String]) -> Result<Vec<DiscussionHit>> {
    let needles: Vec<String> = terms
        .iter()
        .flat_map(|t| t.split_whitespace())
        .map(str::to_lowercase)
        .collect();
    if needles.is_empty() {
        bail!("discuss search needs at least one keyword");
    }
    let hits_any = |text: &str| {
        let lower = text.to_lowercase();
        needles.iter().any(|n| lower.contains(n.as_str()))
    };

    let mut docs = store.list_live_discussions();
    docs.extend(store.list_archived_discussions());
    // (topic or slug hit, hit) — the flag is the first sort key.
    let mut hits: Vec<(bool, DiscussionHit)> = docs
        .iter()
        .filter_map(|doc| {
            let info = info_from_doc(doc);
            let mut matches = Vec::new();
            for (kind, value) in [("topic", &info.topic), ("slug", &info.slug)] {
                if hits_any(value) {
                    matches.push(DiscussionMatch {
                        kind: kind.to_string(),
                        where_: "frontmatter".to_string(),
                        text: value.clone(),
                    });
                }
            }
            let frontmatter_hit = !matches.is_empty();
            matches.extend(decision_lines(&doc.text).filter(|m| hits_any(&m.text)));
            (!matches.is_empty()).then_some((frontmatter_hit, DiscussionHit { info, matches }))
        })
        .collect();
    hits.sort_by(|(a_fm, a), (b_fm, b)| {
        b_fm.cmp(a_fm)
            .then_with(|| b.info.created.cmp(&a.info.created))
            .then_with(|| a.info.slug.cmp(&b.info.slug))
    });
    Ok(hits.into_iter().map(|(_, hit)| hit).collect())
}

/// Every decision line of a record in document order, each tagged with its
/// kind and location. A `**Ruled out**:` marker counts only under a round
/// heading (its number comes from the nearest heading above); the three
/// conclusion markers count only inside `## Conclusion`. The list-item lines
/// directly under a counted marker belong to it (same kind and location, one
/// match per line); any other line — blank, prose, another `**Field**:` —
/// ends that block. Structure is read exactly as [`count_rounds`] and
/// [`section_body_range`] read it: headings at column 0 only, a malformed
/// round heading is no round, fenced code is skipped.
fn decision_lines(text: &str) -> impl Iterator<Item = DiscussionMatch> + '_ {
    let mut in_fence = false;
    let mut round: Option<String> = None;
    let mut in_conclusion = false;
    // The counted marker whose list items are still being collected.
    let mut continuing: Option<(&'static str, String)> = None;
    text.lines().filter_map(move |raw| {
        let line = raw.trim_end();
        if is_fence_line(line) {
            in_fence = !in_fence;
            continuing = None;
            return None;
        }
        if in_fence {
            return None;
        }
        if line.starts_with(SCAFFOLD_ROUND_PREFIX) || line.starts_with(PRE_SCAFFOLD_ROUND_PREFIX) {
            round = round_number(line);
            in_conclusion = false;
            continuing = None;
            return None;
        }
        if STRUCTURAL_HEADERS.contains(&line) {
            in_conclusion = line == "## Conclusion";
            round = None;
            continuing = None;
            return None;
        }
        let content = line.trim_start();
        let marker = if content.starts_with(RULED_OUT_MARKER) {
            round.as_ref().map(|n| ("ruled-out", format!("round-{n}")))
        } else if in_conclusion {
            CONCLUSION_MARKERS
                .iter()
                .find(|(marker, _)| content.starts_with(marker))
                .map(|(_, kind)| (*kind, "conclusion".to_string()))
        } else {
            None
        };
        if let Some((kind, where_)) = marker {
            continuing = Some((kind, where_.clone()));
            return Some(DiscussionMatch { kind: kind.to_string(), where_, text: content.to_string() });
        }
        if is_list_item(content) {
            return continuing.as_ref().map(|(kind, where_)| DiscussionMatch {
                kind: kind.to_string(),
                where_: where_.clone(),
                text: content.to_string(),
            });
        }
        continuing = None;
        None
    })
}

/// A markdown list item (`- `, `* `, `+ ` or `1. `) — the shape a decision
/// marker's continuation lines take.
fn is_list_item(content: &str) -> bool {
    if content.starts_with("- ") || content.starts_with("* ") || content.starts_with("+ ") {
        return true;
    }
    let digits = content.chars().take_while(|c| c.is_ascii_digit()).count();
    digits > 0 && content[digits..].starts_with(". ")
}

/// The number of a round heading — scaffold `### Round N — <mode> (<date>)`
/// or pre-scaffold `## Round N` — and `None` for a malformed one, which
/// [`count_rounds`] does not count either.
fn round_number(line: &str) -> Option<String> {
    let rest = if is_scaffold_round_heading(line) {
        line.strip_prefix(SCAFFOLD_ROUND_PREFIX)?
    } else {
        line.strip_prefix(PRE_SCAFFOLD_ROUND_PREFIX)?
    };
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    (!digits.is_empty()).then_some(digits)
}

/// Reject blank content at the write boundary. The CLI turns a forgotten `--stdin` into an
/// empty string, so guarding here — one place, covering local CLI / remote CLI / desktop —
/// makes that silent failure a loud error instead of a written-but-empty section.
fn ensure_content(content: &str) -> Result<()> {
    if content.trim().is_empty() {
        bail!("discussion content is empty — pass non-empty content via stdin (did you forget --stdin?)");
    }
    Ok(())
}

/// Set (or replace) the `## Context` section — the one-time framing written after mode pick.
pub fn set_context(store: &dyn Store, slug: &str, content: &str) -> Result<()> {
    ensure_content(content)?;
    let content = escape_colliding_lines(content);
    let text = load_live(store, slug)?;
    match replace_section(&text, "Context", &content) {
        Some(t) => {
            store.write_live_discussion(slug, &t)?;
            Ok(())
        }
        None => bail!(
            "discussion '{slug}' has no '## Context' section (pre-scaffold layout) — edit the file directly"
        ),
    }
}

/// Append a discussion round. Content is supplied verbatim (from the skill via stdin).
pub fn add_round(store: &dyn Store, slug: &str, mode: &str, content: &str) -> Result<usize> {
    ensure_content(content)?;
    let content = escape_colliding_lines(content);
    let mut text = load_live(store, slug)?;
    let round_no = count_rounds(&text) + 1;
    let date = util::today();
    // Scaffolded layout: insert at the end of the `## Rounds` section. Pre-scaffold
    // documents fall back to appending a level-2 round at the end.
    if let Some((_, e)) = section_body_range(&text, "Rounds") {
        let entry = format!(
            "{SCAFFOLD_ROUND_PREFIX}{round_no} — {mode} ({date})\n\n{}\n\n",
            content.trim_end()
        );
        text.insert_str(e, &entry);
    } else {
        if !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&format!(
            "\n{PRE_SCAFFOLD_ROUND_PREFIX}{round_no} — {mode} ({date})\n\n{}\n",
            content.trim_end()
        ));
    }
    store.write_live_discussion(slug, &text)?;
    Ok(round_no)
}

/// The text of the `## Conclusion` section, if the discussion has one (the scaffold's
/// placeholder comment does not count as content).
pub fn conclusion_text(store: &dyn Store, slug: &str) -> Option<String> {
    conclusion_body(&store.read_discussion(slug)?.text)
}

fn conclusion_body(text: &str) -> Option<String> {
    let (s, e) = section_body_range(text, "Conclusion")?;
    let body = strip_html_comments(&text[s..e]).trim().to_string();
    (!body.is_empty()).then_some(body)
}

/// Whether a discussion's Conclusion section holds real content (the scaffold's
/// placeholder comment does not count) — the single contract point (design D3) shared
/// by the archive co-archival guard, the conclude closing step, and every listing edge
/// (server route, host bridge, desktop-core). Reads live-first with archived fallback;
/// a missing or unreadable record counts as not concluded, so the guard leaves doubtful
/// records live instead of sweeping them into the archive.
pub fn discussion_concluded(store: &dyn Store, slug: &str) -> bool {
    conclusion_text(store, slug).is_some()
}

/// Mark a discussion as promoted to a change (the discussion side of the bidirectional link).
/// A discussion can fan out into several changes, so `promoted_to` is a comma-separated
/// accumulator: repeated promotes append the new change name rather than being dropped.
/// Accumulating a change also drops the record's `hold: true` flag: the staged spin-out
/// it was waiting for now exists, so the record rejoins the ordinary lifecycle.
pub fn mark_promoted(store: &dyn Store, slug: &str, change: &str) -> Result<()> {
    let mut text = load_live(store, slug)?;
    for from in ["status: open", "status: concluded"] {
        if text.contains(from) {
            text = text.replacen(from, "status: promoted", 1);
            break;
        }
    }
    let accumulated = match frontmatter_value(&text, "promoted_to") {
        Some(existing) => {
            let known = existing.split(',').map(str::trim).any(|c| c == change);
            if !known {
                text = text.replacen(
                    &format!("promoted_to: {existing}"),
                    &format!("promoted_to: {existing}, {change}"),
                    1,
                );
            }
            !known
        }
        None => {
            let stamped = text.replacen(
                "status: promoted\n",
                &format!("status: promoted\npromoted_to: {change}\n"),
                1,
            );
            let landed = stamped != text;
            text = stamped;
            landed
        }
    };
    // A NEW change name is the spin-out the hold flag was waiting for — clear it. All
    // three spin-out paths (promote, `new change --from-discussion`, seal) come through
    // here, so one removal covers them; `link` writes no discussion side and keeps the
    // record byte-identical. The idempotent branch (re-sealing a change already in the
    // list, e.g. a re-ingest after `conclude --hold` flagged it) is not a spin-out and
    // must leave the flag alone.
    if accumulated {
        if let Some(t) = set_frontmatter_line(&text, "hold", None) {
            text = t;
        }
    }
    store.write_live_discussion(slug, &text)?;
    Ok(())
}

/// The discard-side inverse of [`mark_promoted`]: unlink a discarded change from a
/// discussion. Removes the change name from the record's `promoted_to` comma
/// accumulator; when other change names remain the record stays `promoted`, but once
/// the list empties the `promoted_to` line is dropped and the status reverts — to
/// `concluded` when the record carries a real conclusion, else `open` (a promote/link
/// can raise an `open` discussion, so the revert restores its true prior state). The
/// Context/Rounds/Conclusion sections are never touched — only the frontmatter link
/// fields change (same layer `mark_promoted` writes). Returns the record's status
/// after unlinking (`"promoted"` when merely shrunk, else the reverted status), or
/// `None` when there was nothing to do: no live record for the slug (skipped, not an
/// error — the record may be archived or gone), or the change was not in the list
/// (idempotent — re-running discard leaves an already-unlinked record byte-identical).
pub fn unlink_discarded(store: &dyn Store, slug: &str, change: &str) -> Result<Option<String>> {
    let Some(mut text) = store.read_live_discussion(slug) else {
        return Ok(None);
    };
    let Some(existing) = frontmatter_value(&text, "promoted_to") else {
        return Ok(None);
    };
    let current: Vec<&str> =
        existing.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    let remaining: Vec<&str> = current.iter().copied().filter(|s| *s != change).collect();
    if remaining.len() == current.len() {
        // change was never linked here — idempotent no-op, no write
        return Ok(None);
    }
    if remaining.is_empty() {
        // last link died: drop the promoted_to line and revert the status
        let reverted = if conclusion_text(store, slug).is_some() { "concluded" } else { "open" };
        text = text.replacen(&format!("promoted_to: {existing}\n"), "", 1);
        text = text.replacen("status: promoted", &format!("status: {reverted}"), 1);
        store.write_live_discussion(slug, &text)?;
        Ok(Some(reverted.to_string()))
    } else {
        // still referenced by other changes: shrink the list, keep promoted
        text = text.replacen(
            &format!("promoted_to: {existing}"),
            &format!("promoted_to: {}", remaining.join(", ")),
            1,
        );
        store.write_live_discussion(slug, &text)?;
        Ok(Some("promoted".to_string()))
    }
}

/// Stamp the re-ingest-pending flag on every **active** change in a re-concluded
/// discussion's `promoted_to`. The conclude-side mirror of [`unlink_discarded`]: a
/// discussion that was already reflected (its `promoted_to` is non-empty because
/// `seal` wrote it) and is now re-concluded flags each of its changes as stale
/// against the new conclusion. Change names that resolve to no active meta —
/// archived or gone — are skipped (their spec deltas are already in canon; a
/// re-ingest is impossible). Each active change's `restale_from` comma accumulator
/// gains this slug (idempotent: already present skips the write). Returns the active
/// change names carrying the flag, for CLI reporting. `promoted_to` absent/empty, or
/// resolving entirely to non-active changes, writes no change meta.
fn stamp_restale(store: &dyn Store, slug: &str, discussion_text: &str) -> Result<Vec<String>> {
    let Some(promoted) = frontmatter_value(discussion_text, "promoted_to") else {
        return Ok(Vec::new());
    };
    let mut flagged = Vec::new();
    for change in promoted.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let Some(mut meta) = store.read_change_meta(change) else {
            continue; // archived or gone — not an active change, skip
        };
        // 壞 metadata 卡跳過（沿 archived/gone 的 skip 原則）：不得對壞檔
        // append，也不得使 conclude 因單一壞檔中止——使用者修檔後重新 conclude。
        let Ok(parsed) = crate::model::ChangeMeta::from_text(Some(&meta)) else {
            continue;
        };
        let existing = parsed.restale_from.as_deref().map(str::trim).unwrap_or("");
        if parsed.restale_from().iter().any(|s| s == slug) {
            // already flagged for this slug — idempotent, skip the change-side write
        } else if existing.is_empty() {
            if !meta.ends_with('\n') && !meta.is_empty() {
                meta.push('\n');
            }
            meta.push_str(&format!("restale_from: {slug}\n"));
            store.write_change_meta(change, &meta)?;
        } else {
            meta = meta.replacen(
                &format!("restale_from: {existing}"),
                &format!("restale_from: {existing}, {slug}"),
                1,
            );
            store.write_change_meta(change, &meta)?;
        }
        flagged.push(change.to_string());
    }
    Ok(flagged)
}

/// Clear one discussion slug from a change's `restale_from` accumulator — the seal-side
/// inverse of [`stamp_restale`]. When the slug is the sole value the whole line is
/// dropped; otherwise the remaining slugs are kept. The slug being absent (or no
/// `restale_from` field at all) is an idempotent no-op that skips the write. Only the
/// `restale_from` field is touched; every other meta field stays byte-identical.
fn clear_restale(store: &dyn Store, change: &str, slug: &str) -> Result<()> {
    let Some(mut meta) = store.read_change_meta(change) else {
        return Ok(());
    };
    // Change meta is bare YAML (no `---` frontmatter fence), so parse via ChangeMeta
    // like `link`/`stamp_restale` do — `frontmatter_value` only reads discussion docs.
    // 深度防禦：唯一呼叫者 seal 已對壞 metadata 守門，此處到達即應可解析；
    // 萬一未來新增未守門的呼叫者，fail closed 而非靜默疊寫。
    let parsed = crate::model::ChangeMeta::from_text(Some(&meta)).map_err(|reason| {
        crate::model::MetaError { change: change.to_string(), reason }
    })?;
    let existing = match parsed.restale_from.as_deref().map(str::trim) {
        Some(e) if !e.is_empty() => e.to_string(),
        _ => return Ok(()), // no restale_from — nothing to clear
    };
    let current: Vec<&str> =
        existing.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    let remaining: Vec<&str> = current.iter().copied().filter(|s| *s != slug).collect();
    if remaining.len() == current.len() {
        return Ok(()); // slug not present — idempotent no-op, no write
    }
    if remaining.is_empty() {
        meta = meta.replacen(&format!("restale_from: {existing}\n"), "", 1);
    } else {
        meta = meta.replacen(
            &format!("restale_from: {existing}"),
            &format!("restale_from: {}", remaining.join(", ")),
            1,
        );
    }
    store.write_change_meta(change, &meta)?;
    Ok(())
}

/// Outcome of promoting a discussion into a change.
#[derive(Debug)]
pub struct PromoteOutcome {
    pub change: String,
    pub path: PathBuf,
}

/// Strip an archive-style `YYYY-MM-DD-` prefix from a candidate change name —
/// archived names are historical references, not active names to reuse. Kept
/// only when something remains after the prefix.
fn strip_date_prefix(name: &str) -> &str {
    crate::util::strip_date_prefix(name)
}

/// Promote a discussion into a new change (the whole flow, shared by CLI and
/// desktop): refuse archived records, derive the change name (explicit name or
/// the slug, minus any archive date prefix), create the change with a
/// `from_discussion` link, prefill the proposal's Why from the conclusion
/// (topic as fallback), and mark the discussion promoted. Any failure before a
/// step leaves the later steps unexecuted, so a name collision never marks the
/// discussion.
pub fn promote(
    store: &dyn Store,
    slug: &str,
    name: Option<&str>,
    actor: Option<&str>,
) -> Result<PromoteOutcome> {
    match info(store, slug) {
        None => bail!("discussion '{slug}' not found — run `speclink discuss new` first"),
        Some(i) if i.archived => {
            bail!("discussion '{slug}' is archived — move it out of discussions/archive/ to promote it")
        }
        Some(_) => {}
    }
    let change_name = strip_date_prefix(name.unwrap_or(slug)).to_string();
    let schema =
        crate::config::WorkflowConfig::from_text(store.read_workflow_config().as_deref())?
            .schema_name();
    let dir = crate::newcmd::new_change(store, &change_name, None, &schema, None, Some(slug), actor)?;
    // Prefill the proposal's Why from the discussion conclusion (topic as fallback);
    // the remaining sections stay as TBD markers for /speclink-propose to complete.
    let why = conclusion_text(store, slug).unwrap_or_else(|| {
        info(store, slug).map(|i| i.topic).unwrap_or_else(|| slug.to_string())
    });
    let proposal = format!(
        "## Why\n\n{why}\n\n## What Changes\n\n<!-- TBD: derive from the discussion -->\n\n## Capabilities\n\n### New Capabilities\n\n<!-- TBD -->\n\n## Impact\n\n<!-- TBD -->\n"
    );
    store.write_artifact(&change_name, "proposal.md", &proposal)?;
    mark_promoted(store, slug, &change_name)?;
    Ok(PromoteOutcome { change: change_name, path: dir })
}

/// Link a discussion to an EXISTING change — the ingest-side counterpart of
/// `promote` (which scaffolds a new change). Forges ONLY the change-side chain:
/// `from_discussion` in the change metadata. Marking the discussion promoted is
/// NOT done here — that reflection is sealed by [`seal`] once ingest has folded
/// the discussion's content in, so a linked-but-unfilled change never reads as
/// "已轉出". The discussion record is left byte-identical by this call. Archive
/// co-travel still engages: it is driven by the change-side `from_discussion`,
/// not by the discussion's status.
/// The discussion↔change relationship is many-to-many: a change already born of
/// one discussion can be re-linked to a later one (an ingest that revisits an
/// earlier decision), so `from_discussion` is a comma-separated accumulator that
/// appends rather than rejecting. Guards run before any write (a rejection leaves
/// the change meta byte-identical); re-linking the same pair is an idempotent
/// success that skips the change-side write.
pub fn link(store: &dyn Store, slug: &str, change: &str) -> Result<()> {
    match info(store, slug) {
        None => bail!("discussion '{slug}' not found — run `speclink discuss new` first"),
        Some(i) if i.archived => {
            bail!("discussion '{slug}' is archived — move it out of discussions/archive/ to link it")
        }
        Some(_) => {}
    }
    let Some(mut meta) = store.read_change_meta(change) else {
        bail!("Change '{change}' not found.");
    };
    // Fail-closed gate: corrupt metadata must not read as "no source
    // discussion" and take the from_discussion append.
    let parsed = crate::model::ChangeMeta::from_text(Some(&meta)).map_err(|reason| {
        crate::model::MetaError { change: change.to_string(), reason }
    })?;
    let existing = parsed.from_discussion.as_deref().map(str::trim).unwrap_or("");
    if parsed.from_discussions().iter().any(|s| s == slug) {
        // chain already forged for this slug — idempotent, skip the change-side write
    } else if existing.is_empty() {
        // no source discussion yet — add the line (tolerating a missing trailing newline)
        if !meta.ends_with('\n') && !meta.is_empty() {
            meta.push('\n');
        }
        meta.push_str(&format!("from_discussion: {slug}\n"));
        store.write_change_meta(change, &meta)?;
    } else {
        // already born of another discussion — append this slug to the comma list
        meta = meta.replacen(
            &format!("from_discussion: {existing}"),
            &format!("from_discussion: {existing}, {slug}"),
            1,
        );
        store.write_change_meta(change, &meta)?;
    }
    Ok(())
}

/// 內容落地後的封印：把討論標記已轉出（status: promoted、promoted_to 累加變更名）。
/// `link` 只鑄變更側鏈、不再翻狀態——「標記已轉出」的職責移交本動詞，由 ingest 於
/// artifacts 落地完成時呼叫。前置守衛全數通過方寫入：討論存在且未封存、變更存在、且
/// 變更 meta 的 from_discussion 清單已含該 slug（鏈須先由 link／promote／new change
/// 鑄妥）。守衛失敗回可記錄的 Err，兩側檔案逐位元不變。冪等：promoted_to 已含該變更名
/// 時 `mark_promoted` 改寫等值內容。
pub fn seal(store: &dyn Store, slug: &str, change: &str) -> Result<()> {
    match info(store, slug) {
        None => bail!("discussion '{slug}' not found — run `speclink discuss new` first"),
        Some(i) if i.archived => {
            bail!("discussion '{slug}' is archived — move it out of discussions/archive/ to seal it")
        }
        Some(_) => {}
    }
    let Some(meta) = store.read_change_meta(change) else {
        bail!("Change '{change}' not found.");
    };
    // Fail-closed gate: a corrupt document must report itself, not a missing
    // from_discussion chain.
    let parsed = crate::model::ChangeMeta::from_text(Some(&meta)).map_err(|reason| {
        crate::model::MetaError { change: change.to_string(), reason }
    })?;
    if !parsed
        .from_discussions()
        .iter()
        .any(|s| s == slug)
    {
        bail!("Change '{change}' is not linked to discussion '{slug}' — run `speclink discuss link` first.");
    }
    mark_promoted(store, slug, change)?;
    // Sealing is the honest "content landed" act: clear this discussion's re-ingest flag
    // from the change (the seal-side inverse of the conclude-time stamp). Per-slug — a
    // change stale against another discussion keeps that slug pending its own re-seal.
    clear_restale(store, change, slug)
}

/// 討論卡的看板欄內排序鍵（frontmatter 的 `board_rank`）。沿 `promoted_to` 的
/// 同款模式：獨立讀取函式、不進 `DiscussionInfo`，`discuss list --json` 逐位元不變。
pub fn board_rank(store: &dyn Store, slug: &str) -> Option<String> {
    let doc = store.read_discussion(slug)?;
    frontmatter_value(&doc.text, "board_rank").filter(|v| !v.is_empty())
}

/// 寫入、原位代換或移除 frontmatter 的一行純量——discuss 側所有 frontmatter 文字
/// 手術的共同落點（`board_rank`、`hold`）。`value` 為 `Some` 時第一條 `<key>:` 行
/// 原位代換、之後的重複行一併移除，沒有就插在 frontmatter 尾端（closing `---` 前）；
/// 為 `None` 時移除每一條 `<key>:` 行。鍵的認法與 [`frontmatter_value`] 相同，讀寫
/// 兩邊看法一致；其餘內容逐位元組保留，行尾沿該檔既有的換行（LF 或 CRLF）。
/// 未閉合的 frontmatter（缺尾 `---`）與讀端同樣寬鬆——整檔視為 frontmatter，原位代換
/// 與移除照做；只有「要新插一行卻找不到尾 `---`」與「開頭不是 `---`」回 `None`。
fn set_frontmatter_line(text: &str, key: &str, value: Option<&str>) -> Option<String> {
    let prefix = format!("{key}:");
    let mut out = String::with_capacity(text.len() + prefix.len() + 8);
    let mut state = 0u8; // 0＝等開頭 ---、1＝frontmatter 內、2＝frontmatter 後
    let mut opened = false;
    let mut closed = false;
    let mut placed = false;
    for (i, l) in text.split_inclusive('\n').enumerate() {
        match state {
            0 => {
                out.push_str(l);
                opened = i == 0 && l.trim_end() == "---";
                state = if opened { 1 } else { 2 };
            }
            1 => {
                let eol = if l.ends_with("\r\n") { "\r\n" } else { "\n" };
                if l.trim_end() == "---" {
                    if let (Some(v), false) = (value, placed) {
                        out.push_str(&format!("{prefix} {v}{eol}"));
                    }
                    out.push_str(l);
                    closed = true;
                    state = 2;
                } else if l.starts_with(&prefix) {
                    // 第一條原位代換；重複鍵與移除都是整行丟掉。
                    if let (Some(v), false) = (value, placed) {
                        out.push_str(&format!("{prefix} {v}{eol}"));
                        placed = true;
                    }
                } else {
                    out.push_str(l);
                }
            }
            _ => out.push_str(l),
        }
    }
    (closed || placed || (opened && value.is_none())).then_some(out)
}

/// 寫入（或原位更新）一筆 live 討論的看板排序鍵：既有 `board_rank:` 行原位代換
/// （多出來的重複鍵一併收掉，與讀端只認第一行的看法對齊），否則插入 frontmatter
/// 尾端（closing `---` 前）；其餘內容逐位元組保留。走 [`set_frontmatter_line`]，
/// 非法 rank、封存或不存在的討論、無 frontmatter 可插皆回明確錯誤（封存記錄不上看板）。
pub fn set_board_rank(store: &dyn Store, slug: &str, rank: &str) -> Result<()> {
    if !crate::util::is_valid_board_rank(rank) {
        bail!("invalid board rank '{rank}' — lowercase ASCII letters only");
    }
    let text = load_live(store, slug)?;
    let Some(out) = set_frontmatter_line(&text, "board_rank", Some(rank)) else {
        bail!("discussion '{slug}' has no frontmatter — cannot set board rank");
    };
    store.write_live_discussion(slug, &out)?;
    Ok(())
}

/// The change names a discussion has fanned out into — the frontmatter's
/// comma-separated `promoted_to` accumulator, live or archived. Kept out of
/// `DiscussionInfo` so `discuss list --json` stays bit-identical (design D2).
pub fn promoted_to(store: &dyn Store, slug: &str) -> Vec<String> {
    store
        .read_discussion(slug)
        .map(|doc| promoted_to_in(&doc.text))
        .unwrap_or_default()
}

/// The `promoted_to` accumulator of one record's text, in frontmatter order —
/// for callers that already hold the exact document (live or archived) and
/// must not let a reused slug's live record answer for its archived namesake.
pub fn promoted_to_in(text: &str) -> Vec<String> {
    frontmatter_value(text, "promoted_to")
        .map(|v| {
            v.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Whether one record's text holds a real Conclusion (the scaffold's
/// placeholder comment does not count) — the text-level twin of
/// [`discussion_concluded`].
pub fn concluded_in(text: &str) -> bool {
    conclusion_body(text).is_some()
}

/// Whether one record's text carries the staged-spin-out hold flag (frontmatter
/// `hold: true`) — the text-level twin of [`discussion_held`], shaped like
/// [`concluded_in`]. Only the exact value `true` counts. Module-private: the
/// archive guard and the conclude closing step both go through [`discussion_held`].
fn held_in(text: &str) -> bool {
    frontmatter_value(text, "hold").is_some_and(|v| v == "true")
}

/// Whether a discussion asked to stay live until its next spin-out — the single
/// contract point shared by the archive co-archival guard and the conclude closing
/// step, shaped like [`discussion_concluded`]. A missing or unreadable record counts
/// as not held (the concluded guard already keeps doubtful records live).
pub fn discussion_held(store: &dyn Store, slug: &str) -> bool {
    store.read_discussion(slug).is_some_and(|doc| held_in(&doc.text))
}

/// Archive a live discussion under its creation date. Returns the archived
/// file name, or `None` when no live discussion exists. Same-day name
/// collisions are resolved by the store so co-archival never fails on a
/// reused slug.
pub fn archive_discussion(store: &dyn Store, slug: &str) -> Result<Option<String>> {
    let Some(text) = store.read_live_discussion(slug) else {
        return Ok(None);
    };
    let created = frontmatter_value(&text, "created")
        .filter(|c| !c.is_empty())
        .unwrap_or_else(util::today);
    store.archive_discussion(slug, &created)
}

/// Delete a live discussion outright — the exit for a record that turned out not to be
/// needed. Refuses once rounds exist (unless `force`): a discussion that examined real
/// trade-offs should keep its reasoning via `conclude` + `archive` instead.
pub fn discard_discussion(store: &dyn Store, slug: &str, force: bool) -> Result<()> {
    let Some(text) = store.read_live_discussion(slug) else {
        if store.archived_discussion_exists(slug) {
            bail!("discussion '{slug}' is archived — archived records are kept, not discarded");
        }
        bail!("discussion '{slug}' not found");
    };
    let rounds = round_traces(&text);
    if rounds > 0 && !force {
        // Typed refusal: same frozen text, classified `refused` by the command layer.
        return Err(crate::command::Refusal(format!(
            "discussion '{slug}' has {rounds} recorded round(s) — `conclude` + `archive` keeps the reasoning; pass --force to delete anyway"
        ))
        .into());
    }
    store.delete_live_discussion(slug)?;
    Ok(())
}

/// Outcome of [`conclude`]: the restale-flagged change names, whether the closing
/// step auto-archived the record (its spun-out changes had all left the in-flight set),
/// and — when the closing archive step failed — the reason. The failure rides in the
/// outcome instead of an `Err` so the conclusion and restale writes stay committed on
/// every store (a remote Unit of Work would discard them on `Err`); the caller turns
/// `closing_error` into its own non-zero exit.
pub struct ConcludeOutcome {
    pub restale_flagged: Vec<String>,
    pub auto_archived: bool,
    pub closing_error: Option<String>,
    /// Whether the record carries the hold flag after this write.
    pub held: bool,
}

/// Write the conclusion into the `## Conclusion` section (replacing the placeholder — or a
/// previous conclusion, so a revised conclusion stays a single section) and mark the
/// discussion concluded.
pub fn conclude(
    store: &dyn Store,
    slug: &str,
    content: &str,
    hold: bool,
) -> Result<ConcludeOutcome> {
    ensure_content(content)?;
    let content = escape_colliding_lines(content);
    let mut text = load_live(store, slug)?;
    // Flip status: open -> concluded in frontmatter. A promoted discussion (status:
    // promoted) has no "status: open" to match, so a re-conclude preserves promoted.
    text = text.replacen("status: open", "status: concluded", 1);
    text = match replace_section(&text, "Conclusion", &content) {
        Some(t) => t,
        None => {
            // Pre-scaffold document: append the section.
            if !text.ends_with('\n') {
                text.push('\n');
            }
            format!("{text}\n## Conclusion\n\n{}\n", content.trim_end())
        }
    };
    // The hold flag rides the same write as the conclusion, so no half state can
    // survive a failure. Concluding without --hold restates the intent: an existing
    // flag is dropped. A record with nowhere to put the flag refuses `--hold` outright
    // rather than dropping it silently; without `--hold` it concludes as before.
    text = match set_frontmatter_line(&text, "hold", hold.then_some("true")) {
        Some(t) => t,
        None if hold => bail!("discussion '{slug}' has no frontmatter — cannot hold it live"),
        None => text,
    };
    let held = held_in(&text);
    store.write_live_discussion(slug, &text)?;
    // Re-concluding an already-reflected discussion (promoted_to non-empty) flags each
    // of its active changes as stale against the new conclusion. Returns the flagged
    // change names for the CLI to report; empty when nothing was reflected yet.
    let restale_flagged = stamp_restale(store, slug, &text)?;
    // Closing step: a spun-out discussion whose changes have all left the in-flight set
    // has no future change archive left to co-archive it, so conclude closes the record
    // itself. Corrupt change metadata fails closed (the same discipline as `link`): a
    // change whose references cannot be read counts as still referencing, so a doubtful
    // record stays live rather than being mis-archived. A failed archive step rides in
    // `closing_error` (see [`ConcludeOutcome`]) — the caller recovers with a plain
    // `discuss archive`. A record concluded with `--hold` still owes a change that does
    // not exist yet, so the closing step never fires on it: the flag's next spin-out
    // clears it, and that change's archive co-archives the record.
    let still_referenced = crate::model::list_changes(store).iter().any(|c| {
        c.meta_error.is_some() || c.meta.from_discussions().iter().any(|s| s == slug)
    });
    let mut closing_error = None;
    let auto_archived = if !still_referenced && !held && !promoted_to(store, slug).is_empty() {
        match archive_discussion(store, slug) {
            Ok(moved) => moved.is_some(),
            Err(e) => {
                closing_error = Some(e.to_string());
                false
            }
        }
    } else {
        false
    };
    Ok(ConcludeOutcome { restale_flagged, auto_archived, closing_error, held })
}

#[cfg(test)]
mod tests {
    use crate::store::Store;
    use crate::teststore::TestStore;

    /// A scaffolded discussion document with a written conclusion.
    fn concluded_doc(slug: &str, topic: &str, decision: &str) -> String {
        format!(
            "---\ntopic: {topic}\nslug: {slug}\nstatus: concluded\ncreated: 2026-01-02\n---\n\n\
             # Discussion: {topic}\n\n\
             ## Context\n\nFixture context.\n\n\
             ## Rounds\n\n### Round 1 — assumptions (2026-01-02)\n\n**Focus**: scope\n\n\
             ## Conclusion\n\n**Decision**: {decision}\n"
        )
    }

    /// A scaffolded discussion whose conclusion is still the placeholder comment.
    fn open_doc(slug: &str, topic: &str) -> String {
        format!(
            "---\ntopic: {topic}\nslug: {slug}\nstatus: open\ncreated: 2026-01-02\n---\n\n\
             # Discussion: {topic}\n\n\
             ## Context\n\nFixture context.\n\n\
             ## Rounds\n\n\
             ## Conclusion\n\n<!-- Written by `speclink discuss conclude` -->\n"
        )
    }

    /// A promoted discussion (spin-out already recorded) whose conclusion is still the
    /// placeholder comment — the mid-discussion spin-out state.
    fn promoted_unconcluded_doc(slug: &str, promoted_to: &str) -> String {
        format!(
            "---\ntopic: {slug}\nslug: {slug}\nstatus: promoted\npromoted_to: {promoted_to}\ncreated: 2026-01-02\n---\n\n\
             # Discussion: {slug}\n\n\
             ## Context\n\nFixture context.\n\n\
             ## Rounds\n\n\
             ## Conclusion\n\n<!-- Written by `speclink discuss conclude` -->\n"
        )
    }

    // --- conclude 閉環（conclusion-gated-discussion-archive）---

    #[test]
    fn conclude_auto_archives_when_all_promoted_changes_are_archived() {
        // promoted_to 非空、無在途變更引用 → conclude 順手封存，結論隨記錄進封存區。
        let store = TestStore::with_live_discussion("alpha", &promoted_unconcluded_doc("alpha", "cut"));

        let outcome = super::conclude(&store, "alpha", "**Decision**: done", false).unwrap();

        assert!(outcome.auto_archived, "outcome carries the auto-archive fact");
        assert!(!store.live_discussion_exists("alpha"), "record leaves the live set");
        assert!(store.archived_discussion_exists("alpha"));
        let archived = store.archived_discussions.borrow().get("alpha").cloned().unwrap();
        assert!(archived.contains("**Decision**: done"), "conclusion travels into the archive");
        assert!(archived.contains("status: promoted"), "promoted status is preserved");
    }

    #[test]
    fn conclude_leaves_record_live_while_promoted_change_in_flight() {
        // 仍有在途變更引用 → 只寫結論，不封存。
        let store = TestStore::with_meta(
            "cut",
            "schema: spec-driven\ncreated: 2026-01-02\nfrom_discussion: alpha\n",
        );
        store
            .discussions
            .borrow_mut()
            .insert("alpha".into(), promoted_unconcluded_doc("alpha", "cut"));

        let outcome = super::conclude(&store, "alpha", "**Decision**: done", false).unwrap();

        assert!(!outcome.auto_archived);
        assert!(store.live_discussion_exists("alpha"), "record stays live");
        assert!(!store.archived_discussion_exists("alpha"));
        assert!(store.discussion("alpha").contains("**Decision**: done"));
    }

    #[test]
    fn conclude_without_promotion_keeps_existing_behavior() {
        // promoted_to 缺席 → 行為不變：status 轉 concluded、記錄留在途。
        let store = TestStore::with_live_discussion("alpha", &open_doc("alpha", "Alpha"));

        let outcome = super::conclude(&store, "alpha", "**Decision**: done", false).unwrap();

        assert!(!outcome.auto_archived);
        assert!(store.live_discussion_exists("alpha"));
        assert!(!store.archived_discussion_exists("alpha"));
        assert!(store.discussion("alpha").contains("status: concluded"));
    }

    #[test]
    fn conclude_archive_step_failure_keeps_conclusion_and_outcome() {
        // 閉環封存步失敗 → 結論與 restale 結果照常回傳（Ok），失敗原因入
        // closing_error 由呼叫端呈現；結論寫入不回滾（已結論、仍在途）。
        // 命令層回 Ok 也讓 remote 的 Unit of Work 照常 commit——「不回滾」
        // 在本機與 remote 同語意。
        let store = TestStore::with_live_discussion("alpha", &promoted_unconcluded_doc("alpha", "cut"));
        *store.fail_archive_discussion.borrow_mut() = true;

        let outcome = super::conclude(&store, "alpha", "**Decision**: done", false).unwrap();

        assert!(!outcome.auto_archived);
        assert!(
            outcome.closing_error.as_deref().unwrap_or("").contains("simulated"),
            "closing failure reason travels in the outcome: {:?}",
            outcome.closing_error
        );
        assert!(store.live_discussion_exists("alpha"), "record stays live");
        assert!(
            store.discussion("alpha").contains("**Decision**: done"),
            "conclusion is not rolled back"
        );
    }

    #[test]
    fn conclude_closing_step_fails_closed_on_corrupt_change_meta() {
        // 壞 meta fail-closed（與 link 的紀律一致）：在途變更的 .openspec.yaml
        // 解析失敗時，讀不出它引用誰——視同仍引用，不誤封存討論。
        const BAD: &str = ": : :\n\t bad yaml [unclosed\n";
        let store = TestStore::with_meta("broken-cut", BAD);
        store
            .discussions
            .borrow_mut()
            .insert("alpha".into(), promoted_unconcluded_doc("alpha", "broken-cut"));

        let outcome = super::conclude(&store, "alpha", "**Decision**: done", false).unwrap();

        assert!(!outcome.auto_archived, "corrupt in-flight meta blocks the closing step");
        assert!(outcome.closing_error.is_none());
        assert!(store.live_discussion_exists("alpha"), "record stays live");
        assert!(!store.archived_discussion_exists("alpha"));
    }

    // --- hold 旗標（discussion-spinout-hold）---

    /// 帶 `hold: true` 的已轉出討論——分期立案（下一刀還沒建立）的在途狀態。
    fn held_promoted_doc(slug: &str, promoted_to: &str) -> String {
        format!(
            "---\ntopic: {slug}\nslug: {slug}\nstatus: promoted\npromoted_to: {promoted_to}\ncreated: 2026-01-02\nhold: true\n---\n\n\
             # Discussion: {slug}\n\n\
             ## Context\n\nFixture context.\n\n\
             ## Rounds\n\n\
             ## Conclusion\n\n<!-- Written by `speclink discuss conclude` -->\n"
        )
    }

    #[test]
    fn conclude_with_hold_writes_the_flag_and_keeps_status_rules() {
        // 帶 hold：旗標入 frontmatter，status 轉換規則（open -> concluded）不變。
        let store = TestStore::with_live_discussion("alpha", &open_doc("alpha", "Alpha"));

        let outcome = super::conclude(&store, "alpha", "**Decision**: done", true).unwrap();

        assert!(outcome.held, "outcome 記錄本次寫入後帶旗標");
        let text = store.discussion("alpha");
        assert!(super::held_in(&text), "frontmatter 帶 hold: true");
        assert!(super::discussion_held(&store, "alpha"));
        assert!(text.contains("status: concluded"), "status 轉換規則不變");
        assert!(text.contains("**Decision**: done"), "結論與旗標同一次落盤");
    }

    #[test]
    fn conclude_without_hold_removes_an_existing_flag() {
        // 不帶 hold 的再次 conclude＝重述意圖：既有旗標行消失。
        // 在途變更引用本討論，閉環不觸發，記錄留在途可供檢視。
        let store = TestStore::with_meta(
            "cut",
            "schema: spec-driven\ncreated: 2026-01-02\nfrom_discussion: alpha\n",
        );
        store.discussions.borrow_mut().insert("alpha".into(), held_promoted_doc("alpha", "cut"));

        let outcome = super::conclude(&store, "alpha", "**Decision**: redone", false).unwrap();

        assert!(!outcome.held);
        let text = store.discussion("alpha");
        assert!(!super::held_in(&text), "旗標行被移除");
        assert!(!text.contains("hold: true"), "整行消失，不留殘句");
        assert!(text.contains("status: promoted"), "promoted 狀態保持");
    }

    #[test]
    fn mark_promoted_clears_the_hold_flag() {
        // 下一刀轉出＝旗標的償還：promoted_to 累加、hold 行消失。
        let store = TestStore::with_live_discussion("alpha", &held_promoted_doc("alpha", "cut-a"));

        super::mark_promoted(&store, "alpha", "cut-b").unwrap();

        let text = store.discussion("alpha");
        assert!(text.contains("promoted_to: cut-a, cut-b"), "累加下一刀");
        assert!(!super::held_in(&text), "旗標由轉出清除");
        assert!(!text.contains("hold: true"));
    }

    #[test]
    fn link_leaves_the_hold_flag_untouched_and_seal_clears_it() {
        // link 對討論記錄逐位元不變（不清旗標）；補標的 seal 才清。
        let doc = held_promoted_doc("alpha", "cut-a");
        let store = TestStore::with_meta("cut-b", "schema: spec-driven\ncreated: 2026-01-02\n");
        store.discussions.borrow_mut().insert("alpha".into(), doc.clone());

        super::link(&store, "alpha", "cut-b").unwrap();
        assert_eq!(store.discussion("alpha"), doc, "link 不改討論記錄");

        super::seal(&store, "alpha", "cut-b").unwrap();
        let text = store.discussion("alpha");
        assert!(text.contains("promoted_to: cut-a, cut-b"));
        assert!(!super::held_in(&text), "seal 經 mark_promoted 清旗標");
    }

    #[test]
    fn mark_promoted_keeps_the_hold_flag_when_nothing_accumulates() {
        // 冪等分支（promoted_to 已含該變更名）不是新刀：re-ingest 舊變更的 seal
        // 不得清旗標，否則分期第二刀的來源記錄會被舊變更的封存掃走。
        let store = TestStore::with_live_discussion("alpha", &held_promoted_doc("alpha", "cut-a"));

        super::mark_promoted(&store, "alpha", "cut-a").unwrap();

        let text = store.discussion("alpha");
        assert!(super::held_in(&text), "沒有新刀累加，旗標保留");
        assert!(text.contains("promoted_to: cut-a\n"), "promoted_to 不變");
    }

    #[test]
    fn conclude_with_hold_rejects_a_record_without_frontmatter() {
        // 無 frontmatter 放不下旗標：帶 hold 明確報錯、不落盤；不帶 hold 沿
        // pre-scaffold 既有路徑照常結論。
        let doc = "# Discussion: bare\n\n## Rounds\n";
        let store = TestStore::with_live_discussion("bare", doc);

        assert!(super::conclude(&store, "bare", "**Decision**: x", true).is_err());
        assert_eq!(store.discussion("bare"), doc, "拒絕時記錄逐位元不變");

        let outcome = super::conclude(&store, "bare", "**Decision**: x", false).unwrap();
        assert!(!outcome.held);
        assert!(store.discussion("bare").contains("## Conclusion"));
    }

    #[test]
    fn conclude_rewrites_a_hand_edited_hold_line_in_place() {
        // 以 key 為單位改寫：手改成 hold: false 的行被原位換成 true（不另插一行），
        // 不帶 hold 時任何 hold: 行都移除。讀（frontmatter_value）寫兩邊看法一致。
        let store = TestStore::with_meta(
            "cut",
            "schema: spec-driven\ncreated: 2026-01-02\nfrom_discussion: alpha\n",
        );
        let doc = held_promoted_doc("alpha", "cut").replacen("hold: true\n", "hold: false\n", 1);
        store.discussions.borrow_mut().insert("alpha".into(), doc);

        let outcome = super::conclude(&store, "alpha", "**Decision**: later", true).unwrap();
        assert!(outcome.held);
        let text = store.discussion("alpha");
        assert_eq!(text.matches("\nhold:").count(), 1, "只有一行 hold:");
        assert!(super::held_in(&text));

        store.discussions.borrow_mut().insert(
            "alpha".into(),
            held_promoted_doc("alpha", "cut").replacen("hold: true\n", "hold: yes\n", 1),
        );
        let outcome = super::conclude(&store, "alpha", "**Decision**: done", false).unwrap();
        assert!(!outcome.held);
        assert!(!store.discussion("alpha").contains("\nhold:"), "任何 hold: 行都移除");
    }

    #[test]
    fn conclude_with_hold_follows_the_record_line_ending() {
        // CRLF 記錄：插入的旗標行沿該檔行尾，不混排。
        let doc = open_doc("alpha", "Alpha").replace('\n', "\r\n");
        let store = TestStore::with_live_discussion("alpha", &doc);

        let outcome = super::conclude(&store, "alpha", "**Decision**: done", true).unwrap();

        assert!(outcome.held);
        let text = store.discussion("alpha");
        assert!(text.contains("hold: true\r\n---"), "旗標行以 CRLF 收尾: {text:?}");
        assert!(super::held_in(&text));
    }

    #[test]
    fn frontmatter_line_surgery_on_unclosed_frontmatter_matches_the_reader() {
        // 未閉合 frontmatter（缺尾 ---）：讀端 frontmatter_value 把整檔當 frontmatter，
        // 寫端要同樣寬鬆——既有行原位代換、移除照做；只有「要新插一行卻找不到尾」才拒絕。
        let unclosed = "---\ntopic: x\nslug: x\nstatus: open\nboard_rank: b\nhold: true\n";
        let store = TestStore::with_live_discussion("x", unclosed);

        super::set_board_rank(&store, "x", "n").unwrap();
        assert_eq!(store.discussion("x"), unclosed.replacen("board_rank: b", "board_rank: n", 1));

        let outcome = super::conclude(&store, "x", "**Decision**: done", false).unwrap();
        assert!(!outcome.held);
        assert!(!store.discussion("x").contains("hold:"), "未閉合仍能移除旗標");
        assert!(!super::discussion_held(&store, "x"), "讀寫兩端看法一致");

        let bare = "---\ntopic: y\nslug: y\nstatus: open\n";
        let store = TestStore::with_live_discussion("y", bare);
        assert!(super::conclude(&store, "y", "**Decision**: x", true).is_err(), "沒有尾行可插");
        assert_eq!(store.discussion("y"), bare);
    }

    #[test]
    fn mark_promoted_keeps_the_hold_flag_when_promoted_to_did_not_land() {
        // 旗標清除以 promoted_to 真的寫進去為準。前提用的是既知缺口，不是目標行為：
        // mark_promoted 以 "status: promoted\n" 做 replacen，CRLF 記錄落空、promoted_to
        // 根本沒落地——那是 promote 路徑本來就有的 CRLF 破口，本測試只釘住「沒累加就不清
        // 旗標」這一條，不背書 promoted_to 落空本身。
        let doc = open_doc("alpha", "Alpha")
            .replacen("created: 2026-01-02\n", "created: 2026-01-02\nhold: true\n", 1)
            .replace('\n', "\r\n");
        let store = TestStore::with_live_discussion("alpha", &doc);

        super::mark_promoted(&store, "alpha", "cut-b").unwrap();

        let text = store.discussion("alpha");
        assert!(!text.contains("promoted_to:"), "前提：promoted_to 沒寫進去");
        assert!(super::held_in(&text), "沒有累加就不清旗標");
    }

    #[test]
    fn conclude_with_hold_skips_the_closing_archive() {
        // 閉環條件（promoted_to 非空、無在途引用）成立，但帶 hold → 不封存、留在途。
        let store = TestStore::with_live_discussion("alpha", &promoted_unconcluded_doc("alpha", "cut"));

        let outcome = super::conclude(&store, "alpha", "**Decision**: cut-b later", true).unwrap();

        assert!(!outcome.auto_archived, "帶 hold 必然不閉環");
        assert!(outcome.held);
        assert!(outcome.closing_error.is_none());
        assert!(store.live_discussion_exists("alpha"), "記錄留在 live 集合");
        assert!(!store.archived_discussion_exists("alpha"));
        assert!(super::held_in(&store.discussion("alpha")));
    }

    #[test]
    fn conclude_with_hold_does_not_duplicate_the_flag_line() {
        // 重複帶 hold：旗標行只有一條。
        let store = TestStore::with_meta(
            "cut",
            "schema: spec-driven\ncreated: 2026-01-02\nfrom_discussion: alpha\n",
        );
        store.discussions.borrow_mut().insert("alpha".into(), held_promoted_doc("alpha", "cut"));

        let outcome = super::conclude(&store, "alpha", "**Decision**: still deferred", true).unwrap();

        assert!(outcome.held);
        let text = store.discussion("alpha");
        assert_eq!(text.matches("hold: true").count(), 1, "不產生第二行");
    }

    // --- 空內容 guard（discuss-content-guard；拒絕靜默寫入空區段） ---

    #[test]
    fn add_round_rejects_empty_content() {
        let doc = open_doc("alpha", "Alpha");
        let store = TestStore::with_live_discussion("alpha", &doc);
        assert!(super::add_round(&store, "alpha", "assumptions", "").is_err());
        assert!(super::add_round(&store, "alpha", "assumptions", "   \n\t ").is_err());
        assert_eq!(store.discussion("alpha"), doc, "空內容不得改動記錄");
    }

    #[test]
    fn conclude_rejects_empty_content_and_keeps_status() {
        let doc = open_doc("alpha", "Alpha");
        let store = TestStore::with_live_discussion("alpha", &doc);
        assert!(super::conclude(&store, "alpha", "", false).is_err());
        assert!(super::conclude(&store, "alpha", "  \n ", false).is_err());
        assert_eq!(
            store.discussion("alpha"),
            doc,
            "空 conclude 不得翻狀態或改動記錄"
        );
    }

    #[test]
    fn set_context_rejects_empty_content() {
        let doc = open_doc("alpha", "Alpha");
        let store = TestStore::with_live_discussion("alpha", &doc);
        assert!(super::set_context(&store, "alpha", "").is_err());
        assert!(super::set_context(&store, "alpha", "   ").is_err());
        assert_eq!(store.discussion("alpha"), doc, "空內容不得覆寫 Context");
    }

    // --- board_rank（看板排序欄位；desktop-card-reorder） ---

    #[test]
    fn board_rank_reads_frontmatter_only() {
        // 讀取限 frontmatter：本文出現「board_rank:」字樣不得誤讀。
        let store = TestStore::with_live_discussion("alpha", &open_doc("alpha", "Alpha"));
        assert!(super::board_rank(&store, "alpha").is_none());

        let with_rank = open_doc("alpha", "Alpha")
            .replacen("status: open\n", "status: open\nboard_rank: n\n", 1);
        let store2 = TestStore::with_live_discussion("alpha", &with_rank);
        assert_eq!(super::board_rank(&store2, "alpha").as_deref(), Some("n"));

        let body_decoy = open_doc("alpha", "Alpha") + "\nboard_rank: fake\n";
        let store3 = TestStore::with_live_discussion("alpha", &body_decoy);
        assert!(super::board_rank(&store3, "alpha").is_none());
    }

    #[test]
    fn set_board_rank_inserts_into_frontmatter_preserving_rest_verbatim() {
        // spec「meta 寫入路徑對 board_rank 互不破壞」討論側：插入 frontmatter
        // 尾端（closing --- 前），其餘內容逐位元組不變。
        let doc = open_doc("alpha", "Alpha");
        let store = TestStore::with_live_discussion("alpha", &doc);
        super::set_board_rank(&store, "alpha", "n").unwrap();
        let expected = doc.replacen(
            "created: 2026-01-02\n---\n",
            "created: 2026-01-02\nboard_rank: n\n---\n",
            1,
        );
        assert_eq!(store.discussion("alpha"), expected);
    }

    #[test]
    fn set_board_rank_replaces_existing_frontmatter_line_in_place() {
        let doc = open_doc("alpha", "Alpha")
            .replacen("status: open\n", "status: open\nboard_rank: b\n", 1);
        let store = TestStore::with_live_discussion("alpha", &doc);
        super::set_board_rank(&store, "alpha", "abn").unwrap();
        assert_eq!(
            store.discussion("alpha"),
            doc.replacen("board_rank: b\n", "board_rank: abn\n", 1)
        );
    }

    #[test]
    fn set_board_rank_rejects_invalid_values_and_non_live_records() {
        // 值驗證同變更側（僅小寫英文字母）；封存記錄不上看板、不可寫。
        let store = TestStore::with_live_discussion("alpha", &open_doc("alpha", "Alpha"));
        for bad in ["", "N", "a1", "a b", "a\nstatus: forged"] {
            assert!(
                super::set_board_rank(&store, "alpha", bad).is_err(),
                "invalid rank {bad:?} must be rejected"
            );
        }
        assert_eq!(store.discussion("alpha"), open_doc("alpha", "Alpha"), "no write on reject");

        let archived = TestStore::default();
        archived
            .archived_discussions
            .borrow_mut()
            .insert("old".to_string(), concluded_doc("old", "Old", "done"));
        assert!(super::set_board_rank(&archived, "old", "n").is_err());
        assert!(super::set_board_rank(&archived, "ghost", "n").is_err());
    }

    #[test]
    fn discussion_info_json_is_unchanged_by_board_rank() {
        // spec「board_rank 不進 CLI 輸出且既有輸出逐位元不變」討論側：
        // DiscussionInfo 不攜帶 rank（沿 promoted_to 的獨立讀取模式），
        // 含 rank 的記錄序列化結果與無 rank 時逐位元一致。
        let doc = open_doc("alpha", "Alpha");
        let with_rank = doc.replacen("status: open\n", "status: open\nboard_rank: n\n", 1);
        let info_of = |text: &str| {
            serde_json::to_string(&super::info_from_doc(&crate::store::DiscussionDoc {
                slug: "alpha".to_string(),
                text: text.to_string(),
                path: std::path::PathBuf::from("discussions/alpha.md"),
                archived: false,
            }))
            .unwrap()
        };
        let ranked_json = info_of(&with_rank);
        assert_eq!(ranked_json, info_of(&doc), "board_rank must not affect discuss list --json");
        assert!(!ranked_json.contains("board_rank") && !ranked_json.contains("boardRank"));
    }

    // --- promote flow (design D1) ---

    #[test]
    fn promote_rejects_missing_discussion() {
        let store = TestStore::default();
        let err = super::promote(&store, "ghost", None, None).unwrap_err();
        assert!(err.to_string().contains("not found"), "err: {err}");
    }

    #[test]
    fn promote_rejects_archived_discussion() {
        let store = TestStore::default();
        store
            .archived_discussions
            .borrow_mut()
            .insert("old-topic".to_string(), concluded_doc("old-topic", "Old", "done"));
        let err = super::promote(&store, "old-topic", None, None).unwrap_err();
        assert!(err.to_string().contains("archived"), "err: {err}");
        assert!(!store.change_exists("old-topic"), "no change may be created");
    }

    #[test]
    fn promote_derives_change_name_from_slug_by_default() {
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &concluded_doc("alpha-search", "Alpha search", "build alpha search"),
        );
        let outcome = super::promote(&store, "alpha-search", None, None).unwrap();
        assert_eq!(outcome.change, "alpha-search");
        assert!(store.change_exists("alpha-search"));
    }

    #[test]
    fn promote_uses_explicit_name_when_given() {
        let store = TestStore::with_live_discussion(
            "beta-cache",
            &concluded_doc("beta-cache", "Beta cache", "add cache layer"),
        );
        let outcome =
            super::promote(&store, "beta-cache", Some("cache-layer"), None).unwrap();
        assert_eq!(outcome.change, "cache-layer");
        assert!(store.change_exists("cache-layer"));
        assert!(!store.change_exists("beta-cache"));
    }

    #[test]
    fn promote_strips_archive_date_prefix_from_derived_name() {
        // Archive-style date prefixes are historical references, not active
        // change names — derivation normalizes them away (either form).
        let store = TestStore::with_live_discussion(
            "2026-07-06-retro",
            &concluded_doc("2026-07-06-retro", "Retro", "do the retro"),
        );
        let outcome = super::promote(&store, "2026-07-06-retro", None, None).unwrap();
        assert_eq!(outcome.change, "retro");

        let store2 = TestStore::with_live_discussion(
            "gamma-x",
            &concluded_doc("gamma-x", "Gamma x", "ship gamma"),
        );
        let outcome2 =
            super::promote(&store2, "gamma-x", Some("2026-01-02-gamma-cut"), None).unwrap();
        assert_eq!(outcome2.change, "gamma-cut");
    }

    #[test]
    fn promote_creates_change_with_from_discussion_meta() {
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &concluded_doc("alpha-search", "Alpha search", "build alpha search"),
        );
        super::promote(&store, "alpha-search", None, None).unwrap();
        let meta = store.meta("alpha-search");
        assert!(meta.starts_with("schema: spec-driven\ncreated: "), "meta: {meta}");
        assert!(meta.contains("from_discussion: alpha-search\n"), "meta: {meta}");
    }

    #[test]
    fn promote_prefills_proposal_why_from_conclusion() {
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &concluded_doc("alpha-search", "Alpha search", "build alpha search"),
        );
        super::promote(&store, "alpha-search", None, None).unwrap();
        let proposal = store.read_artifact("alpha-search", "proposal.md").unwrap();
        assert_eq!(
            proposal,
            "## Why\n\n**Decision**: build alpha search\n\n## What Changes\n\n<!-- TBD: derive from the discussion -->\n\n## Capabilities\n\n### New Capabilities\n\n<!-- TBD -->\n\n## Impact\n\n<!-- TBD -->\n"
        );
    }

    #[test]
    fn promote_prefills_topic_when_no_conclusion() {
        // Placeholder-only conclusion → the topic is the Why fallback.
        let store =
            TestStore::with_live_discussion("open-one", &open_doc("open-one", "Open topic"));
        super::promote(&store, "open-one", None, None).unwrap();
        let proposal = store.read_artifact("open-one", "proposal.md").unwrap();
        assert!(proposal.starts_with("## Why\n\nOpen topic\n"), "proposal: {proposal}");
    }

    #[test]
    fn promote_marks_promoted_and_accumulates_on_fan_out() {
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &concluded_doc("alpha-search", "Alpha search", "build alpha search"),
        );
        super::promote(&store, "alpha-search", None, None).unwrap();
        let text = store.discussion("alpha-search");
        assert!(text.contains("status: promoted\n"), "text: {text}");
        assert!(text.contains("promoted_to: alpha-search\n"), "text: {text}");

        // Second cut: promoted_to becomes a comma-separated accumulator.
        super::promote(&store, "alpha-search", Some("second-cut"), None).unwrap();
        let text = store.discussion("alpha-search");
        assert!(text.contains("promoted_to: alpha-search, second-cut\n"), "text: {text}");
    }

    #[test]
    fn promote_fails_when_change_already_exists_and_leaves_discussion_untouched() {
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &concluded_doc("alpha-search", "Alpha search", "build alpha search"),
        );
        store.metas.borrow_mut().insert("alpha-search".to_string(), "schema: spec-driven\n".to_string());
        let before = store.discussion("alpha-search");
        let err = super::promote(&store, "alpha-search", None, None).unwrap_err();
        assert!(err.to_string().contains("already exists"), "err: {err}");
        assert_eq!(store.discussion("alpha-search"), before, "discussion must not be marked");
    }

    // --- promoted_to query (design D2) ---

    #[test]
    fn promoted_to_absent_yields_empty() {
        let store =
            TestStore::with_live_discussion("open-one", &open_doc("open-one", "Open topic"));
        assert!(super::promoted_to(&store, "open-one").is_empty());
        assert!(super::promoted_to(&store, "no-such-slug").is_empty());
    }

    #[test]
    fn promoted_to_single_value() {
        let mut doc = concluded_doc("alpha-search", "Alpha search", "x");
        doc = doc.replacen(
            "status: concluded\n",
            "status: promoted\npromoted_to: first-cut\n",
            1,
        );
        let store = TestStore::with_live_discussion("alpha-search", &doc);
        assert_eq!(super::promoted_to(&store, "alpha-search"), vec!["first-cut".to_string()]);
    }

    #[test]
    fn promoted_to_comma_accumulated_values() {
        let mut doc = concluded_doc("alpha-search", "Alpha search", "x");
        doc = doc.replacen(
            "status: concluded\n",
            "status: promoted\npromoted_to: first-cut, second-cut\n",
            1,
        );
        let store = TestStore::with_live_discussion("alpha-search", &doc);
        assert_eq!(
            super::promoted_to(&store, "alpha-search"),
            vec!["first-cut".to_string(), "second-cut".to_string()]
        );
    }

    #[test]
    fn promoted_to_reads_archived_records_too() {
        // The archived page needs the fan-out list for auto-archived discussions.
        let mut doc = concluded_doc("done-topic", "Done", "x");
        doc = doc.replacen(
            "status: concluded\n",
            "status: promoted\npromoted_to: only-cut\n",
            1,
        );
        let store = TestStore::default();
        store.archived_discussions.borrow_mut().insert("done-topic".to_string(), doc);
        assert_eq!(super::promoted_to(&store, "done-topic"), vec!["only-cut".to_string()]);
    }

    // --- unlink on discard（spec「討論隨變更廢棄解鏈」；design D2） ---

    /// concluded_doc 提升為 promoted，promoted_to 設為指定清單。
    fn promoted_concluded(slug: &str, topic: &str, decision: &str, to: &str) -> String {
        concluded_doc(slug, topic, decision).replacen(
            "status: concluded\n",
            &format!("status: promoted\npromoted_to: {to}\n"),
            1,
        )
    }

    #[test]
    fn unlink_reverts_to_concluded_when_last_link_dies() {
        // spec Example「回退前後的 frontmatter」＋「最後連結死亡回退 concluded」：
        // 唯一值移除 → promoted_to 行消失、status 回 concluded；Context/Rounds/Conclusion
        // 逐位元不變（回退後 == 原 concluded 記錄）。
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &promoted_concluded("alpha-search", "Alpha search", "build alpha search", "cut-a"),
        );
        let reverted = super::unlink_discarded(&store, "alpha-search", "cut-a").unwrap();
        assert_eq!(reverted.as_deref(), Some("concluded"));
        assert_eq!(
            store.discussion("alpha-search"),
            concluded_doc("alpha-search", "Alpha search", "build alpha search"),
            "promoted_to 行消失、status 回 concluded、其餘逐位元不變"
        );
    }

    #[test]
    fn unlink_shrinks_list_and_keeps_promoted_when_others_remain() {
        // spec「仍有其他變更時維持 promoted」：多值僅縮減、status 維持 promoted。
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &promoted_concluded("alpha-search", "Alpha search", "x", "cut-a, cut-b"),
        );
        let reverted = super::unlink_discarded(&store, "alpha-search", "cut-a").unwrap();
        assert_eq!(reverted.as_deref(), Some("promoted"));
        assert_eq!(
            store.discussion("alpha-search"),
            promoted_concluded("alpha-search", "Alpha search", "x", "cut-b"),
            "移除 cut-a、保留 cut-b、status 維持 promoted"
        );
    }

    #[test]
    fn unlink_reverts_to_open_when_no_conclusion() {
        // spec「無結論的討論回退 open」：Conclusion 為空的 open 討論經 link 後廢棄 → 回 open。
        let raised = open_doc("open-one", "Open topic").replacen(
            "status: open\n",
            "status: promoted\npromoted_to: cut\n",
            1,
        );
        let store = TestStore::with_live_discussion("open-one", &raised);
        let reverted = super::unlink_discarded(&store, "open-one", "cut").unwrap();
        assert_eq!(reverted.as_deref(), Some("open"));
        assert_eq!(
            store.discussion("open-one"),
            open_doc("open-one", "Open topic"),
            "promoted_to 行消失、status 回 open、其餘逐位元不變"
        );
    }

    #[test]
    fn unlink_skips_missing_record_without_error() {
        // spec「缺失記錄跳過」：無 live 記錄（不存在或僅存於 archive）→ Ok(None)、不失敗。
        let empty = TestStore::default();
        assert_eq!(super::unlink_discarded(&empty, "ghost", "cut").unwrap(), None);

        let archived = TestStore::default();
        archived
            .archived_discussions
            .borrow_mut()
            .insert("old".into(), promoted_concluded("old", "Old", "x", "cut"));
        assert_eq!(super::unlink_discarded(&archived, "old", "cut").unwrap(), None);
    }

    #[test]
    fn unlink_is_idempotent_on_already_unlinked_record() {
        // spec「對已解鏈的討論重跑冪等」：重跑對 promoted_to 已無該名的記錄 → Ok(None)、不改檔。
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &promoted_concluded("alpha-search", "Alpha search", "x", "cut-a"),
        );
        super::unlink_discarded(&store, "alpha-search", "cut-a").unwrap();
        let after_first = store.discussion("alpha-search");
        let rerun = super::unlink_discarded(&store, "alpha-search", "cut-a").unwrap();
        assert_eq!(rerun, None, "已解鏈記錄重跑不回報狀態");
        assert_eq!(store.discussion("alpha-search"), after_first, "重跑不改檔");
    }

    #[test]
    fn unlink_ignores_a_change_that_was_never_linked() {
        // 冪等的另一面：promoted_to 有值但不含目標變更名 → 不動、不失敗、不回報。
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &promoted_concluded("alpha-search", "Alpha search", "x", "cut-a, cut-b"),
        );
        let before = store.discussion("alpha-search");
        assert_eq!(super::unlink_discarded(&store, "alpha-search", "cut-z").unwrap(), None);
        assert_eq!(store.discussion("alpha-search"), before);
    }

    // --- link flow（spec「討論以 link 動詞併入既有變更」；design D1–D4） ---

    #[test]
    fn link_writes_change_meta_and_leaves_discussion_untouched() {
        // link 只鑄變更側鏈：變更 meta 增寫 from_discussion，討論記錄逐位元不變
        // （「已轉出」標記移交 seal）。
        let doc = concluded_doc("alpha-search", "Alpha search", "build alpha search");
        let store = TestStore::with_live_discussion("alpha-search", &doc);
        store
            .metas
            .borrow_mut()
            .insert("existing-cut".into(), "schema: spec-driven\ncreated: 2026-01-03\n".into());
        super::link(&store, "alpha-search", "existing-cut").unwrap();
        let meta = store.meta("existing-cut");
        assert!(meta.contains("from_discussion: alpha-search\n"), "meta: {meta}");
        assert_eq!(store.discussion("alpha-search"), doc, "討論逐位元不變（link 不再標記 promoted）");
    }

    #[test]
    fn link_accepts_open_discussion_without_marking() {
        // 前置條件與 promote 一致：open 討論也可併入；但 link 不翻狀態，討論仍 open。
        let doc = open_doc("open-one", "Open topic");
        let store = TestStore::with_live_discussion("open-one", &doc);
        store.metas.borrow_mut().insert("cut".into(), "schema: spec-driven\n".into());
        super::link(&store, "open-one", "cut").unwrap();
        assert!(store.meta("cut").contains("from_discussion: open-one\n"));
        assert_eq!(store.discussion("open-one"), doc, "討論仍 open、逐位元不變");
    }

    #[test]
    fn link_rejects_missing_discussion_without_writes() {
        let store = TestStore::default();
        store.metas.borrow_mut().insert("cut".into(), "schema: spec-driven\n".into());
        let err = super::link(&store, "ghost", "cut").unwrap_err();
        assert!(err.to_string().contains("not found"), "err: {err}");
        assert_eq!(store.meta("cut"), "schema: spec-driven\n", "change meta must be untouched");
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn link_rejects_archived_discussion_without_writes() {
        let store = TestStore::default();
        store
            .archived_discussions
            .borrow_mut()
            .insert("old-topic".into(), concluded_doc("old-topic", "Old", "done"));
        store.metas.borrow_mut().insert("cut".into(), "schema: spec-driven\n".into());
        let err = super::link(&store, "old-topic", "cut").unwrap_err();
        assert!(err.to_string().contains("archived"), "err: {err}");
        assert_eq!(store.meta("cut"), "schema: spec-driven\n", "change meta must be untouched");
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn link_rejects_missing_change_without_discussion_write() {
        let doc = concluded_doc("alpha-search", "Alpha search", "x");
        let store = TestStore::with_live_discussion("alpha-search", &doc);
        let err = super::link(&store, "alpha-search", "no-such-change").unwrap_err();
        assert!(err.to_string().contains("not found"), "err: {err}");
        assert_eq!(store.discussion("alpha-search"), doc, "discussion must be untouched");
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn link_appends_to_from_discussion_when_change_already_linked() {
        // spec「出身自討論的變更再併入新討論」：change meta 的 from_discussion 於既有值
        // 尾端累加本 slug、既有值保留；本討論標 promoted；先前連結的討論記錄逐位元不變。
        let doc = concluded_doc("beta-cache", "Beta cache", "x");
        let store = TestStore::with_live_discussion("beta-cache", &doc);
        let other = concluded_doc("other-topic", "Other", "y").replacen(
            "status: concluded\n",
            "status: promoted\npromoted_to: cut\n",
            1,
        );
        store.discussions.borrow_mut().insert("other-topic".into(), other.clone());
        store
            .metas
            .borrow_mut()
            .insert("cut".into(), "schema: spec-driven\nfrom_discussion: other-topic\n".into());
        super::link(&store, "beta-cache", "cut").unwrap();
        let meta = store.meta("cut");
        assert!(meta.contains("from_discussion: other-topic, beta-cache\n"), "meta: {meta}");
        assert_eq!(store.discussion("beta-cache"), doc, "本討論逐位元不變（link 不標記）");
        assert_eq!(store.discussion("other-topic"), other, "prior discussion untouched");
    }

    #[test]
    fn link_is_idempotent_when_slug_already_in_from_discussion_list() {
        // spec「同一組合重跑為冪等」（該討論僅為 from_discussion 清單其中一員）：
        // change 側不再寫、討論側改寫等值內容。
        let store = TestStore::with_live_discussion(
            "beta-cache",
            &concluded_doc("beta-cache", "Beta cache", "x"),
        );
        store.metas.borrow_mut().insert(
            "cut".into(),
            "schema: spec-driven\nfrom_discussion: alpha-search, beta-cache\n".into(),
        );
        super::link(&store, "beta-cache", "cut").unwrap();
        let meta_after = store.meta("cut");
        let writes_after = *store.meta_writes.borrow();
        super::link(&store, "beta-cache", "cut").unwrap();
        assert_eq!(store.meta("cut"), meta_after, "meta must be unchanged");
        assert!(
            meta_after.contains("from_discussion: alpha-search, beta-cache\n"),
            "existing list preserved, not appended: {meta_after}"
        );
        assert_eq!(
            *store.meta_writes.borrow(),
            writes_after,
            "change side must not rewrite when slug already present"
        );
    }

    #[test]
    fn link_rejects_corrupt_change_meta_without_writes() {
        // spec「link 對壞 metadata 拒絕且兩側皆不寫」：壞檔不得被解讀為
        // 「無 from_discussion 鏈」而追加行——兩側檔案逐位元不變。
        const BAD: &str = ": : :\n\t bad yaml [unclosed\n";
        let doc = concluded_doc("alpha-search", "Alpha search", "x");
        let store = TestStore::with_live_discussion("alpha-search", &doc);
        store.metas.borrow_mut().insert("broken-cut".into(), BAD.into());
        let err = super::link(&store, "alpha-search", "broken-cut").unwrap_err();
        assert!(
            err.to_string().contains("openspec/changes/broken-cut/.openspec.yaml"),
            "error must name the metadata file: {err}"
        );
        assert_eq!(store.meta("broken-cut"), BAD, "change meta byte-identical");
        assert_eq!(store.discussion("alpha-search"), doc, "discussion byte-identical");
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    // --- seal flow（spec「內容落地以 seal 動詞標記已轉出」） ---

    #[test]
    fn seal_marks_promoted_when_chain_forged() {
        // 鏈已鑄妥（變更 meta 含 from_discussion: slug）→ 討論翻 promoted、累加 promoted_to。
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &concluded_doc("alpha-search", "Alpha search", "build alpha search"),
        );
        store.metas.borrow_mut().insert(
            "existing-cut".into(),
            "schema: spec-driven\nfrom_discussion: alpha-search\n".into(),
        );
        super::seal(&store, "alpha-search", "existing-cut").unwrap();
        let text = store.discussion("alpha-search");
        assert!(text.contains("status: promoted\n"), "text: {text}");
        assert!(text.contains("promoted_to: existing-cut\n"), "text: {text}");
    }

    #[test]
    fn seal_rejects_when_chain_not_forged_without_writes() {
        // 變更存在但 meta 的 from_discussion 未含該 slug → 拒絕、兩側逐位元不變。
        let doc = concluded_doc("alpha-search", "Alpha search", "x");
        let store = TestStore::with_live_discussion("alpha-search", &doc);
        store.metas.borrow_mut().insert("cut".into(), "schema: spec-driven\n".into());
        let err = super::seal(&store, "alpha-search", "cut").unwrap_err();
        assert!(err.to_string().contains("not linked"), "err: {err}");
        assert_eq!(store.discussion("alpha-search"), doc, "discussion untouched");
        assert_eq!(store.meta("cut"), "schema: spec-driven\n", "change meta untouched");
    }

    #[test]
    fn seal_rejects_corrupt_change_meta_not_misreporting_the_chain() {
        // spec「seal 對壞 metadata 拒絕且不誤報鏈缺失」：錯誤指出 metadata
        // 損壞（而非 from_discussion 不含該 slug）；兩側檔案逐位元不變。
        const BAD: &str = ": : :\n\t bad yaml [unclosed\n";
        let doc = concluded_doc("alpha-search", "Alpha search", "x");
        let store = TestStore::with_live_discussion("alpha-search", &doc);
        store.metas.borrow_mut().insert("broken-cut".into(), BAD.into());
        let err = super::seal(&store, "alpha-search", "broken-cut").unwrap_err();
        assert!(
            err.to_string().contains("openspec/changes/broken-cut/.openspec.yaml"),
            "error must name the metadata file: {err}"
        );
        assert!(
            !err.to_string().contains("not linked"),
            "must not misreport a missing chain: {err}"
        );
        assert_eq!(store.meta("broken-cut"), BAD, "change meta byte-identical");
        assert_eq!(store.discussion("alpha-search"), doc, "discussion byte-identical");
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn seal_rejects_missing_discussion_and_missing_change() {
        // 討論不存在。
        let store = TestStore::default();
        store.metas.borrow_mut().insert(
            "cut".into(),
            "schema: spec-driven\nfrom_discussion: ghost\n".into(),
        );
        assert!(super::seal(&store, "ghost", "cut").unwrap_err().to_string().contains("not found"));
        // 變更不存在（討論存在）。
        let store2 =
            TestStore::with_live_discussion("alpha", &concluded_doc("alpha", "Alpha", "x"));
        assert!(super::seal(&store2, "alpha", "no-such-change").unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn seal_rejects_archived_discussion() {
        let store = TestStore::default();
        store
            .archived_discussions
            .borrow_mut()
            .insert("old".into(), concluded_doc("old", "Old", "x"));
        store.metas.borrow_mut().insert(
            "cut".into(),
            "schema: spec-driven\nfrom_discussion: old\n".into(),
        );
        assert!(super::seal(&store, "old", "cut").unwrap_err().to_string().contains("archived"));
    }

    #[test]
    fn seal_is_idempotent_when_already_promoted() {
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &promoted_concluded("alpha-search", "Alpha search", "x", "existing-cut"),
        );
        store.metas.borrow_mut().insert(
            "existing-cut".into(),
            "schema: spec-driven\nfrom_discussion: alpha-search\n".into(),
        );
        let before = store.discussion("alpha-search");
        super::seal(&store, "alpha-search", "existing-cut").unwrap();
        assert_eq!(store.discussion("alpha-search"), before, "重跑不改檔");
    }

    #[test]
    fn link_same_pair_is_idempotent() {
        // spec「同一組合重跑為冪等」：Ok、兩側內容逐位元不變、變更側不再寫。
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &concluded_doc("alpha-search", "Alpha search", "x"),
        );
        store.metas.borrow_mut().insert("cut".into(), "schema: spec-driven\n".into());
        super::link(&store, "alpha-search", "cut").unwrap();
        let meta_after = store.meta("cut");
        let doc_after = store.discussion("alpha-search");
        let writes_after = *store.meta_writes.borrow();
        super::link(&store, "alpha-search", "cut").unwrap();
        assert_eq!(store.meta("cut"), meta_after);
        assert_eq!(store.discussion("alpha-search"), doc_after);
        assert_eq!(*store.meta_writes.borrow(), writes_after, "change side must not rewrite");
    }

    #[test]
    fn seal_accumulates_promoted_to_on_fan_out() {
        // spec「promoted_to 逗號累加、既有值保留」：已 promoted 的討論再經 seal 併入
        // 另一變更（fan-out 累加現由 seal 承接，非 link）。
        let doc = concluded_doc("alpha-search", "Alpha search", "x").replacen(
            "status: concluded\n",
            "status: promoted\npromoted_to: first-cut\n",
            1,
        );
        let store = TestStore::with_live_discussion("alpha-search", &doc);
        store.metas.borrow_mut().insert(
            "second-cut".into(),
            "schema: spec-driven\nfrom_discussion: alpha-search\n".into(),
        );
        super::seal(&store, "alpha-search", "second-cut").unwrap();
        let text = store.discussion("alpha-search");
        assert!(text.contains("promoted_to: first-cut, second-cut\n"), "text: {text}");
    }

    #[test]
    fn link_tolerates_meta_without_trailing_newline() {
        // meta 讀-改-寫的尾換行容錯（inprogress 同款模式）。
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &concluded_doc("alpha-search", "Alpha search", "x"),
        );
        store
            .metas
            .borrow_mut()
            .insert("cut".into(), "schema: spec-driven\ncreated: 2026-01-03".into());
        super::link(&store, "alpha-search", "cut").unwrap();
        let meta = store.meta("cut");
        assert!(
            meta.contains("created: 2026-01-03\nfrom_discussion: alpha-search\n"),
            "meta: {meta}"
        );
    }

    // --- discuss new：slug 覆寫與後備衍生（spec「討論記錄以 --slug 覆寫檔名」「未帶 --slug 時自主題衍生檔名」） ---

    #[test]
    fn new_discussion_rejects_invalid_slug_override() {
        // spec「非法值一覽」Example 表：大寫、非 ASCII、底線、空白、首尾連字號、連續連字號、空字串。
        let store = TestStore::default();
        for bad in [
            "Board-Search",
            "看板搜尋",
            "board_search",
            "board search",
            "-board",
            "board-",
            "board--search",
            "",
        ] {
            let err = super::new_discussion(&store, "看板搜尋列", Some(bad), None, None).unwrap_err();
            assert!(err.to_string().contains("kebab-case"), "slug {bad:?} err: {err}");
        }
        assert!(store.list_live_discussions().is_empty(), "invalid slug must not create files");
    }

    #[test]
    fn new_discussion_accepts_valid_slug_override_and_keeps_topic() {
        let store = TestStore::default();
        let info = super::new_discussion(&store, "看板搜尋列", Some("board-search-2"), None, None).unwrap();
        assert_eq!(info.slug, "board-search-2");
        assert_eq!(info.topic, "看板搜尋列");
        let text = store
            .read_live_discussion("board-search-2")
            .expect("record stored under override slug");
        assert!(text.contains("slug: board-search-2\n"), "text: {text}");
        assert!(text.contains("topic: 看板搜尋列\n"), "text: {text}");
    }

    #[test]
    fn new_discussion_slug_override_conflicts_with_existing() {
        let store = TestStore::with_live_discussion("taken", &open_doc("taken", "Taken"));
        let before = store.discussion("taken");
        let err = super::new_discussion(&store, "另一個主題", Some("taken"), None, None).unwrap_err();
        assert!(err.to_string().contains("already exists"), "err: {err}");
        assert_eq!(store.discussion("taken"), before, "existing record must not be overwritten");
    }

    #[test]
    fn new_discussion_fallback_derivation_is_unchanged() {
        // spec「衍生規則對照」Example 表：後備行為與本變更前逐位元一致。
        for (topic, want) in [
            ("Board Search", "board-search"),
            ("config context 與 rules GUI 編輯", "config-context-與-rules-gui-編輯"),
            ("看板 搜尋列", "看板-搜尋列"),
        ] {
            let store = TestStore::default();
            let info = super::new_discussion(&store, topic, None, None, None).unwrap();
            assert_eq!(info.slug, want, "topic: {topic}");
            assert_eq!(info.topic, topic);
            assert!(store.read_live_discussion(want).is_some(), "file under derived slug");
        }
        // 純 ASCII 標點主題衍生為空 → 報錯。
        let store = TestStore::default();
        let err = super::new_discussion(&store, "!?!", None, None, None).unwrap_err();
        assert!(err.to_string().contains("could not derive"), "err: {err}");
    }

    // --- discuss new：蓋建立者章（spec「討論記錄蓋建立者章」） ---

    #[test]
    fn new_discussion_stamps_created_by_when_identity_present() {
        let store = TestStore::default();
        let id = "Base Line <base@example.com>";
        let info = super::new_discussion(&store, "看板搜尋列", Some("board-search-3"), Some(id), None).unwrap();
        // frontmatter 蓋 created_by、且 DiscussionInfo（→ --json createdBy）帶同值。
        let text = store.read_live_discussion("board-search-3").expect("record stored");
        assert!(text.contains(&format!("created_by: {id}\n")), "frontmatter: {text}");
        assert_eq!(info.created_by.as_deref(), Some(id));
    }

    #[test]
    fn new_discussion_omits_created_by_when_identity_absent() {
        let store = TestStore::default();
        let info = super::new_discussion(&store, "看板搜尋列", Some("board-search-4"), None, None).unwrap();
        // 無身分：frontmatter 不含 created_by、createdBy 缺席。
        let text = store.read_live_discussion("board-search-4").expect("record stored");
        assert!(!text.contains("created_by:"), "frontmatter should omit created_by: {text}");
        assert_eq!(info.created_by, None);
    }

    // --- restale flag：conclude 蓋章 / seal 清除（reconclude-restale） ---

    /// A promoted discussion (status: promoted, promoted_to set) with a written conclusion.
    fn promoted_doc(slug: &str, topic: &str, promoted_to: &str, decision: &str) -> String {
        format!(
            "---\ntopic: {topic}\nslug: {slug}\nstatus: promoted\npromoted_to: {promoted_to}\ncreated: 2026-01-02\n---\n\n\
             # Discussion: {topic}\n\n\
             ## Context\n\nFixture context.\n\n\
             ## Rounds\n\n### Round 1 — assumptions (2026-01-02)\n\n**Focus**: scope\n\n\
             ## Conclusion\n\n**Decision**: {decision}\n"
        )
    }

    #[test]
    fn conclude_stamps_restale_on_active_promoted_change() {
        let store =
            TestStore::with_live_discussion("alpha", &promoted_doc("alpha", "Alpha", "cut-a", "old"));
        store.metas.borrow_mut().insert(
            "cut-a".to_string(),
            "schema: spec-driven\nfrom_discussion: alpha\n".to_string(),
        );
        let flagged =
            super::conclude(&store, "alpha", "**Decision**: new direction", false).unwrap().restale_flagged;
        assert_eq!(flagged, vec!["cut-a".to_string()]);
        assert!(store.meta("cut-a").contains("restale_from: alpha"), "meta: {}", store.meta("cut-a"));
        // 討論維持 promoted、promoted_to 不變，僅 Conclusion 改寫。
        let disc = store.discussion("alpha");
        assert!(disc.contains("status: promoted\n"), "stays promoted: {disc}");
        assert!(disc.contains("promoted_to: cut-a\n"), "promoted_to intact: {disc}");
        assert!(disc.contains("**Decision**: new direction"), "conclusion rewritten: {disc}");
    }

    #[test]
    fn conclude_restale_skips_archived_change() {
        // promoted_to 同含 active 與已歸檔變更；僅 active 被蓋。
        let store = TestStore::with_live_discussion(
            "alpha",
            &promoted_doc("alpha", "Alpha", "cut-a, arch-b", "old"),
        );
        store.metas.borrow_mut().insert("cut-a".to_string(), "schema: spec-driven\n".to_string());
        // arch-b 僅存於封存（read_change_meta 回 None）——非 active。
        store
            .archived_metas
            .borrow_mut()
            .insert("arch-b".to_string(), "schema: spec-driven\n".to_string());
        let flagged = super::conclude(&store, "alpha", "**Decision**: new", false).unwrap().restale_flagged;
        assert_eq!(flagged, vec!["cut-a".to_string()], "only active flagged");
        assert!(store.meta("cut-a").contains("restale_from: alpha"));
        assert!(!store.change_exists("arch-b"), "archived never active");
        assert_eq!(
            store.archived_metas.borrow().get("arch-b").unwrap(),
            "schema: spec-driven\n",
            "archived meta untouched"
        );
    }

    #[test]
    fn conclude_restale_skips_corrupt_meta_change_without_writing() {
        // fail-closed 掃尾：promoted_to 指向的 change metadata 損壞時跳過該卡
        // （沿 archived/gone 的 skip 原則——單一壞檔不得使 conclude 中止），
        // 壞檔逐位元不變、其餘 active change 照常蓋章。
        const BAD: &str = ": : :\n\t bad yaml [unclosed\n";
        let store = TestStore::with_live_discussion(
            "alpha",
            &promoted_doc("alpha", "Alpha", "cut-a, broken-b", "old"),
        );
        store.metas.borrow_mut().insert("cut-a".to_string(), "schema: spec-driven\n".to_string());
        store.metas.borrow_mut().insert("broken-b".to_string(), BAD.to_string());
        let flagged = super::conclude(&store, "alpha", "**Decision**: new", false).unwrap().restale_flagged;
        assert_eq!(flagged, vec!["cut-a".to_string()], "corrupt change is not flagged");
        assert!(store.meta("cut-a").contains("restale_from: alpha"));
        assert_eq!(store.meta("broken-b"), BAD, "corrupt meta must not be appended to");
    }

    #[test]
    fn conclude_promoted_to_empty_stamps_nothing() {
        // concluded-but-not-promoted：promoted_to 缺席 → 不蓋章。
        let store =
            TestStore::with_live_discussion("alpha", &concluded_doc("alpha", "Alpha", "old"));
        store.metas.borrow_mut().insert("cut-a".to_string(), "schema: spec-driven\n".to_string());
        let flagged = super::conclude(&store, "alpha", "**Decision**: new", false).unwrap().restale_flagged;
        assert!(flagged.is_empty());
        assert_eq!(*store.meta_writes.borrow(), 0, "no change meta written");
        assert_eq!(store.meta("cut-a"), "schema: spec-driven\n", "change meta untouched");
    }

    #[test]
    fn conclude_restale_stamp_is_idempotent() {
        let store =
            TestStore::with_live_discussion("alpha", &promoted_doc("alpha", "Alpha", "cut-a", "old"));
        store.metas.borrow_mut().insert(
            "cut-a".to_string(),
            "schema: spec-driven\nrestale_from: alpha\n".to_string(),
        );
        let before = store.meta("cut-a");
        let flagged =
            super::conclude(&store, "alpha", "**Decision**: newer", false).unwrap().restale_flagged;
        assert_eq!(flagged, vec!["cut-a".to_string()], "still reported stale");
        assert_eq!(store.meta("cut-a"), before, "no duplicate accumulation");
        assert_eq!(*store.meta_writes.borrow(), 0, "idempotent — no change meta write");
    }

    #[test]
    fn seal_clears_restale_slug_keeping_others() {
        let store =
            TestStore::with_live_discussion("alpha", &concluded_doc("alpha", "Alpha", "done"));
        store.metas.borrow_mut().insert(
            "cut-a".to_string(),
            "schema: spec-driven\nfrom_discussion: alpha\nrestale_from: alpha, beta\n".to_string(),
        );
        super::seal(&store, "alpha", "cut-a").unwrap();
        let meta = store.meta("cut-a");
        assert!(meta.contains("restale_from: beta\n"), "alpha cleared, beta kept: {meta}");
        assert!(!meta.contains("restale_from: alpha"), "alpha gone: {meta}");
        assert!(store.discussion("alpha").contains("status: promoted\n"), "sealed → promoted");
    }

    #[test]
    fn seal_clears_restale_line_when_last_slug() {
        let store =
            TestStore::with_live_discussion("alpha", &concluded_doc("alpha", "Alpha", "done"));
        store.metas.borrow_mut().insert(
            "cut-a".to_string(),
            "schema: spec-driven\nfrom_discussion: alpha\nrestale_from: alpha\n".to_string(),
        );
        super::seal(&store, "alpha", "cut-a").unwrap();
        let meta = store.meta("cut-a");
        assert!(!meta.contains("restale_from"), "restale_from line dropped: {meta}");
        assert!(meta.contains("from_discussion: alpha\n"), "other fields intact: {meta}");
    }

    #[test]
    fn seal_restale_clear_is_noop_when_absent() {
        let store =
            TestStore::with_live_discussion("alpha", &concluded_doc("alpha", "Alpha", "done"));
        store.metas.borrow_mut().insert(
            "cut-a".to_string(),
            "schema: spec-driven\nfrom_discussion: alpha\n".to_string(),
        );
        super::seal(&store, "alpha", "cut-a").unwrap();
        assert!(!store.meta("cut-a").contains("restale_from"), "no restale_from introduced");
    }

    // --- kind 標記（add-improve-flow；規格「討論記錄以 --kind 標記改進討論」） ---

    #[test]
    fn new_discussion_with_kind_writes_frontmatter_and_reports_it() {
        let store = TestStore::default();
        let info =
            super::new_discussion(&store, "核心結構改進", Some("improve-core"), None, Some("improve"))
                .unwrap();
        assert_eq!(info.kind.as_deref(), Some("improve"));
        let text = store.discussion("improve-core");
        assert!(text.contains("\nkind: improve\n"), "frontmatter 應含 kind: {text}");
    }

    #[test]
    fn new_discussion_rejects_kind_outside_the_whitelist_without_writing() {
        for bad in ["refactor", "IMPROVE", "", "improve\nstatus: forged"] {
            let store = TestStore::default();
            let err = super::new_discussion(&store, "主題", Some("alpha"), None, Some(bad))
                .expect_err("非白名單 kind 必須拒絕")
                .to_string();
            assert!(err.contains("improve"), "訊息須說明僅接受 improve：{err}");
            assert!(store.discussions.borrow().is_empty(), "拒絕時不得落檔（kind={bad:?}）");
        }
    }

    #[test]
    fn new_discussion_without_kind_leaves_frontmatter_unchanged() {
        let store = TestStore::default();
        let info = super::new_discussion(&store, "主題", Some("alpha"), None, None).unwrap();
        assert!(info.kind.is_none(), "無 kind 時欄位缺席");
        assert!(!store.discussion("alpha").contains("kind:"), "不得寫入 kind 行");
    }

    #[test]
    fn new_discussion_rejects_multiline_topic_without_writing() {
        // topic 逐字寫入 frontmatter——夾帶換行可注入偽造的 kind:/status: 行
        // （frontmatter_value 取第一個命中），必須在系統邊界擋下。
        for bad in ["x\nkind: improve\nstatus: promoted", "x\rkind: improve", "x\r\ny"] {
            let store = TestStore::default();
            let err = super::new_discussion(&store, bad, Some("plain-a"), None, None)
                .expect_err("多行 topic 必須拒絕")
                .to_string();
            assert!(err.contains("invalid topic"), "訊息須點名 topic：{err}");
            assert!(store.discussions.borrow().is_empty(), "拒絕時不得落檔（topic={bad:?}）");
        }
    }

    #[test]
    fn empty_kind_frontmatter_reads_as_plain() {
        // 手改記錄寫出空值 `kind:` 不得讓 payload 冒出 "kind": ""——
        // 讀取端正規化為缺席，維持「缺席即省略」的形狀不變量。
        let doc = open_doc("gamma", "Gamma").replacen("status: open\n", "status: open\nkind:\n", 1);
        let store = TestStore::with_live_discussion("gamma", &doc);
        assert!(super::info(&store, "gamma").unwrap().kind.is_none(), "空值 kind 視為一般討論");
    }

    #[test]
    fn kind_is_read_from_frontmatter_and_absent_records_are_plain() {
        let plain = TestStore::with_live_discussion("alpha", &open_doc("alpha", "Alpha"));
        assert!(super::info(&plain, "alpha").unwrap().kind.is_none(), "舊記錄視為一般討論");

        let marked = open_doc("beta", "Beta")
            .replacen("status: open\n", "status: open\nkind: improve\n", 1);
        let store = TestStore::with_live_discussion("beta", &marked);
        // info 與 list 共用 info_from_doc；list 面的曝露由 CLI 整合測試把關
        // （TestStore 不供列表）。
        assert_eq!(super::info(&store, "beta").unwrap().kind.as_deref(), Some("improve"));
    }

    // --- 結構錨定與撞名內容跳脫（fix-discuss-section-anchor） ---

    #[test]
    fn add_round_appends_after_prior_round_with_level2_content_line() {
        // spec Example「兩輪順序與內文歸屬」：Round 1 內文含「## 背景」行，
        // add_round 追加 Round 2 仍落在 Round 1 完整內文之後、結構 Conclusion 之前；
        // 「## 背景」不撞結構、維持原樣。
        let store = TestStore::with_live_discussion("alpha", &open_doc("alpha", "Alpha"));
        super::add_round(&store, "alpha", "explore", "## 背景\n首輪本文").unwrap();
        super::add_round(&store, "alpha", "explore", "次輪本文").unwrap();
        let text = store.discussion("alpha");
        let pos = |needle: &str| {
            text.find(needle).unwrap_or_else(|| panic!("missing {needle:?} in: {text}"))
        };
        let round1 = pos("### Round 1");
        let bg = pos("## 背景");
        let first = pos("首輪本文");
        let round2 = pos("### Round 2");
        let second = pos("次輪本文");
        let conclusion = pos("## Conclusion");
        assert!(
            round1 < bg && bg < first && first < round2 && round2 < second && second < conclusion,
            "文件順序須為 Round 1 標題→Round 1 完整內文→Round 2 標題→Round 2 內文→結構 Conclusion: {text}"
        );
    }

    #[test]
    fn conclude_lands_in_structural_conclusion_when_round_content_collides() {
        // spec「結論寫入不落入輪內」：輪內文原始輸入含整行「## Conclusion」→
        // 落盤即跳脫；conclude 後結論寫入結構 Conclusion 區段、既有輪內文不被改寫。
        let store = TestStore::with_live_discussion("alpha", &open_doc("alpha", "Alpha"));
        super::add_round(&store, "alpha", "explore", "偽結論引子\n## Conclusion\n偽結論本文")
            .unwrap();
        assert!(
            store.discussion("alpha").contains("\\## Conclusion"),
            "add_round 落盤前須跳脫撞名行: {}",
            store.discussion("alpha")
        );
        super::conclude(&store, "alpha", "**Decision**: real", false).unwrap();
        let text = store.discussion("alpha");
        assert!(
            text.contains("偽結論引子\n\\## Conclusion\n偽結論本文"),
            "既有輪內文（含跳脫行）不得被 conclude 改寫: {text}"
        );
        let header = text.find("\n## Conclusion\n").expect("structural header");
        let fake_body = text.find("偽結論本文").unwrap();
        let decision = text.find("**Decision**: real").expect("conclusion written");
        assert!(
            fake_body < header && header < decision,
            "結論須寫入結構 Conclusion 區段（於輪內文之後）: {text}"
        );
        assert_eq!(
            super::conclusion_text(&store, "alpha").as_deref(),
            Some("**Decision**: real"),
            "conclusion_text 須讀到結構區段的結論"
        );
    }

    #[test]
    fn count_rounds_ignores_colliding_content_and_numbering_stays_consecutive() {
        // spec「撞名輪標題行不膨脹輪計數」：內文行首「### Round 」跳脫落盤，
        // 計數不膨脹、下一輪編號連續。
        let store = TestStore::with_live_discussion("alpha", &open_doc("alpha", "Alpha"));
        super::add_round(&store, "alpha", "explore", "### Round 99 — fake (2026-01-01)\n本文")
            .unwrap();
        assert_eq!(super::count_rounds(&store.discussion("alpha")), 1, "撞名行不得計數");
        let no = super::add_round(&store, "alpha", "explore", "次輪").unwrap();
        assert_eq!(no, 2, "編號須連續、無跳號");
        // list/info 的 rounds 欄位走同一 count_rounds 鏈路。
        assert_eq!(super::info(&store, "alpha").unwrap().rounds, 2, "discuss list 輪數同合法計數");
        // 手改記錄的非法輪標題形狀（無編號）不計數；pre-scaffold「## Round 」照舊容忍。
        let legacy = "## Rounds\n\n### Round 備註不是輪\n\n## Round 1 — old (2026-01-01)\n";
        assert_eq!(super::count_rounds(legacy), 1);
    }

    #[test]
    fn set_context_preserves_pre_scaffold_rounds() {
        // pre-scaffold 版面（有 ## Context、無 ## Rounds/## Conclusion）上
        // set_context 只得替換脈絡本文，其後的 level-2 輪區段不得被覆寫。
        let doc = "---\ntopic: Legacy\nslug: legacy\nstatus: open\ncreated: 2026-01-02\n---\n\n\
                   # Discussion: Legacy\n\n\
                   ## Context\n\n舊脈絡。\n\n\
                   ## Round 1 — assumptions (2026-01-02)\n\n首輪本文\n";
        let store = TestStore::with_live_discussion("legacy", doc);
        super::set_context(&store, "legacy", "新脈絡").unwrap();
        let text = store.discussion("legacy");
        assert!(text.contains("新脈絡"), "text: {text}");
        assert!(
            text.contains("## Round 1 — assumptions (2026-01-02)\n\n首輪本文"),
            "pre-scaffold 輪不得被覆寫: {text}"
        );
    }

    #[test]
    fn conclusion_boundary_stops_at_pre_scaffold_round() {
        // pre-scaffold 上 conclude 之後追加的 ## Round 區段——結論讀取不吞、
        // re-conclude 不刪。
        let doc = "---\ntopic: Legacy\nslug: legacy\nstatus: concluded\ncreated: 2026-01-02\n---\n\n\
                   # Discussion: Legacy\n\n\
                   ## Context\n\n脈絡。\n\n\
                   ## Conclusion\n\n**Decision**: done\n\n\
                   ## Round 2 — explore (2026-01-03)\n\n次輪本文\n";
        let store = TestStore::with_live_discussion("legacy", doc);
        assert_eq!(
            super::conclusion_text(&store, "legacy").as_deref(),
            Some("**Decision**: done"),
            "結論讀取不得吞掉其後的輪區段"
        );
        super::conclude(&store, "legacy", "**Decision**: revised", false).unwrap();
        let text = store.discussion("legacy");
        assert!(
            text.contains("## Round 2 — explore (2026-01-03)\n\n次輪本文"),
            "re-conclude 不得刪除其後的輪區段: {text}"
        );
        assert_eq!(
            super::conclusion_text(&store, "legacy").as_deref(),
            Some("**Decision**: revised")
        );
    }

    #[test]
    fn discard_refuses_on_malformed_round_heading() {
        // 手改壞形狀（ASCII 連字號）仍是「有輪」的證據——保護面寬鬆偵測，
        // 不因計數收緊而放行無 --force 的刪除。
        let doc = open_doc("alpha", "Alpha").replacen(
            "## Rounds\n",
            "## Rounds\n\n### Round 1 - broken (2026-01-02)\n\n首輪本文\n",
            1,
        );
        let store = TestStore::with_live_discussion("alpha", &doc);
        assert!(super::discard_discussion(&store, "alpha", false).is_err(), "壞形狀輪仍須擋刪");
        assert_eq!(store.discussion("alpha"), doc, "拒絕時不得刪檔");
    }

    #[test]
    fn fenced_layout_quotes_are_not_escaped_and_not_structure() {
        // ``` 圍欄內引用文件版面——不跳脫、不計數、不作區段邊界。
        let store = TestStore::with_live_discussion("alpha", &open_doc("alpha", "Alpha"));
        let quoted = "示範版面：\n```\n## Conclusion\n### Round 9 — fake (2026-01-01)\n```\n收尾";
        super::add_round(&store, "alpha", "explore", quoted).unwrap();
        let text = store.discussion("alpha");
        assert!(
            text.contains("```\n## Conclusion\n### Round 9 — fake (2026-01-01)\n```"),
            "圍欄內不得跳脫: {text}"
        );
        assert_eq!(super::count_rounds(&text), 1, "圍欄內輪標題不計數");
        super::conclude(&store, "alpha", "**Decision**: real", false).unwrap();
        let text = store.discussion("alpha");
        assert!(text.contains("```\n## Conclusion\n"), "圍欄內容不被 conclude 改寫: {text}");
        assert_eq!(
            super::conclusion_text(&store, "alpha").as_deref(),
            Some("**Decision**: real"),
            "結論讀取須錨定結構區段"
        );
    }

    #[test]
    fn scaffold_round_heading_shape_matches_ui_parser() {
        // 引擎輪標題判準與 UI splitRounds 同形——`<mode> (<date>)` 缺一不可。
        assert!(super::is_scaffold_round_heading("### Round 1 — assumptions (2026-01-02)"));
        assert!(super::is_scaffold_round_heading("### Round 12 — grill (2026-12-31)"));
        for bad in [
            "### Round 1 — mode",              // 缺日期括號
            "### Round 1 — (2026-01-02)",      // 缺 mode
            "### Round — mode (2026-01-02)",   // 缺編號
            "### Round 1 - mode (2026-01-02)", // ASCII 連字號
        ] {
            assert!(!super::is_scaffold_round_heading(bad), "{bad:?} 不是合法輪標題");
        }
    }

    #[test]
    fn unbalanced_fence_is_escaped_at_write_so_structure_stays_sound() {
        // 寫入邊界強制圍欄成對：落單的圍欄行跳脫，其後全文的結構解析不受污染。
        let store = TestStore::with_live_discussion("alpha", &open_doc("alpha", "Alpha"));
        super::add_round(&store, "alpha", "explore", "前文\n```\n沒有關閉的引用").unwrap();
        let text = store.discussion("alpha");
        assert!(text.contains("前文\n\\```\n沒有關閉的引用"), "落單圍欄行須跳脫: {text}");
        let no = super::add_round(&store, "alpha", "explore", "次輪本文").unwrap();
        assert_eq!(no, 2, "圍欄狀態不得外溢到計數");
        let text = store.discussion("alpha");
        let r2 = text.find("### Round 2").unwrap();
        let conc = text.find("\n## Conclusion\n").unwrap();
        assert!(r2 < conc, "新輪仍須落在結構 Conclusion 之前: {text}");
        super::conclude(&store, "alpha", "**Decision**: real", false).unwrap();
        let text = store.discussion("alpha");
        assert_eq!(
            super::conclusion_text(&store, "alpha").as_deref(),
            Some("**Decision**: real"),
            "conclude 不得誤走 pre-scaffold 後備: {text}"
        );
        assert_eq!(text.matches("## Conclusion").count(), 1, "不得追加第二個 Conclusion: {text}");
    }

    #[test]
    fn discard_ignores_fenced_round_quotes() {
        // 零輪討論的 Context 圍欄引用輪標題——不得擋下 discard。
        let doc = open_doc("alpha", "Alpha").replacen(
            "## Context\n\nFixture context.\n",
            "## Context\n\n```\n### Round 1 — quoted (2026-01-02)\n```\n",
            1,
        );
        let store = TestStore::with_live_discussion("alpha", &doc);
        super::discard_discussion(&store, "alpha", false).unwrap();
        assert!(store.read_live_discussion("alpha").is_none(), "零輪記錄應可直接刪除");
    }

    #[test]
    fn set_context_escapes_colliding_lines() {
        // 三個寫入動詞的跳脫——set_context 面的直接測試。
        let store = TestStore::with_live_discussion("alpha", &open_doc("alpha", "Alpha"));
        super::set_context(&store, "alpha", "引子\n## Rounds\n### Round 3 — fake (2026-01-01)")
            .unwrap();
        let text = store.discussion("alpha");
        assert!(
            text.contains("引子\n\\## Rounds\n\\### Round 3 — fake (2026-01-01)"),
            "text: {text}"
        );
        assert_eq!(super::count_rounds(&text), 0);
    }

    // --- discuss search（discuss-search-recall）---

    /// A scaffolded record whose Rounds and Conclusion bodies are the caller's.
    fn search_doc(slug: &str, topic: &str, created: &str, rounds: &str, conclusion: &str) -> String {
        format!(
            "---\ntopic: {topic}\nslug: {slug}\nstatus: open\ncreated: {created}\n---\n\n\
             # Discussion: {topic}\n\n\
             ## Context\n\nFixture context.\n\n\
             ## Rounds\n\n{rounds}\n\
             ## Conclusion\n\n{conclusion}\n"
        )
    }

    fn terms(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    fn kinds(hit: &super::DiscussionHit) -> Vec<(&str, &str)> {
        hit.matches.iter().map(|m| (m.kind.as_str(), m.where_.as_str())).collect()
    }

    #[test]
    fn search_hits_topic_as_frontmatter() {
        let store = TestStore::with_live_discussion(
            "alpha",
            &search_doc("alpha", "Golden snapshot policy", "2026-07-01", "", ""),
        );
        let hits = super::search(&store, &terms(&["golden"])).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].info.slug, "alpha");
        assert!(!hits[0].info.archived);
        assert_eq!(kinds(&hits[0]), [("topic", "frontmatter")]);
        assert_eq!(hits[0].matches[0].text, "Golden snapshot policy");
    }

    #[test]
    fn search_hits_slug_as_frontmatter() {
        let store = TestStore::with_live_discussion(
            "sse-transport",
            &search_doc("sse-transport", "Transport choice", "2026-07-01", "", ""),
        );
        let hits = super::search(&store, &terms(&["sse"])).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(kinds(&hits[0]), [("slug", "frontmatter")]);
        assert_eq!(hits[0].matches[0].text, "sse-transport");
    }

    #[test]
    fn search_hits_ruled_out_line_with_its_round_number_in_archived_record() {
        let rounds = "### Round 1 — assumptions (2026-07-01)\n\n**Focus**: scope\n**Ruled out**: nothing yet\n\n\
                      ### Round 2 — interview (2026-07-02)\n\n**Focus**: drawer\n\
                      **Ruled out**: RichDetailDrawer 加 readOnly 旗標（分支地獄）\n";
        let store = TestStore::default();
        store.archived_discussions.borrow_mut().insert(
            "spec-drawer-trace-links".into(),
            search_doc("spec-drawer-trace-links", "Trace links", "2026-07-01", rounds, "**Decision**: two hops\n"),
        );
        let hits = super::search(&store, &terms(&["drawer"])).unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].info.archived, "archived records are searched by default");
        // Focus 行不算；slug 命中排前、ruled-out 行在後（文件順序）。
        assert_eq!(kinds(&hits[0]), [("slug", "frontmatter"), ("ruled-out", "round-2")]);
        assert_eq!(hits[0].matches[1].text, "**Ruled out**: RichDetailDrawer 加 readOnly 旗標（分支地獄）");
    }

    #[test]
    fn search_hits_the_three_conclusion_decision_lines() {
        let conclusion = "**Decision**: drawer stays read-only\n\
                          **Rejected alternatives**: drawer readOnly flag\n\
                          **Deferred**: drawer AND mode\n\
                          Prose mentioning drawer does not count.\n";
        let store = TestStore::with_live_discussion(
            "alpha",
            &search_doc("alpha", "Alpha", "2026-07-01", "", conclusion),
        );
        let hits = super::search(&store, &terms(&["drawer"])).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(
            kinds(&hits[0]),
            [("decision", "conclusion"), ("rejected", "conclusion"), ("deferred", "conclusion")]
        );
        assert_eq!(hits[0].matches[0].text, "**Decision**: drawer stays read-only");
    }

    #[test]
    fn search_is_case_insensitive() {
        let store = TestStore::with_live_discussion(
            "alpha",
            &search_doc("alpha", "SSE transport", "2026-07-01", "", "**Deferred**: Golden regen\n"),
        );
        let hits = super::search(&store, &terms(&["sse"])).unwrap();
        assert_eq!(kinds(&hits[0]), [("topic", "frontmatter")]);
        let hits = super::search(&store, &terms(&["GOLDEN"])).unwrap();
        assert_eq!(kinds(&hits[0]), [("deferred", "conclusion")]);
    }

    #[test]
    fn search_matches_any_of_several_terms() {
        let store = TestStore::with_live_discussion(
            "a",
            &search_doc("a", "Golden policy", "2026-07-01", "", ""),
        );
        store.discussions.borrow_mut().insert(
            "b".into(),
            search_doc("b", "Transport", "2026-08-01", "", "**Deferred**: SSE reconnect\n"),
        );
        let hits = super::search(&store, &terms(&["golden", "sse"])).unwrap();
        let slugs: Vec<&str> = hits.iter().map(|h| h.info.slug.as_str()).collect();
        assert_eq!(slugs, ["a", "b"]);
    }

    #[test]
    fn search_ignores_evidence_and_other_non_decision_lines() {
        let rounds = "### Round 1 — interview (2026-07-01)\n\n**Focus**: sidecar\n**Position**: sidecar first\n\
                      **Evidence**: see sidecar.rs\n**Open**: sidecar naming\n\nProse about sidecar.\n";
        let store = TestStore::with_live_discussion(
            "alpha",
            &search_doc("alpha", "Alpha", "2026-07-01", rounds, "Free text about sidecar.\n"),
        );
        let hits = super::search(&store, &terms(&["sidecar"])).unwrap();
        assert!(hits.is_empty(), "only decision lines count: {hits:?}");
    }

    #[test]
    fn search_tolerates_records_without_rounds_or_conclusion() {
        // 只有 frontmatter 與 Context：仍以 topic 參與比對。
        let bare = "---\ntopic: Drawer scope\nslug: bare\nstatus: open\ncreated: 2026-07-01\n---\n\n\
                    # Discussion: Drawer scope\n\n## Context\n\nseed\n";
        let store = TestStore::with_live_discussion("bare", bare);
        // 有 Ruled out 行但沒有任何輪標題：該行不算決定行，topic 仍命中。
        store.discussions.borrow_mut().insert(
            "headless".into(),
            "---\ntopic: Drawer again\nslug: headless\nstatus: open\ncreated: 2026-06-01\n---\n\n\
             ## Rounds\n\n**Ruled out**: drawer flag\n"
                .into(),
        );
        let hits = super::search(&store, &terms(&["drawer"])).unwrap();
        assert_eq!(hits.len(), 2);
        for hit in &hits {
            assert_eq!(kinds(hit), [("topic", "frontmatter")], "slug {}", hit.info.slug);
        }
    }

    #[test]
    fn search_rejects_an_empty_or_blank_term_list() {
        let store = TestStore::with_live_discussion(
            "alpha",
            &search_doc("alpha", "Alpha", "2026-07-01", "", ""),
        );
        assert!(super::search(&store, &[]).is_err());
        assert!(super::search(&store, &terms(&["  ", ""])).is_err());
    }

    #[test]
    fn search_orders_frontmatter_hits_first_then_created_newest_first() {
        // spec「討論定案以 search 動詞可查」的排序表：A topic → C slug → B conclusion → D round。
        let store = TestStore::default();
        let mut docs = store.discussions.borrow_mut();
        docs.insert("a".into(), search_doc("a", "Golden policy", "2026-07-01", "", ""));
        docs.insert("golden-regen".into(), search_doc("golden-regen", "Regen", "2026-06-01", "", ""));
        docs.insert("b".into(), search_doc("b", "B", "2026-08-01", "", "**Deferred**: golden later\n"));
        docs.insert(
            "d".into(),
            search_doc(
                "d",
                "D",
                "2026-05-01",
                "### Round 1 — interview (2026-05-01)\n\n**Ruled out**: golden inline\n",
                "",
            ),
        );
        drop(docs);
        let hits = super::search(&store, &terms(&["golden"])).unwrap();
        let slugs: Vec<&str> = hits.iter().map(|h| h.info.slug.as_str()).collect();
        assert_eq!(slugs, ["a", "golden-regen", "b", "d"]);
        assert_eq!(kinds(&hits[3]), [("ruled-out", "round-1")]);
    }

    #[test]
    fn search_breaks_same_day_ties_by_slug() {
        let store = TestStore::default();
        let mut docs = store.discussions.borrow_mut();
        docs.insert("zeta".into(), search_doc("zeta", "Golden Z", "2026-07-01", "", ""));
        docs.insert("alpha".into(), search_doc("alpha", "Golden A", "2026-07-01", "", ""));
        docs.insert("mid".into(), search_doc("mid", "M", "2026-07-01", "", "**Decision**: golden\n"));
        docs.insert("late".into(), search_doc("late", "L", "2026-07-02", "", "**Decision**: golden\n"));
        drop(docs);
        let hits = super::search(&store, &terms(&["golden"])).unwrap();
        let slugs: Vec<&str> = hits.iter().map(|h| h.info.slug.as_str()).collect();
        assert_eq!(slugs, ["alpha", "zeta", "late", "mid"]);
    }

    #[test]
    fn search_hit_serializes_as_info_fields_plus_matches() {
        let store = TestStore::with_live_discussion(
            "alpha",
            &search_doc("alpha", "Golden", "2026-07-01", "", "**Deferred**: golden regen\n"),
        );
        let hits = super::search(&store, &terms(&["golden"])).unwrap();
        let v = serde_json::to_value(&hits[0]).unwrap();
        let mut keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            ["archived", "created", "matches", "path", "rounds", "slug", "status", "topic"],
            "the list payload's fields flattened, plus matches"
        );
        let m = &v["matches"][1];
        let mut mkeys: Vec<&str> = m.as_object().unwrap().keys().map(String::as_str).collect();
        mkeys.sort_unstable();
        assert_eq!(mkeys, ["kind", "text", "where"]);
        assert_eq!(m["where"], "conclusion");
        assert_eq!(m["kind"], "deferred");
    }

    #[test]
    fn search_splits_each_term_on_whitespace_like_the_server_does() {
        // design D4／Non-Goals：關鍵字含空白不支援——CLI 位置參數與 server 的 q 皆以空白切詞。
        // 引擎統一切詞，讓帶引號的多字參數在本機與 remote 得到同一組命中。
        let store = TestStore::with_live_discussion(
            "a",
            &search_doc("a", "Golden policy", "2026-07-01", "", ""),
        );
        store.discussions.borrow_mut().insert(
            "b".into(),
            search_doc("b", "Transport", "2026-08-01", "", "**Deferred**: SSE reconnect\n"),
        );
        let hits = super::search(&store, &terms(&["golden  sse"])).unwrap();
        let slugs: Vec<&str> = hits.iter().map(|h| h.info.slug.as_str()).collect();
        assert_eq!(slugs, ["a", "b"], "one quoted argument behaves as two keywords");
    }

    #[test]
    fn search_hits_list_item_lines_that_continue_a_decision_marker() {
        // review 第一輪 must-fix：封存記錄慣用「標記獨占一行、內容寫在下一行條列」。
        // 標記行之後緊接的條列行都算該決定行的一部分；散文與下一個標記不算。
        let rounds = "### Round 1 — interview (2026-07-01)\n\n**Focus**: x\n\
                      **Ruled out**:\n- 只在 tray.ts 修落頁\n- 把 drawer 拿掉\n\n\
                      **Open**: drawer naming\n";
        let conclusion = "**Decision**: 兩件事：\n- keep drawer\n- ship\n\n\
                          **Deferred**:\n- drawer AND mode\nProse mentioning drawer.\n";
        let store = TestStore::with_live_discussion(
            "alpha",
            &search_doc("alpha", "Alpha", "2026-07-01", rounds, conclusion),
        );
        let hits = super::search(&store, &terms(&["drawer"])).unwrap();
        assert_eq!(hits.len(), 1);
        let got: Vec<(&str, &str, &str)> = hits[0]
            .matches
            .iter()
            .map(|m| (m.kind.as_str(), m.where_.as_str(), m.text.as_str()))
            .collect();
        assert_eq!(
            got,
            [
                ("ruled-out", "round-1", "- 把 drawer 拿掉"),
                ("decision", "conclusion", "- keep drawer"),
                ("deferred", "conclusion", "- drawer AND mode"),
            ]
        );
    }

    #[test]
    fn search_reads_structure_like_the_rest_of_the_parser() {
        // review 第一輪：縮排的輪標題／結構標題其他解析器不認，搜尋也不認。
        let rounds = "### Round 1 — interview (2026-07-01)\n\n**Ruled out**: drawer a\n\n\
                      \x20 ### Round 2 — interview (2026-07-02)\n\n**Ruled out**: drawer b\n\n\
                      \x20 ## Conclusion\n\n**Decision**: drawer c\n";
        let store = TestStore::with_live_discussion(
            "alpha",
            &search_doc("alpha", "Alpha", "2026-07-01", rounds, ""),
        );
        let hits = super::search(&store, &terms(&["drawer"])).unwrap();
        assert_eq!(
            kinds(&hits[0]),
            [("ruled-out", "round-1"), ("ruled-out", "round-1")],
            "an indented heading is content: both lines stay in round 1 and no conclusion opens"
        );
    }

    #[test]
    fn search_drops_round_attribution_after_a_malformed_round_heading() {
        // review 第一輪：壞形狀輪標題 count_rounds 不算輪，搜尋也不把其下的行掛到上一輪。
        let rounds = "### Round 1 — interview (2026-07-01)\n\n**Ruled out**: golden a\n\n\
                      ### Round 3\n\n**Ruled out**: golden b\n";
        let store = TestStore::with_live_discussion(
            "alpha",
            &search_doc("alpha", "Alpha", "2026-07-01", rounds, ""),
        );
        let hits = super::search(&store, &terms(&["golden"])).unwrap();
        assert_eq!(kinds(&hits[0]), [("ruled-out", "round-1")]);
        assert_eq!(hits[0].matches[0].text, "**Ruled out**: golden a");
    }
}
