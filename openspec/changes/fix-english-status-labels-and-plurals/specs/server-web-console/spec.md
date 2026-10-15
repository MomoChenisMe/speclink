## ADDED Requirements

### Requirement: 後台英文計數文案依數量使用單數或複數

Server SPA 的英文介面文案中，凡以數量代入名詞的計數文案（例如 repository、user），數量為 1 時 SHALL 使用單數形（含動詞一致），數量為 0 或 2 以上時 SHALL 使用複數形；繁中文案 SHALL 維持不變。後台英文字典新增「`{n}` 之後接複數名詞」的計數文案時 SHALL 同時提供單數形，並 SHALL 以自動化測試守住。

#### Scenario: 總覽與專案列的單數

- **WHEN** 英文介面的總覽有 1 位尚未啟用的受邀者，且專案只有 1 個儲存庫
- **THEN** 需要處理區顯示「1 user has not activated yet.」，專案列顯示「Key demo · 1 repository」，專案計數卡提示顯示「1 repository」

#### Scenario: 數量為 2 以上時用複數

- **WHEN** 專案有 2 個儲存庫
- **THEN** 專案列顯示「Key demo · 2 repositories」
