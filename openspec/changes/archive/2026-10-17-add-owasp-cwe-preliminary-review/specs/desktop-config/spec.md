## ADDED Requirements

### Requirement: 產出政策的初步安全審查控制

產出流程頁的既有產出政策卡 SHALL 在 audit 下方新增主開關「啟用 OWASP / CWE 安全審查」，說明 SHALL 為「依選定的 OWASP / CWE 安全標準與規則，對 SDD 規格、原始碼及 API 進行初步安全審查。」預設關閉時 SHALL 隱藏全部安全子設定，不發送新掃描、不影響既有 audit／Verify／Archive。啟用後 SHALL 顯示範圍、標準／版本、審查項目、資料安裝狀態、排除目錄／個別規則停用與儲存，沿用既有元件、字型、深色及 zh-TW／en 多語系；SHALL 不另開 OWASP 專案設定頁。locale／spec_locale／tdd／audit／worktree 及 Remote 隱藏 worktree 的既有行為 SHALL 保持。

#### Scenario: 主開關顯示與還原

- **WHEN** 使用者依序開啟、儲存、關閉、儲存並重新開啟設定
- **THEN** 子設定隨開關顯示／隱藏，已選值仍可還原，原有五項政策不變，停用沒有掃描 IPC

#### Scenario: 文案與深色模式

- **WHEN** 在 zh-TW／en 介面與淺色／深色外觀檢視安全設定
- **THEN** 標籤與說明依語系顯示、繁中完全符合指定文案，使用共用樣式且控制項可閱讀、具可存取名稱

### Requirement: 範圍與標準各自獨立選用

範圍 SHALL 單選 off「關閉／不指定範圍」、web-api「Web 應用程式 + API」、api「僅 API / 後端服務」、web「僅 Web 應用程式」，並說明各範圍重點與 off 不等於主開關停用。Top 10 SHALL 有關閉／2021／2025；API Top 10 有關閉／2019／2023；ASVS 有關閉／4.0.3／5.0.0，啟用時顯示L1／L2／L3；CWE版本 SHALL 由已安裝清單產生。SHALL 允許單一、多個或全部關閉，不將 ASVS／CWE 當 Top 10 的前提。三種審查 SHALL 各自開關，主開關開啟但標準全關 SHALL 顯示未配置提示。

#### Scenario: 只選 Top 10 2025

- **WHEN** 主開關開啟、範圍選 web-api，只選 Top 10 2025，其他三個標準關閉後儲存再重開
- **THEN** 所有選項與版本還原，ASVS等級隱藏，不要求安裝ASVS或CWE，可看Top10資料安裝狀態

#### Scenario: 全標準關閉與範圍 off

- **WHEN** enabled=true、scope=off、四標準關閉
- **THEN** 主開關仍開，顯示已啟用但未配置與至少選一個標準的提示，自有基本規則選項仍可明確啟用

#### Scenario: 未安裝的已選 CWE

- **WHEN** 政策已選某CWE版本但此裝置沒有安裝
- **THEN** 保留並標記該版本缺失，不靜默改選其他版本或關閉該標準

### Requirement: 安全政策 Local 與 Remote 儲存

安全設定 SHALL 使用既有政策改寫及儲存流程，保留未知欄位／註解及未編輯葉值。Remote SHALL 沿用 revision 對照、editor授權及reader唯讀；衝突保留輸入，無條件覆寫不存在；離線或認證失效 SHALL 浮出錯誤且不假裝儲存成功。儲存成功後 SHALL 更新快照，重新進入顯示一致內容。

#### Scenario: Remote 衝突與 reader

- **WHEN** 另一client先更新revision，或reader開啟安全設定
- **THEN** 前者保留我的輸入並顯示server現值對照，後者只讀且儲存停用，server文件不被覆蓋

### Requirement: 應用程式設定的安全標準資料管理

應用程式設定 SHALL 新增「安全標準資料」分類，與專案無關且零專案時可進入。SHALL 顯示標準與可用版本、安裝狀態、來源／授權、明確下載／更新清單、重下載、未使用版本刪除與離線包匯入；工作時顯示進度及取消，失敗顯示具體階段／原因。下載只能由使用者操作觸發；從審查缺資料提示導向此頁 SHALL 不自動開始下載。

#### Scenario: 零專案管理資料

- **WHEN** Desktop沒有開啟專案而使用者進入應用程式設定
- **THEN** 可檢視及管理標準資料，不要求新增專案，也不在專案根建立資料檔

#### Scenario: 取消與失敗保舊

- **WHEN** 使用者重下載已安裝版後取消，或收到格式／版本錯誤
- **THEN** 顯示取消或具體錯誤，原已安裝版本仍可用，未完成版本不顯示installed
