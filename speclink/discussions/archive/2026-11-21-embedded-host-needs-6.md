---
topic: 內嵌宿主：管理者實際匯入規格資料夾之後的第六輪（四項）
slug: embedded-host-needs-6
status: promoted
created: 2026-11-21
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: export-failure-reason-and-location, import-replace-mode
---

# Discussion: 內嵌宿主：管理者實際匯入規格資料夾之後的第六輪（四項）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

一個內嵌宿主做了管理頁：管理者在那裡登記規格資料、改流程政策、按一個按鈕把 repo 裡的規格資料夾匯入空的範圍（宿主在程序內用 `engine.exportScope()` 加 `host.importScope`）。用 `@speclink/engine` 0.15.0 實際操作之後，提出四項（F1 再匯入一次、F2 進度、F3 匯出失敗沒有原因代碼與位置、F4 範圍摘要）。這份討論接在 embedded-host-needs-5 之後。那個宿主是私有系統，記錄只談通用的情境。
需求已經明確（每一項都有現況、查證方式與需求），沒有先做需求釐清；輸入是文件，所以把文件的每個主張對照 `main`（就是 v0.15.0）逐項分類，並用本機的 Node 綁定實跑量測。
相關規格：node-sdk（「宿主的匯出與匯入方法」「本機引擎的 exportScope 方法」「匯出與匯入的失敗帶原因代碼」）、host-kit（「範圍的匯出與匯入」「匯入的大小上限由宿主設定」「執行前掛鉤可以拒絕命令」）、workspace-migration（「import 端點僅限 CreateNew 且原子」）、store-port-contract（「鍵的命名規則」）。目前沒有進行中的變更。
Source doc: ../speclink-embedded-host-needs-6.md
Prior discussions: embedded-host-needs, embedded-host-needs-2, embedded-host-needs-3, embedded-host-needs-4, embedded-host-needs-5

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-11-21)

**Focus**: 文件的四項主張對照 `main`（v0.15.0）後各屬於哪一類：已確認、與現況不符、要使用者決定
**Position**: F3 該修、F2 不用做、F4 只補文件、F1 是要使用者決定的一項。
- 決策地圖：F1 再匯入一次（做不做 → 取代時流程設定跟誰走 → 匯入後才建立的變更與討論怎麼辦 → 回報的形狀）；F3（空白的 `schema` → 新的原因代碼 → 錯誤物件的位置欄位）；F2 進度；F4 範圍摘要；最後是分刀
- F3 已確認：`schema: ''` 被讀成「指名空字串這個產出流程」，沒有這個鍵才用預設的 `spec-driven`（`lifecycle/model.rs` 的 `schema_name`）；找不到時經 `transfer.rs` 的 `resolve_local_schema` 回傳一句文字，沒有 `reason`、沒有變更名稱；node-sdk「匯出與匯入的失敗帶原因代碼」明文把「變更指定的產出流程找不到」列為沒有代碼
- F3 實跑（本機 Node 綁定，進行中與已封存的變更結果相同）：`created: ''`、`depends_on: ''`、`archived_at: ''`、`schema: ~` 匯出成功；`schema: nope`、`schema: 123` 失敗且沒有 `reason`、沒有位置；`reviewed_tasks_total: ''`、`reviewed_rounds: ''`、`reviewed_scope: ''`、重複的鍵失敗，`reason` 是 `invalid_change_metadata`，檔案路徑只寫在訊息裡
- F3 的建議（使用者沒有異議）：空白的 `schema` 視同沒有這個鍵；「變更指名的產出流程不能用」給一個新代碼；這個新代碼與 `invalid_change_metadata` 的錯誤物件帶變更名稱與路徑兩個欄位，訊息也加上變更名稱（Speclink Desktop 的「遷移到 Server」走同一個 `local_documents`）
- F2 實測：1,636 份文件、18.8 MB、241 個已封存變更的規格資料夾，跑三次，`engine.exportScope()` 3.5 到 3.8 秒，`host.importScope()`（記憶體儲存）2.3 到 2.9 秒；匯入只呼叫宿主的 `list` 一次與 `commit` 一次，寫進資料庫的時間在宿主自己的 `commit` 函式裡。結論：不加進度回呼
- F4 與現況不符：文件說鍵的命名規則「沒有寫成契約」，實際上是 store-port-contract「鍵的命名規則」，SDK 文件也有對照表；誤解來自 SDK 文件那句「你的儲存不需要認得文件的種類」
- F4 三樣資料宿主都能自己算：文件數用鍵的前綴在自己的儲存裡數；最後寫入的人與時間是 `commit` 收到的 `actor` 與 `at`；各階段的變更數是每個範圍一次 `list --json`（實測 1 到 5 毫秒，會經過 `beforeCommand`）。結論：只補 SDK 文件
- F1 已確認：範圍裡有任何文件就拒絕（`transfer.rs` 的 `import_with_events` 以空前綴列出）；流程設定是鍵 `config.yaml` 這一份文件，所以先調政策就會擋住匯入
- F1 已有前例：embedded-host-needs 定了「匯入維持只能進空的範圍」，embedded-host-needs-3 第 4 輪否決覆寫非空的範圍
**Ruled out**: F2 的進度回呼 — 引擎的兩步合計約 6 到 7 秒，等待的時間主要是宿主自己的下載，匯入的寫入時間在宿主自己的 `commit` 函式裡；F4 新的摘要呼叫 — 只是把「列出再數」包一層，宿主用自己的資料庫一次查詢數十個範圍更便宜；F4 把儲存鍵的命名規則「寫成契約」 — 它已經是契約，缺的是文件說清楚
**Open**: F1 要不要做取代模式；F1 取代時流程設定跟誰走；F1 匯入後才建立的變更與討論怎麼處理；F1 回報與試跑的形狀；F3 新代碼的名稱與欄位名稱；分刀

### Round 2 — interview (2026-11-21)

**Focus**: F1 要不要在 `host.importScope` 加「取代」模式，推翻前兩輪「匯入只能進空的範圍」
**Position**: 做取代模式，只在程序內的呼叫提供。
- `host.importScope` 多一個取代模式：一次提交讓範圍的內容等於匯入包，回報新增、變更、刪除各幾份
- 同一個呼叫可以試跑：只回報數字，不寫入
- HTTP 的 `POST /import` 與 Speclink Desktop 的「遷移到 Server」維持只收空的範圍，workspace-migration「import 端點僅限 CreateNew 且原子」不動
- `beforeCommand` 看得到這次是取代，宿主可以拒絕
- 要改 host-kit「範圍的匯出與匯入」與 node-sdk「宿主的匯出與匯入方法」；文件寫明取代會刪掉匯入後在宿主裡做的全部改動
- 推翻前例的理由：embedded-host-needs-3 的第二個理由「四支儲存實作都要改」已經不成立（匯入只用儲存接口的三個函式，實作只有 `transfer.rs` 一處）；第一個理由管的是 HTTP 端點，取代不上線路就不違反
- 新的證據：管理者實際回答「需要重新匯入」（團隊不會同一天全部改用宿主）；替代做法要換一個不能改的代號，並重做存取設定與 `remote.repo`；SDK 文件現有的「先在你的儲存裡刪掉全部文件再匯入」沒有說版本號怎麼處理，清空與匯入之間連線中的用戶端會看到空的範圍，也可能寫入
**Ruled out**: 維持不做、只把清空範圍的做法寫清楚 — 清空與匯入是兩步，中間有空窗，版本號與通知用戶端都留給每個宿主自己處理；取代模式也開在 HTTP 端點 — 違反 workspace-migration「wire SHALL NOT 暴露 Overwrite」，原版用戶端沒有這個需求
**Open**: F1 取代時流程設定跟誰走；F1 匯入後才建立的變更與討論怎麼處理；F1 回報與試跑的形狀；F3 新代碼的名稱與欄位名稱；分刀

### Round 3 — interview (2026-11-21)

**Focus**: F1 取代時，流程設定與「匯入後才在宿主建立、匯入包沒有」的變更與討論怎麼處理
**Position**: 取代是純粹的「範圍的內容等於匯入包」，沒有例外。
- 依據：流程政策（`locale`、`tdd`、`audit`、`worktree`…）與專案說明、產出規則、審查規則都在同一份文件 `config.yaml`，repo 的 `speclink/config.yaml` 會進匯入包；這份文件只能整份換掉或整份留下，除非引擎逐鍵合併
- 取代後 `config.yaml` 是匯入包的版本；匯入包沒有的鍵（含匯入後才在宿主建立的變更與討論）被刪除
- 政策由宿主重新套用：取代前讀 `workflow-config show --json`，取代後把管理頁的鍵寫回去；SDK 文件給這段範例，「先調政策、再匯入」用同一段範例處理
- 試跑的回報列出鍵，不只給數字：管理者看不到 repo，要知道會刪掉哪幾個變更與討論才能決定；宿主用鍵的前綴分類
- 內容相同的文件不重寫：版本與修改時間不變，連線中的用戶端不用重新讀取
- 例子：範圍有管理者設了 `tdd: true` 的 `config.yaml`、舊的 `specs/auth/spec.md`、匯入後建立的 `changes/add-x/…` 與 `discussions/d1.md`；匯入包有 repo 的 `config.yaml`（沒有 `tdd`）、新的 `specs/auth/spec.md`、`specs/billing/spec.md`。取代後：`config.yaml` 與 `specs/auth/spec.md` 是匯入包的內容，`specs/billing/spec.md` 新增，`changes/add-x/…` 與 `discussions/d1.md` 刪除
**Ruled out**: 取代時保留範圍裡的 `config.yaml` — repo 裡改過的專案說明與產出規則永遠進不來；引擎逐鍵合併流程設定 — 引擎要定義哪些鍵算政策，多一份要維護的清單；保留匯入包沒有的變更與討論（合併） — repo 已封存的變更在範圍裡會同時留著進行中的那一份，repo 的刪除也永遠不會生效
**Open**: F1 回報與試跑的形狀、空的匯入包與並行寫入的邊界；F3 新代碼的名稱與欄位名稱；分刀

### Round 4 — interview (2026-11-21)

**Focus**: 剩下的細節（F1 回報與邊界、F3 代碼與欄位）與分刀
**Position**: 分成兩個變更依序做：cut 1 `export-failure-reason-and-location`（F3 加 F2、F4 的文件），cut 2 `import-replace-mode`（F1）。
- 兩個變更都會改 node-sdk「匯出與匯入的失敗帶原因代碼」與 `transfer.rs`，所以不能平行
- F3：新代碼是 `change_schema_unusable`，涵蓋產出流程找不到、找到但無法解析、有引擎列不出的檔案樣式三種；錯誤物件的欄位是 `change` 與 `path`
- F1：呼叫多 `mode`（`'create'` 或 `'replace'`，預設 `'create'`）與 `dryRun`；回報多 `added`、`changed`、`removed` 三個鍵的清單；試跑照樣先問 `beforeCommand`
- F1 的兩個邊界：空的匯入包加取代模式被拒絕；列出之後另一方寫入同一批鍵時整筆拒絕並給新的原因代碼
- 先做 cut 1 的理由：它小，宿主把它排在最重，先發版就能讓大的規格資料夾匯入成功；cut 2 會刪除資料，單獨審查
**Ruled out**: 合成一個變更 — 取代模式會刪除資料，和小的錯誤訊息修正放在一起審查會分散焦點，也會拖住 F3 的發版；兩個變更平行做 — 改同一條需求與同一個檔案，後封存的那個 delta 會對不上
**Open**: 沒有

## Conclusion

**Decision**: 四項裡 F3 修正、F1 做取代模式、F2 與 F4 只補文件，分成兩個變更依序做。
- **cut 1 `export-failure-reason-and-location`**: 匯出因為某個變更的產出流程而失敗時帶原因代碼與位置，空白的 `schema` 視同沒有設定，並補上 F2、F4 的文件
  - 變更的 metadata 裡 `schema` 是空字串或只有空白時，視同沒有這個鍵，用預設的 `spec-driven`；這條規則對全部讀取端生效，不只匯出
  - `engine.exportScope` 多一個原因代碼 `change_schema_unusable`，`code` 仍是 `error`，涵蓋三種失敗：變更指名的產出流程找不到、產出流程找到但內容無法解析、產出流程有引擎列不出的檔案樣式
  - 不併進 `invalid_change_metadata`：既有代碼的意思不能改，而且處理方法不同（一個是 metadata 檔壞了，一個是那台機器上沒有那個產出流程）
  - `change_schema_unusable` 與 `invalid_change_metadata` 的錯誤物件多兩個字串欄位：`change`（變更在磁碟上的名稱，已封存的是帶日期的名稱，例如 `2026-03-03-old-one`）與 `path`（metadata 檔相對於專案根目錄的路徑，用正斜線）
  - 產出流程失敗的訊息前面加上變更名稱；`invalid_change_metadata` 的訊息不變。Speclink Desktop 的「遷移到 Server」走同一個 `local_documents`，會顯示新的訊息
  - 其他沒有代碼的失敗（同一份文件被列出兩次）維持沒有 `reason`
  - 要改寫 node-sdk「匯出與匯入的失敗帶原因代碼」（它明文把「變更指定的產出流程找不到」列為沒有代碼）與「本機引擎的 exportScope 方法」；空白 `schema` 的規則屬於哪一份規格，提案時對照 change-lifecycle 與 workflow-schemas 決定
  - 實跑的現況（提案時當成測試的例子）：`created: ''`、`depends_on: ''`、`archived_at: ''`、`schema: ~` 匯出成功；`schema: ''`、`schema: nope`、`schema: 123` 失敗且沒有 `reason`、沒有位置；`reviewed_tasks_total: ''`、`reviewed_rounds: ''`、`reviewed_scope: ''`、重複的鍵以 `invalid_change_metadata` 失敗，路徑只寫在訊息裡
  - 例子：`speclink/changes/archive/2026-03-03-old-one/.meta.yaml` 寫 `schema: ''` 時匯出成功，那個變更照 `spec-driven` 列出文件；寫 `schema: nope` 時 Error 的 `code` 是 `error`、`reason` 是 `change_schema_unusable`、`change` 是 `2026-03-03-old-one`、`path` 是 `speclink/changes/archive/2026-03-03-old-one/.meta.yaml`，訊息以變更名稱開頭
  - SDK 文件（F4）：寫明鍵的對照表是契約（store-port-contract「鍵的命名規則」），改寫「你的儲存不需要認得文件的種類」那句；加一段「範圍的摘要」：文件數用鍵的前綴在自己的儲存裡數、最後寫入的人與時間取自 `commit` 收到的 `actor` 與 `at`、各階段的變更數用每個範圍一次 `list --json`（會經過 `beforeCommand`）
  - SDK 文件（F2）：寫明耗時的量級（1,636 份文件、18.8 MB 的規格資料夾，匯出約 3.5 到 3.8 秒，匯入到記憶體儲存約 2.3 到 2.9 秒），以及進度在宿主自己的下載與 `commit` 函式裡；匯入只呼叫宿主的 `list` 一次與 `commit` 一次
- **cut 2 `import-replace-mode`**: `host.importScope` 加取代模式與試跑，一次提交讓範圍的內容等於匯入包
  - `HostImportCall` 多兩個選填欄位：`mode`（`'create'` 或 `'replace'`，預設 `'create'`，行為與現在相同）與 `dryRun`
  - 取代是純粹的「範圍的內容等於匯入包」，沒有例外：匯入包有而範圍沒有的鍵新增、內容不同的鍵更新、匯入包沒有的鍵刪除（含 `config.yaml`，含匯入後才在宿主建立的變更與討論）
  - 內容相同的文件不重寫，版本與修改時間不變
  - 全部寫入與刪除在正好一次提交裡，帶一筆 scope-imported 事件；範圍的版本號照常加一
  - `HostImportReport` 多三個鍵的清單 `added`、`changed`、`removed`，兩種模式都給
  - 試跑不提交、不通知 `afterCommit`，回報與真的執行相同的三個清單；它會讀範圍的內容，所以照樣先問 `beforeCommand`
  - `beforeCommand` 收到的 `scope-import` 參數多 `mode` 與 `dryRun`；命令名稱不變，`commandNames` 不變
  - 空的匯入包加取代模式被拒絕：一次錯誤的呼叫不能清空整個範圍
  - 列出之後、提交之前另一方寫入同一批鍵時整筆拒絕，範圍不變，可以重試，並給一個新的原因代碼（名稱提案時定）；另一方寫入的鍵不在匯入包、列出時也不存在時匯入不會察覺，那份文件原樣保留（和現有的建立模式同一個邊界）
  - `limits.importBytes` 照舊管取代模式
  - 只在程序內的呼叫提供：`POST /import`、Speclink Desktop 的「遷移到 Server」與 workspace-migration「import 端點僅限 CreateNew 且原子」都不變
  - 流程政策由宿主重新套用：取代前讀 `workflow-config show --json`，取代後把管理頁的鍵寫回去；SDK 文件給這段範例，「先調政策、再匯入」用同一段範例處理
  - SDK 文件「在伺服端匯入」的「要重新匯入」那一點改寫成取代模式，寫明取代會刪掉匯入後在宿主裡做的全部改動（變更、討論、蓋章）
  - 要改寫 host-kit「範圍的匯出與匯入」、node-sdk「宿主的匯出與匯入方法」與「匯出與匯入的失敗帶原因代碼」
  - 介面深度：取代是既有接縫 `transfer.rs` 匯入函式的一個模式，不是新模組；它藏住比對、每個鍵的預期版本與單次提交，只有這一處實作
  - 例子：範圍有管理者設了 `tdd: true` 的 `config.yaml`、舊的 `specs/auth/spec.md`、匯入後建立的 `changes/add-x/proposal.md` 與 `discussions/d1.md`；匯入包有 repo 的 `config.yaml`（沒有 `tdd`）、新的 `specs/auth/spec.md` 與 `specs/billing/spec.md`。試跑回報 `added` 是 `specs/billing/spec.md`，`changed` 是 `config.yaml` 與 `specs/auth/spec.md`，`removed` 是 `changes/add-x/proposal.md` 與 `discussions/d1.md`，範圍不變；真的執行後範圍正好是匯入包的三份文件，宿主再把 `tdd: true` 寫回去
- F2 不加進度回呼；F4 不加摘要呼叫
**Rationale**: F3 的失敗讓管理者看到一句沒有位置的英文訊息，而且那四個舊檔案的 `schema` 是不帶任何資訊的空字串，所以同時放寬讀取並補上代碼與位置。F1 前兩輪都否決，這次推翻有三個原因：管理者實際需要（團隊不會同一天全部改用宿主）、「四支儲存實作都要改」這個理由已經不成立（匯入只用儲存接口的三個函式）、而且只在程序內提供就不違反線路上只收空範圍的規定；取代做成純粹的「等於匯入包」，是因為流程政策和專案說明在同一份文件裡，任何例外都要引擎定義哪些鍵算政策。F2 實測引擎的兩步合計約 6 到 7 秒，等待的時間在宿主自己的下載。F4 的三樣資料宿主用已經是契約的鍵與自己的 `commit` 就能算，新的呼叫只是多包一層。
**Rejected alternatives**:
- F1 維持不做、只把清空範圍的做法寫清楚 — 清空與匯入是兩步，中間連線中的用戶端會看到空的範圍也可能寫入，版本號與通知都留給每個宿主自己處理
- 取代模式也開在 HTTP 的 `POST /import` — 違反 workspace-migration「wire SHALL NOT 暴露 Overwrite」，原版用戶端沒有這個需求
- 取代時保留範圍裡的 `config.yaml` — repo 裡改過的專案說明與產出規則永遠進不來
- 引擎逐鍵合併流程設定 — 引擎要定義哪些鍵算政策，多一份要維護的清單
- 取代時保留匯入包沒有的變更與討論（合併） — repo 已封存的變更在範圍裡會同時留著進行中的那一份，repo 的刪除也永遠不會生效
- 產出流程找不到併進 `invalid_change_metadata` — 規格寫明既有代碼的意思不能改，兩者的處理方法也不同
- F2 的進度回呼 — 引擎的兩步合計只要幾秒，匯入的寫入時間在宿主自己的 `commit` 函式裡
- F4 新的摘要呼叫 — 只是把「列出再數」包一層，宿主用自己的資料庫一次查詢數十個範圍更便宜
- F4 把儲存鍵的命名規則寫成契約 — 它已經是 store-port-contract 的正式需求，缺的是 SDK 文件說清楚
- 合成一個變更 — 取代模式會刪除資料，和小的錯誤訊息修正放在一起會分散審查的焦點，也會拖住 F3 的發版
- 兩個變更平行做 — 改同一條需求與同一個檔案，後封存的那個 delta 會對不上
**Deferred**:
- 大的規格資料夾寫進真實資料庫的耗時 — 這次只量了記憶體儲存；等宿主匯入成功後有實際數字再看
- 符號連結與列不出內容這兩種失敗也帶 `path` 欄位 — 宿主這一輪沒有遇到，訊息裡已經有路徑
- 持續同步 repo 的規格資料夾進宿主 — 前幾輪都延後；取代模式是宿主主動再匯入一次，不是同步
- 來源文件列為還沒確認的兩個部分（大的規格資料夾匯入後用戶端的表現、管理頁的名稱與用戶端欄位的對照）— 是宿主環境的實測，不是已知的缺陷
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion embedded-host-needs-6
