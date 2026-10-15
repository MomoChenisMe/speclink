# Fork 客製功能完整盤點：升級官方 0.9.0

盤點日期：2026-10-15（Asia/Taipei）。此文件先於合併建立；整合結果及實際測試以 `UPGRADE_REPORT.md` 為準。

## Git 證據與保護

- Fork 升級前：`f9d307138ab8638198d0c34a354360df34fad464`。
- 官方正式 tag：`v0.9.0`，commit `b2c96f548db08d9d16ea34acbc3dd848073224d8`。
- 共同祖先：`eb44f950e96e8023239bdfab145fa487062c99cf`。
- 已核對 GitHub Release：非 draft、非 prerelease，發布時間 `2026-10-15T13:45:11Z`。
- origin：`https://github.com/mosutw/speclink.git`；upstream：`https://github.com/MomoChenisMe/speclink.git`。
- 開始時位於 main、工作區乾淨，沒有未提交或未追蹤的非忽略檔。
- 備份分支：`backup/pre-upstream-0.9-20261015-f9d30713`。
- 升級分支：`upgrade/upstream-0.9-20261015`；不修改 main、不 push、不發 Release。
- 共同祖先至 Fork 共 164 個變動路徑、14 筆專屬提交（含合併及 CI 重觸發）。不是 164 個獨立功能。
- 比對使用實際 diff、程式、測試、設定及 workflow；附錄逐檔列出原始提交，避免只憑 commit message。

## 功能盤點

| ID | 客製項目及用途 | 程式／設定／測試證據 | 原始提交 | 官方 0.9 比對與建議 |
|---|---|---|---|---|
| C01 | Copilot 成為內建工具；CLI init/update/adopt、停用清理、資產探測、Desktop 初始選擇／設定／checkout | `workspace/skills.rs` 的 Tool::Copilot；`workspace/init.rs` 的共享 SyncTarget；`copilot_tools.rs`；Desktop core project/settings tests；WorkspaceChooser | `904c9a9e`、`742ea8bd` | 官方已完整提供相同工具入口，另有 `.github/skills`、Copilot `/speclink-` 語法、目錄與 symlink 清理測試。採官方工具實作，遷移驗證舊 Fork 設定；不疊兩份 Tool/SyncPlan。 |
| C02 | Codex 與 Copilot 共用 `.agents/skills` 與中性本文；實際代理決定 --agent | `skills.rs` 的 SHARED_EXECUTION、dynamic_agent、slash_replacement；兩組生成檔及 golden | `904c9a9e`、`184e9923` | 官方分別使用 `.agents/skills`／`.github/skills`，保留各工具正確語法及身份。以官方分目標生成取代共享目錄的實作細節；確認所有工作流仍可產生，保留使用者自有技能。 |
| C03 | Node SDK 支援 `skills.render(..., {target: 'copilot'})` | `crates/adapters/speclink-node/src/render.rs`、`index.d.ts`、`__test__/render.spec.ts`、sdk-node 中英文文件 | `904c9a9e` | 官方已提供 Copilot target，採官方；刪重前核對 renderer／型別／實際測試，不再斷言 Copilot 與 Codex byte-identical。 |
| C04 | 未設 locale/spec_locale 時採執行主機 OS 語言；環境／明示值優先；不改 UI 語言、不回寫設定 | core config EnvOverrides.system_locale；Host policy/bridge；sys-locale dependency；CLI config、各層 tests | `904c9a9e` | 官方仍以 English 為未設定預設，這是獨有行為。保留 Host 注入、server 端解析、中文 tw／日文 ja／其他 en；與新版 review_rules 等政策欄位合併。 |
| C05 | `workflow-config languages --json`：本地／遠端只讀取得真正的產出語言，舊 server 不猜語言 | CLI config.rs；protocol ConfigResponse.languages/WorkflowLanguages；Server config route；Host workflow_languages；query_routes/phase2 tests | `904c9a9e` | 官方無此查詢／metadata。完整保留，檢查新增官方 ConfigResponse 欄位、授權及錯誤處理；不得採 client OS 代替 server OS。 |
| C06 | baseline/propose/ingest 技能與 schema 說明遵守系統語系預設；保留結構英文及 canonical/env 分層 | `assets/skills/{baseline,config,ingest,propose}.md`、fork.schema.yaml、instructions.rs、docs/configuration、generated skills/golden | `904c9a9e`、`184e9923` | 官方技能已到 v1.48.0 且有新品質站／review rules。以官方新本文為底，只保留語言差異，再從整合引擎生成；不整批沿用 v1.44.0 舊檔。 |
| C07 | 裝置級 system/light/dark，首次 render 前套色，跨專案、重開記憶；非法／讀取失敗跟系統 | Desktop theme.ts/main.tsx/App.tsx；theme.test.ts；App/settings tests | `538d024b` | 官方 packages/ui/src/theme.ts、index.html boot、GeneralPage 已有完整三選，key 是 speclink.theme，Fork 是 speclink.uiTheme。採官方單一偏好架構並遷移舊 key，保留既有選擇，不建第二套 UI/控制器。 |
| C08 | 外觀保存失敗保留原選項與配色、顯示可重試錯誤 | theme.setPreference 先寫再通知；AppSettingsView role=alert；讀寫失敗與鍵盤 tests | `538d024b` | 官方 writeThemePreference 靜默吞寫入錯誤。使用者已明確選擇保留 Fork 行為：在官方 GeneralPage 整合失敗提示，不新增重複設定項。 |
| C09 | macOS 面板主題即時同步、無 workspace 更新仍生效、首份 snapshot 前不呈現；await listen 再 ready、延遲訂閱清理、內容出現後量測高度 | tray.ts、panel/main.tsx、tray/trayPanel tests | `538d024b` | 官方改用 theme-changed 事件、原生 setTheme，另新增 Windows panel。保留官方跨平台與原生主題，適配 Fork 的握手、首次呈現與清理修正，避免事件競態及孤立雙通道。 |
| C10 | toast 接有效配色、既有通知切換一致；CSS 覆寫優先 | shared sonner.tsx/theme.css、對應 tests、Desktop App | `538d024b` | 官方新色票及 Markdown/UI 必須保留；官方 Sonner 仍固定 system。保留可傳 theme 與目前偏好同步，不沿用舊色票覆蓋官方新設計。青綠色票本來來自共同祖先，不是 Fork 額外開發的色票。 |
| C11 | 專案設定頂部唯讀完整目錄：local root／remote checkoutRoot／無副本提示；切換、離線、revision 衝突不錯置 | App.tsx locator；ProjectSettingsView；中英文 messages；projectSettingsView/App tests | `538d024b` | 官方新分類式設定沒有此目錄區塊。適配到新 ProjectSettingsView，各分類共用；保留不查 Server 目錄、不增加寫入、恢復頁分支。 |
| C12 | 目錄選取複製保留連續空白、Tab、換行及長路徑 | ProjectSettingsView whiteSpace pre-wrap、overflow-wrap anywhere；三個原文／計算樣式回歸案例 | `a886c9dd` | 完整保留；舊 jsdom textContent 不能證明實際樣式。新測試要仍檢查保留空白的計算樣式；實機複製未重測要如實標示。 |
| C13 | Windows Portable：EXE 旁 marker opt-in，data/{desktop,speclink,webview} 隔離，搬移後跟隨；唯讀時報錯不回退 | Host portable.rs/context.rs；Tauri portable.rs/lib.rs；portable unit tests；docs/windows-portable | `1fe2783e`、`70ab5454` | 官方無 portable marker/data 實作。完整保留並適配新版 Tauri 啟動、平台 config 與 Windows panel 的新 WebView；安裝版不得受影響。 |
| C14 | Portable 登入只在記憶體；不讀／寫機器 Keychain/credentials；CLI 用 SPECLINK_TOKEN；停用安裝器更新 | remote credentials.rs/auth.rs；CLI connection.rs；Tauri ConnectionsState/client_store；main.tsx updater/cliInstall 注入 | `1fe2783e` | 官方無 portable 模式。保留，特別核對新的 about/app-menu/updates 頁不能繞過 portable 更新限制。 |
| C15 | 自用 Desktop Build：macOS universal ad-hoc、Windows NSIS、Linux x64/arm64，版本資訊及同源 CLI sidecar | `.github/workflows/desktop-build.yml`；desktop-sidecar.mjs；正式 release.yml 沒有 Fork diff | `6f1946b0`、`1fe2783e` | 完整保留手動 workflow，適配官方 0.9 新 macOS/Windows Tauri config、資源與依賴；官方正式 release 的簽章／npm／updater 流程照收。 |
| C16 | Windows Portable ZIP staging／壓縮、解壓執行、設定寫入與搬移驗證；static CRT；產物可追溯 | windows-portable.mjs/test；desktop-build.yml PowerShell smoke；LICENSE/README/resources/build-info | `1fe2783e` | 官方 Release 六項 assets 沒有 portable ZIP。使用者已選擇維持 Actions artifact，不擴充正式 Release；不能只靠 YAML 無衝突判定成功，Mac 本地只能測 staging，Windows 仍需 runner。 |
| C17 | Claude 技能清單守門改由 core registry 取得，不依賴 checkout 是否選用 Claude | scripts/claude-code/skill-groups.test.mjs | `904c9a9e` | 保留此獨有穩健性，核對官方 registry/skill 分組是否有新增入口。 |
| C18 | 專案工具選集及歷史／文件：三工具設定、客製正式規格、封存證據、手冊；重觸發 CI 無 runtime 變更 | .speclink.yaml、兩組 archive、discussion、manual、各 canonical specs、docs、Cargo.lock | 多筆，見附錄；`c688c569` 為 CI 觸發 | 不覆寫使用者三工具設定，不丟封存歷史。正式規格合併官方新條款與客製行為；歷史舊路徑維持史實。手冊可能過時，不能把共同祖先已有的 Claude plugin 誤報為 Fork 新 runtime。 |

## 已確認的整合界線

- 使用者於本次對話選擇：外觀保存失敗保留 Fork 行為；Portable 維持 Actions artifact，不擴充 Release。
- 官方 0.9 新 UI、Light/Dark/System、原生視窗主題、Windows panel、Copilot native commands、review rules、安全修正與版本號均須保留。
- 官方 0.9 與 Fork 的 Copilot「目錄／本文相同」不相同；採官方原生路徑及各代理語法，保留的是功能入口及使用者設定，不保留重複 renderer。
- OS 語系、Server 語言查詢、Portable 隔離及 readonly 目錄展示是獨有能力，不能被官方檔案整批取代。
- 升級前 Server Web 外觀實機驗收 5.4 已經由使用者豁免；這不是升級後驗收通過證據。
- Windows installer／Portable、新 WebView 資料目錄與發行物解壓執行尚需升級後重新驗證，不能沿用 0.8 的通過紀錄。

## 逐檔覆蓋附錄

以下附錄由共同祖先與升級前 SHA 的實際 `git diff --name-status` 及逐檔 `git log` 產生。每個路徑均列入；生成檔、golden、文件及封存檔也不省略。

| 狀態 | 路徑 | Fork 提交（非 merge） | 官方同路徑也變更 |
|---|---|---|---|
| M | `.agents/skills/speclink-apply-with-worktree/SKILL.md` | `904c9a9e` | 是 |
| M | `.agents/skills/speclink-apply/SKILL.md` | `904c9a9e` | 是 |
| M | `.agents/skills/speclink-archive/SKILL.md` | `904c9a9e` | 是 |
| M | `.agents/skills/speclink-audit/SKILL.md` | `904c9a9e` | 是 |
| M | `.agents/skills/speclink-baseline/SKILL.md` | `904c9a9e` | 是 |
| M | `.agents/skills/speclink-commit/SKILL.md` | `904c9a9e` | 是 |
| M | `.agents/skills/speclink-config/SKILL.md` | `904c9a9e` | 是 |
| M | `.agents/skills/speclink-discuss/SKILL.md` | `904c9a9e` | 是 |
| M | `.agents/skills/speclink-drift/SKILL.md` | `904c9a9e` | 是 |
| M | `.agents/skills/speclink-improve/SKILL.md` | `904c9a9e` | 是 |
| M | `.agents/skills/speclink-ingest/SKILL.md` | `904c9a9e` | 是 |
| M | `.agents/skills/speclink-manual/SKILL.md` | `904c9a9e` | 是 |
| M | `.agents/skills/speclink-propose/SKILL.md` | `904c9a9e` | 是 |
| M | `.agents/skills/speclink-quality/SKILL.md` | `904c9a9e` | 是 |
| M | `.agents/skills/speclink-review/SKILL.md` | `904c9a9e` | 是 |
| M | `.agents/skills/speclink-trace/SKILL.md` | `904c9a9e` | 是 |
| M | `.agents/skills/speclink-verify/SKILL.md` | `904c9a9e` | 是 |
| M | `.agents/skills/speclink-worktree-merge/SKILL.md` | `904c9a9e` | 是 |
| M | `.claude/skills/speclink-analyze/SKILL.md` | `184e9923` | 是 |
| M | `.claude/skills/speclink-apply-with-worktree/SKILL.md` | `184e9923` | 是 |
| M | `.claude/skills/speclink-apply/SKILL.md` | `184e9923` | 是 |
| M | `.claude/skills/speclink-archive/SKILL.md` | `184e9923` | 是 |
| M | `.claude/skills/speclink-audit/SKILL.md` | `184e9923` | 是 |
| M | `.claude/skills/speclink-baseline/SKILL.md` | `184e9923` | 是 |
| M | `.claude/skills/speclink-commit/SKILL.md` | `184e9923` | 是 |
| M | `.claude/skills/speclink-config/SKILL.md` | `184e9923` | 是 |
| M | `.claude/skills/speclink-discuss/SKILL.md` | `184e9923` | 是 |
| M | `.claude/skills/speclink-drift/SKILL.md` | `184e9923` | 是 |
| M | `.claude/skills/speclink-improve/SKILL.md` | `184e9923` | 是 |
| M | `.claude/skills/speclink-ingest/SKILL.md` | `184e9923` | 是 |
| M | `.claude/skills/speclink-manual/SKILL.md` | `184e9923` | 是 |
| M | `.claude/skills/speclink-propose/SKILL.md` | `184e9923` | 是 |
| M | `.claude/skills/speclink-quality/SKILL.md` | `184e9923` | 是 |
| M | `.claude/skills/speclink-review/SKILL.md` | `184e9923` | 是 |
| M | `.claude/skills/speclink-trace/SKILL.md` | `184e9923` | 是 |
| M | `.claude/skills/speclink-verify/SKILL.md` | `184e9923` | 是 |
| M | `.claude/skills/speclink-worktree-merge/SKILL.md` | `184e9923` | 是 |
| A | `.github/workflows/desktop-build.yml` | `1fe2783e`, `6f1946b0` | 否 |
| M | `.speclink.yaml` | `184e9923`, `742ea8bd` | 是 |
| M | `Cargo.lock` | `904c9a9e` | 是 |
| M | `apps/desktop/core/src/project.rs` | `904c9a9e` | 是 |
| M | `apps/desktop/core/src/settings.rs` | `904c9a9e` | 是 |
| M | `apps/desktop/src-tauri/src/lib.rs` | `1fe2783e` | 是 |
| A | `apps/desktop/src-tauri/src/portable.rs` | `1fe2783e` | 否 |
| M | `apps/desktop/src/App.tsx` | `538d024b`, `904c9a9e` | 是 |
| M | `apps/desktop/src/__tests__/App.test.tsx` | `538d024b`, `904c9a9e` | 是 |
| M | `apps/desktop/src/__tests__/appSettingsView.test.tsx` | `538d024b` | 是 |
| M | `apps/desktop/src/__tests__/assetUpdatePrompt.test.tsx` | `904c9a9e` | 是 |
| M | `apps/desktop/src/__tests__/messages.test.ts` | `904c9a9e` | 是 |
| M | `apps/desktop/src/__tests__/projectSettingsView.test.tsx` | `a886c9dd`, `538d024b`, `904c9a9e` | 是 |
| A | `apps/desktop/src/__tests__/theme.test.ts` | `538d024b` | 否 |
| M | `apps/desktop/src/__tests__/tray.test.ts` | `538d024b` | 是 |
| M | `apps/desktop/src/__tests__/trayPanel.test.tsx` | `538d024b` | 是 |
| M | `apps/desktop/src/__tests__/workspace.test.ts` | `904c9a9e` | 是 |
| M | `apps/desktop/src/__tests__/workspaceChooser.test.tsx` | `904c9a9e` | 是 |
| M | `apps/desktop/src/adapter/connections.ts` | `904c9a9e` | 是 |
| M | `apps/desktop/src/components/WorkspaceChooser.tsx` | `904c9a9e` | 是 |
| M | `apps/desktop/src/i18n/messages.ts` | `538d024b`, `904c9a9e` | 是 |
| M | `apps/desktop/src/index.css` | `538d024b` | 是 |
| M | `apps/desktop/src/main.tsx` | `538d024b`, `1fe2783e` | 是 |
| M | `apps/desktop/src/panel/main.tsx` | `538d024b` | 是 |
| A | `apps/desktop/src/theme.ts` | `538d024b` | 否 |
| M | `apps/desktop/src/tray.ts` | `538d024b` | 是 |
| M | `apps/desktop/src/views/AppSettingsView.tsx` | `538d024b` | 是 |
| M | `apps/desktop/src/views/ProjectSettingsView.tsx` | `a886c9dd`, `538d024b`, `904c9a9e` | 是 |
| M | `crates/adapters/speclink-cli/src/verbs/config.rs` | `904c9a9e` | 是 |
| M | `crates/adapters/speclink-cli/src/verbs/connection.rs` | `1fe2783e` | 否 |
| M | `crates/adapters/speclink-cli/src/verbs/init.rs` | `904c9a9e` | 是 |
| M | `crates/adapters/speclink-cli/tests/it/config_fail_closed.rs` | `904c9a9e` | 否 |
| A | `crates/adapters/speclink-cli/tests/it/copilot_tools.rs` | `904c9a9e` | 否 |
| M | `crates/adapters/speclink-cli/tests/it/init_tools.rs` | `904c9a9e` | 是 |
| M | `crates/adapters/speclink-cli/tests/it/instructions_policy.rs` | `904c9a9e` | 否 |
| M | `crates/adapters/speclink-cli/tests/it/main.rs` | `904c9a9e` | 否 |
| M | `crates/adapters/speclink-cli/tests/it/tools_descriptor.rs` | `904c9a9e` | 否 |
| M | `crates/adapters/speclink-cli/tests/it/workflow_config.rs` | `904c9a9e` | 是 |
| M | `crates/adapters/speclink-node/__test__/render.spec.ts` | `904c9a9e` | 是 |
| M | `crates/adapters/speclink-node/index.d.ts` | `904c9a9e` | 是 |
| M | `crates/adapters/speclink-node/src/render.rs` | `904c9a9e` | 是 |
| M | `crates/engine/speclink-core/assets/schema/spec-driven/fork.schema.yaml` | `904c9a9e` | 否 |
| M | `crates/engine/speclink-core/assets/skills/baseline.md` | `904c9a9e` | 是 |
| M | `crates/engine/speclink-core/assets/skills/config.md` | `904c9a9e` | 是 |
| M | `crates/engine/speclink-core/assets/skills/ingest.md` | `904c9a9e` | 是 |
| M | `crates/engine/speclink-core/assets/skills/propose.md` | `904c9a9e` | 是 |
| M | `crates/engine/speclink-core/src/workspace/config.rs` | `904c9a9e` | 是 |
| M | `crates/engine/speclink-core/src/workspace/config/tests.rs` | `904c9a9e` | 是 |
| M | `crates/engine/speclink-core/src/workspace/init.rs` | `904c9a9e` | 是 |
| M | `crates/engine/speclink-core/src/workspace/init/tests.rs` | `904c9a9e` | 是 |
| M | `crates/engine/speclink-core/src/workspace/instructions.rs` | `904c9a9e` | 是 |
| M | `crates/engine/speclink-core/src/workspace/skills.rs` | `904c9a9e` | 是 |
| M | `crates/engine/speclink-core/tests/golden/assets.lock` | `904c9a9e` | 是 |
| M | `crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md` | `904c9a9e` | 是 |
| M | `crates/engine/speclink-core/tests/golden/claude.snapshot.md` | `904c9a9e` | 是 |
| M | `crates/engine/speclink-core/tests/golden/codex.snapshot.md` | `904c9a9e` | 是 |
| M | `crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md` | `904c9a9e` | 是 |
| M | `crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md` | `904c9a9e` | 是 |
| M | `crates/engine/speclink-core/tests/it/render_golden.rs` | `904c9a9e` | 是 |
| M | `crates/host/speclink-host/Cargo.toml` | `904c9a9e` | 否 |
| M | `crates/host/speclink-host/src/bridge.rs` | `904c9a9e` | 否 |
| M | `crates/host/speclink-host/src/context.rs` | `1fe2783e` | 是 |
| M | `crates/host/speclink-host/src/lib.rs` | `1fe2783e` | 是 |
| M | `crates/host/speclink-host/src/policy.rs` | `904c9a9e` | 否 |
| A | `crates/host/speclink-host/src/portable.rs` | `1fe2783e` | 否 |
| M | `crates/host/speclink-server/src/api/routes.rs` | `904c9a9e` | 是 |
| M | `crates/host/speclink-server/tests/it/api/query_routes.rs` | `904c9a9e` | 否 |
| M | `crates/host/speclink-server/tests/it/phase2_chain.rs` | `904c9a9e` | 是 |
| M | `crates/protocol/speclink-protocol/src/query.rs` | `904c9a9e` | 否 |
| M | `crates/protocol/speclink-remote/src/auth.rs` | `1fe2783e` | 否 |
| M | `crates/protocol/speclink-remote/src/credentials.rs` | `1fe2783e` | 否 |
| M | `docs/configuration.md` | `904c9a9e` | 是 |
| M | `docs/configuration.zh-TW.md` | `904c9a9e` | 是 |
| A | `docs/development/windows-portable.md` | `1fe2783e` | 否 |
| M | `docs/getting-started.md` | `904c9a9e` | 是 |
| M | `docs/getting-started.zh-TW.md` | `904c9a9e` | 是 |
| M | `docs/sdk-node.md` | `904c9a9e` | 是 |
| M | `docs/sdk-node.zh-TW.md` | `904c9a9e` | 是 |
| A | `openspec/changes/archive/2026-10-10-copilot-project-tool/.evidence.json` | `904c9a9e` | 否 |
| A | `openspec/changes/archive/2026-10-10-copilot-project-tool/.openspec.yaml` | `904c9a9e` | 否 |
| A | `openspec/changes/archive/2026-10-10-copilot-project-tool/design.md` | `904c9a9e` | 否 |
| A | `openspec/changes/archive/2026-10-10-copilot-project-tool/proposal.md` | `904c9a9e` | 否 |
| A | `openspec/changes/archive/2026-10-10-copilot-project-tool/specs/baseline-skill/spec.md` | `904c9a9e` | 否 |
| A | `openspec/changes/archive/2026-10-10-copilot-project-tool/specs/desktop-config/spec.md` | `904c9a9e` | 否 |
| A | `openspec/changes/archive/2026-10-10-copilot-project-tool/specs/node-sdk/spec.md` | `904c9a9e` | 否 |
| A | `openspec/changes/archive/2026-10-10-copilot-project-tool/specs/workflow-config/spec.md` | `904c9a9e` | 否 |
| A | `openspec/changes/archive/2026-10-10-copilot-project-tool/specs/workspace-tools/spec.md` | `904c9a9e` | 否 |
| A | `openspec/changes/archive/2026-10-10-copilot-project-tool/tasks.md` | `904c9a9e` | 否 |
| A | `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/.evidence.json` | `242a5836` | 否 |
| A | `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/.openspec.yaml` | `242a5836` | 否 |
| A | `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/design.md` | `242a5836` | 否 |
| A | `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/proposal.md` | `242a5836` | 否 |
| A | `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/specs/desktop-app/spec.md` | `242a5836` | 否 |
| A | `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/specs/desktop-config/spec.md` | `242a5836` | 否 |
| A | `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/specs/tray-status-menu/spec.md` | `242a5836` | 否 |
| A | `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/tasks.md` | `242a5836` | 否 |
| A | `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/validation.md` | `242a5836` | 否 |
| A | `openspec/discussions/archive/2026-10-07-copilot-project-tool.md` | `904c9a9e` | 否 |
| A | `openspec/discussions/archive/2026-10-12-desktop-theme-and-project-directory.md` | `242a5836` | 否 |
| M | `openspec/manual/about.md` | `904c9a9e` | 是 |
| M | `openspec/manual/baseline.md` | `904c9a9e` | 是 |
| A | `openspec/manual/claude-session-focus.md` | `904c9a9e` | 否 |
| A | `openspec/manual/claude-skill-bar.md` | `904c9a9e` | 否 |
| A | `openspec/manual/claude-workflow-panel.md` | `904c9a9e` | 否 |
| M | `openspec/manual/desktop-projects.md` | `904c9a9e` | 是 |
| M | `openspec/manual/desktop-update.md` | `904c9a9e` | 是 |
| M | `openspec/manual/index.md` | `904c9a9e` | 是 |
| M | `openspec/manual/init-workspace.md` | `904c9a9e` | 是 |
| M | `openspec/manual/policy-config.md` | `904c9a9e` | 是 |
| M | `openspec/manual/worktree.md` | `904c9a9e` | 是 |
| M | `openspec/specs/baseline-skill/spec.md` | `904c9a9e` | 否 |
| A | `openspec/specs/claude-code-session-focus/spec.md` | `904c9a9e` | 否 |
| A | `openspec/specs/claude-code-skill-bar/spec.md` | `904c9a9e` | 否 |
| A | `openspec/specs/claude-code-workflow-panel/spec.md` | `904c9a9e` | 否 |
| M | `openspec/specs/desktop-app/spec.md` | `242a5836` | 是 |
| M | `openspec/specs/desktop-config/spec.md` | `242a5836`, `904c9a9e` | 是 |
| M | `openspec/specs/node-sdk/spec.md` | `904c9a9e` | 是 |
| M | `openspec/specs/tray-status-menu/spec.md` | `242a5836` | 是 |
| M | `openspec/specs/workflow-config/spec.md` | `904c9a9e` | 是 |
| M | `openspec/specs/workspace-tools/spec.md` | `904c9a9e` | 是 |
| M | `packages/ui/src/__tests__/sonner.test.tsx` | `538d024b` | 是 |
| M | `packages/ui/src/__tests__/theme.test.ts` | `538d024b` | 是 |
| M | `packages/ui/src/components/ui/sonner.tsx` | `538d024b` | 是 |
| M | `packages/ui/src/theme.css` | `538d024b` | 是 |
| M | `scripts/claude-code/skill-groups.test.mjs` | `904c9a9e` | 否 |
| A | `scripts/desktop/windows-portable.mjs` | `1fe2783e` | 否 |
| A | `scripts/desktop/windows-portable.test.mjs` | `1fe2783e` | 否 |


## 整合追蹤（2026-10-16）

上述 18 類均已有明確決策與對應實作，詳見 UPGRADE_FEATURE_MATRIX.md 與 UPGRADE_REPORT.md。89 個衝突逐檔解法、實際測試結果、官方既有型別問題的修復，以及仍待 Windows／實機驗收的邊界均記於報告。沒有將文字無衝突等同功能已驗證，也沒有刪除兩組客製封存歷史。
