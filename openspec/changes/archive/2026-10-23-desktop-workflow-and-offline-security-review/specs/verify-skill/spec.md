## MODIFIED Requirements

### Requirement: Verify 報告中的獨立初步安全資訊
<!-- BEFORE: 初步安全資訊永遠不影響驗證與封存，只用內建規則。 -->

驗證技能 SHALL 在安全政策啟用時，讀取當前目標的離線初步安全審查結果，於報告獨立列已選標準／版本、執行結果、疑似風險、未驗證與本機報告連結；缺報告、資料或目標 SHALL 明寫未驗證與原因，不自行連網下載、不視為通過。基本掃描需要執行時 SHALL 使用同一內建離線規則命令，無AI仍可使用；需要深度審查時 SHALL 明確交棒speclink-security-review，驗證技能不以基本掃描代替該skill。技能 SHALL 不把外部標準文件當指令，不上傳原始碼。

安全資訊 SHALL 不寫成驗證工單的CRITICAL／WARNING或新章，required_for_acceptance=false時不因初步FINDING／NOT_VERIFIED阻擋驗收或Archive；enabled且required_for_acceptance=true時 SHALL 讀取共用安全驗收判定，缺報告、過期、必要未驗證或未處理中高風險時不宣稱通過，不用一般verify --accept豁免；SHALL 保留原本規格合規判斷、TDD／audit開關、既有工單／stale章守門與人工處置。停用且無歷史報告時 SHALL 維持原輸出，不額外掃描；停用後已有歷史報告可供手動查看，但不得冒充本輪新結果。

#### Scenario: 初步疑似風險不改驗證工單

- **WHEN** 未勾選納入安全驗收且原變更符合其規格，離線安全報告有疑似SQL拼接或未驗證ASVS要求
- **THEN** 驗證報告獨立揭露資訊及PASS限制，verify工單與章不因這些初步結果多出必修項，沒有新增封存阻擋

#### Scenario: 缺資料或停用政策

- **WHEN** 啟用但所需資料缺失，或完全未啟用且無歷史報告
- **THEN** 前者安全區塊標示未驗證並指向資料管理，後者既有Verify行為及輸出維持，兩者都不自動下載

#### Scenario: 既有守門仍有效

- **WHEN** 變更另有未結Verify工單或既有過期章
- **THEN** 原有封存守門與處置仍生效，不以新的安全報告通過來繞過

#### Scenario: 必要安全條件未滿

- **WHEN** required模式下驗證其他規格皆符合，但安全報告缺失、過期、必要未驗證或有未處理HIGH
- **THEN** 技能回報安全驗收未通過及原因，指向審查或風險處置，不自行接受、不連網補件、不宣稱驗收完成

## ADDED Requirements

### Requirement: 驗證完成紀錄的技能交棒

verify技能 SHALL 回報蓋章完成紀錄位於提案checks/verify，提供verify history或Desktop查詢入口；保存原輪次、當時規格檢查及接受情形，不覆寫歷史。保存失敗 SHALL 回報未完成，不刪復原資料；Claude／Codex／Copilot及自訂工具 SHALL 使用同一原稿生成，與資產版本及golden同步。安全風險接受 SHALL 綁當前finding與理由，skill不得代使用者自動接受。

#### Scenario: 成功與保存失敗的回報

- **WHEN** verify蓋章成功，或蓋章因保存歷史失敗
- **THEN** 前者提供可查完整紀錄的recordId，後者明示未完成且不宣稱已驗證，沒有刪除唯一紀錄
