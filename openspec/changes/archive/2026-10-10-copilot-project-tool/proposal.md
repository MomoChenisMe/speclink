## Why

使用 Copilot CLI／VS Code 的開發者、PO 與 PM，需要在 Speclink 專案設定中選取 Copilot，讓代理能使用 discuss、propose、apply 與品質關卡等既有 SDD 技能。討論 copilot-project-tool 已決定與 Codex 共用技能目錄及內容；只增加畫面選項會讓初始化、更新與停用清理仍無法使用。

## What Changes

- **BREAKING**：`locale` 與 `spec_locale` 未設定時，各自改為採用執行 Speclink 的作業系統語言；中文對應 `tw`、日文對應 `ja`，其他語言或無法取得語言時使用 `en`。明確設定及原有環境變數優先順序維持；不把偵測值寫回 config。Local 使用本機語言，Remote 使用執行指令的 server 語言，不使用 App 介面偏好。
- App 的 `locale`／`spec_locale` 選單僅提供「未設定（系統語系）」與 `tw`、`ja`、`en`，不再提供新的 `auto` 選項。舊 `spec_locale: auto` 仍可讀，畫面顯示「跟隨 locale（舊設定）」；使用者改選未設定並儲存才移除該鍵，既有 CLI 寫入合法代碼與引擎 auto 語意維持。

- 新增內建工具 copilot，沿用 .speclink.yaml 的 tools 清單，沒有新欄位或預設勾選；專案設定、新專案／既有工作區啟用、遠端 checkout 的本機工具綁定與 CLI init/update 一致接受。
- Codex／Copilot 共用 .agents/skills/speclink-*/SKILL.md；任一被選就生成一份，兩者都取消才清理 Speclink 管理的技能。同步、過期探測、孤兒清理與降級守門共用同一份期望內容。
- **BREAKING**：Codex 生成本文改為共享中性內容，技能以名稱引用，實際操作使用有效 CLI 動詞，不再綁定 $ 前綴或把技能當成不存在的子指令；propose／review／verify 以實際執行工具填寫 --agent，無法確認時省略。所有非 Claude 技能共用既有資產，不複製流程。
- **BREAKING**：自訂描述子名稱 copilot 成為保留名，寫入前拒絕並提供遷移指引；使用者可改名為 copilot-custom 保留原目錄／invocation，或手動改為內建字串 copilot，下一次成功同步才依既有足跡清理原產物。不自動轉換或覆寫描述子。
- Node SDK 的 skills.render 接受 copilot，codex 與 copilot 渲染回傳同一份內容，和 CLI 對等技能一致；claude／neutral 的現有介面保留。
- CLI init 的 --tools 接受三種工具的非空組合，互動選擇加入 Copilot；Copilot init 不新增子指令、stdin payload 或 init JSON 模式。成功 exit 0，無效選集／描述子衝突 exit 1 且零寫入；update 沿用既有 --allow-downgrade，無新增旗標。

## Capabilities

### New Capabilities

無。已比對 workspace-tools、desktop-config、node-sdk、workflow-config、baseline-skill；工具生成、設定與 SDK 渲染都屬既有能力，不另建 Copilot 能力。

### Modified Capabilities

- `workflow-config`: 未設定語言的系統預設與覆寫順序、init 範本和 show 人眼預設說明、技能語言預設說明；既有 canonical JSON 形狀不變。
- `baseline-skill`: canonical specLocale 為 null 時的語言指引改用系統語系，保留 rules.specs 套用與失敗停止規則。

- `workspace-tools`: 內建 Copilot 選擇、共享生成與清理、描述子遷移、共用本文與執行工具標記、過期／降級守門的共享檔案判定。
- `desktop-config`: 設定頁與初始化工具選擇加入 Copilot，保存真實工具選集並依共享目錄判斷保留；遠端設定頁維持 Workflow，工具仍屬 checkout。
- `node-sdk`: skills.render 新增 copilot target，與 codex／CLI 的共享輸出一致。

## Impact

- 受影響元件：speclink-core 負責工具解析、渲染與同步；speclink-cli 負責選擇與輸出；speclink-desktop-core／apps/desktop 負責快照與 UI；speclink-node 負責 SDK 型別與渲染。Copilot 工具整合本身無新相依、Server API 或認證流程；本輪語言預設新增 Host 的 sys-locale 相依，仍無新 Server API 路由或認證流程；品質補救於既有 GET /config 回應追加可選語言資料。
- 修改程式範圍：`crates/engine/speclink-core/src/workspace/skills.rs`、`crates/engine/speclink-core/src/workspace/config.rs`、`crates/engine/speclink-core/src/workspace/init.rs`、`crates/engine/speclink-core/assets/skills/`、`crates/adapters/speclink-cli/src/verbs/init.rs`、`apps/desktop/core/src/settings.rs`、`apps/desktop/core/src/project.rs`、`apps/desktop/src/views/ProjectSettingsView.tsx`、`apps/desktop/src/components/WorkspaceChooser.tsx`、`apps/desktop/src/App.tsx`、`apps/desktop/src/components/AssetUpdatePrompt.tsx`、`apps/desktop/src/adapter/workspace.ts`、`apps/desktop/src/i18n/messages.ts`、`crates/adapters/speclink-node/src/render.rs`、`crates/adapters/speclink-node/index.d.ts`。
- 測試與生成物：`crates/engine/speclink-core/src/workspace/config/tests.rs`、`crates/engine/speclink-core/src/workspace/init/tests.rs`、`crates/engine/speclink-core/tests/it/render_golden.rs`、`crates/engine/speclink-core/tests/golden/`、`crates/adapters/speclink-cli/tests/it/init_tools.rs`、`crates/adapters/speclink-cli/tests/it/tools_descriptor.rs`、`apps/desktop/src/__tests__/`、`crates/adapters/speclink-node/__test__/render.spec.ts`、`.agents/skills/`；新增 CLI 整合測試 `crates/adapters/speclink-cli/tests/it/copilot_tools.rs` 並在 `crates/adapters/speclink-cli/tests/it/main.rs` 註冊。另修正 `scripts/claude-code/skill-groups.test.mjs`：技能分頁表對照核心正典 registry，不依賴此 checkout 選取 Claude。無刪除程式檔。
- 中英文件：`docs/configuration.zh-TW.md`、`docs/configuration.md`、`docs/getting-started.zh-TW.md`、`docs/getting-started.md`、`docs/sdk-node.zh-TW.md`、`docs/sdk-node.md`。

### 相容性影響

init/update 人眼輸出的格式、stdout／stderr 分工與 --no-color 行為維持；工具名稱清單及共享路徑去重屬刻意變更，既有 JSON 欄位名與型別不變，工具資訊增加 copilot。Codex 本文與資產版本／golden 鎖同批刻意更新，Copilot 整合對 Claude 及既有 neutral 本文只更新資產版本戳；本輪系統語系另刻意更新各工具的語言說明（config、baseline、propose、ingest）及內建 specs 指引；不重新注入 AGENTS.md、CLAUDE.md 或 Copilot 指令檔，也不改工具使用者設定。原 Codex 使用者執行 update 即轉為共享內容；原 copilot 自訂描述子需先按上述指引手動選擇遷移。Copilot CLI、VS Code 與同時啟用 Claude 的實機驗收會檢查實際載入的技能內容，不能只看檔案存在。


### 本輪追加範圍與相容性（conversation context）

使用者於實機測試期間指定：兩個語言欄位未設定時採用執行 Speclink 電腦的作業系統語言，App 不再提供 auto 選項。此 ingest 回填已完成的實作與測試，不創建新變更。

追加受影響範圍為 `crates/host/speclink-host/{Cargo.toml,src/policy.rs,src/bridge.rs}`、`Cargo.lock`、`crates/engine/speclink-core/src/workspace/{config.rs,instructions.rs,init.rs,config/tests.rs}`、`crates/engine/speclink-core/assets/schema/spec-driven/fork.schema.yaml`、`crates/engine/speclink-core/assets/skills/{baseline,config,ingest,propose}.md`、`crates/adapters/speclink-cli/src/verbs/config.rs`、`crates/adapters/speclink-cli/tests/it/{config_fail_closed,instructions_policy}.rs`、`apps/desktop/src/views/ProjectSettingsView.tsx`、`apps/desktop/src/i18n/messages.ts` 與其既有測試，及 configuration 中英文件、既有 golden／assets.lock／共享技能。

語言預設原實作不新增 CLI 介面；品質補救追加的只讀 languages 查詢見下段。原有指令的旗標、stdin 與 exit code 維持；成功 exit 0，config 解析失敗維持非零 exit code 與 stderr 說明，stdout 無 payload。show 人眼對未設定語言的說明由 English 改為 system language；show --json 的 locale／specLocale 仍為字串或 null，不把系統語言冒充正典值。instructions 的 locale:string 與 specs 指引會反映新的有效語言，欄位形狀維持；--no-color 不新增 ANSI。

使用者已完成 Copilot CLI／VS Code 來源、共享本文、建立者及只留 Copilot 的保留驗收，並自行勾選 7.1／7.2。Claude 共存與只留 Claude 的清理實機驗收依使用者決定略過，仍有原自動回歸；不宣稱略過項目已通過。

## 品質補救（2026-10-10）

Review／Verify 發現 Baseline 的 canonical null 無法得知遠端 Server 語言。追加只讀 `workflow-config languages [--json]`：讀取正典語言設定，配合執行主機的 OS 預設，不套用 SPECLINK_*；JSON 為 locale 顯示名稱與 specLocale 明確代碼。Local 由 Host 解析，Remote 使用既有 GET /config 的新增可選 languages 資料。show JSON 的 null 與原有形狀維持；舊 Server 未提供資料時以非零 exit code、stderr 升級提示、stdout 空白停止，不猜 Client 語言。沒有新 API 路由或寫入。Baseline 在盤點前查這個來源，再以 specLocale 決定散文語言。

同步補 null／auto 的 server 日文、client 繁中、覆寫不可跨越及零寫入測試；政策範例補 spec_locale:ja 並保留 locale／rules，同時保留 legacy auto 的 writer 回歸。語言資料、技能與資產版本同步更新，不改原完成任務或 Claude 手測略過記錄。
