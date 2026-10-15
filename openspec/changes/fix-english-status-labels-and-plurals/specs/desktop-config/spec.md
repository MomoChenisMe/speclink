## ADDED Requirements

### Requirement: 英文計數文案依數量使用單數或複數

桌面 app 與 `packages/ui` 共用元件的英文介面文案中，凡以數量代入名詞的計數文案（例如 scenario、round、task、day、requirement、change），數量為 1 時 SHALL 使用單數形（含動詞一致，例如「1 requirement has no source change」），數量為 0 或 2 以上時 SHALL 使用複數形。繁中文案不區分單複數，SHALL 維持不變。英文字典新增「`{n}` 之後接複數名詞」的計數文案時 SHALL 同時提供單數形，並 SHALL 以自動化測試守住。

#### Scenario: 數量為 1 時用單數

- **WHEN** 英文介面的規格詳情頁顯示一條只有 1 個 scenario 的需求，以及一則只有 1 輪的討論卡
- **THEN** 需求顯示「1 scenario」，討論卡顯示「1 round」

#### Scenario: 數量為 0 或 2 以上時用複數

- **WHEN** 英文介面顯示 0 個任務與 3 天前的時間
- **THEN** 分別顯示「0 tasks」與「3 days ago」

#### Scenario: 新增只有複數形的計數文案時測試失敗

- **WHEN** 開發者在英文字典新增值為「{n} widgets」的訊息但沒有提供單數形
- **THEN** 字典測試失敗並點名該訊息鍵
