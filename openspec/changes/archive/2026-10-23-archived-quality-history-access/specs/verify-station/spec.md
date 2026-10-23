## MODIFIED Requirements

### Requirement: 驗證完成紀錄查詢
<!-- BEFORE: 讀取驗證歷史先驗證全部類別的文件，任一缺檔連帶阻擋完整驗證紀錄。 -->

CLI SHALL 提供 verify history CHANGE [--id ID] [--json]，不讀stdin，支援全域--no-color。無ID時JSON SHALL 為{records:[{recordId:string,kind:string,createdAt:string,outcome:string,relativePath:string,contentDigest:string,readError?:string}]}；readError SHALL 僅於該筆文件缺失、完整性不符或讀取失敗時存在，值分別為missing_check_record、integrity_error、check_record_read_error，outcome SHALL 保持原驗證結果。有ID時 SHALL 為{recordId:string,content:string,metadata:{createdAt:string,actor:string|null,tool:string|null,outcome:string,scopeDigest:string}}，只檢查所選文件並回完整遮罩工單；其他審查、驗證或安全歷史的內容異常 SHALL NOT 阻擋完整目標。所有查詢 SHALL 驗證索引結構、ID與允許路徑，不讀任意檔案。

健康清單的人眼五欄、JSON六個既有欄位與單筆全文 SHALL 維持既有位元級輸出；部分文件異常時 SHALL 刻意由整組失敗改為清單成功exit0且保留異常項，JSON加readError，人眼於原五欄後加readError=代碼。--no-color SHALL 無ANSI。omitted SHALL 保留原省略政策結果且不讀已省略文件。查詢異常目標、omitted目標、找不到變更、非法ULID、壞索引、索引IO及整體來源失敗 SHALL stderr報錯及exit1，參數錯誤 SHALL exit2。

CHANGE SHALL 接受active名稱或已封存dated名稱，只讀該變更index列明的checks/verify路徑；Remote reader SHALL 可讀，401／403／離線 SHALL 明示失敗，不回假空清單。查閱 SHALL 無文件、章或revision寫入，不需提供expectedRevision；新輪次、discard及重蓋章 SHALL 不覆蓋舊紀錄，verify show SHALL 仍只讀活動工單。此讀取隔離 SHALL NOT 改變新驗證蓋章、安全驗收、報告提交、風險接受或封存的完整性守門。

#### Scenario: 查詢已蓋章紀錄的人眼與JSON

- **WHEN** 對完成蓋章的活動或已封存變更執行verify history CHANGE --json --no-color及帶合法--id的人眼查詢
- **THEN** 前者回單一camelCase物件，後者stdout列同一完整輪次與完成metadata，健康清單及全文維持原輸出，皆無ANSI且exit0；舊工單已不在活動路徑

#### Scenario: 缺歷史與非法ID

- **WHEN** 舊變更從未保存完成紀錄，或--id為../../secret
- **THEN** 前者清單為空而不編造Git歷史，後者exit1且不讀變更外檔案；Remote401不假裝清單為空

#### Scenario: 審查或安全文件缺損仍可讀完整驗證

- **WHEN** 同一索引中的審查或安全報告文件缺失，但六筆驗證文件完整
- **THEN** 驗證清單exit0回六筆且完整單筆可讀，不因其他類別的文件缺失失敗，也不把查閱結果作為安全驗收通過

#### Scenario: 一筆驗證異動而其他五筆完整

- **WHEN** 六筆驗證中的一筆內容被修改，或某筆文件發生單筆讀取錯誤
- **THEN** 清單保留六筆並於問題項目分別標示integrity_error或check_record_read_error，outcome不變，其餘五筆可讀；問題單筆查詢exit1且無錯誤全文，無文件或索引寫入

#### Scenario: 索引與來源錯誤不偽裝成歷史缺席

- **WHEN** 索引內容錯誤或不可讀、變更不存在，或Remote遇401／403／離線
- **THEN** 清單及單筆查詢皆明示失敗，stderr及exit1，不回假空清單、不影響原章與revision

#### Scenario: 查閱隔離不放寬安全驗收

- **WHEN** 缺損歷史中的完整驗證文件查閱成功，接著在required_for_acceptance生效時嘗試新驗證蓋章或封存
- **THEN** 既有全組完整性與安全驗收守門仍執行，缺必要報告或內容不符不得通過，一般接受選項不得豁免，原活動工單與原章不被修改
