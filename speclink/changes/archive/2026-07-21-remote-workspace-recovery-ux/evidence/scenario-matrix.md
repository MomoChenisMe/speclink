# remote-workspace-recovery-ux Scenario 覆蓋矩陣

核對日期：2026-07-21。四份 delta spec 共 43 個 Scenario；下表逐項列出自動測試或 [macOS GUI 全鏈證據](gui/README.md)。

縮寫：

- `RO`：`apps/desktop/src/__tests__/remoteOpen.test.ts`
- `RR`：`apps/desktop/src/__tests__/remoteResilience.test.tsx`
- `RWR`：`apps/desktop/src/__tests__/remoteWorkspaceRecovery.test.tsx`
- `PT`：`apps/desktop/src/__tests__/projectTabs.test.tsx`
- `TR`：`apps/desktop/src/__tests__/tray.test.ts`
- `TP`：`apps/desktop/src/__tests__/trayPanel.test.tsx`
- `RD-Rust`：`apps/desktop/src-tauri/tests/remote_data.rs`
- `TM-Rust`：`apps/desktop/src-tauri/tests/tray_menu.rs`
- `GUI NN`：`evidence/gui/README.md` 對應步驟與同號畫面

## remote-resilience

| Scenario | 覆蓋 |
| --- | --- |
| 撤銷 device family 後原地復活 | `RR`「needs-reauth 導向聚焦登入，成功後依序 re-handshake…」；GUI 04–07 |
| Tray 顯示 needs-reauth 但不自動奪焦 | `TR`「原生 needs-reauth submenu 的顯式詳情與登入才聚焦主視窗」；`TP` active restoring／offline／needs-reauth 投影；`TM-Rust` focus policy |
| 從 Tray 明確選擇重新登入 | `TR`「面板 recovery action…詳情／設定／登入才顯示並聚焦主視窗」及原生 needs-reauth 測試；GUI 05 |
| 已建立 session 離線保留最後內容 | `RR`「保留清單、呈現 stale/cloud-off…」；`TP`「offline session 則保留 stale 內容」 |
| 重啟後 handshake 失敗不偽造 stale | `RR`「啟動時 handshake 失敗…不洩漏上一個 workspace 資料」；`TP` active no-session recovery card；GUI 01、08 |
| server 恢復後兩條路徑各自收斂 | `RR`「online 事件會自動全量重查…並清除 stale」；`RO`「retry succeeds in place…」；GUI 07、13 |

## remote-workspace-data

| Scenario | 覆蓋 |
| --- | --- |
| 新開啟入口 handshake 失敗不建分頁 | `RO`「failure…leaves no tab or session behind」及「handshake 失敗時保留原 local 分頁與 session」；`RD-Rust` `open_fails_closed_on_403_404_and_ambiguity` |
| 重啟後 remote 分頁恢復成功 | `RO`「activating a restored remote tab re-runs the handshake and adopts the session」及 checkoutRoot restore 測試 |
| 重啟後 credential 失效進入復原頁 | `RO`「a failed re-handshake keeps the remote tab selected as an error destination」；`RWR` needs-reauth primary action；GUI 04 |
| server 不可達時 retry 原地恢復 | `RO`「retry succeeds in place without adding a duplicate tab」；GUI 01、07、13 |
| transport failure 分類為 unreachable | `RO` structured failure table（`status: null` → `unreachable`）；`RWR` localized recovery destination；GUI 01 |
| HTTP status 對應復原分類 | `RO` structured failure table（401／403／404）；GUI 04、06 |
| 無法解析的 rejection 安全降階 | `RO`「fails safely for legacy strings and unknown objects」；`RWR` technical detail progressive disclosure |
| 失敗 payload 不洩漏 credential | `RD-Rust` `remote_open_failure_serializes_machine_readable_fields_without_credentials`；`TR` snapshot 不攜帶 technical detail |

## tray-status-menu

### 選單專案切換

| Scenario | 覆蓋 |
| --- | --- |
| 點選非作用中專案完成切換且不奪焦 | `TR` 參數化「點非作用中 local 專案以 locator key 呼叫 activateTab」且不走 openProjectAt |
| 點選 remote 專案分頁完成切換 | `TR` 同一參數化測試的 remote case |
| remote 切換失敗轉為復原 submenu | `TR`「共用狀態投影讓 restoring／error／needs-reauth 成為原生復原項」；GUI 09、14 |
| 原生選單直接 retry 不奪焦 | `TR`「原生 error submenu 直接 retry 不顯示或聚焦主視窗」；`TM-Rust` focus policy；GUI 15 |
| 原生選單顯式詳情動作取得焦點 | `TR` 原生 needs-reauth 顯式動作測試；`TM-Rust` focus policy；GUI 16 |

### 面板樣式（macOS）

| Scenario | 覆蓋 |
| --- | --- |
| 面板樣式下點擊圖示彈出貼齊面板 | `TR`「panel 樣式…點擊圖示觸發 onPanelToggle」；GUI 08、19 |
| 右鍵點擊圖示與左鍵等價 | `TR`「panel 樣式：右鍵點擊圖示同樣觸發 onPanelToggle」 |
| ready workspace 的區塊順序與分割線 | `TP`「區塊順序…」及「分割線恰三條…」 |
| 點擊專案 tab 原地切換 | `TP`「專案區列出分頁且作用中標示、點非作用中專案回呼切換」 |
| 點擊 remote 專案 tab 原地切換 | `TP`「點 remote 專案 tab 以 locator key 回呼切換」；`TR` panel open-project locator-key 接線 |
| remote handshake 失敗顯示復原卡 | `TP`「active no-session error 以精簡復原卡取代舊資料」；GUI 08 |
| 面板 retry 原地恢復 | `TP` active unreachable retry；`TR` panel recovery action 接線；GUI 10、13 |
| 面板顯式開啟詳情或重新登入 | `TR` panel recovery action focus 測試；`TM-Rust` focus policy；GUI 11、12 |
| 已建立 session 離線保留 Panel stale 內容 | `TP`「offline session 則保留 stale 內容」；`RR` established-session stale 測試 |
| tab 條尾端快速加入專案 | `TP`「tab 條尾端有『加入專案』動作項」；`TR` add-project 先喚起主視窗再開資料夾流程 |
| 分區標題顯示項目計數 | `TP`「各分區標題帶項目計數徽章」 |
| 全無變更時三個生命週期分區常駐 | `TP` 同名測試 |
| 部分有資料時空階段分區仍常駐 | `TP` 同名測試 |
| 進度條依階段深淺 | `TP`「進度條填色依階段套用共用色階」 |
| 開啟面板無預設焦點 | `TP`「複製鈕退出 tab 順序」；GUI 08、19（前景保持 ChatGPT、無預設焦點框） |
| 面板不搶焦點且失焦自動收合 | GUI 19：開啟前／後／點外後前景皆為 ChatGPT，`320×418 layer 4` Panel 點外後自 on-screen window list 消失 |
| 面板內以常駐複製鈕複製 | `TP` 變更／討論複製回呼與 1.2 秒勾號回饋測試 |
| 面板高度自適應內容 | GUI 08、13、19 的 recovery／ready 真視窗；`TP` 分區溢出展開／收合測試（實際入口由 `panel/main.tsx` 的 ResizeObserver 封頂 640） |
| 面板建立失敗退回原生選單 | `store.test.ts` `panelFallback`；`TR` 樣式即時切回 native；GUI 09、14–16（一次性故障注入後已移除） |

## workspace-session

| Scenario | 覆蓋 |
| --- | --- |
| handshake 失敗仍選取該分頁 | `RO` failed re-handshake 與同步 restoring selection；`RR` startup failure；`PT` error tab selectable；GUI 01、18 |
| retry 成功原地建立 session | `RO`「retry succeeds in place without adding a duplicate tab」；GUI 07、13 |
| 較舊 handshake 不搶回作用中分頁 | `RO`「an older handshake result updates its own tab without stealing activeKey」 |
| 同分頁只接受最新 retry 結果 | `RO`「only accepts the latest retry generation for the same tab」 |
| local 分頁切換維持既有行為 | `RR`「同一個 remote 狀態事件不改變本地分頁」；既有 session/store 完整回歸；GUI 17、18 |

## 核對結果

- 43／43 Scenario 均有自動測試或直接 GUI 證據。
- `speclink analyze remote-workspace-recovery-ux --json`：Coverage／Consistency／Gaps 均 Clean；Critical 0、Warning 0（40 筆均為非阻塞 Suggestion）。
- `speclink validate remote-workspace-recovery-ux`：通過。
