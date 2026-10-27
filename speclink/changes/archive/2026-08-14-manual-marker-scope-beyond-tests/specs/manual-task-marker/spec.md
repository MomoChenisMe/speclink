## ADDED Requirements

### Requirement: 任務行的手動任務標記與解析

tasks.md 的任務行 SHALL 支援手動任務標記 `[M]`:位於 checkbox 之後的前綴槽。`[M]` 標記代表該任務需使用者親手操作、agent 無法代行——不限於測試,人工驗收、建立外部服務帳號、放置金鑰等人工操作皆屬之。`[M]` SHALL 為唯一承載語意的前綴標記,解析為該任務的 manual 旗標;歷史遺留的 `[P] ` 前綴 SHALL 仍被剝除但不承載任何旗標(舊檔顯示容忍——封存區與外部使用者 repo 的既有檔案不因移除而滲出字面前綴)。解析器 SHALL 以重複剝除方式接受兩種前綴(順序不敏感、各至多出現一次);任務的顯示描述 SHALL 剝除全部前綴標記。無任何標記的任務行為 SHALL 逐位元不變。

#### Scenario: 解析手動任務

- **WHEN** tasks.md 含一行「- [ ] [M] 手動驗證匯入結果」
- **THEN** 該任務解析為 manual=true,描述為「手動驗證匯入結果」(不含標記)

#### Scenario: 非測試類的手動任務同樣解析

- **WHEN** tasks.md 含一行「- [ ] [M] 至 GitHub 建立 OAuth app 並填入 client id」
- **THEN** 該任務解析為 manual=true——標記語意不區分測試與其他人工操作,引擎不判讀描述內容

#### Scenario: 舊 [P] 前綴只剝不承載

- **WHEN** tasks.md 含一行「- [x] [P] 舊任務」或「- [x] [P] [M] 混用行」
- **THEN** 描述不含任何前綴標記;`[P]` 不落任何旗標,混用行的 manual 仍為 true

#### Scenario: 無標記行為不變

- **WHEN** tasks.md 僅含無前綴標記的任務行
- **THEN** 解析結果(描述、done、stable ID)與標記引入前逐位元一致

##### Example: 前綴解析

| 任務行 | manual | 描述 |
| ------ | ------ | ---- |
| - [ ] [M] 手測匯入 | true | 手測匯入 |
| - [ ] [M] 至外部服務放置金鑰 | true | 至外部服務放置金鑰 |
| - [x] [P] 舊任務 | false | 舊任務 |
| - [x] [P] [M] 混用 | true | 混用 |
| - [ ] 寫解析器 | false | 寫解析器 |

## MODIFIED Requirements

### Requirement: apply 技能的手動任務處理

apply 技能文字 SHALL 指示:`[M]` 任務不由 agent 代勾——手動任務由使用者親自執行與勾選,不限於測試;寫碼任務全數完成時 SHALL 回報 apply 完成並向使用者點名尚餘的 `[M]` 任務。技能文字 SHALL 另含前置手動任務的處理指示:某寫碼任務依賴未勾的 `[M]` 任務時(如外部服務帳號尚未建立),agent SHALL 停下並請使用者先完成該手動任務,SHALL NOT 代勾、SHALL NOT 繞過。技能模板 SHALL 於 claude 與 codex 兩工具正典化生成,golden 對照涵蓋。

#### Scenario: 寫碼任務完成即回報

- **WHEN** apply 進行至寫碼任務全數勾選、僅餘一個 `[M]` 任務未勾
- **THEN** 技能回報實作完成,點名該手動任務留待使用者執行,不代勾該任務

#### Scenario: 前置手動任務擋住寫碼任務

- **WHEN** apply 進行至某寫碼任務,其實作依賴一個未勾的 `[M]` 任務
- **THEN** 技能指示停下並請使用者先完成該手動任務,不代勾、不繞過

#### Scenario: 技能模板生成

- **WHEN** 執行 speclink update
- **THEN** claude 與 codex 的 apply 技能檔含手動任務處理原則(含前置擋路即停),與 golden 對照一致

### Requirement: ingest 技能的起草標記指引

ingest 技能文字 SHALL 含手動任務的 `[M]` 起草指引:凡 agent 無法代行、需使用者親手操作的任務(不限於測試)皆標 `[M]`,agent 做得到的任務(寫碼與自動化測試)不標;指引 SHALL 以正誤例並列(對比對)呈現:正例(`[M]` 緊接 checkbox、編號在後)與誤例(編號在前)並列,附後果說明(引擎不認、任務被算成寫碼任務而卡住完成度)與「checkbox 後恰一個空格」規則。技能模板 SHALL 於 claude 與 codex 兩工具正典化生成,golden 對照涵蓋。

#### Scenario: ingest 補任務時的指引

- **WHEN** ingest 流程因需求變更補起草需使用者親手操作的任務行
- **THEN** 技能文字指示該任務行以 `[M]` 緊接 checkbox 起草,並可對照誤例辨識錯型

#### Scenario: 技能模板生成

- **WHEN** 執行 speclink update
- **THEN** claude 與 codex 的 ingest 技能檔含對比對起草指引,與 golden 對照一致

## REMOVED Requirements

### Requirement: 任務行的手動測試標記與解析

**Reason**: `[M]` 語意自「手動測試」放寬為「任何需使用者親手操作的任務」,舊需求名與定義文綁死測試語意,隨改名重寫。
**Migration**: 由本 delta 新增的「任務行的手動任務標記與解析」承接——解析行為逐位元不變,僅語意定義放寬。
