## ADDED Requirements

### Requirement: 看板卡片浮現待重新反映徽章

desktop 看板的變更卡片 SHALL 於該變更 meta 的 restale_from 非空時，顯示一枚「待重新反映」徽章，提示該變更反映的討論已被重新結論、待 re-ingest。徽章的資料源 SHALL 為變更 meta 的 restale_from 欄位，經桌面看板查詢路徑的 Rust 變更序列化曝為資料欄、透過 tauriDataSource 傳至前端、由 packages/ui 的看板卡片元件渲染——全程僅讀既存 meta 欄位，SHALL NOT 於載入時掃描討論記錄。restale_from 為空或缺席時卡片 SHALL NOT 顯示該徽章。徽章 SHALL 與既有卡片視覺語言（主題化樣式）一致。此浮現不改變看板欄位派生規則（全完成＞有 started＞其餘）——徽章與欄位歸屬正交。

#### Scenario: 過期變更卡片顯示徽章

- **WHEN** 看板渲染一個 meta 帶非空 restale_from 的變更卡片
- **THEN** 該卡片顯示「待重新反映」徽章；徽章不影響卡片所在看板欄位

#### Scenario: 非過期變更卡片無徽章

- **WHEN** 看板渲染一個 meta 無 restale_from（或為空）的變更卡片
- **THEN** 該卡片不顯示「待重新反映」徽章，其餘呈現與本變更前一致
