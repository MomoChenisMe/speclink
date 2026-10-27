## Why

review／verify／quality 目前把「任務全勾」當「實作完成」的代理指標:review 技能開跑守門、verify 引擎落工單守門、兩站引擎蓋章守門都要求任務全數完成。手動測試任務戳破這個代理——代碼已完成、只剩人工驗收時,三站全被擋住,使用者被迫「先手測再審查」;而 review 後的修碼會作廢手測結果,最貴的人工驗收被排在最容易白做的位置。引擎沒有任務分類的正典依據,技能層只能靠 agent 讀任務文字猜,判斷不可靠。

本 change 源自已結論的討論 review-before-manual-test-tasks(結論:引入 `[M]` 標記、三道守門統一改判「寫碼任務全完成」、freshness 失效判定接上封存守門)。

## What Changes

- tasks.md 任務行引入手動測試標記 `[M]`(循既有 `[P]` 前綴慣例):解析器讀入 Task 的 manual 旗標,並隨 instructions／query 的任務 payload 上線(逐任務 manual 欄位與進度拆分欄位)。
- 三道守門統一改用同一預測子「寫碼任務全完成」(非 `[M]` 任務全數勾選;`[M]` 任務不計):
  - review 技能的開跑守門(技能文字,原為任務全數完成即停)
  - verify 引擎的落工單守門(原為任務未全完成即拒 add-round)
  - 兩站引擎的蓋章守門(原為任務全數完成才可 stamp)
- 章的語意由「成品定案」改定為「驗證過(可驗的部分)」;「成品定案」語意由封存承接。
- freshness 失效判定的任務錨改為 manual-aware:任務總數改變、或任一寫碼任務未勾 → 章失效;僅補勾 `[M]` 任務不使章失效。內容錨(scope 檔雜湊)行為不變。
- **BREAKING**(刻意行為變更)封存新增章失效守門:change 帶有 review 章或 verify 章、且該章依失效判定為過期(蓋章後改過 scope 檔、或任務錨破)時,單筆封存 SHALL 拒絕並指路重跑對應站別;無章、或章欄位不全(Unknown)時照舊放行。既有「任務完成度守門」(全任務含 `[M]`)不變——手測不做仍封不了。
- 技能文字同步:review(開跑守門改判寫碼任務)、verify(中途盤點與成品驗證的分流改判寫碼任務)、quality(前提轉述改判寫碼任務)、propose(tasks.md 起草時教 `[M]` 標記慣例)、apply(一行原則:`[M]` 任務不代勾、寫碼任務全勾即回報完成)——含 assets、.claude 同步與 golden 三連動。
- 動詞 --json 契約釘新欄位:任務 payload 的逐任務 manual 欄位與進度拆分欄位入 verb-contract 契約(加欄不改名、不移除既有欄位)。

## Capabilities

### New Capabilities

- `manual-task-marker`: tasks.md 的 `[M]` 手動測試任務標記——行文法、解析為 manual 旗標、instructions／query payload 曝光,以及「寫碼任務全完成」預測子的正典定義。

### Modified Capabilities

- `review-station`: 蓋章守門的任務條件改為寫碼任務全完成;失效判定的任務錨改 manual-aware,並開放封存守門消費判定結果。
- `verify-station`: 落工單守門與蓋章守門的任務條件改為寫碼任務全完成;失效判定任務錨同步 manual-aware。
- `change-lifecycle`: 新增「封存的章失效守門」——過期章擋單筆封存;既有任務完成度守門維持全任務判定。
- `review-skill`: 開跑守門自檢與「任務未完成即停」情境改判寫碼任務全完成。
- `verify-skill`: 工單落地的分流條件(成品驗證 vs 中途盤點)改判寫碼任務全完成。
- `quality-skill`: 兩站時序的前提轉述改判寫碼任務全完成。
- `propose-skill`: tasks 起草新增 `[M]` 標記慣例——人工驗收類任務標記後,寫碼與驗收的完成度得以分開判讀。
- `verb-contract`: --json 輸出形狀凍結契約釘入任務 payload 的新增欄位(逐任務 manual 與進度拆分)。

## Impact

- Affected specs: manual-task-marker(新增)、review-station、verify-station、change-lifecycle、review-skill、verify-skill、quality-skill、propose-skill、verb-contract
- Affected code:
  - New: crates/speclink-cli/tests/it/manual_task_gates.rs
  - Modified:
    - crates/speclink-core/src/tasks.rs(解析 `[M]` → manual 旗標;manual-aware 進度統計)
    - crates/speclink-core/src/station.rs(蓋章守門、verify 落工單守門、freshness 任務錨改判寫碼任務)
    - crates/speclink-core/src/review.rs 與 crates/speclink-core/src/verify.rs(freshness 門面簽名補 manual-aware 計數)
    - crates/speclink-core/src/archive.rs(封存流程新增章失效守門)
    - crates/speclink-core/src/instructions.rs(任務 payload 的 manual 欄位與進度拆分)
    - crates/speclink-protocol/src/query.rs(wire 形狀新增 manual 欄位)
    - crates/speclink-core/src/skills.rs(五個技能 asset 文字與 MARKER_VERSION)
    - crates/speclink-core/assets/skills 下的 review、verify、quality、propose、apply 技能資產(claude 與 codex 兩形)
    - .claude/skills/speclink-review/SKILL.md、.claude/skills/speclink-verify/SKILL.md、.claude/skills/speclink-quality/SKILL.md、.claude/skills/speclink-propose/SKILL.md、.claude/skills/speclink-apply/SKILL.md(與 assets 同步)
    - crates/speclink-core/tests/golden 下的 snapshot 與 assets.lock(技能三連動再生)
    - crates/speclink-cli/tests/it/review_verbs.rs、crates/speclink-cli/tests/it/verify_verbs.rs、crates/speclink-cli/tests/it/archive_readiness_gate.rs(守門行為的既有測試調整)
  - Removed: (none)
