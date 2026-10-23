# security-review Specification

## Purpose

定義 SpecLink 自有的離線初步安全審查：對明確授權的本機目標執行有資源限制、可取消的規格／程式碼／API 靜態規則，保留疑似風險與未驗證原因。此能力不執行受檢程式、不依賴 AI 或外部掃描引擎，也不裁定正式安全認證或封存准入。

## Requirements

### Requirement: 審查主開關與獨立自有規則

審查 SHALL 使用當次專案 security_verification；enabled=false 時 SHALL 不讀受檢來源、不下載資料，結果為 DISABLED。scope=off SHALL 只表示不自動限定 Web／API，不關閉主開關。三種 reviews SHALL 各自控制規格、程式碼、API 邊界審查。全部標準關閉時 SHALL 明示 unconfigured，標準化部分不執行；另行勾選的自有基本規則 SHALL 可執行，其結果不冒充 OWASP／ASVS 已驗證。

#### Scenario: 舊專案沒有安全政策

- **WHEN** 在沒有 security_verification 的舊專案執行 security run --json
- **THEN** stdout 報告的 configurationState 為 disabled、檢查為 DISABLED、來源讀取數為零，exit 0，既有 audit 與 TDD 設定不變

#### Scenario: 全部標準關閉但自有規則啟用

- **WHEN** enabled=true、所有標準 enabled=false 且 reviews.code=true，執行 security run
- **THEN** 顯示已啟用但未配置及至少選一個標準才能執行標準化審查的提示，另列自有程式規則結果，不顯示任何標準通過


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
### Requirement: 明確目標與離線 Remote 快照

Local 審查 SHALL 以開啟專案或 --root 指定且已授權的 canonical root 為邊界。Remote 的 security prepare SHALL 是獨立明確操作，經既有連線取得政策與選定 change 的 SDD 文件及 revision／digest，完整驗證後原子保存準備快照；SHALL 不取送原始碼。security run SHALL 不連網，Remote SHALL 僅讀同 origin／scope／checkout 的完整快照並揭露 preparedAt／policyRevision。缺快照、缺 checkout 或未知 change SHALL 記明未驗證原因，不掃其他專案、不自動準備、不修改 Context Projection。Remote 準備遇 401／離線時 SHALL 保留舊快照並回錯。

#### Scenario: 已準備 Remote 在離線執行

- **WHEN** 使用者先準備指定 scope／change／checkout，關閉網路後執行 security run --root 對應目錄 --json
- **THEN** 沒有網路請求，使用準備的政策／規格與當次本機原始碼，報告標記 preparedAt 與 policyRevision，不宣稱知道遠端最新版本

#### Scenario: 純 Remote 沒有原始碼

- **WHEN** Remote 專案沒有選取授權本機 checkout，執行基本程式碼審查
- **THEN** 程式碼項目為 NOT_VERIFIED 並指出 source_unavailable，exit 1，不掃 server 或另一個本機根

#### Scenario: 準備資料來源錯誤

- **WHEN** 準備快照屬於另一個 origin 或準備時認證失效
- **THEN** 錯誤指出 binding_mismatch 或認證失效，現有快照不變，run 不用錯誤身分的資料繼續


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
### Requirement: 有界讀取與可調排除

掃描 SHALL 不執行受檢程式、Shell、package scripts 或生成判斷指令。預設 SHALL 排除 .git、node_modules、target、dist、build、.speclink/context、既有生成標記檔與二進位，並支援根相對 exclude_dirs；絕對／點點／盤符／空排除值 SHALL 拒絕。SHALL 不跟隨 symlink／reparse point，拒讀 device／FIFO／socket，核對開啟後的根與一般檔身分，不能只做先檢查再開檔。每項跳過 SHALL 有原因且不算檢查成功。

#### Scenario: 排除目錄與一般檔

- **WHEN** 目標含 node_modules／target 與使用者排除的 fixtures，其他 src 有支援來源
- **THEN** 只檢查 src，coverage 列排除原因，排除內容的 secrets 不出現在結果或日誌

#### Scenario: 連結逸出與特殊檔案

- **WHEN** 專案內 symlink 指向根外秘密、reparse point 指向另一根，或包含 FIFO
- **THEN** 不讀其內容，記錄 untrusted_link／skipped_link／special_file 的未驗證原因，沒有根外秘密輸出且不因 FIFO 掛住


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
### Requirement: 資源上限與取消保留覆蓋

SHALL 限單檔 1 MiB、總量 100 MiB、10,000 檔、深度 32、60 秒、每檔 100 findings、總計 10,000 findings。單檔／深度超限 SHALL 記跳過，全域上限 SHALL 停止並保留未處理目標與 complete=false。Desktop 取消及 CLI Ctrl+C SHALL 在兩秒內回應，保存安全的部分報告，exit 130；規則／解析／I/O 失敗 SHALL 不轉成 PASS。同程序第二個掃描 SHALL 回 busy，事件 SHALL 綁開始時 projectKey。

#### Scenario: 上限邊界

- **WHEN** 讀取 1,048,577 bytes 的來源檔，或掃描到第 10,001 個檔案
- **THEN** 前者記 file_too_large，後者停止並記 file_limit／remainingTargets，對應未驗證且 complete=false、exit 1

#### Scenario: 取消後切換專案

- **WHEN** 在專案 A 開始掃描後切到 B 並取消 A 的工作
- **THEN** 兩秒內停止，部分報告與事件仍屬 A，B 不顯示 A 的結果，沒有 PASS 取代未完成項目


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
### Requirement: 六類基本規則與可測規則清單

內建規則 SHALL 有唯一 ID、title、type、severity、languages、description、references、check 與 remediation，固定 selector 不執行下載或專案提供的程式。Rust／TypeScript／JavaScript SHALL 至少有 SL-SEC-001 憑證、002 SQL 拼接、003 HTML 輸出、004 命令拼接、005 詳細錯誤直出、006 不安全設定的適用矩陣；JSON／YAML 僅補充設定檢查。SHALL 支援 disabled_rules 個別停用；每規則／適用語言 SHALL 有命中、安全、誤判與未支援案例。文字模式結果 SHALL 是 potential_risk／needs_review，不能宣稱漏洞已證實。

#### Scenario: SQL 拼接與參數化區分

- **WHEN** JS query 接插入 userId 的 SQL template，另一份 query 使用固定 SQL 與參數陣列
- **THEN** 前者有 SL-SEC-002 疑似風險與修正建議，後者不因參數陣列而命中，兩者都不被描述為已證實可利用

#### Scenario: 停用規則與未支援語言

- **WHEN** disabled_rules 含 SL-SEC-001，目標另有 Python 程式
- **THEN** 001 為 DISABLED，Python 為 unsupported_language／NOT_VERIFIED，其他規則仍執行，不能把 Python 沒有命中記 PASS


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
### Requirement: 規格安全要求與 API 靜態邊界

規格審查 SHALL 在 proposal／design／specs／tasks 中，以目標證據檢視認證、授權、輸入型別／長度、敏感資料、錯誤處理及安全驗收六項；缺說明 SHALL 分類 missing_requirement 或 needs_review，不分類已確認漏洞。API 邊界 SHALL 檢查 OpenAPI 3.0／3.1 本機 JSON／YAML 的空值、型別、長度、數值上下限、非法參數、授權與錯誤輸出，僅解析內部引用，SHALL 不讀外部 $ref 或呼叫 endpoint。沒有可辨識目標 SHALL 留 NOT_VERIFIED；NOT_APPLICABLE 必須有具體適用性理由。

#### Scenario: API 規格缺長度要求

- **WHEN** API 規格定義字串參數但沒有長度限制
- **THEN** 報告為安全要求缺漏／needs_review，定位該規格，不聲稱後端原始碼已證實缺少驗證

#### Scenario: 沒有 API 目標或含外部引用

- **WHEN** 專案無可辨識 API，或 OpenAPI 的 $ref 指向外部網址
- **THEN** 前者七個邊界項目回目標不足的 NOT_VERIFIED，後者回未解析外部引用，沒有網路或根外檔案讀取


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
### Requirement: 版本化標準對應與缺資料狀態

所有標準引用 SHALL 含 standardId、version、controlId 及可信 sourceUrl。CWE 為分類與說明，不是全部弱點的測試器；標準未啟用不作對應。只啟用 Top 10 SHALL 不要求 ASVS／CWE 安裝。缺選定資料 SHALL 記 missing_data 並提供資料管理入口，不下載也不 PASS。ASVS SHALL 展開選定版本及等級的要求，無測試能力者逐項 NOT_VERIFIED；無可靠對應時 SHALL 只保留自有規則 ID，不能編造控制項號。

#### Scenario: 僅 Top 10 2025 可獨立審查

- **WHEN** 只啟用並安裝 Top 10 2025，其他三標準停用
- **THEN** 基本規則與該版的可信對應可執行，不要求其他資料，不使用 2021 的 ID 替代 2025

#### Scenario: 缺資料與 ASVS 不支援項

- **WHEN** 選定標準未安裝，或選 ASVS L2 但某要求無對應規則
- **THEN** 前者 missing_data，後者該版本／等級要求 NOT_VERIFIED，均保留原因，不成為標準通過


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
### Requirement: 離線審查 CLI 契約

security run [--root PATH] [--change NAME] [--json]、prepare [--root PATH] [--change NAME] [--json]、rules [--json] SHALL 不讀 stdin，支援全域 --no-color。run stdout SHALL 是單一遮罩 Report JSON 或人眼摘要含 reportId、計數與 PASS 限制；完整且無未驗證的執行即使有 FINDING 仍 exit 0，有未完成／NOT_VERIFIED exit 1，取消 130。起始授權／設定失敗 SHALL 只有 stderr，無偽造報告。prepare stdout SHALL 是 {projectKey:string,preparedAt:string,policyRevision:number|null,documentCount:number,checkoutAvailable:boolean}；rules stdout SHALL 是 {ruleVersion:string,rules:array}，每筆含規則必備欄位，成功 0、錯誤 1。進度 SHALL 只到 stderr，--no-color SHALL 無 ANSI；clap 語法錯誤維持 2。

#### Scenario: JSON 含風險仍可完整完成

- **WHEN** 授權範圍、資料與所有啟用檢查均可完成且找到疑似風險，執行 security run --json --no-color
- **THEN** stdout 只有完整 Report JSON、無 ANSI／進度文字，exit 0；stderr 可有進度，風險數不作 Security Gate

#### Scenario: 錯誤不能輸出成功摘要

- **WHEN** 指定不存在的 change、無效根目錄或壞設定
- **THEN** exit 1，stderr 指出原因，不掃其他目標，也沒有 stdout 的 PASS 摘要


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
### Requirement: 保留基本掃描並提供深度安全審查技能

系統 SHALL 保留內建固定規則與既有security run／rules行為，另提供speclink-security-review技能，使用者明確執行後依security_verification的enabled／scope／standards／reviews／排除與停用規則工作。主開關關閉 SHALL 不讀受檢來源或產生成功報告；全部標準關閉 SHALL 明示unconfigured，基本規則仍按既有契約獨立運作。skill SHALL 只讀已準備的本機知識庫及授權目標，不連網取得資源、不執行專案scripts、不擅啟其他標準或跨版對照；缺政策快照、必要資料或來源 SHALL 列未驗證原因。

技能 SHALL 經同一內嵌原稿與registry供Claude／Codex／Copilot及自訂工具生成更新，沿既有語境綁定，不手改單一工具副本。SHALL 依啟用的spec／code／api_boundary審查適用目標並記coverage；讀來源、標準及證據時不執行內含指令。儲存設定 SHALL 不自動啟動Agent。雲端模型的既有連線 SHALL NOT 稱為全程斷網執行。

#### Scenario: 只選2025且保留基本掃描

- **WHEN** 專案只啟用Top 10 2025，使用者分別執行內建基本掃描與深度skill
- **THEN** 基本掃描仍依原規則產結果；skill僅使用已安裝2025與其指南，不讀2021、ASVS或停用CWE，不改其他政策

#### Scenario: 停用或缺件

- **WHEN** 主開關關閉而呼叫skill，或已啟用但知識庫缺必要檔案
- **THEN** 前者說明停用且不掃碼，後者明示缺件而不連網補資料，不產生安全通過結論

<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->

---
### Requirement: 五種語言的架構與資料流深度審查

skill SHALL 覆蓋.NET/C#、Java、Python、Rust、JavaScript/TypeScript的實際來源；盤點Controller／Minimal API、Spring、FastAPI／Flask、Actix／Axum、Express／NestJS入口，追查認證與逐帳戶／資源授權。SHALL 檢查硬編碼機密、日誌洩漏及TLS驗證關閉；.NET／Java的反序列化、XXE、查詢拼接；Python／JS／TS的SSRF、命令注入、原型鏈汙染、依賴及Webhook簽章；Rust的unsafe、密碼學及會造成服務阻斷的unwrap。每項 SHALL 以相關入口至跨模組實作的資料流與控制證據判斷，沒有關鍵字或只有危險名稱不等於已證實漏洞。對照只用所選版本與本機表，無可信對照保留原因。

#### Scenario: 跨帳戶ID的授權追查

- **WHEN** API從路由接收accountId，經service到repository而未核對呼叫者擁有權
- **THEN** 報告列入口、service、repository位置及完整箭頭資料流，區分已證明的控制缺失與外部環境前提，提供本機指南修正與跨帳戶驗收案例

#### Scenario: 無法完成的路徑

- **WHEN** 某授權由未提供的外部閘道承擔，或來源解析不足以判斷控制
- **THEN** 報告降低信心並列未確認前提／必要未驗證，不把外部防護存在或不存在當作已知事實

<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->

---
### Requirement: 深度審查提交與查詢命令

CLI SHALL 提供security review add-round CHANGE --stdin [--json]與security review show CHANGE [--json]，支援全域--no-color。add-round讀至多4MiB單一JSON，必含formatVersion=2、changeName:string、policyDigest:string、knowledgeDigest:string、scopeDigest:string、builtinReportId:string|null、complete:boolean、targets:string[]、checks:array及findings:array；Host補齊時間、actor、projectKey與reportId。check SHALL 含checkId:string、status:string、reason:string、evidencePaths:string[]、resolvesCheckIds:string[]；finding SHALL 含findingId/title/severity/confidence/confidenceReason:string、references/evidence:array及codeSnippet/dataFlow/impact/remediation/acceptance:string。references含standardId/version/controlId/localPath:string，evidence含path:string、line:number、endLine:number、function:string。severity為low|medium|high|critical，最多10000findings，片段最多400字元；ID重名、越界證據、錯版本、非啟用標準、未知check或無九項內容 SHALL 拒絕且不寫檔。行號 SHALL 為正整數且endLine不小於line。legacy匯入 SHALL 另接受只有legacyImportReportId:string的輸入形式，匹配舊專案／變更後標legacy，不能取代v2驗收。

add-round成功JSON SHALL 為{reportId:string,changeName:string,relativePath:string,complete:boolean}；show不讀stdin，回{changeName:string,reports:[{reportId:string,executionMode:string,relativePath:string,complete:boolean}],acceptance:{required:boolean,status:string,blockingFindings:number,notVerified:number,reasons:string[]}}，status為reference|passed|blocked|stale。人眼stdout SHALL 列同資訊及報告路徑，成功0、操作或輸入錯誤stderr及1、參數錯誤2；JSON不混進度／ANSI。查未知變更 SHALL 失敗，不回空成功報告。

Remote提交 SHALL 使用editor／owner及expected revision，401／403／離線／CAS衝突無server寫入、保留checkout遮罩暫存供重試，不能把未提交報告當已驗收；reader可查詢，沒有本機checkout不能送另一專案來源。

#### Scenario: 提交及查詢人眼與JSON

- **WHEN** 提交同專案且通過資料驗證的v2報告，再以show --json --no-color及人眼查詢
- **THEN** 提交exit0且回合法ULID／相對路徑，show的camelCase欄位及型別符合契約，人眼同列結果與PASS限制，皆無ANSI或原始秘密

#### Scenario: 越界引用與Remote衝突

- **WHEN** 提交../證據、未啟用ASVS對照、未知change，或Remote提交遇revision衝突
- **THEN** exit1、stderr明示原因且無半套索引／報告，衝突保留本機暫存而不覆蓋他人結果

<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->

---
### Requirement: 安全驗收的等級完整性與新鮮度判定

只有enabled及required_for_acceptance皆true時 SHALL 新增安全驗收要求，否則結果為reference而不阻擋原流程。required模式 SHALL 要求相同project/change、完整且未過期的deep-skill報告與啟用基本檢查結果；必要集合為啟用reviews的適用目標及所選版本／ASVS等級控制項，停用規則不列必要，NOT_APPLICABLE須有證據。基本結果的NOT_VERIFIED可由skill以resolvesCheckIds逐項附證據補齊，原始結果不改寫，不能整批假定已驗證。未執行、必要資料缺失、complete=false、必要未驗證、全部標準／reviews關閉 SHALL 不通過。 深度complete=false一律阻擋；基本報告若僅因必要NOT_VERIFIED而complete=false，可在每一項均有本次帶證據的resolvesCheckIds後，將組合證據判為完整，但不得改原報告。基本取消、預算／讀取失敗或資料缺件不能由複核豁免；不屬所選程式／規格目標的二進位等資產仍保留原coverage，不列為必要程式檢查。

未處理CRITICAL／HIGH／MEDIUM SHALL 阻擋，已修復須當前複核證據，或有綁當前reportId／findingId／policyDigest／scopeDigest的明確接受理由；LOW不阻擋。政策實質審查欄位、目標檔案清單／digest或知識資料代次改變 SHALL 判stale，舊接受不得套新報告；archive保留設定不使報告過期。Local核對新增／刪除來源，Remote核對server政策／SDD及當次checkout提交的來源指紋，SHALL 明示client-attested，不宣稱server獨立讀驗本機來源。缺當次來源核對 SHALL 列source_freshness_unverified。

#### Scenario: 風險等級與必要資料矩陣

- **WHEN** required模式依序取得只有LOW且檢查完整、未處理HIGH、已明確接受HIGH、零風險但缺必要資料的報告
- **THEN** 結果依序為passed、blocked、passed、blocked；HIGH接受理由可查，缺資料不因零finding而通過

#### Scenario: 程式碼或政策變更

- **WHEN** 已接受風險後新增來源檔、修改受檢內容或切換標準版本，再要求驗收
- **THEN** 回stale或blocked並列原因，舊報告與接受紀錄不改寫，必須重新審查及按新報告接受

##### Example: 納入驗收的具體矩陣

| required生效 | 審查完整 | 風險與處置 | 結果 |
| --- | --- | --- | --- |
| false | false | 未處理HIGH | reference，不新增驗收阻擋 |
| true | true | 只有LOW | passed |
| true | true | 一筆未處理MEDIUM | blocked |
| true | true | 一筆未處理HIGH | blocked |
| true | true | 一筆CRITICAL有本報告明確接受理由 | passed |
| true | false | 零finding但缺必要資料 | blocked |
| true | true | HIGH接受來自舊scopeDigest | stale／blocked |

<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->

---
### Requirement: 安全風險明確接受命令

security review accept CHANGE --stdin [--json] SHALL 讀最多16KiB JSON {reportId:string,findingId:string,reason:string}，支援--no-color。reason去空白後1..2000字元且需可信actor，SHALL 只接受當前報告真實finding；原報告不修改。成功stdout JSON為{decisionId:string,reportId:string,findingId:string,status:"accepted"}或同資訊人眼摘要、exit0；錯誤stderr及1，參數錯誤2；JSON不含ANSI。skill SHALL 在使用者明確接受指定項目與理由後才呼叫，不自行接受；一般驗證--accept不替代本命令。Remote遵守同一權限及CAS，失敗無接受紀錄。

#### Scenario: 有理由的接受與空理由

- **WHEN** 使用者明確接受當前HIGH並提供理由，或提交空白理由／舊reportId
- **THEN** 前者exit0且可查具身份時間的獨立決策，後者exit1且不新增決策、不改原finding嚴重度

安全決策查詢補充契約：security review show 的JSON另含 decisions陣列，每筆為decisionId/action/reportId/findingId/reason/actor/acceptedAt/policyDigest/scopeDigest；reason與actor回應前遮罩，人眼逐筆列相同資訊。reports另含legacy:boolean，明確辨認舊匯入。decisions.jsonl每份最多16MiB，與完整報告合計納入原200MiB專案上限，超限拒寫，不改原報告。

<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->