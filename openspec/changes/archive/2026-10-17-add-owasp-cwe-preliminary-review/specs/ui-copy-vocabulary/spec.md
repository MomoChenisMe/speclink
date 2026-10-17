## Purpose

確保繁中產品字詞符合台灣使用習慣。

## ADDED Requirements

### Requirement: 繁體中文採台灣用語

產品 zh-TW 文案與跟隨系統繁中 SHALL 共用台灣用語字典，default SHALL 稱「預設」。LANGUAGE.md SHALL 保留約定，產品字典的維護測試 SHALL 檢查不使用的對應用詞；SHALL NOT 改寫官方原文、使用者內容或封存歷史。

#### Scenario: 系統繁中與明示繁中相同

- **WHEN** 系統為zh-TW且偏好跟隨系統，或使用者明示zh-TW
- **THEN** 皆採用zh-TW字典，「Default」對應「預設」；切英文維持英文文案
