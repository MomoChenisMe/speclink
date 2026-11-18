## ADDED Requirements

### Requirement: 本機專案的匯出與遷移產生相同的文件集合

對同一個本機專案，Node SDK 的 engine.exportScope() 回傳的包與 Speclink Desktop「遷移到 Server」送出的匯入包 SHALL 含相同的文件集合：把 Desktop 的包轉成以鍵標示的形式之後，兩者的鍵、內容與摘要 SHALL 逐項相同。兩者的中止條件 SHALL 相同：會讓 Desktop 的遷移中止的來源（壞掉的變更 metadata、引擎不支援的文件形式）SHALL 同樣讓 exportScope 失敗，並指出同一個檔案。

之後 Bundle 組裝的涵蓋範圍有任何增減（見「Bundle 組裝涵蓋本地 workspace 全集」），兩個入口 SHALL 同時跟著改變，SHALL NOT 出現一個入口有、另一個入口沒有的文件種類。Speclink Desktop 的遷移流程、確認畫面、送出的內容與成功後對本機的處理 SHALL 維持既有行為。

#### Scenario: 兩個入口的文件逐項相同

- **WHEN** 對一個含進行中變更（有 evidence）、已封存變更（有 evidence）、正式規格、進行中與已封存的討論、流程設定、詞彙表與看板順序的本機專案，分別取得 Speclink Desktop 的遷移匯入包與 engine.exportScope() 的包
- **THEN** 把前者轉成以鍵標示的形式後，兩者的文件數量相同，每一項的鍵、內容與摘要都相同

#### Scenario: 中止條件相同

- **WHEN** 本機專案有一個已封存變更的 metadata 無法解析，分別執行 Desktop 的遷移匯入包組裝與 engine.exportScope()
- **THEN** 兩者都失敗，訊息指出同一個檔案；兩者都沒有產生包

#### Scenario: Desktop 的遷移行為不變

- **WHEN** 使用者在 Speclink Desktop 對一個本機專案執行「遷移到 Server」到一個空的遠端專案
- **THEN** 遷移成功，遠端專案的內容與引入本需求之前的遷移結果相同；本機的備份與轉換照既有流程進行
