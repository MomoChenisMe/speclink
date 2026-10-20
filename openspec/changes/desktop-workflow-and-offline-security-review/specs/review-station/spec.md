## MODIFIED Requirements

### Requirement: 蓋章守門與蓋章效果
<!-- BEFORE: 蓋章刪除活動工單，Remote無法回讀已刪文字，本機只剩Git歷史。 -->

系統 SHALL 提供 `speclink review stamp <change> [--accept]`。守門條件:(1) change 的寫碼任務全數完成(非 `[M]` 任務全數勾選;手動測試任務不計,見 manual-task-marker 的寫碼任務全完成預測子);(2) 工單末輪零未解必修 findings。必修 SHALL 以嚴重度界定:CRITICAL 與 WARNING 級為必修、擋乾淨蓋章;SUGGESTION 級 SHALL NOT 擋章——末輪僅含 SUGGESTION 級 findings 時蓋章照常放行。`--accept` SHALL 僅豁免條件 (2) 的必修部分。守門通過時系統 SHALL 於同一原子寫入內:將 `reviewed_at`/`reviewed_by`/`reviewed_with`/`reviewed_tasks_total`(蓋章時全任務總數,含 `[M]` 任務)/`reviewed_scope`(指紋清單)寫入該 change 的 `.openspec.yaml`,並刪除 `review.md`。不得出現「章已寫入而工單仍存在」的中間狀態。活動工單刪除前 SHALL 將完整輪次、當時規則、接受情形、操作者及章指紋保存於checks/review/<ULID>.md，並於checks/index.json登錄；完成紀錄不視為未結工單，蓋章後及保留紀錄的封存後仍可查詢。這是刻意新增的檔案效果，原成功stdout及JSON SHALL 維持；歷史、索引或章寫入失敗 SHALL 不回成功，本機journal恢復原工單／metadata／索引、Remote交易不提交；恢復未完成時 SHALL 明示history_recovery_required。

#### Scenario: 寫碼任務未全完成即拒絕

- **WHEN** change 的寫碼任務為 4/5 完成時執行 `review stamp`
- **THEN** exit code 非零,stderr 說明寫碼任務未全數完成並列計數(4/5),metadata 與工單皆不變

#### Scenario: 僅餘手動任務可蓋章

- **WHEN** change 的寫碼任務 4/4 全勾、一個 `[M]` 任務未勾,工單末輪 findings 為空時執行 `review stamp`
- **THEN** exit code 0,五個 reviewed 欄位寫入且 `review.md` 刪除

#### Scenario: 末輪有未解 findings 且未帶 --accept

- **WHEN** 工單末輪含至少一筆 CRITICAL 或 WARNING 級 findings 且執行 `review stamp`(無 `--accept`)
- **THEN** exit code 非零,stderr 點名未解必修數並提示 `--accept` 或先修正重審

#### Scenario: 僅 SUGGESTION 的末輪乾淨蓋章

- **WHEN** 寫碼任務全數完成且工單末輪僅含 SUGGESTION 級 findings 時執行 `review stamp`(無 `--accept`)
- **THEN** exit code 0,五個 reviewed 欄位寫入且 `review.md` 刪除;兩種模式均可從checks/review的完成紀錄查到SUGGESTION原文

#### Scenario: 帶保留蓋章

- **WHEN** 工單末輪含必修 findings 且執行 `review stamp --accept`
- **THEN** exit code 0,章寫入且工單刪除

#### Scenario: 乾淨蓋章

- **WHEN** 任務 5/5 完成且工單末輪 findings 為空時執行 `review stamp`
- **THEN** exit code 0,`.openspec.yaml` 含五個 reviewed 欄位且 `review.md` 不存在

##### Example: 蓋章寫入的任務錨

- **GIVEN** change 有 5 個任務,其中 4 個寫碼任務全數勾選、1 個 `[M]` 任務未勾,工單 Round 2 的 findings 為空
- **WHEN** `review stamp` 成功
- **THEN** `.openspec.yaml` 內 `reviewed_tasks_total` 為 5(全任務總數,含未勾的 `[M]` 任務)

#### Scenario: 歷史寫入失敗保全工單

- **WHEN** 蓋章時完成紀錄、索引或metadata寫入失敗
- **THEN** 命令非零退出，不顯示完成；原工單與章恢復原值，若恢復失敗則明示history_recovery_required且保留復原資料


## ADDED Requirements

### Requirement: 審查完成紀錄查詢

CLI SHALL 提供review history CHANGE [--id ID] [--json]，不讀stdin，支援全域--no-color。無ID時JSON為{records:[{recordId:string,kind:string,createdAt:string,outcome:string,relativePath:string,contentDigest:string}]}；有ID時為{recordId:string,content:string,metadata:{createdAt:string,actor:string|null,tool:string|null,outcome:string,scopeDigest:string}}。人眼同列紀錄摘要或完整遮罩工單；成功0，找不到／非法ULID／壞索引／讀取失敗stderr及1，參數錯誤2。CHANGE SHALL 接受active名稱或已封存dated名稱，只能讀該變更index列明的checks/review路徑，不讀任意檔案。Remote reader可讀，401／離線明示失敗；新輪次、discard及重蓋章 SHALL 不覆蓋舊紀錄。既有review show只讀活動工單的契約 SHALL 不變。

#### Scenario: 查詢已蓋章紀錄的人眼與JSON

- **WHEN** 對完成蓋章的變更分別執行review history CHANGE --json --no-color及帶合法--id的人眼查詢
- **THEN** 前者回單一camelCase物件，後者stdout列同一完整輪次與完成metadata，皆無ANSI且exit0；舊工單已不在活動路徑

#### Scenario: 缺歷史與非法ID

- **WHEN** 舊變更從未保存完成紀錄，或--id為../../secret
- **THEN** 前者清單為空而不編造Git歷史，後者exit1且不讀變更外檔案；Remote401不假裝清單為空
