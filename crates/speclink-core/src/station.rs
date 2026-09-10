//! 品質站的共通生命週期（design D1）：工單骨架產生／驗證／解析、structured
//! phase／patch、蓋章原子寫入、指紋計算、失效純函式與 archive 守門檢查。
//!
//! 站別差異收斂為一組常數（[`Station`]）：工單檔名、meta 欄位前綴、標題詞、
//! 訊息用的站別名詞，以及「add-round 是否要求任務全完成」的刻意不對稱。
//! 兩個站就是兩組常數（[`REVIEW`]／[`VERIFY`]）——守門與失效規則永遠同一份
//! 實作，呼叫端直接帶站別常數呼叫本模組的函式，沒有站別門面。
//!
//! 工單本身是 sidecar：不註冊進 workflow schema，僅由動詞經 `&dyn Store` 讀寫
//! （與 discuss 動詞同型）——本地隨 git、remote 走 store 文件管道。

use crate::keylines::KeyLines;
use crate::model::{ChangeMeta, ReviewedScopeEntry};
use crate::store::Store;
use anyhow::{anyhow, bail, Result};

/// 站別識別（lifecycle-station-anchors D1）：[`crate::model::ChangeMeta::anchors`]
/// 對它窮舉 match 選章欄位——第三站來時漏補臂是編譯錯誤，不是靜默讀到空章。
/// 除此之外不對外有其他用途（CLI 的 snapshot namespace `StationNs` 是另一層）。
#[derive(Clone, Copy)]
pub enum StationId {
    Review,
    Verify,
}

/// 一個品質站的站別常數組。新增站別＝新增一組常數（[`REVIEW`]／[`VERIFY`]
/// 之外再加一組）、[`StationId`] 加一值，不建 trait。
pub struct Station {
    /// 站別識別——章欄位對映的 match 鍵。
    pub id: StationId,
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

/// 審查站工單文件（change 目錄下的相對路徑）。
pub const REVIEW_DOC: &str = "review.md";

/// 驗證站工單文件（change 目錄下的相對路徑）。
pub const VERIFY_DOC: &str = "verify.md";

/// 審查站的站別常數組。`round_requires_tasks_complete` 為 false 是刻意的：
/// 審查技能的執行起點即自檢任務完成度，不存在中途跑情境，守門留在技能層。
pub const REVIEW: Station = Station {
    id: StationId::Review,
    doc: REVIEW_DOC,
    meta_prefix: "reviewed",
    title: "Review",
    noun: "review",
    noun_phrase: "review",
    recheck: "re-review",
    round_requires_tasks_complete: false,
};

/// 驗證站的站別常數組。`round_requires_tasks_complete` 為 true 是
/// verify-station-parity design D3 的刻意不對稱：盤點輪誤落工單會讓「未結工單」
/// 失去語意、還會誤觸 archive 守門。
pub const VERIFY: Station = Station {
    id: StationId::Verify,
    doc: VERIFY_DOC,
    meta_prefix: "verified",
    title: "Verify",
    noun: "verify",
    noun_phrase: "verification",
    recheck: "re-verify",
    round_requires_tasks_complete: true,
};

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

/// 讀檔閉包本體：repo-root 相對路徑 → 檔案原始位元組；回 None＝缺檔或 I/O
/// 失敗（UTF-8 分流收在指紋入口，讀取器不預判文字性）。指紋與失效判定的
/// 全部簽名共用這一個別名——下次再換讀取形不必改遍每個簽名。
pub type ScopeReadFn<'a> = dyn Fn(&str) -> Option<Vec<u8>> + 'a;

/// 內容錨的讀取器：None 代表沒有工作樹可讀（remote 封存通道），內容錨因此
/// 不判定。
pub type ScopeReader<'a> = Option<&'a ScopeReadFn<'a>>;

/// 章失效的原因——封存守門的拒絕訊息素材（design D5）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaleReason {
    /// 任務錨破：總數已非蓋章時的數，或仍有寫碼任務未完成。
    TaskAnchor { stamped_total: usize, total: usize, code_complete: usize, code_total: usize },
    /// 內容錨破：這個 scope 檔的現值指紋不符（含檔案已不存在）。
    ContentAnchor { path: String },
}

/// 章的雙錨原料（自站別 meta 欄位取出後交給共用判定）。
#[derive(Clone, Copy)]
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
        let counts = crate::tasks::counts_for(store, change);
        if !counts.code_done() {
            bail!(crate::command::Refusal(format!(
                "change '{change}' has {}/{} code tasks complete — a {} ticket \
                 records the finished work; mid-flight check-ins stay in the conversation",
                counts.code_complete, counts.code_total, st.noun
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
    read_file: &ScopeReadFn<'_>,
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
    read_file: &ScopeReadFn<'_>,
) -> Result<Vec<(String, String)>> {
    let mut entries = Vec::with_capacity(paths.len());
    for path in paths {
        ensure_repo_relative(path)?;
        let Some(content) = read_file(path) else {
            // 讀不到可能是缺檔，也可能是 I/O 失敗（非 UTF-8 不是讀取失敗，
            // 走位元組指紋）——只說事實。
            bail!("cannot read scope file '{path}' — it must be present and readable");
        };
        entries.push((path.clone(), content_fingerprint_bytes(&content)));
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

    // 守門 (1)：寫碼任務全數完成——`[M]` 手測任務不計（章＝驗證過，成品定案的
    // 語意由封存承接）。零寫碼任務比照 archive gate 放行。
    let counts = crate::tasks::counts_for(store, change);
    if !counts.code_done() {
        bail!(crate::command::Refusal(format!(
            "change '{change}' has {}/{} code tasks complete — {} stamp \
             requires every code task done ([M] manual tasks excluded)",
            counts.code_complete, counts.code_total, st.noun
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
    // 錨記全任務總數（含 `[M]`）——欄位形狀不變，舊章資料相容。
    Ok(StampGate { raw_meta, paths, tasks_total: counts.total })
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
    // 文字手術（直接用 `KeyLines`，不走 `edit_meta`：meta 原文已由 `stamp_gate`
    // 讀過並驗過，且「先刪工單再寫章」要求寫入前插一步刪除）：先移除既有站別
    // 五鍵（重蓋不留重複鍵、章永遠落在檔尾），再寫新章；其餘欄位逐位元組保留。
    let prefix = st.meta_prefix;
    let stamp_keys =
        ["at", "by", "with", "tasks_total", "scope"].map(|suffix| format!("{prefix}_{suffix}"));
    let [at_key, by_key, with_key, tasks_total_key, scope_key] = &stamp_keys;
    let mut m = KeyLines::whole(&gate.raw_meta);
    for key in &stamp_keys {
        m.remove(key);
    }
    // 身分／工具／指紋一律過 YAML 純量守門（沿 started_* 的同一道）：帶 `:`、
    // `#` 或換行的字串會注入欄位或炸掉整份 meta——而工單已在同一步刪除，
    // 無從回復。
    m.set(at_key, &crate::util::today())?;
    if let Some(actor) = actor {
        m.set(by_key, &crate::util::yaml_scalar(actor))?;
    }
    if let Some(tool) = tool {
        m.set(with_key, &crate::util::yaml_scalar(tool))?;
    }
    m.set(tasks_total_key, &gate.tasks_total.to_string())?;
    let body: Vec<String> = entries
        .iter()
        .flat_map(|(path, hash)| {
            [
                format!("  - path: {}", crate::util::yaml_scalar(path)),
                format!("    hash: {}", crate::util::yaml_scalar(hash)),
            ]
        })
        .collect();
    m.set_block(scope_key, &body)?;

    // 同一原子寫入（design D3）：remote 走 bridge 的 staged commit 天然原子；
    // 本地順序為先刪工單再寫章——中斷時寧可退回「未檢查」，也不得出現
    // 「章已寫而工單仍在」的半套狀態（spec 明文禁止的唯一中間態）。
    store.delete_artifact(change, st.doc)?;
    store.write_change_meta(change, &m.text())?;
    Ok(())
}

/// 文字指紋規則：CRLF→LF 正規化後的 SHA-256（spec「內容指紋錨與失效判定」）。
/// 這是 [`content_fingerprint_bytes`] 分流後的文字分支——新呼叫端一律走那個
/// 唯一入口，不直接呼叫這裡（原名原簽名保留是 design D1：既有測試與消費端
/// 都在用）。兩站共用同一實作——指紋規則位元級同構是 design D2 的硬性要求。
pub fn content_fingerprint(text: &str) -> String {
    hex_sha256(text.replace("\r\n", "\n").as_bytes())
}

/// 指紋計算的唯一入口——呼叫端一律用這個，UTF-8 判定不在各呼叫端重複
/// （spec「指紋分流表」）：可 UTF-8 解讀 → 轉呼文字規則（既有已落的章逐位元組
/// 相容）；不可解讀 → 原始位元組 SHA-256（不做行尾正規化——binary 沒有
/// 「行」，正規化會讓不同位元組算出同一指紋）。
pub fn content_fingerprint_bytes(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(text) => content_fingerprint(text),
        Err(_) => hex_sha256(bytes),
    }
}

/// SHA-256 十六進位——兩條指紋規則共用的輸出形。
fn hex_sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

/// 失效判定純函式（design D3、D4）：任務錨＝當前全任務總數仍等於蓋章時總數，
/// 且寫碼任務全完成——`[M]` 手測任務的勾選狀態不影響章（章＝驗證過，手測是
/// 封存的事）；內容錨＝全部 scope 檔的現值指紋相符（缺檔即不符）。任一錨破 →
/// Stale；全符 → Fresh；meta 未帶完整章 → Unknown。錨直接從 meta 依站別取
/// （[`ChangeMeta::anchors`]），呼叫端不再手工組 [`StampAnchors`]。
pub fn freshness(
    st: &Station,
    meta: &ChangeMeta,
    counts: &crate::tasks::Counts,
    read_file: &ScopeReadFn<'_>,
) -> Freshness {
    let anchors = meta.anchors(st);
    if !is_stamped(anchors) {
        return Freshness::Unknown;
    }
    match stale_reason(anchors, counts, Some(read_file)) {
        Some(_) => Freshness::Stale,
        None => Freshness::Fresh,
    }
}

/// meta 是否帶了可判定的完整章（缺席讀作未蓋章）。
pub fn is_stamped(anchors: StampAnchors<'_>) -> bool {
    anchors.tasks_total.is_some() && anchors.stamped_at.is_some() && !anchors.scope.is_empty()
}

/// [`freshness`] 的判定本體，附破錨原因。未蓋章（見 [`is_stamped`]）與兩錨皆符
/// 都回 None——呼叫端先以 `is_stamped` 分辨。`read_file` 為 None 時只判任務錨：
/// remote 封存通道無工作樹可讀，內容錨無從判定（design D5 的已知非對稱）。
pub fn stale_reason(
    anchors: StampAnchors<'_>,
    counts: &crate::tasks::Counts,
    read_file: ScopeReader<'_>,
) -> Option<StaleReason> {
    let stamped_total = anchors.tasks_total?;
    if counts.total != stamped_total || !counts.code_done() {
        return Some(StaleReason::TaskAnchor {
            stamped_total,
            total: counts.total,
            code_complete: counts.code_complete,
            code_total: counts.code_total,
        });
    }
    let read_file = read_file?;
    anchors.scope.iter().find_map(|entry| match read_file(&entry.path) {
        Some(content) if content_fingerprint_bytes(&content) == entry.hash => None,
        _ => Some(StaleReason::ContentAnchor { path: entry.path.clone() }),
    })
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
    //! 與五欄寫入由底下的 `review`／`verify` 子模組覆蓋。
    use super::*;
    use crate::teststore::TestStore;

    const META: &str = "schema: spec-driven\ncreated: 2026-07-01\n";
    const REPO: &[(&str, &str)] = &[("crates/a/src/lib.rs", "fn a() {}\n")];

    fn store_with_round(round: &str) -> TestStore {
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", "- [x] 1 a\n");
        add_round(&VERIFY, &store, "demo", round).expect("round recorded");
        store
    }

    fn stamp_demo(store: &TestStore, accept: bool) -> Result<()> {
        let files =
            |p: &str| REPO.iter().find(|(k, _)| *k == p).map(|(_, v)| v.as_bytes().to_vec());
        let present = |p: &str| REPO.iter().any(|(k, _)| *k == p);
        stamp(&VERIFY, store, "demo", accept, None, None, &files, &present)
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

    #[test]
    fn freshness_takes_meta_and_reads_only_its_own_station() {
        // lifecycle-station-anchors D2：freshness 直接吃 meta，錨由 `meta.anchors(st)`
        // 取——無章 Unknown、錨全符 Fresh、任務錨破 Stale；另一站的章不影響本站判定。
        let h = content_fingerprint("fn a() {}\n");
        let counts = |md: &str| crate::tasks::counts(&crate::tasks::parse(md));
        let five_done = "- [x] a\n- [x] b\n- [x] c\n- [x] d\n- [x] e\n";
        let four_of_five = "- [x] a\n- [x] b\n- [x] c\n- [x] d\n- [ ] e\n";
        let rf = |p: &str| REPO.iter().find(|(k, _)| *k == p).map(|(_, v)| v.as_bytes().to_vec());
        let unstamped = crate::model::ChangeMeta::from_text(Some(META)).expect("meta parses");
        assert_eq!(freshness(&REVIEW, &unstamped, &counts(five_done), &rf), Freshness::Unknown);
        let both = format!(
            "{META}reviewed_at: 2026-08-01\nreviewed_tasks_total: 5\nreviewed_scope:\n  - path: crates/a/src/lib.rs\n    hash: {h}\n\
             verified_at: 2026-08-01\nverified_tasks_total: 3\nverified_scope:\n  - path: crates/a/src/lib.rs\n    hash: {h}\n"
        );
        let meta = crate::model::ChangeMeta::from_text(Some(&both)).expect("meta parses");
        assert_eq!(
            freshness(&REVIEW, &meta, &counts(five_done), &rf),
            Freshness::Fresh,
            "review anchors match; the broken verify anchor must not leak in"
        );
        assert_eq!(
            freshness(&VERIFY, &meta, &counts(five_done), &rf),
            Freshness::Stale,
            "verify task anchor (3 vs 5) is broken"
        );
        assert_eq!(
            freshness(&REVIEW, &meta, &counts(four_of_five), &rf),
            Freshness::Stale,
            "review task anchor breaks on uncheck"
        );
    }

    /// 審查站：工單動詞、蓋章守門與五個 `reviewed_*` 欄位的寫入、scope 注入
    /// 蓋章、structured rounds、指紋與失效判定——全部帶 `&REVIEW` 走本模組的函式。
    mod review {
        use super::super::*;
        use crate::teststore::TestStore;

        const META: &str = "schema: spec-driven\ncreated: 2026-07-01\n";

        const ROUND_1: &str = "**Scope**: crates/a/src/lib.rs, crates/b/src/util.rs\n\n- [CRITICAL] crates/a/src/lib.rs — unwrap on user input\n- [SUGGESTION] crates/b/src/util.rs — rename helper\n";
        const ROUND_2: &str = "**Scope**: crates/a/src/lib.rs\n\n- [WARNING] crates/a/src/lib.rs — possible Feature Envy\n";
        const SUGGESTION_ROUND: &str =
            "**Scope**: crates/b/src/util.rs\n\n- [SUGGESTION] crates/b/src/util.rs — rename helper\n";

        fn store_with_change() -> TestStore {
            TestStore::with_meta("demo", META)
        }

        /// total 個寫碼任務、前 done 個已勾（無 `[M]`）——失效判定的計數輸入。
        fn code_counts(total: usize, done: usize) -> crate::tasks::Counts {
            let md: String =
                (0..total).map(|i| if i < done { "- [x] t\n" } else { "- [ ] t\n" }).collect();
            crate::tasks::counts(&crate::tasks::parse(&md))
        }

        // --- spec「審查工單的建立與追加」---

        #[test]
        fn add_round_creates_ticket_with_round_1_on_first_call() {
            // spec Scenario「首輪建立工單」：無工單＋合法內容 → 建檔且自 Round 1 起算。
            let store = store_with_change();
            let round = add_round(&REVIEW, &store, "demo", ROUND_1).expect("first round");
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
            add_round(&REVIEW, &store, "demo", ROUND_1).expect("first round");
            let after_first = store.read_artifact("demo", REVIEW_DOC).expect("ticket");
            let round = add_round(&REVIEW, &store, "demo", ROUND_2).expect("second round");
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
            let err = add_round(&REVIEW, &store, "ghost", ROUND_1).expect_err("missing change must be rejected");
            assert!(err.to_string().contains("ghost"), "error must name the change: {err}");
            // 沿 in-progress add／set_board_rank 的同款防護：非單一路徑段名稱拒絕。
            assert!(add_round(&REVIEW, &store, "../evil", ROUND_1).is_err());
            assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
        }

        #[test]
        fn add_round_rejects_content_without_scope_without_writing() {
            // spec Scenario「內容缺少 Scope」：缺 `**Scope**:`（或清單為空）→ 拒絕、工單不變。
            let store = store_with_change();
            for bad in ["", "   \n", "- [CRITICAL] a.rs — no scope line\n", "**Scope**:  \n"] {
                let res = add_round(&REVIEW, &store, "demo", bad);
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
                assert!(add_round(&REVIEW, &store, "demo", bad).is_err(), "content {bad:?} must be rejected");
            }
            assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
        }

        // --- spec「審查工單的讀取」---

        #[test]
        fn show_round_trips_rounds_scope_findings_and_last_round() {
            // spec Scenario「讀取既有工單的 JSON」的解析核心：rounds 長度、每輪 index、
            // scope 清單與分級 findings 逐欄位；lastRound 指向末輪。
            let store = store_with_change();
            add_round(&REVIEW, &store, "demo", ROUND_1).expect("round 1");
            add_round(&REVIEW, &store, "demo", ROUND_2).expect("round 2");
            let ticket = show(&REVIEW, &store, "demo").expect("ticket parses");
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
            let err = show(&REVIEW, &store, "demo").expect_err("no ticket must error");
            assert!(err.to_string().contains("no review ticket"), "error must say so: {err}");
        }

        // --- spec「放棄審查」---

        #[test]
        fn discard_deletes_ticket_and_leaves_meta_untouched() {
            // spec Scenario「放棄既有工單」：工單刪除、`.openspec.yaml` 位元級不變。
            let store = store_with_change();
            add_round(&REVIEW, &store, "demo", ROUND_1).expect("round 1");
            discard(&REVIEW, &store, "demo").expect("discard");
            assert!(!store.artifact_exists("demo", REVIEW_DOC), "ticket must be gone");
            assert_eq!(store.meta("demo"), META, "metadata must stay byte-identical");
            assert_eq!(*store.meta_writes.borrow(), 0, "discard must not write metadata");
        }

        #[test]
        fn discard_errors_when_no_ticket() {
            // spec Scenario「無工單可放棄」。
            let store = store_with_change();
            let err = discard(&REVIEW, &store, "demo").expect_err("no ticket must error");
            assert!(err.to_string().contains("no review ticket"), "error must say so: {err}");
        }

        // --- spec「蓋章守門與蓋章效果」---

        const TASKS_5_DONE: &str = "- [x] 1 a\n- [x] 2 b\n- [x] 3 c\n- [x] 4 d\n- [x] 5 e\n";
        const TASKS_4_OF_5: &str = "- [x] 1 a\n- [x] 2 b\n- [x] 3 c\n- [x] 4 d\n- [ ] 5 e\n";
        /// 四個寫碼任務全勾、第五個是未勾的 `[M]` 手測——寫碼任務全完成預測子成立。
        const TASKS_CODE_DONE_MANUAL_OPEN: &str =
            "- [x] 1 a\n- [x] 2 b\n- [x] 3 c\n- [x] 4 d\n- [ ] [M] 5 手測\n";
        const CLEAN_ROUND: &str = "**Scope**: crates/a/src/lib.rs\n";

        const FILE_A: &str = "fn a() {}\n";
        const FILE_B: &str = "fn b() {}\n";

        /// repo 檔案讀取替身：固定 (path, content) 表（讀檔閉包吃位元組）。
        fn files<'a>(map: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<Vec<u8>> + 'a {
            move |p: &str| map.iter().find(|(k, _)| *k == p).map(|(_, v)| v.as_bytes().to_vec())
        }

        /// repo 檔案存在替身：與 `files` 共用同一張表。
        fn present<'a>(map: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> bool + 'a {
            move |p: &str| map.iter().any(|(k, _)| *k == p)
        }

        /// 非 UTF-8 位元組替身（PNG 式檔頭；0x89 開頭即非法 UTF-8）。
        const BIN_A: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0xFF, 0x00];

        /// repo 位元組讀取替身：binary scope 案例用（文字案例走 `files`）。
        fn byte_files<'a>(map: &'a [(&'a str, &'a [u8])]) -> impl Fn(&str) -> Option<Vec<u8>> + 'a {
            move |p: &str| map.iter().find(|(k, _)| *k == p).map(|(_, v)| v.to_vec())
        }

        /// repo 檔案存在替身：與 `byte_files` 共用同一張表。
        fn byte_present<'a>(map: &'a [(&'a str, &'a [u8])]) -> impl Fn(&str) -> bool + 'a {
            move |p: &str| map.iter().any(|(k, _)| *k == p)
        }

        const REPO: &[(&str, &str)] =
            &[("crates/a/src/lib.rs", FILE_A), ("crates/b/src/util.rs", FILE_B)];

        fn stamp_demo(store: &TestStore, accept: bool) -> Result<()> {
            stamp(
                &REVIEW,
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
            add_round(&REVIEW, &store, "demo", CLEAN_ROUND).expect("round 1");
            let err = stamp_demo(&store, false).expect_err("incomplete tasks must refuse");
            assert!(err.to_string().contains("4/5"), "error must show the count: {err}");
            assert_eq!(store.meta("demo"), META, "metadata must stay byte-identical");
            assert_eq!(*store.meta_writes.borrow(), 0);
            assert!(store.artifact_exists("demo", REVIEW_DOC), "ticket must survive refusal");
        }

        #[test]
        fn stamp_lands_when_only_manual_tasks_remain_and_anchors_the_full_total() {
            // spec Scenario「僅餘手動任務可蓋章」＋Example「蓋章寫入的任務錨」：
            // 寫碼 4/4 全勾、一個 [M] 未勾 → 蓋章成功，錨為全任務總數 5（含 [M]）。
            let store = store_with_change();
            store.put_artifact("demo", "tasks.md", TASKS_CODE_DONE_MANUAL_OPEN);
            add_round(&REVIEW, &store, "demo", CLEAN_ROUND).expect("round 1");
            stamp_demo(&store, false).expect("manual-only remainder must stamp");
            let meta = crate::model::ChangeMeta::from_text(Some(&store.meta("demo"))).expect("meta");
            assert_eq!(meta.reviewed_tasks_total, Some(5), "anchor counts the [M] task too");
            assert!(!store.artifact_exists("demo", REVIEW_DOC), "ticket must be deleted");
        }

        #[test]
        fn stamp_refusal_names_the_code_task_counts() {
            // 拒絕訊息點名寫碼任務——手測任務不計入計數。
            let store = store_with_change();
            store.put_artifact("demo", "tasks.md", "- [x] a\n- [ ] b\n- [ ] [M] 手測\n");
            add_round(&REVIEW, &store, "demo", CLEAN_ROUND).expect("round 1");
            let err = stamp_demo(&store, false).expect_err("open code task must refuse");
            let msg = err.to_string();
            assert!(msg.contains("1/2"), "counts must exclude the [M] task: {msg}");
            assert!(msg.contains("code task"), "message must name code tasks: {msg}");
        }

        #[test]
        fn stamp_refuses_unresolved_findings_without_accept() {
            // spec Scenario「末輪有未解 findings 且未帶 --accept」：拒絕並提示 --accept。
            let store = store_with_change();
            store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
            add_round(&REVIEW, &store, "demo", ROUND_1).expect("round with findings");
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
            add_round(&REVIEW, &store, "demo", ROUND_1).expect("round with findings");
            stamp_demo(&store, true).expect("--accept must stamp");
            assert!(!store.artifact_exists("demo", REVIEW_DOC), "ticket must be deleted");
            let meta = ChangeMeta::from_text(Some(&store.meta("demo"))).expect("meta parses");
            assert!(meta.reviewed_at.is_some());
        }

        #[test]
        fn stamp_allows_a_suggestion_only_last_round() {
            // spec Scenario「僅 SUGGESTION 的末輪乾淨蓋章」：SUGGESTION 不是必修，
            // 無 --accept 也放行——五欄寫入且工單刪除。分界計數本身歸
            // station.rs 的守門測試；這裡釘站別 wiring 與蓋章效果。
            let store = store_with_change();
            store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
            add_round(&REVIEW, &store, "demo", SUGGESTION_ROUND).expect("suggestion-only round");
            stamp_demo(&store, false).expect("suggestion-only round must stamp clean");
            assert!(!store.artifact_exists("demo", REVIEW_DOC), "ticket must be deleted");
            let meta = ChangeMeta::from_text(Some(&store.meta("demo"))).expect("meta parses");
            assert_eq!(meta.reviewed_at.as_deref(), Some(crate::util::today().as_str()));
            assert_eq!(meta.reviewed_by.as_deref(), Some("Rev <r@example.com>"));
            assert_eq!(meta.reviewed_with.as_deref(), Some("claude"));
            assert_eq!(meta.reviewed_tasks_total, Some(5));
            assert!(!meta.reviewed_scope.is_empty(), "scope fingerprints must be recorded");
        }

        #[test]
        fn stamp_clean_round_writes_five_fields_and_deletes_ticket() {
            // spec Scenario「乾淨蓋章」＋Example「蓋章寫入的任務錨」：5/5 任務、末輪
            // 零 findings → 五欄位齊備（reviewed_tasks_total 為 5）、工單不存在。
            let store = store_with_change();
            store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
            add_round(&REVIEW, &store, "demo", ROUND_1).expect("round 1 with findings");
            add_round(&REVIEW, &store, "demo", CLEAN_ROUND).expect("round 2 clean");
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
            add_round(&REVIEW, &store, "demo", ROUND_1).expect("round 1: a + b");
            add_round(&REVIEW, &store, "demo", CLEAN_ROUND).expect("round 2: a only");
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
            add_round(&REVIEW, &store, "demo", "**Scope**: crates\\a\\src\\lib.rs\n").expect("round");
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
            add_round(&REVIEW, &store, "demo", "**Scope**: gone/missing.rs\n").expect("round");
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
            add_round(&REVIEW, &store, "demo", ROUND_1).expect("round 1: a + b");
            add_round(&REVIEW, &store, "demo", CLEAN_ROUND).expect("round 2 clean");
            let survivors: &[(&str, &str)] = &[("crates/a/src/lib.rs", FILE_A)];
            stamp(&REVIEW, &store, "demo", false, None, None, &files(survivors), &present(survivors))
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
            add_round(&REVIEW, &store, "demo", CLEAN_ROUND).expect("round");
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
            add_round(&REVIEW, &store, "demo", CLEAN_ROUND).expect("round");
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
            add_round(&REVIEW, &store, "demo", ROUND_1).expect("round with findings");
            add_round(&REVIEW, &store, "demo", CLEAN_ROUND).expect("clean round");
            let entries = vec![
                scope_entry("crates/b/src/util.rs", &content_fingerprint(FILE_B)),
                scope_entry("crates/a/src/lib.rs", &content_fingerprint(FILE_A)),
            ];
            stamp_with_scope(&REVIEW, &store, "demo", false, Some("Rev <r@example.com>"), Some("claude"), entries, vec![])
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
            add_round(&REVIEW, &store, "demo", ROUND_1).expect("round"); // 聯集：a + b
            let missing = vec![scope_entry("crates/a/src/lib.rs", &content_fingerprint(FILE_A))];
            let err = stamp_with_scope(&REVIEW, &store, "demo", true, None, None, missing, vec![])
                .expect_err("missing path must refuse");
            assert!(err.to_string().contains("crates/b/src/util.rs"), "must name the gap: {err}");
            let extra = vec![
                scope_entry("crates/a/src/lib.rs", &content_fingerprint(FILE_A)),
                scope_entry("crates/b/src/util.rs", &content_fingerprint(FILE_B)),
                scope_entry("crates/c/extra.rs", &content_fingerprint("x")),
            ];
            let err = stamp_with_scope(&REVIEW, &store, "demo", true, None, None, extra, vec![])
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
            add_round(&REVIEW, &store, "demo", CLEAN_ROUND).expect("round"); // 聯集：a
            let dupes = vec![
                scope_entry("crates/a/src/lib.rs", &content_fingerprint(FILE_A)),
                scope_entry("crates/a/src/lib.rs", &content_fingerprint("tampered")),
            ];
            let err = stamp_with_scope(&REVIEW, &store, "demo", true, None, None, dupes, vec![])
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
            add_round(&REVIEW, &store, "demo", ROUND_1).expect("round: a + b");
            add_round(&REVIEW, &store, "demo", CLEAN_ROUND).expect("clean round");
            let entries = vec![scope_entry("crates/a/src/lib.rs", &content_fingerprint(FILE_A))];
            stamp_with_scope(
                &REVIEW,
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
            add_round(&REVIEW, &store, "demo", CLEAN_ROUND).expect("round"); // 聯集：a
            let a_entry = || vec![scope_entry("crates/a/src/lib.rs", &content_fingerprint(FILE_A))];
            let err = stamp_with_scope(
                &REVIEW,
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
                stamp_with_scope(&REVIEW, &store, "demo", true, None, None, a_entry(), vec!["not/in/union.rs".into()])
                    .expect_err("outside-union declaration must refuse");
            assert!(err.to_string().contains("not/in/union.rs"), "names the stray: {err}");
            let err =
                stamp_with_scope(&REVIEW, &store, "demo", true, None, None, vec![], vec!["crates/a/src/lib.rs".into()])
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
            add_round(&REVIEW, &store, "demo", "**Scope**: src/@odd #1.rs\n").expect("round");
            let repo: &[(&str, &str)] = &[("src/@odd #1.rs", FILE_A)];
            stamp(&REVIEW, &store, "demo", false, None, None, &files(repo), &present(repo)).expect("stamp");
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
            add_round(&REVIEW, &store, "demo", CLEAN_ROUND).expect("round");
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
            // read_file 回 None 是 I/O 失敗（如 EACCES；非 UTF-8 不是讀取失敗，走位
            // 元組指紋）——說成「不存在」會把人送去找一個明明還在的檔。
            let store = store_with_change();
            store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
            add_round(&REVIEW, &store, "demo", "**Scope**: assets/logo.png\n").expect("round");
            let err = stamp(&REVIEW, &store, "demo", false, None, None, &files(&[]), &|_: &str| true)
                .expect_err("must refuse");
            let msg = err.to_string();
            assert!(msg.contains("assets/logo.png"), "names the file: {msg}");
            assert!(
                !msg.contains("does not exist"),
                "must not assert absence it cannot know: {msg}"
            );
        }

        #[test]
        fn stamp_records_byte_hash_for_non_utf8_scope_and_freshness_tracks_bytes() {
            // spec Scenario「非 UTF-8 scope 檔可蓋章」＋「蓋章後 binary 內容變動失效」：
            // reviewed_scope 記原始位元組 SHA-256；失效判定共用同一分流。
            let store = store_with_change();
            store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
            add_round(&REVIEW, &store, "demo", "**Scope**: assets/logo.png\n").expect("round");
            let repo: &[(&str, &[u8])] = &[("assets/logo.png", BIN_A)];
            stamp(&REVIEW, &store, "demo", false, None, None, &byte_files(repo), &byte_present(repo))
                .expect("a present non-UTF-8 scope file must stamp");
            let meta = ChangeMeta::from_text(Some(&store.meta("demo"))).expect("meta parses");
            // 位元組規則本身由 content_fingerprint_bytes 的 golden 測試釘住；這裡釘
            // 站別 wiring：章欄位記的就是分流入口算出的值。
            assert_eq!(meta.reviewed_scope[0].hash, content_fingerprint_bytes(BIN_A));
            let counts = code_counts(5, 5);
            assert_eq!(freshness(&REVIEW, &meta, &counts, &byte_files(repo)), Freshness::Fresh);
            const BIN_A_FLIPPED: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0xFF, 0x01];
            let changed: &[(&str, &[u8])] = &[("assets/logo.png", BIN_A_FLIPPED)];
            assert_eq!(freshness(&REVIEW, &meta, &counts, &byte_files(changed)), Freshness::Stale);
        }

        #[test]
        fn stamp_survives_identity_and_agent_strings_carrying_yaml_indicators() {
            // `--agent "codex: cli"`、含 `#` 的 git user.name、或帶換行的身分字串以
            // 純量直寫會注入欄位或整份炸掉——而工單已在同一步刪除，無從回復。
            // 沿 started_* 的既有 clean() 紀律：控制字元壓平、危險純量加引號。
            let store = store_with_change();
            store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
            add_round(&REVIEW, &store, "demo", CLEAN_ROUND).expect("round");
            stamp(
                &REVIEW,
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
            add_round(&REVIEW, &store, "demo", CLEAN_ROUND).expect("round");
            let entries = vec![scope_entry("crates/a/src/lib.rs", "dead\nbeef: injected")];
            stamp_with_scope(&REVIEW, &store, "demo", false, None, None, entries, vec![]).expect("stamp");
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
            add_round(&REVIEW, &store, "demo", CLEAN_ROUND).expect("round");
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
                let Err(err) = add_round(&REVIEW, &store, "demo", bad) else {
                    panic!("scope {bad:?} must be rejected");
                };
                assert!(
                    err.to_string().contains("repo-root relative"),
                    "error must explain the requirement: {err}"
                );
            }
            assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
        }

        // --- spec「審查工單的建立與追加」：structured rounds（Phase／Patch 成對）---

        fn structured_round(phase: &str, hex: &str) -> String {
            format!(
                "**Phase**: {phase}\n**Patch**: sha256:{hex}\n**Scope**: src/lib.rs\n\n- [CRITICAL] src/lib.rs — unwrap on user input\n"
            )
        }

        #[test]
        fn structured_discovery_round_parses_phase_and_patch_hash() {
            // spec Scenario「首輪建立 structured discovery 工單」的解析核心。
            let store = store_with_change();
            let hex = "a".repeat(64);
            add_round(&REVIEW, &store, "demo", &structured_round("discovery", &hex)).expect("structured round");
            let ticket = show(&REVIEW, &store, "demo").expect("ticket parses");
            let round = &ticket.rounds[0];
            assert_eq!(round.phase, Some(RoundPhase::Discovery));
            assert_eq!(round.patch_hash.as_deref(), Some(format!("sha256:{hex}").as_str()));
            // 原文帶 phase／patch 行（人眼工單）。
            let doc = store.read_artifact("demo", REVIEW_DOC).expect("ticket doc");
            assert!(doc.contains("**Phase**: discovery"), "{doc}");
            assert!(doc.contains(&format!("**Patch**: sha256:{hex}")), "{doc}");
        }

        #[test]
        fn structured_round_rejects_unpaired_or_malformed_fields_without_writing() {
            // spec Scenario「phase 與 patch 必須成對」＋格式驗證：兩欄只出現其一、
            // phase token 無效、hash 非 64 lowercase hex → 非零拒絕且工單零寫入。
            let store = store_with_change();
            let hex = "a".repeat(64);
            for bad in [
                format!("**Phase**: discovery\n**Scope**: src/lib.rs\n"),
                format!("**Patch**: sha256:{hex}\n**Scope**: src/lib.rs\n"),
                structured_round("exploration", &hex),
                structured_round("discovery", "zz"),
                structured_round("discovery", &"A".repeat(64)),
                format!("**Phase**: discovery\n**Patch**: {hex}\n**Scope**: src/lib.rs\n"),
            ] {
                assert!(add_round(&REVIEW, &store, "demo", &bad).is_err(), "content {bad:?} must be rejected");
            }
            assert_eq!(*store.artifact_writes.borrow(), 0, "refusals must not write");
            assert!(!store.artifact_exists("demo", REVIEW_DOC));
        }

        #[test]
        fn second_discovery_is_rejected_after_a_structured_round() {
            // spec Scenario「第二個 discovery 被拒絕」：後續輪只能是 validation，
            // 工單位元級不變。
            let store = store_with_change();
            let hex = "a".repeat(64);
            add_round(&REVIEW, &store, "demo", &structured_round("discovery", &hex)).expect("round 1");
            let before = store.read_artifact("demo", REVIEW_DOC).expect("ticket");
            let err = add_round(&REVIEW, &store, "demo", &structured_round("discovery", &"b".repeat(64)))
                .expect_err("a second discovery must be rejected");
            assert!(err.to_string().contains("validation"), "error explains the sequence: {err}");
            assert_eq!(store.read_artifact("demo", REVIEW_DOC).as_deref(), Some(before.as_str()));
        }

        #[test]
        fn validation_appends_after_structured_and_legacy_tickets() {
            // spec Scenario「追加 validation 不改寫既有輪」＋「legacy ticket 後 SHALL
            // 能追加 validation round」。
            let store = store_with_change();
            add_round(&REVIEW, &store, "demo", &structured_round("discovery", &"a".repeat(64))).expect("r1");
            let before = store.read_artifact("demo", REVIEW_DOC).expect("ticket");
            let round =
                add_round(&REVIEW, &store, "demo", &structured_round("validation", &"b".repeat(64))).expect("r2");
            assert_eq!(round, 2);
            let after = store.read_artifact("demo", REVIEW_DOC).expect("ticket");
            assert!(after.starts_with(&before), "append-only across structured rounds");
            let ticket = show(&REVIEW, &store, "demo").expect("parses");
            assert_eq!(ticket.last_round().phase, Some(RoundPhase::Validation));

            let legacy_store = store_with_change();
            add_round(&REVIEW, &legacy_store, "demo", ROUND_1).expect("legacy r1");
            add_round(&REVIEW, &legacy_store, "demo", &structured_round("validation", &"c".repeat(64)))
                .expect("validation may follow a legacy ticket");
        }

        #[test]
        fn validation_is_rejected_on_a_fresh_ticket() {
            // 無任何輪次可驗收：首輪 validation 拒絕、零寫入。
            let store = store_with_change();
            let err = add_round(&REVIEW, &store, "demo", &structured_round("validation", &"a".repeat(64)))
                .expect_err("fresh-ticket validation must be rejected");
            assert!(err.to_string().contains("discovery"), "error names the required phase: {err}");
            assert_eq!(*store.artifact_writes.borrow(), 0);
        }

        #[test]
        fn legacy_round_parses_with_null_phase_and_patch() {
            // spec Scenario「legacy round 保持相容」：兩欄缺席解析為 None，既有行為不變。
            let store = store_with_change();
            add_round(&REVIEW, &store, "demo", ROUND_1).expect("legacy round");
            let ticket = show(&REVIEW, &store, "demo").expect("parses");
            assert_eq!(ticket.rounds[0].phase, None);
            assert_eq!(ticket.rounds[0].patch_hash, None);
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

        #[test]
        fn content_fingerprint_bytes_branches_on_utf8_readability() {
            // spec Example「指紋分流表」三列：UTF-8(LF)＝既有文字規則、UTF-8(CRLF)
            // 與同內容 LF 版雜湊相同、非 UTF-8 位元組＝原始位元組 SHA-256。
            assert_eq!(content_fingerprint_bytes(FILE_A.as_bytes()), content_fingerprint(FILE_A));
            assert_eq!(content_fingerprint_bytes(b"a\r\nb\r\n"), content_fingerprint("a\nb\n"));
            let png: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0xFF, 0x00];
            let raw_sha = {
                use sha2::{Digest, Sha256};
                format!("{:x}", Sha256::digest(png))
            };
            assert_eq!(content_fingerprint_bytes(png), raw_sha, "no line-ending normalization");
        }

        #[test]
        fn content_fingerprint_bytes_boundaries_bom_empty_and_type_flip() {
            // 分流邊界：BOM 與空位元組都是合法 UTF-8（走文字規則）；蓋章後檔案在
            // 文字↔binary 間轉換，指紋必變 → 失效判定 stale（兩個方向各釘一次）。
            assert_eq!(content_fingerprint_bytes(b"\xEF\xBB\xBFa\r\n"), content_fingerprint("\u{feff}a\n"));
            assert_eq!(content_fingerprint_bytes(b""), content_fingerprint(""));
            let path = "crates/a/src/lib.rs";
            let counts = code_counts(5, 5);
            let text_stamped = stamped_meta(5, &[(path, &content_fingerprint(FILE_A))]);
            let now_binary: &[(&str, &[u8])] = &[(path, BIN_A)];
            assert_eq!(
                freshness(&REVIEW, &text_stamped, &counts, &byte_files(now_binary)),
                Freshness::Stale,
                "text stamp, file turned binary"
            );
            let bin_stamped = stamped_meta(5, &[(path, &content_fingerprint_bytes(BIN_A))]);
            assert_eq!(
                freshness(&REVIEW, &bin_stamped, &counts, &files(&[(path, FILE_A)])),
                Freshness::Stale,
                "binary stamp, file turned text"
            );
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
            assert_eq!(freshness(&REVIEW, &meta, &code_counts(5, 5), &files(REPO)), Freshness::Fresh);
        }

        #[test]
        fn freshness_modified_scope_file_is_stale() {
            // spec Example「指紋比對」：檔案追加一行 → 現值雜湊不為 H1 → stale。
            let h1 = content_fingerprint(FILE_A);
            let meta = stamped_meta(5, &[("crates/a/src/lib.rs", h1.as_str())]);
            let grown = format!("{FILE_A}fn extra() {{}}\n");
            let now = [("crates/a/src/lib.rs", grown.as_str())];
            assert_eq!(freshness(&REVIEW, &meta, &code_counts(5, 5), &files(&now)), Freshness::Stale);
        }

        #[test]
        fn freshness_missing_scope_file_is_stale() {
            // spec：任一 scope 檔內容雜湊不符「含檔案已不存在」→ stale。
            let h = content_fingerprint(FILE_A);
            let meta = stamped_meta(5, &[("crates/a/src/lib.rs", h.as_str())]);
            assert_eq!(freshness(&REVIEW, &meta, &code_counts(5, 5), &files(&[])), Freshness::Stale);
        }

        #[test]
        fn freshness_line_ending_change_stays_fresh() {
            // spec Scenario「行尾差異不觸發失效」：LF → CRLF 仍 fresh。
            let h = content_fingerprint(FILE_A);
            let meta = stamped_meta(5, &[("crates/a/src/lib.rs", h.as_str())]);
            let crlf = FILE_A.replace('\n', "\r\n");
            let now = [("crates/a/src/lib.rs", crlf.as_str())];
            assert_eq!(freshness(&REVIEW, &meta, &code_counts(5, 5), &files(&now)), Freshness::Fresh);
        }

        #[test]
        fn freshness_task_anchor_breaks_on_recount_or_uncheck() {
            // spec：任務狀態不再是「蓋章當時任務總數的全完成」→ stale——新增任務
            //（總數變）與退勾（未全完成）皆觸發。
            let h = content_fingerprint(FILE_A);
            let meta = stamped_meta(5, &[("crates/a/src/lib.rs", h.as_str())]);
            assert_eq!(freshness(&REVIEW, &meta, &code_counts(6, 6), &files(REPO)), Freshness::Stale, "task count grew");
            assert_eq!(freshness(&REVIEW, &meta, &code_counts(5, 4), &files(REPO)), Freshness::Stale, "task unchecked");
        }

        #[test]
        fn freshness_ignores_manual_task_toggles() {
            // spec Scenario「蓋章後補勾手動任務不失效」：蓋章當時 5 個任務（4 寫碼全勾
            // ＋1 個未勾 [M]），事後補勾該 [M] → 兩種狀態都 fresh。
            let h = content_fingerprint(FILE_A);
            let meta = stamped_meta(5, &[("crates/a/src/lib.rs", h.as_str())]);
            let counts = |md: &str| crate::tasks::counts(&crate::tasks::parse(md));
            let open = "- [x] a\n- [x] b\n- [x] c\n- [x] d\n- [ ] [M] 手測\n";
            let checked = "- [x] a\n- [x] b\n- [x] c\n- [x] d\n- [x] [M] 手測\n";
            assert_eq!(freshness(&REVIEW, &meta, &counts(open), &files(REPO)), Freshness::Fresh, "手測未勾");
            assert_eq!(freshness(&REVIEW, &meta, &counts(checked), &files(REPO)), Freshness::Fresh, "手測補勾");
        }

        #[test]
        fn freshness_unstamped_meta_is_unknown() {
            // 缺席讀作未審查：無章的 meta 沒有可判定的錨。
            let meta = ChangeMeta::from_text(Some(META)).expect("meta parses");
            assert_eq!(freshness(&REVIEW, &meta, &code_counts(5, 5), &files(REPO)), Freshness::Unknown);
        }
    }

    /// 驗證站：同一組動詞帶 `&VERIFY`，外加 `add_round` 的任務全完成守門
    /// （刻意不對稱）、五個 `verified_*` 欄位，以及與審查站互不遮蔽的斷言。
    mod verify {
        use super::super::*;
        use crate::teststore::TestStore;

        const META: &str = "schema: spec-driven\ncreated: 2026-07-01\n";
        const TASKS_5_DONE: &str = "- [x] 1 a\n- [x] 2 b\n- [x] 3 c\n- [x] 4 d\n- [x] 5 e\n";
        const TASKS_4_OF_5: &str = "- [x] 1 a\n- [x] 2 b\n- [x] 3 c\n- [x] 4 d\n- [ ] 5 e\n";
        /// 四個寫碼任務全勾、第五個是未勾的 `[M]` 手測——寫碼任務全完成預測子成立。
        const TASKS_CODE_DONE_MANUAL_OPEN: &str =
            "- [x] 1 a\n- [x] 2 b\n- [x] 3 c\n- [x] 4 d\n- [ ] [M] 5 手測\n";

        const ROUND_1: &str = "**Scope**: crates/a/src/lib.rs, crates/b/src/util.rs\n\n- [CRITICAL] crates/a/src/lib.rs — requirement R2 has no implementation\n- [SUGGESTION] crates/b/src/util.rs — design says otherwise\n";
        const ROUND_2: &str = "**Scope**: crates/a/src/lib.rs\n\n- [WARNING] crates/a/src/lib.rs — scenario 3 untested\n";
        const SUGGESTION_ROUND: &str =
            "**Scope**: crates/b/src/util.rs\n\n- [SUGGESTION] crates/b/src/util.rs — design says otherwise\n";

        /// total 個寫碼任務、前 done 個已勾（無 `[M]`）——失效判定的計數輸入。
        fn code_counts(total: usize, done: usize) -> crate::tasks::Counts {
            let md: String =
                (0..total).map(|i| if i < done { "- [x] t\n" } else { "- [ ] t\n" }).collect();
            crate::tasks::counts(&crate::tasks::parse(&md))
        }

        /// 任務全數完成的 change——驗證工單的前提（design D3）。
        fn finished_change() -> TestStore {
            let store = TestStore::with_meta("demo", META);
            store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
            store
        }

        fn structured_round(phase: &str, hex: &str) -> String {
            format!(
                "**Phase**: {phase}\n**Patch**: sha256:{hex}\n**Scope**: src/lib.rs\n\n- [CRITICAL] src/lib.rs — requirement R2 has no implementation\n"
            )
        }

        // --- spec「驗證工單的建立與追加」---

        #[test]
        fn add_round_refuses_until_every_task_is_done() {
            // spec Scenario「任務未全完成即拒絕落工單」＋design D3：verify 檢查可中途
            // 跑，但盤點輪不得落工單——4/5 → 非零拒絕、零寫入、無檔案建立。
            let store = TestStore::with_meta("demo", META);
            store.put_artifact("demo", "tasks.md", TASKS_4_OF_5);
            let err = add_round(&VERIFY, &store, "demo", ROUND_1).expect_err("incomplete tasks must refuse");
            let msg = err.to_string();
            assert!(msg.contains("4/5"), "error must show the count: {msg}");
            assert!(
                err.downcast_ref::<crate::command::Refusal>().is_some(),
                "typed Refusal so the runtime classifies refused"
            );
            assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
            assert!(!store.artifact_exists("demo", VERIFY_DOC), "no ticket may appear");
        }

        #[test]
        fn add_round_lands_the_ticket_when_only_manual_tasks_remain() {
            // spec Scenario「僅餘手動任務可落工單」：寫碼 4/4 全勾、一個 [M] 未勾 → 放行。
            let store = TestStore::with_meta("demo", META);
            store.put_artifact("demo", "tasks.md", TASKS_CODE_DONE_MANUAL_OPEN);
            assert_eq!(add_round(&VERIFY, &store, "demo", ROUND_1).expect("manual-only remainder lands"), 1);
            assert!(store.artifact_exists("demo", VERIFY_DOC), "ticket must be created");
        }

        #[test]
        fn add_round_refusal_names_the_code_task_counts() {
            // spec「驗證工單的建立與追加」：拒絕訊息點名寫碼任務——手測任務不計入。
            let store = TestStore::with_meta("demo", META);
            store.put_artifact("demo", "tasks.md", "- [x] a\n- [ ] b\n- [ ] [M] 手測\n");
            let err = add_round(&VERIFY, &store, "demo", ROUND_1).expect_err("open code task must refuse");
            let msg = err.to_string();
            assert!(msg.contains("1/2"), "counts must exclude the [M] task: {msg}");
            assert!(msg.contains("code task"), "message must name code tasks: {msg}");
        }

        #[test]
        fn add_round_creates_ticket_with_round_1_on_first_call() {
            // spec Scenario「首輪建立工單」：任務全完成＋合法內容 → 建檔且自 Round 1 起算。
            let store = finished_change();
            let round = add_round(&VERIFY, &store, "demo", ROUND_1).expect("first round");
            assert_eq!(round, 1);
            let doc = store.read_artifact("demo", VERIFY_DOC).expect("ticket must be created");
            assert!(doc.starts_with("# Verify — demo\n"), "station title heads the ticket: {doc}");
            assert!(doc.contains("## Round 1"), "fixed skeleton must carry the round header: {doc}");
            assert!(
                doc.contains("**Scope**: crates/a/src/lib.rs"),
                "round content must be carried verbatim: {doc}"
            );
        }

        #[test]
        fn add_round_appends_round_2_keeping_round_1_byte_identical() {
            // spec Scenario「追加輪次不改寫既有輪」：append-only，Round 1 位元級不變。
            let store = finished_change();
            add_round(&VERIFY, &store, "demo", ROUND_1).expect("first round");
            let after_first = store.read_artifact("demo", VERIFY_DOC).expect("ticket");
            let round = add_round(&VERIFY, &store, "demo", ROUND_2).expect("second round");
            assert_eq!(round, 2);
            let after_second = store.read_artifact("demo", VERIFY_DOC).expect("ticket");
            assert!(
                after_second.starts_with(&after_first),
                "append-only: Round 1 must stay byte-identical\nbefore: {after_first}\nafter: {after_second}"
            );
            assert!(after_second.contains("## Round 2"));
        }

        #[test]
        fn add_round_rejects_content_without_scope_without_writing() {
            // spec Scenario「內容缺少 Scope」：缺 `**Scope**:`（或清單為空）→ 拒絕、零寫入。
            let store = finished_change();
            for bad in ["", "   \n", "- [CRITICAL] a.rs — no scope line\n", "**Scope**:  \n"] {
                let Err(err) = add_round(&VERIFY, &store, "demo", bad) else {
                    panic!("content {bad:?} must be rejected");
                };
                assert!(err.to_string().contains("**Scope**:"), "error must explain the format: {err}");
            }
            assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
            assert!(!store.artifact_exists("demo", VERIFY_DOC));
        }

        #[test]
        fn add_round_rejects_missing_change_and_unsafe_names() {
            // 沿 in-progress add／set_board_rank 的同款防護：不存在的 change 與
            // 非單一路徑段名稱皆拒絕、零寫入。
            let store = finished_change();
            let err = add_round(&VERIFY, &store, "ghost", ROUND_1).expect_err("missing change must be rejected");
            assert!(err.to_string().contains("ghost"), "error must name the change: {err}");
            assert!(add_round(&VERIFY, &store, "../evil", ROUND_1).is_err());
            assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
        }

        #[test]
        fn structured_round_rejects_unpaired_or_malformed_fields_without_writing() {
            // spec Scenario「phase 與 patch 必須成對」＋格式驗證：兩欄只出現其一、
            // phase token 無效、hash 非 64 lowercase hex → 非零拒絕且工單零寫入。
            let store = finished_change();
            let hex = "a".repeat(64);
            for bad in [
                "**Phase**: discovery\n**Scope**: src/lib.rs\n".to_string(),
                format!("**Patch**: sha256:{hex}\n**Scope**: src/lib.rs\n"),
                structured_round("exploration", &hex),
                structured_round("discovery", "zz"),
                structured_round("discovery", &"A".repeat(64)),
                format!("**Phase**: discovery\n**Patch**: {hex}\n**Scope**: src/lib.rs\n"),
            ] {
                assert!(add_round(&VERIFY, &store, "demo", &bad).is_err(), "content {bad:?} must be rejected");
            }
            assert_eq!(*store.artifact_writes.borrow(), 0, "refusals must not write");
            assert!(!store.artifact_exists("demo", VERIFY_DOC));
        }

        #[test]
        fn structured_discovery_round_parses_phase_and_patch_hash() {
            // spec Scenario「追加 structured validation」的前半：Round 1 是 discovery，
            // phase／patch 進工單原文並解析回值。
            let store = finished_change();
            let hex = "a".repeat(64);
            add_round(&VERIFY, &store, "demo", &structured_round("discovery", &hex)).expect("structured round");
            let ticket = show(&VERIFY, &store, "demo").expect("ticket parses");
            assert_eq!(ticket.rounds[0].phase, Some(RoundPhase::Discovery));
            assert_eq!(ticket.rounds[0].patch_hash.as_deref(), Some(format!("sha256:{hex}").as_str()));
            let doc = store.read_artifact("demo", VERIFY_DOC).expect("ticket doc");
            assert!(doc.contains("**Phase**: discovery"), "{doc}");
            assert!(doc.contains(&format!("**Patch**: sha256:{hex}")), "{doc}");
        }

        #[test]
        fn discovery_then_validation_is_the_only_allowed_sequence() {
            // spec Scenario「第二個 discovery 被拒絕」＋「追加 structured validation」：
            // structured Round 1 只能是 discovery，其後只能是 validation；第二個
            // discovery 拒絕且工單位元級不變。
            let store = finished_change();
            add_round(&VERIFY, &store, "demo", &structured_round("discovery", &"a".repeat(64))).expect("r1");
            let before = store.read_artifact("demo", VERIFY_DOC).expect("ticket");
            let err = add_round(&VERIFY, &store, "demo", &structured_round("discovery", &"b".repeat(64)))
                .expect_err("a second discovery must be rejected");
            assert!(err.to_string().contains("validation"), "error explains the sequence: {err}");
            assert_eq!(store.read_artifact("demo", VERIFY_DOC).as_deref(), Some(before.as_str()));

            let round = add_round(&VERIFY, &store, "demo", &structured_round("validation", &"c".repeat(64)))
                .expect("validation may follow discovery");
            assert_eq!(round, 2);
            let after = store.read_artifact("demo", VERIFY_DOC).expect("ticket");
            assert!(after.starts_with(&before), "append-only across structured rounds");
            assert_eq!(show(&VERIFY, &store, "demo").expect("parses").last_round().phase, Some(RoundPhase::Validation));
        }

        #[test]
        fn validation_is_rejected_on_a_fresh_ticket() {
            // 無任何輪次可驗收：首輪 validation 拒絕、零寫入。
            let store = finished_change();
            let err = add_round(&VERIFY, &store, "demo", &structured_round("validation", &"a".repeat(64)))
                .expect_err("fresh-ticket validation must be rejected");
            assert!(err.to_string().contains("discovery"), "error names the required phase: {err}");
            assert_eq!(*store.artifact_writes.borrow(), 0);
        }

        #[test]
        fn legacy_round_parses_with_null_phase_and_patch() {
            // spec Scenario「legacy round 保持相容」：兩欄缺席解析為 None，行為不變。
            let store = finished_change();
            add_round(&VERIFY, &store, "demo", ROUND_1).expect("legacy round");
            let ticket = show(&VERIFY, &store, "demo").expect("parses");
            assert_eq!(ticket.rounds[0].phase, None);
            assert_eq!(ticket.rounds[0].patch_hash, None);
        }

        #[test]
        fn add_round_rejects_scope_paths_that_escape_the_repo_root() {
            // Scope 是指紋讀檔的路徑來源；remote 模式的工單來自 server，等於由 server
            // 指定 client 讀哪個本機檔——守門落在文法層（兩站共用同一道）。
            let store = finished_change();
            for bad in [
                "**Scope**: /etc/passwd\n",
                "**Scope**: ../../../etc/passwd\n",
                "**Scope**: C:\\Windows\\win.ini\n",
            ] {
                let Err(err) = add_round(&VERIFY, &store, "demo", bad) else {
                    panic!("scope {bad:?} must be rejected");
                };
                assert!(
                    err.to_string().contains("repo-root relative"),
                    "error must explain the requirement: {err}"
                );
            }
            assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
        }

        // --- spec「驗證工單的讀取」---

        #[test]
        fn show_round_trips_rounds_scope_findings_and_last_round() {
            // spec Scenario「讀取 JSON」的解析核心：rounds 長度、每輪 index、scope 清單
            // 與分級 findings 逐欄位；lastRound 指向末輪。
            let store = finished_change();
            add_round(&VERIFY, &store, "demo", ROUND_1).expect("round 1");
            add_round(&VERIFY, &store, "demo", ROUND_2).expect("round 2");
            let ticket = show(&VERIFY, &store, "demo").expect("ticket parses");
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
                        text: "requirement R2 has no implementation".to_string(),
                    },
                    Finding {
                        severity: Severity::Suggestion,
                        path: "crates/b/src/util.rs".to_string(),
                        text: "design says otherwise".to_string(),
                    },
                ]
            );
            let last = ticket.last_round();
            assert_eq!(last.index, 2);
            assert_eq!(last.scope, vec!["crates/a/src/lib.rs".to_string()]);
            assert_eq!(last.findings.len(), 1);
            assert_eq!(last.findings[0].severity, Severity::Warning);
            assert_eq!(last.findings[0].text, "scenario 3 untested");
        }

        #[test]
        fn show_errors_when_change_has_no_ticket() {
            // spec Scenario「無工單」：錯誤說明該 change 無「驗證」工單——站別詞不得
            // 說成 review，否則使用者被送去看另一個站的工單。
            let store = finished_change();
            let err = show(&VERIFY, &store, "demo").expect_err("no ticket must error");
            assert!(err.to_string().contains("no verify ticket"), "error must say so: {err}");
            assert!(
                err.downcast_ref::<NotFound>().is_some(),
                "typed NotFound so the server maps 404"
            );
        }

        // --- spec「放棄驗證」---

        #[test]
        fn discard_deletes_the_verify_ticket_and_leaves_meta_untouched() {
            // spec Scenario「放棄既有工單」：工單刪除、`.openspec.yaml` 位元級不變。
            let store = finished_change();
            add_round(&VERIFY, &store, "demo", ROUND_1).expect("round 1");
            discard(&VERIFY, &store, "demo").expect("discard");
            assert!(!store.artifact_exists("demo", VERIFY_DOC), "ticket must be gone");
            assert_eq!(store.meta("demo"), META, "metadata must stay byte-identical");
            assert_eq!(*store.meta_writes.borrow(), 0, "discard must not write metadata");
        }

        #[test]
        fn discard_leaves_the_review_ticket_alone() {
            // 兩站互不遮蔽：放棄驗證不得動到審查工單（cleanup namespace 站別化的
            // 引擎面；snapshot 面由 CLI 測試覆蓋）。
            let store = finished_change();
            add_round(&VERIFY, &store, "demo", ROUND_1).expect("verify round");
            add_round(&REVIEW, &store, "demo", ROUND_1).expect("review round");
            let review_doc = store.read_artifact("demo", REVIEW_DOC).expect("ticket");
            discard(&VERIFY, &store, "demo").expect("discard");
            assert!(!store.artifact_exists("demo", VERIFY_DOC), "verify ticket must be gone");
            assert_eq!(
                store.read_artifact("demo", REVIEW_DOC).as_deref(),
                Some(review_doc.as_str()),
                "the review ticket must survive byte-identically"
            );
        }

        #[test]
        fn discard_errors_when_no_ticket() {
            // spec：無工單時非零 exit code。
            let store = finished_change();
            let err = discard(&VERIFY, &store, "demo").expect_err("no ticket must error");
            assert!(err.to_string().contains("no verify ticket"), "error must say so: {err}");
        }

        // --- spec「驗證蓋章守門與蓋章效果」---

        const CLEAN_ROUND: &str = "**Scope**: crates/a/src/lib.rs\n";
        const FILE_A: &str = "fn a() {}\n";
        const FILE_B: &str = "fn b() {}\n";
        const REPO: &[(&str, &str)] =
            &[("crates/a/src/lib.rs", FILE_A), ("crates/b/src/util.rs", FILE_B)];

        /// repo 檔案讀取替身：固定 (path, content) 表（讀檔閉包吃位元組）。
        fn files<'a>(map: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<Vec<u8>> + 'a {
            move |p: &str| map.iter().find(|(k, _)| *k == p).map(|(_, v)| v.as_bytes().to_vec())
        }

        /// repo 檔案存在替身：與 `files` 共用同一張表。
        fn present<'a>(map: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> bool + 'a {
            move |p: &str| map.iter().any(|(k, _)| *k == p)
        }

        /// 非 UTF-8 位元組替身（PNG 式檔頭；0x89 開頭即非法 UTF-8）。
        const BIN_A: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0xFF, 0x00];

        /// repo 位元組讀取替身：binary scope 案例用（文字案例走 `files`）。
        fn byte_files<'a>(map: &'a [(&'a str, &'a [u8])]) -> impl Fn(&str) -> Option<Vec<u8>> + 'a {
            move |p: &str| map.iter().find(|(k, _)| *k == p).map(|(_, v)| v.to_vec())
        }

        /// repo 檔案存在替身：與 `byte_files` 共用同一張表。
        fn byte_present<'a>(map: &'a [(&'a str, &'a [u8])]) -> impl Fn(&str) -> bool + 'a {
            move |p: &str| map.iter().any(|(k, _)| *k == p)
        }

        fn stamp_demo(store: &TestStore, accept: bool) -> Result<()> {
            stamp(
                &VERIFY,
                store,
                "demo",
                accept,
                Some("Ver <v@example.com>"),
                Some("claude"),
                &files(REPO),
                &present(REPO),
            )
        }

        #[test]
        fn stamp_refuses_unresolved_findings_without_accept() {
            // spec Scenario「末輪有未解 findings 且未帶 --accept」：拒絕並提示 --accept
            // 或先修正重驗；站別祈使詞為 re-verify（審查站是 re-review）。
            let store = finished_change();
            add_round(&VERIFY, &store, "demo", ROUND_1).expect("round with findings");
            let err = stamp_demo(&store, false).expect_err("unresolved findings must refuse");
            let msg = err.to_string();
            assert!(msg.contains("--accept"), "error must offer --accept: {msg}");
            assert!(msg.contains("re-verify"), "station word must be the verify one: {msg}");
            assert_eq!(*store.meta_writes.borrow(), 0);
            assert!(store.artifact_exists("demo", VERIFY_DOC), "ticket must survive refusal");
        }

        #[test]
        fn stamp_refuses_when_tasks_regress_after_the_ticket_was_opened() {
            // 守門 (1) 與審查站同一條：工單開立後任務被退勾 → 4/5 拒絕，訊息用
            // 「verify stamp」而非另一站的動詞。
            let store = finished_change();
            add_round(&VERIFY, &store, "demo", CLEAN_ROUND).expect("round 1");
            store.put_artifact("demo", "tasks.md", TASKS_4_OF_5);
            let err = stamp_demo(&store, false).expect_err("incomplete tasks must refuse");
            let msg = err.to_string();
            assert!(msg.contains("4/5"), "error must show the count: {msg}");
            assert!(msg.contains("verify stamp"), "error must name the verb: {msg}");
            assert_eq!(store.meta("demo"), META, "metadata must stay byte-identical");
            assert_eq!(*store.meta_writes.borrow(), 0);
            assert!(store.artifact_exists("demo", VERIFY_DOC), "ticket must survive refusal");
        }

        #[test]
        fn stamp_lands_when_only_manual_tasks_remain_and_anchors_the_full_total() {
            // spec Scenario「僅餘手動任務可蓋章」：寫碼全勾、[M] 未勾 → 蓋章成功，
            // verified_tasks_total 記全任務總數（含 [M]）。
            let store = finished_change();
            add_round(&VERIFY, &store, "demo", CLEAN_ROUND).expect("round 1");
            store.put_artifact("demo", "tasks.md", TASKS_CODE_DONE_MANUAL_OPEN);
            stamp_demo(&store, false).expect("manual-only remainder must stamp");
            let meta = crate::model::ChangeMeta::from_text(Some(&store.meta("demo"))).expect("meta");
            assert_eq!(meta.verified_tasks_total, Some(5), "anchor counts the [M] task too");
            assert!(!store.artifact_exists("demo", VERIFY_DOC), "ticket must be deleted");
        }

        #[test]
        fn stamp_clean_round_writes_five_fields_and_deletes_the_ticket() {
            // spec Scenario「乾淨蓋章」：五個 verified 欄位齊備、`verify.md` 不存在。
            let store = finished_change();
            add_round(&VERIFY, &store, "demo", ROUND_1).expect("round 1 with findings");
            add_round(&VERIFY, &store, "demo", CLEAN_ROUND).expect("round 2 clean");
            stamp_demo(&store, false).expect("clean stamp");
            assert!(!store.artifact_exists("demo", VERIFY_DOC), "ticket must be deleted");
            let raw = store.meta("demo");
            let meta = crate::model::ChangeMeta::from_text(Some(&raw)).expect("meta parses");
            assert_eq!(meta.verified_at.as_deref(), Some(crate::util::today().as_str()));
            assert_eq!(meta.verified_by.as_deref(), Some("Ver <v@example.com>"));
            assert_eq!(meta.verified_with.as_deref(), Some("claude"));
            assert_eq!(meta.verified_tasks_total, Some(5));
            assert!(!meta.verified_scope.is_empty(), "scope fingerprints must be recorded");
            assert!(raw.starts_with(META), "existing fields preserved byte-for-byte: {raw}");
        }

        #[test]
        fn stamp_writes_the_task_anchor_from_the_ticket_moment() {
            // spec Example「蓋章寫入的任務錨」：8 個任務、7 個寫碼任務全勾、1 個
            // `[M]` 未勾 → `verified_tasks_total` 為 8（未勾的 [M] 也計入錨）。
            let store = TestStore::with_meta("demo", META);
            let tasks: String = (1..=7).map(|i| format!("- [x] {i} t\n")).collect::<String>()
                + "- [ ] [M] 8 hand check\n";
            store.put_artifact("demo", "tasks.md", &tasks);
            add_round(&VERIFY, &store, "demo", CLEAN_ROUND).expect("clean round 1");
            stamp_demo(&store, false).expect("stamp");
            let meta = crate::model::ChangeMeta::from_text(Some(&store.meta("demo"))).expect("parses");
            assert_eq!(meta.verified_tasks_total, Some(8));
        }

        #[test]
        fn stamp_with_accept_overrides_findings_and_stamps() {
            // spec「`--accept` SHALL 僅豁免後者」：帶保留蓋章 → 章寫入且工單刪除。
            let store = finished_change();
            add_round(&VERIFY, &store, "demo", ROUND_1).expect("round with findings");
            stamp_demo(&store, true).expect("--accept must stamp");
            assert!(!store.artifact_exists("demo", VERIFY_DOC), "ticket must be deleted");
            let meta = crate::model::ChangeMeta::from_text(Some(&store.meta("demo"))).expect("parses");
            assert!(meta.verified_at.is_some());
        }

        #[test]
        fn stamp_allows_a_suggestion_only_last_round() {
            // spec Scenario「僅 SUGGESTION 的末輪乾淨蓋章」：SUGGESTION 不是必修，
            // 無 `--accept` 也放行——五欄寫入且工單刪除。分界計數本身歸
            // station.rs 的守門測試；這裡釘站別 wiring 與蓋章效果。
            let store = finished_change();
            add_round(&VERIFY, &store, "demo", SUGGESTION_ROUND).expect("suggestion-only round");
            stamp_demo(&store, false).expect("suggestion-only round must stamp clean");
            assert!(!store.artifact_exists("demo", VERIFY_DOC), "ticket must be deleted");
            let meta = crate::model::ChangeMeta::from_text(Some(&store.meta("demo"))).expect("parses");
            assert_eq!(meta.verified_at.as_deref(), Some(crate::util::today().as_str()));
            assert_eq!(meta.verified_by.as_deref(), Some("Ver <v@example.com>"));
            assert_eq!(meta.verified_with.as_deref(), Some("claude"));
            assert_eq!(meta.verified_tasks_total, Some(5));
            assert!(!meta.verified_scope.is_empty(), "scope fingerprints must be recorded");
        }

        #[test]
        fn stamp_leaves_the_review_station_untouched() {
            // 兩站互不遮蔽：驗證蓋章不得動到審查工單或 `reviewed_*` 欄位。
            const STAMPED: &str = "schema: spec-driven\nreviewed_at: 2026-08-01\nreviewed_by: Rev <r@example.com>\nreviewed_with: claude\nreviewed_tasks_total: 5\nreviewed_scope:\n  - path: crates/a/src/lib.rs\n    hash: dead\n";
            let store = TestStore::with_meta("demo", STAMPED);
            store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
            add_round(&REVIEW, &store, "demo", CLEAN_ROUND).expect("review round");
            add_round(&VERIFY, &store, "demo", CLEAN_ROUND).expect("verify round");
            stamp_demo(&store, false).expect("verify stamp");
            assert!(!store.artifact_exists("demo", VERIFY_DOC), "verify ticket must be gone");
            assert!(
                store.artifact_exists("demo", REVIEW_DOC),
                "the review ticket must survive a verify stamp"
            );
            let raw = store.meta("demo");
            let meta = crate::model::ChangeMeta::from_text(Some(&raw)).expect("meta parses");
            assert_eq!(meta.reviewed_at.as_deref(), Some("2026-08-01"), "review stamp intact: {raw}");
            assert_eq!(meta.reviewed_scope.len(), 1, "review anchors intact: {raw}");
            assert_eq!(meta.reviewed_scope[0].hash, "dead");
            assert!(meta.verified_at.is_some(), "verify stamp landed: {raw}");
        }

        #[test]
        fn stamp_scope_is_the_sorted_union_of_all_rounds() {
            // spec「驗證指紋錨與失效判定」：指紋範圍＝工單各輪 Scope 聯集（去重、排序）。
            let store = finished_change();
            add_round(&VERIFY, &store, "demo", ROUND_1).expect("round 1: a + b");
            add_round(&VERIFY, &store, "demo", CLEAN_ROUND).expect("round 2: a only");
            stamp_demo(&store, false).expect("stamp");
            let meta = crate::model::ChangeMeta::from_text(Some(&store.meta("demo"))).expect("parses");
            let paths: Vec<&str> = meta.verified_scope.iter().map(|e| e.path.as_str()).collect();
            assert_eq!(paths, vec!["crates/a/src/lib.rs", "crates/b/src/util.rs"]);
            assert_eq!(meta.verified_scope[0].hash, content_fingerprint(FILE_A));
            assert_eq!(meta.verified_scope[1].hash, content_fingerprint(FILE_B));
        }

        #[test]
        fn restamp_replaces_verified_fields_without_duplication() {
            // 重驗後重蓋：五欄位原位更新（含多行 verified_scope 區塊），不留重複鍵，
            // 其餘欄位（含審查章）逐位元組保留。
            let old = "schema: spec-driven\ncreated: 2026-07-01\nverified_at: 2026-07-10\nverified_by: Old <o@example.com>\nverified_with: codex\nverified_tasks_total: 3\nverified_scope:\n  - path: old/file.rs\n    hash: deadbeef\nboard_rank: n\n";
            let store = TestStore::with_meta("demo", old);
            store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
            add_round(&VERIFY, &store, "demo", CLEAN_ROUND).expect("round");
            stamp_demo(&store, false).expect("re-stamp");
            let raw = store.meta("demo");
            assert_eq!(raw.matches("verified_at:").count(), 1, "no duplicate keys: {raw}");
            assert_eq!(raw.matches("verified_scope:").count(), 1, "no duplicate keys: {raw}");
            assert!(!raw.contains("old/file.rs"), "stale scope block must be gone: {raw}");
            assert!(raw.contains("board_rank: n\n"), "unrelated fields survive: {raw}");
            let meta = crate::model::ChangeMeta::from_text(Some(&raw)).expect("re-stamped meta parses");
            assert_eq!(meta.verified_tasks_total, Some(5));
            assert_eq!(meta.verified_scope.len(), 1);
            assert_eq!(meta.verified_scope[0].path, "crates/a/src/lib.rs");
        }

        #[test]
        fn stamp_normalizes_backslash_paths_into_meta() {
            // spec「路徑正規化…與審查站位元級同構」：Windows 路徑 `\` → `/` 後入章，
            // 否則同一檔在兩個平台指紋成兩筆、章永遠 stale。
            let store = finished_change();
            add_round(&VERIFY, &store, "demo", "**Scope**: crates\\a\\src\\lib.rs\n").expect("round");
            stamp_demo(&store, false).expect("stamp");
            let meta = crate::model::ChangeMeta::from_text(Some(&store.meta("demo"))).expect("parses");
            assert_eq!(meta.verified_scope[0].path, "crates/a/src/lib.rs");
        }

        #[test]
        fn stamp_quotes_scalars_so_yaml_metacharacters_survive() {
            // 身分／工具／path 以未引號純量直寫會注入欄位或炸掉整份 meta——而工單已在
            // 同一步刪除，無從回復。寫入端負責引號（與審查站同一道 yaml_scalar）。
            let store = TestStore::with_meta("demo", META);
            store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
            add_round(&VERIFY, &store, "demo", "**Scope**: src/@odd #1.rs\n").expect("round");
            let repo: &[(&str, &str)] = &[("src/@odd #1.rs", FILE_A)];
            stamp(
                &VERIFY,
                &store,
                "demo",
                false,
                Some("Ver: the #1 <v@example.com>\nboard_rank: injected"),
                Some("codex: cli"),
                &files(repo),
                &present(repo),
            )
            .expect("stamp");
            let raw = store.meta("demo");
            let meta = crate::model::ChangeMeta::from_text(Some(&raw)).expect("meta must parse: {raw}");
            assert_eq!(meta.verified_scope[0].path, "src/@odd #1.rs", "path round-trips: {raw}");
            assert_eq!(meta.verified_with.as_deref(), Some("codex: cli"), "{raw}");
            assert!(!raw.contains("\nboard_rank: injected"), "no field injection: {raw}");
        }

        #[test]
        fn stamp_records_byte_hash_for_non_utf8_scope_and_freshness_tracks_bytes() {
            // spec Scenario「非 UTF-8 scope 檔可蓋章」：兩站算出的指紋完全一樣——共用
            // station 同一份實作（規格稱「位元級同構」）；位元組變動 → stale、未變 →
            // fresh。
            let store = finished_change();
            add_round(&VERIFY, &store, "demo", "**Scope**: assets/logo.png\n").expect("round");
            let repo: &[(&str, &[u8])] = &[("assets/logo.png", BIN_A)];
            stamp(&VERIFY, &store, "demo", false, None, None, &byte_files(repo), &byte_present(repo))
                .expect("a present non-UTF-8 scope file must stamp");
            let meta = crate::model::ChangeMeta::from_text(Some(&store.meta("demo"))).expect("parses");
            // 位元組規則本身由 review 側的 golden 測試釘住；這裡釘站別 wiring：
            // 章欄位記的就是分流入口算出的值。
            assert_eq!(meta.verified_scope[0].hash, content_fingerprint_bytes(BIN_A));
            let counts = code_counts(5, 5);
            assert_eq!(freshness(&VERIFY, &meta, &counts, &byte_files(repo)), Freshness::Fresh);
            const BIN_A_FLIPPED: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0xFF, 0x01];
            let changed: &[(&str, &[u8])] = &[("assets/logo.png", BIN_A_FLIPPED)];
            assert_eq!(freshness(&VERIFY, &meta, &counts, &byte_files(changed)), Freshness::Stale);
        }

        #[test]
        fn stamp_still_refuses_unreadable_and_all_gone_scope() {
            // 既有語意維持：讀不到就拒絕蓋章，這條沒有因為指紋分流而放寬。兩條
            // 拒章路由訊息不同，分別比對區別性措辭——只查路徑會讓路由接錯仍綠燈。
            let store = finished_change();
            add_round(&VERIFY, &store, "demo", CLEAN_ROUND).expect("round");
            let err = stamp(&VERIFY, &store, "demo", false, None, None, &|_: &str| None, &|_: &str| true)
                .expect_err("present but unreadable must refuse");
            let msg = err.to_string();
            assert!(msg.contains("crates/a/src/lib.rs"), "names the file: {msg}");
            assert!(msg.contains("must be present and readable"), "I/O route wording: {msg}");
            let err = stamp(&VERIFY, &store, "demo", false, None, None, &|_: &str| None, &|_: &str| false)
                .expect_err("all-gone union must refuse");
            let msg = err.to_string();
            assert!(msg.contains("crates/a/src/lib.rs"), "names the file: {msg}");
            assert!(msg.contains("gone from the work tree"), "all-gone route wording: {msg}");
            assert_eq!(*store.meta_writes.borrow(), 0);
        }

        #[test]
        fn stamp_refuses_on_corrupt_meta_and_without_a_ticket() {
            // 沿 set_board_rank 的 fail-closed gate：壞 metadata 不得被疊寫；無工單
            // 不可蓋章。
            let store = finished_change();
            let err = stamp_demo(&store, false).expect_err("no ticket must refuse");
            assert!(err.to_string().contains("no verify ticket"), "{err}");

            const BAD: &str = ": : :\n\t bad yaml [unclosed\n";
            let corrupt = TestStore::with_meta("demo", BAD);
            corrupt.put_artifact("demo", "tasks.md", TASKS_5_DONE);
            add_round(&VERIFY, &corrupt, "demo", CLEAN_ROUND).expect("round");
            let err = stamp_demo(&corrupt, false).expect_err("corrupt meta must refuse");
            assert!(
                err.to_string().contains("openspec/changes/demo/.openspec.yaml"),
                "error must name the metadata file: {err}"
            );
            assert_eq!(corrupt.meta("demo"), BAD);
            assert_eq!(*corrupt.meta_writes.borrow(), 0);
            assert!(corrupt.artifact_exists("demo", VERIFY_DOC), "ticket must survive refusal");
        }

        // --- spec「驗證指紋錨與失效判定」---

        /// 帶完整驗證章的 meta：tasks_total 任務錨＋entries 內容錨。
        fn stamped_meta(tasks_total: usize, entries: &[(&str, &str)]) -> crate::model::ChangeMeta {
            let mut y = format!(
                "schema: spec-driven\nverified_at: 2026-08-01\nverified_by: Ver <v@example.com>\nverified_with: claude\nverified_tasks_total: {tasks_total}\nverified_scope:\n"
            );
            for (p, h) in entries {
                y.push_str(&format!("  - path: {p}\n    hash: {h}\n"));
            }
            crate::model::ChangeMeta::from_text(Some(&y)).expect("meta parses")
        }

        #[test]
        fn freshness_matches_the_review_station_bit_for_bit() {
            // spec「路徑正規化與行尾 CRLF→LF 後 SHA-256 規則 SHALL 與審查站位元級
            // 同構（共用同一實作）」：同一組錨與現值，兩站判定必須逐一相同。
            let h = content_fingerprint(FILE_A);
            let path = "crates/a/src/lib.rs";
            let grown = format!("{FILE_A}fn extra() {{}}\n");
            let crlf = FILE_A.replace('\n', "\r\n");
            let cases: &[(&str, &[(&str, &str)], usize, usize, Freshness)] = &[
                ("all anchors match", REPO, 5, 5, Freshness::Fresh),
                ("scope file modified", &[(path, "")], 5, 5, Freshness::Stale),
                ("scope file gone", &[], 5, 5, Freshness::Stale),
                ("task count grew", REPO, 6, 6, Freshness::Stale),
                ("task unchecked", REPO, 5, 4, Freshness::Stale),
            ];
            for (label, repo, total, complete, want) in cases {
                let repo: Vec<(&str, &str)> = repo
                    .iter()
                    .map(|(p, c)| (*p, if *p == path && c.is_empty() { grown.as_str() } else { *c }))
                    .collect();
                let verified = stamped_meta(5, &[(path, h.as_str())]);
                assert_eq!(
                    freshness(&VERIFY, &verified, &code_counts(*total, *complete), &files(&repo)),
                    *want,
                    "verify station, case: {label}"
                );
                let reviewed = crate::model::ChangeMeta {
                    reviewed_at: verified.verified_at.clone(),
                    reviewed_tasks_total: verified.verified_tasks_total,
                    reviewed_scope: verified.verified_scope.clone(),
                    ..Default::default()
                };
                assert_eq!(
                    freshness(&REVIEW, &reviewed, &code_counts(*total, *complete), &files(&repo)),
                    *want,
                    "review station must agree, case: {label}"
                );
            }
            // spec Scenario「行尾差異不觸發失效」：LF → CRLF 仍 fresh。
            let now = [(path, crlf.as_str())];
            let meta = stamped_meta(5, &[(path, h.as_str())]);
            assert_eq!(freshness(&VERIFY, &meta, &code_counts(5, 5), &files(&now)), Freshness::Fresh);

            // spec Scenario「蓋章後補勾手動任務不失效」：勾 [M] 前後皆 fresh。
            let counts = |md: &str| crate::tasks::counts(&crate::tasks::parse(md));
            let open = "- [x] a\n- [x] b\n- [x] c\n- [x] d\n- [ ] [M] 手測\n";
            let checked = "- [x] a\n- [x] b\n- [x] c\n- [x] d\n- [x] [M] 手測\n";
            assert_eq!(freshness(&VERIFY, &meta, &counts(open), &files(REPO)), Freshness::Fresh, "手測未勾");
            assert_eq!(freshness(&VERIFY, &meta, &counts(checked), &files(REPO)), Freshness::Fresh, "手測補勾");

            // binary 分支的兩站同構直接釘在同一張表：位元組章在兩站以同一分流
            // 實作判定——未變 fresh、變動 stale，判定逐一相同。
            const BIN_FLIPPED: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0xFF, 0x01];
            let bin_cases: &[(&str, &[(&str, &[u8])], Freshness)] = &[
                ("binary unchanged", &[(path, BIN_A)], Freshness::Fresh),
                ("binary bytes flipped", &[(path, BIN_FLIPPED)], Freshness::Stale),
            ];
            let h_bin = content_fingerprint_bytes(BIN_A);
            for (label, now, want) in bin_cases {
                let verified = stamped_meta(5, &[(path, h_bin.as_str())]);
                assert_eq!(
                    freshness(&VERIFY, &verified, &code_counts(5, 5), &byte_files(now)),
                    *want,
                    "verify station, case: {label}"
                );
                let reviewed = crate::model::ChangeMeta {
                    reviewed_at: verified.verified_at.clone(),
                    reviewed_tasks_total: verified.verified_tasks_total,
                    reviewed_scope: verified.verified_scope.clone(),
                    ..Default::default()
                };
                assert_eq!(
                    freshness(&REVIEW, &reviewed, &code_counts(5, 5), &byte_files(now)),
                    *want,
                    "review station must agree, case: {label}"
                );
            }
        }

        #[test]
        fn freshness_reads_absence_as_unverified() {
            // 缺席讀作未驗證：無驗證章的 meta 沒有可判定的錨，即使審查章齊備。
            const REVIEWED_ONLY: &str = "schema: spec-driven\nreviewed_at: 2026-08-01\nreviewed_tasks_total: 5\nreviewed_scope:\n  - path: crates/a/src/lib.rs\n    hash: dead\n";
            let meta = crate::model::ChangeMeta::from_text(Some(REVIEWED_ONLY)).expect("parses");
            assert_eq!(freshness(&VERIFY, &meta, &code_counts(5, 5), &files(REPO)), Freshness::Unknown);
        }
    }
}
