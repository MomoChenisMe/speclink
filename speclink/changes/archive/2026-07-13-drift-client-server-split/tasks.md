## 1. WorkspaceFacts 型別與規格面拆分（design 決策一：三段純函式留在 core 的 drift 模組，蒐集器落 host；決策二：WorkspaceFacts 為封閉輸入結構，缺席即 unavailable）

- [x] 1.1 撰寫失敗測試，覆蓋「drift 運算拆分為規格面與工作區面純函式」的規格面：compute_spec_drift 以測試 Store 產出 Specs 維度與規格假設、相同輸入重複呼叫結果逐欄相同、不執行任何 git 程序；WorkspaceFacts 封閉結構的三值語意（有值／空值／不可用）逐欄可表達（crates/speclink-core/src/drift.rs 的 #[cfg(test)]）。cargo test -p speclink-core 觀察紅燈。
- [x] 1.2 實作 WorkspaceFacts 型別與 compute_spec_drift 純函式（自現行 analyze 抽出 Specs 維度與 spec_assumptions 消費路徑），1.1 轉綠；驗證 cargo test -p speclink-core 全綠。

## 2. 工作區面運算與 unavailable 語意（design 決策二）

- [x] 2.1 撰寫失敗測試，覆蓋「WorkspaceFacts 缺席時四維度標 unavailable」：缺席時 Time／Structure／Tasks／Environment 標 unavailable 且不帶分數；「有 checkout 但 git 不可用」的 facts 產出與現行 git-unavailable fallback 字串與分數逐位元一致；有值 facts 產出與現行四維度逐欄一致（crates/speclink-core/src/drift.rs 測試，輸入自現行 analyze 消費面逆推）。紅燈。
- [x] 2.2 實作 compute_workspace_drift 純函式（自現行 analyze 抽出四維度運算，git 呼叫全數改讀 facts），2.1 轉綠；驗證 cargo test -p speclink-core 全綠。

## 3. 共用 merger 與 coverage／stale（design 決策三：merger 是唯一合併與 coverage／stale 裁決點）

- [x] 3.1 撰寫失敗測試，覆蓋「單一 merger 裁決合併、coverage 與 stale」：full coverage 合併結果與現行 DriftReport 逐欄一致；工作區面缺席時 coverage 為 spec-only 且四維度保留 unavailable 條目；basis digests 不符時標 stale 並僅列不符項；coverage 與 stale 為非常態才出現的選填欄位（crates/speclink-core/src/drift.rs 測試）。紅燈。
- [x] 3.2 實作 merge_drift_reports 與 CombinedDriftReport（serde camelCase 選填欄位），3.1 轉綠；驗證 cargo test -p speclink-core 全綠。

## 4. host 蒐集器與 DriftBundle（design 決策四：DriftBundle 由 host 產生，內容對齊藍圖 §6.5）

- [x] 4.1 撰寫失敗測試，覆蓋「DriftBundle 固定漂移檢查的基準」與蒐集器行為：produce_drift_bundle 內容齊備（binding、change 名、spec／tasks／policy basis digests、created metadata、design 與 tasks 內容、evidence 摘要、產生時間）且同一狀態重複產生 basis digests 逐項相同；WorkspaceFacts 蒐集器對 git 可用與不可用的 workspace 分別產出正確三值 facts；完整流程結束後 workspace 無任何檔案被寫入（crates/speclink-host/src/drift.rs 的 #[cfg(test)]）。紅燈。
- [x] 4.2 實作 host 蒐集器（drift 專用 git 輔助函式自 crates/speclink-core/src/util.rs 隨遷，遷移前以 git grep 盤點呼叫者、共用者不動）與 produce_drift_bundle，4.1 轉綠；驗證 cargo test -p speclink-host 與 cargo test -p speclink-core 全綠。

## 5. 本地串接與輸出凍結（design 決策五：本地 cmd_drift 三段串接、輸出逐位元凍結）

- [x] 5.1 建置並保存 baseline exe（cargo build --release -p speclink-cli）於非 scratchpad 位置，準備涵蓋 git 可用、git 不可用、無 design、broken anchors 四情境的樣本 workspace 並記錄對照步驟。
- [x] 5.2 cmd_drift 與命令層 drift 查詢改為三段串接（host 蒐集 → 兩段運算 → merger → 現行渲染；crates/speclink-cli/src/commands.rs、crates/speclink-core/src/command/mod.rs），覆蓋「本地 drift 路徑輸出凍結」：四情境的人眼與 --json 輸出、exit code 與 baseline 逐位元一致；移除 analyze 舊單體路徑的孤兒碼；驗證 parity 31 項／color 16 項／twin 8 情境全綠。

## 6. 全量收尾

- [x] 6.1 對新公開 API 跑 sharp-edges 稽核檢查表（speclink instructions --skill audit）並修正發現；驗證 cargo test --workspace 與 npm run test:all 全綠；git diff --stat 對照 proposal Impact 清單檢查改動面無溢出。
