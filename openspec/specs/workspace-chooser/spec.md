# workspace-chooser Specification

## Purpose

新增 workspace 時的來源選擇流程：本機資料夾與 remote scope 兩種來源的分流、以 scopes 清單選擇取代文字輸入，以及把 checkout 綁定到 remote scope 時的驗證與 marker 寫入。本 capability 保證使用者用選的而不必背識別字串，且已帶 remote marker 的資料夾在探測時被正確分流。

## Requirements

### Requirement: 新增 Workspace 的來源分流

Desktop 的所有開啟入口（視窗頂列、空狀態、分頁列加號、伺服器頁籤）SHALL 匯流至單一「新增 Workspace」chooser：第一步 SHALL 分流「本機資料夾」與「Server」。本機路徑 SHALL 沿用既有資料夾選擇、專案探測與初始化流程且行為不變；伺服器頁籤入口 SHALL 預選該 server 直達 scope 選擇步驟。

#### Scenario: 本機開啟行為凍結

- **WHEN** 經 chooser 選擇本機資料夾開啟既有 speclink 專案
- **THEN** 分頁建立與看板呈現與 chooser 導入前一致；未初始化資料夾仍走既有 init 確認流程


<!-- @trace
source: server-wording-debrand
updated: 2026-09-03
-->

---
### Requirement: scopes 清單選擇取代文字輸入

server 路徑 SHALL 呈現已登入 connections 供選擇（含就地新增並登入後回流）；選定 server 後 SHALL 以 scopes 端點回應呈現 Project 分組的 Repos 清單供單選——SHALL NOT 要求使用者手動輸入 repo 識別。無任何 membership 時 SHALL 呈現空清單與繁中說明而非錯誤。選定後 SHALL 進入 checkout 分流：略過即以 spec-only 開啟 remote 分頁。

#### Scenario: 清單選擇開出 spec-only 分頁

- **WHEN** 於 chooser 選擇已登入 server，自 scopes 清單選定一個 Project/Repo 並略過 checkout
- **THEN** handshake 成功後 remote 分頁開啟，無任何 repo 識別的手動輸入步驟


<!-- @trace
source: workspace-chooser-onboarding
updated: 2026-07-20
code:
  - Cargo.lock
  - apps/desktop/core/src/project.rs
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/connections.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectTabs.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/__tests__/serversPanel.test.tsx
  - apps/desktop/src/__tests__/session.test.ts
  - apps/desktop/src/__tests__/tabs.test.ts
  - apps/desktop/src/__tests__/workspaceChooser.test.tsx
  - apps/desktop/src/adapter/connections.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/components/ServersPanel.tsx
  - apps/desktop/src/components/WorkspaceChooser.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/main.tsx
  - apps/desktop/src/session.ts
  - apps/desktop/src/store.ts
  - apps/desktop/src/tabs.ts
-->

---
### Requirement: checkout 綁定驗證與 marker 寫入

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


<!-- @trace
source: unify-agent-tool-bootstrap
updated: 2026-07-24
code:
  - apps/desktop/core/src/project.rs
  - apps/desktop/core/src/settings.rs
  - apps/desktop/src-tauri/src/connections.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/__tests__/remoteResilience.test.tsx
  - apps/desktop/src/__tests__/serversPanel.test.tsx
  - apps/desktop/src/__tests__/workspaceChooser.test.tsx
  - apps/desktop/src/adapter/connections.ts
  - apps/desktop/src/components/WorkspaceChooser.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/store.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/init_tools.rs
  - crates/speclink-cli/tests/remote_connect.rs
  - crates/speclink-cli/tests/remote_section.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/init.rs
-->

---
### Requirement: remote marker 資料夾的探測分流

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


<!-- @trace
source: unify-agent-tool-bootstrap
updated: 2026-07-24
code:
  - apps/desktop/core/src/project.rs
  - apps/desktop/core/src/settings.rs
  - apps/desktop/src-tauri/src/connections.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/__tests__/remoteResilience.test.tsx
  - apps/desktop/src/__tests__/serversPanel.test.tsx
  - apps/desktop/src/__tests__/workspaceChooser.test.tsx
  - apps/desktop/src/adapter/connections.ts
  - apps/desktop/src/components/WorkspaceChooser.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/store.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/init_tools.rs
  - crates/speclink-cli/tests/remote_connect.rs
  - crates/speclink-cli/tests/remote_section.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/init.rs
-->

---
### Requirement: 最近開啟清單

「新增 Workspace」chooser 的第一步 SHALL 在「本機資料夾」與「Server」兩張來源卡下方列出最近開啟清單。app SHALL 於每次本機或 remote workspace 成功開啟時（經 chooser、分頁點擊、remote marker 探測或本機轉 remote）把該 workspace（locator 與顯示名）記入清單最前；同 locator 的 workspace SHALL 只保留一筆；清單 SHALL 最多保留 20 筆，超過時 SHALL 丟棄最舊的一筆。本機轉 remote 成功時 SHALL 移除該資料夾的本機條目並記入 remote 條目。記錄 SHALL 持久化於 app 本機狀態（localStorage 鍵 `speclink.recentWorkspaces`，JSON `{ version: 1, entries: [{ locator, name }] }`），SHALL NOT 寫入任何專案目錄；關閉分頁與分頁列的上限淘汰 SHALL NOT 改變記錄。

清單顯示時 SHALL 濾掉目前分頁列上已開著的 workspace（以 locator key 比對）；濾後為空時 SHALL NOT 顯示「最近開啟」區段（含標題）。本機條目 SHALL 顯示資料夾名稱與完整路徑；remote 條目 SHALL 顯示連線名稱與 workspace 顯示名（projectName/repoName），連線名稱 SHALL 自連線登錄即時查得。每筆條目 SHALL 提供移除操作，執行後 SHALL 自畫面與持久化記錄移除。

點本機條目時 app SHALL 先探測該路徑，探測成功 SHALL 關閉 chooser 並執行與「本機資料夾」選同一路徑相同的開啟流程（既有專案直接開啟、未初始化資料夾走 init 確認、帶 remote marker 的資料夾走既有分流）；探測失敗 SHALL 把該條目轉為錯誤態並顯示錯誤原因，SHALL NOT 建立分頁或切換專案，記錄 SHALL 保留至使用者移除。點 remote 條目時 app SHALL 先驗證原 checkout 綁定仍與該 scope 一致（無 checkout 綁定的條目 SHALL 跳過此驗證），再執行與 scope 選擇流程相同的 remote 開啟；驗證或 handshake 失敗 SHALL 轉為錯誤態並顯示錯誤原因。

remote 條目的連線狀態 SHALL 於連線清單讀取成功後才判定：連線已自連線登錄移除 SHALL 以錯誤態呈現「連線已移除」，連線仍在但未登入 SHALL 以錯誤態呈現「連線已登出」，兩者 SHALL 停用開啟操作且 SHALL 保留移除操作。連線清單讀取失敗或尚未完成時 SHALL NOT 在顯示面判定為任一錯誤態，開啟操作 SHALL 維持可用；此時使用者點該條目，app SHALL 於開啟流程內以現有清單補判——連線仍解不出且該條目綁有 checkout 時 SHALL 以「連線已移除」轉錯誤態，無 checkout 綁定的條目 SHALL 照常執行 remote 開啟。錯誤態 SHALL 於 chooser 重新開啟時清除。

app 升級後首次啟動、localStorage 尚無最近開啟鍵時，app SHALL 以持久化分頁補種清單（最後開啟的分頁在最前）；鍵已存在（含空清單）時 SHALL NOT 補種。鍵內容為壞 JSON、version 不為 1 或形狀不識別時 SHALL 讀為空清單且 app 照常啟動；個別條目形狀不識別時 SHALL 只丟棄該條目。介面文案 SHALL 依介面語言提供：zh-TW 為「最近開啟」／「自最近開啟移除」／「連線已移除」／「連線已登出」，en 為「Recently opened」／「Remove from recently opened」／「Connection removed」／「Connection signed out」。

#### Scenario: 關閉分頁後仍列於最近開啟

- **WHEN** 使用者依序開啟本機專案 A 與 B，關閉 B 的分頁，再開啟「新增 Workspace」
- **THEN** 第一步兩張來源卡下方顯示「最近開啟」區段，列出 B（資料夾名稱與完整路徑）且不列 A；A 與 B 的專案目錄內均無因此新增的檔案

#### Scenario: 記錄全在分頁列上時不顯示區段

- **WHEN** 最近開啟記錄只含 A 與 B，且 A、B 都在分頁列上時開啟「新增 Workspace」
- **THEN** 第一步只顯示兩張來源卡，無「最近開啟」標題與任何條目

#### Scenario: 去重上移與上限截尾

- **WHEN** 使用者依序成功開啟多個 workspace
- **THEN** 記錄依最新在前排列、同 workspace 只留一筆、最多 20 筆

##### Example: 順序、去重與上限

| 開啟順序 | 記錄（最新在前） | 說明 |
| -------- | ---------------- | ---- |
| A, B, A | A, B | A 重複開啟只留一筆並移到最前 |
| A, B, C（B 分頁其後被關閉） | C, B, A | 關閉分頁不動記錄 |
| W1 … W21（21 個相異 workspace） | W21 … W2 | 第 21 筆記入時丟最舊的 W1 |

#### Scenario: 點本機條目直接開啟

- **WHEN** 使用者點最近開啟中的本機專案 B（資料夾仍是 speclink 專案）
- **THEN** chooser 關閉，B 的分頁出現於分頁列並成為活躍分頁，看板呈現 B 的內容，與經「本機資料夾」選同一路徑的結果一致

#### Scenario: 點未初始化資料夾的條目仍走 init 確認

- **WHEN** 使用者點最近開啟中的本機條目，而該資料夾的 openspec 骨架已被移除
- **THEN** app 顯示既有的初始化確認對話框，未經確認前不寫入該資料夾

#### Scenario: 點 remote 條目以原綁定開啟

- **WHEN** 使用者點最近開啟中的 remote 條目（連線仍已登入、原本綁有本機 checkout）
- **THEN** app 以同一 connection、同一 Project／Repo 與同一 checkout 資料夾開啟 remote workspace，分頁列出現該 workspace 並成為活躍分頁，chooser 關閉

#### Scenario: 本機資料夾已消失時轉錯誤態

- **WHEN** 最近開啟中本機專案 B 的資料夾已被刪除，使用者點該條目
- **THEN** 該條目顯示錯誤標記與單行錯誤原因，chooser 保持開啟，分頁列與活躍專案不變；條目仍在清單中，直到使用者以移除操作清除

#### Scenario: remote 連線已移除時直接呈現錯誤態

- **WHEN** 最近開啟中 remote 條目所屬的連線已自「伺服器」頁移除，使用者開啟「新增 Workspace」
- **THEN** 該條目以錯誤態呈現「連線已移除」（en 為「Connection removed」），開啟操作停用，移除操作仍可用

#### Scenario: remote 連線已登出時停用開啟

- **WHEN** 最近開啟中 remote 條目所屬的連線仍在連線登錄但已登出，使用者開啟「新增 Workspace」
- **THEN** 該條目以錯誤態呈現「連線已登出」（en 為「Connection signed out」），開啟操作停用，移除操作仍可用

#### Scenario: 連線清單讀取失敗時不判定錯誤態

- **WHEN** 開啟「新增 Workspace」時連線清單讀取失敗（清單保持為空）
- **THEN** remote 條目 SHALL NOT 呈現「連線已移除」或「連線已登出」，開啟操作維持可用

#### Scenario: 清單未就緒時點條目由開啟流程補判

- **WHEN** 連線清單尚未讀到（讀取失敗或仍在進行中），使用者點一個綁有 checkout 的 remote 條目
- **THEN** 該條目以「連線已移除」轉錯誤態，SHALL NOT 建立分頁；同一情況下點無 checkout 綁定的 remote 條目 SHALL 照常以原 connection 與 scope 開啟

#### Scenario: 點 remote 條目時原 checkout 已失效

- **WHEN** 最近開啟中 remote 條目的 checkout 資料夾已被刪除，使用者點該條目
- **THEN** 該條目顯示錯誤標記與單行錯誤原因，SHALL NOT 建立分頁或切換專案，移除操作仍可用

#### Scenario: 移除條目後重啟不再出現

- **WHEN** 使用者對最近開啟中的條目執行移除，再重啟 app 並開啟「新增 Workspace」
- **THEN** 該條目不再出現於清單，其餘條目順序不變

#### Scenario: 升級後首次啟動自分頁補種

- **WHEN** localStorage 存有分頁 A、B（依序開啟）而無最近開啟鍵時啟動新版 app，隨後關閉 A 的分頁並開啟「新增 Workspace」
- **THEN** 最近開啟列出 A；持久化記錄為 B、A（最新在前）；再次重啟不會重複補種

#### Scenario: 壞資料歸零且不補種

- **WHEN** localStorage 的最近開啟鍵被手改為無法解析的內容後啟動 app
- **THEN** app 照常啟動、不崩潰、不彈錯誤；「新增 Workspace」第一步無「最近開啟」區段；下一次成功開啟 workspace 後該鍵寫回 version 1 的合法內容且只含這一筆


<!-- @trace
source: server-wording-debrand
updated: 2026-09-03
-->