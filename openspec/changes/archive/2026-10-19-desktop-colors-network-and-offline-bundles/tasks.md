## 1. 固定色票與專案導覽

- [x] 1.1 Red：在 `apps/desktop/src/__tests__/projectColors.test.ts`、`apps/desktop/src/__tests__/projectSettingsView.test.tsx`、`apps/desktop/src/__tests__/trayPanel.test.tsx` 與 `packages/ui/src/__tests__/theme.test.ts` 補上「裝置層專案圖示底色」的七色深淺、預覽固定深色與主題一致案例；先執行 npm test -w @speclink/desktop 及 npm test -w @speclink/ui，確認因舊配色失敗。 <!-- speclink-task:tsk_01M59WZ8NF5VKM9AY7EK3GXXR0 -->
- [x] 1.2 Green／Refactor：依「固定深淺色票」設計修改 `packages/ui/src/theme.css`、`apps/desktop/src/components/projectTile.ts`、`apps/desktop/src/views/ProjectSettingsView.tsx` 與 `apps/desktop/src/i18n/messages.ts`，主視窗／系統匣共用深淺與白／黑字，預設不再用主色；驗證1.1測試轉綠及「專案分頁列存於 app 本機」的儲存、reader與失敗保值案例仍通過。 <!-- speclink-task:tsk_01M59WZ8NFGA3ZSBKNJBDP6ZHN -->
- [x] 1.3 Red：在 `apps/desktop/src/__tests__/App.test.tsx`、`apps/desktop/src/__tests__/store.test.ts` 加入「設定頁點專案返回看板」兩種設定模式、同專案／異專案、草稿取消／確認、鍵盤切換及失敗留原頁案例；npm test -w @speclink/desktop 確認目前切換不離開設定的失敗。 <!-- speclink-task:tsk_01M59WZ8NFCCJEGDRHGS13FWE7 -->
- [x] 1.4 Green／Refactor：於 `apps/desktop/src/App.tsx` 統一離開設定前的意圖與捨棄確認，於 `apps/desktop/src/store.ts` 的 activateTab／enterProject／Remote啟用成功提交，在由設定模式發起時同次切看板，過期意圖不作用；驗證1.3及既有返回／詳情互斥測試以 npm test -w @speclink/desktop 通過。 <!-- speclink-task:tsk_01M59WZ8NFAJ119XBN5H6R4DQH -->

## 2. 裝置偏好與診斷記錄器

- [x] 2.1 Red：在 `apps/desktop/core/src/runtime_preferences.rs` 的cfg(test)加入「裝置網路與診斷偏好」缺檔預設、camelCase結構、未知版本、寫入故障、portable路徑與明確重設案例；cargo test -p speclink-desktop-core 確認缺少契約而失敗，不以編譯錯誤代替行為斷言。 <!-- speclink-task:tsk_01M59WZ8NGWPDGRQSMFR26SFTP -->
- [x] 2.2 Green／Refactor：實作 `apps/desktop/core/src/runtime_preferences.rs`，在 `apps/desktop/core/src/lib.rs` 註冊；原子寫入後才發布、壞檔阻止新受管工作、失敗保舊，無專案依賴；以cargo test -p speclink-desktop-core驗證2.1轉綠。 <!-- speclink-task:tsk_01M59WZ8NGXB58Z0YX9SQJETS2 -->
- [x] 2.3 Red：於 `apps/desktop/core/src/diagnostics.rs` 的cfg(test)補上「有界且遮蔽敏感資訊的診斷紀錄」開關、4×2MiB輪替、16KiB單筆、256佇列、writeFailed及匯出秘密sentinel檢查；覆蓋「診斷匯出與明確觸發的更新連線測試」中的匯出／取消／損壞行為，以cargo test -p speclink-desktop-core確認尚未滿足契約。 <!-- speclink-task:tsk_01M59WZ8NGNH97RMJ3FRBGRJJN -->
- [x] 2.4 Green／Refactor：實作 `apps/desktop/core/src/diagnostics.rs` 的固定schema、分類與有界worker、二次過濾及9MiB ZIP匯出，於 `apps/desktop/core/Cargo.toml` 與 `apps/desktop/core/src/lib.rs` 接既有ZIP／SHA工具；以cargo test -p speclink-desktop-core確認匯出不含token、密碼、query、完整路徑與專案內容，關閉後不接收事件。 <!-- speclink-task:tsk_01M59WZ8NGW3Z14S0A6K7QK6KF -->

## 3. 共用 Proxy 決策與標準下載

- [x] 3.1 Red：在 `crates/host/speclink-host/src/network.rs` 及 `apps/desktop/core/src/network.rs` 的cfg(test)定義「受管下載的一致 Proxy 模式」測試：manual合法／不合法URL、direct略過環境、system各平台固定來源與NO_PROXY、unsupported協定／認證、工作設定快照；cargo test -p speclink-host及cargo test -p speclink-desktop-core確認契約未滿足。 <!-- speclink-task:tsk_01M59WZ8NG7ANRK2YWG4505HGQ -->
- [x] 3.2 Green／Refactor：依「一致的 Proxy 決策」實作 `crates/host/speclink-host/src/network.rs` 與 `apps/desktop/core/src/network.rs`，在兩crate的 `src/lib.rs` 及 `Cargo.toml` 接入已核對hyper-util系統matcher與camelCase DTO；不改程序環境、不擴Remote路由，支援範圍與未取得固定路由結果可見；以上兩個Cargo package測試轉綠。 <!-- speclink-task:tsk_01M59WZ8NG1R4JXFQE6F6KY5R1 -->
- [x] 3.3 Red：在 `crates/host/speclink-host/src/security_data.rs` 與 `apps/desktop/core/src/security.rs` 的cfg(test)用Proxy trap加入「Proxy 與來源驗證分離」：私有Proxy可連、官方來源／redirect限制仍在、失敗不fallback、DNS取消、TLS錯誤與紀錄；以cargo test -p speclink-host及cargo test -p speclink-desktop-core確認失敗原因是舊transport未接Proxy／底層錯誤被抹除。 <!-- speclink-task:tsk_01M59WZ8NGN8XVMCZXDGWM1ZDT -->
- [x] 3.4 Green／Refactor：於 `crates/host/speclink-host/src/security_data.rs` 注入路由與結構化診斷，僅允許設定端點使用Proxy resolver；`apps/desktop/core/src/security.rs` 在工作開始取偏好快照，獨立CLI保留既有預設；以兩個Cargo package測試驗證來源驗證、限制、取消與失敗保舊未退化。 <!-- speclink-task:tsk_01M59WZ8NG78XMMZE9JPTMHNP7 -->

## 4. 更新橋接及連線測試

- [x] 4.1 Red：在 `apps/desktop/src/__tests__/updater.test.ts` 與 `apps/desktop/src-tauri/tests/it/` 的既有Tauri測試框架加入候選更新保存Proxy快照、direct不走環境、下載取消不安裝、plugin錯誤狀態為null案例；以npm test -w @speclink/desktop與cargo test -p speclink-desktop確認新橋接缺失。 <!-- speclink-task:tsk_01M59WZ8NG4QR280TCKRHN9NTY -->
- [x] 4.2 Green／Refactor：依「更新橋接與連線測試」於 `apps/desktop/src-tauri/src/updater_commands.rs` 實作薄plugin adapter並註冊 `apps/desktop/src-tauri/src/lib.rs`，修改 `apps/desktop/src/adapter/updater.ts` 保持UpdaterAdapter介面；在 `apps/desktop/src-tauri/Cargo.toml` pin updater及明確reqwest版本、同步 `Cargo.lock`，保留驗簽與安裝；以4.1命令驗證，plugin未公開的HTTP碼不得猜測。 <!-- speclink-task:tsk_01M59WZ8NGQ40XNDRS805901B5 -->
- [x] 4.3 Red：在 `apps/desktop/core/src/network.rs` 的cfg(test)及 `apps/desktop/src/__tests__/appSettingsView.test.tsx` 補上使用者觸發的更新連線測試、407、TLS、20秒／2MiB／三跳上限與Debug關閉不發請求；以cargo test -p speclink-desktop-core及npm test -w @speclink/desktop確認失敗。 <!-- speclink-task:tsk_01M59WZ8NGTT2VNXQH0YFYKCB9 -->
- [x] 4.4 Green／Refactor：在 `apps/desktop/core/src/network.rs` 實作僅內建更新端點的有界GET探測、在 `apps/desktop/src-tauri/src/runtime_commands.rs` 接出network_probe_update，記area=network-probe並保持原更新狀態；以4.3命令及HTTP trap驗證HTTP碼／null、獨立operationId與零自動額外請求。 <!-- speclink-task:tsk_01M59WZ8NGMNN4MT0GGZ0GMA67 -->

## 5. 設定頁與畫面錯誤接線

- [x] 5.1 Red：在 `apps/desktop/src/__tests__/settingsColumn.test.tsx`、`apps/desktop/src/__tests__/appSettingsView.test.tsx`、`apps/desktop/src/__tests__/App.test.tsx` 補上「設定頁圖形化讀寫兩層設定」「側欄導覽結構」新增分類搜尋與零專案流程，並測畫面render錯誤／unhandledrejection、重試出口及開關停錄；npm test -w @speclink/desktop確認新行為尚未實現。 <!-- speclink-task:tsk_01M59WZ8NGCCVWVQ7D0GPYSCZS -->
- [x] 5.2 Green／Refactor：依「裝置偏好與設定分類」修改 `apps/desktop/src/settingsCatalog.ts`、`apps/desktop/src/views/AppSettingsView.tsx`、`apps/desktop/src/store.ts`、`apps/desktop/src/i18n/messages.ts`，新增 `apps/desktop/src/views/settings/NetworkDiagnosticsPage.tsx` 及 `apps/desktop/src/adapter/runtime.ts`，在 `apps/desktop/src-tauri/src/runtime_commands.rs`／`apps/desktop/src-tauri/src/lib.rs` 委派core；沿用共用設定元件，加入必要的dialog save權限至 `apps/desktop/src-tauri/capabilities/default.json`；以npm test -w @speclink/desktop及cargo test -p speclink-desktop驗證IPC型別、搜尋、儲存失敗、匯出與Probe結果。 <!-- speclink-task:tsk_01M59WZ8NGHCZWKQX4Y1DJK4FC -->
- [x] 5.3 Green／Refactor：依「有界診斷紀錄與匯出」新增 `apps/desktop/src/diagnostics.tsx` 並由 `apps/desktop/src/App.tsx` 單次掛接ErrorBoundary／window監聽；不傳任意console、錯誤message或專案內容，更新頁呈現失敗有重試／診斷出口；用5.1已先失敗的測試及npm test -w @speclink/desktop驗證只有Debug啟用時記錄、卸載不重複監聽與秘密sentinel不送IPC。 <!-- speclink-task:tsk_01M59WZ8NG8CNX038Y4N0N1S86 -->

## 6. 單包匯出與集合格式

- [x] 6.1 Red：於 `crates/engine/speclink-core/src/security/data.rs`、`crates/engine/speclink-fs/src/security.rs` 的cfg(test)加入「已安裝標準資料的離線匯出」所需的同代raw讀取、來源／授權保留、CWE XML重匯入與ExportReceipt camelCase型別；cargo test -p speclink-core及cargo test -p speclink-fs確認缺少匯出能力。 <!-- speclink-task:tsk_01M59WZ8NGYZ400FZGGMRVJEK2 -->
- [x] 6.2 Green／Refactor：依「單包匯出與集合包匯入」於 `crates/engine/speclink-core/src/security/data.rs` 增DTO／repository匯出來源讀取，於 `crates/engine/speclink-fs/src/security.rs` 共用generation pin與指紋檢查，新增 `crates/engine/speclink-fs/src/security_packages.rs` 生成v1包；在 `crates/engine/speclink-fs/src/lib.rs` 註冊並於 `crates/engine/speclink-fs/Cargo.toml` 接tempfile，驗證6.1轉綠，來源／指紋一致、imported不升級信任。 <!-- speclink-task:tsk_01M59WZ8NGPD8S4HW32Y6AE0TW -->
- [x] 6.3 Red：在 `crates/engine/speclink-fs/src/security_packages.rs` cfg(test)建立集合包惡意fixture與邊界，涵蓋「受限 ZIP 與安全 XML 匯入」及「離線集合包匯入與逐版本結果」的32包／512MiB／1GiB限制、重複／遞迴／Zip Slip／未列entry／SHA／授權失敗；cargo test -p speclink-fs確認缺失行為失敗。 <!-- speclink-task:tsk_01M59WZ8NGS5KCTSYPTSVHPS3Y -->
- [x] 6.4 Green／Refactor：實作 `crates/engine/speclink-fs/src/security_packages.rs` 的v2集合codec、串流限額及受控staging，單包沿原嚴格validator，不讓集合路徑放寬v1；以cargo test -p speclink-fs驗證6.3及原CWE XML／ZIP測試通過，預檢失敗不改current。 <!-- speclink-task:tsk_01M59WZ8NG4KQA5XENGFQVYS9Z -->

## 7. Host資料交換與部分結果

- [x] 7.1 Red：在 `crates/host/speclink-host/src/security_data.rs` cfg(test)補上清單凍結、invalid跳過、空清單、已選資料讀取失敗、目的檔已存在／取消／寫入故障，以及全部預檢後逐版本提交的第二包失敗；cargo test -p speclink-host確認缺少export／batch契約。 <!-- speclink-task:tsk_01M59WZ8NGV3STXGMPBF78KPE0 -->
- [x] 7.2 Green／Refactor：於 `crates/host/speclink-host/src/security_data.rs` 實作共用export及batch import裁決、明示partial／failed／cancelled／notAttempted，與 `crates/engine/speclink-fs/src/security_packages.rs` 的無覆寫原子發佈組合；以cargo test -p speclink-host及cargo test -p speclink-fs驗證已選讀取失敗整次export不發佈、預檢零提交、部分成功保留及其餘版本保舊。 <!-- speclink-task:tsk_01M59WZ8NGNZ0ZB5MC93ERHMR3 -->

## 8. CLI及桌面資料管理入口

- [x] 8.1 Red：在 `crates/adapters/speclink-cli/tests/it/security.rs` 為「資料管理 CLI 輸入輸出」「離線資料包CLI匯出與集合匯入」加入單包／全部export、人眼／JSON／--no-color、互斥參數、既有目的檔、v2 partial與取消exit code；斷言完整camelCase欄位、型別及stdout／stderr分流，以cargo test -p speclink-cli確認新入口測試失敗並保留v1回歸對照。 <!-- speclink-task:tsk_01M59WZ8NGKSZ5MHHTJBXQTHT4 -->
- [x] 8.2 Green／Refactor：在 `crates/adapters/speclink-cli/src/verbs/security.rs` 加export與v2 import呈現，只經Host，不在adapter組ZIP；以cargo test -p speclink-cli及cargo test -p speclink-core --test it render_golden::驗證8.1、既有單包人眼／JSON位元級輸出與無關CLI golden不變。 <!-- speclink-task:tsk_01M59WZ8NGXA9CPAZNP2QYYMQT -->
- [x] 8.3 Red：在 `apps/desktop/src/__tests__/securityDataPage.test.tsx` 與 `apps/desktop/core/src/security.rs` cfg(test)加入單版本／全部匯出選檔、取消、進度、集合import逐項partial、Remote離線仍可用與目的檔保留；以npm test -w @speclink/desktop及cargo test -p speclink-desktop-core確認入口契約未滿足。 <!-- speclink-task:tsk_01M59WZ8NGAWGR220HBVTRYATH -->
- [x] 8.4 Green／Refactor：於 `apps/desktop/src/views/settings/SecurityDataPage.tsx`、`apps/desktop/src/adapter/security.ts`、`apps/desktop/core/src/security.rs` 及 `apps/desktop/src-tauri/src/security_commands.rs` 接出Host匯出／集合匯入，保留application-data工作與取消，不將partial當單一成功；更新 `apps/desktop/src/i18n/messages.ts` 中英對照。以8.3命令及cargo test -p speclink-desktop驗證DataRequest／receipt型別、零專案與Remote認證失效不發專案請求。 <!-- speclink-task:tsk_01M59WZ8NG8JGK4R5QX01BRM4C -->

## 9. 同類位置、安全邊界與文件

- [x] 9.1 排查同類寫法並套用audit discipline：以rg搜尋projectTileClass、50%、activateTab、try_proxy_from_env、download_failed與updater check呼叫；核對 `apps/desktop/src/components/ProjectRail.tsx`、`apps/desktop/src/panel/TrayPanel.tsx`、`apps/desktop/src/views/ProjectSettingsView.tsx`、`apps/desktop/src/App.tsx`、`apps/desktop/src/store.ts`、`apps/desktop/src/adapter/updater.ts`、`crates/host/speclink-host/src/security_data.rs` 皆使用共用行為，`apps/desktop/src-tauri/src/remote.rs` 與 `crates/protocol/speclink-remote/` 的Remote路由明列不修原因。依speclink instructions --skill audit逐一檢查Proxy URL注入／TLS保護／診斷秘密／ZIP及輸出原子性，將實際結果寫入本變更design的驗證記錄；新修正各先補失敗測試，再以對應Cargo package或workspace測試轉綠。 <!-- speclink-task:tsk_01M59WZ8NGV62MZJZSM8MBSDYV -->
- [x] 9.2 在 `docs/configuration.zh-TW.md`、`docs/configuration.md`、`docs/product-status.zh-TW.md`、`docs/product-status.md` 說明裝置偏好、Debug匯出、固定Proxy支援限制與集合包部分成功；核對中英敘述與四份delta規格一致、無關CLI輸出不變，執行node --test scripts/*.test.mjs scripts/*/*.test.mjs完成詞彙、連結與中英對照檢查。 <!-- speclink-task:tsk_01M59WZ8NGSDTYDRJVJM6P9HNS -->
