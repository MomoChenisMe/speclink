## ADDED Requirements

### Requirement: init 內建 Agent 工具選擇

所有 `speclink init`（filesystem 與 Remote Store）SHALL 在任何 Workspace 寫入前解析至少一個內建 Agent 工具。顯式 `--tools` SHALL 接受 `claude`、`codex` 或逗號分隔的兩者並跳過詢問；解析後為空或含未知名稱 SHALL 以非零 exit code 失敗。未提供 `--tools` 且 stdin 為互動終端時，CLI SHALL 於 stderr 明示詢問 Claude 與 Codex 並允許選一個或兩個；兩者皆未選 SHALL NOT 開始 init，並 SHALL 繼續要求有效選擇。未提供 `--tools` 且 stdin 非互動終端時，CLI SHALL 在零寫入狀態以非零 exit code 失敗，stderr SHALL 指出 `--tools` 與三種有效選法，stdout SHALL 為空。init SHALL NOT 將 redirected／piped stdin 當作工具答案，SHALL NOT 新增 stdin payload 或 `--json` 介面。此行為是對 Spectra 2.3.1 與既有 footprint 自動偵測的刻意分歧；顯式提供 `--tools` 時既有人眼成功輸出與 `--no-color` 行為 SHALL 維持既有基線。

#### Scenario: filesystem init 顯式選擇 Codex

- **WHEN** 在空目錄執行 filesystem init 並顯式提供 `--tools codex`
- **THEN** exit code 為 0，stdout 沿用既有 Initialized 與 Generated files 摘要，`.speclink.yaml` 的 built-in tools 僅含 `codex`，並生成 Codex Skills 與 `AGENTS.md` Speclink 區塊

#### Scenario: Remote init 顯式選擇兩個工具

- **WHEN** 在空目錄執行 Remote Store init 並顯式提供 `--tools claude,codex`、有效 project URL 與 repo
- **THEN** exit code 為 0，`.speclink.yaml` 同時含兩個 built-in tools 與 remote section，生成兩組 Skills 及兩份指令區塊，且不存在 `openspec/`

#### Scenario: 互動終端選擇 Claude 與 Codex

- **WHEN** 未提供 `--tools` 且 stdin 為互動終端，使用者對 Claude 與 Codex 都回答 yes
- **THEN** 詢問文字只寫入 stderr，init 以兩個工具執行，成功摘要寫入 stdout

#### Scenario: 互動終端不得提交空選集

- **WHEN** 未提供 `--tools` 且 stdin 為互動終端，使用者對 Claude 與 Codex 都回答 no
- **THEN** CLI 不建立 `.speclink.yaml`、`openspec/`、`.gitignore`、Skills 或指令檔，並再次要求至少選取一個工具

#### Scenario: 非互動 init 缺少 tools 零寫入失敗

- **WHEN** stdin 為 pipe 或 redirect，執行 init 且未提供 `--tools`
- **THEN** exit code 非 0、stdout 為空，stderr 單行訊息包含 `--tools`、`claude` 與 `codex`，且目標目錄內容逐項不變

#### Scenario: 空或未知的顯式 tools 被拒

- **WHEN** 執行 init 並提供空的 `--tools` 值或含 `vscode` 的值
- **THEN** exit code 非 0，stderr 指出選集為空或 unknown tool，且任何 Workspace 檔案都未建立或修改

#### Scenario: no-color 不改變工具選擇語意

- **WHEN** 以 `--no-color` 執行互動式 init 並選取任一有效工具
- **THEN** prompt 與成功輸出不含 ANSI escape sequence，exit code 與檔案效果和有色模式相同

### Requirement: built-in tools 權威收斂

Workspace 工具同步 SHALL 將請求中的 Claude／Codex 集合視為 built-in tools 的完整期望狀態。被選取的 built-in SHALL 生成或更新其 `speclink-*` Skills 與對應 `SPECLINK:START..END` 指令區塊；未選取的 built-in SHALL 移除 Speclink 產生的 Skills 與指令區塊，SHALL 保留區塊外的使用者內容，且指令檔在清理後全空時才刪除。同步 SHALL 保留 `.speclink.yaml` 內的 custom descriptor、unknown tool entry、remote、spec_dir 與其他頂層鍵。相同期望狀態重試 SHALL 收斂到相同檔案結果，不重複 marker 或破壞使用者內容。

#### Scenario: Claude 切換為 Codex並保留自訂工具

- **WHEN** `.speclink.yaml` 含 `claude`、一個 custom descriptor、remote section 與未知頂層鍵，`CLAUDE.md` 同時含 Speclink 區塊和使用者文字，然後同步 built-in 選集 `[codex]`
- **THEN** `tools` 保留 custom descriptor 並將 built-in 集合改為僅 `codex`，remote 與未知鍵值不變，Codex Skills／`AGENTS.md` 被補齊，Claude Skills／Speclink 區塊被移除，且 `CLAUDE.md` 的使用者文字仍存在

##### Example: built-in 選集轉換

| 原 built-in | 新選集 | 保留 custom descriptor | 受管結果 |
| --- | --- | --- | --- |
| claude | codex | 是 | 移除 Claude、補齊 Codex |
| claude,codex | claude | 是 | 更新 Claude、移除 Codex |
| codex | claude,codex | 是 | 補齊兩者且不重複 marker |

#### Scenario: 既有選集缺少產物時自動補齊

- **WHEN** `.speclink.yaml` 的 built-in tools 為 `[codex]`，但 `AGENTS.md` Speclink 區塊或任一 Codex Skill 缺少，然後再次同步 `[codex]`
- **THEN** 缺少或過期的受管產物被補齊至正典內容，其他使用者檔案維持不變，且同步成功

#### Scenario: 壞設定在寫入前被拒

- **WHEN** `.speclink.yaml` 無法解析，然後請求同步任一 built-in 選集
- **THEN** 同步以單行解析錯誤失敗，原設定、Skills 與指令檔逐字元不變
