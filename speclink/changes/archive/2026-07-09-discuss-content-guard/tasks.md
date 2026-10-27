## 1. core 空內容 guard

- [x] 1.1 [Red] 於 crates/speclink-core/src/discuss.rs 測試模組寫失敗測試，實現 D1：空內容 guard 置於 core::discuss 而非各 CLI handler——add_round／conclude／set_context 對空及純空白內容回 Err 且不改動檔案、conclude 遇空不翻 status。驗證：`cargo test -p speclink-core`（Windows 如遇 cdylib 連結問題以 `--lib` 限縮）見紅。
- [x] 1.2 [Green] 於 add_round／conclude／set_context 開頭加 content.trim().is_empty() → bail（訊息含空內容與 --stdin 提示）令 1.1 轉綠；同時保持 D3：Round 維持純 append-only（不新增 fill／edit 動詞）。驗證：`cargo test -p speclink-core`。

## 2. CLI stdin 讀取條件

- [x] 2.1 [Red] 於 crates/speclink-cli/tests 寫失敗整合測試，實現 D2：CLI 以 IsTerminal 判管線讀取 stdin，--stdin 降為相容旗標——涵蓋「管線非空內容未帶 --stdin 仍正確寫入」與「管線空內容 → 錯誤退出、不寫空區段」。驗證：`cargo test -p speclink-cli` 相關整合測試見紅。
- [x] 2.2 [Green] 將 commands 與 remote_commands 的 discuss context／add-round／conclude 改為「stdin 非互動終端（std::io::IsTerminal）即讀取」、--stdin 保留為被接受旗標，令 2.1 轉綠並使「討論內容寫入動詞拒絕空內容」需求成立。驗證：`cargo test -p speclink-cli`。

## 3. 收尾與回歸

- [x] 3.1 [Refactor] 檢視 guard 與 stdin 讀取變更：確認錯誤訊息一致、未觸及 discuss 以外寫入路徑、--json 輸出契約不變，並套用 sharp-edges 稽核確認未引入新的靜默失敗或危險預設。驗證：`cargo test -p speclink-core` 與 `cargo test -p speclink-cli` 全綠。
