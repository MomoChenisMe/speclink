---
topic: 詳情面板在審查／驗證進行中顯示工單內容的分頁
slug: drawer-quality-ticket-tab
status: promoted
created: 2026-09-22
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: drawer-quality-ticket-tab
---

# Discussion: 詳情面板在審查／驗證進行中顯示工單內容的分頁

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在 wadpilot 專案的變更詳情面板看到「審查中」「驗證中」章，但面板沒有地方看工單（review.md／verify.md）內容；希望多一個分頁，在工單檔存在時才出現，蓋章或放棄後工單被刪就消失。需求已具體（顯示條件＋內容來源），未走 grill 階段。相關規格：desktop-app（狀態列章籤、已封存側標示）、review-station 與 verify-station（工單建立、蓋章刪檔、封存帶走）、client-protocol（GET /changes/{name}/review 與 /verify 的 ReviewTicketResponse）；無進行中變更。
Prior discussions: spec-drawer-trace-links, review-before-manual-test-tasks, worktree-handoff-quality-mention, schema-engine-openspec-parity, archived-parity-and-spec-purpose

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-22)

**Focus**: 工單分頁的顯示條件、內容來源與範圍（活面板／已封存面板）的初始決策空間
**Position**: 分頁以「工單檔存在」為唯一顯示條件，內容由既有讀取路取得，範圍先聚焦活面板：
- 顯示條件直接用清單資料上的 reviewStatus === inReview／verifyStatus === inVerify（apps/desktop/core/src/query.rs:85-111 以工單檔存在算出），監看遞迴（watch.rs:95）故分頁會自動出現與消失
- 本機讀檔走既有 document 指令（query.rs:379 無白名單）；遠端不放寬 artifact cat 白名單（command/mod.rs:1367），改接 server 既有 GET /changes/{name}/review／verify（ReviewTicketResponse 含 rounds 與 content）
- 引擎已有結構化解析（station.rs Round／Finding），CLI review show --json 與 server 路都輸出同形狀，桌面兩條路可共用一個 DTO
- 分頁名用正典詞「品質關卡」，面板 tab 一律稱「分頁」（LANGUAGE.md 換頁詞條）
- 使用者第一輪回饋：第一版就要完整顯示（不要只渲染 markdown 原文）；已封存側傾向不做
**Ruled out**: 另開「檔案存在」查詢——與既有狀態雙軌易不同步；放寬 artifact cat 白名單——會連帶改 CLI 的 verb-contract 與規格
**Open**: 「完整顯示」的具體樣式（結構化輪次／findings 渲染 vs 原文）；單一「品質關卡」分頁 vs 審查／驗證兩個分頁；已封存側是否要分頁（蓋章者無工單、僅帶走者有化石工單，本 repo 封存區目前零份）；使用者停在該分頁時工單消失的退場行為

### Round 2 — interview (2026-09-22)

**Focus**: 兩站工單如何分開呈現，以及已封存側是否納入
**Position**: 審查與驗證各自一個條件式分頁（與標頭章籤同名），已封存側在「曾審查／曾驗證未通過」時也顯示同樣的分頁：
- 使用者回饋：quality 兩站合跑時整篇放一起太長，要分開；並確認若帶走工單進封存就要做已封存側
- 活面板：reviewStatus === inReview 出「審查」分頁、verifyStatus === inVerify 出「驗證」分頁，皆無時分頁列維持原五個；分頁名與狀態列章籤一字不差，看到「審查中」就知道去哪個分頁
- 已封存側：--carry-review／--carry-verify 時工單隨目錄移入封存區（review-station「明示帶走」scenario；桌面「照樣帶走」按鈕即 archive_carry_at，verbs.rs:44），狀態 reviewedNotPassed／verifiedNotPassed（cache.rs:95-106）即分頁顯示條件；遠端 /archived/{dated}/artifacts/{*} 無白名單（read_api.rs:146），可讀 review.md 原文
- 解析：引擎 parse_ticket 目前私有（station.rs:787），需開一個純函式（文字→Ticket）讓 desktop core 對本機活工單、本機封存工單、遠端封存原文統一解析；遠端活工單直接吃 GET /changes/{name}/review／verify 的 rounds
- 每個分頁內：輪次可收合（末輪展開）、finding 一列一條帶嚴重度色章、(accepted) 轉「已接受」籤、範圍檔案收在「範圍 N 檔」後
**Ruled out**: 單一「品質關卡」分頁內放兩段——兩站合跑時太長；左右對照輪次——面板預設寬約半視窗，finding 列要放路徑加描述，且兩站輪數與階段各自獨立，左右對齊沒有意義；分頁內再套審查／驗證切換鈕——多一層控制，章籤與分頁同名更直接
**Open**: phase 的使用者可見文案（暫定 discovery＝首輪、validation＝複驗；不能用「驗證輪」以免與驗證站撞名）；使用者停在分頁時工單被刪的退場（暫定退回提案分頁）；點狀態列章籤是否直接跳到對應分頁

### Round 3 — interview (2026-09-22)

**Focus**: 輪次階段的使用者可見文案與收尾的兩個小決策
**Position**: discovery 顯示「首輪」、validation 顯示「複驗」，使用者已點頭；其餘兩項以慣例收掉：
- 「首輪／複驗」入 LANGUAGE.md 新詞條，avoid「驗證輪」（與驗證站撞名）、「發現輪」
- 使用者停在分頁時工單被刪：分頁隨狀態消失，選中分頁退回「提案」
- 點狀態列章籤跳到對應分頁：不進第一版，記為延後
**Open**: 無

## Conclusion

**Decision**: 變更詳情面板與已封存詳情面板各加兩個條件式分頁「審查」「驗證」，工單檔存在時才出現，內容為結構化渲染的工單（非 markdown 原文）。
- 顯示條件：活面板 reviewStatus === inReview 出「審查」、verifyStatus === inVerify 出「驗證」；已封存面板 reviewedNotPassed／verifiedNotPassed 同法。皆無時分頁列維持原樣。分頁名與狀態列章籤一字不差。
- 分頁內容：段標題（站名、目前輪數、末輪 CRITICAL／WARNING／SUGGESTION 計數）；輪次可收合、末輪展開、舊輪收合；每輪標「首輪」（discovery）或「複驗」（validation）與「範圍 N 檔」（點開列檔）；finding 一列一條——嚴重度色章（CRITICAL 紅、WARNING 琥珀、SUGGESTION 灰）、路徑等寬字、描述原文不翻譯；行尾 (accepted) 轉「已接受」籤。legacy 輪（無 Phase／Patch）不標階段。
- 資料路：desktop core 新增查詢回同一種 Ticket DTO（rounds[{index, phase, patchHash, scope[], findings[{severity, path, text}]}]）。本機活工單走引擎 station::show；本機封存工單與遠端封存原文（/archived/{dated}/artifacts/{*} 無白名單）走引擎新開的純解析函式（parse_ticket 由私有升為 pub）；遠端活工單走既有 GET /changes/{name}/review／verify 的 rounds。不放寬 artifact cat 白名單。
- 退場：工單被刪（蓋章／放棄）時監看刷新狀態，分頁消失；使用者正停在該分頁時退回「提案」。
- 範例：審查工單 Round 1（discovery，2 條 WARNING）、Round 2（validation，1 條 WARNING 帶 (accepted)、1 條 SUGGESTION）→「審查」分頁段標題「審查 · 第 2 輪（複驗）CRITICAL 0 · WARNING 1 · SUGGESTION 1」，第 2 輪展開列兩條，WARNING 那條帶「已接受」籤，第 1 輪收合顯示「首輪 · 範圍 N 檔 · 2 條」。
**Rationale**: 桌面已用「工單檔存在」算出四種狀態，顯示條件零新查詢；引擎與 server 都已有結構化輪次，完整顯示只差桌面接線與一個純解析函式，成本與原文渲染相差不大；兩站合跑時整篇太長，分頁與章籤同名最直接。
**Rejected alternatives**: 單一「品質關卡」分頁放兩段——兩站合跑時太長；左右對照輪次——面板半視窗寬放不下路徑加描述，且兩站輪次獨立無對照意義；分頁內再套切換鈕——多一層控制；markdown 原文渲染——使用者要第一版完整顯示；另開「檔案存在」查詢——與既有狀態雙軌；放寬 artifact cat 白名單——牽動 CLI verb-contract 與規格；階段文案「驗證輪」——與驗證站撞名。
**Deferred**: 點狀態列章籤直接跳到對應分頁；蓋章後（工單已刪）的歷史回看——fs 模式只在 git 歷史、remote 不保留，本次不做。
**Capture to**: proposal（新變更：desktop core 查詢＋tauri 指令＋remote 指令、packages/ui 工單元件與兩面板分頁、引擎 parse_ticket 公開、desktop-app 與 client-protocol delta 規格）；LANGUAGE.md（新詞條「首輪」「複驗」；面板 tab 用「分頁」不用「頁籤」）
**Next**: /speclink-propose --from-discussion drawer-quality-ticket-tab
