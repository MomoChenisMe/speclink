## ADDED Requirements

### Requirement: 安裝通路文件與發布狀態誠實化

README（中英）SHALL 提供安裝區塊：桌面三平台安裝檔的下載入口（macOS dmg、Windows 安裝器、Linux AppImage 與 deb）、CLI 的一行安裝指令（安裝腳本與 Homebrew tap 兩種），並將從原始碼建置的安裝方式降為開發者導向段落；getting-started（中英）的安裝節 SHALL 與 README 呈現的通路一致。文件 SHALL NOT 指示尚不存在的安裝入口：@speclink/engine 未發布至 npm 期間，sdk-node 文件（中英）SHALL 明示尚未發布至 npm，並改以 repo 內建置的載入路徑示範。中英兩語版本 SHALL 維持結構與事實對等。

#### Scenario: README 安裝區塊涵蓋桌面與 CLI 通路

- **WHEN** 使用者開啟任一語言的 README 尋找安裝方式
- **THEN** 安裝區塊列出桌面三平台的下載入口與 CLI 的一行安裝指令（安裝腳本與 Homebrew tap），從原始碼建置位於開發者導向段落而非首選位置

#### Scenario: sdk-node 不再宣稱 npm 入口

- **WHEN** 讀者依任一語言的 sdk-node 文件嘗試取得 @speclink/engine
- **THEN** 文件明示該套件尚未發布至 npm，示範採 repo 內建置路徑，全文無 npm install 該套件的指示
