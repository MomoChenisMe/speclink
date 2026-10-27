## Why

macOS 系統匣面板（tray-copy-and-panel-mode 產出、tray-macos-panel-only 裁決為 macOS 唯一樣式）目前的視覺仍複製原生選單的樣子：緊湊列、hr 分隔線、近乎無色——webview 面板的呈現自由度完全沒有發揮。使用者比對 CodexBar 的系統匣面板後，希望升級為卡片設計感並加強品牌主色（teal）的存在感，與桌面看板的視覺語彙呼應。另有一個實際缺陷：面板每次開啟，焦點自動落在第一顆複製鈕上，出現突兀的藍色 focus ring（成因：面板成為 key window 後，WebKit 把焦點交給面板中唯一可 tab 的元素——第一顆複製鈕）。

目標使用者：以桌面 app 監看 SDD 工作流的開發者。使用情境：點擊系統匣圖示快速一覽專案的變更進度與討論、原地切換專案、複製變更名稱／討論 slug 餵 CLI 動詞——全程不必開主視窗。

本提案源自討論 tray-panel-card-design 的結論，四項決定全數採納。

## What Changes

- **專案選擇改為 CodexBar 式橫向 tab 條**：垂直專案列（打勾＋名稱）改為橫向可捲動的 tab 條；每個 tab 為專案名首字母的圓角方塊 avatar＋專案名，超寬時左右捲動（隱藏捲軸）。作用中專案的 tab 以實心主色（teal）圓角卡＋反白文字呈現（呼應桌面側欄選中態），非作用中 tab hover 時鋪淡主色底。點擊 tab 沿用既有原地切換語意：切換作用中專案、下方內容隨快照更新，不喚起主視窗。
- **分區卡片化**：生命週期分區（提案中／進行中／已就緒）與討論分區（討論／已轉出）各自改為半透明圓角卡片容器，疊在既有毛玻璃（vibrancy）之上；分區標題放大、間距放寬，取代現行 hr 分隔線。非每列一卡。
- **主色加量**：變更列的進度條依階段套用看板同款的 teal 深淺階梯（提案中／進行中／已就緒漸深）；分區標題圖示以 teal 上色；面板底可疊極淡的 teal 漸層 wash（不遮蔽毛玻璃）。
- **修復複製鈕自動 focus**：面板開啟時焦點不得自動落在任何互動元素上——複製鈕退出 tab 順序（面板為滑鼠驅動介面，列本體本就不可鍵盤操作），開啟後不再出現藍色 focus ring。
- **vibrancy 材質換 HudWindow**：原 Menu 材質於淺色模式近乎不透、毛玻璃不可辨（違反「毛玻璃底可透出」的規格要求）——換為透感最強的 HudWindow（真實視窗實測裁決，2026-07-16）。
- **tab 條尾端快速加入專案**：「加入專案」動作項先喚起主視窗（桌面切至其所在處，確保對話框可見——實測修訂 2026-07-16：直開時選擇器落在不可見桌面）再開資料夾選擇器（沿用主視窗「開啟專案」語意——store 既有的資料夾選擇流程），選定即加入分頁並切換、取消無事；面板動作事件新增 add-project、i18n 新增對應鍵——使用者實測後補充（ingest 2026-07-16）。
- **分區標題計數徽章**：各分區標題顯示項目計數，徽章與看板欄計數同語彙（STAGE_BADGE 單一來源）——使用者實測後補充（ingest 2026-07-16）。
- **空狀態卡最小高度**：討論零筆與全無變更的空狀態卡維持最小高度、內容垂直置中，不塌陷成細條——使用者實測後補充（ingest 2026-07-16）。
- **原生對話框在地化宣告**：資料夾選擇器等 macOS 原生對話框現為固定英文（app 未宣告在地化語言）——於 Info.plist 宣告 zh-Hant 與 en，使其跟隨系統語言——使用者實測後補充（2026-07-16）。
- 更新 tray-status-menu 規格的「面板樣式（macOS）」需求：專案區呈現改為 tab 條並明文原地切換語意、分區卡片化、進度條階段深淺、開啟時無預設焦點。

## Non-Goals

- 不擴充面板資訊量：不做 CodexBar 的 per-tab 狀態小條（需跨專案資料，快照無此欄）、用量統計等新資料；TraySnapshot 結構與主視窗推送管線不動。
- 不動原生選單：非 macOS 平台的正常路徑與 macOS 面板建立失敗的後備維持現狀（含複製、slug 化、分流、溢出摺疊）。
- 不動 Rust 面板視窗行為（vibrancy 材質除外——實測裁決換 HudWindow，見 What Changes）：NSPanel 轉換、貼齊定位、失焦收合、高度自適應全部維持；不在 Rust 端處理焦點問題。面板寬度預設維持現值，僅當卡片內距與 tab 條於真實視窗實測後確認過擠，才同步調整 Rust 與前端兩處的寬度常數。
- 不引入多色相配色：守單一 teal 色相原則（深淺表達生命週期推進），不追隨 CodexBar 的多色。
- 不動 crates/（speclink-core、speclink-cli）：無 CLI 子指令、旗標、輸出或 exit code 變更，無回歸對照影響。
- 不改 .speclink.yaml 與 openspec/config.yaml：無新設定欄位。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `tray-status-menu`: 「面板樣式（macOS）」需求修改——專案區由垂直列改為橫向 tab 條（首字母 avatar、作用中實心主色、可捲動、點擊原地切換不喚主視窗）；分區以卡片容器呈現；進度條依階段套 teal 深淺；面板開啟時焦點不得自動落於任何互動元素。

## Impact

- Affected specs: `tray-status-menu`（修改「面板樣式（macOS）」需求）
- Affected code:
  - Modified: apps/desktop/src/panel/TrayPanel.tsx、apps/desktop/src/__tests__/trayPanel.test.tsx、packages/ui/src/stage.ts、packages/ui/src/components/KanbanBoard.tsx（進度條色階抽升為共用匯出，看板改讀共用來源、視覺零變化）、apps/desktop/src-tauri/src/panel.rs（僅 vibrancy 材質一行：Menu → HudWindow）、apps/desktop/src/tray.ts（面板動作接線新增 add-project）、apps/desktop/src/panel/main.tsx（add-project 回呼接線）、apps/desktop/src/i18n/messages.ts（「加入專案」鍵，zh-TW 與 en）、apps/desktop/src/__tests__/tray.test.ts（add-project 接線測試）
  - Modified（條件式）: apps/desktop/src-tauri/src/panel.rs 與 apps/desktop/src/panel/main.tsx——僅當寬度實測放寬時同步兩處面板寬度常數；apps/desktop/src/panel/main.tsx 另於 focus 修復主手段不足時加入後備（視窗 focus 時 blur）
  - New: apps/desktop/src-tauri/Info.plist（在地化宣告：CFBundleLocalizations 含 zh-Hant 與 en、CFBundleAllowMixedLocalizations——Tauri 建置時自動合併進 bundle）
  - Removed: (none)
- 影響 crate：不動 speclink-core／speclink-cli；範圍限桌面 app 前端呈現層（apps/desktop、packages/ui）。
- 相容性影響：無——CLI 人眼與 --json 輸出皆不變，回歸對照不受影響。
