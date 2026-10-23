## MODIFIED Requirements

### Requirement: 安全政策區塊預設與獨立選項
<!-- BEFORE: 安全政策沒有是否納入驗收欄位，安全結果一律不作驗收門檻。 -->

security_verification SHALL 為單一政策來源，新增 required_for_acceptance:boolean，缺席為false；其JSON讀取欄位為requiredForAcceptance:boolean。enabled=false時該值保留但不生效。required=true而全部標準或三個reviews關閉時SHALL 顯示unconfigured並不能通過安全驗收，不替使用者啟用其他標準。缺席／null 為 enabled=false、scope=off、四標準停用、ASVS level=L1、reviews.spec/code/api_boundary=true、disabled_rules/exclude_dirs 為空。主開關與 scope off SHALL 不同義；標準 SHALL 獨立，Top 10 限2021／2025、API限2019／2023、ASVS限4.0.3／5.0.0與L1／L2／L3，CWE選已安裝版本字串。停用 SHALL 保留已選版本及子設定；啟用標準缺版本、錯型別、非法enum SHALL 報 invalid_config，不退預設。未知既有欄位 SHALL 保留，不當作可執行功能；不新增環境覆寫或 .speclink.yaml 政策來源。

#### Scenario: 各標準互不影響

- **WHEN** 先設定 Top 10 2025 與 ASVS 5.0.0 L2，再只停用 ASVS
- **THEN** Top 10 仍啟用2025，ASVS版本及L2保留但不執行，tdd／audit／worktree值不變

#### Scenario: 非法安全政策

- **WHEN** enabled 是字串、scope=desktop、ASVS level=L4 或啟用標準沒有版本
- **THEN** 載入失敗並指出欄位，不能轉為 disabled 或執行審查

#### Scenario: 舊政策與停用後保值

- **WHEN** 載入沒有required_for_acceptance的舊設定，或暫時停用已勾選該值的主開關
- **THEN** 前者讀取為false且沒有新增安全驗收阻擋，後者保留true但暫不生效，重新啟用後才納入驗收


### Requirement: 安全政策部分更新與保值
<!-- BEFORE: security部分更新不接受納入驗收欄位。 -->

CLI SHALL 提供 workflow-config security --stdin [--dry-run] [--no-color]，stdin為至多64KiB單一YAML部分更新mapping。空白／非mapping／未知輸入鍵／不合法葉值 SHALL 拒絕且原文不動；缺--stdin exit1，語法錯誤exit2。required_for_acceptance SHALL 為合法布林葉節點，字串true SHALL 拒絕。只改指定葉節點，未觸及內容（包含原文件未知安全子鍵、頂層鍵、註解、tdd／audit／worktree）SHALL 保留。複雜alias／merge／重複鍵無法安全改寫 SHALL 拒寫而非重建整份。成功stdout一行摘要exit0，--dry-run stdout逐行diff且不寫入，錯誤stderr exit1；不新增--json。Remote SHALL 共用改寫、editor授權及expected revision CAS，衝突／離線／401不排隊、不覆蓋。

#### Scenario: 修改一個葉節點保留未知內容

- **WHEN** 原文件含未知 future_option 與其註解，stdin只提供 enabled:false
- **THEN** 只改主開關，future_option、註解、已選版本及其他政策仍原樣保留，重新開啟可還原子設定

#### Scenario: dry-run 與失敗原檔不變

- **WHEN** 對合法patch執行--dry-run，或提供65,537bytes／未知鍵／錯誤型別
- **THEN** 前者只有實際差異且檔案不變，後者stderr錯誤exit1、沒有diff與檔案效果

#### Scenario: Remote 並行修改

- **WHEN** 另一人於讀取與提交間更新政策
- **THEN** exit1並提示重讀重試，不覆蓋他人資料；Desktop保留使用者輸入並走現有逐欄對照

## ADDED Requirements

### Requirement: 檢查紀錄封存政策

check_records.archive SHALL 提供review／verify／security三個布林值，缺席皆true；只控制封存完整紀錄，活動提案蓋章後仍保存，改值不刪既有歷史。JSON讀取 SHALL 增checkRecords:{archive:{review:boolean,verify:boolean,security:boolean}}。CLI SHALL 新增workflow-config check-records --stdin [--dry-run]，最多64KiB YAML部分mapping；無--json，成功摘要stdout及0，dry-run只輸出實際逐行差異，錯誤stderr及1，語法錯誤2，支援全域--no-color且不含ANSI。未知輸入鍵／錯型別／空mapping SHALL 拒絕，既有未知政策及註解逐字保值，純量安全跳脫。Remote SHALL 依editor授權及expected revision提交，401／離線／衝突不改政策。

#### Scenario: 只修改安全紀錄的封存選項

- **WHEN** stdin為archive: {security: false}並成功保存，再讀取workflow-config show --json
- **THEN** checkRecords.archive為review=true、verify=true、security=false，其餘政策與註解不變，既有安全紀錄仍可查

#### Scenario: dry-run與非法值

- **WHEN** 使用--dry-run提交合法更新，或以字串false／未知鍵提交
- **THEN** 前者stdout只列差異且無寫入，後者exit1及stderr點名欄位、無文件效果；兩者--no-color皆無ANSI
