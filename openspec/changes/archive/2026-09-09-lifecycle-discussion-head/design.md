## Context

討論記錄 `openspec/discussions/<slug>.md` 的 frontmatter 目前由 `crates/speclink-core/src/discuss.rs` 內散在各動詞的字串手術維護：

- `mark_promoted`（promote、`new change --from-discussion`、seal 三條路徑的共同落點）：先對**整份文字**做 `replacen("status: open" 或 "status: concluded" → "status: promoted")`，再以 `replacen("status: promoted\n", …)` 插入 `promoted_to` 行；只有「新名字累加成功」才用 `set_frontmatter_line` 清 `hold`。
- `unlink_discarded`（discard 的解鏈）：三處 `replacen` 縮清單、刪 `promoted_to` 行、回退 status。
- `conclude`：一處 `replacen("status: open" → "status: concluded")`，`hold` 走 `set_frontmatter_line`。
- `set_board_rank`：走 `set_frontmatter_line`。
- 讀取側五支文字探測：`frontmatter_value`（六個欄位各讀一次）、`promoted_to_in`、`concluded_in`、`held_in`、`board_rank`；`DiscussionInfo` 被凍結（`discuss list --json` 逐位元不變），所以 `promoted_to`、`discussion_concluded`、`discussion_held`、`board_rank` 四支公開函式各自讀檔，desktop `entry()` 每張卡額外讀 2 次、`board_sorted_active` 再讀 1 次；speclink-host 的 `bridge::discussions_extras` 自己抓原文再呼叫 `promoted_to_in`／`concluded_in`，server route 再把結果拼回 DTO。

**這個缺陷在本 change 立案當下就發作了一次**：對討論 improve-lifecycle-layer 執行 `speclink new change --from-discussion` 時，`mark_promoted` 的 `text.contains("status: open")` 命中的是記錄**內文**第 48 行（候選 1 的問題描述裡引用了這個字串），frontmatter 的 `status: concluded` 反而沒被改，`promoted_to` 也沒寫進去（`replacen("status: promoted\n")` 找不到錨點、靜默失敗）。內文已手動修回；這正是「整份文字 replacen」與「只認 frontmatter」兩套寫法並存的實害。

第二個問題是「討論何時結束生命」的判準有兩份公式：`conclude` 的閉環步把 `c.meta_error.is_some()` 視為「仍引用」（fail-closed）；`archive()` 的討論隨行封存段只看 `c.meta.from_discussions()`，meta 損壞的在途變更被當作不引用，討論會被掃進封存區。規格 `discussion-docs` 對 link／seal 的壞 meta 已明定拒絕、對討論記錄讀取失敗已明定「留在途」，archive 側的放行是漏寫不是刻意。

change 的 `.openspec.yaml` 有同型問題（八個編輯型寫入點各自 `push_str`／`replacen`），由刀二處理；本 change 立的原語 `KeyLines` 是兩份文件共用的地基。

## Goals / Non-Goals

**Goals:**

- 討論 frontmatter 的狀態轉移規則只有一份可讀的實作（`DiscussionHead` 的三個方法），寫入只碰 frontmatter 圍欄內，內文撞字串在結構上不可能。
- 讀取側一趟解析：`DiscussionInfo` 由 head 產出、附帶四個不進 JSON 的欄位；desktop／server／host 的三個內部轉接消失，host crate 不再知道 frontmatter 的文字格式。
- 討論收尾三條件（無在途引用、已有結論、無 hold）與封存步只有一份：`close_if_finished`；archive 路徑對壞 meta 對齊為 fail-closed（規格補 scenario）。
- 立一個位元組保留的逐行編輯原語 `KeyLines`，形狀以本 change 的討論側消費者驗證，供刀二的 change meta 八個寫入點沿用。
- 所有 CLI 人眼輸出、`--json`、server wire、desktop payload 逐位元不變；golden 零變動。

**Non-Goals:**

- 不動 change `.openspec.yaml` 的任何寫入點（刀二）；不動封存守門鏈、站別門面（刀三）。
- 不改 `DiscussionInfo` 的 JSON 形狀（四個新欄位標 `serde(skip)`）；不動 speclink-protocol 的 wire DTO 定義。
- 不改 Context／Rounds／Conclusion 的區段函式（`section_body_range`、`replace_section`、`conclusion_body`、`escape_colliding_lines`）；head 只管 frontmatter。
- 不把 status 未知值改為拒絕（手寫文件、讀端容錯是既有方向）。
- 不改 `discuss new` 的樣板建檔（從零組整份文字，沒有既有內容要保留）。
- 已否決做法（討論 Round 2–4）：討論與 change 各自一份型別化回寫；serde 重新序列化（違反兩處明文的位元組保留決策、吃掉手寫註解與鍵序、remote bridge 比對原文）；head 只裝欄位、轉移留在動詞；`DiscussionInfo` 直接加進 JSON；收尾判準兩邊都放寬；把「promoted_to 非空」塞進共同裁決（link 型討論永遠不隨行封存）；共同函式吃 `fail_closed: bool`（參數化保留分歧）。

## Decisions

### D1 KeyLines 落點為獨立模組 keylines.rs

`KeyLines` 放在新模組 `crates/speclink-core/src/keylines.rs`（`lib.rs` 註冊 `pub mod keylines`），不放 `util.rs`（雜項 fs／時間／路徑輔助）也不放 `model.rs`（change meta 的型別）。理由：它是純領域演算法、不碰儲存媒介，且刀二起同時服務討論記錄與 change meta 兩份文件——放在任一方的模組裡都會讓另一方跨模組借用；獨立模組讓兩個消費者對等。約 150 行實作＋單元測試，符合「一檔一概念」。介面只長到本 change 討論側實際用到的形狀（review Round 1 決定）；刀二要改 change meta 時再依真實需求長出來。

### D2 KeyLines 介面與語意

```rust
pub struct KeyLines { /* 每行含自己的行尾；frontmatter 區域索引 [1, end)；closed；eol */ }
impl KeyLines {
    pub fn frontmatter(text: &str) -> Option<KeyLines>;   // 首行不是 --- 回 None
    pub fn get(&self, key: &str) -> Option<&str>;         // 區域內第一條 key: 的值（trim）
    pub fn set(&mut self, key: &str, value: &str) -> Result<()>;
    pub fn remove(&mut self, key: &str);                  // 區塊鍵含縮排續行區塊
    pub fn list(&self, key: &str) -> Vec<String>;         // 逗號清單：split、trim、去空
    pub fn text(&self) -> String;                         // 回寫
}
```

- **區域**：只有 frontmatter（首行 `---` 到下一個 `---` 之間）。原提案的 `Region::Whole`、`push_list`／`remove_list`、`changed` 在本 change 沒有非測試呼叫端，依 CLAUDE.md「不加沒被要求的彈性」於 review Round 1 拿掉；逗號清單的累加／縮清單由 `DiscussionHead` 在 `Vec<String>` 上做、再以 `set` 寫回。
- **鍵的認法**：只認第 0 欄的 `key:` 前綴，與舊 `frontmatter_value`／`set_frontmatter_line`／`strip_stamp_lines` 三者現況一致。
- **`set`**：第一條 `key:` 行原位代換為 `key: value`（沿該行既有行尾），其餘重複鍵丟掉（區塊鍵連同區塊）；缺則補在 frontmatter 尾端（closing `---` 前）——**不是**舊 `mark_promoted` 的「插在 `status:` 行之後」，新 `promoted_to` 行因此落在 frontmatter 最後（head 測試以完整記錄釘住）。兩種失敗都不改任何行：鍵或值含 `\n`／`\r`（換行注入，task 2.3 稽核加的守衛）；鍵不存在而 frontmatter 未閉合（缺尾 `---`）無處可插。
- **純量鍵與區塊鍵**：`key: value`（冒號後有內容）是純量行，`remove` 與丟重複只碰那一行；`key:`（冒號後為空）是區塊鍵，緊接其後的縮排行、第 0 欄的 `- ` 序列項與區塊內的空行一併移除（`strip_stamp_lines` 的規則，刀二 station 章的 `reviewed_scope` 區塊需要它）。純量行之後的空行與 `- ` 條列不屬於它——未閉合 frontmatter 上區域＝整份文件，這條界線就是內文不被吃掉的保證（review Round 1 修正）。
- **逗號清單**（`promoted_to`、刀二的 `from_discussion`／`restale_from` 同一語意）：`list` 為 split(',')、trim、去空。
- **行尾**：每行沿用自己的行尾（LF 或 CRLF）；新插入的行沿用文件多數行尾（沒有換行的單行文件用 LF）。
- **值逐字寫入**：`KeyLines` 不做 YAML 跳脫，跳脫責任在呼叫端。刀一 head 寫的值只有 status 列舉字面、change 名（單一路徑段）、`true`、小寫 ASCII 的 board_rank，皆不需跳脫；刀二凡寫自由文字的呼叫端必須先走 `util::yaml_scalar`。
- **未閉合 frontmatter**：與讀端同樣寬鬆——整檔視為區域，原位代換與移除照做，只有新插一行會失敗（沿 `set_frontmatter_line`），失敗以 `Err` 回報。

### D3 DiscussionHead 只管 frontmatter、讀端永遠成功、回寫沿 None 契約

```rust
pub enum Status { Open, Concluded, Promoted, Unknown(String) }
impl Status { pub fn as_str(&self) -> &str; }   // Unknown 回原字串，投影逐位元保留
pub struct DiscussionHead {
    pub slug: Option<String>, pub topic: Option<String>, pub status: Status,
    pub created: Option<String>, pub created_by: Option<String>, pub kind: Option<String>,
    pub promoted_to: Vec<String>, pub hold: bool, pub board_rank: Option<String>,
    // 私有：KeyLines（frontmatter 區域）或 None（記錄沒有 frontmatter）；解析當下的四欄快照；hold 重述旗標
}
impl DiscussionHead {
    pub fn parse(text: &str) -> DiscussionHead;          // 永遠成功；無 frontmatter → 全預設
    pub fn write_back(&self) -> Result<Option<String>>;  // Ok(None)＝沒有 frontmatter 可錨定；Err＝未閉合、無處插新行
    pub fn promote(&mut self, change: &str) -> bool;     // 見 D4
    pub fn unlink(&mut self, change: &str, has_conclusion: bool) -> Option<Status>;
    pub fn conclude(&mut self, hold: bool);
}
```

- **欄位對映**：`status` 缺席 → `Open`（沿 `info_from_doc` 現況）；`open`／`concluded`／`promoted` 三個字面對映三值；其餘字面 → `Unknown(原字串)`，讀端不拒絕。`hold` 只有字面 `true` 為真（沿 `held_in`）。`kind` 與 `board_rank` 空值正規化為缺席（沿現況）。`promoted_to` 走 `KeyLines::list`。
- **回寫**：`write_back` 只把**與解析當下不同**的可寫欄位同步回 `KeyLines`——`status` 以 `as_str` 寫、`promoted_to` 空則 `remove` 否則 `set` 為「, 」串接、`hold` 真則 `set("hold","true")` 否則 `remove`、`board_rank` 有則 `set`、其餘五欄唯讀不寫——然後回 `text()`。沒動到的欄位一個位元都不碰：`set_board_rank` 不會順手補 `status: open`、刪手改的 `hold: false`、重排 `promoted_to` 的空白（review Round 1 修正）。例外是 hold 的「重述」：`promote`（新名字）與 `conclude` 對 hold 是明確意圖，即使布林值沒變（`hold: yes` 讀成 false 再 `conclude(false)`）也把任何 `hold:` 行整行移除，沿今天 `set_frontmatter_line(None)` 的語意。記錄沒有 frontmatter 時回 `Ok(None)`，呼叫端沿今天的 None 契約決定：`conclude --hold`／`set_board_rank` 回既有錯誤，其餘動詞保留原文不落檔。frontmatter 未閉合而要新插的行無處可插時回 `Err`——與「沒有 frontmatter」分開，四支動詞一律 `?` 往上丟（promote／seal 在這種手寫記錄上大聲失敗、不再靜默回成功）。為了讓這個失敗不留下半成品，`mark_promoted` 拆出乾跑步 `promoted_text(store, slug, change) -> Result<Option<String>>`（做同一個 `promote` 轉移與 `write_back`、不落檔）；`mark_promoted` 本身是「乾跑＋寫檔」的薄殼，`seal` 仍直接呼叫它（review Round 2 修正）。
- **未知 status 的轉移**：`Unknown` 在三個方法裡視同 `Open`（promote → Promoted、conclude → Concluded）。今天的 `replacen` 對未知字面是靜默不改；改為視同 open 是對手寫壞值的收斂，規格未定義此輸入、不補 scenario，但在 head 的單元測試釘住。

### D4 三條轉移規則住在 head 方法上

- `promote(change) -> bool`：status 由 Open／Concluded／Unknown 轉 Promoted（已是 Promoted 則不動）；`promoted_to` 去重累加；**只有新名字才清 hold**（回 true）；名字已在清單則 hold 不動、回 false。這是今天 `mark_promoted` 的全部語意（含「再 seal 一個已在清單的 change 不是轉出、不清 hold」），差別只在寫入只碰 frontmatter。
- `unlink(change, has_conclusion) -> Option<Status>`：change 不在清單 → `None`（冪等、呼叫端不落檔）；移除後仍有名字 → status 保持 Promoted、回 `Some(Promoted)`；清單清空 → 移除 `promoted_to` 行、status 回退為 `has_conclusion ? Concluded : Open`、回 `Some(回退值)`。`has_conclusion` 由呼叫端以 `conclusion_body(&text).is_some()` 提供，head 不讀內文。
- `conclude(hold)`：status Open／Unknown → Concluded（Promoted 保持 Promoted、Concluded 不動）；`hold` 依參數設或清。
- 呼叫端統一形狀：`let mut head = DiscussionHead::parse(&text); head.xxx(…); match head.write_back()? { Some(t) => store.write_live_discussion(slug, &t), None => … }`。`set_board_rank` 同形（設 `board_rank` 後回寫；None → 既有錯誤訊息「has no frontmatter — cannot set board rank」）。`unlink_discarded` 在 `None`（沒落檔）時回 `Ok(None)`，不回報一個檔案裡不存在的狀態。新插的 `promoted_to` 行落在 frontmatter 尾端（見 D2）。
- 先算討論側、後建 change：`discuss::promote` 與 `new change --from-discussion`（`command::run_new_change`）都先呼叫 `promoted_text` 取得回寫文字，再建 change 目錄與 proposal，最後才把那份文字落檔。記錄無處插行時在任何東西落地前就失敗，重跑不會撞「already exists」的半成品；`newcmd::new_change` 不碰討論記錄，先讀後寫與舊順序讀到同一份文字。

### D5 DiscussionInfo 以 serde(skip) 欄位吸收四個獨立讀取

`DiscussionInfo` 新增：

```rust
#[serde(skip)] pub concluded: bool,
#[serde(skip)] pub head: DiscussionHead,   // promoted_to／hold／board_rank 都在這裡
```

- 原提案是四個獨立欄位（`promoted_to`／`concluded`／`held`／`board_rank`）；review Round 1 指出前三者與 `DiscussionHead` 的同名欄位是同一組資料的第二份宣告，改為直接內嵌 head（`DiscussionHead`／`Status` 因此 `derive(Default)`，供反序列化與 remote 端建構）。
- `info_from_doc` 改為：`let head = DiscussionHead::parse(&doc.text)`，投影欄位從 head 取（`slug`／`topic` 缺席回退 `doc.slug`、`created` 缺席回空字串、`status` 用 `as_str`），`concluded = conclusion_body(&doc.text).is_some()`（同一份已握有的全文，不再讀檔），`rounds` 與 `path`／`archived` 不變，`head` 整個帶著。
- `Serialize` 端 skip → `discuss list --json`、`discuss show --json`、`discuss new --json` 逐位元不變（golden 與 CLI 測試為回歸網）。`Deserialize` 端 skip 取 `Default`，remote 解碼不受影響；`crates/speclink-cli/src/verbs/discuss.rs` 的 `to_discussion_info`（wire → core 型別）以 `DiscussionHead::default()` 建 head、只回填 wire 的 `promotedTo`，`concluded` 取 wire 的 `concluded.unwrap_or(false)`（wire 不帶 hold／board_rank，protocol 不動）。
- 三個內部轉接移除：desktop `entry()` 直接用 `info.head.promoted_to`／`info.concluded`，`board_sorted_active` 直接用 `info.head.board_rank`（每卡讀檔 4 次降為 1 次）；server 的 `discussion_dtos`（原名 `discussion_dtos_with_extras`，已不撈 extras 故改名）對每個 info 直接填 `dto.promoted_to = info.head.promoted_to`、`dto.concluded = Some(info.concluded)`——不再 `spawn_blocking`、不再 materialize bridge view；host `bridge::discussions_extras` 整支刪除。server 的值來源改變但值不變：原本 bridge 讀的就是同一份記錄，`remote_verb_parity` 測試釘住。
- 公開讀取函式：`promoted_to(store, slug)` 簽名不變、改由 `DiscussionHead::parse` 實作（trace.rs 仍呼叫）。`board_rank(store, slug)`、`discussion_concluded`、`discussion_held` 三支在本 change 之後沒有跨模組呼叫端（desktop 改讀 head、archive 改走 `close_if_finished`），依 CLAUDE.md 死碼規則刪除；`close_if_finished` 直接用 `conclusion_body` 與 `DiscussionHead::parse(..).hold` 判定（review Round 1 決定）。

### D6 close_if_finished 收三條件與封存步、archive 路徑 fail-closed

```rust
/// 三條件皆成立才封存：無在途變更引用（meta 損壞的變更視為仍引用）、已有結論、無 hold。
/// Ok(Some(file))＝已封存；Ok(None)＝條件不成立或無 live 記錄；Err＝封存步失敗。
pub fn close_if_finished(store: &dyn Store, slug: &str) -> Result<Option<String>>
```

- 引用檢查：`model::list_changes(store).iter().any(|c| c.meta_error.is_some() || c.meta.from_discussions().iter().any(|s| s == slug))`——即今天 conclude 側的公式。
- `conclude` 的閉環步改為：`if !head.promoted_to.is_empty() { match close_if_finished(store, slug) { Ok(m) => auto_archived = m.is_some(), Err(e) => closing_error = Some(e.to_string()) } }`。「promoted_to 非空」是 conclude 的觸發條件、不進共同裁決（link 型討論的 promoted_to 為空，靠 archive 路徑收尾）；`closing_error` 包裝不變（結論寫入不回滾）。
- `archive()` 的討論段改為：對 `change.meta.from_discussions()` 的每個 slug 呼叫 `close_if_finished`，`Ok(Some(file))` 收進 `archived_discussions`，`Ok(None)` 與 `Err` 皆跳過（今天 `.ok().flatten()` 就是吞掉封存步錯誤——變更本身已封存，討論封存步失敗不讓變更封存失敗）。「排除自己」不需參數：`archive()` 呼叫此段時變更已搬走、`list_changes` 看不到它。
- **行為變化**：archive 路徑對 meta 損壞的在途變更改為視同仍引用——討論留在途、不出現在隨行封存清單、變更照常封存。規格 `discussion-docs`「討論以 link 動詞併入既有變更」要求 MODIFIED、補 scenario「在途變更 meta 損壞時討論不隨變更封存」。
- 寫入順序與失敗狀態：`conclude` 維持「先寫結論（含 hold）、再 restale、再閉環封存」；`archive()` 維持「變更搬走、蓋章、再逐討論封存」——任一討論封存步失敗，變更已封存、該討論仍在途，下次任一引用它的變更封存或 `discuss archive` 可收尾（與今天相同）。

### D7 刪除五支文字探測函式與全部 frontmatter replacen

`frontmatter_value`、`set_frontmatter_line`、`promoted_to_in`、`concluded_in`、`held_in` 刪除，`board_rank(store, slug)`、`discussion_concluded`、`discussion_held` 也一併刪除（見 D5）；`discuss.rs` 非測試碼內對 frontmatter 的 `replacen`（`mark_promoted` 三處、`unlink_discarded` 三處、`conclude` 一處）刪除。`stamp_restale` 讀 `promoted_to` 改用 head；`archive_discussion` 讀 `created` 改用 head。`discuss.rs` 內對 change meta 的 `replacen`（`link` 的 `from_discussion`、`stamp_restale`／`clear_restale` 的 `restale_from`）**保留不動**——那是刀二的範圍。

### D8 測試策略：既有斷言不動、新增落在三處

- `discuss.rs`、`archive.rs`、desktop `discussions.rs`、server 路由與 `remote_verb_parity` 的既有測試不改斷言字串，只有呼叫的函式名跟著改（`held_in`／`discussion_held` 改為 `DiscussionHead::parse(..).hold`、`board_rank(store, slug)` 改為 `info_from_doc(..).head.board_rank`）。唯一例外：`mark_promoted_keeps_the_hold_flag_when_promoted_to_did_not_land` 釘的是舊 `replacen("status: promoted\n")` 在 CRLF 記錄上落空的既知缺口（測試自己註明不背書），本 change 修好那個缺口、前提消失，改寫為 `mark_promoted_lands_promoted_to_on_a_crlf_record_and_clears_the_hold`，釘住 CRLF 記錄 promoted_to 沿 CRLF 落地、hold 清除。
- 新增：`keylines.rs` 單元測試（`get`／`set` 原位與尾補／重複鍵丟（區塊鍵連區塊）／`remove` 純量鍵只刪一行、區塊鍵含縮排區塊與 `- ` 序列項／`list`／CRLF 沿用／未閉合 frontmatter 的插入拒絕／換行注入拒絕）；`discuss.rs` head 測試（三個轉移方法的規則表：promote 新名清 hold、舊名不清、unlink 回退 concluded 或 open、Unknown 視同 open、無 frontmatter 回 None；以及「內文含 `status: open` 字串時 promote 只改 frontmatter」——本 change 立案時發作的那個案例；`write_back` 只寫有改動的欄位、未閉合插入回 `Err`）；`archive.rs` 一條「meta 損壞的在途變更擋住隨行封存」（對應新 scenario）。
- golden：`cargo test -p speclink-core --test it render_golden::` 預期零變動；CLI `--test it` 的 discuss 家族與 `remote_verb_parity` 全綠。

## Implementation Contract

**可觀察行為（使用者／呼叫端）**

- `speclink discuss list --json`、`discuss show --json`、`discuss new --json`、`discuss conclude`（含 `--hold`、`--json`）、`discuss promote`／`seal`／`link`、`new change --from-discussion`、`discard`、`archive` 的人眼輸出與 `--json` payload 逐位元不變。
- 唯一變化：`speclink archive <change>` 時，若另有在途變更的 `.openspec.yaml` 存在但解析失敗，該變更的來源討論不隨行封存（留在 `openspec/discussions/`、不出現在隨行封存輸出），變更本身照常封存、exit code 0。
- 附帶收斂（規格已如此要求、今天的實作違反）：promote／seal／`new change --from-discussion` 對內文含 `status: open` 或 `status: concluded` 字串的記錄，只改 frontmatter 的 `status` 與 `promoted_to`，內文逐位元不變。
- 第二個附帶收斂：CRLF 行尾的記錄，promote 型動詞現在真的寫入 `promoted_to`（沿 CRLF）並清 `hold`；舊版的 `replacen("status: promoted\n")` 在 CRLF 上落空、promoted_to 從未落地。新插的 `promoted_to` 行落在 frontmatter 尾端（closing `---` 前），不再緊接 `status:` 之後。
- 第三個可觀察差異（規格未定義的輸入）：未閉合 frontmatter（缺尾 `---`）且尚無 `promoted_to` 行的手寫記錄，`discuss promote`／`new change --from-discussion`／`seal` 以非零 exit 失敗（舊版靜默不寫、回成功）；promote 與 `new change --from-discussion` 在失敗前不建 change 目錄、不寫 proposal，記錄逐位元不變。
- desktop 看板討論列表與 server `GET /discussions` 的 payload 值與形狀不變。

**介面與資料形狀**：見 D2（`KeyLines`）、D3／D4（`DiscussionHead`、`Status`）、D5（`DiscussionInfo` 四個 `serde(skip)` 欄位）、D6（`close_if_finished`）。刪除的公開項：`discuss::promoted_to_in`、`discuss::concluded_in`、`discuss::board_rank`、`discuss::discussion_concluded`、`discuss::discussion_held`、`speclink_host::bridge::discussions_extras`。保留簽名的公開項：`discuss::promoted_to`、`set_board_rank`、`mark_promoted`、`unlink_discarded`、`conclude`、`archive_discussion`。新增的公開項：`discuss::promoted_text(store, slug, change) -> Result<Option<String>>`（`mark_promoted` 的乾跑，見 D3／D4）、`discuss::close_if_finished`（D6）、`discuss::DiscussionHead`／`Status`（D3）、`keylines::KeyLines`（D2）。

**失敗模式**

- `KeyLines::set` 在未閉合 frontmatter 上新插一行 → `Err`，不改任何行；鍵或值含 `\n`／`\r` → `Err`，不改任何行（換行注入守衛）。
- `DiscussionHead::write_back` 在無 frontmatter 記錄上 → `Ok(None)`；`conclude --hold` 轉為既有錯誤「discussion '<slug>' has no frontmatter — cannot hold it live」、`set_board_rank` 轉為既有錯誤「has no frontmatter — cannot set board rank」，`mark_promoted` 不落檔、成功返回，`unlink_discarded` 回 `Ok(None)`。未閉合 frontmatter 而新行無處可插 → `Err`，四支動詞原樣往上丟（非零 exit、記錄不變）；`promote` 與 `new change --from-discussion` 因為先算 `promoted_text` 再建 change，失敗時 change 目錄與 proposal 都不落地。
- `close_if_finished` 封存步失敗 → `Err`；conclude 端進 `closing_error`（結論不回滾、非零 exit），archive 端跳過該討論（變更已封存）。
- 討論記錄讀取失敗 → `close_if_finished` 回 `Ok(None)`（留在途，沿規格）。

**驗收條件**

1. `cargo test -p speclink-core`（含 `--test it`）全綠，golden 零 diff。
2. `cargo test -p speclink-host -p speclink-server -p speclink-cli --test it`（discuss 家族、`remote_verb_parity`）全綠。
3. `cargo test -p speclink-desktop-core` 全綠。
4. `grep -n 'frontmatter_value\|set_frontmatter_line\|promoted_to_in\|concluded_in\|held_in\|discussions_extras' crates apps --include='*.rs'` 零命中；`discuss.rs` 非測試碼內 `replacen(` 只剩 change meta 三處（`from_discussion`、`restale_from`）。
5. 新 scenario 對應的 `archive.rs` 測試：在途變更 meta 壞掉時 `archived_discussions` 為空、討論仍 live。
6. 手動：對本 change 的來源討論 improve-lifecycle-layer（內文第 48 行含 `status: open` 字串）跑一次 promote 型動詞後，內文逐位元不變、只有 frontmatter 改動。

**範圍邊界**

- In：`keylines.rs`（新）、`lib.rs`、`discuss.rs`、`archive.rs` 討論隨行封存段、`speclink-host/src/bridge.rs`、`speclink-server/src/routes.rs` 的 `discussion_dtos`（原名 `discussion_dtos_with_extras`）、`speclink-cli/src/verbs/discuss.rs` 的 `to_discussion_info`、`apps/desktop/core/src/discussions.rs`、`discussion-docs` delta。
- Out：change `.openspec.yaml` 的任何寫入點（含 `discuss.rs` 內的 `link`／`stamp_restale`／`clear_restale`）、`archive()` 的守門鏈、`review.rs`／`verify.rs`／`station.rs`、speclink-protocol、`discuss new` 建檔、討論區段函式。

## Risks / Trade-offs

- [golden 或 CLI 測試意外變動] → `DiscussionInfo` 新欄位全標 `serde(skip)`；先跑 `render_golden::` 與 discuss 家族 CLI 測試建立基準，改完比對；任何 diff 都是缺陷不是「順手更新」。
- [Windows CRLF 記錄] → `KeyLines` 每行沿自己的行尾、新行沿多數行尾；單元測試含 CRLF 案例；既有 `conclude_with_hold_follows_the_record_line_ending` 測試不動。
- [remote parity：server 的 `promotedTo`／`concluded` 值來源改變] → 值來自同一份記錄（`list_discussions`／`list_archived` 各自讀自己那側，reused slug 不會互相答錯）；`remote_verb_parity` 與 server 路由測試釘住；移除 `spawn_blocking` 後路由變成純同步組裝，無 async 邊界變化。
- [無 frontmatter 的 pre-scaffold 記錄] → `write_back` 回 `None`，各動詞沿今天 `set_frontmatter_line` 的 None 契約；`conclude_with_hold_rejects_a_record_without_frontmatter` 不動。
- [Unknown status 視同 open 是對壞值的行為收斂] → 規格未定義該輸入；head 單元測試釘住；不補 scenario。
- [刀二會用同一原語改 change meta，介面若不夠用] → `remove` 已含縮排區塊（station 章寫入的需求）、逗號清單三式已含（`from_discussion`／`restale_from` 的需求）；刀二 propose 期若仍缺，補在原語上而非各寫入點。
- [`close_if_finished` 每個 slug 各掃一次 `list_changes`] → 與今天相同的成本（今天兩處也各掃一次）；多來源討論的變更是少數。
- [跨平台] → 純文字演算法、不碰路徑；`archive_discussion` 的檔名與同日撞名解法不動。

## Migration Plan

無資料遷移：記錄格式不變、JSON 不變。部署即生效；回滾即還原程式碼。唯一的行為變化（壞 meta fail-closed）對既有使用者的處置：修好那份 `.openspec.yaml` 後，再封存任一引用該討論的變更，或直接 `speclink discuss archive <slug>`。

## Open Questions

（無。`KeyLines` 落點與命名已由 D1 定案；刀三是否拆刀留待刀三 propose 期。）
