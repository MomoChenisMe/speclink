## MODIFIED Requirements

### Requirement: Release 說明含下載指南

release job SHALL 以腳本產生雙語的 Release 說明：開頭為英文下載指南，其後接該版的英文更新日誌片段；再其後為一個預設收合的 `<details>` 區塊，`<summary>` 為「繁體中文」，區塊內依序為繁中下載指南與繁中更新日誌片段；GitHub 自動產生的比較連結接續於最後。兩種語言的片段 SHALL 由 release-notes 的渲染腳本自 release-notes.json 畫出。每種語言的指南 SHALL 含：一般使用者安裝檔對照表（macOS 一列指向 universal dmg 並註明 Apple Silicon 與 Intel 同一檔、Windows NSIS 安裝器一列、Linux 桌面機依架構各一 AppImage、Linux 伺服器或無圖形介面一列指向 CLI 一行安裝而非任何檔案；檔名含該版號且與資產命名一致，兩種語言的檔名 SHALL 逐字相同）、CLI 的三條一行指令（npm 全域安裝、安裝腳本 curl 一行、Homebrew）、server 的一行啟動節（npx 指令與 Docker image 一行，寫法比照 CLI 安裝節），並 SHALL 註明更新包（.app.tar.gz）與 latest.json 為自動更新機制使用、毋須手動下載。指南 SHALL NOT 提及 .sig、.deb、SHA256SUMS.txt 或 CLI 壓縮檔。產生器 SHALL 於 tag 不符 vX.Y.Z 格式時以非零結束，SHALL 於 release-notes.json 沒有該版條目時以非零結束且 stdout 保持空白，且 SHALL 有驗證兩種語言輸出內容的自動化測試。

#### Scenario: 發布後 Release 頁開頭為下載指南

- **WHEN** push tag 完成發布後檢視該 GitHub Release 頁
- **THEN** 說明開頭為英文的四列安裝檔對照表、三條 CLI 安裝指令與 server 一行啟動節（npx 與 Docker），.app.tar.gz 與 latest.json 標註為自動更新機制使用，其後為英文更新日誌片段（`## X.Y.Z (YYYY-MM-DD)` 與分組條目）；再其後為收合的「繁體中文」區塊，展開後為繁中指南與繁中片段；自動產生的比較連結位於最後

#### Scenario: 指南檔名對齊版號與資產命名

- **WHEN** 以 tag vX.Y.Z 執行下載指南產生器
- **THEN** 英文與繁中兩段輸出中的每個檔名都含該版號、彼此逐字相同，且與 release 管線對應平台的資產命名一致

#### Scenario: JSON 缺該版時產生器失敗

- **WHEN** 以 tag v0.3.0 執行產生器而 release-notes.json 沒有 0.3.0 條目
- **THEN** 產生器以非零結束並於 stderr 說明，Release 說明不產生
