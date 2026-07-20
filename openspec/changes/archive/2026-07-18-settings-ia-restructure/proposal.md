## Summary

設定頁資訊架構重構：把與專案無關的「本機設定」「伺服器」抽成獨立的應用程式設定頁，config.yaml／.speclink.yaml 歸入各專案自己的專案設定頁。

## Motivation

現行設定頁把兩層專案設定檔（config.yaml、.speclink.yaml）與應用程式層級內容（本機設定、伺服器連線）混在同一組頁簽，且整頁掛在 active session 之下，造成三個實際問題：

1. 應用程式層級的內容（介面語言、伺服器連線）與現行分頁無關，卻要先開一個專案才能操作——零分頁時設定入口整個不可達，新使用者無法把 remote workspace 當第一個 workspace 開啟。
2. remote 分頁下 config.yaml／.speclink.yaml 兩簽不適用，目前以「四簽＋兩張不可用提示卡」的方式呈現（remote-data-source 驗證期的臨時修補、未入規格），資訊架構混淆。
3. 使用者於 remote workspace 手動驗證時明確反映：希望本機設定與伺服器獨立成一頁、專案設定自成一頁。

## Proposed Solution

- 側欄底部的「設定」改為**應用程式設定頁**：含「本機設定」「伺服器」兩簽，不依賴 active session，零分頁時仍可進入與操作。
- 新增**專案設定頁**入口：含「config.yaml」「.speclink.yaml」兩簽，內容與行為沿用現行實作（三卡、寫入驗證、解析失敗簽級警示）；remote 分頁下整頁呈現單一不可用說明。
- 移除 SettingsView 的 workspaceSettingsNotice 四簽提示卡分支——由新的頁面拆分取代。

## Alternatives Considered

- 維持單一設定頁、只把預設簽依 session 種類切換：已實作為臨時修補，但應用程式層級內容仍被鎖在 session 之下，零分頁不可達的缺口沒解。
- 應用程式設定做成獨立視窗（如 macOS Preferences）：超出需求，桌面 app 現行導覽皆為單視窗切頁，不引入新視窗管理。

## Impact

- Affected specs: `desktop-config`（設定頁組織需求改寫為兩頁）、`desktop-app`（側欄導覽結構加入專案設定項）、`desktop-connections`（伺服器面移至應用程式設定頁）
- Affected code:
  - Modified: `apps/desktop/src/App.tsx`（導覽項與路由接線）、`apps/desktop/src/store.ts`（boardView 增加專案設定值）、`apps/desktop/src/i18n/messages.ts`（導覽與頁簽文案）、`apps/desktop/src/__tests__/App.test.tsx`
  - New: `apps/desktop/src/views/AppSettingsView.tsx`、`apps/desktop/src/views/ProjectSettingsView.tsx`、`apps/desktop/src/__tests__/appSettingsView.test.tsx`、`apps/desktop/src/__tests__/projectSettingsView.test.tsx`
  - Removed: `apps/desktop/src/views/SettingsView.tsx`、`apps/desktop/src/__tests__/settingsView.test.tsx`（內容拆分搬移至上述新檔）
