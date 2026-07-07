## 1. speclink-core 設定回寫（design D4 設定回寫純函式落在 speclink-core：text→text＋Mapping 讀-改-寫）

- [x] 1.1 紅：在 crates/speclink-core/src/config.rs 的 #[cfg(test)] 撰寫工作流政策欄位更新函式的失敗測試——涵蓋：更新 locale／spec_locale／tdd／audit、未觸及鍵（rules、context、schema）逐字元保留、政策欄位設回預設值即移除鍵、輸入為壞 YAML 時回 Err、輸出文字可被 WorkflowConfig::from_text 解析且欄位值正確。驗證：cargo test -p speclink-core 出現預期紅燈。
- [x] 1.2 綠：於 crates/speclink-core/src/config.rs 實作 update_workflow_config_text（serde_yaml::Mapping 讀-改-寫、text→text 純函式，不觸碰檔案系統）。驗證：1.1 測試全綠。
- [x] 1.3 紅：撰寫 .speclink.yaml tools 清單更新函式的失敗測試——涵蓋：內建工具（claude／codex）選集代換、自訂工具描述子物件與 remote／spec_dir 鍵原樣保留、壞 YAML 輸入回 Err、輸出可被 AppConfig 解析。驗證：cargo test -p speclink-core 出現預期紅燈。
- [x] 1.4 綠：實作 update_app_config_tools_text，1.3 測試全綠；並確認 crates/speclink-cli 零改動、cargo test --workspace 全綠（CLI 人眼與 --json 輸出回歸對照不受影響）。

## 2. desktop-core 專案開啟與初始化橋接（design D3 開啟專案三態流程）

- [x] 2.1 紅：新增 apps/desktop/core/src/project.rs 的 tempdir 整合測試——開啟專案三態：(1) 選定含 openspec/ 的目錄回專案資訊（root 與專案名）；(2) 選定專案子目錄時向上探索命中專案根，涵蓋 spec 需求「執行期切換專案 root」的探索語意；(3) 未命中任何專案時回 uninitialized 且目標目錄零寫入；不存在或不可讀路徑回 Err 單行訊息。驗證：cargo test -p speclink-desktop-core 出現預期紅燈。
- [x] 2.2 綠：實作 project.rs 的 open_project_at（沿用 Workspace::discover 探索語意，不切換任何狀態、僅回報判定結果）。驗證：2.1 測試全綠。
- [x] 2.3 紅：撰寫 init_project_at 的 tempdir 測試，涵蓋 spec 需求「未初始化目錄經確認後自動初始化」的檔案效果——預設 claude：openspec/（specs/、changes/archive/、config.yaml）、.speclink.yaml（tools 含 claude）、CLAUDE.md 的 SPECLINK marker、.claude/skills/；加選 codex 時另有 AGENTS.md marker 與 .agents/skills/；目標不可寫時回 Err。驗證：cargo test -p speclink-desktop-core 出現預期紅燈。
- [x] 2.4 綠：實作 init_project_at（消費 speclink_core::init::init，force=false、spec_dir 固定 openspec）。驗證：2.3 測試全綠。

## 3. desktop-core 設定讀寫橋接（design D5 寫入前後雙重解析驗證）

- [x] 3.1 紅：新增 apps/desktop/core/src/settings.rs 的讀取測試——read_settings_at 區分三種狀態：檔案缺席／欄位未設定（回預設值狀態、無 parseError）、檔案存在且可解析（回實際欄位值與自訂描述子的存在標記）、檔案存在但解析失敗（回 parseError 訊息而非靜默預設）。驗證：cargo test -p speclink-desktop-core 出現預期紅燈。
- [x] 3.2 綠：實作 read_settings_at。驗證：3.1 測試全綠。
- [x] 3.3 紅：撰寫寫入測試，涵蓋 spec 需求「設定寫入具解析驗證且失敗浮出」——寫前解析原文失敗即中止、序列化結果先驗證再寫檔、寫後回讀再驗；任一步失敗時磁碟檔案與操作前逐字元一致且 Err 訊息指明檔案與階段；原檔解析失敗時拒絕寫入；tools 寫入成功後技能同步（加選 codex 生成 .agents/skills/ 與 AGENTS.md marker、取消工具清理殘留）。驗證：cargo test -p speclink-desktop-core 出現預期紅燈。
- [x] 3.4 綠：實作 write_workflow_fields_at 與 write_tools_at（呼叫 1.2／1.4 的 core 純函式，檔案讀寫與雙重驗證在此層；tools 寫入後呼叫 speclink_core::init::update）。驗證：3.3 測試全綠。
- [x] 3.5 對本階段新增的 API 與參數處理執行 sharp-edges audit checklist（speclink instructions --skill audit 取得清單），逐項記錄結論；發現的尖銳邊立即以紅綠循環補測試與修正。驗證：audit 清單逐項有結論、cargo test -p speclink-desktop-core 全綠。

## 4. Tauri 層（design D1 執行期可變 root：AppState 持 Mutex<PathBuf>；design D6 桌面專屬 command 不擴 SpeclinkDataSource）

- [x] 4.1 AppState 的 root 由 PathBuf 改為 Mutex<PathBuf>（apps/desktop/src-tauri/src/lib.rs），既有 14 個 command 改為鎖內複製 root 後照舊委派——行為契約：既有查詢與管理操作結果不變。驗證：cargo build -p speclink-desktop 通過、npm test -w apps/desktop 既有測試全綠。
- [x] 4.2 新增七支桌面專屬 Tauri command 並註冊 invoke_handler：open_project（判定三態，命中即更新 Mutex 內 root）、init_project（init 成功後切換 root）、current_project（回目前 root 與專案名）、project_stats（唯讀回指定路徑專案的進行中變更數，不切換 root、路徑失效回 Err——design D11 分頁徽章採背景快照制）、read_settings、write_app_tools、write_workflow_config——薄包裝委派 desktop-core，payload 欄位 camelCase、錯誤 Err(String)，與既有 command 慣例一致；此組 command 不進 packages/ui 的 SpeclinkDataSource（design D6）。驗證：cargo build -p speclink-desktop 通過，行為正確性由第 2、3 章 desktop-core 測試承載，Tauri 層以 code review 確認為無邏輯薄包裝。
- [x] 4.3 引入原生對話框（design D2 資料夾選擇採 tauri-plugin-dialog）：apps/desktop/src-tauri/Cargo.toml 加依賴並註冊 plugin、apps/desktop/src-tauri/capabilities/default.json 授予 dialog 開啟權限、apps/desktop/package.json 加 @tauri-apps/plugin-dialog——行為契約：前端可喚起原生資料夾選擇器並取得絕對路徑。驗證：cargo build -p speclink-desktop 與 npm run build -w apps/desktop 通過；實際喚起於 8.2 真實視窗驗證確認。

## 5. i18n 基建與字串抽 key（design D7 自製輕量 i18n（I18nProvider），packages/ui 匯出 I18nProvider 與 useI18n）

- [x] 5.1 紅：撰寫 packages/ui 的 i18n 單元測試——I18nProvider 依 locale 提供 t(key)、app 層 messages 與內建字典合併、缺 key 時 t 回傳 key 本身、zh-TW 與 en 內建字典 key 集合相等。驗證：npm test -w packages/ui 出現預期紅燈。
- [x] 5.2 綠：實作 packages/ui/src/i18n.tsx（React context＋zh-TW／en 內建字典）並自 packages/ui/src/index.ts 匯出 I18nProvider 與 useI18n。驗證：5.1 測試全綠。
- [x] 5.3 packages/ui 全部 13 個元件（packages/ui/src/components）的硬編 zh-TW 顯示字串抽 key 改經 t(key)；既有以中文斷言的元件測試改為包 I18nProvider locale zh-TW 後照舊斷言（斷言字串不變即回歸保護），並新增至少一個元件於 en 下渲染英文字串的測試。驗證：npm test -w packages/ui 全綠。
- [x] 5.4 apps/desktop 字串抽 key（apps/desktop/src/App.tsx 與 store 的使用者可見訊息）進 apps/desktop/src/i18n/messages.ts 並經 provider 合併；實作系統語言偵測與 app 本機偏好（design D8 語言偏好與最近清單存 localStorage：navigator.language 以 zh 開頭判 zh-TW 否則 en、localStorage 單鍵、null 表跟隨系統），涵蓋 spec 需求「UI 介面語言支援 zh-TW 與 en」的判定表（zh-TW／zh-CN→zh-TW，en-US／ja-JP→en）。驗證：npm test -w apps/desktop 全綠（含語言判定表的參數化測試）。

## 6. 前端：開啟專案流程與專案分頁列（design D3 開啟專案三態流程、design D8 語言偏好與最近清單存 localStorage、design D10 專案分頁列（UI 形態對齊 Spectra）、design D11 分頁徽章採背景快照制）

- [x] 6.1 紅：撰寫 store 與元件測試，涵蓋 spec 需求「專案分頁列存於 app 本機」——開啟專案 action 三態（命中即於 command 成功後整批 refresh、uninitialized 顯示初始化確認對話框、取消則畫面與狀態不動）；分頁列持久化行為（localStorage 存路徑＋顯示名＋順序＋最後活躍、上限 10、成功開啟去重上移、關閉分頁即移除、模擬重啟後還原）；點分頁切換走與開啟專案相同語意；徽章（active 分頁隨 refresh 更新、背景分頁保留啟動時 project_stats 查得的快照值）；分頁指向已消失路徑時轉錯誤態、點擊顯示錯誤並可自分頁移除且不寫入任何專案目錄。驗證：npm test -w apps/desktop 出現預期紅燈。
- [x] 6.2 綠：實作 apps/desktop 的 workspace adapter（直接 invoke 七支桌面 command，不經 SpeclinkDataSource）與 apps/desktop/src/components/ProjectTabs.tsx——分頁列（active 分頁 teal 粗框標示目前專案、✕ 僅 active 與 hover 顯示、「＋」掛分頁列尾端與右上「開啟專案」雙入口皆接 dialog 選擇器）、徽章＋hover tooltip「N 個進行中變更」（shadcn tooltip 原語之現缺者入 packages/ui/src/components/ui，維持 teal 設計系統）、初始化確認對話框（AI 工具多選 claude／codex 預設勾 claude；遵循 design D3 的寫入型確認框按鈕原則——取消鈕靠左持預設焦點、建立鈕靠右拉開距離）。驗證：6.1 測試全綠。
- [x] 6.3 綠：零分頁空狀態引導頁（含「開啟專案」操作與既有專案／一般目錄初始化說明，取代空看板）、失效分頁錯誤態 UI（警示標記＋自分頁移除）、鍵盤切換（Ctrl+Tab 循環、Ctrl+1..9 直達）。行為契約以 specs/desktop-config/spec.md「專案分頁列存於 app 本機」的空狀態、錯誤態與切換 Scenario 為準。驗證：npm test -w apps/desktop 對應測試全綠。
- [x] 6.4 分頁列取代 App.tsx 頂欄的「目前專案」佔位：啟動與每次切換後經 current_project 同步 active 分頁標示。驗證：npm test -w apps/desktop 斷言切換後 active 分頁與其標示更新。

## 7. 前端：設定頁（design D9 設定頁視圖與表單原語擴充：SettingsView 為 apps/desktop 視圖，原語入 packages/ui/components/ui）

- [x] 7.1 紅：撰寫 SettingsView 測試——載入呈現兩檔現值（未設定欄位呈預設狀態）、tools 多選與自訂描述子不可編輯項、locale／spec_locale 下拉與 tdd／audit 開關的寫入呼叫、parseError 時顯示警告且該檔表單停用、UI 語言三選（跟隨系統／zh-TW／en）切換即時全介面生效且不觸碰 config.yaml。驗證：npm test -w apps/desktop 出現預期紅燈。
- [x] 7.2 綠：實作 apps/desktop/src/views/SettingsView.tsx（涵蓋 spec 需求「設定頁圖形化讀寫兩層設定」）與所需 shadcn 原語（select、switch、checkbox 之現缺者，入 packages/ui/src/components/ui）；App.tsx 導航擴為三視圖（變更看板／已封存／設定），側欄「設定」接上 handler；tools 與政策欄位旁附說明文字（承接被 Mapping 讀-改-寫移除的範本註解教學角色，並區分 UI 語言與 config locale）。驗證：7.1 測試全綠。
- [x] 7.3 斷言 UI 語言偏好與 config.yaml 的 locale 互不影響：切換 UI 語言不改 config.yaml 內容、寫入 config locale 不改 UI 語言。驗證：npm test -w apps/desktop 對應測試綠。

## 8. 整合驗證與收尾

- [x] 8.1 全套自動化驗證：cargo test --workspace、npm test -w packages/ui、npm test -w apps/desktop 全綠；git diff 確認 crates/speclink-cli 零改動，scratchpad 的 parity／color 回歸對照照常通過（CLI 輸出未受影響）。
- [x] 8.2 真實視窗驗證（cargo build --release -p speclink-desktop 前先關閉執行中的 exe；操作前先確認使用者沒在使用螢幕）：實際切換專案與子目錄向上探索、未初始化目錄確認後 init（含加選 codex 的檔案效果檢視）、取消初始化零寫入、設定頁寫入後以編輯器檢視檔案（未觸及鍵保留、設回預設移除鍵、自訂描述子保留）、手動改壞 config.yaml 後設定頁警告與拒寫、UI 語言切換即時生效與重啟保持、分頁列跨重啟還原與去重、點分頁與 Ctrl+Tab／Ctrl+1..9 切換專案、背景分頁徽章快照與 active 分頁即時更新、刪除專案目錄後分頁錯誤態與自分頁移除、零分頁空狀態引導頁、既有看板拖曳互動回歸。驗證：上述每項有截圖或觀察記錄，行為與 specs/desktop-config/spec.md 各 Scenario 一致。
