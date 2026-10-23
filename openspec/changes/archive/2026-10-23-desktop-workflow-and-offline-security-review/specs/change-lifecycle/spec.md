## ADDED Requirements

### Requirement: 必要安全驗收的封存判定

archive在enabled及required_for_acceptance皆true時 SHALL 於任何檔案效果前以相同安全驗收規則判定。缺當前deep報告、缺啟用基本結果、必要資料／項未完成、過期或未處理中高風險 SHALL 非零拒絕並列原因；--mark-tasks-complete、--no-validate、--skip-specs、--carry-review／--carry-verify及一般--accept SHALL 不豁免。required=false或主開關停用 SHALL 維持既有封存成功輸出與守門，保留參考報告不視為未結工單。CLI、Desktop、Server及bulk SHALL 共用判定，不以未經驗證的UI通過標記代替。

#### Scenario: 沒跑驗證也不能略過必要安全檢查

- **WHEN** required模式沒有安全報告，使用者對任務已完成且沒有verify章的變更直接封存
- **THEN** exit1及stderr指向缺少安全審查，Desktop顯示同一原因，變更／規格／任務未改動

#### Scenario: 參考模式與明確接受

- **WHEN** 參考模式有HIGH參考報告，或required模式的完整當前報告僅有已附理由接受的HIGH及LOW
- **THEN** 安全檢查不額外阻擋，其他既有任務／工單／章守門仍正常生效，原成功輸出保持回歸對照

### Requirement: 封存時按類別保留完成紀錄

archive SHALL 讀當次check_records.archive三布林值，預設皆true；true將完整對應checks文件及索引一併移入封存區，false只省略該類已完成完整紀錄。封存前的活動提案紀錄、已封存歷史及carry帶走的未結工單 SHALL 不被此設定刪除。required模式 SHALL 另保留最小acceptance.json收據的政策／報告digest、接受理由、結果及時間，不含程式片段；完整報告被省略時索引須明示omitted，不留下壞文件連結。

裁切 SHALL 在受控staging驗證完成後與封存同次提交；失敗保留活動提案及既有紀錄，不顯示已封存。Remote交易／revision衝突 SHALL 無部分效果；本機未完成journal須先恢復，不以半套索引完成封存。封存確認頁 SHALL 列三類保留及省略數，不把摘要收據冒充完整報告。

#### Scenario: 三類保留的八種組合

- **WHEN** 變更有三類紀錄，分別以review／verify／security的8種true／false組合封存
- **THEN** 每次完整文件精確符合選項，預設三類皆保留；省略的索引標omitted且無斷鏈，required的最小收據及未結carry工單仍在

#### Scenario: staging或提交失敗

- **WHEN** 封存裁切、寫入或Remote CAS提交失敗
- **THEN** 原活動提案與三類紀錄完整保留，沒有已封存成功狀態或僅剩部分報告
