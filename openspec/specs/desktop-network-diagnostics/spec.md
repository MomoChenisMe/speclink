# desktop-network-diagnostics Specification

## Purpose

定義桌面應用程式的裝置層網路偏好、更新與安全標準下載共用的 Proxy 決策，以及可匯出且限制容量的診斷紀錄。邊界涵蓋本機設定、畫面錯誤及受管下載，不接管遠端專案的認證與資料連線，也不自動上傳診斷內容。

## Requirements

### Requirement: 裝置網路與診斷偏好

Desktop SHALL 在應用程式設定的「網路與診斷」分類提供 Debug mode、Proxy模式／位址、診斷匯出及重設設定；零專案、Remote離線或認證失效時仍可使用，不發送專案寫入或revision請求。偏好 SHALL 使用 formatVersion=1、debugEnabled:boolean、proxy:{mode:"system"|"manual"|"direct",url:string|null}，缺檔預設debugEnabled=false、mode=system、url=null；安裝版存應用程式設定目錄的runtime-preferences.json，portable存既有可攜資料根的desktop目錄。SHALL 寫暫存後原子替換、成功後才套用；失敗保留原檔及有效值並顯示settings_write_failed。壞JSON或未知版本 SHALL 顯示preferences_invalid、停止新受管網路工作，直到使用者修正或明確重設，不靜默覆寫。SHALL NOT 寫任何專案政策、SDD artifacts、touched或snapshot檔案。

#### Scenario: 零專案儲存與重開
- **WHEN** 沒有開啟專案的使用者啟用Debug mode並選手動Proxy http://proxy.example:8080，儲存後重開app
- **THEN** 設定及記錄開關保留，頁面顯示正在記錄，專案目錄未新增或修改檔案

#### Scenario: 偏好寫入失敗或損壞
- **WHEN** 寫入設定遭拒，或啟動時偏好檔有未知formatVersion
- **THEN** 前者顯示settings_write_failed並保留舊值，後者顯示preferences_invalid且新更新／標準下載不開始；只有明確重設成功才以預設值恢復


<!-- @trace
source: desktop-colors-network-and-offline-bundles
updated: 2026-10-19T23:00:19+08:00
-->

---
### Requirement: 受管下載的一致 Proxy 模式

更新檢查、更新下載、安全標準清單刷新及安裝下載 SHALL 套用相同Proxy決策。manual SHALL 只接受最長2048字元的http://host:port，port為1..65535，不含userinfo、非根path、query、fragment；HTTPS目的地經HTTP CONNECT。direct SHALL 明確略過環境與系統Proxy。system SHALL 先使用標準環境Proxy及NO_PROXY，再使用Windows單一固定HTTP Proxy／macOS固定HTTP或HTTPS Proxy；Linux僅讀程序繼承環境。SHALL 在頁面說明不支援PAC／WPAD、Windows分協定Proxy字串、Linux桌面私有設定、手動認證、SOCKS及自訂CA；系統未提供固定路由時 SHALL 顯示本次直接連線並記system-no-static-route，解析出不支援協定或認證時 SHALL 回proxy_unsupported。manual輸入錯誤 SHALL 顯示proxy_invalid，不儲存。已選定Proxy但連線失敗 SHALL NOT 改為直連。設定變更 SHALL 於下一個工作生效，進行中工作／候選更新的下載維持原快照並於頁上說明。

#### Scenario: 同一手動Proxy涵蓋更新與標準
- **WHEN** 使用者設定可達的HTTP Proxy後依序檢查更新、下載更新及下載標準資料
- **THEN** 三種請求皆由該Proxy接收，HTTPS目的地仍驗證來源憑證；診斷紀錄顯示manual及同一Proxy端點

#### Scenario: 直接連線忽略環境Proxy
- **WHEN** 程序環境有HTTPS_PROXY且使用者選直接連線後開始更新或標準下載
- **THEN** 環境Proxy沒有收到請求，紀錄顯示direct；連線失敗不改走另一種模式

#### Scenario: 系統路由與支援範圍
- **WHEN** 系統模式遇HTTPS_PROXY及NO_PROXY排除目的地，或沒有固定Proxy可用
- **THEN** 前者依排除規則直連，後者頁面與紀錄明示system-no-static-route及直接連線，不聲稱PAC已生效；系統給出SOCKS或需認證路由時顯示proxy_unsupported

#### Scenario: 非法設定與工作快照
- **WHEN** 使用者輸入含密碼的Proxy URL，或在已有更新下載時修改有效Proxy
- **THEN** 前者顯示proxy_invalid且舊值不變，後者既有下載繼續沿原路由、下一個工作使用新值


<!-- @trace
source: desktop-colors-network-and-offline-bundles
updated: 2026-10-19T23:00:19+08:00
-->

---
### Requirement: Proxy 與來源驗證分離

設定的Proxy端點 SHALL 允許私有或回環位址，但標準資料目的地仍 SHALL 限核准官方HTTPS主機與443、維持redirect白名單與最多三跳。直連 SHALL 沿用公開IP限制；經Proxy只允許解析並連至明確選定的Proxy端點，目的地主機透過Proxy解析並維持端到端TLS名稱驗證。SHALL 保留取消、期限、下載大小、授權與更新簽章檢查，SHALL NOT 因Debug或Proxy設定而放寬憑證驗證。

#### Scenario: 內網Proxy與不可信來源
- **WHEN** 使用者明確設定內網Proxy並下載官方標準，回應又導向未核准主機
- **THEN** 內網Proxy連線可進行，但未核准redirect被拒絕，舊標準資料不變；紀錄標示來源驗證失敗


<!-- @trace
source: desktop-colors-network-and-offline-bundles
updated: 2026-10-19T23:00:19+08:00
-->

---
### Requirement: 有界且遮蔽敏感資訊的診斷紀錄

Debug mode SHALL 預設關閉，開啟後記錄畫面錯誤、未處理的非同步錯誤、更新與標準下載的開始／結果，並使用同一operationId關聯工作。JSONL事件 SHALL 為固定schema，含formatVersion:number、timestamp:string、operationId:string、area:string、event:string、durationMs:number|null、destination:string|null、proxyMode:string|null、route:string|null、proxyEndpoint:string|null、httpStatus:number|null、errorCode:string|null、errorChain:array<{kind:string,code:string}>、droppedCount:number；分類至少區分DNS、connect、timeout、tls、proxy-auth、http、parse、cancelled與unknown。無回應或底層不公開的狀態 SHALL 記null並標示原因，不猜測HTTP碼或故障根因。SHALL NOT 記任意console內容、headers、body、密碼、金鑰、專案文件及完整本機路徑；目的地 SHALL 移除userinfo、query與fragment，非允許固定路徑只留origin，錯誤鏈只收允許的類別及代碼。

記錄 SHALL 存裝置設定根的diagnostics目錄，每檔最多2MiB、最多4檔共8MiB、單筆16KiB、佇列256筆，超額輪替或累計droppedCount。SHALL 不阻塞UI；寫入失敗 SHALL 顯示診斷記錄失敗而不改變下載結果。關閉完成後 SHALL 停止接收新事件且保留舊紀錄；重啟開關沿已儲存偏好。更新頁呈現失敗 SHALL 可記錄，錯誤畫面提供重試及前往網路與診斷的入口。

#### Scenario: 畫面錯誤及TLS問題有證據
- **WHEN** Debug已開啟，更新頁呈現失敗或官方下載憑證不被信任
- **THEN** 前者有frontend錯誤事件及重試／診斷入口，後者有tls分類、可取得的底層代碼與同一operationId；紀錄沒有假稱Proxy為根因

#### Scenario: 關閉與容量限制
- **WHEN** 關閉Debug後再發生錯誤，或開啟時紀錄／佇列達上限
- **THEN** 關閉後不新增該錯誤事件，開啟時以輪替或droppedCount維持限制且介面可操作

#### Scenario: 敏感內容與記錄失敗
- **WHEN** 外部錯誤包含Bearer token、Proxy密碼、帶query的URL及本機絕對路徑，或diagnostics目錄無法寫入
- **THEN** 事件只保留允許的分類、代碼與清理後目的地，秘密與完整路徑不落盤；記錄失敗有頁面狀態但原下載結果不被改寫


<!-- @trace
source: desktop-colors-network-and-offline-bundles
updated: 2026-10-19T23:00:19+08:00
-->

---
### Requirement: 診斷匯出與明確觸發的更新連線測試

Desktop SHALL 可在Debug開或關時匯出既有診斷，透過原生儲存選擇器產生最多9MiB的ZIP，含manifest.json的appVersion／os／arch／exportedAt／debugEnabled／skippedRecords及各JSONL檔name／sha256，不含專案或偏好檔。SHALL 再次按固定schema過濾事件，損壞資料計入skippedRecords，取消不輸出、寫入失敗不留下完成包且不覆寫原檔。

Debug開啟時 SHALL 提供「測試更新連線」，只在使用者觸發後請求內建更新HTTPS endpoints，沿相同Proxy設定／User-Agent，以GET取得狀態或錯誤鏈，限制20秒、實際2MiB及三跳HTTPS redirects；SHALL 不接受任意網址、不下載安裝檔、不改更新狀態。結果 SHALL 明示是獨立測試，回{operationId:string,results:[{destination:string,httpStatus:number|null,errorCode:string|null,durationMs:number}]}，紀錄area=network-probe。更新元件沒有公開的狀態 SHALL 保持null，SHALL NOT 自動多發測試或將測試結果冒充原更新回應。

#### Scenario: 關閉後匯出與取消
- **WHEN** 關閉Debug後匯出舊紀錄，或取消儲存選擇器
- **THEN** 前者得到含版本與已遮蔽紀錄的ZIP，後者不建檔、不顯示成功；沒有可匯出紀錄時清楚顯示空狀態

#### Scenario: 更新連線測試收到407
- **WHEN** Debug已啟用，使用者點測試更新連線且Proxy要求認證
- **THEN** 頁面與network-probe紀錄顯示可觀察的407或proxy-auth錯誤與耗時，不下載或安裝更新，不把該結果冒稱之前更新請求的回應


<!-- @trace
source: desktop-colors-network-and-offline-bundles
updated: 2026-10-19T23:00:19+08:00
-->