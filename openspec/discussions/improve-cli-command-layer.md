---
topic: CLI 指令層的本機/remote 接縫重整——輸出面重複、模式分岔散佈、型別轉接落點
slug: improve-cli-command-layer
status: promoted
promoted_to: cli-render-unification
created: 2026-08-07
created_by: MomoChen <momochenisme@gmail.com>
kind: improve
---

# Discussion: CLI 指令層的本機/remote 接縫重整——輸出面重複、模式分岔散佈、型別轉接落點

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者執行 /speclink-improve，未指定方向，範圍由 git 熱點推得。近三個月變更次數：crates/speclink-cli/src/commands.rs 68 次居冠（3003 行）、main.rs 35 次（917 行）、remote_commands.rs 29 次（1425 行）；近一個月同樣是最熱的程式碼聚落。封存 change 的 .evidence.json 共動分析：動到 CLI 三檔任一的 7 個 change 中，6 個同時改了至少兩支——每個動詞的改動天然橫跨三檔。定界為「CLI 指令層（main.rs／commands.rs／remote_commands.rs）與其通往引擎（core/command）與 remote 管線（protocol → server → client）的接縫」。

排除項（Step 1 決策記錄比對）：
- init.rs＋7 份 golden 快照＋assets.lock 的衍生鏈是共動最高的聚落（10–13 個 change），但屬另一個範圍，且其同步紀律已逐 skill 正典化，留給下次掃描。
- remote-verb-parity（2026-07-30）的已否決項不重提：slugify 強制 ASCII、lifecycle 保留槽的 server 狀態機、bulk archive、in-progress bail 導向 claim。該討論是在既有結構內「接通管線」，其 Rationale 明言「同族缺口同一條管線，分批修等於付多次 protocol 變更成本」——本輪候選正是它未觸及的結構面。
- desktop store.ts／App.tsx 熱點屬 UI 範圍，與 CLI 層候選不可比，另計。

進行中 change 為零、進行中討論為零，無撞車風險。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — scan (2026-08-07)

**Focus**: CLI 指令層有哪些值得做的結構改進
**Position**: 這一層的核心摩擦是「一個動詞、三個檔、兩份輸出」——remote 支援是後來長上去的，parity 靠複製貼上與測試盯著，而不是靠結構保證。候選如下：

---

**候選 1：每個動詞只留一份輸出渲染，remote 靠型別轉接共用**

- **Files**: crates/speclink-cli/src/commands.rs、crates/speclink-cli/src/remote_commands.rs
- **Problem**: 緊耦合跨接縫洩漏，且已有漂移實證。同一個動詞的輸出邏輯寫兩份：discuss 全家的成功訊息在兩檔逐字重複（commands.rs:2620–2821 vs remote_commands.rs:1285–1372）；list 的人類可讀渲染整段複製（commands.rs:394–424 vs remote_commands.rs:105–118），而且已經漂移——本機版會渲染 `(invalid .openspec.yaml)` 紅字標記（commands.rs:411），remote 版的 wire 明明帶著 meta_error（remote_commands.rs:83 有映射）卻沒渲染這個標記。
- **Solution**: 專案裡已有正確樣板——status／show／instructions 走「wire DTO 轉回 core 型別、餵同一支渲染函式」（remote_commands.rs:182 to_status_report；commands.rs:473 render_show 註解明言「兩模式餵進同一個 ShowOutcome，輸出逐位元一致」）。把這個樣板推成全面規則：每個動詞的渲染只存在一支函式，吃 core outcome 型別；remote 路徑一律先轉型再餵同一支渲染。
- **Wins**: 兩模式輸出一致從「靠 parity 測試盯」變成「結構上不可能不一致」；新動詞只寫一份輸出；remote_list 這類漂移直接絕種。刪除測試通過：刪掉 remote 側的重複渲染後,每個動詞的輸出面收攏到一處,行為可整體理解。
- **Recommendation**: strongly recommended

---

**候選 2：本機/remote 的分岔決策收到 dispatch 一處，逼每個動詞表態**

- **Files**: crates/speclink-cli/src/commands.rs（dispatch:6–42 與全檔 22 處 remote_ctx() 分岔）、crates/speclink-cli/src/remote_commands.rs
- **Problem**: 同一個決策散在多處＋有事故實證的失效模式。「這個指令在 remote 模式該怎麼跑」散在 22 個函式開頭的 `if let Some(ctx) = remote_ctx()`；忘了寫這行的動詞會靜默走本機 store。remote-verb-parity 盤點出的 A 類缺口（cmd_show 讀本機空 store 回錯資料、cmd_in_progress 靜默寫本機丟失開工歸屬）正是這個結構的產物：新動詞的預設行為是「靜默本機」，沒有任何東西逼它回答 remote 怎麼辦。
- **Solution**: 模式判定上移到 dispatch 一次做完，之後以「每個動詞必須同時交出本機臂與 remote 臂」的形狀落地（窮盡 match 或雙臂表）——remote 不支援也必須是明寫的 bail，不能是忘記。
- **Wins**: cmd_show 那類靜默錯資料的事故從「靠人工盤點抓」變成「少一臂就編譯不過」；看一處就能盤出全部動詞的 remote 覆蓋。刪除測試通過：刪掉 22 處散佈分岔，模式決策集中一處。
- **Recommendation**: worth exploring（摩擦證據是四個候選裡最硬的，但形狀有真選擇——trait 雙臂、窮盡表、或維持函式對但加 lint 守門——需要烤問裁定）

---

**候選 3：include! 文字包含改成真模組，按動詞族重切檔案**

- **Files**: crates/speclink-cli/src/main.rs:916–917、commands.rs、remote_commands.rs
- **Problem**: 理解一個概念要跳好幾個檔。main.rs 用 `include!` 把兩支檔案文字塞進來——三檔實為一個 5300 行的編譯單元，零模組邊界，任何東西都看得到任何東西。同一個動詞的完整故事（clap 參數、本機臂、remote 臂、渲染）散在三個檔案；封存 evidence 顯示動到 CLI 的 7 個 change 有 6 個同時改兩支以上。
- **Solution**: 改成真正的 mod，按動詞族重切：一族一檔（discuss、station、list、config…），檔內含該族的參數定義、本機臂、remote 臂與共用渲染。與候選 1、2 天然互補——先做 1、2 再切檔最順。
- **Wins**: 改一個動詞開一個檔就夠（共動證據直接消解）；模組邊界讓越界依賴看得見。刪除測試通過：刪掉「本機檔/remote 檔」的切法、改按動詞族收攏,是把散在三處的同一概念集中,不是搬家。
- **Recommendation**: worth exploring

---

**候選 4：wire→core 型別轉接收斂進 speclink-remote，一份轉接兩個消費端用**

- **Files**: crates/speclink-cli/src/remote_commands.rs（to_status_report:182、to_apply_instructions:220、to_artifact_instructions:253）、crates/speclink-remote/src/client.rs、apps/desktop/src-tauri/src/remote.rs
- **Problem**: protocol 是刻意的純 DTO crate（只依賴 serde＋schemars，註解明言不得滲入 core／host），所以 wire→core 的欄位對拷散落在消費端：CLI 手寫三支 to_* 轉接，desktop 的 remote.rs（2386 行）直接吃 protocol 型別另養一套。wire 每加一個欄位（如 remote-verb-parity 補的 startedAt）要在多個消費端各補一次。
- **Solution**: speclink-remote 已同時依賴 core 與 protocol——把 wire→core 的正典轉接收進去（client 直接回 core 型別，或提供轉接模組），CLI 與 desktop 共用一份。
- **Wins**: wire 欄位演進只付一次轉接成本；desktop 的 remote 路徑有機會重用 fs 路徑既有的 core→看板轉換。
- **Recommendation**: speculative（CLI 端收益確定但小；desktop 端能省多少取決於它 fs／remote 兩路的內部形狀，尚未驗證——若 desktop 的目標型別本來就不是 core 型別，此候選只剩 CLI 端小勝）

---

**我的首選**：候選 1。證據最硬（remote_list 的 invalid 標記漂移是現行活漂移，不是推測）、形狀最無爭議（樣板已存在於同一個檔案裡，只是沒推到底）、單獨可落地，並為候選 2、3 鋪路。候選 2 的摩擦其實更痛（事故實證），但形狀需要先烤問。

要深入哪一個？

### Round 2 — interview (2026-08-07)

**Focus**: 候選 1 的範圍——wire 缺欄位的動詞要不要與純渲染收斂同批做
**Position**: 使用者裁定「一次做齊、含 wire 補欄位」（否決我提的分批案）。據此完成全動詞面盤點，分岐分三類：

**A 類·純渲染重複（wire 已載夠，收斂只動 CLI 兩檔）**：
- list：人眼渲染整段複製且已漂移——本機渲染 `(invalid .openspec.yaml)`（commands.rs:411），remote 有 meta_error 映射（remote_commands.rs:83）卻不渲染
- discuss 全家 10 個子指令的成功訊息逐字重複（commands.rs:2620–2821 vs remote_commands.rs:1285–1372）
- task done/undone：成功行與 --json 組裝兩份（remote_commands.rs:886–957）
- in-progress remove、discard、station add-round/stamp/discard 的成功行
- station show --json：本機走 ticket_json（commands.rs:2361，註明「欄位集合與 null 語意是對外契約」），remote 直印 wire DTO（remote_commands.rs:1184）——兩條組裝路，形狀是否已同形需在 propose 期驗證

**B 類·wire 缺欄位（要動 protocol→server→client 三層）**：
- archive：ArchiveResponse 只有 specs.capability（protocol/command.rs:110–113）；缺 dated_name、各 cap 的 added/modified/removed/renamed 計數、snapshot_created、archived_discussions、evidence_recorded（本機 print_archive_outcome 全印，commands.rs:806–834）
- station show 人眼路徑：本機印工單原文（commands.rs:2323 `print!("{doc}")`），remote 用結構化欄位拼摘要（remote_commands.rs:1186–1201）——wire 不載原文，補法是 ticket 回應加原文欄位（serde default 向後相容）

**C 類·明文分歧（提議保留，不追 byte parity）**：
- new change 的 `Path:` 行——server 端路徑對本機使用者無意義（本機印，remote 不印）
- list 的 worktree 欄——本機主 checkout 的觀察面，remote 恆缺席（remote_commands.rs:84–86 已有註解）
- status --schema 的 remote bail、workflow-config 的 config.yaml label——remote-verb-parity 已裁定的 C 類設計決定

**介面深度四項檢查**（候選 1 無條件過站）：
1. 接縫位置：渲染接縫定在「core outcome 型別 → stdout」，兩模式的資料組裝都在其上游；模式差異只准活在組裝與守門，不准活在渲染
2. 轉接數量：每動詞一支 wire→core 轉接（既有 to_* 樣板），單層、不疊 wrapper；轉接搬進 speclink-remote 是候選 4 的事、本刀不動
3. 深度：渲染函式背後藏 color、凍結標記文字、--json 形狀等真行為，不是轉發殼
4. 刪除測試：刪掉 remote 側重複渲染後每動詞的輸出面集中一處——複雜度集中而非搬移

**Ruled out**: 分批做（先純渲染、wire 缺口另批）——使用者裁定一次收完；「更小刀只修 list＋discuss」同時出局
**Open**: C 類明文分歧清單是否照上列保留；station show 人眼 parity 的補法（wire 載原文 vs 本機改結構化）待裁

### Round 3 — interview (2026-08-07)

**Focus**: parity 的邊界——C 類明文分歧的去留與 station show 人眼 parity 的補法
**Position**: 使用者照提案裁定兩點：(1) C 類明文分歧保留——new change 的 Path 行、list 的 worktree 欄、status --schema bail、workflow-config 的 config.yaml label 都是語意上正確的差異（server 端路徑對本機使用者無意義、worktree 是本機主 checkout 的觀察面），落地時寫進 design 的分歧清單，逐項附理由；(2) station show 人眼 parity 走「wire 補工單原文欄位」（serde default 向後相容），本機維持 print 原文零變更——反方向（本機改結構化摘要）會動到本機凍結輸出，違反「remote 對齊 fs」的方向性。
**Ruled out**: C 類分歧抹平（remote 也印 server 側路徑等）——語意錯誤的假 parity；本機 station show 改結構化摘要——動本機凍結輸出，方向反了
**Open**: 無——轉入結論

## Conclusion

**Decision**: 做候選 1 的全量版——CLI 每個動詞只留一份輸出渲染函式（吃 core outcome 型別），remote 路徑一律 wire→core 轉接後餵同一支渲染，模式差異只准活在資料組裝與守門；wire 缺欄位的動詞同批補齊 protocol→server→client 三層（archive 的 dated_name／各 cap 計數／snapshot_created／archived_discussions／evidence_recorded，station show 的工單原文欄位，皆 serde default 向後相容）；C 類明文分歧保留並寫進 design 分歧清單（new change 的 Path 行、list 的 worktree 欄、status --schema bail、workflow-config 的 config.yaml label）。remote_list 漂移修正（invalid 標記）是刻意的 remote 輸出對齊，凍結輸出測試同步改。
**Rationale**: 兩模式輸出 parity 目前靠複製貼上與測試盯，已出現活漂移（remote list 不渲染 invalid 標記）。共用樣板已存在且是既有設計決策（status／show／instructions／validate／analyze 走「DTO 轉回本地型別、同一渲染」），本刀是把它推成全動詞規則，讓 parity 從測試保證變成結構保證。一次做齊是使用者裁定——同族缺口同一條管線，分批要付多次 protocol 變更成本（remote-verb-parity 同一條理由）。
**Rejected alternatives**: 分批做（第一刀純渲染、wire 缺口另批）——使用者裁定一次收完；最小刀只修 list＋discuss——同上；本機 station show 改結構化摘要以達 parity——動本機凍結輸出，違反「remote 對齊 fs」的方向性；C 類分歧抹平——server 端路徑對本機使用者無意義，是語意錯誤的假 parity。
**Deferred**: 候選 2（本機/remote 分岔決策收攏 dispatch、逼每動詞表態）——非否決，事故類型有實證（remote-verb-parity 的 A 類靜默缺口），本刀落地後優先回訪；候選 3（include! 改真模組、按動詞族重切）——與 1、2 互補，順序在後；候選 4（wire→core 轉接收進 speclink-remote 供 CLI 與 desktop 共用）——desktop 端收益未驗證，屬 speculative；station show --json 兩條組裝路（ticket_json vs wire DTO 直印）的形狀同形驗證——propose 期做。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion improve-cli-command-layer
