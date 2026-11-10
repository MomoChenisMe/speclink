---
topic: 新增 pi、cursor 為內建 AI 工具（.speclink.yaml 的 tools、desktop 專案設定 .speclink.yaml 頁的 AI 工具勾選、init --tools）
slug: pi-cursor-tool-support
status: promoted
created: 2026-11-10
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: add-pi-cursor-tools
---

# Discussion: 新增 pi、cursor 為內建 AI 工具（.speclink.yaml 的 tools、desktop 專案設定 .speclink.yaml 頁的 AI 工具勾選、init --tools）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者要求在 claude、codex、copilot 之外新增 pi 與 cursor 兩個內建 AI 工具，涵蓋 .speclink.yaml 的 tools、desktop 專案設定 .speclink.yaml 頁的 AI 工具勾選與 init --tools。此題由 station-subagent-model 第 10 輪拆出（子代理模型表格的列取自 Tool::ALL，兩個工具加進來後會自動多兩列）。需求目標可驗證（init、desktop、.speclink.yaml 都能選 pi、cursor，並產出各工具讀得到、叫用語法正確的技能），未經 grill 直接進假設；使用者先附上 pi 與 cursor 的官方文件事實，討論中再以 pi 原始碼補查。
偵查相關：正式規格 workspace-tools（init 內建 Agent 工具選擇、built-in tools 權威收斂、tools 自訂描述子的接受與驗證、技能檔過期探測、Copilot 渲染目標）、node-sdk（渲染 API）、workspace-chooser、desktop-config、user-documentation；目前沒有進行中的變更；程式碼 crates/engine/speclink-core/src/workspace/skills.rs（Tool 列舉、slash_replacement）、workspace/init.rs（detect_footprint_tools）、workspace/config.rs（描述子保留名稱與目錄）、crates/adapters/speclink-cli/src/verbs/init.rs（互動詢問）、crates/adapters/speclink-node/src/render.rs 與 index.d.ts、apps/desktop/src/builtinTools.ts、integrations/herdr/mods/speclink/model.js（面板的工具前綴與技能目錄對照）。
Prior discussions: github-copilot-tool-support, station-subagent-model, agent-tool-sdk-layering, init-marker-openspec-alignment

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-11-10)

**Focus**: pi、cursor 加成內建工具的目錄、叫用語法、內容與改動面——比照 Copilot 的做法是否全部適用
**Position**: 使用者確認八條假設全部成立，做法比照 add-copilot-tool：一個變更新增 pi、cursor 兩個內建工具，Tool::ALL 順序為 claude、codex、copilot、pi、cursor。
- 事實更正：pi 文件只寫「user or project skills directory」，pi 原始碼（earendil-works/pi 的 packages/coding-agent/src/core/package-manager.ts）的掃描順序為 .pi/skills → .agents/skills（往上找到 repo 根）→ ~/.pi/agent/skills → ~/.agents/skills；同名技能保留先找到的並顯示警告；專案層兩個目錄只在專案被信任（isProjectTrusted）時才讀；repo 已搬到 earendil-works/pi
- pi：技能寫到 .pi/skills/，引用寫 /skill:speclink-<name>，--agent pi；同時勾 codex 時 pi 仍拿到 pi 版（.pi/skills 最先掃），代價是每次啟動 18 則同名警告，接受並寫進文件
- cursor：技能寫到 .cursor/skills/，引用寫 /speclink-<name>，--agent cursor；Cursor 也讀 .agents/skills、.claude/skills、.codex/skills，文件沒寫同名時用哪份，比照 Copilot 不寫程式繞開、以 [M] 任務實測
- 內容：兩者都照 codex 版的技能子集（18 份，不含 analyze）與中性 frontmatter，只換叫用前綴與 {{TOOL}}；Cursor 文件列的欄位不含 license、compatibility，預期略過，併入 [M] 任務確認
- 子代理：不產生 .cursor/agents/、不改技能內文；pi 不能派子代理時套用 station-shared.md 既有的「主對話自己跑檢查」
- 不產生任何指令檔；footprint 偵測不因 .pi/、.cursor/ 推斷新工具（很多 repo 有 .cursor/ 只為放 rules）
- 改動面：引擎 Tool 列舉與兩份新 golden、assets.lock 與版號；CLI init --tools、互動詢問由三題變五題、錯誤訊息；desktop BUILTIN_TOOLS；Node SDK skills.render 的 target；中英文件；workspace-tools 寫死「三個工具名」「三題」處要改（預期內的正式規格衝突）
- 連帶效果：名為 pi、cursor 或 skills_dir 指向 .pi/skills、.cursor/skills 的自訂描述子升級後被拒；station-subagent-model 的表格自動多兩列
- 盤點另發現：Herdr 面板 integrations/herdr/mods/speclink/model.js 已列 copilot，且目前把 pi 對到 codex（SPECLINK_TOOL.pi = "codex"，缺技能提示寫 --tools codex）；Herdr 程式內建 cursor 程式定義（match cursor-agent），面板尚無 cursor 項目
**Ruled out**: pi 與 codex 共用 .agents/skills——叫用前綴互斥（$ 與 /skill:），統一就得改 codex 的 golden，與 github-copilot-tool-support 否決 .agents/skills 的理由相同；產生 .cursor/agents/ 子代理檔——沒被要求，模型設定屬 station-subagent-model；footprint 偵測推斷 pi、cursor——.cursor/ 常只放 rules，desktop 會預選使用者沒要的工具
**Open**: Herdr 面板要不要一起改（pi 改對到 pi、補 cursor 的前綴與技能目錄）

## Conclusion

**Decision**: 新增第四、第五個內建 Agent 工具 `pi` 與 `cursor`，比照 add-copilot-tool 的做法，一個變更全包。
- 順序：Tool::ALL 為 claude、codex、copilot、pi、cursor；CLI 提示、互動詢問、desktop 勾選、錯誤訊息都照這個順序
- pi 目錄：`.pi/skills/`——pi 原始碼的掃描順序 .pi/skills → .agents/skills → ~/.pi/agent/skills → ~/.agents/skills、同名先找到者勝，此目錄只有 pi 讀且最先掃；同時勾 codex 時 pi 仍拿到 pi 版，代價是 pi 每次啟動對 18 份同名技能各顯示一則警告，接受並寫進文件
- pi 叫用：`/speclink:` 換成 `/skill:speclink-`，`{{TOOL}}` 為 `pi`、`{{PLAN_DIR}}` 為空字串
- cursor 目錄：`.cursor/skills/`——只有 Cursor 讀；叫用 `/speclink:` 換成 `/speclink-`，`{{TOOL}}` 為 `cursor`、`{{PLAN_DIR}}` 為空字串
- 渲染：兩者都沿用 codex 版的技能子集（for_codex，共 18 份，不含 analyze；worktree 政策關閉時再排除兩顆 worktree 技能）與中性 frontmatter（name、description、license、compatibility、metadata）；不帶 Claude 專用的 agent、disallowedTools、fork 前言；除叫用前綴與 `{{TOOL}}` 以外與 codex 版逐字相同
- 子代理：不產生 `.cursor/agents/`、不改技能內文；pi 沒有子代理時由技能既有的「harness 不能派子代理時主對話自己跑檢查」承接；子代理模型設定屬 station-subagent-model，其表格的列取自 Tool::ALL 會自動多兩列
- 同名重複：不寫程式繞開。tasks 加 [M] 手動任務：同時勾 claude、codex、cursor 時，在 Cursor 編輯器與 cursor-agent 確認 / 選單用到 Cursor 版（引用寫 /speclink-、--agent cursor），並確認 Cursor 接受 license、compatibility 欄位；同時勾 codex、pi 時確認 pi 用到 pi 版
- 不產生任何指令檔（AGENTS.md、CLAUDE.md、.cursor/rules 皆不寫），沿用「路由由技能承載」的既定決定
- footprint 偵測（.speclink.yaml 無 tools 時的遺留回退、desktop checkout 預選）不因 `.pi/`、`.cursor/` 存在而推斷新工具
- 工具名只收 `pi`、`cursor`，不收 cursor-agent 等別名
- 改動面：引擎 Tool 列舉與兩份新 golden（pi、cursor 樣本，依內嵌資產版本鎖定紀律處理 assets.lock 與版號）；描述子保留名稱與目錄自動跟著 Tool::ALL；CLI init 的 --tools、互動詢問由三題變五題、TOOLS_HINT 與「Pick at least one tool」訊息；desktop BUILTIN_TOOLS（開專案選擇畫面、設定頁 AI 工具卡、checkout 綁定共用）與技能過期偵測；Node SDK skills.render 的 target 型別與錯誤訊息；Herdr 面板 integrations/herdr/mods/speclink/model.js（SPECLINK_TOOL.pi 由 codex 改為 pi、新增 cursor 的前綴 / 與技能目錄 .cursor/skills、.agents/skills、.claude/skills、.codex/skills 及其家目錄版本）與 scripts/herdr/panel-model.test.mjs；中英文件（getting-started、configuration，寫明 pi 同名警告與 pi 需信任專案才讀專案技能）
- 既有規則的連帶效果要寫進提案：自訂描述子 name 為 pi 或 cursor、或 skills_dir 為 `.pi/skills`、`.cursor/skills` 者，升級後會因「與內建工具衝突」被拒；取消勾選 pi 或 cursor 時只移除該目錄下 speclink- 前綴的目錄，使用者自己的技能不動；Herdr 面板對 pi 的缺技能提示改為 `--tools pi`
- 規格面：workspace-tools（init 內建 Agent 工具選擇的工具名與題數、built-in tools 權威收斂、描述子驗證的內建名稱與目錄清單、比照「Copilot 渲染目標」新增 Pi 與 Cursor 渲染目標需求並把差異表擴成五列）、node-sdk、workspace-chooser、desktop-config、user-documentation
- 範例：`speclink init --tools codex,pi,cursor` 產生 `.agents/skills/speclink-*/`、`.pi/skills/speclink-*/`、`.cursor/skills/speclink-*/` 各 18 份；`.pi/skills/speclink-propose/SKILL.md` 內文引用寫 `/skill:speclink-apply`、建立變更帶 `--agent pi`；`.cursor/skills/speclink-propose/SKILL.md` 內文引用寫 `/speclink-apply`、建立變更帶 `--agent cursor`；兩者 frontmatter 都沒有 agent、disallowedTools 行
**Rationale**: 每個工具都讀同一份 SKILL.md 格式，多一個內建工具只多一個目錄、一個叫用前綴與一個 `--agent` 值；給每個工具它自己最先讀的目錄，就能在多工具並存時仍拿到叫用語法正確的版本，代價只是 pi 的同名警告與 Cursor 未記載的同名規則，前者可接受、後者交給手動實測。
**Rejected alternatives**:
- pi 與 codex 共用 `.agents/skills` — 叫用前綴互斥（`$speclink-` 與 `/skill:speclink-`），統一就得改 codex 的 golden，與 github-copilot-tool-support 否決 .agents/skills 的理由相同
- 只補文件、讓 pi 讀 codex 版、Cursor 讀既有目錄 — pi 與 Cursor 會持續拿到錯的叫用語法與錯的 `--agent` 記錄（Herdr 面板目前就是把 pi 對到 codex）
- 產生 `.cursor/agents/` 子代理檔 — 沒被要求，模型設定屬 station-subagent-model
- footprint 偵測推斷 pi、cursor — 很多 repo 有 `.cursor/` 只為放 rules，desktop 會預選使用者沒要的工具
- 分成 pi、cursor 兩個變更 — 兩者改同一批檔（Tool 列舉、assets.lock、版號），分刀會讓版號行對撞
**Deferred**:
- 使用者層安裝（~/.pi/agent/skills、~/.cursor/skills）— speclink 對所有內建工具都只產專案層技能，沒有使用者提出需要
- cursor-agent、pi-coding-agent 等工具名別名 — 目前沒有使用者需要
- 把 speclink 嵌進 pi 當工具（extension 或 tool-call）— 屬 agent-tool-sdk-layering 討論的範圍
**Capture to**: proposal | design | spec | tasks
**Next**: /speclink-propose --from-discussion pi-cursor-tool-support
