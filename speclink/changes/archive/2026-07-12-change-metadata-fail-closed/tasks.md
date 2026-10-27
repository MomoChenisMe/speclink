## 1. 解析點與診斷欄位（design 決策一：from_text 改回 Result，Change 帶 meta_error 診斷欄位）

- [x] 1.1 撰寫失敗測試：ChangeMeta::from_text 對「存在但壞 YAML」回 Err（帶解析原因）、None／空文件／欄位缺席回 Ok 預設（crates/speclink-core/src/model.rs 的 #[cfg(test)]）；Store 建構的 Change 對壞檔以 meta 預設值＋meta_error 承載解析原因且照常出現在 list_changes（crates/speclink-fs/tests/store_fs.rs）。cargo test -p speclink-core -p speclink-fs 觀察紅燈。
- [x] 1.2 實作簽名變更並編譯跟進：from_text 回 Result、Change 新增 meta_error 欄位（crates/speclink-core/src/model.rs），全部呼叫點以編譯錯誤窮舉修正（crates/speclink-fs/src/lib.rs、crates/speclink-core/src/teststore.rs、crates/speclink-node/src/store_bridge.rs、apps/desktop/core/src/cache.rs、apps/desktop/core/src/manage.rs），1.1 轉綠；驗證 cargo build --release 全 workspace 編譯通過。

## 2. core 守門：生命週期與討論（design 決策二：守門下沉 core 流程函式，命令層只做錯誤碼映射）

- [x] 2.1 撰寫「壞 metadata 使生命週期寫入 fail closed」的失敗測試：in-progress add 與 claim 拒絕且 .openspec.yaml 逐位元不變（未疊寫 started_* 行）、task done 與 task undone 拒絕且 tasks.md 不變、discard 未帶與帶 --force 皆拒且 change 目錄保留、archive 拒絕且正典未併入、new artifact 拒絕不以預設 schema 解析（crates/speclink-core/src/inprogress.rs、crates/speclink-core/src/discard.rs、crates/speclink-core/src/command/mod.rs 測試）。紅燈。
- [x] 2.2 實作 MetaError 錯誤型別（帶 change 名、workspace 相對路徑與解析原因）與各流程函式守門，2.1 轉綠；驗證 cargo test -p speclink-core 全綠。
- [x] 2.3 撰寫並轉綠「討論鏈結動詞對壞 change metadata 拒絕」的測試：discuss link 對壞檔 change 拒絕且 .openspec.yaml 與討論記錄逐位元不變；discuss seal 的錯誤指出 metadata 損壞而非 from_discussion 鏈缺失（crates/speclink-core/src/discuss.rs 測試）。

## 3. 命令層映射與 list 診斷（design 決策三：錯誤碼沿用 invalid_config，不擴碼；決策四：list 對 invalid change 的診斷呈現）

- [x] 3.1 撰寫失敗測試，覆蓋「change metadata 損壞的跨入口處置」與「穩定錯誤碼註冊表」新增的 .openspec.yaml 情境：runtime 對壞檔 change 的 status／instructions／validate／analyze／drift／artifact cat 回錯誤碼 invalid_config 且訊息含 workspace 相對路徑與原因；list outcome 壞檔項目帶診斷、有效項目不帶且內容不變（crates/speclink-core/src/command/mod.rs 測試）。紅燈。
- [x] 3.2 實作 MetaError 到 invalid_config 的命令層映射與 list 診斷 outcome，3.1 轉綠；驗證 cargo test -p speclink-core 全綠。
- [x] 3.3 撰寫並轉綠 CLI 整合測試：speclink list 人眼輸出於壞檔行附 invalid 標記、--json 該項含 metaError 欄位（camelCase、字串型別、僅壞檔情境出現，其餘欄位形狀不變）；status 對壞檔非零 exit code 且 stderr 指出檔案與原因；in-progress add 與 discard --force 對壞檔拒絕（crates/speclink-cli/tests/ 沿用既有整合測試佈局，渲染改動落 crates/speclink-cli/src/commands.rs）。
- [x] 3.4 驗證輸出凍結（design 決策六：輸出凍結與壞檔情境的測試邊界）：有效 metadata workspace 的 parity 31 項／color 16 項／twin 8 情境回歸對照全綠；cargo test -p speclink-cli 全綠。

## 4. Node dispatch 一致性

- [x] 4.1 撰寫並轉綠 vitest：dispatch(['status', '--change', 壞檔 change]) 以 Error 拒絕、code 為 invalid_config、message 與 CLI 訊息文字相同；dispatch(['list']) 壞檔項目帶 metaError 欄位（crates/speclink-node/__test__/）。驗證 crates/speclink-node 內 npm run build 與 npm test 全綠。

## 5. 桌面看板防護（design 決策五：看板補章與排序寫入的防護）

- [x] 5.1 撰寫「壞 metadata 不參與看板排序寫入」的失敗測試：set_board_rank 於文字手術前拒絕且 .openspec.yaml 逐位元不變（crates/speclink-core/src/model.rs 測試）；欄內補章排除 invalid 卡且其餘缺 rank 有效卡照常補章、看板 payload 對壞檔卡帶 invalid 標記且看板照常列出全部卡片（apps/desktop/core/src/manage.rs、apps/desktop/core/src/cache.rs 測試）。紅燈。
- [x] 5.2 實作排序寫入守門、補章排除與 board payload 的 invalid 欄位，5.1 轉綠；UI 卡片顯示最小 invalid 標記並以 vitest 斷言標記渲染（packages/ui/src/adapter.ts 型別、packages/ui/src/components/ChangeCard.tsx）。驗證 npm test -w packages/ui 與 npm test -w apps/desktop 全綠。

## 6. 全量回歸收尾

- [x] 6.1 root 單一指令全量驗證：npm run test:all 全綠（Rust workspace、packages/ui、apps/desktop、crates/speclink-node 四面，符合 delivery-baseline 交付前提）；git grep 確認 ChangeMeta 解析僅剩 model.rs 單一 typed 入口，無殘留 fail-open 呼叫。
