# phase2-acceptance Specification

## Purpose

TBD - created by archiving change 'phase2-e2e-chain'. Update Purpose after archive.

## Requirements

### Requirement: 八環節單一劇本連續可走

SHALL 存在單一連續劇本測試：以真實 CLI binary 對真 server（SQLite driver、tempdir 隔離、無外部服務依賴）依序走完——setup 開箱（stdout token → /setup 建 Admin 與 Project/Repo）、invite 與 PAT 取得、propose（new change 與全部 artifacts）、policy（寫入 workflow config 後 instructions 輸出 SHALL 反映政策變化、改回 SHALL 恢復）、task done 攜 touched files（evidence 記錄與 task-completed 事件 SHALL 同時可查）、context（投影完整且 manifest 驗證通過）、drift（有 checkout 的完整報告）、archive（正典 specs 更新、change 入 archive）。環節 SHALL 共用同一資料庫與帳號，SHALL NOT 各自重新播種。

#### Scenario: 全鏈劇本綠

- **WHEN** 於乾淨環境執行全鏈劇本測試
- **THEN** 八環節依序通過；archive 後查詢的正典 specs 含本劇本 change 的 delta 內容；全程共用步驟 (1)-(2) 建立的帳號與 scope

#### Scenario: policy 變化可觀察

- **WHEN** 劇本於 propose 後修改 workflow config 再取 instructions，隨後改回再取
- **THEN** 兩次 instructions 輸出的差異恰反映政策差異；改回後輸出恢復

---

<!-- @trace
source: phase2-e2e-chain
updated: 2026-07-16
code:
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/common/subscriber.rs
  - crates/speclink-server/tests/phase2_chain.rs
-->

---
### Requirement: event recovery 伴隨劇本收斂

劇本 SHALL 含伴隨的 SSE 訂閱者：於工作流中途強制斷線並漏掉後續事件，重連時 SHALL 依既有規則恢復——序號仍在保留範圍即續傳補齊、已清理即收 reset 並以輪詢與查詢全量收斂後重新訂閱；兩條恢復路徑 SHALL 各被至少一次劇本配置覆蓋（以保留筆數組態控制）。劇本結尾 SHALL 斷言訂閱者累積視角（以事件識別去重後）與直接查詢的正典一致，SHALL NOT 有訂閱者視角遺漏的變更。

#### Scenario: 續傳路徑收斂

- **WHEN** 訂閱者於 task done 後斷線、錯過投影與 archive 相關事件，以 Last-Event-ID 重連（序號未被清理）
- **THEN** 漏掉的事件依序補齊無重複；結尾視角與正典一致

#### Scenario: reset 路徑收斂

- **WHEN** 保留筆數組態設為極小值使斷線期間序號被清理，訂閱者重連
- **THEN** 收到 reset 訊號；以輪詢與查詢全量收斂並重新訂閱後，結尾視角與正典一致

---

<!-- @trace
source: phase2-e2e-chain
updated: 2026-07-16
code:
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/common/subscriber.rs
  - crates/speclink-server/tests/phase2_chain.rs
-->

---
### Requirement: 失敗現場可讀且 CI 必跑

劇本任一步失敗 SHALL 報出步驟編號與名稱，並附 server stderr 尾段與 workspace 現場摘要；SHALL NOT 只留裸 assert 差異。劇本 SHALL 位於 CI 必跑測試路徑，SHALL NOT 以靜默 ignore 存在；劇本揭露的產品缺陷 SHALL 以獨立 change 修復，劇本內 SHALL NOT 順手修改產品程式碼。

#### Scenario: 失敗訊息含步驟名

- **WHEN** 任一環節斷言失敗（以開發期人為注入驗證）
- **THEN** 失敗輸出含步驟編號/名稱與 server stderr 尾段；可據此直接定位環節

<!-- @trace
source: phase2-e2e-chain
updated: 2026-07-16
code:
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/common/subscriber.rs
  - crates/speclink-server/tests/phase2_chain.rs
-->