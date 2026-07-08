## Context

現況：視窗設定僅 tauri.conf.json 一處（1100×720、未置中），src-tauri 的 Rust 端無程式化視窗建立。側欄為 App.tsx 中的 flex 縱排，變更／規格／已封存／設定四項連排於頂部。設定頁 SettingsView.tsx 為單欄四卡：專案設定卡（config.yaml 的專案說明＋產出規則，卡內含兩分頁與卡層級共享編輯態）、UI 語言卡、.speclink.yaml 卡、openspec/config.yaml 政策卡——同檔內容不相鄰、本機偏好夾在專案檔之間。

前端體系：React＋Tailwind v4＋packages/ui 原語（Tabs、Card、Checkbox、NativeSelect）＋Zustand；UI 文案經 i18n 字典（zh-TW 與 en 鍵集合相等，有測試把關）。寫入層：adapter 的 writeWorkflowContext／writeWorkflowRules／writeWorkflowConfig／writeAppTools 各自獨立呼叫。

約束：不動 crates（speclink-core／speclink-cli）；寫入語意受既有 desktop-config 規格保護，本刀不得改變任何檔案寫入效果；GUI 互動與視窗行為 jsdom 測不出，須以真實視窗驗證（本 repo 既定慣例）。

## Goals / Non-Goals

**Goals:**

- 視窗預設 1440×900、啟動置中。
- 「設定」導覽項沉至側欄底部，其餘導覽項行為不變。
- 設定頁三頁簽組織（config.yaml／.speclink.yaml／本機設定），專案說明與產出規則拆為獨立卡並各自編輯，解析失敗以簽級警示浮出。

**Non-Goals:**

- 視窗狀態記憶（記住上次大小位置）。
- 任何讀寫邏輯、寫入語意、IPC／adapter 介面改動。
- Spectra 式設定子導航；packages/ui 新元件。

## Decisions

**D1：頁簽標籤檔名直出，且檔名不進 i18n 字典。**
三頁簽標籤為 config.yaml、.speclink.yaml（字面常數，兩語系相同，屬技術術語保留英文）與「本機設定」（i18n 鍵，雙語各譯）。卡片標題（專案說明、產出規則、產出政策、AI 工具、介面語言）全數經 i18n 字典。
替代：人話頁簽標籤——使用者於討論裁定檔名直出，否決；檔名也塞進字典——無翻譯需求、徒增雙語重複鍵，否決。此裁定與 LANGUAGE.md「工程詞不進使用者文案」原則刻意抵觸，隨本刀寫入例外條目。

**D2：專案說明與產出規則拆為獨立卡、各持編輯態。**
每卡自持 editing 旗標與草稿，編輯／取消／儲存互不影響；儲存各自呼叫既有的 writeWorkflowContext 或 writeWorkflowRules。原合併卡「一次儲存兩分頁」的呈現行為改為各卡各存，但檔案寫入效果（僅代換目標鍵、清空移除鍵、雙重解析驗證）逐場景維持既有規格。
替代：保留合併卡與內層分頁——頁簽內再包分頁兩層混淆，否決；全頁單一編輯模式——改動半徑大且與既有卡級模式不連續，否決。

**D3：解析失敗簽級呈現。**
config.yaml 簽掛工作流層解析錯誤、.speclink.yaml 簽掛應用層解析錯誤：失敗時該簽首部浮出既有橫幅、該簽全部表單／編輯鈕停用，且頁簽標籤加警示點（未切至該簽也可見）；本機設定簽不掛任何解析錯誤。
替代：僅維持卡內橫幅——使用者停在其他簽時完全看不到失敗，違反「失敗浮出」精神，否決。

**D4：視窗預設純設定變更。**
tauri.conf.json 的視窗項設 width 1440、height 900、center true；不掛 window-state 外掛、不寫 Rust。
替代：tauri-plugin-window-state 記憶上次狀態——超出需求（Non-Goal），否決。

**D5：側欄以彈性空間把設定推至底部。**
側欄容器既為縱向 flex，於設定項前以彈性空間（或等效的自動邊距）將其推至底部；不加分隔線等新視覺元素，導覽項樣式與行為（高亮、切頁語意）不變。
替代：底部獨立分組＋分隔線——未被要求的視覺元素，YAGNI，否決。

## Implementation Contract

**行為**：①啟動後視窗邏輯尺寸 1440×900 且於主螢幕置中；②側欄「設定」貼底、其上為空白彈性區，變更／規格／已封存維持頂部原序與徽章行為；③設定頁預設落在 config.yaml 簽，三簽卡片歸屬如 proposal 所列，各簽首行 mono 檔案路徑註記（本機設定簽為「僅存於此裝置」說明）；④專案說明卡與產出規則卡可各自進入編輯——一卡編輯中另一卡唯讀可用，取消僅還原本卡；⑤任一層設定檔解析失敗時對應簽的標籤帶警示點、簽內橫幅浮出、表單停用；⑥所有檔案寫入效果與既有 desktop-config 規格場景逐字不變。

**介面／資料形狀**：無新 IPC command、無 adapter 簽章變動；i18n 字典新增頁簽與卡標題鍵（zh-TW 與 en 鍵集合維持相等）。

**失敗模式**：解析失敗簽停用編輯（既有行為的簽級延伸）；寫入失敗維持既有卡內單行錯誤訊息、表單值不遺失。

**驗收判準**：
- npm test -w apps/desktop：App 測試斷言側欄結構（設定項沉底）；settingsView 測試群斷言三頁簽標籤與預設簽、各簽卡片歸屬、兩卡獨立編輯態、簽級警示點與停用、既有寫入語意測試全數保留通過；messages 鍵集合相等測試通過。
- 真實視窗驗證（release exe）：量測啟動視窗尺寸與螢幕置中位置；實點三頁簽切換、進出編輯態；截圖確認警示點與 mono 註記呈現。
- 不涉及 CLI，parity／color 回歸對照無關。

**範圍邊界**：in——tauri.conf.json 視窗項、App.tsx 側欄佈局、SettingsView.tsx 重構、i18n 字典鍵、兩個測試檔、LANGUAGE.md 例外條目。out——Non-Goals 全數（視窗記憶、寫入語意、子導航、新元件）。

## Risks / Trade-offs

- **jsdom 測不出視窗與頁簽實際互動**：置中、尺寸、點擊切簽的真實行為以 release exe 真實視窗驗證補位（repo 既定慣例：操作前確認使用者未使用螢幕、文字一律剪貼簿貼上）。
- **拆卡重構牽動既有 settingsView 測試**：TDD 先改測試——寫入語意場景逐案保留（僅呈現層選擇器更新），防止重構偷改寫入行為。
- **1440×900 對小螢幕（如 1366×768）會超出且 Tauri 不自動夾限**：使用者已確認目標螢幕吃得下；若日後回報，降尺寸為一行設定變更。
- **多螢幕／DPI 縮放下的置中**：交由 Tauri center 語意處理，驗證於主螢幕進行，不自行實作座標計算（避免過度設計）。
