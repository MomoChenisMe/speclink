---
topic: 桌面 archive 缺少 evidence gate 的放行入口 → evidence gate 誤擋正常流程
slug: evidence-gate-false-blocks
status: promoted
promoted_to: evidence-home-and-trace-slim
created: 2026-08-04
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 桌面 archive 缺少 evidence gate 的放行入口 → evidence gate 誤擋正常流程

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

evidence-home-and-trace-slim 剛把「本地 archive 強制 evidence gate」實作落地（尚未封存）。原始題目是桌面 App 沒有 `--waive-evidence` 的對應入口（CLI 拒絕訊息在桌面只會變成一條單行 toast，三行重跑指令塞不進去也不能複製）。討論開場的沙盒探針卻發現問題在更前面：gate 的觸發條件會誤擋兩種完全照規矩走的流程，桌面按鈕只是逃生門，門本身才是病灶。

模式：assumptions——相關程式碼充足（apps/desktop/src/store.ts、App.tsx、packages/ui/src/components/ReviewArchiveDialog.tsx、apps/desktop/core/src/verbs.rs、crates/speclink-core/src/tasks.rs、crates/speclink-core/src/evidence.rs），另以兩個沙盒探針驗證行為。

相關脈絡：討論 post-archive-spec-value 是 gate 的出處；change evidence-home-and-trace-slim 是 gate 的實作載體；carry-review 三選項對話框（ReviewArchiveDialog + archive_carry_review IPC）是桌面放行入口的既有前例。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-04)

**Focus**: 先修 gate 的判定規則，還是先補桌面放行按鈕？
**Position**: 使用者裁定走「先修門」——gate 觸發條件本身會誤擋好人，先補按鈕等於幫有問題的門開逃生口、養成一律放行的習慣。
- 探針一（缺席誤擋）：純規格 change 兩個任務都乖乖走 `task done`，但 evidence 只在「有未認領的髒程式檔」時才寫（tasks.rs:302-326，`git_changed_files` 排除 openspec/），整個 change 零證據 → 封存被擋，拒絕訊息卻叫人「用 task done 完成任務」——使用者剛做過。
- 探針二（stale 死路）：先 commit 再勾任務（好習慣）→ 勾的當下樹乾淨、不寫新 entry，但勾任務改了 tasks.md → 舊 entry 的 tasks basis 對不上 → 判 stale；照拒絕訊息「重跑受影響任務」重跑，樹仍乾淨、仍不寫 entry，**永遠 stale**，唯一出口是 --waive-evidence。
- 桌面面的事實仍成立：失敗只有單槽 toast（store.ts:78 showFailureToast），可複製的重跑指令與旗標說明進不了 GUI；但這是次因。
**Ruled out**: 先補桌面按鈕（B 案）——死路對 CLI 使用者同樣存在，按鈕治標；且會把「每次都按放行」變成常態，gate 形同虛設。
**Open**: 修門的具體形狀（缺席與 stale 兩種誤擋怎麼解）；`--mark-tasks-complete` 與 gate 的交互（標完即改 tasks.md，恐必然 stale）；門修好後桌面還需不需要放行入口。

### Round 2 — assumptions (2026-08-04)

**Focus**: 修門提案(每勾必記)被逐案檢視後,上拉到機制層——touched/evidence 這整套追蹤還該不該存在?並以 OpenSpec 上游做對照。
**Position**: 對照確認 speclink 的追蹤機制全屬自建,上游零機制;touched 的三個歷史用途已拔掉一個、剩兩個有真實讀者。
- 修門提案(task done 每勾必記一筆 entry、空檔案清單合法)可解缺席與 stale 兩種誤擋,但逐案檢視(跑測試的任務、--mark-tasks-complete、桌面 toast)顯示補丁鏈仍在延長,使用者質疑機制本身。
- OpenSpec 上游對照(讀 src/core/archive.ts 原始碼):archive=驗證+delta 合併+搬目錄,零溯源注入(無 @trace)、零檔案追蹤(無 touched/evidence)、task 勾選零副作用檔;檔案歸屬完全交給 git 與 agent。@trace 與 touched 都是 speclink 自建。
- touched.json 的三個歷史用途:①commit skill 檔案歸屬(原始動機,仍有真實需求——平行 change 選擇性提交)②@trace code 清單來源(本刀已拔——兩欄 trace,此用途消滅)③drift/in-progress-remove 的工作痕跡判定(仍在用)。
- 搬家緣由回顧(proposal Why):.speclink/ 被 gitignore→記錄不進版控、跨機器不存在、封存後被技能刪除;@trace code 清單因此成了唯一被 commit 的歸屬紀錄,偏偏它又是髒檔猜測與平行 session 污染的根源。搬進 change 目錄=記錄自己進 git、隨封存移動、刪除步驟消失——此價值獨立於 gate,不受拆門影響。
- 桌面 UI 仍消費 @trace 的 source 欄位(packages/ui/src/trace.ts、SpecDrawer/SpecList),兩欄 trace 有讀者、保留。
**Ruled out**: 丙案(續修門/每勾必記)——補丁鏈證明機制在本機單人場景撐不起自己;全面向 OpenSpec 看齊(連 v1 檔案清單也拆)——會失去平行 change 選擇性提交的檔案歸屬,那是使用者的原始且仍然成立的需求。
**Open**: 甲(拆門留帳:gate+旗標全拆,v1+v2 記錄不動)vs 乙(連 v2 帳拆:退回 v1 純檔案清單,Phase 2 地基一併退場);拆門後桌面按鈕議題自然消滅待確認。

### Round 3 — assumptions (2026-08-04)

**Focus**: 拆門之後,帳裡的 basisDigests(三張 sha 指紋)還需不需要?
**Position**: 使用者定案甲′——門拆掉之外,帳裡的 sha 與指紋判定模組一併退場;帳收斂到「每一欄都有讀者的歷史事實」。
- sha 讀者盤點:門(唯一判斷者,拆)、drift(現場重算 current_basis_digests、從不回讀帳上存的 sha)、commit skill(只讀檔案清單)——拆門後 sha 零讀者。
- 「留給遠端版當地基」的理由收回:遠端的門應由 server 自記自判,client 本機自報的 sha 不可信;且實測遠端模式 task done 目前根本不寫證據,地基論不成立。
- 尺:只留「歷史事實」(誰/何時/哪個 commit/改了哪些檔——不會過期,稽核有讀者),不留「現況宣稱」與「等未來機器來讀」的欄位;正典裡 374 個過期 code 清單是現況宣稱必然腐爛的實證。
- 指紋計算函式 current_basis_digests 保留(drift 現場算的讀者);舊帳已存的 sha 欄位不清理,讀取端忽略未知欄位、無害。
- 附帶決定:封存零證據 change 時 stderr 印一行不擋人的提示(無旗標、不影響結果),給 AI 代理回頭檢查的訊號。
**Ruled out**: 甲(sha 留著不拆)——零讀者的資料即概念負債,與本場收斂原則直接矛盾;Phase 2 保留論已被 server 自記自判的架構事實推翻。
**Open**: (無——進結論)

## Conclusion

**Decision**: 走甲′——拆除本地 archive 的 evidence gate 全套(守門判斷、`--waive-evidence` 旗標、wire 的 `waiveEvidence` 參數、桌面/node 預設值),並將 evidence 帳瘦身:移除 entries 的 basisDigests 欄位與指紋判定模組(core 的 staleness 判定、host 的 check_archive_evidence 檢查函式;上一刀立進正典的對應需求以 REMOVED 撤掉)。帳保留 taskId/taskDesc/actor/headCommit/touchedFiles/recordedAt。封存零證據的 change 時 stderr 印一行不擋人的提示(無旗標)。保留不動:`.evidence.json` 搬家(change 目錄、進版控、隨封存)、@trace 兩欄一律注入、bulk 整潔工作樹守門移除、v1 檔案清單(commit 歸屬)、current_basis_digests 計算函式(drift 現場算的讀者)、桌面 dot 檔過濾。

**Rationale**: 兩把尺——「只留歷史事實,不留現況宣稱」與「每筆資料要有讀者」。門是用更多機制對抗記錄過期,方向本身錯了:沙盒探針證明它誤擋的全是守規矩的流程(純文件 change 零證據被擋、先 commit 再勾走進重跑無效的 stale 死路),而它擋不住想繞過的人(拒絕訊息自己印出放行指令);補丁鏈(修門→每勾必記→測試任務→mark-tasks-complete→桌面 toast→桌面按鈕)證明機制在本機單人+AI 場景撐不起自己。sha 拆門後零讀者;遠端版的門應由 server 自記自判,client 自報指紋不是地基。此決定反轉 post-archive-spec-value 的「本地 archive 強制 evidence gate」裁決,依據即上述實證。

**Rejected alternatives**: B 先補桌面放行按鈕——死路對 CLI 同樣存在,且養成一律放行的習慣,門形同虛設;丙 續修門(task done 每勾必記)——可解兩種誤擋但補丁鏈持續延長;甲 拆門留 sha——零讀者資料即概念負債,Phase 2 保留論不成立;全面 OpenSpec 化(連 v1 檔案清單也拆)——上游零機制成立於「一次一個 change、commit 交給 agent」,會失去平行 change 選擇性提交的檔案歸屬(原始且仍成立的需求)與桌面 @trace source 溯源(有 UI 讀者)。

**Deferred**: 測試結果(pass 數、指令)記入帳的欄位擴充——另行提案;遠端團隊版的 server 端 evidence 記錄與守門——Phase 2 時由 server 自記自判重新設計,不以本機帳為地基。

**Capture to**: 既有 change evidence-home-and-trace-slim(proposal 的 What Changes 與相容性影響、design 的 gate 決策反轉、specs 的 verify-evidence delta 改寫、tasks 新增拆除任務)

**Next**: speclink discuss link evidence-gate-false-blocks evidence-home-and-trace-slim → /speclink-ingest evidence-home-and-trace-slim
