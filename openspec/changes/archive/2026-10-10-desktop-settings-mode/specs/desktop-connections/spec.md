## MODIFIED Requirements

### Requirement: 伺服器管理最小面

應用程式設定 SHALL 提供「伺服器」頁（app 全域範圍、與任何專案無關）：頁標題右側「新增連線」鈕展開頁頂的新增卡（URL 與顯示名、新增與取消；新增成功即收起並進入登入流程）。「連線」卡 SHALL 每個 saved server 一列——左為顯示名與狀態籤（已登入／未登入／登入過期／等待授權／失敗）、等寬 origin 與身分名；右為「登出」（已登入）或「登入」／「重新登入」（未登入／登入過期）與「⋯」選單（開啟專案、移除）；等待授權的細節與金鑰輸入 SHALL 於該列下方就地展開；無連線時顯示空狀態。「預設」卡 SHALL 有一列「新增遠端專案時預設使用」下拉（選項為已登入連線與「不預選」），偏好存 app 本機；新增專案對話框的 Server 步驟 SHALL 預選該連線（該連線不存在或未登入時，或對話框帶入口意圖開啟——指定連線、或資料夾的 remote marker 指向的伺服器——時不預選），使用者仍可改選。「登入方式」卡 SHALL 以兩列說明裝置登入（預設）與存取金鑰（伺服器不支援裝置登入時就地改用），不提供其他動作。登入成功時，該連線列 SHALL 伴隨開啟工作區的行動呼籲且該入口 SHALL 取得鍵盤焦點——引導使用者銜接下一步，SHALL NOT 自動開啟工作區選擇器。表單控制項 SHALL 使用專案自建 UI 元件、文案為繁體中文。此頁為最小管理面。

#### Scenario: 新增後清單即時反映

- **WHEN** 於伺服器頁按「新增連線」、填 URL 與顯示名後送出
- **THEN** 新增卡收起、連線卡立即出現該列並進入登入流程；完成登入後該列顯示身分名與「已登入」籤

#### Scenario: 登入成功聚焦開啟工作區

- **WHEN** 任一連線經 device 或 PAT 登入成功
- **THEN** 該連線列的開啟工作區入口取得鍵盤焦點且視覺顯眼；工作區選擇器未自動開啟

#### Scenario: 預設連線於新增專案預選

- **WHEN** 使用者於「預設」卡選定連線 A 為新增遠端專案時預設使用，之後開新增專案對話框選 Server 來源
- **THEN** Server 步驟預選 A（可改選）；若 A 已登出或被移除，則步驟不預選任何連線

#### Scenario: 帶入口意圖開啟時不預選預設連線

- **WHEN** 預設連線為已登入的 Y，使用者開啟一個 remote marker 指向伺服器 X（尚未登入）的本機資料夾，新增專案對話框因此直接進入 Server 步驟
- **THEN** Y 不被預選、頁尾不出現「下一步」；使用者須自行選擇連線或依 X 新增並登入

#### Scenario: 連線列的動作依狀態

- **WHEN** 伺服器頁有一個已登入與一個登入過期的連線
- **THEN** 前者右側為「登出」與「⋯」，後者右側為「重新登入」與「⋯」且狀態籤為警示色；「⋯」選單含開啟專案與移除

### Requirement: device login 預設與 PAT fallback

新增或登入連線時 Desktop SHALL 先嘗試 device flow（對 server 的 device 初始化端點探測）：支援時 SHALL 以 server 回傳的 verification URI 與 `user_code` 查詢參數開啟系統瀏覽器，並依 server 指示的間隔輪詢至核准、拒絕或逾時，逐一以可讀狀態回報。等待授權期間，發起登入的介面（伺服器頁的連線列，或工作區選擇器的新增並登入區）SHALL 就地呈現等待授權面：顯示裝置碼與驗證網址（兩者各附複製操作）、以有效期限起算的剩餘時間倒數、以及取消操作——使用者無需依賴已開啟的瀏覽器分頁即可換另一部裝置完成核准，也無需切換到其他頁面才看得到登入狀態。取消 SHALL 立即停止輪詢、該連線回到未登入狀態且 SHALL NOT 留存任何 credential。輪詢至逾時仍未核准時 SHALL 顯示逾時的可讀狀態。瀏覽器尚未登入 server 時，登入成功後 SHALL 返回同一裝置核准流程，裝置碼 SHALL 已預填且使用者 SHALL 經過下一步與明確核准／拒絕確認。明確不支援（404/405）時 SHALL 就地顯示 PAT 貼上輸入作為 fallback；網路不可達或 5xx SHALL 顯示連線錯誤、SHALL NOT 進入 PAT fallback。PAT 登入 SHALL 以身分查詢驗證有效後才存入 Keychain。登入成功後 SHALL 呈現該連線的身分顯示名。

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
