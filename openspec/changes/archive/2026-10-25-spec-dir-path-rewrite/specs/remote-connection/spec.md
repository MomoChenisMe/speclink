## MODIFIED Requirements

### Requirement: remote 初始化與連接指令

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

`speclink init --store remote --url <url> [--repo <name>]` SHALL 在寫入前取得至少一個明示的 Claude／Codex 選集：顯式 `--tools` SHALL 直接採用；互動終端缺少 `--tools` SHALL 於 stderr 詢問；非互動終端缺少 `--tools` SHALL 在零寫入狀態以非零 exit code 失敗並指引顯式值。選集有效時，Remote init SHALL 執行 Workspace init（`.speclink.yaml` tools、Skills、`.gitignore`）並將 URL 與 repo 寫入 `.speclink.yaml` 的 remote section（檔案不存在時建立、既有欄位保留），且 SHALL NOT 建立 `speclink/` 目錄樹、獨立連接檔或任何指令檔。

`speclink link <url> [--repo <name>]` SHALL 維持既有行為，只寫入或更新 remote section並保留 tools 及其他欄位；`speclink unlink` SHALL 移除 remote section並保留檔內其他欄位。init 或 link 當下若已有可用憑證，CLI SHALL 立即向 Server 查驗 repo 是否屬於專案並回報結果；無憑證時 SHALL 提示執行 auth login。此變更 SHALL NOT 為 init／link 新增 stdin payload 或 `--json` 介面；顯式 `--tools` 的既有成功 stdout、stderr、exit code 與 `--no-color` 行為 SHALL 維持基線。

#### Scenario: Remote init 顯式選擇 Claude

- **WHEN** 於空目錄執行 Remote Store init，提供有效 project URL、repo 與 `--tools claude`
- **THEN** 生成 Claude Skills、`.gitignore` 與含 `tools: [claude]`、remote URL／repo 的 `.speclink.yaml`，不存在 `speclink/`、獨立連接檔與 `CLAUDE.md`，exit code 為 0

#### Scenario: Remote init 互動選擇 Codex

- **WHEN** stdin 為互動終端，執行 Remote Store init 且未提供 `--tools`，使用者選取 Codex並不選 Claude
- **THEN** prompt 寫入 stderr，生成 Codex Skills 而無 `AGENTS.md`，`.speclink.yaml` built-in tools 僅含 `codex`，成功摘要寫入 stdout且 exit code 為 0

#### Scenario: Remote init 非互動缺少 tools 被拒

- **WHEN** stdin 非互動，執行 Remote Store init 且未提供 `--tools`
- **THEN** exit code 非 0、stdout 為空、stderr 指引 `--tools claude`／`codex`／`claude,codex`，且 `.speclink.yaml`、`.gitignore`、Skills 與 `speclink/` 均未建立

#### Scenario: link 保留既有 tools 與其他欄位

- **WHEN** `.speclink.yaml` 已含 built-in tools、custom descriptor 與未知頂層鍵，以有效憑證執行 link 至 project URL 與已註冊 repo
- **THEN** remote section 被寫入，tools 清單與未知鍵原值保留，未觸發工具詢問或受管產物同步，exit code 為 0

#### Scenario: link 時 repo 不在專案註冊表

- **WHEN** 已有可用憑證，執行 link 並指定 Server 註冊表不存在的 repo
- **THEN** exit code 非 0，stderr 訊息指出 repo 不在專案內並列出可用註冊名，remote section 不被寫入

#### Scenario: unlink 移除連接

- **WHEN** 於 Remote 模式專案執行 unlink
- **THEN** `.speclink.yaml` 的 remote section 被移除、tools 與其他欄位保留，後續指令回到 filesystem 模式行為

### Requirement: remote 區段與模式解析

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

`.speclink.yaml` 的 remote 區段（欄位：url 選填——缺席時由環境變數 SPECLINK_STORE_URL 供給、含專案範疇；repo 選填為本 repo 在專案內的註冊名）存在時，CLI SHALL 以 remote 模式運作；不存在時 SHALL 以 fs 模式運作。remote 區段與 speclink/ 目錄並存時，remote 模式 SHALL 生效且 CLI SHALL 於 stderr 輸出一行並存警告。環境變數 SPECLINK_STORE_URL 存在時 SHALL 覆寫區段的 url。remote 區段存在但區段 url 與環境變數皆缺席時，CLI SHALL 以非 0 exit code 明確失敗並同時提示 remote.url 欄位與 SPECLINK_STORE_URL 兩種設定方式，SHALL NOT 靜默改以 fs 模式執行。

`.speclink.yaml` 檔案存在但無法解析（YAML 語法錯誤或型別不符）時，模式判定 SHALL fail-closed：任何依賴模式判定或應用層設定的指令 SHALL 以非零 exit code 失敗，stderr SHALL 指出 .speclink.yaml 與解析原因；SHALL NOT 視為無 remote 區段而以 fs 模式執行，SHALL NOT 發出任何遠端請求。檔案不存在時 SHALL 以 fs 模式運作。此 fail-closed 行為為刻意設計。

#### Scenario: 有 remote 區段即 remote 模式

- **WHEN** .speclink.yaml 含 remote 區段（url 指向團隊 server），執行 speclink list --json
- **THEN** 指令向區段 url 的契約端點發出請求（而非讀取本地 speclink/），輸出 JSON 形狀與 fs 模式一致

#### Scenario: 並存時 remote 勝出並警告

- **WHEN** .speclink.yaml 含 remote 區段且專案根同時有 speclink/ 目錄，執行 speclink list
- **THEN** 指令以 remote 模式執行，stderr 恰有一行警告指出兩者並存且 remote 生效

#### Scenario: 環境變數覆寫區段 url

- **WHEN** 設定 SPECLINK_STORE_URL 指向另一 server，執行 speclink list
- **THEN** 請求發往環境變數指定的 url

#### Scenario: url 兩處皆缺時明確失敗

- **WHEN** remote 區段僅含 repo 欄位、未設 SPECLINK_STORE_URL，執行 speclink list
- **THEN** exit code 非 0，stderr 訊息指出 url 缺失並同時提示 remote.url 欄位與 SPECLINK_STORE_URL 兩種設定方式，不以 fs 模式執行

#### Scenario: 壞 .speclink.yaml 不落入 fs 模式

- **WHEN** .speclink.yaml 含 YAML 語法錯誤且專案根有 speclink/ 目錄，執行 speclink list
- **THEN** exit code 非 0，stderr 指出 .speclink.yaml 與解析原因，指令不讀取本地 speclink/、不發出任何遠端請求

### Requirement: 殘留連接檔的遷移警告

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

專案根存在 .speclink.remote.yaml 時，CLI SHALL 於 stderr 輸出一行遷移警告——指引將 url 與 repo 搬入 .speclink.yaml 的 remote 區段並刪除舊檔，並說明舊檔不參與模式判定——且 SHALL NOT 解析該檔內容；模式判定 SHALL 僅以 .speclink.yaml 為準。此警告 SHALL NOT 影響指令結果與 exit code。

#### Scenario: 殘留舊檔僅警告不生效

- **WHEN** 專案根含 .speclink.remote.yaml（url 指向某 server），.speclink.yaml 無 remote 區段且 speclink/ 目錄存在，執行 speclink list
- **THEN** 指令以 fs 模式讀取本地 speclink/ 執行，stderr 恰有一行遷移警告，exit code 為 0

### Requirement: Remote Workspace bootstrap 跨入口一致性

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

CLI Remote init 與 Desktop Remote checkout 綁定 SHALL 使用相同的正典生成來源，對相同 Claude／Codex 選集 SHALL 產生語意相同的 `.speclink.yaml.tools`、Skills 與 `AGENTS.md`／`CLAUDE.md` Speclink 區塊。兩個入口 SHALL 使用 Remote 指令區塊措辭、SHALL NOT 建立本機 `speclink/`，且 SHALL 保留 remote section、custom descriptor、未知設定及指令區塊外的使用者內容。Desktop 綁定既有相符 marker 時 SHALL 重新同步受管產物；同步完全成功前 SHALL NOT 開始 Remote handshake。

#### Scenario: CLI 與 Desktop 的 Codex bootstrap 同構

- **WHEN** 對兩個等價的新 Git checkout，分別以 CLI Remote init 與 Desktop checkout 綁定選取 Codex及相同 project／repo
- **THEN** 兩個 checkout 的 built-in tools 都僅含 `codex`、Codex Skills 與 `AGENTS.md` Remote Speclink 區塊使用相同正典內容、remote binding 語意相同，且兩者均不存在 `speclink/`

#### Scenario: Desktop 補齊既有 Remote Workspace

- **WHEN** 既有 checkout 的 remote section 與所選 scope 相符、built-in tools 為 `[claude,codex]`，但任一 Skills 或指令區塊缺少或過期
- **THEN** Desktop 以相同正典來源補齊兩個工具的受管產物，保留 remote／custom／使用者內容，成功後才開始 handshake

#### Scenario: Remote bootstrap 不改變 Server 內容

- **WHEN** CLI 或 Desktop 只執行本機 Remote Workspace bootstrap
- **THEN** 除既有 init credential 驗證或 Desktop 後續 handshake 外，不上傳、合併或修改 Server 規格內容，本機不建立 `speclink/`
