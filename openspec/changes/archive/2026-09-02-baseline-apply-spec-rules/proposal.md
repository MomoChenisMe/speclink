## Why

Baseline 技能直接把正式 specs 寫進 openspec/specs/，不建立 change，因此 openspec/config.yaml 的 rules.specs 沒有任何路徑能到它手上：一般 specs artifact 的產出規則靠 speclink instructions specs 的 payload 注入，而這個動詞在沒有 active change 時只回「No active changes」。現行 baseline.md 的 Step 1 只讀 project context 與 spec_locale，全文沒有 rules，結果是團隊寫進 config 的 specs 產出規則對第一批規格基準完全不生效，且使用者看不出來哪些規則有套、哪些沒套。

目標使用者是透過 AI 代理跑 SDD 的開發者，情境是既有 codebase 首次建立規格基準（baseline 站）或補未覆蓋能力（gap filling）。相關討論：baseline-apply-spec-rules（2026-09-02，已結論）。

## What Changes

- Baseline 技能的 Step 1 改為執行 speclink workflow-config show --json 取得正典 workflow config（config.yaml 或 remote store 的 config 文件所載的值，不套用 SPECLINK_* 環境變數覆寫），從 payload 讀 context、specLocale（auto 時以同一 payload 的 locale 為準）與 rules.specs；不再直接讀 openspec/config.yaml。指令失敗（含壞 config 的 fail-closed）時停下回報，不退回手讀 YAML、不自行解析。
- rules.specs 存在且非空時，技能逐字套用到本輪產生的每一份正式 spec，不做「哪條適用 baseline」的篩選；未設定或空清單時行為維持不變。
- 寫入前的 capability map 確認訊息固定帶一段「本輪套用的 specs 產出規則」，逐條列出原文；未設定時明寫不套用。
- 最終報告固定帶一段：本輪是否套用 rules.specs、實際套用了哪幾條；未設定時明寫無。
- 技能明文載明：這些規則是 Agent 產生內容時必須遵守的指令，speclink validate --specs --all --strict 只檢查結構，不機械式驗證自由文字規則。
- Baseline 既有六項邊界寫入新規格：只記錄目前已存在的行為、不修改 code、不建立 change、已有 specs 時只做 gap filling、寫入前等待使用者確認 capability map、最後執行 strict validation。
- ASSET_VERSION 由 v1.26.1 升到 v1.27.0；五份 golden snapshot 與 assets.lock 同批再生（刻意變更）；repo 內 .claude/skills 與 .agents/skills 的生成技能檔由 speclink update 再生。
- crates/speclink-core/tests/it/render_golden.rs 新增一支測試，斷言生成的 Baseline 技能會取得 workflow config、讀 rules.specs、明文要求每份正式 spec 遵守規則、未設定時照常執行、最終報告揭露套用的規則。
- docs/workflow.md 與 docs/workflow.zh-TW.md 的 baseline 站 Input 行補上 workflow config（context、specLocale、rules.specs）。

影響的技能與工具：只有 baseline 技能；claude（.claude/skills/speclink-baseline/SKILL.md）與 codex（.agents/skills/speclink-baseline/SKILL.md）兩側同源渲染。skills.rs 的 registry 項（id、description、for_codex）不變。

不新增或變更任何 CLI 子指令、旗標、stdin 與 exit code；不新增或變更 openspec/config.yaml 與 .speclink.yaml 的任何欄位與預設值。

相容性影響：
- CLI 的人眼輸出與 --json payload 不變，不破壞任何 CLI 回歸對照。
- 五份 golden snapshot 的 baseline 段與版本戳改變，屬刻意變更，同批再生。
- 既有使用者升級後執行 speclink update，即可取得新版 baseline 技能；生成的全部 speclink-* SKILL.md 的版本戳由 v1.26.1 變為 v1.27.0，其他技能內文不變。
- Baseline 的 Local／Remote 支援範圍不變：三個動詞（list、validate、workflow-config）本就是 Dual 動詞，且 workflow-config show --json 在 fs 與 remote 共用同一輸出函式，payload 形狀一致；直接寫檔到 openspec/specs/ 的本機行為維持原樣。

## Capabilities

### New Capabilities

- `baseline-skill`: Baseline 技能的行為契約——技能檔的渲染與 golden 保護、盤點前取得 workflow config 並套用 rules.specs（含 capability map 與最終報告的揭露）、以及既有六項行為邊界。步驟三的規格掃描找到最近的四個 capability，皆不承載此契約：workflow-config 只規範 workflow-config show／set 動詞本身的輸出與寫入語意，不管誰消費它；skill-routing 的 Purpose 明文「不管單一技能的內文行為（那屬各 per-skill capability）」，對 baseline 只管 description 與出口句；config-skill 是 speclink-config 技能自己的契約，本次僅作為寫法範本；user-documentation 只管使用者文件。改名討論 rename-onboard-to-baseline（2026-09-01）當時因行為未變而不建 baseline-skill spec，本次行為改變，理由反轉。

### Modified Capabilities

（無。workflow-config 動詞的輸出與寫入語意不變；skill-routing 對 baseline 的 description 與出口句不變；user-documentation 對 workflow 兩語言「每站載明輸入」的既有要求已涵蓋本次的文件補句，不需新增 requirement。）

## Impact

- Affected specs：新增 baseline-skill；無修改。
- Affected code：
  - New：
    - openspec/specs/baseline-skill/spec.md（由本 change 的 delta 封存後生成）
  - Modified：
    - crates/speclink-core/assets/skills/baseline.md
    - crates/speclink-core/src/init.rs（ASSET_VERSION 常數）
    - crates/speclink-core/tests/it/render_golden.rs
    - crates/speclink-core/tests/golden/claude.snapshot.md
    - crates/speclink-core/tests/golden/codex.snapshot.md
    - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
    - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
    - crates/speclink-core/tests/golden/claude-worktree.snapshot.md
    - crates/speclink-core/tests/golden/assets.lock
    - docs/workflow.md
    - docs/workflow.zh-TW.md
    - .claude/skills/speclink-baseline/SKILL.md
    - .agents/skills/speclink-baseline/SKILL.md
    - .claude/skills/ 與 .agents/skills/ 底下其餘 speclink-* 技能檔（僅版本戳 v1.26.1→v1.27.0）
  - Removed：無
- 相依與系統：不新增相依；不影響 speclink-cli、speclink-host、server 與 desktop 的程式碼。
