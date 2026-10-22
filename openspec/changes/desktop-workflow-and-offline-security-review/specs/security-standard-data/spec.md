## MODIFIED Requirements

### Requirement: 應用程式資料目錄與離線版本清單
<!-- BEFORE: 標準資料固定在應用程式資料目錄的 security/standards，沒有自訂本機或 UNC 標準根目錄。 -->

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

## ADDED Requirements

### Requirement: 深度審查完整離線資料包

使用者明確下載／重新下載選定標準時 SHALL 一併安裝該版本原文、可信版本化對照、適用的修復指南及授權來源文件。核准registry SHALL 區分standard／mapping／guidance，每檔有固定revision、相對路徑及sha256，不以浮動master／latest、虛構LTS版本或參考清單的installed欄位冒充已驗證資料。guidance SHALL 只作修正依據，不擴充專案檢核標準；關閉CWE時不提供其字典或對照引用。下載／匯入／匯出 SHALL 沿既有同代generation、大小、路徑、ZIP/XML、來源、取消及失敗保舊限制，無必要指南則knowledgeStatus=missing而非ready。舊v1包 SHALL 仍可供基本掃描，缺指南只影響深度審查；新v1與v2集合匯出 SHALL 保留完整同代原始補充檔，不只導出連結。

必要指南 SHALL 涵蓋認證、授權、機密儲存、日誌、TLS、REST、SQL／XSS／命令注入、反序列化、XXE、SSRF、原型鏈、Webhook及拒絕服務，語言指南只採官方實際存在且核准的檔案；Python／Rust SHALL 使用適用通用指南，不假造語言檔案。官方不存在、版本不符、授權或檔案缺失 SHALL 拒絕標為完整並保留舊包。

#### Scenario: 下載後斷網與交換完整包

- **WHEN** 安裝選定Top 10 2025的完整資料包，斷網後匯出再於另一台匯入
- **THEN** 兩台均可讀相同原文及指南並查SHA／來源；沒有網路請求、沒有安裝2021或ASVS，匯入來源仍標imported

#### Scenario: 浮動來源與舊包缺件

- **WHEN** 新清單把master標為2024-LTS或指定不存在指南，或載入缺指南的合法舊包
- **THEN** 前者不能成為ready資料且舊代不變，後者仍可基本掃描但深度檢查明示missing_knowledge，不臨時下載

### Requirement: 本機知識庫準備命令

CLI SHALL 提供security kb prepare [--root PATH] [--json]，不讀stdin，全域--no-color可用；只讀已安裝選定版本，將完整快照保存到授權checkout的.security_kb，不送網路或原始碼。JSON stdout SHALL 為{projectKey:string,path:string,policyDigest:string,files:number,bytes:number,standards:[{standardId:string,version:string,dataSha256:string}],knowledgeDigest:string}，人眼列同資訊；成功0、缺件／目標／I/O失敗stderr及1、取消130、參數錯誤2，進度只至stderr且JSON無ANSI。

快照 SHALL 含manifest、standards、mappings、cheatsheets及來源授權；manifest綁projectKey、政策、每檔SHA及資料代次，僅複製已選標準與指南。準備 SHALL 檢查暫存完整性後替換，失敗保舊，不覆蓋無本工具manifest的使用者目錄或連結；生成快照gitignored。Remote需先明確準備政策／規格，缺快照／checkout SHALL 失敗且不自動連server或下載。審查開始前 SHALL 驗manifest與本機檔案，不跟隨文件內網址。

#### Scenario: 本機準備及人眼JSON

- **WHEN** 有完整已安裝資料與授權checkout，依序執行prepare --json --no-color及人眼prepare
- **THEN** exit0，輸出欄位型別正確且指同一完整快照，未選版本未出現在manifest，沒有ANSI或網路請求

#### Scenario: 同名目錄或必要檔被改

- **WHEN** .security_kb是使用者既有目錄、symlink，或資料SHA不符／缺必要指南
- **THEN** exit1與stderr原因，不覆蓋目錄、不讀根外目標、不破壞先前完整快照
