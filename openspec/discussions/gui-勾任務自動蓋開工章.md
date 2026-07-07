---
topic: GUI 勾任務自動蓋開工章
slug: gui-勾任務自動蓋開工章
status: promoted
promoted_to: task-done-implies-started
created: 2026-07-06
---

# Discussion: GUI 勾任務自動蓋開工章

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：使用者回報本地模式看板「開工後狀態不更新」。調查分成兩層：①即時刷新失效是 watcher 掛錯 root 的已知缺陷（desktop-watcher-root-fix，已修復）；②但 desktop-discussion-board 做了 2/14 個任務、meta 卻無 started_at——因為開工章只由 /speclink-apply 的 `speclink in-progress add` 蓋，GUI 勾任務（set_task_done_at）不蓋章，卡片永遠停在「提案中」。使用者裁定：GUI 勾選也應該自動蓋開工章。

模式：assumptions——相關源碼充足（packages/ui/src/stage.ts、apps/desktop/core/src/manage.rs、crates/speclink-core/src/inprogress.rs、crates/speclink-cli/src/commands.rs）。

相關 changes/specs：desktop-watcher-root-fix（即時刷新修復，5/5 待歸檔）、desktop-board-parity（已歸檔，stage 派生「誰開工了才算進行中」的出處）、desktop-app spec（看板與監看需求所在）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-06)

**Focus**: GUI 勾任務該不該自動蓋開工章（started_at）、蓋章縫隙落在哪一層
**Position**: 該蓋（使用者裁定）。蓋章位置定在桌面 core 的 set_task_done_at（apps/desktop/core/src/manage.rs:46）內、僅於 done=true 時呼叫共用的 speclink_core::inprogress::add——重用其冪等、首章保留、YAML 注入清洗；started_by 取 git identity（與 CLI 同源 core::util::git_identity），started_with 缺席（人的點擊無 agent 可歸屬，遵循「不能歸屬即缺席」規則）。CLI `task done` 不動——其檔案副作用受 Spectra parity 保護，agent 流程已由 apply 的顯式 in-progress add 補位。
**Ruled out**: ① stage.ts 改以 completedTasks>0 推斷進行中——顯示層推斷不落盤、丟失 started_by 歸屬、看板與 meta（隨 repo 移動的唯一事實源）分歧；② 蓋章下沉到 speclink-core 的 task 層共用——會讓 CLI task done 的檔案副作用偏離 Spectra 基線，違反 parity 紅線。
**Open**: 拖曳排序（move_task_at）算不算開工？前端 setTaskDone 後靠既有 refresh／watcher 事件反映移欄是否足夠（無需新接線）？

### Round 2 — assumptions (2026-07-06)

**Focus**: 使用者追問——所有操作都透過 CLI 引擎執行是否可行？（挑戰第 1 輪 Ruled out ②）
**Position**: 立場修正。「CLI 引擎」拆成兩種讀法：(a) GUI 以子行程呼叫 CLI exe——不建議；(b) 「任務完成 ⇒ 蓋開工章」下沉為共用引擎（speclink-core）層級的規則，CLI 與 GUI 兩個寫入路徑都呼叫同一原語 inprogress::add——採納，優於第 1 輪的桌面層方案。新事證：① CLI `task done` 本就走引擎（commands.rs:1562 呼叫 core::tasks::mark_done）；② apply 正典勾任務路徑就是 CLI `task done`（speclink-apply/SKILL.md:185），開工章是可被跳過的獨立步驟——desktop-discussion-board 2/14 無章即為實證，只修 GUI 會留著 agent 主路徑的同一個洞；③ parity 重估：inprogress::add 靜默，CLI 輸出（stdout/stderr/exit）不變，輸出基線不受影響；meta 內生命週期標記本就是 speclink 對 Spectra 的既有擴充（Spectra 存 host-local SQLite、零讀者），不存在要保護的 Spectra 檔案契約；自我基線雙沙盒是防意外回歸，蓄意行為變更更新基線即可。
**Ruled out**: ① GUI 子行程呼叫 CLI exe——行程啟動開銷與 Windows 主控台閃現、GUI 與 CLI exe 版本漂移、錯誤處理退化為解析文字、CLI 有 remote_ctx 路由（GUI 可能因設定意外走遠端）、且 CLI 今日不蓋章，殼呼叫本身不解決問題還得組合兩條指令；② 把桌面任務編輯整套統一進引擎（複用 mark_done）——桌面面需要 uncheck＋拖曳排序的超集，引擎無對應原語，整併屬 YAGNI，留待 web server 需要時再議。
**Open**: GUI 勾選路徑是否也要記 touched-files（CLI task done 會記 git dirty 檔案；GUI 補勾當下 dirty 集合可能與任務無關）？直接編輯 tasks.md（人工／agent Edit）繞過所有程式路徑的殘洞——是否以讀取端混合派生（completedTasks>0 亦視為進行中）補上，或先擱置？

### Round 3 — assumptions (2026-07-06)

**Focus**: 兩個開放點的裁定——GUI touched-files 與「手改 tasks.md 殘洞」的最嚴謹解法
**Position**: ① GUI 勾選也記 touched-files（使用者裁定）。為免 CLI 與桌面兩處重複「mark_done＋寫回＋touched-record＋蓋開工章」的四步組合，引擎（speclink-core 的 tasks 層）新增單一任務完成協作函式；CLI cmd_task 與桌面 set_task_done_at（done=true）成為薄呼叫端——CLI 保留 Spectra 錯誤順序與輸出包裝，桌面保留 ordinal 對映與 uncheck/move 本地面。接縫深度：函式後藏四件實際行為，刪除測試成立（刪掉它兩端就得重複組合）。② 殘洞最嚴謹解法＝顯示用讀取端派生閉合、記錄絕不事後補章：stage 派生加入 completedTasks>0 → 進行中（ready 欄本就是純任務派生，設計一致），由構造涵蓋所有繞道——編輯器手改、agent 直改、git pull 拉進他人手改；meta 開工章維持「行動當下由工具誠實記錄」的事件語意——事後自動補章會偽造歸屬與日期（偵測日≠開工日、本機 git identity≠實際動手者），違反專案既有「不能歸屬即缺席」規則。兩機制互補：章在而任務 0 完成（apply 開工即蓋）靠章移欄；有進度而無章靠派生移欄、抽屜不顯示開工歸屬列。
**Ruled out**: watcher 偵測 tasks.md 自動補章——僅 app 執行中有效、讀寫回饋環風險、偽造歸屬；引擎 list 讀取路徑自我修復寫回——讀操作產生檔案副作用，破壞唯讀期望與 CI/git status；git hook 對帳——安裝負擔且繞得過。
**Open**: （無——例示已定：手改勾 2/14 的 change 出現在進行中欄、抽屜無「開工」列；其後任一工具路徑首次勾任務即補上首章與歸屬）

## Conclusion

**Decision**: 「完成任務」的完整語意——勾章、touched-files 記錄、蓋開工章（started_at）——下沉為引擎（speclink-core tasks 層）的單一任務完成協作函式，CLI `task done` 與桌面 GUI 勾選（done=true）都成為它的薄呼叫端。看板顯示層另於 stage 派生加入 completedTasks>0 → 進行中，涵蓋手改 tasks.md、agent 直改、git pull 等繞過工具的路徑；絕不事後自動補章——缺席的歸屬維持缺席。
**Rationale**: agent 主路徑（apply → CLI task done）與 GUI 路徑必須共用同一語意，規則只寫一次；顯示正確性只有讀取端派生能由構造涵蓋所有寫入者，事件歸屬只有行動當下的工具能誠實記錄——兩機制互補。parity 無虞：inprogress::add 靜默，CLI 輸出基線不變；meta 生命週期標記本為 speclink 對 Spectra 的既有擴充（Spectra 存 host-local SQLite、零讀者），無檔案契約要保護。
**Rejected alternatives**: GUI 子行程呼叫 CLI exe（版本漂移、remote_ctx 意外路由、主控台閃現、CLI 今日不蓋章故仍需組合）；只修 GUI 桌面層（CLI 路徑同洞再發——desktop-discussion-board 2/14 無章實證）；watcher／list 讀取路徑事後補章（偽造歸屬與日期、讀操作寫檔）；桌面任務編輯整套併入引擎（uncheck/move 超集無引擎原語，YAGNI）。
**Deferred**: drift/verify 對「有進度無章」的診斷提示（真的咬到再議）；web server 端任務操作共用同一協作函式（web-server-postgres 範圍內自然發生）。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion gui-勾任務自動蓋開工章
