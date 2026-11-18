## Context

**詞彙表。** 引擎只有讀取：`LanguageShow` 命令呼叫儲存介面（`speclink-core` 的 `Store` trait）的 `read_language`。線上有 `GET /language`。沒有寫入的命令，`Store` 也沒有寫入詞彙表的方法。

**討論的封存。** `Store` 有 `archive_discussion(slug, created)`：把進行中的記錄搬進封存，名稱是 `<建立日期>-<slug>.md`。反方向沒有方法。`list_archived_discussions` 依封存名稱排序回傳全部封存的記錄。四個動詞在討論已封存時拒絕（`lifecycle/discuss.rs`），訊息寫「move it out of discussions/archive/ to …」。

**儲存介面的實作有四個。** 本機檔案（`speclink-fs`）、儲存接口上的視圖（`speclink-host` 的 `port_view.rs`，寫入收進一次提交）、worktree 覆蓋層（`speclink-host` 的 `worktree.rs`，寫入一律落在主副本）、測試用的記憶體版（`speclink-core` 的 `teststore.rs`）。`Store` 加方法時四個都要實作。

**線上與遠端的 CLI。** 每個引擎命令在 `http/coverage.rs` 要有歸屬（沒有萬用分支）。CLI 連到遠端專案時，各動詞有自己的遠端分支，把回應轉成和本機相同的結果再交給共用的輸出函式。

**技能的原稿。** `assets/skills/discuss.md` 有兩段用「規格在不在這個 checkout」分成兩種寫法：把用詞偏移記進詞彙表（在 checkout 時給檔案路徑，不在時說沒有動詞可寫）、封存錯了的復原（在 checkout 時手動搬檔，不在時請使用者到 Server 處理）。`assets/skills/propose.md` 的最後一刀救援有同樣的兩種寫法。原稿改動要連動內嵌資產的版本號（`workspace/init.rs` 的 `ASSET_VERSION`）、渲染 golden 與資產鎖定檔。

## Goals / Non-Goals

**Goals:**

- 詞彙表可以經動詞寫入，討論可以經動詞從封存搬回；本機、遠端與內嵌宿主三種形態都可用，結果相同。
- 兩個動詞都是引擎命令，自然經過執行前掛鉤與一次提交。
- 技能裡這兩件事不再分「規格在不在這個 checkout」。

**Non-Goals:**

- 不做詞彙表的局部編輯（加一個詞條、改一個欄位）。整份寫回和 artifact 的改寫方式一致，引擎不需要解析詞彙表的結構。
- 不檢查詞彙表的內容格式；它是自由的 Markdown。
- 不做變更（change）的解除封存：封存變更會合併規格，不能單純搬回。
- 不提供「指定搬回哪一份封存」的參數；一律搬回同 slug 最新的一份。
- 不改 Speclink Desktop 的畫面；它沒有編輯詞彙表與搬回討論的操作。
- 搬回不改記錄的內容：不會自動補回 `hold` 這類旗標。

## Decisions

### D1 兩個新的引擎命令：LanguageReplace 與 DiscussRestore

- `Command::LanguageReplace { content }`，穩定名稱 `language-replace`。掛鉤看到的參數不含內文，只有內文的位元組數（`bytes`），比照匯入命令不把內容交給掛鉤的做法。結果 `LanguageReplaceOutcome { created }`。事件 `LanguageReplaced`（名稱 `language-replaced`，沒有額外內容）。
- `Command::DiscussRestore { slug }`，穩定名稱 `discuss-restore`，參數含 slug。結果 `DiscussRestoreOutcome { slug, restored_from }`。事件 `DiscussionRestored { slug }`（名稱 `discussion-restored`）。

執行的步驟：

- `LanguageReplace`：內容去掉頭尾空白後是空的就拒絕（`invalid_argv`，訊息指出內容是空的）；先用 `read_language` 判斷 `created`；再呼叫 `Store::write_language`。內容原樣寫入，不補也不刪結尾換行。
- `DiscussRestore`：slug 先過既有的 slug 檢查；`live_discussion_exists` 為真就拒絕（`refused`）；沒有封存的記錄就回 not-found；否則呼叫 `Store::restore_discussion`。

替代方案：把詞彙表當成一種 artifact，沿用 `new artifact`。否決：artifact 屬於某個變更，詞彙表屬於整個專案，硬套會讓 `--change` 變成必填卻沒有意義。

### D2 儲存介面新增兩個方法，四個實作各自保證原子

`Store` 新增：

- `write_language(&self, content: &str) -> Result<()>`
- `restore_discussion(&self, slug: &str) -> Result<Option<String>>`：把該 slug 封存名稱排序最後的一份搬回進行中，回傳它封存時的名稱；沒有封存的記錄時回傳 `None`。

寫入順序與失敗後的狀態：

- 本機檔案：`write_language` 用既有的原子寫檔（先寫暫存檔再改名）。`restore_discussion` 是一次改名（封存的檔案改名成進行中的檔案）；改名失敗時兩邊都維持原樣。目的檔已存在的情況在命令層先擋掉。
- 儲存接口上的視圖：兩個方法都只把操作排進這次命令的寫入清單，隨命令結束時的一次提交送出。`restore_discussion` 排的是「寫入進行中的鍵」加「刪除封存的鍵」兩筆，放在同一次提交，並各自帶預期版本（進行中的鍵必須不存在、封存的鍵必須是讀到的版本），所以不會出現兩邊都有或都沒有。
- worktree 覆蓋層：比照其他討論寫入，一律轉給主副本。
- 測試用的記憶體版：直接改記憶體裡的表。

替代方案：不加 `restore_discussion`，在命令裡用既有方法組合（讀封存、寫進行中）。否決：`Store` 沒有刪除封存記錄的方法，組合出來會留下兩份。

### D3 動詞的解析與輸出落在共用的 speclink-verbs

- `language` 新增子動詞 `replace`，旗標 `--stdin`（必填）與 `--json`。沒有 `--stdin` 是用法錯誤。
- `discuss` 新增子動詞 `restore <slug>`，旗標 `--json`。
- 輸出函式：`language replace` 的兩種成功行（`✓ Replaced the LANGUAGE document`、`✓ Created the LANGUAGE document`）與 `{ "created" }`；`discuss restore` 的兩行（`✓ Restored discussion: <slug>`、`  From: <名稱>`）與 `{ "slug", "restoredFrom" }`。
- Agent 工具的動詞清單由同一份定義產生，自動帶出兩個新動詞。`language replace` 在工具裡把內容放在 `stdin`，和其他帶 `--stdin` 的動詞相同。

### D4 線上的兩個端點與遠端的 CLI 分支

- `wire_coverage`：`LanguageReplace` → `PUT /language`；`DiscussRestore` → `POST /discussions/{slug}/restore`。
- 請求與回應型別放在 `speclink-protocol`：`LanguageReplaceRequest { content }`、`LanguageReplaceResponse { created }`、`DiscussRestoreResponse { slug, restored_from }`（序列化成 `restoredFrom`）。
- 路由經既有的命令閘道：editor 限定、revision 前置條件、掛鉤、事件都沿用。`PUT /language` 不套用政策鍵的權限檢查。
- 事件歸類（`commit.rs`）：`discussion-restored` 和其他討論事件同一組（`scope` 是 `discussion`、`resourceId` 是 slug）；`language-replaced` 不加進任何一組，落在「其他」，用戶端收到後重新讀取全部。
- `speclink-remote` 的用戶端新增 `replace_language(content)` 與 `restore_discussion(slug)`；CLI 的遠端分支呼叫它們，把回應轉成結果後交給 D3 的輸出函式。

### D5 已封存的拒絕訊息改成指向 restore

`lifecycle/discuss.rs` 四處訊息的後半句由「move it out of discussions/archive/ to continue it／promote it／link it／seal it」改成「run `speclink discuss restore <slug>` to continue it／promote it／link it／seal it」。前半句（`discussion '<slug>' is archived — `）、拒絕的條件與 exit code 不變。CLI 測試裡斷言舊文字的地方同批更新。

### D6 技能原稿改用兩個動詞，並前進資產版本

`assets/skills/discuss.md`：

- 「Vocabulary drift」那一段之後原本分兩種情況的區塊，換成一段不分情況的指示：用 `language show` 讀出全文（沒有文件時從空白開始），加上或修改詞條，用 `language replace --stdin` 整份寫回；使用者拒絕記錄時不寫。
- 「Archived by mistake?」原本分兩種情況的區塊，換成一段：執行 `discuss restore <slug>`，記錄回到進行中，每個動詞都能再用。

`assets/skills/propose.md`：標錯 `--last` 的救援，兩種情況的區塊換成一段：執行 `discuss restore <slug>`。

連動的衍生物照既有的紀律一次做完：`ASSET_VERSION` 前進一版、重新產生渲染 golden 與 `assets.lock`、用新建的 binary 執行 `speclink update` 重新產生本 repo 已安裝的技能檔。原稿的改動要通過既有的守門：技能不得直接讀寫規格資料夾（`skill_verbization`）、強調字的數量上限（`skill_authoring`）、規格路徑只在條件區塊裡（這次是移除條件區塊，不是新增）。

### D7 使用者文件

- 專案能力狀態（中英）：已知缺口表移除「沒有把封存的討論記錄搬回來的動詞」與「沒有寫入詞彙表的動詞」兩列，移到表格下方「已經解決」的清單，並寫上對應的動詞。
- 動詞契約文件（中英）：動詞表新增兩個動詞；端點表新增兩列；事件對照表的討論那一列加入 `discussion-restored`，並在「其他」的說明提到 `language-replaced`；回應範例新增兩個端點。

## Implementation Contract

**行為**

- `speclink language replace --stdin`：把 stdin 整份寫成詞彙表；不存在就建立；空內容拒絕。之後 `language show` 讀到同樣的內容。
- `speclink discuss restore <slug>`：把同 slug 最新的一份封存記錄搬回進行中，內容不變。
- 兩個動詞在本機、遠端、內嵌宿主的人眼輸出逐位元相同。
- 技能 discuss 與 propose 用這兩個動詞，不分規格在不在 checkout。

**介面與資料形狀**

- 引擎命令 `language-replace`（掛鉤參數 `{ bytes }`）與 `discuss-restore`（參數含 slug）；事件 `language-replaced`、`discussion-restored`（內容 `{ slug }`）。
- `Store::write_language(&self, content: &str) -> Result<()>`；`Store::restore_discussion(&self, slug: &str) -> Result<Option<String>>`。
- CLI：`speclink language replace --stdin [--json] [--no-color]`；`speclink discuss restore <SLUG> [--json] [--no-color]`。
- JSON：`{ "created": boolean }`；`{ "slug": string, "restoredFrom": string }`。
- 線上：`PUT /api/speclink/v1/projects/{key}/language`，本文 `{ "content": string }`，回應 `{ "created": boolean }`；`POST …/discussions/{slug}/restore`，回應 `{ "slug": string, "restoredFrom": string }`。

**失敗模式**

- 空內容：`invalid_argv`（HTTP 400 `invalid_argument`），零寫入。
- 已有進行中的同 slug 記錄：`refused`（HTTP 409），零寫入。沒有封存的記錄：not-found（HTTP 404）。
- reader：HTTP 403。掛鉤拒絕：`refused` 與宿主給的原因。
- revision 衝突、離線、認證失效：和其他寫入動詞相同。
- 儲存層在提交時失敗：整筆不寫（D2）。

**驗收**

- `cargo test -p speclink-core`：兩個命令的單元測試、四處訊息的新文字、命令名稱與參數的 golden、渲染 golden 與資產鎖定檔（刻意更新）、技能的兩個守門測試。
- `cargo test -p speclink-fs`：本機檔案的兩個方法。
- `cargo test -p speclink-verbs`：解析與輸出。
- `cargo test -p speclink-host`：兩個端點的情境、事件歸類、對照表、`port_dual_path` 的本機與儲存接口對照（兩個新命令在兩條路徑結果相同）。
- `cargo test -p speclink-cli --test it`：`doc_verbs` 與 `remote_verb_parity` 的新情境。
- `npm --prefix crates/adapters/speclink-node run build` 後 `npm --prefix crates/adapters/speclink-node test`：`host.dispatch` 執行兩個動詞。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過。

**範圍**

- 範圍內：上面列的命令、儲存介面與四個實作、輸出、端點、遠端分支、四處訊息、兩支技能的原稿與衍生物、文件。
- 範圍外：詞彙表的結構化編輯、變更的解除封存、Speclink Desktop、其他技能的內容。

## Risks / Trade-offs

- [`Store` 加方法會讓四個實作都要改，漏一個就編譯不過] → 這是預期的提醒；`port_dual_path` 的對照測試確認本機與儲存接口兩條路徑結果相同。
- [引擎新增命令會讓沒有萬用分支的對照表編譯不過] → `wire_coverage` 在這個變更補上。變更 hook-read-function 新增的讀取類判定也是這種對照；把這個變更排在它之後，並在那張對照裡把兩個新命令歸為寫入類。
- [整份寫回詞彙表時，兩個人同時改會互相覆蓋] → 遠端路徑沿用寫入動詞的 revision 檢查；技能指示「讀出、修改、立刻寫回」，不隔著其他步驟。和 artifact 的整份寫回是同一種取捨。
- [回歸對照] → 既有動詞的輸出不變；刻意變更的只有四處拒絕訊息的後半句、命令 golden、渲染 golden 與資產鎖定檔。CLI 測試裡斷言舊訊息的地方用搜尋「move it out of」找出並同批更新。
- [資產版本號的連動容易漏] → D6 列出四件要一起做的事；任務清單把它們放在同一組，並在收尾用 `git status` 盤點重新產生的技能檔。若同時有另一個改技能原稿的變更，後合併的那一個要重新產生衍生物，不是挑一邊。
- [跨平台] → 本機檔案的改名在 Windows 上目的檔存在時會失敗，命令層已先擋掉這個情況；封存名稱一律用正斜線以外不含路徑分隔的檔名輸出（`restoredFrom` 只有檔名），所以輸出沒有平台差異。
- [搬回的記錄可能馬上又符合自動封存的條件] → 搬回不改內容；文件與技能寫明搬回之後接著做原本被擋住的那一步（再轉出一刀或繼續討論）。
- [朝儲存解耦靠攏] → 兩個動詞都只經 `Store` 讀寫，引擎不直接碰檔案系統；技能少了兩處依 checkout 分岔的文字。
