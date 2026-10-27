## MODIFIED Requirements

### Requirement: remote 初始化與連接指令

`speclink init --store remote --url <url> [--repo <name>]` SHALL 在寫入前取得至少一個明示的 Claude／Codex 選集：顯式 `--tools` SHALL 直接採用；互動終端缺少 `--tools` SHALL 於 stderr 詢問；非互動終端缺少 `--tools` SHALL 在零寫入狀態以非零 exit code 失敗並指引顯式值。選集有效時，Remote init SHALL 執行 Workspace init（`.speclink.yaml` tools、Skills、`.gitignore`）並將 URL 與 repo 寫入 `.speclink.yaml` 的 remote section（檔案不存在時建立、既有欄位保留），且 SHALL NOT 建立 `openspec/` 目錄樹、獨立連接檔或任何指令檔。

`speclink link <url> [--repo <name>]` SHALL 維持既有行為，只寫入或更新 remote section並保留 tools 及其他欄位；`speclink unlink` SHALL 移除 remote section並保留檔內其他欄位。init 或 link 當下若已有可用憑證，CLI SHALL 立即向 Server 查驗 repo 是否屬於專案並回報結果；無憑證時 SHALL 提示執行 auth login。此變更 SHALL NOT 為 init／link 新增 stdin payload 或 `--json` 介面；顯式 `--tools` 的既有成功 stdout、stderr、exit code 與 `--no-color` 行為 SHALL 維持基線。

#### Scenario: Remote init 顯式選擇 Claude

- **WHEN** 於空目錄執行 Remote Store init，提供有效 project URL、repo 與 `--tools claude`
- **THEN** 生成 Claude Skills、`.gitignore` 與含 `tools: [claude]`、remote URL／repo 的 `.speclink.yaml`，不存在 `openspec/`、獨立連接檔與 `CLAUDE.md`，exit code 為 0

#### Scenario: Remote init 互動選擇 Codex

- **WHEN** stdin 為互動終端，執行 Remote Store init 且未提供 `--tools`，使用者選取 Codex並不選 Claude
- **THEN** prompt 寫入 stderr，生成 Codex Skills 而無 `AGENTS.md`，`.speclink.yaml` built-in tools 僅含 `codex`，成功摘要寫入 stdout且 exit code 為 0

#### Scenario: Remote init 非互動缺少 tools 被拒

- **WHEN** stdin 非互動，執行 Remote Store init 且未提供 `--tools`
- **THEN** exit code 非 0、stdout 為空、stderr 指引 `--tools claude`／`codex`／`claude,codex`，且 `.speclink.yaml`、`.gitignore`、Skills 與 `openspec/` 均未建立

#### Scenario: link 保留既有 tools 與其他欄位

- **WHEN** `.speclink.yaml` 已含 built-in tools、custom descriptor 與未知頂層鍵，以有效憑證執行 link 至 project URL 與已註冊 repo
- **THEN** remote section 被寫入，tools 清單與未知鍵原值保留，未觸發工具詢問或受管產物同步，exit code 為 0

#### Scenario: link 時 repo 不在專案註冊表

- **WHEN** 已有可用憑證，執行 link 並指定 Server 註冊表不存在的 repo
- **THEN** exit code 非 0，stderr 訊息指出 repo 不在專案內並列出可用註冊名，remote section 不被寫入

#### Scenario: unlink 移除連接

- **WHEN** 於 Remote 模式專案執行 unlink
- **THEN** `.speclink.yaml` 的 remote section 被移除、tools 與其他欄位保留，後續指令回到 filesystem 模式行為

## REMOVED Requirements

### Requirement: 指令區塊的 remote 變體

**Reason**: 指令檔 marker 區塊全面停止生成，remote 措辭變體隨之失去載體。
**Migration**: remote 語意由技能本文承載——技能於 remote 模式指引一律使用 speclink 動詞、經 Context Projection 唯讀取用規格內容，且 instructions payload 的 contextFiles 已指向正確位置，無需另行注入措辭。
