## Why

透過 AI agent 開發的開發者常一次推進好幾個變更，工作主場在終端機或 Herdr 這類 agent 宿主裡；要看某個變更目前的規格時，得切到 Speclink 桌面 app、再自己找到那個專案與變更，步驟多又容易看錯。討論 herdr-speclink-panel 定案「Herdr 管執行、speclink 本體管規劃與閱讀」，兩邊靠一條網址連起來——本變更是那條網址：外部程式送出 `speclink://` 網址，桌面 app 直接打開對應專案的變更或討論詳情頁。使用情境涵蓋 apply、quality 等 workflow 階段中「回頭查規格」的那一步，也是之後 `desktop-roadmap`（同一份討論的第 3 刀）開路線圖的入口基礎。

## What Changes

- 桌面 app（apps/desktop）註冊 `speclink` 網址格式，支援兩種網址：開變更詳情、開討論詳情；網址帶專案資料夾的絕對路徑與變更名稱或討論 slug
- 收到網址時：主視窗顯示並取得焦點；目標專案已是分頁就切過去，否則照「開啟專案」的既有探測流程開啟並加成分頁；該專案的資料載入完成後開啟詳情頁，互斥與宿主規則與系統匣的「開啟此變更」相同
- app 沒在執行時，網址會啟動 app，並在分頁還原後處理該網址；app 已在執行時，交給執行中的那一個處理，不另開第二個 app
- 網址格式不對、或在該專案找不到該變更／討論時，主視窗照常顯示，並以既有的 toast 說明原因；專案路徑的探測結果（不是專案、未初始化、未啟用、帶 remote 標記）沿用「開啟專案」的既有處理
- 網址只能開啟與檢視，不觸發任何寫入；路徑是未啟用或未初始化的資料夾時，沿用既有的確認對話框，確認前零寫入
- macOS 由 Tauri deep-link 外掛註冊網址格式；Windows、Linux 另加 single-instance 外掛把網址轉給執行中的 app
- Windows、Linux：不帶網址再次啟動 app，改為把既有主視窗帶到前景（single-instance 的必然結果，與 macOS 行為一致）
- 相容性影響：不改任何 CLI 指令與輸出（人眼與 `--json` 皆不變、不影響回歸對照）；桌面行為唯一的變化是 Windows、Linux 不再能同時開兩個 app，使用者無需遷移

## Non-Goals

- 不支援 remote 專案的網址（只處理本機專案資料夾）
- 不把 linked worktree 路徑換算成主 checkout：呼叫端應傳主 checkout 的路徑；傳入 worktree 路徑時照「開啟專案」的既有探測結果處理
- 不在本變更加開路線圖的網址：留給 `desktop-roadmap`
- 不提供 `speclink open` 之類的 CLI 動詞或 `speclink-desktop` 啟動參數當入口：討論已否決，兩者仍要經網址或另一條通道通知 app
- 不改 Herdr 端：Herdr mod 依討論結論以 `feat(herdr)` 另外提交

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 新增「以 speclink 網址開啟詳情頁」需求——外部程式以網址叫桌面 app 開啟指定專案的變更或討論詳情頁

## Impact

- Affected specs: desktop-app
- Affected code:
  - New:
    - apps/desktop/src/deepLink.ts
    - apps/desktop/src/__tests__/deepLink.test.ts
  - Modified:
    - apps/desktop/src-tauri/Cargo.toml
    - apps/desktop/src-tauri/src/lib.rs
    - apps/desktop/src-tauri/tauri.conf.json
    - apps/desktop/src-tauri/capabilities/default.json
    - apps/desktop/package.json
    - apps/desktop/src/App.tsx
    - apps/desktop/src/store.ts
    - apps/desktop/src/tray.ts
    - apps/desktop/src/__tests__/store.test.ts
    - apps/desktop/src/i18n/messages.ts
    - Cargo.lock
    - package-lock.json
- 影響的 app：apps/desktop（Tauri 殼與前端）；speclink-core、speclink-cli 不動
- 設定：tauri.conf.json 新增 deep-link 外掛設定（網址格式 `speclink`），無使用者設定欄位
- 依賴：新增 Rust crate tauri-plugin-deep-link、tauri-plugin-single-instance（Windows、Linux），前端套件 @tauri-apps/plugin-deep-link
