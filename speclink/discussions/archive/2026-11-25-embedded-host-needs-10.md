---
topic: 內嵌宿主第十輪的六項需求（討論代號 embedded-host-needs-10）。來源文件：~/Documents/GitHub/speclink-embedded-host-needs-10.md，接在討論 embedded-host-needs-9 之後；請把文件裡的每一項當成待檢驗的主張，對照程式碼確認。
slug: embedded-host-needs-10
status: promoted
created: 2026-11-25
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: task-title-convention, change-and-discussion-list-fields, desktop-uses-list-fields
---

# Discussion: 內嵌宿主第十輪的六項需求（討論代號 embedded-host-needs-10）。來源文件：~/Documents/GitHub/speclink-embedded-host-needs-10.md，接在討論 embedded-host-needs-9 之後；請把文件裡的每一項當成待檢驗的主張，對照程式碼確認。

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

內嵌宿主在自己的桌面用戶端做了一組唯讀的規格畫面（清單依階段分段、變更頁、正式規格與討論的全文），用 `@speclink/engine` 0.19.0（engine v1.61.0）在真實資料上操作一輪之後，逐項檢查引擎給的資料夠不夠用，回報六項：H1 變更沒有人看得懂的標題；H2 清單讀不到開工的時間與階段；H3 討論的清單沒有最後修改的時間；H4 建立的人沒有身分 id；H5 時間的欄位不夠算出各階段停了多久；H6 任務的範本沒有短標題的寫法。來源文件已經把每一項寫成待檢驗的主張，所以沒有 grill 階段，直接逐條對照程式碼與實測（暫時的本機專案，CLI 0.19.0）查證。相關的正式規格是 verb-contract（清單的詳細輸出、已封存變更的清單、任務取消勾選）、change-lifecycle（change metadata 時間欄位寫入時戳、對外輸出日期）、change-plan（`plan --json` 的 `stage`）、discussion-docs（討論記錄蓋建立者章）、client-protocol（變更清單的建立者與來源討論欄位）、workflow-schemas（tasks 的指示）、desktop-app（看板全尺寸卡的描述列、任務列）；目前沒有進行中的變更。
Source doc: ~/Documents/GitHub/speclink-embedded-host-needs-10.md
Prior discussions: embedded-host-needs-9, embedded-host-needs-7, remote-verb-parity, desktop-card-identity-and-meta, list-json-delta-capabilities-doc, gui-勾任務自動蓋開工章

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-11-25)

**Focus**: 來源文件 H1–H6 的每一條主張，對照程式碼與實測（暫時的本機專案，CLI 0.19.0）是否成立
**Position**: 六項的現況大致屬實，但有四處和文件不同或文件沒有看到；使用者確認查證無誤，並要求再查證一次、向宿主那一側問清楚實際情況之後才下結論。
- H1 屬實：`summary` 取提案 `## Why` 之後第一行的前 30 個字元、不看字詞邊界（`lifecycle/listing.rs` 的 `truncate_summary(t, 30)`）；實跑得到 `這個變更讓 Desktop Client Credentia…`；變更沒有其他標題類的欄位
- H1「30 是不是刻意的」：沒有規格或註解寫出理由；這個上限從第一個 commit 就有，人眼的 `speclink list` 每個變更一行（`• 名稱 [n/m] — 摘要`，`speclink-verbs` 的 `render/query.rs`），JSON 與 `GET /changes` 用同一個值；不帶 `--detail` 的 `list --json` 承諾逐位元不變
- H1「Desktop 用什麼當標題」：卡片標題是變更名稱，描述列是 `whyExcerpt`（Why 段落第一個非空行、不截斷，`apps/desktop/core/src/query.rs` 的 `why_excerpt`）；Desktop 不用 `summary`；desktop-app「看板全尺寸卡」寫明這個欄位不屬 CLI `--json` 對齊範圍；Desktop 的遠端資料源沒有這個欄位
- H2 屬實：引擎的 `stage()`（`lifecycle/model.rs`）與 `packages/ui/src/stage.ts` 的 `changeStage` 是同一條規則；`GET /changes` 與 `GET /changes/{name}` 有 `startedAt`（只有日期）；`list --json --detail`、`show --json --detail`、`status --json` 都沒有 `startedAt` 與 `stage`；依賴成環時 `plan` 整個失敗，`stage()` 不經過 `plan`
- H2 `status` 的完整清單：只有 `done`（至少一個任務而且全部勾完）與 `in-progress` 兩個值，`done` 等於 `stage` 的 `ready`
- H3 屬實：`discuss list --json` 與 `discuss show --json` 沒有 `modifiedAt`；討論的 `created` 在記錄裡只存日期；`Store` trait 沒有討論的修改時間，但 `port_view.rs` 每個鍵都有修改時間、本機有檔案時間
- H4 屬實：`new change`（`lifecycle/newcmd.rs`）、`discuss new` 與 `discuss promote`（`lifecycle/discuss.rs`）只寫 `created_by`；`claim` 寫 `claimed_by_id`；宿主套件每次呼叫都已經把呼叫者的 id 放進 `ExecutionContext.actor_id`（`speclink-host` 的 `kit.rs`）
- H4 與文件不同：`POST /changes` 收 `fields`，CLI 也有 `--field`；經 HTTP 入口建立的項目沒有宿主的 id，是因為呼叫端不會送宿主保留前綴的欄位
- H4 文件沒看到：`show --json` 沒有 `createdBy`
- H5 屬實但屬刻意：change-lifecycle「change metadata 時間欄位寫入時戳、對外輸出日期」規定現有輸出只給日期；沒有「任務全部完成」的時間與預計完成日
- H6 屬實：範本（`tasks.template.md`）與指示（`fork.schema.yaml` 的 tasks）只要求行為與驗證；`packages/ui` 的 `splitTaskText` 只認全形「：」與「驗證：」；`instructions apply --json` 的任務只有 `id`、`description`、`done`、`manual`，`id` 是流水號
- 決策樹：D1 清單給哪一句讓人認得出變更 → D2 `stage`／`startedAt` 放哪裡、`status` 要不要動 → D3 討論 `modifiedAt` 的來源 → D4 `createdById` 由誰寫、出現在哪些輸出 → D5 完整時間、任務全部完成的時間、預計完成日 → D6 短標題的分隔符號、apply JSON 要不要帶群組 → D7 拆成幾個變更
**Open**: D1 H1 用 `whyExcerpt` 或標題欄位；D2 H2 兩個鍵的落點與 `status`；D3 H3 `modifiedAt` 的來源；D4 H4 寫入點與輸出點；D5 H5 三個小項；D6 H6 分隔符號與 `group`；D7 拆成幾個變更；宿主那一側的實際情況（等回覆）

### Round 2 — assumptions (2026-11-25)

**Focus**: 第二次查證有沒有推翻第一輪的事實或建議
**Position**: 第一輪的事實全部成立；第二次查證找到三處會改到建議的地方，都在 D1 與 D5。
- D5「`task undone` 時清掉完成時間」和正式規格衝突：verb-contract 規定取消勾選是純狀態翻轉，不寫 touched 記錄、不改開工標記（`lifecycle/tasks.rs` 的 `uncomplete` 只寫回 tasks.md）
- D5「Desktop 勾任務走同一個引擎函式」只對一半：單一勾選走 `tasks::complete`，但「全部標完成」走 `apps/desktop/core/src/manage.rs` 的 `set_all_tasks_at`，自己翻勾選框、自己補側效；完成時間要寫在兩個地方，手動改 tasks.md 的不會有
- D5 不能用完成證據代替：evidence 只在這次完成有新的髒檔時才記一筆（verify-evidence），沒有 checkout 的宿主不會有
- D1 多一個好處：Desktop 的遠端資料源沒有 `whyExcerpt`（`remote.rs` 沒有這個欄位），HTTP 回應帶上之後遠端專案的卡片也有描述列
- 其餘再確認：`discuss promote` 經 `newcmd::new_change` 建立變更、同樣只收顯示名稱；`GET /changes` 每一筆都讀 metadata 補 `startedAt`；需求名稱的覆蓋檢查比對整行任務文字（`quality/analyzer.rs` 的 `req_covered`），短標題的寫法不影響它；這個 repo 的 269 份已封存提案，Why 第一行的長度中位數 294 個字元，30 個字元以內的只有 1 份
**Ruled out**: 勾完最後一個任務的時間在 `task undone` 時清掉 — 違反取消勾選是純狀態翻轉；用 evidence 的 `recordedAt` 推算任務全部完成的時間 — 沒有髒檔就沒有記錄
**Open**: D1–D7 都還沒定；宿主那一側的實際情況（等回覆）

### Round 3 — interview (2026-11-25)

**Focus**: 宿主那一側的實際情況會不會改變六項的做法（以書面問了提出這份文件的一方，回覆逐題標明實測、讀碼或推測）
**Position**: 回覆確認了 H2、H3、H4 的做法，縮小了 H5，並改變 H1 與 H6 的範圍。
- H1：提案 Why 的第一行是一整段，不是一句標題（宿主的真實資料中位數約 190 個字元，和這個 repo 的量測一致）；`whyExcerpt` 只能當說明那一行，不能當大標題；H1 是提出方自己看資料時的觀察，看畫面的人沒有對標題表示過意見
- H2：畫面只經 `dispatch` 跑動詞、不經 HTTP 入口；判斷階段只要 `stage`，`startedAt` 用來畫時程；只有日期夠用；宿主讀 JSON 只取用到的鍵，加鍵是安全的
- H3：宿主的儲存對進行中與已封存的討論都有修改時間；現在就會拿來排序與顯示最後更新；從 repo 匯入的文件，修改時間是匯入的時間（和變更的 `modifiedAt` 同一個限制）；已封存的討論現在不需要
- H4：身分 id 是宿主給的識別字，讀得到專案的人本來就看得到；`dispatch` 的呼叫者是使用者本人；舊項目不回填可以接受；只需要建立者的 id
- H5：以天計就夠，「`created` 回完整時間」可以拿掉；用到階段時間的總覽已經有提案、還沒實作；手動勾完而沒有完成時間可以接受，畫面當成不知道；預計完成日用自由欄位
- H6：任務是繁體中文；宿主的任務頁讀 `show --json` 的 `tasks` 全文、自己解析 Markdown，沒有用 `instructions apply --json` 當資料來源；任務不能收合是六項裡唯一由看畫面的人實際指出的問題；每個範圍各自在 `rules.tasks` 加規則行得通，但每個範圍都要加
- 回覆另外提到一個不在文件裡的缺口：沒有只回一個變更摘要的動詞（`show --json` 沒有 `status` 與任務數）；回覆說 `show --json` 也沒有認領與關卡，這點和程式碼不符（`render/query.rs` 的 `show_json` 有記錄時會輸出 `claimed`、`review`、`verify`）
**Ruled out**: `instructions apply --json` 的任務加 `group` — 沒有畫面讀這個輸出，只多群組也不夠畫任務頁；把 `whyExcerpt` 當成 H1 的全部答案 — 它是一整段，解決的是說明被截在 30 個字元，不是沒有標題
**Open**: D1 標題欄位現在做或延後；D5 任務全部完成的時間的寫入規則；D6 分隔符號與驗證段的標記；D7 拆成幾個變更

### Round 4 — interview (2026-11-25)

**Focus**: 這一輪新增的資料，Speclink Desktop 要不要一併用上
**Position**: 使用者決定 Desktop 這一輪一併做三項，第四項（階段改讀引擎的值）要先聽說明再決定。
- 一併做：變更詳情頁的出身那一行接「就緒日」（`readyAt`），做法和開工日相同（`ChangeDetailPage.tsx` 的 `startedLabel`）
- 一併做：卡片顯示最後更新；討論卡片用新的 `modifiedAt`，變更卡片現在也沒有這一項，為了一致一起加；卡片順序仍然是拖排的順序
- 一併做：用 `createdById` 做「我建立的」篩選；只有遠端專案有 id，本機專案（git 身分）沒有
- 原本就在結論裡的三處不變：`splitTaskText` 認三組標記、`why_excerpt` 改呼叫引擎、「全部標完成」寫 `ready_at`
- 階段的規則現在有三份、內容相同：引擎的 `model::stage`、`packages/ui/src/stage.ts` 的 `changeStage`（9 個檔案呼叫）、`apps/desktop/src-tauri/src/remote.rs` 的 `change_stage`（遠端專案拖排時判斷欄）
**Open**: Desktop 的階段要不要改讀引擎的 `stage`；「我建立的」篩選在本機專案怎麼表現；結論要重寫，把 Desktop 的三項放進去（放進 cut 2 或另開一個變更）

### Round 5 — interview (2026-11-25)

**Focus**: Desktop 的階段要不要改讀引擎給的 `stage`
**Position**: 要，一併做：`packages/ui/src/stage.ts` 的 `changeStage` 與 `apps/desktop/src-tauri/src/remote.rs` 的 `change_stage` 改成清單有 `stage` 就用它、沒有才自己算。
- 呼叫這兩個函式的地方不改，畫面不變
- 自己算的那一段留著當退路：連到舊版 Server 時清單沒有 `stage`
- 使用者選它的理由是之後階段的規則只改引擎一處；這條規則改過一次，當時引擎以外的兩份都要跟著改
- Desktop 本機與遠端的清單資料都要帶上 `stage`
**Ruled out**: 拿掉 Desktop 自己的判斷、只讀引擎的值 — 連到舊版 Server 的 Desktop 會沒有階段
**Open**: 「用 id 辨認建立者」在 Desktop 的形狀（看板已經有依名稱的建立者篩選）；結論要重寫，把 Desktop 的四項放進去

### Round 6 — interview (2026-11-25)

**Focus**: 「用 id 辨認建立者」在 Desktop 做成哪一種
**Position**: 把看板既有的「建立者」篩選改成用 id 認人，不加「我建立的」。
- 現況：`packages/ui/src/search.ts` 的 `BoardFilters.createdBy` 用「名字 <信箱>」全等比對，下拉列出看板上出現過的每個名稱（`KanbanBoard.tsx`）
- 改成：有 `createdById` 的卡片依 id 分組，名稱只當標籤；沒有 id 的（本機專案、舊項目）照舊用名稱
- 效果：改過名稱的人不會被拆成兩個選項，同名的兩個人不會被合成一個
- Desktop 的變更與討論清單資料要帶上 `createdById`
**Ruled out**: 加「我建立的」選項 — whoami 的回應只有名稱與 handle、沒有 id（`speclink-protocol` 的 `WhoamiUser`），要改協定與 HTTP 入口，而既有的下拉本來就能選到自己；只加「我建立的」、其他人照舊用名稱 — 同樣要改 whoami，改名與同名的問題還在
**Open**: 結論要重寫，把 Desktop 的四項（就緒日、卡片顯示最後更新、階段改讀引擎的值、建立者篩選用 id）放進去

## Conclusion

**Decision**: 六項裡做五項半，Desktop 一併用上新資料，拆成三個變更：任務的短標題寫法、變更與討論的清單欄位、Desktop 用上新欄位；標題欄位、結構化的任務清單與完整時間的鍵都不在這一輪做。
- **cut 1 `task-title-convention`**: 任務的指示與範本建議「短標題＋分隔符號＋行為＋驗證段」的寫法，Desktop 的切分規則跟著認得（H6；名稱提案時可改）
  - 寫法放在引擎的 tasks 指示（`fork.schema.yaml`）與範本（`tasks.template.md`）兩處：指示說明規則與例子，範本的佔位文字改成同一個形狀；每個範圍都拿得到，不用各自在 `rules.tasks` 加規則
  - 分隔符號與驗證段的標記跟著 `locale`：繁體中文用全形「：」與「驗證：」，日文用全形「：」與對應的詞，英文用 `: ` 與對應的詞；確切的詞提案時定
  - `packages/ui` 的 `splitTaskText` 認得這三組標記；行內程式碼裡的冒號不算分隔
  - 使用者文件寫明任務文字怎麼切成編號、標題、內文與驗證段，內嵌的宿主照這份規則解析 `show --json` 的 `tasks`，不用猜
  - 已經寫好的任務不遷移；需求名稱的覆蓋檢查比對整行任務文字，不受影響
  - `instructions apply --json` 不加群組或其他鍵
  - 例（`locale: tw`）：`- [ ] 1.1 匯出 CSV：export --csv 寫出含標題列的檔案。驗證：cargo test -p demo export_csv` 切成編號 `1.1`、標題「匯出 CSV」、內文「export --csv 寫出含標題列的檔案。」、驗證「cargo test -p demo export_csv」
- **cut 2 `change-and-discussion-list-fields`**: 動詞的清單與單項輸出補上說明、階段、開工日、建立者 id、任務全部完成的日期與討論的修改時間（H1–H5；名稱提案時可改）
  - H1：`list --json --detail` 與 `show --json` 加 `whyExcerpt`：提案 Why 段落第一個非空行、不截斷，沒有就不出現；規則搬進 speclink-core，Desktop 的 `why_excerpt` 改成呼叫它；`summary` 與它的 30 個字元上限不動
  - H1：`whyExcerpt` 是說明那一行，不是標題；宿主與 Desktop 一樣用名稱當標題
  - H2：同兩個輸出加 `stage`（`proposed`／`in-progress`／`ready`，直接用 `model::stage`，不經 `plan`，依賴成環時照樣有值）與 `startedAt`（只有日期，沒開工就不出現）
  - H2：`status` 不改名、不合併；verb-contract 文件寫明它只有 `done` 與 `in-progress` 兩個值、`done` 等於 `stage` 的 `ready`；`status --json` 不加鍵
  - H3：`discuss list --json`（含 `--archived`）與 `discuss show --json` 的 `info` 加 `modifiedAt`，意思和變更清單相同：儲存回報的最後修改時間，RFC 3339 的 UTC 時間，儲存答不出來就不出現；`Store` 加一個方法、預設答不出來，本機用檔案時間，經儲存接口時用鍵的修改時間
  - H4：`new change`、`discuss new`、`discuss promote` 在執行環境有呼叫者的 id 時多寫 `created_by_id`；`createdById` 出現在 `list --json --detail`、`show --json`、`discuss list --json`、`discuss show --json` 與 `list --archived --json`；沒有 id（本機 CLI、匯入的內容、舊項目）就不出現，不用 null、不回填
  - H4：`show --json` 同時補上 `createdBy`
  - H5：新增任務全部完成的時間（metadata 鍵暫名 `ready_at`、輸出 `readyAt`）：一次任務完成讓全部任務都勾完時寫入，再次發生時覆寫；`task undone` 不動它；輸出只在目前全部勾完時出現、只給日期；寫入點是 `tasks::complete` 與 Desktop 的「全部標完成」，兩處共用 core 的一個函式；手動改 tasks.md 勾完的沒有這個時間
  - H5：`created` 維持只給日期，不加完整時間的鍵；預計完成日不做，宿主用自由欄位
  - 新欄位同時加進 HTTP 回應（變更清單、單一變更、討論清單與單一討論），經遠端連線的 CLI 輸出與本機相同；Desktop 的遠端資料源因此也有 `whyExcerpt`，遠端專案的卡片多出描述列
  - 例：`list --json --detail` 的一筆 `{"completedTasks":3,"created":"2026-11-24","createdBy":"Ann <ann@example.com>","createdById":"u_1","modifiedAt":"2026-11-25T14:44:50Z","name":"add-demo","readyAt":"2026-11-25","stage":"ready","startedAt":"2026-11-24","status":"done","summary":"這個變更讓 Desktop Client Credentia…","totalTasks":3,"whyExcerpt":"這個變更讓 Desktop Client Credentials 登入可以在沒有瀏覽器的機器上完成。"}`；`discuss list --json` 的一筆多 `"createdById":"u_1"` 與 `"modifiedAt":"2026-11-25T14:50:00Z"`
- **cut 3 `desktop-uses-list-fields`**: Desktop 用上 cut 2 的新欄位：就緒日、卡片的最後更新、階段讀引擎的值、建立者篩選用 id（相依 cut 2；名稱提案時可改）
  - 就緒日：變更詳情頁左欄的出身那一行，在建立日、開工日之後接就緒日（`readyAt`），做法和開工日相同；沒有值時整段不出現
  - 最後更新：看板的變更卡片與討論卡片顯示最後更新的相對時間（取 `modifiedAt`，沿用規格頁的天級寫法：今天／昨天／N 天前）；沒有值時不出現；卡片順序仍然是拖排的順序，不改
  - 階段：`packages/ui/src/stage.ts` 的 `changeStage` 與 `apps/desktop/src-tauri/src/remote.rs` 的 `change_stage` 改成清單有 `stage` 就用它、沒有才自己算；呼叫端與畫面不變；自己算的那一段留著，連到舊版 Server 時用
  - 建立者篩選：看板既有的「建立者」下拉改成用 id 認人：有 `createdById` 的卡片依 id 分組、名稱只當標籤，沒有 id 的照舊用「名字 <信箱>」全等比對；不加「我建立的」
  - Desktop 本機與遠端的清單資料帶上 `stage`、`createdById` 與討論的 `modifiedAt`，單一變更的資料帶上 `readyAt`
  - 例：一張討論卡片今天加了一輪，meta 列在「N 輪」旁顯示「今天」；同一個人先後用兩個名稱建立的兩張遠端卡片，下拉只出現一個選項，選它時兩張都留下
- 來源文件的主張大致屬實；不同的地方：`POST /changes` 收 `fields`（缺的是保留前綴只有宿主寫得了）、`show --json` 連 `createdBy` 都沒有、`status` 有 `done` 與 `in-progress` 兩個值、`created` 只給日期是正式規格的規定
- 順序：cut 1 和另外兩個改的檔案大致不重疊；cut 3 要等 cut 2；最後轉出的那一個帶 `--last`
- 要改的正式規格（提案時確認）：workflow-schemas 與 desktop-app 的任務列（cut 1）；verb-contract「清單的詳細輸出」「已封存變更的清單」與 `show` 的欄位、change-lifecycle 的時間欄位與任務完成、discussion-docs「討論記錄蓋建立者章」、client-protocol 與 server-verb-api 的清單欄位、store-abstraction、desktop-app 的描述列來源與「全部標完成」（cut 2）；desktop-app 的看板卡片、詳情頁出身列、「看板搜尋過濾卡片」與「看板欄位由生命週期標記驅動」、client-protocol 的清單欄位（cut 3）
- 來源文件「這一輪確認足夠的部分」不要求改動，沒有逐條再驗證
**Rationale**: 取捨的主軸是只補引擎已經知道、動詞卻沒有說出來的事實，不新增要人維護的資料：階段的規則、開工日、呼叫者的 id、Why 的第一行與儲存的修改時間都已經在引擎或儲存裡，HTTP 入口與 Desktop 也已經在用，內嵌的宿主經 `dispatch` 只讀得到動詞，所以缺口是動詞和其他出口不對等。唯一新記下的資料是任務全部完成的時間，因為它事後補不回來，而且用到它的總覽已經在規劃。標題欄位是另一種東西：Why 的第一行實測是一整段，要短標題就得新增一個由人或 Agent 維護的欄位，而看畫面的人沒有提過這個需求，所以這一輪只把說明那一行補齊。任務的寫法選擇放進引擎的指示而不是各範圍的規則，因為同一個問題在 Desktop 的每個使用者身上都有，而且它是六項裡唯一由看畫面的人實際指出的。Desktop 那四項是使用者決定一併做的：新欄位既然由引擎給出，官方的畫面跟著用，階段的規則之後也只要改引擎一處；它們獨立成一個變更，是因為都要等引擎與 HTTP 回應先有這些欄位，分開也讓介面的改動單獨驗收。
**Rejected alternatives**:
- 改 `summary` 的截斷規則（不斷在字中間或放寬上限） — 不帶 `--detail` 的 `list --json` 與人眼的 `list` 承諾逐位元不變
- `stage` 取自 `plan` — 依賴成環時 `plan` 整個失敗
- 合併或改名 `status` — 凍結的輸出，既有的讀取端靠它
- 維持 remote-verb-parity 的否決（不由引擎輸出 stage） — 當時的理由是會變成第三份實作；之後 change-plan 已把 `stage()` 放進引擎，動詞直接用它
- 討論的 `modifiedAt` 取最後一輪的日期 — 只有日期、結論沒有日期，和變更的 `modifiedAt` 意思不同
- 沒有 id 時輸出 null、或回填舊項目的 `created_by_id` — 違反「沒有值就不出現」的慣例；舊項目沒有可靠的來源
- `created` 改回完整時間，或另加完整時間的鍵 — 前者違反 change-lifecycle 的輸出規定；用到時間的畫面以天計，日期就夠
- 任務全部完成的時間在 `task undone` 時清掉 — verb-contract 規定取消勾選是純狀態翻轉
- 用 evidence 的 `recordedAt` 推算任務全部完成的時間 — 沒有新的髒檔就沒有記錄，沒有 checkout 的宿主不會有
- 預計完成日成為正式欄位 — 自由欄位已經夠用
- 短標題只靠各範圍的 `rules.tasks` — 每個範圍都要自己加，匯入的範圍由別人維護
- 短標題獨立一行、說明另起一行 — 任務的解析、stable ID 與覆蓋檢查都以勾選框那一行為單位
- 只認全形冒號 — 英文寫的任務照樣不能收合
- `instructions apply --json` 的任務加 `group` — 沒有畫面讀這個輸出，只多群組也不夠畫任務頁
- Desktop 拿掉自己的階段判斷、只讀引擎的值 — 連到舊版 Server 時清單沒有 `stage`
- Desktop 加「我建立的」篩選 — whoami 的回應沒有 id，要改協定與 HTTP 入口，而既有的建立者下拉本來就能選到自己
- 看板改成依最後更新排序 — 卡片順序以拖排的順序為準（board-card-order）
- Desktop 的四項併進 cut 2 — 引擎、協定與介面一次改，變更太大；介面要等欄位先有
- 全部合成一個變更 — 任務的寫法和清單欄位改的是不同的檔案與規格，分開可以各自驗收
**Deferred**:
- 變更的標題欄位（propose 時寫、之後可改） — Why 的第一行是一整段，當不了標題；但看畫面的人沒有提過，等有人實際需要再開
- 引擎輸出結構化的任務清單（群組、編號、標題、內文、驗證段） — 現在只有 Desktop 與一個宿主各自解析，先把切分規則寫成文件；兩邊的解析出現分歧時再做
- `show --json` 補上 `status` 與任務數，讓單一變更不用讀整份清單 — 不在這份文件的六項裡，提出方還沒有正式提出
- `list --archived --json` 的 `whyExcerpt` — 每次清單都要多讀每個已封存變更的提案（Desktop 為此做了快取），等總覽真的要用再做
- 開工者的 id（`startedById`） — 目前沒有畫面要用
- 匯入的文件，`modifiedAt` 是匯入的時間 — 和變更清單的 `modifiedAt` 同一個限制，文件已寫明它只是提示；匯入時保留原時間是匯入功能的事
- 本機專案的最後更新取自檔案時間，checkout 或複製也會改到它 — 卡片上的「N 天前」因此可能偏新；先沿用規格頁同樣的來源，有人被誤導再處理
**Capture to**: proposal | design | spec | tasks
**Next**: /speclink-propose --from-discussion embedded-host-needs-10
