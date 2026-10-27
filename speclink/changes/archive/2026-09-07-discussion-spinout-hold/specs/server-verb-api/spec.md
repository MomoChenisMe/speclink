## ADDED Requirements

### Requirement: 討論結論端點轉傳保留旗標並回填 held

討論結論端點 SHALL 將請求的 hold 欄位直通引擎的結論命令（缺席即 false，行為與本變更前完全相同），並自引擎結果回填 held（camelCase 布林、僅 true 時出鍵）。hold 為 true 時 SHALL NOT 觸發閉環封存（回應無 autoArchived 鍵）、討論 SHALL 維持於 live 清單。判準、寫入與旗標清除 SHALL 由引擎執行，server 路由 SHALL NOT 重複實作。既有欄位、狀態碼與錯誤語意 SHALL 維持不變。

#### Scenario: 結論端點帶 hold 回填 held

- **WHEN** 對閉環條件原本成立（promoted_to 非空且全數轉出變更已封存）的討論以 hold: true 呼叫討論結論端點
- **THEN** HTTP 200，回應含 held: true 且無 autoArchived 鍵；該討論仍在 live 清單、不出現於封存清單；scope revision 前進

#### Scenario: 結論端點不帶 hold 行為不變

- **WHEN** 以只含 content 的請求呼叫討論結論端點
- **THEN** 回應與本變更前同形（無 held 鍵），閉環判斷結果與本變更前一致
