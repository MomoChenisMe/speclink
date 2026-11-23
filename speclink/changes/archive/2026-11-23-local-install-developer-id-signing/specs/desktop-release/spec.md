## MODIFIED Requirements

### Requirement: 本機安裝的新鮮度斷言

repo SHALL 提供本機建置安裝入口（node scripts/desktop/desktop-install.mjs），把桌面 app 的本機建置與安裝收成單一指令並以版號斷言取代信任。執行時 SHALL 依序：(0) 進入任何步驟之前先行把關——帶 `--install` 而平台非 macOS，或 updater 的簽章環境變數缺失時，SHALL 單行錯誤（點名平台或缺失的變數名）並以非零結束，SHALL NOT 進入任何建置；(1) 印出當前 HEAD、分支、工作樹是否乾淨與源碼的產物層版號，並在 macOS 上決定這次建置的程式碼簽章身分後印出：環境變數 `APPLE_SIGNING_IDENTITY` 有值時 SHALL 用它且 SHALL NOT 查找鑰匙圈；沒有值時 SHALL 用 `security find-identity -v -p codesigning` 列出的第一個「Developer ID Application」身分；兩者皆無、或查找本身無法執行或以非零結束時，SHALL 印出一行說明這次以 ad-hoc 簽章建置、重裝後鑰匙圈會重新詢問（查找失敗時附上原因），並 SHALL 繼續建置、SHALL NOT 因此以非零結束；非 macOS 平台 SHALL NOT 查找程式碼簽章身分；(2) 執行 sidecar 佈署（永遠重新建置，SHALL NOT 沿用 src-tauri/binaries/ 既有檔案）；(3) 前端建置與 tauri bundle——決定了程式碼簽章身分時 SHALL 只對 tauri bundle 這一步以它簽 app 與 bundle 內的 sidecar CLI，SHALL NOT 公證；(4) 斷言 bundle 內 sidecar CLI 的引擎版號等於源碼產物層版號，不等時 SHALL 印出兩邊版號並以非零結束；決定了程式碼簽章身分時 SHALL 再斷言 bundle 內 sidecar CLI 的簽章帶 Team ID 且不是 ad-hoc，不符時 SHALL 印出該 CLI 實際的簽章資訊並以非零結束、SHALL NOT 進行安裝；以 ad-hoc 建置時不做這道簽章斷言。帶 `--install` 時 SHALL 續行：(5) 確認 app 未執行——執行中 SHALL 單行錯誤停止，SHALL NOT 代為結束程序；(6) 覆蓋 /Applications 安裝，SHALL 先將新版完整佈到暫存路徑再換上，使拷貝失敗時既有安裝原封不動；(7) 斷言安裝版 CLI 的引擎版號同版，不等時 SHALL 印出兩邊版號並以非零結束。安裝步驟（5–7）僅支援 macOS；建置步驟（1–4）平台中立。任一步驟失敗 SHALL 以非零結束且 SHALL NOT 繼續後續步驟。

#### Scenario: 建置並通過 bundle 斷言

- **WHEN** 於簽章環境變數齊備的源碼樹執行 node scripts/desktop/desktop-install.mjs
- **THEN** 依序輸出 HEAD 與源碼產物層版號、重建 sidecar、完成 bundle，並以 bundle 內 CLI 的引擎版號等於源碼版號通過斷言，exit code 0

#### Scenario: bundle 版號不符即失敗

- **WHEN** bundle 內 sidecar CLI 的引擎版號與源碼產物層版號不等（如建置鏈沿用了過期 binary）
- **THEN** 印出兩邊版號、exit code 非零、不進行安裝

#### Scenario: 安裝後斷言安裝版同版

- **WHEN** 帶 --install 執行且 app 未執行、建置斷言通過
- **THEN** 覆蓋安裝後以安裝版 CLI 的引擎版號同版通過第二道斷言，exit code 0

#### Scenario: app 執行中拒絕安裝

- **WHEN** 帶 --install 執行而 Speclink app 程序仍在執行
- **THEN** 單行錯誤說明需先關閉 app、exit code 非零、/Applications 零變動

#### Scenario: 簽章環境變數缺失即停

- **WHEN** 簽章環境變數未設定時執行 node scripts/desktop/desktop-install.mjs
- **THEN** 單行錯誤指出缺失的變數名、exit code 非零、不進入建置（含 sidecar 重建）

#### Scenario: 非 macOS 帶 --install 即停

- **WHEN** 於非 macOS 平台執行 node scripts/desktop/desktop-install.mjs --install
- **THEN** 單行錯誤指出平台、exit code 非零、不進入建置

#### Scenario: 鑰匙圈有 Developer ID 憑證時以它簽章

- **WHEN** 於 macOS、未設定 `APPLE_SIGNING_IDENTITY`、鑰匙圈有一張有效的「Developer ID Application」憑證時執行 node scripts/desktop/desktop-install.mjs
- **THEN** 輸出印出這張憑證的身分名稱；bundle 內的 sidecar CLI 帶該憑證的 Team ID、不是 ad-hoc；版號斷言與簽章斷言都通過，exit code 0

#### Scenario: 環境變數指定的簽章身分優先

- **WHEN** 於 macOS 設定了 `APPLE_SIGNING_IDENTITY` 時執行 node scripts/desktop/desktop-install.mjs
- **THEN** 輸出印出這個值，tauri bundle 以它簽章，腳本不查找鑰匙圈

#### Scenario: 找不到憑證時照 ad-hoc 建置

- **WHEN** 於 macOS、未設定 `APPLE_SIGNING_IDENTITY`、鑰匙圈沒有有效的「Developer ID Application」憑證（或查找無法執行）時執行 node scripts/desktop/desktop-install.mjs
- **THEN** 輸出有一行說明這次以 ad-hoc 簽章建置、重裝後鑰匙圈會重新詢問；建置與版號斷言照常進行，不做簽章斷言，其餘步驟成功時 exit code 0

#### Scenario: 簽章結果不符即失敗

- **WHEN** 決定了程式碼簽章身分，但 bundle 內 sidecar CLI 的簽章是 ad-hoc 或沒有 Team ID
- **THEN** 印出該 CLI 實際的簽章資訊、exit code 非零、不進行安裝、/Applications 零變動

#### Scenario: 同一個簽章身分重裝後身分不變

- **WHEN** 以同一個程式碼簽章身分先後兩次帶 --install 執行，兩次都成功
- **THEN** 兩次安裝版 sidecar CLI 的簽章識別碼與 Team ID 都相同
