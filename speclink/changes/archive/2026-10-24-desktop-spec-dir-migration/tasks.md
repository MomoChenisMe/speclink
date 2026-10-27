## 1. 探測回報遷移狀態（D1）

- [x] 1.1 （Red）crates/engine/speclink-core/src/workspace/init.rs 的探測測試：舊佈局可遷移時 `migration.state` 為 `pending` 且 `metaFiles` 為資料檔數；有其他 worktree、未提交修改或位於 linked worktree 時為 `deferred` 並附原因與 worktree 路徑；新佈局、明寫 spec_dir 與遠端為 `none`；技能檔為目前版本但待遷移時探測結果仍帶 `pending`；序列化欄位為 camelCase。驗證：`cargo test -p speclink-core probe` 新測試失敗 <!-- speclink-task:tsk_01M5J3EJ5NQXJKZR56AP9538FR -->
- [x] 1.2 （Green）`AssetProbe` 新增 `migration`，以 migrate.rs 的 `plan` 計算，預設資料夾改用實際解析。驗證：`cargo test -p speclink-core probe` 通過；apps/desktop/core/src/project.rs 的 `probe_assets_at` 與 `update_assets_at`（回傳加上遷移報告）測試以 `cargo test -p speclink-desktop-core project` 通過 <!-- speclink-task:tsk_01M5J3EJ5NXGX2T9THJX0E99JC -->

## 2. 提示、更新與監看（D2）

- [x] 2.1 （Red）apps/desktop 的 assetPrompt 與對話框測試釘住需求「技能檔提示承載規格資料夾遷移」：待遷移時對話框寫明 `openspec/` 改名為 `speclink/`、資料檔數與不會 commit；暫緩時列出原因與 worktree 路徑、按更新只更新技能檔；完成時顯示要 commit 的路徑與兩則提醒並呼叫 `rearmWatch`；技能檔不過期但待遷移時仍出現提示；略過只記這一版；遠端工作區沒有遷移提示。驗證：`npm test -w apps/desktop -- assetPrompt AssetNoticeCard` 失敗 <!-- speclink-task:tsk_01M5J3EJ5NG5GSBFPSK5DSYSQE -->
- [x] 2.2 （Green）修改 apps/desktop/src/assetPrompt.ts、apps/desktop/src/components/AssetNoticeCard.tsx、apps/desktop/src/store.ts 與 apps/desktop/src/i18n/messages.ts：依 `migration` 產生提示與文案，更新前暫停監看、完成後 `rearmWatch`，錯誤原樣顯示在對話框。驗證：`npm test -w apps/desktop -- assetPrompt AssetNoticeCard` 通過 <!-- speclink-task:tsk_01M5J3EJ5N4VQAR5APK2A36GB7 -->
- [x] 2.3 apps/desktop/src-tauri/src/watch.rs 的測試新增：專案由 `openspec/` 改名為 `speclink/` 後重新解析與重掛，`speclink/` 下的寫入觸發通知；監看函式改用與資料夾名稱無關的命名。驗證：`cargo test -p speclink-desktop watch`（先依記憶補 sidecar 與 server-web dist）通過 <!-- speclink-task:tsk_01M5J3EJ5N5C2WBEB65TWHKY53 -->

## 3. 啟用與初始化（D3、D4）

- [x] 3.1 （Red）crates/engine/speclink-core/src/workspace/init.rs 的 adopt 測試釘住需求「工作區補齊入口」：OpenSpec 專案匯入後 `speclink/` 內容與原檔位元級相同、資料檔為 `.meta.yaml`、`.speclink.yaml` 沒有 spec_dir；骨架缺件補在 `speclink/`；`.gitignore` 追加；重複執行冪等；tools 空清單拒絕；未提交修改時回錯誤且零寫入。驗證：`cargo test -p speclink-core adopt` 失敗 <!-- speclink-task:tsk_01M5J3EJ5NK9ER5P7DGE08TNM4 -->
- [x] 3.2 （Green）`adopt` 依 D3 先匯入再補齊。驗證：`cargo test -p speclink-core adopt` 通過 <!-- speclink-task:tsk_01M5J3EJ5NYGSXN9S47HZSAY1E -->
- [x] 3.3 （Red→Green）需求「未啟用資料夾經確認後補齊啟用」與「未初始化目錄經確認後自動初始化」：apps/desktop/core/src/project.rs 的 `Unadopted` 探測附「需要匯入」旗標、初始化改用預設資料夾；前端啟用對話框在需要匯入時顯示改名與 OpenSpec CLI 讀不到的說明，初始化對話框寫 `speclink/`；匯入失敗時顯示單行錯誤且不切換專案。驗證：`cargo test -p speclink-desktop-core project` 與 `npm test -w apps/desktop -- emptyWorkspace workspaceChooser App` 通過 <!-- speclink-task:tsk_01M5J3EJ5NYDV0ZEQNV565EVHE -->

## 4. 畫面標示（D4）

- [x] 4.1 （Red→Green）需求「畫面依實際規格資料夾與資料檔名顯示」：設定快照帶出規格資料夾相對路徑（apps/desktop/core/src/settings.rs、apps/desktop/src/adapter/workspace.ts、apps/desktop/src/settingsCatalog.ts），四個專案設定頁（WorkflowPage、RulesPage、ContextPage、ReviewRulesPage）的 `PageNote` 與分類欄組名以它組成；packages/ui/src/components/DiscussionDetailPage.tsx 的記錄路徑以資料來源的資料夾組成；packages/ui/src/i18n.tsx 的 `card.invalidMetaTitle` 改為「變更資料檔解析失敗」（英文同步）；messages.ts 其餘寫死 `openspec/` 的說明改為 `speclink/` 或「本機規格資料夾」。測試涵蓋新佈局、舊佈局與討論記錄路徑三個情境。驗證：`npm test -w apps/desktop -- projectSettingsView settingsColumn` 與 `npm test -w packages/ui -- discussionDetailPage ui` 通過，且 `grep -rn "openspec" apps/desktop/src packages/ui/src --include=*.ts --include=*.tsx` 在非測試檔案只剩註解，以及必須寫出舊名的遷移與匯入說明（`assets.*` 遷移文案、`app.adoptImportNote`）與遷移狀態的資料夾代碼比對 <!-- speclink-task:tsk_01M5J3EJ5NWY4CRVSPZMW5DC09 -->

## 5. 本機遷移到遠端（D5）

- [x] 5.1 （Red→Green）需求「遷移成功後才轉換本地且一律備份」與「新舊預設資料夾都可遷移到遠端」：apps/desktop/src-tauri/src/remote.rs 的前置檢查只拒絕明寫自訂資料夾、備份名依實際資料夾組成並保留序號規則、上傳已封存變更資料檔一律以 `.meta.yaml` 為鍵（並存時取 `.meta.yaml`）。測試涵蓋 `speclink.migrated-<日期>-2` 序號、舊佈局可遷移、自訂資料夾被拒且無遠端寫入。驗證：`cargo test -p speclink-desktop remote` 通過 <!-- speclink-task:tsk_01M5J3EJ5NR8ZV88Y9CX78012P -->

## 6. 測試夾具與收尾

- [x] 6.1 apps/desktop/core/src/testfixture.rs 與 desktop 前端測試的夾具改為 `speclink/` 與 `.meta.yaml`，只有驗證舊佈局、匯入與遷移的測試保留 `openspec/`。驗證：`cargo test -p speclink-desktop-core` 與 `npm test -w apps/desktop` 全部通過 <!-- speclink-task:tsk_01M5J3EJ5N5Y82PXPM57EA4K7F -->
- [x] [M] 6.2 用桌面 app（dev 視窗）開啟一個仍是 openspec/ 的舊專案副本：確認提示對話框寫明會改名、按更新後資料夾改名且看板持續自動更新；再開啟一個 OpenSpec 專案副本，確認啟用對話框寫明改名並在確認後匯入 <!-- speclink-task:tsk_01M5J3EJ5N6F4XH6TBP0HV5SXB -->
- [x] 6.3 受影響面測試與文件守門全部通過。驗證：`cargo test -p speclink-core`、`cargo test -p speclink-desktop-core`、`cargo test -p speclink-desktop`、`npm test -w apps/desktop`、`npm test -w packages/ui`、`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 皆通過，且 `speclink validate desktop-spec-dir-migration` 通過 <!-- speclink-task:tsk_01M5J3EJ5N756JYPD5K48XMXPE -->

## 7. 技能足跡同步不遷移（D6）

- [x] 7.1 （Red→Green）需求「技能足跡同步不遷移規格資料夾」：crates/engine/speclink-core/src/workspace/init.rs 新增只同步技能檔的入口（不遷移、降級守門照舊），apps/desktop/core/src/settings.rs 的 `write_workflow_fields_at` 與 crates/adapters/speclink-cli/src/verbs/config.rs 的 `workflow-config set worktree` 改用它。測試涵蓋：符合遷移條件的舊佈局專案切換 worktree 後 `openspec/` 原封不動、沒有 `speclink/`、worktree 技能生成，CLI stdout 沒有遷移提示。驗證：`cargo test -p speclink-core sync_skills`、`cargo test -p speclink-desktop-core settings`、`cargo test -p speclink-cli workflow_config` 通過 <!-- speclink-task:tsk_01M5MS9NDHHMVZ93G58CJMRTJQ -->
