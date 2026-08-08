## 1. desktop-sidecar.mjs 可測化與新參數

- [x] 1.1 [Red] 新增 scripts/desktop-sidecar.test.mjs（node:test，慣例同 scripts/desktop-install.test.mjs）：斷言「決策二：dev 佈 debug profile，--profile 參數白名單驗證」與「決策四：desktop-sidecar.mjs 重構為可測形狀」的契約——參數解析（無旗標預設 release、--profile debug 合法、白名單外的值拋出點名該值與合法值的錯誤）、建置產物來源路徑推導（profile × --target × Windows .exe 四種組合）、跳過複製判定（目的檔不存在 → 複製；內容相同 → 跳過；內容相異 → 複製）。驗證：node --test scripts/desktop-sidecar.test.mjs 於重構前因無匯出而失敗。 <!-- speclink-task:tsk_01KZGTV9GQG1EK4GGTBG8HMZGA -->
- [x] 1.2 [Green] 重構 scripts/desktop-sidecar.mjs：匯出 1.1 所列純函式，main 以 import.meta.url 執行判斷薄委派（同 scripts/desktop-install.mjs 模式，匯入不執行佈署）；實作 --profile 參數與內容相同即跳過。行為契約：無參數執行仍為 release 建置與佈署（保護 scripts/desktop-install.mjs 與 CI --target 兩個既有呼叫者）；來源檔缺失時明確報錯非零，不靜默跳過。驗證：node --test scripts/desktop-sidecar.test.mjs 全數通過。 <!-- speclink-task:tsk_01KZGTV9GQ2J5H72WKV02QYPD1 -->

## 2. predev hook 接線

- [x] 2.1 [Red] 於 scripts/dev.test.mjs 的設定守門區新增斷言：apps/desktop/package.json 的 scripts SHALL 含 predev、其內容呼叫 desktop-sidecar.mjs 且帶 --profile debug——守「決策一：掛載點為 apps/desktop 的 npm predev hook」，防 hook 被移除或改成 release 而靜默退化。驗證：node --test scripts/dev.test.mjs 於 predev 尚未加入時新斷言失敗、既有測試維持通過。 <!-- speclink-task:tsk_01KZGTV9GQTGGWZCBX6XPHCQVM -->
- [x] 2.2 [Green] 於 apps/desktop/package.json 的 scripts 新增 predev（以 node 呼叫 ../../scripts/desktop-sidecar.mjs 帶 --profile debug），既有 dev、build、test、tauri scripts 不變。行為契約：npm 對 run dev 自動先跑 predev，三個 dev 入口（npm run dev、npm run dev:desktop、apps/desktop 內 tauri dev）皆於編譯前佈署 sidecar。驗證：node --test scripts/dev.test.mjs 全數通過；npm test -w apps/desktop 通過，確認 package.json 變動未擾動測試 harness。 <!-- speclink-task:tsk_01KZGTV9GQ64S5YQ477J2BW1CH -->

## 3. 行為驗收與回歸確認

- [x] 3.1 手動驗收「dev 啟動自動佈署當前 checkout 的 sidecar」的全新 checkout 場景：暫時把 apps/desktop/src-tauri/binaries/ 移出後執行 npm run dev:desktop，確認終端先出現 sidecar 建置與佈署輸出、dev 視窗正常開啟、binaries/ 檔案就位（debug 內容，與 target/debug/speclink 同 hash）。驗證：逐項目視確認並記錄；結束後环境復原。 <!-- speclink-task:tsk_01KZGTV9GQJVC951Y9XH7Z65ZY -->
- [x] 3.2 手動驗收「決策三：內容相同即跳過複製（防抖）」：緊接 3.1 再啟動一次 npm run dev:desktop，確認 sidecar 檔案未被改寫（mtime 不變）且終端未因 sidecar 出現 speclink-desktop 重編輸出。驗證：比對兩次啟動間檔案 mtime 與終端輸出。 <!-- speclink-task:tsk_01KZGTV9GQ8Q3Z7X7WQ6ESFK3V -->
- [x] 3.3 手動驗收失敗模式：執行 node scripts/desktop-sidecar.mjs --profile bogus，確認 exit code 為 1 且 stderr 點名 bogus 與合法值清單；佐證 predev 失敗時 npm run dev 中止、tauri dev 不啟動。驗證：echo 檢查 exit code 並目視錯誤訊息。 <!-- speclink-task:tsk_01KZGTV9GQ74KVT93MA8S6R2K9 -->
- [x] 3.4 確認 release 路徑未回歸：git diff 確認 scripts/desktop-install.mjs 零改動；無參數執行 node scripts/desktop-sidecar.mjs 一次，確認仍為 release 建置並佈署至同一目的位置。驗證：diff 為空＋佈署完成訊息與 binaries/ 檔案為 release 內容。 <!-- speclink-task:tsk_01KZGTV9GQBMMPF3D8CR7J7160 -->
- [x] 3.5 套用 sharp-edges audit 檢查清單於新設定面（--profile 參數、predev hook）：白名單外值大聲失敗不靜默、無參數預設（release）即現行安全行為、predev 隱性觸發有佈署輸出訊息可循且有 2.1 設定守門防誤刪。驗證：逐項對照 speclink instructions --skill audit 的 Discipline 清單並記錄結論。 <!-- speclink-task:tsk_01KZGTV9GQHK221W0HMMAEGYPN -->
- [x] 3.6 收尾回歸：node --test "scripts/**/*.test.mjs" 全量通過。本次零 Rust 改動、不涉 CLI 輸出面，golden 與 cargo 測試預期零影響，不另跑全量（CI 守門）；apps/desktop 面已於 2.2 驗過。驗證：scripts 全量測試綠燈。 <!-- speclink-task:tsk_01KZGTV9GQKHQ9JB2VX5MZRC2X -->
