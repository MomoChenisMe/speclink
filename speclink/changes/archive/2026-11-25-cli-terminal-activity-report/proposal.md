## Why

在 Herdr（終端機工具 herdr-core，指令是 `hc`）裡用 AI agent 開發的人，靠 Speclink 面板看「哪個 agent 在做哪個變更或討論」。面板認 agent 的依據是終端機上的三個標籤：`speclink.change`、`speclink.discussion`、`speclink.skill`。今天這些標籤只在兩種情況會貼上：

- 從面板、技能跳窗或 `herdr-speclink` 技能開分頁時，開的人順手貼上。
- 使用者在 agent 裡直接打字時，靠選用的 hook（`integrations/herdr/mods/speclink/label-skill.mjs`、`pi-labels.ts`）貼上。

hook 要使用者自己裝，Codex 只測過一種事件，Pi 只有 extension，Cursor 完全沒有接上的機制，所以面板常常認不出 agent：那一列看不到 agent，開工鈕也還在。

不過每一種 agent 的 speclink 技能最後都會呼叫 speclink CLI，而且帶著變更名稱或討論的 slug。所以讓 CLI 自己在 Herdr 的終端機裡回報，就能涵蓋跑 speclink 指令的每一種 agent，使用者不用裝任何東西。例外是 Codex：實測發現它預設的沙盒不讓指令連到 Unix socket，要使用者自己打開沙盒網路才有效（見 Non-Goals）。

使用情境：apply、apply-with-worktree、propose、discuss、quality（review／verify 的 prepare）、archive 這些階段的 agent 跑 speclink 指令時。

## What Changes

- speclink CLI 執行下列指令成功後，如果自己跑在 Herdr 的終端機裡（環境變數 `HC_TERMINAL_ID` 與 `HC_TERMINAL_SOCKET` 都有值），就替這個終端機貼標籤。規則和 Herdr mod 現在的 `commandLabels`（`integrations/herdr/mods/speclink/model.js`）一致：
  - 算「正在做變更」的指令：`new change <名稱>`、`instructions apply --change <名稱>`、帶 `--change` 的 `task done`、`in-progress add <名稱>`、`review prepare <名稱>`、只帶一個名稱的 `archive <名稱>`。（hook 的規則裡還有 `verify prepare`，但這個指令不存在：驗證站沒有 prepare，兩站共用 `review prepare` 錄的起點。）貼 `speclink.change=<名稱>`；`speclink.discussion` 在 `new change … --from-discussion <slug>` 時設成那個 slug，其他時候清掉。
  - 算「正在做討論」的指令：帶 `--slug` 的 `discuss new`、`discuss context <slug>`、`discuss add-round <slug>`、`discuss conclude <slug>`。貼 `speclink.discussion=<slug>`，清掉 `speclink.change`。
  - `show`、`status`、`list` 這類只讀的指令不算，`speclink.skill` 一律不動。
- 回報的方式：CLI 直接連 `HC_TERMINAL_SOCKET` 這個 socket，送一行 JSON-RPC（方法 `terminal.label`），再讀一行回覆；連線、送出、等回覆整段最多 100 毫秒。CLI 絕不執行 `hc`，也不開任何子程序，所以 socket 連不上時不會啟動新的 core。
- 不在 Herdr 裡時，CLI 的行為和現在完全相同：不連線、不開程序、不多輸出、不多延遲。回報失敗、逾時都不改變 stdout、stderr、結束碼。
- 新增環境變數 `SPECLINK_TERMINAL_REPORT`：值是 `false`（不分大小寫）時關掉回報。不設或其他值時開著。
- 這個 repo 的 `.cargo/config.toml` 設 `SPECLINK_TERMINAL_REPORT=false`：開發者在 Herdr 裡跑 `cargo test` 時，測試開的 speclink 子程序不會把假的變更名稱貼到開發者自己的終端機上。
- 更新 `integrations/herdr/README.md` 與 `README.en.md` 的「做這個變更或討論的 agent」「追蹤直接打的指令」「技能會送給誰」三段，以及 `docs/configuration.md` 與 `docs/configuration.zh-TW.md` 的環境變數表。

做法比較（細節見 design 的 D1）：選「CLI 直接認 Herdr」，不選「CLI 提供通用的活動回報出口、由 Herdr 整合來接」。通用出口要使用者另外設定，等於把「hook 要手動裝所以常漏」的問題原樣搬過去；它也要每個指令多開一個程序，很難同時守住「不留背景程序」和「不感覺得出變慢」。

相容性影響：人眼輸出、`--json`、stderr 與結束碼都不變，回歸對照不受影響，既有使用者不用做任何遷移。沒有新增子指令、旗標或設定檔欄位，stdin 的讀法不變。生成的技能檔不變（claude、codex、copilot、pi、cursor 都一樣）。

三種形態下的結果：

- **本機 repo**：可用。
- **連到遠端的 checkout**：可用，規則相同。標籤描述的是「這個終端機在做什麼」，和文件放在哪裡無關；指令在 Server 接受之後才回報，不多送任何請求給 Server。
- **沒有本機規格檔的內嵌宿主**（Node SDK、server、desktop）：不適用。回報只在 CLI 程式的進入點，內嵌宿主不經過那裡，也不是跑在某個 agent 的終端機裡。

## Non-Goals

- 不拿掉現有的 hook（`label-skill.mjs`、`pi-labels.ts`）和面板、技能跳窗、`herdr-speclink` 技能貼標籤的做法。CLI 回報上線並實測後，再另外決定要不要簡化它們。
- 不改 Herdr mod 的程式（`integrations/herdr/mods/`、`integrations/herdr/skills/`）。它們不走 speclink 規格；這個變更只管 CLI 這一側，以及 `integrations/herdr/README` 兩種語言的說明。
- CLI 不貼 `speclink.skill`：CLI 不知道是哪個技能在呼叫它。
- 不處理「指令沒帶名稱」的情況（例如沒帶 `--change` 的 `task done`、沒帶 `--slug` 的 `discuss new`）：要知道引擎選了哪個名稱，得改動每個動詞的回傳值，和 hook 現在的規則一致即可。
- 不支援 Herdr 以外的終端機工具，也不做通用的回報出口。
- 不繞過 agent 的沙盒：Codex 預設的沙盒擋 Unix socket（2026-11-25 用 Codex CLI 0.171 實測），CLI 這一側沒辦法、也不該繞過它。`integrations/herdr/README` 兩種語言寫明這個限制，以及在 `~/.codex/config.toml` 打開 `sandbox_workspace_write.network_access` 的做法與代價；Codex 照舊靠 hook。

## Capabilities

### New Capabilities

- `terminal-activity-report`: speclink CLI 在 Herdr 終端機裡替自己的終端機貼上「正在做哪個變更或討論」的標籤，包含算數的指令、標籤內容、時間上限、關閉方式，以及不在 Herdr 裡時完全不動作的保證。現有規格掃過的最接近的是 `workspace-tools`（「CLI 的技能版本提醒」也是一個不改變輸出的旁支行為，但它管的是 AI 工具指令檔與技能版本）、`command-runtime`（管動詞在各入口的一致語意與輸出定義，不管 CLI 對外部程式的通知）、`verb-contract`（管 remote 通道的契約）；三者都沒有涵蓋 CLI 對終端機宿主的回報。

### Modified Capabilities

(none)

## Impact

- 目標使用者：在 Herdr 裡用 AI agent（Claude Code、Codex、GitHub Copilot、Pi、Cursor）開發的人；不用 Herdr 的人看不到任何差別。
- Affected specs: `terminal-activity-report`（新增）
- Affected crates: speclink-cli。speclink-core、speclink-verbs、speclink-host、Node SDK、server、apps/desktop 都不改。
- Affected code:
  - New: `crates/adapters/speclink-cli/src/terminal_report.rs`、`crates/adapters/speclink-cli/tests/it/terminal_report.rs`、`.cargo/config.toml`
  - Modified: `crates/adapters/speclink-cli/src/main.rs`、`crates/adapters/speclink-cli/tests/it/main.rs`、`integrations/herdr/README.md`、`integrations/herdr/README.en.md`、`docs/configuration.md`、`docs/configuration.zh-TW.md`
  - Removed: (none)
- 依賴：只用 Rust 標準函式庫的 Unix socket 與執行緒，不加新的 crate。Windows 沒有這個功能（Rust 標準函式庫沒有 Windows 的 Unix socket），在那裡什麼都不做。
