## ADDED Requirements

### Requirement: remote 區段與模式解析

`.speclink.yaml` 的 remote 區段（欄位：url 選填——缺席時由環境變數 SPECLINK_STORE_URL 供給、含專案範疇；repo 選填為本 repo 在專案內的註冊名）存在時，CLI SHALL 以 remote 模式運作；不存在時 SHALL 以 fs 模式運作。remote 區段與 openspec/ 目錄並存時，remote 模式 SHALL 生效且 CLI SHALL 於 stderr 輸出一行並存警告。環境變數 SPECLINK_STORE_URL 存在時 SHALL 覆寫區段的 url。remote 區段存在但區段 url 與環境變數皆缺席時，CLI SHALL 以非 0 exit code 明確失敗並同時提示 remote.url 欄位與 SPECLINK_STORE_URL 兩種設定方式，SHALL NOT 靜默改以 fs 模式執行。

#### Scenario: 有 remote 區段即 remote 模式

- **WHEN** .speclink.yaml 含 remote 區段（url 指向團隊 server），執行 speclink list --json
- **THEN** 指令向區段 url 的契約端點發出請求（而非讀取本地 openspec/），輸出 JSON 形狀與 fs 模式一致

#### Scenario: 並存時 remote 勝出並警告

- **WHEN** .speclink.yaml 含 remote 區段且專案根同時有 openspec/ 目錄，執行 speclink list
- **THEN** 指令以 remote 模式執行，stderr 恰有一行警告指出兩者並存且 remote 生效

#### Scenario: 環境變數覆寫區段 url

- **WHEN** 設定 SPECLINK_STORE_URL 指向另一 server，執行 speclink list
- **THEN** 請求發往環境變數指定的 url

#### Scenario: url 兩處皆缺時明確失敗

- **WHEN** remote 區段僅含 repo 欄位、未設 SPECLINK_STORE_URL，執行 speclink list
- **THEN** exit code 非 0，stderr 訊息指出 url 缺失並同時提示 remote.url 欄位與 SPECLINK_STORE_URL 兩種設定方式，不以 fs 模式執行

### Requirement: 殘留連接檔的遷移警告

專案根存在 .speclink.remote.yaml 時，CLI SHALL 於 stderr 輸出一行遷移警告——指引將 url 與 repo 搬入 .speclink.yaml 的 remote 區段並刪除舊檔，並說明舊檔不參與模式判定——且 SHALL NOT 解析該檔內容；模式判定 SHALL 僅以 .speclink.yaml 為準。此警告 SHALL NOT 影響指令結果與 exit code。

#### Scenario: 殘留舊檔僅警告不生效

- **WHEN** 專案根含 .speclink.remote.yaml（url 指向某 server），.speclink.yaml 無 remote 區段且 openspec/ 目錄存在，執行 speclink list
- **THEN** 指令以 fs 模式讀取本地 openspec/ 執行，stderr 恰有一行遷移警告，exit code 為 0

## MODIFIED Requirements

### Requirement: remote 初始化與連接指令

speclink init --store remote --url <url> [--repo <name>] SHALL 執行 workspace init（指令檔 marker、技能、settings、gitignore）並將 url 與 repo 寫入 .speclink.yaml 的 remote 區段（檔案不存在時建立、既有欄位保留），且 SHALL NOT 建立 openspec/ 目錄樹、SHALL NOT 建立獨立連接檔。speclink link <url> [--repo <name>] SHALL 寫入或更新 remote 區段；speclink unlink SHALL 移除 remote 區段並保留檔內其他欄位。init 或 link 當下若已有可用憑證，CLI SHALL 立即向 server 查驗 repo 是否屬於專案並回報結果；無憑證時 SHALL 提示執行 speclink auth login。

#### Scenario: remote 初始化不建規格樹

- **WHEN** 於空目錄執行 speclink init --store remote --url https://team.example.com/speclink/projects/foo --repo backend
- **THEN** 生成 CLAUDE.md marker、技能目錄與含 remote 區段（url 與 repo 欄位如參數）的 .speclink.yaml，且不存在 openspec/ 目錄與 .speclink.remote.yaml；exit code 為 0

#### Scenario: link 保留既有欄位

- **WHEN** .speclink.yaml 已含 tools 清單，以有效憑證執行 speclink link https://team.example.com/speclink/projects/foo --repo backend 且 repo 在註冊表
- **THEN** remote 區段被寫入且 tools 清單原值保留；exit code 為 0

#### Scenario: link 時 repo 不在專案註冊表

- **WHEN** 已有可用憑證，執行 speclink link https://team.example.com/speclink/projects/foo --repo typo-name，而 server 註冊表無 typo-name
- **THEN** exit code 非 0，stderr 訊息指出 repo 不在專案內並列出可用的註冊名清單，remote 區段不被寫入

#### Scenario: unlink 移除連接

- **WHEN** 於 remote 模式專案執行 speclink unlink
- **THEN** .speclink.yaml 的 remote 區段被移除、檔內其他欄位保留，後續指令回到 fs 模式的行為

### Requirement: repo 身分攜帶與歸屬防呆

remote 模式下每個動詞 SHALL 自動攜帶 remote 區段的 repo 名；server 回應 change 歸屬不符時，CLI SHALL 以非 0 exit code 結束並輸出同時指出 change 歸屬 repo 與當前 repo 名的單行訊息。

#### Scenario: 跑錯 repo 被擋下

- **WHEN** 於 remote 區段 repo 欄位為 frontend 的專案執行 speclink claim add-rate-limit，而該 change 歸屬 backend
- **THEN** exit code 非 0，stderr 訊息同時含 backend 與 frontend 兩個名稱與改正指引

### Requirement: git remote 參考值的輔助警告

speclink link 與 speclink auth status 執行時，若 server 註冊表提供本 repo 的 git url 參考值且與本地 git remote 不一致，CLI SHALL 於 stderr 輸出一行輔助警告（提示可能在 fork 或鏡像上工作）；此警告 SHALL NOT 影響指令結果與 exit code（僅警告、不強制）。本地非 git 目錄或 server 未提供參考值時 SHALL 靜默略過此檢查。

#### Scenario: fork 上工作僅警告不阻擋

- **WHEN** 本地 git remote 指向 fork，而 server 註冊表的 git url 參考值為原始 repo，以有效憑證執行 speclink link 某專案 url --repo backend
- **THEN** remote 區段照常寫入、exit code 為 0，stderr 出現一行 fork／鏡像提示警告

#### Scenario: 無參考值時靜默

- **WHEN** server 註冊表未提供本 repo 的 git url 參考值，執行 speclink auth status
- **THEN** 不輸出任何 git remote 相關警告

## REMOVED Requirements

### Requirement: 連接檔與模式解析
