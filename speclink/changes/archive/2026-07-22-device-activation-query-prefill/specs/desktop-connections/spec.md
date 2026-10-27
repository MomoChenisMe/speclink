## MODIFIED Requirements

### Requirement: device login 預設與 PAT fallback
<!-- BEFORE: Desktop 開啟 verification 頁並輪詢，但未明定攜帶 user code 或未登入瀏覽器的登入往返。 -->

新增或登入連線時 Desktop SHALL 先嘗試 device flow（對 server 的 device 初始化端點探測）：支援時 SHALL 以 server 回傳的 verification URI 與 `user_code` 查詢參數開啟系統瀏覽器，並依 server 指示的間隔輪詢至核准、拒絕或逾時，逐一以可讀狀態回報。瀏覽器尚未登入 server 時，登入成功後 SHALL 返回同一裝置核准流程，裝置碼 SHALL 已預填且使用者 SHALL 經過下一步與明確核准／拒絕確認。明確不支援（404/405）時 SHALL 就地顯示 PAT 貼上輸入作為 fallback；網路不可達或 5xx SHALL 顯示連線錯誤、SHALL NOT 進入 PAT fallback。PAT 登入 SHALL 以身分查詢驗證有效後才存入 Keychain。登入成功後 SHALL 呈現該連線的身分顯示名。

#### Scenario: device login 完整走通

- **WHEN** 對支援 device flow 的 server 按下登入，且系統瀏覽器尚無 server session
- **THEN** 瀏覽器登入後返回已預填裝置碼的核准頁；使用者明確核准後，app 輪詢至 granted、存 refresh credential 入 Keychain、顯示登入身分

#### Scenario: 已登入瀏覽器直接進入預填流程

- **WHEN** 對支援 device flow 的 server 按下登入，且系統瀏覽器已有有效 server session
- **THEN** 瀏覽器直接顯示已預填裝置碼的核准頁，並保留下一步與明確核准／拒絕確認

#### Scenario: 不支援 device flow 才現 PAT 輸入

- **WHEN** 對回應 404 於 device 初始化端點的 server 按下登入
- **THEN** 就地顯示 PAT 輸入；輸入有效 PAT 後登入成功並顯示身分

#### Scenario: 瀏覽器端拒絕授權

- **WHEN** device login 輪詢期間使用者於瀏覽器拒絕該裝置
- **THEN** app 停止輪詢並顯示已拒絕的可讀狀態，不留任何 credential
