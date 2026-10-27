## MODIFIED Requirements

### Requirement: 安裝通路文件與發布狀態誠實化

README（中英）SHALL 提供安裝區塊：桌面三平台安裝檔的下載入口（macOS dmg、Windows 安裝器、Linux AppImage 與 deb）、CLI 的一行安裝指令（安裝腳本與 Homebrew tap 兩種），並將從原始碼建置的安裝方式降為開發者導向段落；getting-started（中英）的安裝節 SHALL 與 README 呈現的通路一致。文件 SHALL NOT 指示尚不存在的安裝入口；發布管線已接、只待首個 release 才上架的通路 SHALL 以「管線已接＋生效時點」表述，SHALL NOT 讓讀者誤以為現在就裝得到：@speclink/engine 的 sdk-node 文件（中英）SHALL 以 `npm install @speclink/engine` 為主路徑，同段 SHALL 明示實際可安裝以首個帶 engine 的 release 為準，並 SHALL 保留自 repo 建置作為替代路徑。中英兩語版本 SHALL 維持結構與事實對等。

<!-- REMOVED-SCENARIO: sdk-node 不再宣稱 npm 入口 -->

#### Scenario: README 安裝區塊涵蓋桌面與 CLI 通路

- **WHEN** 使用者開啟任一語言的 README 尋找安裝方式
- **THEN** 安裝區塊列出桌面三平台的下載入口與 CLI 的一行安裝指令（安裝腳本與 Homebrew tap），從原始碼建置位於開發者導向段落而非首選位置

#### Scenario: sdk-node 以 npm install 為主路徑並標注生效時點

- **WHEN** 讀者依任一語言的 sdk-node 文件嘗試取得 @speclink/engine
- **THEN** 文件以 `npm install @speclink/engine` 為主路徑，同段明示實際可安裝以首個帶 engine 的 release 為準，且自 repo 建置仍以替代路徑呈現
