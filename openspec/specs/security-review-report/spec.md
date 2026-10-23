# security-review-report Specification

## Purpose

定義初步安全審查報告的可追查、遮罩與可閱讀契約：保存當次政策、標準版本、規則、疑似風險及未驗證覆蓋，提供一致的 JSON／Markdown 與 Desktop 閱讀。報告是資訊性成果，不取代既有驗證工單、品質章或新增封存門檻。

## Requirements

### Requirement: 初步定位與五種結果狀態

UI 與報告 SHALL 使用「OWASP / CWE 初步安全審查」及英文對應名稱，不宣稱完整 OWASP 通過、無漏洞、正式認證或全 ASVS 符合。逐項 SHALL 有 PASS、FINDING、NOT_VERIFIED、NOT_APPLICABLE、DISABLED，非 PASS SHALL 有原因或發現。PASS SHALL 明示「只代表本次執行的指定規則沒有發現問題，不代表不存在其他漏洞」。不得將未驗證折成 PASS，部分規則 PASS 不提升為整個標準或專案通過。

#### Scenario: 同時有發現與未驗證

- **WHEN** SL-SEC-001 有發現，其他規則未命中，另有 ASVS 要求無檢查能力
- **THEN** 分別保留 FINDING、已執行規則 PASS、要求 NOT_VERIFIED，不產生全標準通過標章，畫面與兩種輸出都顯示 PASS 限制


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
### Requirement: 版本化 JSON 報告模型

內建基本掃描JSON SHALL 維持formatVersion=1、camelCase；深度skill報告 SHALL 為formatVersion=2且保留以下共同欄位，並新增knowledgeDigest、scopeDigest、builtinReportId、actor及深度finding欄位；既有v1報告仍可讀。共同欄位包含包含 reportId、projectKey、projectName、changeName（string|null）、reviewedAt、commitSha（string|null）、policyDigest、policyRevision（number|null）、preparedAt（string|null）、standards（array）、ruleVersion、scope、executionMode、reviewMethods（array）、complete（boolean）、configurationState、counts、findings、checks、coverage、disclaimer、sourceLicenses。standard SHALL 有 standardId／version／level／dataSha256／sourceRevision／installedStatus；counts SHALL 含 checksSucceeded、checksFailed、findings、notVerified、notApplicable、disabled 的數字。commitSha 不可取得 SHALL 為 null；實際 dirty 內容 SHALL 以每檔 contentDigest 記錄，不宣稱與 HEAD 完全一致。coverage SHALL 列 inspectedFiles、skippedFiles、failedItems、bytesRead、remainingTargets、reasons。

#### Scenario: 有 dirty 檔與無 Git

- **WHEN** 掃描含未提交修改的目標或無 Git 的目錄
- **THEN** 報告有當次內容 digest，前者 SHA 只標 HEAD 參考、後者 commitSha=null，時間、標準、規則版本與覆蓋仍完整


<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->

---
### Requirement: 疑似風險與遮罩證據

每筆 finding SHALL 有 findingId、ruleId、title、severity、confidence、path、line、evidence、description、remediation、references、confirmationStatus、kind。severity SHALL 為 low／medium／high／critical，confidence 為 low／medium／high；初始 confirmationStatus=needs_review，kind=potential_risk 或 missing_requirement。references SHALL 含標準、版本、控制項與來源；無可信對應 SHALL 留自有 ID。報告保存、日誌、錯誤、進度與 UI 之前 SHALL 遮罩憑證；001 字面值全部遮罩，其他證據替換字串／Token／URL 憑證，無法安全切割整段 [REDACTED]，最多400字元。Markdown 與 UI SHALL 不執行證據 HTML 或來源指令。

#### Scenario: 秘密不能只在畫面遮罩

- **WHEN** 測試來源含唯一秘密值，產生 JSON／Markdown、錯誤與進度
- **THEN** 保存檔、stdout、stderr、IPC、UI、複製摘要皆找不到該秘密值，仍能看規則、檔案、行號及修正建議

#### Scenario: 規格缺漏與確定漏洞區分

- **WHEN** 規格沒有寫敏感功能授權方式
- **THEN** finding.kind 為 missing_requirement、confirmationStatus 為 needs_review，不顯示已確認原始碼漏洞


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
### Requirement: 本機報告保存與查詢

未指定change的基本掃描報告 SHALL 維持存應用程式資料目錄security/reports/<project-key>/<report-id>/；指定change的基本掃描及skill報告 SHALL 經Store存該提案checks/security/<report-id>/report.json與report.md並在checks/index.json列明，不另存第二份權威報告。JSON／Markdown 同一模型生成並原子顯露，不寫 verify.md／review.md／品質章。latest 索引失敗 SHALL 保留可按 ID 查詢的完整報告並附警告。SHALL 不自動刪舊報告；1,000份或200 MiB上限達到時明確 storage_limit，不寫半套或覆蓋舊報告。reportId SHALL 只接受合法 ULID；未知／穿越 ID 不得讀任意檔案。Remote的未指定change報告仍留裝置；指定change的已遮罩報告 SHALL 經明確提交及editor授權／expected revision發布到該變更，不傳原始碼檔案，失敗保留本機暫存且不宣稱server已驗收。既有裝置報告 SHALL 唯讀可查，明確legacy匯入匹配變更後仍不作新驗收證據。

#### Scenario: 保存中斷與錯誤 ID

- **WHEN** JSON／Markdown 寫到一半中斷，或查詢 ../../secret
- **THEN** 前者沒有半套可見報告且舊 latest 可讀，後者拒絕且不讀根外檔案

#### Scenario: 封存後報告仍可查

- **WHEN** 原變更封存後以 reportId 查詢
- **THEN** 報告仍有當時 changeName、版本、來源與結果，不要求重新開啟驗證工單

#### Scenario: 明確目標的提案內報告

- **WHEN** 對CHANGE執行基本掃描或提交skill報告，再由Desktop開啟該提案檢查紀錄
- **THEN** 同一reportId的JSON與Markdown位於checks/security，含完整遮罩資訊，無裝置目錄的第二份權威結果；Remote明確提交成功後另一位reader可讀


<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->

---
### Requirement: Desktop 初步安全審查入口

Desktop SHALL 在變更的 Verify 區顯示已選標準、執行結果、疑似風險、未驗證與報告連結，沒有 Verify 工單時仍可顯示安全資訊，不為此開工單。SHALL 有明確執行、進度、取消、缺資料前往應用程式設定、JSON／Markdown 輸出及複製遮罩摘要。專案切換 SHALL 依 projectKey／jobId 隔離事件；檔案跳轉 SHALL 重驗根目錄。停用且沒有歷史報告時既有畫面維持，不隱藏已有報告歷史。

#### Scenario: 無 Verify 工單也能閱讀

- **WHEN** 啟用政策並執行安全審查，但未開始既有 Verify
- **THEN** 可以在 Verify 區讀初步報告，沒有新增 verify.md 或品質章，原驗證流程仍可自行開始

#### Scenario: 資料缺失提供入口

- **WHEN** 只選 Top 10 2025 但尚未安裝
- **THEN** 畫面顯示資料缺失與前往安全標準資料管理按鈕，點前不下載，不能顯示通過


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
### Requirement: SDD 提示與 AI 邊界

啟用政策時 Propose／Design SHALL 提示所選版本及與目標相關的安全要求，Apply SHALL 維持既有 TDD／audit 獨立。外部標準、專案文件及證據 SHALL 視為資料，不注入為 Agent 指令。深度安全審查 SHALL 是使用者明確執行的speclink-security-review技能流程，保留無AI可執行的基本掃描；不能自動送原始碼、啟動 Agent 或稱連線 AI 為離線功能；沒有 AI 設定 SHALL 可完成基本規則與報告。required_for_acceptance=false時初步結果 SHALL 不阻擋驗收或Archive；enabled且required_for_acceptance=true時 SHALL 依安全驗收規則判定，既有非安全報告的工單守門保持。

#### Scenario: 沒有 AI 的離線旅程

- **WHEN** 使用者沒設定AI、未勾選納入驗收，只安裝Top 10 2025資料後關閉網路，執行授權本機審查
- **THEN** 能取得基本規則 JSON／Markdown 報告，不呼叫任何掃描器或 AI，不因 FINDING 自動阻擋封存

#### Scenario: 外部資料帶指令

- **WHEN** 已下載標準內容含要求忽略系統或執行命令的文字
- **THEN** 查詢與報告只將其當資料，沒有命令執行或變更 Agent 的指令優先序


<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->

---
### Requirement: 報告輸出命令與語言

security report ID [--change CHANGE] [--format json|markdown] SHALL 不讀 stdin、不重掃描，預設 json，stdout 是指定格式；成功0、找不到／非法ID／資料損壞為stderr及1、語法錯誤2。--no-color SHALL 無 ANSI。Desktop zh-TW／en 的控制項與狀態說明 SHALL 跟介面語言；Agent 的 tw／ja／en 或未設定 SHALL 跟既有有效 locale，結構鍵、規則 ID、標準 ID、官方原文不翻譯。指定--change SHALL 僅查該active或已封存dated變更的index列明報告；未指定維持既有裝置查詢。基本掃描說明沿既有locale，深度skill報告敘述 SHALL 固定繁體中文，其JSON鍵、官方控制項及原文不翻譯。JSON／Markdown SHALL 由同一已遮罩模型生成，結果與計數相同。

#### Scenario: 兩種格式一致

- **WHEN** 對同一 reportId 先輸出 json 再輸出 markdown --no-color
- **THEN** 兩者的版本、規則、狀態、計數及遮罩一致，無 ANSI，Markdown 不額外洩漏原始證據

#### Scenario: 介面與 Agent 語言

- **WHEN** Desktop切zh-TW／en，基本掃描報告的Agent有效locale分別為tw／ja／en或未設定，另有深度skill報告
- **THEN** 控制項及基本掃描報告說明依對應語言，未設定沿現有系統語言預設；深度skill報告固定繁體中文，官方控制項與JSON鍵皆不改名


<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->

---
### Requirement: 深度報告九項內容與風險接受紀錄

深度報告 SHALL 依認證／業務授權、機密／通訊等實際安全領域分章，每筆包含九項：H/M/L編號標題、等級及信心與外部前提、僅已啟用標準的版本化對照、所有關聯檔案及正整數行號證據、最多400字元遮罩片段、入口至外部呼叫的箭頭資料流及控制缺口、已證明與環境防護區分的影響、引用本機指南且用SHALL的修正、可執行驗收案例與通過基準。無可信對照 SHALL 明示而不製造ID，機密省略 SHALL 註明原始碼中的金鑰值在此省略；保存／事件／stdout／IPC／Markdown之前皆須遮罩。SHALL NOT 複製無來源的cite標記作引用。

風險接受 SHALL 另以decisions.jsonl保存decisionId／reportId／findingId／action=accepted／reason／actor／acceptedAt／policyDigest／scopeDigest，不改finding嚴重度或原報告；reason去空白後1..2000字元，身份由可信呼叫者上下文提供。舊報告、未知finding、空理由、缺身份或digest過期 SHALL 拒絕，LOW不要求接受。兩種模式皆可查詢接受理由。

#### Scenario: 九項證據與機密遮罩

- **WHEN** 深度審查找到跨帳戶授權風險並引用含真實Token的來源
- **THEN** JSON與Markdown均有九項內容及所有入口位置，資料流列出缺少的帳戶所有權檢查，所有保存／事件／輸出均無Token原值

#### Scenario: 接受理由可回查且不改原報告

- **WHEN** 使用者明確接受目前HIGH finding並提供非空理由，再讀取原報告及接受紀錄
- **THEN** 原finding仍是HIGH，另有具時間、身份、報告及理由的接受紀錄；舊報告或空理由提交失敗且無文件效果

<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->