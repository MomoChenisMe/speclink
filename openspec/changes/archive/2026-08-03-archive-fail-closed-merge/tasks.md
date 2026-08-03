<!-- 開發依 tdd-workflow：先寫失敗測試（紅燈），再實作（綠燈），最後重構與收尾。 -->

## 1. 守門與合併計畫的紅燈測試

- [x] 1.1 依規格需求「封存合併 fail-closed 守門」與設計「違規清單與聚合錯誤形狀」，於 crates/speclink-core/src/archive.rs 測試模組新增六條違規的失敗測試：ADDED 撞名、MODIFIED／REMOVED／RENAMED 缺目標、同需求多操作區段、RENAMED 目標已存在、新 capability 出現非 ADDED、以及「多條違規聚合一次回報且錯誤含 capability／操作／需求名／原因與 drift → ingest 補救指引」。完成時 cargo test -p speclink-core 出現對應紅燈（現行為靜默跳過故斷言失敗）。 <!-- speclink-task:tsk_01KZ3CGZ9NZGWJHTR25J9306NM -->
- [x] 1.2 依設計「凍結測試翻轉」，翻轉 crates/speclink-core/src/archive.rs 既有凍結測試：「ADDED 已存在跳過」「MODIFIED 缺目標跳過」「正典不存在時 MODIFIED 物化成新規格」改為拒絕斷言；同批確認 --no-validate 情境仍拒絕、--skip-specs 情境不觸發守門。完成時翻轉後測試為紅燈、其餘 archive 測試不受影響。 <!-- speclink-task:tsk_01KZ3CGZ9N42CT4J84RBJPB7CQ -->
- [x] 1.3 依規格需求「兩階段合併計畫與零半套寫入」新增失敗測試：雙 capability 其一違規時零檔案效果（兩正典未動、無 snapshot、change 原位），與成功路徑「全部 snapshot 先於正典寫入」的順序斷言（crates/speclink-core/src/archive.rs 測試模組）。完成時兩測試紅燈。 <!-- speclink-task:tsk_01KZ3CGZ9NFPY96TY5FA2FDQ87 -->

## 2. 合併引擎實作（綠燈）

- [x] 2.1 依設計「守門落點：speclink-core 合併引擎單點裁決」與「違規清單與聚合錯誤形狀」，在 crates/speclink-core/src/archive.rs 實作 merge plan 驗證：讀全部 capability 的 delta 與正典、產出六條違規清單與逐 capability 合併結果，違規時回傳聚合錯誤（含補救指引文案），實現「封存合併 fail-closed 守門」需求。完成時 1.1 與 1.2 測試全綠，speclink archive 對違規 change 以非零 exit code 拒絕。 <!-- speclink-task:tsk_01KZ3CGZ9NP0G78ZB4SAW1YNCN -->
- [x] 2.2 依設計「兩階段合併：先產完整 merge plan、全數驗證後才寫入」，將 archive 套用改為兩階段：plan 全數通過後，依「全部 snapshot 備份 → 全部正典寫回 → change 移入封存區」順序執行寫入（crates/speclink-core/src/archive.rs），落實「兩階段合併計畫與零半套寫入」需求。完成時 1.3 測試綠燈，成功路徑的封存輸出與現行一致。 <!-- speclink-task:tsk_01KZ3CGZ9N2FAPYRRDJFPK7KQ6 -->
- [x] 2.3 依設計「scenario superset check 與明示刪除聲明」實作規格需求「MODIFIED 的 scenario 保全與明示刪除聲明」：解析正典目標需求的 scenario 名（CRLF 先正規化為 LF）、比對 delta 區塊、缺漏未聲明即列入違規並逐條點名；REMOVED-SCENARIO 聲明註解於寫入正典前剝除（crates/speclink-core/src/archive.rs，剝除與既有 BEFORE 註解處理同層）。完成時新增測試涵蓋「漏抄點名」「聲明放行且正典無註解」「CRLF 樣本」三情境並全綠。 <!-- speclink-task:tsk_01KZ3CGZ9N4GH86CMGYT8455ZN -->
- [x] 2.4 實作規格需求「新 capability 的 Purpose 自 delta 帶入」（同名設計決策）：delta 檔含 Purpose 區段時複製為新正典 Purpose，未提供沿用現行占位骨架，既有 capability 的 Purpose 不受 delta 影響（crates/speclink-core/src/archive.rs，必要時於 crates/speclink-core/src/model.rs 補 Purpose 區段擷取）。完成時三情境測試全綠。 <!-- speclink-task:tsk_01KZ3CGZ9N0QC199624F1S12KS -->

## 3. 判定共用：drift assumptions 與 bulk 預檢改呼叫 plan 驗證

- [x] 3.1 實作規格需求「過期判定單源共用」：將 crates/speclink-core/src/drift.rs 的 spec assumptions 與 crates/speclink-cli/src/commands.rs 的 bulk archive 預檢收斂為呼叫 merge plan 驗證同一判定，reason 文案改為拒絕語意；speclink drift --json 的欄位結構（camelCase shape）不變、僅 reason 字串更新。完成時新增「同一過期 delta 於 drift、bulk 預檢、單筆 archive 三處認定一致」測試並全綠。 <!-- speclink-task:tsk_01KZ3CGZ9NY4SV55SP9MYRZJCE -->
- [x] 3.2 同批更新 crates/speclink-core/tests/render_golden.rs 與 crates/speclink-cli/tests/ 受影響整合測試（archive 拒絕輸出、drift reason 文案，含 --no-color 人眼斷言與 fs／remote 對照）。完成時 cargo test -p speclink-core --test it 與 speclink-cli 整合測試（含 remote_verb_parity）全綠，且變更僅限提案相容性影響段列明的刻意項目。 <!-- speclink-task:tsk_01KZ3CGZ9NDCTFMR0X99EERMEV -->

## 4. 技能文字與收尾驗證

- [x] 4.1 改寫 crates/speclink-core/assets/skills/archive.md 的引擎行為敘述：移除「靜默跳過、封存前先把既存 ADDED 轉 MODIFIED 以觸發重注入」補救教學，改為「引擎拒絕、依錯誤清單以 drift → ingest 修 delta」動線，並補 REMOVED-SCENARIO 聲明與 delta Purpose 區段的使用說明。完成時技能文字與新引擎行為一致（內容審閱，無殘留 silent-skip 敘述）。 <!-- speclink-task:tsk_01KZ3CGZ9N6E2VZP6N3702QK9C -->
- [x] 4.2 端對端驗證：建立含過期 delta 的樣本 change 實跑 speclink archive 確認拒絕訊息逐條列明並附補救指引、修正 delta 後重跑封存成功且新 capability Purpose 來自 delta；最後全套 cargo test（speclink-core 單元＋it、speclink-cli 整合）綠燈。完成時以上手動斷言與測試結果記錄於任務證據。 <!-- speclink-task:tsk_01KZ3CGZ9NZ9WCZ49Q4BBTS1EP -->
