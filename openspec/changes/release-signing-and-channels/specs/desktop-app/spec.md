## MODIFIED Requirements

### Requirement: 安裝 CLI 指令到 PATH

desktop SHALL 佈署 CLI 指令並呈現目前狀態（未安裝／已安裝含版本／版本不符）。佈署方式依平台：macOS 於 ~/.local/bin 建立指向 app bundle 內 CLI 的 symlink，且 app 啟動偵測到未安裝或版本不符時 SHALL 自動佈署（symlink 冪等，毋須使用者操作）；Windows 由 NSIS 安裝器將安裝目錄寫入使用者 PATH（app 內僅呈現狀態）；Linux deb 佈署 /usr/bin/speclink；Linux AppImage 將 CLI 複製至 ~/.local/bin，且 app 啟動偵測到版本不符時 SHALL 自動重新佈署。PATH 設定依平台：macOS 於 ~/.local/bin 不在 PATH 時 SHALL 自動追加一行 PATH 匯出至 ~/.zprofile（帶識別註解、冪等、僅缺席時寫一次）；Linux 佈署目錄不在 PATH 時 SHALL 提示加入方式。

#### Scenario: 安裝後 CLI 與 desktop 同版

- **WHEN** macOS 使用者啟動 app 完成自動佈署後於終端執行 speclink --version
- **THEN** 輸出的版本與 desktop 版本一致，且該指令解析自 ~/.local/bin 的 symlink

#### Scenario: 已安裝呈現狀態

- **WHEN** CLI 已佈署且版本一致時開啟安裝 CLI 介面
- **THEN** 介面顯示已安裝與目前版本，不重複執行佈署

#### Scenario: macOS 啟動自動佈署與自我修復

- **WHEN** macOS 使用者啟動 app 且 ~/.local/bin 無 speclink（或版本與 app 不符）
- **THEN** desktop 自動建立（或重建）symlink，無需使用者操作；佈署失敗不阻斷 app 啟動

#### Scenario: AppImage 版本不符自我修復

- **WHEN** Linux AppImage 更新後啟動，偵測到 ~/.local/bin 的 CLI 版本與 app 不符
- **THEN** desktop 自動以新版重新複製佈署，無需使用者操作

#### Scenario: macOS 自動追加 PATH 且冪等

- **WHEN** macOS 佈署完成而 ~/.local/bin 不在 PATH，且 ~/.zprofile 尚無 speclink 識別註解行
- **THEN** desktop 於 ~/.zprofile 追加帶識別註解的 PATH 匯出一行；重複啟動不重複追加；已在 PATH 或註解行已存在時不寫入

#### Scenario: 佈署目錄不在 PATH 時提示

- **WHEN** Linux 佈署完成但 ~/.local/bin 不在使用者 PATH
- **THEN** 介面提示需將該目錄加入 PATH 及加入方式
