## Context

`desktop-settings-mode` 落地後設定模式有分類欄（群組「應用程式」五項）、`settingsSection`、五個設定零件與 `SettingsChoice`／`Switch`；更新狀態機與更新日誌對話框可用；`Wordmark`（橫式鎖版，深色自動切換）在 packages/ui；opener 的 `openUrl` 已允許 `https://github.com/MomoChenisMe/speclink*`。原生選單「關於 Speclink」目前落到設定頁。版本資訊分散：app 版號在前端 `getVersion()`、技能檔版本 `ASSET_VERSION` 與引擎版本只在 Rust。

## Goals / Non-Goals

**Goals**

- 「關於」群組與「關於 Speclink」頁，依原型四張卡。
- 一個 Tauri 命令回全部版本資訊；「複製版本資訊」一次複製。
- 原生選單「關於 Speclink」直達此頁。
- 第三方授權清單由腳本產生。

**Non-Goals**

- 第三方授權全文；系統原生關於面板；server-web。

## Decisions

### D1 版本資訊命令

`apps/desktop/core/src/project.rs` 新增 `app_info(app_version: &str) -> AppInfo { app_version, engine_version, platform, arch, assets_version }`：engine_version 取 speclink-core 的 crate 版本（speclink-core 既有的 `pub const VERSION: &str = env!("CARGO_PKG_VERSION")`）、platform／arch 取 `std::env::consts::{OS, ARCH}` 並在 desktop-core 轉成顯示文字、assets_version 取 `speclink_core::init::ASSET_VERSION`。Tauri 命令 `app_info` 以 `app.package_info().version` 帶入 app 版號、回 camelCase JSON。前端 `apps/desktop/src/adapter/appInfo.ts`：`loadAppInfo(): Promise<AppInfo>`；store 於首次進關於頁載入並快取 `appInfo`。平台顯示文字：macOS／Windows／Linux＋arch（arm64／x64，與 CLI `--version` 一致），其餘值原樣顯示。

### D2 關於頁

`AboutPage`（apps/desktop/src/views/settings/AboutPage.tsx）：`SettingsPageHeader`「關於 Speclink」＋說明「版本、授權與連結。」

- **品牌卡**：`Card` 內 `Wordmark` 高 40px；其下灰字單行「`<app>` · 引擎 `<engine>` · `<平台 arch>` · 更新狀態」（更新狀態為 `StatusPill`：已是最新 success／有新版本 warning／檢查中 neutral／無法檢查 destructive／未檢查 neutral）；再一行 15px 產品描述（i18n `about.tagline`）。不放裸 `BrandMark`。
- **版本卡**三列：「檢查更新」（說明含上次檢查時間——store 新增 `lastCheckedAt`，手動與背景檢查完成時更新，顯示為相對時間；控制為 outline 鈕，與更新頁同一 `checkForUpdates(true)`）、「更新日誌」（outline「檢視」開瀏覽模式對話框）、「複製版本資訊」（outline「複製」：寫入四行文字 `Speclink <app>` / `Engine <engine>` / `Platform <os> <arch>` / `Skills <assets>`，沿 `CopyButton` 的已複製回饋）。
- **連結卡**三列整列可點（`SettingsRow` 的 `onClick` 變體：hover 灰底、右端外開圖示）：「原始碼」（說明等寬 `github.com/MomoChenisMe/speclink`）、「說明文件」（`https://github.com/MomoChenisMe/speclink/tree/main/docs`）、「回報問題」（`https://github.com/MomoChenisMe/speclink/issues/new?body=<URL 編碼的版本四行>`）；皆走 `openUrl`，失敗以失敗 toast。
- **授權卡**兩列：「授權條款」（說明「MIT License · © 2026 MomoChen」；outline「檢視」開 `LicenseDialog`，內容為 `../../../LICENSE?raw`）、「第三方授權」（說明「桌面 app 與 CLI 用到的開源套件清單」；outline「檢視」開同一對話框，內容為 D3 的清單：每筆一列「名稱 版本 · 授權」，npm 與 cargo 兩節）。

### D3 第三方授權清單

`scripts/desktop/third-party-licenses.mjs`：讀 `package-lock.json` 取 `apps/desktop` 與 `packages/ui` 的生產相依（遞迴 `dependencies`，不含 dev），授權取 lock 條目的 `license` 欄（npm 寫 lock 時照抄套件的 package.json，免讀 node_modules）；執行 `cargo metadata --format-version 1 --manifest-path apps/desktop/src-tauri/Cargo.toml` 取非 workspace 成員套件的 name／version／license；輸出 `apps/desktop/src/generated/third-party-licenses.json`（`{ npm: [...], cargo: [...] }`，各依名稱排序；授權缺席為 null，對話框以 UI 語言顯示「未標示」）。`apps/desktop/package.json` 的 `predev` 與新增 `prebuild` 串接此腳本；根 `.gitignore` 加 `apps/desktop/src/generated/`（與 `apps/desktop/dist/` 同處；`apps/desktop` 沒有自己的 .gitignore）；測試檔以 fixture lock 與 fake metadata 驗證輸出形狀與排序，不碰真 cargo。缺檔時 `AboutPage` 以空清單呈現「清單尚未產生」。

### D4 分類欄與選單

`settingsCatalog.ts` 加群組「關於」（`group: "about"`）一項 `about`，`SettingsColumn` 第二個 `SettingsGroupHeader`；`settingsSection` 型別加 `"about"`；麵包屑「設定 / 關於 Speclink」；原生選單 `about` → `setBoardView("settings")`＋section about。

### D5 文案

i18n 新增 `about.*`（title、description、tagline、engine、versionCard、checkUpdate、lastChecked、neverChecked、releaseNotes、copyInfo、copyInfoHint、linksCard、source、docs、docsHint、report、reportHint、licenseCard、license、licenseHint、thirdParty、thirdPartyHint、notGenerated、view）與 `settings.section.about`、`settings.group.about`；兩語系鍵集合相等。

## Risks / Trade-offs

- **`cargo metadata` 在 predev 的時間**：約 1–3 秒，與 sidecar 建置同列；建置機沒有 cargo 時腳本以警告略過 cargo 節。
- **LICENSE `?raw` 打包**：約 1KB，無害。
- **連結網址寫死**：倉庫改名要改常數；集中在 `AboutPage` 頂部一處。

## Migration Plan

1. D1 命令與 adapter；D3 腳本與測試。
2. D2 頁與 D4 接線；文案。

## Open Questions

- 無。
