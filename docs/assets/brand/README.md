# Speclink 品牌資產

**繁體中文** · [English](README.en.md)

正式標誌採「接合 S」：兩段形狀在中央接合，代表需求、規格與實作之間的連結。App 圖示使用青綠圓角底板與白色 S，底板外側透明；Windows 的圓角直接存在圖檔內，不依賴系統遮罩。

## 配色

| 角色 | 色值 | 用途 |
| --- | --- | --- |
| 青綠 | `#167873` | 標記、App 底板、`link` 字樣 |
| 深灰 | `#292c32` | 淺色背景的 `Spec` 字樣 |
| 淺灰白 | `#f5f5f5` | 深色背景的 `Spec` 字樣 |
| 淺青綠 | `#4bb9b3` | 深色背景的標記與 `link` 字樣 |

品牌固定色值與介面 theme token 分開管理；介面主色仍以 `packages/ui/src/theme.css` 為準。

## 檔案

`svg/` 是可編輯的向量原稿，文字已轉成路徑，不需安裝字型。`transparent/` 是透明 PNG；本目錄的同名 PNG 為白底（`-dark` 為深色底）版本。

| 檔名（省略副檔名） | 內容 |
| --- | --- |
| `speclink-app-icon` | 圓角底板＋白色 S，供 App／favicon 使用 |
| `speclink-logo-mark` | 青綠 S，無底板、無文字 |
| `speclink-wordmark` | 純文字 `Speclink`，Spec 深灰、link 青綠 |
| `speclink-wordmark-dark` | 深色背景用純文字版 |
| `speclink-logo-horizontal` | S＋字樣的橫式組合 |
| `speclink-logo-horizontal-dark` | 深色背景用橫式組合 |
| `speclink-logo-vertical` | S 在上、字樣在下 |
| `speclink-logo-system-sheet` | 全套組合與縮小尺寸一覽（展示板有背景） |

## App 使用位置

- 介面（Desktop 與 Server Web 共用）：`packages/ui/src/assets/` 的三個 SVG——`logo-mark.svg`（裸標記，由 `speclink-logo-mark.svg` 複製並把填色改成 `currentColor`）、`logo-horizontal.svg` 與 `logo-horizontal-dark.svg`（逐字複製）；由 `packages/ui/src/components/Brand.tsx` 的 `BrandMark`（標記跟主色走）與 `Wordmark`（橫式鎖版，深色偏好取深色版）渲染。換 Logo 只換這三個檔。
- Desktop favicon：`apps/desktop/public/logo-mark.png`（`index.html` 引用）。
- 原生 App：`apps/desktop/src-tauri/icons/`，包含 Windows `.ico`、macOS `.icns` 與各尺寸 PNG。
- 系統匣：18／36 px 單色 S（無底板），macOS 以 alpha 作為系統 template；36 px 版本同時內嵌於 `apps/desktop/src/trayIcon.ts`。

重新產生原生圖示：

```sh
npx tauri icon docs/assets/brand/transparent/speclink-app-icon.png -o /tmp/speclink-icons
```

只將輸出目錄頂層的桌面圖示複製到 `apps/desktop/src-tauri/icons/`；不收錄自動產生的 iOS／Android 資產。系統匣使用獨立的單色圖檔，更新後需同步其 base64 常數。
