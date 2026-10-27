## MODIFIED Requirements

### Requirement: remote 破壞性操作確認一致

remote 分頁的 archive 確認對話 SHALL 沿用與本地相同的確認路徑，描述 SHALL 指出將寫入 server 上的 scope（Project/Repo 名）；deleteChange 於 remote SHALL 走與本地 discard 對齊的守門語意（未開工即刪；帶開工痕跡由 server 拒絕，沿既有刪除失敗呈現路徑回報）；offline 期間 archive SHALL 隨寫入遮罩停用。

#### Scenario: remote archive 確認指出 scope

- **WHEN** 於 remote 分頁對就緒的 change 觸發 archive
- **THEN** 確認對話呈現且描述含該 Project/Repo 名；確認後寫入 server，取消則無任何變更
