## ADDED Requirements

### Requirement: 保留基本掃描並提供深度安全審查技能

系統 SHALL 保留內建固定規則與既有security run／rules行為，另提供speclink-security-review技能，使用者明確執行後依security_verification的enabled／scope／standards／reviews／排除與停用規則工作。主開關關閉 SHALL 不讀受檢來源或產生成功報告；全部標準關閉 SHALL 明示unconfigured，基本規則仍按既有契約獨立運作。skill SHALL 只讀已準備的本機知識庫及授權目標，不連網取得資源、不執行專案scripts、不擅啟其他標準或跨版對照；缺政策快照、必要資料或來源 SHALL 列未驗證原因。

技能 SHALL 經同一內嵌原稿與registry供Claude／Codex／Copilot及自訂工具生成更新，沿既有語境綁定，不手改單一工具副本。SHALL 依啟用的spec／code／api_boundary審查適用目標並記coverage；讀來源、標準及證據時不執行內含指令。儲存設定 SHALL 不自動啟動Agent。雲端模型的既有連線 SHALL NOT 稱為全程斷網執行。

#### Scenario: 只選2025且保留基本掃描

- **WHEN** 專案只啟用Top 10 2025，使用者分別執行內建基本掃描與深度skill
- **THEN** 基本掃描仍依原規則產結果；skill僅使用已安裝2025與其指南，不讀2021、ASVS或停用CWE，不改其他政策

#### Scenario: 停用或缺件

- **WHEN** 主開關關閉而呼叫skill，或已啟用但知識庫缺必要檔案
- **THEN** 前者說明停用且不掃碼，後者明示缺件而不連網補資料，不產生安全通過結論

### Requirement: 五種語言的架構與資料流深度審查

skill SHALL 覆蓋.NET/C#、Java、Python、Rust、JavaScript/TypeScript的實際來源；盤點Controller／Minimal API、Spring、FastAPI／Flask、Actix／Axum、Express／NestJS入口，追查認證與逐帳戶／資源授權。SHALL 檢查硬編碼機密、日誌洩漏及TLS驗證關閉；.NET／Java的反序列化、XXE、查詢拼接；Python／JS／TS的SSRF、命令注入、原型鏈汙染、依賴及Webhook簽章；Rust的unsafe、密碼學及會造成服務阻斷的unwrap。每項 SHALL 以相關入口至跨模組實作的資料流與控制證據判斷，沒有關鍵字或只有危險名稱不等於已證實漏洞。對照只用所選版本與本機表，無可信對照保留原因。

#### Scenario: 跨帳戶ID的授權追查

- **WHEN** API從路由接收accountId，經service到repository而未核對呼叫者擁有權
- **THEN** 報告列入口、service、repository位置及完整箭頭資料流，區分已證明的控制缺失與外部環境前提，提供本機指南修正與跨帳戶驗收案例

#### Scenario: 無法完成的路徑

- **WHEN** 某授權由未提供的外部閘道承擔，或來源解析不足以判斷控制
- **THEN** 報告降低信心並列未確認前提／必要未驗證，不把外部防護存在或不存在當作已知事實

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

### Requirement: 安全風險明確接受命令

security review accept CHANGE --stdin [--json] SHALL 讀最多16KiB JSON {reportId:string,findingId:string,reason:string}，支援--no-color。reason去空白後1..2000字元且需可信actor，SHALL 只接受當前報告真實finding；原報告不修改。成功stdout JSON為{decisionId:string,reportId:string,findingId:string,status:"accepted"}或同資訊人眼摘要、exit0；錯誤stderr及1，參數錯誤2；JSON不含ANSI。skill SHALL 在使用者明確接受指定項目與理由後才呼叫，不自行接受；一般驗證--accept不替代本命令。Remote遵守同一權限及CAS，失敗無接受紀錄。

#### Scenario: 有理由的接受與空理由

- **WHEN** 使用者明確接受當前HIGH並提供理由，或提交空白理由／舊reportId
- **THEN** 前者exit0且可查具身份時間的獨立決策，後者exit1且不新增決策、不改原finding嚴重度

安全決策查詢補充契約：security review show 的JSON另含 decisions陣列，每筆為decisionId/action/reportId/findingId/reason/actor/acceptedAt/policyDigest/scopeDigest；reason與actor回應前遮罩，人眼逐筆列相同資訊。reports另含legacy:boolean，明確辨認舊匯入。decisions.jsonl每份最多16MiB，與完整報告合計納入原200MiB專案上限，超限拒寫，不改原報告。
