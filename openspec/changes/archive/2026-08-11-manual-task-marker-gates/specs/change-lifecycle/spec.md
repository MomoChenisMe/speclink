## ADDED Requirements

### Requirement: 封存的章失效守門

單筆封存 SHALL 於任務完成度守門之後、任何封存檔案效果之前,對 change 的 review 章與 verify 章各執行一次失效判定(依各站「指紋錨與失效判定」條文):章欄位齊備且判為 stale 時 SHALL 拒絕封存——非零 exit code,stderr 點名過期的站別與破錨原因(內容錨列出首個不符的檔案;任務錨述明計數),並指路重跑該站技能後再封存;兩章皆 stale 時 SHALL 並列點名。無章、或章欄位不全(Unknown)的 change SHALL 放行,其封存行為與本守門引入前逐位元一致。任務未完成與章失效並存時,任務完成度守門 SHALL 先拒且其訊息不變。`--mark-tasks-complete` 路徑 SHALL 於前置全勾寫入之前判定章失效——stale 拒絕時 tasks.md 逐位元不變,未手測的 `[M]` 任務不得被代勾。任一站工單開立中時,該站之章 SHALL 不入失效判定——該站的封存處置(擋下或 `--carry-*` 帶走)由未結工單守門承載,已被重開工單取代的章不得攔路。本守門 SHALL 於引擎封存流程本體生效,一體適用 CLI 單筆、桌面封存動詞與 server 封存通道;批次封存經同一流程,stale 章的拒絕沿未結工單守門的既有 fail-fast 樣式中止批次並點名該 change,SHALL NOT 靜默跳過。remote 封存通道無工作樹可讀,SHALL 僅判任務錨、跳過內容錨——此非對稱屬已知限制。

#### Scenario: 蓋章後改碼的封存被拒

- **WHEN** review 章蓋成後修改任一 `reviewed_scope` 檔案內容,任務全數完成後執行單筆封存
- **THEN** 非零 exit code,stderr 點名 review 章已過期並列出首個內容不符的檔案,指路重跑審查站;openspec/ 下任何檔案逐位元不變

#### Scenario: 補勾手動任務後封存放行

- **WHEN** 寫碼任務全完成時兩章蓋成,之後勾選最後一個 `[M]` 任務且 scope 檔零改動,執行單筆封存
- **THEN** exit code 0,封存照常完成——補勾 `[M]` 任務不使章失效

#### Scenario: 無章與 Unknown 章放行

- **WHEN** 對無任何章、或章欄位不全的任務全完成 change 執行單筆封存
- **THEN** 封存行為(人眼與 --json 輸出、exit code、檔案效果)與本守門引入前逐位元一致

#### Scenario: 任務守門先於章失效守門

- **WHEN** change 同時有未完成的寫碼任務與已失效的 review 章,執行單筆封存(未帶 --mark-tasks-complete)
- **THEN** 拒絕訊息為既有任務完成度守門訊息(完成數/總數與兩條出路),不提及章失效

#### Scenario: 帶旗標封存的拒絕路徑零寫入

- **WHEN** 章已失效的 change 以 `--mark-tasks-complete` 執行單筆封存
- **THEN** 拒絕來自章失效守門且 tasks.md 逐位元不變——未勾的 `[M]` 任務不被代勾

#### Scenario: 工單開立中的站不入失效判定

- **WHEN** 兩站蓋章後重開兩張工單,scope 檔已改動,帶 `--carry-review --carry-verify` 執行單筆封存
- **THEN** 封存放行,工單隨目錄帶走——已被重開工單取代的章不擋 carry 處置

#### Scenario: remote 通道僅判任務錨

- **WHEN** 經 server 封存通道對「蓋章後改過 scope 檔、任務錨未破」的 change 觸發封存
- **THEN** 封存放行——remote 側無工作樹,內容錨不判定;任務錨破(如蓋章後新增任務)時仍拒絕

##### Example: 守門判定

| 章狀態 | 蓋章後變動 | 結果 |
| ------ | ---------- | ---- |
| review 章齊備 | scope 檔內容改變 | 拒絕,點名 review 站 |
| 兩章齊備 | 補勾 [M] 任務 | 放行 |
| 兩章齊備 | 新增一個任務 | 拒絕,任務錨破 |
| 無章 | 任意 | 放行(行為不變) |
| 章欄位不全 | 任意 | 放行(行為不變) |
