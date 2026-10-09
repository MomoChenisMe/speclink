## ADDED Requirements

### Requirement: macOS 原生選單

桌面 app 於 macOS SHALL 以自己的選單列取代 Tauri 預設選單,共六組:Speclink(關於 Speclink、檢查更新…、設定… ⌘,、服務、隱藏 Speclink ⌘H、隱藏其他 ⌥⌘H、顯示全部、結束 Speclink ⌘Q)、檔案(開啟專案… ⌘O、關閉專案 ⌘W、安裝 CLI…)、編輯(復原、重做、剪下、複製、貼上、全選)、檢視(變更 ⌘1、已封存 ⌘2、規格 ⌘3、手冊 ⌘4、專案設定、搜尋看板 ⌘F、重新整理 ⌘R、下一個專案 ⌃Tab)、視窗(縮到最小 ⌘M、縮放、全部移到最前)、說明(手冊、更新日誌、GitHub、回報問題)。選單文案 SHALL 依 UI 語言偏好呈現 zh-TW 或 en,語言切換後 SHALL 立即重建;沒有作用中專案時關閉專案、檢視組全部與說明的手冊 SHALL 停用。各項動作 SHALL 為:開啟專案開「新增專案」對話框;關閉專案關閉作用中專案;變更／已封存／規格／手冊／專案設定切至對應頁;搜尋看板聚焦看板搜尋輸入;重新整理重讀作用中專案;下一個專案循環切換作用中專案;檢查更新執行手動檢查並進入應用程式設定頁的軟體更新卡;設定與安裝 CLI 進入應用程式設定頁;關於 Speclink 於「關於」頁落地前進入應用程式設定頁;更新日誌開啟更新日誌對話框;GitHub 與回報問題以系統瀏覽器開啟專案的 GitHub 頁與新 issue 頁;視窗組與編輯組為系統預設行為。Windows 與 Linux SHALL NOT 顯示選單列,既有鍵盤快捷鍵行為不變。

#### Scenario: 選單文案跟隨 UI 語言

- **WHEN** macOS 使用者於設定頁把 UI 語言自 zh-TW 切為 en
- **THEN** 選單列六組標題與項目立即變為英文(File、Edit、View、Window、Help 與 Open Project…、Close Project、Changes、Archived、Specs、Manual 等),切回 zh-TW 後立即變回繁中

#### Scenario: 檢視組快捷鍵切頁

- **WHEN** 有作用中專案時按 ⌘3
- **THEN** 主區切至規格頁、專案欄「規格」為作用中;按 ⌘1 回到看板

#### Scenario: 搜尋看板經選單聚焦

- **WHEN** 看板顯示中,使用者自檢視選單選「搜尋看板」或按 ⌘F
- **THEN** 看板搜尋輸入取得焦點,行為與「看板搜尋過濾卡片」需求的快捷鍵一致

#### Scenario: 無專案時相關項目停用

- **WHEN** app 於零專案狀態展開檔案與檢視選單
- **THEN** 「關閉專案」與檢視組全部項目呈停用、不可觸發;「開啟專案…」可用,按下開「新增專案」對話框

#### Scenario: 說明組開外部網址

- **WHEN** 使用者自說明選單選「回報問題」
- **THEN** 系統瀏覽器開啟 https://github.com/MomoChenisMe/speclink/issues/new,app 畫面無變化

#### Scenario: Windows 無選單列

- **WHEN** 於 Windows 啟動桌面 app
- **THEN** 視窗無選單列;Ctrl+Tab 與 Ctrl+1..9 仍可切換專案、Ctrl+F 仍聚焦看板搜尋
