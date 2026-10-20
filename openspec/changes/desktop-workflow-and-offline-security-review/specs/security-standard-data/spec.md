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
