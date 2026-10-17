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

JSON SHALL 為 formatVersion=1、camelCase，包含 reportId、projectKey、projectName、changeName（string|null）、reviewedAt、commitSha（string|null）、policyDigest、policyRevision（number|null）、preparedAt（string|null）、standards（array）、ruleVersion、scope、executionMode、reviewMethods（array）、complete（boolean）、configurationState、counts、findings、checks、coverage、disclaimer、sourceLicenses。standard SHALL 有 standardId／version／level／dataSha256／sourceRevision／installedStatus；counts SHALL 含 checksSucceeded、checksFailed、findings、notVerified、notApplicable、disabled 的數字。commitSha 不可取得 SHALL 為 null；實際 dirty 內容 SHALL 以每檔 contentDigest 記錄，不宣稱與 HEAD 完全一致。coverage SHALL 列 inspectedFiles、skippedFiles、failedItems、bytesRead、remainingTargets、reasons。

#### Scenario: 有 dirty 檔與無 Git

- **WHEN** 掃描含未提交修改的目標或無 Git 的目錄
- **THEN** 報告有當次內容 digest，前者 SHA 只標 HEAD 參考、後者 commitSha=null，時間、標準、規則版本與覆蓋仍完整


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
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

報告 SHALL 存應用程式資料目錄的 security/reports/<project-key>/<report-id>/，JSON／Markdown 同一模型生成並原子顯露，不寫 verify.md／review.md／品質章。latest 索引失敗 SHALL 保留可按 ID 查詢的完整報告並附警告。SHALL 不自動刪舊報告；1,000份或200 MiB上限達到時明確 storage_limit，不寫半套或覆蓋舊報告。reportId SHALL 只接受合法 ULID；未知／穿越 ID 不得讀任意檔案。Remote 報告仍留裝置，不自動上傳，輸出可由使用者手動分享。

#### Scenario: 保存中斷與錯誤 ID

- **WHEN** JSON／Markdown 寫到一半中斷，或查詢 ../../secret
- **THEN** 前者沒有半套可見報告且舊 latest 可讀，後者拒絕且不讀根外檔案

#### Scenario: 封存後報告仍可查

- **WHEN** 原變更封存後以 reportId 查詢
- **THEN** 報告仍有當時 changeName、版本、來源與結果，不要求重新開啟驗證工單


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
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

啟用政策時 Propose／Design SHALL 提示所選版本及與目標相關的安全要求，Apply SHALL 維持既有 TDD／audit 獨立。外部標準、專案文件及證據 SHALL 視為資料，不注入為 Agent 指令。AI 輔助 SHALL 是使用者明確選用的遮罩摘要／既有技能流程，不能自動送原始碼、啟動 Agent 或稱連線 AI 為離線功能；沒有 AI 設定 SHALL 可完成基本規則與報告。初步結果 SHALL 不預設阻擋 Archive，既有非安全報告的工單守門保持。

#### Scenario: 沒有 AI 的離線旅程

- **WHEN** 使用者沒設定 AI，只安裝 Top 10 2025 資料後關閉網路，執行授權本機審查
- **THEN** 能取得基本規則 JSON／Markdown 報告，不呼叫任何掃描器或 AI，不因 FINDING 自動阻擋封存

#### Scenario: 外部資料帶指令

- **WHEN** 已下載標準內容含要求忽略系統或執行命令的文字
- **THEN** 查詢與報告只將其當資料，沒有命令執行或變更 Agent 的指令優先序


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
### Requirement: 報告輸出命令與語言

security report ID [--format json|markdown] SHALL 不讀 stdin、不重掃描，預設 json，stdout 是指定格式；成功0、找不到／非法ID／資料損壞為stderr及1、語法錯誤2。--no-color SHALL 無 ANSI。Desktop zh-TW／en 的控制項與狀態說明 SHALL 跟介面語言；Agent 的 tw／ja／en 或未設定 SHALL 跟既有有效 locale，結構鍵、規則 ID、標準 ID、官方原文不翻譯。JSON／Markdown SHALL 由同一已遮罩模型生成，結果與計數相同。

#### Scenario: 兩種格式一致

- **WHEN** 對同一 reportId 先輸出 json 再輸出 markdown --no-color
- **THEN** 兩者的版本、規則、狀態、計數及遮罩一致，無 ANSI，Markdown 不額外洩漏原始證據

#### Scenario: 介面與 Agent 語言

- **WHEN** Desktop 切 zh-TW／en，Agent 有效 locale 分別為 tw／ja／en 或未設定
- **THEN** 控制項與生成報告說明依對應語言，未設定遵循現有系統語言預設；官方控制項與 JSON 鍵不改名


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->