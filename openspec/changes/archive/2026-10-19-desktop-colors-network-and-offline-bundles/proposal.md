## Why

透過 AI 代理跑 SDD 的開發者、PO／PM，在桌面操作專案及公司受限網路環境中，遇到以下六項問題：

1. **色票預覽不易判斷實際選色**：設定頁未選中的色票會變淺，難以直接比較各色票被選取後的深色外觀。
2. **專案圖示的選取狀態不夠清楚**：需要一致的深底白字與淺底黑字，讓主視窗與系統匣中的目前專案更容易辨認，預設色票也遵循同一規則。
3. **設定頁點專案後未必回到專案頁面**：使用者想繼續操作專案，卻仍停在設定模式，切換結果不符合點擊預期。
4. **公司環境的異常缺乏診斷證據**：更新頁內容看不到、標準資料下載只顯示 download_failed，無法分辨畫面、連線、Proxy或憑證問題，需要可記錄及匯出的資訊來查明原因。
5. **公司網路需要明確的 Proxy 設定**：目前不同下載功能的 Proxy 行為不一致，也缺少裝置設定入口，使用者無法依公司提供的連線方式設定更新與標準下載。
6. **離線標準資料缺少匯出途徑**：目前能匯入離線包，卻無法把已安裝資料匯出帶到受限電腦，需要單一版本及全部資料的匯出方式。

## What Changes

- 設定色票預覽固定深底白字、僅外框標示選擇；實際專案圖示選取深底白字、未選取淺底黑字，預設未選取淺灰底。紫色明訂為深紫羅蘭 #9400D3／蘭花紫 #DA70D6，主視窗與系統匣一致。
- 兩種設定模式點左側專案後進該專案看板，包含目前專案；未儲存設定沿用捨棄確認。
- 應用程式設定新增「網路與診斷」分類：預設關閉的 Debug mode、受容量限制且遮蔽敏感資訊的畫面／網路紀錄、診斷匯出，以及沿用系統／手動／直接連線三種 Proxy 模式。涵蓋更新檢查、更新下載與安全標準下載，無專案也可使用。
- 針對更新元件可能隱去 HTTP 狀態的限制，Debug mode 提供使用者明確觸發的「測試更新連線」，以相同 Proxy 設定取得連線結果，不額外自動送出探測。
- 安全標準資料頁增加單一標準版本與全部有效已安裝版本的離線匯出；保留既有單包 ZIP 格式，全部匯出為含單包的集合 ZIP，同一匯入入口支援整批包。預檢全部資料後逐版本提交，途中失敗明示已完成與未完成項目。
- 新增 CLI：security data export [STANDARD --version VERSION | --all] --output PATH [--json]；擴充 security data import PATH [--json] 接受集合包。無 stdin；成功0、操作失敗1、取消130、參數錯誤2；進度與錯誤至 stderr，結果至 stdout，支援 --no-color。

## Capabilities

### New Capabilities

- `desktop-network-diagnostics`: 裝置網路偏好、跨下載器一致的 Proxy 決策、可匯出且有界的診斷紀錄與明確觸發的連線測試。既有 desktop-config 只涵蓋設定分類與政策編輯，desktop-app 只規範更新互動，均沒有網路路由與診斷紀錄契約。

### Modified Capabilities

- `desktop-app`: 調整裝置層專案圖示深淺與色票預覽，補上設定頁點專案回看板的行為。
- `desktop-config`: 應用程式設定增加「網路與診斷」分類，沿用分類搜尋、記憶與零專案可用性。
- `security-standard-data`: 新增離線資料包匯出、集合包匯入及 CLI 契約；保留單包檢查、授權與來源語意。

## Impact

- 相關正式規格掃描：desktop-app、desktop-config、security-standard-data；新增與修改能力如上，沒有進行中變更依賴。
- Affected code:
  - Modified: `packages/ui/src/theme.css`、`apps/desktop/src/components/projectTile.ts`、`apps/desktop/src/views/ProjectSettingsView.tsx`、`apps/desktop/src/App.tsx`、`apps/desktop/src/store.ts`、`apps/desktop/src/settingsCatalog.ts`、`apps/desktop/src/views/AppSettingsView.tsx`、`apps/desktop/src/adapter/updater.ts`、`apps/desktop/src/adapter/security.ts`、`apps/desktop/src/views/settings/SecurityDataPage.tsx`、`apps/desktop/src/i18n/messages.ts`、`apps/desktop/core/src/security.rs`、`apps/desktop/core/src/lib.rs`、`apps/desktop/src-tauri/src/lib.rs`、`apps/desktop/src-tauri/src/security_commands.rs`、`apps/desktop/src-tauri/Cargo.toml`、`apps/desktop/src-tauri/capabilities/default.json`、`apps/desktop/core/Cargo.toml`、`crates/engine/speclink-core/src/security/data.rs`、`crates/engine/speclink-fs/src/security.rs`、`crates/engine/speclink-fs/src/lib.rs`、`crates/engine/speclink-fs/Cargo.toml`、`crates/host/speclink-host/src/security_data.rs`、`crates/host/speclink-host/src/lib.rs`、`crates/host/speclink-host/Cargo.toml`、`crates/adapters/speclink-cli/src/verbs/security.rs`、`Cargo.lock`；受影響面的既有測試與 `docs/configuration.zh-TW.md`、`docs/configuration.md`、`docs/product-status.zh-TW.md`、`docs/product-status.md` 同步更新。
- 相容性影響：既有單包 import、人眼與 JSON 輸出及其他 CLI 指令保持原契約；只有集合包 import 使用新的逐項結果，部分成功 exit1 並回傳明示 partial 的結果，不冒稱全成功。新增 export 是新增入口，舊單包無須轉換；舊版無法匯入集合包時，仍可由使用者在解壓工具取出內含的單包分別匯入。
- 裝置偏好新增 formatVersion=1、debugEnabled=false、proxy.mode=system、proxy.url=null，存於應用程式設定目錄的 runtime-preferences.json；portable 使用既有可攜資料根的 desktop 目錄。診斷存於同一裝置根的 diagnostics 目錄。不變更 openspec/config.yaml、.speclink.yaml 或生成的 claude／codex 技能。
- 依賴：共用既有 ureq、ZIP 與 Tauri updater；明確引入與目前 updater 同版的 reqwest 0.13，以及既有依賴樹中的 hyper-util 0.1 系統 Proxy matcher，開啟所需平台功能。FS將既有測試使用的tempfile列為執行期依賴以提供無覆寫暫存發佈；桌面core明列ZIP／SHA依賴。更新仍保留 Tauri 簽章驗證與原安裝流程。

## Planned New Files

以下為本變更預計建立的新檔，實作前尚不存在：

- `apps/desktop/core/src/runtime_preferences.rs`、`apps/desktop/core/src/diagnostics.rs`、`apps/desktop/core/src/network.rs`、`apps/desktop/src-tauri/src/runtime_commands.rs`、`apps/desktop/src-tauri/src/updater_commands.rs`、`apps/desktop/src/adapter/runtime.ts`、`apps/desktop/src/diagnostics.tsx`、`apps/desktop/src/views/settings/NetworkDiagnosticsPage.tsx`、`crates/host/speclink-host/src/network.rs`、`crates/engine/speclink-fs/src/security_packages.rs`。
