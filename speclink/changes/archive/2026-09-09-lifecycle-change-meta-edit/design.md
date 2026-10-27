## Context

刀一（lifecycle-discussion-head，已封存）立了 `KeyLines`：對「頂層 `key: value` 行」做原位手術、其餘位元組逐字保留的逐行編輯原語。落地的介面只長到討論側實際用到的形狀：`frontmatter(text) -> Option<KeyLines>`（區域＝首行 `---` 到下一個 `---`）、`get`、`list`、`set`（第一條原位代換、重複鍵連區塊丟、缺則補在 closing `---` 前）、`remove`（區塊鍵連縮排續行、第 0 欄 `- ` 序列項與區塊內空行一起移除）、`text`。模組文件明寫「刀二要改 change meta 時再依真實需求長出來」。

change meta `.openspec.yaml` 與討論 frontmatter 的差別有三：不帶 `---` 圍欄（整份文件就是映射）、有縮排區塊鍵（`reviewed_scope:`／`verified_scope:` 之下的 `- path` ／ `hash` 序列）、以及同一份文字被 remote bridge 的 staged commit 逐位元比對。八個編輯型寫入點的共同前奏是「`read_change_meta` → `ChangeMeta::from_text` 或 `check_meta_text` fail-closed → 手術 → `write_change_meta`」，各自的差異只在守衛順序與凍結訊息。

現況三套手術做法：`set_board_rank` 逐行掃描原位代換、`strip_stamp_lines` 依鍵前綴剝行含區塊、`link`／restale 以 `replacen` 對整段值做累加或刪行。四處各自補尾端換行；三個逗號清單鍵（`from_discussion`、`restale_from`、討論側 `promoted_to`）的 split／trim／filter／join 寫了三遍（討論側那份刀一已收進 `KeyLines::list`）。

## Goals / Non-Goals

**Goals:**

- change meta 的編輯只有一條路：`edit_meta` 四步＋`KeyLines` 的純量／清單／區塊操作；八處手寫前奏歸一。
- `KeyLines` 用八個真實案例驗證形狀，補上 change meta 需要的四個能力；介面錯誤在這一刀暴露，不留到刀三。
- 零行為變化：八處寫出的位元組與今天相同，凍結訊息、守衛順序、冪等零寫入語意全部保留。

**Non-Goals:**

- 不改 `ChangeMeta` 的 serde 結構、讀取路徑、`list_changes`／`find_change`。
- 不動 `new change` 的樣板建檔（`newcmd.rs` 的 `format!` 組整份新檔）與 `archive()` 對已封存 meta 的 `archived_*` 追加。
- 不動封存守門鏈、`run_archive`、站別門面 `review.rs`／`verify.rs`（刀三）。
- 不給 `ChangeMeta` 加完整 setter 集合（`set_started`、`set_claimed`……）——每個欄位族一支 setter 等於把八個寫入點的形狀搬進 model.rs，`edit_meta` 閉包已足夠（討論 Round 7 已否決）。
- 不做只改三個逗號清單點的半套（原語只吃一半、前奏仍是五份；Round 7 已否決）。
- 不改 remote／server／desktop 任何一端：它們經 `Command` 或 host 呼叫同一支 core 函式，沒有平行實作。

## Decisions

### D1 KeyLines 長出 whole 區域與清單／區塊三式

```rust
impl KeyLines {
    pub fn whole(text: &str) -> KeyLines;                                    // 整份文件為區域
    pub fn push_list(&mut self, key: &str, item: &str) -> Result<()>;        // 累加去重；已含不改
    pub fn remove_list(&mut self, key: &str, item: &str) -> Result<()>;      // 縮清單；清空即 remove
    pub fn set_block(&mut self, key: &str, body: &[String]) -> Result<()>;   // 區塊鍵：remove 後在區域尾端寫 key: 行＋續行
}
```

- **`whole`**：區域＝`[0, lines.len())`，永遠「已閉合」（尾端可插）。插入時若最後一行沒有行尾，先給它補一個（沿今天四處 `if !meta.ends_with('\n') { push('\n') }` 的語意）；空文件插入得到 `key: value\n`。新行行尾沿文件多數行尾、平手或無換行用 LF（沿刀一）。內部以 `start` 欄位（frontmatter 為 1、whole 為 0）區分兩種區域，不引入 `Region` 列舉——兩個建構子已足夠命名。
- **`push_list`**：`list(key)` 已含 `item` → 不改；否則 `set(key, 既有項＋item 以「, 」串接)`（既有為空清單時就是單值）。既有值的原始間距不保留、統一為「, 」——今天三處累加寫的也是 `"{existing}, {item}"`，既有段落逐字保留；差異只在手寫成 `a,b`（無空白）的清單，累加後會變成 `a, b, c`，`ChangeMeta` 讀端 trim 後語意相同。既有值被引號包住（手寫 `"a, b"`）→ `Err` 不改：引號夾在清單中間會讓整份文件解析失敗，今天的 `replacen` 對這種檔是靜默不匹配。回傳不帶 bool：三個呼叫端都不需要「是否新增」，冪等零寫入由 `edit_meta` 的文字比對承擔。
- **`remove_list`**：`item` 不在清單 → 不改；移除後仍有項 → `set` 為「, 」重組；清空 → `remove(key)`（整行消失，沿今天 `replacen("restale_from: …\n", "")`）。回 `Result<()>`，失敗同 `set`。
- **值逐字讀出、不解引號**：`get` 回 trim 後的原文。走 `yaml_scalar` 寫入的值（身分、工具名）讀回要比對時，呼叫端先過 `util::yaml_unscalar`（`yaml_scalar` 的反向）；change 名與討論 slug 不會被加引號，直接比對。
- **`set_block`**：`remove(key)`（連既有區塊）後在區域尾端寫 `key:` 行與 `body` 每一行（呼叫端提供含縮排的續行原文，例如 `  - path: x` 與 `    hash: y`；本方法逐行加行尾，不做縮排判斷）。任一行含換行 → `Err` 不改任何行（沿 `set` 的注入拒絕）；frontmatter 未閉合無處可插 → `Err`。
- 既有 `set`／`remove` 的區塊鍵認法（冒號後為空）與續行認法（縮排、`- ` 序列項、空行）已涵蓋 `strip_stamp_lines` 的規則，不改。

### D2 edit_meta 四步與回傳形狀

```rust
/// 讀→fail-closed 解析→套閉包→回寫；Ok(None)＝change 沒有 meta 文件；
/// Err(MetaError)＝文件存在但解析失敗（零寫入）；text() 與原文相同時不寫。
pub fn edit_meta<T>(
    store: &dyn Store,
    name: &str,
    edit: impl FnOnce(&mut KeyLines) -> anyhow::Result<T>,
) -> anyhow::Result<Option<T>>
```

- 落點 `crates/speclink-core/src/model.rs`（`ChangeMeta` 與 `check_meta_text` 的家）。
- 四步：`store.read_change_meta(name)` 為 `None` → `Ok(None)`；`check_meta_text(name, Some(&meta))?`（錯誤型別 `MetaError`，可 `downcast_ref` 分辨）；`let mut m = KeyLines::whole(&meta); let out = edit(&mut m)?;`；`if m.text() != meta { store.write_change_meta(name, &m.text())? }`；`Ok(Some(out))`。
- **「不存在」的政策留給呼叫端**：八處對找不到 change 的反應不同（`add` 靜默 `Ok(false)`、`remove` 與 `set_board_rank` 回錯誤、`stamp_restale` 跳過、`clear_restale` 靜默 `Ok(())`），所以 `edit_meta` 只回 `None`，訊息與 exit 行為由呼叫端沿今天的字面決定。
- **名稱單一路徑段守衛留在呼叫端**：`add`（靜默）、`remove`（loud）、`set_board_rank`（loud）三處各有自己的訊息；`edit_meta` 不重複這道。
- **冪等零寫入由「文字相同不寫」承擔**：閉包不必自己判斷要不要寫；`link` 同一組合重跑、`add` 已蓋章、`clear_restale` 無此 slug 都自然零寫入，`TestStore::meta_writes` 計數釘住。
- 閉包可讀 `store`（借用捕獲）：`remove` 的零工作痕跡守門需要在 fail-closed 解析之後、寫入之前讀 tasks 與 touched，放進閉包保住今天的守衛順序（壞 meta 先報 `MetaError`，再談「有痕跡不得撤銷」）。

### D3 七個寫入點改走 edit_meta、各自守門與訊息不動

| 寫入點 | 前置守衛（不動） | 閉包內容 | `None` 的處置 |
| --- | --- | --- | --- |
| `set_board_rank` | rank 合法、名稱單段 | `m.set("board_rank", rank)` | `bail!("change not found: {name}")` |
| in-progress `add` | 名稱單段→`Ok(false)` | 任一 `started_*` 存在→`Ok(false)`；否則 `set` `started_at`、`started_by`／`started_with`（值過 `yaml_scalar`）→`Ok(true)` | `Ok(false)` |
| in-progress `remove` | 名稱單段→not found | 零工作痕跡守門（`RevertBlocked`）；無 `started_*`→`Ok(false)`；否則 `remove` 三鍵→`Ok(true)` | not found 錯誤 |
| `run_claim` | `supports_ownership`（actor 存在改在閉包內判定：`guard_meta` 已在函式最前面擋下壞 meta，not-found 仍先於缺身分回報，可觀察順序與今天相同） | 缺 actor→`Refusal` 凍結字串；`get("claimed_by")` 過 `yaml_unscalar`、`get("claimed_at")` 做今天的四臂 match（同 holder→`Ok(false)`、他人→`Refusal` 凍結字串、半章→`Refusal` 凍結字串）；`set` `claimed_at`、`claimed_by`（過 `yaml_scalar`）→`Ok(true)` | `NotFound` 凍結字串 |
| `link` | 討論存在且未封存 | `m.push_list("from_discussion", slug)` | `bail!("Change '{change}' not found.")` |
| `stamp_restale` | — | `m.push_list("restale_from", slug)` | `continue`（archived or gone） |
| `clear_restale` | — | `m.remove_list("restale_from", slug)` | `Ok(())` |

- `run_claim` 的閉包錯誤走 `crate::command::Refusal` 型別，經既有 `classify` 映射為 `ErrorCode::Refused`，字串逐字不變；`MetaError` 經 `classify` 映射為 `invalid_config`（今天就是）。
- `stamp_restale` 對壞 meta 的處置是**跳過**（今天：`let Ok(parsed) = from_text else { continue }`）：呼叫端對 `edit_meta` 的 `Err` 做 `downcast_ref::<MetaError>()`，命中即 `continue`，其他錯誤照傳。`flagged` 沿今天語意：不論本次是否新增都推入（已旗標的 change 也回報）。
- `link` 對「`from_discussion:` 存在但值為空」的手寫檔：今天會追加第二條同名鍵（重複鍵、之後 fail-closed），`push_list` 改為原位寫入單值——對壞檔的收斂，正常檔零差異。
- in-progress 對「`started_at:` 存在但值為空」的手寫檔：今天 serde 讀作 `None`，`add` 會追加第二條同名鍵（之後 fail-closed）、`remove` 靜默 `Ok(false)`；改走 `KeyLines::get` 後空值鍵讀作「已存在」，`add` 靜默 `Ok(false)` 不疊寫、`remove` 連同其後的縮排續行一起移除並回 `Ok(true)`——同樣是對壞檔的收斂，正常檔零差異。
- 各處讀 `ChangeMeta` 欄位的判斷（`started_*` 是否存在、`claimed_*` 配對、`from_discussions()` 是否已含）改為 `KeyLines::get`／`list`——與 `ChangeMeta` 的 serde 讀取對同一份文字的看法一致（頂層鍵第 0 欄、逗號清單 trim 去空），既有測試釘住。

### D4 站別章寫入直接用 KeyLines、刪除先於寫入

`write_stamp` 不走 `edit_meta`，理由有二：章的 meta 原文 `gate.raw_meta` 已由 `stamp_gate` 讀過並 `check_meta_text` 驗過（再讀一次會讓守門與寫入看到兩份可能不同的文字）；規格要求「先刪工單再寫章」，`delete_artifact` 必須插在手術與寫入之間，`edit_meta` 的四步不容插隊。改法：

```rust
let mut m = KeyLines::whole(&gate.raw_meta);
for key in [_at, _by, _with, _tasks_total, _scope] { m.remove(&key); }   // 取代 strip_stamp_lines
m.set(&format!("{prefix}_at"), &today)?;
if let Some(actor) = actor { m.set(&format!("{prefix}_by"), &yaml_scalar(actor))?; }
if let Some(tool)  = tool  { m.set(&format!("{prefix}_with"), &yaml_scalar(tool))?; }
m.set(&format!("{prefix}_tasks_total"), &tasks_total.to_string())?;
m.set_block(&format!("{prefix}_scope"), &entries.map(|(p,h)| [format!("  - path: {}", yaml_scalar(p)), format!("    hash: {}", yaml_scalar(h))]).flatten())?;
store.delete_artifact(change, st.doc)?;
store.write_change_meta(change, &m.text())?;
```

- 先 `remove` 五鍵再 `set`，讓章永遠落在檔尾——與今天「剝除後追加」的位元組相同（若改成原位 `set`，重蓋章會留在舊位置，既有比對整份 meta 的站別測試會照出差異）。唯一例外是 CRLF meta：今天章行一律補 LF，改後沿多數行尾寫 CRLF——見 Risks 的 Windows CRLF 條目，屬收斂、無可觀察差異。
- `strip_stamp_lines` 刪除；其縮排區塊規則已在刀一併入 `KeyLines::remove`。
- 寫入順序與失敗狀態不變：刪工單失敗 → 章未寫、工單仍在；寫章失敗 → 工單已刪、章未寫，退回「未檢查」（規格允許的唯一方向）。

### D5 零行為變化的驗證面

- **位元組回歸網**：`inprogress.rs` 的 `add_appends_started_fields_and_preserves_existing_fields_verbatim`／`add_preserves_existing_board_rank_verbatim`／`remove_strips_started_lines_and_preserves_the_rest_verbatim`、`model.rs` 的 `set_board_rank_*` 六條、`discuss.rs` 的 `link_*`／`conclude_restale_*`／`seal_*` 家族、`station.rs`／`review.rs`／`verify.rs` 比對整份 meta 的蓋章測試、`command/mod.rs` 的 claim 測試——全部不改斷言。
- **零寫入斷言**：既有「without_writes」「idempotent」測試繼續用 `TestStore::meta_writes` 或內容比對釘住；`edit_meta` 自己的單元測試補四條（不存在→`None` 且零寫入、壞檔→`MetaError` 且零寫入、閉包不改→零寫入、閉包改→恰一次寫入）。
- **原語單元測試**：`whole` 對尾行缺換行與空文件的插入、`push_list` 去重與空清單、`remove_list` 縮清單與清空刪行、`set_block` 對既有區塊的取代與換行注入拒絕、CRLF 沿用。
- golden：`render_golden::` 零 diff；CLI `--test it` 全綠。

## Implementation Contract

**可觀察行為**：無變化。`speclink` 所有動詞的 stdout／stderr／exit code／`--json` 逐位元不變；八個寫入點寫出的 `.openspec.yaml` 位元組與今天相同；remote／server／desktop 經同一支 core 函式，無平行實作。

**介面**：見 D1（`KeyLines::whole`／`push_list`／`remove_list`／`set_block`）、D2（`model::edit_meta`）。刪除的私有項：`station::strip_stamp_lines`。所有公開函式簽名不變：`model::set_board_rank`、`inprogress::add`／`remove`、`discuss::link`、`command` 的 `Claim` 臂。

**失敗模式**

- `edit_meta`：meta 不存在 → `Ok(None)`；解析失敗 → `Err(MetaError)`，零寫入；閉包 `Err` → 原樣傳出，零寫入；寫入失敗 → 儲存層錯誤傳出（單一寫入，無半套）。
- `KeyLines::set`／`push_list`／`set_block` 對含換行的鍵或值 → `Err`，不改任何行。
- 各動詞對「不存在」與「壞 meta」的訊息與 exit 行為沿今天字面（D3 表）。

**驗收條件**

1. `cargo test -p speclink-core` 全綠；`cargo test -p speclink-core --test it render_golden::` 零 diff。
2. `cargo test -p speclink-cli --test it` 全綠（in-progress、claim、discuss、review／verify 家族）。
3. `grep -rn 'strip_stamp_lines\|fn from_text(Some(&meta))' crates/speclink-core/src` 零命中於 `strip_stamp_lines`；`grep -n 'write_change_meta(' crates/speclink-core/src/*.rs crates/speclink-core/src/command/mod.rs` 非測試碼只剩 `model::edit_meta` 一處與 `station::write_stamp` 一處。
4. `grep -n "ends_with('\\\\n')" crates/speclink-core/src/inprogress.rs crates/speclink-core/src/discuss.rs crates/speclink-core/src/command/mod.rs crates/speclink-core/src/model.rs` 在八個寫入點所在函式內零命中（尾端補行只剩 `KeyLines::whole` 一份）。
5. `TestStore::meta_writes` 在 `edit_meta` 四條單元測試中分別為 0／0／0／1。

**範圍邊界**

- In：`keylines.rs`（四個能力）、`model.rs`（`edit_meta`、`set_board_rank`）、`inprogress.rs`、`command/mod.rs` 的 `run_claim`、`station.rs` 的 `write_stamp`、`discuss.rs` 的 `link`／`stamp_restale`／`clear_restale`，以及各自的測試。
- Out：`newcmd.rs`、`archive.rs`（含 `archived_*` 追加與守門鏈）、`review.rs`／`verify.rs`、`ChangeMeta` 結構、任何 CLI／server／desktop 檔案、規格。

## Risks / Trade-offs

- [站別章重蓋的位元組位置] → D4 明定先 `remove` 再 `set`，章落檔尾；station／review／verify 既有整份 meta 比對測試釘住。
- [`remove` 的 `started_*` 撤銷今天過濾整份文件、`KeyLines::remove` 也是整份（whole 區域）] → 語意相同；`remove_strips_started_lines_and_preserves_the_rest_verbatim` 釘住。
- [守衛順序漂移（例如 `remove` 的痕跡守門跑到 fail-closed 之前）] → 守門放進閉包（D2）；`remove_refuses_*` 與 `add_refuses_on_corrupt_meta_without_writing` 釘住訊息與零寫入。
- [claim 的凍結字串經閉包→`classify` 轉一手] → 閉包用 `Refusal` 型別、字串常量不動；command 層 claim 測試比對訊息。
- [remote bridge 的 staged commit 比對原文] → 寫出位元組相同，且不變時不寫（今天 `link` 冪等也不寫）；`remote_verb_parity` 釘住。
- [Windows CRLF meta] → `KeyLines` 每行沿自己的行尾、新行沿多數；`whole` 的尾端補行沿多數行尾（今天四處一律補 LF——對 CRLF 檔今天會混行尾，改後沿多數是收斂；`ChangeMeta` 讀端對兩種行尾都能解析，無可觀察差異）；補一條 CRLF 單元測試。
- [壞檔（重複鍵、空值鍵）的行為收斂] → 對正常檔零差異；規格未定義該輸入；單元測試釘住新語意，不補 scenario。
- [golden 與 CLI 回歸] → 改碼前先跑一次建立基準；任何 diff 視為缺陷。

## Migration Plan

無資料遷移、無格式變化。部署即生效；回滾即還原程式碼。

## Open Questions

（無。刀三是否拆刀留待刀三 propose 期。）
