## 1. Store 介面與 fs 實作（speclink-fs）

- [ ] 1.1 撰寫 fs 儲存實作的行為測試：於 `crates/speclink-fs/tests/store_fs.rs` 建立暫存專案 fixture，斷言 change 列舉與排序（updated_at 截整秒、由新至舊）、artifact 讀寫與存在檢查、缺檔／缺目錄回空清單或預設值、損壞 .openspec.yaml 回預設 metadata、discussion 建立與附加——此時尚無實作，測試應編譯失敗或紅燈
- [ ] 1.2 實作使測試轉綠：於 `crates/speclink-core/src/store.rs` 定義同步、object-safe 的儲存介面（領域動詞方法集：列舉 changes 與 metadata、artifact 讀／寫／存在、delta spec 與能力列舉、正典 spec 讀寫、archive 搬移、discussion 建立／讀取／附加／歸檔、workflow-config 讀取、updated_at 查詢）；於 `crates/speclink-fs/src/lib.rs` 與 `crates/speclink-fs/src/layout.rs` 平移 `crates/speclink-core/src/paths.rs` 的規格佈局知識並實作介面；`Cargo.toml` workspace members 加入 speclink-fs、`crates/speclink-fs/Cargo.toml` 補齊 package 欄位與對 core 的依賴
- [ ] 1.3 重構：以 cargo clippy --workspace 檢視介面命名與方法集（嚴格對應現行 fs 呼叫盤點，不為未來情境預加方法），清理平移殘留
- [ ] 1.4 驗證：執行 cargo test -p speclink-fs 全綠；cargo build --workspace 成功（覆蓋需求：預設檔案系統佈局不變）

## 2. core 各流程模組改接儲存介面

- [ ] 2.1 撰寫「core 不得直接存取規格目錄」檢查測試：於 `crates/speclink-core/tests/no_direct_fs.rs` 掃描 crates/speclink-core/src/ 原始碼，斷言除 workspace.rs（宿主側路徑）與 util.rs（通用寫檔工具）外不存在對 spec 目錄內容的 std::fs 呼叫——此時為紅燈
- [ ] 2.2 改接 model 與 discuss：`crates/speclink-core/src/model.rs`、`crates/speclink-core/src/discuss.rs` 的檔案操作替換為介面呼叫（影響指令：speclink list、speclink discuss 系列）；執行 cargo test 全綠
- [ ] 2.3 改接 status、validate、analyzer：`crates/speclink-core/src/status.rs`、`crates/speclink-core/src/validate.rs`、`crates/speclink-core/src/analyzer.rs`（影響指令：speclink status、speclink validate、speclink analyze）；執行 cargo test 全綠
- [ ] 2.4 改接 newcmd、tasks、inprogress、preflight：`crates/speclink-core/src/newcmd.rs`、`crates/speclink-core/src/tasks.rs`、`crates/speclink-core/src/inprogress.rs`、`crates/speclink-core/src/preflight.rs`（影響指令：speclink new change、speclink new artifact、speclink task 系列）；執行 cargo test 全綠
- [ ] 2.5 改接 archive 與 drift 並收尾：`crates/speclink-core/src/archive.rs`、`crates/speclink-core/src/drift.rs` 的規格文件搬移改經介面（git 查詢行為不變，影響指令：speclink archive、drift 相關）；宿主側路徑（.speclink/ 工作目錄、touched、snapshots、專案根 walk-up 探索）移入新檔 `crates/speclink-core/src/workspace.rs`；刪除 `crates/speclink-core/src/paths.rs` 並更新 `crates/speclink-core/src/lib.rs` 模組宣告——2.1 的檢查測試轉綠
- [ ] 2.6 重構：清理各模組平移後的重複程式碼與無用 re-export；cargo clippy --workspace 無新警告
- [ ] 2.7 驗證：cargo test 全 workspace 綠燈，且既有測試檔未修改任何斷言

## 3. CLI 注入與回歸對照

- [ ] 3.1 CLI 組裝點注入 fs 實作：`crates/speclink-cli/src/main.rs` 與 `crates/speclink-cli/src/commands.rs` 於指令進入點建立 fs 儲存實作並傳入 core 流程（所有 speclink 指令）；cargo test 全綠
- [ ] 3.2 驗證（回歸對照）：執行 scratchpad 的 parity_suite（31 項）、color_suite（16 項）、twin harness（8 情境），預期全數通過；任何不一致視為本 change 的缺陷修復後重跑（覆蓋需求：儲存重構後既有指令行為保持不變）

## 4. 雙語文件骨架與 README

- [ ] 4.1 撰寫架構篇：`docs/architecture.md`（英文）與 `docs/architecture.zh-TW.md`（繁體中文），內容涵蓋引擎—Store—呈現三層、儲存介面縫線位置、speclink-fs 的角色、與後續 config-system-rework／verb-contract-and-remote-client／node-sdk 三個 change 的關係圖；兩語版語意對等
- [ ] 4.2 撰寫入門篇：`docs/getting-started.md` 與 `docs/getting-started.zh-TW.md`，以純本地情境走完 speclink init → discuss → propose → apply → verify → archive 一輪，含每步的指令與預期輸出摘要；`README.md` 新增 Documentation 章節，雙語連結上述四份文件
- [ ] 4.3 驗證：README 引用的四個文件路徑皆存在；cargo build --release 成功且 speclink --version 正常回應（覆蓋需求：工作區建置包含預設儲存實作）
