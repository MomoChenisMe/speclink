## ADDED Requirements

### Requirement: claim 端點持久化與 ownership 衝突語意

POST /changes/{name}/claim SHALL 經 Command gateway 直通引擎的 Claim 命令（移除回聲 stub）：認領成功時回應攜帶認領後的 claimedBy 且寫入隨 Unit of Work 落盤；同人重複認領回應成功且零寫入；已被他人認領 SHALL 回 HTTP 409、reason 為八值封閉 registry 既有的 refused、message 含目前持有人與建議動作（SHALL NOT 擴充 error reason registry）；change 不存在回 404；本端點 SHALL 比照其他寫入動詞為 editor 限定（reader 收 403、scope 零改動）。變更清單與單 change 讀取回應的 claimedBy SHALL 自 meta 的 claimed_by 組裝（未認領即省略），使認領跨重啟、跨裝置可見。

#### Scenario: 認領落盤且清單可見

- **WHEN** editor 對未認領的 change 呼叫 POST /changes/{name}/claim，隨後呼叫 GET /changes 與 GET /changes/{name}
- **THEN** claim 回應含呼叫者為 claimedBy；兩個讀取回應的該 change 皆含同值 claimedBy；server 重啟後讀取結果不變

#### Scenario: 他人認領衝突與 reader 拒絕

- **WHEN** 另一 editor 對已認領的 change 呼叫同端點，接著一位 reader 對未認領的 change 呼叫同端點
- **THEN** 前者收 409、reason 為 refused、message 含目前持有人、meta 零改動；後者收 403、scope 零改動
