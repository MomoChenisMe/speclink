## ADDED Requirements

### Requirement: 標記位置的 change 驗證檢查

change 驗證 SHALL 對 tasks.md 的每個解析後任務檢查手動標記位置,命中下列任一錯型即報 error、使該 change 驗證結果為 invalid:(A)「編號在前」——顯示描述的首個空白分隔 token 僅含 ASCII 數字與句點且至少含一個數字,且下一個 token 為字面 `[M]`;(B)「行首殘留」——顯示描述以字面 `[M]` 開頭(後接空白或即為結尾)。檢查 SHALL 僅施於描述開頭,SHALL NOT 掃描描述中段(行文或反引號提及 `[M]` 不構成違規);已勾與未勾任務 SHALL 同等檢查。錯誤訊息 SHALL 自帶修復指引:含 tasks.md 邏輯路徑(正斜線)、任務序號、描述前綴引文,並以正誤例並列指明 `[M]` 須緊接 checkbox。tasks.md 缺席或無命中時 SHALL NOT 產生任何 error 或 warning,既有驗證輸出逐位元不變;既有錯誤 SHALL 先列,本檢查的錯誤後附。

#### Scenario: 編號在前報 error

- **WHEN** 某 change 的 tasks.md 含一行「- [ ] 6.2 [M] 手動驗收」,執行該 change 的驗證
- **THEN** 驗證結果 invalid,error 訊息含任務序號與描述引文,並以正誤例並列指明 `[M]` 須緊接 checkbox

#### Scenario: 行首殘留報 error

- **WHEN** 某 change 的 tasks.md 含一行「- [ ]  [M] 手測匯入」(checkbox 後兩個空格,前綴槽漏接),執行該 change 的驗證
- **THEN** 驗證結果 invalid,error 訊息點名 checkbox 後恰一個空格

#### Scenario: 正確前綴與中段字面提及不報

- **WHEN** 某 change 的 tasks.md 僅含「- [ ] [M] 手測匯入」與「- [x] 1.1 前綴剝除迴圈同時接受 `[P]` 與 `[M]` 的說明文字」兩行,執行該 change 的驗證
- **THEN** 驗證不因標記位置產生任何 error 或 warning

##### Example: 誤置判定

| 任務行 | 判定 |
| ------ | ---- |
| - [ ] [M] 3.2 手測匯入 | 通過 |
| - [ ] 3.2 [M] 手測匯入 | 錯型 A |
| - [ ] 1.10 [M] 手測 | 錯型 A |
| - [ ]  [M] 手測 | 錯型 B |
| - [ ] 說明 `[M]` 剝除規則 | 通過 |

### Requirement: ingest 技能的起草標記指引

ingest 技能文字 SHALL 含手動測試任務的 `[M]` 起草指引,且 SHALL 以正誤例並列(對比對)呈現:正例(`[M]` 緊接 checkbox、編號在後)與誤例(編號在前)並列,附後果說明(引擎不認、任務被算成寫碼任務而卡住完成度)與「checkbox 後恰一個空格」規則。技能模板 SHALL 於 claude 與 codex 兩工具正典化生成,golden 對照涵蓋。

#### Scenario: ingest 補任務時的指引

- **WHEN** ingest 流程因需求變更補起草含人工驗收的任務行
- **THEN** 技能文字指示該任務行以 `[M]` 緊接 checkbox 起草,並可對照誤例辨識錯型

#### Scenario: 技能模板生成

- **WHEN** 執行 speclink update
- **THEN** claude 與 codex 的 ingest 技能檔含對比對起草指引,與 golden 對照一致
