## ADDED Requirements

### Requirement: 系統匣樣式偏好
於 macOS，設定頁 SHALL 提供「系統匣樣式」偏好（「原生選單」／「面板」二選），預設 SHALL 為原生選單；切換 SHALL 即時對系統匣生效（無需重啟）並持久化於 app 本機。app 本機偏好缺此值或值非法時 SHALL 視為原生選單（向後相容：舊安裝升級後行為不變、不浮出錯誤）。非 macOS 平台設定頁 SHALL NOT 顯示此偏好，系統匣固定為原生選單。此偏好 SHALL NOT 寫入 .speclink.yaml 或 openspec/config.yaml，兩檔內容 SHALL NOT 因切換而改動。

#### Scenario: 切換即時生效並持久化
- **WHEN** 使用者於 macOS 設定頁將系統匣樣式由「原生選單」切為「面板」
- **THEN** 未重啟 app 的情況下點擊系統匣圖示即改為彈出面板，重啟 app 後仍為面板樣式，且 .speclink.yaml 與 openspec/config.yaml 內容未被此操作改動

#### Scenario: 舊安裝缺此偏好視為原生選單
- **WHEN** app 於 app 本機偏好不含系統匣樣式值的狀態下啟動
- **THEN** 偏好讀取成功、無錯誤浮出，系統匣以原生選單樣式運作

#### Scenario: 非 macOS 平台不顯示此偏好
- **WHEN** 使用者於 Windows 或 Linux 開啟設定頁
- **THEN** 設定頁不出現「系統匣樣式」偏好，系統匣維持原生選單
