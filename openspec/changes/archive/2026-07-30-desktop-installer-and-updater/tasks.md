## 1. 更新描述檔組裝腳本

- [x] 1.1 撰寫 latest.json 組裝的單元測試 scripts/release-latest-json.test.mjs：給定 tag（v0.2.0）與各平台更新包檔名＋簽章內容，斷言輸出 JSON 的 version 為 0.2.0、platforms 含 darwin-aarch64／darwin-x86_64／windows-x86_64／linux-x86_64 四鍵、各鍵 url 指向該 Release asset 下載路徑、signature 為對應簽章內容；缺任一平台輸入時以非零結束（fail-closed）。驗證：node --test 執行（掛進 package.json 既有 scripts/**/*.test.mjs glob）紅燈確認測試有效。 <!-- speclink-task:tsk_01KYS5D1WNJQ34X1EKHK0F3Q6Y -->
- [x] 1.2 實作 scripts/release-latest-json.mjs 使 1.1 測試轉綠：讀產物目錄與 tag、組裝並輸出 latest.json，落實 design 決策 D2：更新描述檔由 workflow 組裝，端點固定 GitHub Releases與需求「更新描述檔隨 release 發布」。驗證：npm run test:all 的 scripts 段全綠。 <!-- speclink-task:tsk_01KYS5D1WNXFQJ2AHCTCQTA6SE -->

## 2. Tauri bundle 與 updater 設定

- [x] 2.1 產生 updater 金鑰對並接線：公鑰與更新端點（releases/latest/download/latest.json）寫入 apps/desktop/src-tauri/tauri.conf.json 的 plugins.updater 設定；私鑰與密語存入 GitHub secrets TAURI_SIGNING_PRIVATE_KEY／TAURI_SIGNING_PRIVATE_KEY_PASSWORD（需使用者操作 repo 設定並離線備份私鑰），落實 design 決策 D3：更新完整性用 Tauri updater 自產金鑰，與 OS 簽章正交與需求「更新包簽章」。驗證：tauri.conf.json 可解析、本機以私鑰環境變數執行 tauri build 產出更新包與對應簽章檔。 <!-- speclink-task:tsk_01KYS5D1WNGAQJV42MJR3NP19X -->
- [x] 2.2 掛載 tauri-plugin-updater：apps/desktop/src-tauri/Cargo.toml 加相依、apps/desktop/src-tauri/src/lib.rs 註冊 plugin、apps/desktop/package.json 加前端綁定、apps/desktop/src-tauri/capabilities 開對應權限。驗證：cargo build -p speclink-desktop 通過、npm test -w apps/desktop 不因新相依變紅。 <!-- speclink-task:tsk_01KYS5D1WN210S2DHSB4BFEHER -->
- [x] 2.3 CLI 以 sidecar 隨附：tauri.conf.json 設 externalBin 納入 speclink CLI（建置前置步驟把 CLI binary 佈到 target triple 命名位置）。驗證：本機 tauri build 產出的 bundle 內含 CLI，且其 --version 與 desktop 版本一致。 <!-- speclink-task:tsk_01KYS5D1WN9K9GPWY92T0W4EPA -->

## 3. 桌面自動更新流程

- [x] 3.1 撰寫更新狀態機測試（apps/desktop/core，vitest）：閒置→檢查中→發現新版（含目標版本）→徵求同意→下載→待重啟的狀態轉移；檢查失敗（離線）靜默回閒置；簽章驗證失敗轉錯誤態且既有安裝不受影響；手動檢查且已最新時回報已是最新。驗證：npm test -w apps/desktop 紅燈確認測試有效。 <!-- speclink-task:tsk_01KYS5D1WNG3DBAQQHRZ7P8X62 -->
- [x] 3.2 實作更新狀態機與 store 接線使 3.1 轉綠：apps/desktop/core 純邏輯（不依賴 Tauri）、apps/desktop/src/store.ts 接 plugin 事件、src-tauri command 單行委派，落實需求「桌面自動更新」與 design 決策 D6：更新流程邏輯歸 apps/desktop/core，Tauri 殼單行委派。驗證：npm test -w apps/desktop 全綠。 <!-- speclink-task:tsk_01KYS5D1WNPFY7QPJTDGRWKBNP -->
- [x] 3.3 更新介面：啟動背景檢查的新版通知（顯示版本、同意／稍後）、設定頁手動「檢查更新」入口、套用後重啟提示。驗證：vitest 元件測試綠；真實視窗操作確認徵詢出現、拒絕時不下載（依 CLAUDE.md GUI 驗證備忘）。 <!-- speclink-task:tsk_01KYS5D1WN0RAKKXG1AZFSZ29E -->

## 4. 安裝 CLI 指令動作

- [x] 4.1 撰寫 CLI 佈署邏輯測試（apps/desktop/core，vitest）：macOS 產生 ~/.local/bin symlink 指向 app bundle 內 CLI；Linux AppImage 複製至 ~/.local/bin 且版本不符時判定需重佈署；Windows 與 deb 僅回報狀態不佈署；佈署目錄不在 PATH 時輸出提示旗標；狀態判定涵蓋未安裝／已安裝同版／版本不符。驗證：npm test -w apps/desktop 紅燈確認測試有效。 <!-- speclink-task:tsk_01KYS5D1WNK6PWKQZ1Q1Q06700 -->
- [x] 4.2 實作佈署邏輯與介面使 4.1 轉綠：core 純邏輯＋src-tauri 檔案系統 command（單行委派）＋設定頁「安裝 CLI 指令」動作與狀態呈現；AppImage 啟動時版本不符自動重佈署，落實需求「安裝 CLI 指令到 PATH」與 design 決策 D5：CLI 以 sidecar 隨附，佈署進 PATH 依平台分流。驗證：npm test -w apps/desktop 全綠；macOS 真實視窗執行動作後終端 speclink --version 與 desktop 同版。 <!-- speclink-task:tsk_01KYS5D1WNTZPYADJG754F5VW4 -->
- [x] 4.3 Windows NSIS 安裝目錄寫入使用者 PATH：apps/desktop/src-tauri/tauri.conf.json 掛 NSIS installer hook 檔（apps/desktop/src-tauri/windows 之下）。驗證：Windows 機器安裝後新終端可直接執行 speclink --version（跨機器驗證，依 CLAUDE.md 備忘）。 <!-- speclink-task:tsk_01KYS5D1WNQ7GZCM1F5KWV8JNZ -->

## 5. Release workflow

- [x] 5.1 release.yml 新增 desktop build job：matrix 出 macOS dmg（aarch64／x86_64）、Windows NSIS（x86_64）、Linux AppImage＋deb（x86_64／aarch64），Linux runner 先安裝 GTK／WebKit 系統依賴（援引 ci.yml 清單），建置前跑 npm ci 與 desktop 前端 build，上傳安裝檔與更新包簽章為 artifacts，落實需求「Release 產出三平台桌面安裝檔」與 design 決策 D1：沿用既有 matrix 自組 desktop build job，不改用 tauri-action。驗證：workflow 語法過 actionlint 或 push 預演 tag 後該 job 全綠。 <!-- speclink-task:tsk_01KYS5D1WNPWVFGPACPXR3N0TK -->
- [x] 5.2 release job 整合：下載 desktop artifacts、執行 scripts/release-latest-json.mjs 組裝 latest.json、SHA256SUMS.txt 收錄全部新增檔案、release needs 閘門納入 desktop build（任一形態失敗不發布）。驗證：預演 tag 的 Release assets 清單符合 desktop-release spec 的完整安裝檔集場景。 <!-- speclink-task:tsk_01KYS5D1WNGB5KK4VQM8HPAPJZ -->
- [x] 5.3 OS 簽章 env-gated 條件步驟骨架：macOS／Windows 簽章步驟以對應 secrets 存在為條件，缺 secrets 時跳過且不影響其餘步驟，落實需求「OS 程式碼簽章為可插鑰匙開關」與 design 決策 D4：OS 簽章開關＝secrets 存在才跑的條件步驟。驗證：目前 repo 無 OS 簽章 secrets 的狀態下預演 workflow 全綠、產未簽章安裝檔。 <!-- speclink-task:tsk_01KYS5D1WNZA7NNQCVYJH8CQYT -->

## 6. 端到端驗收

- [x] 6.1 以預發布 tag 走完整更新迴圈：安裝前一版 desktop → push 新預發布 tag → app 內收到更新徵詢（顯示新版本號）→ 同意後套用並重啟為新版 → 安裝的 CLI 隨之同版；同時確認離線啟動不彈錯。驗證：真實視窗全程操作記錄結果，逐項對照 desktop-app delta spec 的四個場景。 <!-- speclink-task:tsk_01KYS5D1WN7K0HG8RA194JVEWB -->
