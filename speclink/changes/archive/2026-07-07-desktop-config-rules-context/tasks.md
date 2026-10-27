## 1. 前置相依檢查

- [x] 1.1 確認 desktop-config-multiproject 已完成封存、openspec/specs/desktop-config/spec.md 已為正典 spec，並執行 /speclink-drift 比對本變更 delta 與正典措辭（MODIFIED 需求「設定頁圖形化讀寫兩層設定」以主刀封存後文字為基準），有出入先校正 delta。驗證：speclink validate desktop-config-rules-context 通過且 drift 檢查無待處理項。

## 2. speclink-core 純函式擴充（design D1 擴充主刀政策純函式的變更集（speclink-core 單一寫入真相））

- [x] 2.1 紅：於 crates/speclink-core/src/config.rs 的 #[cfg(test)] 撰寫變更集擴充的失敗測試——context 三態（設值／移除／不動）；rules 整份代換（含條目順序保持）；某節空清單移除該 artifact 鍵、全部為空移除 rules 鍵；條目存入前 trim、空字串條目滌除；以反引號或 at 符號開頭的條目寫出後可被 WorkflowConfig::from_text 解析且值逐字元還原（自動加引號）；未觸及鍵（remote、spec_dir 等，即 MODIFIED 需求「設定頁圖形化讀寫兩層設定」的保留名單）逐字元保留；壞 YAML 輸入回 Err。驗證：cargo test -p speclink-core 出現預期紅燈。
- [x] 2.2 綠：實作 update_workflow_config_text 的 context 與 rules 變更集（維持 text→text 純函式與 serde_yaml::Mapping 讀-改-寫，不觸檔案系統），2.1 測試全綠。驗證：cargo test -p speclink-core 全綠，且 crates/speclink-cli 零改動、CLI 回歸對照不受影響。

## 3. desktop-core 設定橋接擴充（design D3 設定讀取 payload 擴充 context、rules 與 schemaArtifacts；design D4 寫入安全與鍵移除語意沿用主刀雙重驗證）

- [x] 3.1 紅：於 apps/desktop/core/src/settings.rs 撰寫失敗測試——讀取 payload 新增 context、rules、schemaArtifacts 三欄（camelCase 序列化；檔案缺席回未設定狀態；解析失敗檔回 parseError 而非靜默預設；schemaArtifacts 為活躍 schema 的 artifact id 依引擎顯示序）；寫入函式接受 context 三態與 rules 代換並沿用雙重驗證流程（寫檔前解析原文與驗證新文字、寫檔後回讀再驗，任一步失敗回指明檔案與階段的單行 Err 且磁碟檔案逐字元不變）。驗證：cargo test -p speclink-desktop-core 出現預期紅燈。
- [x] 3.2 綠：實作讀取與寫入擴充（呼叫 2.2 的 core 純函式，檔案讀寫與驗證留在本層），3.1 測試全綠。驗證：cargo test -p speclink-desktop-core 全綠。
- [x] 3.3 對新增的設定寫入路徑套用 sharp-edges 檢查清單（speclink instructions --skill audit 取得）——重點：壞輸入不落檔、錯誤不靜默、鍵移除語意無型別混淆。驗證：檢查清單逐項核對並將結果記於本任務完成註記，無未處理項。

## 4. 設定頁前端（design D2 產出規則清單編輯器採 schema 固定鍵與上下移排序；design D5 GUI 文案採正典詞且新字串全數進 i18n 字典）

- [x] 4.1 紅：撰寫 SettingsView 失敗測試，覆蓋需求「設定頁編輯專案說明與產出規則」——「專案說明」多行文字區呈現現值與寫入呼叫；「產出規則」以 schemaArtifacts 固定鍵分節且無自由鍵輸入、條目新增／編輯／刪除／上下移排序（順序進入寫入 payload）、清空觸發鍵移除語意；config.yaml 解析失敗（parseError）時兩區段停用；區段名「專案說明」「產出規則」與註解遺失說明文字皆經 i18n 字典（zh-TW／en key 集合相等）。驗證：npm test -w apps/desktop 出現預期紅燈。
- [x] 4.2 綠：apps/desktop/src/views/SettingsView.tsx 實作兩區段（清單編輯器為 SettingsView 內部元件、上下移按鈕排序），apps/desktop/src/adapter/tauriDataSource.ts 與 apps/desktop/src/i18n/messages.ts 對應擴充，4.1 測試全綠。驗證：npm test -w apps/desktop 全綠。

## 5. 建置與真實視窗驗證

- [x] 5.1 前端與桌面殼建置成功。驗證：npm run build -w apps/desktop 與 cargo build --release -p speclink-desktop 皆 exit 0（重建前先關閉執行中的 speclink-desktop exe）。
- [x] 5.2 真實視窗驗證（操作前先確認使用者未在使用螢幕；於臨時測試工作區啟動 release exe，備妥含 rules、context 與註解的 config.yaml）：編輯專案說明後開檔核對值與其餘鍵保留；於 tasks 節新增以 at 符號開頭的條目後檔案仍可被引擎解析（設定頁重讀無 parseError）；上下移排序後檔案條目順序對調；清空專案說明與某節條目後對應鍵被移除；手動改壞 config.yaml 後兩區段停用且不可儲存。驗證：CopyFromScreen 截圖與檔案內容逐項核對相符。
