## Purpose

定義裝置共用的 OWASP／CWE 標準資料生命週期：只從核准官方版本取得機器可讀資料，保存授權與完整性資訊，提供有界、可取消且失敗保舊的安裝、匯入與刪除。其資料與專案政策、自有程式碼分離，審查時只讀已安裝版本而不自動連網。

## ADDED Requirements

### Requirement: 應用程式資料目錄與離線版本清單

標準 SHALL 存於 SpecLink 應用程式資料目錄的 security/standards，不存任意專案；Desktop／CLI SHALL 使用同一絕對路徑解析並支援 portable。無可用絕對目錄 SHALL 明確失敗，不退回工作目錄。已安裝 CWE 版本 SHALL 動態列出。list／查詢／基本審查／報告 SHALL 在所需資料已安裝後離線使用，缺資料不自動下載。

#### Scenario: Desktop 安裝後 CLI 離線可用

- **WHEN** 使用者在 Desktop 安裝資料後關閉網路，從另一個專案呼叫 security data list --json
- **THEN** CLI 列同一已安裝版本，沒有網路請求，兩個專案工作目錄都沒有新增標準原文

#### Scenario: 缺裝置路徑與缺資料

- **WHEN** 無法解析應用程式絕對資料目錄，或專案選定版本尚未安裝
- **THEN** 前者回 data_directory_unavailable 且不寫工作目錄，後者標示 missing_data 並提供前往資料管理操作

### Requirement: 核准來源與版本資料解析

可用矩陣 SHALL 含 Top 10 2021／2025、API Top 10 2019／2023、ASVS 4.0.3／5.0.0、已支援的 CWE 版本。來源 SHALL 限 OWASP 官方 repository／release 與 MITRE 固定版資產，JSON／Markdown／XML 機器解析，不依 HTML 排版。refresh／install SHALL 由使用者明確觸發，不能審查時發起。SHALL 拒絕浮動 latest 冒充固定版、任意 URL、非 HTTPS、私有／回環位址與未核准 redirect；最多三跳。refresh SHALL 只列可由現有 parser 驗證的候選，新格式標不可用，不載入遠端規則或程式。

#### Scenario: 兩版 ASVS 等級不同格式

- **WHEN** 安裝 ASVS 4.0.3 與 5.0.0 後分別選 L2
- **THEN** 4.0.3 依官方逐級 Required、5.0.0 依最低 L 展開 L1／L2，版本及項目 ID 保持各自獨立，不把缺欄位視為通過

#### Scenario: 來源或內容版本不符

- **WHEN** 清單聲稱 5.0.0 但根 Version 為 4.0.3，或下載導向非核准主機
- **THEN** 安裝拒絕並顯示 version_mismatch 或 untrusted_source，既有版本與 current 指標不變

### Requirement: 來源與授權 metadata

每個 generation SHALL 保存 standard_id、version、source_url、license_name、license_url、attribution、downloaded_at、sha256、installed_status、source_format 及來源 revision／完整性種類／parser 版本。JSON CLI SHALL 用 camelCase。原始資料、解析衍生物與自有程式 SHALL 分開，不以程式碼的 MIT 取代原授權。SHALL 保存各實際資產通知、署名、授權與修改說明，介面可檢視；缺必要授權資訊 SHALL 拒絕安裝。自算 SHA SHALL 標內容指紋，不能宣稱官方 checksum 或認證。

#### Scenario: 舊 ASVS 資產沒有官方 digest

- **WHEN** 從官方固定 release 首次下載未附官方 digest 的資料
- **THEN** 保存 source-tls 完整性種類、實際 sha256、官方來源與授權，不套用 latest 的雜湊或標示官方 checksum 已驗證

#### Scenario: 資料與程式授權分開

- **WHEN** 檢視 ASVS／API CC BY-SA 資料或 CWE MITRE 資料的來源授權
- **THEN** 顯示各自來源條款、署名及解析變更說明，不顯示為 SpecLink MIT 程式授權，匯出引用保留 attribution

### Requirement: 受限 ZIP 與安全 XML 匯入

離線包 SHALL 為 formatVersion=1 的 ZIP，根 manifest.json 列 standardId／version／sourceRevision／授權與 entries 的 name／sha256，內容僅列明 raw 資料及 LICENSE。SHALL 在寫入前檢查全部 entry，拒絕絕對／點點／盤符／反斜線路徑、symlink、executable、重名／大小寫碰撞、未列檔案與指紋不符，不任意 extract。XML SHALL 拒絕 DTD、外部與非內建實體，不取外部 schema。匯入 SHALL 標 imported，包內自填 checksum 不提供官方可信來源證明。

#### Scenario: Zip Slip 與實體攻擊

- **WHEN** 包內有 ../outside、C:\\secret、symlink、重名或 XML 外部實體
- **THEN** 回 invalid_package，不在應用程式受控暫存外寫檔、不讀外部實體、不取網路，舊資料不變

#### Scenario: 離線匯入合法資料

- **WHEN** 無網路時匯入版本、指紋、結構及授權皆有效的包
- **THEN** 安裝成功、來源種類為 imported、可供離線查詢，清單與 metadata 顯示正確版本

官方 CWE 內層封裝若將 regular XML 設為可執行權限（已核實4.20為100750），專用解析路徑 SHALL 只接受單一、名稱符合所選版本的 regular XML，以有界 bytes 解析且不沿用原權限；所有連結／特殊檔仍拒絕，外層離線包的可執行項目限制不變。SHALL NOT 解壓至任意路徑或執行 XML 內容。

#### Scenario: 官方 CWE XML 帶執行權限位元

- **WHEN** 固定版本的官方 CWE ZIP 僅有 cwec_v4.20.xml、regular mode 100750，內容通過4.20根版本及結構驗證
- **THEN** 只將 XML 當資料保存，檔案不帶封裝執行權限；包內不是XML、其他名稱、多檔、連結及特殊檔一律拒絕

### Requirement: 有界下載與可取消進度

安裝 SHALL 限下載 64 MiB、解壓總量 256 MiB、128 entries、單 XML 128 MiB、其他單檔 2 MiB、100,000 標準項目、XML 深度 64、120 秒。大小 SHALL 按實際讀取核對，不能只信 header。SHALL 提供階段／bytes／files 進度及取消，兩秒內停止且不提交未完成版本；同版多行程安裝／刪除 SHALL 互斥，第二個操作回 busy。

#### Scenario: 巨大回應與壓縮膨脹

- **WHEN** 下載實際超過 67,108,864 bytes，或解壓超過 268,435,456 bytes
- **THEN** 停止並顯示 size_limit，舊版可讀，沒有安裝成功狀態

#### Scenario: 使用者取消下載

- **WHEN** Desktop 點取消或 CLI 按 Ctrl+C
- **THEN** 兩秒內結束，CLI exit 130，進度呈取消，未完成 generation 不可見，既有資料可離線使用

### Requirement: 原子安裝與錯誤保舊

下載／匯入 SHALL 依暫存、驗證、完整 generation、原子替換 current 指標的順序提交。讀者 SHALL 只見完整同代資料；版本衝突、網路、解析、權限、磁碟寫入或取消在提交前失敗 SHALL 不改舊指標。提交後清理失敗 SHALL 明示已安裝附 cleanup 警告，不回未安裝。崩潰 staging SHALL 不在 installed 清單可見；下一次清理不得刪仍活躍工作。

#### Scenario: 更新失敗與讀取競態

- **WHEN** 重新下載已安裝版本時新內容解析失敗，或讀取與新代提交同時發生
- **THEN** 前者舊代 SHA／current 不變，後者讀到完整舊代或完整新代，沒有跨代拼接結果

#### Scenario: 指標提交後清理失敗

- **WHEN** 新 generation 已完整提交但舊暫存清理失敗
- **THEN** 回成功附 cleanup 警告，清單顯示新代，下一次能安全重試清理

### Requirement: 刪除本機已知未使用版本

remove SHALL 檢查執行中 pin、本機已知專案政策與準備快照引用，使用中 SHALL 拒絕。SHALL 說明檢查只涵蓋本機已知引用，不宣稱知道全部未開啟遠端專案。刪除 SHALL 不修改政策；之後選到缺資料的專案 SHALL 顯示缺失。刪除與讀取／安裝 SHALL 互斥，刪除失敗留下可重試狀態而不顯示可用。

#### Scenario: 使用中的資料不能刪

- **WHEN** 某版本正被掃描使用或本機準備政策引用，請求刪除
- **THEN** 顯示 in_use 與本機已知原因，資料和政策不變

#### Scenario: 移除未使用資料

- **WHEN** 無本機已知引用的版本刪除成功
- **THEN** 清單不再列為 installed、政策文件不變、舊報告仍保有當時版本與來源 metadata

### Requirement: 資料管理 CLI 輸入輸出

security data SHALL 提供 list [--json]、refresh [--json]、install STANDARD --version VERSION [--reinstall] [--json]、import PATH [--json]、remove STANDARD --version VERSION [--json]；無 stdin、支援 --no-color，成功0、錯誤1、取消130、clap 語法錯誤2。list／refresh stdout JSON SHALL 為 {standards:[{standardId:string,versions:[{version:string,status:string,sourceUrl:string,licenseName:string,licenseUrl:string,attribution:string,sourceFormat:string}]}]}。install／import SHALL 回 {standardId,version,installedStatus,sha256,sourceRevision,integrityKind} 字串欄位；remove 回 {standardId:string,version:string,removed:boolean}。人眼 SHALL 顯示同資訊，進度與錯誤至 stderr，--json stdout 不混進度；installedStatus SHALL 區分 missing／installed／invalid。

#### Scenario: 清單與安裝 JSON

- **WHEN** 執行 list --json --no-color，然後明確 install owasp_top10 --version 2025 --json
- **THEN** stdout 各為單一符合指定型別的 JSON，無 ANSI，進度在 stderr；安裝成功為 installed 且有內容指紋

#### Scenario: 不存在或壞包

- **WHEN** import 指定不存在的檔案或 install 指定不支援版本
- **THEN** exit 1，stderr 指出原因，stdout 沒有成功 payload，現有清單及已安裝資料不變


### Requirement: 下載中的版本列即時回饋

Desktop SHALL 在點選安裝或重新下載後，立即將當次標準及版本的按鈕顯示為下載中，直到工作結算；其他版本 SHALL 保留文案且停用。取消／失敗 SHALL 還原按鈕，SHALL NOT 冒稱已安裝。

#### Scenario: API 2019 下載時僅更新該列

- **WHEN** API 2019 正在下載而 API 2023 尚未安裝
- **THEN** 2019 顯示「下載中」，2023 仍顯示「下載」且不能按下，失敗後2019還原為「下載」
