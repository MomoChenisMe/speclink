## 1. 指紋分流（core 共用實作）

- [x] 1.1 先寫紅測（內容指紋錨與失效判定的分流規則）：bytes 入口對 UTF-8 內容（LF 與 CRLF 兩形）回傳與既有文字規則相同的雜湊；對非 UTF-8 位元組回傳原始位元組 SHA-256——spec 指紋分流表三列各一案例。驗證：cargo test -p speclink-core 出現預期紅燈 <!-- speclink-task:tsk_01M0EW8QK6TW5DKQQTBDZ5HSD9 -->
- [x] 1.2 實作 D1 指紋分流入口：station 模組新增 content_fingerprint_bytes（UTF-8 可解→轉呼既有文字規則；否則位元組 SHA-256）；content_fingerprint 原名原簽名保留。驗證：1.1 轉綠 <!-- speclink-task:tsk_01M0EW8QK7CYSGD61HER3E80T5 -->
- [x] 1.3 先寫紅測（含 binary 的蓋章與失效）：工單 Scope 含存在的非 UTF-8 檔時 stamp 成功且 reviewed_scope 記位元組雜湊；落章後該檔位元組變動 → freshness 判 stale；聯集全消失拒章與 I/O 錯誤拒章的既有語意維持——review 與 verify 兩站各覆蓋（驗證指紋錨與失效判定同構）。驗證：cargo test -p speclink-core 出現預期紅燈 <!-- speclink-task:tsk_01M0EW8QK7XKWMXM1RQKHKYHA0 -->
- [x] 1.4 實作 D2 讀檔閉包換位元組：fingerprint_scope 與 freshness 的閉包簽名改回傳位元組，None 維持缺檔／I/O 失敗語意；core 的 review 與 verify 兩模組的閉包跟上。驗證：1.3 轉綠且 cargo test -p speclink-core 全綠（既有文字指紋測試值不變） <!-- speclink-task:tsk_01M0EW8QK7DHF5NFWYDGM9KHPA -->

## 2. 呼叫端連動

- [x] 2.1 CLI remote 章路徑與 desktop core 重算的讀檔閉包改位元組（D2 讀檔閉包換位元組的其餘兩個消費端），行為不變、僅簽名跟上。驗證：cargo test -p speclink-cli --test it 與 cargo test -p speclink-desktop-core 全綠 <!-- speclink-task:tsk_01M0EW8QK7NCQ06QM28KWV3E95 -->

## 3. 收尾

- [x] 3.1 全量回歸與規格對齊確認：cargo test -p speclink-core、cargo test -p speclink-cli --test it、cargo test -p speclink-desktop-core 三套全綠；speclink validate stamp-binary-fingerprint 通過；D3 規格的最終態（兩站 delta）與實作行為逐 scenario 對讀一遍。驗證：三套測試與 validate 輸出全部通過 <!-- speclink-task:tsk_01M0EW8QK74HW9Z03N3Y4R04DV -->
