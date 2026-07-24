## Context

目前 scripts/dev.mjs 會驗證 dev 設定、建置 Desktop 前端，再啟動 speclink-server 與 Tauri Desktop 兩個長時間程序。CLI 不在這條可重現鏈中；測試者若直接執行 PATH 中的 speclink，可能使用未安裝、過期或來自另一個 checkout 的 binary。

本變更只跨 package scripts 與 Node dev harness，不進入 Rust domain 或 transport 層。speclink-cli 仍是既有 binary；speclink-core、storage abstraction、Remote Protocol、序列化設定、git 行為與 CLI 輸出契約均不變。主要使用者是透過 AI 代理或終端測試 Local／Remote SDD workflow 的開發者、PO 與 PM。

## Goals / Non-Goals

**Goals:**

- npm run dev 在任何 Server／Desktop 長時間程序啟動前，保證目前 checkout 的 debug CLI binary 已成功建置。
- 提供 npm run cli -- <args>，固定執行該 checkout 的 binary，並保留 CLI 原有的互動與退出語意。
- 讓相同流程在 Windows、macOS 與 Linux 可測，且不受 PATH 中其他 speclink 版本影響。
- 以 Node 內建測試能力遵循紅、綠、重構，不新增 dependency。

**Non-Goals:**

- 不安裝、升級、移除或切換全域 CLI，也不修改 PATH／shell profile。
- 不改動 speclink-cli 或 speclink-core 的 Rust 實作、指令、輸出、設定與 storage boundary。
- 不讓 CLI 成為 dev lifecycle 管理的長時間程序。
- 不處理 release 安裝或 production deployment。

## Decisions

### 先建置 CLI 再啟動長時間程序

scripts/dev.mjs 在既有設定驗證後，以同步 prerequisite 執行 cargo build -p speclink-cli；成功後才繼續 Desktop 前端 build，最後才 spawn Server 與 Desktop。任何 prerequisite 失敗都回傳其非零 exit code，且此時 child process 清單仍為空。npm run dev:reset 保持只重置 .dev，不觸發 build。

替代方案是讓 Server／Desktop 先啟動後背景建置 CLI；這會短暫產生「環境看似成功、CLI 尚不可用」的半完成狀態，也無法滿足 build failure 零殘留程序，因此不採用。另一替代方案是每次 CLI 呼叫時才建置，會把版本正確性延後到使用時才發現，亦不採用。

### 以 checkout 內 binary 提供 CLI wrapper

package-level 的 cli script 呼叫獨立 Node wrapper。wrapper 從自身所在 repo root 解析 target/debug/speclink；Windows 使用 speclink.exe，其他平台使用 speclink。它不查詢 PATH，也不透過 cargo install。

wrapper 將 npm 傳入的 argv 原序交給 binary，stdio 使用 inherit，環境沿用目前 process，並將 child 的 exit code 傳回 npm。工作目錄使用 npm 提供的 INIT_CWD；該值不存在時退回 process.cwd()，讓 npm --prefix 指向 Speclink repo 時，CLI 仍可作用於呼叫者所在的測試 repo。binary 無法啟動或因 signal 結束時，以可讀 stderr 與非零 exit code 呈現，不靜默改用 PATH CLI。

一般互動可使用 npm run cli -- <args>；需要純 machine-readable stdout 時，文件 SHALL 使用 npm run --silent cli -- <args>，排除 npm 自身的 lifecycle 訊息。wrapper 本身不增加任何 stdout 內容。

替代方案是 npm script 直接寫 target/debug/speclink；這無法一致處理 Windows 副檔名、INIT_CWD 與錯誤轉送。另一替代方案是 cargo run -p speclink-cli -- <args>，每次呼叫都會經過 Cargo 且其輸出混入 CLI 互動，故不採用。

### 以小型可注入函式測試程序邊界

延續 dev.mjs 既有純函式測試風格，只抽出 prerequisite 執行與 CLI wrapper 啟動所需的最小函式，讓測試注入假的 spawn／platform／cwd。測試觀察命令順序、binary path、argv、stdio、cwd、exit code 與失敗時未 spawn 長時間程序；不啟動真實 Server、Desktop 或 CLI。

替代方案是新增 process orchestration framework 或只做端到端測試。前者超出需求，後者慢且難以穩定覆蓋 Windows path 與失敗分支，因此皆不採用。這個選擇不引入新的流程抽象，也不影響 storage 解耦的規格引擎。

## Implementation Contract

**Behavior**

- npm run dev 驗證設定後，先成功建置目前 checkout 的 speclink-cli，再建置 Desktop 前端並啟動 Server／Desktop。
- CLI build 回傳非零狀態或無法啟動時，npm run dev 顯示失敗、回傳非零狀態，且不啟動 Server／Desktop。
- npm run cli -- <args> 執行目前 checkout 的 target/debug/speclink 或 target/debug/speclink.exe；PATH 中是否存在同名 CLI 不影響選擇。
- npm run dev 的長時間 child 與 signal 收束契約維持兩個程序；npm run dev:reset 的行為不變。

**Interface / data shape**

- 新 package script：cli，呼叫 Node wrapper；`<args>` 不解析、不改寫，直接傳給既有 Speclink CLI。
- wrapper stdin、stdout、stderr 全部 inherit；不建立新 JSON envelope，也不改變既有 --json camelCase payload、--no-color 或人眼輸出。
- 需要直接解析 --json 或比對位元級輸出時，呼叫端使用 npm run --silent cli -- <args>，避免 npm lifecycle 訊息混入 stdout。
- wrapper child cwd 為 INIT_CWD，缺席時為 process.cwd()；環境變數原樣繼承。
- 不新增 `.speclink.yaml`、openspec/config.yaml、serde schema、protocol API、skill 或 injection block。

**Failure modes**

- CLI prerequisite build 失敗：回傳原狀態；若無可用狀態則回傳 1；不啟動長時間 child。
- checkout binary 缺失、不可執行或 spawn error：stderr 明確指出無法執行 checkout CLI，exit code 非零，不 fallback 到 PATH。
- CLI 本身正常以非零狀態結束：wrapper 傳回相同狀態；stdin/stdout/stderr 仍由 CLI 直接呈現。

**Acceptance criteria**

- Node tests 證明 CLI build 發生在 Server／Desktop spawn 之前，並證明 build failure 時長時間 child spawn 次數為零。
- Node tests 在 win32 與非 win32 情境確認 binary path，並確認 PATH 中的假 speclink 不會被選用。
- Node tests 證明 argv、stdio、INIT_CWD／fallback cwd 與 exit code 轉送。
- 既有 dev config tests、root test:all 與 Speclink strict validation 維持通過。
- Remote getting started 中的命令可讓沒有全域 CLI 的測試者，以同一 checkout 完成 CLI 驗證。

**Scope boundaries**

實作限於 package scripts、Node dev harness／wrapper、其測試及操作文件。不得改動 Rust CLI 行為、Server／Desktop runtime、Remote 認證／membership、資料格式、git 互動或使用者安裝狀態。

## Risks / Trade-offs

- [Risk] npm run dev 首次啟動增加 CLI build 時間 → 顯示明確建置階段，並只建置 speclink-cli package，後續由 Cargo incremental build 降低成本。
- [Risk] Windows executable path 與 shell 行為不同 → 直接解析 .exe，不依賴 shell path expansion，並以 platform-injected tests 覆蓋。
- [Risk] npm 改變 script cwd，導致 CLI 操作錯誤 repo → 優先使用 INIT_CWD，文件提供從測試 repo 搭配 npm --prefix 的範例。
- [Risk] wrapper 吞掉 CLI 輸出或狀態，破壞 parity → stdio inherit、args 不解析、exit code 原樣回傳，並以程序邊界測試保護；既有人眼與 --json fixture 不需更新。
- [Trade-off] wrapper 依賴 npm 的 INIT_CWD 來還原呼叫端 cwd；直接在 repo root 執行時仍以 repo root 為 cwd，這與使用者實際呼叫位置一致。

## Migration Plan

不需資料或設定 migration。合併後，開發者可繼續使用 npm run dev；首次啟動會新增 CLI build gate。若需回滾，只需移除 cli package script／wrapper 與 dev prerequisite，既有全域 CLI、Rust build artifact 與 .dev 資料皆不受影響。

## Open Questions

無。
