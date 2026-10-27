## Context

desktop 與 CLI 以 library 共用同一套引擎 crates，但驗證層分岔：desktop 在 app 層（`apps/desktop/src-tauri/src/credentials.rs` 的 CredentialStore trait＋keyring 實作、`apps/desktop/src-tauri/src/connections.rs` 的 refresh 換發編排）持有 Keychain 憑證；CLI 只透過 `crates/speclink-remote/src/auth.rs` 解析 SPECLINK_TOKEN 與 credentials.yaml。裝置授權的 HTTP 原語（initiate／poll／refresh／whoami／revoke，RefreshResponse 含 expires_in）已在 `crates/speclink-remote/src/device.rs`，desktop 的編排刻意不依賴 Tauri 型別——下沉條件成熟。

server 端 refresh credential 為 single-use＋reuse 偵測：重用已換發或已撤銷的值即整族撤銷。任何共享設計都必須序列化換發，否則兩行程併發即互相登出。

## Goals / Non-Goals

**Goals:**

- desktop 登入後，同機 CLI 免登入直接可用（反向亦然：CLI 裝置授權登入後 desktop 可用）
- CLI 獲得裝置授權登入（互動預設），PAT 路徑（--pat／--token-stdin／credentials.yaml）零變化
- 憑證儲存與換發編排單一實作，落在 speclink-remote，desktop 與 CLI 共用
- 純 CLI／headless／CI 使用者零退化

**Non-Goals:**

- headless 環境的裝置授權憑證持久化（refresh 不落明文檔）
- PAT 改寫入 Keychain；server 端任何變更；desktop UI 變更
- macOS Keychain 首次存取系統提示的繞過（OS 行為，僅文件說明）

## Decisions

### CredentialStore 與換發編排下沉至 speclink-remote

CredentialStore trait（get／set／delete，鍵＝origin＋CredentialKind）、keyring 生產實作與 in-memory 測試實作自 desktop 移入新模組 `crates/speclink-remote/src/credentials.rs`；換發編排移入新模組 `crates/speclink-remote/src/refresh.rs`。desktop 的 connections.rs 改為呼叫下沉後的函式，credentials.rs 刪除。CredentialKind 增列 Bearer（見 bearer 快取決策）。keyring 相依自 desktop 的 Cargo.toml 移至 speclink-remote。

替代方案：落在 speclink-host——否決：host 是引擎的應用層邊界，不該認識 OS Keychain 這種 client 端儲存；speclink-remote 已是「典型 client 能力」的家（device 原語在此），且 CLI 與 desktop 都已依賴它。另一替代：獨立新 crate——否決：單一消費場景不值得多一個 workspace 成員，屬過度設計。

### 檔案鎖序列化跨行程換發

換發全程（讀 refresh → 呼叫換發端點 → 回寫新 refresh）持使用者設定目錄下的獨立鎖檔（與 credentials.yaml 分離，避免與 PAT 讀寫互鎖）。以跨平台檔案鎖 crate（fd-lock 或 fs2）實作，Windows／macOS／Linux 行為一致。Keychain 是單機儲存，因此單機檔案鎖即足以覆蓋所有競爭者（desktop、多個並行 CLI 行程）。鎖內二次讀取：取得鎖後重讀 refresh 條目——若已被先行者換新，直接用新值換 bearer 或複用其 bearer 快取，不得拿舊值打換發端點。

替代方案：樂觀重試（撞 reuse 偵測後重登入）——否決：reuse 偵測的後果是整族撤銷，重試等於必然登出，不可接受。每客戶端一族——已於討論否決：沒有消除重複登入。

### Bearer 快取入 Keychain 避免逐指令 rotation

CLI 行程短命、無記憶體可跨呼叫留存 bearer；若每個 speclink 動詞都換發，rotation 頻率暴增，且「server 已換發、本機回寫前崩潰」的掉族視窗被放大。故短效 access token 連同絕對到期時刻以 JSON 存入 Keychain 的 Bearer 條目（serde 結構：token 與 expiresAt 兩欄位，內部儲存格式、非 wire 契約；條目缺席或無法解析一律視為無快取，向後相容——舊版寫入的 Keychain 不含 Bearer 條目，行為等同快取未命中）。解析順序：Bearer 未到期即直接使用（不取鎖）；到期、缺席或被 server 以 401 拒絕才走檔案鎖換發，換發成功即回寫新 bearer 快取。401 時強制換發重試一次，再失敗即報 needs-reauth。desktop 的記憶體內 bearer 快取保留為快路徑，換發成功時同步回寫 Keychain 條目。

替代方案：每次 CLI 呼叫都換發——否決：rotation 網路往返拖慢每個動詞、掉族視窗放大、並行 CLI 行程鎖競爭加劇。快取落明文檔——否決：Keychain 已可用時無理由降級儲存。

### CLI 憑證解析階梯與靜默下探

解析順序固定四層：SPECLINK_TOKEN 環境變數 → Keychain refresh（經 bearer 快取與換發）→ Keychain PAT → credentials.yaml PAT。「靜默下探」只發生在解析階段：某層不可用（平台無 keyring 服務、Keychain 存取被拒、條目不存在）即續探下一層；四層皆空才報未登入（維持現有錯誤訊息與非 0 exit code，指引 speclink auth login）。一旦選定憑證，該次動詞內被 server 拒絕時 SHALL NOT 改用其他層（維持正典 remote-auth「不靜默改用其他憑證來源」的契約）——refresh 族已撤銷時清除該 refresh 與 bearer 條目並報 needs-reauth，下一次執行才自然解析至剩餘層。既有兩層（env → credentials.yaml）相對順序不變，確保現役 CI 與腳本零受影響。

替代方案：keyring 錯誤時硬性報錯——否決：Linux headless（無 Secret Service）與 macOS 使用者按 Deny 都會讓既有 PAT 使用者無法工作，違反零退化目標。

### auth login 雙軌與 TTY 判定

speclink auth login 的行為矩陣：

- 互動 TTY＋無旗標：裝置授權——initiate（server 不支援時回報並指引 --pat）→ 能開瀏覽器則開啟核准頁、一律印 verification URL 與 user code（供他機核准）→ 依宣告間隔輪詢 → approved 後 refresh 與 bearer 寫入 Keychain，顯示身分資訊。denied／expired 以非 0 exit code 結束並區分訊息。
- --pat：互動貼 PAT，驗證後寫入 credentials.yaml（現行為改掛旗標，儲存位置不變）。
- --token-stdin：行為、儲存位置、exit code 完全不變。
- 非互動且無旗標：非 0 exit code，指引 --token-stdin。
- Keychain 不可用（無 keyring 服務）時選裝置授權：非 0 exit code，說明 refresh 不落明文檔並指引 --pat／SPECLINK_TOKEN。

--pat 與 --token-stdin 互斥（clap conflicts_with）。裝置授權成功後印出的身分資訊與 auth status 一致（whoami）。

替代方案：預設維持貼 PAT、裝置授權掛旗標——否決：新使用者最佳路徑應免手建 token；互動情境的破壞性變更已於提案「相容性影響」記載，腳本路徑（--token-stdin）不受影響。

### auth status 顯示憑證來源層

人眼輸出增列來源層描述；--json 新增 credentialSource 欄位（camelCase 欄位名），值域：env、keychain_refresh、keychain_pat、credentials_file。既有欄位與語意不動。未登入路徑不變。

替代方案：不標示來源——否決：四層階梯下「用到哪個憑證」是除錯共享問題的第一個問句，缺它就得靠猜。

### auth logout 的撤銷與本機清除語意

speclink auth logout：Keychain 有 refresh 時呼叫 revoke 端點撤銷整個 credential family（共用一族的對稱後果：desktop 同時登出），之後清除該 origin 的本機憑證——Keychain 的 Refresh／Bearer／Pat 條目與 credentials.yaml 條目。server 端 PAT 不撤銷（PAT 由使用者於 server 側自管，可能供其他機器使用）。四處皆無憑證時報「未登入」、非 0 exit code。revoke 端點網路失敗時仍清除本機憑證並警告 server 側 family 未撤銷。

替代方案：只清本機不撤銷 family——否決：refresh 已換發副本仍存活於 server，登出語意不完整；反向（撤銷 server 端 PAT）——否決：越權清除使用者自管的長效憑證。

## Implementation Contract

**行為（使用者可觀測）**：

1. desktop 已登入某 origin 的前提下，同機執行任一 remote 動詞（如 speclink list --json）：不出現任何登入提示、指令成功。首次執行時 macOS 可能跳 Keychain 系統提示（OS 行為）。
2. 純 CLI：互動執行 speclink auth login 完成裝置授權後，desktop 對同 origin 免登入。
3. SPECLINK_TOKEN 存在時永遠勝出（現有契約不變）；只有 credentials.yaml PAT 的既有使用者行為完全不變。
4. 兩行程（desktop＋CLI，或多個 CLI）併發觸發換發不得造成任何一方被登出。
5. speclink auth logout 後，同機 desktop 與 CLI 對該 origin 一致回到未登入狀態。

**介面／資料形狀**：

- speclink-remote 公開：CredentialStore trait 與 CredentialKind（Refresh／Pat／Bearer）、keyring 與 in-memory 實作、bearer 取得編排函式（含檔案鎖換發），與既有 resolve_token 並存整合為單一解析入口。函式與模組命名依 snake_case 慣例，實作時定案。
- CLI：auth login 新增 --pat 旗標（與 --token-stdin 互斥）；auth logout 新子指令；auth status 的 --json 新增 credentialSource 欄位。人眼輸出遵循現有 ANSI／--no-color 慣例。
- Keychain 條目：service 與 account（kind:origin）字串與 desktop 現值逐字相同；Bearer 條目值為 JSON（token、expiresAt）。
- 鎖檔：使用者設定目錄下獨立檔案，僅供換發序列化，不承載資料。

**失敗模式**：

- keyring 層不可用（無服務、存取被拒）：靜默下探，不落錯誤訊息（auth status 除外，可標示層不可用）。
- refresh 換發回 reuse／revoked：清除本機 refresh 與 bearer 條目，報 needs-reauth 與 speclink auth login 指引，非 0 exit code。
- 換發網路暫時性失敗：原樣中繼錯誤，不清除憑證。
- 鎖等待逾時（有時間上限，非無限阻塞）：非 0 exit code，訊息指出疑似有他行程長時間持鎖。

**驗收判準**：

- cargo test -p speclink-remote：in-memory store 上覆蓋階梯四層命中順序、鎖內二次讀取、bearer 到期與 401 強制換發、logout 清除範圍——全數新增單元測試通過。
- CLI 整合測試（crates/speclink-cli 既有測試佈局）：--token-stdin 與 credentials.yaml 路徑既有測試不改仍綠；新增非互動無旗標報錯、auth status credentialSource、auth logout 未登入路徑。測試不得依賴真實 OS keyring（以 in-memory 注入或環境隔離）。
- 換發併發：兩執行緒／行程同時請求 bearer，斷言恰一次換發、零 reuse 錯誤（in-memory store＋stub server 層級）。
- cargo test -p speclink-desktop 既有連線／登入測試仍綠（編排改呼叫下沉函式後）。
- 人眼輸出回歸：render golden 與 CLI 輸出測試同批更新，diff 僅含本提案記載的變更（auth login 預設流程、auth status 新列）。

**範圍邊界**：

- In scope：speclink-remote 新模組與整合、CLI auth 子指令面、desktop 改用下沉層、上述測試。
- Out of scope：server 端點、desktop UI 與兩段式 device login UX、credentials.yaml 格式、技能文件之外的說明文件重寫。

## Risks / Trade-offs

- [Linux headless／CI 無 Secret Service，keyring 呼叫失敗] → 階梯靜默下探設計即為此而生；所有自動化測試以 in-memory store 注入，不觸真實 keyring；CI 現有 SPECLINK_TOKEN 路徑不經 keyring。
- [macOS Keychain ACL：CLI 首次讀 desktop 建立的條目跳系統提示，使用者按 Deny] → Deny 視為該層不可用、靜默下探至 PAT；文件說明按「永遠允許」的預期路徑。
- [rotation 回寫前行程崩潰＝掉族] → 既存風險不因本變更擴大；bearer 快取把換發頻率從逐指令降到到期才換，縮小視窗；發生時 needs-reauth 指引重登入。
- [auth login 互動預設變更破壞使用者肌肉記憶] → 破壞面限互動 TTY；--token-stdin 不動；提案相容性影響已記載，錯誤與提示文案指引 --pat。
- [人眼輸出回歸對照] → golden 與 CLI 測試同批更新為驗收判準之一，dirty 樹不得再生 golden（專案既知地雷）。
- [Windows 檔案鎖語意差異（強制鎖）] → 鎖檔獨立、不與資料檔共用 handle；採用跨平台鎖 crate 而非手寫 flock。

## Migration Plan

1. Keychain 條目鍵逐字不變：既有 desktop 登入在升級後直接被 CLI 讀到，無資料遷移。
2. credentials.yaml 不改格式：舊 CLI 與新 CLI 可互換讀寫。
3. 相依搬移：keyring 自 apps/desktop/src-tauri/Cargo.toml 移至 crates/speclink-remote；desktop 經 re-export 或直接依賴均可，以最小 diff 為準。
4. 回滾＝revert：無 schema 遷移、無單向資料轉換；新版寫入的 Bearer 條目對舊版 desktop 是未知 kind，不被讀取、無害。

## Open Questions

（無——討論已裁定共用一族＋檔案鎖；deferred 項目（ACL 提示文案細節、PAT 入 Keychain）明確列於 Non-Goals。）
