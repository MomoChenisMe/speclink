---
topic: 專案設定裡的AI 工具現在只有claude 和 codex , 我想要增加 Copilot
slug: copilot-project-tool
status: promoted
created: 2026-10-07
created_by: mosutw <luke1974@gmail.com>
promoted_to: copilot-project-tool
---

# Discussion: 專案設定裡的AI 工具現在只有claude 和 codex , 我想要增加 Copilot

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者希望專案設定的 AI 工具加入 Copilot，並已確認同時支援 Copilot CLI 與 VS Code。這次聚焦有專案 checkout 的技能安裝與更新；Copilot SDK 嵌入式 runtime 的工具呼叫是鄰近但不同的需求。需先釐清使用入口，因此先問 CLI／VS Code 範圍，使用者回答「都支援」；後續確認技能目錄、內容與多工具共存，再決定設定／初始化／更新的完整交付。
相關正式規格：workspace-tools（內建工具目前 claude／codex、自訂描述子、中性渲染、同步／清理）、desktop-config（AI 工具勾選與自訂描述子保留）。相關程式：crates/engine/speclink-core/src/workspace/skills.rs、config.rs、init.rs；apps/desktop/src/views/ProjectSettingsView.tsx、apps/desktop/src/components/WorkspaceChooser.tsx。官方證據：https://docs.github.com/en/copilot/concepts/agents/about-agent-skills、https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/add-skills、https://code.visualstudio.com/docs/agent-customization/agent-skills。三份文件确认 Copilot 可讀專案技能；CLI 與 VS Code 都接受 .github/skills、.claude/skills、.agents/skills。
Prior discussions: agent-tool-sdk-layering（open；目前談嵌入式 SDK／技能投遞，與本次專案設定整合分開）、node-sdk-completion-and-doc-alignment（Copilot tools adapter 遞延，觸發條件為其他 agent 平台要接）、release-first-and-distribution（後續功能依需求另案討論）、collab-scenario-replan（Agent 生態視需求成刀）、manual-spec-edit-integrity（桌面不內嵌 agent）、sdd-engine-as-sdk-with-pluggable-document-storage-for-team-scenarios（技能與引擎共享、宿主自行選擇 CLI 或工具呼叫）

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — interview (2026-10-07)

**Focus**: 專案設定新增 Copilot 要支援哪個使用入口
**Position**: 使用者確認同時支援 Copilot CLI 與 VS Code。
- 兩者共用專案技能交付，後續需確認技能目錄、內容與呼叫方式，以及初始化／設定儲存／更新的同步行為。
- 正式規格 workspace-tools 與 desktop-config 目前只列 claude／codex，加入內建 copilot 需明確修改既有契約；自訂描述子與中性渲染則是可沿用的既有能力。
- 官方文件确认兩入口皆支援 Agent Skills；現有 agent-tool-sdk-layering 討論聚焦嵌入式 SDK，此次專案技能整合不以它完成為前提。
**Open**: 技能目錄（.github/skills 或共用 .agents/skills）與 Claude／Codex 同名技能共存；技能內容與呼叫方式；是否一併涵蓋 CLI init/update、桌面新專案及設定同步；停用清理與既有自訂 copilot 描述子相容性。

### Round 2 — interview (2026-10-07)

**Focus**: Copilot 的技能檔是否與 Codex 共用專案目錄
**Position**: 使用者選擇與 Codex 共用 .agents/skills，不另建 .github/skills。
- Copilot CLI 與 VS Code 的官方技能目錄均接受 .agents/skills；沿用已確認的雙入口範圍。
- 現行 Codex 技能的引用前綴是 $speclink-，內建同步與清理仍以每個工具各自目錄處理；加入共享目錄需要同一份產物與共同保留判定，不能讓兩個工具輪流生成不同內容或由其中一個停用就刪掉另一個仍需的檔案。
- workspace-tools 的中性渲染是已有機制，Codex golden 逐位元保護則是既有契約；若選擇共用中性內容，需在提案中明示 Codex 產物的刻意變更並更新回歸快照。
**Ruled out**: Copilot 另用 .github/skills——使用者選擇共用；同一路徑由 Codex 與 Copilot 各自生成不同內容——產物互相覆寫、過期判定與停用清理無法一致。
**Open**: 是否讓 Codex 與 Copilot 共用中性技能內容（技能引用用名稱，動詞用 speclink CLI）；是否一併涵蓋 CLI init/update、桌面新專案及設定同步；既有自訂 copilot 描述子相容性。

### Round 3 — interview (2026-10-07)

**Focus**: 共用目錄是否也使用一致的技能內容
**Position**: 使用者接受 Codex 與 Copilot 統一成共用內容，包含既有 Codex 技能文字與快照的刻意更新。
- 技能用 speclink-apply 等名稱引用，不在共享本文綁定 Codex 的 $ 前綴；真正的流程操作仍執行 speclink CLI。
- 僅 Codex、僅 Copilot、兩者同時選取，皆生成同一套 .agents/skills/speclink-*/SKILL.md；兩者皆停用才清理共享受管技能，其他使用者技能保留。
- 沿用既有技能資產與中性渲染機制，但必須區分技能呼叫與 CLI 動詞；不能因移除前綴而把 speclink-apply 技能改成不存在的 speclink apply 子指令。
- 不以複製整套技能資產或增加新 SDK／MCP 層解決共用；更新與過期判定須讀取同一份共用期望內容。
**Ruled out**: 維持不同的 Codex／Copilot 本文並寫入同一目錄——會互相覆寫；盲目將所有技能入口改寫為 CLI 子指令——技能與 CLI 動詞不一一對應。
**Open**: 設定、新專案初始化與 CLI init/update 的完整交付邊界需在結論明列；既有名為 copilot 的自訂描述子相容性與實機驗收细節留待提案查證及設計，不默默覆寫使用者設定。

## Conclusion

**Decision**: 在專案 AI 工具新增內建 copilot，同時支援 Copilot CLI 與 VS Code，與 Codex 共用 .agents/skills 及一致的技能內容。
- 設定清單接受 copilot；桌面專案設定、新專案初始化／既有工作區啟用、CLI init 的顯式與互動工具選擇、update、遠端 checkout 的技能生成與工具同步，均沿用同一工具選擇及同步流程接入。CLI init 的非互動選擇與零寫入拒絕規則維持；空工具選集仍依既有契約拒絕。
- 僅 Codex、僅 Copilot、Codex 與 Copilot 同時啟用，皆在 .agents/skills/speclink-*/SKILL.md 生成同一份內容；不另建 .github/skills、不重複生成第二套技能資產、不依選取順序改變產物。
- 共享本文以 speclink-apply 等技能名稱引用流程，不綁定 $speclink- 前綴；實際讀寫操作依既有 speclink CLI 動詞執行。技能呼叫與 CLI 動詞必須分清，不得生成不存在的 speclink apply／speclink propose 子指令。Copilot 的斜線入口由工具自身技能機制提供，本文與文件分清兩者。
- 沿用既有技能資產、中性渲染與非 Claude 技能集合，worktree 政策仍決定相關技能是否生成；不為 Copilot 再造一層 SDK、MCP 或桌面內嵌 agent。Claude 的技能與受管目錄維持既有方式。
- 以共享目錄的使用者集合判斷保留與清理：Codex 或 Copilot 任一仍啟用就保留；兩者皆停用才移除其中 Speclink 管理的技能，保留使用者自己的技能。同步、過期判定、孤兒清理與降級守門必須共用同一個期望產物集合；相同選集重試收斂。
- Codex 既有技能本文與 golden 快照的更新屬刻意相容性變更，提案、規格及文件必須明示；使用者原來的技能名稱不變。不新增 AGENTS.md 或 copilot-instructions.md 的注入，不修改 AI 工具的使用者設定檔，不自動授予終端機權限。
- 具體驗收：勾選 Codex 與 Copilot後，共享目錄只有一套 speclink-*；取消 Codex、仍保留 Copilot後檔案內容不變；改為只有 Codex也不變；改為只選 Claude時共享受管技能才清除，使用者自己的技能留下。Copilot CLI 與 VS Code 均應能發現並使用共享技能，驗收須檢查實際選到的技能及其內容；Claude 同時存在時也要驗證同名技能的載入，不能僅以檔案存在聲稱支援。
- 規格落點沿用 workspace-tools 與 desktop-config 的 delta；若提案查證顯示其他既有能力契約受到影響，隨同明列，不直接改寫正式規格。
**Rationale**: 使用者已確認雙入口、共用目錄及共用內容；兩入口的官方文件都接受 .agents/skills。現有 workspace-tools 已有共享資產、中性渲染與集中同步，完整接入工具選擇比只增加畫面選項更符合可用的專案工具。共用檔案必須有共同保留判定，否則停用其中一個工具會誤刪另一個工具仍需要的技能；Codex 本文輸出需明示刻意更新，避免兩种產物互相覆寫。
**Rejected alternatives**:
- Copilot 另寫 .github/skills — 使用者裁定共用 .agents/skills，兩入口已支援。
- 同一路徑各自渲染 Codex 與 Copilot 版本 — 產物互相覆寫，過期判定與清理不一致。
- 保留 Codex 專用 $ 前綴作為共享本文 — 使用者接受改為共用內容。
- 只新增 UI 勾選項 — 引擎與 CLI 仍拒絕 copilot，無法交付可用的工具設定。
- 為此新增 SDK／MCP 工具層或嵌入 agent — 本次只需專案技能投遞；agent-tool-sdk-layering 的嵌入式需求繼續由原討論處理。
- 把技能名稱直接當 CLI 子指令 — 技能與 CLI 動詞不是一一對應，會生成不可執行的指示。
**Deferred**:
- 既有名為 copilot 的自訂描述子相容性 — 內建名稱加入後可能與現有描述子衝突，交由 propose 查證並定義明確遷移／拒絕訊息；不得默默覆寫或刪除使用者描述子。
- 共享本文中的執行工具標記（含 propose 的 --agent） — 現行 {{TOOL}} 隨渲染目標寫死，交由 propose 定義共享檔如何記錄實際執行者；不得一律冒稱 codex 或 copilot。
- Copilot CLI／VS Code 的實機版本與驗收環境 — apply 前確認可用環境及驗收方式；本討論只查證官方文件與現有程式，不宣稱已實機驗證。
**Capture to**: proposal 與 design（建議一個變更 copilot-project-tool）；delta specs 更新 workspace-tools、desktop-config；tasks 與中英操作文件記載驗收及遷移。
**Next**: $speclink-propose --from-discussion copilot-project-tool
