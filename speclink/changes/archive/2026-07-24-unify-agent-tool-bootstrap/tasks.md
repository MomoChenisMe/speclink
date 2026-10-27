<!--
Each task states the observable contract and its verification target. Paths locate the work;
the task is complete only when the named behavior and verification both hold.
-->

## 1. Core 工具收斂

- [x] 1.1 RED — 為「Core 單一 Workspace 工具同步入口」、「built-in tools 權威收斂」、「Built-in 選擇收斂且保留自訂描述子」與「Core and configuration contract」在 `crates/speclink-core/src/init.rs`、`crates/speclink-core/src/config.rs` 加入失敗測試：逐列覆蓋 spec example 的 Claude／Codex 轉換、custom descriptor／unknown key／remote section 保留、marker 外文字保留、缺漏產物補齊、壞 YAML 零寫入及 Remote mode 不建 `openspec/`；執行 `cargo test -p speclink-core init::tests config::tests` 並確認新斷言先因缺少共用收斂入口而失敗。 <!-- speclink-task:tsk_01KY92CSK34ZTPJKHA1QFBAP1Y -->
- [x] 1.2 GREEN — 在 `crates/speclink-core/src/init.rs` 實作非空 built-in `Tool` 選集的共用 reconciliation，重用 `update_app_config_tools_text` 與既有 generate／prune，使 filesystem init、Remote init 與既有 Workspace 對相同選集產生相同受管結果且保留非 built-in 設定；執行 `cargo test -p speclink-core init::tests config::tests`，確認 1.1 全綠且既有 golden tests 不變。 <!-- speclink-task:tsk_01KY92CSK3DV9JRDRJ0QT1RWHW -->
- [x] 1.3 REFACTOR — 將 `apps/desktop/core/src/settings.rs` 的 built-in tools 寫入／同步與 `apps/desktop/core/src/project.rs` 的 init 接線改用 Core 單一入口，移除重複 orchestration 而不改變 Project Settings 與本機 init 可觀察結果；執行 `cargo test --manifest-path apps/desktop/core/Cargo.toml` 與 `cargo test -p speclink-core`，確認新增及既有設定測試全綠。 <!-- speclink-task:tsk_01KY92CSK3JVXTX81BV404FDZZ -->

## 2. CLI init 工具選擇

- [x] 2.1 RED — 為「init 內建 Agent 工具選擇」、「remote 初始化與連接指令」、「CLI 互動解析停留在 speclink-cli」與「CLI observable behavior」新增 `crates/speclink-cli/tests/init_tools.rs` 並補 `crates/speclink-cli/tests/remote_section.rs`／`remote_connect.rs`：覆蓋三種顯式 `--tools`、空／未知值、非互動 filesystem／remote 零寫入、互動 helper 單選／雙選／全否重試、stderr／stdout／exit code 與 `--no-color`；先執行 `cargo test -p speclink-cli --test init_tools --test remote_section --test remote_connect`，確認新案例因現有自動偵測或缺少 prompt 而失敗。 <!-- speclink-task:tsk_01KY92CSK3WA0M34RHHBB7NZ9T -->
- [x] 2.2 GREEN — 在 `crates/speclink-cli/src/main.rs`、`crates/speclink-cli/src/commands.rs`、`crates/speclink-cli/src/remote_commands.rs` 以 `IsTerminal` 與可注入行讀寫 helper 實作統一工具解析：顯式旗標跳過詢問、互動 prompt 寫 stderr且要求至少一項、非互動缺旗標在 Core 呼叫前單行失敗；不得新增 terminal crate、stdin payload 或 `--json`，並以 2.1 指令確認全部轉綠。 <!-- speclink-task:tsk_01KY92CSK3FFP35C4C786JGQYK -->
- [x] 2.3 REFACTOR — 更新 repository 內受 breaking 行為影響的 CLI integration fixtures，使非互動 init 明示 `--tools`，並執行 `cargo test -p speclink-cli`、scratchpad parity／color／twin harness；驗證只有「缺省 --tools」這項明載刻意分歧改變，顯式選集的成功輸出、ANSI／`--no-color` 與既有 JSON shape 維持基線。 <!-- speclink-task:tsk_01KY92CSK3JK17BP2DRXJ2QWNY -->

## 3. Desktop checkout 兩階段 IPC

- [x] 3.1 RED — 為「checkout 綁定驗證與 marker 寫入」、「Desktop checkout 採先檢查、後同步的兩階段 IPC」與「Desktop IPC and UI contract」在 `apps/desktop/src-tauri/src/connections.rs` 加入測試：`inspect_checkout` 零寫入、相符／不符 marker、非 Git、壞 YAML、現有 built-in tools／footprint 預選且無 Claude fallback；`bind_checkout` 覆蓋空／未知選集拒絕、既有 marker 補齊、Claude 切 Codex、custom descriptor 保留及失敗不回傳 root。執行 `cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml connections::`，確認新介面案例先失敗。 <!-- speclink-task:tsk_01KY92CSK37AJWNS9NC6TWHSDA -->
- [x] 3.2 GREEN — 在 `apps/desktop/src-tauri/src/connections.rs`、`apps/desktop/src-tauri/src/lib.rs` 實作並註冊 `inspect_checkout` 與帶 `tools: Vec<String>` 的 `bind_checkout`，重做邊界驗證後呼叫 Core reconciliation；在 `apps/desktop/src/adapter/connections.ts` 定義 camelCase `{ root: string, tools: string[] }` 與型別安全參數，確保 IPC 不攜帶 credential／token。執行 3.1 Rust 測試及 `npm test -w apps/desktop`，確認 Rust／TypeScript 接線可編譯且測試轉綠。 <!-- speclink-task:tsk_01KY92CSK3MG4XXFENES4R6JCF -->
- [x] 3.3 AUDIT／REFACTOR — 對 Desktop IPC 新參數套用 sharp-edges checklist：空陣列、重複／未知 tool、可交換的 origin／project／repo 字串、路徑逸出、壞設定與 marker 不一致皆須 fail loud，最簡呼叫不得跳過驗證或清理 custom descriptor；在 `apps/desktop/src-tauri/src/connections.rs` 以負向測試鎖定後執行 `cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml`，確認無 silent success 或危險預設。 <!-- speclink-task:tsk_01KY92CSK3AH3RN1M2WDJ9C5GF -->

## 4. Workspace chooser 工具 UI

- [x] 4.1 RED — 在 `apps/desktop/src/__tests__/workspaceChooser.test.tsx` 為 checkout folder mode 加入失敗測試：先 inspect 才顯示 Claude／Codex checkbox與既有選集、空選集時 Open disabled、submit 依序 bind 後 openRemote、busy 防重送、錯誤保留 path／選集且不 openRemote；測試名稱引用「Desktop IPC and UI contract」，執行 `npm test -w apps/desktop -- src/__tests__/workspaceChooser.test.tsx` 並確認新案例先失敗。 <!-- speclink-task:tsk_01KY92CSK3Y4A034T9H6JW9D93 -->
- [x] 4.2 GREEN — 在 `apps/desktop/src/components/WorkspaceChooser.tsx`、`apps/desktop/src/adapter/connections.ts`、`apps/desktop/src/i18n/messages.ts` 實作 folder path 與 built-in tool 選集狀態、繁中／英文標籤、disabled／busy／retry 行為，讓所有受管同步成功後才呼叫 openRemote；執行 4.1 測試與 `npm run build -w apps/desktop`，確認 UI 契約全綠且 production build 成功。 <!-- speclink-task:tsk_01KY92CSK3AVS0MDBMGQ3B1Z7C -->

## 5. 既有 marker 與分頁恢復 gate

- [x] 5.1 RED — 為「remote marker 資料夾的探測分流」與「失敗不開啟 Workspace並以可重試收斂取代跨檔回滾」在 `apps/desktop/src/__tests__/remoteOpen.test.ts`、`apps/desktop/src/__tests__/App.test.tsx`、`apps/desktop/src/store.ts` 的外部行為加入失敗測試：有 built-in tools 先 reconciliation 後 handshake、缺少選集導入預填 checkout chooser、同步失敗不建 tab／session且不 handshake、並存衝突的「以 server 為準」先同步再開啟；執行對應 Vitest 檔並確認新案例先失敗。 <!-- speclink-task:tsk_01KY92CSK3PBCRKPYYR780ZR9R -->
- [x] 5.2 GREEN — 在 `apps/desktop/src/store.ts`、`apps/desktop/src/App.tsx`、`apps/desktop/src/main.tsx` 與 Workspace chooser intent 接線加入 Remote binding／tab recovery reconciliation gate，使有選集自動補齊、無選集要求明示選擇、錯誤可重試且使用者內容不被回滾；執行 5.1 測試與完整 `npm test -w apps/desktop`，確認所有入口均在成功同步前不建立 remote session。 <!-- speclink-task:tsk_01KY92CSK3JSYKSZ97N5BJR4KD -->

## 6. 跨入口一致性與交付驗證

- [x] 6.1 RED — 為「Remote Workspace bootstrap 跨入口一致性」新增跨入口整合案例，對等價 Git checkout 分別執行 CLI Remote init 與 Desktop bind，斷言 built-in tools、Remote marker、Skills、Remote `AGENTS.md`／`CLAUDE.md` 正典內容同構且都不建 `openspec/`；測試同時覆蓋重試冪等與 Server 內容零修改，執行 `cargo test -p speclink-cli --test remote_connect` 及 Desktop Tauri 測試並確認缺少統一接線時先失敗。 <!-- speclink-task:tsk_01KY92CSK37MRH4E5Z4CJW5BDN -->
- [x] 6.2 GREEN／REFACTOR／AUDIT — 修正 6.1 暴露的最後接線差異，依「Acceptance criteria and scope boundaries」完成三角色 sharp-edges audit，並執行 `cargo test --workspace`、`npm test -w apps/desktop`、`npm run build -w apps/desktop`、`speclink analyze unify-agent-tool-bootstrap --json`、`speclink validate unify-agent-tool-bootstrap`；最後手動驗收新 Git checkout、既有 marker 缺 Skills、Claude 切 Codex三條流程，確認非互動零寫入、使用者內容保留、失敗不開啟與安全重試，且 Critical／Warning 為 0。 <!-- speclink-task:tsk_01KY92CSK3PMKJCG4JH17JP51P -->
