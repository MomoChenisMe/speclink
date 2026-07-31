## Why

macOS 系統匣面板有三個體驗缺口，皆由使用者截圖回報：(1) 內容超過上限高度後整頁捲動，專案 tab 條與動作區跟著被捲走，長清單下導覽與常用動作不可及；(2) HudWindow 毛玻璃無亮度自適應，深色背景（深色 IDE／終端機）下面板明顯偏暗，與 macOS 26 原生選單（Liquid Glass 亮度自適應）視覺落差大；(3) 動作區只有 app 層「設定」入口，缺少與主視窗側欄對應的「專案設定」，得先開主視窗再點側欄才到得了。三者同屬 tray 面板呈現層、同批檔案，併一個 change 修正。

目標使用者：把桌面 app 常駐系統匣、透過 tray 快速掌握變更／討論狀態的開發者。使用情境：日常自 tray 面板查看各階段進度、跳轉主視窗各頁（對應 SDD workflow 全階段的狀態一瞥面）。

## What Changes

- 面板版面改三段式：固定頁首（專案 tab 條＋分割線）、可捲中段（討論／已轉出／生命週期分區卡，含復原卡與 stale 條分支）、固定頁尾（分割線＋動作區）。捲動面從 body 移入中段容器；中段捲軸維持 WebKit 預設 overlay 捲軸（捲動時浮現、自動淡出）；root 主色漸層 wash 固定於視窗、不再隨內容捲動。
- 高度自適應量測基準改為「頁首高＋中段自然內容高＋頁尾高」，上限維持現值；內容未超限時面板貼合、無內部捲動（既有要求維持不變）。
- 毛玻璃補光：維持 HudWindow 材質，面板 root 增加主題色半透明補光底層（跟隨系統深淺色模式），深色背景下亮度明顯上提、淺色背景下毛玻璃仍可辨；全平台單一路徑、不分 macOS 版本。
- 動作區新增「專案設定」：順序改為「開啟 Speclink → 專案設定 → 設定 → 結束」；點擊喚起主視窗並切至作用中專案的專案設定頁；文案沿用既有「專案設定」詞條、圖示沿用主視窗側欄同款；於 tray 選單模型層新增——macOS 面板與非 macOS 原生選單同步取得入口。

## Non-Goals

- 不引入 NSGlassEffectView 真 Liquid Glass（objc2 執行期插入＋版本偵測 fallback）——討論裁定暫緩，補光實測不滿意時另案重啟。
- 不換 vibrancy 材質：Menu／Popover 同屬無亮度自適應的舊世代材質，且 Menu 已於 2026-07-16 實測裁決淘汰（近乎不透、毛玻璃不可辨）。
- 不動 Rust 面板視窗行為：NSPanel 轉換、貼齊定位、失焦收合、材質套用皆維持現狀。
- 不改非 macOS 原生選單的區段順序與其餘行為——僅動作區同步新增「專案設定」一項。
- 中段捲軸不隱藏也不常駐——維持系統 overlay 預設（隱藏失去可捲提示、常駐佔寬突兀，皆已裁決排除）。
- recovery／stale 狀態不對「專案設定」特判守門——導覽後的呈現交由主視窗既有行為（與現行「設定」一致）。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `tray-status-menu`: 面板內部捲動限於中段內容分區（專案 tab 條與動作區常駐不捲動）；面板底色錨定主題背景色（深色背景下不明顯偏暗、淺色背景下毛玻璃仍可辨）；動作區由三項改四項（新增「專案設定」——原生選單需求與面板區塊順序兩處同步）。

## Impact

- Affected specs: `tray-status-menu`（delta：面板樣式（macOS）的捲動範圍與底色要求、原生選單與面板的動作區項目）
- Affected code（全部在 apps/desktop 前端；無 Rust 端、無 CLI 改動）:
  - Modified: apps/desktop/panel.html、apps/desktop/src/panel/main.tsx、apps/desktop/src/panel/TrayPanel.tsx、apps/desktop/src/tray.ts、apps/desktop/src/__tests__/tray.test.ts、apps/desktop/src/__tests__/trayPanel.test.tsx
  - New: (none)
  - Removed: (none)
- 相容性影響：非 macOS 原生選單動作區多一項「專案設定」（純新增、無移除或改序既有項）；CLI 人眼輸出與 `--json` 契約無任何變動，無 golden 回歸影響；i18n 沿用既有詞條（zh-TW 與 en 皆已存在），無新增鍵。
