---
topic: 內嵌宿主：把引擎接到宿主自己的 Agent 與畫面之後的第七輪（五項）
slug: embedded-host-needs-7
status: promoted
created: 2026-11-23
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: archived-list-and-detail-reads, door-reads-ask-before-command, render-host-options-and-tool-verbs, reserved-field-prefixes, cross-resource-events-and-event-names
---

# Discussion: 內嵌宿主：把引擎接到宿主自己的 Agent 與畫面之後的第七輪（五項）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

內嵌引擎的宿主把引擎接到自己的 Agent（一個 `speclink` 工具加六支產生的技能）與桌面畫面（範圍總覽、單一項目全文、依 `afterCommit` 的事件推送）之後，提出第七輪的五項需求 G1 到 G5。來源文件把每一項寫成待檢驗的主張，目標與界線都明確，所以沒有先做需求釐清，直接逐項對照程式碼（main，等於 v0.16.0 加一個文件 commit）分成「成立」「和程式碼不同」「要使用者決定」。
相關的規格：host-kit、node-sdk、free-fields、server-event-stream、verb-contract、skill-authoring、discussion-docs、store-port-contract、review-station、verify-station。目前沒有進行中的變更。
Source doc: ../speclink-embedded-host-needs-7.md
Prior discussions: embedded-host-needs-6, embedded-host-needs-5, embedded-host-needs-4, embedded-host-needs-2, embedded-host-needs, remote-remaining-gaps

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-11-23)

**Focus**: 來源文件的五項主張，各自和程式碼對不對得上
**Position**: 五項的問題都存在，但有六處和文件寫的不同，其中兩處（G1、G4 的權限）範圍比文件大，兩處（G3 的提問工具、G5 的替代做法）比文件小。
- G1 成立：`new change --from-discussion` 只送 `change-created`，`discard` 解開討論只送 `change-discarded`（`speclink-core/src/command/mod.rs` 的 `events_of`）
- G1 範圍更大：`archive` 連帶封存來源討論只送 `change-archived`；對已轉出的討論再次 conclude 會在別的變更寫 `restale_from`（`discuss.rs` 的 `stamp_restale`）只送 `discussion-concluded`；`discuss link` 寫變更的 `from_discussion`、`discuss seal` 在清掉過期標記時寫變更的 metadata，兩者的事件只對到討論
- G1 的替代做法和文件不同：文件「宿主現在的替代做法」寫沒有；宿主的儲存 `commit` 函式收到每一筆寫入的鍵，鍵的命名是契約（store-port-contract「鍵的命名規則」），所以現在就能從鍵推出改了哪些資源，只是要自己把鍵和 `afterCommit` 對起來
- G1 的影響面：Speclink Desktop 收到任何事件都整批重讀（`apps/desktop/src-tauri/src/event_manager.rs`），所以只有依資源分開推送的宿主會看到過期的畫面
- G2 部分不同：SDK 沒有事件名稱清單（成立）；但文件「唯一的來源是 verb-contract 的表格」不對，Rust 已有唯一的對照函式 `event_resource(kind, payload)`（`speclink-host/src/commit.rs`），示範 server 的推送用它，缺的只是 Node SDK 沒有匯出
- G3 成立：discuss 寫 Grep／Glob，trace 用 `git log`／`git blame`，propose、ingest、analyze 指到 `speclink-apply`，config 要讀 `speclink-review` 的技能檔；`RenderOptions` 多給的鍵不報錯；`agentTool.description` 是從 CLI 命令定義產生的固定字串（`speclink-verbs/src/tool.rs`）
- G3 不同：每一處 `AskUserQuestion` 都已有「沒有這個工具就用文字問並等回答」的退路（`assets/skills/propose.md:13`、`analyze.md:56`、`config.md:97`、`ingest.md:7`），不需要開關
- G3 補充：`target: 'neutral'` 是給五個內建 AI 工具以外的 harness 用的（`.speclink.yaml` 的自訂工具），假設有程式碼的 checkout；`specInCheckout: false` 只表示規格不在 checkout 裡，沒有「完全沒有 checkout」的選項。區塊條件目前只有 `SPEC_IN_CHECKOUT` 一個（`workspace/skills.rs` 的 `resolve_blocks`）
- G4 資料成立三樣半：沒有動詞列出已封存的變更；`list --json --detail` 沒有 `fromDiscussions` 與 `createdBy`（`ListChangeJson`）；`discuss list|show --json` 沒有 `promotedTo`；`review show --json`／`verify show --json` 沒有工單全文，但引擎的結果有（`ReviewShowOutcome.content`），是輸出層刻意不放（`render/station.rs`）
- G4 不同：讀已封存變更的一份文件已經做得到，`artifact cat <artifact> --change <名稱或帶日期的名稱>` 會讀封存的那一份（本機實跑確認；`show <已封存的變更>` 回 not found 這一點成立）
- G4 進行中的關卡沒有輪數成立：判定只看工單在不在（`quality/station.rs` 的 `status`），要輪數得多讀並解析工單
- G4 權限範圍更大：入口不問 `beforeCommand` 的讀取有八條，不只三條 `/archived`：`GET /config`、`GET /board-order`、`GET /specs/{capability}/document`、`GET /search`、三條 `/archived`、`POST /context`（整個範圍的文件快照）。入口只在 `Request::run` 與看板順序的寫入問掛鉤（`http/routes.rs`）。這符合 host-kit「引擎命令與看板順序的寫入經過掛鉤」的字面，但規格沒寫這八條是刻意不問
- G4 範圍清單回 404 不問掛鉤：host-kit「範圍與能力由宿主的函式決定」明文規定，維持
- G5 成立：argv 可以寫任何鍵，宿主帶同名鍵時宿主的值勝出；三個建立命令（`new-change`、`discuss-new`、`discuss-promote`）的 `params.fields` 是合併後的結果，分不出來源
- G5 不同：宿主不必掃 argv。`change field set|unset`、`discuss field set|unset` 與兩條 `PUT …/fields` 都是引擎命令，`beforeCommand` 收到整理好的 `params.set` 與 `params.unset`（`Command::params` 把欄位原樣序列化）；建立命令也能在 `beforeCommand` 對合併後的值做「呼叫者能不能從這個專案發起」的檢查。剩下的風險只有「標成另一個他有權限的專案」
- G5 和正式規格：free-fields 規定引擎不解讀鍵、不因鍵改變守門；保留前綴放在宿主套件也要在規格寫明例外。SDK 的型別說明寫自由欄位「from your own code, never from the model」，但模型用 argv 寫得了同名的鍵，說明和行為有落差
- 前議：`promotedTo` 不進引擎的 JSON 是 remote-remaining-gaps 的決定（保 CLI 輸出逐位元不變）；要重開須說明理由為何不再成立
**Open**: D1 一次提交改到兩種資源時事件怎麼補（G1）；D2 SDK 匯出什麼（G2，看 D1）；D3 動詞補哪幾樣資料、工單全文與輪數用什麼形狀、`promotedTo` 要不要重開前議（G4）；D4 八條不問掛鉤的讀取怎麼處理、入口回應的欄位要不要寫進文件（G4）；D5 技能文字與工具說明用什麼方式調整、加幾個開關（G3）；D6 保留欄位鍵由宿主套件擋、讓掛鉤看得出來源、或只補文件（G5）；D7 分刀與順序

### Round 2 — interview (2026-11-23)

**Focus**: D1 一次提交改到兩種資源時，事件怎麼補
**Position**: G1 要做：改到幾個資源就送幾個事件，維持「一個事件只指一個資源」，並加一個守門測試。
- `new change --from-discussion` 多送 `discussion-promoted {slug, change}`，和 `discuss promote` 相同
- `archive` 連帶封存幾份討論，就多送幾個 `discussion-archived {slug}`
- `discard` 解開討論時多送一個新事件，暫名 `discussion-unlinked {slug, change}`
- 再次 conclude 標記到的變更、`discuss link` 與 `discuss seal` 改到的變更，每個變更多送一個新事件，暫名 `change-provenance-changed {change}`
- 事件只在真的寫入時送（`events_of` 既有的原則）：`seal` 只有清掉過期標記時才寫變更；再次 conclude 回報的清單含已經標記、沒有重寫的變更；`link` 重複時不寫。提案時要讓結果分得出「真的寫了」
- 守門測試（換模型重新檢查後加上，使用者尚未明確回覆）：`speclink-host/src/port_exec.rs` 組提交時同時拿到寫入的鍵與事件；測試對每個命令檢查「每個被寫到的進行中變更與討論，都有一個指到它的事件」。五種情況是讀程式碼找到的，不保證找全；測試能機械地找全，也擋住以後新命令再犯
- 使用者同意這個做法與五種情況
**Ruled out**: 一個事件對到多個資源 — 四種情況（從討論建立變更、連帶封存、解開、再次 conclude）的 payload 根本沒有另一個資源的名稱，還是要補事件或改 payload；剩下的 `link`／`seal` 為了一致也用多送事件。（上一版理由「Speclink Desktop 要改」不對：它收到任何事件都整批重讀）
**Open**: D2；D3；D4；D5；D6；D7。D1 的兩個新事件名稱提案時定

### Round 3 — interview (2026-11-23)

**Focus**: D2 SDK 匯出什麼，讓宿主把事件對到資源
**Position**: G2 要做：匯出名稱清單加對照函式，兩者都來自 Rust 既有的那張表。
- `eventNames`：全部事件名稱的凍結陣列，形狀與用法和 `commandNames` 相同；宿主用它寫測試，引擎新增事件時測試會失敗
- `eventResource(name, payload)`：回傳 `{ scope, resourceId }`，就是 `speclink-host/src/commit.rs` 的 `event_resource`，和示範 server 推送用的是同一份
- 例子：`eventResource('discussion-promoted', { slug: 'two', change: 'add-two' })` 回 `{ scope: 'discussion', resourceId: 'two' }`；`eventResource('workflow-config-changed', { keys: ['tdd'] })` 回 `{ scope: 'unknown', resourceId: '' }`，由宿主自己依名稱處理
- D1 新增的事件要同批進這張表與 verb-contract 的 Event stream 表格
- 使用者同意守門測試與「事件只在真的寫入時送」（D1 的兩個補充，沒有提出異議）
**Ruled out**: 靜態對照表（每個名稱附指資源的鍵與類別）— 宿主還要自己從 payload 取值，`change-archived` 這種特例（類別是規格、值是變更名稱）要另外說明；函式直接給答案
**Open**: D3；D4；D5；D6；D7。`eventNames` 要不要含宿主套件自己的事件（例如 scope-imported、看板順序的事件），提案時對照 `commandNames` 的做法決定

### Round 4 — interview (2026-11-23)

**Focus**: D4 入口裡八條不問 `beforeCommand` 的讀取怎麼處理
**Position**: 八條都改成先問掛鉤，被拒絕時回 409、reason 為 refused、訊息是宿主給的原因；入口回應的欄位表先不寫。
- 八條是 `GET /config`、`GET /board-order`、`GET /specs/{capability}/document`、`GET /search`、`GET /archived`、`GET /archived/{datedName}/capabilities`、`GET /archived/{datedName}/artifacts/{path}`、`POST /context`
- 能對到既有命令的用既有的命令名稱，宿主現有的規則自動生效：`GET /config` 用 `workflow-config-show`，`GET /specs/{capability}/document` 用 `show`，`GET /archived/{datedName}/artifacts/{path}` 用 `artifact-cat`
- 對不到的新增名稱並放進 `commandNames`（和 `put-board-order` 同一種做法）：`GET /search`、`POST /context`、`GET /board-order`
- 兩條列出已封存的路由暫定新增名稱；D3 若加了列出已封存變更的動詞，就改用那個命令
- 範圍清單沒有這個專案時回 404、不問掛鉤維持不變（host-kit「範圍與能力由宿主的函式決定」的明文；不讓人從錯誤訊息猜出專案存不存在）
- 要改寫 host-kit「HTTP 入口處理原版用戶端的請求」裡「經入口執行的引擎命令與看板順序的寫入經過掛鉤」那一句，改成入口的每一條讀寫都經過掛鉤
- 相容性影響：只放行自己認得的名稱的宿主，升級後會擋掉 Speclink Desktop 的 `/context`、`/search`、`/board-order`，直到把新名稱加進權限表；`commandNames` 的測試會提醒。Speclink Desktop 在這三條收到 409 時的表現，提案時實測
- 例子：某個身分在範圍清單裡，掛鉤對他一律拒絕、原因「帳號已停用」。現在 `GET /changes` 回 409 加原因，`GET /archived` 與 `POST /context` 回 200 加內容；改完三條都回 409、reason 為 refused、訊息「帳號已停用」
**Ruled out**: 只補三條 `/archived`（來源文件要求的範圍）— 另外五條仍繞過掛鉤，包含回整個範圍文件的 `/context`；不改程式只補文件、寫明這八條由 `scopes` 決定 — 宿主仍要理解兩條權限路徑，掛鉤「每次呼叫問一次」留著例外；現在就寫入口回應的欄位表 — 欄位只在 `speclink-protocol` 的原始碼、沒有對外發布的檔案，D3 讓動詞讀得到同樣的資料之後宿主不必依賴入口的欄位
**Open**: D3；D5；D6；D7

### Round 5 — interview (2026-11-23)

**Focus**: D3 動詞要補哪幾樣畫面需要的資料
**Position**: 五樣都補，全部做在引擎命令裡，本機、遠端連線、內嵌宿主得到同一份輸出；`promotedTo` 重開 remote-remaining-gaps 的決定。
- 列出已封存的變更：`list` 加 `--archived`；每一筆的欄位照入口 `GET /archived` 現有的 `ArchivedItem`（帶日期的名稱、日期、名稱、任務數、建立的人、來源討論、兩個關卡的結果、自由欄位），再加 delta 的 capability 名稱。文件用既有的 `artifact cat <artifact> --change <帶日期的名稱>` 讀
- 變更的來源討論與建立的人：`list --json --detail` 加 `fromDiscussions` 與 `createdBy`
- 進行中關卡的輪數：`list --json --detail` 與 `show --json --detail` 的進行中關卡加 `rounds`；成本是每個有工單的變更多讀一份文件，只有 `--detail` 才讀
- 工單全文：`review show --json --detail`、`verify show --json --detail` 加 `content`。不直接加進 `--json`：審查與驗證技能讀 `review show --json` 取最後一輪，直接加會每次把全文放進模型的上下文
- 討論轉出的變更：`discuss list --json`、`discuss show --json` 加 `promotedTo`，沒有值時不出現
- 重開前議的理由：當時的理由是 CLI 的 JSON 逐位元不變。之後加的 `fields`、`createdBy`、`kind` 都採用「沒有值就不出現」，已是慣例；當時讀這份輸出的只有 CLI 與 Speclink Desktop，資料在 server 端補上就夠，現在內嵌宿主直接讀動詞的輸出，沒有 server 可以補
- 相容性影響：只有 `promotedTo` 會改既有輸出——已轉出的討論多一個鍵，沒有轉出的逐位元相同；golden 同批更新。其餘四樣都在新旗標或 `--detail` 後面，既有輸出不變
- 遠端連線的 CLI 走同一支輸出函式，線上格式缺的欄位（進行中關卡的輪數）同批以選填欄位補上
- D4 的連動：`GET /archived` 與它的 `capabilities` 改用 `list` 這個命令名稱問掛鉤，不新增名稱
- 例子：`list --json --detail` 的一筆有 `"createdBy": "Ann <ann@example.com>"`、`"fromDiscussions": ["two"]`、`"review": { "state": "inProgress", "rounds": 2 }`；`discuss list --json` 裡 `two` 那一筆多 `"promotedTo": ["add-two"]`；`list --archived --json` 回 `{ "archived": [{ "datedName": "2026-11-22-add-two", "date": "2026-11-22", "name": "add-two" }] }`
**Ruled out**: 工單全文直接加進 `review show --json` — 既有輸出每一筆都會變，而且技能每次讀都帶全文；`promotedTo` 維持由 server 端組裝（前議）— 內嵌宿主經 `dispatch` 讀動詞，沒有那一層；進行中的輪數不做、由宿主另外讀工單 — 總覽畫面每個變更要多一次呼叫
**Open**: D5；D6；D7

### Round 6 — interview (2026-11-23)

**Focus**: D5 技能文字與工具說明怎麼依宿主的能力調整
**Position**: G3 做小的：`skills.render` 多兩個選項，引擎在前言多寫標準說明，技能內文不改；`agentTool` 加一個可以排除動詞的函式。
- 份量的依據：來源文件寫的代價是模型「可能」做錯，沒有實際觀察到的錯誤；`skills.render` 回傳字串，宿主今天也能自己加前言
- `skills`（可用的技能名稱清單）：前言多一句「這個環境只提供清單裡的技能；清單外的不要載入、不要建議使用者執行，改說這一步由工程師在自己的電腦做」；不認得的技能名稱報錯
- `checkout: false`（完全沒有 checkout，名稱暫定）：前言多一句「沒有程式碼、沒有 git 歷史；跳過讀程式碼與讀 git 的步驟，事實只用 speclink 的動詞查證，並在回答裡說明跳過了什麼」；和 `specInCheckout: true` 同時給時報錯
- 不認得的選項鍵報錯，和 `createHost` 相同（現況是打錯鍵名沒有任何提示）
- 不做「有沒有提問工具」的開關：每一處 `AskUserQuestion` 都已有用文字問的退路
- 「檔案搜尋」與「git 歷史」合成一個開關：沒有程式碼的宿主兩樣都沒有
- `agentTool` 加一個產生說明的函式，接受要排除的動詞；排除 `review` 會一併排除它底下的全部動詞；不認得的動詞報錯；原本的 `agentTool.description` 不變
- `target: 'neutral'` 維持原意（五個內建 AI 工具以外的 harness），不另外加給內嵌宿主的 target
- 例子：`skills.render('propose', { target: 'neutral', invocation: 'tool-call', specInCheckout: false, checkout: false, skills: ['discuss', 'trace', 'propose', 'ingest', 'analyze', 'config'] })` 的前言說明只有這六支技能、沒有程式碼與 git；內文最後仍提到 `speclink-apply`，模型依前言改說「實作由工程師在自己的電腦做」。多給打錯的鍵 `{ skils: [...] }` 直接丟出錯誤
**Ruled out**: 改技能內文、新增條件區塊（沒有程式碼時整段換掉）— 六支技能都要改、輸出的組合加倍、之後每次改技能都要顧兩種版本，而目前沒有實際觀察到的錯誤；只補文件、教宿主自己加前言 — 每個宿主各寫各的，用語和技能對不上；提問工具的開關 — 退路已經在技能裡；檔案搜尋與 git 歷史各一個開關 — 沒有只缺其中一樣的情境
**Open**: D6；D7

### Round 7 — interview (2026-11-23)

**Focus**: D6 宿主保留的欄位鍵怎麼做
**Position**: G5 要做：`createHost` 接受 `reservedFieldPrefixes`，請求要寫或刪這些前綴的鍵時由宿主套件拒絕，只有宿主自己帶的欄位寫得了。
- 落點只有一處：`speclink-host/src/kit.rs` 的 `attach_fields`，那裡同時拿得到請求寫的欄位與宿主帶的欄位；`dispatch` 與 `handleRequest` 都經過它，所以 CLI、Speclink Desktop、Agent 三個入口一次涵蓋，以後新增會寫欄位的命令也自動涵蓋
- 涵蓋：三個建立命令（`new change`、`discuss new`、`discuss promote`）的 `--field`，`change field set|unset`、`discuss field set|unset`，以及入口的 `PUT /changes/{name}/fields` 與 `PUT /discussions/{slug}/fields`
- 請求寫了保留的鍵就拒絕，即使宿主這次也帶了同名的鍵（現在是宿主的值默默勝出）
- 錯誤碼是 `refused`，訊息指出是哪一個鍵；入口回 409
- 前綴不合自由欄位的鍵規則時 `createHost` 報錯；沒有設定的宿主行為和現在完全相同
- 和正式規格的關係：free-fields「引擎不解讀任何鍵、不因鍵改變守門」維持成立——規則由宿主宣告、由宿主套件執行，性質和 `beforeCommand` 相同，引擎核心不認得任何鍵；提案時在 host-kit 加一條需求寫明
- 讓 SDK 型別說明的「from your own code, never from the model」成立
- 介面深度：不是新模組，是既有接縫（宿主套件合併欄位那一步）多一個檢查；刪掉它，宿主就回到自己在 `beforeCommand` 寫規則
- 例子（`reservedFieldPrefixes: ['host.']`）：argv `['new', 'change', 'spoof', '--field', 'host.project=p2']` 被拒絕、沒有寫入；argv `['new', 'change', 'ok']` 加宿主帶的 `fields: { 'host.project': 'p1' }` 成功；argv `['change', 'field', 'set', 'ok', 'host.project=p2']` 與 `['change', 'field', 'unset', 'ok', 'host.project']` 被拒絕；`PUT /changes/ok/fields` 的內容含 `host.project` 回 409；`--field team=web` 不受影響
**Ruled out**: 讓 `beforeCommand` 多收到「宿主這次帶了哪些欄位」、由宿主自己寫規則 — 宿主要檢查 `params.fields`、`params.set`、`params.unset` 三處，以後新命令換了參數名稱就會漏；只補文件、教宿主在 `beforeCommand` 檢查 — 今天做得到，但「把來源標成另一個他有權限的專案」擋不住，SDK 說明和行為的落差也還在
**Open**: D7 分刀與順序。兩個界線這次不處理、結論時列為延後：保留的鍵建立後不能改（宿主帶的欄位只在建立時寫入）；匯入不檢查保留的鍵（寫入的是整份文件，只有管理者能做）

### Round 8 — interview (2026-11-23)

**Focus**: D7 五項怎麼分成變更、用什麼順序
**Position**: 分成五個變更，依序一次做一個：G4 的資料、G4 的權限、G3、G5、G1 加 G2。
- 順序照來源文件的份量：G4 最重，G3、G5 其次，G1 與 G2 最輕
- 唯一硬性的先後：入口的權限要等資料那一個，因為 `GET /archived` 用 `list` 這個命令名稱問掛鉤，而 `list --archived` 是資料那一個變更加的
- G1 和 G2 合成一個：新事件要同批進 `event_resource` 那張表
- G4 拆成兩個：資料動的是引擎的輸出與 golden，權限動的是入口而且影響既有宿主的權限表，相容性影響要單獨寫進更新日誌
- 第 3、4、5 個彼此獨立，順序可以對調
- 結論帶 hold，五個變更都轉出之前記錄不自動封存
**Ruled out**: 平行做 — 五個變更都改 node-sdk 的規格與 `index.d.ts`，後封存的 delta 會對不上；G4 合成一個 — 權限那一半會擋掉既有宿主沒登記的名稱，和純粹加欄位的改動放在一起會分散審查的焦點
**Open**: 沒有待決定的項目。延後的三件事寫進結論：改技能內文的條件區塊、保留的鍵建立後改值與匯入時檢查、入口回應的欄位表

## Conclusion

**Decision**: 五項都做，分成五個變更依序一次做一個；G4 拆成資料與權限兩個，G1 和 G2 合成一個。
- **cut 1 `archived-list-and-detail-reads`**: 動詞讀得到宿主畫面要的五樣資料，全部做在引擎命令裡，本機、遠端連線、內嵌宿主得到同一份輸出
  - `list` 加 `--archived`：列出已封存的變更，每一筆的欄位照入口 `GET /archived` 現有的 `ArchivedItem`（帶日期的名稱、日期、名稱、任務數、建立的人、來源討論、兩個關卡的結果、自由欄位），再加 delta 的 capability 名稱
  - 已封存變更的文件不新增動詞：既有的 `artifact cat <artifact> --change <名稱或帶日期的名稱>` 已經讀得到（本機實跑確認）
  - `list --json --detail` 加 `fromDiscussions` 與 `createdBy`
  - `list --json --detail` 與 `show --json --detail` 的進行中關卡加 `rounds`；每個有工單的變更多讀一份文件，只有 `--detail` 才讀
  - `review show --json --detail`、`verify show --json --detail` 加 `content`（工單全文）；不直接加進 `--json`，因為審查與驗證技能讀它取最後一輪，直接加會每次把全文放進模型的上下文
  - `discuss list --json`、`discuss show --json` 加 `promotedTo`，沒有值時不出現。這是推翻 remote-remaining-gaps「引擎不輸出、由 server 端組裝」的決定：當時的理由是 CLI 的 JSON 逐位元不變，之後 `fields`、`createdBy`、`kind` 都採用「沒有值就不出現」已成慣例；而且內嵌宿主經 `dispatch` 直接讀動詞的輸出，沒有 server 那一層可以補
  - 相容性影響：只有 `promotedTo` 改既有輸出（已轉出的討論多一個鍵，沒有轉出的逐位元相同），golden 同批更新；其餘都在新旗標或 `--detail` 後面
  - 遠端連線的 CLI 走同一支輸出函式；線上格式缺的欄位（進行中關卡的輪數）同批以選填欄位補上
  - 例子：`list --json --detail` 的一筆有 `"createdBy": "Ann <ann@example.com>"`、`"fromDiscussions": ["two"]`、`"review": { "state": "inProgress", "rounds": 2 }`；`discuss list --json` 裡 `two` 那一筆多 `"promotedTo": ["add-two"]`；`list --archived --json` 回 `{ "archived": [{ "datedName": "2026-11-22-add-two", "date": "2026-11-22", "name": "add-two" }] }`
- **cut 2 `door-reads-ask-before-command`**: 入口裡八條不經過引擎命令的讀取都先問 `beforeCommand`，被拒絕時回 409、reason 為 refused、訊息是宿主給的原因
  - 八條：`GET /config`、`GET /board-order`、`GET /specs/{capability}/document`、`GET /search`、`GET /archived`、`GET /archived/{datedName}/capabilities`、`GET /archived/{datedName}/artifacts/{path}`、`POST /context`
  - 用既有的命令名稱：`GET /config` 用 `workflow-config-show`，`GET /specs/{capability}/document` 用 `show`，`GET /archived/{datedName}/artifacts/{path}` 用 `artifact-cat`，兩條列出已封存的路由用 `list`（cut 1 的 `--archived`）
  - 新增名稱並放進 `commandNames`（和 `put-board-order` 同一種做法）：`GET /search`、`POST /context`、`GET /board-order`
  - 範圍清單沒有這個專案時回 404、不問掛鉤維持不變（host-kit「範圍與能力由宿主的函式決定」的明文；不讓人從錯誤訊息猜出專案存不存在）
  - 要改寫 host-kit「HTTP 入口處理原版用戶端的請求」裡「經入口執行的引擎命令與看板順序的寫入經過掛鉤」那一句，改成入口的每一條讀寫都經過掛鉤
  - 相容性影響：只放行自己認得的名稱的宿主，升級後會擋掉 Speclink Desktop 的 `/context`、`/search`、`/board-order`，直到把新名稱加進權限表；`commandNames` 的測試會提醒。Speclink Desktop 在這三條收到 409 時的表現，提案時實測
  - 例子：某個身分在範圍清單裡，掛鉤對他一律拒絕、原因「帳號已停用」。現在 `GET /changes` 回 409 加原因，`GET /archived` 與 `POST /context` 回 200 加內容；改完三條都回 409、reason 為 refused、訊息「帳號已停用」
- **cut 3 `render-host-options-and-tool-verbs`**: `skills.render` 多兩個選項，引擎在前言多寫標準說明，技能內文不改；`agentTool` 加一個可以排除動詞的函式
  - `skills`（可用的技能名稱清單）：前言多一句「這個環境只提供清單裡的技能；清單外的不要載入、不要建議使用者執行，改說這一步由工程師在自己的電腦做」；不認得的技能名稱報錯
  - `checkout: false`（完全沒有 checkout，名稱暫定）：前言多一句「沒有程式碼、沒有 git 歷史；跳過讀程式碼與讀 git 的步驟，事實只用 speclink 的動詞查證，並在回答裡說明跳過了什麼」；和 `specInCheckout: true` 同時給時報錯
  - 不認得的選項鍵報錯，和 `createHost` 相同
  - 不做「有沒有提問工具」的開關：每一處 `AskUserQuestion` 都已有「沒有這個工具就用文字問並等回答」的退路
  - 「檔案搜尋」與「git 歷史」合成 `checkout` 一個開關
  - `agentTool` 加一個產生說明的函式，接受要排除的動詞；排除 `review` 會一併排除它底下的全部動詞；不認得的動詞報錯；原本的 `agentTool.description` 不變
  - `target: 'neutral'` 維持原意（五個內建 AI 工具以外的 harness），不另外加給內嵌宿主的 target
  - 例子：`skills.render('propose', { target: 'neutral', invocation: 'tool-call', specInCheckout: false, checkout: false, skills: ['discuss', 'trace', 'propose', 'ingest', 'analyze', 'config'] })` 的前言說明只有這六支技能、沒有程式碼與 git；多給打錯的鍵 `{ skils: [...] }` 直接丟出錯誤
- **cut 4 `reserved-field-prefixes`**: `createHost` 接受 `reservedFieldPrefixes`，請求要寫或刪這些前綴的鍵時由宿主套件拒絕，只有宿主自己帶的欄位（`HostDispatchCall.fields`、`HostRequestExtras.fields`）寫得了
  - 落點只有一處：`speclink-host/src/kit.rs` 的 `attach_fields`；`dispatch` 與 `handleRequest` 都經過它
  - 涵蓋三個建立命令（`new change`、`discuss new`、`discuss promote`）的 `--field`，`change field set|unset`、`discuss field set|unset`，以及入口的 `PUT /changes/{name}/fields` 與 `PUT /discussions/{slug}/fields`
  - 請求寫了保留的鍵就拒絕，即使宿主這次也帶了同名的鍵
  - 錯誤碼是 `refused`，訊息指出是哪一個鍵；入口回 409
  - 前綴不合自由欄位的鍵規則時 `createHost` 報錯；沒有設定的宿主行為和現在完全相同
  - free-fields「引擎不解讀任何鍵、不因鍵改變守門」維持成立：規則由宿主宣告、由宿主套件執行，引擎核心不認得任何鍵；在 host-kit 加一條需求寫明
  - 介面深度：不是新模組，是既有接縫（宿主套件合併欄位那一步）多一個檢查
  - 例子（`reservedFieldPrefixes: ['host.']`）：argv `['new', 'change', 'spoof', '--field', 'host.project=p2']` 被拒絕、沒有寫入；argv `['new', 'change', 'ok']` 加宿主帶的 `fields: { 'host.project': 'p1' }` 成功；argv `['change', 'field', 'set', 'ok', 'host.project=p2']` 與 `['change', 'field', 'unset', 'ok', 'host.project']` 被拒絕；`PUT /changes/ok/fields` 的內容含 `host.project` 回 409；`--field team=web` 不受影響
- **cut 5 `cross-resource-events-and-event-names`**: 一次提交改到幾個資源就送幾個事件，加守門測試，SDK 匯出 `eventNames` 與 `eventResource`
  - 維持「一個事件只指一個資源」
  - `new change --from-discussion` 多送 `discussion-promoted {slug, change}`，和 `discuss promote` 相同
  - `archive` 連帶封存幾份討論，就多送幾個 `discussion-archived {slug}`
  - `discard` 解開討論時多送一個新事件，暫名 `discussion-unlinked {slug, change}`
  - 再次 conclude 標記到的變更、`discuss link` 與 `discuss seal` 改到的變更，每個變更多送一個新事件，暫名 `change-provenance-changed {change}`
  - 事件只在真的寫入時送：`seal` 只有清掉過期標記時才寫變更，再次 conclude 回報的清單含已經標記、沒有重寫的變更，`link` 重複時不寫；結果要分得出「真的寫了」
  - 守門測試：`speclink-host/src/port_exec.rs` 組提交時同時拿到寫入的鍵與事件；對每個命令檢查「每個被寫到的進行中變更與討論，都有一個指到它的事件」。上面五種情況是讀程式碼找到的，測試負責找全並擋住以後的新命令
  - `eventNames`：全部事件名稱的凍結陣列，形狀與用法和 `commandNames` 相同；要不要含宿主套件自己的事件，提案時對照 `commandNames` 決定
  - `eventResource(name, payload)`：回傳 `{ scope, resourceId }`，就是 `speclink-host/src/commit.rs` 的 `event_resource`，和示範 server 的推送同一份
  - 新事件同批進這張表與 docs/verb-contract 的 Event stream 表格
  - 例子：`eventResource('discussion-promoted', { slug: 'two', change: 'add-two' })` 回 `{ scope: 'discussion', resourceId: 'two' }`；`eventResource('workflow-config-changed', { keys: ['tdd'] })` 回 `{ scope: 'unknown', resourceId: '' }`
- 順序：cut 2 要等 cut 1（`GET /archived` 用 cut 1 加的 `list --archived` 問掛鉤）；cut 3、4、5 彼此獨立，順序可以對調；不平行做
- 來源文件「這一輪確認足夠的部分」不要求改動，沒有再逐條驗證
**Rationale**: 五項的問題對照程式碼後都存在，而且每一項的修正都小、都只有一個落點，所以全部做。取捨的主軸是「宿主只需要理解一條路」：資料一律從動詞讀得到，權限一律經過 `beforeCommand`，事件一律涵蓋被寫到的資源，宿主保留的鍵一律只來自宿主的程式。G3 例外做小：來源文件寫的代價是模型「可能」做錯、沒有實際觀察到的錯誤，所以只加前言，不改技能內文。份量與順序照來源文件（G4 最重），唯一硬性的先後是入口的權限要用資料那一個變更加的命令。
**Rejected alternatives**:
- G1 讓一個事件對到多個資源 — 四種情況的 payload 沒有另一個資源的名稱，還是要補事件
- G2 匯出靜態對照表（名稱、指資源的鍵、類別）— 宿主還要自己取值，`change-archived` 這種特例要另外說明
- G3 改技能內文、新增條件區塊 — 六支技能都要改、輸出的組合加倍，而目前沒有實際觀察到的錯誤
- G3 只補文件、教宿主自己加前言 — 每個宿主各寫各的，用語和技能對不上
- G3 提問工具的開關 — 用文字問的退路已經在技能裡
- G4 工單全文直接加進 `review show --json` — 既有輸出每一筆都變，技能每次讀都帶全文
- G4 `promotedTo` 維持由 server 端組裝 — 內嵌宿主經 `dispatch` 讀動詞，沒有那一層
- G4 只補三條 `/archived` 問掛鉤 — 另外五條仍繞過掛鉤，包含回整個範圍文件的 `/context`
- G4 不改程式、寫明八條讀取由 `scopes` 決定 — 宿主仍要理解兩條權限路徑
- G4 範圍清單回 404 時也帶掛鉤的原因 — 規格明文是 404，帶原因會讓人猜出專案存在
- G5 讓 `beforeCommand` 看得出哪些欄位是宿主帶的 — 宿主要檢查三處參數，以後新命令換了參數名稱就會漏
- G5 只補文件 — 「把來源標成另一個他有權限的專案」擋不住，SDK 說明和行為的落差還在
- 平行做五個變更 — 都改 node-sdk 的規格與 `index.d.ts`，後封存的 delta 會對不上
- G4 合成一個變更 — 權限那一半會擋掉既有宿主沒登記的名稱，和純粹加欄位的改動放在一起會分散審查的焦點
**Deferred**:
- 改技能內文、沒有程式碼時把整段步驟換掉 — 等宿主實際看到模型做錯，再針對那一支技能處理
- 保留的鍵建立後要改值 — 宿主帶的欄位只在建立時寫入，目前沒有改值的需求
- 匯入時檢查保留的鍵 — 匯入寫的是整份文件，只有管理者能做
- 入口回應的欄位表 — cut 1 讓動詞讀得到同樣的資料之後，宿主不必依賴入口的欄位
- 在 `afterCommit` 直接給寫入的鍵 — 宿主的儲存 `commit` 函式已經收到，事件補齊並加守門測試後不需要第二條路
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion embedded-host-needs-7
