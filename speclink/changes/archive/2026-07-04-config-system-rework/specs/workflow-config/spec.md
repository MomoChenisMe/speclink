## ADDED Requirements

### Requirement: 工作流政策的正典歸屬與四層解析順序
工作流政策欄位（locale、spec_locale、tdd、audit）的正典值 SHALL 儲存於 openspec/config.yaml（經儲存介面讀取）。有效值 SHALL 依下列順序解析，先命中者勝：SPECLINK_LOCALE／SPECLINK_SPEC_LOCALE／SPECLINK_TDD／SPECLINK_AUDIT 環境變數 ＞ .speclink.yaml 的同名舊鍵（相容層）＞ openspec/config.yaml ＞ 內建預設（locale 未設定＝English、tdd 與 audit＝false）。布林環境變數僅接受 true 或 false，其他值 SHALL 視為未設定並落到下一層。

#### Scenario: 正典值生效
- **WHEN** .speclink.yaml 不含政策鍵、openspec/config.yaml 設定 tdd: true，執行 speclink instructions tasks --change 某 change --json
- **THEN** payload 反映 tdd 開關為開啟（tasks 指引含 TDD 紀律內容），stderr 無 deprecation 警告

#### Scenario: 舊鍵相容層勝過正典值
- **WHEN** .speclink.yaml 設定 locale: tw 且 openspec/config.yaml 設定 locale: ja，執行 speclink instructions proposal --change 某 change --json
- **THEN** payload 的 locale 欄位為 Traditional Chinese (繁體中文)，且 stderr 出現一行 deprecation 警告

#### Scenario: 環境變數覆寫一切
- **WHEN** 設定環境變數 SPECLINK_TDD=false，而 .speclink.yaml 與 openspec/config.yaml 均設定 tdd: true，執行 speclink instructions tasks --change 某 change --json
- **THEN** payload 反映 tdd 開關為關閉

#### Scenario: 非法布林環境變數落到下一層
- **WHEN** 設定 SPECLINK_AUDIT=yes（非法值），openspec/config.yaml 設定 audit: true，執行任一讀取政策的指令
- **THEN** 有效 audit 值為 true（環境變數被忽略，不輸出錯誤）

### Requirement: 舊政策鍵的 deprecation 警告
當 .speclink.yaml 含有 locale、spec_locale、tdd、audit 任一鍵時，CLI SHALL 於每次指令執行時向 stderr 輸出恰一行警告，內容 SHALL 列出偵測到的鍵名並指引搬移至 openspec/config.yaml。stdout（含 --json 輸出）SHALL NOT 受警告影響。無舊鍵時 SHALL NOT 輸出警告。

#### Scenario: 含舊鍵時單行警告
- **WHEN** .speclink.yaml 含 tdd: true 與 audit: true，執行 speclink list --json
- **THEN** stderr 恰有一行警告且同時列出 tdd 與 audit 兩個鍵名，stdout 的 JSON 與無警告情境完全相同，exit code 為 0

#### Scenario: 無舊鍵時無警告
- **WHEN** .speclink.yaml 僅含 tools 與 spec_dir，執行 speclink list
- **THEN** stderr 無任何 deprecation 警告

### Requirement: init 範本的政策寫入位置
speclink init SHALL 將政策欄位的示例寫入 openspec/config.yaml 範本（locale、spec_locale、tdd、audit 的註解示例區），且 .speclink.yaml 範本 SHALL NOT 含任何政策鍵（僅 tools 與 spec_dir 相關內容）。

#### Scenario: 新專案初始化的範本內容
- **WHEN** 於空目錄執行 speclink init
- **THEN** 生成的 openspec/config.yaml 含 locale、spec_locale、tdd、audit 的註解示例；生成的 .speclink.yaml 不含此四鍵；exit code 為 0

#### Scenario: 既有專案不受範本變更影響
- **WHEN** 於已初始化且 .speclink.yaml 含舊政策鍵的專案執行 speclink update
- **THEN** 兩個設定檔內容不被改寫（update 不觸碰設定檔），僅 stderr 出現 deprecation 警告
