## MODIFIED Requirements

### Requirement: device login 預設與 PAT fallback

新增或登入連線時 Desktop SHALL 先嘗試 device flow（對 server 的 device 初始化端點探測）：支援時 SHALL 以 server 回傳的 verification URI 與 `user_code` 查詢參數開啟系統瀏覽器，並依 server 指示的間隔輪詢至核准、拒絕或逾時，逐一以可讀狀態回報。等待授權期間，發起登入的介面（伺服器頁籤的連線列，或工作區選擇器的新增並登入區）SHALL 就地呈現等待授權面：顯示裝置碼與驗證網址（兩者各附複製操作）、以有效期限起算的剩餘時間倒數、以及取消操作——使用者無需依賴已開啟的瀏覽器分頁即可換另一部裝置完成核准，也無需切換到其他頁面才看得到登入狀態。取消 SHALL 立即停止輪詢、該連線回到未登入狀態且 SHALL NOT 留存任何 credential。輪詢至逾時仍未核准時 SHALL 顯示逾時的可讀狀態。瀏覽器尚未登入 server 時，登入成功後 SHALL 返回同一裝置核准流程，裝置碼 SHALL 已預填且使用者 SHALL 經過下一步與明確核准／拒絕確認。明確不支援（404/405）時 SHALL 就地顯示 PAT 貼上輸入作為 fallback；網路不可達或 5xx SHALL 顯示連線錯誤、SHALL NOT 進入 PAT fallback。PAT 登入 SHALL 以身分查詢驗證有效後才存入 Keychain。登入成功後 SHALL 呈現該連線的身分顯示名。

#### Scenario: device login 完整走通

- **WHEN** 對支援 device flow 的 server 按下登入，且系統瀏覽器尚無 server session
- **THEN** 瀏覽器登入後返回已預填裝置碼的核准頁；使用者明確核准後，app 輪詢至 granted、存 refresh credential 入 Keychain、顯示登入身分

#### Scenario: 等待授權面顯示碼與網址

- **WHEN** device login 進入等待授權（瀏覽器已開啟、授權尚未核准）
- **THEN** 該連線列顯示裝置碼與驗證網址並各附複製操作、剩餘時間倒數與取消；複製裝置碼後剪貼簿內容即為該碼

#### Scenario: 從工作區選擇器發起登入同樣呈現等待授權面

- **WHEN** 於工作區選擇器的 server 步驟新增連線並按下登入，且 server 支援 device flow
- **THEN** 選擇器內就地顯示等待授權面（裝置碼、驗證網址、倒數與取消）；取消即停止輪詢並停留在 server 步驟；server 明確不支援時就地顯示 PAT 輸入

#### Scenario: 取消等待不留 credential

- **WHEN** 等待授權期間使用者按下取消
- **THEN** 輪詢停止、連線列回到未登入狀態、Keychain 無任何新增 credential；瀏覽器端該授權請求自然逾期

#### Scenario: 已登入瀏覽器直接進入預填流程

- **WHEN** 對支援 device flow 的 server 按下登入，且系統瀏覽器已有有效 server session
- **THEN** 瀏覽器直接顯示已預填裝置碼的核准頁，並保留下一步與明確核准／拒絕確認

#### Scenario: 不支援 device flow 才現 PAT 輸入

- **WHEN** 對回應 404 於 device 初始化端點的 server 按下登入
- **THEN** 就地顯示 PAT 輸入；輸入有效 PAT 後登入成功並顯示身分

#### Scenario: 瀏覽器端拒絕授權

- **WHEN** device login 輪詢期間使用者於瀏覽器拒絕該裝置
- **THEN** app 停止輪詢並顯示已拒絕的可讀狀態，不留任何 credential

### Requirement: 伺服器管理最小面

應用程式設定頁 SHALL 提供伺服器頁籤（app 全域範圍、與任何專案分頁無關）：呈現 saved servers 清單（顯示名、origin、登入狀態與身分）、新增連線（URL 與顯示名）、登入、登出與移除操作。登入成功時，該連線列 SHALL 伴隨開啟工作區的行動呼籲且該入口 SHALL 取得鍵盤焦點——引導使用者銜接下一步，SHALL NOT 自動開啟工作區選擇器。表單控制項 SHALL 使用專案自建 UI 元件、文案為繁體中文。此頁籤為最小管理面。

#### Scenario: 新增後清單即時反映

- **WHEN** 於伺服器頁籤新增 URL 與顯示名
- **THEN** 清單立即出現該條目並進入登入流程；完成登入後條目顯示身分名

#### Scenario: 登入成功聚焦開啟工作區

- **WHEN** 任一連線經 device 或 PAT 登入成功
- **THEN** 該連線列的開啟工作區入口取得鍵盤焦點且視覺顯眼；工作區選擇器未自動開啟
