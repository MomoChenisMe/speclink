## ADDED Requirements

### Requirement: 後台狀態徽章語意色

Console 的狀態徽章 SHALL 依語意上色:正常/啟用/有效為綠系;異常為紅系;停權為琥珀系;已撤銷為中性且與有效態可一眼區辨。祕密揭示橫幅(PAT 建立後的一次性明碼、邀請連結揭示)SHALL 為綠系成功樣式,SHALL NOT 使用主題主色。純 metadata 徽章(成員角色、稽核來源)SHALL 維持中性。同一事實於同頁的橫幅與徽章 SHALL 呈現一致的語意色層級。

#### Scenario: 儲存健康徽章

- **WHEN** 總覽頁與系統頁呈現儲存 online/offline 狀態
- **THEN** 徽章分別以綠系/紅系呈現,與同頁的離線警示橫幅語意一致

#### Scenario: 成員狀態徽章

- **WHEN** 成員清單呈現 active 與停權成員
- **THEN** active 為綠系、停權為琥珀系

#### Scenario: 憑證狀態徽章

- **WHEN** PAT 清單、裝置憑證清單或工作階段清單呈現有效與已撤銷項目
- **THEN** 有效為綠系、已撤銷為中性,兩態可區辨

#### Scenario: 揭示橫幅為成功語意

- **WHEN** 建立 PAT 後顯示一次性明碼、或產生邀請連結
- **THEN** 揭示橫幅以綠系成功樣式呈現,非主題主色
