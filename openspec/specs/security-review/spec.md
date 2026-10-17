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