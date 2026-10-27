## Context

`desktop-settings-mode` 落地後設定模式有分類欄（群組「應用程式」五項）、`settingsSection`、五個設定零件與 `SettingsChoice`／`Switch`；更新狀態機與更新日誌對話框可用；`Wordmark`（橫式鎖版，深色自動切換）在 packages/ui；opener 的 `openUrl` 已允許 `https://github.com/MomoChenisMe/speclink*`。原生選單「關於 Speclink」目前落到設定頁。版本資訊分散：app 版號在前端 `getVersion()`，引擎版號（`ASSET_VERSION`，CLI `--version` 的 engine）只在 Rust。

apply 後實機試用發現應用程式設定有三組重複：「目前版本」「檢查更新」「更新日誌」各在一般、更新、關於三頁；一般頁的介面語言卡等於整個「語言與外觀」頁、CLI 卡等於 CLI 頁第一列；更新頁每個舊版本列的「檢視」與「全部更新日誌」開同一個對話框、內容相同。使用者選定 D6 到 D8 的收斂方式。

## Goals / Non-Goals

**Goals**

- 「關於」群組與「關於 Speclink」頁，依原型四張卡。
- 一個 Tauri 命令回全部版本資訊；「複製版本資訊」一次複製。
- 原生選單「關於 Speclink」直達此頁。
- 第三方授權清單由腳本產生。
- 應用程式設定的每個設定列只出現在一個分類頁（D6、D7）；更新日誌的「檢視」看得到該版（D8）。

**Non-Goals**

- 第三方授權全文；系統原生關於面板；server-web。
- 背景檢查沒找到新版時狀態退回閒置、狀態籤顯示「尚未檢查」——更新狀態機（`apps/desktop/src/core/updater.ts`）不動。
- 專案設定四頁（盤點後無重複）、伺服器頁、CLI 頁內容不動。

## Decisions

### D1 版本資訊命令

`apps/desktop/core/src/project.rs` 新增 `app_info(app_version: &str) -> AppInfo { app_version, engine_version, platform, arch }`：engine_version 取 `speclink_core::init::ASSET_VERSION`——與 CLI `--version` 的「engine vX.Y.Z」同一個值（speclink-core 的 crate 版本跟著 workspace 版號，永遠等於 app 版號，不另列）、platform／arch 取 `std::env::consts::{OS, ARCH}` 並在 desktop-core 轉成顯示文字。Tauri 命令 `app_info` 以 `app.package_info().version` 帶入 app 版號、回 camelCase JSON。前端 `apps/desktop/src/adapter/appInfo.ts`：`loadAppInfo(): Promise<AppInfo>`；store 於首次進關於頁載入並快取 `appInfo`。平台顯示文字：macOS／Windows／Linux＋arch（arm64／x64，與 CLI `--version` 一致），其餘值原樣顯示。

### D2 關於頁

`AboutPage`（apps/desktop/src/views/settings/AboutPage.tsx）：`SettingsPageHeader`「關於 Speclink」＋說明「版本、授權與連結。」

- **品牌卡**：`Card` 內 `Wordmark` 高 40px；其下灰字單行「`<app>` · 引擎 `<engine>` · `<平台 arch>`」＋更新狀態籤＋「複製版本資訊」鈕；再一行 15px 產品描述（i18n `about.tagline`）。不放裸 `BrandMark`。更新狀態籤沿用更新頁 `latestOf(state, null)` 的 `StatusPill`（已是最新 success／有新版本 warning／檢查中 neutral／無法檢查 destructive／未檢查 neutral），包成 `<button>`（無障礙名「更新狀態：<籤文>」），點擊呼叫 `onOpenUpdates`——App 接到 store 既有的 `openSettingsUpdate()`（切到更新頁並聚焦版本卡或下載與安裝卡）。複製鈕為 `CopyButton size="sm"`（label「複製版本資訊」），寫入三行文字 `Speclink <app>` / `Engine <engine>` / `Platform <os> <arch>`；`appInfo` 為 null 時版本列為「—」且不渲染複製鈕。updater 未注入時不渲染狀態籤。
- **不設版本卡**（D6）：沒有「檢查更新」「更新日誌」列，也不記錄上次檢查時間。
- **連結卡**三列整列可點（packages/ui 新增 `SettingsLinkRow`：版面同 `SettingsRow`、整列是按鈕、hover 灰底、右端外開圖示；`SettingsRow` 不變）：「原始碼」（說明等寬 `github.com/MomoChenisMe/speclink`）、「說明文件」（`https://github.com/MomoChenisMe/speclink/tree/main/docs`）、「回報問題」（`https://github.com/MomoChenisMe/speclink/issues/new?body=<URL 編碼的版本三行>`）；皆走 `AboutPage` 匯出的 `openExternal`（`openUrl`，失敗以失敗 toast），原生選單的 GitHub／回報問題共用同一函式與 `REPO_URL`／`ISSUE_URL` 常數。
- **授權卡**兩列：「授權條款」（說明「MIT License · © 2026 MomoChen」；outline「檢視」開 `LicenseDialog`，內容為 `../../../LICENSE?raw`）、「第三方授權」（說明「桌面 app 與 CLI 用到的開源套件清單」；outline「檢視」開同一對話框，內容為 D3 的清單：每筆一列「名稱 版本 · 授權」，npm 與 cargo 兩節）。

### D3 第三方授權清單

`scripts/desktop/third-party-licenses.mjs`：讀 `package-lock.json` 取 `apps/desktop` 與 `packages/ui` 的生產相依（遞迴 `dependencies`，不含 dev），授權取 lock 條目的 `license` 欄（npm 寫 lock 時照抄套件的 package.json，免讀 node_modules）；執行 `cargo metadata --locked --format-version 1 --manifest-path apps/desktop/src-tauri/Cargo.toml`（`--locked`：Cargo.lock 過期時失敗而不改寫，CI 之後的 `cargo test --locked` 鎖檔守門才有效），從 `resolve.nodes` 相依圖以 `speclink-desktop` 與 `speclink-cli` 為根遞迴走正式相依（`dep_kinds` 含 `kind: null`；不含 dev 與 build 相依），workspace 成員只穿過、不列入，取其餘套件的 name／version／license；輸出 `apps/desktop/src/generated/third-party-licenses.json`（`{ npm: [...], cargo: [...] }`，各依名稱排序；授權缺席為 null，對話框以 UI 語言顯示「未標示」）。`apps/desktop/package.json` 的 `predev` 與新增 `prebuild` 串接此腳本；根 `.gitignore` 加 `apps/desktop/src/generated/`（與 `apps/desktop/dist/` 同處；`apps/desktop` 沒有自己的 .gitignore）；測試檔以 fixture lock 與 fake metadata 驗證輸出形狀與排序，不碰真 cargo。缺檔時 `AboutPage` 以空清單呈現「清單尚未產生」。

### D4 分類欄與選單

`settingsCatalog.ts` 加群組「關於」（`group: "about"`）一項 `about`（`rows` 只列關於頁實有的列：複製版本資訊、三個連結、兩個授權），`SettingsColumn` 依群組渲染 `SettingsGroupHeader`（群組內無命中分類時連標題隱藏；群組間的上方間距依實際顯示的第一組決定）；`settingsSection` 型別加 `"about"`；麵包屑「設定 / 關於 Speclink」；原生選單 `about` → `setBoardView("settings")`＋section about。

### D5 文案

i18n 新增 `about.*`（title、description、tagline、engine、updateStatus、copyInfo、linksCard、source、docs、docsHint、report、reportHint、licenseCard、license、licenseHint、thirdParty、thirdPartyHint、notGenerated、licenseUnspecified、view、close）與 `settings.section.about`、`settings.group.about`。D6、D7 收斂後成為孤兒的鍵自兩語系移除：`settings.section.appearance`、`settings.uiLocaleLabel`、`updater.cardTitle`、`updater.help`，並加進 `messages.test.ts` 的「舊鍵已移除」清單；一般頁合併卡的標題用 `settings.interfaceCard`，值改為「語言與外觀」／「Language & Appearance」。兩語系鍵集合相等。

### D6 關於頁只回答「這是哪一版」

關於頁拿掉版本卡（見 D2）。apply 期曾加的 store `lastCheckedAt` 與相對時間文案（`about.lastChecked`、`neverChecked`、`justNow`、`minutesAgo`、`hoursAgo`、`versionCard`、`copyInfoHint`）一併移除，因為它們只服務被拿掉的「檢查更新」列；store 的前景重檢節流變數改回原名 `lastCheckedAt`（閉包層，不進 state）。`AboutPageProps` 去掉 `lastCheckedAt`、加 `onOpenUpdates: () => void`，原本整包的 `updater` 縮為 `updaterState?: UpdaterState`（關於頁只需要狀態）；`AppSettingsView` 的 `about` 面帶 `onOpenUpdates`，並把 `updater?.state` 傳給關於頁。

### D7 一般頁收斂

刪除 `appearance` 分類：`settingsCatalog.ts` 的 `AppSettingsCategory.id` 去掉 `"appearance"`，一般頁的 `rows` 改為顯示語言、外觀、鍵盤快捷鍵三個列標籤鍵；`AppSettingsView` 去掉 appearance 分支。`AppearancePage.tsx` 刪除，`LocaleRow`、`ThemeRow`、`InterfacePrefsProps` 移入 `GeneralPage.tsx`。`GeneralPage` 只渲染「語言與外觀」卡（兩列）、「鍵盤快捷鍵」卡與系統匣面板錯誤列，去掉 `updater`、`cliInstall` props 與 `UpdaterInlineStatus`；`AppSettingsView` 不再把這兩面傳給一般頁。`settingsSection` 只存在記憶體（不持久化），沒有舊值 `appearance` 需要遷移。

### D8 更新日誌檢視單一版本

`ReleaseNotesDialog` 的 browse 模式加選配 `focusVersion?: string`：每個條目的 `<section>` 帶 `data-version`；有 `focusVersion` 時，以捲動區的 callback ref 在節點掛上（每次開啟）時找到對應 section 呼叫 `scrollIntoView({ block: "start" })` 一次，找不到該版時維持頂端——Radix 的 Portal 在開啟後下一輪才掛上內容，effect 執行時還拿不到節點。`AppSettingsUpdaterProps.onShowReleaseNotes` 改為 `(version?: string) => void`；`UpdatesPage` 舊版本列的「檢視」帶該列版號，「全部更新日誌」不帶；App 的 releaseNotes 狀態 browse 分支帶 `focusVersion`。原生選單「說明 › 更新日誌」維持不帶版號。

## Risks / Trade-offs

- **`cargo metadata` 在 predev 的時間**：約 1–3 秒，與 sidecar 建置同列；建置機沒有 cargo、或 Cargo.lock 過期時，腳本以警告略過 cargo 節。
- **LICENSE `?raw` 打包**：約 1KB，無害。
- **連結網址寫死**：倉庫改名要改常數；集中在 `AboutPage` 頂部一處。
- **D7 移除既有分類**：習慣從「語言與外觀」進入的使用者要改點一般頁；分類欄搜尋「外觀」「語言」仍會命中一般頁，原生選單「設定… ⌘,」直達一般頁，影響小。
- **D8 捲動的測試**：jsdom 沒有 `scrollIntoView`，測試以 `vi.fn()` 掛到 `Element.prototype` 後斷言呼叫對象為該版 section；實際捲動位置由手動驗收確認。

## Migration Plan

1. D1 命令與 adapter；D3 腳本與測試。
2. D2 頁與 D4 接線；文案。
3. D6 關於頁收斂；D7 一般頁收斂；D8 更新日誌捲動；孤兒文案清理。

## Open Questions

- 無。
