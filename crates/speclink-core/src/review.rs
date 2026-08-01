//! 審查品質站（design D1／D2）：工單生命週期動詞（add-round／show／discard）
//! 與蓋章。站別差異（工單檔名、meta 欄位前綴、狀態詞）集中為常數——後續 verify
//! 站接入時只補常數組與動詞註冊，不建 trait（單一實例期，YAGNI）。
//!
//! 工單 `review.md` 是 sidecar：不註冊進 workflow schema，僅由動詞經 `&dyn Store`
//! 讀寫（與 discuss 動詞同型）——本地隨 git、remote 走 store 文件管道。

use crate::model::ChangeMeta;
use crate::store::Store;
use anyhow::{anyhow, bail, Result};

/// 審查站工單文件（change 目錄下的相對路徑）。
pub const REVIEW_DOC: &str = "review.md";

/// 工單 findings 行的分級。token 即工單文法與 CLI `--json` 的對外契約
/// （spec「審查工單的建立與追加」：severity ∈ CRITICAL／WARNING／SUGGESTION）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Critical,
    Warning,
    Suggestion,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Critical => "CRITICAL",
            Severity::Warning => "WARNING",
            Severity::Suggestion => "SUGGESTION",
        }
    }

    pub fn parse(s: &str) -> Option<Severity> {
        match s {
            "CRITICAL" => Some(Severity::Critical),
            "WARNING" => Some(Severity::Warning),
            "SUGGESTION" => Some(Severity::Suggestion),
            _ => None,
        }
    }
}

/// 一筆分級 finding（工單行 `- [severity] path — 描述`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub severity: Severity,
    pub path: String,
    pub text: String,
}

/// 一輪審查：`## Round N` 區段的解析結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Round {
    pub index: usize,
    pub scope: Vec<String>,
    pub findings: Vec<Finding>,
}

/// 解析後的工單。經 `add_round` 建立的工單至少含一輪。
#[derive(Debug, Clone)]
pub struct Ticket {
    pub rounds: Vec<Round>,
}

impl Ticket {
    /// 末輪——續輪 subagent 取待辦的入口（CLI `--json` 的 `lastRound`）。
    pub fn last_round(&self) -> &Round {
        self.rounds.last().expect("a ticket always carries at least one round")
    }
}

/// 動詞層可辨識的「查無」（change 或工單缺席）——server 端經 command
/// `classify` 映 404；CLI 人眼路徑只見訊息文字，行為不變。
#[derive(Debug)]
pub struct NotFound(pub String);

impl std::fmt::Display for NotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for NotFound {}

/// 追加一輪審查（工單不存在則建立，自 Round 1 起算）。回傳本輪編號。
/// 內容須含非空的 `**Scope**:` 檔案清單；`- [` 開頭行須為合法 findings 行；
/// 既有輪次 append-only 不改寫（spec「審查工單的建立與追加」）。
pub fn add_round(store: &dyn Store, change: &str, content: &str) -> Result<usize> {
    ensure_single_segment(change)?;
    if !store.change_exists(change) {
        bail!(NotFound(format!("change not found: {change}")));
    }
    // 寫入前先驗證（系統邊界：stdin 為外部輸入）——拒絕路徑零寫入。
    parse_round_body(content)?;
    let (mut text, next) = match store.read_artifact(change, REVIEW_DOC) {
        // 追加前解析既有工單：壞檔 fail-closed，不得在其上疊寫。
        Some(existing) => {
            let next = parse_ticket(&existing)?.last_round().index + 1;
            (existing, next)
        }
        None => (format!("# Review — {change}\n"), 1),
    };
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str(&format!("\n## Round {next}\n\n{}\n", content.trim_end()));
    store.write_artifact(change, REVIEW_DOC, &text)?;
    Ok(next)
}

/// 讀取並解析工單（spec「審查工單的讀取」）。無工單回錯誤。
pub fn show(store: &dyn Store, change: &str) -> Result<Ticket> {
    ensure_single_segment(change)?;
    if !store.change_exists(change) {
        bail!(NotFound(format!("change not found: {change}")));
    }
    let Some(text) = store.read_artifact(change, REVIEW_DOC) else {
        bail!(NotFound(format!("no review ticket for change '{change}'")));
    };
    parse_ticket(&text)
}

/// 放棄審查：刪除工單、不寫任何 metadata（spec「放棄審查」）。無工單回錯誤。
pub fn discard(store: &dyn Store, change: &str) -> Result<()> {
    ensure_single_segment(change)?;
    if !store.change_exists(change) {
        bail!(NotFound(format!("change not found: {change}")));
    }
    if !store.artifact_exists(change, REVIEW_DOC) {
        bail!(NotFound(format!("no review ticket for change '{change}'")));
    }
    store.delete_artifact(change, REVIEW_DOC)
}

/// 凍結度：章的雙錨判定結果（design D3——讀取端純函式，desktop-core 呼叫，
/// CLI 不輸出）。Unknown＝meta 未帶完整的章（缺席讀作未審查）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freshness {
    Fresh,
    Stale,
    Unknown,
}

/// scope 注入蓋章（design D4a）：remote 承載——工作樹持有者預算好的
/// (path, hash) 清單直接入章。守門與 `stamp` 完全相同；額外驗證分割
/// 「provided ∪ missing ＝工單各輪 Scope 聯集且不相交」（CAS 式保護），
/// 不成立即拒。`missing` 是 checkout 持有者對「聯集中已不存在的檔」的明示
/// 宣告——server 無工作樹無從驗證存在性，宣告與雜湊同屬提交端的權威。
pub fn stamp_with_scope(
    store: &dyn Store,
    change: &str,
    accept: bool,
    actor: Option<&str>,
    tool: Option<&str>,
    scope: Vec<crate::model::ReviewedScopeEntry>,
    missing: Vec<String>,
) -> Result<()> {
    let gate = stamp_gate(store, change, accept)?;
    let mut entries: Vec<(String, String)> =
        scope.into_iter().map(|e| (e.path.replace('\\', "/"), e.hash)).collect();
    entries.sort();
    entries.dedup();
    // 集合相等是「集合」：同一 path 兩份雜湊時差集皆空，放行會讓章落重複項，
    // freshness 逐筆比對必有一筆不符——該章從蓋下的那刻起就永遠 stale。
    if let Some(w) = entries.windows(2).find(|w| w[0].0 == w[1].0) {
        bail!(crate::command::Refusal(format!(
            "scope lists '{}' more than once with differing hashes — \
             one fingerprint per file",
            w[0].0
        )));
    }
    let declared_gone = scope_union(missing.iter().map(String::as_str));
    let outside: Vec<&str> = declared_gone
        .iter()
        .filter(|p| !gate.paths.contains(*p))
        .map(String::as_str)
        .collect();
    if !outside.is_empty() {
        bail!(crate::command::Refusal(format!(
            "declared-missing paths are not in the ticket's scope union: [{}]",
            outside.join(", ")
        )));
    }
    let provided: Vec<&String> = entries.iter().map(|(p, _)| p).collect();
    let overlap: Vec<&str> = declared_gone
        .iter()
        .filter(|p| provided.contains(p))
        .map(String::as_str)
        .collect();
    if !overlap.is_empty() {
        bail!(crate::command::Refusal(format!(
            "paths declared missing but also provided with a fingerprint: [{}]",
            overlap.join(", ")
        )));
    }
    let expected: Vec<&String> =
        gate.paths.iter().filter(|p| !declared_gone.contains(*p)).collect();
    ensure_scope_remainder(&expected, &declared_gone)?;
    let gap: Vec<&str> =
        expected.iter().filter(|p| !provided.contains(p)).map(|p| p.as_str()).collect();
    let extra: Vec<&str> = provided
        .iter()
        .filter(|p| !gate.paths.contains(**p))
        .map(|p| p.as_str())
        .collect();
    if !gap.is_empty() || !extra.is_empty() {
        bail!(crate::command::Refusal(format!(
            "provided scope does not match the ticket's scope union (missing: [{}], \
             unexpected: [{}]) — re-read the ticket and recompute the hashes",
            gap.join(", "),
            extra.join(", ")
        )));
    }
    write_stamp(store, change, &gate, actor, tool, &entries)
}

/// 蓋章（spec「蓋章守門與蓋章效果」）：守門＝任務全完成＋末輪零未解 findings
/// （`accept` 僅豁免後者）；通過時以工單各輪 Scope 聯集計算指紋，於同一原子
/// 寫入內落五個 `reviewed_*` 欄位並刪除工單。`read_file` 供指紋計算讀取
/// repo-root 相對路徑的檔案內容（remote 模式亦讀本地工作樹）；`file_exists`
/// 判定聯集檔案是否仍在工作樹——修正可能刪除／改名早輪審過的檔，死檔跳過
/// 不入錨（無從指紋也無從再變動），存在但讀不到者仍 fail-closed。
pub fn stamp(
    store: &dyn Store,
    change: &str,
    accept: bool,
    actor: Option<&str>,
    tool: Option<&str>,
    read_file: &dyn Fn(&str) -> Option<String>,
    file_exists: &dyn Fn(&str) -> bool,
) -> Result<()> {
    let gate = stamp_gate(store, change, accept)?;
    let (present, gone): (Vec<String>, Vec<String>) =
        gate.paths.iter().cloned().partition(|p| file_exists(p));
    ensure_scope_remainder(&present, &gone)?;
    let entries = fingerprint_scope(&present, read_file)?;
    write_stamp(store, change, &gate, actor, tool, &entries)
}

/// 跳過死檔跳到一個不剩就不是「審查過」——工作樹與工單嚴重脫節，fail-closed
/// 並指名檔案與處置（兩個蓋章入口共用）。
fn ensure_scope_remainder<P: AsRef<str>>(remaining: &[P], gone: &[String]) -> Result<()> {
    if remaining.is_empty() {
        bail!(crate::command::Refusal(format!(
            "every scope file in the ticket is gone from the work tree ([{}]) — nothing \
             left to fingerprint; restore the files or discard the ticket",
            gone.join(", ")
        )));
    }
    Ok(())
}

/// 各輪 Scope 的聯集：`\`→`/` 正規化、去重、排序（決定性）。指紋範圍的唯一
/// 定義——本地蓋章與 remote CLI 的預算路徑共走這裡，兩端不各寫一份。
pub fn scope_union<'a>(scopes: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut paths: Vec<String> = scopes.into_iter().map(|p| p.replace('\\', "/")).collect();
    paths.sort();
    paths.dedup();
    paths
}

/// 逐檔算內容指紋（`read_file` 收 repo-root 相對路徑）。缺檔 fail-closed——
/// 讀不到就無從證明審查過的是哪份內容。路徑在讀取前過 repo-root 相對守門：
/// remote 模式的清單來自 server 回應，是外部輸入。
pub fn fingerprint_scope(
    paths: &[String],
    read_file: &dyn Fn(&str) -> Option<String>,
) -> Result<Vec<(String, String)>> {
    let mut entries = Vec::with_capacity(paths.len());
    for path in paths {
        ensure_repo_relative(path)?;
        let Some(content) = read_file(path) else {
            // 讀不到可能是缺檔，也可能是讀得到但非 UTF-8——只說事實。
            bail!("cannot read scope file '{path}' as text — it must be present and UTF-8");
        };
        entries.push((path.clone(), content_fingerprint(&content)));
    }
    Ok(entries)
}

/// 兩個蓋章入口共用的守門結果：解析通過的 meta 原文、工單 Scope 聯集
/// （`\`→`/` 正規化、去重、排序）與任務錨總數。
struct StampGate {
    raw_meta: String,
    paths: Vec<String>,
    tasks_total: usize,
}

fn stamp_gate(store: &dyn Store, change: &str, accept: bool) -> Result<StampGate> {
    ensure_single_segment(change)?;
    if !store.change_exists(change) {
        bail!(NotFound(format!("change not found: {change}")));
    }
    // Fail-closed gate（沿 set_board_rank）：文字手術前先解析，壞檔不得被疊寫。
    let raw_meta = store.read_change_meta(change).unwrap_or_default();
    crate::model::check_meta_text(change, Some(&raw_meta))?;
    let Some(ticket_text) = store.read_artifact(change, REVIEW_DOC) else {
        bail!(NotFound(format!("no review ticket for change '{change}'")));
    };
    let ticket = parse_ticket(&ticket_text)?;

    // 守門 (1)：任務全數完成（零任務 change 比照 archive gate 放行）。
    let tasks_md = store.read_artifact(change, "tasks.md").unwrap_or_default();
    let (total, complete, _) = crate::tasks::progress(&crate::tasks::parse(&tasks_md));
    if total > 0 && complete < total {
        bail!(crate::command::Refusal(format!(
            "change '{change}' has {complete}/{total} tasks complete — review stamp \
             requires all tasks done"
        )));
    }
    // 守門 (2)：末輪零未解 findings；`--accept` 僅豁免此條。
    let unresolved = ticket.last_round().findings.len();
    if unresolved > 0 && !accept {
        bail!(crate::command::Refusal(format!(
            "the last round has {unresolved} unresolved finding(s) — fix and re-review, \
             or pass --accept to stamp with reservations"
        )));
    }

    let paths =
        scope_union(ticket.rounds.iter().flat_map(|r| r.scope.iter().map(String::as_str)));
    Ok(StampGate { raw_meta, paths, tasks_total: total })
}

fn write_stamp(
    store: &dyn Store,
    change: &str,
    gate: &StampGate,
    actor: Option<&str>,
    tool: Option<&str>,
    entries: &[(String, String)],
) -> Result<()> {
    // 文字手術：先剝除既有 reviewed_* 區塊（重蓋不留重複鍵），再附加新章；
    // 其餘欄位逐位元組保留。
    let mut out = strip_reviewed_lines(&gate.raw_meta);
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    // 身分／工具／指紋一律過 YAML 純量守門（沿 started_* 的同一道）：帶 `:`、
    // `#` 或換行的字串會注入欄位或炸掉整份 meta——而工單已在同一步刪除，
    // 無從回復。
    out.push_str(&format!("reviewed_at: {}\n", crate::util::today()));
    if let Some(actor) = actor {
        out.push_str(&format!("reviewed_by: {}\n", crate::util::yaml_scalar(actor)));
    }
    if let Some(tool) = tool {
        out.push_str(&format!("reviewed_with: {}\n", crate::util::yaml_scalar(tool)));
    }
    out.push_str(&format!("reviewed_tasks_total: {}\n", gate.tasks_total));
    out.push_str("reviewed_scope:\n");
    for (path, hash) in entries {
        out.push_str(&format!(
            "  - path: {}\n    hash: {}\n",
            crate::util::yaml_scalar(path),
            crate::util::yaml_scalar(hash)
        ));
    }

    // 同一原子寫入（design D3）：remote 走 bridge 的 staged commit 天然原子；
    // 本地順序為先刪工單再寫章——中斷時寧可退回「未審查」，也不得出現
    // 「章已寫而工單仍在」的半套狀態（spec 明文禁止的唯一中間態）。
    store.delete_artifact(change, REVIEW_DOC)?;
    store.write_change_meta(change, &out)?;
    Ok(())
}

/// 內容指紋：CRLF→LF 正規化後的 SHA-256 十六進位（spec「內容指紋錨與失效判定」）。
pub fn content_fingerprint(text: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(text.replace("\r\n", "\n").as_bytes());
    format!("{:x}", hasher.finalize())
}

/// 失效判定純函式（design D3）：任務錨＝當前任務仍是「蓋章當時總數的全完成」；
/// 內容錨＝全部 scope 檔的現值指紋相符（缺檔即不符）。任一錨破 → Stale；
/// 全符 → Fresh；meta 未帶完整章 → Unknown。
pub fn freshness(
    meta: &ChangeMeta,
    tasks_total: usize,
    tasks_complete: usize,
    read_file: &dyn Fn(&str) -> Option<String>,
) -> Freshness {
    let Some(reviewed_total) = meta.reviewed_tasks_total else {
        return Freshness::Unknown;
    };
    if meta.reviewed_at.is_none() || meta.reviewed_scope.is_empty() {
        return Freshness::Unknown;
    }
    if tasks_total != reviewed_total || tasks_complete != tasks_total {
        return Freshness::Stale;
    }
    for entry in &meta.reviewed_scope {
        match read_file(&entry.path) {
            Some(content) if content_fingerprint(&content) == entry.hash => {}
            _ => return Freshness::Stale,
        }
    }
    Freshness::Fresh
}

/// 剝除 meta 的全部 `reviewed_*` 頂層行（含 `reviewed_scope:` 之下的縮排區塊），
/// 其餘行逐位元組保留——重蓋章的前半場手術。
fn strip_reviewed_lines(meta: &str) -> String {
    let mut out = String::with_capacity(meta.len());
    let mut in_scope_block = false;
    for line in meta.split_inclusive('\n') {
        if in_scope_block {
            // 區塊的續行：縮排行、第 0 欄的序列項（`- path:` 也是合法 YAML），
            // 以及區塊內的空行。只認縮排會把其餘兩者留下，重蓋後 meta 成
            // mapping 混孤立序列、之後所有動詞對該 change fail-closed。
            if line.trim().is_empty() || line.starts_with([' ', '\t']) || line.starts_with("- ") {
                continue;
            }
            in_scope_block = false;
        }
        if line.starts_with("reviewed_scope:") {
            in_scope_block = true;
            continue;
        }
        if ["reviewed_at:", "reviewed_by:", "reviewed_with:", "reviewed_tasks_total:"]
            .iter()
            .any(|k| line.starts_with(k))
        {
            continue;
        }
        out.push_str(line);
    }
    out
}

/// 沿 in-progress add／set_board_rank 的同款防護：change 名稱必須是單一路徑段，
/// 否則可能經 store 的相對路徑觸及 changes/ 外的文件。
fn ensure_single_segment(name: &str) -> Result<()> {
    if name.contains(['/', '\\', ':']) || name.contains("..") {
        bail!("invalid change name: {name}");
    }
    Ok(())
}

/// Scope 項須是 repo-root 相對路徑：指紋讀檔以 `root.join(p)` 解析，絕對路徑會
/// 整個取代 root、`..` 會爬出工作樹。remote 模式的工單來自 server，若不守門即
/// 等於由 server 指定 client 讀哪個本機檔並把雜湊回傳——守門落在文法層，
/// stdin 與工單解析共用同一道。
fn ensure_repo_relative(path: &str) -> Result<()> {
    let normalized = path.replace('\\', "/");
    if normalized.starts_with('/')
        || normalized.split('/').any(|seg| seg == ".." || seg.contains(':'))
    {
        bail!(
            "scope path must be repo-root relative (no leading `/`, no `..`, \
             no drive letter): {path}"
        );
    }
    Ok(())
}

/// 解析並驗證一輪內容：非空 `**Scope**:` 清單＋合法 findings 行。
/// stdin 驗證與 show 的輪次解析共用同一文法——動詞產生的格式即動詞驗證的格式。
fn parse_round_body(content: &str) -> Result<(Vec<String>, Vec<Finding>)> {
    if content.trim().is_empty() {
        bail!("round content is empty — a round must carry a `**Scope**:` line");
    }
    let mut scope: Option<Vec<String>> = None;
    let mut findings = Vec::new();
    for line in content.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("**Scope**:") {
            if scope.is_some() {
                bail!("multiple `**Scope**:` lines in one round — one scope list per round");
            }
            let list: Vec<String> = rest
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
            if list.is_empty() {
                bail!("`**Scope**:` must list at least one repo-root relative path");
            }
            for path in &list {
                ensure_repo_relative(path)?;
            }
            scope = Some(list);
        } else if t.starts_with("##") && !t.starts_with("###") {
            // `## Round N` 是輪次分隔符——內容夾帶二級標題會偽造輪次結構。
            bail!("round content must not contain `## ` headings (round delimiter): {t}");
        } else if t.starts_with("- [") {
            findings.push(parse_finding(t)?);
        }
    }
    let Some(scope) = scope else {
        bail!("round content must contain a `**Scope**:` line listing the files reviewed");
    };
    Ok((scope, findings))
}

/// 解析一行 finding：`- [severity] path — 描述`。
fn parse_finding(line: &str) -> Result<Finding> {
    let rest = line.strip_prefix("- [").expect("caller matched the prefix");
    let Some((sev, tail)) = rest.split_once(']') else {
        bail!("malformed finding line (no closing `]`): {line}");
    };
    let Some(severity) = Severity::parse(sev) else {
        bail!("unknown severity '{sev}' — expected CRITICAL, WARNING, or SUGGESTION");
    };
    let Some((path, text)) = tail.trim_start().split_once(" — ") else {
        bail!("malformed finding line (expected `- [severity] path — text`): {line}");
    };
    Ok(Finding {
        severity,
        path: path.trim().to_string(),
        text: text.trim().to_string(),
    })
}

/// 解析整份工單：`## Round N` 分段，每段過同一輪文法。
fn parse_ticket(text: &str) -> Result<Ticket> {
    let mut rounds = Vec::new();
    let mut current: Option<(usize, String)> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("## Round ") {
            if let Some((index, body)) = current.take() {
                rounds.push(build_round(index, &body)?);
            }
            let index: usize = rest
                .trim()
                .parse()
                .map_err(|_| anyhow!("malformed round header in review ticket: {line}"))?;
            current = Some((index, String::new()));
        } else if let Some((_, body)) = current.as_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    if let Some((index, body)) = current.take() {
        rounds.push(build_round(index, &body)?);
    }
    if rounds.is_empty() {
        bail!("review ticket has no rounds — corrupt document");
    }
    Ok(Ticket { rounds })
}

fn build_round(index: usize, body: &str) -> Result<Round> {
    let (scope, findings) = parse_round_body(body)?;
    Ok(Round { index, scope, findings })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::teststore::TestStore;

    const META: &str = "schema: spec-driven\ncreated: 2026-07-01\n";

    const ROUND_1: &str = "**Scope**: crates/a/src/lib.rs, crates/b/src/util.rs\n\n- [CRITICAL] crates/a/src/lib.rs — unwrap on user input\n- [SUGGESTION] crates/b/src/util.rs — rename helper\n";
    const ROUND_2: &str = "**Scope**: crates/a/src/lib.rs\n\n- [WARNING] crates/a/src/lib.rs — possible Feature Envy\n";

    fn store_with_change() -> TestStore {
        TestStore::with_meta("demo", META)
    }

    // --- spec「審查工單的建立與追加」---

    #[test]
    fn add_round_creates_ticket_with_round_1_on_first_call() {
        // spec Scenario「首輪建立工單」：無工單＋合法內容 → 建檔且自 Round 1 起算。
        let store = store_with_change();
        let round = add_round(&store, "demo", ROUND_1).expect("first round");
        assert_eq!(round, 1);
        let doc = store.read_artifact("demo", REVIEW_DOC).expect("ticket must be created");
        assert!(doc.contains("## Round 1"), "fixed skeleton must carry the round header: {doc}");
        assert!(
            doc.contains("**Scope**: crates/a/src/lib.rs"),
            "round content must be carried verbatim: {doc}"
        );
    }

    #[test]
    fn add_round_appends_round_2_keeping_round_1_byte_identical() {
        // spec Scenario「追加輪次不改寫既有輪」：append-only，Round 1 位元級不變。
        let store = store_with_change();
        add_round(&store, "demo", ROUND_1).expect("first round");
        let after_first = store.read_artifact("demo", REVIEW_DOC).expect("ticket");
        let round = add_round(&store, "demo", ROUND_2).expect("second round");
        assert_eq!(round, 2);
        let after_second = store.read_artifact("demo", REVIEW_DOC).expect("ticket");
        assert!(
            after_second.starts_with(&after_first),
            "append-only: Round 1 must stay byte-identical\nbefore: {after_first}\nafter: {after_second}"
        );
        assert!(after_second.contains("## Round 2"));
    }

    #[test]
    fn add_round_rejects_missing_change_without_writing() {
        // spec Scenario「change 不存在」：拒絕且無檔案建立。
        let store = store_with_change();
        let err = add_round(&store, "ghost", ROUND_1).expect_err("missing change must be rejected");
        assert!(err.to_string().contains("ghost"), "error must name the change: {err}");
        // 沿 in-progress add／set_board_rank 的同款防護：非單一路徑段名稱拒絕。
        assert!(add_round(&store, "../evil", ROUND_1).is_err());
        assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
    }

    #[test]
    fn add_round_rejects_content_without_scope_without_writing() {
        // spec Scenario「內容缺少 Scope」：缺 `**Scope**:`（或清單為空）→ 拒絕、工單不變。
        let store = store_with_change();
        for bad in ["", "   \n", "- [CRITICAL] a.rs — no scope line\n", "**Scope**:  \n"] {
            let res = add_round(&store, "demo", bad);
            let Err(err) = res else {
                panic!("content {bad:?} must be rejected");
            };
            assert!(err.to_string().contains("**Scope**:"), "error must explain the format: {err}");
        }
        assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
        assert!(!store.artifact_exists("demo", REVIEW_DOC));
    }

    #[test]
    fn add_round_rejects_malformed_findings_and_round_header_injection() {
        // 系統邊界驗證（stdin 為外部輸入）：severity 非三檔之一、findings 行文法
        // 破損、或內容夾帶 `## ` 行（偽造輪次分隔）→ 拒絕且零寫入。
        let store = store_with_change();
        for bad in [
            "**Scope**: a.rs\n- [BLOCKER] a.rs — unknown severity\n",
            "**Scope**: a.rs\n- [CRITICAL a.rs — unclosed bracket\n",
            "**Scope**: a.rs\n## Round 99\n",
        ] {
            assert!(add_round(&store, "demo", bad).is_err(), "content {bad:?} must be rejected");
        }
        assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
    }

    // --- spec「審查工單的讀取」---

    #[test]
    fn show_round_trips_rounds_scope_findings_and_last_round() {
        // spec Scenario「讀取既有工單的 JSON」的解析核心：rounds 長度、每輪 index、
        // scope 清單與分級 findings 逐欄位；lastRound 指向末輪。
        let store = store_with_change();
        add_round(&store, "demo", ROUND_1).expect("round 1");
        add_round(&store, "demo", ROUND_2).expect("round 2");
        let ticket = show(&store, "demo").expect("ticket parses");
        assert_eq!(ticket.rounds.len(), 2);
        let r1 = &ticket.rounds[0];
        assert_eq!(r1.index, 1);
        assert_eq!(
            r1.scope,
            vec!["crates/a/src/lib.rs".to_string(), "crates/b/src/util.rs".to_string()]
        );
        assert_eq!(
            r1.findings,
            vec![
                Finding {
                    severity: Severity::Critical,
                    path: "crates/a/src/lib.rs".to_string(),
                    text: "unwrap on user input".to_string(),
                },
                Finding {
                    severity: Severity::Suggestion,
                    path: "crates/b/src/util.rs".to_string(),
                    text: "rename helper".to_string(),
                },
            ]
        );
        let last = ticket.last_round();
        assert_eq!(last.index, 2);
        assert_eq!(last.scope, vec!["crates/a/src/lib.rs".to_string()]);
        assert_eq!(last.findings.len(), 1);
        assert_eq!(last.findings[0].severity, Severity::Warning);
        assert_eq!(last.findings[0].text, "possible Feature Envy");
    }

    #[test]
    fn show_errors_when_change_has_no_ticket() {
        // spec Scenario「無工單」：非零收場的核心語意——錯誤說明該 change 無工單。
        let store = store_with_change();
        let err = show(&store, "demo").expect_err("no ticket must error");
        assert!(err.to_string().contains("no review ticket"), "error must say so: {err}");
    }

    // --- spec「放棄審查」---

    #[test]
    fn discard_deletes_ticket_and_leaves_meta_untouched() {
        // spec Scenario「放棄既有工單」：工單刪除、`.openspec.yaml` 位元級不變。
        let store = store_with_change();
        add_round(&store, "demo", ROUND_1).expect("round 1");
        discard(&store, "demo").expect("discard");
        assert!(!store.artifact_exists("demo", REVIEW_DOC), "ticket must be gone");
        assert_eq!(store.meta("demo"), META, "metadata must stay byte-identical");
        assert_eq!(*store.meta_writes.borrow(), 0, "discard must not write metadata");
    }

    #[test]
    fn discard_errors_when_no_ticket() {
        // spec Scenario「無工單可放棄」。
        let store = store_with_change();
        let err = discard(&store, "demo").expect_err("no ticket must error");
        assert!(err.to_string().contains("no review ticket"), "error must say so: {err}");
    }

    // --- spec「蓋章守門與蓋章效果」---

    const TASKS_5_DONE: &str = "- [x] 1 a\n- [x] 2 b\n- [x] 3 c\n- [x] 4 d\n- [x] 5 e\n";
    const TASKS_4_OF_5: &str = "- [x] 1 a\n- [x] 2 b\n- [x] 3 c\n- [x] 4 d\n- [ ] 5 e\n";
    const CLEAN_ROUND: &str = "**Scope**: crates/a/src/lib.rs\n";

    const FILE_A: &str = "fn a() {}\n";
    const FILE_B: &str = "fn b() {}\n";

    /// repo 檔案讀取替身：固定 (path, content) 表。
    fn files<'a>(map: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |p: &str| map.iter().find(|(k, _)| *k == p).map(|(_, v)| v.to_string())
    }

    /// repo 檔案存在替身：與 `files` 共用同一張表。
    fn present<'a>(map: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> bool + 'a {
        move |p: &str| map.iter().any(|(k, _)| *k == p)
    }

    const REPO: &[(&str, &str)] =
        &[("crates/a/src/lib.rs", FILE_A), ("crates/b/src/util.rs", FILE_B)];

    fn stamp_demo(store: &TestStore, accept: bool) -> Result<()> {
        stamp(
            store,
            "demo",
            accept,
            Some("Rev <r@example.com>"),
            Some("claude"),
            &files(REPO),
            &present(REPO),
        )
    }

    #[test]
    fn stamp_refuses_when_tasks_incomplete() {
        // spec Scenario「任務未全完成即拒絕」：4/5 → 拒絕，metadata 與工單皆不變。
        let store = store_with_change();
        store.put_artifact("demo", "tasks.md", TASKS_4_OF_5);
        add_round(&store, "demo", CLEAN_ROUND).expect("round 1");
        let err = stamp_demo(&store, false).expect_err("incomplete tasks must refuse");
        assert!(err.to_string().contains("4/5"), "error must show the count: {err}");
        assert_eq!(store.meta("demo"), META, "metadata must stay byte-identical");
        assert_eq!(*store.meta_writes.borrow(), 0);
        assert!(store.artifact_exists("demo", REVIEW_DOC), "ticket must survive refusal");
    }

    #[test]
    fn stamp_refuses_unresolved_findings_without_accept() {
        // spec Scenario「末輪有未解 findings 且未帶 --accept」：拒絕並提示 --accept。
        let store = store_with_change();
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", ROUND_1).expect("round with findings");
        let err = stamp_demo(&store, false).expect_err("unresolved findings must refuse");
        assert!(err.to_string().contains("--accept"), "error must offer --accept: {err}");
        assert_eq!(*store.meta_writes.borrow(), 0);
        assert!(store.artifact_exists("demo", REVIEW_DOC));
    }

    #[test]
    fn stamp_with_accept_overrides_findings_and_stamps() {
        // spec Scenario「帶保留蓋章」：--accept → 章寫入且工單刪除。
        let store = store_with_change();
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", ROUND_1).expect("round with findings");
        stamp_demo(&store, true).expect("--accept must stamp");
        assert!(!store.artifact_exists("demo", REVIEW_DOC), "ticket must be deleted");
        let meta = ChangeMeta::from_text(Some(&store.meta("demo"))).expect("meta parses");
        assert!(meta.reviewed_at.is_some());
    }

    #[test]
    fn stamp_clean_round_writes_five_fields_and_deletes_ticket() {
        // spec Scenario「乾淨蓋章」＋Example「蓋章寫入的任務錨」：5/5 任務、末輪
        // 零 findings → 五欄位齊備（reviewed_tasks_total 為 5）、工單不存在。
        let store = store_with_change();
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", ROUND_1).expect("round 1 with findings");
        add_round(&store, "demo", CLEAN_ROUND).expect("round 2 clean");
        stamp_demo(&store, false).expect("clean stamp");
        assert!(!store.artifact_exists("demo", REVIEW_DOC), "ticket must be deleted");
        let raw = store.meta("demo");
        let meta = ChangeMeta::from_text(Some(&raw)).expect("meta parses");
        assert_eq!(meta.reviewed_at.as_deref(), Some(crate::util::today().as_str()));
        assert_eq!(meta.reviewed_by.as_deref(), Some("Rev <r@example.com>"));
        assert_eq!(meta.reviewed_with.as_deref(), Some("claude"));
        assert_eq!(meta.reviewed_tasks_total, Some(5));
        assert!(!meta.reviewed_scope.is_empty(), "scope fingerprints must be recorded");
        assert!(raw.starts_with(META), "existing fields preserved byte-for-byte: {raw}");
    }

    #[test]
    fn stamp_scope_is_sorted_union_of_all_rounds() {
        // design D3：指紋範圍＝工單各輪 Scope 聯集（去重、排序保確定性）。
        let store = store_with_change();
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", ROUND_1).expect("round 1: a + b");
        add_round(&store, "demo", CLEAN_ROUND).expect("round 2: a only");
        stamp_demo(&store, false).expect("stamp");
        let meta = ChangeMeta::from_text(Some(&store.meta("demo"))).expect("meta parses");
        let paths: Vec<&str> = meta.reviewed_scope.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(paths, vec!["crates/a/src/lib.rs", "crates/b/src/util.rs"]);
        assert_eq!(meta.reviewed_scope[0].hash, content_fingerprint(FILE_A));
        assert_eq!(meta.reviewed_scope[1].hash, content_fingerprint(FILE_B));
    }

    #[test]
    fn stamp_normalizes_backslash_paths_into_meta() {
        // design D3：Windows 路徑 `\` → `/` 正規化後寫入 reviewed_scope。
        let store = store_with_change();
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", "**Scope**: crates\\a\\src\\lib.rs\n").expect("round");
        stamp_demo(&store, false).expect("stamp");
        let meta = ChangeMeta::from_text(Some(&store.meta("demo"))).expect("meta parses");
        assert_eq!(meta.reviewed_scope[0].path, "crates/a/src/lib.rs");
    }

    #[test]
    fn stamp_refuses_when_no_ticket_or_every_scope_file_gone() {
        // 無工單不可蓋章；聯集全數消失代表工作樹與工單嚴重脫節——跳過到一個
        // 不剩就不是「審查過」，fail-closed 並指名檔案與處置。
        let store = store_with_change();
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        let err = stamp_demo(&store, false).expect_err("no ticket must refuse");
        assert!(err.to_string().contains("no review ticket"), "{err}");
        add_round(&store, "demo", "**Scope**: gone/missing.rs\n").expect("round");
        let err = stamp_demo(&store, false).expect_err("all-gone scope must refuse");
        assert!(err.to_string().contains("gone/missing.rs"), "error must name the file: {err}");
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn stamp_skips_scope_files_deleted_by_later_fixes() {
        // 引擎死檔卡章（Round 5 必修）：修正把早輪審過的檔刪除／改名後，聯集
        // 中的死檔無從指紋也無從再變動——跳過不入錨，其餘照常，蓋章不得永久
        // 卡死。存在但讀不到者仍 fail-closed（見 stamp_reports_unreadable_*）。
        let store = store_with_change();
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", ROUND_1).expect("round 1: a + b");
        add_round(&store, "demo", CLEAN_ROUND).expect("round 2 clean");
        let survivors: &[(&str, &str)] = &[("crates/a/src/lib.rs", FILE_A)];
        stamp(&store, "demo", false, None, None, &files(survivors), &present(survivors))
            .expect("deleted scope file must not block the stamp");
        assert!(!store.artifact_exists("demo", REVIEW_DOC), "ticket must be deleted");
        let meta = ChangeMeta::from_text(Some(&store.meta("demo"))).expect("meta parses");
        let paths: Vec<&str> = meta.reviewed_scope.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(paths, vec!["crates/a/src/lib.rs"], "dead path must not be anchored");
    }

    #[test]
    fn stamp_restamp_replaces_reviewed_fields_without_duplication() {
        // 再審後重蓋：五欄位原位更新（含多行 reviewed_scope 區塊），不留重複鍵，
        // 其餘欄位逐位元組保留——沿 started_*／board_rank 的文字手術紀律。
        let old = "schema: spec-driven\ncreated: 2026-07-01\nreviewed_at: 2026-07-10\nreviewed_by: Old <o@example.com>\nreviewed_with: codex\nreviewed_tasks_total: 3\nreviewed_scope:\n  - path: old/file.rs\n    hash: deadbeef\nboard_rank: n\n";
        let store = TestStore::with_meta("demo", old);
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", CLEAN_ROUND).expect("round");
        stamp_demo(&store, false).expect("re-stamp");
        let raw = store.meta("demo");
        assert_eq!(raw.matches("reviewed_at:").count(), 1, "no duplicate keys: {raw}");
        assert_eq!(raw.matches("reviewed_scope:").count(), 1, "no duplicate keys: {raw}");
        assert!(!raw.contains("old/file.rs"), "stale scope block must be gone: {raw}");
        assert!(raw.contains("schema: spec-driven\n"), "{raw}");
        assert!(raw.contains("created: 2026-07-01\n"), "{raw}");
        assert!(raw.contains("board_rank: n\n"), "{raw}");
        let meta = ChangeMeta::from_text(Some(&raw)).expect("meta parses");
        assert_eq!(meta.reviewed_tasks_total, Some(5));
        assert_eq!(meta.reviewed_scope.len(), 1);
        assert_eq!(meta.reviewed_scope[0].path, "crates/a/src/lib.rs");
    }

    #[test]
    fn stamp_refuses_on_corrupt_meta_without_writing() {
        // 沿 set_board_rank 的 fail-closed gate：壞 metadata 不得被疊寫。
        const BAD: &str = ": : :\n\t bad yaml [unclosed\n";
        let store = TestStore::with_meta("demo", BAD);
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", CLEAN_ROUND).expect("round");
        let err = stamp_demo(&store, false).expect_err("corrupt meta must refuse");
        assert!(
            err.to_string().contains("openspec/changes/demo/.openspec.yaml"),
            "error must name the metadata file: {err}"
        );
        assert_eq!(store.meta("demo"), BAD);
        assert_eq!(*store.meta_writes.borrow(), 0);
        assert!(store.artifact_exists("demo", REVIEW_DOC), "ticket must survive refusal");
    }

    // --- design D4a：scope 注入蓋章（remote 承載）---

    #[test]
    fn stamp_with_scope_stamps_using_provided_entries() {
        // D4a：工作樹持有者（remote CLI）預算好的 (path, hash) 直接入章，server
        // 不重算；亂序提交仍按 path 排序落章（決定性）。
        let store = store_with_change();
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", ROUND_1).expect("round with findings");
        add_round(&store, "demo", CLEAN_ROUND).expect("clean round");
        let entries = vec![
            scope_entry("crates/b/src/util.rs", &content_fingerprint(FILE_B)),
            scope_entry("crates/a/src/lib.rs", &content_fingerprint(FILE_A)),
        ];
        stamp_with_scope(&store, "demo", false, Some("Rev <r@example.com>"), Some("claude"), entries, vec![])
            .expect("provided-scope stamp");
        assert!(!store.artifact_exists("demo", REVIEW_DOC), "ticket must be deleted");
        let meta = ChangeMeta::from_text(Some(&store.meta("demo"))).expect("meta parses");
        let paths: Vec<&str> = meta.reviewed_scope.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(paths, ["crates/a/src/lib.rs", "crates/b/src/util.rs"]);
        assert_eq!(meta.reviewed_scope[0].hash, content_fingerprint(FILE_A));
        assert_eq!(meta.reviewed_tasks_total, Some(5));
    }

    #[test]
    fn stamp_with_scope_rejects_path_set_mismatch_without_writing() {
        // D4a：提交 path 集合與工單各輪 Scope 聯集不完全相等（CAS 式保護——
        // 工單在讀取後被追加輪次）→ 拒絕並指名差集，工單與 meta 皆不動。
        let store = store_with_change();
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", ROUND_1).expect("round"); // 聯集：a + b
        let missing = vec![scope_entry("crates/a/src/lib.rs", &content_fingerprint(FILE_A))];
        let err = stamp_with_scope(&store, "demo", true, None, None, missing, vec![])
            .expect_err("missing path must refuse");
        assert!(err.to_string().contains("crates/b/src/util.rs"), "must name the gap: {err}");
        let extra = vec![
            scope_entry("crates/a/src/lib.rs", &content_fingerprint(FILE_A)),
            scope_entry("crates/b/src/util.rs", &content_fingerprint(FILE_B)),
            scope_entry("crates/c/extra.rs", &content_fingerprint("x")),
        ];
        let err = stamp_with_scope(&store, "demo", true, None, None, extra, vec![])
            .expect_err("extra path must refuse");
        assert!(err.to_string().contains("crates/c/extra.rs"), "must name the extra: {err}");
        assert_eq!(*store.meta_writes.borrow(), 0);
        assert!(store.artifact_exists("demo", REVIEW_DOC), "ticket must survive refusal");
    }

    #[test]
    fn stamp_with_scope_rejects_duplicate_paths_without_writing() {
        // D4a 的集合相等是「集合」——同一 path 提交兩份雜湊時差集皆空而矇混過關，
        // 章會落兩筆同 path，freshness 逐筆比對必有一筆不符 → 該章永遠 stale。
        let store = store_with_change();
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", CLEAN_ROUND).expect("round"); // 聯集：a
        let dupes = vec![
            scope_entry("crates/a/src/lib.rs", &content_fingerprint(FILE_A)),
            scope_entry("crates/a/src/lib.rs", &content_fingerprint("tampered")),
        ];
        let err = stamp_with_scope(&store, "demo", true, None, None, dupes, vec![])
            .expect_err("duplicate path must refuse");
        assert!(err.to_string().contains("crates/a/src/lib.rs"), "must name the dupe: {err}");
        assert_eq!(*store.meta_writes.borrow(), 0);
        assert!(store.artifact_exists("demo", REVIEW_DOC), "ticket must survive refusal");
    }

    fn scope_entry(path: &str, hash: &str) -> crate::model::ReviewedScopeEntry {
        crate::model::ReviewedScopeEntry { path: path.to_string(), hash: hash.to_string() }
    }

    #[test]
    fn stamp_with_scope_accepts_a_declared_missing_partition() {
        // 引擎死檔卡章的 remote 面（Round 5 必修）：server 無工作樹，檔案是否
        // 仍存在只有 checkout 持有者知道——client 明示宣告 missing，server 驗
        // 「provided ∪ missing ＝聯集且不相交」後放行，章只錨仍存在的檔。
        let store = store_with_change();
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", ROUND_1).expect("round: a + b");
        add_round(&store, "demo", CLEAN_ROUND).expect("clean round");
        let entries = vec![scope_entry("crates/a/src/lib.rs", &content_fingerprint(FILE_A))];
        stamp_with_scope(
            &store,
            "demo",
            false,
            None,
            None,
            entries,
            vec!["crates/b/src/util.rs".into()],
        )
        .expect("declared-missing partition must stamp");
        let meta = ChangeMeta::from_text(Some(&store.meta("demo"))).expect("meta parses");
        let paths: Vec<&str> = meta.reviewed_scope.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(paths, vec!["crates/a/src/lib.rs"], "declared-missing path must not anchor");
    }

    #[test]
    fn stamp_with_scope_rejects_bad_missing_declarations() {
        // 分割不成立即拒：missing 與 provided 重疊、宣告聯集外的路徑、或宣告到
        // 一個不剩——工單與 meta 皆不動。
        let store = store_with_change();
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", CLEAN_ROUND).expect("round"); // 聯集：a
        let a_entry = || vec![scope_entry("crates/a/src/lib.rs", &content_fingerprint(FILE_A))];
        let err = stamp_with_scope(
            &store,
            "demo",
            true,
            None,
            None,
            a_entry(),
            vec!["crates/a/src/lib.rs".into()],
        )
        .expect_err("overlap must refuse");
        assert!(err.to_string().contains("crates/a/src/lib.rs"), "names the overlap: {err}");
        let err =
            stamp_with_scope(&store, "demo", true, None, None, a_entry(), vec!["not/in/union.rs".into()])
                .expect_err("outside-union declaration must refuse");
        assert!(err.to_string().contains("not/in/union.rs"), "names the stray: {err}");
        let err =
            stamp_with_scope(&store, "demo", true, None, None, vec![], vec!["crates/a/src/lib.rs".into()])
                .expect_err("empty remainder must refuse");
        assert!(err.to_string().contains("crates/a/src/lib.rs"), "names the gone files: {err}");
        assert_eq!(*store.meta_writes.borrow(), 0);
        assert!(store.artifact_exists("demo", REVIEW_DOC), "ticket must survive refusals");
    }

    #[test]
    fn stamp_quotes_scope_scalars_so_yaml_metacharacters_survive() {
        // path 以未引號純量寫出時，「空白＋#」會被當註解截斷（該檔永遠 stale），
        // 而 `@`／`*`／`&`／`!` 開頭讓整份 .openspec.yaml 解析失敗——之後所有
        // 動詞對該 change fail-closed。寫入端負責引號。
        let store = store_with_change();
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", "**Scope**: src/@odd #1.rs\n").expect("round");
        let repo: &[(&str, &str)] = &[("src/@odd #1.rs", FILE_A)];
        stamp(&store, "demo", false, None, None, &files(repo), &present(repo)).expect("stamp");
        let raw = store.meta("demo");
        let meta = ChangeMeta::from_text(Some(&raw)).expect("re-parse must survive: {raw}");
        assert_eq!(meta.reviewed_scope.len(), 1);
        assert_eq!(meta.reviewed_scope[0].path, "src/@odd #1.rs", "path round-trips: {raw}");
        assert_eq!(meta.reviewed_scope[0].hash, content_fingerprint(FILE_A));
    }

    #[test]
    fn restamp_strips_a_scope_block_containing_blank_lines() {
        // 手改過的 meta 可能在 reviewed_scope 區塊裡留空行；把空行當區塊結束會
        // 讓其後的縮排項原樣留下，重蓋後成「mapping 接孤立縮排序列」而解析不能。
        let old = "schema: spec-driven\nreviewed_at: 2026-07-10\nreviewed_tasks_total: 3\nreviewed_scope:\n  - path: old/a.rs\n    hash: dead\n\n  - path: old/b.rs\n    hash: beef\ncreated: 2026-07-01\n";
        let store = TestStore::with_meta("demo", old);
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", CLEAN_ROUND).expect("round");
        stamp_demo(&store, false).expect("re-stamp");
        let raw = store.meta("demo");
        assert!(!raw.contains("old/a.rs"), "stale scope must be gone: {raw}");
        assert!(!raw.contains("old/b.rs"), "stale scope must be gone across the blank: {raw}");
        assert!(raw.contains("created: 2026-07-01\n"), "unrelated fields survive: {raw}");
        let meta = ChangeMeta::from_text(Some(&raw)).expect("re-stamped meta must still parse");
        assert_eq!(meta.reviewed_scope.len(), 1);
    }

    #[test]
    fn stamp_reports_unreadable_scope_files_without_claiming_they_are_missing() {
        // read_file 回 None 也可能是「讀得到但不是 UTF-8」——說成「不存在」會把
        // 人送去找一個明明還在的檔。
        let store = store_with_change();
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", "**Scope**: assets/logo.png\n").expect("round");
        let err = stamp(&store, "demo", false, None, None, &files(&[]), &|_: &str| true)
            .expect_err("must refuse");
        let msg = err.to_string();
        assert!(msg.contains("assets/logo.png"), "names the file: {msg}");
        assert!(
            !msg.contains("does not exist"),
            "must not assert absence it cannot know: {msg}"
        );
    }

    #[test]
    fn stamp_survives_identity_and_agent_strings_carrying_yaml_indicators() {
        // `--agent "codex: cli"`、含 `#` 的 git user.name、或帶換行的身分字串以
        // 純量直寫會注入欄位或整份炸掉——而工單已在同一步刪除，無從回復。
        // 沿 started_* 的既有 clean() 紀律：控制字元壓平、危險純量加引號。
        let store = store_with_change();
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", CLEAN_ROUND).expect("round");
        stamp(
            &store,
            "demo",
            false,
            Some("Rev: the #1 <r@example.com>\nboard_rank: injected"),
            Some("codex: cli"),
            &files(REPO),
            &present(REPO),
        )
        .expect("stamp");
        let raw = store.meta("demo");
        let meta = ChangeMeta::from_text(Some(&raw)).expect("meta must still parse: {raw}");
        assert_eq!(meta.reviewed_with.as_deref(), Some("codex: cli"), "{raw}");
        assert!(
            meta.reviewed_by.as_deref().is_some_and(|by| by.starts_with("Rev: the #1")),
            "identity round-trips: {raw}"
        );
        assert!(!raw.contains("\nboard_rank: injected"), "no field injection: {raw}");
    }

    #[test]
    fn stamp_with_scope_survives_a_hash_carrying_a_newline() {
        // 提交的 hash 未經文法驗證（server 不重算），換行會讓雙引號純量跨行、
        // 續行落在第 0 欄 → meta 解析不能。
        let store = store_with_change();
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", CLEAN_ROUND).expect("round");
        let entries = vec![scope_entry("crates/a/src/lib.rs", "dead\nbeef: injected")];
        stamp_with_scope(&store, "demo", false, None, None, entries, vec![]).expect("stamp");
        let raw = store.meta("demo");
        ChangeMeta::from_text(Some(&raw)).expect("meta must still parse: {raw}");
        assert!(!raw.contains("\nbeef: injected"), "no field injection: {raw}");
    }

    #[test]
    fn restamp_strips_a_top_level_scope_sequence() {
        // `reviewed_scope:` 之下的序列項在第 0 欄也是合法 YAML；只認縮排區塊會把
        // `- path:` 留下，重蓋後 meta 成 mapping 混 sequence、之後所有動詞 fail-closed。
        let old = "schema: spec-driven\nreviewed_at: 2026-07-10\nreviewed_tasks_total: 3\nreviewed_scope:\n- path: old/file.rs\n  hash: deadbeef\n";
        let store = TestStore::with_meta("demo", old);
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", CLEAN_ROUND).expect("round");
        stamp_demo(&store, false).expect("re-stamp");
        let raw = store.meta("demo");
        assert!(!raw.contains("old/file.rs"), "stale scope sequence must be gone: {raw}");
        let meta = ChangeMeta::from_text(Some(&raw)).expect("re-stamped meta must still parse");
        assert_eq!(meta.reviewed_scope.len(), 1);
        assert_eq!(meta.reviewed_scope[0].path, "crates/a/src/lib.rs");
    }

    #[test]
    fn add_round_rejects_scope_paths_that_escape_the_repo_root() {
        // Scope 是指紋讀檔的路徑來源，而讀檔以 `root.join(p)` 解析——絕對路徑會
        // 取代 root、`..` 會往上爬。remote 模式的工單來自 server，等於由 server
        // 指定 client 讀哪個本機檔，故守門落在文法層（stdin 與工單解析共用）。
        let store = store_with_change();
        for bad in [
            "**Scope**: /etc/passwd\n",
            "**Scope**: ../../../etc/passwd\n",
            "**Scope**: crates/../../secrets.rs\n",
            "**Scope**: C:\\Windows\\win.ini\n",
        ] {
            let Err(err) = add_round(&store, "demo", bad) else {
                panic!("scope {bad:?} must be rejected");
            };
            assert!(
                err.to_string().contains("repo-root relative"),
                "error must explain the requirement: {err}"
            );
        }
        assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
    }

    // --- spec「內容指紋錨與失效判定」---

    #[test]
    fn content_fingerprint_normalizes_crlf_and_detects_change() {
        // 行尾 CRLF→LF 正規化後雜湊（git autocrlf 環境不誤降級）；內容不同則不同。
        assert_eq!(content_fingerprint("a\r\nb\r\n"), content_fingerprint("a\nb\n"));
        assert_ne!(content_fingerprint("a\nb\n"), content_fingerprint("a\nc\n"));
        let hex = content_fingerprint("");
        assert_eq!(hex.len(), 64, "sha-256 hex digest");
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// 帶完整章的 meta：tasks_total 任務錨＋entries 內容錨。
    fn stamped_meta(tasks_total: usize, entries: &[(&str, &str)]) -> ChangeMeta {
        let mut y = format!(
            "schema: spec-driven\nreviewed_at: 2026-08-01\nreviewed_by: Rev <r@example.com>\nreviewed_with: claude\nreviewed_tasks_total: {tasks_total}\nreviewed_scope:\n"
        );
        for (p, h) in entries {
            y.push_str(&format!("  - path: {p}\n    hash: {h}\n"));
        }
        ChangeMeta::from_text(Some(&y)).expect("meta parses")
    }

    #[test]
    fn freshness_all_anchors_match_is_fresh() {
        let h = content_fingerprint(FILE_A);
        let meta = stamped_meta(5, &[("crates/a/src/lib.rs", h.as_str())]);
        assert_eq!(freshness(&meta, 5, 5, &files(REPO)), Freshness::Fresh);
    }

    #[test]
    fn freshness_modified_scope_file_is_stale() {
        // spec Example「指紋比對」：檔案追加一行 → 現值雜湊不為 H1 → stale。
        let h1 = content_fingerprint(FILE_A);
        let meta = stamped_meta(5, &[("crates/a/src/lib.rs", h1.as_str())]);
        let grown = format!("{FILE_A}fn extra() {{}}\n");
        let now = [("crates/a/src/lib.rs", grown.as_str())];
        assert_eq!(freshness(&meta, 5, 5, &files(&now)), Freshness::Stale);
    }

    #[test]
    fn freshness_missing_scope_file_is_stale() {
        // spec：任一 scope 檔內容雜湊不符「含檔案已不存在」→ stale。
        let h = content_fingerprint(FILE_A);
        let meta = stamped_meta(5, &[("crates/a/src/lib.rs", h.as_str())]);
        assert_eq!(freshness(&meta, 5, 5, &files(&[])), Freshness::Stale);
    }

    #[test]
    fn freshness_line_ending_change_stays_fresh() {
        // spec Scenario「行尾差異不觸發失效」：LF → CRLF 仍 fresh。
        let h = content_fingerprint(FILE_A);
        let meta = stamped_meta(5, &[("crates/a/src/lib.rs", h.as_str())]);
        let crlf = FILE_A.replace('\n', "\r\n");
        let now = [("crates/a/src/lib.rs", crlf.as_str())];
        assert_eq!(freshness(&meta, 5, 5, &files(&now)), Freshness::Fresh);
    }

    #[test]
    fn freshness_task_anchor_breaks_on_recount_or_uncheck() {
        // spec：任務狀態不再是「蓋章當時任務總數的全完成」→ stale——新增任務
        //（總數變）與退勾（未全完成）皆觸發。
        let h = content_fingerprint(FILE_A);
        let meta = stamped_meta(5, &[("crates/a/src/lib.rs", h.as_str())]);
        assert_eq!(freshness(&meta, 6, 6, &files(REPO)), Freshness::Stale, "task count grew");
        assert_eq!(freshness(&meta, 5, 4, &files(REPO)), Freshness::Stale, "task unchecked");
    }

    #[test]
    fn freshness_unstamped_meta_is_unknown() {
        // 缺席讀作未審查：無章的 meta 沒有可判定的錨。
        let meta = ChangeMeta::from_text(Some(META)).expect("meta parses");
        assert_eq!(freshness(&meta, 5, 5, &files(REPO)), Freshness::Unknown);
    }
}
