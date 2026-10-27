## Why

透過 AI 代理執行 SDD 的開發者、PO 與 PM，在 Desktop 恢復或切換 remote workspace 時，若 handshake 失敗，目前只看到分頁驚嘆號與 hover 才出現的底層英文錯誤；分頁不會成為作用中目的地，Tray 切換也可能靜默失敗，使使用者無法理解狀態或就地復原。Phase 3 已具備 remote session、離線 stale snapshot、重新認證與 Tray 專案切換，本變更補齊這些能力共同缺少的錯誤復原 UX。

## What Changes

- remote workspace 分頁於重啟恢復或手動切換的 handshake 期間提供立即 loading 回饋；失敗後分頁仍可成為作用中的 navigation destination，主內容顯示可操作的復原頁，而非維持上一個 workspace 且像點擊無效。
- 主視窗以繁體中文呈現 workspace、server 與可理解的錯誤摘要，提供重新連線、前往對應伺服器設定／重新登入、查看技術資訊及自分頁移除；原始 transport 訊息降為漸進揭露，不再由原生 title tooltip 承擔唯一錯誤資訊。
- 正常、連線中、server 不可達、需要重新認證與其他 handshake 錯誤使用一致且可存取的分頁狀態；既有已建立 session 的 offline stale 唯讀內容與自動收斂語意維持不變。
- Tray 與主視窗共用同一份 workspace 復原狀態：macOS 面板 tab 呈現狀態並在作用中錯誤 workspace 下顯示精簡復原卡；原生選單為錯誤 workspace 提供狀態文字與復原 submenu。Tray 直接重試時不喚起主視窗，只有使用者明確選擇詳情、設定或重新登入才將主視窗帶到前景。
- loading、失敗與恢復使用可見文字、圖示及語意化狀態共同表達，支援鍵盤操作與螢幕閱讀器公告，且不依賴 hover 或顏色作為唯一訊號。
- 相容性影響：Desktop Tray 既有「切換失敗不另顯示錯誤」契約改為就地提供精簡復原 UI；正常 local／remote 切換與不奪焦語意不變。CLI 人眼輸出、CLI JSON、Speclink／Spectra 回歸對照、server HTTP API 與既有設定檔格式皆不變。

## Non-Goals

- 不跨 app 重啟持久化 remote 看板或文件 snapshot；尚未成功重建 session 時只呈現復原頁，不偽造 stale 資料。
- 不新增離線寫入佇列、背景重放、衝突合併或新的 retry worker；已建立 session 的 Polling、ETag、SSE 與自動收斂演算法維持既有實作。
- 不重新設計 Workspace Chooser、伺服器登入協定、connection registry、OS Keychain、Tray 生命週期／討論內容區塊或全域通知中心。
- 不變更 speclink-core、speclink-cli、CLI 子指令／旗標／stdin／exit code、.speclink.yaml、openspec/config.yaml、skills 或 CLAUDE.md／AGENTS.md 注入區塊。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workspace-session`: 作用中分頁可處於 restoring／error 而尚無可用 session；navigation、資料操作邊界與移除／重試語意須明確。
- `remote-workspace-data`: 重啟恢復與手動切換的 handshake loading、失敗呈現、重試成功及錯誤資訊層級改為可驗證的復原流程。
- `remote-resilience`: 明確區分「既有 session 離線而保留 stale snapshot」與「重啟後尚未重建 session 的復原頁」，並讓 needs-reauth 復原入口跨主視窗與 Tray 一致。
- `tray-status-menu`: macOS 面板與原生選單改為呈現 remote workspace 復原狀態與操作，取代切換失敗時完全不在 Tray 顯示的既有要求。

## Impact

- Affected specs: workspace-session, remote-workspace-data, remote-resilience, tray-status-menu
- Affected code:
  - New:
    - apps/desktop/src/components/RemoteWorkspaceRecovery.tsx
    - apps/desktop/src/__tests__/remoteWorkspaceRecovery.test.tsx
  - Modified:
    - apps/desktop/src/App.tsx
    - apps/desktop/src/main.tsx
    - apps/desktop/src/session.ts
    - apps/desktop/src/store.ts
    - apps/desktop/src/tray.ts
    - apps/desktop/src/panel/TrayPanel.tsx
    - apps/desktop/src/panel/main.tsx
    - apps/desktop/src/components/ProjectTabs.tsx
    - apps/desktop/src/i18n/messages.ts
    - apps/desktop/src/__tests__/App.test.tsx
    - apps/desktop/src/__tests__/projectTabs.test.tsx
    - apps/desktop/src/__tests__/remoteOpen.test.ts
    - apps/desktop/src/__tests__/remoteResilience.test.tsx
    - apps/desktop/src/__tests__/tray.test.ts
    - apps/desktop/src/__tests__/trayPanel.test.tsx
    - apps/desktop/src-tauri/src/lib.rs
    - apps/desktop/src-tauri/src/remote.rs
    - apps/desktop/src-tauri/tests/remote_data.rs
  - Removed: none
- Affected crates: speclink-core and speclink-cli are explicitly unaffected; Desktop React/Tauri boundaries only.
- Dependencies: no new runtime or development dependency.
