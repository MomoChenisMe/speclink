# security-standard-data Specification

## Purpose

定義裝置共用的 OWASP／CWE 標準資料生命週期：只從核准官方版本取得機器可讀資料，保存授權與完整性資訊，提供有界、可取消且失敗保舊的安裝、匯入與刪除。其資料與專案政策、自有程式碼分離，審查時只讀已安裝版本而不自動連網。

## Requirements

### Requirement: 應用程式資料目錄與離線版本清單

標準 SHALL 預設存於 SpecLink 應用程式資料目錄的 security/standards，不存任意專案；Desktop／CLI SHALL 使用同一絕對路徑解析並支援 portable。裝置明確設定自訂標準根目錄時 SHALL 改用該目錄，Windows 支援完整 UNC，其他平台支援已掛載的絕對路徑；自訂值直接代表標準根，不再加 security/standards。設定缺檔／null SHALL 沿用預設，壞設定或無可用絕對目錄 SHALL 明確失敗，不退回工作目錄。已安裝 CWE 版本 SHALL 動態列出。list／查詢／基本審查／報告 SHALL 在所需資料已安裝後無需 HTTP 下載使用，缺資料不自動下載；選網路共用位置時 SHALL 明示需要該檔案系統連線，不將 UNC 存取宣稱完全斷網。

共用根 SHALL 僅存標準版本及完整同代原文、對照、指南、授權；私人報告、準備快取、使用引用、認證與診斷 SHALL 留本機，專案 .security_kb 沿原快照行為。每次操作 SHALL 固定同一根與 generation，維持 SHA、路徑及必要文件驗證。多人寫入同版本 SHALL 經跨程序鎖互斥，以同一共用目錄 staging 及原子指標提交；忙碌、鎖不支援或發佈失敗 SHALL 拒寫並保舊，不顯示半套版本。唯讀使用者 SHALL 可查詢、基本／深度資料讀取與匯出，不在共用根建立私人引用。自訂根的刪除 SHALL 停用，不能依單機引用判斷全團隊無人使用；既有預設本機刪除政策不變。切換／還原 SHALL 不自動搬移或清除原資料，斷線 SHALL 明示不可用且不自動下載本機副本。

#### Scenario: Desktop 安裝後 CLI 離線可用

- **WHEN** 使用者在 Desktop 安裝資料後關閉網路，從另一個專案呼叫 security data list --json
- **THEN** CLI 列同一已安裝版本，沒有網路請求，兩個專案工作目錄都沒有新增標準原文

#### Scenario: 缺裝置路徑與缺資料

- **WHEN** 無法解析應用程式絕對資料目錄，或專案選定版本尚未安裝
- **THEN** 前者回 data_directory_unavailable 且不寫工作目錄，後者標示 missing_data 並提供前往資料管理操作

#### Scenario: 兩台電腦共用同一資料

- **WHEN** 兩台 Windows 裝置選同一 UNC 標準根，一台安裝完整版本後另一台執行 security data list --json 與基本審查
- **THEN** 另一台讀到相同指紋資料且不再下載，既有 JSON 欄位型別、人眼輸出及 --no-color 行為不變；私人報告不出現在共用根

#### Scenario: 共用寫入競爭與失敗保舊

- **WHEN** 兩個程序同時更新同版本，或 share 不支援必要鎖／原子指標發佈
- **THEN** 競爭者回 busy 或明確 I/O 錯誤，原完整 generation 可讀且沒有半套成功，不能無鎖覆寫

#### Scenario: 共用位置不可用或唯讀

- **WHEN** share 離線、權限遭拒或只有讀取權
- **THEN** 前兩者明確失敗且不退回本機；唯讀者可讀及匯出而變更動作被拒，自訂根刪除一律停用，舊版本與本機私人資料不受影響


<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->

---
### Requirement: 核准來源與版本資料解析

可用矩陣 SHALL 含 Top 10 2021／2025、API Top 10 2019／2023、ASVS 4.0.3／5.0.0、已支援的 CWE 版本。來源 SHALL 限 OWASP 官方 repository／release 與 MITRE 固定版資產，JSON／Markdown／XML 機器解析，不依 HTML 排版。refresh／install SHALL 由使用者明確觸發，不能審查時發起。SHALL 拒絕浮動 latest 冒充固定版、任意 URL、非 HTTPS、私有／回環位址與未核准 redirect；最多三跳。refresh SHALL 只列可由現有 parser 驗證的候選，新格式標不可用，不載入遠端規則或程式。

#### Scenario: 兩版 ASVS 等級不同格式

- **WHEN** 安裝 ASVS 4.0.3 與 5.0.0 後分別選 L2
- **THEN** 4.0.3 依官方逐級 Required、5.0.0 依最低 L 展開 L1／L2，版本及項目 ID 保持各自獨立，不把缺欄位視為通過

#### Scenario: 來源或內容版本不符

- **WHEN** 清單聲稱 5.0.0 但根 Version 為 4.0.3，或下載導向非核准主機
- **THEN** 安裝拒絕並顯示 version_mismatch 或 untrusted_source，既有版本與 current 指標不變


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
### Requirement: 來源與授權 metadata

每個 generation SHALL 保存 standard_id、version、source_url、license_name、license_url、attribution、downloaded_at、sha256、installed_status、source_format 及來源 revision／完整性種類／parser 版本。JSON CLI SHALL 用 camelCase。原始資料、解析衍生物與自有程式 SHALL 分開，不以程式碼的 MIT 取代原授權。SHALL 保存各實際資產通知、署名、授權與修改說明，介面可檢視；缺必要授權資訊 SHALL 拒絕安裝。自算 SHA SHALL 標內容指紋，不能宣稱官方 checksum 或認證。

#### Scenario: 舊 ASVS 資產沒有官方 digest

- **WHEN** 從官方固定 release 首次下載未附官方 digest 的資料
- **THEN** 保存 source-tls 完整性種類、實際 sha256、官方來源與授權，不套用 latest 的雜湊或標示官方 checksum 已驗證

#### Scenario: 資料與程式授權分開

- **WHEN** 檢視 ASVS／API CC BY-SA 資料或 CWE MITRE 資料的來源授權
- **THEN** 顯示各自來源條款、署名及解析變更說明，不顯示為 SpecLink MIT 程式授權，匯出引用保留 attribution


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
### Requirement: 受限 ZIP 與安全 XML 匯入

單一版本離線包 SHALL 為 formatVersion=1 的 ZIP，根 manifest.json 列 standardId／version／sourceRevision／授權與 entries 的 name／sha256，內容僅列明 raw 資料及 LICENSE。SHALL 在寫入前檢查全部 entry，拒絕絕對／點點／盤符／反斜線路徑、symlink、executable、重名／大小寫碰撞、未列檔案與指紋不符，不任意 extract。XML SHALL 拒絕 DTD、外部與非內建實體，不取外部 schema。匯入 SHALL 標 imported，包內自填 checksum 不提供官方可信來源證明。

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

formatVersion=2集合包 SHALL 依「離線集合包匯入與逐版本結果」處理，內含單包仍逐一遵守本需求全部限制，不將外層格式放寬套用到單包。


<!-- @trace
source: desktop-colors-network-and-offline-bundles
updated: 2026-10-19T23:00:19+08:00
-->

---
### Requirement: 有界下載與可取消進度

安裝 SHALL 限下載 64 MiB、解壓總量 256 MiB、128 entries、單 XML 128 MiB、其他單檔 2 MiB、100,000 標準項目、XML 深度 64、120 秒。大小 SHALL 按實際讀取核對，不能只信 header。SHALL 提供階段／bytes／files 進度及取消，兩秒內停止且不提交未完成版本；同版多行程安裝／刪除 SHALL 互斥，第二個操作回 busy。

#### Scenario: 巨大回應與壓縮膨脹

- **WHEN** 下載實際超過 67,108,864 bytes，或解壓超過 268,435,456 bytes
- **THEN** 停止並顯示 size_limit，舊版可讀，沒有安裝成功狀態

#### Scenario: 使用者取消下載

- **WHEN** Desktop 點取消或 CLI 按 Ctrl+C
- **THEN** 兩秒內結束，CLI exit 130，進度呈取消，未完成 generation 不可見，既有資料可離線使用


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
### Requirement: 原子安裝與錯誤保舊

下載／匯入 SHALL 依暫存、驗證、完整 generation、原子替換 current 指標的順序提交。讀者 SHALL 只見完整同代資料；版本衝突、網路、解析、權限、磁碟寫入或取消在提交前失敗 SHALL 不改舊指標。提交後清理失敗 SHALL 明示已安裝附 cleanup 警告，不回未安裝。崩潰 staging SHALL 不在 installed 清單可見；下一次清理不得刪仍活躍工作。

#### Scenario: 更新失敗與讀取競態

- **WHEN** 重新下載已安裝版本時新內容解析失敗，或讀取與新代提交同時發生
- **THEN** 前者舊代 SHA／current 不變，後者讀到完整舊代或完整新代，沒有跨代拼接結果

#### Scenario: 指標提交後清理失敗

- **WHEN** 新 generation 已完整提交但舊暫存清理失敗
- **THEN** 回成功附 cleanup 警告，清單顯示新代，下一次能安全重試清理


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
### Requirement: 刪除本機已知未使用版本

remove SHALL 檢查執行中 pin、本機已知專案政策與準備快照引用，使用中 SHALL 拒絕。SHALL 說明檢查只涵蓋本機已知引用，不宣稱知道全部未開啟遠端專案。刪除 SHALL 不修改政策；之後選到缺資料的專案 SHALL 顯示缺失。刪除與讀取／安裝 SHALL 互斥，刪除失敗留下可重試狀態而不顯示可用。

#### Scenario: 使用中的資料不能刪

- **WHEN** 某版本正被掃描使用或本機準備政策引用，請求刪除
- **THEN** 顯示 in_use 與本機已知原因，資料和政策不變

#### Scenario: 移除未使用資料

- **WHEN** 無本機已知引用的版本刪除成功
- **THEN** 清單不再列為 installed、政策文件不變、舊報告仍保有當時版本與來源 metadata


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
### Requirement: 資料管理 CLI 輸入輸出

security data SHALL 提供 list [--json]、refresh [--json]、install STANDARD --version VERSION [--reinstall] [--json]、import PATH [--json]、remove STANDARD --version VERSION [--json]；無 stdin、支援 --no-color，成功0、錯誤1、取消130、clap 語法錯誤2。list／refresh stdout JSON SHALL 為 {standards:[{standardId:string,versions:[{version:string,status:string,sourceUrl:string,licenseName:string,licenseUrl:string,attribution:string,sourceFormat:string}]}]}。install／單包import SHALL 回 {standardId,version,installedStatus,sha256,sourceRevision,integrityKind} 字串欄位；remove 回 {standardId:string,version:string,removed:boolean}。人眼 SHALL 顯示同資訊，進度與錯誤至 stderr，--json stdout 不混進度；installedStatus SHALL 區分 missing／installed／invalid。

#### Scenario: 清單與安裝 JSON

- **WHEN** 執行 list --json --no-color，然後明確 install owasp_top10 --version 2025 --json
- **THEN** stdout 各為單一符合指定型別的 JSON，無 ANSI，進度在 stderr；安裝成功為 installed 且有內容指紋

#### Scenario: 不存在或壞包

- **WHEN** import 指定不存在的檔案或 install 指定不支援版本
- **THEN** exit 1，stderr 指出原因，stdout 沒有成功 payload，現有清單及已安裝資料不變

CLI SHALL 新增「離線資料包CLI匯出與集合匯入」規定的export入口，既有import PATH亦接受集合包並依該需求輸出逐項結果；單包人眼與JSON輸出保持既有位元級契約，不強制舊包轉換。


<!-- @trace
source: desktop-colors-network-and-offline-bundles
updated: 2026-10-19T23:00:19+08:00
-->

---
### Requirement: 下載中的版本列即時回饋

Desktop SHALL 在點選安裝或重新下載後，立即將當次標準及版本的按鈕顯示為下載中，直到工作結算；其他版本 SHALL 保留文案且停用。取消／失敗 SHALL 還原按鈕，SHALL NOT 冒稱已安裝。

#### Scenario: API 2019 下載時僅更新該列

- **WHEN** API 2019 正在下載而 API 2023 尚未安裝
- **THEN** 2019 顯示「下載中」，2023 仍顯示「下載」且不能按下，失敗後2019還原為「下載」

<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
### Requirement: 已安裝標準資料的離線匯出

Desktop SHALL 在安全標準資料頁提供每個有效已安裝標準版本的「匯出」及頁級「全部匯出」，以原生儲存選擇器選路徑。單包 SHALL 沿用formatVersion=1，含manifest.json、已列raw資料與LICENSE及來源／授權／署名／sourceRevision／SHA資訊；SHALL NOT 包含parsed.json、鎖、generation、專案內容或把自算指紋當官方認證。匯出 SHALL 完全離線，不安裝、不補下載、不更動已安裝資料與current。

全部匯出 SHALL 凍結起始清單，按standardId與version字串順序選全部有效installed版本，invalid列入skipped並顯示原因，missing不補裝。沒有有效版本 SHALL 回missing_data且不輸出空包；已選版本讀取／指紋驗證失敗 SHALL 整次失敗，不少包冒稱全部。結果 SHALL 為{path:string,formatVersion:number,bytes:number,packages:[{standardId:string,version:string,sha256:string}],skipped:[{standardId:string,version:string,reason:string}]}。

匯出 SHALL 保留完整同代資料，寫同目錄暫存、完成並sync後無覆寫原子發佈。已存在回file_exists；取消或寫入／容量失敗 SHALL 清理暫存、不覆寫既有目的檔、不留完成包；不支援原子無覆寫發佈的檔案系統 SHALL 明確失敗。SHALL 提供進度及取消，不在UI主執行緒組包。

#### Scenario: 單一ASVS版本離線交換
- **WHEN** 在離線電腦匯出已安裝的ASVS 5.0.0，再於另一台離線電腦匯入
- **THEN** 得到可依既有v1匯入的ZIP，另一台可查詢相同版本，來源授權與raw指紋一致，匯入來源標imported；原電腦current與專案檔案不變

#### Scenario: 全部選取與無資料
- **WHEN** 本機有兩個有效已安裝版本及一個invalid版本而選全部匯出，或完全沒有有效版本
- **THEN** 前者包內精確含兩個版本，invalid逐項顯示在skipped；後者顯示missing_data，不輸出空包、不發網路請求

#### Scenario: 取消及目的檔保留
- **WHEN** 使用者取消儲存、取消組包，或目的檔已存在／磁碟不足
- **THEN** 不顯示匯出成功，原檔及已安裝資料保持，暫存清理，錯誤或取消明示

<!-- @trace
source: desktop-colors-network-and-offline-bundles
updated: 2026-10-19T23:00:19+08:00
-->

---
### Requirement: 離線集合包匯入與逐版本結果

全部匯出 SHALL 為集合ZIP，根manifest.json為{formatVersion:2,packages:[{standardId:string,version:string,name:string,sha256:string}]}，name精確為packages/<standardId>/<version>.zip，每個內包為v1。SHALL 最多32個內包、33個outer entries、64KiB manifest、outer及內包總bytes最多512MiB；每個v1維持64MiB壓縮／256MiB展開／128entries限制，全部內包raw累計最多1GiB。SHALL 按實際讀取核對，拒絕重複版本、遞迴集合、未列entry、指紋不符、encrypted、ZIP64、連結、可執行項、路徑穿越及大小寫碰撞，不任意extract。

既有匯入入口 SHALL 自動辨識v1或v2。v2 SHALL 預檢所有包的格式、SHA、來源、授權及可解析性，通過前不得提交任一版本；通過後依manifest順序逐版本原子提交。任一提交失敗或取消 SHALL 停止，已成功版本保留、當前失敗版本保舊、後續notAttempted；清理警告不撤銷已提交。SHALL 明確呈現逐項結果，不能用單一成功訊息掩蓋partial。所有匯入來源皆標imported，不因匯出機器原本下載來源而升級信任。既有單包輸出與保舊行為不變。

#### Scenario: 全部匯出一次匯入
- **WHEN** 使用者將含OWASP Top 10 2025及ASVS 5.0.0的集合包帶到無網路裝置匯入
- **THEN** 同一入口安裝兩版本並列逐項成功，資料皆可離線查詢、來源均imported，未更動專案政策

#### Scenario: 內包損壞預檢不寫入
- **WHEN** 第二個內包SHA錯誤、缺授權或含../路徑
- **THEN** 整包預檢失敗，第一個版本也沒有提交，兩版本既有current與資料不變

#### Scenario: 中途提交失敗與取消
- **WHEN** 三包預檢通過、第一包完成、第二包遇權限錯誤或取消
- **THEN** 第一包保留成功，第二包舊current保持，第三包未執行，頁面顯示partial或cancelled及逐項結果

<!-- @trace
source: desktop-colors-network-and-offline-bundles
updated: 2026-10-19T23:00:19+08:00
-->

---
### Requirement: 離線資料包CLI匯出與集合匯入

CLI SHALL 提供security data export [STANDARD --version VERSION | --all] --output PATH [--json]，STANDARD與--all互斥、--version只搭STANDARD、--output必要，無stdin，支援global --no-color。成功exit0、操作失敗1、取消130、參數錯誤2；人眼stdout列路徑／版本／跳過項，JSON stdout為「已安裝標準資料的離線匯出」規定的單物件；進度與錯誤至stderr，失敗無成功payload。--no-color SHALL 不含ANSI，不改JSON欄位。

security data import PATH [--json] SHALL 維持v1人眼及JSON位元級輸出；v2在提交開始後回{formatVersion:2,status:"installed"|"partial"|"failed"|"cancelled",results:[{standardId:string,version:string,status:"installed"|"failed"|"notAttempted",sha256:string|null,errorCode:string|null}]}。JSON結果至stdout，人眼stdout列同資訊；全部完成exit0、提交失敗exit1、取消exit130，進度與錯誤至stderr；預檢失敗stdout空且exit1，預檢取消stdout空且exit130。SHALL 不接受stdin、不修改專案文件。新CLI人眼文案維持英文，JSON鍵camelCase不翻譯；tw／ja／en或未設政策及中文弱語言偵測行為不變。Remote專案離線或認證失效不妨礙這些裝置資料操作，不發revision請求。

#### Scenario: 人眼與JSON單包匯出
- **WHEN** 離線執行security data export owasp_asvs --version 5.0.0 --output /tmp/asvs.zip --json --no-color，或省略--json使用另一個未存在路徑
- **THEN** exit0，前者stdout為含path／formatVersion／bytes／packages／skipped且型別正確的單JSON、無ANSI與進度，後者stdout列相同資訊；進度只在stderr

#### Scenario: CLI參數及目的檔錯誤
- **WHEN** 同時指定STANDARD與--all，或匯出到已存在目的檔
- **THEN** 前者exit2、後者exit1且stderr指出file_exists，stdout沒有成功payload，目的檔保持原內容

#### Scenario: 集合匯入部分成功JSON
- **WHEN** 以--json匯入三包，預檢通過後第一包成功、第二包寫入失敗
- **THEN** exit1，stdout單JSON的status為partial、results依序installed／failed／notAttempted，sha256成功項為string而其餘為null，錯誤項errorCode為string；stderr呈錯誤，無ANSI或混入進度

<!-- @trace
source: desktop-colors-network-and-offline-bundles
updated: 2026-10-19T23:00:19+08:00
-->

---
### Requirement: 深度審查完整離線資料包

使用者明確下載／重新下載選定標準時 SHALL 一併安裝該版本原文、可信版本化對照、適用的修復指南及授權來源文件。內建 spec_registry.json SHALL 以 standardCatalog 固定引用 sources.json 定義的標準版本，resources 僅接受 role=guidance 並附適用標準、固定revision、相對路徑、sha256及授權；CWE 對照由已安裝標準快照推導，不列為 registry resource。不以浮動master／latest、虛構LTS版本或參考清單的installed欄位冒充已驗證資料。guidance SHALL 只作修正依據，不擴充專案檢核標準；關閉CWE時不提供其字典或對照引用。下載／匯入／匯出 SHALL 沿既有同代generation、大小、路徑、ZIP/XML、來源、取消及失敗保舊限制，無必要指南則knowledgeStatus=missing而非ready。舊v1包 SHALL 仍可供基本掃描，缺指南只影響深度審查；新v1與v2集合匯出 SHALL 保留完整同代原始補充檔，不只導出連結。

必要指南 SHALL 涵蓋認證、授權、機密儲存、日誌、TLS、REST、SQL／XSS／命令注入、反序列化、XXE、SSRF、原型鏈、Webhook及拒絕服務，語言指南只採官方實際存在且核准的檔案；Python／Rust SHALL 使用適用通用指南，不假造語言檔案。官方不存在、版本不符、授權或檔案缺失 SHALL 拒絕標為完整並保留舊包。

#### Scenario: 下載後斷網與交換完整包

- **WHEN** 安裝選定Top 10 2025的完整資料包，斷網後匯出再於另一台匯入
- **THEN** 兩台均可讀相同原文及指南並查SHA／來源；沒有網路請求、沒有安裝2021或ASVS，匯入來源仍標imported

#### Scenario: 浮動來源與舊包缺件

- **WHEN** 新清單把master標為2024-LTS或指定不存在指南，或載入缺指南的合法舊包
- **THEN** 前者不能成為ready資料且舊代不變，後者仍可基本掃描但深度檢查明示missing_knowledge，不臨時下載

<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->

---
### Requirement: 本機知識庫準備命令

CLI SHALL 提供security kb prepare [--root PATH] [--change CHANGE] [--json]，不讀stdin，全域--no-color可用；只讀已安裝選定版本，將完整快照保存到授權checkout的.security_kb，不送網路或原始碼。Remote SHALL 依 --change 讀取對應已準備快照；省略時才使用 project scope。JSON stdout SHALL 為{projectKey:string,path:string,policyDigest:string,files:number,bytes:number,standards:[{standardId:string,version:string,dataSha256:string}],knowledgeDigest:string}，人眼列同資訊；成功0、缺件／目標／I/O失敗stderr及1、取消130、參數錯誤2，進度只至stderr且JSON無ANSI。

快照 SHALL 含manifest、standards、mappings、cheatsheets及來源授權；manifest綁projectKey、政策、每檔SHA及資料代次，僅複製已選標準與指南。準備 SHALL 檢查暫存完整性後替換，失敗保舊，不覆蓋無本工具manifest的使用者目錄或連結；生成快照gitignored。Remote需先明確準備政策／規格，缺快照／checkout SHALL 失敗且不自動連server或下載。審查開始前 SHALL 驗manifest與本機檔案，不跟隨文件內網址。

#### Scenario: 本機準備及人眼JSON

- **WHEN** 有完整已安裝資料與授權checkout，依序執行prepare --json --no-color及人眼prepare
- **THEN** exit0，輸出欄位型別正確且指同一完整快照，未選版本未出現在manifest，沒有ANSI或網路請求

#### Scenario: 同名目錄或必要檔被改

- **WHEN** .security_kb是使用者既有目錄、symlink，或資料SHA不符／缺必要指南
- **THEN** exit1與stderr原因，不覆蓋目錄、不讀根外目標、不破壞先前完整快照

<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->