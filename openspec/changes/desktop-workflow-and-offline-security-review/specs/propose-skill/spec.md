## ADDED Requirements

### Requirement: 提案內容預設以條列起草

共用propose技能與spec-driven提案模板 SHALL 要求Why／What Changes一事一項條列起草，保留完整動機與變更內容，不以單段長文取代多個獨立需求。此預設 SHALL 經同一模板、原稿與registry提供所有專案及Claude／Codex／Copilot，不依專案名稱或個別設定；init／update產物與資產版本／golden SHALL 同步。既有提案不因更新模板被批次改寫。

#### Scenario: 新專案採用相同條列預設

- **WHEN** 兩個不同專案取得內建提案模板與propose技能並起草含多項動機的提案
- **THEN** 兩者都被要求將Why與What Changes一事一項列出，與共用提案畫面的條列顯示一致，不需專案特例
