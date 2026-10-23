## ADDED Requirements

### Requirement: 檢查歷史與安全驗收的共用契約

Protocol SHALL 定義同一組檢查歷史查詢、深度報告提交／查詢、接受風險與安全驗收結果型別，由CLI／Server／Remote client／Node／Desktop共用。歷史list SHALL 回{records:[{recordId:string,kind:string,createdAt:string,outcome:string,relativePath:string,contentDigest:string}]}，單筆回{recordId:string,content:string,metadata:{createdAt:string,actor:string|null,tool:string|null,outcome:string,scopeDigest:string}}；安全show SHALL 回{changeName:string,reports:[{reportId:string,executionMode:string,relativePath:string,complete:boolean}],acceptance:{required:boolean,status:string,blockingFindings:number,notVerified:number,reasons:string[]}}。status限定reference|passed|blocked|stale；提交成功回{reportId:string,changeName:string,relativePath:string,complete:boolean}，接受成功回{decisionId:string,reportId:string,findingId:string,status:"accepted"}。檢查歷史內容或深度報告只接受所屬變更的index相對路徑，不能用任意path參數繞過授權。

設定讀取 SHALL 加securityVerification.requiredForAcceptance:boolean與checkRecords:{archive:{review:boolean,verify:boolean,security:boolean}}，缺席的新欄位讀預設false與三個true；其他既有回應欄位不改名，無新操作的回歸輸出維持。新動詞遇不支援server SHALL 回unsupported_version，不把必要安全政策靜默當停用；required_for_acceptance=true的專案須使用支持版server及client。

Remote新寫入 SHALL 同時包含既有expectedRevision及可信actor，reader回403、認證失效401、CAS衝突revision_conflict，皆零server文件效果；離線回明確失敗而不排隊假裝成功。安全／歷史錯誤reason SHALL 至少包含missing_security_review、security_acceptance_blocked、security_report_stale、missing_knowledge、history_recovery_required、source_freshness_unverified、identity_required。UI不自行將非成功HTTP改為passed。

#### Scenario: Local與Remote型別一致

- **WHEN** 同一提案的已完成紀錄及安全結果分別經Local與Remote reader查詢
- **THEN** camelCase欄位、nullable型別、計數與結果一致；Remote相對路徑不揭露server絕對路徑

#### Scenario: 舊server與衝突

- **WHEN** 新client向舊server提交深度報告，或兩位editor同revision接受風險
- **THEN** 前者明示unsupported_version且不宣稱安全功能停用，後者只有第一筆成功、另一筆衝突且無覆寫

## MODIFIED Requirements

### Requirement: 標準 error reason registry

protocol 的錯誤回應 SHALL 為 status、reason、message 三元組；reason SHALL 屬封閉 registry：not_found、permission_denied、revision_conflict、invalid_argument、invalid_config、refused、unavailable、internal，以及安全／歷史操作的missing_security_review、security_acceptance_blocked、security_report_stale、missing_knowledge、history_recovery_required、source_freshness_unverified、identity_required與unsupported_version。新reason以對應操作的人眼原因回報；既有八項對映保持。typed client SHALL 把 reason 對映到 CLI 既有錯誤訊息，同一 reason 的訊息文字 SHALL 與現行 remote error translation 逐位元一致；未知 reason SHALL 對映為一般錯誤而非崩潰。

#### Scenario: reason 對映沿用現行訊息

- **WHEN** stub server 對某動詞回 revision_conflict 的錯誤回應
- **THEN** CLI 以非零 exit code 結束且 stderr 訊息與現行 409 情境的訊息逐位元一致

#### Scenario: 未知 reason 不崩潰

- **WHEN** stub server 回傳 registry 之外的 reason 字串
- **THEN** client 以一般錯誤處理並保留 message 供顯示，不 panic
