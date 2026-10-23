## Context

本變更為 Feature，含公司 Proxy 更新問題的 Bug Fix。來源是 desktop-workflow-and-offline-security-review 與 owasp-cwe-acceptance-policy 的已定案結論，後者覆蓋舊的「安全結果永遠不影響驗收」方向。基本掃描仍是不依賴 AI 的固定規則執行器；新增 skill 是使用者明確執行的深度審查。

已確認的現況：ProjectRail 按 tabs 陣列呈現、tabs 持久化已能保存順序；UpdatesPage 在 updater props 不存在時只顯示標題，內附更新日誌因此也消失，背景檢查失敗則回 idle；這些是可重現的畫面缺口，不等於已證實公司網路根因。ProxyPolicy 已供下載共用。write_stamp 刪除 review.md／verify.md；安全報告目前存在裝置資料目錄。Store 有變更相對 artifact 讀寫，Host 的 Remote bridge 已可交易提交多筆文件。

## Goals / Non-Goals

**Goals:**

- 交付原始九項需求及可選安全驗收，保留基本掃描與既有政策、品質章及離線資料包。
- 讓另一位實作者能依明確的命令、文件格式、失敗狀態與測試完成，不重新判斷需求。
- 在同一變更依桌面、資料、紀錄、skill、驗收順序落地；雖跨十四份既有能力，保存與驗收共用同一報告模型，因此不拆出彼此重疊的變更。任務按可獨立驗證的小步分組。

**Non-Goals:**

- 不移除基本掃描，不整合 Checkmarx 或引入第三方掃描 runtime，不新增本機模型部署。
- 不自動啟動 AI，不讓儲存政策觸發掃描，不連網取得審查參考資料，不修改受檢程式。
- 不新增 PAC／WPAD、Proxy 認證、自訂 CA 或任意 URL 下載器；以既有支持的路由修復可重現錯誤， unsupported 狀態需說清楚。公司環境若證實需要額外能力，回報證據與另案範圍，不能冒稱本案已驗收。
- 不重新設計整個 Desktop、Store 或通用交易框架，不以一般審查的 --accept 代替安全風險逐項接受。

## Decisions

### 提案預設條列顯示

共用 ProposalDoc 是所有專案、Local／Remote、活動與已封存提案的唯一顯示落點。Why／What Changes／Proposed Solution 使用既有 NumberedSteps；已列點內容保留各項與巢狀結構，未列點的連續散文區塊作一項，不依標點推測拆句。列點與散文混合時依來源順序連成同一編號序列並顯示項目數。Markdown 連結、程式區塊與續行保留，不修改原檔；空章節不產生空編號。未知章節與無二級標題文件沿原 Markdown 顯示。

新提案的共用 spec-driven 模板與 propose 原稿明定 Why、What Changes 一事一項；透過 init／update 套用所有專案，無專案特例或切換設定。本次提案 Why 同步改為保留原意的條列。以 ProposalDoc 測試兩份不同專案內容、散文與清單混合、空內容及未知章節，並以 render_golden 核對三種工具原稿及資產版本。

### 專案排序

ProjectRail 沿用 locatorKey、tabs 與 persistTabs，由 store 的單一 reorderProject 動作提交。使用 packages/ui 既有 dnd-kit 配置垂直排序，右鍵補上「上移／下移」，讓鍵盤使用者也可操作；不引入另一套拖曳依賴。拖曳達 6px 才開始，拖動不誤觸專案切換；Escape 取消、移出可放區不改順序，第一／末項的對應選單停用。只有完整且無重複的既有 locatorKey 排列可提交；失效項、Remote 離線項仍可排序。持久化失敗維持舊順序並顯示錯誤。activeKey、session、草稿及色票都不動；Tray 與專案切換快捷鍵使用同一 tabs 順序，新增附尾、關閉只刪該項。

### 更新頁與 Proxy 診斷

拆開「內附內容」與「需要 updater 的動作」。目前版本及內附日誌一律可呈現；沒有 updater 時禁用網路動作並顯示「更新服務尚未就緒」，不只剩標題。手動檢查失敗仍顯示 checkFailed；背景檢查維持不打擾，但另保留 lastCheck={checkedAt,status,errorCode,operationId} 給設定頁，不讓 idle 被誤解為檢查成功。此欄位不持久化機密、不改現有 idle／available 等 reducer 分支含義。

保留既有 ProxyPolicy 與進行中工作路由快照。可測的錯誤類別為 connect／timeout／tls／proxy-auth／http／parse／updater-unavailable，無可得狀態碼就留 null；診斷沿用固定 schema 及 operationId，不記 body 或憑證。以測試 Proxy 的 CONNECT 成功、407、斷線及假回應覆蓋，並核對 AppSettingsView、更新提示、下載列及診斷測試的同類顯示。只修實測命中的共通問題，保留 endpoint 白名單與更新簽章。公司端須人工匯出遮罩診斷並確認更新頁，不能用模擬通過取代公司實測。

### 十二條問號說明

BaselineSmellList 增加逐名稱 help 按鈕，沿用 @speclink/ui 的 Popover、Button、Tooltip；點擊、Enter／Space、Escape 與焦點返回都可用。名稱仍由引擎 REVIEW_BASELINE_SMELLS 提供；說明以同名 key 對照 i18n zh-TW／en，測試要求十二名皆有一筆用途與例子，未知名稱只顯示「此版本未提供說明」。同一元件用於審查規則頁及工單的套用規則列，避免兩種說明漂移。英文名稱不翻譯；共通註解寫明 extend 的預設清單只列建議、replace 不套用。

### 離線知識資料

新增的 spec_registry.json 是程式內建的核准指引目錄，採 formatVersion=1，standardCatalog 固定引用 sources.json；resources 僅接受 role=guidance，含 id、appliesTo、sourceUrl、sourceRevision、files（name/url/sha256）、licenseName/licenseUrl/attribution。標準原文與版本仍由 sources.json 定義；CWE 對照由已安裝標準快照推導，不另列為 registry resource。只接受經發行流程核對的固定 commit 或官方固定版資產；不用附件的 status 判斷本機安裝，不接受任意使用者 URL。

指南最小清單為 Authentication、Authorization、Cryptographic Storage、Logging、TLS、REST、SQL Injection Prevention、XSS Prevention、OS Command Injection Defense、Deserialization、XXE Prevention、SSRF Prevention、Prototype Pollution Prevention、Webhook Security、Denial of Service，另附已驗證存在的 DotNet／Java／Nodejs 指南。Python／Rust 以這些通用指南及資料流證據給工程修正，不假設附件的 Python_Security／Rust_Security 檔名存在；無適用依據時明示未驗證。發行任務要將每個真實檔名、固定 commit、指紋與授權寫進 registry；只要缺一份必要檔案，該深度資料包就不能宣告 ready。來源核對依 OWASP 官方 CheatSheetSeries repository，已核對的頁面為 https://cheatsheetseries.owasp.org/cheatsheets/DotNet_Security_Cheat_Sheet.html 、https://cheatsheetseries.owasp.org/cheatsheets/Java_Security_Cheat_Sheet.html 、https://cheatsheetseries.owasp.org/cheatsheets/Nodejs_Security_Cheat_Sheet.html 。網頁只用於開發時核實來源，執行時只使用包內 Markdown。

下載在使用者明確安裝／重新下載時一次取得所選版本的完整 generation（原文、對照、指南、授權）。對照依官方控制項 witness 解析；CWE 只在啟用時提供其字典及引用，停用的 ASVS／API／其他版本不加入 KB。guidance 是修正依據而非新增檢核標準。現有 ZIP v1 可列出原始補充檔與 SHA，舊包維持基本掃描可用但 knowledgeStatus=missing；新包的深度可用性以必要檔案清單驗證，匯出／集合包也帶同代補充檔。維持單包64MiB、展開256MiB、128 entries、單非XML2MiB等現有上限，禁止 symlink、路徑穿越及外部 XML 實體。

本機 prepare 只讀已安裝資料，在授權 checkout 的 .security_kb 產生可重建、gitignored 的快照，含 manifest.json、standards、mappings、cheatsheets。manifest 固定 projectKey、policyDigest、資料 generation／revision／SHA 與每檔相對路徑／SHA；Top 10 2021+CWE 可產生 mappings/owasp_2021_cwe.json，其他版本使用其版本化名稱，不能讀錯版本。先在受控暫存目錄寫完驗證，再替換完整快照；缺件、取消、外來同名目錄、連結或 SHA 不符保舊並失敗。快照重建只取代帶本工具 manifest 且屬相同 projectKey 的目錄，不覆蓋使用者自行建立的知識庫。Remote 需先明確執行既有 security prepare 取得政策／規格，kb prepare 與 skill 審查本身不呼叫網路。沒有授權 checkout 時明示 source_unavailable。

### 完成紀錄與封存保留

所有變更內完成紀錄共用 checks/index.json（formatVersion=1），entries 為 {recordId,kind,createdAt,outcome,relativePath,contentDigest}；recordId 為 ULID，kind=review|verify|security，relativePath 限工具產生的該變更子路徑。審查／驗證置於 checks/review/<id>.md、checks/verify/<id>.md，保留完整工單原文及完成時間、操作者、工具、指紋、當時規則與 accepted 狀態；原工單仍從 review.md／verify.md 建立，成功蓋章後刪除活動工單，以維持既有未結工單判定。新一輪建立新 record，不覆蓋舊紀錄；discard 不新造成功紀錄，也不刪歷史。

安全報告置於 checks/security/<reportId>/report.json 與 report.md，接受紀錄為同目錄 decisions.jsonl；只有通過驗證及遮罩的完整檔案才列入 index。未指定 change 的原 security run／report 維持裝置資料契約；指定 change 一律透過 Store 寫進提案。舊裝置報告不自動搬、不覆寫，可按舊 ID 查詢；透過 add-round 的 legacyImportReportId 明確帶入匹配專案／變更的 v1 報告並標 legacy，不作新驗收證據。

core 的 history 模組負責格式、路徑及完成狀態；Host 決定授權與提交；FsStore 處理本機原子檔案，Remote bridge 維持既有 staged transaction。查詢只走 index 及已列相對路徑，不新增任意檔案瀏覽 API，也不複製一套 Store。Desktop 文件樹新增「檢查紀錄」群組，列三種類、時間、結果及最近紀錄，可選任一歷次文件；已封存相同入口唯讀，缺舊文件顯示未保存而非重新執行。

check_records.archive.review／verify／security 皆預設 true，語意限定為封存時是否帶走完整歷史，活動提案一律保存。封存讀取當次已儲存設定並顯示三類保留／略過數；false 只排除已完成的該類完整文件，不刪活動提案、未結工單、以前已封存文件。安全驗收開啟時，另保留 checks/acceptance.json 的最小裁決收據（政策 digest、報告 digest、接受理由、結果與時間，不含程式片段），完整 security 報告不保留也不抹除驗收依據；UI 清楚說明此差別。

### 多筆寫入與失敗恢復

蓋章先完成既有守門及安全驗收（verify 才需要），再產生全部歷史 bytes、索引、章 metadata 與活動工單刪除計畫。Remote 以現有 UoW／CAS 原子提交；401、reader、revision 衝突不寫檔，Desktop 保留未提交的報告／接受輸入供重試，原始碼不送 server。

本機採專用 checks 提交記錄而非泛用交易框架：在變更內暫存寫齊、sync，寫入 .checks-transaction.json（transactionId、舊工單／metadata 的 digest、工具生成的目標及復原備份）；再發佈完整歷史、刪活動工單、寫 index、最後寫章 metadata 並移除 journal。journal 存在時，查詢明示 history_recovery_required，不顯示完成；下一次相同變更寫入先依 journal 恢復工單、metadata 及 index 至舊狀態，再重試。journal／備份路徑只能在該變更內且拒連結；若寫入／復原失敗就維持未成功並明示錯誤，不清理唯一備份。歷史先寫成但尚未提交的孤兒不列 index，恢復時清理；沒有「蓋章成功卻無紀錄」狀態。

封存先判任務、現有章與新安全政策，再在受控 staging 建立按設定裁切的封存文件樹；驗證 index 引用存在後以同一封存提交發佈。原活動目錄在封存提交前不刪文件；裁切／寫入失敗保留活動提案且不得顯示已封存。Remote 用既有交易；本機沿用封存失敗備份並增加 checks journal 的回復資料。保留設定只作用於完整紀錄，不使 carry-review／carry-verify 帶走的未結工單被刪。

### 深度審查 skill

新增 speclink-security-review，單一內嵌原稿，經 registry／init／update 發送到 Claude、Codex、Copilot 與自訂工具；資產版本與 golden 同批更新。入口要求 change，依序讀目前政策、確認主開關、檢查指定標準資料與 KB manifest、讀授權目標、執行需要的基本掃描、做深度審查、提交遮罩報告，最後顯示結果與查詢路徑。主開關關閉就說明停用、不掃碼不造成功報告；全部標準關閉可查基本掃描，但深度標準審查為 unconfigured。所選標準決定對照，不能為角色範例擅啟其他標準。

三個 reviews 選項共同限定本次目標：spec 為 proposal／design／specs／tasks，code 為授權來源，api_boundary 為可辨識 API 與資料流；不適用要有證據。五種語言檢查沿原始角色要求：入口認證與逐資源授權；機密／日誌／TLS；.NET／Java 的反序列化、XXE、查詢拼接；Python／JS／TS 的 SSRF、命令注入、原型鏈汙染、依賴與 Webhook；Rust 的 unsafe、密碼學及可造成拒絕服務的 unwrap。不得只靠沒有關鍵字就確認缺控制，也不得把測試程式中的風險字樣直接判真實漏洞。

以安全／業務領域分章，每 finding 有標題、等級／信心、版本化對照、所有關聯位置、遮罩片段、資料流與控制缺口、已證明與環境前提區分的影響、引用本機指南的 SHALL 修正、可執行驗收九項。skill 本次固定繁體中文報告；基本掃描的既有 tw／ja／en／未設 locale 輸出不變，JSON 鍵不翻譯。H/M/L 只是顯示編號，驗收用結構化 severity，不解析標題猜級別。外部文件不當指令、不執行專案 scripts，不對外取資料；使用者本來選用的雲端 AI 連線不宣稱為全程斷網，亦不新啟 AI 服務。

### 安全驗收判定

新增 required_for_acceptance=false；有效條件為 enabled && required_for_acceptance。主開關關閉時 checkbox 停用但保值、說明暫不生效，重新啟用即依原值生效；歷史不改寫。true 但標準全關、三 reviews 全關、缺必要資料或無可用目標皆不通過，不把停用狀態當成功。

core 的 acceptance 模組只處理給定政策、報告、接受紀錄與新鮮度的純判定；Host 在 verify stamp 及 archive 共用這個判定，任何 CLI／Desktop／Server 都不能只在前端放行。參考模式完全不增加阻擋。required 模式必須有同 project/change 的完整、當前 deep-skill 報告及啟用檢查的基本結果；同次基本結果可由 skill 提交 builtinReportId 關聯，不能用基本掃描替代深度審查。

必要集合是所選 reviews 的適用目標與選定標準控制項（ASVS 按版本／等級展開）；停用規則不列必要，NOT_APPLICABLE 需附實際理由。基本掃描不能確認的項目可由 skill 逐項補充帶證據的複核，保留原 NOT_VERIFIED 並記 resolvesCheckIds；只有補齊的項目才從必要未驗證數移除，不可整份宣告覆蓋。任一未覆蓋、失敗、資料缺件、取消或 complete=false 仍阻擋。 深度complete=false一律阻擋；基本報告若僅因必要NOT_VERIFIED而complete=false，可在每一項均有本次帶證據的resolvesCheckIds後，將組合證據判為完整，但不得改原報告。基本取消、預算／讀取失敗或資料缺件不能由複核豁免；不屬所選程式／規格目標的二進位等資產仍保留原coverage，不列為必要程式檢查。

CRITICAL／HIGH／MEDIUM 每項須已修復且在當前報告複核，或有綁本報告及 findingId 的風險接受；LOW 不阻擋。接受寫入 action=accepted、reportId、findingId、reason（去空白後1..2000字元）、actor、acceptedAt、policyDigest、scopeDigest；身份由 Host 的既有 actor 決定，無 actor 回 identity_required。skill 對每項列出原文與影響讓使用者明確接受，不能自行接受；一般 verify --accept 仍只豁免其原必修項，不豁免安全狀態。

新鮮度以政策的實質審查欄位、所選標準／指南 generation SHA、目標文件與來源檔清單及每檔 digest 綁定；保留設定不屬審查政策 digest。required checkbox 變更要重新計算驗收，但不改舊報告；只從參考切 required 且其他 digest 相同、原報告完整時可重新判定。Local 在驗收前重讀授權清單（含新增／刪除）比對；Remote 驗 server 可持有的政策與 SDD revision，原始碼仍由對應 checkout 端重算提交，server 揭露 sourceFreshness=client-attested，不宣稱能獨立讀驗本機原始碼。缺這次核對回 source_freshness_unverified；不存在 checkout 不借用別的目錄。任何來源／政策改變讓舊接受不適用，須以新報告明確再接受。

### 公司 Proxy 打包後驗收

2026-10-21 使用者指定公司環境只接受打包版本，取代前述將公司實機操作視為開發期間可完成的安排。保留 12.3 的手動任務、ID 與未勾選狀態，標示「開發期間跳過，打包後執行」。這項延期不阻擋繼續實作或產生測試安裝檔，不改任何引擎守門，也不代表驗收成功或允許直接封存。先前模擬結果繼續有效但不能代簽；正式審查、驗證時分開列程式完成與延期人工項。

交付順序依使用者追加說明固定為：完成實作與合併前必要檢查 → 將功能分支合併回本專案 main → 從 main 的明確 commit 打包產生安裝檔 → 在公司安裝並執行 12.3。公司驗收在合併與打包之後，不以尚未完成的 12.3 阻擋本次合併安排，也不把合併視為驗收或封存。打包與驗收紀錄須能對應 main commit、安裝檔版本與名稱，不能拿功能分支的開發視窗代替。這是後續交付順序，本輪 ingest 不執行合併或打包。

打包後由實際操作者在公司安裝，記錄 main commit、安裝檔名稱、Desktop 版本、OS、Proxy 模式、版本／日誌、檢查／下載、遮罩 operationId 及錯誤分類至 company-proxy.md。有未簽章更新包時驗手動 Release 入口與明確狀態，不為驗收關閉簽章。尚無安裝檔時只記延期，不建立虛構的版本或通過結果。

### 本分支更新與原作者版本

更新頁明列「本專案 mosutw/speclink」及「原作者 MomoChenisMe/speclink」，各有固定專案連結、版本、查詢狀態與檢查時間。主要按鈕查本分支，原作者版本另提供查詢入口；原作者資料不進入安裝候選或版本比較，不因原作者版號較高而提示覆蓋安裝。AboutPage 的原始碼／問題回報與原生選單連結指向本分支，另外保留可點擊原作者專案署名；中英介面一致。

已核對 tauri.conf.json 的 updater 端點已是 mosutw，但 AboutPage 還指向原作者；.github/workflows/desktop-build.yml 明寫不提供簽章更新包或 latest.json。因此不能只換網址就宣稱自動更新可用。延續既有 updater 驗簽與版本正規化；有合法本分支描述檔與可驗簽套件才使用安裝流程，沒有描述檔／對應平台／可信簽章時，以本分支已發布桌面 Release metadata 呈現版本並提供固定 Release 頁「手動下載」，不將 JSON 404 說成已是最新，也不退回原作者安裝包。簽章不符仍報失敗，不以自動下載未簽包作替代。

版本 metadata 在 desktop-core 組裝、Host 共用網路政策執行，Tauri 殼只委派；前端不直接 fetch GitHub。採 GitHub 官方 List releases API，僅兩個固定 owner/repo，過濾 draft／prerelease、只認本專案既有桌面 tag 規則及平台安裝資產；依發佈時間選最新正式桌面版，不拿 CLI-only Release 當桌面版。每次最多取 100 筆；範圍內無合格版本時顯示未取得桌面版本，不聲稱專案沒有發行。API 回應欄位只取 tag_name、published_at、html_url 與 assets 所需資料，使用既有逾時／大小／取消限制與 Proxy 快照；404、403／429、斷線、TLS、解析分別呈現可得原因，無狀態碼仍留 null。來源連結僅開啟對應固定專案下的 Release，白名單補兩個 repo 與所需 API；不擴成任意 URL。

兩份查詢狀態分離，舊成功版本可暫留並標「上次取得」與時間，失敗不冒充最新；尚無成功資料顯示未取得。內附目前版本／日誌永遠可讀。本次不要求建置時連網，也不自動取得或更換發行私鑰、公鑰；若後續要發布自動更新包，需由維護者提供匹配簽章的正式發行設定，現有手動打包照常可用。

### 專案貢獻與原作署名

README.md／README.zh-TW.md 增加平行署名：mosutw（本分支維護者，連 GitHub 個人頁）、Codex（AI 開發協作工具）、MomoChenisMe（原作者，連原作專案）；不移除 LICENSE、原作者既有貢獻或既有歷史。不新增 Desktop Contributors 清單，這次需求指截圖中的 GitHub 側欄及專案說明。

CONTRIBUTING.md 說明 GitHub 側欄由平台依提交身分統計，非 README 可設定；使用者的 author email 須連結本人 GitHub 帳號，工作合入預設分支後再檢查 Contributors。Co-authored-by 用於真實共同作者的帳號關聯 email；Codex 的 AI 協作先以 README 誠實標示，不假造機器人帳號或 email，也不承諾它必然出現在側欄。後續若使用者指定可驗證的 bot 身分才另按 GitHub 規則處理。本次只交付說明，不改 git config、不重寫歷史、不產生 commit 或推送。參考 GitHub 官方 contributors 與 multiple-authors 文件，查核連結記於 ingest 證據。

### 安全標準資料存放路徑設定

頁面沿 SettingsCard／SettingsRow、既有路徑選擇器與輸入原語，提供目前生效絕對路徑、可手動輸入／選擇目錄、儲存、取消、還原預設。設定是整台電腦的標準資料位置，不是專案工作流政策，也不跟 Remote 專案同步。未設定沿用 security_data_dir()/security/standards（portable 仍沿原行為）；自訂值直接代表標準根目錄，不額外追加 security/standards。Windows 接受磁碟絕對路徑、完整 UNC 如 `\\fileserver\team\speclink-standards`；macOS／Linux 使用已掛載共用磁碟的絕對路徑，不新增 SMB client、帳密或掛載流程。拒相對路徑、不完整 UNC、URL、NUL，不做環境變數或 ~ 展開。

Host 唯一解析與保存 `global_config_dir()/security-data-preferences.json`（新增，非 RuntimePreferences 的副本），格式 `{formatVersion:1,standardsPath:string|null}`。缺檔／null 為預設，舊設定不用遷移；錯型別／不支援版本／不可讀明確失敗，不能悄悄用本機另下一份。Host 對這個設定要求全域目錄為絕對路徑，不能沿 global_config_dir 的工作目錄 fallback 寫檔。以同目錄暫存、sync、原子替換保存設定，失敗保舊。Desktop 讀寫用新增委派 IPC，CLI／Node／Desktop／Server 既有資料、掃描、KB 入口由各自 Host 共用同一解析；Server 的位置只對其服務帳號及主機生效，不能將用戶端 UNC 傳給 server 當路徑。

FsSecurityData 保留 application_data 的本機用途，另接受 Host 已解析的 standards 根；只替換 standards/generations/current.json 等標準資料讀寫根，不將 security/reports、references、prepared、工作鎖／診斷、憑證或專案 .security_kb 整包搬到網路共享。CLI 舊命令、旗標與 JSON shape 不變；Desktop 新路徑快照為 `{standardsPath:string|null,effectivePath:string,isDefault:boolean,access:string}`，access=readWrite|readOnly|unavailable。寫入 IPC 輸入 `{standardsPath:string|null}`；設定檔及回應均 camelCase。清單／標準下載／匯入／匯出／刪除／KB／掃描統一取相同根，既有 StandardRepository 契約不複製一套網路實作。

儲存時候選目錄須存在且可讀；唯讀共用目錄仍可選用，UI 清楚停用下載／匯入／刪除等變更動作；匯出讀標準資料、寫使用者選定的本機輸出，不因來源唯讀而停用。目錄不存在、拒絕存取、主機離線、設定落盤失敗保留舊值與草稿。之後共用位置斷線顯示不可用，不顯示空清單或 fallback 本機；I/O 在既有背景工作執行，取消後不提交結果、不阻塞 UI，OS 未返回的 SMB I/O 不承諾可立即中斷。路徑切換期間有該裝置資料安裝／匯入／刪除／掃描／KB 工作就回 busy，避免開始一半改根；每個工作固定一份路徑快照。還原預設同樣先驗證預設目錄（沿既有本機初始化流程）再保存 null，不搬移、刪除或合併舊／新目錄。

共用安裝沿現有不可變 generation、版本級跨程序檔案鎖、reader pin 及 current.json 發佈，staging 必須在同一 share／磁碟。兩個 writer 爭用同版本時一方 busy，不能只靠單機記憶體鎖；share 不支援必要鎖或原子發佈時拒絕寫入，保持可讀舊 generation，不假設所有 NAS 都能安全寫入。唯讀使用者只讀可開啟的 pin、不建立鎖／參考檔；版本使用引用仍存各裝置，故共用刪除不得依單機 references 推斷全團隊無人使用：自訂根的刪除預設停用，管理者在共用位置外明確維護；本機預設刪除行為保持。每次讀取仍驗 SHA／必要文件／路徑邊界，選 UNC 不豁免資料完整性。共用節省的是完整下載資料，.security_kb 仍是各 checkout 的既有本機快照，本次不改成共用可寫快照。

### 檢查紀錄檔名包含審查或驗證開始時間

每輪審查／驗證開始時由系統建立穩定的 UTC 開始時間；蓋章完成紀錄時以 `YYYYMMDDHHmmss<ULID>.md` 命名，時間取該輪開始時間而非蓋章時間，ULID 保持可獨立查詢的 `recordId`。相同秒內開始的輪次仍以不同 ULID 區分；`checks/index.json` 的 `relativePath` 指向完整檔名，舊 ULID 檔名仍可讀，不為舊紀錄改名。若紀錄完成寫入失敗，既有原子復原規則仍適用。

### 品質關卡點選狀態展開對應紀錄

Desktop 提案詳情的品質關卡中，「已審查」與「已驗證」狀態各自作為可操作的展開控制項；展開後從同一 Store 載入相應類別的紀錄，依開始時間由新到舊列出時間、結果與 ID。選取一筆後沿用歷史文件檢視器顯示該筆內容。兩類展開狀態獨立，鍵盤可操作；載入錯誤、空清單、切換專案與 Remote 離線沿用現有歷史查詢錯誤語意，不觸發重跑。

範圍只涵蓋審查與驗證品質關卡，不改安全審查列表、蓋章守門或索引 schema 的 `recordId`。品質關卡的「已審查」與「已驗證」是 Desktop 歷史紀錄唯一查閱入口；文件區只呈現進行中的 `review.md`／`verify.md`，移除文件樹及已封存詳情的歷史紀錄入口，避免同一紀錄重複列出。

### 歷史紀錄只從品質關卡查閱

提案詳情文件區僅呈現尚在進行中的審查與驗證工單。蓋章保存的歷史紀錄不顯示為文件節點或獨立檢查紀錄清單；使用者展開品質關卡「已審查」或「已驗證」時才載入該類歷史，按開始時間由新到舊列出並可開啟內容。兩類展開狀態獨立；變更或專案切換時清除舊選取。已封存詳情不重複提供審查／驗證歷史清單。

## Implementation Contract

### SDK 技能 API 與公開型別補查修復

2026-10-22 使用者要求完成 CI 核對與 SDK 修復，採補查建議的現行技能 API 方向。保留全部原始九項需求與已完成任務；本段更新 delta 的要求，不改寫歷史品質證據或 canonical specs。

- 行為與介面：沿 `index.js` 的 skills.list()／skills.render(name, options) 與 Rust 共用渲染器；options 為 target（claude／codex／copilot／antigravity／neutral）、invocation（cli／tool-call，預設 cli）、specDir（預設 openspec）、toolName（neutral 專用，預設 speclink）。回傳字串、不寫檔、與相同 specDir 的 CLI 技能同源。
- 決策：對齊現行 API；不另建已退役的 instructions.render 或 store 渲染軸。delta 明確移除「渲染 API 的 worktree 軸」，避免封存後仍留下依賴退役 API 的矛盾要求；createEngine 的 Store 儲存契約保持原樣。
- 型別與錯誤：手寫公開 `index.d.ts` 補 antigravity。編譯測試經套件入口解析 package.json 的 types，五個 target 與完整選項均可用；以預期編譯錯誤檢查未知 target／invocation、store／worktree 與 instructions 匯入。執行測試保留未知 target／技能拒絕，補未知 invocation 拒絕、預設值與自訂 specDir／toolName 替換。
- 驗收：在 Node 套件新增 TypeScript 開發依賴，npm ci 單獨安裝即可執行編譯測試，不依賴根 workspace 的 compiler。先以新測試重現 Antigravity TS2322，再修型別；沿既有 npm test 納入 CI。建置原生 binding 後跑 Node SDK 全組，保留 CLI 同源、Codex／Antigravity 獨立目錄及無指令檔測試。最後跑 scripts 全組、analyze、validate、git diff --check。
- 範圍：本輪只修 SDK 契約、型別、必要測試與雙語 SDK 說明，記錄最新提交 CI 查詢結果；不處理另一組 vscode／dirty Git 覆蓋與 configuration 表格缺口，不清除或重開品質工單、不蓋章、不合併或發版。那些補查項留待下一步完整品質檢查前處理，公司 Proxy 仍打包後驗收。

2026-10-21 使用者同意修復 Windows 封存鎖：所有新版本的 checks 寫入／復原／封存，先取得 `<spec_dir>/.checks-locks/<sha256(change-name)>.lock`，再建立或讀取活動提案，整段搬移／備份／發佈仍持鎖；鎖檔不跟提案搬移且不可置於單機快取。同一spec store的各程序共用同一路徑；目錄與鎖檔拒連結，鎖目錄內建立只含 `*` 的 .gitignore，鎖不列文件樹。既有提案內 .checks.lock 只作舊版相容性偵測：若仍被鎖定就回busy，不移除舊鎖。升級時須先停止所有舊版寫入者，混用新舊鎖定協定不受支援；不能以先解除新鎖再搬移或放棄跨程序保護規避Windows限制。沿原復原journal及generation格式，不修改原已完成任務，追加16.1作為接續實作前置修復。

2026-10-21 追加範圍：打包後公司驗收、本分支更新與原作者資訊、中英專案署名、標準根目錄設定與 UNC 共用。不在範圍：現在打包／發行／推送、建立 GitHub bot、重寫作者歷史、取得簽章金鑰、SMB 帳密管理、共用私人報告、刪除其他主機仍會使用的資料、自動搬移舊資料。原已完成任務逐字保留；新增工作使用第 13–15 組任務，完成前不得沿用 42／42 已完成的舊結論。

所有新增 CLI 命令支援全域 --no-color；JSON stdout 是單一物件、camelCase，不混 ANSI／進度；人眼 stdout 說明同一結果；錯誤 stderr，成功0、資料／授權／I/O／判定失敗1、參數錯誤2，kb prepare 取消130。除明列 --stdin 外不讀 stdin。新增英文 CLI 文案，UI zh-TW／en，skill 固定繁中，既有基本掃描語言與中文弱語言偵測不動。

| 入口 | 輸入與輸出 |
| --- | --- |
| workflow-config security --stdin [--dry-run] | 原YAML部分patch增加 required_for_acceptance:boolean；原摘要、diff、exit不變；show JSON 的 securityVerification 增 requiredForAcceptance:boolean |
| workflow-config check-records --stdin [--dry-run] | 最多64KiB YAML，如 archive: {review: true, verify: true, security: true}；只改 check_records；stdout 摘要或實際行diff，無 --json；未知鍵／錯型別拒寫；show 增 checkRecords:{archive:{review:boolean,verify:boolean,security:boolean}} |
| security kb prepare [--root PATH] [--change CHANGE] [--json] | stdout {projectKey:string,path:string,policyDigest:string,files:number,bytes:number,standards:[{standardId:string,version:string,dataSha256:string}],knowledgeDigest:string}；完全本機，Remote 依指定提案載入已準備的政策，缺件 missing_knowledge 不改舊快照 |
| security review add-round CHANGE --stdin [--json] | 至多4MiB JSON 的 deep report 輸入，schema 見下；由 Host 補 reportId／actor／時間及驗證來源digest，stdout {reportId:string,changeName:string,relativePath:string,complete:boolean}；不接受任意路徑，舊報告匯入使用只有 legacyImportReportId:string 的另一種輸入，並標legacy |
| security review show CHANGE [--json] | stdout {changeName:string,reports:[{reportId:string,executionMode:string,relativePath:string,complete:boolean}],acceptance:{required:boolean,status:string,blockingFindings:number,notVerified:number,reasons:string[]}}；status=reference|passed|blocked|stale；人眼列同資訊與 PASS 限制 |
| security review accept CHANGE --stdin [--json] | 至多16KiB JSON {reportId:string,findingId:string,reason:string}；只接受當前報告真實 finding；stdout {decisionId:string,reportId:string,findingId:string,status:"accepted"}，舊報告／未知ID／缺身份／空理由拒寫 |
| review history CHANGE／verify history CHANGE [--id ID] [--json] | 無id回 {records:[{recordId:string,kind:string,createdAt:string,outcome:string,relativePath:string,contentDigest:string}]}；有id回 {recordId:string,content:string,metadata:{createdAt:string,actor:string|null,tool:string|null,outcome:string,scopeDigest:string}}。人眼無id列摘要、有id列遮罩原文；支援active名稱及已封存dated名稱，ULID與根邊界驗證 |
| security report ID [--change CHANGE] [--format json|markdown] | 無change維持原裝置查詢；有change僅讀該提案index對應完整報告，支援已封存dated名稱；預設json，不重掃；不存在回1及stderr |
| verify stamp／archive | required模式新增 security_acceptance_blocked／security_report_stale／missing_security_review 錯誤與原因，失敗無文件效果；參考模式及既有成功stdout／JSON保持回歸對照，新增查詢API提供詳情 |

Deep report JSON 使用 formatVersion=2，與既有 v1 並行讀取；必備 changeName、policyDigest、knowledgeDigest、scopeDigest、builtinReportId:string|null、complete:boolean、targets:string[]、checks:[{checkId,status,reason,evidencePaths,resolvesCheckIds}]、findings:[{findingId,title,severity,confidence,confidenceReason,references,evidence,codeSnippet,dataFlow,impact,remediation,acceptance}]。references 含 standardId/version/controlId/localPath，evidence 含 path/line/endLine/function；severity 小寫 low|medium|high|critical。每個 finding 九項必填、references 可空但須明示無可信對照，檔案必須落授權範圍、行號大於0、最多10000findings；check狀態沿五種結果。版本不符、非已啟用對照、未知check／finding重名或證據超範圍皆拒絕。最多每段片段400字元、秘密在寫檔及回應前遮罩，不能只由前端遮罩。JSON與Markdown由同一遮罩模型生成；更長資料流使用結構段落，不放大程式片段上限。

**安全驗收摘要查閱**

安全歷史查詢沿用 `/security/history`，其 list 回應另有可選 `acceptanceReceipt:{createdAt,content,contentDigest}`（不存在省略）；content 為從驗證過的 checks/acceptance.json 產生的遮罩 Markdown。Local 使用相同 Host 查詢。Desktop 在安全紀錄類別以「安全驗收摘要」獨立列出，沒有完整報告時仍可唯讀檢視。摘要不是 ULID 報告，不送入報告端點，也不納入新一輪驗收判定。舊 server 省略欄位表示沒有提供摘要。

### 深度提交的識別與指紋

驗證／封存的Host交棒使用只在程式內建立的AcceptanceProof（不提供JSON反序列化或CLI輸入）：綁changeName/projectKey與checks/index.json指紋，攜帶已驗證的basic/deep模型、決策與當次來源核對。core再次用同一純evaluate函式判定並比對當前政策及index；缺Host核對或指紋改動即拒絕。舊stamp入口仍可呼叫，但required模式沒有交棒證據不得通過；CLI、Node、Desktop在各自Host邊界讀來源，不新增轉送Store。

- 必要檢查ID沿用標準控制項的 `{standardId}/{version}/{controlId}`；所選目標家族另外使用 `review/spec`、`review/code`、`review/api_boundary`。Host依所選版本與ASVS等級列出必要集合，未選項不得自行加入。每個目標家族的evidencePaths覆蓋其實際目標，NOT_APPLICABLE必須有實際理由。
- 基本報告的複核ID是 `{ruleId}::{target}`；空ruleId的覆蓋缺口用 `coverage::{target}`。resolvesCheckIds只能指向關聯基本報告中真實的NOT_VERIFIED項，不消除原FINDING。Host公開待複核ID，skill不猜編號。
- 新深度／KB的policyDigest以SecurityPolicy的已知欄位序列化JSON後取SHA-256，固定requiredForAcceptance=false；封存保留設定不納入。v1原policyDigest繼續維持原文SHA。Host在建立深度報告時驗證builtinReportId確實是當次政策與來源，之後用這筆不可變關聯與深度的實質政策指紋判定，不能僅因checkbox或保留設定改動就作廢原完整證據。
- scopeDigest以按相對路徑排序的 `[{path,contentDigest}]` UTF-8 JSON取SHA-256；檔案指紋取實際位元組，路徑用正斜線。目標限所選程式與proposal／design／specs／tasks；工具管理的章metadata、checks、journal／lock、KB及忽略清單不納入來源新鮮度，workflow設定由policyDigest負責，KB由knowledgeDigest負責。因此verify蓋章或產生報告不會自行使報告過期。
- `security review show --json`另提供submission物件：projectKey、policyDigest、knowledgeDigest、scopeDigest、targets、requiredChecks、builtinReportId、unverifiedChecks。這是Host計算的提交上下文，不含原始碼或秘密；缺資料時附明確原因，不製造可提交的完整狀態。
- Remote提交與verify／archive的當次核對使用securitySourceAttestation：policyDigest、knowledgeDigest、scopeDigest、sources（path／contentDigest）、checkedAt；由授權checkout的Host生成。Server驗其政策與SDD來源，只將原始碼部分標為client-attested；required時缺少當次核對即拒絕，不把先前一次的提交當成永久新鮮。

Remote交棒細節：binding新增securityReviewV2與checkHistory能力（缺席皆false）及securityProjectKey。securityProjectKey由server以既有public_url的origin、所屬project/repo及checkout=null的ReviewBinding計算；本機prepared快照仍用原含checkout的cache key，另保存此canonical key，避免兩台checkout產生兩份提案身分。無change的v1裝置報告key不變。KB與提案報告採canonical key；knowledgeDigest只綁projectKey、實質policyDigest、檔案SHA及所選標準的standardId/version/dataSHA/sourceRevision/parserVersion，不納安裝時間或隨機generation ID，確保相同內容可在裝置間交換。

Remote基本掃描維持零HTTP：指定change時先保存明示「待發佈」的遮罩草稿。security review add-round才是明確發佈，在同次交易帶入所關聯的基本報告及深度報告；server成功前不將草稿當成提案權威，失敗保留供重試，成功後本機只作帶發布收據的快取。遠端讀者只讀server index。資源審查只用已安裝知識；查詢自己的server紀錄與明確發布是應用程式資料操作，不是下載OWASP資源。

新寫入expectedRevision指向checks/index.json的文件revision（不存在為0）；接受風險同次重寫相同index bytes以推進revision，兩個editor使用同一revision時僅首筆成功。verify/archive帶securitySourceAttestation時同樣核對當前index revision。attestation只含change/project身分、指紋、sources路徑/SHA/lineCount、所選標準內容身分與checkedAt，不傳原始碼；server比對自己的政策／SDD、已安裝完整標準資料及scope，程式來源與行數明示client-attested。server所需標準包須由服務使用者預先安裝或離線匯入，缺件回missing_knowledge，不在審查期間下載。CLI深度stdin仍限4MiB，Remote成對報告加attestation的HTTP body限8MiB；單段程式片段仍限400字元。每次敏感操作重新生成核對，checkedAt須是有效UTC時間、距server時間不超過5分鐘且不可超前30秒。

Protocol 型別在 speclink-protocol 單一定義，沿既有領域 verb 路由新增 request／response；Remote 的 mutating request 帶 expectedRevision、editor／owner 權限及actor，reader只查詢。403、401、revision_conflict、offline不排隊寫入、不假裝完成；資料準備與本機基本掃描不需遠端變更revision，但Remote報告發佈是明確寫入、需要連線，離線時可保留checkout遮罩暫存供稍後提交，不能把未提交結果當成server已驗收。不得傳送原始碼檔案，只有已遮罩且限量的報告證據。

UI 沿用 SettingsCard、SettingsRow、Checkbox、Popover、Tooltip、Button、CodeEditor、DetailNav 及現有Markdown閱讀器；通用排序動作在store、通用說明在BaselineSmellList、歷史讀取在共用adapter。Tauri 新command只單行委派desktop-core，非Tauri的錯誤轉換／進度／取消放desktop-core。樣式變體由 @speclink/ui 擁有，不在apps用className覆蓋其顏色、圓角、陰影。

驗收矩陣至少含：排序取消與重開；無updater／Proxy407仍可讀日誌；十二條zh-TW／en與鍵盤；安裝後斷網的KB及skill、缺件保舊、只選2025不讀2021／ASVS／CWE；兩站蓋章與每個提交失敗點、Remote CAS；九欄報告遮罩；required off／LOW／HIGH接受／缺件／stale／一般--accept；封存三種開關的8種組合、未結工單carry；已封存歷史及舊v1查詢。

提案預設條列顯示驗收：兩個不同來源的 Why 均使用共用編號列；散文＋清單按原順序顯示且編號連續，字串／連結／程式區塊無遺失，無專案名稱或目錄條件。新模板及三種工具的 propose 原稿要求一事一項，舊檔唯讀顯示。

## Risks / Trade-offs

- [既有輸出與生成資產回歸] → 以CLI位元級對照、render_golden及assets.lock保護；只有新增欄位與有change的儲存效果刻意改變，快照同批更新。
- [Windows／macOS／Linux 的rename、檔案鎖與路徑差異] → 同磁碟暫存、合法相對路徑、ULID、拒連結、失敗保舊，模擬鎖檔／容量不足；不假設POSIX可覆寫資料夾rename。
- [來源檔名、版本或授權錯誤] → 發行前以真實官方固定資產驗證registry，缺一份必要文件則深度資料包不可標ready；舊基本掃描不被指南缺失阻擋。
- [AI的判斷不等於漏洞證實] → 必須有入口／資料流／信心及外部前提，保留NOT_VERIFIED，接受理由可查；納入驗收也不宣稱正式資安認證。
- [Remote無法獨立讀本機來源] → 明示client-attested且要求當次checkout核對、政策與SDD由server驗證；不傳原始碼彌補這項限制。
- [公司Proxy環境未取證] → 模擬測試加人工公司驗收任務，若仍失敗以遮罩診斷列未解根因，不寫「已修復」。

## Migration Plan

1. 先交付能讀舊檔的新政策與Protocol欄位，舊設定required=false、archive三項true；混版本server對新命令明確unsupported_version，不靜默略過驗收。
2. 建立本機紀錄與提交復原，再接Desktop歷史；不能從Git歷史推造過往未保存的工單。舊裝置報告繼續唯讀，明確匯入為legacy。
3. 新完整資料包與skill隨發行安裝；使用者明確重下載／匯入完整包再準備KB，未準備時顯示缺件且基本掃描仍可用。
4. 最後接required模式的verify／archive及風險接受，原專案不自動開啟。回退舊程式不會理解新驗收政策，因此啟用required的專案須使用支持版CLI／server，UI顯示版本需求；回退前由使用者明確取消政策，不刪新紀錄。

## Open Questions

無未定的產品行為。公司端實際故障原因屬執行期取證項，列人工驗收任務；資料來源固定commit／指紋屬發行資產核對交付，未核對不得發佈為可用資料包。

安全決策查詢補充契約：security review show 的JSON另含 decisions陣列，每筆為decisionId/action/reportId/findingId/reason/actor/acceptedAt/policyDigest/scopeDigest；reason與actor回應前遮罩，人眼逐筆列相同資訊。reports另含legacy:boolean，明確辨認舊匯入。decisions.jsonl每份最多16MiB，與完整報告合計納入原200MiB專案上限，超限拒寫，不改原報告。

## 2026-10-21 追加：Antigravity 與技能目錄標籤

### Antigravity 內建工具與獨立技能目錄

- 行為與資料：Tool::ALL 擴為 claude、codex、copilot、antigravity；既有 .speclink.yaml tools 字串清單可記錄 antigravity，Codex 的 agents 別名不變。Antigravity 生成於 .agent/skills，採與 Copilot 相同的非 Claude frontmatter 與 /speclink- 呼叫樣式，不生成 AGENTS.md、GEMINI.md 或其他指令檔。預設選集仍只有 claude。
- 來源依據：官方 https://antigravity.google/docs/skills 現行預設 .agents/skills 並保留 .agent/skills 相容。本專案採仍支援的 .agent/skills，使 Codex 的 $speclink- 與 Antigravity 的 /speclink- 生成內容各自保存。這是明確的產品目錄決定，不宣稱 .agent 是官方現行預設。
- 邊界：生成、更新、過期探測與取消工具後清理沿 Tool::ALL 與既有生成計畫，不建立另一套流程。取消只清理 .agent/skills 下受管 speclink-*；使用者自有技能、.agent 其他檔案與 Codex .agents 全數保留。不得跟隨 symlink 清理。
- 驗證：同時勾選 codex／antigravity 時兩目錄存在且呼叫前綴各自正確；取消 antigravity 後只移除其受管檔；未知工具與自訂描述子撞名／撞目錄仍在任何寫入前拒絕。CLI 人眼提示及非互動錯誤列四種支援名，JSON shape 與退出碼不變。

### 共用工具名稱與技能目錄標籤

- Desktop 的 builtinTools 作清單及標籤唯一來源，設定頁、初始化／啟用確認、checkout 綁定共用。顯示格式為工具名加空格及根目錄方括號：claude [.claude]、codex [.agents]、copilot [.github]、antigravity [.agent]；完整技能目錄分別為各根目錄下的 skills。
- 儲存與傳輸值仍為純工具名；不得把顯示標籤寫入 tools。無障礙名稱含工具與目錄。自訂描述子仍為原有唯讀列，不改變資料契約。
- 範圍包含四種內建工具的共用接線、中英說明與相關回歸；不加入 Antigravity 帳號登入、模型選擇、全域設定、外部安裝器或舊自訂描述子的靜默遷移。
- 既有工作區仍可讀；原本 name: antigravity 或指向 .agent/skills 的自訂描述子因與新內建工具衝突而拒絕，錯誤提示改名 antigravity-custom／改用內建字串，無寫入副作用。

### 風險與驗收

- 內建矩陣由三變四可能影響各 adapter 的提示、測試 fixture 與資產 golden；沿既有更新 golden 流程，測 CLI 與 Node 同名同目錄、逐段檢查差異，不改舊工具渲染內容。
- Windows／macOS／Linux 均使用專案相對目錄與既有檔案 API；以共用 Rust 與 Desktop 測試驗證，沒有真實 Antigravity 執行環境時不宣稱已在該工具實機驗收。
- 原 13 項品質問題已在 Round 2 複核解決；本追加擴充範圍，原品質流程以 scope changed 結束，未蓋章。新任務完成後需對擴充範圍重新品質檢查。

## 技能原稿格式守門修復

本項只修正既有技能的寫法與資產同步，不變更流程或檢核標準。archive 與 verify 降低非必要的大寫強調字；security-review 原稿在 EMPHASIS_CAPS 登記現有 SHALL 修復規範與 CRITICAL 驗收標籤，共兩個，不提高既有原稿上限。沿既有 ASSET_VERSION、golden、assets.lock 與 update 同步各工具，保持四種工具的生成契約。全部原稿共用格式檢查；不調整品質章、安全驗收或人工 Proxy 驗收規則。
