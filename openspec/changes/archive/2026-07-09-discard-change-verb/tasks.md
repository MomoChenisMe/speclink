## 1. 前置：相依確認

- [x] 1.1 確認 rediscuss-promoted-change 的 from_discussion 累積器已實作（speclink-core 存在逗號清單分割讀取且 cargo test -p speclink-core --lib 全綠）；尚未實作則先完成該變更再動工本變更——本變更的解鏈逐 slug 建立在該讀取函式上

## 2. store 層：變更刪除能力

- [x] 2.1 撰寫失敗測試（crates/speclink-fs/tests/store_fs.rs）：store 刪除變更後目錄消失、變更清單不再含該名、目錄內含子目錄與多檔時整棵移除；驗證：cargo test -p speclink-fs 新案例紅燈
- [x] 2.2 實作 design「D3 — Store trait 新增變更刪除方法」：crates/speclink-core/src/store.rs 增刪除變更方法（沿 delete_live_discussion 先例）、crates/speclink-fs/src/lib.rs 以 std::fs 遞迴刪除實作、crates/speclink-node/src/store_bridge.rs 對映 JS 回呼；驗證：cargo test -p speclink-fs 與 cargo test -p speclink-core --lib 全綠

## 3. core：討論解鏈與狀態回退

- [x] 3.1 撰寫失敗測試（crates/speclink-core/src/discuss.rs 的 #[cfg(test)]）：spec「討論隨變更廢棄解鏈」全情境——唯一值移除後回退 concluded（promoted_to 行消失）、多值僅縮減維持 promoted、Conclusion 空的記錄回退 open、缺失 slug 跳過不失敗、Context/Rounds/Conclusion 逐位元不變、對已解鏈討論重跑冪等；驗證：cargo test -p speclink-core --lib 新案例紅燈
- [x] 3.2 實作 design「D2 — 解鏈與狀態回退」：promoted_to 逗號清單移除該變更名、空清單移除整行並依 Conclusion 區非空與否回退 concluded／open（沿 mark_promoted 的字串替換模式）；驗證：cargo test -p speclink-core --lib 全綠

## 4. core 編排與 CLI：discard 動詞

- [x] 4.1 撰寫失敗測試（新模組 crates/speclink-core/src/discard.rs 的 #[cfg(test)]）：spec「變更以 discard 動詞廢棄」——動工痕跡守衛矩陣（started_at × 已勾任務四格）、守衛拒絕時零寫入、force 放行、成功路徑刪除變更目錄與 touched 紀錄、解鏈先於目錄刪除、目錄刪除失敗時已完成解鏈不回滾且結果明示已解鏈清單、變更不存在報錯；驗證：cargo test -p speclink-core --lib 新案例紅燈
- [x] 4.2 實作 design「D1 — 頂層動詞 discard 與動工痕跡守衛」：core 編排（守衛 → 逐 slug 解鏈 → 刪 touched 紀錄 → 刪變更目錄 → 回報結果），鏡射 archive 的頂層動詞模組模式；驗證：cargo test -p speclink-core --lib 全綠
- [x] 4.3 CLI 佈線：crates/speclink-cli/src/main.rs 增 discard 子指令（位置參數變更名、--force、--json）、crates/speclink-cli/src/commands.rs 的人眼輸出（報告刪除的變更與每份解鏈討論的 slug 及回退後狀態；--no-color 無 ANSI）與 --json camelCase payload、crates/speclink-cli/src/remote_commands.rs 於 remote 模式報不支援（鏡射 discuss discard 訊息模式）；驗證：cargo build --release -p speclink-cli 通過，沙盒實跑成功、守衛拒絕、--force、變更不存在、--json 五條路徑輸出符合 spec 情境
- [x] 4.4 回歸快掃：discard 為純新增動詞，既有指令輸出零變動——以變更前 baseline exe 抽查 archive、list、discuss list 的人眼與 --json 輸出逐位元一致；驗證：diff 無差異

## 5. 文件與收尾

- [x] 5.1 依 design「D4 — README 文件同步，遠端契約不動」更新文件：README.md 與 README.en.md 的「指令參考——變更生命週期」表各補 discard 一列（含守衛與 --force 說明）、「SDD 工作流」節補一句砍掉另開流程（discard 後討論回 concluded、可再轉出後繼變更）；驗證：內容審視兩份語意一致，docs/verb-contract.md 與 docs/verb-contract.zh-TW.md 無變動
- [x] 5.2 端對端「砍掉另開」流程驗證：沙盒中討論 promote 成變更 c1 → speclink discard c1 → 討論 frontmatter 回 status: concluded 且無 promoted_to 行 → 對同討論 promote --name c2 → promoted_to 僅含 c2；驗證：逐步 CLI 輸出與檔案效果符合 spec「討論隨變更廢棄解鏈」與「變更以 discard 動詞廢棄」
- [x] 5.3 全面回歸與 artifact 驗證：cargo test --workspace --lib 全綠；speclink validate discard-change-verb 通過
