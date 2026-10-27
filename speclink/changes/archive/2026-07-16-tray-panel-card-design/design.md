## Context

macOS 系統匣面板（tray-status-menu「面板樣式（macOS）」）目前是原生選單式的薄渲染：apps/desktop/src/panel/TrayPanel.tsx 以緊湊列＋hr 分隔線呈現 TraySnapshot，Rust 側（apps/desktop/src-tauri/src/panel.rs）負責 NSPanel 轉換、vibrancy 毛玻璃、貼齊定位與失焦收合。討論 tray-panel-card-design 裁定四項升級：專案 tab 條、分區卡片化、teal 主色加量、修復複製鈕自動 focus。

既有約束：
- 色彩原則「單一 teal 色相以深淺表達生命週期推進」——packages/ui/src/stage.ts 的 STAGE_BADGE 已是徽章配色的單一來源；進度條深淺階梯目前以私有 STAGE_STYLE 寫在 packages/ui/src/components/KanbanBoard.tsx。
- 面板是主視窗推送快照的薄渲染（design D5，tray-copy-and-panel-mode）：不自建資料查詢路徑、動作以 tray-panel-action 事件回流。
- 複製鈕自動 focus 的成因：panel.rs 的 show_and_make_key() 使面板成為 key window 後，WebKit 把焦點交給文件中第一個可 tab 元素——面板列本體皆為 div，唯一的 button 是第一顆複製鈕，於是出現 macOS 藍色 focus ring。

本變更全落在桌面 app 前端呈現層（apps/desktop 與 packages/ui），不動 crates/（speclink-core、speclink-cli）——無流程邏輯、無 CLI 輸出、無序列化與 git 互動。

## Goals / Non-Goals

**Goals:**

- 專案選擇改為 CodexBar 式橫向 tab 條，作用中專案以實心主色卡呈現，點擊原地切換（沿用既有 open-project 語意）。
- 生命週期與討論分區改為半透明圓角卡片容器疊在 vibrancy 上，分區標題放大、間距放寬。
- 進度條依階段套用與看板同源的 teal 深淺階梯；分區圖示上色；面板底可疊極淡 teal 漸層 wash。
- 面板開啟時焦點不落在任何互動元素（無藍色 focus ring）。

**Non-Goals:**

- 不擴充面板資訊量（per-tab 狀態小條、用量統計）；TraySnapshot 結構與推送管線不動。
- 不動原生選單（非 macOS 正常路徑＋macOS 失敗後備）。
- 不動 Rust 面板視窗行為（vibrancy、NSPanel、定位、失焦收合、高度自適應）；不在 Rust 端處理焦點。
- 不引入多色相配色；不動 crates/；不改 .speclink.yaml 與 openspec/config.yaml。
- 面板寬度預設維持現值——僅實測確認過擠時才同步調整 Rust 與前端兩處常數（見風險）。

## Decisions

### D1 專案區＝橫向 tab 條，active 實心主色卡

每個 tab 由「專案名首字母的圓角方塊 avatar」＋「專案名」組成；容器橫向排列、超寬時 overflow 捲動並隱藏捲軸。作用中專案的 tab 鋪實心 primary 底＋primary-foreground 文字（呼應桌面側欄選中態與 CodexBar 的選中卡）；非作用中 tab hover 鋪淡 primary 底。點擊 tab 發既有 open-project 動作——主視窗 openProjectAt 切換作用中專案、store 訂閱重推快照、面板內容原地更新，不喚主視窗。

- 替代案：垂直列＋淡 teal 底 pill（討論第 2 輪方案）——被 tab 條取代；chip 形態下實心 teal 不再與整列 hover 反白相撞，顧慮解除。
- 替代案：資料夾圖示做 tab icon——各 tab 視覺相同、喪失辨識度，首字母 avatar 勝出。同首字母撞名不做特殊處理（下方名稱可區分），保持簡單。
- 替代案：per-tab 進度小條（CodexBar 同款）——需跨專案資料，快照無此欄，屬資訊擴充，排除。

### D2 分區卡片化：一分區一卡，疊在 vibrancy 上

提案中／進行中／已就緒／討論／已轉出各分區包進一個半透明圓角卡片容器（以主題 token 的前景色低透明度為底，如 foreground 的 5% 級距，不寫死色值——深淺色模式都要能透出毛玻璃）；分區標題字級放大、圖示以 primary 上色；卡片間以間距分隔，移除現行 hr。討論空態沿用既有「討論 {n}」文案呈現於細卡。

- 替代案：每列一卡——320px 寬度下臃腫、資訊密度掉太多，排除（討論第 1 輪）。
- 替代案：維持 hr 分隔線僅調色——達不到「卡片設計感」的目標，排除。

### D3 色階單一來源：進度條深淺階梯抽升至 stage.ts

面板進度條需要與看板同款的階段深淺（提案中 50%、進行中 75%、已就緒 100% 的 primary 透明度階梯）。KanbanBoard.tsx 的私有 STAGE_STYLE 已含此階梯——把進度條色階（bar）與圖示色（iconCls）抽升為 packages/ui/src/stage.ts 的具名匯出（與 STAGE_BADGE 同模式、同註解慣例），KanbanBoard 改讀共用匯出，TrayPanel 直接引用。

- 替代案：TrayPanel 內重述同值——兩處 magic value 必然漂移；stage.ts 的 STAGE_BADGE 註解已明文「共用此單一來源，避免兩處配色分歧」，抽升是專案自身慣例的延續，非新抽象層。

### D4 focus 修復：複製鈕退出 tab 順序（前端解）

CopyButton 設 tabIndex=-1。面板是滑鼠驅動介面：列本體是 div 本就不可鍵盤操作，複製鈕退出 tab 順序與整體互動模型一致。若真實視窗驗證發現 WebKit 仍給其他元素焦點框，後備手段：面板視窗 focus 事件觸發時 blur 目前的 activeElement（panel/main.tsx 入口層，與既有事件監聽同處）。

- 替代案：Rust 端攔 initialFirstResponder——跨 tauri-nspanel 層 API 不確定、複雜度高一級，前端可解就不動 Rust。
- 替代案：CSS outline-none 隱藏 focus ring——焦點仍在按鈕上，Enter 會誤觸發複製，治標不治本，排除。

### D6 vibrancy 材質改 HudWindow

真實視窗實測修訂（2026-07-16）。首輪真實視窗驗證發現：Menu 材質於淺色模式近乎不透明、毛玻璃不可辨——不滿足規格「面板毛玻璃底 SHALL 可透出」。經使用者裁決換為透感最強的 HudWindow（panel.rs 的 apply_vibrancy 材質參數一行）。同輪一併修正：內容根容器補 13px 圓角裁切（與 apply_vibrancy 半徑一致）——漸層 wash 畫在 webview 方形範圍會於頂角留下畫出 vibrancy 圓角外的方形殘料。

- 替代案：維持 Menu——真 NSMenu 同款材質，但透感目標實測不成立，排除。
- 替代案：Sidebar——透感介於兩者之間，留作 HudWindow 對比不佳時的後備。
- 替代案：Popover——前一變更（tray-copy-and-panel-mode）實測比 Menu 更不透，早已排除。

### D7 tab 條尾端快速加入專案

使用者實測補充（ingest 2026-07-16）。tab 條尾端固定一個「加入專案」動作項（加號圖示的 tab 形態、hover 淡主色底；同為 div 非 button——守 D4 的焦點約束）。點擊發新的面板動作事件 kind「add-project」，主視窗接線層（tray.ts）走 openIn 路徑（與「開啟此變更」同款）：先喚起主視窗再轉呼 store 既有的資料夾選擇流程（openProjectViaDialog：開選擇器→選定走 openProjectAt、取消即無事）——不新建任何資料路徑。i18n 新增「加入專案」鍵（zh-TW 與 en 字典 key 集合相等的既有測試自動把關）。

實測修訂（2026-07-16）：首版直呼 openProjectViaDialog 不喚主視窗——主視窗位於另一桌面（Space）時，選擇器開在不可見的桌面、macOS 不切換過去，使用者按了沒反應。改走 openIn 先顯示主視窗：macOS 聚焦另一桌面的視窗會自動切換 Space，選擇器隨之於前景可見。

- 替代案：面板內嵌路徑輸入——原生選擇器已是主視窗同語意的既有流程，嵌輸入框過度設計，排除。
- 替代案：維持不喚主視窗、只把選擇器拉到目前桌面——tauri-plugin-dialog 無指定 Space 的把手，走不通。
- 風險：資料夾選擇器彈出時面板失焦自動收合——屬既有失焦行為的自然結果，選定後專案照常加入並成為作用中，不視為缺陷。

### D8 分區計數徽章與空狀態卡最小高度

使用者實測補充（ingest 2026-07-16）。分區標題右側顯示該分區項目計數，徽章沿用 stage.ts 的 STAGE_BADGE 單一來源（生命週期分區取對應階段、討論與已轉出分區沿用看板討論欄同款）——與桌面看板欄計數同語彙。空狀態卡（討論零筆、全無變更）給最小高度並將內容垂直置中，比例不塌陷；討論空狀態卡改以「分區標題＋計數 0」呈現，與非空時同構。

- 替代案：於 TraySnapshot 增設計數欄——計數可由既有清單長度導出，增欄違反「不擴充資訊」範圍決定，排除。
- 替代案：空分區完全隱藏——討論分區是常駐錨點（原生選單同構），隱藏會讓「目前沒有討論」不可見，排除。

### D9 原生對話框在地化宣告

使用者實測補充（2026-07-16）。加入專案的資料夾選擇器（NSOpenPanel）介面固定英文——AppKit 只以 app 於 Info.plist 宣告過的語言渲染系統對話框，Tauri 預設 bundle 未宣告任何在地化。修法：新增 apps/desktop/src-tauri/Info.plist 部分檔（Tauri v2 建置時自動與產生的 Info.plist 合併），宣告 CFBundleLocalizations（zh-Hant、en）與 CFBundleAllowMixedLocalizations——系統對話框隨系統語言呈現，app 自身 UI 文案（硬編碼繁中）不受影響。

- 替代案：於前端對話框 API 傳自訂按鈕文字——tauri-plugin-dialog 的 pick folder 無此參數，且逐處覆寫不如一次宣告，排除。
- 驗證面：bundle 內 Info.plist 可斷言（plutil 讀鍵值）；對話框實際語言屬 AppKit 行為，以實機核對收尾。

### D5 面板寬度遞延

預設維持現行寬度。tab 條與卡片內距於真實視窗實測後若確認過擠，才放寬——寬度常數在 panel.rs 與 panel/main.tsx 各有一份，放寬時兩處必須同步（單處改會造成視窗與內容量測不一致）。

## Implementation Contract

**可觀察行為**（macOS 面板開啟時）：

- 頂部為橫向專案 tab 條：每 tab 顯示首字母 avatar＋專案名；作用中專案 tab 實心 primary 底反白；tab 數量超寬時可橫向捲動且不顯示捲軸。點擊非作用中 tab 後，面板下方變更／討論內容原地切換為該專案，主視窗不被喚起、面板不收合。
- 生命週期與討論分區各自呈現為圓角卡片：卡片底為半透明（毛玻璃可透出）、分區標題含 primary 色圖示。無 hr 分隔線。
- 面板底為高透感毛玻璃（HudWindow 材質）：面板背後的內容經 blur 可辨；內容根容器以 13px 圓角裁切，與 vibrancy 圓角吻合、頂角無方形殘料。
- 有任務的變更列進度條填色依階段深淺：提案中最淺、進行中次之、已就緒最深（與看板欄位同階梯）。
- tab 條尾端有「加入專案」動作項：點擊先喚起主視窗（桌面切至其所在 Space）再開啟資料夾選擇器，選定即加入分頁並成為作用中專案、取消則分頁無變化（D7）。
- 各分區標題右側帶項目計數徽章（與看板欄計數同語彙）；討論零筆與全無變更的空狀態卡有最小高度且內容垂直置中（D8）。
- 面板開啟瞬間無任何元素帶焦點框；複製鈕不可經 Tab 鍵聚焦，點擊仍複製並顯示勾號回饋 1.2 秒（既有行為不變）。

**介面／資料形狀**：

- TraySnapshot 結構不變；TrayPanelProps 增一個 onAddProject 回呼；tray-panel-action 事件的 kind 集合為 open-project／open-change／open-discussion／open-app／add-project（新增最後一項，D7）——add-project 由主視窗接線層轉呼 store 既有資料夾選擇流程。
- i18n 新增「加入專案」鍵（zh-TW＋en，key 集合相等測試把關）。
- 分區標題計數徽章取 STAGE_BADGE 單一來源；空狀態卡帶最小高度且內容垂直置中（D8）。
- packages/ui/src/stage.ts 新增階段色階具名匯出（進度條填色與圖示色的 Record<Stage, string>），KanbanBoard 與 TrayPanel 共同引用；STAGE_BADGE 既有匯出不變。
- 無新 i18n 鍵；無新 IPC 指令；無新依賴。

**失敗模式**：無新增——複製失敗仍靜默；面板建立失敗仍退回原生選單；快照未達前仍為空窗短暫態。

**驗收準則**：

- npm test -w apps/desktop 綠：trayPanel.test.tsx 更新後涵蓋——tab 條呈現與 active 標記、點 tab 發 open-project、分區卡片容器存在、進度條依階段套色、CopyButton 的 tabIndex=-1、複製回呼與勾號回饋既有案例不回歸。
- npm test -w packages/ui 綠：stage.ts 匯出擴充後看板既有測試不回歸。
- 真實視窗驗證（jsdom 測不出視覺與焦點）：release build 開啟面板截圖確認——毛玻璃上可見卡片層次與 teal 色階、tab 條可捲動、開啟瞬間無藍色 focus ring；深淺色模式各驗一次。

**範圍邊界**：

- In scope：apps/desktop/src/panel/TrayPanel.tsx、apps/desktop/src/__tests__/trayPanel.test.tsx、packages/ui/src/stage.ts、packages/ui/src/components/KanbanBoard.tsx（僅改讀共用匯出）、apps/desktop/src-tauri/src/panel.rs（僅 vibrancy 材質參數，D6）、apps/desktop/src/tray.ts（僅 add-project 動作接線，D7）、apps/desktop/src/panel/main.tsx（僅 add-project 回呼接線，D7）、apps/desktop/src/i18n/messages.ts（僅「加入專案」鍵，D7）、apps/desktop/src/__tests__/tray.test.ts（add-project 接線測試）；寬度實測放寬時另含 panel.rs 與 panel/main.tsx 的寬度常數。
- Out of scope：tray.ts 的快照建構與其餘事件接線、panel.rs 其餘行為、原生選單、TraySnapshot 結構、crates/。

## Risks / Trade-offs

- [tabIndex=-1 後 WebKit 仍把焦點框給其他元素] → 後備已定：panel/main.tsx 於視窗 focus 時 blur activeElement；真實視窗驗證列為驗收任務，jsdom 不可信。
- [半透明卡片底在深色模式對比不足或蓋死毛玻璃] → 以主題 token 的透明度級距取色、不寫死色值；深淺兩模式各截圖驗證。
- [HudWindow 高透感下文字對比不足] → 卡片半透明底墊高內容對比；實測不佳退 Sidebar（D6 後備）。
- [320px 下 tab 條＋卡片內距過擠] → D5 遞延：實測後才放寬，且兩處寬度常數同步改，避免視窗與內容量測不一致。
- [抽升 stage.ts 色階動到看板] → KanbanBoard 僅改引用來源、值不變，packages/ui 測試須全綠；視覺零變化是驗收前提。
- [回歸對照] → 不動 crates/，CLI 人眼與 --json 輸出零變更，回歸對照（parity／color／twin）不受影響。
- [跨平台] → TrayPanel 僅 macOS 面板載入；原生選單路徑不動，Windows／Linux 零影響。
