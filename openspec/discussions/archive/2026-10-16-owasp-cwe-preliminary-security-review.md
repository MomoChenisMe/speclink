---
topic: OWASP / CWE 初步安全審查模組
slug: owasp-cwe-preliminary-security-review
status: promoted
created: 2026-10-16
created_by: mosutw <luke1974@gmail.com>
promoted_to: add-owasp-cwe-preliminary-review
---

# Discussion: OWASP / CWE 初步安全審查模組

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者要求在 SpecLink 新增「OWASP / CWE 初步安全審查」（OWASP / CWE Preliminary Security Review），提供無第三方掃描器、無 AI 也能執行的離線基本規則、獨立標準版本選擇、系統層資料管理、遮罩報告及提示性的 SDD 工作流整合。最終須實際修改原始碼並交付測試與維護文件；本次透過 speclink-discuss 先確認架構，尚未開始實作。需求已有可驗收的目標與邊界，無須 grill；使用者於第一輪假設後回覆「確認」，同意本紀錄中的方向。

已讀 AGENTS.md、根 Cargo.toml／package.json、openspec/config.yaml，並調查 Desktop 的 WorkflowPage.tsx、settingsCatalog.ts、desktop-core/settings.rs、core/workspace/config.rs、host/policy.rs、audit／apply／verify／archive 技能原稿與 archive 守門。已核對相關 crate 相依與官方 OWASP／MITRE 來源。現有 Desktop 是 React／TypeScript；audit 為 AI Agent 的 sharp-edges 指引，並非既有離線規則執行器；政策文字改寫保留未觸及內容，Remote 寫入已有授權與 revision 檢查。

相關正式規格：workflow-config、desktop-config、server-policy-write；後續工作流設計須一併核對 verify-station 與 archive 的既有契約。調查時無進行中的變更，也無同題 live 討論，因此沒有在途 delta 相依。workflow-config Purpose 尚有四層解析的歷史措辭；現行程式與 tdd-switch-apply-wiring 的已落地結論為三層，不能照過時敘述新增舊鍵相容層。

Prior discussions: config-skill-rule-routing, custom-review-rules, quality-gate-speed, desktop-ui-redesign, baseline-apply-spec-rules, tdd-switch-apply-wiring, improve-cli-command-layer, worktree-flow-gaps, worktree-parallel-apply, worktree-toggle-and-guards, code-review-convergence-boundary, cli-desktop-credential-sharing, config-yaml-setup-skill, remote-workspace-local-skill-bootstrap, link-seal-timing, desktop-window-and-settings-polish, 專案選擇對齊-spectra

前次決策中直接相關者：custom-review-rules 已建立 core 單一政策解析／改寫落點與 Remote CAS；config-skill-rule-routing 將既有安全類規則在 audit 啟用時歸 audit，這次新增的是獨立的確定性離線審查，不變更既有 audit 的分流；tdd-switch-apply-wiring 要求由有效政策 payload 驅動 apply，而非由技能自行讀本機舊設定。其他搜尋命中含工作流、桌面設定及安全相關歷史決策，未因本次需求重開其已裁定的無關範圍。

Git 起始於 main 且工作目錄乾淨；已依使用者要求建立並切換 feat/owasp-cwe-preliminary-review，沒有 stash、reset、覆蓋或刪除使用者修改。這是單一完整功能變更的七個實作階段，不將任何驗收功能默默降為後續選配。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-16)

**Focus**: 新的初步安全審查應如何沿用既有架構，同時保持離線、設定相容性與不阻擋封存？
**Position**: 使用者確認十二項架構假設，完整功能範圍均保留，轉入單一變更的規劃。
- 決策樹依序為：政策契約 → Desktop 與 Local／Remote 持久化 → 標準版本與授權資料 → 離線檔案邊界及規則 → 報告 → SDD 整合 → 相容性測試與交付；開發依使用者指定的七個 Phase 進行。
- 政策沿用 core/workspace/config.rs 的 WorkflowConfig 與文字改寫接縫；舊專案缺欄位即停用，保留未修改 YAML、註解及未知欄位，不另建政策來源。現有 workflow-config 與 server-policy-write 已涵蓋保值、失敗拒寫與 Remote revision 檢查；安全政策區塊是新需求。
- Desktop 在 WorkflowPage.tsx 的產出政策卡 audit 下方新增主開關及條件顯示子設定，沿用 @speclink/ui、多語系與深色模式；資料管理放 settingsCatalog.ts 管理的應用程式設定。既有 Remote 不顯示 worktree 的行為維持。
- 主開關、scope 的 off 與各標準關閉各自獨立；版本及 ASVS 等級依實際資料解析。全部標準關閉時是已啟用但未配置，獨立基本規則結果與標準對應結果分開呈現。
- 規則與報告模型歸 core，執行生命週期歸 Host，檔案 I/O 沿既有檔案層，Desktop／CLI 經共用契約呈現；先支援 Rust、TypeScript、JavaScript 六類基本規則。audit.md 是 AI 指引，無既有離線引擎可直接調用，只重用其 sharp-edges 觀點與流程接點。
- SDD 規格缺少安全要求與已確認的原始碼漏洞分開；API 初步審查採靜態方式，不呼叫受檢 API。無法確認授權或邊界處理時回報需要人工確認，不能靠關鍵字缺席證明漏洞。
- 官方資料由使用者明確下載或匯入，放應用程式資料目錄，保留來源與授權 metadata，經限量、結構與版本驗證後原子安裝；不自動連網、不執行資料內命令、不增加掃描工具。官方來源已確認存在機器可讀格式；實際版本套件與授權逐版核對。
- 掃描不執行受檢程式與 scripts；限制授權根目錄、符號連結、遞迴、大小、總量與時間並支援取消。未完成項目保留原因，錯誤與不支援均不能當 PASS。
- JSON／Markdown 獨立報告含版本、覆蓋情況與五種狀態；證據保存前遮罩。PASS 只表示執行的規則未命中，不能聲稱安全認證或無漏洞。
- Verify 只新增安全報告資訊，不把疑似發現寫成既有必修工單項目；archive.rs 對未結 verify 工單有守門，直接混入會違反不預設阻擋 Archive 的要求。
- Remote 政策沿用既有授權與 CAS；程式碼只在明確授權的本機 checkout 審查，沒有 checkout 回報未驗證。離線執行須已有所需政策、規格與標準資料，不把 Remote 存取宣稱為離線。
- AI 是額外選用流程，無 AI 仍可完成基本規則；有網路的 AI 不稱離線，也不藉新審查流程自動傳送原始碼。Rust、TS、CLI、Desktop 需覆蓋相容性、UI、資料、規則、工作流與失敗案例，另交付 Desktop 操作步驟及維護文件。
- 新介面承載真正行為：core 隱藏規則判斷與報告語意，Host 隱藏執行、取消與資源控制；只沿用必要的 Tauri 委派，不增加純轉送抽象。移除新模組時只移除安全審查能力，原本 TDD／audit／Verify／Archive 仍可運作。
- 具體例：僅選 Top 10 2025 且尚未安裝資料時顯示資料缺失，不回 PASS、不背景下載；另行啟用的自有基本規則可以有獨立結果，但不能冒充 Top 10 驗證。
**Ruled out**: 第三方掃描器或要求額外 runtime——違反產品限制；只改 AI 技能——無 AI 便無法執行；另建 OWASP 專案設定頁或政策檔——重複現有入口與資料來源；把初步發現塞入 Verify 必修工單——誤判會觸發既有封存守門；把未命中、缺資料或未支援當成安全通過——沒有驗證依據；以純 Remote Store 當成可掃描的本機原始碼——儲存契約並未提供這項保證。
**Open**: 無待使用者裁決的架構問題；各版本下載套件／授權、資源上限、CLI 與報告保存契約、離線匯入格式及細部畫面交 proposal／design 核實與規定，完整功能與驗收範圍不縮減。

## Conclusion

**Decision**: 在 feat/owasp-cwe-preliminary-review 分支規劃並交付完整的「OWASP / CWE 初步安全審查」，沿用既有政策與分層架構，以自有離線規則為基礎，安全報告不預設阻擋封存。
- 功能定位：正式中文名稱為「OWASP / CWE 初步安全審查」，英文為「OWASP / CWE Preliminary Security Review」。不得聲稱完整 OWASP 掃描通過、正式資安認證、無安全漏洞或符合全部 ASVS。PASS 僅代表本次執行的指定規則沒有發現問題，不代表不存在其他漏洞；UI 與報告均須揭露。
- 工具限制：不安裝、下載、打包或呼叫 Semgrep、OSV-Scanner、ZAP、Checkmarx、Fortify、SonarQube Scanner 或其他獨立 SAST／SCA／DAST；不要求使用者加裝 Java、Python、Docker 等檢測環境。優先沿用既有 Rust／TypeScript 與相依；新增一般程式庫前核對授權、必要性、版本相容性與引入量，不能引入掃描框架。
- 政策唯一來源：security_verification 放在現有工作流政策文件，沿用 WorkflowConfig 與 core 的文字層改寫，不另建衝突設定。缺欄位預設 disabled，舊專案可正常載入；讀、編、存須保留其他未知欄位與未觸及的 YAML／註解。擴充 security 區塊時亦須保護未編輯的未知內容，不可用 typed 結構重序列化整份文件。Rust、Desktop 與 CLI 的語意相同，Local Repo 與 Remote Store 各走現有契約；不憑過時 Purpose 恢復已移除的 .speclink.yaml 舊政策層。
- 產出政策 UI：保留 locale、spec_locale、tdd、audit、worktree 的既有設定與適用條件；在 audit 下方新增「啟用 OWASP / CWE 安全審查」，預設 false，說明為「依選定的 OWASP / CWE 安全標準與規則，對 SDD 規格、原始碼及 API 進行初步安全審查。」關閉時隱藏子設定且不執行新檢查，不影響 audit、Verify 或 Archive。啟用時顯示範圍、標準版本、審查項目、安裝狀態與儲存操作。沿用 @speclink/ui、字型、深色模式及多語系；不另建 OWASP 專案設定頁。Remote 既有不顯示 worktree 的契約維持。
- 範圍為單選：off＝關閉／不指定範圍，web-api＝Web 應用程式 + API，api＝僅 API／後端服務，web＝僅 Web 應用程式。scope off 不等於主開關停用，仍可產生政策；目標資訊不足即未驗證。web-api 涵蓋前後端、API、輸入及權限；api 著重認證、物件授權、參數、資料庫與錯誤資訊；web 著重 HTML／XSS、Cookie、瀏覽器與前端輸入。
- 標準各自獨立：OWASP Top 10 可關閉或選 2021／2025；API Security Top 10 可關閉或選 2019／2023；ASVS 可關閉或選 4.0.3／5.0.0，啟用後選 L1／L2／L3；CWE 可關閉或選本機動態列出的已安裝版本。不得強迫 Top 10 使用者同時啟用 ASVS 或 CWE。允許單一、多個與全部關閉；ASVS 要求及等級按實際版本解析，無法檢查的要求不視為通過。官方對應的身分包含標準、版本、控制項 ID，不能跨版混同或自行編號；沒有可信對應就只保留 SL-SEC 自有 ID。
- 三個審查項目獨立：SDD 規格審查讀 proposal、design、specs、tasks，找認證、敏感操作授權、輸入型別／長度、安全錯誤處理、資料保護與安全驗收要求；缺少說明回報安全要求缺漏或需要人工確認。程式碼基本審查先支援 Rust、TypeScript、JavaScript，至少涵蓋疑似硬編碼憑證、SQL 拼接、不安全 HTML 輸出、命令拼接、詳細錯誤直出、不安全設定。API／參數邊界採靜態審查，涵蓋空值、型別、字串長度、數值上下限、非法參數、疑似缺授權與錯誤輸出，不呼叫受檢 API，不因缺少關鍵字而宣稱漏洞已證實。
- 自有規則與 audit：現有 audit 是 AI Agent 指引，不是離線引擎；沿用其 sharp-edges 的檢查觀點與工作流接點，不複製完整 audit 流程或改變其開關。自有規則格式採現有 serde 可處理的 JSON 或 YAML，包含唯一 ID、標題、類型、語言、嚴重程度、描述、檢查方式、可信版本化參考與修正建議。可個別停用規則，每條有正向、負向、誤判與不支援案例；文字模式命中一律是疑似風險，不可靠地證明 SQL Injection／XSS 已存在。
- 模組與介面：core 擁有政策型別、規則判斷、標準資料的純解析／模型、結果及報告語意；Host 擁有執行裁決、生命週期、取消與資源控制；授權檔案 I/O 沿既有檔案層，Desktop core 承接可獨立測試的桌面邏輯。CLI／Server／Desktop 使用共用 Host／Protocol 契約，Tauri 殼沿既有單行委派，不建立多層純轉送包裝。刪除新模組只會失去新安全審查，不應破壞既有 TDD／audit／Verify／Archive。
- 標準資料管理：在應用程式設定新增「安全標準資料」，包含標準與可用版本清單、明確觸發的下載／更新清單、已安裝版本、重新下載、刪除未使用版本、離線包匯入、來源授權檢視、進度、取消與下載／解析錯誤。資料存 SpecLink 應用程式資料目錄，不能落任意專案。Desktop 與 CLI 須共用標準資料解析與相容的資料定位契約，涵蓋現有跨平台及 portable 模式。
- 下載信任邊界：只接受官方來源或核准版本清單，優先 Markdown／CSV／JSON／XML，不依 HTML 固定版面解析；不得由任意遠端 URL 決定執行檔或程式碼。驗證檔案大小、完整性、格式結構、實際版本及要求／等級。SHA-256 必須區分實際內容指紋與可信清單提供的期望值，不能只算一次雜湊就宣稱來源已驗證。下載／解析／取消失敗不破壞舊版本，採暫存、驗證、原子替換。防路徑穿越、Zip Slip、非預期覆寫、巨大檔案與解壓膨脹、不安全 XML 外部實體及資料觸發命令；不支援的壓縮／離線包格式明確拒絕，不嘗試執行內容。
- 資料來源與授權：每份安裝資料保留 standard_id、version、source_url、license_name、license_url、attribution、downloaded_at、sha256、installed_status、source_format。以官方 OWASP/Top10、OWASP/API-Security、OWASP/ASVS 的版本化機器可讀資產及 MITRE 官方 CWE 資料為候選。已查 ASVS 與 API Security 官方聲明為 CC BY-SA 4.0，CWE 條款要求保留 MITRE 版權與授權文字；Top10 官方 repository LICENSE 與網站頁尾有不同授權標示，必須以實際取得的各版本資產與附帶聲明逐項核對，不能直接套全域授權假設。資料原文、解析／衍生資料及自有程式碼分開，不把第三方文件混成 MIT 授權程式；保留署名、變更說明及適用的相同方式分享要求。不確定可否重新散布時採使用者下載，不把自行下載當作免除使用義務。官方來源：https://github.com/OWASP/Top10/blob/master/LICENSE 、https://top10.owasp.org/2025/0x00_2025-Introduction/ 、https://github.com/OWASP/API-Security 、https://github.com/OWASP/ASVS 、https://github.com/OWASP/ASVS/releases/tag/v5.0.0_release 、https://cwe.mitre.org/data/downloads.html 、https://cwe.mitre.org/about/termsofuse.html 。
- 離線與 Remote：標準已安裝且所需政策／規格／程式碼已在本機可用時，查詢、基本規則及報告均離線執行。審查期間不自動下載、不呼叫外部網站、不傳送原始碼。Remote 政策沿用 editor／reader 授權與 expected revision 的 CAS；衝突保留輸入並按現有對照流程處理，不繞過授權、不排隊假裝儲存成功。程式碼掃描只讀使用者明確授權的本機 checkout；純 Remote 沒有 checkout 或離線缺政策／規格時保留未驗證原因，不把 Remote Store 當成原始碼或離線保證。
- AI 輔助為額外選用流程：與 SpecLink 內建離線規則分開標示模式與結果，沒有 AI 仍可執行基本審查；需連線者不稱離線，也不因啟用新政策自動呼叫 AI 或上傳原始碼。外部標準、專案文件與證據當成資料，不可轉成 Agent 指令。
- 掃描邊界：不執行受檢程式、Shell 指令或 package scripts；排除 .git、node_modules、target、dist、build、大型二進位與依專案慣例的生成物，可調整排除目錄。限制授權根目錄、符號連結逸出、目錄循環／深度、單檔大小、總讀取量與執行時間，支援取消。檔案錯誤、超限、逾時、解析／規則失敗及不支援語言須保留原因與覆蓋情況，不得回 PASS；任何未讀取內容不得被算成已檢查。
- 執行流程：讀目前政策 → 確認選定標準／版本及安裝可用性 → 確認實際目標 → 讀適用規格與原始碼 → 執行啟用規則 → 彙整疑似風險與未驗證項目 → 按標準及版本產報告。缺資料明示並提供前往系統設定操作，不背景下載。全部標準關閉時明示已啟用但未配置、提示至少選一個才能做標準化審查；另行啟用的自有基本規則可獨立執行並分區報告。
- 報告：提供 Desktop 可讀的「初步安全審查報告」，可輸出 JSON／Markdown。基本資訊含專案、時間、可得的 Git SHA、標準與版本、規則版本、範圍、執行模式、審查方式及檢查成功／失敗計數；每筆發現含規則 ID、標題、嚴重程度、confidence、檔案／行號、遮罩證據、風險說明、修正建議、版本化對應及確認狀態。至少支援 PASS／FINDING／NOT_VERIFIED／NOT_APPLICABLE／DISABLED，分清單項狀態與整體覆蓋，不把部分檢查合成全標準通過。敏感值在進入保存的報告、事件或 UI 前遮罩，不能只在畫面遮罩卻把完整 Token 寫進 JSON。
- SDD 整合：Propose／Design 依目標提示適用安全要求，不塞無關規格；Apply 保留 TDD 與 audit，各自開關獨立。生成技能原稿修改須連動資產版本、golden、assets.lock 與生成足跡。Verify 顯示已選標準、執行結果、疑似風險、未驗證與報告連結。安全報告是獨立資訊，不把疑似風險混入現有必修工單或新增封存阻擋；既有未結工單與過期章的守門仍保持。Security Gate 若日後需要，另開功能。
- 驗收例：開啟主開關，選 off／web-api／api／web 任一範圍，只啟用 Top 10 2025、其餘全部關閉並儲存；在應用程式設定由使用者下載對應資料後，關閉網路，對授權本機專案執行基本規則並取得有版本／規則／風險說明的報告，不安裝第三方掃描器。尚未安裝時回資料缺失，沒有發現的已執行規則才可 PASS；不支援／未完成的部分仍為未驗證。
- 開發順序與文件：單一完整變更依 Phase 1 架構分析 → Phase 2 proposal／design／specs／tasks → Phase 3 政策及 UI → Phase 4 標準資料管理 → Phase 5 基本規則與規格審查 → Phase 6 報告與 Verify → Phase 7 測試及文件。新增能力可按 security-review、security-standard-data 與鄰近既有 capability 邊界規劃，確切名稱由 proposal 核對。不做無關全面重構，不重置／覆蓋使用者修改。
- 測試與交付：Rust／TypeScript 單元與整合測試涵蓋舊設定、開關／各版本獨立、ASVS 關閉不影響 Top 10、原政策不變、未知欄位 round-trip；UI 顯示／隱藏、版本、文案、繁中、深色及重開還原；資料安裝、壞格式、版本不符、離線、缺資料、失敗保舊、取消及惡意封包／路徑；規則命中、負例、誤判、停用、未支援、遮罩與錯誤不 PASS；工作流停用無變化、啟用帶要求、Verify 報告、audit 獨立與不阻擋 Archive。依現有規範跑受影響 package 測試、相關 golden／CLI 契約、型別／格式及可用建置，最後任務含 scripts 守門；完整測試維持由 CI 守門，無法執行的項目記錄原因，不宣稱通過。交付架構摘要、實際檔案清單、已完成／未完成原因、來源授權、測試結果、限制、Desktop 操作及維護技術文件。
**Rationale**: 既有政策、Remote 授權與檔案改寫已能承載設定，重用它們可降低相容性風險；真正缺的是可無 AI 執行的有限離線規則、可信標準資料生命週期及保留覆蓋缺口的報告。把結果定位為疑似風險與未驗證資訊，能讓功能有用並保留原 SDD 的裁決方式，不讓簡單模式的誤判變成封存障礙。
**Rejected alternatives**:
- 引入第三方掃描器或額外安全 runtime — 違反使用者限制與離線部署目標。
- 只擴充 AI 技能、不做離線執行器 — 無 AI 就無法驗收，且無法保證結果可重現。
- 獨立 OWASP 專案設定頁／政策來源或各端各寫一套 — 造成重複入口、資料漂移與 Local／Remote 不一致。
- 強制 ASVS／CWE 作為 Top 10 前置條件 — 違反標準互相獨立的設定契約。
- 缺資料、未支援、解析失敗或規則沒命中即宣稱整體安全 — 未完成的檢查沒有通過依據。
- 將安全發現寫入既有 Verify 必修清單或新增 Security Gate — 既有工單會觸發封存守門，不符合初期提示性定位。
- 審查時自動下載、呼叫 API、執行專案 scripts 或上傳原始碼 — 越過離線與授權邊界。
- 用一份通用 OWASP 授權覆蓋全部版本，或把下載資料直接混入 MIT 程式碼 — 各來源與實際資產有各自授權義務。
- 將純 Remote Store 視為具備本機程式碼 — 現有契約沒有這項能力，會產生虛假的已驗證狀態。
**Deferred**:
- 各指定版本的精確下載資產、固定版本識別、完整性信任來源與逐包授權 — 在 design 依官方實際資料核實，不以浮動 master／latest 冒充固定版本；授權矛盾未解的資產不得先打包。
- 單檔／總量／深度／時間上限、離線包格式、進度取消機制與原子安裝細節 — 在 design 指定可測門檻，這些都是本功能交付內容。
- CLI 子指令、JSON 欄位、報告路徑／保存期限、Remote 報告分享與資料版本使用中判定 — 由 proposal／design 定義最小且一致的契約，不擅自擴大遠端原始碼能力或默默刪除需求。
- AI 輔助的細部入口與可提供的遮罩上下文 — 交 design 界定；AI 非基本驗收前提，既有 Agent 可用且使用者明確選用時才協助。
- Security Gate — 使用者已明定日後獨立擴充，本次不做。
**Capture to**: proposal | design | spec | tasks；若新增共用詞彙，於 LANGUAGE.md 記錄「初步安全審查」與既有「審查規則」「品質關卡」的差異，避免把初步審查稱為認證或全標準驗證。
**Next**: $speclink-propose --from-discussion owasp-cwe-preliminary-security-review
