## Why

使用者常在討論前就有自備的規劃文件——自寫的 markdown 或 plan mode 產出的計劃——但 speclink-discuss 技能目前只吃一句話 topic，codebase scout 也明文不看 docs；文件裡的主張不會被逐條對 codebase 驗證，可行性漏洞要到 apply 階段才爆。propose 側同樣有缺口：--from-discussion 只讀討論記錄，若記錄引用了原始文件（記錄只存討論結果、不內嵌全文），propose 讀不到底層規劃；而不經 discuss 直接以自備文件建立提案，目前只有 plan mode 對話觸發的 plan 檔偵測一條窄路。

目標使用者是透過 AI 代理跑 SDD 的開發者／PO／PM；使用情境是 workflow 的 discuss 與 propose 兩階段（speclink-discuss 與 speclink-propose 技能，工具面各涵蓋 claude 與 codex 技能實例與內嵌 assets）。

## What Changes

- **discuss 吃文件（預填樹）**：topic 可直接指定文件路徑（自寫 markdown、plan mode 產出、repo 內 docs、任意可讀路徑）。技能將文件視為「別人寫的 assumptions 清單」：萃取文件主張當決策樹節點，對 codebase 逐條分診為三類——證實（附程式碼證據）、牴觸（文件說 X 而程式碼是 Y）、真決策（送使用者裁定）。
- **discuss 記錄慣例**：討論記錄的 Context 固定寫一行 Source doc: <路徑>；輪的 Evidence 引用文件段落標題或短句；記錄只存討論結果，SHALL NOT 內嵌整份規劃文件。
- **propose --from-discussion 跟隨文件引用**：討論記錄含 Source doc 行時，propose 讀取原始文件並以疊加語意合成——文件為底層、討論為勝出層：討論有決定的以討論為準；討論未觸及的文件內容補位；討論 Ruled out 的內容不得復活於提案。
- **propose 直接文件入口**：新增 --from-doc <路徑> 引數慣例（仿 --from-discussion 的技能文字約定，非引擎旗標），供不經 discuss 直接以自備文件建立提案；與既有 plan 檔偵測（對話觸發、僅限 plan mode 路徑）並存，requirement source 優先序更新為：明確引數 → --from-doc → 討論記錄 → plan 檔偵測 → 對話上下文。以 --from-doc 建立的提案，proposal 的 Why 或 Impact 須含一行 Source doc: <路徑> 留存來源文件出處（--from-discussion 的提案由 link 記錄出處，--from-doc 的提案由這一行記錄）——同為技能文字約定，引擎零改動。
- **使用者文件補記新入口**：docs/workflow.md 與 docs/workflow.zh-TW.md 的 discuss 段 Input 補「topic 可為文件路徑」，propose 段 Input 與 Claude/Codex 呼叫行補 --from-doc <path> 變體，以符合正典 user-documentation「完整工作流指南說明用途與使用時機」對每階段輸入的既有要求（僅補實作、不動該正典規格）。
- **落地面**：discuss 與 propose 兩技能檔各三處實例同步（內嵌 assets、claude 與 codex 技能目錄），render golden 同批再生（四份 snapshot 均內嵌兩技能文字，於乾淨樹以 UPDATE_GOLDEN=1 執行 render_golden 測試再生並審視 diff）。

相容性影響：引擎零改動，所有 CLI 指令的人眼輸出與 --json shape 不變；golden snapshot 的變更是本提案的刻意產出（技能文字更新），同批更新並在此記載；既有討論記錄與提案流程不受影響——未給文件時兩技能行為照舊。

## Non-Goals

- 不動引擎：speclink-core 與 speclink-cli 的任何指令、旗標、frontmatter 欄位均不變——Source doc 與 --from-doc 都是技能文字約定，不是引擎語法。
- 不採用 grill-with-docs 的 inline 詞彙落檔——未收斂的詞會燒進正典 LANGUAGE.md；詞彙飄移維持結論時捕捉（其輸出側能力 speclink 已有：討論記錄、LANGUAGE.md、design.md）。
- 文件不只作背景素材——已於討論中裁定走預填樹逐條 stress-test。
- propose 不對文件做 grilling——逐條質疑是 discuss 的職責；propose 只消費（合成或原樣採用），角色不混。
- 不修改使用者的原始規劃文件——討論記錄存決策差分，合成是 propose 的責任。
- 不拆成兩個 change——Source doc 慣例的寫方（discuss）與讀方（propose）必須同批落地，分家會漂移。
- 不動 getting-started 文件——正典「Getting Started 僅使用已驗證入口」要求文中旗標須可由 speclink --help 觀察，而 --from-doc 是技能文字約定、非引擎旗標，寫入會違反該需求；workflow 指南無此限制，補記落在 workflow 兩檔。

## Capabilities

### New Capabilities

- `propose-skill`: speclink-propose 技能的文件輸入紀律——from-discussion 跟隨 Source doc 引用與疊加語意、--from-doc 直接文件入口。

### Modified Capabilities

- `discuss-skill`: 新增文件輸入相關需求——文件作為預填樹來源逐條分診、Source doc 記錄慣例（既有決策樹遍歷等需求不變，僅新增）。

## Impact

- Affected specs: 修改 `discuss-skill`（新增需求）、新增 `propose-skill`
- Affected code:
  - Modified:
    - `crates/speclink-core/assets/skills/discuss.md`
    - `crates/speclink-core/assets/skills/propose.md`
    - `.claude/skills/speclink-discuss/SKILL.md`
    - `.claude/skills/speclink-propose/SKILL.md`
    - `.agents/skills/speclink-discuss/SKILL.md`
    - `.agents/skills/speclink-propose/SKILL.md`
    - `crates/speclink-core/tests/golden/claude.snapshot.md`
    - `crates/speclink-core/tests/golden/codex.snapshot.md`
    - `crates/speclink-core/tests/golden/neutral-cli.snapshot.md`
    - `crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md`
    - `docs/workflow.md`
    - `docs/workflow.zh-TW.md`
  - New: (none)
  - Removed: (none)
- 影響的 crate／app：speclink-core（assets 與 golden 測試資料）與使用者文件（docs/）；引擎程式碼與 CLI 不動。
- 影響的技能與工具：speclink-discuss 與 speclink-propose 技能，claude（`.claude/skills/`）與 codex（`.agents/skills/`）兩者。
