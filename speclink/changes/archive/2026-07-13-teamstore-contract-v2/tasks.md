## 1. crate 骨架與契約型別（design 決策一：契約落點為獨立 crate speclink-store，與 speclink-core 零相依；決策三：typed error 為封閉的 Store 錯誤集合，與 command 錯誤碼分層；決策四：定址採 Project／Repo scope 加邏輯 document locator）

- [x] 1.1 建立 crate 並撰寫契約型別的失敗測試，覆蓋「讀取以 typed Result 區分失敗類別」與「文件定址採 Project 與 Repo scope 的邏輯 locator」的型別面：StoreError 封閉六類（not_found、permission_denied、revision_conflict 帶 expected/actual、unavailable、corrupt 帶原因、backend）各附穩定錯誤碼字串；DocRef 三元組與 DocumentId 封閉 enum 涵蓋六種文件種類；manifest 型別含 contract version、capabilities 與三級能力等級（crates/speclink-store/src/error.rs、crates/speclink-store/src/types.rs 的 #[cfg(test)]；根 Cargo.toml 追加 workspace 成員）。cargo test -p speclink-store 觀察紅燈。
- [x] 1.2 實作契約型別與 TeamStore trait 定義（同步、object-safe——design 決策二：契約維持同步、object-safe）：manifest／health／migrate／snapshot／begin_unit_of_work／commit(uow, event_records)／rollback／export／import／outbox 讀取與確認（crates/speclink-store/src/lib.rs、crates/speclink-store/src/uow.rs），1.1 轉綠；驗證 cargo build --release 全 workspace 編譯通過且不引入 async 相依。

## 2. in-memory reference：讀取與 snapshot

- [x] 2.1 撰寫失敗測試，覆蓋「consistent snapshot 提供固定時點視圖」與 scope 隔離讀取：mixed snapshot 情境（讀方取得 snapshot 後寫方 commit，讀方仍見固定時點的舊內容且 revision 不變）、損壞文件讀取回 corrupt 而非空值、跨 repo 讀取不回傳他方內容（crates/speclink-store/src/memory.rs 的 #[cfg(test)]）。紅燈。
- [x] 2.2 實作 in-memory reference store 的儲存模型、snapshot 與 typed 讀取（crates/speclink-store/src/memory.rs），2.1 轉綠；驗證 cargo test -p speclink-store 全綠。

## 3. Unit of Work、CAS 與 history（design 決策五：Unit of Work 為唯一寫入路徑，commit 是唯一原子點）

- [x] 3.1 撰寫失敗測試，覆蓋「Unit of Work 是唯一寫入路徑且 commit 原子」與「immutable history 記錄每次文件變更」：CAS race 兩 UoW 併發恰一方成功、敗方拿到 revision_conflict 與 actual；新建攜「不得已存在」語意；rollback 無殘留；建立／修改／刪除三 commit 後歷史含三筆 revision（末筆 tombstone、各帶 actor 與 UTC 時間戳與 digest 與來源 command）；回退以追加新 revision 表達（crates/speclink-store/src/memory.rs、crates/speclink-store/src/uow.rs 測試）。紅燈。
- [x] 3.2 實作 UoW 暫存、commit 原子生效（文件寫入＋project revision 遞增＋history 追加）與 revision_conflict 判定，3.1 轉綠；驗證 cargo test -p speclink-store 全綠。

## 4. transactional outbox 與故障注入（design 決策七：conformance suite 與 in-memory reference 同 crate 交付，故障注入為第一級設計）

- [x] 4.1 撰寫失敗測試，覆蓋「transactional outbox 與 cursor 重讀」：event records 與文件寫入同原子落 outbox、自 cursor 0 重讀得到與生效 commit 一一對應的事件序列、消費確認後不重複；in-memory reference 的故障注入點（文件寫入後、history 追加後、outbox 追加前後崩潰）——partial commit 不外洩、outbox 追加失敗整個 commit 不生效、crash recovery 重建後文件／revision／history／outbox 四者一致（crates/speclink-store/src/memory.rs 測試）。紅燈。
- [x] 4.2 實作 outbox、cursor、確認與故障注入機制，4.1 轉綠；驗證 cargo test -p speclink-store 全綠。

## 5. export 與 import（design 決策八：export/import 為 versioned bundle，round-trip 屬 conformance）

- [x] 5.1 撰寫失敗測試，覆蓋「export 與 import 以 versioned bundle 往返」：bundle 帶格式版本、scope、project revision 與逐文件 digest；round-trip 到全新 store 逐文件一致且歷史以 import 為起點；digest 或版本驗證失敗即拒絕且不部分套用（crates/speclink-store/src/types.rs、crates/speclink-store/src/memory.rs 測試）。紅燈。
- [x] 5.2 實作 bundle 型別、export 與 import 驗證語意，5.1 轉綠；驗證 cargo test -p speclink-store 全綠。

## 6. conformance suite（design 決策六：manifest 宣告 capabilities 與能力等級，conformance 按宣告分級執行）

- [x] 6.1 撰寫失敗測試，覆蓋「conformance suite 可對任意實作重用執行」與「manifest 宣告契約版本、能力與等級」：conformance 入口接受任意 TeamStore trait object；以刻意殘缺的測試替身驗證「宣告 single-node 但缺 outbox capability 於能力檢查階段判整體不通過並指出缺失」（crates/speclink-store/src/conformance/mod.rs 測試）。紅燈。
- [x] 6.2 實作 conformance 入口與六類 gate 情境（CAS race、mixed snapshot、partial commit、outbox failure、crash recovery、tenant scope）依 manifest 宣告分級執行；in-memory reference 通過完整 suite 並回報 capability 清單與契約版本，6.1 轉綠。
- [x] 6.3 重構：第 2–5 節的情境斷言與 conformance 情境對齊、抽出共用 fixture，消除重複樣板；驗證 cargo test -p speclink-store 全綠且測試數不減。

## 7. 全量收尾

- [x] 7.1 對新公開 API 跑 sharp-edges 稽核檢查表（speclink instructions --skill audit）並修正發現；驗證 cargo test --workspace 與 npm run test:all 全綠；git diff --stat 確認既有 crates 原始碼零改動（僅根 Cargo.toml 與 Cargo.lock 的 workspace 成員追加），既有 CLI 輸出無需重跑 parity／color／twin 對照（無行為變更）。
