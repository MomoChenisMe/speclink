## 1. 原子寫入口（TDD：先測試後實作）

- [x] 1.1 crates/speclink-core/src/util.rs 測試模組先寫測試：既有行為不變面——寫入內容正確、自動建父目錄、覆寫既有檔；原子面——寫入成功後目的目錄無暫存檔殘留；並行壓力面（#[cfg(unix)]，落實 spec「本地檔案寫入原子落盤」的「並行讀者不見半份內容」）——寫者執行緒交替寫入兩份等長全文、讀者執行緒迴圈讀取，斷言每次讀到的內容恆等於其中一份完整全文，絕無空檔或混合內容。此時測試對現行實作紅燈（普通 fs::write 會讓讀者讀到截斷內容）。 <!-- speclink-task:tsk_01KZQ8VZSAT04D41BVB1ST6XSD -->
- [x] 1.2 實作 util::write_file 原子化使 1.1 全綠：同目錄暫存檔（檔名帶唯一後綴避免並行寫者互撞）→ std::fs::rename 至目的路徑；rename 失敗時退回直接 std::fs::write 並清理暫存檔（Windows sharing violation 語意——行為不劣於原子化前，不把平台限制放大成動詞失敗）。 <!-- speclink-task:tsk_01KZQ8VZSAD3MZVZ1FEHQ8SKT3 -->
- [x] 1.3 補退回路徑測試：模擬 rename 必然失敗的目的位置（如目的路徑為既存目錄之類的可攜情境；不可攜則以單元層注入方式覆蓋），斷言內容仍正確落盤且暫存檔被清理。 <!-- speclink-task:tsk_01KZQ8VZSAZQ38NE2S1DSAZEVV -->

## 2. 旁路收編

- [x] 2.1 crates/speclink-cli/src/verbs/config.rs：設定檔編輯的直接 fs::write 改走 speclink_core util::write_file，錯誤訊息維持既有 label 語境；既有 config 動詞測試全綠。 <!-- speclink-task:tsk_01KZQ8VZSA1A9GCRK7F6X1TDN6 -->
- [x] 2.2 apps/desktop/core/src/settings.rs：兩處 config.yaml 寫入（政策欄位與 context/rules）改走 util::write_file；speclink-desktop-core 既有 settings 測試全綠。 <!-- speclink-task:tsk_01KZQ8VZSA01HZ54HTBWH8KM68 -->

## 3. 驗證

- [x] 3.1 受影響 crates 全量：cargo test -p speclink-core -p speclink-cli -p speclink-desktop-core 全綠；cargo clippy 零新警告。 <!-- speclink-task:tsk_01KZQ8VZSAHXCZJEWJCDFK1ASV -->
- [ ] [M] 3.2 跨平台：合併回主分支後於 CI 確認 Windows 全綠——重點看 rename 覆蓋語意與退回路徑；若 Windows 首跑轉紅，先以「新測試第一次在 Windows 跑」歸因（期望值同源分流），再判斷是否為實作缺陷。確認綠燈後勾掉本任務。 <!-- speclink-task:tsk_01KZQ8VZSAF1CJ594F9HH987FZ -->
