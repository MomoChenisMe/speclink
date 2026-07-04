## 1. 前置確認

- [ ] 1.1 確認 verb-contract-and-remote-client 已完成歸檔：執行 speclink list --json 確認不含該 change、openspec/specs/remote-connection/spec.md 已存在（含「連接檔與模式解析」等需求）；接著執行 speclink drift remote-section-in-speclink-yaml 確認無 stale delta assumptions。任一條件不成立即停止並回報——本 change 不可先行動工。（影響路徑：openspec/changes/remote-section-in-speclink-yaml/）

## 2. 設定結構（speclink-core）

- [ ] 2.1 撰寫 remote 區段解析測試：.speclink.yaml 含 remote 區段（url 與 repo、僅 repo、空區段）時應用設定各欄位的解析結果；無 remote 鍵的既有檔案照常解析且區段為缺席。預期：新測試先失敗（紅）。（crates/speclink-core/src/config.rs 的 tests 模組）
- [ ] 2.2 實作應用設定的 remote 選填區段（serde default、向後相容），使 2.1 測試轉綠。驗證：cargo test -p speclink-core 全綠。（crates/speclink-core/src/config.rs）

## 3. 模式解析（speclink-core）

- [ ] 3.1 撰寫「remote 區段與模式解析」需求的測試：有 remote 區段→remote 模式；無區段→fs 模式；區段存在但區段 url 與 SPECLINK_STORE_URL 皆缺→回傳明確錯誤（含兩種設定方式的提示資料）；SPECLINK_STORE_URL 存在→覆寫區段 url；專案根含殘留 .speclink.remote.yaml→偵測結果旗標殘留且不影響模式判定。預期：新測試先失敗（紅）。（crates/speclink-core/src/workspace.rs 的 tests 模組）
- [ ] 3.2 實作「remote 區段與模式解析」需求：發現邏輯改讀 .speclink.yaml 的 remote 區段、移除獨立連接檔的解析路徑、新增殘留檔偵測（core 回傳結構化結果，不含呈現字串）。驗證：cargo test -p speclink-core 全綠。（crates/speclink-core/src/workspace.rs、crates/speclink-core/src/config.rs）

## 4. CLI 檔案效果與警告（speclink-cli、speclink-core）

- [ ] 4.1 撰寫「remote 初始化與連接指令」需求的整合測試：speclink init --store remote 在空目錄生成含 remote 區段的 .speclink.yaml（不建 openspec/、不建 .speclink.remote.yaml）；speclink link 寫入／更新區段且保留既有 tools 欄位值；speclink unlink 移除區段、保留其他欄位、後續指令回到 fs 模式。預期：新測試先失敗（紅）。（crates/speclink-cli/tests/remote_section.rs）
- [ ] 4.2 實作 init --store remote、link、unlink 的檔案效果（讀取—修改—寫回 .speclink.yaml 的 remote 區段），使 4.1 測試轉綠。驗證：cargo test -p speclink-cli 全綠。（crates/speclink-core/src/init.rs、crates/speclink-cli/src/commands.rs）
- [ ] 4.3 撰寫「殘留連接檔的遷移警告」需求的測試：專案根含 .speclink.remote.yaml 時任一指令的 stderr 恰有一行以 speclink: warning: 開頭的遷移警告（含搬移指引），stdout 與 exit code 不受影響；url 兩處皆缺時 exit code 非 0 且 stderr 同時提示 remote.url 與 SPECLINK_STORE_URL。預期：新測試先失敗（紅）。（crates/speclink-cli/tests/remote_section.rs）
- [ ] 4.4 實作「殘留連接檔的遷移警告」的輸出（cli 層組裝訊息文字），使 4.3 測試轉綠。驗證：cargo test 全 workspace 綠。（crates/speclink-cli/src/commands.rs）

## 5. 文件與收尾驗證

- [ ] 5.1 更新團隊模式雙語文件的連接設定段落：remote 區段格式、模式訊號、SPECLINK_STORE_URL 用法、自 .speclink.remote.yaml 的遷移步驟與註解不保留的限制。驗證：兩份文件內容對應一致。（docs/team-mode.md、docs/team-mode.zh-TW.md）
- [ ] 5.2 全面驗證：cargo build --release 與 cargo test 全綠；fs 模式回歸對照（parity_suite 31 項／color_suite 16 項）通過且無刻意更新；手動煙霧測試——speclink link 後 .speclink.yaml 出現 remote 區段且 speclink list --json 形狀不變、speclink unlink 後回到 fs 模式、放置殘留 .speclink.remote.yaml 得到恰一行警告。（crates/、docs/）
- [ ] 5.3 確認「repo 身分攜帶與歸屬防呆」與「git remote 參考值的輔助警告」兩需求的行為不變——本 change 對兩者僅為措辭級規格更新（repo 名讀取來源改為 remote 區段），不動其實作；以既有整合測試通過為驗證。（crates/speclink-cli/tests/）

## 6. 設計對應（design 決策 ↔ 任務）

- Decision 1: remote 區段存在即模式訊號（不設 type 欄位）→ 任務 3.1、3.2
- Decision 2: url 可缺省，由 SPECLINK_STORE_URL 供給；兩處皆缺為明確失敗 → 任務 3.1、4.3
- Decision 3: 殘留 .speclink.remote.yaml 不解析、僅單行警告 → 任務 4.3、4.4
- Decision 4: serde 結構與向後相容 → 任務 2.1、2.2
- Decision 5: core / cli 邊界 → 任務 3.2、4.4
- 命名慣例 → 任務 4.2、5.1
