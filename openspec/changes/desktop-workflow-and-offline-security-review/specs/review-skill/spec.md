## ADDED Requirements

### Requirement: 審查完成紀錄的技能交棒

review技能 SHALL 說明蓋章後活動工單移除但完整紀錄存於提案checks/review，並回報可供review history與Desktop查詢的recordId；SHALL NOT 指示刪除完成紀錄、把歷史文件視為未結工單或重新從Git拼造未保存的舊工單。首輪／續輪、分級、修復收斂及既有--accept的行為 SHALL 不變；記錄當時規則與接受情形，後續設定改變不回寫舊紀錄。原稿經同一registry生成至各工具，資產版本與golden SHALL 同批更新。

#### Scenario: 蓋章後提供可查詢紀錄

- **WHEN** review技能成功蓋章後回報結果
- **THEN** 使用者可由回報recordId或Desktop讀完整輪次及當時規則，沒有要求再開活動工單

#### Scenario: 保存失敗

- **WHEN** 蓋章因歷史保存或復原失敗而非零退出
- **THEN** 技能回報未完成及原因，不宣稱已審查完成，不刪復原備份或另造空白成功紀錄
