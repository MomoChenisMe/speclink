//! `KeyLines` — 對「頂層 `key: value` 行」做原位手術的逐行編輯原語。
//!
//! 討論記錄的 frontmatter 與 change 的 `.openspec.yaml` 都是手寫友善的 YAML 純量
//! 映射：本模組只認第 0 欄的 `key:` 前綴行，在區域內原位代換、移除、讀寫逗號
//! 清單、取代縮排區塊，其餘位元組逐字保留、永不重新序列化（手寫註解、鍵序、
//! 行尾全部留下）。兩個建構子決定區域：[`KeyLines::frontmatter`] 的區域是首行
//! `---` 到下一個 `---` 之間（討論側）；[`KeyLines::whole`] 的區域是整份文件
//! （change meta 不帶圍欄，整份就是映射），永遠可在尾端插入——尾行缺換行時
//! 插入前先補一個（沿文件多數行尾）。
//!
//! 介面：`get`／`list` 讀；`set`／`remove` 純量鍵；`push_list`／`remove_list`
//! 逗號清單累加與縮減（去重、統一以「, 」串接、清空即整行移除）；`set_block`
//! 區塊鍵（移除既有同名鍵與區塊後，在區域尾端寫 `key:` 行＋呼叫端給定的縮排
//! 續行原文）。
//!
//! **值逐字讀出、不解 YAML 引號**：`get` 回冒號後 trim 過的原文。寫入走
//! `util::yaml_scalar` 的值（身分、工具名）讀回要比對時，呼叫端先過
//! `util::yaml_unscalar`；change 名與討論 slug 不會被加引號，直接比對即可。
//!
//! **值逐字寫入、不做 YAML 跳脫**：`set` 把 `value` 原樣接在 `key: ` 後面。
//! 跳脫責任在呼叫端——寫自由文字（topic、人名）前先走 `util::yaml_scalar`；
//! 寫列舉字面、change 名、`true`、小寫 ASCII 排序鍵不需跳脫。
//!
//! **行尾**：每行沿用自己的行尾（LF 或 CRLF）；新插入的行沿用文件多數行尾，
//! 平手或沒有換行時用 LF。
//!
//! **純量鍵與區塊鍵**：`key: value`（冒號後有內容）是純量行，移除或丟重複時只碰
//! 那一行；`key:`（冒號後為空）是區塊鍵，緊接其後的縮排行、第 0 欄的 `- ` 序列項
//! 與區塊內的空行都屬於它，一併移除。純量行之後的空行與 `- ` 條列是別人的內容
//! ——未閉合 frontmatter 上區域＝整份文件，這條界線就是內文不被吃掉的保證。
//!
//! # 稽核結論（sharp edges）
//!
//! - **換行注入**：`set`／`push_list`／`set_block` 的鍵、值或續行含 `\n`／`\r` 一律
//!   回 `Err` 且不改任何行——這不是跳脫，是拒絕「一個值變成兩行、偽造出另一個鍵」
//!   的整類注入；呼叫端拿到自由文字仍要先 `yaml_scalar`。
//! - **未閉合 frontmatter**（缺尾 `---`）：與讀端同樣寬鬆，其餘整檔視為區域，原位
//!   代換與移除照做；只有新插一行無處可插回 `Err`。`whole` 區域沒有這條：尾端
//!   永遠可插。
//! - **空文件**：`frontmatter("")` 回 `None`（沒有首行 `---`）；`whole("")` 是空區域，
//!   插入得到 `key: value\n`。
//! - **只有 `---` 一行**：「未閉合、區域為空」——`get` 回 `None`、原位代換無鍵可換、
//!   新插一行回 `Err`；`text()` 逐位元不變。
//! - **鍵名含冒號**：鍵名逐字比對，`get("a:b")` 只認 `a:b:` 開頭的行；`get("a")`
//!   對 `a: b: c` 回 `b: c`，對 `a:b: v` 回 `b: v`（`a:` 前綴命中、第一個冒號之後
//!   全是值）。不做 YAML 鍵名解析，刻意行為；呼叫端的鍵名都是固定識別字。
//! - **值為空字串**：`set(key, "")` 寫成 `key:`（不留尾端空白）；`get` 回 `Some("")`、
//!   `list` 回空清單——與手寫的 `kind:` 空值行一致。注意這樣的行在 `remove`／丟重複
//!   時算區塊鍵。
//! - **重複鍵三條以上**：`set` 只留第一條（原位代換）、其餘整行（區塊鍵連區塊）丟；
//!   `remove` 全部移除。讀端只認第一條，寫後讀寫看法一致。
//! - **混合行尾**：代換沿該行、插入沿多數；移除不動其他行的行尾。不做正規化。
//! - **`---` 圍欄認法**：與既有讀端同樣寬鬆，`trim()` 後等於 `---` 即為圍欄
//!   （容忍尾端空白）。

use anyhow::{bail, Result};

/// 一份文件的逐行視圖；可編輯區域由建構子決定（frontmatter 圍欄內，或整份文件）。
#[derive(Debug, Clone)]
pub struct KeyLines {
    /// 每行含自己的行尾（最後一行可能沒有）。
    lines: Vec<String>,
    /// 區域起點：frontmatter 為 1（跳過首行 `---`），whole 為 0。
    start: usize,
    /// 區域終點（不含）：frontmatter 指向 closing `---` 行（未閉合時為
    /// `lines.len()`）；whole 恆為 `lines.len()`。
    end: usize,
    /// 尾端可插入：frontmatter 有 closing `---`（新行補在它前面）；whole 恆為 true。
    can_append: bool,
    /// 新插入行沿用的行尾。
    eol: &'static str,
}

fn eol_of(line: &str) -> &'static str {
    if line.ends_with("\r\n") {
        "\r\n"
    } else if line.ends_with('\n') {
        "\n"
    } else {
        ""
    }
}

fn majority_eol(lines: &[String]) -> &'static str {
    let crlf = lines.iter().filter(|l| l.ends_with("\r\n")).count();
    let lf = lines.iter().filter(|l| l.ends_with('\n')).count() - crlf;
    if crlf > lf {
        "\r\n"
    } else {
        "\n"
    }
}

fn is_fence(line: &str) -> bool {
    line.trim() == "---"
}

/// 第 0 欄 `key:` 命中時回冒號後的原文（含行尾）。
fn value_after_key<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.strip_prefix(key)?.strip_prefix(':')
}

/// 區塊鍵：冒號後除空白外沒有內容，值住在後面的縮排區塊。
fn is_block_key(line: &str, key: &str) -> bool {
    value_after_key(line, key).is_some_and(|rest| rest.trim().is_empty())
}

/// 區塊的續行（站別章的剝除規則已收攏到這裡）：縮排行、第 0 欄的 `- ` 序列項、
/// 以及區塊內的空行。
fn is_continuation(line: &str) -> bool {
    line.trim().is_empty() || line.starts_with([' ', '\t']) || line.starts_with("- ")
}

impl KeyLines {
    /// 首行 `---` 到下一個 `---` 之間為區域；首行不是 `---` 回 `None`。
    pub fn frontmatter(text: &str) -> Option<KeyLines> {
        let lines: Vec<String> = text.split_inclusive('\n').map(str::to_string).collect();
        if !lines.first().is_some_and(|l| is_fence(l)) {
            return None;
        }
        let eol = majority_eol(&lines);
        let close = lines
            .iter()
            .skip(1)
            .position(|l| is_fence(l))
            .map(|i| i + 1);
        let (end, can_append) = match close {
            Some(i) => (i, true),
            None => (lines.len(), false),
        };
        Some(KeyLines {
            lines,
            start: 1,
            end,
            can_append,
            eol,
        })
    }

    /// 整份文件為區域（change meta：不帶 `---` 圍欄的純 YAML 映射）。尾端永遠
    /// 可插；空文件是空區域。
    pub fn whole(text: &str) -> KeyLines {
        let lines: Vec<String> = text.split_inclusive('\n').map(str::to_string).collect();
        let eol = majority_eol(&lines);
        KeyLines {
            end: lines.len(),
            lines,
            start: 0,
            can_append: true,
            eol,
        }
    }

    fn region_lines(&self) -> impl Iterator<Item = (usize, &str)> {
        self.lines[self.start..self.end]
            .iter()
            .enumerate()
            .map(|(i, l)| (self.start + i, l.as_str()))
    }

    /// 尾端可插的唯一守門：未閉合 frontmatter 無處可插回 `Err`。要在改任何行之前呼叫。
    fn ensure_can_append(&self, key: &str) -> Result<()> {
        if !self.can_append {
            bail!("frontmatter is not closed — cannot insert `{key}:`");
        }
        Ok(())
    }

    /// 區域尾端就是文件尾端且尾行缺行尾時，先補一個（沿多數行尾），新行才不會黏在
    /// 它後面。
    fn pad_tail(&mut self) {
        if self.end == self.lines.len() {
            if let Some(last) = self.lines.last_mut() {
                if eol_of(last).is_empty() {
                    last.push_str(self.eol);
                }
            }
        }
    }

    /// 區域內第一條 `key:` 的值（trim 後）。
    pub fn get(&self, key: &str) -> Option<&str> {
        self.region_lines()
            .find_map(|(_, l)| value_after_key(l, key))
            .map(str::trim)
    }

    /// 逗號清單：split(',')、trim、去空。鍵缺席回空清單。
    pub fn list(&self, key: &str) -> Vec<String> {
        self.get(key)
            .map(|v| {
                v.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn render(key: &str, value: &str, eol: &str) -> String {
        if value.is_empty() {
            format!("{key}:{eol}")
        } else {
            format!("{key}: {value}{eol}")
        }
    }

    /// 移除索引 `i` 的鍵行；區塊鍵連同續行區塊一起移除。
    fn drop_key_line(&mut self, i: usize, key: &str) {
        let block = is_block_key(&self.lines[i], key);
        self.lines.remove(i);
        self.end -= 1;
        while block && i < self.end && is_continuation(&self.lines[i]) {
            self.lines.remove(i);
            self.end -= 1;
        }
    }

    /// 第一條 `key:` 行原位代換為 `key: value`（沿該行既有行尾），其餘重複鍵丟掉
    /// （區塊鍵連區塊）；缺則補在 closing `---` 前。失敗：鍵或值含換行（注入），
    /// 或鍵不存在而 frontmatter 未閉合、無處可插——皆不改任何行。
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        if key.contains(['\n', '\r']) || value.contains(['\n', '\r']) {
            bail!(
                "`{key}` value must be a single line: {}",
                value.escape_debug()
            );
        }
        let first = self
            .region_lines()
            .find(|(_, l)| value_after_key(l, key).is_some())
            .map(|(i, _)| i);
        match first {
            Some(first) => {
                let eol = eol_of(&self.lines[first]);
                self.lines[first] = Self::render(key, value, eol);
                let mut i = first + 1;
                while i < self.end {
                    if value_after_key(&self.lines[i], key).is_some() {
                        self.drop_key_line(i, key);
                    } else {
                        i += 1;
                    }
                }
            }
            None => {
                self.ensure_can_append(key)?;
                self.pad_tail();
                self.lines
                    .insert(self.end, Self::render(key, value, self.eol));
                self.end += 1;
            }
        }
        Ok(())
    }

    /// 移除每一條 `key:` 行；區塊鍵連同續行區塊一起移除。鍵不存在時不改。
    pub fn remove(&mut self, key: &str) {
        let mut i = self.start;
        while i < self.end {
            if value_after_key(&self.lines[i], key).is_some() {
                self.drop_key_line(i, key);
            } else {
                i += 1;
            }
        }
    }

    /// 逗號清單累加：`item` 已在清單不改；否則以「, 」串接既有項與 `item` 原位寫回
    /// （既有為空清單時就是單值）。既有值的原始間距不保留、統一為「, 」。失敗同
    /// [`KeyLines::set`]（換行注入、無處可插），另外既有值被引號包住時也拒絕——
    /// 引號夾在清單中間會讓整份文件解析失敗，沒有寫端會產生這種值。
    pub fn push_list(&mut self, key: &str, item: &str) -> Result<()> {
        if self.get(key).is_some_and(|v| v.starts_with(['"', '\''])) {
            bail!("`{key}` holds a quoted value — cannot append to it as a comma list");
        }
        let mut items = self.list(key);
        if items.iter().any(|s| s == item) {
            return Ok(());
        }
        items.push(item.to_string());
        self.set(key, &items.join(", "))
    }

    /// 逗號清單縮減：`item` 不在清單不改；移除後仍有項則以「, 」重組原位寫回；清空
    /// 則整行移除。失敗同 [`KeyLines::set`]。
    pub fn remove_list(&mut self, key: &str, item: &str) -> Result<()> {
        let items = self.list(key);
        let remaining: Vec<&str> =
            items.iter().map(String::as_str).filter(|s| *s != item).collect();
        if remaining.len() == items.len() {
            return Ok(());
        }
        if remaining.is_empty() {
            self.remove(key);
            return Ok(());
        }
        self.set(key, &remaining.join(", "))
    }

    /// 區塊鍵：移除既有同名鍵（連區塊）後，在區域尾端寫 `key:` 行與 `body` 每一行
    /// （呼叫端提供含縮排的續行原文；本方法逐行加行尾，不做縮排判斷）。失敗：鍵或
    /// 任一行含換行（注入），或 frontmatter 未閉合無處可插——皆不改任何行。
    pub fn set_block(&mut self, key: &str, body: &[String]) -> Result<()> {
        if key.contains(['\n', '\r']) {
            bail!("`{key}` key must be a single line");
        }
        if let Some(line) = body.iter().find(|l| l.contains(['\n', '\r'])) {
            bail!(
                "`{key}` block line must be a single line: {}",
                line.escape_debug()
            );
        }
        self.ensure_can_append(key)?;
        self.remove(key);
        self.pad_tail();
        let eol = self.eol;
        let block = std::iter::once(Self::render(key, "", eol))
            .chain(body.iter().map(|l| format!("{l}{eol}")));
        for line in block {
            self.lines.insert(self.end, line);
            self.end += 1;
        }
        Ok(())
    }

    /// 回寫整份文件（含區域外的行）。
    pub fn text(&self) -> String {
        self.lines.concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FM: &str = "---\ntopic: Alpha\nslug: alpha\nstatus: open\ncreated: 2026-01-02\n---\n\n\
                      # Discussion: Alpha\n\nstatus: open — 內文引用的字串\n";

    // --- 建構與 get ---

    #[test]
    fn frontmatter_returns_none_when_first_line_is_not_a_fence() {
        assert!(KeyLines::frontmatter("topic: x\n---\n").is_none());
        assert!(KeyLines::frontmatter("").is_none());
        assert!(KeyLines::frontmatter("\n---\nstatus: open\n---\n").is_none());
    }

    #[test]
    fn get_reads_only_column_zero_keys_inside_the_region() {
        let kl = KeyLines::frontmatter(FM).unwrap();
        assert_eq!(kl.get("status"), Some("open"));
        assert_eq!(kl.get("topic"), Some("Alpha"));
        assert_eq!(kl.get("missing"), None);
        // 內文的 `status: open` 不在區域內；縮排行不是頂層鍵。
        let body_only = "---\nslug: a\n  status: indented\n---\n\nstatus: open\n";
        let kl = KeyLines::frontmatter(body_only).unwrap();
        assert_eq!(kl.get("status"), None);
        // 鍵名是前綴不算命中：`slug_extra:` 不是 `slug:`。
        let kl = KeyLines::frontmatter("---\nslug_extra: b\n---\n").unwrap();
        assert_eq!(kl.get("slug"), None);
    }

    #[test]
    fn get_trims_the_value() {
        let kl = KeyLines::frontmatter("---\nboard_rank:   n   \n---\n").unwrap();
        assert_eq!(kl.get("board_rank"), Some("n"));
    }

    // --- set ---

    #[test]
    fn set_replaces_first_line_in_place_and_drops_duplicates() {
        let text = "---\ntopic: T\nstatus: open\ncreated: 2026-01-02\nstatus: concluded\n---\n\nbody status: open\n";
        let mut kl = KeyLines::frontmatter(text).unwrap();
        kl.set("status", "promoted").unwrap();
        assert_eq!(
            kl.text(),
            "---\ntopic: T\nstatus: promoted\ncreated: 2026-01-02\n---\n\nbody status: open\n"
        );
    }

    #[test]
    fn set_appends_before_the_closing_fence_when_absent() {
        let mut kl = KeyLines::frontmatter(FM).unwrap();
        kl.set("hold", "true").unwrap();
        assert_eq!(
            kl.text(),
            "---\ntopic: Alpha\nslug: alpha\nstatus: open\ncreated: 2026-01-02\nhold: true\n---\n\n\
             # Discussion: Alpha\n\nstatus: open — 內文引用的字串\n"
        );
        assert_eq!(kl.get("hold"), Some("true"));
    }

    #[test]
    fn set_on_unclosed_frontmatter_rejects_the_insert_and_keeps_the_text() {
        let text = "---\nstatus: open\ncreated: 2026-01-02\n";
        let mut kl = KeyLines::frontmatter(text).unwrap();
        assert!(kl.set("hold", "true").is_err());
        assert_eq!(kl.text(), text);
        // 原位代換不需要尾 `---`，照做。
        kl.set("status", "concluded").unwrap();
        assert_eq!(kl.text(), "---\nstatus: concluded\ncreated: 2026-01-02\n");
    }

    #[test]
    fn set_keeps_the_line_ending_of_the_replaced_line() {
        let mut kl =
            KeyLines::frontmatter("---\nstatus: open\r\ncreated: 2026-01-02\n---\n").unwrap();
        kl.set("status", "promoted").unwrap();
        assert_eq!(
            kl.text(),
            "---\nstatus: promoted\r\ncreated: 2026-01-02\n---\n"
        );
    }

    #[test]
    fn set_drops_a_duplicate_block_key_together_with_its_block() {
        let text = "---\nk: 1\nk:\n  - a\n  - b\nz: 9\n---\n";
        let mut kl = KeyLines::frontmatter(text).unwrap();
        kl.set("k", "2").unwrap();
        assert_eq!(kl.text(), "---\nk: 2\nz: 9\n---\n");
    }

    // --- remove ---

    #[test]
    fn remove_drops_every_key_line_and_a_block_key_with_its_block() {
        let text = "---\na: 1\nreviewed_scope:\n  - path: x\n- path: y\n\n  hash: z\nb: 2\nreviewed_scope: dup\n---\nbody\n";
        let mut kl = KeyLines::frontmatter(text).unwrap();
        kl.remove("reviewed_scope");
        assert_eq!(kl.text(), "---\na: 1\nb: 2\n---\nbody\n");
    }

    #[test]
    fn remove_of_a_scalar_key_keeps_the_blank_and_list_lines_after_it() {
        // 未閉合 frontmatter：區域＝整份文件。`hold: true` 是純量行，移除只吃這一行，
        // 後面的空行與 `- ` 條列是內文、逐位元保留。
        let text = "---\nstatus: open\nhold: true\n\n- item a\n- item b\n";
        let mut kl = KeyLines::frontmatter(text).unwrap();
        kl.remove("hold");
        assert_eq!(kl.text(), "---\nstatus: open\n\n- item a\n- item b\n");
    }

    #[test]
    fn remove_of_an_absent_key_is_a_no_op() {
        let mut kl = KeyLines::frontmatter(FM).unwrap();
        kl.remove("hold");
        assert_eq!(kl.text(), FM);
    }

    #[test]
    fn remove_works_on_unclosed_frontmatter() {
        let mut kl = KeyLines::frontmatter("---\nstatus: open\nhold: true\n").unwrap();
        kl.remove("hold");
        assert_eq!(kl.text(), "---\nstatus: open\n");
    }

    // --- 逗號清單 ---

    #[test]
    fn list_splits_on_commas_trims_and_drops_empties() {
        let kl = KeyLines::frontmatter("---\npromoted_to: a, b,,  c \n---\n").unwrap();
        assert_eq!(kl.list("promoted_to"), vec!["a", "b", "c"]);
        assert!(kl.list("missing").is_empty());
        assert!(KeyLines::frontmatter("---\npromoted_to:\n---\n")
            .unwrap()
            .list("promoted_to")
            .is_empty());
    }

    // --- 行尾 ---

    #[test]
    fn crlf_records_insert_new_lines_with_crlf() {
        let mut kl = KeyLines::frontmatter("---\r\nstatus: open\r\n---\r\n\r\nbody\r\n").unwrap();
        kl.set("hold", "true").unwrap();
        assert_eq!(
            kl.text(),
            "---\r\nstatus: open\r\nhold: true\r\n---\r\n\r\nbody\r\n"
        );
    }

    #[test]
    fn new_lines_follow_the_majority_line_ending() {
        let mut kl = KeyLines::frontmatter("---\r\na: 1\r\nb: 2\r\nc: 3\n---\n").unwrap();
        kl.set("d", "4").unwrap();
        assert_eq!(kl.text(), "---\r\na: 1\r\nb: 2\r\nc: 3\nd: 4\r\n---\n");
    }

    #[test]
    fn text_round_trips_untouched_input_byte_for_byte() {
        let text = "---\r\ntopic: x\n---\n\n## Body\n\nno newline at end";
        assert_eq!(KeyLines::frontmatter(text).unwrap().text(), text);
    }

    // --- 稽核邊界（sharp edges） ---

    #[test]
    fn set_rejects_a_key_or_value_with_a_line_break() {
        let text = "---\nstatus: open\n---\n";
        let mut kl = KeyLines::frontmatter(text).unwrap();
        assert!(kl.set("hold", "true\nstatus: promoted").is_err());
        assert!(kl.set("hold\nx", "true").is_err());
        assert!(kl.set("hold", "true\r").is_err());
        assert_eq!(kl.text(), text);
    }

    #[test]
    fn lone_fence_is_an_unclosed_empty_frontmatter() {
        for text in ["---\n", "---", "--- \n"] {
            let mut kl = KeyLines::frontmatter(text).unwrap();
            assert_eq!(kl.get("status"), None);
            assert!(kl.set("status", "open").is_err());
            kl.remove("status");
            assert_eq!(kl.text(), text);
        }
    }

    #[test]
    fn keys_are_matched_literally_even_with_colons() {
        let kl = KeyLines::frontmatter("---\na:b: v\na: b: c\n---\n").unwrap();
        assert_eq!(kl.get("a:b"), Some("v"));
        // `a:b: v` 也是 `a:` 開頭的行——第一個冒號之後全是值，第一條命中就回。
        assert_eq!(kl.get("a"), Some("b: v"));
        assert_eq!(
            KeyLines::frontmatter("---\na: b: c\n---\n")
                .unwrap()
                .get("a"),
            Some("b: c")
        );
    }

    #[test]
    fn empty_value_writes_a_bare_key_and_reads_back_empty() {
        let mut kl = KeyLines::frontmatter("---\nkind: improve\n---\n").unwrap();
        kl.set("kind", "").unwrap();
        assert_eq!(kl.text(), "---\nkind:\n---\n");
        assert_eq!(kl.get("kind"), Some(""));
        assert!(kl.list("kind").is_empty());
        kl.set("hold", "").unwrap();
        assert_eq!(kl.text(), "---\nkind:\nhold:\n---\n");
    }

    #[test]
    fn three_or_more_duplicate_keys_collapse_on_set_and_vanish_on_remove() {
        let text = "---\nk: 1\nother: x\nk: 2\nk:\n  cont\nk: 4\n---\n";
        let mut kl = KeyLines::frontmatter(text).unwrap();
        kl.set("k", "z").unwrap();
        // 重複的純量鍵只丟那一行；重複的區塊鍵（`k:`）連 `  cont` 一起丟。
        assert_eq!(kl.text(), "---\nk: z\nother: x\n---\n");
        let mut kl = KeyLines::frontmatter(text).unwrap();
        kl.remove("k");
        assert_eq!(kl.text(), "---\nother: x\n---\n");
        assert_eq!(kl.get("k"), None);
    }

    // --- whole（change meta：整份文件為區域）---

    #[test]
    fn whole_edits_the_entire_document_and_pads_a_missing_final_newline_before_insert() {
        let mut kl = KeyLines::whole("schema: spec-driven\ncreated: 2026-07-01");
        assert_eq!(kl.get("schema"), Some("spec-driven"));
        assert_eq!(kl.get("created"), Some("2026-07-01"));
        kl.set("board_rank", "n").unwrap();
        assert_eq!(kl.text(), "schema: spec-driven\ncreated: 2026-07-01\nboard_rank: n\n");
    }

    #[test]
    fn whole_on_an_empty_document_inserts_a_single_line() {
        let mut kl = KeyLines::whole("");
        assert_eq!(kl.get("k"), None);
        assert_eq!(kl.text(), "");
        kl.set("k", "v").unwrap();
        assert_eq!(kl.text(), "k: v\n");
    }

    #[test]
    fn whole_crlf_document_inserts_new_lines_with_crlf() {
        let mut kl = KeyLines::whole("a: 1\r\nb: 2\r\n");
        kl.set("c", "3").unwrap();
        assert_eq!(kl.text(), "a: 1\r\nb: 2\r\nc: 3\r\n");
        // 尾行缺換行：補行也沿多數行尾。
        let mut kl = KeyLines::whole("a: 1\r\nb: 2");
        kl.set("c", "3").unwrap();
        assert_eq!(kl.text(), "a: 1\r\nb: 2\r\nc: 3\r\n");
        let mut kl = KeyLines::whole("a: 1\r\n");
        kl.set_block("s", &["  - x".to_string()]).unwrap();
        assert_eq!(kl.text(), "a: 1\r\ns:\r\n  - x\r\n");
    }

    #[test]
    fn whole_set_replaces_in_place_and_remove_drops_blocks_from_line_zero() {
        // 第 0 行也在區域內；`---` 不是圍欄，只是一行普通內容。
        let text = "board_rank: a\nreviewed_scope:\n  - path: x\n    hash: y\nstarted_at: 2026-01-01\n---\n";
        let mut kl = KeyLines::whole(text);
        kl.set("board_rank", "b").unwrap();
        kl.remove("reviewed_scope");
        kl.remove("started_at");
        assert_eq!(kl.text(), "board_rank: b\n---\n");
    }

    // --- push_list ---

    #[test]
    fn push_list_appends_with_comma_space_and_skips_present_items() {
        let mut kl = KeyLines::whole("schema: spec-driven\nfrom_discussion: alpha\ncreated: 2026-07-01\n");
        kl.push_list("from_discussion", "alpha").unwrap();
        assert_eq!(kl.text(), "schema: spec-driven\nfrom_discussion: alpha\ncreated: 2026-07-01\n");
        kl.push_list("from_discussion", "beta").unwrap();
        assert_eq!(kl.text(), "schema: spec-driven\nfrom_discussion: alpha, beta\ncreated: 2026-07-01\n");
        kl.push_list("from_discussion", "beta").unwrap();
        assert_eq!(kl.text(), "schema: spec-driven\nfrom_discussion: alpha, beta\ncreated: 2026-07-01\n");
        assert_eq!(kl.list("from_discussion"), vec!["alpha", "beta"]);
    }

    #[test]
    fn push_list_on_an_absent_or_empty_key_writes_a_single_value() {
        let mut kl = KeyLines::whole("schema: spec-driven\n");
        kl.push_list("restale_from", "alpha").unwrap();
        assert_eq!(kl.text(), "schema: spec-driven\nrestale_from: alpha\n");
        // 存在但空值：原位寫單值，不追加第二條同名鍵。
        let mut kl = KeyLines::whole("from_discussion:\nschema: spec-driven\n");
        kl.push_list("from_discussion", "alpha").unwrap();
        assert_eq!(kl.text(), "from_discussion: alpha\nschema: spec-driven\n");
    }

    #[test]
    fn push_list_rejects_an_item_with_a_line_break_and_keeps_the_text() {
        let text = "from_discussion: alpha\n";
        let mut kl = KeyLines::whole(text);
        assert!(kl.push_list("from_discussion", "b\nstatus: forged").is_err());
        assert_eq!(kl.text(), text);
    }

    #[test]
    fn push_list_refuses_a_quoted_list_value_and_keeps_the_text() {
        // 手寫成引號包住的清單：逗號累加會把引號夾在中間、整份文件解析失敗——拒絕不改。
        for text in ["from_discussion: \"a, b\"\n", "from_discussion: 'a'\n"] {
            let mut kl = KeyLines::whole(text);
            assert!(kl.push_list("from_discussion", "c").is_err());
            assert_eq!(kl.text(), text);
        }
    }

    // --- remove_list ---

    #[test]
    fn remove_list_shrinks_the_list_and_drops_the_line_when_empty() {
        let text = "schema: spec-driven\nrestale_from: alpha, beta\ncreated: 2026-07-01\n";
        let mut kl = KeyLines::whole(text);
        kl.remove_list("restale_from", "gamma").unwrap();
        kl.remove_list("missing", "alpha").unwrap();
        assert_eq!(kl.text(), text);
        kl.remove_list("restale_from", "alpha").unwrap();
        assert_eq!(kl.text(), "schema: spec-driven\nrestale_from: beta\ncreated: 2026-07-01\n");
        kl.remove_list("restale_from", "beta").unwrap();
        assert_eq!(kl.text(), "schema: spec-driven\ncreated: 2026-07-01\n");
        assert_eq!(kl.get("restale_from"), None);
    }

    // --- set_block ---

    #[test]
    fn set_block_replaces_an_existing_block_and_lands_at_the_region_tail() {
        let text = "schema: spec-driven\nreviewed_scope:\n  - path: old\n    hash: h0\ncreated: 2026-07-01\n";
        let mut kl = KeyLines::whole(text);
        kl.set_block("reviewed_scope", &["  - path: a".to_string(), "    hash: h1".to_string()])
            .unwrap();
        assert_eq!(
            kl.text(),
            "schema: spec-driven\ncreated: 2026-07-01\nreviewed_scope:\n  - path: a\n    hash: h1\n"
        );
        // 空 body：只寫 `key:` 行。
        kl.set_block("reviewed_scope", &[]).unwrap();
        assert_eq!(kl.text(), "schema: spec-driven\ncreated: 2026-07-01\nreviewed_scope:\n");
        // 尾行缺換行：插入前先補。
        let mut kl = KeyLines::whole("a: 1");
        kl.set_block("s", &["  - x".to_string()]).unwrap();
        assert_eq!(kl.text(), "a: 1\ns:\n  - x\n");
    }

    #[test]
    fn set_block_rejects_a_line_break_in_any_line_and_keeps_the_text() {
        let text = "schema: spec-driven\nreviewed_scope:\n  - path: old\n";
        let mut kl = KeyLines::whole(text);
        assert!(kl
            .set_block("reviewed_scope", &["  - path: a\n    hash: h".to_string()])
            .is_err());
        assert!(kl.set_block("reviewed_scope", &["  - path: a\r".to_string()]).is_err());
        assert!(kl.set_block("k\nx", &[]).is_err());
        assert_eq!(kl.text(), text);
    }

    #[test]
    fn set_block_inside_frontmatter_lands_before_the_closing_fence() {
        let mut kl = KeyLines::frontmatter("---\nslug: a\n---\nbody\n").unwrap();
        kl.set_block("scope", &["  - x".to_string()]).unwrap();
        assert_eq!(kl.text(), "---\nslug: a\nscope:\n  - x\n---\nbody\n");
        // 未閉合 frontmatter 無處可插。
        let text = "---\nslug: a\n";
        let mut kl = KeyLines::frontmatter(text).unwrap();
        assert!(kl.set_block("scope", &[]).is_err());
        assert_eq!(kl.text(), text);
    }

    #[test]
    fn mixed_line_endings_survive_a_remove_untouched() {
        let mut kl = KeyLines::frontmatter("---\r\na: 1\nhold: true\r\nb: 2\r\n---\n").unwrap();
        kl.remove("hold");
        assert_eq!(kl.text(), "---\r\na: 1\nb: 2\r\n---\n");
    }
}
