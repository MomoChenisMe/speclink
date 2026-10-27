## 1. 工作區與 Tauri 殼骨架

- [x] 1.1 撰寫 src-tauri 啟動 smoke 測試（Rust `#[cfg(test)]`）：斷言 Tauri app builder 可建構且能注入指向 openspec/ 專案根的 core context；此時未建立 app，測試應紅（編譯失敗或 panic）。驗證：`cargo test -p speclink-desktop-core` 該測試失敗。
- [x] 1.2 落實「桌面 app 落於 apps/desktop/，新增 JS 工作區」決定：建立 apps/desktop/（Tauri app，src-tauri/ 為新 Cargo workspace member，speclink-core 以 path 依賴引用）與 packages/ui/，並於 repo 根建立 npm workspaces 納管兩者。驗證：1.1 smoke 測試轉綠、`cargo build -p speclink-desktop` 成功、`npm install` 於根解析 workspace。
- [x] 1.3 重構骨架：抽出 core context 初始化為單一函式、確認 crates/ 既有 workspace members 未受影響。驗證：`cargo test --workspace` 全綠、既有 CLI 測試套件未變動。

## 2. Tauri command 唯讀查詢層（直嵌 core payload）

- [x] 2.1 撰寫 Tauri command 測試（紅）：針對 list changes、list specs、get document、status 四個 command，斷言其回傳結構與對應 core `--json` payload 同形狀（camelCase 欄位如 changeName、applyRequires、artifacts 存在且型別正確）。驗證：`cargo test -p speclink-desktop-core` 相關測試失敗。
- [x] 2.2 落實「Tauri 殼直嵌 speclink-core，而非 sidecar 呼叫 CLI」與「桌面透過內嵌 core 的既有 payload 供資料，不新增引擎邏輯」：實作上述四個 command 為對 speclink-core 既有 payload builder 的薄包裝，交付需求「桌面 app 直嵌引擎並以本地檔案為真相」的資料讀取路徑（不 spawn CLI、不移動文件真相）。驗證：2.1 測試轉綠，斷言與 `speclink list --json`／`speclink status --change X --json` 同形狀。
- [x] 2.3 撰寫非專案目錄測試（紅）後實作空狀態語意：於不含 speclink 標記的目錄，command 回傳明確的空狀態結果而非 panic。驗證：新增測試先紅後綠，`cargo test -p speclink-desktop-core`。

## 3. Tauri command 動詞操作層

- [x] 3.1 撰寫動詞 command 測試（紅）：validate、analyze、archive 各一，斷言成功資料與失敗訊息/語意對應 core（analyze 發現項嚴重度對應 `speclink analyze --json`、archive 前置未滿足時回報失敗且不標記歸檔）。驗證：`cargo test -p speclink-desktop-core` 動詞測試失敗。
- [x] 3.2 落實需求「桌面 app 提供動詞操作面」：實作三個動詞 command（validate/analyze/archive）為 core 的薄包裝，失敗時回傳 core 錯誤訊息不靜默吞掉。驗證：3.1 測試轉綠。
- [x] 3.3 對動詞 command 的參數處理套用 sharp-edges 稽核清單（change 名輸入邊界、不存在的 change、路徑穿越）。以 `speclink instructions --skill audit` 取清單逐項核對並補測試。驗證：稽核項對應測試綠。

## 4. 歸檔清單 SQLite 衍生快取

- [x] 4.1 撰寫快取測試（紅）：斷言歸檔清單自 SQLite 快取讀取且內容與歸檔目錄一致；快取檔不存在或 schema 版本不符時由歸檔目錄重建；active 清單不觸及快取。驗證：`cargo test -p speclink-desktop-core` 快取測試失敗。
- [x] 4.2 落實「SQLite 快取限縮於歸檔清單，active changes/specs 即時讀」與需求「歸檔清單經衍生快取加速且可重建」：實作帶 schema 版本欄位的 SQLite 索引快取，僅涵蓋歸檔 change 清單，提供刪除後重建路徑；active 讀取維持即時經 core。驗證：4.1 測試全綠。

## 5. 共用 React 元件庫與 adapter 介面（packages/ui）

- [x] 5.1 撰寫 adapter 介面與元件測試（紅，vitest）：定義以領域語彙（列出 change／列出 spec／取得文件／執行動詞）表述的 data adapter 介面，並為 change 看板、文件樹、文件檢視元件寫測試（給定假 adapter 資料應正確渲染、動詞按鈕應呼叫 adapter），元件尚未實作應紅。驗證：`npm test -w packages/ui` 失敗。
- [x] 5.2 落實「前端採 React + TypeScript，封裝為獨立共用元件庫」與需求「前端元件庫與資料源解耦」：實作 packages/ui 的 adapter 介面與三個元件，元件不引用 Tauri 專屬全域、僅透過注入 adapter 取資料。驗證：5.1 vitest 測試轉綠。

## 6. 桌面前端整合

- [x] 6.1 撰寫 core-backed adapter 測試（紅，vitest）：桌面前端提供以 Tauri invoke 呼叫第 2/3 組 command 的 adapter 實作，斷言其符合 packages/ui 的 adapter 介面。驗證：`npm test -w apps/desktop` 失敗。
- [x] 6.2 落實需求「桌面 app 呈現 change 與 spec 的清單與內容」：實作桌面前端注入 core-backed adapter，組裝 change 清單/看板、spec 清單、選定後的文件檢視三個視圖，並呈現需求「桌面 app 直嵌引擎並以本地檔案為真相」的非專案空狀態。驗證：6.1 測試轉綠、手動於回歸專案啟動確認三視圖與空狀態。

## 7. 打包與端到端驗證

- [x] 7.1 產出單一 Windows 可執行檔（Tauri bundle），交付「雙擊即跑、本地零依賴」的可觀察行為。驗證：`npm run tauri build`（或等效）產出 exe，於乾淨環境雙擊啟動成功。
- [x] 7.2 端到端與回歸驗證：以 fs 模式既有回歸專案為資料手動走查四項可觀察行為（清單、文件檢視、動詞、空狀態）；確認 CLI fs 模式輸出回歸對照（parity_suite／color_suite／twin harness）維持通過——本刀未觸碰 core 呈現應天然不變。驗證：手動走查通過、回歸套件全綠。

## 8. 前端樣式與狀態：Tailwind + shadcn/ui + Zustand

- [x] 8.1 落實「前端樣式與狀態：Tailwind + shadcn/ui + Zustand」的樣式基底：在 packages/ui 與 apps/desktop 接上 TailwindCSS（Vite 插件、content globs 含 packages/ui），並將 shadcn/ui 設計系統原語（至少 Button、Card）原始碼複製進 packages/ui/src/components/ui/，作為跨桌面/web 共用設計系統。交付：一個 shadcn Button 能以 Tailwind class 正確渲染。驗證：`npm test -w packages/ui` 新增的 Button 渲染測試綠、`npm run build -w apps/desktop` 成功且產出 Tailwind CSS。
- [x] 8.2 以 shadcn 原語與 Tailwind class 重構領域元件 ChangeBoard／DocumentTree／DocumentViewer，維持 props 純呈現與 adapter 解耦（元件不引用 Tauri 全域、不依賴 store）。交付：三元件外觀改用設計系統，既有行為（名稱、進度、動詞按鈕、選取、空狀態）不變。驗證：既有 `npm test -w packages/ui` 全數維持綠（作為重構回歸護欄）。
- [x] 8.3 於 apps/desktop 導入 Zustand store 管理 app 狀態（選取的 change/spec、載入的文件、動詞結果）；App 由 store 取狀態，共用元件仍經 props 取資料不依賴 store。交付：選取一個 change 會更新 store 的 selection 並觸發文件載入。驗證：新增 store 單元測試（先紅後綠，斷言 select→state 轉移）＋既有 `npm test -w apps/desktop` 全綠。
- [x] 8.4 移除手寫的 apps/desktop/src/styles.css，改由 Tailwind 提供樣式；重建前端與 release exe 並確認啟動。交付：styles.css 移除後 app 外觀由設計系統呈現、無殘留 class 失樣。驗證：`npm run build -w apps/desktop` 成功、`cargo build --release -p speclink-desktop` 成功、exe 啟動存活未崩潰。

## 9. 看板式生命週期佈局

- [x] 9.1 落實「看板式生命週期佈局」的階段派生：於 packages/ui 新增純函式 changeStage(change)，依 totalTasks/completedTasks 回傳 proposed／in-progress／ready，Archived 由歸檔清單另計。驗證：`npm test -w packages/ui` 新增的 changeStage 表格測試綠（0 tasks→proposed、部分→in-progress、全完成→ready）。
- [x] 9.2 以 shadcn 原語實作 KanbanBoard 與 ChangeCard（packages/ui）：欄位 Proposed／In Progress／Ready／Archived，卡片依 changeStage 歸欄、顯示進度與狀態，維持 props 純呈現與 adapter 解耦。驗證：`npm test -w packages/ui` 新增測試斷言各 change 落在正確欄位、卡片動詞按鈕回呼。
- [x] 9.3 實作 DetailDrawer 側滑抽屜（shadcn Sheet）：呈現選定 change 的 artifact DAG（status 的 artifacts 與 done/ready/blocked）、tasks 清單（解析 tasks.md 的 `- [ ]`／`- [x]` checkbox，唯讀）、與文件內容。含 tasks.md checkbox 解析純函式。驗證：`npm test -w packages/ui` 斷言 checkbox 解析（勾/未勾計數）與抽屜三區渲染。
- [x] 9.4 以 @dnd-kit 實作拖放，將卡片拖到 Archived 欄彈出 shadcn AlertDialog 確認後執行 archive 動詞（apps/desktop 接 DnD→確認→store.runVerb('archive')）；其他階段不接受拖放。驗證：`npm test -w apps/desktop` 斷言拖到 Archived 觸發確認流程並呼叫 archive、拖到其他欄不觸發。
- [x] 9.5 完成「看板式生命週期佈局：欄位分組 × 側滑抽屜 × 拖放歸檔」整合：App 換成看板佈局並接 Zustand（抽屜開關與選定 change、archived 清單、階段分組），重建前端與 release exe 並確認啟動。驗證：`npm test -w apps/desktop` 全綠、`npm run build` 與 `cargo build --release` 成功、exe 啟動存活。

## 10. Spectra 風清單佈局取代看板

- [x] 10.1 落實「Spectra 風清單佈局取代看板」的富文本基底：於 packages/ui 新增 Markdown 元件（react-markdown＋remark-gfm，樣式含標題/行內 code pill/GFM checkbox/表格）與 shadcn Tabs、Input 原語。驗證：`npm test -w packages/ui` 全綠、`npm run build -w apps/desktop` 產出富文本樣式。
- [x] 10.2 實作 ChangeListItem（packages/ui）：可展開卡片、卡內 shadcn Tabs（提案/設計/任務/規格），各分頁經注入 loader 懶載入 artifact 並以 Markdown 富文本渲染；維持 props 純呈現與 adapter 解耦。驗證：展開 change 顯示提案富文本、`npm test -w apps/desktop` 斷言展開載入 proposal.md。
- [x] 10.3 實作 ChangeList（packages/ui）：工具列含搜尋與「進行中／已封存」切換，清單渲染 ChangeListItem。驗證：`npm test -w apps/desktop` 斷言搜尋過濾清單。
- [x] 10.4 完成「Spectra 風清單佈局取代看板：可展開清單 × 卡內分頁 × 富文本渲染」整合：App 換成 Spectra 殼（頂欄專案名/開啟專案佔位 ＋ 左側欄 變更/規格/備忘/設定 ＋ 主清單），新增唯讀 command change_capabilities 供規格分頁列 capability，重建 exe 並以真實截圖驗證清單/展開/分頁/富文本。驗證：`cargo test -p speclink-desktop-core` 全綠、`cargo build --release` 成功、截圖確認 Spectra 風佈局。

## 11. 看板為主視圖與 Spectra 級詳情

- [x] 11.1 撰寫 Rust 測試（紅）後實作「看板為主視圖、Spectra 級詳情面板與細節功能補齊」的兩個新 command：change_meta（回傳 createdBy/createdWith/created，camelCase；不存在的 change 回 None）與 delete_change（僅刪 active change 目錄、路徑安全、不存在回 Err、刪除後 list 不再含該 change）。驗證：`cargo test -p speclink-desktop-core` 新測試先紅後綠。
- [x] 11.2 前端純函式 specDeltaCounts（packages/ui）：解析 delta spec 的 ADDED/MODIFIED/REMOVED/RENAMED Requirements 區段內 Requirement 數，回傳各操作計數。驗證：`npm test -w packages/ui` 表格測試綠。
- [x] 11.3 實作 RichDetailDrawer（packages/ui）：寬幅 Sheet，標頭（名稱＋複製鈕＋metadata 列＋進度條）、動作列（分析/驗證/封存/刪除）、Tabs 提案/設計/任務（n/m 徽章）/規格（+a ~m），各分頁 Markdown 懶載入。驗證：`npm test -w packages/ui` 斷言 metadata 列、分頁計數與刪除回呼。
- [x] 11.4 App 整合：主視圖回到 KanbanBoard（看板/清單切換，看板預設）、點卡片開 RichDetailDrawer、刪除經 AlertDialog 確認後呼叫 deleteChange 並刷新，adapter 補 changeMeta/deleteChange。重建 exe 並以真實截圖與點擊驗證看板、詳情分頁與刪除確認。驗證：`npm test -w apps/desktop` 全綠、`cargo build --release` 成功、截圖確認。

## 12. 自適應佈局與主色系

- [x] 12.1 看板高度自適應：看板容器與欄位填滿主區高度（h-full、欄 flex-1 自適應分配寬度、min-w 250px 之下才橫向捲動）、卡片於欄內縱向捲動；清單視圖維持整頁縱向捲動。驗證：既有前端測試全綠、真實截圖確認欄位填滿視窗高度且橫向捲軸不再浮於畫面中間。
- [x] 12.2 主色系定調 teal 青綠（連結語意、與 Spectra 紅區隔）：淺/深色主題的 primary/accent/ring token 換為 teal（oklch hue 192），markdown code pill 隨 primary 連動。驗證：`npm run build -w apps/desktop` 成功、截圖確認主色為 teal。

## 13. 看板中文化與卡片極簡化

- [x] 13.1 看板全面中文化：欄位標題改 提案中／進行中／已就緒，卡片移除階段徽章（欄位即階段、資訊重複）。驗證：`npm test -w packages/ui` 更新後全綠、截圖確認中文欄位。
- [x] 13.2 卡片極簡化：移除卡面 validate/analyze/archive 按鈕（動作歸詳情抽屜），僅「已就緒」階段卡片保留一顆封存按鈕；點卡片開詳情抽屜不變。驗證：`npm test -w packages/ui` 斷言 ready 卡有封存鈕、非 ready 卡無任何動詞鈕。
- [x] 13.3 封存欄移出看板：看板僅呈現 提案中／進行中／已就緒 三欄，封存清單歸清單視圖的已封存分頁；拖曳卡片時浮現「拖到此封存」落點區（放開觸發既有確認流程）。驗證：`npm test -w apps/desktop` 全綠、截圖確認三欄佈局與封存清單在清單視圖。
- [x] 13.4 卡片複製名稱鈕：hover 顯示於名稱旁，點擊複製 change 名至剪貼簿並短暫顯示勾號，不觸發開卡。驗證：`npm test -w packages/ui` 斷言複製呼叫 clipboard 且未觸發 onOpenChange。
- [x] 13.5 任務分頁 Spectra 式排版：GFM 任務清單 checkbox 懸掛縮排對齊（絕對定位、正常文字流不破壞行內 code）、完成項刪除線＋灰色（含 code pill 連動變灰）。驗證：重建後真實截圖確認分組標題、縮排與刪除線。

## 14. 封存入口與大螢幕自適應

- [x] 14.1 封存入口與 Spectra 式封存列：頂欄新增「已封存 N」按鈕直達清單視圖的已封存分頁；封存列升級為 日期＋名稱＋hover 複製完整封存名。驗證：`npm test -w apps/desktop` 斷言入口切換與封存列渲染。
- [x] 14.2 大螢幕比例自適應：看板欄位加最大寬度（max-w 360px，超寬螢幕不再無限拉伸）、詳情抽屜寬度於 2xl 斷點隨視窗縮放（46vw、上限 1100px）。驗證：前端測試全綠、超寬視窗真實截圖確認比例正常。

## 15. 詳情互動任務與版面精修

- [x] 15.1 撰寫 Rust 測試（紅）後實作「詳情互動任務與版面精修：任務勾選/排序回寫 tasks.md、封存獨立頁、彩色 delta 與活化看板」的兩個寫入 command：set_task_done（1-based 序數定位 checkbox 行、雙向勾選/取消、僅動該行）與 move_task（checkbox 行上下移動、群組標題不動、越界回 Err）。驗證：`cargo test -p speclink-desktop-core` 新測試先紅後綠。
- [x] 15.2 互動任務分頁（packages/ui）：parseTaskDoc（群組標題＋任務含序數）與 TaskList 元件（可勾選 checkbox、每列上下移動鈕），經注入回呼觸發寫入後重載。驗證：`npm test -w packages/ui` 斷言解析、勾選與移動回呼。
- [x] 15.3 詳情抽屜精修：寬度流動 max(720px,42vw)＋全螢幕切換鈕、分頁加 icon、規格 delta 計數上色（+綠 ~琥珀 -紅）、任務分頁換 TaskList 並於寫入後刷新清單計數。驗證：`npm test -w apps/desktop` 全綠、真實截圖確認。
- [x] 15.4 封存獨立頁與看板活化：移除看板/清單切換（看板即主視圖）、封存改獨立頁（搜尋＋列表）、看板欄位飾條/icon/計數徽章採單一 teal 色相以深淺表達階段推進（依使用者回饋自多色收斂、守主色系）、卡片 hover 浮起、進度條隨階段深淺。驗證：`npm test -w apps/desktop` 全綠、真實截圖確認活化看板與封存頁。
- [x] 15.5 修正拖曳裁切：拖曳視覺改用 DragOverlay 浮動複本（最上層渲染、不受欄位 overflow 裁切、不再撐出欄內捲軸），原卡片拖曳中變淡留位；hover 移除位移改邊框/陰影回饋（消除抖動）。驗證：`npm test -w packages/ui` 全綠、手動拖曳確認無裁切。
