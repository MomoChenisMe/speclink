## ADDED Requirements

### Requirement: 桌面自動更新

desktop SHALL 於啟動後在背景檢查 GitHub Releases 更新端點，並提供手動「檢查更新」入口。發現新版時 SHALL 顯示目標版本並徵得使用者同意後才下載套用；SHALL NOT 靜默安裝。更新包簽章驗證失敗時 SHALL 拒絕安裝並顯示錯誤。檢查失敗（離線、端點不可達）SHALL 靜默，不阻擋啟動、不彈出錯誤。

#### Scenario: 發現新版徵求同意後套用

- **WHEN** 啟動時端點回報的版本高於目前版本，且使用者在更新提示中同意
- **THEN** desktop 下載並套用更新，提示重新啟動；重啟後執行中的版本為新版

#### Scenario: 檢查失敗靜默

- **WHEN** 啟動時無網路或更新端點不可達
- **THEN** desktop 照常啟動，不顯示任何錯誤提示；手動檢查時才顯示「無法檢查更新」

#### Scenario: 簽章不符拒裝

- **WHEN** 下載的更新包簽章與內嵌公鑰驗證不符
- **THEN** desktop 拒絕安裝、顯示更新失敗錯誤，既有安裝不受影響

#### Scenario: 手動檢查已是最新

- **WHEN** 使用者觸發「檢查更新」且目前已是最新版本
- **THEN** 介面顯示已是最新，不出現下載動作

### Requirement: 安裝 CLI 指令到 PATH

desktop SHALL 提供「安裝 CLI 指令」動作，並呈現目前狀態（未安裝／已安裝含版本／版本不符）。佈署方式依平台：macOS 於 ~/.local/bin 建立指向 app bundle 內 CLI 的 symlink；Windows 由 NSIS 安裝器將安裝目錄寫入使用者 PATH（app 內動作僅呈現狀態）；Linux deb 佈署 /usr/bin/speclink；Linux AppImage 將 CLI 複製至 ~/.local/bin，且 app 啟動偵測到版本不符時 SHALL 自動重新佈署。佈署目錄不在 PATH 時 SHALL 提示加入方式。

#### Scenario: 安裝後 CLI 與 desktop 同版

- **WHEN** macOS 使用者觸發安裝 CLI 動作後於終端執行 speclink --version
- **THEN** 輸出的版本與 desktop 版本一致，且該指令解析自 ~/.local/bin 的 symlink

#### Scenario: 已安裝呈現狀態

- **WHEN** CLI 已佈署且版本一致時開啟安裝 CLI 介面
- **THEN** 介面顯示已安裝與目前版本，不重複執行佈署

#### Scenario: AppImage 版本不符自我修復

- **WHEN** Linux AppImage 更新後啟動，偵測到 ~/.local/bin 的 CLI 版本與 app 不符
- **THEN** desktop 自動以新版重新複製佈署，無需使用者操作

#### Scenario: 佈署目錄不在 PATH 時提示

- **WHEN** 安裝動作完成但 ~/.local/bin 不在使用者 PATH
- **THEN** 介面提示需將該目錄加入 PATH 及加入方式
