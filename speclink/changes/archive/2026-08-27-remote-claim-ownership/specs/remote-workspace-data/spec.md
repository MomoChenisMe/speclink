## ADDED Requirements

### Requirement: 認領操作與認領人呈現

remote 分頁 SHALL 提供認領面：詳情抽屜對未認領的 change 提供認領操作（capability 依 handshake role——editor 以上可用；reader 呈現停用附繁體中文說明）；已認領的 change SHALL 於看板卡片與詳情抽屜呈現認領人（資料源＝清單與單 change 讀取回應的 claimedBy）；認領撞 ownership 衝突（409 refused）時 SHALL 呈現目前持有人與建議動作、沿既有錯誤呈現路徑；本地分頁 SHALL NOT 出現認領面（RemoteOnly 動詞不在本地偽造入口）。

#### Scenario: editor 認領後跨端可見

- **WHEN** editor 於 remote 分頁對未認領的 change 執行認領，隨後重新載入清單
- **THEN** 看板卡片與詳情抽屜呈現認領人為自己；同 scope 的另一台裝置載入清單亦見同一認領人

#### Scenario: 衝突呈現與 reader 停用

- **WHEN** 另一 editor 對已認領的 change 執行認領；一位 reader 開啟 remote 分頁
- **THEN** 前者見「已由目前持有人認領」的呈現與建議動作、看板狀態不變；後者的認領操作呈現停用附繁中說明；本地分頁兩者皆無認領面
