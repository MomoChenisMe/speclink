## ADDED Requirements

### Requirement: 桌面 app 直嵌引擎並以本地檔案為真相
桌面 app SHALL 以 Tauri 殼直接內嵌 speclink-core（非 spawn CLI 子進程），於本地 openspec/ 專案根運作，且 SHALL NOT 改變 fs 模式下 markdown 檔案的真相地位——所有呈現資料由內嵌 core 讀取檔案取得，app 不將任何 change/spec 文件的真相移出檔案系統。

#### Scenario: 於 fs 專案根啟動並讀取本地文件
- **WHEN** 使用者於含 openspec/ 的專案根啟動桌面 app
- **THEN** app 經內嵌 core 讀取本地 markdown 顯示 change 與 spec，未 spawn speclink CLI 子進程，且未寫入或搬移任何文件真相

#### Scenario: 非專案目錄啟動顯示空狀態而非崩潰
- **WHEN** 使用者於不含 speclink 專案標記的目錄啟動桌面 app
- **THEN** app 顯示明確的「非 speclink 專案」空狀態，不崩潰、不產生錯誤彈窗堆疊

### Requirement: 桌面 app 呈現 change 與 spec 的清單與內容
桌面 app SHALL 呈現當前專案的 change 清單（含每個 change 的 proposal 與 tasks 完成度狀態）與 spec 清單，並 SHALL 於使用者選定任一 change 或 spec 時顯示其對應 markdown 文件內容（change 的 proposal/design/tasks、spec 的 spec.md）。清單與狀態資料的欄位與值 SHALL 與對應 CLI `--json` 輸出一致。

#### Scenario: 顯示 change 清單與狀態
- **WHEN** app 於含多個 active change 的專案啟動
- **THEN** 每個 change 以其名稱與 proposal/tasks 狀態呈現，欄位與值對應 speclink list 與 speclink status 的 --json 輸出

#### Scenario: 選定 change 顯示其文件內容
- **WHEN** 使用者在清單中選定一個 change
- **THEN** app 顯示該 change 的 proposal 內容，並可切換檢視其 design 與 tasks（若存在）

#### Scenario: 選定 spec 顯示其正典內容
- **WHEN** 使用者選定一個 spec
- **THEN** app 顯示該 spec 的正典 spec.md 內容

### Requirement: 桌面 app 提供動詞操作面
桌面 app SHALL 讓使用者對選定 change 執行 status、validate、analyze、archive，並對專案執行 list、show，全部經內嵌 core 執行。動詞的可觀察結果（成功資料、失敗訊息與失敗語意）SHALL 與對應 CLI 指令一致；失敗時 app SHALL 於 UI 呈現 core 的錯誤訊息，SHALL NOT 靜默吞掉失敗。

#### Scenario: 執行 validate 呈現結果
- **WHEN** 使用者對一個 change 觸發 validate
- **THEN** app 呈現與 speclink validate 一致的通過或失敗結果，失敗時顯示其錯誤訊息

#### Scenario: 執行 analyze 呈現發現項
- **WHEN** 使用者對一個 change 觸發 analyze
- **THEN** app 呈現 analyze 的發現項，其嚴重度與訊息對應 speclink analyze 的 --json 輸出

#### Scenario: archive 前置未滿足時失敗顯示
- **WHEN** 使用者對尚未滿足歸檔前置的 change 觸發 archive
- **THEN** app 呈現 core 回報的失敗訊息，不將該 change 標為已歸檔

### Requirement: 歸檔清單經衍生快取加速且可重建
桌面 app SHALL 以本地 SQLite 索引快取歸檔（archived）change 的清單資料以加速呈現；此快取 SHALL 為衍生資料——可刪除後由檔案系統真相重建，且 SHALL 帶 schema 版本標記。active change 與 spec 的清單 SHALL NOT 依賴此快取，一律即時經 core 讀取檔案。快取與檔案真相不一致時，檔案真相 SHALL 為準。

#### Scenario: 歸檔清單由快取呈現
- **WHEN** app 呈現歸檔 change 清單
- **THEN** 清單資料自 SQLite 快取讀取，內容與檔案系統中的歸檔目錄一致

#### Scenario: 快取遺失時重建
- **WHEN** SQLite 快取檔不存在或版本不符
- **THEN** app 由歸檔目錄重建快取後呈現清單，不崩潰

#### Scenario: active 清單不經快取
- **WHEN** app 呈現 active change 清單
- **THEN** 清單即時經 core 讀取檔案取得，不讀取 SQLite 快取

### Requirement: 前端元件庫與資料源解耦
桌面 app 的呈現元件（change 看板、文件樹、文件檢視）SHALL 封裝為與資料源解耦的共用元件庫：元件經注入的 data adapter 取得資料，adapter 介面 SHALL 以領域語彙（列出 change、列出 spec、取得文件、執行動詞）定義，SHALL NOT 直接依賴 Tauri 專屬全域。桌面 app SHALL 提供以內嵌 core 為後端的 adapter 實作。

#### Scenario: 桌面注入 core adapter 渲染看板
- **WHEN** 桌面 app 以其 core-backed adapter 提供 change 列表
- **THEN** 共用看板元件據此渲染，元件本身未引用任何 Tauri 專屬全域

#### Scenario: adapter 介面以領域語彙定義
- **WHEN** 檢視 adapter 介面定義
- **THEN** 其方法以 change/spec/document/verb 領域語彙表述，使非 Tauri 後端（如後續 HTTP 後端）可提供另一實作而元件不變
