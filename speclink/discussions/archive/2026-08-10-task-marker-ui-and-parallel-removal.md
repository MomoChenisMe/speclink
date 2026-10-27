---
topic: 任務標記的 desktop UI/UX 與 [P] 平行標記的處置
slug: task-marker-ui-and-parallel-removal
status: promoted
promoted_to: task-marker-ui-and-parallel-removal
created: 2026-08-10
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 任務標記的 desktop UI/UX 與 [P] 平行標記的處置

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因:manual-task-marker-gates 落地後的兩條延伸線——(1) 使用者問 `[P]` 平行標記什麼情況會被加上;(2) desktop 對 `[M]`/`[P]` 尚無 UI/UX 設計(上一 change 刻意接受字面顯示為過渡、GUI 列為 Deferred)。

模式:assumptions(scout 檔案:packages/ui/src/tasks.ts、packages/ui/src/components/TaskList.tsx、ChangeCard.tsx、crates/speclink-core/src/tasks.rs、instructions/query payload)。

現況地圖(已驗證):`[P]` 在全正典只出現於 tasks 起草指引的翻譯保留規則(tasks.instruction.md:12),無任何指引教何時加、無技能與 GUI 消費——OpenSpec 血統的休眠慣例;desktop 任務列原文照印字面前綴(tasks.ts 只剝行尾 stable-ID 註解),卡片進度只有全量 completedTasks/totalTasks(ChangeCard.tsx:42),無「寫碼完成、剩手測」的區分;封存區有三個舊 change 的 tasks.md 實際帶 `[P]`;parallel 欄位在 node SDK/UI/desktop 零消費。

相關 change:manual-task-marker-gates(已完成 11/11,尚未封存)。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-10)

**Focus**: `[P]` 的處置(啟用/休眠/移除)與 desktop 標記 UI 的範圍切分
**Position**: 使用者裁定 `[P]` 直接移除;UI 四項假設(剝前綴+徽章、卡片待手測狀態、勾選不加料)全數接受:
- `[P]` 移除的依據:全正典唯一出處是 tasks 起草指引的翻譯保留規則,無指引教何時加、無技能與 GUI 消費——「教了沒人用是假承諾」的休眠慣例,使用者選擇清掉而非等待未來的平行執行機制
- 移除深度採「認得但不承載」:封存區有三個舊 change 的 tasks.md 實際帶 `[P]`(且產品在外部使用者的 repo 也可能有),解析器保留前綴剝離讓舊檔顯示乾淨,但 Task.parallel 欄位、instructions/query payload 的 parallel 欄位、起草指引的 `[P]` 提及全數移除;起草指引的翻譯保留規則改點名 `[M]`(上一 change 的遺漏一併補上)
- 任務列:UI 端 tasks.ts 比照引擎剝除前綴,`[M]` 顯示「手動測試」徽章;`[P]` 舊標記只剝不顯
- 卡片:寫碼全完成且尚有未勾 `[M]` 時顯示「待手測」狀態 chip——list --json 需加 code 計數欄(additive,沿 instructions payload 慣例)
- 勾選互動不加料:徽章足以標示身分,無高亮、無彈窗、無分組
**Ruled out**: `[P]` 維持休眠(原建議——使用者裁定移除);啟用 `[P]`(需先設計 apply 平行執行機制,獨立大案);任務列保留字面 `[M]`(工程 token 違反詞彙原則);完全移除 `[P]` 解析(舊檔的字面前綴會滲進描述顯示)
**Open**: 徽章與 chip 的文案定稿(「手動測試」/「待手測」——詞彙新條目候選);卡片 chip 的擺位與樣式(ASCII 稿待使用者確認);parallel 欄位自 wire 移除的版本偏斜面(新舊 client/server 組合)是否需要特別處理

### Round 2 — interview (2026-08-10)

**Focus**: 徽章樣式的三選一——純符號 vs 符號+文字 vs 字面 [M]
**Position**: 使用者裁定符號+文字 chip(✋ 圖示+「手動測試」),經 ASCII 三案並列比較後定案:
- 符號給掃視速度、文字給第一次看到的人;i18n 走既有文案表
- 勾完後徽章保留(身分標示不劃線)、任務文字照舊劃線
- 卡片「⏳ 待手測(剩 N 項)」chip 與任務列徽章同家族視覺
- 字面 [M] 的例外論證被否:詞彙原則的既有例外(slug、config.yaml、worktree)都是給「可複製、可輸入的把手」,任務列是檢視面,[M] 在 GUI 無把手用途
**Ruled out**: 純符號(新使用者要 hover 才懂,辨識成本轉嫁);字面 [M](工程 token 直出,詞彙欠帳)
**Open**: 無——進入結論

### Round 3 — interview (2026-08-10)

**Focus**: 徽章擺位——使用者追加約束「不得破壞編號排版」
**Position**: 徽章自「編號前」移至行尾右對齊,編號欄與現況逐位元同位:
- 任務列的排版核心是「checkbox 後直接接編號」的對齊欄,徽章插在編號前會把 3.2 推右、破壞縱向對齊
- 定案:chip 釘在該列右上角(flex 尾端),長文字換行時徽章不動,右緣形成手測任務的掃視欄
- 勾完後文字劃線、徽章保留不劃線
**Ruled out**: 徽章置於編號前(破壞編號縱向對齊——使用者明示約束);編號後內嵌(編號對齊但打斷「編號→描述」閱讀流,右對齊版更乾淨);獨立圖示欄(全部列都要讓位,且退化成純符號)
**Open**: 無——進入結論

## Conclusion

**Decision**: 一個 change 收四件事。(1) `[P]` 平行標記移除,深度為「認得但不承載」:解析器保留 `[P] ` 前綴剝離(封存區與外部使用者 repo 的舊檔顯示容忍),Task.parallel 欄位、instructions/query payload 的 parallel 欄位、起草指引的 `[P]` 提及全數移除;起草指引的翻譯保留規則改點名 `[M]`(上一 change 遺漏補上)。(2) 任務列:UI 端剝離 `[M]`/`[P]` 前綴;`[M]` 任務顯示「✋ 圖示+手動測試」小 chip,置於該列行尾右對齊——編號起始欄與現況同位、長文字換行徽章不動;勾完後文字劃線、徽章保留;`[P]` 舊標記只剝不顯。(3) 變更卡:list --json 的 change 項增 code 計數欄(加欄不改名,沿 apply payload 慣例);寫碼任務全完成且尚有未勾 `[M]` 時,進度條下顯示「⏳ 待手測(剩 N 項)」chip;其他狀態卡片逐位元不變。(4) 詞彙:「手動測試」(任務徽章)與「待手測」(卡片狀態)入 LANGUAGE.md 新條目。
**Rationale**: `[P]` 全正典唯一出處是翻譯保留規則、無指引教何時加、無任何消費者——教了沒人用是假承諾,使用者裁定清掉而非等待未來的平行執行機制。UI 的核心資訊是「代碼收工、輪到你了」:卡片 chip 讓這個新流程狀態(寫碼完成、待手測)在看板上可辨,任務徽章讓手測任務在列表可掃;字面 `[M]` 是工程 token,詞彙原則的例外線(slug/config.yaml/worktree)只給「可複製輸入的把手」,任務列是檢視面套不上。
**Rejected alternatives**: `[P]` 維持休眠(原建議,使用者否決)或啟用(需先設計 apply 平行執行機制,獨立大案);完全移除 `[P]` 解析(舊檔字面前綴滲進顯示);純符號徽章(新使用者要 hover 才懂);字面 `[M]`(工程 token 直出);徽章置於編號前(破壞編號縱向對齊)或編號後內嵌(打斷閱讀流);勾選互動加高亮/彈窗/分組(徽章已足)。
**Deferred**: lucide 圖示確切選型(Hand 或同義,實作時定);parallel 欄位自 wire 移除的版本偏斜——新 client 讀舊 server 的多餘欄位由 serde 忽略、舊 client 讀新 server 缺欄位落 loud error,與 manual 加欄同一 fail-closed 姿態、隨版本一起出貨,不另做相容層。
**Capture to**: proposal(新 change;LANGUAGE.md 詞彙條目隨 change 落)
**Next**: /speclink-propose --from-discussion task-marker-ui-and-parallel-removal
