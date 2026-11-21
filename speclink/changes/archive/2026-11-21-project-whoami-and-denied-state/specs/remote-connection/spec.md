## MODIFIED Requirements

### Requirement: remote 初始化與連接指令
<!-- BEFORE: 專案註冊多個 repo 時，已登入的 link 與 init 不論有沒有 --repo 都收到 Server 的 409（訊息要求指定 X-Speclink-Repo），查驗無法完成 -->

`speclink init --store remote --url <url> [--repo <name>]` SHALL 在寫入前取得至少一個明示的 Claude／Codex 選集：顯式 `--tools` SHALL 直接採用；互動終端缺少 `--tools` SHALL 於 stderr 詢問；非互動終端缺少 `--tools` SHALL 在零寫入狀態以非零 exit code 失敗並指引顯式值。選集有效時，Remote init SHALL 執行 Workspace init（`.speclink.yaml` tools、Skills、`.gitignore`）並將 URL 與 repo 寫入 `.speclink.yaml` 的 remote section（檔案不存在時建立、既有欄位保留），且 SHALL NOT 建立 `speclink/` 目錄樹、獨立連接檔或任何指令檔。

`speclink link <url> [--repo <name>]` SHALL 維持既有行為，只寫入或更新 remote section並保留 tools 及其他欄位；`speclink unlink` SHALL 移除 remote section並保留檔內其他欄位。init 或 link 當下若已有可用憑證，CLI SHALL 立即向 Server 查驗 repo 是否屬於專案並回報結果；無憑證時 SHALL 提示執行 auth login。查驗時專案註冊兩個以上 repo 而指令沒有提供 `--repo`：link SHALL 以非 0 exit code 結束、於 stderr 輸出指引以 `speclink link <url> --repo <key>` 指名並列出全部可用註冊名的訊息、且 SHALL NOT 寫入 remote section；init SHALL 照既有順序完成初始化，之後以非 0 exit code 結束並於 stderr 輸出同一則訊息。專案只註冊一個 repo 而沒有提供 `--repo`、或有提供已註冊的 `--repo` 時，兩個指令的 stdout、stderr 與 exit code SHALL 與變更前相同；專案註冊多個 repo 這件事本身 SHALL NOT 使查驗失敗。此變更 SHALL NOT 為 init／link 新增 stdin payload 或 `--json` 介面；顯式 `--tools` 的既有成功 stdout、stderr、exit code 與 `--no-color` 行為 SHALL 維持基線。

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

#### Scenario: 多個 repo 的專案以 --repo 連接成功

- **WHEN** 已有可用憑證，專案註冊 erp 與 crm 兩個 repo，執行 link 並提供 `--repo crm`
- **THEN** stdout 回報 crm 已註冊於專案與連接成功，remote section 寫入 URL 與 repo crm，exit code 為 0

#### Scenario: link 時專案有多個 repo 而沒有指名

- **WHEN** 已有可用憑證，專案註冊 erp 與 crm 兩個 repo，執行 link 而沒有提供 `--repo`
- **THEN** exit code 非 0，stderr 的訊息含 `--repo`、erp 與 crm，remote section 不被寫入；`--no-color` 下訊息的文字相同

#### Scenario: 多個 repo 的專案指定不存在的 repo

- **WHEN** 已有可用憑證，專案註冊 erp 與 crm 兩個 repo，執行 link 並提供 `--repo nope`
- **THEN** exit code 非 0，stderr 為 `Error: repo 'nope' is not registered in this project (available: erp, crm)`，remote section 不被寫入

#### Scenario: init 時專案有多個 repo 而沒有指名

- **WHEN** 已有可用憑證，專案註冊 erp 與 crm 兩個 repo，以有效的 `--tools` 執行 Remote Store init 而沒有提供 `--repo`
- **THEN** 初始化的產物（Skills、`.gitignore`、含 remote URL 的 `.speclink.yaml`）已建立，exit code 非 0，stderr 的訊息含 `speclink link`、`--repo`、erp 與 crm

## ADDED Requirements

### Requirement: Server 回報暫時無法服務時顯示它的原因

Server 以 500 以上的狀態碼回應、且回應是可解析的錯誤信封、信封的 reason 為 unavailable、message 不是空字串時，用戶端的錯誤訊息 SHALL 是 `server unavailable — <message>`，其中 `<message>` 是信封的 message 原文，SHALL NOT 翻譯或截短；這個訊息 SHALL NOT 含 `check the connection url`。這是刻意的變更：變更前這種回應顯示 `server unavailable — check the connection url (`remote.url` in .speclink.yaml or SPECLINK_STORE_URL)`。

其他情況 SHALL 維持既有的位元級輸出：500 以上而回應不是可解析的信封、信封的 reason 不是 unavailable、或 message 是空字串時，訊息 SHALL 是既有的 `server unavailable — check the connection url (`remote.url` in .speclink.yaml or SPECLINK_STORE_URL)`；連不上 Server 時 SHALL 是既有的 `server unreachable — …`。exit code SHALL 維持非 0。

#### Scenario: Server 回 503 並說明原因

- **WHEN** Server 對登入方式的查詢回應 503，信封的 reason 為 unavailable、message 為「系統維護中，預計 10 分鐘後恢復」，執行 `speclink auth login --browser`
- **THEN** exit code 非 0，stderr 為 `Error: server unavailable — 系統維護中，預計 10 分鐘後恢復`，不含 `check the connection url`

#### Scenario: 500 以上而沒有可用的原因

- **WHEN** Server 回應 502 且本文不是錯誤信封，執行 `speclink list`
- **THEN** exit code 非 0，stderr 的訊息與變更前逐字相同，含 `check the connection url`

##### Example: 各種回應對應的訊息

| 回應 | 訊息 | 說明 |
| ---- | ---- | ---- |
| 503，信封 reason=unavailable、message=「資料庫切換中」 | `server unavailable — 資料庫切換中` | 顯示 Server 的原因 |
| 503，信封 reason=unavailable、message 為空字串 | `server unavailable — check the connection url (…)` | 沒有原因，維持既有訊息 |
| 500，信封 reason=internal | `server unavailable — check the connection url (…)` | reason 不是 unavailable，維持既有訊息 |
| 502，本文是 HTML | `server unavailable — check the connection url (…)` | 不是信封，維持既有訊息 |
| 連線被拒 | `server unreachable — check the connection url (…)` | 與變更前逐字相同 |
