## ADDED Requirements

### Requirement: 結論後交棒單推 propose 入口

技能檔的下一步建議段 SHALL 規定：討論已寫入結論且結論值得自己開變更時，僅建議 propose 技能的 --from-discussion 入口，SHALL NOT 於該邊並列 promote。promote 的教學 SHALL 僅保留於中途轉出段（多需求討論中單項談定、討論未完時先立案）。其餘既有出邊（結論併入既有變更走 link 與 ingest、結論為不做仍照常結案後走 archive、無實質內容走 discard）SHALL 維持不變。

#### Scenario: 結論邊僅推 propose 入口

- **WHEN** 檢視 discuss 技能資產的下一步建議段
- **THEN** 「結論值得自己開變更」的邊僅含 propose 的 --from-discussion 入口；promote 僅出現於中途轉出教學段
