## Context

release 管線（.github/workflows/release.yml）已對 tag push 產出 CLI＋server 五 target 壓縮檔、SHA256SUMS 與 Docker 映像，並承諾「四形態全有全無」；desktop（apps/desktop/src-tauri，Tauri 2，bundle 已 active）明文不在 release 產物內。本變更把 desktop 升格為第五形態：三平台安裝檔＋自動更新，更新端點用 GitHub Releases 靜態檔，不自架伺服器。前情裁定（討論 one-click-install-and-run、keychain-always-allow-reprompt）：暫不投資 OS 簽章，但開關要預留。

## Goals / Non-Goals

**Goals**

- tag push 一次產出 desktop 三平台安裝檔與更新描述檔，隨 GitHub Release 發布
- desktop 內建自動更新：啟動檢查、徵得同意、套用重啟；更新包以自產金鑰簽名驗證
- 安裝檔內含 CLI；desktop 提供「安裝 CLI 指令」動作把 CLI 帶進 PATH，並隨 desktop 更新同版
- OS 簽章做成 env-gated 開關：secrets 不存在時照常產未簽章版

**Non-Goals**

- 不做 notarization／不購憑證；不自架 update server；CLI 不做獨立 self-update
- 不含 clone 開發者一鍵執行與文件（change dev-quickstart-and-docs）
- 不做分階段灰度、更新統計、差分更新

## Decisions

### D1：沿用既有 matrix 自組 desktop build job，不改用 tauri-action

desktop 安裝檔由新增的 workflow job 以 Tauri CLI 建置：macOS 出 dmg（aarch64 與 x86_64 各一）、Windows 出 NSIS 安裝器（x86_64）、Linux 出 AppImage 與 deb（x86_64 與 aarch64）。Linux runner 需先安裝 GTK／WebKit 系統庫（ci.yml 已有同款前例）。

- 替代：tauri-action 一站包辦建置＋latest.json＋上傳——但它自成一套 release 流程，與既有自組 matrix、smoke、checksums、全有全無 gating 疊床架屋；沿用自組 job 讓五形態進同一個 needs 閘門。
- 替代：macOS 出 universal 單檔——單一產物較簡潔，但需額外 lipo 流程且體積翻倍；per-arch 直接復用既有兩列 mac matrix。

### D2：更新描述檔由 workflow 組裝，端點固定 GitHub Releases

Tauri CLI 建置時對每個更新包產 minisign 簽章檔；workflow 收齊各平台產物後，由 scripts/release-latest-json.mjs 組裝 latest.json（version 對齊 tag、各平台 url 指向同一 release 的 asset、signature 內嵌對應簽章），與安裝檔一併上傳。app 內 updater 端點固定為 releases/latest/download/latest.json 這一個 URL。

- 替代：自架 update server——僅灰度／統計／私有 repo 需要，全數不在情境（討論已排除）。
- 替代：手寫 latest.json 附進 release——每版手工易錯，組裝腳本一次寫死格式。

### D3：更新完整性用 Tauri updater 自產金鑰，與 OS 簽章正交

以 Tauri CLI 產一組 minisign 金鑰對：公鑰寫進 tauri.conf.json 的 updater 設定（隨版本庫提交，非機密）；私鑰與密語只存 GitHub secrets（TAURI_SIGNING_PRIVATE_KEY／TAURI_SIGNING_PRIVATE_KEY_PASSWORD），CI 建置時簽更新包。簽名驗證失敗的更新 app 一律拒裝。私鑰另做離線備份——遺失私鑰等於換鑰匙，舊版 app 將驗不過新簽名、只能手動重裝。

- 替代：不簽更新包——tauri-plugin-updater 強制要求簽名，且未簽章 OS 產物更需要傳輸層以外的完整性保障，無此選項。

### D4：OS 簽章開關＝secrets 存在才跑的條件步驟

workflow 內 macOS 簽章／Windows 簽章步驟以 secrets 是否存在為條件：不存在→跳過、產未簽章安裝檔、workflow 綠燈；存在→執行簽章。本變更只落地條件骨架與文件註記，不填任何鑰匙。

- 替代：現在購憑證一步到位——成本先於需求（討論已排除，觸發時機記於 Deferred）。

### D5：CLI 以 sidecar 隨附，佈署進 PATH 依平台分流

CLI binary 以 Tauri externalBin（sidecar）隨安裝檔佈署。進 PATH 的方式：

- **macOS**：desktop 內「安裝 CLI 指令」動作在 ~/.local/bin 建 symlink 指進 app bundle——app 路徑固定（/Applications/Speclink.app），更新後 symlink 自動指向新版。
- **Windows**：NSIS 安裝器 hook 把安裝目錄寫入使用者 PATH，更新重跑安裝器即同版，app 內動作僅顯示狀態。
- **Linux deb**：佈署 /usr/bin/speclink，包管理器負責同版。
- **Linux AppImage**：掛載點隨執行變動、symlink 不可行，改為把 CLI 複製到 ~/.local/bin；app 每次啟動比對版本，不符即重新複製（自我修復，免使用者記得重裝）。

搭配規則：動作後偵測 ~/.local/bin 是否在 PATH，不在則提示加入方式。

- 替代：mac 用 /usr/local/bin（VS Code 模式）——需提權對話框，~/.local/bin 免提權且為現代慣例；PATH 未含時提示補上即可。
- 替代：CLI self-update 子指令——與 desktop 更新重複（討論已排除）。

### D6：更新流程邏輯歸 apps/desktop/core，Tauri 殼單行委派

檢查更新、徵求同意、套用、失敗處理的狀態機與文案狀態放 apps/desktop/core（純邏輯、可獨立測試）；src-tauri 的 command 只委派 plugin 呼叫。

（落點澄清：本 change 各處「apps/desktop/core」指前端純邏輯層 **apps/desktop/src/core/**——TypeScript、不依賴 Tauri、vitest 可測；與既有 Rust crate speclink-desktop-core（目錄 apps/desktop/core/）無涉。更新與 CLI 佈署的狀態機、平台分流判定皆落於此；Tauri 殼的委派走 tauri-plugin-updater 的前端綁定與薄 command。）啟動時非阻塞背景檢查，檢查失敗（離線、端點 404）靜默不擋啟動；找到新版→通知列顯示版本並徵求同意→下載套用→提示重啟。無背景靜默安裝。

- 替代：靜默自動裝——開發工具使用者預期掌控重啟時機，且未簽章環境下靜默替換二進位觀感不佳。

## Implementation Contract

**In scope 的可觀察行為**

1. push tag v* 後，該 GitHub Release 的 assets 包含：既有五 target CLI／server 壓縮檔（命名與內容不變）、Speclink dmg（aarch64、x86_64）、NSIS 安裝器（x64）、AppImage 與 deb（x64、arm64）、各更新包的簽章、latest.json；SHA256SUMS.txt 收錄全部檔案。任一形態失敗則 Release 不發布（needs 閘門）。
2. latest.json：version 等於 tag（去 v 前綴）、platforms 至少含 darwin-aarch64／darwin-x86_64／windows-x86_64／linux-x86_64 各自的 url（指向同 release asset）與 signature；由組裝腳本產出，腳本有單元測試（node --test，掛進 scripts 既有測試 glob）。
3. app 啟動後背景檢查更新：有新版→通知徵求同意→同意後下載套用→重啟為新版；檢查失敗靜默；簽名驗證失敗→拒裝並顯示錯誤。手動「檢查更新」入口同義。
4. 安裝 CLI 動作完成後：終端執行 speclink --version 與 desktop 版本一致；動作介面呈現已安裝／未安裝／版本不符狀態；AppImage 版本不符時 app 啟動自動重佈署。
5. 簽章開關：repo 無簽章 secrets 時 workflow 全綠且產未簽章安裝檔——驗收即現況 CI 成功＋assets 齊全。

**Out of scope**：notarization 與憑證採購、update server、CLI self-update、灰度／統計、開發文件（change dev-quickstart-and-docs）。

**驗收落點**：apps/desktop/core 更新狀態機與 CLI 佈署邏輯的 vitest 測試；scripts/release-latest-json.mjs 的 node --test；desktop 真實視窗驗證更新徵詢與安裝 CLI 動作；release workflow 以 tag 預演驗 assets 清單。

## Risks / Trade-offs

- [未簽章 dmg 在 macOS Sequoia 需系統設定放行，第一印象差] → 文件放行教學（change ②）＋簽章開關已預留，觸發時機明訂於討論 Deferred
- [未簽章＋自動更新：每次更新 cdhash 變，remote 用戶鑰匙圈重授權數次] → 已為接受的取捨，寫入使用者文件（change ②）；純本地用戶無感
- [updater 私鑰洩漏＝可推送惡意更新；遺失＝舊版驗不過新簽名] → 私鑰僅存 GitHub secrets＋離線備份；洩漏時換鑰匙出新版並公告手動重裝
- [desktop build 失敗會擋下整個 release（五形態全有全無）] → 刻意選擇，與既有 Docker gating 同理；desktop build 先在 ci.yml 級別驗證常綠
- [Linux runner 缺 GTK／WebKit 系統庫] → workflow 安裝步驟援引 ci.yml 既有依賴清單
- [跨平台 PATH 佈署行為分歧（symlink／NSIS／deb／複製）] → 分流邏輯集中 apps/desktop/core 並逐平台單元測試；AppImage 以啟動時版本比對自我修復
- [回歸對照] → CLI 人眼輸出與 --json 完全不動，render golden 不受影響；release.yml 既有 steps 不改名不改序，只增列

## Migration Plan

無既有安裝用戶。首個含 updater 的版本即更新基線：更早的手動安裝版無自動更新能力，屬預期；文件註明從該版起支援。

## Open Questions

無——簽章投資觸發時機屬討論 Deferred，非本變更待決項。
