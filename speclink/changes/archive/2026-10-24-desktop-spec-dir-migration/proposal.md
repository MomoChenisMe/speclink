## Why

spec-dir-migration 讓 CLI 與引擎改用 `speclink/` 與 `.meta.yaml`，並在 `speclink update` 時自動遷移、在 `speclink init` 時匯入 OpenSpec 專案；但多數使用者透過桌面 app 更新技能檔與開啟專案，桌面這條路若不跟上，使用者按下「更新」時不知道資料夾會被改名、遷移後畫面停止自動更新、OpenSpec 專案的「啟用」仍把內容留在 `openspec/`，文案與設定頁也繼續寫 `openspec/`。本變更是討論 positioning-and-process-cost 分刀計畫第 7 刀在 propose 時拆出的桌面部分，依賴 spec-dir-migration，並同樣在第一批六刀發版之後才封存。

目標使用者與情境：用桌面 app 開專案的開發者（團隊情境下含 PO 與 PM）——app 升版後開啟舊專案、看到技能檔過期提示並按更新；以「啟用」開啟 OpenSpec 專案；在專案設定頁查看檔案位置；把本機專案遷移到遠端。

## What Changes

- 引擎的技能檔過期探測多回報遷移狀態（無、待遷移、暫緩與原因、待改名的資料檔數）；桌面的技能檔提示卡與對話框在待遷移時寫明 `openspec/` 會改名為 `speclink/`、N 個資料檔會改名、不會 commit，暫緩時列出原因、按更新只更新技能檔，完成後顯示與 CLI 相同的提醒；略過照舊只記這一版。
- 遷移完成後，桌面重新解析規格資料夾並重啟監看，畫面持續自動更新。app 自己升版時不在背景遷移任何專案。
- 引擎的工作區補齊入口（adopt，桌面「啟用」走這條）遇到 OpenSpec 專案時改為匯入：與 init 匯入相同，把 `openspec/` 搬到 `speclink/`、資料檔改名、既有內容不改寫，有其他 worktree 或未提交修改時拒絕；啟用對話框寫明資料夾會改名、OpenSpec CLI 之後讀不到。
- 桌面「初始化」建立 `speclink/`；畫面上提到規格資料夾與資料檔的地方（初始化、啟用、來源選擇與並存衝突、遷移到遠端的說明、設定頁的檔案註記、討論詳情的記錄路徑、資料檔解析失敗的提示）依專案實際的資料夾與檔名顯示。
- 本機遷移到遠端的備份資料夾改用實際資料夾名稱 `<資料夾名>.migrated-<日期>`；預設 `speclink/` 與舊佈局 `openspec/` 都可遷移（只有明寫的自訂資料夾仍拒絕）；上傳的已封存變更資料檔一律以 `.meta.yaml` 為鍵。
- 切換 worktree 政策後的技能足跡同步（CLI `workflow-config set worktree` 與桌面產出流程頁的開關）只同步技能檔、不再順帶遷移規格資料夾：apply 期發現它們走引擎 `update()`，舊佈局專案會在沒有說明、沒有暫停監看的情況下被改名。遷移只留給 `speclink update` 與桌面技能檔提示的「更新」。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workspace-tools`: 「工作區補齊入口」遇到 OpenSpec 專案改為匯入到 `speclink/`。
- `desktop-config`: 「未初始化目錄經確認後自動初始化」建立 `speclink/`；「未啟用資料夾經確認後補齊啟用」改為匯入並說明改名；新增畫面依實際規格資料夾與資料檔名顯示的需求。
- `desktop-app`: 新增技能檔提示承載規格資料夾遷移的需求。
- `workspace-migration`: 「遷移成功後才轉換本地且一律備份」改以實際資料夾名稱備份；新增新舊預設資料夾與資料檔名都可遷移的需求。
- `spec-dir-migration`: 新增技能足跡同步不遷移規格資料夾的需求。

## Impact

- Affected specs: `workspace-tools`, `desktop-config`, `desktop-app`, `workspace-migration`, `spec-dir-migration`
- Affected code:
  - Modified: crates/engine/speclink-core/src/workspace/init.rs, crates/engine/speclink-core/src/workspace/migrate.rs, crates/adapters/speclink-cli/src/verbs/config.rs, apps/desktop/core/src/project.rs, apps/desktop/core/src/settings.rs, apps/desktop/core/src/discussions.rs, apps/desktop/core/src/testfixture.rs, apps/desktop/src/adapter/workspace.ts, apps/desktop/src-tauri/src/remote.rs, apps/desktop/src-tauri/src/watch.rs, apps/desktop/src-tauri/src/lib.rs, apps/desktop/src/App.tsx, apps/desktop/src/store.ts, apps/desktop/src/assetPrompt.ts, apps/desktop/src/components/AssetNoticeCard.tsx, apps/desktop/src/components/AssetNoticeDialog.tsx, apps/desktop/src/components/ProjectSettingsColumn.tsx, apps/desktop/src/i18n/messages.ts, apps/desktop/src/views/ProjectSettingsView.tsx, apps/desktop/src/views/project-settings/useProjectSettings.ts, apps/desktop/src/views/project-settings/WorkflowPage.tsx, apps/desktop/src/views/project-settings/RulesPage.tsx, apps/desktop/src/views/project-settings/ContextPage.tsx, apps/desktop/src/views/project-settings/ReviewRulesPage.tsx, apps/desktop/src/settingsCatalog.ts, packages/ui/src/adapter.ts, packages/ui/src/i18n.tsx, packages/ui/src/components/DiscussionDetailPage.tsx
- 影響的 crate／app：speclink-core（探測回報遷移狀態、adopt 匯入）、apps/desktop/core（快照帶實際資料夾、啟用與探測委派）、apps/desktop/src-tauri（遠端遷移備份、監看重啟）、apps/desktop 與 packages/ui（提示、對話框與文案）。
- CLI：只改一處——`workflow-config set worktree` 的技能足跡同步不再遷移規格資料夾（CLI 的遷移與匯入仍由 spec-dir-migration 提供，`speclink update` 照舊遷移）。
- 設定欄位：沒有新增；`spec_dir` 預設值的改變來自 spec-dir-migration。
- 生成的技能：沒有技能原稿變動，不升 ASSET_VERSION。
- 相容性影響：桌面「啟用」OpenSpec 專案會搬移資料夾（之前不動既有內容）；本機遷移到遠端的備份資料夾名稱改為依實際資料夾；其餘畫面只改文案。
