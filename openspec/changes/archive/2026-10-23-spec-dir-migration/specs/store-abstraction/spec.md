## REMOVED Requirements

### Requirement: 預設檔案系統佈局不變

**Reason**: 規格資料夾預設改為 `speclink/`、變更資料檔改名為 `.meta.yaml`，「佈局不變」已不成立

**Migration**: 由「預設檔案系統佈局」承接：資料夾內的 specs、changes、changes/archive、discussions 與 config.yaml 形狀不變，三個原有情境（建立 change 的檔案系統效果、損壞 metadata 的容錯、變更排序）照舊保留，只把資料夾改為 `speclink/`、資料檔改為 `.meta.yaml`，並新增舊檔名的讀取順序

## ADDED Requirements

### Requirement: 預設檔案系統佈局
預設儲存實作 SHALL 使用規格資料夾（預設 `speclink/`，決定方式見 spec-dir-migration）下的佈局：specs/<capability>/spec.md、changes/<name>/、changes/archive/、discussions/、config.yaml。每個變更的資料檔 SHALL 為 changes/<name>/.meta.yaml（已封存時為 changes/archive/<dated-name>/.meta.yaml），欄位與格式維持既有規則。讀取變更資料檔時 SHALL 先讀 `.meta.yaml`，不存在時才讀舊檔名 `.openspec.yaml`；兩者並存時以 `.meta.yaml` 為準。寫入 SHALL 只寫 `.meta.yaml`，該變更資料夾有 `.openspec.yaml` 時 SHALL 在同一次寫入中刪除它。封存移動變更資料夾時，資料檔隨資料夾移動、檔名不變。

#### Scenario: 建立 change 的檔案系統效果一致
- **WHEN** 在使用 `speclink/` 的專案執行 speclink new change demo-change --agent claude
- **THEN** 於 speclink/changes/demo-change/ 建立 .meta.yaml，其欄位（schema、created、created_by）與既有格式一致，沒有建立 .openspec.yaml，exit code 為 0

#### Scenario: 損壞 metadata 的容錯行為一致
- **WHEN** 某 change 目錄內的 .meta.yaml 內容無法解析，執行 speclink list --json
- **THEN** 該 change 仍以預設 metadata 列出，指令不中斷、不輸出錯誤，與既有行為一致

#### Scenario: 變更排序行為一致
- **WHEN** 存在多個 change 且其檔案修改時間不同，執行 speclink list
- **THEN** 排序依「change 內最近修改時間（截整秒）由新至舊」，與既有行為一致

#### Scenario: 只有舊檔名時照常讀取
- **WHEN** speclink/changes/old/ 只有 .openspec.yaml（含 `depends_on: base`），執行 speclink plan --json
- **THEN** old 的 dependsOn 含 base，exit code 為 0

#### Scenario: 寫入時移除舊檔名
- **WHEN** speclink/changes/old/ 只有 .openspec.yaml，執行 speclink change depends old --on base2
- **THEN** speclink/changes/old/ 只有 .meta.yaml，內容為原 .openspec.yaml 加上新的 depends_on 值，.openspec.yaml 已不存在

#### Scenario: 新舊檔名並存時以新檔名為準
- **WHEN** speclink/changes/both/ 同時有 .meta.yaml（schema: no-spec-change）與 .openspec.yaml（schema: spec-driven），執行 speclink status --change both --json
- **THEN** schemaName 為 no-spec-change

### Requirement: 遠端封存變更資料檔的鍵

遠端 store 的已封存變更文件以相對檔名為鍵。系統讀取已封存變更的資料檔時 SHALL 先讀鍵 `.meta.yaml`，不存在時才讀舊鍵 `.openspec.yaml`；新封存的變更 SHALL 以 `.meta.yaml` 為鍵寫入。既有遠端資料 SHALL NOT 被搬移或改鍵。

#### Scenario: 舊鍵照常讀取
- **WHEN** 遠端 store 有一個以 `.openspec.yaml` 為鍵的已封存變更，client 讀取已封存清單
- **THEN** 該項的建立者與來源討論等欄位照常帶出，與舊鍵改名前相同

#### Scenario: 新封存寫新鍵
- **WHEN** 在遠端模式封存變更 demo
- **THEN** 遠端 store 中該已封存變更的資料檔鍵為 `.meta.yaml`，沒有 `.openspec.yaml` 鍵
