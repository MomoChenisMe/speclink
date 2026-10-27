## MODIFIED Requirements

### Requirement: remote 初始化與連接指令
<!-- BEFORE: Remote init 在未提供 tools 且無 footprint 時默認 Claude，Desktop checkout 綁定只寫 remote marker。 -->

`speclink init --store remote --url <url> [--repo <name>]` SHALL 在寫入前取得至少一個明示的 Claude／Codex 選集：顯式 `--tools` SHALL 直接採用；互動終端缺少 `--tools` SHALL 於 stderr 詢問；非互動終端缺少 `--tools` SHALL 在零寫入狀態以非零 exit code 失敗並指引顯式值。選集有效時，Remote init SHALL 執行 Workspace init（`.speclink.yaml` tools、指令檔 Speclink marker、Skills、settings、`.gitignore`）並將 URL 與 repo 寫入 `.speclink.yaml` 的 remote section（檔案不存在時建立、既有欄位保留），且 SHALL NOT 建立 `openspec/` 目錄樹或獨立連接檔。

`speclink link <url> [--repo <name>]` SHALL 維持既有行為，只寫入或更新 remote section並保留 tools 及其他欄位；`speclink unlink` SHALL 移除 remote section並保留檔內其他欄位。init 或 link 當下若已有可用憑證，CLI SHALL 立即向 Server 查驗 repo 是否屬於專案並回報結果；無憑證時 SHALL 提示執行 auth login。此變更 SHALL NOT 為 init／link 新增 stdin payload 或 `--json` 介面；顯式 `--tools` 的既有成功 stdout、stderr、exit code 與 `--no-color` 行為 SHALL 維持基線。

#### Scenario: Remote init 顯式選擇 Claude

- **WHEN** 於空目錄執行 Remote Store init，提供有效 project URL、repo 與 `--tools claude`
- **THEN** 生成 `CLAUDE.md` Speclink marker、Claude Skills、settings、`.gitignore` 與含 `tools: [claude]`、remote URL／repo 的 `.speclink.yaml`，不存在 `openspec/` 與獨立連接檔，exit code 為 0

#### Scenario: Remote init 互動選擇 Codex

- **WHEN** stdin 為互動終端，執行 Remote Store init 且未提供 `--tools`，使用者選取 Codex並不選 Claude
- **THEN** prompt 寫入 stderr，生成 Codex Skills 與 `AGENTS.md` Remote Speclink marker，`.speclink.yaml` built-in tools 僅含 `codex`，成功摘要寫入 stdout且 exit code 為 0

#### Scenario: Remote init 非互動缺少 tools 被拒

- **WHEN** stdin 非互動，執行 Remote Store init 且未提供 `--tools`
- **THEN** exit code 非 0、stdout 為空、stderr 指引 `--tools claude`／`codex`／`claude,codex`，且 `.speclink.yaml`、`.gitignore`、Skills、指令檔與 `openspec/` 均未建立

#### Scenario: link 保留既有 tools 與其他欄位

- **WHEN** `.speclink.yaml` 已含 built-in tools、custom descriptor 與未知頂層鍵，以有效憑證執行 link 至 project URL 與已註冊 repo
- **THEN** remote section 被寫入，tools 清單與未知鍵原值保留，未觸發工具詢問或受管產物同步，exit code 為 0

#### Scenario: link 時 repo 不在專案註冊表

- **WHEN** 已有可用憑證，執行 link 並指定 Server 註冊表不存在的 repo
- **THEN** exit code 非 0，stderr 訊息指出 repo 不在專案內並列出可用註冊名，remote section 不被寫入

#### Scenario: unlink 移除連接

- **WHEN** 於 Remote 模式專案執行 unlink
- **THEN** `.speclink.yaml` 的 remote section 被移除、tools 與其他欄位保留，後續指令回到 filesystem 模式行為

## ADDED Requirements

### Requirement: Remote Workspace bootstrap 跨入口一致性

CLI Remote init 與 Desktop Remote checkout 綁定 SHALL 使用相同的正典生成來源，對相同 Claude／Codex 選集 SHALL 產生語意相同的 `.speclink.yaml.tools`、Skills 與 `AGENTS.md`／`CLAUDE.md` Speclink 區塊。兩個入口 SHALL 使用 Remote 指令區塊措辭、SHALL NOT 建立本機 `openspec/`，且 SHALL 保留 remote section、custom descriptor、未知設定及指令區塊外的使用者內容。Desktop 綁定既有相符 marker 時 SHALL 重新同步受管產物；同步完全成功前 SHALL NOT 開始 Remote handshake。

#### Scenario: CLI 與 Desktop 的 Codex bootstrap 同構

- **WHEN** 對兩個等價的新 Git checkout，分別以 CLI Remote init 與 Desktop checkout 綁定選取 Codex及相同 project／repo
- **THEN** 兩個 checkout 的 built-in tools 都僅含 `codex`、Codex Skills 與 `AGENTS.md` Remote Speclink 區塊使用相同正典內容、remote binding 語意相同，且兩者均不存在 `openspec/`

#### Scenario: Desktop 補齊既有 Remote Workspace

- **WHEN** 既有 checkout 的 remote section 與所選 scope 相符、built-in tools 為 `[claude,codex]`，但任一 Skills 或指令區塊缺少或過期
- **THEN** Desktop 以相同正典來源補齊兩個工具的受管產物，保留 remote／custom／使用者內容，成功後才開始 handshake

#### Scenario: Remote bootstrap 不改變 Server 內容

- **WHEN** CLI 或 Desktop 只執行本機 Remote Workspace bootstrap
- **THEN** 除既有 init credential 驗證或 Desktop 後續 handshake 外，不上傳、合併或修改 Server 規格內容，本機不建立 `openspec/`
