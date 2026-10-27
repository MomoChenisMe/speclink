## MODIFIED Requirements

### Requirement: remote 區段與模式解析
<!-- BEFORE: .speclink.yaml 存在但解析失敗時，靜默視為無 remote 區段而落入 fs 模式 -->
`.speclink.yaml` 的 remote 區段（欄位：url 選填——缺席時由環境變數 SPECLINK_STORE_URL 供給、含專案範疇；repo 選填為本 repo 在專案內的註冊名）存在時，CLI SHALL 以 remote 模式運作；不存在時 SHALL 以 fs 模式運作。remote 區段與 openspec/ 目錄並存時，remote 模式 SHALL 生效且 CLI SHALL 於 stderr 輸出一行並存警告。環境變數 SPECLINK_STORE_URL 存在時 SHALL 覆寫區段的 url。remote 區段存在但區段 url 與環境變數皆缺席時，CLI SHALL 以非 0 exit code 明確失敗並同時提示 remote.url 欄位與 SPECLINK_STORE_URL 兩種設定方式，SHALL NOT 靜默改以 fs 模式執行。

`.speclink.yaml` 檔案存在但無法解析（YAML 語法錯誤或型別不符）時，模式判定 SHALL fail-closed：任何依賴模式判定或應用層設定的指令 SHALL 以非零 exit code 失敗，stderr SHALL 指出 .speclink.yaml 與解析原因；SHALL NOT 視為無 remote 區段而以 fs 模式執行，SHALL NOT 發出任何遠端請求。檔案不存在時 SHALL 以 fs 模式運作。此 fail-closed 行為屬對 Spectra 2.3.1 的刻意分歧（Spectra 於壞檔時靜默退回預設）。

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

#### Scenario: 壞 .speclink.yaml 不落入 fs 模式

- **WHEN** .speclink.yaml 含 YAML 語法錯誤且專案根有 openspec/ 目錄，執行 speclink list
- **THEN** exit code 非 0，stderr 指出 .speclink.yaml 與解析原因，指令不讀取本地 openspec/、不發出任何遠端請求
