# Windows x64 隨身碟獨立版

這份 ZIP 包含 Speclink 桌面程式與同一份原始碼建置的 CLI。EXE 旁的 `speclink-portable` 標記會啟用隨身碟獨立模式，App 與 CLI 都將自己的設定放在同資料夾的 `data` 中，不使用安裝版的設定或 Windows 登入憑證。使用端不需要 Rust、Node.js 或 npm。

## 下載與啟動

1. 正式桌面發布請從此儲存庫的 **Releases** 下載 `desktop-vX.Y.Z` 下的 `Speclink_<版本>_x64-portable.zip`，直接解壓縮即可。若要測試未發布的分支，在 **Actions → Desktop Build → Run workflow** 選分支並執行，再依以下步驟下載 Artifacts。
2. 等 Windows 工作成功後，從該次 **Summary → Artifacts** 下載 `desktop-x86_64-pc-windows-msvc-portable`。
3. 解開 GitHub 下載的外層 ZIP，再解開裡面的 `Speclink_<版本>_x64-portable.zip`。
4. 將整個資料夾放到自己可讀寫的位置，雙擊 **`speclink-desktop.exe`**。

請保留同資料夾的 `speclink.exe` 與其他檔案；不要只搬走桌面 EXE。`build-info.json` 記錄建置來源及 commit，可用來分辨同版號的不同建置。原本的 `desktop-x86_64-pc-windows-msvc` 仍提供 EXE 安裝器。

## 系統需求與設定

- 使用 Windows x64，建議 Windows 10／11。
- 電腦必須已有 **Microsoft Edge WebView2 Runtime**。本 ZIP 不附 WebView2；缺少時請安裝 [Microsoft 官方 WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)。不能只憑已安裝 Edge 判定 Runtime 存在。[Microsoft 部署說明](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution)
- 本流程未使用 Windows 發行者憑證，首次啟動可能出現未識別發行者提示。
- 整個資料夾必須可寫入。首次啟動建立 `data`，不能寫入時報錯停止，不退回本機目錄。
- 不要移除 `speclink-portable` 標記；沒有標記的 Windows EXE 會使用原本的安裝版模式。macOS／Linux 不受標記影響。
- 不會自動建立捷徑、加入 PATH 或登錄解除安裝項目。可自行為桌面 EXE 建立捷徑。

要使用 CLI，在解壓縮後的資料夾開啟 PowerShell：

```powershell
.\speclink.exe --version
```

## 哪些資料會跟著隨身碟

```text
Speclink_<版本>_x64-portable/
├── speclink-desktop.exe
├── speclink.exe
├── speclink-portable
└── data/                       # 首次使用才建立，不會打進下載包
    ├── desktop/connections.json # 遠端連線清單，不含登入密鑰
    ├── speclink/                # CLI 全域設定、自訂 schema、連線鎖檔
    └── webview/main/            # 介面偏好、分頁／最近清單、WebView 快取
```

換電腦時搬移整個資料夾。專案檔案仍在你選擇的專案目錄；若要一起帶走，請將專案 clone 到隨身碟。最近清單記錄的是完整路徑，磁碟代號改變時請重新開啟專案，設定本身仍會保留。

遠端登入憑證只存在本次 App 行程的記憶體；重開 App 或換電腦時重新登入。完全退出 App 才結束行程，只關閉視窗而留在系統匣並不會清除本次登入。Desktop 與另外啟動的 CLI 不共用這份記憶體登入。

隨身碟版 CLI 不提供會儲存憑證的 `auth login`；需要連線時，使用目前終端機行程的 `SPECLINK_TOKEN`，用完清除。它也不會讀取 `credentials.yaml` 中的舊密鑰。不要把 token 寫入隨身碟的啟動腳本或設定檔。

這個模式隔離的是 Speclink 自己的設定，不保證電腦完全沒有使用痕跡。Windows、外部瀏覽器、Git、Claude／Codex／Copilot 各自的安裝、登入與紀錄仍由它們管理；本 ZIP 不包含這些工具。WebView2 Runtime 仍需由電腦提供。

## 更新與移除

更新時先從系統匣完全退出舊 App，將新的 ZIP 解壓縮到另一個資料夾，再把舊版的整個 `data` 資料夾複製進新版，確認標記仍在後啟動。請先保留舊版與 `data` 備份，確認正常後再移除。

**隨身碟模式會停用內建更新與其操作入口，請使用上述手動更新方式。** 原本的安裝版仍保留既有更新功能。

移除時完全退出 App，再刪除解壓縮資料夾。此操作也會刪掉其中的 `data` 設定；先備份想保留的資料。放在其他位置的專案不受影響。拔除隨身碟前請退出 App 並使用 Windows 安全移除。

## 建置與驗收

Windows 工作先建置 CLI 與 Tauri 桌面程式，再執行 `node scripts/desktop/windows-portable.mjs`，以 PowerShell `Compress-Archive` 產生 ZIP。Windows 建置使用靜態 C runtime，WebView2 仍由系統提供。

四個平台工作完成後，Artifacts 會有五項：macOS 一項、Windows 安裝版與免安裝版各一項、Linux 兩項，保留 14 天。手動執行不會建立 Release；推送 `desktop-vX.Y.Z` 後，四平台成功才會自動將五種附件發布至 Release。操作步驟見[以 Tag 發布桌面版](../development.zh-TW.md#desktop-tags)。

工作流程會解開 ZIP、執行內附 CLI、驗證設定寫入旁邊的 `data`，再搬移資料夾確認設定仍可讀取。發布前仍需在 Windows 實際驗收：

- 雙擊桌面 EXE 能開啟完整介面。
- 能開啟本機專案，Claude／Codex／GitHub Copilot 選項符合專案設定。
- App 能辨識同資料夾內的 CLI，關鍵操作正常。
- 完全退出並重新開啟後，設定可正常保留，遠端連線須重新登入。
- 在電腦 A 修改介面偏好，退出後將整個資料夾搬到電腦 B，偏好與連線清單仍在。
- 安裝版與隨身碟版各自保留設定、登入互不共用；隨身碟版沒有安裝器更新入口。
- 隨身碟唯讀時顯示錯誤，不使用本機 AppData。
