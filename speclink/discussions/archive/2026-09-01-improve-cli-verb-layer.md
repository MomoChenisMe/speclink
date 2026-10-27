---
topic: CLI 動詞層的引擎入口與 remote adapter 補完——外部架構報告查證入案
slug: improve-cli-verb-layer
status: promoted
promoted_to: cli-typed-engine-entry, remote-ctx-workspace
created: 2026-09-01
created_by: MomoChen <momochenisme@gmail.com>
kind: improve
---

# Discussion: CLI 動詞層的引擎入口與 remote adapter 補完——外部架構報告查證入案

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

來源：使用者以 /speclink-improve 傳入外部架構報告（matt-improve-report-C8WO18/reports/architecture-review-20260901-093511.html，2026-09-01）。報告自定範圍「crates/speclink-cli 動詞層、候選上限 2」。本討論把報告當候選來源，逐項對現行程式碼查證後入案；範圍即報告範圍——CLI 動詞層（src/verbs/ 13 族檔）與底座（common.rs、remote_base.rs）通往引擎與 remote 管線的接縫。

查證結果：報告主幹成立，數字幾乎全對——拆封儀式 28 處（實測 28）、&dyn Store 轉型報告 20 實測 21、open_project 報告 23 實測 22（20 處同形）、remote 臂 workspace 重取 6 處（實測 6）。

排除項（Step 1 決策記錄比對）：
- 報告候選 2 的子項「wire→core 轉換 10/10 收進 speclink_remote::convert」不入案。improve-wire-convert-seam（2026-08-10）已以 no change 收案並立落點規則：一支轉接在出現第二個要同一 core 型別的消費端時才搬進 convert；消費端的欄位取捨政策永遠留在消費端。前提至今未變：verbs/ 內 8 支 to_* 仍各為單一消費端，desktop 只消費 convert.rs 既有兩支（remote.rs:1491、1501）。報告未指名此否決，也未提出前提翻轉的證據，依規不得重提。報告的首選理由（「每個新 remote 功能再手寫一份轉換」）正建立在這個被砍除的子項上，故其首選排序不採。
- dispatch 模式表不動（cli-mode-dispatch-convergence，08-08 已定案）——兩候選皆在臂內與底座動刀，報告也明寫保留。
- 族檔硬規則合規檢查（cli-verb-family-modules，08-10）：族檔互不 import、跨族共用升底座——兩候選落點（common.rs、remote_base.rs）皆為底座，合規。
- remote-fix-plan-gaps（08-27）扇出的三刀已全數落地封存（含 2026-08-27-remote-evidence-scope-wiring，其斷點 station.rs:648-661 與候選 2 相鄰但已收工）。進行中變更與進行中討論皆為零，無撞車。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — scan (2026-09-01)

**Focus**: 外部報告的兩個候選查證後是否成立、以何形狀入案
**Position**: 兩候選的摩擦都查證為真。動詞層有兩條重複的儀式——fs 臂的引擎呼叫拆封、remote 臂的 workspace 重取——都是族檔重切後留下的結構債。候選如下：

---

**候選 1：引擎呼叫收成型別化入口（deep module）**

- **Files**: crates/speclink-cli/src/common.rs:35（run_command）、src/verbs/ 的 11 個族檔
- **Problem**: 訊號 2（shallow module）。run_command 的回傳是整個 CommandOutcome sum type，每個 caller 自己拆封。實測：`let core::command::CommandOutcome::X(..) = outcome else { unreachable!(..) }` 拆封 28 處（verbs/ 全域）、`let store: &dyn Store = &store` 轉型 21 處、open_project() 呼叫 22 處（20 處逐字同形）。「哪個 Command 產哪個 outcome」的不變式散成 28 句人肉斷言，interface 的學習成本逼近 implementation。
- **Solution**: common.rs 收型別化入口——每種 Command 回它自己的 outcome 型別，open_project、&dyn 轉型、execute、拆封整段吸進 implementation。fs 臂從三、四行儀式縮成一行型別化呼叫。
- **Wins**: 28 句 unreachable! 斷言歸零，不變式只剩一份；新動詞不可能拆錯臂；落點在底座，合族檔硬規則。刪除測試通過：刪掉 28 處散拆、同一行為集中一處可整體理解，是集中不是搬家。
- **Recommendation**: strongly recommended

---

**候選 2：RemoteCtx 帶上它已探索到的 workspace（補完 adapter）**

- **Files**: src/remote_base.rs:15–24、verbs/station.rs:666,682,741、verbs/checks.rs:330、verbs/instructions.rs:233、verbs/progress.rs:209
- **Problem**: 訊號 4（耦合跨接縫洩漏）。remote_ctx() 在 remote_base.rs:24 必先探索 Workspace 才能解析模式，卻只回 RemoteCtx { client } 把 workspace 丟掉。6 個 remote 臂各自重取，養出 4 種缺席策略（require_workspace 標準錯、discover? else bail 自訂訊息、? 傳播加 git_available 過濾、.ok().flatten() 靜默略過）。而 remote 模式成立的前提就是 workspace 存在——remote_ctx() 探索不到 workspace 時直接回 fs 模式——所以多數缺席分支守的是不可能發生的狀態。
- **Solution**: RemoteCtx 加 workspace 欄位，remote 臂直接取用；「remote 模式 ⇒ workspace 存在」的不變式收進 remote_base 一份，不可能狀態的缺席分支變死碼刪除。動詞級語意留在臂內（checks.rs 的 git_available 過濾、progress.rs 的 best-effort 空集合是刻意語意，不動）。順帶同刀：4 處 call site（new.rs:224、progress.rs:199,232、documents.rs:112）用外層 match 重寫了 remote_resolve_change 已實作的 Some 直通，可改成 checks.rs:290 的一行形（let-else）。
- **Wins**: 6 處重取歸零；4 種缺席策略收斂成「問題不存在」；4 處 resolve 儀式縮成一行。刪除測試通過：刪掉重取與缺席分支後，workspace 生命週期集中在 remote_ctx() 一處。
- **Recommendation**: worth exploring——摩擦已證，但形狀有真問題要烤：哪些缺席差異是動詞語意、哪些是意外；且報告原版的第三個子項（轉換收進 convert）已依 improve-wire-convert-seam 的決策記錄砍除，砍除後本候選的收益比報告宣稱的小一截。

---

**報告首選不採的理由**：報告把候選 2 排第一，論據是「每個新 remote 功能再手寫一份轉換」——那正是被砍除的子項。砍除後候選 2 剩下的是中型清理，候選 1 才是槓桿最大的一刀（1 個 interface、28 個呼叫點、每個未來動詞受益）。

**我的首選**：候選 1。證據最硬（三組數字全數實測吻合）、形狀最無爭議（純底座深化，零行為變更）、單獨可落地，且讓候選 2 之後動 remote 臂時的 diff 更乾淨。

要深入哪一個？

### Round 2 — interview (2026-09-01)

**Focus**: 候選 1 的接縫位置——型別化入口住 CLI 底座，還是下沉引擎
**Position**: 偵察改變了題目。「哪個 Command 產哪個 outcome」的不變式今天有三個消費端各自斷言：CLI verbs/ 28 處 let-else＋unreachable!；server routes.rs 36 處 CommandOutcome:: 引用、約 20 個 `_ => wrong_outcome(..)` 臂（拆錯當執行期錯誤回報）；node SDK lib.rs 8 處。host bridge.rs 的 command_label 是第四個引用者但性質不同——窮盡 match 產 audit 標籤，不是拆封，不在本題內。依 improve-wire-convert-seam（08-10）落點規則的精神——出現第二個要同一 core 型別的消費端才下沉，這裡已有三個——提案：

1. 型別轉換下沉 core（additive）：每個 outcome 型別一支 TryFrom<CommandOutcome>（或 into_* accessor），不動 Command／CommandOutcome 既有形狀，已發佈的 node API 不破壞。
2. CLI common.rs 收薄的泛型入口 run<T>(store, ws, cmd)——吸收 &dyn 轉型＋execute＋轉換三段儀式；open_project 不吸收。實測不合身：discuss.rs 一次開專案供 11 次 run_command、多個臂在引擎呼叫後仍用 ws（渲染、host 呼叫），22 處 open_project 對 29 處 run_command 非 1:1。報告的「一行搞定」草圖只對拆封那一半。
3. server／node 的遷移不進本刀（範圍紀律：本掃描範圍是 CLI 動詞層）；core 層就位後它們是各自的後續小刀。

**Ruled out**: （待裁定後記錄）
**Open**: 接縫位置由使用者裁定——(a) 下沉 core＋CLI 薄入口（我的提案）；(b) 只動 CLI（common.rs 收 27 支具名函式或泛型入口，core 不動，不變式仍留在 server／node）；(c) 全鏈一刀（core＋CLI＋server＋node 同批遷移，刀變大）。CLI 端入口形狀細節（泛型 TryFrom vs 具名函式）隨裁定於下一輪收斂。

### Round 3 — interview (2026-09-01)

**Focus**: 候選 1 接縫位置裁定；候選 2 的形狀確認與兩候選的介面深度檢查
**Position**: 使用者裁定候選 1 走 (a)——型別轉換下沉 core（每個 outcome 型別一支 TryFrom<CommandOutcome>，additive、不破壞已發佈 node API）＋CLI common.rs 收薄泛型入口 run<T>(store, ws, cmd)，吸收 &dyn 轉型＋execute＋拆封；open_project 不吸收；server／node 遷移留作 core 層就位後的後續小刀。呼叫端形狀隨選項一併定案：`let list: ListOutcome = run(&store, Some(&ws), cmd)?`。mismatch 時 run<T> 的錯誤語意（保留 panic 的 unreachable 語意，或改與 server wrong_outcome 同型的錯誤值）歸 propose／design 階段。

候選 2 形狀（逐點查證後的提案）：RemoteCtx 加 `ws: core::workspace::Workspace` 欄位（remote_ctx() 在 remote_base.rs:24 已探索到手，帶出即可）。六個重取點的處置：station.rs:666、:682 的 require_workspace()? 與 :741 的 discover? else bail 皆為不可能狀態的守門，改欄位取用、守門刪除；checks.rs:330 的 git_available 過濾是真語意（有 workspace 不代表有 git），保留過濾、只刪重探索；instructions.rs:233 的靜默略過分支與 progress.rs:209 的 best-effort 空集合——progress 的註解自己就寫著「remote mode already resolved, so a config error here is unreachable」——缺席分支刪除，語意不變。順帶同刀：new.rs:224、progress.rs:199,232、documents.rs:112 四處以外層 match 重寫 remote_resolve_change 已實作的 Some 直通，改 checks.rs:290 的 let-else 一行形。

介面深度四項檢查——
候選 1：(1) 接縫位置：outcome 型別對應下沉到擁有此不變式的模組（core::command::execute 本來就實作它）；CLI 專屬的 context 組裝留在 CLI 入口。(2) 轉接數量：單層——core 的 TryFrom＋CLI 一支泛型入口，run<T> 取代 run_command 而非包住它。(3) 深度：入口藏 context 組裝＋execute＋錯誤映射＋outcome 轉換，介面一支泛型函式。(4) 刪除測試：刪掉即回到 28 處散拆＋21 處轉型，是集中。
候選 2：(1) 接縫位置：workspace 解析本來就發生在 remote_ctx() 內，只是結果沒帶過接縫；補欄位是讓 adapter 載完它已產出的東西。(2) 轉接數量：仍是一個 RemoteCtx，零新增 wrapper。(3) 深度：remote_ctx() 藏探索＋模式解析＋握手＋binding，加 ws 後臂改消費、不再重推導。(4) 刪除測試：刪掉欄位即回到 6 處重取＋4 種缺席策略，是集中。

**Ruled out**: 候選 1 只動 CLI（common.rs 具名函式、core 不動）——不變式仍在 server（約 20 個 wrong_outcome 臂）與 node（8 處）各留一份，server 之後收斂得重做一次映射；全鏈一刀（core＋CLI＋server＋node 同批）——刀跨 4 個 crate，凍結輸出測試要四邊同時盯，範圍紀律不許。
**Open**: 收尾分刀——兩候選一刀還是兩刀、先後順序，由使用者裁定。

## Conclusion

**Decision**: 兩候選皆落地，分兩刀依序立案。刀 1（引擎入口型別化）：core 為每個 outcome 型別補一支 TryFrom<CommandOutcome>（additive，不動 Command／CommandOutcome 既有形狀、不破壞已發佈 node API）；CLI common.rs 收薄泛型入口 run<T>(store, ws, cmd) 吸收 &dyn 轉型＋execute＋拆封三段儀式，open_project 維持獨立；verbs/ 的 28 處 let-else＋unreachable! 與 21 處轉型歸零。刀 2（RemoteCtx 補完）：RemoteCtx 加 ws: Workspace 欄位（remote_ctx() 已探索到手、帶過接縫即可）；6 處 remote 臂重取刪除；不可能狀態的缺席守門（station.rs:666,682,741、instructions.rs:233、progress.rs:209）刪除，checks.rs:330 的 git_available 過濾與 progress 的 best-effort 空集合語意保留；順帶把 new.rs:224、progress.rs:199,232、documents.rs:112 四處重寫 Some 直通的外層 match 改為 let-else 一行形。
**Rationale**: 「哪個 Command 產哪個 outcome」的不變式有三個消費端各自斷言（CLI 28、server 36 處引用約 20 個 wrong_outcome 臂、node SDK 8），擁有並實作此不變式的模組是 core::command::execute——型別對應下沉到擁有者，規則才真正只剩一份；CLI 專屬的 context 組裝留在 CLI 入口，兩層各深其所。remote 側：workspace 解析本來就發生在 remote_ctx() 內，adapter 少載一個它已產出的欄位，讓 6 個臂重推導並養出 4 種守不可能狀態的缺席策略——補欄位是純減法。分兩刀因兩候選檔案面幾乎不重疊（core＋fs 臂 vs remote_base＋remote 臂），每刀 review 面乾淨；刀 1 先行讓刀 2 動 remote 臂時的 diff 更乾淨。介面深度四項檢查兩候選皆過站（Round 3）。
**Rejected alternatives**: 報告候選 2 的子項「wire→core 轉換 10/10 收進 speclink_remote::convert」——improve-wire-convert-seam（08-10）已否決的原案重提，落點規則前提未變（8 支 to_* 仍單一消費端、desktop 只消費 convert.rs 既有兩支），掃描階段即不入案；候選 1 只動 CLI（core 不動）——不變式仍在 server／node 各留一份，server 之後收斂得重做映射；全鏈一刀（core＋CLI＋server＋node 同批遷移）——跨 4 個 crate，凍結輸出測試四邊同盯，範圍紀律不許；兩候選一刀合做——刀跨 core 與兩側儀式，review 面變大、回溯糾纏；刀 2 先行——無 remote 線待接的急迫證據，讓大槓桿的刀 1 先回本（使用者裁定）；報告原首選排序（候選 2 第一）——其論據建立在被砍除的 convert 子項上。
**Deferred**: server routes.rs 約 20 個 wrong_outcome 臂與 node SDK 8 處拆封遷移到 core 的 TryFrom 層——刀 1 落地後各自的後續小刀；run<T> 對 mismatch 的錯誤語意（保留 unreachable 的 panic 語意 vs 與 server wrong_outcome 同型的錯誤值）——歸刀 1 的 propose／design 階段。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion improve-cli-verb-layer（先立刀 1，刀 2 隨後）
