## Context

Baseline 技能（事實來源 crates/speclink-core/assets/skills/baseline.md）盤點既有 codebase 後直接把正式 specs 寫進 openspec/specs/，不建立 change。openspec/config.yaml 的 rules 是「依 artifact 注入產出指令」的清單：一般 artifact 由 speclink instructions <artifact> --json 的 payload 帶 rules 給 Agent，但該動詞在沒有 active change 時只回「No active changes」，Baseline 因此拿不到 rules.specs。現行 baseline.md 的 Step 1 直讀 config.yaml 取 context 與 spec_locale，沒有 rules。

既有可沿用的入口：speclink workflow-config show --json 是 Dual 動詞，fs 與 remote 模式共用同一個輸出函式（crates/speclink-cli/src/verbs/config.rs 的 print_workflow_config），payload 含 context、specLocale 與 rules（artifact id 對規則字串陣列），形狀由 workflow-config 規格與 crates/speclink-cli/tests/it/workflow_config.rs 保護；config 無法解析時 fail-closed。

技能內文的行為斷言集中在 crates/speclink-core/tests/it/render_golden.rs：以 skill_for_both_tools 讀取 claude 與 codex 兩側的生成技能檔，逐一斷言 needle 存在或不存在；渲染產物整體由五份 golden snapshot 與 assets.lock 保護，ASSET_VERSION（crates/speclink-core/src/init.rs）現為 v1.26.1。

Baseline 沒有自己的 capability spec；skill-routing 的 Purpose 明文「不管單一技能的內文行為（那屬各 per-skill capability）」。討論 baseline-apply-spec-rules（2026-09-02）已定案九條假設，本設計只落實，不再開新分支。

## Goals / Non-Goals

**Goals:**

- Baseline 盤點前取得正典 workflow config（不套用環境變數覆寫），繼續使用 context 與 specLocale，並讀到 rules.specs。
- rules.specs 存在時明確套用到本輪產生的每一份正式 spec；不存在時行為不變。
- 使用者在 capability map 確認與最終報告兩處都看得到本輪套用了哪些 specs 產出規則。
- 技能明文載明規則是 Agent 產生內容時的指令，speclink validate 只查結構。
- Baseline 既有六項邊界首次正典化為 baseline-skill capability。
- 生成的技能檔有內容斷言測試；golden、assets.lock、版號、repo 生成物與 workflow 文件同批更新。

**Non-Goals:**

- 不新增或變更任何 CLI 子指令、旗標、stdin、exit code、人眼輸出或 --json payload。
- 不新增或變更 openspec/config.yaml 與 .speclink.yaml 的欄位或預設值。
- 不擴張 Baseline 的 Local／Remote 支援範圍；直接寫檔到 openspec/specs/ 的本機行為不動。
- 不讓引擎機械式驗證 rules.specs 的自由文字；不新增 validate lint。
- 不篩選「哪條規則適用於 baseline」；不翻譯規則原文。
- 不改 skills.rs 的 registry 項（id、description、for_codex、worktree_gated）。
- 不動 skill-routing、workflow-config、config-skill、user-documentation 四份既有規格。
- 不改 docs/configuration 兩份文件。
- 不在 Baseline 引入 speclink language show 或其他新的輸入來源。
- 已封存的討論與變更、@trace 清單一律不回改。

已排除的做法（討論 Ruled out）：規格放進 workflow-config 或 skill-routing（職責混入）；手讀 config.yaml 或自寫 YAML 解析（remote 模式讀不到本機檔、重複引擎邏輯）；沿用 speclink instructions specs --json（無 change 時無 payload）；規則適用性篩選（判準無法固定、報告不可預期）；只升 patch 版號（與新增步驟的份量不符）；擴張 remote 支援（spec 寫回 server 是另一個 change）。

## Decisions

### 設定入口沿用 workflow-config show

Step 1 由「直讀 config.yaml」改為執行 speclink workflow-config show --json，從 payload 讀 context、specLocale、rules.specs 三個欄位。理由：這是既有 Dual 動詞，fs 與 remote 共用同一輸出函式，形狀已由規格與 CLI 測試釘死；技能不必知道設定住在哪裡，符合「storage 解耦的規格驅動引擎」方向——技能只認 CLI contract，不認檔案位置。指令失敗（壞 config 的 fail-closed、remote 離線或認證失效）時技能停下回報，SHALL NOT 退回手讀 YAML；理由：fail-closed 是 workflow-config 的刻意設計，技能自行解析等於繞過它。替代方案：直讀檔案（remote 模式下本機沒有正典檔）、instructions specs（無 change 時無 payload）——皆已在討論排除。

落點：只改 crates/speclink-core/assets/skills/baseline.md 的內文，領域邏輯與 CLI 程式碼零改動；同一份 asset 渲染到 claude 與 codex 兩側，local 與 remote 共用同一契約，不存在平行實作。

### 規則逐字套用不篩選

rules.specs 存在且非空時，Step 4 寫的每一份 spec 都 MUST 遵守每一條規則，原文照套、不翻譯、不判斷「這條是不是 baseline 用得到」。理由：本 repo 的 rules.specs 多為條件句（「動到既有輸出的規格須標明…」），對新 spec 自然不觸發，篩選只會引入 Agent 每次不同的判準，讓報告不可預期。未設定或空清單時，Step 4 的既有規則段不變。

### 揭露段固定格式

capability map 的確認訊息（Step 3）與最終報告（Step 5）帶同一段固定文字。有規則時：首行「Specs rules applied this run (from rules.specs, N entries):」，其後逐條編號列出原文；無規則時：單行「Specs rules applied this run: none (no rules.specs configured)」。理由：同一段兩處出現，使用者在寫入前與寫入後看到的是同一份清單；固定字面讓測試可以 needle 斷言，也讓 Codex 與 Claude 兩側渲染一致。技能同時明文：這些規則是 Agent 產生內容時必須遵守的指令，speclink validate --specs --all --strict 只檢查結構，不驗證自由文字規則。

### 新 capability baseline-skill

新增 openspec 的 baseline-skill capability，三條 requirement：渲染與 golden 保護（比照 config-skill）、盤點前取得 workflow config 並套用 specs 產出規則（本次新行為，含揭露與失敗即停）、基準盤點的行為邊界（既有六項首次正典化）。理由：skill-routing 明文不管單一技能內文；每個流程技能各有一個 per-skill capability 是既定慣例；改名討論當時因行為未變而不建，本次行為改變，理由反轉。既有六項邊界一併寫入是為了讓後續變更不能默默改掉它們——它們此前只受 golden 保護，沒有規格。

### 版號 v1.27.0 與衍生物再生

ASSET_VERSION 由 v1.26.1 升到 v1.27.0（新增一個步驟與兩處揭露段，份量同改名 v1.25.0 與新技能 v1.26.0 的 minor 先例）。同批再生五份 golden（claude、codex、neutral-cli、neutral-tool-call、claude-worktree）與 assets.lock；repo 根的 .claude/skills 與 .agents/skills 由本次建置的 CLI 執行 update 再生，其餘技能只變版本戳。re生 lock 須在乾淨樹上執行，避免把未提交的無關內容燒進指紋。

### 測試落點 render_golden

在 crates/speclink-core/tests/it/render_golden.rs 新增一支測試 baseline_skill_loads_workflow_config_and_applies_specs_rules，沿用 skill_for_both_tools("baseline-rules", "baseline") 讀兩側生成檔，斷言 Implementation Contract 列出的正向 needle 全部存在、負向 needle 不存在。理由：既有的 review、verify、commit、trace 技能行為斷言都在這裡，同一沙盒與同一 init 流程；TDD 順序下先寫此測試（Red），再改 asset（Green）。

## Implementation Contract

**行為（使用者可觀察）**

1. Agent 執行 Baseline 時，Step 1 先跑 speclink list --specs 與 speclink workflow-config show --json。後者輸出正典值（不套用 SPECLINK_* 環境變數覆寫）。從其 payload 取 context（專案說明，作為盤點與每份 spec 的背景）、specLocale（null＝英文；auto＝以同一 payload 的 locale 為準；其他為語系代碼，spec 散文用該語言，結構標記與 SHALL/MUST 維持英文）、rules.specs（字串陣列，或不存在）。
2. rules.specs 非空時，Step 4 的每一份 spec 都遵守每一條規則；capability map 確認訊息與最終報告各帶一段固定揭露段（格式見下）。
3. rules.specs 不存在或為空時，spec 內容規則與現行相同；兩處揭露段各為單行「none」句。
4. speclink workflow-config show --json 以非零 exit code 結束（含壞 config、remote 離線或認證失效）時，技能回報錯誤並停止，不寫任何 spec，不改讀 config.yaml。
5. 其他步驟不變：只記錄現況、不改 code、不建 change、已有 specs 時只補缺、map 確認後才寫、最後 speclink validate --specs --all --strict、出口只有 propose 與 discuss。

**技能檔的固定字面（渲染後兩側相同；測試的 needle）**

正向 needle（每個都 MUST 出現在 .claude/skills/speclink-baseline/SKILL.md 與 .agents/skills/speclink-baseline/SKILL.md）：

| 用途 | needle |
| --- | --- |
| 取得 config | speclink workflow-config show --json |
| 讀規則欄位 | rules.specs |
| 每份 spec 遵守 | MUST honour every entry |
| 未設定照常 | none (no rules.specs configured) |
| 揭露段首句 | Specs rules applied this run |
| 揭露段完整首行 | (from rules.specs, |
| 規則非機械驗證 | checks structure only |
| 失敗即停不手讀 | never fall back to reading |

負向 needle（MUST NOT 出現）：(project context, and `spec_locale` — write spec prose（舊 Step 1 直讀句的片語；不依賴 spec_dir 渲染與 read 的大小寫）。

揭露段格式（Step 3 與 Step 5 共用）：

有規則時——

    Specs rules applied this run (from rules.specs, N entries):
    1. <第一條原文>
    2. <第二條原文>

無規則時——

    Specs rules applied this run: none (no rules.specs configured)

**asset 改動的段落**（crates/speclink-core/assets/skills/baseline.md）

- Step 1：標題改為含 load the workflow config；程式碼區塊加 speclink workflow-config show --json；刪除直讀 config.yaml 的句子；新增三欄位說明與失敗即停句。
- Step 3：capability map 呈現後補「確認訊息 MUST 帶揭露段」與兩種格式。
- Step 4 的 Rules：新增一條「Honour every rules.specs entry…validate checks structure only」。
- Step 5：報告項目新增「the specs rules applied this run（同 Step 3 的段落）」。
- Guardrails：新增「Don't apply rules silently」與「Don't hand-read config.yaml」兩條。
- Step 4 既有規則「prose in `spec_locale`」改為「prose in the `specLocale` language」，同一概念只用 payload 的名字。
- 其餘段落（Step 2 盤點、Step 4 的模板與其他 Rules、既有 Guardrails、Next steps）逐字不動。

**版號與衍生物**

- crates/speclink-core/src/init.rs 的 ASSET_VERSION 為 "v1.27.0"。
- 五份 golden 與 assets.lock 再生後的 diff 只含 baseline 段落的變更與版本戳。
- repo 根 .claude/skills 與 .agents/skills 全部 speclink-* SKILL.md 的 frontmatter 版本戳為 v1.27.0；speclink-baseline/SKILL.md 含上表全部正向 needle。

**文件**

- docs/workflow.md 的 baseline 站 Input 行列入 the workflow config from speclink workflow-config show --json（context、specLocale、rules.specs）。
- docs/workflow.zh-TW.md 同站的 Input 行列入以 speclink workflow-config show --json 取得的 workflow config（專案說明 context、specLocale 與 specs 產出規則 rules.specs）。兩語言同段對等。

**驗收**

- cargo test -p speclink-core --test it baseline_skill_（新測試）綠。
- cargo test -p speclink-core --test it 全綠（含 render_golden 五份與資產版本鎖）。
- node --test scripts/vocabulary-guard.test.mjs 綠。
- cargo fmt --all -- --check 無差異。
- speclink validate baseline-apply-spec-rules 與 speclink validate --specs --all --strict 通過。

**範圍邊界**

- In scope：上述 asset 段落、版號、golden、lock、測試、repo 生成物、workflow 兩語言文件、baseline-skill delta spec。
- Out of scope：CLI／host／server／desktop 程式碼；skills.rs；其他技能 asset；configuration 文件；任何 config 欄位。

## Risks / Trade-offs

- [五份 golden 與 assets.lock 是位元級對照，手動編輯必錯] → 一律用 UPDATE_GOLDEN=1 與 UPDATE_ASSETS_LOCK=1 走 render_golden 測試再生，再不帶環境變數重跑確認全綠；lock 再生前以 git status 確認樹乾淨。
- [repo 根跑 update 若打到 ~/.local/bin 的舊安裝版，會靜默套舊內文] → 以本次建置的 CLI（cargo run -p speclink-cli -- update）執行，並 grep 生成檔確認含 speclink workflow-config show --json。
- [版本戳同時改動全部生成 SKILL.md，git status 會出現三十多個檔] → 收尾以 git status 逐一盤點，只加入本 change 的檔案；不使用 git add -A。
- [跨平台：golden 比對含換行] → render_golden 既有 normalize_eol 正規化，新測試只做 contains 斷言，不比對換行。
- [規則原文含 Markdown 或反引號時，揭露段的編號清單可能被渲染器吃掉格式] → 技能要求逐條「原文」列出，不做二次格式化；使用者看到的即 config 內文。
- [rules.specs 很長時 map 確認訊息變長] → 接受；揭露的目的就是讓規則不默默生效，長度是內容決定。
- [remote 模式下 Baseline 仍直接寫本機 openspec/specs/] → 本次不擴張，明列於 Non-Goals；config 讀取走 Dual 動詞後至少不會讀錯設定。
- [vocabulary guard 可能對 docs 的「規則」用字誤報] → 文件用正典詞「產出規則」並把 rules.specs 放 code span。

## Migration Plan

- 使用者升級 CLI 後在 workspace 執行 speclink update；update 的既有版本比對會把全部生成技能檔換到 v1.27.0。
- 無資料遷移；config.yaml 不需改動。
- 回滾：還原本 change 的提交並重跑 speclink update 即回到 v1.26.1 的技能檔。

## Open Questions

無。九條設計假設已在討論 baseline-apply-spec-rules 全數確認。
