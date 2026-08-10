//! 品質站的共通生命週期（design D1）：工單骨架產生／驗證／解析、structured
//! phase／patch、蓋章原子寫入、指紋計算、失效純函式與 archive 守門檢查。
//!
//! 站別差異收斂為一組常數（[`Station`]）：工單檔名、meta 欄位前綴、標題詞、
//! 訊息用的站別名詞，以及「add-round 是否要求任務全完成」的刻意不對稱。
//! 兩個薄實例分別是 [`crate::review`]（審查站）與 [`crate::verify`]（驗證站）——
//! 兩者只補常數組與委派，守門與失效規則永遠同一份實作。
//!
//! 工單本身是 sidecar：不註冊進 workflow schema，僅由動詞經 `&dyn Store` 讀寫
//! （與 discuss 動詞同型）——本地隨 git、remote 走 store 文件管道。

use crate::model::ReviewedScopeEntry;
use crate::store::Store;
use anyhow::{anyhow, bail, Result};

/// 一個品質站的站別常數組。新增站別＝新增一組常數＋委派函式，不建 trait。
pub struct Station {
    /// 工單文件（change 目錄下的相對路徑），如 `review.md`。
    pub doc: &'static str,
    /// meta 欄位前綴，如 `reviewed`——同時是訊息中的過去分詞（files reviewed）。
    pub meta_prefix: &'static str,
    /// 工單首行標題詞，如 `Review`。
    pub title: &'static str,
    /// 訊息與 CLI 家族名，如 `review`。
    pub noun: &'static str,
    /// 站別檢查活動的名詞片語，如 `review`／`verification`——[`noun`] 是 CLI 詞，
    /// 這個是散文詞（「finish the verification」而非「finish the verify」）。
    pub noun_phrase: &'static str,
    /// 「修正後重跑本站」的祈使詞，如 `re-review`。
    pub recheck: &'static str,
    /// 刻意不對稱（design D3）：`add-round` 是否要求任務全數完成。驗證工單語意
    /// 限定為成品驗證（verify 檢查可中途跑，盤點輪不得落工單）；審查站無此守門。
    pub round_requires_tasks_complete: bool,
}

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

/// 結構化輪次的 phase token（spec「審查工單的建立與追加」：discovery|validation）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundPhase {
    Discovery,
    Validation,
}

impl RoundPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoundPhase::Discovery => "discovery",
            RoundPhase::Validation => "validation",
        }
    }

    pub fn parse(s: &str) -> Option<RoundPhase> {
        match s {
            "discovery" => Some(RoundPhase::Discovery),
            "validation" => Some(RoundPhase::Validation),
            _ => None,
        }
    }
}

/// 一輪檢查：`## Round N` 區段的解析結果。`phase`／`patch_hash` 為結構化輪次
/// 的 frozen patch identity；legacy 輪次兩欄皆 None。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Round {
    pub index: usize,
    pub phase: Option<RoundPhase>,
    pub patch_hash: Option<String>,
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

/// 凍結度：章的雙錨判定結果（design D3——讀取端純函式，desktop-core 呼叫，
/// CLI 不輸出）。Unknown＝meta 未帶完整的章（缺席讀作未蓋章）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freshness {
    Fresh,
    Stale,
    Unknown,
}

/// 章的雙錨原料（自站別 meta 欄位取出後交給共用判定）。
pub struct StampAnchors<'a> {
    pub stamped_at: Option<&'a str>,
    pub tasks_total: Option<usize>,
    pub scope: &'a [ReviewedScopeEntry],
}

/// 追加一輪（工單不存在則建立，自 Round 1 起算）。回傳本輪編號。
/// 內容須含非空的 `**Scope**:` 檔案清單；`- [` 開頭行須為合法 findings 行；
/// 既有輪次 append-only 不改寫（spec「審查／驗證工單的建立與追加」）。
pub fn add_round(st: &Station, store: &dyn Store, change: &str, content: &str) -> Result<usize> {
    ensure_single_segment(change)?;
    if !store.change_exists(change) {
        bail!(NotFound(format!("change not found: {change}")));
    }
    // 寫入前先驗證（系統邊界：stdin 為外部輸入）——拒絕路徑零寫入。
    let body = parse_round_body(st, content)?;
    // 刻意不對稱（design D3）：驗證工單語意限定為成品驗證，中途盤點輪不落工單
    // ——誤落的盤點輪會讓「未結工單」失去語意，還會誤觸 archive 守門。
    if st.round_requires_tasks_complete {
        let tasks_md = store.read_artifact(change, "tasks.md").unwrap_or_default();
        let (total, complete, _) = crate::tasks::progress(&crate::tasks::parse(&tasks_md));
        if total > 0 && complete < total {
            bail!(crate::command::Refusal(format!(
                "change '{change}' has {complete}/{total} tasks complete — a {} ticket \
                 records the finished work; mid-flight check-ins stay in the conversation",
                st.noun
            )));
        }
    }
    // 追加前解析既有工單：壞檔 fail-closed，不得在其上疊寫。
    let existing = store.read_artifact(change, st.doc);
    let ticket = existing.as_deref().map(|t| parse_ticket(st, t)).transpose()?;
    // Sequence guard（spec「工單的建立與追加」）：工單首個結構化 round 是
    // discovery；已有結構化 round 後只能追加 validation；validation 必須有可
    // 驗收的既有輪次（legacy ticket 也算）。
    match body.phase {
        Some(RoundPhase::Discovery)
            if ticket.as_ref().is_some_and(|t| t.rounds.iter().any(|r| r.phase.is_some())) =>
        {
            bail!(
                "the ticket already carries a structured round — subsequent structured \
                 rounds must be validation"
            );
        }
        Some(RoundPhase::Validation) if ticket.is_none() => {
            bail!(
                "a validation round needs an existing ticket to validate — the first \
                 structured round is discovery"
            );
        }
        _ => {}
    }
    let (mut text, next) = match existing {
        Some(existing) => {
            let next = ticket.expect("existing document parsed above").last_round().index + 1;
            (existing, next)
        }
        None => (format!("# {} — {change}\n", st.title), 1),
    };
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str(&format!("\n## Round {next}\n\n{}\n", content.trim_end()));
    store.write_artifact(change, st.doc, &text)?;
    Ok(next)
}

/// 讀取並解析工單（spec「工單的讀取」）。無工單回錯誤。
pub fn show(st: &Station, store: &dyn Store, change: &str) -> Result<Ticket> {
    show_with_content(st, store, change).map(|(ticket, _)| ticket)
}

/// `show` 加帶工單原文：同一次讀取同時給出解析結果與文件文本，讓人眼路徑
/// （印原文）與 wire 回應（content 欄位）不必各自再讀一次同一份文件。
/// 原文恆為 `Some`——工單缺席在這裡就是錯誤，`Option` 只為呼叫端與 wire 的
/// 選填欄位同形。
pub fn show_with_content(
    st: &Station,
    store: &dyn Store,
    change: &str,
) -> Result<(Ticket, Option<String>)> {
    ensure_single_segment(change)?;
    if !store.change_exists(change) {
        bail!(NotFound(format!("change not found: {change}")));
    }
    let Some(text) = store.read_artifact(change, st.doc) else {
        bail!(NotFound(format!("no {} ticket for change '{change}'", st.noun)));
    };
    let ticket = parse_ticket(st, &text)?;
    Ok((ticket, Some(text)))
}

/// 放棄本站：刪除工單、不寫任何 metadata（spec「放棄審查／放棄驗證」）。
/// 無工單回錯誤。
pub fn discard(st: &Station, store: &dyn Store, change: &str) -> Result<()> {
    ensure_single_segment(change)?;
    if !store.change_exists(change) {
        bail!(NotFound(format!("change not found: {change}")));
    }
    if !store.artifact_exists(change, st.doc) {
        bail!(NotFound(format!("no {} ticket for change '{change}'", st.noun)));
    }
    store.delete_artifact(change, st.doc)
}

/// scope 注入蓋章（design D4a）：remote 承載——工作樹持有者預算好的
/// (path, hash) 清單直接入章。守門與 `stamp` 完全相同；額外驗證分割
/// 「provided ∪ missing ＝工單各輪 Scope 聯集且不相交」（CAS 式保護），
/// 不成立即拒。`missing` 是 checkout 持有者對「聯集中已不存在的檔」的明示
/// 宣告——server 無工作樹無從驗證存在性，宣告與雜湊同屬提交端的權威。
pub fn stamp_with_scope(
    st: &Station,
    store: &dyn Store,
    change: &str,
    accept: bool,
    actor: Option<&str>,
    tool: Option<&str>,
    scope: Vec<ReviewedScopeEntry>,
    missing: Vec<String>,
) -> Result<()> {
    let gate = stamp_gate(st, store, change, accept)?;
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
    write_stamp(st, store, change, &gate, actor, tool, &entries)
}

/// 蓋章（spec「蓋章守門與蓋章效果」）：守門＝任務全完成＋末輪零待處理必修
/// （CRITICAL／WARNING）findings，SUGGESTION 不擋章（`accept` 僅豁免必修條件）；
/// 通過時以工單各輪 Scope 聯集計算指紋，於同一原子寫入內落五個站別欄位並刪除
/// 工單。`read_file` 供指紋計算讀取 repo-root 相對
/// 路徑的檔案內容（remote 模式亦讀本地工作樹）；`file_exists` 判定聯集檔案是否
/// 仍在工作樹——修正可能刪除／改名早輪檢查過的檔，死檔跳過不入錨（無從指紋也
/// 無從再變動），存在但讀不到者仍 fail-closed。
pub fn stamp(
    st: &Station,
    store: &dyn Store,
    change: &str,
    accept: bool,
    actor: Option<&str>,
    tool: Option<&str>,
    read_file: &dyn Fn(&str) -> Option<String>,
    file_exists: &dyn Fn(&str) -> bool,
) -> Result<()> {
    let gate = stamp_gate(st, store, change, accept)?;
    let (present, gone): (Vec<String>, Vec<String>) =
        gate.paths.iter().cloned().partition(|p| file_exists(p));
    ensure_scope_remainder(&present, &gone)?;
    let entries = fingerprint_scope(&present, read_file)?;
    write_stamp(st, store, change, &gate, actor, tool, &entries)
}

/// 跳過死檔跳到一個不剩就不是「檢查過」——工作樹與工單嚴重脫節，fail-closed
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
/// 讀不到就無從證明檢查過的是哪份內容。路徑在讀取前過 repo-root 相對守門：
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

fn stamp_gate(st: &Station, store: &dyn Store, change: &str, accept: bool) -> Result<StampGate> {
    ensure_single_segment(change)?;
    if !store.change_exists(change) {
        bail!(NotFound(format!("change not found: {change}")));
    }
    // Fail-closed gate（沿 set_board_rank）：文字手術前先解析，壞檔不得被疊寫。
    let raw_meta = store.read_change_meta(change).unwrap_or_default();
    crate::model::check_meta_text(change, Some(&raw_meta))?;
    let Some(ticket_text) = store.read_artifact(change, st.doc) else {
        bail!(NotFound(format!("no {} ticket for change '{change}'", st.noun)));
    };
    let ticket = parse_ticket(st, &ticket_text)?;

    // 守門 (1)：任務全數完成（零任務 change 比照 archive gate 放行）。
    let tasks_md = store.read_artifact(change, "tasks.md").unwrap_or_default();
    let (total, complete, _) = crate::tasks::progress(&crate::tasks::parse(&tasks_md));
    if total > 0 && complete < total {
        bail!(crate::command::Refusal(format!(
            "change '{change}' has {complete}/{total} tasks complete — {} stamp \
             requires all tasks done",
            st.noun
        )));
    }
    // 守門 (2)：末輪零待處理必修（CRITICAL／WARNING）findings；SUGGESTION 不擋章；
    // `--accept` 僅豁免此條。計數含帶 `(accepted)` 的必修行（design D2：接受不豁免
    // 守門，只改走 `--accept`），故訊息用 outstanding 而非 unresolved。
    let outstanding = ticket
        .last_round()
        .findings
        .iter()
        .filter(|f| f.severity != Severity::Suggestion)
        .count();
    if outstanding > 0 && !accept {
        bail!(crate::command::Refusal(format!(
            "the last round has {outstanding} outstanding must-fix finding(s) (CRITICAL/WARNING) \
             — fix and {}, or pass --accept to stamp with reservations",
            st.recheck
        )));
    }

    let paths =
        scope_union(ticket.rounds.iter().flat_map(|r| r.scope.iter().map(String::as_str)));
    Ok(StampGate { raw_meta, paths, tasks_total: total })
}

fn write_stamp(
    st: &Station,
    store: &dyn Store,
    change: &str,
    gate: &StampGate,
    actor: Option<&str>,
    tool: Option<&str>,
    entries: &[(String, String)],
) -> Result<()> {
    // 文字手術：先剝除既有站別區塊（重蓋不留重複鍵），再附加新章；其餘欄位
    // 逐位元組保留。
    let prefix = st.meta_prefix;
    let mut out = strip_stamp_lines(st, &gate.raw_meta);
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    // 身分／工具／指紋一律過 YAML 純量守門（沿 started_* 的同一道）：帶 `:`、
    // `#` 或換行的字串會注入欄位或炸掉整份 meta——而工單已在同一步刪除，
    // 無從回復。
    out.push_str(&format!("{prefix}_at: {}\n", crate::util::today()));
    if let Some(actor) = actor {
        out.push_str(&format!("{prefix}_by: {}\n", crate::util::yaml_scalar(actor)));
    }
    if let Some(tool) = tool {
        out.push_str(&format!("{prefix}_with: {}\n", crate::util::yaml_scalar(tool)));
    }
    out.push_str(&format!("{prefix}_tasks_total: {}\n", gate.tasks_total));
    out.push_str(&format!("{prefix}_scope:\n"));
    for (path, hash) in entries {
        out.push_str(&format!(
            "  - path: {}\n    hash: {}\n",
            crate::util::yaml_scalar(path),
            crate::util::yaml_scalar(hash)
        ));
    }

    // 同一原子寫入（design D3）：remote 走 bridge 的 staged commit 天然原子；
    // 本地順序為先刪工單再寫章——中斷時寧可退回「未檢查」，也不得出現
    // 「章已寫而工單仍在」的半套狀態（spec 明文禁止的唯一中間態）。
    store.delete_artifact(change, st.doc)?;
    store.write_change_meta(change, &out)?;
    Ok(())
}

/// 內容指紋：CRLF→LF 正規化後的 SHA-256 十六進位（spec「內容指紋錨與失效判定」）。
/// 兩站共用同一實作——指紋規則位元級同構是 design D2 的硬性要求。
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
    anchors: StampAnchors<'_>,
    tasks_total: usize,
    tasks_complete: usize,
    read_file: &dyn Fn(&str) -> Option<String>,
) -> Freshness {
    let Some(stamped_total) = anchors.tasks_total else {
        return Freshness::Unknown;
    };
    if anchors.stamped_at.is_none() || anchors.scope.is_empty() {
        return Freshness::Unknown;
    }
    if tasks_total != stamped_total || tasks_complete != tasks_total {
        return Freshness::Stale;
    }
    for entry in anchors.scope {
        match read_file(&entry.path) {
            Some(content) if content_fingerprint(&content) == entry.hash => {}
            _ => return Freshness::Stale,
        }
    }
    Freshness::Fresh
}

/// 未結工單的處置說明（design D4／D5）——archive 守門對每個有未結工單的站別
/// 各產生一段，並列後成為拒絕訊息。
pub fn open_ticket_disposal(st: &Station, name: &str) -> String {
    let (noun, phrase) = (st.noun, st.noun_phrase);
    // 指令欄對齊：標籤長度隨站別詞變動（review／verification），寬度由最長標籤
    // 決定——寫死欄位會讓較長的站別詞把指令擠歪。
    let labels = [
        format!("finish the {phrase} and stamp it:"),
        format!("abandon the {phrase}:"),
        "archive it as-is:".to_string(),
    ];
    let w = labels.iter().map(String::len).max().expect("three labels") + 2;
    format!(
        "change '{name}' has an open {noun} ticket ({}) — settle it before archiving:\n  \
         {:<w$}speclink {noun} stamp {name}\n  \
         {:<w$}speclink {noun} discard {name}\n  \
         {:<w$}speclink archive {name} --carry-{noun} \
         (permanently shown as {}-not-passed)",
        st.doc,
        labels[0],
        labels[1],
        labels[2],
        st.meta_prefix,
        w = w
    )
}

/// 剝除 meta 的全部站別頂層行（含 `<prefix>_scope:` 之下的縮排區塊），
/// 其餘行逐位元組保留——重蓋章的前半場手術。
fn strip_stamp_lines(st: &Station, meta: &str) -> String {
    let prefix = st.meta_prefix;
    let scope_key = format!("{prefix}_scope:");
    let scalar_keys =
        [format!("{prefix}_at:"), format!("{prefix}_by:"), format!("{prefix}_with:"), format!("{prefix}_tasks_total:")];
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
        if line.starts_with(scope_key.as_str()) {
            in_scope_block = true;
            continue;
        }
        if scalar_keys.iter().any(|k| line.starts_with(k)) {
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

/// 一輪內容的解析結果（stdin 驗證與 show 的輪次解析共用）。
struct RoundBody {
    scope: Vec<String>,
    findings: Vec<Finding>,
    phase: Option<RoundPhase>,
    patch_hash: Option<String>,
}

/// 解析並驗證一輪內容：非空 `**Scope**:` 清單＋合法 findings 行；結構化輪次
/// 另帶成對的 `**Phase**:` 與 `**Patch**:`（spec「工單的建立與追加」）。
/// stdin 驗證與 show 的輪次解析共用同一文法——動詞產生的格式即動詞驗證的格式。
fn parse_round_body(st: &Station, content: &str) -> Result<RoundBody> {
    if content.trim().is_empty() {
        bail!("round content is empty — a round must carry a `**Scope**:` line");
    }
    let mut scope: Option<Vec<String>> = None;
    let mut findings = Vec::new();
    let mut phase: Option<RoundPhase> = None;
    let mut patch_hash: Option<String> = None;
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
        } else if let Some(rest) = t.strip_prefix("**Phase**:") {
            if phase.is_some() {
                bail!("multiple `**Phase**:` lines in one round");
            }
            let token = rest.trim();
            phase = Some(RoundPhase::parse(token).ok_or_else(|| {
                anyhow!("unknown phase '{token}' — expected discovery or validation")
            })?);
        } else if let Some(rest) = t.strip_prefix("**Patch**:") {
            if patch_hash.is_some() {
                bail!("multiple `**Patch**:` lines in one round");
            }
            patch_hash = Some(ensure_patch_hash(rest.trim())?);
        } else if t.starts_with("##") && !t.starts_with("###") {
            // `## Round N` 是輪次分隔符——內容夾帶二級標題會偽造輪次結構。
            bail!("round content must not contain `## ` headings (round delimiter): {t}");
        } else if t.starts_with("- [") {
            findings.push(parse_finding(t)?);
        }
    }
    if phase.is_some() != patch_hash.is_some() {
        bail!("`**Phase**:` and `**Patch**:` must appear together — a structured round \
               binds its phase to a frozen patch hash");
    }
    let Some(scope) = scope else {
        bail!(
            "round content must contain a `**Scope**:` line listing the files {}",
            st.meta_prefix
        );
    };
    Ok(RoundBody { scope, findings, phase, patch_hash })
}

/// Patch 行的格式守門：`sha256:` ＋恰好 64 個小寫十六進位字元。
fn ensure_patch_hash(s: &str) -> Result<String> {
    let hex = s
        .strip_prefix("sha256:")
        .ok_or_else(|| anyhow!("malformed patch hash '{s}' — expected `sha256:<64 hex>`"))?;
    if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()) {
        bail!("malformed patch hash '{s}' — expected exactly 64 lowercase hex digits");
    }
    Ok(s.to_string())
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
fn parse_ticket(st: &Station, text: &str) -> Result<Ticket> {
    let mut rounds = Vec::new();
    let mut current: Option<(usize, String)> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("## Round ") {
            if let Some((index, body)) = current.take() {
                rounds.push(build_round(st, index, &body)?);
            }
            let index: usize = rest.trim().parse().map_err(|_| {
                anyhow!("malformed round header in {} ticket: {line}", st.noun)
            })?;
            current = Some((index, String::new()));
        } else if let Some((_, body)) = current.as_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    if let Some((index, body)) = current.take() {
        rounds.push(build_round(st, index, &body)?);
    }
    if rounds.is_empty() {
        bail!("{} ticket has no rounds — corrupt document", st.noun);
    }
    Ok(Ticket { rounds })
}

fn build_round(st: &Station, index: usize, body: &str) -> Result<Round> {
    let body = parse_round_body(st, body)?;
    Ok(Round {
        index,
        phase: body.phase,
        patch_hash: body.patch_hash,
        scope: body.scope,
        findings: body.findings,
    })
}

#[cfg(test)]
mod tests {
    //! 共用蓋章守門的分界測試（design D2：改一處兩站同時生效）——站別 wiring
    //! 與五欄寫入由 review／verify 各自的測試模組覆蓋。
    use super::*;
    use crate::teststore::TestStore;
    use crate::verify::STATION;

    const META: &str = "schema: spec-driven\ncreated: 2026-07-01\n";
    const REPO: &[(&str, &str)] = &[("crates/a/src/lib.rs", "fn a() {}\n")];

    fn store_with_round(round: &str) -> TestStore {
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", "- [x] 1 a\n");
        add_round(&STATION, &store, "demo", round).expect("round recorded");
        store
    }

    fn stamp_demo(store: &TestStore, accept: bool) -> Result<()> {
        let files = |p: &str| REPO.iter().find(|(k, _)| *k == p).map(|(_, v)| v.to_string());
        let present = |p: &str| REPO.iter().any(|(k, _)| *k == p);
        stamp(&STATION, store, "demo", accept, None, None, &files, &present)
    }

    #[test]
    fn gate_ignores_suggestion_findings() {
        // 守門 (2) 的分界：SUGGESTION 不是必修，僅 SUGGESTION 的末輪放行。
        let store = store_with_round(
            "**Scope**: crates/a/src/lib.rs\n\n- [SUGGESTION] crates/a/src/lib.rs — nit\n",
        );
        stamp_demo(&store, false).expect("suggestion-only last round must pass the gate");
    }

    #[test]
    fn gate_counts_only_must_fix_and_names_the_count() {
        // 混合輪：CRITICAL＋WARNING＋SUGGESTION → 計數 2（SUGGESTION 不計），
        // 訊息點名數量與阻斷級別。
        let store = store_with_round(
            "**Scope**: crates/a/src/lib.rs\n\n\
             - [CRITICAL] crates/a/src/lib.rs — broken\n\
             - [WARNING] crates/a/src/lib.rs — fragile\n\
             - [SUGGESTION] crates/a/src/lib.rs — nit\n",
        );
        let err = stamp_demo(&store, false).expect_err("must-fix findings must refuse");
        let msg = err.to_string();
        assert!(msg.contains("2 outstanding must-fix"), "count skips SUGGESTION: {msg}");
        assert!(msg.contains("(CRITICAL/WARNING)"), "names the blocking severities: {msg}");
    }

    #[test]
    fn gate_counts_accepted_must_fix_lines_too() {
        // design D2：`(accepted)` 不另設豁免——已受理必修照樣擋乾淨章、`--accept`
        // 才放行；訊息以 outstanding（而非 unresolved）涵蓋已裁決未修者。
        let store = store_with_round(
            "**Scope**: crates/a/src/lib.rs\n\n\
             - [WARNING] crates/a/src/lib.rs — fragile (accepted)\n",
        );
        let err = stamp_demo(&store, false).expect_err("accepted must-fix still blocks");
        assert!(err.to_string().contains("1 outstanding must-fix"), "{err}");
        stamp_demo(&store, true).expect("--accept stamps over accepted must-fix");
    }
}
