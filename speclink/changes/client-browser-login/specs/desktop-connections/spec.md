## MODIFIED Requirements

### Requirement: device login 預設與 PAT fallback
<!-- BEFORE: 一律先試 device flow，不探索 server 宣告的登入方式；沒有瀏覽器登入與等待瀏覽器登入面 -->

新增或登入連線時 Desktop SHALL 先探索 server 宣告的登入方式，並依 remote-auth「登入方式的探索與選擇」的固定順序（瀏覽器登入、device flow、PAT）選擇；探索回應錯誤或 server 不可達時 SHALL 顯示連線錯誤、SHALL NOT 進入任何登入方式。選到瀏覽器登入時 Desktop SHALL 開啟系統瀏覽器到登入服務的授權網址，並於發起登入的介面（伺服器頁的連線列，或工作區選擇器的新增並登入區）就地呈現等待瀏覽器登入面：一句說明、授權網址（附複製操作）、以期限起算的剩餘時間倒數、以及取消操作；完成後 SHALL 將 refresh credential 存入 Keychain 並呈現身分顯示名；被拒絕與逾時 SHALL 各顯示可讀狀態；取消 SHALL 立即停止等待、該連線回到未登入狀態且 SHALL NOT 留存任何 credential。選到 PAT 時 SHALL 就地顯示 PAT 貼上輸入。server 沒有宣告登入方式時 SHALL 先嘗試 device flow（對 server 的 device 初始化端點探測）。走 device flow 且 server 支援時 SHALL 以 server 回傳的 verification URI 與 `user_code` 查詢參數開啟系統瀏覽器，並依 server 指示的間隔輪詢至核准、拒絕或逾時，逐一以可讀狀態回報。等待授權期間，發起登入的介面（伺服器頁的連線列，或工作區選擇器的新增並登入區）SHALL 就地呈現等待授權面：顯示裝置碼與驗證網址（兩者各附複製操作）、以有效期限起算的剩餘時間倒數、以及取消操作——使用者無需依賴已開啟的瀏覽器分頁即可換另一部裝置完成核准，也無需切換到其他頁面才看得到登入狀態。取消 SHALL 立即停止輪詢、該連線回到未登入狀態且 SHALL NOT 留存任何 credential。輪詢至逾時仍未核准時 SHALL 顯示逾時的可讀狀態。瀏覽器尚未登入 server 時，登入成功後 SHALL 返回同一裝置核准流程，裝置碼 SHALL 已預填且使用者 SHALL 經過下一步與明確核准／拒絕確認。明確不支援（404/405）時 SHALL 就地顯示 PAT 貼上輸入作為 fallback；網路不可達或 5xx SHALL 顯示連線錯誤、SHALL NOT 進入 PAT fallback。PAT 登入 SHALL 以身分查詢驗證有效後才存入 Keychain。登入成功後 SHALL 呈現該連線的身分顯示名。

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

#### Scenario: server 宣告瀏覽器登入時走瀏覽器登入

- **WHEN** 對宣告瀏覽器登入的 server 按下登入
- **THEN** 系統瀏覽器開啟登入服務的授權網址；連線列就地顯示等待瀏覽器登入面（說明、可複製的授權網址、倒數與取消），沒有顯示裝置碼；使用者在瀏覽器完成登入後，Keychain 出現該 origin 的 refresh credential，連線列顯示登入身分

#### Scenario: 等待瀏覽器登入時取消

- **WHEN** 等待瀏覽器登入期間使用者按下取消
- **THEN** 等待停止、連線列回到未登入狀態、Keychain 無任何新增 credential；之後瀏覽器端完成登入也不會讓該連線變成已登入

#### Scenario: 從工作區選擇器發起瀏覽器登入

- **WHEN** 於工作區選擇器的 server 步驟新增一台宣告瀏覽器登入的 server 並按下登入
- **THEN** 選擇器內就地顯示等待瀏覽器登入面；完成後該 server 出現在已登入的清單，可以繼續選擇專案

#### Scenario: 瀏覽器登入被拒絕

- **WHEN** 等待瀏覽器登入期間使用者於登入服務拒絕授權
- **THEN** 連線列顯示已拒絕的可讀狀態，不留任何 credential

#### Scenario: server 只宣告存取金鑰

- **WHEN** 對只宣告 token 的 server 按下登入
- **THEN** 就地顯示 PAT 輸入，沒有先嘗試 device flow

#### Scenario: 探索失敗顯示連線錯誤

- **WHEN** server 對登入方式的探索回應 500，按下登入
- **THEN** 顯示連線錯誤；沒有進入 PAT 輸入，也沒有開啟瀏覽器

### Requirement: 登出撤銷與移除連帶清理
<!-- BEFORE: 登出只刪 refresh 與 PAT 條目，不清 access token 快取；沒有瀏覽器登入的撤銷 -->

登出 SHALL 刪除該 origin 的全部 Keychain entry（refresh credential、access token 快取、PAT）並清除 registry 的身分顯示名，清除的範圍 SHALL 與 speclink auth logout 對金鑰圈的清除相同；持 device 流程的 refresh credential 者 SHALL 盡力呼叫 server 的 revoke 端點撤銷 device family，持瀏覽器登入的 refresh credential 且登入服務列有撤銷端點者 SHALL 盡力向該端點撤銷——server 或登入服務不可達時撤銷失敗 SHALL NOT 阻擋本機刪除；PAT 登出 SHALL 刪除本機 entry 並提示於 server 帳號頁撤銷。移除連線 SHALL 先執行登出語意再刪 registry 條目。

#### Scenario: 登出撤銷 device family

- **WHEN** 對已 device login 的連線按下登出
- **THEN** Keychain entry 刪除、server 端該 device family 被撤銷（帳號頁 device 清單不再含該裝置）、清單呈現未登入狀態

#### Scenario: server 離線時登出仍完成本機清理

- **WHEN** server 不可達時按下登出
- **THEN** 本機 Keychain entry 與身分顯示名仍被清除，UI 呈現未登入

#### Scenario: 登出後 CLI 也回到未登入

- **WHEN** 對已登入的連線按下登出，之後於同機對同一 origin 執行 speclink auth status（無環境變數、無憑證檔）
- **THEN** CLI 顯示未登入；Keychain 無該 origin 的 refresh、access token 快取與 PAT 條目

#### Scenario: 登出瀏覽器登入的連線

- **WHEN** 對以瀏覽器登入的連線按下登出，登入服務列有撤銷端點
- **THEN** 登入服務的撤銷端點收到請求；Keychain entry 刪除；清單呈現未登入狀態

### Requirement: 伺服器管理最小面
<!-- BEFORE: 登入方式卡只有兩列：裝置登入（預設）與存取金鑰 -->

應用程式設定 SHALL 提供「伺服器」頁（app 全域範圍、與任何專案無關）：頁標題右側「新增連線」鈕展開頁頂的新增卡（URL 與顯示名、新增與取消；新增成功即收起並進入登入流程）。「連線」卡 SHALL 每個 saved server 一列——左為顯示名與狀態籤（已登入／未登入／登入過期／等待授權／失敗）、等寬 origin 與身分名；右為「登出」（已登入）或「登入」／「重新登入」（未登入／登入過期）與「⋯」選單（開啟專案、移除）；等待授權的細節與金鑰輸入 SHALL 於該列下方就地展開；無連線時顯示空狀態。「預設」卡 SHALL 有一列「新增遠端專案時預設使用」下拉（選項為已登入連線與「不預選」），偏好存 app 本機；新增專案對話框的 Server 步驟 SHALL 預選該連線（該連線不存在或未登入時，或對話框帶入口意圖開啟——指定連線、或資料夾的 remote marker 指向的伺服器——時不預選），使用者仍可改選。「登入方式」卡 SHALL 以三列說明瀏覽器登入（伺服器宣告支援時優先使用）、裝置登入與存取金鑰（伺服器不支援前兩者時就地改用），不提供其他動作。登入成功時，該連線列 SHALL 伴隨開啟工作區的行動呼籲且該入口 SHALL 取得鍵盤焦點——引導使用者銜接下一步，SHALL NOT 自動開啟工作區選擇器。表單控制項 SHALL 使用專案自建 UI 元件、文案為繁體中文。此頁為最小管理面。

#### Scenario: 新增後清單即時反映

- **WHEN** 於伺服器頁按「新增連線」、填 URL 與顯示名後送出
- **THEN** 新增卡收起、連線卡立即出現該列並進入登入流程；完成登入後該列顯示身分名與「已登入」籤

#### Scenario: 登入成功聚焦開啟工作區

- **WHEN** 任一連線經瀏覽器登入、device 或 PAT 登入成功
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

#### Scenario: 登入方式卡列出三種方式

- **WHEN** 開啟伺服器頁
- **THEN** 「登入方式」卡有三列，依序是瀏覽器登入、裝置登入、存取金鑰；介面語言為 en 時三列以英文呈現

### Requirement: credential 唯一落點為 OS Keychain
<!-- BEFORE: 列舉的 credential 只有 device 流程的 refresh credential 與 PAT -->

登入取得的 credential（瀏覽器登入與 device 流程的 refresh credential、PAT 流程的 PAT）SHALL 逐 server origin 存入 OS Keychain（macOS Keychain／Windows Credential Manager），SHALL NOT 出現於 localStorage、registry 檔、repo、URL、log 或任何 TS 可見狀態；TS 層 SHALL 只見連線狀態與身分顯示名，PAT 僅於使用者貼上時單次過境命令參數。瀏覽器登入的授權網址 SHALL 可顯示於介面，它 SHALL NOT 含授權碼、PKCE verifier 或任何 token；授權碼與 PKCE verifier SHALL 只存在於 Rust 側。access token SHALL 短效僅存 Rust 記憶體；refresh rotation 成功後 SHALL 立即以新 refresh credential 覆寫 Keychain entry。

#### Scenario: Keychain entry 隨登入建立

- **WHEN** 對本地 dev server 完成 device login
- **THEN** 系統 Keychain 出現該 origin 的 refresh credential entry，且 registry 檔與 localStorage 皆無 token 內容

#### Scenario: rotation 後舊 credential 失效仍可用

- **WHEN** access token 過期後 app 以 refresh credential 換新（rotation）再重啟 app
- **THEN** 重啟後以 Keychain 中最新 refresh credential 成功取得 access token，無需重新登入

#### Scenario: 瀏覽器登入不讓 TS 看見憑證

- **WHEN** 對宣告瀏覽器登入的 server 完成登入後檢視 registry 檔、localStorage 與前端收到的事件內容
- **THEN** 三者皆無 access token、refresh token、授權碼與 PKCE verifier；前端只收到登入階段、授權網址與身分顯示名
