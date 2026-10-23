## MODIFIED Requirements

### Requirement: 檢查歷史與安全驗收的共用契約
<!-- BEFORE: 歷史清單只有原結果與內容摘要，沒有單筆文件可讀性欄位；任何文件缺損使整組讀取失敗。 -->

Protocol SHALL 定義同一組檢查歷史查詢、深度報告提交／查詢、接受風險與安全驗收結果型別，由CLI／Server／Remote client／Node／Desktop共用。歷史list SHALL 回{records:[{recordId:string,kind:string,createdAt:string,outcome:string,relativePath:string,contentDigest:string,readError?:string}]}；readError SHALL 只在單筆文件缺失、內容或metadata完整性不符、單筆IO等失敗時序列化，分別為missing_check_record、integrity_error、check_record_read_error，健康項目 SHALL 省略該欄位，讀取缺席欄位 SHALL 相容舊payload。readError SHALL NOT 覆蓋outcome或改寫持久化索引；omitted SHALL 沿用既有結果值且不要求已省略文件存在。清單 SHALL 保留問題項目並可讀其他完整目標；review／verify單筆 SHALL 只驗證所選文件並回{recordId:string,content:string,metadata:{createdAt:string,actor:string|null,tool:string|null,outcome:string,scopeDigest:string}}。異常目標 SHALL 回錯誤而不回異動原文；索引結構錯誤及整體來源失敗 SHALL 仍使整個查詢失敗。

安全show SHALL 回{changeName:string,reports:[{reportId:string,executionMode:string,relativePath:string,complete:boolean}],acceptance:{required:boolean,status:string,blockingFindings:number,notVerified:number,reasons:string[]}}。status限定reference|passed|blocked|stale；提交成功回{reportId:string,changeName:string,relativePath:string,complete:boolean}，接受成功回{decisionId:string,reportId:string,findingId:string,status:"accepted"}。檢查歷史內容或深度報告只接受所屬變更的index相對路徑，不能用任意path參數繞過授權。歷史查閱的隔離 SHALL NOT 放寬提交、接受風險、蓋章或封存的嚴格完整性與安全驗收判定。

設定讀取 SHALL 加securityVerification.requiredForAcceptance:boolean與checkRecords:{archive:{review:boolean,verify:boolean,security:boolean}}，缺席的新欄位讀預設false與三個true；其他既有回應欄位不改名，健康歷史回應及無新操作的回歸輸出 SHALL 維持既有位元級輸出。本次部分缺損清單成功並附readError SHALL 是刻意新增的輸出行為。新動詞遇不支援server SHALL 回unsupported_version，不把必要安全政策靜默當停用；required_for_acceptance=true的專案須使用支持版server及client。舊server在缺損時回整體失敗，新client SHALL 明示錯誤，不模擬逐筆診斷或偽裝成空清單。

Remote新寫入 SHALL 同時包含既有expectedRevision及可信actor，reader回403、認證失效401、CAS衝突revision_conflict，皆零server文件效果；離線回明確失敗而不排隊假裝成功。純歷史查詢 SHALL 不需expectedRevision、不改revision。安全／歷史錯誤reason SHALL 至少包含missing_security_review、security_acceptance_blocked、security_report_stale、missing_knowledge、history_recovery_required、source_freshness_unverified、identity_required。單筆readError SHALL 是歷史回應欄位，不是新增HTTP reason registry。UI SHALL 不自行將非成功HTTP改為passed。

#### Scenario: Local與Remote型別一致

- **WHEN** 同一提案的已完成紀錄及安全結果分別經Local與Remote reader查詢
- **THEN** camelCase欄位、nullable型別、計數與結果一致；Remote相對路徑不揭露server絕對路徑

#### Scenario: 舊server與衝突

- **WHEN** 新client向舊server提交深度報告，或兩位editor同revision接受風險
- **THEN** 前者明示unsupported_version且不宣稱安全功能停用，後者只有第一筆成功、另一筆衝突且無覆寫

#### Scenario: 部分文件缺失的新增欄位

- **WHEN** 同一有效索引有完整紀錄A、缺檔紀錄B與依政策省略的C
- **THEN** 清單仍回A、B、C；A不含readError，B含readError=missing_check_record且outcome維持原值，C保持outcome=omitted且不因省略文件報錯；讀A成功、讀B或C失敗，所有來源文件與revision不變

#### Scenario: 舊健康回應與舊來源失敗

- **WHEN** 新reader讀到沒有readError欄位的健康舊payload，或舊server因缺檔回整體錯誤
- **THEN** 前者正常解析且完整單筆仍經來源驗證，後者顯示可重試來源錯誤，不假裝具逐筆錯誤清單、不重算digest

#### Scenario: 全組驗收與逐筆歷史查閱不同

- **WHEN** 同一缺損資料集查閱完整紀錄後，再提交安全報告、接受風險、蓋章或封存
- **THEN** 查閱成功不取代嚴格完整性檢查，寫入及驗收仍拒絕缺損資料，不留下半套文件、章或revision更新
