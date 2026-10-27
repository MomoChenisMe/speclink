## MODIFIED Requirements

### Requirement: checkout 綁定驗證與 marker 寫入
<!-- BEFORE: checkout 綁定只驗證或寫入 remote marker，沒有讓使用者選擇並同步 Claude／Codex 受管產物。 -->

選擇連接 checkout 時 SHALL 先以零寫入檢查資料夾狀態：含 remote marker（`.speclink.yaml` 的 remote section）時，其 URL origin 與 repo SHALL 與所選 scope 一致；不一致 SHALL 以繁中訊息拒絕並指出 marker 指向。無 marker 時資料夾 SHALL 為 Git repository 方可繼續，非 Git repository SHALL 拒絕。檢查成功後，checkout 步驟 SHALL 顯示 Claude／Codex checkbox 及資料夾路徑；既有 `.speclink.yaml` 的 built-in tools SHALL 成為選取值，缺少 tools 清單時 SHALL 只依實際工具 footprint 預選且 SHALL NOT 默認 Claude。至少一個 built-in 工具與 checkout 路徑齊備前，「開啟 Workspace」SHALL disabled。

提交後 Desktop SHALL 重做 marker 邊界驗證，將 built-in 選集同步至 `.speclink.yaml`，生成或更新所選工具的 Skills 與 `AGENTS.md`／`CLAUDE.md` Speclink 區塊，並清理未選工具的 Speclink 受管產物；custom descriptor、unknown entry、remote／spec_dir／其他設定及指令區塊外的使用者內容 SHALL 保留。無 marker checkout SHALL 寫入與 CLI Remote init 同構的 remote section，既有相符 marker checkout SHALL 仍執行同步，不得提前成功。全部同步成功後 remote locator 的 checkoutRoot SHALL 記錄該資料夾並隨分頁持久化；同步失敗時 SHALL 保持 chooser、路徑與選集供重試，SHALL NOT 建立 remote tab／session 或開始 handshake。分頁 SHALL 以最小面（tooltip）呈現已連接的 checkout 路徑。checkout 綁定 SHALL NOT 改變本階段 capability 可用性。

#### Scenario: marker 不一致拒絕

- **WHEN** 所選資料夾的 remote marker 指向不同 origin 或不同 repo
- **THEN** 唯讀檢查被拒且訊息指出 marker 指向的 origin 與 repo，磁碟內容不變，分頁不建立 checkout 關聯

#### Scenario: 無 marker 的 Git repository 先選工具再綁定

- **WHEN** 使用者選取無 marker 的 Git repository，檢查完成後勾選 Claude 與 Codex並按下「開啟 Workspace」
- **THEN** 按下前資料夾零寫入；提交成功後 `.speclink.yaml` 含兩個 built-in tools 與所選 scope 的 remote section，兩組 Skills／指令區塊存在，CLI 可進入相同 Remote 模式，且 Workspace 才開始 handshake

#### Scenario: checkout 不允許空工具選集

- **WHEN** checkout 路徑已檢查成功但 Claude 與 Codex 均未選取
- **THEN** 「開啟 Workspace」維持 disabled，資料夾內容不變且沒有 tab／session 建立

#### Scenario: 既有相符 marker 缺少 Skills 時補齊

- **WHEN** checkout 的 remote marker 與 scope 相符、built-in tools 為 `[codex]`，但 `AGENTS.md` 區塊或 Codex Skills 缺少，使用者保持 Codex 選取並提交
- **THEN** 缺少的 Codex 受管產物被補齊，remote section 值不變，不建立 `openspec/`，同步成功後才開啟 Workspace

#### Scenario: 既有 checkout 從 Claude 切換為 Codex

- **WHEN** checkout 原 built-in tools 為 `[claude]`，`CLAUDE.md` 同時含 Speclink 區塊與使用者文字，使用者改為只選 Codex並提交
- **THEN** `.speclink.yaml` built-in tools 成為 `[codex]`，Codex Skills／`AGENTS.md` 被補齊，Claude Skills／Speclink 區塊被移除，`CLAUDE.md` 使用者文字與 remote section 保留，且不存在本機 `openspec/`

#### Scenario: 同步失敗不開啟且可重試

- **WHEN** checkout 檢查成功，但同步受管產物時遇到檔案系統寫入錯誤
- **THEN** chooser 顯示含失敗階段的單行錯誤並保留路徑與工具選集，remote tab／session／handshake 均未建立；修正檔案系統後以相同選集再次提交可收斂並開啟

### Requirement: remote marker 資料夾的探測分流
<!-- BEFORE: 僅含 remote marker 且已登入時直接 handshake，未檢查或補齊本機 Agent 工具產物。 -->

專案探測 SHALL 辨識 remote marker。資料夾僅含 marker 且對應 connection 已登入時，若 `.speclink.yaml` 有至少一個有效 built-in tool，Desktop SHALL 先依該選集 reconciliation，成功後才 handshake 開啟 remote tab並以呼叫端原始 path 作為 checkoutRoot；同步失敗 SHALL 顯示錯誤且不建立 session。marker 缺少有效 built-in tool 選集時 SHALL 導向 chooser 的 checkout 步驟、預填 server／scope／path 並要求明示選擇 Claude／Codex，不得直接開啟。無對應 connection 或未登入 SHALL 引導至 chooser 的 server 步驟並預填 server 位址。

marker 與本地 `openspec/` 並存時 SHALL 停下強制選擇，提供三個出口且皆無靜默覆蓋：「繼續本地」（本次以本地開啟、marker 不動）；「以 server 為準」（本地 `openspec/` 改名為帶日期備份後，資料夾轉為 checkout，完成工具 reconciliation 後開啟 remote tab，不上傳本地內容、不改動 server）；「遷移本地內容」（進入 workspace-migration 的遷移流程、目標為空 scope）。對話文案 SHALL 明說「以 server 為準」為備份後棄用本地、非合併。marker YAML 損壞 SHALL 沿 `.speclink.yaml` 既有 fail-closed 語意呈現錯誤。

#### Scenario: 有工具選集的 checkout 直達 remote tab

- **WHEN** 開啟僅含 remote marker 的資料夾、built-in tools 為 `[codex]` 且對應 server 已登入
- **THEN** Desktop 先補齊或更新 Codex 受管產物，成功後不經 chooser 完成 handshake，remote tab 開啟且 checkoutRoot 為該資料夾

#### Scenario: 缺少工具選集時導向 checkout 選擇

- **WHEN** 開啟僅含 remote marker 的資料夾、tools 缺席或不含 Claude／Codex，且對應 server 已登入
- **THEN** Desktop 導向 chooser checkout 步驟並預填原 path 與 scope，Claude／Codex 選集需由使用者確認，且提交成功前不建立 remote tab／session

#### Scenario: 自動補齊失敗不 handshake

- **WHEN** 有 built-in tools 的既有 checkout 在 reconciliation 時遇到檔案系統錯誤
- **THEN** Desktop 顯示帶路徑與失敗階段的錯誤，remote tab／session 不建立且不發出 handshake

#### Scenario: 並存衝突三出口

- **WHEN** 開啟同時含本地 `openspec/` 與 remote marker 的資料夾
- **THEN** 呈現強制選擇對話含三出口：繼續本地以本地開啟；以 server 為準將本地改名備份後執行工具 reconciliation 再開 remote tab且 server 內容未變；遷移本地內容進入遷移流程；無任何自動覆蓋

#### Scenario: 壞 marker fail-closed

- **WHEN** 選取的 checkout 具有無法解析的 `.speclink.yaml`
- **THEN** Desktop 顯示解析錯誤，不修改資料夾、不建立 tab／session且不發出 handshake
