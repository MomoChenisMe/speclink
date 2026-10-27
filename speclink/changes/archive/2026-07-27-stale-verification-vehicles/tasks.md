## 1. 正典 delta 覆蓋確認

- [x] 1.1 依 design「決策一：載體對照表（4 條需求共用）」與「決策二：固定計數一律刪除」確認 4 份 delta（本變更 specs/ 下）僅置換驗證載體子句與刪去固定計數、凍結語意與情境結構不變：需求標題與正典逐字相同（client-protocol「typed client 全面取代 raw JSON 旁路」、reference-server「真實 CLI 端到端一致」、drift-computation「本地 drift 路徑輸出凍結」、host-runtime「組裝點遷移輸出凍結」），且每條需求的 `#### Scenario:` 數量與正典一致。驗證：speclink validate stale-verification-vehicles 通過；逐份 delta 與正典原文 diff 僅含載體子句與固定計數的差異 <!-- speclink-task:tsk_01KYH2447ZCX8AYBQ8J32EY9VB -->

## 2. 路線圖文件改寫

- [x] 2.1 依 design「決策四：路線圖兩處的改寫」改寫 docs/implementation-refactor-roadmap.zh-TW.md：第 1 節結論的「有 parity、golden 與整合測試保護」改為「有 golden 與整合測試保護」；元件表 speclink-cli 列「可保留部分」欄的「parity 護欄」改為「輸出凍結護欄」。不動該文件其他現況描述。驗證：grep -n "parity" docs/implementation-refactor-roadmap.zh-TW.md 命中數為 0；node --test "scripts/**/*.test.mjs" 全數通過 <!-- speclink-task:tsk_01KYH2447ZMTKY6078NRJM7XFT -->

## 3. 收尾驗證

- [x] 3.1 依 design「決策三：豁免清單（不動的措辭）」斷言不存在的載體名已清除且豁免項未被動到：於本變更 specs/ 下的 4 份 delta grep "twin harness"、"parity_suite"、"color_suite" 命中數皆為 0（正典待 archive 併入，故以 delta 內容為準）；client-protocol、reference-server、remote-connection 的 stub server 措辭與 docs/verb-contract.md、docs/verb-contract.zh-TW.md、docs/sdk-node.md 的 parity 字樣逐字不變。驗證：上述 grep 結果與豁免清單完全一致 <!-- speclink-task:tsk_01KYH2447ZZ5C5VKJM03GDX9CM -->
- [x] 3.2 零行為變更確認：cargo test --workspace 全綠；git diff 不含任何 .rs／.ts／.css 源碼行，且 crates/speclink-core/assets/ 與 crates/speclink-core/tests/golden/ 未被修改。驗證：全部測試通過、git status 僅列 openspec/changes/stale-verification-vehicles/ 與 docs/implementation-refactor-roadmap.zh-TW.md <!-- speclink-task:tsk_01KYH2447ZK2ZN7ATCKF0CCCNJ -->
- [x] 3.3 執行 speclink validate stale-verification-vehicles 與 speclink analyze stale-verification-vehicles 確認 artifacts 一致。驗證：validate 通過、analyze 無 Critical 或 Warning <!-- speclink-task:tsk_01KYH2447ZA4DTRFNJ2MEYH06F -->
