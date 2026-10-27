## MODIFIED Requirements

### Requirement: 安裝通路文件與發布狀態誠實化
<!-- BEFORE: 桌面入口含 Linux deb 且 macOS 依晶片分列；CLI 指令為安裝腳本（sh 與 PowerShell）與 Homebrew 兩種 -->

README（中英）SHALL 提供安裝區塊：桌面三平台安裝檔的下載入口（macOS 單一 universal dmg 並註明兩種晶片同一檔、Windows 安裝器、Linux AppImage 依架構各一）、CLI 的三條一行安裝指令（npm 全域安裝、安裝腳本 curl 一行、Homebrew tap），並 SHALL 註明 Windows 沒有安裝腳本（走 npm 或桌面安裝器）、無圖形介面的 Linux 走 CLI 一行安裝，將從原始碼建置的安裝方式降為開發者導向段落；getting-started（中英）的安裝節 SHALL 與 README 呈現的通路一致。文件 SHALL NOT 指示尚不存在的安裝入口；發布管線已接、只待首個 release 才上架的通路 SHALL 以「管線已接＋生效時點」表述，SHALL NOT 讓讀者誤以為現在就裝得到：@speclink/engine 的 sdk-node 文件（中英）SHALL 以 `npm install @speclink/engine` 為主路徑，同段 SHALL 明示實際可安裝以首個帶 engine 的 release 為準，並 SHALL 保留自 repo 建置作為替代路徑。中英兩語版本 SHALL 維持結構與事實對等。

#### Scenario: README 安裝區塊涵蓋桌面與 CLI 通路

- **WHEN** 使用者開啟任一語言的 README 尋找安裝方式
- **THEN** 安裝區塊列出桌面三平台的下載入口（macOS 單一 universal dmg、Windows 安裝器、Linux AppImage）與 CLI 的三條一行安裝指令（npm、安裝腳本、Homebrew tap），不出現 deb 與 PowerShell 安裝腳本，從原始碼建置位於開發者導向段落而非首選位置

#### Scenario: sdk-node 以 npm install 為主路徑並標注生效時點

- **WHEN** 讀者依任一語言的 sdk-node 文件嘗試取得 @speclink/engine
- **THEN** 文件以 `npm install @speclink/engine` 為主路徑，同段明示實際可安裝以首個帶 engine 的 release 為準，且自 repo 建置仍以替代路徑呈現

### Requirement: 安裝章節載明桌面 app 與 CLI 的佈署衝突
<!-- BEFORE: 逐平台表含「Windows 與 deb 由安裝器與套件管理器管理而不動該位置」；無 deb 使用者遷移說明 -->

安裝章節 SHALL 載明桌面 app 與 CLI 共用同一個佈署位置所造成的覆蓋行為：macOS 上桌面 app 於每次啟動將該位置換為指向內建 CLI 的 symlink 並刪除原有檔案，Linux AppImage 僅於版本不符時覆蓋，Windows 由安裝器管理而不動該位置。說明 SHALL 一併給出保留自有 CLI 的做法（改安裝目錄並調整 PATH 順序），SHALL 指出釘選版本會一併失效，並 SHALL 為既有 deb 安裝者註明遷移方式（移除 deb 套件後改裝 AppImage）。

#### Scenario: 先裝 CLI 再裝桌面 app 的人讀得到後果

- **WHEN** 讀者在安裝章節比較桌面 app 與 CLI 兩條路
- **THEN** 可見覆蓋行為的逐平台差異、對釘選版本的影響，以及保留自有 CLI 的具體做法

#### Scenario: deb 安裝者讀得到遷移方式

- **WHEN** 既有 deb 安裝者在任一語言的安裝章節尋找升級方式
- **THEN** 可見「移除 deb 套件後改裝 AppImage」的說明，且章節內不再列 deb 為安裝入口
