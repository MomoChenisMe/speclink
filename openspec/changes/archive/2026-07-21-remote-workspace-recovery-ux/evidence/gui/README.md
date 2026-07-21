# remote-workspace-recovery-ux：macOS GUI 全鏈驗收

驗收日期：2026-07-21（Asia/Taipei）

## 驗收環境與方法

- App：由目前工作樹建立的 `target/release/bundle/macos/Speclink.app`。
- Remote workspace：`Demo 專案/backend`；連線 `gui-verify`（`http://127.0.0.1:8787`）。
- Local workspace：`speclink`、`speclink-test-b`。
- 以真實 macOS System Events／CoreGraphics 直接點擊 tab、按鈕、系統匣圖示與原生選單；截圖只作結果留存，不替代操作斷言。
- Panel retry 前後前景程序皆為 `ChatGPT`；Panel 未成為 key window。點擊「在 Speclink 中查看問題」或「伺服器設定」後，另以 System Events 確認前景程序為 `speclink-desktop`。
- 原生 fallback 透過一次性的 `SPECLINK_GUI_FORCE_TRAY_FALLBACK` 故障注入驗收 Panel 建立失敗路徑；完成後已移除注入點，並重新建立無注入的正式 App bundle。

## 直接操作與畫面證據

| 步驟 | 直接操作與斷言 | 畫面 |
| --- | --- | --- |
| 1 | 在 server down 狀態啟動已儲存 remote tab；warning tab 可選取，主區域只顯示 recovery destination，沒有上一個 workspace 的資料，技術細節預設收合。 | [01-saved-tab-server-down.png](01-saved-tab-server-down.png) |
| 2 | Light mode 以鍵盤 Tab 巡覽，`重新連線` 有清楚 focus ring。 | [02-light-keyboard-focus.png](02-light-keyboard-focus.png) |
| 3 | Dark mode 重複鍵盤巡覽，焦點與狀態不只靠顏色；驗收後已還原 Light mode。 | [03-dark-keyboard-focus.png](03-dark-keyboard-focus.png) |
| 4 | 啟動 server 後在原分頁按 `重新連線`；401 收斂為 needs-reauth，不新增分頁、不退回 local。 | [04-server-up-needs-reauth.png](04-server-up-needs-reauth.png) |
| 5 | 明確按重新認證／伺服器設定後才聚焦主視窗，並聚焦對應連線。 | [05-reauth-focused-main.png](05-reauth-focused-main.png) |
| 6 | 完成真實 device authorization 後，先驗證 actor 無 membership 時同分頁顯示 access denied，沒有偽裝成功。 | [06-reauth-account-access-denied.png](06-reauth-account-access-denied.png) |
| 7 | 補齊驗收 membership 後重試；同一個 `Demo 專案/backend` tab 原地 ready，頁籤列沒有重複 remote tab。 | [07-reauth-same-tab-ready.png](07-reauth-same-tab-ready.png) |
| 8 | server down 時直接點系統匣；Panel 顯示 compact recovery card，無 session 時不顯示舊討論／變更；前景仍為 ChatGPT。 | [08-tray-panel-no-session-no-focus.png](08-tray-panel-no-session-no-focus.png) |
| 9 | Panel 直接按 `重新連線`；失敗仍留在 recovery card，前景程序未改變。 | [10-tray-retry-no-focus.png](10-tray-retry-no-focus.png) |
| 10 | Panel 明確按 `在 Speclink 中查看問題`；主視窗被顯示並聚焦 recovery destination。 | [11-tray-details-focus-main.png](11-tray-details-focus-main.png) |
| 11 | Panel 明確按 `伺服器設定`；主視窗被顯示並聚焦對應連線。 | [12-tray-settings-focus-main.png](12-tray-settings-focus-main.png) |
| 12 | server 恢復後從 Panel 直接 retry；同一 remote tab 原地 ready，Panel 未奪焦。 | [13-tray-retry-same-tab-ready-no-focus.png](13-tray-retry-same-tab-ready-no-focus.png) |
| 13 | 強制 Panel 建立失敗後再次點系統匣；顯示真正的 macOS 原生 fallback 選單，remote error 是可展開項目，前景仍為 ChatGPT。 | [09-tray-native-fallback.png](09-tray-native-fallback.png) |
| 14 | 以游標直接展開 recovery submenu；作用中／錯誤摘要為 disabled informational rows，retry／詳情／設定可操作。 | [14-tray-native-fallback-submenu.png](14-tray-native-fallback-submenu.png) |
| 15 | 在原生 submenu 直接按 `重新連線`；server down 後重新展開仍是 error，前景在 retry 前、後與重開選單後皆為 ChatGPT。 | [15-tray-native-retry-no-focus.png](15-tray-native-retry-no-focus.png) |
| 16 | 在原生 submenu 明確按 `在 Speclink 中查看問題`；System Events 確認 `speclink-desktop` 成為前景，主視窗顯示同一 recovery destination。 | [16-tray-native-details-focus-main.png](16-tray-native-details-focus-main.png) |
| 17 | 以 macOS AX 直接選取 local `speclink` tab；既有看板可正常使用，remote error 不阻斷 local workspace。 | [17-local-tab-remains-usable.png](17-local-tab-remains-usable.png) |
| 18 | 再直接選取 warning remote tab；同一 tab 重新成為 selected recovery destination。 | [18-remote-error-tab-reselectable.png](18-remote-error-tab-reselectable.png) |
| 19 | 在已移除故障注入的最終 App bundle 開啟 Panel，再直接點擊 Panel 外；前景在開啟前、開啟後、點外後皆為 ChatGPT。CGWindowList 由 `320×418, layer 4` 的 Panel 變成無 layer 4 Panel，證實失焦自動收合。 | [19-panel-dismiss-on-focus-loss.png](19-panel-dismiss-on-focus-loss.png) |

## 驗收結論

- server down → error → retry → server up → 同 tab ready：通過。
- 401 → needs-reauth → 明確認證 → 同 tab 復活：通過；403 membership 邊界亦 fail-closed。
- warning tab 可選、無 session 不洩漏舊資料、local workspace 不回歸：通過。
- Panel 為 non-key recovery surface；直接 retry 不顯示／聚焦主視窗，詳情／設定才聚焦：通過。
- Panel 點外失焦自動收合：通過（最終正常版 App bundle；一次性 fallback 程序已在此驗收前結束）。
- 原生 fallback menu／submenu 與相同 focus 邊界：通過。
