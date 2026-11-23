## 1. 簽章身分的判斷（純函式，先寫測試）

- [x] 1.1 在 scripts/desktop/desktop-install.test.mjs 先加會失敗的測試，再在 scripts/desktop/desktop-install.mjs 匯出 `developerIdIdentity(findIdentityOutput)`：從 `security find-identity -v -p codesigning` 的輸出回傳第一個以「Developer ID Application:」開頭的身分名稱（引號內的完整字串），沒有時回傳 null。測試案例：只有一張 Developer ID（回傳它）、兩張 Developer ID（回傳第一張）、只有「Apple Development」身分（null）、`0 valid identities found`（null）、空字串（null）。驗證：node --test scripts/desktop/desktop-install.test.mjs 通過 <!-- speclink-task:tsk_01M84CG9GZ2C6NSSNMCG4W8MWK -->
- [x] 1.2 先加測試，再匯出 `chooseSigningIdentity({ env, platform, lookup })`，依設計 D1、D3 回傳 `{ identity, note }`：非 macOS 回傳 identity 為 null、note 為 null，而且不呼叫 lookup；`env.APPLE_SIGNING_IDENTITY` 有值時回傳它、不呼叫 lookup；lookup 回傳的輸出有 Developer ID 時回傳該名稱；沒有時 identity 為 null、note 是「找不到 Developer ID Application 憑證，這次以 ad-hoc 簽章建置；重裝後鑰匙圈會重新詢問」；lookup 丟出錯誤時 identity 為 null、note 是同一句再附上錯誤訊息。測試以注入的 lookup 記錄是否被呼叫。驗證：node --test scripts/desktop/desktop-install.test.mjs 通過 <!-- speclink-task:tsk_01M84CG9GZ4VR4BC8MY4K0PR8D -->
- [x] 1.3 先加測試，再匯出 `signatureProblem(codesignOutput)`，依設計 D4：輸出有 `TeamIdentifier=` 後接非 `not set` 的值、且沒有 `Signature=adhoc` 時回傳 null；否則回傳說明原因的字串（含原始輸出）。測試案例：正常的 Developer ID 輸出（null）、`Signature=adhoc` 加 `TeamIdentifier=not set`（非 null）、有 Team ID 但同時有 `Signature=adhoc`（非 null）、沒有 TeamIdentifier 行（非 null）。驗證：node --test scripts/desktop/desktop-install.test.mjs 通過 <!-- speclink-task:tsk_01M84CG9GZ6A8DA4XZWQ3YYFJB -->

## 2. 接進安裝流程

- [x] 2.1 scripts/desktop/desktop-install.mjs 的 `main` 在步驟 (1) 印完來源與版號後，以 `chooseSigningIdentity` 決定簽章身分：lookup 以 spawnSync 執行 `security find-identity -v -p codesigning`，無法執行或非零結束時丟出含原因的錯誤（由 1.2 轉成 ad-hoc 的說明）。決定了身分時印出「簽章身分：<身分>」，否則印出 note（非 macOS 兩者皆不印）。步驟 (3) 只有 tauri bundle 那一次子程序的環境變數多帶 `APPLE_SIGNING_IDENTITY`，sidecar 佈署與前端建置不帶。步驟 (0) 的把關與既有輸出行不變。驗證：node --test scripts/desktop/desktop-install.test.mjs 通過；內容審閱確認只有 tauri bundle 的呼叫帶了這個環境變數 <!-- speclink-task:tsk_01M84CG9GZVPMTQFPH18AEJNT3 -->
- [x] 2.2 步驟 (4) 的版號斷言通過後，決定了身分時以 spawnSync 執行 `codesign -dv` 檢查 bundle 內的 sidecar CLI（資訊在 stderr），把輸出交給 `signatureProblem`；回傳非 null 時以單行錯誤（`desktop-install: ` 開頭）加上該原因結束、exit code 1，不進入步驟 (5)～(7)。以 ad-hoc 建置時不執行這一步。同時更新檔頭的用法註解，寫明簽章身分的來源順序。驗證：node --test scripts/desktop/desktop-install.test.mjs 通過；內容審閱確認簽章斷言位於版號斷言之後、安裝之前 <!-- speclink-task:tsk_01M84CG9GZVD5RC5A4MNTE869N -->

## 3. 實機驗證

- [x] [M] 3.1 先結束 Speclink，在看得到螢幕的前景執行 TAURI_SIGNING_PRIVATE_KEY 與密語齊備的 node scripts/desktop/desktop-install.mjs --install。確認輸出有「簽章身分：Developer ID Application: …」一行、版號與簽章兩道斷言都通過、exit code 0；macOS 詢問 codesign 能否使用私鑰時選「永遠允許」。之後在連到遠端專案的 checkout 執行一次 speclink auth status，對鑰匙圈的每個詢問框選「永遠允許」 <!-- speclink-task:tsk_01M84CG9GZ0NBQ4416YKXG7FY0 -->
- [x] [M] 3.2 再結束 Speclink 並重裝一次。確認兩次安裝版 /Applications/Speclink.app/Contents/MacOS/speclink 的 codesign -dv 識別碼與 TeamIdentifier 相同、沒有 Signature=adhoc；再於同一個 checkout 執行 speclink auth status，鑰匙圈不再詢問 <!-- speclink-task:tsk_01M84CG9GZWY6W0W6T2BTWBE7F -->

## 4. 收尾

- [x] 4.1 回歸與規格對照：node --test scripts/*.test.mjs scripts/*/*.test.mjs 全部通過；逐句對照 specs/desktop-release 的 delta 需求「本機安裝的新鮮度斷言」（步驟 (1)、(3)、(4) 的新句子與五個新情境）與 scripts/desktop/desktop-install.mjs 的實作，確認每一句都有對應的程式或測試 <!-- speclink-task:tsk_01M84CG9GZSA3E40J5T2GJD25P -->
