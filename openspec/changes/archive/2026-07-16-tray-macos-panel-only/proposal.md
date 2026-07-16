## Summary

系統匣樣式由「使用者偏好二選」改為「平台固定」：macOS 固定 webview 面板、其餘平台固定原生選單，並移除設定頁的「系統匣樣式」偏好。

## Motivation

tray-copy-and-panel-mode 的實測裁決（design D7，2026-07-16）：使用者確認 webview 面板滿意，裁決保留面板、移除原生選單樣式。原「系統匣樣式」偏好是兩樣式並存試驗期的 A/B 把手，裁決後已無存在必要；面板僅支援 macOS（NSPanel／vibrancy 專屬），非 macOS 平台的系統匣仍以原生選單為唯一互動面，故拆除範圍限 macOS（使用者確認）。

目標使用者：以桌面 app 監看 SDD 工作流的開發者。使用情境：macOS 上點擊系統匣圖示直接得到面板（無需設定）；Windows／Linux 維持原生選單（含複製、slug 化、分流、溢出全部既有功能）。

## Proposed Solution

- 系統匣互動樣式改由平台決定：macOS 一律面板、非 macOS 一律原生選單；不再讀寫任何樣式偏好。
- 移除設定頁「系統匣樣式」偏好卡與 app 本機偏好模組（trayStyle localStorage 單鍵）；store 的樣式狀態改為執行期記憶（初值依平台），不持久化。
- 面板建立失敗的退回行為保留：macOS 面板建立失敗時退回原生選單（選單程式碼跨平台保留，兼作 macOS 失敗後備），並於設定頁本機設定簽浮出單行錯誤（原樣式卡移除後，錯誤以獨立警示行呈現）。
- 原生選單的既有能力（複製、討論 slug 化與分流、分區溢出）全數保留——非 macOS 平台的正常路徑與 macOS 的失敗後備共用同一實作。

## Non-Goals

- 不拆除原生選單程式碼（跨平台必需＋macOS 失敗後備）。
- 不動面板本體行為（毛玻璃、失焦收合、高度自適應、常駐複製鈕、複製回饋皆維持）。
- 不做 Windows／Linux 的面板支援。
- 不動 crates/（speclink-core、speclink-cli）；無 CLI 介面與輸出變更、無回歸對照影響。
- 不改 .speclink.yaml 與 openspec/config.yaml。

## Alternatives Considered

- 保留設定項、僅把 macOS 預設值改為面板——設定項是試驗期把手，裁決已定，留著是無主的可配置性（違反最小化原則），排除。
- 全平台拆除原生選單——Windows／Linux 將只剩無選單圖示（面板無法跨平台），排除（使用者確認限 macOS）。

## Impact

- Affected specs: tray-status-menu（系統匣圖示與原生選單、面板樣式（macOS）改平台固定敘述）、desktop-config（移除「系統匣樣式偏好」需求）
- Affected code:
  - Modified: apps/desktop/src/tray.ts、apps/desktop/src/store.ts、apps/desktop/src/views/SettingsView.tsx、apps/desktop/src/App.tsx、apps/desktop/src/__tests__/tray.test.ts、apps/desktop/src/__tests__/settingsView.test.tsx、apps/desktop/src/i18n/messages.ts
  - Removed: apps/desktop/src/trayStyle.ts、apps/desktop/src/__tests__/trayStyle.test.ts
- 相容性影響：無 CLI 變更；macOS 使用者升級後系統匣直接為面板（原偏好鍵殘留於 localStorage 但不再讀取，無遷移動作）；非 macOS 行為不變。
