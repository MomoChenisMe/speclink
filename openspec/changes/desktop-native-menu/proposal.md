## Summary

讓桌面 app（apps/desktop）在 macOS 擁有自己的原生選單列：六組選單（Speclink、檔案、編輯、檢視、視窗、說明）跟著 UI 語言偏好顯示繁中或英文、切換語言時立即重建，並補上 app 目前沒有的功能入口——開啟專案 ⌘O、關閉專案 ⌘W、變更／已封存／規格／手冊 ⌘1–4、專案設定、搜尋看板 ⌘F、重新整理 ⌘R、下一個專案 ⌃Tab、檢查更新、安裝 CLI、更新日誌、GitHub 與回報問題。Windows／Linux 不做選單列。這是討論 desktop-ui-redesign 的 cut 2 外殼刀的第三塊，與 desktop-shell-redesign 互不相依。

## Motivation

目標使用者是透過 AI 代理跑 SDD 的開發者／PO／PM，使用情境是 macOS 上日常切專案、切頁、找東西。現況是 Tauri 的預設選單列：英文、只有 app 名稱與系統預設項（About、Hide、Quit、Edit、Window），介面切成繁中後選單列仍是英文；app 的切頁、切專案、搜尋、重新整理、檢查更新都沒有選單入口，快捷鍵只有 ⌃Tab 與 ⌃1–9 兩組；說明文件與回報問題要自己去找網址。系統匣選單已經用同一套 JS 選單 API 從 i18n 字典建、跟語言偏好走，app 選單列照同一條路做即可，不需要在 Rust 端另立一套文案。

## Proposed Solution

1. **選單模型純函式**：`apps/desktop/src/appMenu.ts` 的 `buildAppMenuModel({ t, hasProject })` 回傳六組選單的宣告式模型（id、文案鍵、快捷鍵、enabled、動作 id 或 predefined 項），可單元測試；文案全部取自 `apps/desktop/src/i18n/messages.ts` 兩語系（新增 `menu.*` 鍵）。
2. **以 JS 選單 API 安裝**：`installAppMenu(deps)` 只在 macOS 以 `@tauri-apps/api/menu` 的 `Menu.new` 把整棵選項物件一次建出並 `setAsAppMenu()`；UI 語言或「有無專案」改變時整個重建一次（與系統匣選單同一做法）。非 macOS 不安裝、不呼叫任何選單 API。
3. **六組內容**：Speclink（關於 Speclink → 本塊先開應用程式設定頁、cut 4 的「關於」頁落地後改指向它；檢查更新… → 手動檢查並開設定頁軟體更新卡；設定… ⌘,；服務；隱藏 ⌘H、隱藏其他 ⌥⌘H、顯示全部；結束 ⌘Q）、檔案（開啟專案… ⌘O；關閉專案 ⌘W；安裝 CLI… → 開設定頁）、編輯（復原、重做、剪下、複製、貼上、全選——WebView 的文字輸入靠這組預設項才有快捷鍵）、檢視（變更 ⌘1、已封存 ⌘2、規格 ⌘3、手冊 ⌘4、專案設定；搜尋看板 ⌘F；重新整理 ⌘R；下一個專案 ⌃Tab）、視窗（縮到最小 ⌘M、縮放、全部移到最前）、說明（手冊、更新日誌、GitHub、回報問題）。沒有作用中專案時，檔案的關閉專案與檢視組全部、說明的手冊為停用。
4. **動作接法**：切頁、開專案、關專案、重新整理、下一個專案、檢查更新直接呼叫 store 既有動作；搜尋看板與更新日誌以 `window` 自訂事件（`speclink:focus-search`、`speclink:show-release-notes`）交給既有的看板搜尋列與更新日誌對話框——原生快捷鍵被選單吃掉後 WebView 收不到 keydown，所以搜尋列除了既有的 ⌘F keydown 也要聽這個事件；GitHub 與回報問題以 opener 外掛的 `openUrl` 開瀏覽器（Rust 端已註冊外掛；前端補裝 `@tauri-apps/plugin-opener`，能力檔加 `opener:allow-open-url` 並限定 GitHub 網址）。
5. **規格**：desktop-app 新增需求「macOS 原生選單」；desktop-config「UI 介面語言支援 zh-TW 與 en」加一句「macOS 原生選單列跟 UI 語言、切換即重建」。

不新增或變更任何 CLI 指令、技能或 Agent 指令。設定欄位：無。相容性：`--json` 與 CLI 輸出不變；既有 ⌃Tab／⌃1–9 與看板 ⌘F 的行為不變（只是多了選單入口）。

## Non-Goals

- 不做 Windows／Linux 的選單列（隱藏標題列後的行為無機器可查，討論 Deferred）。
- 不建「關於 Speclink」頁（cut 4）；本塊「關於」先開設定頁。
- 不改系統匣選單。
- 不改快捷鍵的既有 keydown 處理（⌃Tab、⌃1–9、⌘F 的 keydown 保留給非 macOS）。

## Alternatives Considered

- Rust `MenuBuilder` 自建並在 Rust 端放兩語系文案——討論結論原本這樣寫；但系統匣選單已以 JS 選單 API 從同一份 i18n 字典建、語言切換時重建，app 選單列走同一條路就只有一份文案與一種重建機制；Rust 端另立文案表會讓「兩語系鍵集合相等」的守門失效一半。選單模型仍是純函式、有單元測試，Tauri 殼不多任何 command。
- 用系統預設的關於面板——放不下鎖版與連結、不跟 UI 語言（討論否決）。

## Impact

- Affected specs: `desktop-app`（ADDED「macOS 原生選單」）、`desktop-config`（MODIFIED「UI 介面語言支援 zh-TW 與 en」——原生選單列跟 UI 語言）
- Affected code:
  - Modified: `apps/desktop/src/App.tsx`、`apps/desktop/src/i18n/messages.ts`、`apps/desktop/src-tauri/capabilities/default.json`、`apps/desktop/package.json`、`package-lock.json`、`packages/ui/src/components/BoardSearchBar.tsx`、`apps/desktop/src/__tests__/App.test.tsx`、`apps/desktop/src/__tests__/messages.test.ts`、`packages/ui/src/__tests__/boardSearchBar.test.tsx`
  - New: `apps/desktop/src/appMenu.ts`、`apps/desktop/src/__tests__/appMenu.test.ts`
  - Removed: 無
