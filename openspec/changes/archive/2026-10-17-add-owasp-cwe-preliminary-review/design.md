## Context

既有 core 的 WorkflowConfig 與 update_workflow_config_text 負責政策解析與局部文字改寫；Host 注入有效政策，Remote 的 config 文件以 editor 權限及 expected revision 寫入。Desktop 的 WorkflowPage 在產出流程頁呈現政策，audit 是生成的 AI sharp-edges 指引，沒有離線規則執行器。Verify 工單及章有封存守門，因此安全報告不能混入必修項目。

使用者已確認單一完整變更、七階段開發。雖超過十五個任務，政策、資料、規則與報告共同組成同一驗收旅程，保持一個 change；以約一小時內的小任務交接，不切掉任何已承諾能力。程式碼根目錄的未知欄位與使用者未提交檔案均是保護對象。

## Goals / Non-Goals

In scope：主開關及獨立標準、離線基本規則、三種審查、資料下載／匯入／管理、遮罩報告、Local／Remote 政策一致、Desktop／CLI、提示性 SDD 整合、測試與維護文件。

Out of scope：第三方掃描引擎、執行受檢程式、主動呼叫受檢 API、依賴弱點資料庫的 SCA、完整資料流／跨函式分析、認證或全 ASVS 通過判定、Security Gate、遠端主機原始碼掃描、新 AI runtime、無關重構。可重用 audit 的觀點與既有流程，不把 AI 當必需能力。

## Decisions

### 政策保值與既有設定介面

在 crates/engine/speclink-core/src/security/policy.rs 定義 SecurityPolicy，WorkflowConfig 的 security_verification 缺席或 null 解讀為預設；新增區塊採 snake_case YAML，對外 JSON 使用顯式 camelCase serde 名稱。已知欄位型別／enum 不合法即 invalid_config，既有未知頂層鍵與安全區塊未知鍵仍可載入，保留其原始文字；不把未知鍵當作已執行的功能。舊版引擎忽略此新區塊並不代表具有審查能力，文件須提示升級。

預設 enabled=false、scope=off、四個標準 enabled=false、ASVS level=L1、reviews.spec/code/api_boundary=true、disabled_rules=[]、exclude_dirs=[]。三個 reviews 是自有規則的獨立啟用選擇；全部標準關閉時仍可執行明確勾選的自有規則，標準化部分標示未配置。關閉主開關保留既有選項，日後開啟可還原；不藉移除整塊設定丟掉未知內容。完全預設且無未知內容時可省略區塊；含未知內容時仍保留。

每個標準是 enabled 與 version，ASVS 加 level；啟用時版本必填，Top 10／API／ASVS 僅接受提案列明版本。CWE enabled 時接受合法版本字串，是否已安裝在執行前判斷，不在 Remote 儲存時要求該 server 有安裝。停用標準保留上次 version／level，但不影響其他標準。

CLI 的 workflow-config security --stdin 讀取至多 64 KiB 的 YAML 部分更新 mapping：空輸入／非物件拒絕，不用空白表示刪除；enabled:false 只停用主開關。僅允許已知可編輯葉節點，未知輸入欄位拒絕，但不刪除原文件的未知欄位。core 逐葉規劃局部文字改寫，經 yaml_scalar 處理純量、再解析核對整份目標 mapping；不得整塊重序列化並丟失未編輯鍵。複雜 YAML alias、merge key 或重複鍵無法保證局部更新時拒寫原文不動，讀取現有合法文件仍維持原有契約。--dry-run 使用既有逐行 diff。五鍵 workflow-config set 不變，不新增環境覆寫，避免嵌套政策再開一套優先序。

Desktop 在 audit 下方加主開關與可隱藏子列，沿用 SettingsCard／SettingsRow／SettingsChoice／Switch／Input／Button／ConfirmDialog／SEMANTIC_TONE／useI18n。label、完整說明及範圍解說使用 messages；介面 zh-TW／en，Agent 文件依既有 tw／ja／en 政策。停用時不發出掃描 IPC。安全設定用相同 core 部分更新接縫，Remote 保留現有 revision 衝突對照、reader 唯讀、離線不排隊寫入；worktree 的 Remote 隱藏維持。


儲存的完整例（引號保證版本是字串，YAML 與 JSON 的鍵名分開）：

    security_verification:
      enabled: true
      scope: web-api
      standards:
        owasp_top10:
          enabled: true
          version: "2025"
        owasp_api_top10:
          enabled: false
          version: null
        owasp_asvs:
          enabled: false
          version: null
          level: L1
        cwe:
          enabled: false
          version: null
      reviews:
        spec: true
        code: true
        api_boundary: true
      disabled_rules: []
      exclude_dirs: []

對外 securityVerification 的四標準鍵為 owaspTop10／owaspApiTop10／owaspAsvs／cwe，reviews.api_boundary 對應 apiBoundary，其餘名稱按契約明列。CWE version 僅接受一到三段、每段一到三位數字的點分字串，不能包含斜線、盤符或點點；未安裝的合法版本可儲存，執行再顯示缺資料。設定 version 數值而非字串時拒絕，不自動猜年份或版本。

### 標準版本資料與授權分離

sources.json 只包含自有清單與來源 metadata，不內嵌官方文件。初始來源族固定為 OWASP/Top10、OWASP/API-Security、OWASP/ASVS、cwe.mitre.org；只採官方英文機器可讀原文，不自行翻譯官方要求。標準標題可用 UI 自有譯文，官方要求原文保留其授權。ID 永遠與 standardId／version 組合，CWE 版本來自已安裝資料。

已核實來源如下，正式網址及 attribution 隨每次安裝記錄；清單中的固定 commit 不是樹或 blob SHA：

| 標準 | 固定來源與解析 |
| --- | --- |
| Top 10 2021／2025 | OWASP/Top10 commit 3a31f35346c3f4e90f350395124382fa53af2afe，2021/docs/en 與 2025/docs/en 的 A01 到 A10 Markdown 及 LICENSE；只取已列明十個分類檔。用標題及 CWE 清單解析，不解析 HTML 畫面。 |
| API Top 10 2019／2023 | OWASP/API-Security commit 33cea37b2ffde3e2e62f6b5d79a029a22a846633，editions/2019/en 與 editions/2023/en 的 0xa1 到 0xaa 風險 Markdown 及 LICENSE；初始清單逐檔核對名稱與標題 API 編號。 |
| ASVS 4.0.3 | https://github.com/OWASP/ASVS/releases/download/v4.0.3_release/OWASP.Application.Security.Verification.Standard.4.0.3-en.json 。根 Version 必須等於 4.0.3；遞迴 Requirements／Items，葉節點 Shortcode、Description 及 L1／L2／L3.Required 逐級讀取。 |
| ASVS 5.0.0 | https://github.com/OWASP/ASVS/releases/download/v5.0.0_release/OWASP_Application_Security_Verification_Standard_5.0.0_en.json 。根 Version 必須等於 5.0.0；葉節點 L 的 1／2／3 為最低等級，選 L2 包含 L1、L2。不能套 4.0.3 的欄位。 |
| CWE | MITRE https://cwe.mitre.org/data/archive.html 的官方版本族／https://cwe.mitre.org/data/downloads.html 公布的固定版 cwec XML ZIP；以 XML 根 Weakness_Catalog 的 Version 核實，串流讀 Weaknesses／Relationships／Description。初始已核對官方清單列出的 4.20，實作固定到 cwec_v4.20.xml.zip，並測試根版本。 |

官方查核：https://github.com/OWASP/Top10 、https://github.com/OWASP/API-Security 、https://github.com/OWASP/ASVS/releases 、https://cwe.mitre.org/about/termsofuse.html 。GitHub release 的 latest 是持續更新預覽，不列為 5.0.0 固定版。舊正式 ASVS 資產的 API digest 為 null，不能拿 latest 的 digest 驗證它們；首次官方 HTTPS 安裝計算內容指紋，標示 integrityKind=source-tls，之後以該指紋核對保存內容。清單附官方 digest 或維護者經查核的 pinned digest 才標示相對應驗證種類，不能把自行計算 SHA 說成官方驗證。

Top10 pinned repository LICENSE 為 CC BY-SA 4.0，但網站頁尾有 CC BY 3.0 文字；以實際 repository 資產適用的 LICENSE 保存其完整聲明與各檔通知，採不隨應用程式重新散布原文的使用者下載方案，不替資料重授權。逐版查核修正：ASVS 4.0.3 release commit d31389d213b320cc2ef0cb1c30792cc80104d8ff 的 LICENSE.md 指向 CC BY-SA 3.0（https://github.com/OWASP/ASVS/blob/d31389d213b320cc2ef0cb1c30792cc80104d8ff/LICENSE.md）；ASVS 5.0.0 release commit 5cf9b032440be53ce345ab3c130fda46ba1ce7a2 的 LICENSE.md 為 CC BY-SA 4.0，API 固定 commit LICENSE 亦為 CC BY-SA 4.0。不得把新版授權套用到舊版。CWE 保留 MITRE copyright、terms 與免責文字。parsed.json 同目錄保留來源及修改說明，資料衍生物不宣稱 MIT。匯出報告附所引用資料的 attribution 與授權連結，不把原文重製混為自有程式；不附無關標準全文。

refresh 只查官方 GitHub 結構化 release／contents API 與已核准 CWE 固定版本候選，不能靠 HTML 版面猜版本；未知格式／新主版列為 unavailable，不提供下載，待來源清單與解析器更新。支援清單更新不等於可下載任意版本：Top 10／API／ASVS 的本次支援矩陣固定，CWE 的新候選須符合官方版本化路徑與既有 XML schema。更新來源不執行遠端程式或載入遠端規則。

### 原子安裝、離線包與取消

Host 的 security_data.rs 管來源與安裝工作，speclink-fs 的 security.rs 做有界檔案 I/O；core 的 standards.rs 只解析 bytes、驗證內容並產生標準模型。新的 context::security_data_dir 必須回絕對應用程式資料路徑：macOS Application Support/speclink、Windows LOCALAPPDATA/speclink、Linux XDG_DATA_HOME/speclink 或 HOME/.local/share/speclink，portable 使用既有 data/speclink。缺少可用絕對路徑時回明確錯誤，不沿用 global_config_dir 的點目錄 fallback。Desktop 與 CLI 用同一解析函式。

資料根下 security/standards/<standard-id>/<version>/ 只放原子指標 current.json 與 immutable generation 目錄；generation 內含原始檔、parsed.json、metadata.json、LICENSE。metadata 保留需求的十一欄，另加 generation、sourceRevision、integrityKind、parserVersion 與各檔指紋。current.json 一次指向完整 generation，不直接逐檔覆蓋既有版。app 根不得接受外部輸入路徑；standard-id enum、version 白名單與 ULID generation 都由可信模型組裝。

安裝順序：開啟每版本鎖檔並以 std::fs::File::try_lock 取得跨行程排他鎖（本機 Rust 文件已核實，1.89 起提供）；鎖隨 handle drop／程序退出釋放 → 寫同磁碟 staging/<job-id> → 限量下載／讀包 → 指紋、版本、格式、ID 唯一性與授權驗證 → 建完整 generation 並 flush → 原子替換 current.json → 成功後清暫存與舊未引用 generation。指標提交前失敗舊版不動；指標提交後清理失敗只回成功附 cleanup 警告，不能謊稱未安裝。Unix 同目錄 rename 提交指標；Windows 已有指標用 windows-sys 0.61.2 的 ReplaceFileW，初次無指標用同磁碟 rename，禁止先刪舊指標再 rename；無法原子替換就拒絕提交。讀取端先讀 current 快照再讀同代檔案並驗指紋，不會混代。崩潰殘留的 staging 不對讀取可見，下一次操作清理逾 24 小時且無活鎖的暫存。

limits：單次下載 64 MiB、解壓總量 256 MiB、最多 128 entries、單一 XML 128 MiB、其他檔案 2 MiB、標準項目最多 100,000、XML 深度 64、單次工作 120 秒。讀取 socket timeout 1 秒，總 timeout 120 秒，取消檢查每個 chunk／entry／XML event；取消後至多 2 秒停止且不提交指標。拒絕 HTTP、TLS 降級、非白名單主機、私有／回環位址與未核准 redirect；GitHub release redirect 只准其官方 asset host，最多三跳。套件大小用實際讀取量核對，不只信 Content-Length。

匯入格式為 ZIP：根 manifest.json（formatVersion=1、standardId、version、sourceRevision、來源與授權 metadata、entries 含 name／sha256），檔案僅 raw/ 下已知標準格式與根 LICENSE；不接受 executable、symlink、絕對路徑、反斜線、盤符、空段／點段、重複／大小寫碰撞與未列入 manifest 的 entry。先檢查全部 entry 再讀至受控 staging，不按包內任意路徑直接 extract。XML 拒絕 DTD、外部實體及非內建 entity，禁止網路解析；ZIP 只開 store／deflate，不攜入其他壓縮 codec。離線包匯入來源標為 imported，不能只靠包內自填 hash 升格為官方驗證。

實際來源格式修正：MITRE 固定 cwec_v4.20.xml.zip 的唯一 cwec_v4.20.xml 為 regular file、Unix mode 100750。官方 CWE 封裝的專用解碼路徑僅接受完全符合所選版本名稱的單一 regular XML，忽略其執行權限位元但從不套用封裝權限；只以有界 bytes 交給 CWE XML parser，通過根版本／結構驗證後按 SpecLink 自訂權限保存。外層離線包仍拒絕帶執行權限項目，symlink／特殊檔在所有路徑均拒絕。此例外不允許命令或程式執行，也不提供通用extract。

進度統一 JobProgress={jobId, phase, completedBytes, totalBytes:null|number, completedFiles, message}，不得包含原始碼／秘密。Desktop core 持有執行緒與 Arc<AtomicBool>、每次工作固定專案身分；Tauri command 單行委派，不在 async event loop 做掃描。每個程序最多一個掃描及一個資料安裝，第二個請求回 busy。跨行程安裝／刪除／掃描 pin 用同版本鎖及引用快照協調，不能靠 UI 停用當互斥。

刪除只在沒有執行中 pin、沒有本機已知專案／準備快照引用該版時允許；同版鎖內先建立刪除 tombstone，更新索引再清檔，失敗顯示可重試且不能顯示仍可用。不聲稱能知道未開啟的遠端專案政策：UI 說明檢查範圍為本機已知引用；日後遇被刪資料的專案顯示缺失，絕不改寫政策。報告自帶當時資料 metadata，不需 pin 原文永久阻擋刪除。

### 離線目標與 Remote 準備快照

security run 是無網路階段。CLI 的 run／report／rules／data 經 ModeFree 入口，不經既有 dual 的 Remote handshake／自動連線；只讀本機安全 binding 身分並選對準備快照，prepare 才建立 remote client。Local 從授權根讀當次政策與 SDD；Remote 必須先由獨立 security prepare 明確操作，透過既有 remote client 取 config 原文／revision 與所選 change 的 proposal／design／specs／tasks，驗證後原子保存至 security/prepared/<project-key>/<change-or-project>/。只保存政策與規格，不上傳或下載原始碼；不修改 Context Projection。快照記錄 origin、project／repo、文件 digest／revision、preparedAt 與明確選取的 canonical checkout，不能被另一個 origin／root 冒用。離線只能說是此準備版本，無法知道遠端最新狀態，報告顯示此限制。準備斷線／401／衝突不替換舊快照。

run 的 --root PATH 與 Desktop 選取資料夾建立當次授權；Local 預設開啟專案根。Remote 未提供 checkout 時產報告含 source unavailable／NOT_VERIFIED，不掃描快取、server 目錄或最近開啟的另一個根；無完整準備快照時亦不連線補抓。切換 Desktop 專案後進行中的工作仍綁原專案，事件以 jobId／projectKey 過濾。

git 只用可信 git 執行檔、固定參數讀 rev-parse --verify HEAD，timeout 2 秒，清除會改變 git 行為的環境變數，禁 hooks／fsmonitor／pager／external diff；不得執行專案 shell。取得不了 SHA 記 null，dirty 檔仍是實際掃描內容，每檔記 contentDigest，不能把 HEAD 當成全部原始碼版本。所有路徑顯示轉為根相對 slash；Windows 大小寫／UNC 與 macOS 大小寫碰撞納入測試。

### 有界掃描與失敗覆蓋

預設排除 .git、node_modules、target、dist、build、.speclink/context、應用程式自有生成技能／golden 的既有生成標記檔及二進位。exclude_dirs 只接受根相對目錄，拒絕絕對、點點、盤符與空字串；比對正規化後路徑，不呼叫 git ls-files 或專案工具。忽略規則不讀可執行設定，不執行生成物判斷指令。

第一版不跟隨任何 symlink／Windows reparse point：跨根者標 untrusted_link，根內者標 skipped_link；均記錄未檢查，避免競態開啟時逸出。Unix 從已授權 root 的 directory fd 逐段 openat(O_NOFOLLOW|O_CLOEXEC)，中間目錄加 O_DIRECTORY 並由 fstat 核實一般檔案；Windows 使用 CreateFileW 的 FILE_FLAG_OPEN_REPARSE_POINT／目錄 FILE_FLAG_BACKUP_SEMANTICS，檢查 reparse 屬性及 GetFinalPathNameByHandleW 的最終路徑仍在 canonical root。政策與 SDD 的本機讀取也走同一安全入口，不先由舊 read_opt 跟隨連結取值。以 handle metadata 核實一般檔案、canonical 授權根與讀取身分；若平台無法提供此保證則拒讀該檔，不用先檢查再不受保護開啟。FIFO／device／socket 拒讀。不支援語言或無法判斷是否二進位保留 NOT_VERIFIED，不算已檢查。

limits：單檔 1 MiB、總讀取 100 MiB、最多 10,000 檔、深度 32、掃描 60 秒、每檔最多 100 findings、總 findings 10,000。單檔／深度超限跳過該項並記原因；全域上限／逾時／取消停止並保留未處理 frontier，complete=false。檢查期限與取消在每個目錄、每 64 KiB chunk、每條規則前後執行，至多 2 秒回應。所有已讀 text 都有已執行／停用／不適用／未驗證的規則矩陣，跳過數量與失敗原因保存在 coverage；排除與錯誤不能算成功檢查。

### 可測基本規則與標準對應

rules.json 是 SpecLink 自有版本化資料，check.mode=pattern、check.selector 為編譯期已知檢查，不提供任意 code、命令或外部規則下載。啟動時驗證 ID 唯一、合法 severity、語言、selector、references、remediation；regex 以 Rust 既有 regex 編譯一次，在最多二十行／8 KiB 的視窗內執行，不做跨函式資料流或專案自行提供的 regex。

| ID | 行為、適用語言與排除證據 |
| --- | --- |
| SL-SEC-001 | Rust／TS／JS 的 password／secret／token／api_key 類名稱賦予非空字串字面值；env 讀取與明確 EXAMPLE／REDACTED／changeme placeholder 不命中。輸出整筆值遮罩，confidence=low。 |
| SL-SEC-002 | Rust 的 format!／加號或 TS／JS template／加號將變數插入 SQL 後交給 query／execute；固定字串配獨立參數陣列不命中，confidence=low。 |
| SL-SEC-003 | TS／JS innerHTML／outerHTML／document.write／dangerouslySetInnerHTML 接非固定值；純 textContent 不命中。Rust 明確 HTML response 接格式化輸入標疑似；上下文無法判定不判安全。 |
| SL-SEC-004 | Rust sh -c／cmd /C 與 TS／JS exec 的命令字串有插值；Rust Command.arg 及 JS execFile 的參數陣列不命中。 |
| SL-SEC-005 | Rust response body 直接帶 debug error，TS／JS HTTP response 直接帶 error.stack／完整 error；單獨 server log 不命中。 |
| SL-SEC-006 | Rust／TS／JS 與 JSON／YAML 設定中 rejectUnauthorized:false、verify_ssl:false、insecure:true 與 httpOnly:false 等已列危險字面值；安全相反值不命中，其他鍵不憑字義猜。 |

SL-SPEC-001 至 006 分別檢查所選 change 中 API 認證、敏感操作授權、型別／長度、敏感資料、錯誤處理、安全驗收的可見敘述，支援 tw／en／ja 的明列詞組；需有 API／敏感操作的正向目標證據才提出缺漏，沒有目標標 NOT_VERIFIED 或有理由的 NOT_APPLICABLE。這些是 missing_requirement／needs_review，不叫 confirmed_vulnerability。

SL-API-001 至 007 分別覆蓋空值、型別、長度、數值範圍、非法參數、授權、錯誤輸出。第一版讀 OpenAPI 3.0／3.1 JSON／YAML 的本機文件（檔案指示 openapi 版本），只解析內部 $ref，外部引用不讀；缺 schema／constraints 是需求缺口，無法看出 source 保護是未驗證。Rust／TS／JS 的來源模式只提供正向可觀察危險證據，不靠找不到驗證函式判漏洞。沒有 OpenAPI／可識別邊界時對七項保留未驗證，仍可掃其他程式規則。

自有規則帶 CWE ID 但不要求安裝 CWE 才能運作；Top 10／API 的對應從所選版官方分類檔內 CWE 參考建立，保留來源檔與段落，只有明確出現在官方對應列表的 ID 才關聯，不能把敘事引用算對應。ASVS 逐項展開選定等級；只有有來源且可獨立測試的直接規則對應才執行，其餘要求保留 NOT_VERIFIED。不因自己推論控制項號而加 mapping。CWE 用於分類／補充說明，沒有測試器的條目不是 PASS。規則未命中僅其 rule result=PASS，不能把分類或整套 ASVS 蓋成通過；finding 與未驗證覆蓋同時保留。

### 遮罩報告與 Desktop 共用呈現

core 的 report.rs 為唯一 JSON／Markdown 模型與渲染來源；所有 evidence、錯誤、進度在序列化或事件前遮罩，SL-SEC-001 不保留原始字面值，其他行證據替換全部字串字面值、URL 憑證與 token-like 值，最多 400 字元。無法安全切割則整段標 [REDACTED]；原始檔內容只在掃描的短期記憶體使用，不能寫 debug log。路徑與使用者字串清除終端控制字元，Markdown evidence 以安全編碼純文字呈現，UI 不渲染來源 HTML 或啟用來源連結指令。

報告格式 formatVersion=1。Report={reportId, projectKey, projectName, changeName:null|string, reviewedAt, commitSha:null|string, policyDigest, policyRevision:null|number, preparedAt:null|string, standards[], ruleVersion, scope, executionMode:"builtin-offline", reviewMethods[], complete:boolean, configurationState:"disabled"|"unconfigured"|"configured", counts, findings[], checks[], coverage, disclaimer, sourceLicenses[]}。欄位皆 camelCase，數字計數不以字串表示；每個 standard 包含 standardId、version、level:null|string、dataSha256、sourceRevision、installedStatus。

Finding={findingId,ruleId,title,severity:"low"|"medium"|"high"|"critical",confidence:"low"|"medium"|"high",path,line,evidence,description,remediation,references[],confirmationStatus:"needs_review",kind:"potential_risk"|"missing_requirement"}。人工確認不在本次新增編輯狀態；不將自動結果預設為已確認漏洞。references 含 standardId／version／controlId／sourceUrl。Check={ruleId,target,status,reason,relatedStandardIds[]}，status 為五種指定值；coverage 含 inspectedFiles、skippedFiles、failedItems、bytesRead、remainingTargets、reasons[]；counts 含 checksSucceeded、checksFailed、findings、notVerified、notApplicable、disabled，不將 FINDING 算作工具失敗。

security/reports/<project-key>/<report-id>/ 寫 JSON、Markdown 完整 staging 後原子顯露，最後更新可重建的 latest.json；latest 寫失敗仍可依 reportId 找到已完成報告，回成功附警告，不毀前一份索引。索引損壞可依完整報告重建；不自動刪舊報告，整體上限 1,000 份／200 MiB，達上限回 storage_limit 且不寫半套。projectKey 是 canonical root 與 binding origin／scope 的 SHA256，不在 reportId 夾帶任意路徑；report 查詢只收 ULID，不能讀任意檔案。不寫 verify.md、review.md 或章 metadata；封存後報告仍可依 ID 開啟，不把它當規格文件保存在 TeamStore。

Desktop 共用 SecurityReviewPanel 放 packages/ui，沿用 SettingsCard／Button／Badge／Collapsible 與現有 Markdown 純文字閱讀原語；資料操作接可選的 adapter 能力，未接入的 server-web 不假裝能掃本機。ChangeDetailPage 在 Verify 區呈現安全資訊，即使沒有 verify 工單也可看／執行；TicketTabBody 只在真工單存在時顯示，不為顯示安全報告開工單。明確的「執行基本審查」、取消、缺資料前往系統設定、JSON／Markdown 匯出與複製遮罩摘要可用；不用新全域導覽或快捷鍵機制。檔案跳轉沿用既有 reveal 能力並重新核對當前專案根。

### 提示性 SDD 整合與技能資產

securityVerification 指引物件固定含 enabled、scope、standards[{standardId,version,level}]、reviews[]、executionMode、dataAvailability、requirements[]。Engine／server 沒有裝置資料目錄，dataAvailability 使用 device_local_check_required，指示使用者以 security data list 查真實安裝狀態；不能把此值描述為已安裝。舊 server 缺欄位保留 null／unsupported_version，非安全操作維持。

instructions 的 proposal／design／tasks／apply payload 在已取得政策上新增 securityVerification（停用仍有 enabled=false 的預設物件），不從外部原文生成命令。只注入所選標準版本、實際範圍、需規劃的安全要求與標準資料可用性；不把 downloaded Markdown 原文直接嵌為高優先權指令。使用者字串當資料引用，官方內容亦視為不可信資料。

Apply 在主開關開且相關 review 啟用時可執行相同 Host 離線流程，失敗回報但不改 audit／tdd 開關；規格要求仍由原 Verify 判斷，本工具的疑似問題不自行轉成其必修項目。Verify 報告列已選標準、結果、風險、未驗證與報告連結；停用且無歷史報告時完全不加段落，不開工單、不產生安全章，不自動修 code。既有未結 verify／review 工單與 stale stamp 守門照舊。

修改 assets/skills/propose.md、apply.md、audit.md（只說明邊界）、verify.md、config.md 與 schema artifact instruction 的必要文字；保持生成工具通用性，ASSET_VERSION、四種工具渲染 golden 與 assets.lock 同批再生。AI 協助只提供使用者明確複製的遮罩摘要及既有技能指引，不啟動 Agent、不送原始碼、不稱離線 AI。LANGUAGE 補「初步安全審查」與品質關卡／審查規則的語意差異。

### 最小相依與跨平台取消

沿用 serde／serde_json／serde_yaml／regex／sha2；資料網路使用既有 ureq 2.12.1（MIT OR Apache-2.0，Rust 1.71）、XML 用 lockfile 的 quick-xml 0.41.0（MIT，Rust 1.79）、ZIP 用 4.6.1（MIT，Rust 1.82），只開必要 store／deflate feature；各型別與 feature 實作時讀本機 crate 原始碼，不能憑記憶填 API。core 加 quick-xml，fs 加 zip、serde_json、sha2；fs 的平台相依沿用 lockfile 的 libc 0.2（MIT OR Apache-2.0）與 windows-sys 0.61.2（MIT OR Apache-2.0），只開 Win32 Storage/FileSystem、Foundation、Security 所需 feature；host 加 ureq，不使 host 依賴 speclink-fs 而產生 adapter 逆向相依：Host 接收 core 定義的受限 reader／資料寫入能力，CLI／Desktop 組裝唯一 Fs 實作，非額外包裝框架。

CLI 新增 ctrlc 3.5 系列的單一 signal handler 設 AtomicBool，沒有既有取消相依可重用；官方 https://github.com/Detegr/rust-ctrlc/blob/master/Cargo.toml 列 MIT/Apache-2.0、Rust 1.69，與本 repo stable 工具鏈相容。此依賴是跨平台 Ctrl+C 取消所必需的一般 OS 函式庫，不是掃描器；解析 lockfile 新增的傳遞相依與 desktop third-party-licenses 清單納入測試及文件。安裝前先比較 resolver diff，拒絕無關框架擴張。Desktop 不引入 ctrlc handler，由取消 IPC 設同一 token。

## Implementation Contract

同一 Host run 行為供 Desktop 及 CLI 使用；Host 不擅自建立網路 client，security run 的呼叫圖沒有 HTTP 能力。資料與 prepare 是另外的明確操作。core 介面接受 SecurityPolicy、StandardSnapshot、AuthorizedInput／SourceDocument、ExecutionBudget／cancel observer，產生可序列化且已遮罩 Report；路徑根與裝置資料路徑由 Host 注入，不讓 Engine 自找 HOME 或寫死媒介。這是針對有界輸入與安裝交易的實質介面，不建立泛用安全插件層。

| 命令／IPC | 可觀察契約 |
| --- | --- |
| workflow-config security --stdin [--dry-run] | 部分 YAML 更新，共用文字手術；成功 stdout 一行摘要／diff，exit 0；錯誤 stderr，exit 1；無 --stdin exit 1，clap 語法錯誤維持 2。 |
| workflow-config show --json | 既有欄位外新增 securityVerification；人眼輸出在 review 後新增一行，其餘位元級保持。 |
| security run [--root PATH] [--change NAME] [--json] | 不讀 stdin、完全離線；stdout 單一 Report JSON 或含 reportId／計數／disclaimer 的人眼摘要；完整（含疑似風險）0，未驗證／失敗覆蓋1，取消130；主開關停用產 DISABLED／不讀來源，未配置但自有規則執行仍顯示未配置，不假稱標準通過。 |
| security prepare [--root PATH] [--change NAME] [--json] | Local 原子記當次政策／規格，Remote 明確下載其政策／規格；stdout {projectKey,preparedAt,policyRevision,documentCount,checkoutAvailable} 或摘要；成功0、錯誤1、取消130。未安裝標準不自動下載。 |
| security rules [--json] | stdout {ruleVersion,rules:[{id,title,type,severity,languages,description,references,check,remediation}]} 或人眼表；0；內建規則資料壞掉stderr與1。 |
| security report ID [--format json|markdown] | stdout 指定格式，預設 json；不讀 stdin、不重新掃描；找不到／非法ID為stderr與1，成功0。 |
| security data list／refresh／install／import／remove | list／refresh stdout {standards:[{standardId,versions:[{version,status,sourceUrl,licenseName,licenseUrl,attribution,sourceFormat}]}]}；install／import stdout {standardId,version,installedStatus,sha256,sourceRevision,integrityKind}；remove stdout {standardId,version,removed:true}。人眼顯示相同資訊。全域 --no-color 無 ANSI；成功0，錯誤1，取消130；進度只到stderr。 |
| Desktop security_start／security_cancel／security_report／security_data_*／security_prepare | command 僅委派 desktop-core，start 立即回 {jobId,projectKey}，事件 security-progress／security-completed 帶原 projectKey；completed 含 reportId 或結構化 {code,message}。取消保留部分 report、complete=false，不提交未完成資料安裝。 |

安裝／掃描的 ERROR 結構為 code、已遮罩 message、stage、target，無原始 body；missing_data、unsupported_language、budget_exceeded、cancelled、invalid_package、version_mismatch、license_missing、busy、storage_limit 均有獨立代碼。失敗前尚無授權根／可用政策時 stdout 無偽造報告，stderr 指出無法開始；開始後能保留安全模型的失敗都產 partial report。未知 change 或壞 config 不掃任意檔案。

舊 server 缺 securityVerification 時，對安全功能顯示 unsupported_version，不把未知政策預設關閉；原 tdd／audit payload 與非安全動作維持。Server 未支援新欄位時不額外開安全作業。

驗收由七份 delta 的 scenario 與 tasks 的具體測試對照；關鍵跨層測試為僅 Top 10 2025、其餘關閉、安裝後禁用網路仍可跑、JSON／Markdown 無測試秘密、Remote 401／revision 衝突不覆寫、無 AI 可執行、部分失敗不 PASS、零新 verify 工單且既有封存不被新報告擋下。

## Risks / Trade-offs

- 回歸輸出與技能資產變更 → 只刻意增加政策欄位／摘要行，更新相關 golden／CLI 結構斷言；不改其他輸出。
- Windows 檔案替換、reparse、大小寫與路徑差異 → 平台條件測試及原子指標交易；無法保證安全時拒讀或拒寫，不做先刪再寫。
- 規則誤判與不完整覆蓋 → confidence、needs_review、逐條 NOT_VERIFIED、無 Security Gate；不把規則 PASS 提升為認證。
- 官方格式或資產日後變動 → 固定 commit／release、版本化 parser 與來源清單、快照 fixtures；更新失敗保舊，不跑網站提供的程式。
- 首次官方舊資產無官方 checksum → 明示 source-tls 信任級別與保存指紋，不冒稱供應鏈簽章驗證；匯入不能自行升格信任。
- 無法枚舉所有未開啟 Remote 政策引用 → 刪除前只判本機已知引用並明示範圍，之後缺資料回未驗證，不靜默下載。
- ASVS 大多要求不能用文字規則驗證 → 按版本與等級完整列未驗證，沒有可信對應不自行編號。
- 裝置本機報告不跨 Remote 隊友同步 → 報告可手動匯出 JSON／Markdown，Verify 只顯示本機可讀連結；分享／伺服器保存另有明確邊界，絕不上傳原始碼。

## Migration Plan

先落純政策與測試，再接 UI／Remote；依七階段增加資料、規則、報告與工作流。沒有新欄位的專案不遷移，主開關仍關閉。使用者執行既有 speclink update 取得新版技能，安裝應用程式本身不下載標準。撤回版本前先停用政策；舊版引擎忽略新區塊，資料與報告留在應用程式資料目錄，不破壞專案或舊技能。新資料安裝只新增 generation，失敗能重試。

## Open Questions

無待使用者裁決的功能範圍。實作前的第一階段任務須核對初始來源清單的逐檔 SHA／LICENSE、Windows ReplaceFileW 與 no-follow 能力、依賴 resolver diff；任一檢驗不符即回報具體衝突並修訂設計，不能把缺失隱藏為已支援或省略驗收功能。


## 實機回饋 Implementation Contract

- SecurityDataPage 以當次 request 記錄 action／standardId／version；點下載後立即只把該版本按鈕顯示為「下載中」，直到 wait 結算。其他版本保持原文案且停用；重下載亦同。取消或失敗後還原，不以進度事件猜測標準版本。
- 專案設定在既有頁面加入「專案圖示底色」，含預設及青綠／藍／紫／粉紅／橘／綠六個色票，立即儲存到 speclink.projectColors，依 locatorKey 隔離。關閉再開或重啟保留；清回預設只移除該項。Local／Remote／無 checkout 都可編輯裝置偏好，不碰設定檔或 Remote revision。儲存失敗顯示錯誤且不假裝成功。
- ProjectRail 與 TrayPanel 共用 projectTileClass；設定色票後，選取時深底白字，未選取時以color-mix(in srgb, 色票700 50%, white)將底色淡化50%，搭配同色系950深色字；不使用整體透明度，以免文字與狀態點一起淡化；無設定保持舊樣式。色彩偏好由 store 經現有 tray snapshot 傳遞，不讓獨立面板另外讀寫設定。預設色與六色票沿用 Tailwind 顏色 token，深淺模式皆保留文字對比。
- 繁中及跟隨系統繁中都使用同一 zh-TW 字典。產品字典禁止「缺省」及「默认」等非台灣用字，default 統一「預設」；LANGUAGE.md 加入詞條與維護測試。不改官方原文或已封存歷史紀錄。
- 驗證：securityDataPage 測試含兩個 API 版本及延遲／失敗；色彩測試含儲存重開、不同 locator、壞值／儲存失敗、主視窗及面板、設定互動；字典與系統語言解析測試、TypeScript 與建置。
