# Upstream 0.10.0 雙方逐檔盤點

以共同祖先 b2c96f548db08d9d16ea34acbc3dd848073224d8 為基礎，分別比較 Fork 1b2ca6ee279c49ed99e64208eba5f86ac14ac78e 與正式 Tag 05e5e65ac043d04e6e0b2a396c0bb0997e52ae70。不是直接把兩端最終檔案相減。

Fork：306 檔（新增 115、修改 191）；Upstream：82 檔（新增 14、修改 68）。交集 56、Fork 獨有 250、Upstream 獨有 26、聯集 332。兩端相對共同祖先均無淨刪除或淨更名；以下另列歷史移動。

Fork 獨有的 250 檔中，232 檔與整合前逐位元一致；18 檔 Copilot 技能依新版 renderer 重產。CI、三工具設定、工作流政策、npm metadata 與 lockfile 全部與整合前相同。

A=新增，M=修改，—=該端未變。衝突欄記錄初次 merge 的 53 個衝突，已全部處理；「保留」不代表所有平台已實機驗收，測試範圍見主報告。

| 路徑 | Fork | Upstream | 曾衝突 | 整合結果 |
|---|---|---|---|---|
| `.agents/skills/speclink-apply-with-worktree/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.agents/skills/speclink-apply/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.agents/skills/speclink-archive/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.agents/skills/speclink-audit/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.agents/skills/speclink-baseline/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.agents/skills/speclink-commit/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.agents/skills/speclink-config/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.agents/skills/speclink-discuss/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.agents/skills/speclink-drift/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.agents/skills/speclink-improve/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.agents/skills/speclink-ingest/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.agents/skills/speclink-manual/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.agents/skills/speclink-propose/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.agents/skills/speclink-quality/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.agents/skills/speclink-review/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.agents/skills/speclink-trace/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.agents/skills/speclink-verify/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.agents/skills/speclink-worktree-merge/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.claude/skills/speclink-analyze/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.claude/skills/speclink-apply-with-worktree/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.claude/skills/speclink-apply/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.claude/skills/speclink-archive/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.claude/skills/speclink-audit/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.claude/skills/speclink-baseline/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.claude/skills/speclink-commit/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.claude/skills/speclink-config/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.claude/skills/speclink-discuss/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.claude/skills/speclink-drift/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.claude/skills/speclink-improve/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.claude/skills/speclink-ingest/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.claude/skills/speclink-manual/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.claude/skills/speclink-propose/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.claude/skills/speclink-quality/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.claude/skills/speclink-review/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.claude/skills/speclink-trace/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.claude/skills/speclink-verify/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.claude/skills/speclink-worktree-merge/SKILL.md` | M | M | 是 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.github/skills/speclink-apply-with-worktree/SKILL.md` | A | — | 否 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.github/skills/speclink-apply/SKILL.md` | A | — | 否 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.github/skills/speclink-archive/SKILL.md` | A | — | 否 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.github/skills/speclink-audit/SKILL.md` | A | — | 否 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.github/skills/speclink-baseline/SKILL.md` | A | — | 否 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.github/skills/speclink-commit/SKILL.md` | A | — | 否 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.github/skills/speclink-config/SKILL.md` | A | — | 否 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.github/skills/speclink-discuss/SKILL.md` | A | — | 否 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.github/skills/speclink-drift/SKILL.md` | A | — | 否 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.github/skills/speclink-improve/SKILL.md` | A | — | 否 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.github/skills/speclink-ingest/SKILL.md` | A | — | 否 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.github/skills/speclink-manual/SKILL.md` | A | — | 否 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.github/skills/speclink-propose/SKILL.md` | A | — | 否 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.github/skills/speclink-quality/SKILL.md` | A | — | 否 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.github/skills/speclink-review/SKILL.md` | A | — | 否 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.github/skills/speclink-trace/SKILL.md` | A | — | 否 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.github/skills/speclink-verify/SKILL.md` | A | — | 否 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.github/skills/speclink-worktree-merge/SKILL.md` | A | — | 否 | 由整合引擎重產；保留工具原生語法、Fork 安全指引與新版對話段落 |
| `.github/workflows/ci.yml` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `.github/workflows/desktop-build.yml` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `.speclink.yaml` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `CHANGELOG.md` | — | M | 否 | 由整合後 release-notes JSON 重產英文日誌 |
| `CHANGELOG.zh-TW.md` | — | M | 否 | 由整合後 release-notes JSON 重產繁中日誌 |
| `Cargo.lock` | M | M | 是 | 14 個 workspace package 同步 0.10.0-build.m1；保留 Fork 依賴與 checksums |
| `Cargo.toml` | M | M | 是 | Upstream 基版 0.10.0，加回 Fork 的 build.m1 預先發行尾碼 |
| `FORK_CUSTOMIZATION_INVENTORY.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `UPGRADE_FEATURE_MATRIX.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `UPGRADE_REPORT.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/core/Cargo.toml` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/core/src/lib.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/core/src/project.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/core/src/security.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/core/src/settings.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/index.html` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src-tauri/capabilities/default.json` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src-tauri/src/lib.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src-tauri/src/panel.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src-tauri/src/portable.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src-tauri/src/remote.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src-tauri/src/security_commands.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src-tauri/tauri.conf.json` | M | M | 是 | 同步 0.10.0-build.m1；識別碼、updater 與平台配置沿用 |
| `apps/desktop/src/App.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/App.test.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/appSettingsView.test.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/messages.test.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/projectColors.test.ts` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/projectRail.test.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/projectSettingsView.test.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/remoteDataSource.test.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/remoteOpen.test.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/remoteWorkspaceRecovery.test.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/securityDataPage.test.tsx` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/securityReview.test.tsx` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/session.test.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/settingsColumn.test.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/store.test.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/tabs.test.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/themeBoot.test.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/tray.test.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/trayPanel.test.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/workspace.test.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/__tests__/workspaceChooser.test.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/adapter/remoteDataSource.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/adapter/security.ts` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/adapter/tauriDataSource.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/adapter/workspace.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/components/ProjectRail.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/components/projectTile.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/i18n/messages.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/main.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/panel/TrayPanel.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/panel/main.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/projectColors.ts` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/release-notes/release-notes.json` | — | M | 否 | 保留 Upstream 0.10.0 新內容，預設語言說明適配 Fork |
| `apps/desktop/src/session.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/settingsCatalog.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/store.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/tray.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/views/AppSettingsView.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/views/ProjectSettingsView.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/views/project-settings/WorkflowPage.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/views/project-settings/useProjectSettings.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/views/settings/AboutPage.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/views/settings/GeneralPage.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/desktop/src/views/settings/SecurityDataPage.tsx` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/server-web/src/__tests__/build.test.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `apps/server-web/tsconfig.json` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/adapters/speclink-cli/Cargo.toml` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/adapters/speclink-cli/src/main.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/adapters/speclink-cli/src/verbs/config.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/adapters/speclink-cli/src/verbs/connection.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/adapters/speclink-cli/src/verbs/instructions.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/adapters/speclink-cli/src/verbs/mod.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/adapters/speclink-cli/src/verbs/security.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/adapters/speclink-cli/tests/it/archive_readiness_gate.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/adapters/speclink-cli/tests/it/config_fail_closed.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/adapters/speclink-cli/tests/it/copilot_tools.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/adapters/speclink-cli/tests/it/init_tools.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/adapters/speclink-cli/tests/it/instructions_policy.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/adapters/speclink-cli/tests/it/main.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/adapters/speclink-cli/tests/it/remote_read_path.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/adapters/speclink-cli/tests/it/security.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/adapters/speclink-cli/tests/it/tools_descriptor.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/adapters/speclink-cli/tests/it/workflow_config.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/adapters/speclink-node/__test__/render.spec.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/Cargo.toml` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/assets/schema/spec-driven/fork.schema.yaml` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/assets/security/rule-locales.json` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/assets/security/rules.json` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/assets/security/sources.json` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/assets/skills/apply.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/assets/skills/audit.md` | M | M | 否 | 新版文件語言規則改讀 Fork languages 查詢；保留系統／Server 預設及安全指引 |
| `crates/engine/speclink-core/assets/skills/baseline.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/assets/skills/config.md` | M | M | 否 | 新版文件語言規則改讀 Fork languages 查詢；保留系統／Server 預設及安全指引 |
| `crates/engine/speclink-core/assets/skills/conversation-language.md` | — | A | 否 | 保留 Upstream 共用對話語言段落原稿 |
| `crates/engine/speclink-core/assets/skills/discuss.md` | — | M | 否 | 新版文件語言規則改讀 Fork languages 查詢；保留系統／Server 預設及安全指引 |
| `crates/engine/speclink-core/assets/skills/improve.md` | — | M | 否 | 新版文件語言規則改讀 Fork languages 查詢；保留系統／Server 預設及安全指引 |
| `crates/engine/speclink-core/assets/skills/ingest.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/assets/skills/propose.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/assets/skills/verify.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/src/lib.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/src/lifecycle/archive/tests.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/src/security/boundary.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/src/security/data.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/src/security/input.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/src/security/mod.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/src/security/policy.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/src/security/prepared.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/src/security/report.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/src/security/rules.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/src/security/standards.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/src/teststore.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/src/workspace/config.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/src/workspace/config/tests.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/src/workspace/init.rs` | M | M | 是 | 資產版升至 v1.50.1，保留 Fork 的 OS 語言 template |
| `crates/engine/speclink-core/src/workspace/instructions.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-core/src/workspace/skills.rs` | — | M | 否 | 採 Upstream 共用對話段落注入，保留既有各工具 renderer |
| `crates/engine/speclink-core/tests/golden/assets.lock` | M | M | 是 | 由整合 renderer 重產快照／指紋，保留雙方內容 |
| `crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md` | M | M | 是 | 由整合 renderer 重產快照／指紋，保留雙方內容 |
| `crates/engine/speclink-core/tests/golden/claude.snapshot.md` | M | M | 是 | 由整合 renderer 重產快照／指紋，保留雙方內容 |
| `crates/engine/speclink-core/tests/golden/codex.snapshot.md` | M | M | 是 | 由整合 renderer 重產快照／指紋，保留雙方內容 |
| `crates/engine/speclink-core/tests/golden/copilot.snapshot.md` | M | M | 是 | 由整合 renderer 重產快照／指紋，保留雙方內容 |
| `crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md` | M | M | 是 | 由整合 renderer 重產快照／指紋，保留雙方內容 |
| `crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md` | M | M | 是 | 由整合 renderer 重產快照／指紋，保留雙方內容 |
| `crates/engine/speclink-core/tests/it/render_golden.rs` | M | M | 否 | 合併雙方回歸；增加 Copilot 共用段落與 Fork 語言／安全段落保留測試 |
| `crates/engine/speclink-core/tests/it/skill_authoring.rs` | — | M | 否 | 保留 Upstream conversation-language 原稿守門 |
| `crates/engine/speclink-fs/Cargo.toml` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-fs/src/lib.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-fs/src/security.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-fs/src/security/prepared.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-fs/src/security/reports.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/engine/speclink-fs/src/security/scan.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/host/speclink-host/Cargo.toml` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/host/speclink-host/src/bridge.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/host/speclink-host/src/context.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/host/speclink-host/src/lib.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/host/speclink-host/src/policy.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/host/speclink-host/src/portable.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/host/speclink-host/src/security.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/host/speclink-host/src/security_data.rs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/host/speclink-server/src/api/routes.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/host/speclink-server/src/identity/auth.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/host/speclink-server/tests/it/api/policy_write.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/host/speclink-server/tests/it/api/query_routes.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/host/speclink-server/tests/it/phase2_chain.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/protocol/speclink-protocol/src/binding.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/protocol/speclink-protocol/src/query.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/protocol/speclink-remote/src/auth.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/protocol/speclink-remote/src/client.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `crates/protocol/speclink-remote/src/credentials.rs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `docs/configuration.md` | M | M | 是 | 合併新版文件／對話語言區別，保留 Fork 系統語言預設及安全政策 |
| `docs/configuration.zh-TW.md` | M | M | 是 | 合併新版文件／對話語言區別，保留 Fork 系統語言預設及安全政策 |
| `docs/development.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `docs/development.zh-TW.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `docs/development/windows-portable.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `docs/security-review-files.txt` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `docs/security-review.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `docs/security-review.zh-TW.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/LANGUAGE.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-10-copilot-project-tool/.evidence.json` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-10-copilot-project-tool/.openspec.yaml` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-10-copilot-project-tool/design.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-10-copilot-project-tool/proposal.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-10-copilot-project-tool/specs/baseline-skill/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-10-copilot-project-tool/specs/desktop-config/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-10-copilot-project-tool/specs/node-sdk/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-10-copilot-project-tool/specs/workflow-config/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-10-copilot-project-tool/specs/workspace-tools/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-10-copilot-project-tool/tasks.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/.evidence.json` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/.openspec.yaml` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/design.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/proposal.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/specs/desktop-app/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/specs/desktop-config/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/specs/tray-status-menu/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/tasks.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/validation.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-16-skill-conversation-language/.evidence.json` | — | A | 否 | 保留 Upstream 新增／修改（未另改寫） |
| `openspec/changes/archive/2026-10-16-skill-conversation-language/.openspec.yaml` | — | A | 否 | 保留 Upstream 新增／修改（未另改寫） |
| `openspec/changes/archive/2026-10-16-skill-conversation-language/design.md` | — | A | 否 | 保留 Upstream 新增／修改（未另改寫） |
| `openspec/changes/archive/2026-10-16-skill-conversation-language/proposal.md` | — | A | 否 | 保留 Upstream 新增／修改（未另改寫） |
| `openspec/changes/archive/2026-10-16-skill-conversation-language/specs/audit-skill/spec.md` | — | A | 否 | 保留 Upstream 新增／修改（未另改寫） |
| `openspec/changes/archive/2026-10-16-skill-conversation-language/specs/config-skill/spec.md` | — | A | 否 | 保留 Upstream 新增／修改（未另改寫） |
| `openspec/changes/archive/2026-10-16-skill-conversation-language/specs/discuss-skill/spec.md` | — | A | 否 | 保留 Upstream 新增／修改（未另改寫） |
| `openspec/changes/archive/2026-10-16-skill-conversation-language/specs/improve-skill/spec.md` | — | A | 否 | 保留 Upstream 新增／修改（未另改寫） |
| `openspec/changes/archive/2026-10-16-skill-conversation-language/specs/skill-authoring/spec.md` | — | A | 否 | 保留 Upstream 新增／修改（未另改寫） |
| `openspec/changes/archive/2026-10-16-skill-conversation-language/tasks.md` | — | A | 否 | 保留 Upstream 新增／修改（未另改寫） |
| `openspec/changes/archive/2026-10-17-add-owasp-cwe-preliminary-review/.evidence.json` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-17-add-owasp-cwe-preliminary-review/.openspec.yaml` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-17-add-owasp-cwe-preliminary-review/design.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-17-add-owasp-cwe-preliminary-review/proposal.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-17-add-owasp-cwe-preliminary-review/specs/desktop-app/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-17-add-owasp-cwe-preliminary-review/specs/desktop-config/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-17-add-owasp-cwe-preliminary-review/specs/security-review-report/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-17-add-owasp-cwe-preliminary-review/specs/security-review/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-17-add-owasp-cwe-preliminary-review/specs/security-standard-data/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-17-add-owasp-cwe-preliminary-review/specs/server-policy-write/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-17-add-owasp-cwe-preliminary-review/specs/ui-copy-vocabulary/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-17-add-owasp-cwe-preliminary-review/specs/verify-skill/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-17-add-owasp-cwe-preliminary-review/specs/workflow-config/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/changes/archive/2026-10-17-add-owasp-cwe-preliminary-review/tasks.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/discussions/archive/2026-10-07-copilot-project-tool.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/discussions/archive/2026-10-12-desktop-theme-and-project-directory.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/discussions/archive/2026-10-16-owasp-cwe-preliminary-security-review.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/discussions/archive/2026-10-16-skill-conversation-language.md` | — | A | 否 | 保留 Upstream 新增／修改（未另改寫） |
| `openspec/manual/about.md` | M | M | 是 | 合併新版 audit／語言手冊；保留 Fork 安全審查入口、語言預設與來源 |
| `openspec/manual/accounts.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/manual/audit.md` | — | A | 否 | 合併新版 audit／語言手冊；保留 Fork 安全審查入口、語言預設與來源 |
| `openspec/manual/baseline.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/manual/claude-session-focus.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/manual/claude-skill-bar.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/manual/claude-workflow-panel.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/manual/desktop-projects.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/manual/discuss.md` | — | M | 否 | 合併新版 audit／語言手冊；保留 Fork 安全審查入口、語言預設與來源 |
| `openspec/manual/index.md` | M | M | 是 | 合併新版 audit／語言手冊；保留 Fork 安全審查入口、語言預設與來源 |
| `openspec/manual/policy-config.md` | M | M | 是 | 合併新版 audit／語言手冊；保留 Fork 安全審查入口、語言預設與來源 |
| `openspec/manual/security-data.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/manual/security-report.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/manual/security-review.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/manual/verify.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/manual/worktree.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/specs/audit-skill/spec.md` | — | A | 否 | 合併新版語言契約，以 Fork languages 解析 OS／Server 語言並明示查詢失敗 |
| `openspec/specs/baseline-skill/spec.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/specs/claude-code-session-focus/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/specs/claude-code-skill-bar/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/specs/claude-code-workflow-panel/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/specs/config-skill/spec.md` | — | M | 否 | 合併新版語言契約，以 Fork languages 解析 OS／Server 語言並明示查詢失敗 |
| `openspec/specs/delivery-baseline/spec.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/specs/desktop-app/spec.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/specs/desktop-config/spec.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/specs/discuss-skill/spec.md` | — | M | 否 | 合併新版語言契約，以 Fork languages 解析 OS／Server 語言並明示查詢失敗 |
| `openspec/specs/improve-skill/spec.md` | — | M | 否 | 合併新版語言契約，以 Fork languages 解析 OS／Server 語言並明示查詢失敗 |
| `openspec/specs/node-sdk/spec.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/specs/security-review-report/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/specs/security-review/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/specs/security-standard-data/spec.md` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/specs/server-policy-write/spec.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/specs/skill-authoring/spec.md` | — | M | 否 | 保留 Upstream 新增／修改（未另改寫） |
| `openspec/specs/tray-status-menu/spec.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/specs/ui-copy-vocabulary/spec.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/specs/verify-skill/spec.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/specs/workflow-config/spec.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `openspec/specs/workspace-tools/spec.md` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `package-lock.json` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `package.json` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/__tests__/archivedDetailPage.test.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/__tests__/changeDetailPage.test.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/__tests__/discussionDetailPage.test.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/__tests__/forkThemeMigration.test.ts` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/__tests__/improveBadge.test.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/__tests__/reviewBadge.test.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/__tests__/securityReviewPanel.test.tsx` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/__tests__/sonner.test.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/__tests__/specDetailPage.test.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/__tests__/theme.test.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/__tests__/themePreference.test.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/__tests__/verifyBadge.test.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/adapter.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/components/ChangeDetailPage.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/components/SecurityReviewPanel.tsx` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/components/ui/sonner.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/i18n.tsx` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/index.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/security.ts` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/theme.css` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/src/theme.ts` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `packages/ui/tsconfig.json` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `scripts/ci/ci-scope.mjs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `scripts/ci/ci-scope.test.mjs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `scripts/claude-code/skill-groups.test.mjs` | M | — | 否 | 逐位元保留 Fork（相對整合前） |
| `scripts/desktop/desktop-release-version.mjs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `scripts/desktop/desktop-release.test.mjs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `scripts/desktop/windows-portable.mjs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |
| `scripts/desktop/windows-portable.test.mjs` | A | — | 否 | 逐位元保留 Fork（相對整合前） |

## Fork 提交與來源

git log --no-merges 共同祖先..整合前：

- 4172a4dc ci(desktop): 支援預先發行版並設定 0.9.0-build.m1
- 01758c3b ci(desktop): 限縮純發布設定變更的檢查範圍
- 9f9d9472 ci(desktop): 新增 Tag 自動建置與發布
- cdc3d0c0 fix(security): 維持 Windows 更新期間的資料可見性
- 13b01c33 fix(security): 修正 Windows 根目錄路徑比對
- 8d8d2ca2 fix(ui): 將專案圖示配色移入共用主題
- 67a77bc6 feat(security): 新增 OWASP／CWE 初步安全審查
- 242a5836 docs(specs): 封存桌面外觀與專案目錄變更
- a886c9dd fix(desktop): 保留專案目錄空白並記錄品質驗證
- 184e9923 chore(skills): 更新 Claude 技能並整理專案工具設定
- 538d024b feat(desktop): 新增外觀選擇與專案目錄顯示
- 1fe2783e feat(desktop): 新增 Windows 隨身碟獨立模式與免安裝包
- 742ea8bd chore(project): 將 Copilot 納入專案工具設定
- 6f1946b0 ci(desktop): 新增三平台手動建置流程
- c688c569 chore(ci): 重新觸發 PR 測試
- 904c9a9e feat(project): 新增 Copilot 支援與系統語系預設

另檢查 merge 歷史：`e8132966` 已整合官方 0.9 與客製相容修正，`41eee53b` 將其合回 Fork；不能把早期 Copilot／外觀舊實作再次新增。`700d3c21` 引入安全審查；其他 PR merge 與提交均保留在雙親歷史中。`git cherry` 的 16 個非 merge patch 都為 Fork 獨有，但 patch 不同不代表功能一定不同；Copilot／外觀已由前次整合採官方架構與客製補強。

## Upstream 提交


- 05e5e65a chore(release): 版號升至 0.10.0
- b0f6726d docs(manual): 新增安全稽核頁，討論與政策頁補上記錄語言與對話語言
- b8dde4e4 speclink(skill-conversation-language): 技能對話語言規則與四個技能的 locale 綁定併入正式規格並封存
- ae3ac22b feat(skills): 技能檔開頭加上共用對話語言規則，四個技能的產出依 locale
- b908c985 docs(changes): 新增技能對話語言與 locale 綁定提案

## 歷史刪除與移動

Fork 歷史共 10 筆 rename，均為外觀／專案目錄變更及討論的封存移動；無歷史 D 刪除。Upstream 本次無 D/R。舊的 desktop theme 控制器已在前次 0.9 整合移植到官方 shared theme；本次沒有恢復舊實作。

| 歷史狀態 | 舊路徑 | 新路徑 |
|---|---|---|
| R100 | `openspec/changes/desktop-theme-and-project-directory/.evidence.json` | `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/.evidence.json` |
| R098 | `openspec/changes/desktop-theme-and-project-directory/.openspec.yaml` | `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/.openspec.yaml` |
| R100 | `openspec/changes/desktop-theme-and-project-directory/design.md` | `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/design.md` |
| R100 | `openspec/changes/desktop-theme-and-project-directory/proposal.md` | `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/proposal.md` |
| R100 | `openspec/changes/desktop-theme-and-project-directory/specs/desktop-app/spec.md` | `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/specs/desktop-app/spec.md` |
| R100 | `openspec/changes/desktop-theme-and-project-directory/specs/desktop-config/spec.md` | `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/specs/desktop-config/spec.md` |
| R100 | `openspec/changes/desktop-theme-and-project-directory/specs/tray-status-menu/spec.md` | `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/specs/tray-status-menu/spec.md` |
| R091 | `openspec/changes/desktop-theme-and-project-directory/tasks.md` | `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/tasks.md` |
| R084 | `openspec/changes/desktop-theme-and-project-directory/validation.md` | `openspec/changes/archive/2026-10-15-desktop-theme-and-project-directory/validation.md` |
| R100 | `openspec/discussions/desktop-theme-and-project-directory.md` | `openspec/discussions/archive/2026-10-12-desktop-theme-and-project-directory.md` |
