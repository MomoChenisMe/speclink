## ADDED Requirements

### Requirement: 變更清單回應攜帶 delta capability 欄位

GET /changes 的清單項 SHALL 帶 deltaCapabilities，值與 GET /changes/{name} 回應的 deltaCapabilities 同源同規則：該變更 delta 規格的 capability 名稱，升冪。變更沒有 delta 規格時 SHALL 省略該鍵。

#### Scenario: 清單項攜帶 delta capability

- **WHEN** scope 內有一個帶 auth delta 規格的變更與一個沒有 delta 規格的變更，呼叫 GET /changes
- **THEN** 前者清單項含 deltaCapabilities: ["auth"]；後者清單項無 deltaCapabilities 鍵；回應整體成功
