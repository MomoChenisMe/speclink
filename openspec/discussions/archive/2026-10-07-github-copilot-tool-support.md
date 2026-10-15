---
topic: 使用者希望可以增加支援github copilot，目前支援claude、codex而已
slug: github-copilot-tool-support
status: promoted
created: 2026-10-07
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: add-copilot-tool
---

# Discussion: 使用者希望可以增加支援github copilot，目前支援claude、codex而已

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者希望 speclink 在 claude、codex 之外增加支援 GitHub Copilot。需求目標可驗證（init／desktop 能選 Copilot 並產出 Copilot 讀得到且內容正確的技能），未經 grill 直接進假設；使用者要求先查證 Copilot 技能資料夾的正式位置與注意事項再下結論，並順帶提到 Copilot 也支援 AGENTS.md。
偵查相關：正式規格 workspace-tools（init 內建 Agent 工具選擇、built-in tools 權威收斂、中性渲染目標的 golden 鎖定）、node-sdk（skills.render target）、workspace-chooser、desktop-config、user-documentation，進行中變更 sync-specs-after-docs-cleanup: user-documentation；程式碼 crates/engine/speclink-core/src/workspace/skills.rs（Tool 列舉）、crates/adapters/speclink-cli/src/verbs/init.rs、crates/adapters/speclink-node/src/render.rs、apps/desktop 的 WorkspaceChooser.tsx／ProjectSettingsView.tsx／core/src/project.rs；歷史決定 commit 9d67e7d1（2026-07-02 把工具矩陣收回 claude＋codex）。
Prior discussions: agent-tool-sdk-layering, init-marker-openspec-alignment, node-sdk-completion-and-doc-alignment

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-07)

**Focus**: 「支援 Copilot」在 speclink 裡要做什麼——補文件、用自訂描述子，還是新增第三個內建工具
**Position**: 初始假設為新增第三個內建工具 copilot，使用者未逐條裁定，要求先查證 Copilot 技能資料夾的正式位置與注意事項。決策樹：
- A 寫程式或只補文件：建議新增內建工具。Copilot 已讀 .claude/skills 與 .agents/skills，但讀到的版本錯——codex 版的技能互相引用寫 $speclink-（Codex 語法，Copilot 用 /），claude 版帶 agent: Explore、disallowedTools、Claude fork context 前言，且變更記錄成 --agent claude
- 重開 commit 9d67e7d1（2026-07-02 收回 claude＋codex）的理由：當時每多一個工具就多一種指令檔格式；如今 Copilot 讀同一份 SKILL.md，多一個工具只多一個目錄與一個前綴
- B 目錄：初判 .github/skills/（待查證）
- C 內文：照 codex 版產生，前綴換 /speclink-，技能子集沿用 for_codex，--agent 自動成 copilot（--agent 是自由字串，引擎不需改）
- D 同時勾 claude＋copilot 會重複：初判不繞開、只寫文件＋手動任務實測
- E 不產生 .github/copilot-instructions.md：沿用 init-marker-openspec-alignment 的「不注入指令檔、路由由技能承載」
- F 改動面：CLI init（--tools＋互動第三題）、desktop 兩處勾選與過期偵測、Node SDK render target、copilot golden＋assets.lock、文件
**Ruled out**: 只補文件指向既有產出——Copilot 會持續拿到錯的叫用語法與錯的 agent 記錄
**Open**: Copilot 技能的正式目錄；同名技能的處理規則；Copilot 專用 frontmatter 的限制；AGENTS.md 支援對本題是否有影響；Copilot 雲端 agent 的執行環境

### Round 2 — assumptions (2026-10-07)

**Focus**: 查證 Copilot 技能資料夾的正式位置、同名處理與其他注意事項（使用者要求先查再下結論）
**Position**: 目錄定為 .github/skills/，理由從「Copilot 自家目錄」升級為「官方文件記載的最高優先序」；假設 D 的重複問題由 Copilot 自己解掉，不需手動任務以外的處理。
- 七個 Copilot 產品（雲端 agent、code review、CLI、Copilot app、VS Code、JetBrains、Visual Studio）都讀專案的 .github/skills、.agents/skills、.claude/skills；docs.github.com 的 about-agent-skills 未標示哪個是推薦
- Copilot CLI 指令參考「Skill locations」：依序 .github/skills → .agents/skills → .claude/skills，同名「first found wins」；copilot skill add --project 寫入 .github/skills；ignoredSkillsLocations 可排除目錄
- VS Code PR #320695（2026-06-09 合併、1.125）：同名技能依固定優先序去重，不再在 / 選單出現兩次；VS Code 內各目錄的確切先後未寫明，留給手動任務實測
- 官方工具彼此不一致：gh skill install --agent github-copilot（gh ≥2.90）把專案技能裝到 .agents/skills（與 Codex、Cursor、Gemini CLI 共用）
- 選 .github/skills 而非 .agents/skills：.agents/skills 已被 codex 版佔用（$speclink- 前綴），兩者同選會打架；.github/skills 只有 Copilot 讀、優先序最高，同時勾 claude／codex 時 Copilot 仍拿到自己的正確版本
- 現況副作用：今天 claude＋codex 專案在 Copilot CLI 會優先吃 .agents/skills 的 codex 版
- frontmatter：agentskills.io 規範 name 1–64 字小寫英數與連字號、須等於目錄名、description ≤1024；speclink 產出全部符合（最長 description 273 字）；Copilot 認得 argument-hint、user-invocable、disable-model-invocation、allowed-tools，VS Code 另有實驗性 context: fork（需開 github.copilot.chat.skillTool.enabled）；Claude 的 agent、disallowedTools 不在 Copilot 清單內；名稱不合法時 Copilot CLI 每次啟動都報錯（copilot-cli #2689）
- 叫用：CLI 與 VS Code 都用 /技能名
- AGENTS.md（使用者題外話）屬實：Copilot CLI 讀 git root 與 cwd 的 CLAUDE.md、AGENTS.md、.github/copilot-instructions.md；VS Code 的 Copilot harness 讀 copilot-instructions.md 或 AGENTS.md；雲端 agent 2025-08-28 起支援 AGENTS.md。speclink 已定不寫任何指令檔，對本題無動作
**Ruled out**: 目錄用 .agents/skills——與 codex 版同目錄、前綴語法衝突，要統一就得改 codex 的 golden，超出範圍；copilot 版加 context: fork——VS Code 標實驗性且要另開設定，CLI 文件未列；allowed-tools 預先核准 shell——官方文件警告會讓被注入的技能直接執行任意指令
**Open**: 使用者是否接受 .github/skills 與「同名交給 Copilot 優先序處理」；argument-hint 這次加不加；雲端 agent 要先在執行環境裝 speclink CLI 是否延後；--tools 是否接受 gh skill 用的 github-copilot 別名

## Conclusion

**Decision**: 新增第三個內建 Agent 工具 `copilot`，技能寫到 `.github/skills/`，內容照 codex 版產生、只把叫用前綴換成 `/speclink-`；一個變更全包。
- 目錄：`.github/skills/`——Copilot CLI 官方優先序 `.github/skills` → `.agents/skills` → `.claude/skills`、同名先找到者勝，此目錄只有 Copilot 讀且優先序最高；同時勾 claude／codex 時 Copilot 仍拿到 Copilot 版
- 渲染：沿用 codex 版的技能子集（for_codex，共 18 份）與中性 frontmatter（name、description、license、compatibility、metadata），`/speclink:` 換成 `/speclink-`，無 plan 目錄；不帶 Claude 專用的 agent、disallowedTools、fork 前言；不加 context: fork、allowed-tools、argument-hint
- agent 記錄：技能內文的 `--agent {{TOOL}}` 自動成 `--agent copilot`；--agent 是自由字串，引擎不需改
- 同名重複：不寫程式繞開，交給 Copilot 的優先序（CLI 先找到者勝、VS Code 1.125 起依名稱去重）；tasks 加一個 [M] 手動任務，在 VS Code 同時勾 claude＋copilot 時確認 / 選單只出現一份且用到 Copilot 版
- 不產生任何指令檔（.github/copilot-instructions.md、AGENTS.md 皆不寫），沿用「路由由技能承載」的既定決定
- 工具名只收 `copilot`，不收 gh skill 用的 `github-copilot` 別名
- 改動面：引擎 Tool 列舉與其 golden（新增 copilot 樣本，依內嵌資產版本鎖定紀律處理 assets.lock 與版號）；CLI init 的 --tools 與互動詢問第三題、錯誤訊息；desktop 開專案選擇畫面與設定頁的勾選、技能過期偵測；Node SDK skills.render 新增 copilot target；中英文件（getting-started、configuration）
- 既有規則的連帶效果要寫進提案：自訂描述子 name 為 copilot、或 skills_dir 為 `.github/skills` 者，升級後會因「與內建工具衝突」被拒；取消勾選 copilot 時只移除 `.github/skills/` 下 speclink- 前綴的目錄，使用者自己的技能不動
- 規格面：workspace-tools（init 內建 Agent 工具選擇、built-in tools 權威收斂、中性渲染目標的 golden、描述子驗證的內建目錄清單）、node-sdk、workspace-chooser、desktop-config、user-documentation
- 範例：`speclink init --tools claude,copilot` 產生 `.claude/skills/speclink-*/` 與 `.github/skills/speclink-*/`；`.github/skills/speclink-propose/SKILL.md` 的 frontmatter 沒有 agent、disallowedTools 行，內文引用寫 `/speclink-apply`，建立變更的指令是 `speclink new change "<name>" --agent copilot`
**Rationale**: Copilot 今天已經讀得到 speclink 的技能，但讀到的是錯的版本（codex 版的 `$speclink-` 語法、claude 版的專用欄位與 agent 記錄），所以要的是一份正確的 Copilot 版而不是「讓它看得見」；2026-07-02 把工具矩陣收回 claude＋codex 的理由是每多一個工具就多一種指令檔格式，如今 Copilot 讀同一份 SKILL.md，多一個工具只多一個目錄與一個前綴，代價低到值得收回該決定。目錄選 `.github/skills` 而不是 gh skill 預設的 `.agents/skills`，關鍵在後者已被 codex 版佔用且前綴語法互斥，而前者在 Copilot 的優先序最高、同選時自然勝出。
**Rejected alternatives**:
- 只補文件、指向既有 `.claude/skills` 或 `.agents/skills` 產出 — Copilot 會持續拿到錯的叫用語法與錯的 agent 記錄
- 文件教使用者用自訂描述子 — 中性渲染會拿掉 `/speclink-` 引用，且不在 init 與 desktop 的選項裡，使用者找不到
- 技能目錄用 `.agents/skills` — 與 codex 版同目錄、前綴語法衝突，要統一就得改 codex 的 golden，超出範圍
- 勾了 claude 就不寫 `.github/skills` 以避開重複 — Copilot 會改讀 Claude 版的錯內容，且 Copilot 已自行處理同名
- Copilot 版加 context: fork — VS Code 標示實驗性且要另開設定，Copilot CLI 文件未列
- allowed-tools 預先核准 shell — 官方文件警告被注入的技能可直接執行任意指令
- 產生 .github/copilot-instructions.md 或 AGENTS.md — 已定不注入指令檔、路由由技能承載
**Deferred**:
- Copilot 雲端 agent 要在 GitHub 執行環境（copilot-setup-steps）先裝 speclink CLI — 屬安裝與環境設定，與技能產出無關，等有人實際要在雲端 agent 跑再處理
- argument-hint 欄位 — 只是輸入框提示，codex 版也沒有，目前沒有需求
- github-copilot 別名 — 目前沒有使用者需要
- 把 speclink 嵌進 Copilot SDK 當工具 — 屬 agent-tool-sdk-layering 討論的範圍
**Capture to**: proposal | design | spec | tasks
**Next**: /speclink-propose --from-discussion github-copilot-tool-support
