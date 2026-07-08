## Summary

桌面 app 三項 UI 調整：視窗預設尺寸自 1100×720 加大為 1440×900 並於啟動時置中；側欄「設定」導覽項沉至底部；設定頁重構為三頁簽（config.yaml／.speclink.yaml／本機設定），為未來設定成長預留擴充空間。

## Motivation

目標使用者是桌面 app 的開發者／PO／PM，情境為日常開啟 app 與調整專案或本機設定。現況三個痛點：①視窗 1100×720 對看板（變更五欄＋討論群組）偏擠，且啟動位置不置中；②「設定」與內容導覽（變更／規格／已封存）在側欄混排，不符工具入口沉底的桌面慣例；③設定頁四張卡順序混亂——同屬 config.yaml 的內容拆成不相鄰兩卡、本機偏好夾在兩個專案檔之間，且單一捲軸沒有未來功能的擴充空間。源頭討論：desktop-window-and-settings-polish（兩輪收斂）。

## Proposed Solution

- **視窗預設**：邏輯尺寸 1440×900、啟動置中——tauri.conf.json 純設定變更，Rust 端無視窗覆寫、不動。
- **側欄**：「設定」導覽項固定於側欄底部（其上為彈性空間），變更／規格／已封存維持頂部原序與既有行為（徽章、切頁語意）。
- **設定頁三頁簽**（標籤檔名直出）：
  - **config.yaml** 簽（預設簽）：「專案說明」卡、「產出規則」卡（自原合併卡拆開、移除內層分頁，各自獨立編輯／取消／儲存）、「產出政策」卡（locale、spec_locale、tdd、audit，即原 openspec/config.yaml 卡更名）。
  - **.speclink.yaml** 簽：「AI 工具」卡（tools 多選與自訂工具唯讀膠囊，即原卡更名歸簽）。
  - **本機設定** 簽：「介面語言」卡；簽內註記「僅存於此裝置、不寫入版本庫」。
  - 各簽首行以 mono 小字註記對應檔案路徑；檔案解析失敗時該簽浮出既有橫幅、頁簽標籤加警示點、該簽表單停用。
  - 讀寫邏輯與寫入語意（鍵保留、清空移除鍵、雙重解析驗證、技能同步）全數不變——本刀僅重排與命名。
- **詞彙**：openspec/LANGUAGE.md 新增明文例外——檔名得作為設定頁頁簽標籤（與「工程詞不進使用者文案」原則的刻意抵觸，經討論裁定）。

不影響 crates（speclink-core／speclink-cli 皆不動）、無 CLI 指令與輸出變動、無設定欄位增減、無技能與注入區塊變動；相容性影響僅限桌面前端呈現。

## Non-Goals

- 視窗狀態記憶（記住上次大小與位置、僅首啟動置中）——僅設預設值，需要時另刀。
- 任何讀寫邏輯或寫入語意改動——寫入行為以既有規格為準，本刀不觸碰。
- Spectra 式設定子導航——頁簽已滿足成長性需求。
- packages/ui 新元件——沿用既有 Tabs／Card 原語。

## Alternatives Considered

- 單頁捲動＋「專案／此電腦」作用域群組標題——未來功能成長空間不足，討論階段否決。
- 人話頁簽標籤（專案設定／整合／此電腦）——使用者裁定檔名直出，開發者工具中檔案即最直觀的心智模型。
- 巢狀頁簽（頁簽內保留專案說明／產出規則分頁）——兩層頁簽混淆，改拆獨立卡。

## Impact

- Affected specs: desktop-app（側欄導覽結構、視窗預設）、desktop-config（設定頁組織與專案設定卡拆分）
- Affected code:
  - Modified:
    - apps/desktop/src-tauri/tauri.conf.json
    - apps/desktop/src/App.tsx
    - apps/desktop/src/views/SettingsView.tsx
    - apps/desktop/src/i18n/messages.ts
    - apps/desktop/src/__tests__/App.test.tsx
    - apps/desktop/src/__tests__/settingsView.test.tsx
    - openspec/LANGUAGE.md
  - New: （無）
  - Removed: （無）
