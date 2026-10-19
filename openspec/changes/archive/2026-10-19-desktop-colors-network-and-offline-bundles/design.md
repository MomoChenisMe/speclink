## Context

來源討論的六項需求已定案。本次是新增診斷、網路與資料交換功能，並修正配色與設定導覽。既有程式的三個邊界：projectTileClass 已供主視窗與系統匣共用；OfficialTransport 用 ureq 2.12.1 且關閉環境 Proxy，resolver 只准官方公開主機；更新使用 Tauri updater 2.10.1，底層 reqwest 0.13。現有 PackageManifest 是單一標準版本 formatVersion=1，FsSecurityData 已有版本鎖、generation pin、指紋驗證與原子 current 指標。

已核對本機套件原始碼：UpdaterBuilder 提供 proxy、no_proxy、configure_client，Update 保存同一設定供下載；hyper-util 0.1.20 的 Matcher::from_system 可共用環境／系統固定 Proxy 決策。更新檢查遇非成功 HTTP 時會隱去部分回應資訊，所以不能保證從 plugin 的最終字串還原狀態碼。

## Goals / Non-Goals

**Goals:** 涵蓋討論的六項要求；裝置偏好與記錄不依賴目前專案；CLI 與 Desktop 共用同一 Host 資料交換流程；每個寫入操作的失敗結果明確且不破壞該版本舊資料。

**Non-Goals:** 不改 TeamStore、Protocol、Server Web、Remote 認證與 SSE 的網路路由；不讓桌面 Proxy 設定自動影響獨立 CLI 程序；不支援手動 Proxy 密碼、NTLM／Kerberos、SOCKS、PAC／WPAD、自訂公司 CA 或關閉 TLS 檢查。公司失敗根因仍須實際紀錄判讀。本次不增加色票數量，不更動專案政策／技能，不把所有版本提交升級成跨版本原子交易。

## Decisions

### 固定深淺色票

在 packages/ui/src/theme.css 定義固定 token，不再用固定混色百分比表示未選取；apps/desktop/src/components/projectTile.ts 是唯一色票樣式對應，主視窗與系統匣沿用。ProjectSettingsView 的預覽明確使用選取態樣式，但 radio checked 與外框仍取真實選擇。所有字色固定選取白／未選取黑，不隨亮暗主題漂移。

| 色票 | 深底（選取及預覽） | 淺底（未選取） |
| --- | --- | --- |
| 預設 | #374151 | #E5E7EB |
| 青綠 | #0F766E | #99F6E4 |
| 藍 | #1D4ED8 | #BFDBFE |
| 紫 | #9400D3 | #DA70D6 |
| 粉紅 | #BE185D | #FBCFE8 |
| 橘 | #C2410C | #FED7AA |
| 綠 | #15803D | #BBF7D0 |

hover 沿既有 token 結構，只改背景，不改選取語意。radio 的焦點框與 disabled、失敗保留舊值沿用；不在呼叫端覆蓋 @speclink/ui 元件變體。

### 設定頁點專案返回看板

App 的返回處理抽出單一「離開設定前」入口，保存待執行意圖為返回或指定專案 key，沿用現有 ConfirmDialog。專案列點擊經此入口，取消不呼叫 activateTab；確認後才執行待定意圖。store 的 activateTab 在探測／Remote既有session或重連成功後，若該次操作自設定模式發起，於提交 activeKey 的同次狀態更新切 boardView=board；非設定頁維持原切換行為，並沿用清除詳情／浮層的啟用狀態；點同專案不提早返回。切換失敗維持舊頁與專案，顯示既有錯誤。快捷鍵切換專案也走同一離開設定入口；在非設定頁維持原有切換行為。取消對話框、過期切換與快速連點不執行較早意圖。

### 裝置偏好與設定分類

新增「網路與診斷」於伺服器與安全標準資料之間，既有安全標準資料分類不可遺失；分類目錄、搜尋列鍵、記憶及頁面分派皆共用 settingsCatalog 與 AppSettingsView。沿用 SettingsPageHeader、SettingsCard、SettingsRow、Button、Input、Select、Switch、ConfirmDialog，新增頁不自行重造原語。

apps/desktop/core/src/runtime_preferences.rs 擁有 RuntimePreferences 的驗證與儲存：serde camelCase，檔案為 {formatVersion:1,debugEnabled:false,proxy:{mode:"system",url:null}}；mode 為 system／manual／direct。安裝版使用既有 connections_path 的設定目錄根；portable 為可攜根下 desktop。先寫同目錄暫存、sync，再原子替換 runtime-preferences.json，成功才發布新的記憶體設定；失敗回 settings_write_failed 且舊設定／檔案保留。缺檔使用上述預設；未知formatVersion或壞JSON顯示 preferences_invalid，停止新受管網路工作，提供明確「重設網路與診斷設定」動作，不能靜默覆寫。重設亦經相同原子寫入。

Tauri runtime_commands 只委派 desktop-core 與注入的路徑／事件橋接。IPC 使用 runtime_preferences_read、runtime_preferences_write、diagnostics_append、diagnostics_export、network_probe_update；frontend adapter/runtime.ts 是唯一 invoke 面。設定頁、診斷匯出與網路狀態在零專案、Remote離線、登入過期時都可用，不發送專案 revision 寫入。

### 一致的 Proxy 決策

crates/host/speclink-host/src/network.rs 定義 ProxyPolicy（設定快照）、ResolvedRoute（direct 或 http_proxy）與結構化 NetworkDiagnostic，不存桌面路徑。desktop-core/network.rs 讀裝置偏好並建立系統路由快照，再把快照傳給 Host 的下載器與更新橋接，供每次請求／redirect 決策。CLI 沿原有網路預設，不自動讀桌面檔。

- manual 僅接受明確 http://host:port，port 1..65535、URL最多2048字元，不接受使用者資訊、非根路徑、query／fragment。私有位址與loopback僅可作使用者設定的Proxy端點；不放寬資料來源URL。拒絕HTTPS Proxy端點與SOCKS並指出不支援，HTTPS目的地透過HTTP CONNECT通道連線。
- direct 必須明確停用環境／系統代理，不能只將Proxy設為None；ureq維持try_proxy_from_env(false)，updater使用no_proxy。
- system 使用 hyper-util 的 client-proxy／client-proxy-system matcher，環境HTTPS_PROXY／HTTP_PROXY／ALL_PROXY與NO_PROXY優先，再取Windows使用者固定Proxy或macOS固定HTTP／HTTPS Proxy；Linux定義為程序繼承的標準Proxy環境變數，不讀GNOME／KDE私有設定。Windows第一版只承諾單一固定HTTP Proxy，分協定設定字串不列為支援；PAC／WPAD不執行。設定頁持續顯示這些範圍，未取得固定路由時明示「系統未提供可用固定 Proxy，本次直接連線」，診斷記錄system-no-static-route，不宣稱PAC已生效。解析出不支援的協定或認證時回proxy_unsupported，不默默改直連。
- manual／system 已選定Proxy但連線失敗，不偷偷fallback到直連。新設定只影響下一個工作；已開始工作與PendingUpdate持有自己的設定快照，頁面提示下載中變更於下次工作生效。

OfficialTransport 建構改接受路由及診斷回呼；官方HTTPS host／redirect白名單與TLS身份驗證維持。直連沿用公開IP resolver；經代理僅對已明確選定的Proxy host:port啟用有界解析，准其私有位址，目的地仍限制官方HTTPS主機與443，代理負責解析目的地、TLS仍核對官方名稱。不對全部DNS解除限制。proxy DNS同樣有取消、並行數與截止限制。

替代方案：只寫環境變數會漏掉現有停用Proxy的下載器、影響其他連線且有程序內競態，因此不用；把所有HTTP client改成一套會擴大到Remote連線，因此僅共用決策與診斷資料。

### 有界診斷紀錄與匯出

desktop-core/diagnostics.rs 實作單一裝置記錄器，位於設定根的 diagnostics；JSON Lines 每檔2MiB、最多4檔共8MiB，單筆最多16KiB，待寫佇列最多256筆，滿時累計 droppedCount，不阻塞UI。檔名固定 events.jsonl 及三個輪替檔；寫入權限依平台限制為目前使用者。記錄I/O失敗只顯示「診斷紀錄無法寫入」狀態，不使更新／標準工作失敗。Debug預設關閉；開啟後才接收事件，關閉完成後拒收新事件且不刪既有紀錄。重啟沿用明確儲存的開關，頁上持續顯示正在記錄。

事件固定欄位：formatVersion:number、timestamp:UTC字串、operationId:string、area:frontend|updater|security-data|network-probe、event:string、durationMs:number|null、destination:string|null、proxyMode:system|manual|direct|null、route:direct|http_proxy|null、proxyEndpoint:string|null、httpStatus:number|null、errorCode:string|null、errorChain:array<{kind:string,code:string}>、droppedCount:number。目的地與Proxy只留scheme／host／port及允許的固定官方路徑，刪除userinfo、query／fragment；未知動態路徑只留origin。不得接收任意console文字、headers、body、金鑰、密碼、專案文件或完整本機路徑。錯誤鏈先分類DNS／connect／timeout／tls／proxy-auth／http／parse／cancelled／unknown與可核對的OS／TLS錯誤代碼，不直接序列化第三方Error的Debug字串；無法分類時明示unknown，不捏造原因。

apps/desktop/src/diagnostics.tsx 設應用程式頂層ErrorBoundary與單一window error／unhandledrejection監聽，記錄錯誤類型、app-relative程式檔及行列等允許欄位（以固定event／errorChain表示，不傳任意錯誤message），包含更新頁render失敗；只註冊一次、只在開關啟用時提交。ErrorBoundary提供重試畫面與前往網路與診斷的出口。

匯出走原生save、取一致的輪替快照，生成ZIP：manifest.json含appVersion、os、arch、exportedAt、debugEnabled及檔案name／sha256，加四個以內的JSONL，總輸出限制9MiB；不包偏好檔或專案。關閉Debug仍能匯出既有紀錄。匯出前再次以固定schema檢查過濾，損壞紀錄列在manifest的skippedRecords:number，不原樣帶出。取消選檔不建檔；寫入失敗不留看似完成的包。

### 更新橋接與連線測試

更新仍由tauri-plugin-updater檢查、下載、驗簽、安裝；將JS adapter改呼叫Rust updater_commands，保留UpdaterAdapter與PendingUpdate介面。Rust建立UpdaterBuilder，使用no_proxy及configure_client明確配置reqwest Proxy::custom，所有endpoint與下載redirect由同一ProxyPolicy解路由；下載持有檢查時的設定快照。pin tauri-plugin-updater到已核對的2.10.1並顯式加入reqwest 0.13相同TLS功能，避免configure_client型別隨隱式升級漂移。Tauri狀態只存目前候選Update與id，不重造更新狀態機；診斷／Proxy決策留desktop-core與Host，殼只做必要plugin adapter。

保留現有取消下載後不安裝、重新啟動、版本檢查及失敗呈現；加入開始／結束事件及可觀察錯誤鏈。plugin未提供的HTTP狀態記null與updater-response-unavailable，禁止從文字猜碼或開啟plugin任意verbose log（其原始輸出會含整個manifest）。

Debug開啟時顯示「測試更新連線」，只由使用者按鈕觸發GET目前內建updater HTTPS endpoints，不接受任意URL、不下載安裝檔。共用相同reqwest client設定、User-Agent、route與timeout，限制20秒、實際回應2MiB、最多3個HTTPS redirects，取得httpStatus及錯誤鏈；失敗仍寫紀錄並在頁上顯示簡要結果，不修改更新狀態或安裝。此測試是獨立請求，紀錄標network-probe，不能宣稱等於先前那次更新的結果。尚未啟用時按鈕不出現，不自動多發請求。

### 單包匯出與集合包匯入

speclink-core/security/data.rs 擴充純資料DTO ExportSource（metadata、原始檔名與bytes）、ExportReceipt、CollectionManifest、BatchImportReceipt；StandardRepository 增加讀取匯出來源能力。speclink-fs/security.rs 共用既有generation讀取／pin／指紋檢查，將原本讀完丟棄的raw bytes交給export，避免依猜測組磁碟路徑或混代。

ZIP codec與同目錄暫存發佈放speclink-fs/security_packages.rs；Host/security_data.rs唯一裁決選取清單、順序、取消與逐版本提交，沿現有decoder注入模式使用FS codec。CLI與desktop-core均經Host，不各自組包。檔案I/O不放進純領域演算法；不影響TeamStore或遠端Protocol，Remote專案的資料管理仍是目前裝置作業。

單包沿用formatVersion=1的PackageManifest與允許的raw／LICENSE內容，保留sourceRevision、sourceUrl、sourceFormat、licenseName、licenseUrl、attribution與每檔SHA。不放parsed.json、generation、鎖或專案內容；CWE以安裝時保存的raw XML與LICENSE產生可匯入的標準包，原ZIP若仍屬既有有效來源內容才依原manifest規則保留，禁止跳過現有格式檢查。重新匯入一律IntegrityKind::Imported，內容指紋不等同官方背書。

全部匯出用集合ZIP，manifest為 {formatVersion:2,packages:[{standardId,version,name,sha256}]}，name精確為packages/<standardId>/<version>.zip；每個內包是原v1單包，外層採Stored避免再壓縮。最多32個內包、33個outer entries、manifest64KiB、outer檔及內包總bytes512MiB；每內包保留64MiB壓縮／256MiB展開／128entries限制，全部內層raw累計最多1GiB。拒絕ZIP64、encrypted、symlink、executable、路徑穿越、大小寫碰撞、重複標準版本、未列entry與遞迴集合包；大小按實際讀取核對。單包及集合包都不得任意extract，僅用受控暫存。

選取／發佈：開始時凍結已安裝清單，依standardId、version字串排序；全部僅選installed有效項，invalid逐項列skipped；沒有有效項回missing_data且不輸出空包。對已選版本取得pin並核對raw檔／授權，讀取失敗整次export失敗，不少包冒稱全部。串流寫同目錄暫存（最多一個內包在記憶體），完成ZIP與sync後無覆寫原子發佈；以tempfile的persist_noclobber並核對平台支援，已存在回file_exists，磁碟／取消／發佈不支援回明確錯誤並清暫存，原目的檔保留。安裝資料與current指標完全不改。

匯入集合：先在受控暫存檢查整個outer及每個inner（含SHA、來源、授權、解析），限一個版本資料在記憶體，其餘staging保存；任一預檢失敗無版本提交。通過後按manifest順序逐版本取得既有安裝鎖、提交完整generation及current。提交失敗或取消就停止，先前成功保留、當前失敗版本保舊、後續notAttempted；清理失敗依現有成功附warning語意處理，不把已提交改報未提交。選此明示部分成功而非跨版本rollback，因現有契約僅每版本原子，避免新增全域鎖／日誌交易。

## Implementation Contract

- CLI export語法：security data export [STANDARD --version VERSION | --all] --output PATH [--json]。STANDARD與--all互斥，--version只搭STANDARD；無stdin，既有global --no-color適用。成功0／操作失敗1／取消130／clap錯誤2。人眼stdout列路徑、包數、各標準版本與跳過項；JSON stdout單物件為 {path:string,formatVersion:number,bytes:number,packages:[{standardId:string,version:string,sha256:string}],skipped:[{standardId:string,version:string,reason:string}]}；進度與失敗stderr，失敗沒有成功payload。
- 既有import PATH不變；v1單包的輸出位元級保持原樣。v2集合成功／部分成功結果為 {formatVersion:2,status:"installed"|"partial"|"failed"|"cancelled",results:[{standardId:string,version:string,status:"installed"|"failed"|"notAttempted",sha256:string|null,errorCode:string|null}]}，JSON至stdout，人眼列相同逐項結果；完成exit0，提交失敗exit1，取消exit130。預檢失敗stdout為空，錯誤stderr；提交開始後即使首項失敗也輸出明示failed的清單。取消後已成功項不撤銷。
- Desktop DataRequest增加export（standardId／version或all擇一、path），既有import分辨v1／v2，不改既有單包receipt。沿用application-data工作、jobId、進度、取消與JobError；匯出成功顯示儲存位置，集合匯入顯示逐項結果並刷新清單，partial／cancelled不得只顯示成功。IPC失敗不把原始外部Error字串穿過前端。
- runtime IPC JSON一律camelCase，read回上述偏好加readError:string|null及diagnosticsStatus:"off"|"recording"|"writeFailed"；write失敗回{code:string,message:string}，不發布新值。diagnostics_append只接受固定schema；export回{path:string,bytes:number,skippedRecords:number}。network_probe_update回{operationId:string,results:[{destination:string,httpStatus:number|null,errorCode:string|null,durationMs:number}]}，實際錯誤鏈留在相同operationId紀錄。
- UI新增zh-TW與en字典；沿既有顯示語言選擇，沒有新增ja模式；CLI新文案維持英文、JSON鍵不翻譯。本次不改tw／ja／en政策解析與中文弱語言偵測。
- in scope：四份delta規格描述的裝置行為、Host資料交換及新增CLI。out of scope：改動SDD文件、雲端上傳診斷、Proxy憑證儲存、自動探測任意網址與Remote路由。除明確選擇的輸出包、裝置偏好、diagnostics及受控暫存外不寫檔；不建立或更動proposal.md、delta spec、.openspec.yaml、.speclink/ touched／snapshots等專案檔案。
- 驗收：固定色票矩陣、兩種設定模式同／異專案及取消切換；代理trap驗證三條網路工作路由與direct零代理請求；DNS／TLS／407／timeout與錯誤頁事件可見且秘密sentinel不進匯出；v1與v2離線round trip、預檢零提交、逐版本部分成功、輸出目的檔保留與取消。具體受影響面指令在tasks列出，原有golden與單包CLI回歸需保持。

## Risks / Trade-offs

- [既有CLI／golden回歸] → v1輸出不擴欄，只有新增export與v2使用新shape；以既有CLI測試與render_golden核對。
- [Windows／macOS／Linux系統Proxy來源不同] → 明列固定Proxy支援範圍與Linux環境變數；以注入快照驗證及平台CI測試，不假設GUI繼承shell環境，也不聲稱支援PAC。
- [Proxy要求認證或公司CA] → 保留proxy-auth／tls錯誤證據，設定頁說明尚不支援，維持TLS驗證；不預設此功能就能解決公司問題。
- [更新plugin隱去回應／TLS細節] → 保留unknown及null，提供使用者觸發的獨立連線測試，不以額外請求偽造原始更新結果。
- [集合匯入跨版本部分成功] → 全包預檢先於任何提交；磁碟／鎖／取消失敗後保留已成功項並輸出逐項結果，不承諾跨版本原子性。
- [資料包與診斷包大檔或敏感內容] → 限額、固定schema、雙重遮蔽與檔案白名單，原子發佈前確認容量，測試壓縮膨脹及秘密sentinel。
- [工作超過15項且跨UI／網路／資料I/O] → 保持單一討論對應單一變更，依已確認的整體桌面需求交付；tasks按邊界拆成可獨立驗證的小步驟，Proxy與診斷共用快照避免分案競爭同一接線。本次不宣告其他變更依賴。

## Migration Plan

缺偏好檔不立即落盤，首次明確儲存才建立；既有色票名稱及locatorKey不變，重啟直接呈現新色值。舊v1離線包照常匯入，v2可取內含v1包給舊版使用。回退版本時忽略新增裝置偏好與診斷目錄，不遷移標準資料current格式。正式規格在此變更封存時才合併，不在提案階段覆寫。

## Open Questions

無阻擋實作的產品決策。公司Proxy／憑證實況須待使用者在實作完成後重現並匯出診斷，屬外部證據收集，不列為本變更完成的必要人工關卡；若需PAC或企業認證另開有證據的變更。


## 實作驗證記錄（2026-10-19）

**同類位置與實際接線**

- 七色 token 與 projectTileClass 為主視窗、ProjectSettingsView 與 TrayPanel 共用；設定預覽固定傳選取態，實際專案列仍取真實 active 狀態。已移除該說明的舊 50% 混色敘述，主題測試核對固定深淺值。
- App 的點擊、數字鍵、Ctrl+Tab 與原生下一專案都經離開設定入口。store 加上切換意圖世代，晚回來的本機探測不能蓋過新選擇；從設定重連 Remote 成功後才啟用，失敗不提前寫入新 activeKey。
- 更新 JS adapter 唯一入口為 Rust updater_check／download／install，候選 Update 保留同一 ProxyPolicy；簽章驗證與安裝仍交 plugin。官方標準下載保留 try_proxy_from_env(false)，由共用 policy 明確注入路由及診斷。Remote 的 remote.rs 與 speclink-remote 保持原路由，因本次只管理更新與標準資料下載；獨立 CLI 不讀桌面偏好。
- DataRequest 與資料 IPC 實際位於 desktop-core/security.rs 與 Tauri lib.rs；security_commands.rs 是審查命令，未另造重複資料入口。CLI 與桌面共用 Host 的選取、版本 pin、匯出與逐版本提交。
- FS 共用同一 generation 讀取與指紋檢查，僅匯出時保留 raw bytes，避免一般讀取額外持有整包原始資料。單包沿用原 validator；集合包先全部預檢與私有 staging，再進 Host 提交。

**Audit discipline 的三種檢查角度**

1. 惡意輸入：手動 Proxy 拒絕帳密、額外路徑與非 HTTP 端點；僅選定 Proxy 可用私有 DNS，官方來源與 TLS 限制保留。診斷限制固定 schema、列舉 route、錯誤分類與來源程式腳本位置，匯出再次過濾秘密 sentinel。ZIP 路徑、SHA、授權、特殊檔、重複、遞迴與限額都有檢查，未任意解壓到使用者路徑；發布使用同目錄 tempfile，完成及 sync 後 persist_noclobber，核對 tempfile 3.27.0 在 macOS／Linux 的 NOREPLACE 與 Windows 不覆寫移動實作及錯誤路徑。
2. 直接沿用預設：Debug 關閉、缺檔不落盤、系統固定 Proxy 缺席時明示直接連線；壞偏好要求明確重設。佇列、輪替、單筆、診斷 ZIP、Probe 與資料包都有固定上限。已選來源讀取失敗會使整次匯出失敗，不會少包冒稱全部。
3. 誤用介面：ProxyMode／ResolvedRoute 與 ExportSelection 以列舉表示；CLI 互斥選項由 clap 拒絕，IPC 匯出選項組合亦驗證。集合結果明示 partial／failed／cancelled／notAttempted，CLI exit 0／1／130 與參數錯誤 2 已驗證，原 v1 JSON／人眼輸出保持。

安全檢查另以先失敗再修正的測試重現並修復：不支援的系統 Proxy 在 redirect 才被選中時不得 panic 或回落直連（reqwest 無法回傳 selector 錯誤時採封閉失敗路由並回報 proxy_unsupported）；路由驗證失敗必須留下診斷且 IPC 保留安全錯誤碼；主畫面無法掛載時，診斷出口仍須獨立可用；僅 local header 使用 ZIP64、沒有 ZIP64 footer 的包亦須拒絕。診斷 route 的未知值在反序列化邊界拒絕。

**實際驗證與限制**

- npm test -w @speclink/desktop：51 檔、1269 項通過。最後 ErrorBoundary 小幅整理後，另跑 diagnostics.test.tsx 四項通過。
- npm test -w @speclink/ui：80 檔、1330 項通過；保留本次 apply 開始前已存在的 Why 編號列表修改。
- npx tsc -p apps/desktop/tsconfig.json --noEmit：通過。
- cargo test -p speclink-core -p speclink-fs -p speclink-host -p speclink-desktop-core -p speclink-desktop -p speclink-cli：整體通過，包含既有 core golden 與單包 CLI 回歸。新增邊界後再跑 Host／desktop-core／CLI，分別 155+5、343、26+560 項通過；FS 最後為 27+1+20 項通過。Tauri 73+74 項測試通過，最後型別接線再以 cargo check -p speclink-desktop 通過。
- 本機 Proxy trap 驗證 CONNECT 與 407；HTTP 探測驗證 Debug 關閉不送請求、實際 2 MiB 限額與 timeout。三跳及 HTTPS 規則以邊界測試驗證，TLS 證據以實際錯誤型別分類測試驗證，不從字串推測。
- v1 CWE XML 與 v2 離線往返、來源／授權／Imported、同代 raw、32 包／512 MiB／1 GiB 限制、壞包零提交、第二版本提交失敗、完成一項後取消及後續 notAttempted、已存在目的檔與取消不發布都有測試。
- node --test scripts/*.test.mjs scripts/*/*.test.mjs：386 項通過，包含詞彙、連結與中英文件檢查。git diff --check 通過。
- 本次沒有下載官方標準測試資料，保留既有需明確提供官方 fixture 的 ignored 測試；沒有取得公司網路的實際紀錄，也沒有宣稱已在 Windows／Linux 原生設定上實測。公司 Proxy／CA 根因仍需實際重現。既有 trace.rs 的未使用 Path import 警告未擴大修正。
- apply payload 的 securityVerification.enabled=false，未另行執行獨立安全掃描、下載資料或啟動品質關卡。strict validate 通過；新能力名稱近似既有 desktop capability 的提示屬命名相近，實際範圍是本次新增的裝置診斷與受管網路，不合併至憑證／專案設定能力。
