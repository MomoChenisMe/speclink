//! 討論結論的分刀解析（plan-unpromoted-cuts design D1）：Decision 欄位裡以粗體刀編號
//! 開頭的頂層列點為一把刀。規則逐條移植自桌面 packages/ui/src/discussionModel.ts 的
//! `cutCard`（第 4 刀改由前端讀引擎資料前，兩邊的判定必須相同）。純函式，不做 I/O。

use super::{DECISION_LABEL, DEFERRED_LABEL, REJECTED_LABEL};
use regex::Regex;
use std::sync::LazyLock;

/// 結論裡規劃的一把刀。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cut {
    /// 刀編號的原文（`cut 2`、`第 2 刀`、`刀二`）。
    pub id: String,
    /// 範圍句；沒有時為空字串。
    pub title: String,
    /// 變更名，依出現順序去重。
    pub names: Vec<String>,
    /// 變更名寫在粗體內（``**cut N `change-name`**``）。false 時名稱取自粗體之後，
    /// 可能混有識別字或檔名，無從判定是否已轉出。
    pub names_in_bold: bool,
}

impl Cut {
    /// 還不在 `promoted_to` 的變更名，依 `names` 的順序。全部已轉出時為空；變更名不在粗體內時
    /// 也為空——無從判定，不算尚未轉出（與前端 `cutPromotion` 的 unknown 同義）。
    pub fn unpromoted(&self, promoted_to: &[String]) -> Vec<String> {
        if !self.names_in_bold {
            return Vec::new();
        }
        self.names.iter().filter(|n| !promoted_to.contains(n)).cloned().collect()
    }
}

/// Decision 欄位到這些標籤行為止。
const NEXT_LABELS: [&str; 5] = [
    "**Rationale**:",
    REJECTED_LABEL,
    DEFERRED_LABEL,
    "**Capture to**:",
    "**Next**:",
];

/// 粗體開頭的刀編號：`cut <編號>`（編號以數字起頭：`cut 3a`；`cutover` 不算）、
/// `第 N 刀`、`刀…`。與 discussionModel.ts 的 `CUT_ID_RE` 同形；`\d` 寫成 `[0-9]`，
/// 因為 Rust 的 `\d` 含全形數字而 JS 的不含。
static CUT_ID: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:cut\s*[0-9][^\s，,：:（(]*|第\s*[^\s刀]+?\s*刀|刀[^\s，,：:（(]*)").unwrap()
});

/// 反引號片段（含內容為空的）。
static TICKS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"`[^`]*`").unwrap());

/// 反引號片段的內容。
static TICK_CONTENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"`([^`]+)`").unwrap());

/// 解析結論（[`super::conclusion_text`] 的回傳值）的 Decision 欄位裡的刀，依結論內的順序。
/// 沒有 Decision 欄位或沒有刀時回空。
pub fn parse_cuts(conclusion: &str) -> Vec<Cut> {
    let mut lines = conclusion.lines();
    let Some(first) = lines.find_map(|l| l.strip_prefix(DECISION_LABEL)) else {
        return Vec::new();
    };
    // 標籤行的其餘文字也屬於欄位（與前端 splitLabeledFields 的 `:\s?` 同形）。
    let first = first.strip_prefix(' ').unwrap_or(first);
    // 開著的圍欄符號串；圍欄行與圍欄內的行都不是列點。
    let mut open: Option<&str> = None;
    std::iter::once(first)
        .chain(lines.take_while(|l| !NEXT_LABELS.iter().any(|label| l.starts_with(label))))
        .filter(|line| {
            let fence = fence_of(line);
            match (open, fence) {
                (None, Some((marks, _))) => open = Some(marks),
                (Some(o), Some((marks, info)))
                    if marks.starts_with(&o[..1]) && marks.len() >= o.len() && info.trim().is_empty() =>
                {
                    open = None
                }
                (None, None) => return true,
                _ => {}
            }
            false
        })
        .filter_map(|line| line.strip_prefix("- "))
        .filter_map(cut_of)
        .collect()
}

/// 圍欄行的符號串（3 個以上 ` 或 ~，容忍前導空白）與其後的 info 字串；不是圍欄行時為 None。
/// 與前端 specSections.ts 的 `FENCE_RE` 同形：收尾行要與開頭同字元、長度不短於開頭、
/// 其後沒有 info 字串。
fn fence_of(line: &str) -> Option<(&str, &str)> {
    let t = line.trim_start();
    let mark = t.chars().next().filter(|c| *c == '`' || *c == '~')?;
    let n = t.len() - t.trim_start_matches(mark).len();
    (n >= 3).then(|| t.split_at(n))
}

/// 頂層列點首行（去掉 `- `）為一把刀時回刀。
fn cut_of(item: &str) -> Option<Cut> {
    let rest = item.trim().strip_prefix("**")?;
    let close = rest.find("**")?;
    let (bold, after) = (&rest[..close], &rest[close + 2..]);
    let plain = TICKS
        .replace_all(bold, " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let id = CUT_ID.find(&plain)?.as_str();
    let scope = plain[id.len()..]
        .trim_start_matches(|c: char| c.is_whitespace() || ":：、—–-".contains(c))
        .trim();
    let tail = after.trim_start();
    let tail = tail.strip_prefix([':', '：']).unwrap_or(tail).trim();
    // 範圍句取自粗體時粗體之後全部是說明；否則以第一個「——」切範圍句與說明。
    let title = if !scope.is_empty() {
        scope
    } else {
        tail.find("——").map_or(tail, |i| tail[..i].trim())
    };
    let in_bold = tick_names(bold);
    let names_in_bold = !in_bold.is_empty();
    Some(Cut {
        id: id.to_string(),
        title: title.to_string(),
        names: if names_in_bold { in_bold } else { tick_names(after) },
        names_in_bold,
    })
}

/// 反引號片段裡的名稱，依出現順序去重；內容含空白的不算（`speclink plan` 這類指令不是變更名）。
fn tick_names(text: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for m in TICK_CONTENT.captures_iter(text) {
        let name = m[1].trim();
        if !name.is_empty() && !name.contains(char::is_whitespace) && !names.iter().any(|n| n == name) {
            names.push(name.to_string());
        }
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 一段只含 Decision 欄位的結論：定論句之後一條頂層列點。
    fn decision_with(item: &str) -> String {
        format!("**Decision**: 定案。\n\n- {item}\n")
    }

    fn only_cut(item: &str) -> Cut {
        let cuts = parse_cuts(&decision_with(item));
        assert_eq!(cuts.len(), 1, "{item:?} → {cuts:?}");
        cuts.into_iter().next().unwrap()
    }

    // --- spec「分刀解析」Example 的八列 ---

    #[test]
    fn example_rows_that_are_cuts_with_names_in_bold() {
        // (列點首行, cut, changes, title)
        let rows: &[(&str, &str, &[&str], &str)] = &[
            (
                "**cut 2 `plan-unpromoted-cuts`**: 引擎解析討論結論的分刀列",
                "cut 2",
                &["plan-unpromoted-cuts"],
                "引擎解析討論結論的分刀列",
            ),
            ("**Cut 3a `add-x`**: 範圍——補充說明", "Cut 3a", &["add-x"], "範圍"),
            ("**第 2 刀 `add-y`**", "第 2 刀", &["add-y"], ""),
            ("**刀二 收尾 `add-z`**", "刀二", &["add-z"], "收尾"),
            ("**cut 1 `speclink plan` `add-w`**: 說明", "cut 1", &["add-w"], "說明"),
        ];
        for (item, id, names, title) in rows {
            let cut = only_cut(item);
            assert_eq!(cut.id, *id, "{item}");
            assert_eq!(cut.names, *names, "{item}");
            assert_eq!(cut.title, *title, "{item}");
            assert!(cut.names_in_bold, "{item}");
        }
    }

    #[test]
    fn example_rows_that_are_not_cuts() {
        for item in ["**cutover `add-v`**: 說明", "`add-u` 不以粗體開頭"] {
            assert_eq!(parse_cuts(&decision_with(item)), [], "{item}");
        }
    }

    #[test]
    fn example_row_cut_without_names_in_bold_is_flagged() {
        // 是刀，但粗體內沒有變更名：名稱改取粗體之後，names_in_bold 為 false（plan 不列出）。
        let cut = only_cut("**cut 2 外殼**（`add-x`）");
        assert_eq!(cut.id, "cut 2");
        assert_eq!(cut.title, "外殼");
        assert_eq!(cut.names, ["add-x"]);
        assert!(!cut.names_in_bold);
    }

    // --- Decision 欄位邊界 ---

    #[test]
    fn a_cut_item_after_the_rationale_label_is_not_a_cut() {
        let conclusion = "**Decision**: 定案。\n\n- **cut 1 `add-a`**: 第一刀\n\n\
                          **Rationale**: 理由。\n\n- **cut 2 `add-b`**: 不在 Decision 內\n";
        let cuts = parse_cuts(conclusion);
        assert_eq!(cuts.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), ["cut 1"]);
    }

    #[test]
    fn a_nested_item_is_not_a_cut() {
        let conclusion = "**Decision**: 定案。\n\n- **cut 1 `add-a`**: 第一刀\n  - **cut 2 `add-b`**: 巢狀\n";
        let cuts = parse_cuts(conclusion);
        assert_eq!(cuts.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), ["cut 1"]);
    }

    #[test]
    fn a_conclusion_without_a_decision_label_has_no_cuts() {
        let conclusion = "**Rationale**: 理由。\n\n- **cut 1 `add-a`**: 第一刀\n";
        assert_eq!(parse_cuts(conclusion), []);
    }

    #[test]
    fn cuts_keep_their_order_in_the_conclusion() {
        let conclusion = "**Decision**: 定案。\n\n- **cut 1 `add-a`**: 一\n- 一般細節\n- **cut 2 `add-b`**: 二\n";
        let cuts = parse_cuts(conclusion);
        assert_eq!(cuts.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), ["cut 1", "cut 2"]);
    }

    // --- sharp-edges 稽核（tasks 5.1）：手改討論檔的五種形狀 ---

    #[test]
    fn crlf_line_endings_parse_like_lf() {
        // 判定：無缺口——`str::lines` 去掉行尾 `\r`，標籤、列點與圍欄判定都不受影響。
        let lf = "**Decision**: 定案。\n\n- **cut 1 `add-a`**: 一\n\n**Rationale**: 理由。\n\n- **cut 2 `add-b`**: 二\n";
        let cuts = parse_cuts(&lf.replace('\n', "\r\n"));
        assert_eq!(cuts, parse_cuts(lf));
        assert_eq!(cuts[0].title, "一");
    }

    #[test]
    fn an_unpaired_bold_is_not_a_cut() {
        // 判定：無缺口——找不到收尾 `**` 就不是刀（與前端 indexOf 同形）。
        assert_eq!(parse_cuts(&decision_with("**cut 1 `add-a`: 說明")), []);
    }

    #[test]
    fn an_unclosed_backtick_in_bold_yields_no_name_in_bold() {
        // 判定：無缺口——未閉合的反引號不成名稱，names_in_bold 為 false，plan 不列出。
        let cut = only_cut("**cut 1 `add-a**: 說明");
        assert_eq!(cut.id, "cut 1");
        assert!(!cut.names_in_bold);
    }

    #[test]
    fn a_very_long_item_parses_in_linear_time() {
        // 判定：無缺口——regex crate 保證線性時間，沒有回溯爆炸；長度不設上限。
        let title = "長".repeat(200_000);
        let cut = only_cut(&format!("**cut 1 `add-a`**: {title}"));
        assert_eq!(cut.title.chars().count(), 200_000);
    }

    #[test]
    fn a_change_name_repeated_across_cuts_stays_in_each_cut() {
        // 判定：無缺口——同一刀內去重；兩把刀寫了同一名稱是結論本身的寫法，照實各列一把。
        let conclusion = "**Decision**: 定案。\n\n- **cut 1 `add-a` `add-a`**: 一\n- **cut 2 `add-a`**: 二\n";
        let cuts = parse_cuts(conclusion);
        assert_eq!(cuts[0].names, ["add-a"]);
        assert_eq!(cuts[1].names, ["add-a"]);
    }

    // --- 圍欄：與前端 specSections.ts 的 fenceFlags 同一條收尾規則 ---

    #[test]
    fn a_shorter_fence_inside_a_longer_one_does_not_close_it() {
        let conclusion = "**Decision**: 定案。\n\n````\n```\n- **cut 1 `add-a`**: 範例\n```\n````\n\
                          - **cut 2 `add-b`**: 真的\n";
        let cuts = parse_cuts(conclusion);
        assert_eq!(cuts.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), ["cut 2"]);
    }

    #[test]
    fn a_tilde_line_does_not_close_a_backtick_fence() {
        let conclusion = "**Decision**: 定案。\n\n```\n~~~\n- **cut 1 `add-a`**: 範例\n```\n\
                          - **cut 2 `add-b`**: 真的\n";
        let cuts = parse_cuts(conclusion);
        assert_eq!(cuts.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), ["cut 2"]);
    }

    #[test]
    fn a_fence_line_with_an_info_string_does_not_close_a_fence() {
        let conclusion = "**Decision**: 定案。\n\n```\n```rust\n- **cut 1 `add-a`**: 範例\n```\n\
                          - **cut 2 `add-b`**: 真的\n";
        let cuts = parse_cuts(conclusion);
        assert_eq!(cuts.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), ["cut 2"]);
    }

    // --- 與前端判定不同、依規格字面的兩處（第 4 刀前端改讀引擎資料後兩邊一致）---

    #[test]
    fn a_bold_cut_paragraph_outside_a_list_item_is_not_a_cut() {
        // 判定：依規格只認行首 `- ` 的頂層列點；前端 splitVerdict 另把列點後的散文段交給 cutCard。
        let conclusion = "**Decision**: 定案。\n\n- **cut 1 `add-a`**: 一\n\n**cut 2 `add-b`**: 散文段\n";
        let cuts = parse_cuts(conclusion);
        assert_eq!(cuts.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), ["cut 1"]);
    }

    #[test]
    fn a_second_decision_label_does_not_end_the_field() {
        // 判定：依規格 Decision 欄位到下一個 Rationale／Rejected alternatives／Deferred／
        // Capture to／Next 標籤行為止，第二行 `**Decision**:` 不是邊界；前端只取最後一段。
        let conclusion = "**Decision**: 一\n\n- **cut 1 `add-a`**: 一\n\n**Decision**: 二\n\n- **cut 2 `add-b`**: 二\n";
        let cuts = parse_cuts(conclusion);
        assert_eq!(cuts.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), ["cut 1", "cut 2"]);
    }

    // --- 轉出判定 ---

    #[test]
    fn unpromoted_lists_the_names_not_in_promoted_to_in_names_order() {
        let cut = only_cut("**cut 1 `add-c` `add-a` `add-b`**: 三個");
        let promoted = ["add-a".to_string()];
        assert_eq!(cut.unpromoted(&promoted), ["add-c", "add-b"]);
        let all = ["add-a".to_string(), "add-b".to_string(), "add-c".to_string()];
        assert!(cut.unpromoted(&all).is_empty());
    }

    #[test]
    fn unpromoted_is_empty_when_the_names_are_not_in_bold() {
        // 粗體內沒有變更名：無從判定，不算尚未轉出。
        let cut = only_cut("**cut 2 外殼**（`add-x`）");
        assert!(cut.unpromoted(&[]).is_empty());
    }

    #[test]
    fn a_cut_item_inside_a_fenced_block_is_not_a_cut() {
        // 與桌面 parseListItems 同一條規則：圍欄內的行不是列點。
        let conclusion = "**Decision**: 定案。\n\n```\n- **cut 1 `add-a`**: 範例\n```\n";
        assert_eq!(parse_cuts(conclusion), []);
    }
}
