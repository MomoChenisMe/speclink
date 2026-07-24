---
topic: 目前若用遠端 speclink server 後，選擇本地專案並開啟 workspace 時，Desktop 應協助初始化本地專案並安裝 Skills；CLI remote 初始化也應提供 Claude／Codex 選擇
slug: remote-workspace-local-skill-bootstrap
status: promoted
promoted_to: unify-agent-tool-bootstrap
created: 2026-07-24
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 目前若用遠端 speclink server 後，選擇本地專案並開啟 workspace 時，Desktop 應協助初始化本地專案並安裝 Skills；CLI remote 初始化也應提供 Claude／Codex 選擇

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

這份討論由 Remote Server 的 Workspace Chooser 行為缺口觸發：Desktop 綁定本機 checkout 時目前只驗證／寫入 remote marker，沒有執行 CLI remote init 已具備的本機指令檔與 Skills 初始化。採 assumptions mode，因為程式碼探索已找到 WorkspaceChooser、ConnectionsAdapter、Tauri bind_checkout、desktop project init、speclink_core::init 與 CLI cmd_init_remote 等完整跨層路徑。相關正典規格為 workspace-chooser 與 remote-connection；現況顯示兩者對「與 CLI remote 初始化同構」的涵蓋範圍不一致。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-24)

**Focus**: Desktop 與 CLI 的 Remote 本機初始化是否都應明示選擇要安裝的 AI 工具 Skills
**Position**: 兩個入口都應讓使用者選擇 Claude／Codex，並共用同一套 Core remote workspace 初始化語意。
- Desktop 在選擇本機資料夾的 checkout 步驟提供 Claude／Codex 複選，完成所選工具的指令區塊與 Skills 初始化後才開啟 Workspace
- CLI 執行 remote init 且未傳 `--tools` 時，在互動式終端詢問 Claude／Codex；已傳 `--tools` 時直接採用指定值
- Desktop 應直接呼叫 CLI 背後的 `speclink_core::init` 正典邏輯，不啟動外部 CLI process
**Ruled out**: 只寫 remote marker，因為本機 checkout 會被視為已連接但 Agent 無可用 Skills；純自動偵測，因為無 footprint 時目前會預設 Claude，可能漏掉 Codex
**Open**: CLI 在非互動終端且未傳 `--tools` 時應失敗並要求顯式指定，還是保留自動偵測相容行為；已有相符 marker 但缺少 Skills 的 checkout 是否也要在綁定時補齊

### Round 2 — assumptions (2026-07-24)

**Focus**: CLI 的 Claude／Codex 明示選擇應只套用 Remote init，還是所有 init
**Position**: 所有 `speclink init` 在未傳 `--tools` 時都應採用相同的工具選擇流程。
- 工具選擇決定的是本機 Workspace 的指令檔與 Skills，與規格儲存在本機 filesystem 或 Remote Server 無關
- `speclink init --store fs` 與 `speclink init --store remote` 應共用選擇契約，僅 store 初始化內容不同
- 顯式 `--tools` 仍是跳過詢問並取得可重現結果的入口
**Ruled out**: 僅 Remote init 詢問，因為會讓同一個工具選擇概念依 Store 類型產生不必要的兩種行為
**Open**: 非互動終端未傳 `--tools` 時應失敗並要求顯式指定，還是沿用自動偵測；已有相符 remote marker 但缺少 Skills 的 checkout 是否於綁定時補齊

### Round 3 — assumptions (2026-07-24)

**Focus**: 非互動終端執行 init 且未傳 `--tools` 時的行為
**Position**: 非互動終端缺少 `--tools` 時應在任何檔案寫入前直接失敗。
- 錯誤訊息應要求顯式傳入 `--tools claude`、`--tools codex` 或 `--tools claude,codex`
- 互動式終端未傳 `--tools` 時才顯示 Claude／Codex 選擇；傳入 `--tools` 一律跳過詢問
- 此契約同時套用 filesystem 與 Remote Store 初始化，確保 CI／腳本結果可重現
**Ruled out**: 非互動環境沿用自動偵測或預設 Claude，因為可能安裝錯誤工具且結果依專案 footprint 改變
**Open**: 已有相符 remote marker 但缺少 Skills 的 checkout 是否於 Desktop 綁定時補齊

### Round 4 — assumptions (2026-07-24)

**Focus**: 既有 checkout 自動補齊的內容是否包含 Agent 指令檔注入
**Position**: Desktop 綁定新舊 checkout 與所有 CLI init 都應以同一套可重複同步邏輯補齊 Skills 與 Agent 指令區塊。
- 已有相符 remote marker 的 checkout 仍須執行同步，不得直接返回而跳過本機 Agent 設定
- 同步內容包含所選工具的 `speclink-*` Skills，以及 `AGENTS.md`／`CLAUDE.md` 中由 `SPECLINK:START..END` 界定的指令區塊
- 指令區塊缺少時插入、過期時更新；區塊外的使用者內容必須保留
- filesystem init、Remote Store init 與 Desktop checkout 綁定共用相同生成來源
**Ruled out**: 只補 Skills，因為 Agent 缺少何時使用 Skills 的專案指引；只處理新 checkout，因為既有 marker 專案無法自我修復
**Open**: `AGENTS.md`／`CLAUDE.md` 應依 Claude／Codex 勾選結果各自生成，還是無論選擇都固定生成兩者；取消勾選是否應移除 Speclink 管理的舊產物

### Round 5 — assumptions (2026-07-24)

**Focus**: Claude／Codex 勾選是否為本機 Agent 設定的權威期望狀態
**Position**: 工具勾選與 `.speclink.yaml` 的 `tools` 清單應共同表達完整期望狀態，而非只做追加安裝。
- 勾選 Codex 即確保 `.agents/skills/speclink-*` 與 `AGENTS.md` 的 Speclink 指令區塊存在且為最新版
- 勾選 Claude 即確保 Claude Skills 與 `CLAUDE.md` 的 Speclink 指令區塊存在且為最新版
- 取消勾選即移除該工具由 Speclink 管理的 Skills 與指令區塊；指令檔區塊外的使用者內容保留，清理後空檔才可刪除
- Desktop、filesystem init、Remote Store init 與後續同步皆採相同契約
**Ruled out**: 選擇只追加不清理，因為會讓磁碟產物與 `tools` 期望狀態分歧，Agent 仍可能載入已取消的 Skills
**Open**: 以既有 Remote checkout 從 Claude 切換為 Codex 的具體修復與失敗案例確認

### Round 6 — assumptions (2026-07-24)

**Focus**: 既有 Remote checkout 從 Claude 切換為 Codex 時的修復、清理與失敗行為
**Position**: 切換工具必須以可重試的同步完成全部受管產物，成功後才開啟 Workspace。
- 將 `.speclink.yaml` 更新為 `tools: [codex]` 並保留 remote section，不建立本機 `openspec/`
- 建立或更新 `AGENTS.md` 的 Speclink 區塊與 Codex Skills
- 移除 Claude Skills 與 `CLAUDE.md` 的 Speclink 區塊，但保留區塊外的使用者內容
- 任一步驟失敗時不得開啟 Workspace，應顯示可操作錯誤並允許修正後安全重試
- CLI 非互動執行 init 且缺少 `--tools` 時，必須在任何寫入前以非零狀態結束
**Ruled out**: 部分同步後仍開啟 Workspace，因為畫面會宣稱 checkout 可用，但 Agent 設定可能處於不一致狀態
**Open**: 無

## Conclusion

**Decision**: 建立跨 Desktop 與所有 `speclink init` 的統一 Agent 工具選擇與本機 Workspace 同步契約。互動式 init 未傳 `--tools` 時明示複選 Claude／Codex；非互動式 init 未傳 `--tools` 時在任何寫入前失敗。Desktop 綁定新舊 Remote checkout 時顯示相同選擇，並在開啟 Workspace 前同步 `.speclink.yaml`、所選工具的 Skills、`AGENTS.md`／`CLAUDE.md` Speclink 區塊及移除未選工具的受管產物。
**Rationale**: Agent 工具設定是本機 Workspace 關注點，與 filesystem／Remote Store 無關；明示且具權威性的工具選擇可避免無 footprint 時默認 Claude、marker 與 Skills 不一致，以及舊 checkout 無法自我修復。Desktop 應直接共用 CLI 背後的 Core 生成／同步邏輯，保持單一正典語意。
**Rejected alternatives**: 只寫 remote marker，會留下無 Skills 的 checkout；只靠自動偵測或預設 Claude，結果不明確且可能漏裝 Codex；只讓 Remote init 詢問，會讓 Store 類型不必要地改變本機工具契約；選擇只追加不清理，會留下已取消工具的可載入產物；Desktop 啟動外部 CLI process，會增加 binary 存在與版本一致性問題。
**Deferred**: 具體 UI 文案、終端互動元件與失敗後的內部復原實作在 proposal／design 階段決定；既定行為是不開啟 Workspace、保留使用者內容且可安全重試。
**Capture to**: proposal，並於設計與 delta specs 更新 workspace-chooser、remote-connection／init 行為
**Next**: `$speclink-propose --from-discussion remote-workspace-local-skill-bootstrap`
