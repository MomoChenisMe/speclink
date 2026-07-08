# Speclink 品牌資產

Speclink 的 Logo 與圖示來源。標誌由「文件輪廓 + 電路節點」構成——規格是文件，節點與走線是規格之間的連結與流程。

## 配色

| 角色 | 值 | 用途 |
| --- | --- | --- |
| Ink（墨藍） | 深墨藍 navy | 文件輪廓、`Spec` 字樣、主要線條 |
| Teal（主色） | `oklch(0.52 0.1 192)` | 電路節點、走線、`link` 字樣、app 主色（`--primary`） |

app 端的完整色票見 `apps/desktop/src/index.css`（Tailwind v4 token，主色 hue 192）。

## 檔案

正式資產放在本目錄，每種鎖版（lockup）都有實心白底與去背兩版：

| 檔案 | 說明 | 建議用途 |
| --- | --- | --- |
| `speclink-logo-horizontal.png` | 橫式鎖版（mark + 字樣並排） | README hero、網站頁首、文件封面 |
| `speclink-logo-vertical.png` | 直式鎖版（mark 在上、字樣在下） | 方形版面、社群大頭貼、啟動畫面 |
| `speclink-logo-mark.png` | 無文字 mark（icon only） | app／視窗圖示、favicon、小尺寸場景 |
| `speclink-wordmark.png` | 純文字 wordmark（無 mark） | 與 mark 併排的頁首鎖版、需要文字標誌但不要圖示時 |
| `speclink-logo-system-sheet.png` | 三種鎖版一覽 | 對外提案、簡報 |
| `transparent/` | 上述各檔的去背版本 | 疊在非白色背景上時使用 |

桌面 app 由 `transparent/` 的兩張圖供給：`speclink-logo-mark.png` 合成 `apps/desktop/src-tauri/icons/`（`tauri icon`，視窗／工作列圖示）與 `apps/desktop/public/logo-mark.png`（頁首圖示與 favicon）；`speclink-wordmark.png` 裁切至緊邊界後為 `apps/desktop/public/speclink-wordmark.png`（頁首文字標誌，與 mark 併排）。

## explorations/

早期概念稿與被淘汰的方向，**不作為正式資產使用**，僅保留設計脈絡。`selected-*` 是選定方向的高解析原稿。

## 深色背景注意

正式鎖版為墨藍字，設計給淺色背景。疊在深色背景時，優先使用實心白底版本以確保可讀性（尚無深色專用變體）。
