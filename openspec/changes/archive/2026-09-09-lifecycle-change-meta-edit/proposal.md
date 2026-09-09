## Why

change 的 meta 檔 `openspec/changes/<name>/.openspec.yaml` 是刀一處理的討論 frontmatter 的鏡像：`ChangeMeta` 有型別化的讀（`from_text`），沒有型別化的寫。八個「編輯既有 meta」的寫入點各自讀原文、做字串手術、寫回：

| 寫入點 | 位置 | 手術方式 |
| --- | --- | --- |
| `set_board_rank` | crates/speclink-core/src/model.rs | 逐行掃描、第一條原位代換、缺則尾端補行 |
| in-progress `add` | crates/speclink-core/src/inprogress.rs | 三行尾端 `push_str` |
| in-progress `remove` | crates/speclink-core/src/inprogress.rs | 過濾三個 `started_*` 行 |
| `run_claim` | crates/speclink-core/src/command/mod.rs | 兩行尾端 `push_str` |
| 站別章 `write_stamp` | crates/speclink-core/src/station.rs | `strip_stamp_lines` 剝五個鍵（含縮排區塊）再追加 |
| `link` | crates/speclink-core/src/discuss.rs | `from_discussion` 逗號累加（`push_str` 或 `replacen`） |
| `stamp_restale` | crates/speclink-core/src/discuss.rs | `restale_from` 逗號累加 |
| `clear_restale` | crates/speclink-core/src/discuss.rs | `restale_from` 縮清單或整行 `replacen` 刪除 |

每處都自己處理「行尾有沒有換行」（四處 `if !meta.ends_with('\n')`）、「原位還是追加」、「縮排區塊」、「逗號清單的 split／trim／filter／join」（三個鍵同一語意、三處各寫一遍），而且每處都先「讀原文 → `from_text` 或 `check_meta_text` fail-closed」再動刀——這段前奏也是八份。三套不同做法（原位代換、區塊剝除、`replacen` 累加）並存，下一個 meta 欄位又要發明第四套。

刀一已立好逐行編輯原語 `KeyLines`（crates/speclink-core/src/keylines.rs），並用討論側四個寫入動詞驗證過形狀；本 change 是它的第二個消費者，用八個真實案例驗證 `set`／`remove`／逗號清單的介面夠不夠——若原語形狀有錯，這裡最早發現、修正成本最低。這是討論 improve-lifecycle-layer 結論三刀中的刀二（候選 5），零行為變化；刀三（封存守門鏈收進 `archive()`＋站別門面刪除）在本 change 封存後立案。

**目標使用者與情境**：透過 AI 代理跑 SDD 的開發者。受影響的 workflow 階段是所有會改 change meta 的動詞——apply 開工（in-progress add）、開工撤銷（in-progress remove）、看板排序（board rank）、團隊模式的 claim、review／verify 蓋章、`discuss link`／`seal`／`conclude` 的 change 側鏈與 restale 旗標。

## What Changes

- **`KeyLines` 長出 change meta 需要的四個能力**（crates/speclink-core/src/keylines.rs）：`whole(text)` 建構子（整份文件為區域，change meta 是不帶 `---` 圍欄的純 YAML 映射；尾行缺換行時插入前先補）、`push_list(key, item) -> Result<()>`（逗號清單累加去重）、`remove_list(key, item) -> Result<()>`（縮清單，清空即整行移除）、`set_block(key, lines)`（移除既有同名鍵與區塊後，在區域尾端寫 `key:` 行＋呼叫端給定的縮排續行；站別章的 `*_scope` 區塊需要它）。既有 `frontmatter`／`get`／`list`／`set`／`remove`／`text` 語意不動。
- **新增 `model::edit_meta(store, name, |m: &mut KeyLines| …) -> Result<Option<T>>`**：四步——讀原文（不存在回 `Ok(None)`）、`check_meta_text` fail-closed 解析（壞檔回 `MetaError`、不落檔）、套用閉包、回寫（`text()` 與原文相同時不寫，冪等路徑零寫入）。
- **七個寫入點改走 `edit_meta`**：`set_board_rank`、in-progress `add`／`remove`、`run_claim`、`link`、`stamp_restale`、`clear_restale`。各自的前置守衛（名稱單一路徑段、找不到 change 的訊息、claim 的 ownership 與 actor 檢查、restale 對壞 meta 的跳過）與凍結字串一字不動，只有「讀→解析→手術→寫」四步縮成 `edit_meta` 加一到三個 `set`／`remove`／`push_list`／`remove_list` 呼叫。
- **站別章 `write_stamp` 直接用 `KeyLines::whole`**：`strip_stamp_lines` 刪除，改為五個鍵 `remove`＋四個純量 `set`＋一個 `set_block`；不走 `edit_meta`，因為章的 meta 原文已在 `stamp_gate` 讀過並驗過，且「先刪工單再寫章」的順序契約要求寫入前插一步 `delete_artifact`。輸出位元組與今天相同（章永遠落在檔尾）。
- **不動**：`new change` 的樣板建檔（從零組整份文字，沒有既有內容要保留）、`ChangeMeta` 的 serde 結構與讀取、`archive()` 對 archived meta 的 `archived_*` 追加（封存後的文件，不在「編輯在途 meta」的八處之列）、討論側的 `DiscussionHead`。

**相容性影響**：零行為變化。所有 CLI 人眼輸出與 `--json` 逐位元不變；八處寫出的 meta 位元組與今天相同（既有 `*_preserves_existing_fields_verbatim` 一族測試不改一字為回歸網）；remote bridge 的 staged commit 比對原文，寫出相同位元組即不受影響。唯一可觀察差異只在手寫出重複鍵的壞檔：`set_board_rank` 今天只換第一條、留下重複；改走 `KeyLines::set` 後其餘重複鍵一併丟掉（討論側已是這個語意，對正常檔零差異）。不涉及 CLI 子指令、旗標、stdin、exit code；不涉及設定欄位；不涉及生成技能；golden 零變動。

## Non-Goals

（本 change 建 design.md，範圍排除與已否決做法記錄在 design 的 Goals／Non-Goals。）

## Capabilities

### New Capabilities

（none）——step 3 掃描：change-lifecycle（in-progress 標記、claim、封存）、review-station／verify-station（章的欄位）、discussion-docs（link／seal／restale）、board-card-order（board_rank）已涵蓋這八個寫入點的全部可觀察行為；本 change 只換實作、不加能力。

### Modified Capabilities

（none）——零行為變化，沒有任何要求的字面需要改。

## Impact

- Affected specs: 無 delta（零行為變化）
- Affected crates／apps: speclink-core（唯一）
- Affected code:
  - New: （無新檔）
  - Modified: crates/speclink-core/src/keylines.rs（`whole`／`push_list`／`remove_list`／`set_block`）、crates/speclink-core/src/model.rs（`edit_meta`、`set_board_rank`）、crates/speclink-core/src/inprogress.rs（`add`／`remove`）、crates/speclink-core/src/command/mod.rs（`run_claim`）、crates/speclink-core/src/station.rs（`write_stamp`、刪 `strip_stamp_lines`）、crates/speclink-core/src/discuss.rs（`link`／`stamp_restale`／`clear_restale`）
  - Removed: （無檔案刪除；刪除的是 `strip_stamp_lines` 函式與八處手寫的前奏）
- 測試面：八處既有測試不改斷言；新增 keylines.rs 四個能力的單元測試、model.rs 的 `edit_meta` 測試（不存在回 None 零寫入、壞檔回 MetaError 零寫入、不變零寫入、變更一次寫入，以 `TestStore::meta_writes` 計數斷言）。
