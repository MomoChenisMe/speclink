## MODIFIED Requirements

### Requirement: 審查完成紀錄查詢
<!-- BEFORE: 清單或任一單筆查詢都先驗證全部歷史文件，其他紀錄缺失也會讓完整目標無法讀取。 -->

CLI SHALL 提供 review history CHANGE [--id ID] [--json]，不讀 stdin，支援全域 --no-color。無 ID 時 JSON SHALL 為{records:[{recordId:string,kind:string,createdAt:string,outcome:string,relativePath:string,contentDigest:string,readError?:string}]}；readError SHALL 僅於該筆檔案缺失、完整性不符或讀取失敗時存在，值分別為missing_check_record、integrity_error、check_record_read_error，不變更outcome。有 ID 時 SHALL 為{recordId:string,content:string,metadata:{createdAt:string,actor:string|null,tool:string|null,outcome:string,scopeDigest:string}}，只驗證所選文件，成功回完整遮罩工單；其他紀錄內容異常 SHALL NOT 封鎖完整目標。索引結構、路徑與ID驗證 SHALL 仍完整執行；不得讀索引外的任意檔案。

健康清單的人眼五欄、JSON六個既有欄位與完整單筆回應 SHALL 維持既有位元級輸出；部分文件異常時 SHALL 刻意由整組失敗改為清單成功exit0並保留異常項，JSON增加readError，人眼在該列原五欄後加readError=代碼。--no-color SHALL 不含ANSI。omitted SHALL 維持原政策語意，不讀已省略文件。有ID且目標缺失、omitted、完整性不符或不可讀，找不到變更、非法ULID、壞索引、索引IO及整體來源失敗 SHALL stderr報錯及exit1；參數錯誤SHALL exit2。

CHANGE SHALL 接受active名稱或已封存dated名稱，只能讀該變更index列明的checks/review路徑。Remote reader SHALL 可讀，401／403／離線 SHALL 明示失敗且不假裝清單為空；查閱不攜帶寫入revision、不變更revision，SHALL NOT 修改歷史、索引或章。新輪次、discard及重蓋章 SHALL 不覆蓋舊紀錄；review show SHALL 仍只讀活動工單。查閱的單筆隔離 SHALL NOT 放寬蓋章、報告寫入或封存前的全組完整性守門。

#### Scenario: 查詢已蓋章紀錄的人眼與JSON

- **WHEN** 對完成蓋章的活動或已封存變更執行review history CHANGE --json --no-color及帶合法--id的人眼查詢
- **THEN** 前者回單一camelCase物件，後者stdout列同一完整輪次與完成metadata，健康清單及全文維持既有輸出，皆無ANSI且exit0；舊工單已不在活動路徑

#### Scenario: 缺歷史與非法ID

- **WHEN** 舊變更從未保存完成紀錄，或--id為../../secret
- **THEN** 前者清單為空而不編造Git歷史，後者exit1且不讀變更外檔案；Remote401不假裝清單為空

#### Scenario: 缺失文件不拖累完整審查紀錄

- **WHEN** 索引列有六筆審查，其中第二筆文件被刪除，而第六筆及驗證文件完整
- **THEN** 審查清單exit0列出六筆且第二筆readError=missing_check_record，讀第六筆exit0，讀第二筆exit1；驗證紀錄仍可查閱，索引及所有既有文件保持不變

#### Scenario: 內容異動與清單後發生缺損

- **WHEN** 某審查紀錄內容指紋或metadata與索引不符，或清單載入後目標文件才被刪除
- **THEN** 前者清單以integrity_error標示且不提供異動內容，後者單筆讀取以當次缺失失敗；其他完整紀錄不被連帶封鎖，不更新索引或指紋

#### Scenario: 壞索引及來源失敗仍拒絕

- **WHEN** 索引無法解析、路徑越界、ID重複、索引不可讀，或Remote認證失效／無權限／離線
- **THEN** 清單及單筆明示失敗，stderr及exit1，不回部分可信清單或空清單，不修改文件及revision

#### Scenario: 查閱隔離不放寬審查蓋章與封存

- **WHEN** 存在缺損歷史但完整審查紀錄可查，接著嘗試對活動提案蓋章或封存
- **THEN** 蓋章及封存仍因完整性不足失敗，活動工單、原章、索引及既有文件保持原狀，查閱成功不作為放行證據
