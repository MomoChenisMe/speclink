## Context

來源是討論 `embedded-host-needs-5`（已結論）。它查證了一個內嵌宿主的實測回報，確認兩項不足並定案修法。現況如下。

- **入口先選 repo 才進路由。** `speclink-host` 的 HTTP 入口對專案底下的每個請求都先執行 repo 的選擇：專案有多個 repo 而請求沒有 `X-Speclink-Repo` 時回 409。專案底下的 `/whoami` 也在這個規則裡，但它回的 `repos` 是整個專案的清單，只為了算 ETag 才用到 repo 的範圍。
- **三個 CLI 動詞不帶 repo 問 `/whoami`。** `speclink link`、`speclink auth status`、`speclink init --store remote` 都以沒有 repo 的用戶端呼叫專案底下的 `/whoami`，拿回應的 `repos` 檢查使用者宣告的 repo。專案有多個 repo 時三者都收到 409。
- **Speclink Desktop 的連線狀態只有三種。** online、offline、needs-reauth 由 Rust 端的 `TokenManager` 以連線為單位判定並廣播。事件管理器的 worker 以（連線、專案、repo）為鍵輪詢 `/sync-state`，失敗時只是不發重新載入的通知。403 不屬於三種狀態的任何一種；500 以上的回應也不算離線（只認沒有狀態碼的連不上）。
- **示範用 Server 的事件串流只在訂閱當下檢查權限。** 串流開著之後，使用者被移出專案或被停用，串流不會關閉。
- **用戶端對 500 以上的回應一律丟掉錯誤信封。** 訊息固定是「server unavailable — check the connection url …」，伺服器回的原因不顯示。

限制：示範用 Server 的專案路由全部經過同一個入口，所以入口只有一份實作；`RemoteError` 在 `speclink-remote` 與 Speclink Desktop 共有 54 處以字面值建構；Speclink Desktop 的連線狀態機與事件管理器既有的落點在 `apps/desktop/src-tauri`。

## Goals / Non-Goals

**Goals:**

- 專案有多個 repo 時，`link`、`auth status`、`init --store remote` 可以用，而且 repo 拼錯或沒指名時列出可用的代號。
- 看板開著時失去存取權限，Speclink Desktop 在最長一個輪詢間隔（30 秒）內顯示原因、標明內容是舊的、停用寫入，恢復後自動復原。
- 連到有推送串流的示範用 Server 時，同樣的情況在一個心跳間隔（預設 15 秒）加上重連的時間內被偵測到。
- 伺服器有回原因時，CLI 與 Speclink Desktop 把原因顯示出來。

**Non-Goals:**

- 串流正常時 Speclink Desktop 也定期輪詢。每個開著的分頁每 30 秒多一個請求，還要改用戶端讀串流的方式；偵測交給提供串流的一方。
- CLI 在已知 repo 時帶上 `X-Speclink-Repo`。`--repo` 拼錯時入口會回 404 而不列代號，而且沒有修掉循環。
- 改變其他專案路由的 repo 規則。多個 repo 而沒有指名時仍然拒絕、不自動選。
- 給 `RemoteError` 加欄位。
- 調整 `init --store remote` 的順序（先初始化、後查驗）。
- 重開 app 後帳號仍被拒絕的畫面。既有的復原頁已經處理（沒有 session 的那條路）。
- 把 Rust 端寫死的中文狀態訊息改成跟隨介面語言。
- 宿主還沒確認的三個部分（成功換新 access token、掛在路徑前綴底下、內部 CA 簽的憑證）。

## Decisions

### D1 專案底下的身分查詢在選 repo 之前回答

入口處理專案請求時，在確認專案屬於這個人之後、選 repo 之前，先看路由是不是 `/whoami`：是的話直接以身分與專案的 repo 清單組出回應，不經 repo 的選擇，也不帶 ETag。請求帶的 `X-Speclink-Repo` 不參與這個回應。身分（沒有時 401）、API 版本標頭（不符時 409）、專案是否在範圍清單裡（不在時 404）三項檢查的順序與結果不變。

- 唯一實作落點：`speclink-host` 的入口。示範用 Server 的專案路由經同一個入口，不另寫一份。
- 路由表以型別標明這件事：每條路由的回答分成「針對一個 repo」與「針對專案」兩種，入口對後者在選 repo 之前回答。`/whoami` 是目前唯一的專案路由。入口不靠比對路徑字串來認它，也沒有只為了填表而存在的處理函式。
- 為什麼不保留 ETag：這個回應的內容來自範圍函式，不來自任何一個範圍的文件；沒有可以對應的版本號。CLI 與 Speclink Desktop 都不讀它。
- 評估過的替代：讓 repo 的選擇在路由是 `/whoami` 時退回第一個 repo。它會讓 ETag 取自一個使用者沒有選的範圍，而且把「不自動選」的規則開了一個例外。

### D2 多個 repo 而沒有指名的訊息寫出兩種指定方式

入口的 409 訊息改成：

```
project '<專案代號>' registers multiple repos — name one with the X-Speclink-Repo header, or set `remote.repo` in .speclink.yaml (candidates: <專案代號>/<repo>, …)
```

狀態碼（409）、`reason`（`refused`）與候選清單的寫法不變。文字只有一份，在 `speclink-host` 產生；示範用 Server 的事件串流等不經入口的路由共用同一個函式，所以一起改到。

- 為什麼不在 CLI 翻譯：這句文字由伺服器產生、CLI 原樣顯示（`refused` 的訊息一律轉述）。在 CLI 依訊息內容改寫等於比對伺服器的字面，舊伺服器與新伺服器的字面不同時會漏接。

### D3 CLI 的 repo 檢查集中在一個函式

`speclink-cli` 的 `remote_base` 模組提供一個檢查，輸入是 `/whoami` 的回應與使用者宣告的 repo（可以沒有）：

- 有宣告：repo 不在清單裡時失敗，訊息沿用既有的 `repo '<名稱>' is not registered in this project (available: …)`。
- 沒有宣告而專案有兩個以上 repo：失敗，訊息是

  ```
  this project registers multiple repos — run `speclink link <url> --repo <key>` with one of: erp, crm
  ```

  其中 `<url>` 是使用者給的專案網址。
- 沒有宣告而專案只有一個 repo 或沒有 repo：通過。

三個呼叫處共用它：`link` 在寫入 remote 區段之前呼叫，失敗就不寫入；`init --store remote` 在既有的位置（初始化完成之後）呼叫，失敗時以非 0 結束，初始化的產物留著，使用者照訊息執行 `link` 即可補上 repo；`auth status` 把失敗當成一行以 `! ` 開頭的提示印出來（既有對未註冊 repo 的做法），exit code 仍是 0。

- 這是對討論結論的一處偏離：結論寫 `init --store remote` 也「不寫入」。實際上 `init` 一直是先寫後查，repo 不在清單時也是如此；這次不改順序，因為改順序會動到 `init` 既有的輸出順序。

### D4 伺服器原因的組字與拆解同源

`speclink-remote` 把「伺服器的原因放進用戶端的句子」這件事集中在一處，組字與拆解用同一組常數：

- 403 的 `permission_denied`：既有的 `access denied — <原因>; ask a project admin`，不變。
- 500 以上：先嘗試讀錯誤信封。信封的 `reason` 是 `unavailable` 且 `message` 不是空字串時，訊息是 `server unavailable — <原因>`；其他情況維持既有的 `server unavailable — check the connection url (…)`。`RemoteError` 的 `reason` 與 `status` 欄位的值維持現況（500 以上時 `reason` 是沒有、`status` 是實際的狀態碼）。
- 新增一個公開函式：給一個 `RemoteError`，取回上面兩種句子裡伺服器的原因；不是這兩種句子時回「沒有」。Speclink Desktop 用它拿原文放進提示列。

- 為什麼不給 `RemoteError` 加欄位：54 處字面值都要改，而需要原文的只有 Speclink Desktop 的兩個地方。
- 為什麼不在 Speclink Desktop 自己切字串：句子的字面屬於 `speclink-remote`，由它提供拆解才不會在改字面時漏改。

### D5 憑證來源的文字不指名登入方式

`auth status` 第二行對鑰匙圈裡的登入憑證顯示 `system keychain (login session, shared with the desktop app)`。`--json` 的 `credentialSource` 值（`keychain_refresh`）不變。鑰匙圈的條目不記錄它來自瀏覽器登入還是裝置登入，所以文字不區分。

### D6 存取被拒由事件管理器判定並以分頁為鍵

事件管理器的 worker 是唯一判定的地方：

- 輪詢 `/sync-state` 的結果是狀態碼 403 時，worker 進入「被拒」，並以（鍵、伺服器的原因）通知；原因用 D4 的函式取出，取不到時是「沒有」。已經在「被拒」而原因相同時不重複通知。
- 輪詢成功時，若先前是「被拒」，worker 離開「被拒」，以（鍵、沒有）通知，並發一次重新載入的通知。
- 訂閱串流失敗不直接判定；它之後一定接著輪詢，由輪詢的結果判定。這也是 D10 的串流關閉會走到的路徑。

通知的出口與既有的重新載入通知並列：事件管理器建立時多收一個回呼。Tauri 的接線用它對前端發事件 `remote-access-state`，payload 是 `{ key, denied, message }`（`key` 是 locator key，`denied` 是布林，`message` 是字串或 null）。

每條訂閱的存取狀態（沒有被拒，或被拒與原因）存在事件管理器裡，以 locator key 可以查到，供 D7 的寫入守門問。它是唯一的一份：沒有另外一份要保持同步的登記。最後一個 session 退訂時，訂閱連同它的狀態一起消失；沒有訂閱的 key 視同沒有被拒。

- 通知在放開狀態的鎖之後才送出，回呼做什麼（發事件）不會擋住同時進來的退訂。退訂之後 worker 不再改狀態。
- 加入既有訂閱的 session（同一個分頁重新訂閱，而新的訂閱比舊的退訂先到）會再收到一次目前的「被拒」：worker 在原因相同時不重複通知，不補這一次的話畫面會以為沒有被拒。
- 前端的訂閱先掛好三個事件的 listener，才送出開始監看的命令，所以上面那一則通知不會早於 listener。

- 為什麼不放進 `TokenManager`：它以連線為單位，同一台伺服器上使用者仍有權限的其他專案分頁會被連帶標成被拒。
- 為什麼只認輪詢的 403：其他讀取路由的 403 也會出現在只是角色不足的人身上（部分讀取需要寫入的角色）；`/sync-state` 對任何有讀取權限的人都成功，它的 403 才代表「這個範圍讀不到」。
- 落點：沿用 `apps/desktop/src-tauri` 既有的事件管理器與連線層，因為狀態機本來就在那裡；沒有新增 Tauri command。

### D7 寫入守門多查一次被拒登記

Rust 端每個寫入命令在既有的「連線是否可寫」檢查旁，再以（連線、專案、repo）問事件管理器這個分頁的存取狀態（D6）；被拒就立即回錯誤，`reason` 是 `access_denied`，訊息是 Server 的原因（沒有原因時是固定的一句中文）。不排隊、不重放。流程設定的寫入是「先讀全文、再寫回」，它在讀取之前就檢查，所以被拒時一個請求都不送。

前端對應的做法是把寫入能力關掉：session 多一個 `accessDenied` 欄位（`{ message }` 或沒有），「可不可以寫」改由一個函式決定——連線是 online 而且沒有 `accessDenied`。既有的離線遮罩（把 11 個寫入能力設成 false）與流程政策的寫入開關都改用這個函式，不另寫一份。

### D8 單一狀態裁決加上 access-denied

`session.ts` 的 `remoteWorkspaceStatus` 是主視窗、復原頁與系統匣共用的狀態裁決。它的結果多一個 `access-denied`：session 存在、連線是 online、而且 `accessDenied` 有值。優先序是 needs-reauth、offline、access-denied、ready。

各處的呈現都從這個結果取，並沿用既有的元件與文案：

- 看板上方的提示列（`App.tsx` 既有的提示列）：`SEMANTIC_SURFACE.danger` 與 `SEMANTIC_TONE.danger`、`AlertTriangle` 圖示；標題用既有的 `remote.recovery.access-deniedTitle`；說明是伺服器的原因（沒有時用既有的 `remote.recovery.access-deniedDesc`）接上新文案 `remote.deniedHint`；保留 stale 標記；沒有按鈕。
- 專案圖示列（`ProjectRail.tsx`）：狀態點用錯誤的紅色，提示文字用既有的 `remote.recovery.access-deniedShort`。
- 系統匣面板（`TrayPanel.tsx`）：分頁下方的狀態文字用既有的 `tray.recovery.accessDenied` 與紅色；面板上方的提示卡用紅色與 `AlertTriangle`。
- 系統匣的原生選單（`tray.ts`）：專案項目的標籤是「專案名 — 沒有存取權限」，和離線時的寫法相同；項目仍然可以點選切換。
- 寫入停用的說明：新文案 `remote.writeDenied`。

新文案兩條，zh-TW 與 en 各一：`remote.deniedHint`（「目前顯示最後成功載入的內容；恢復存取後會自動更新。」／「Showing the last successfully loaded content; it will update automatically when access returns.」）、`remote.writeDenied`（「目前沒有存取權限——恢復存取後才能寫入」／「You have no access right now — writing returns when access does」）。

沿用的 `@speclink/ui` 項目：`SEMANTIC_SURFACE`、`SEMANTIC_TONE`、`cn`、`useI18n`。沒有新增共用元件：提示列與提示卡都是既有的，只多一種色調。

### D9 500 以上計入離線而且離線訊息帶伺服器的原因

`TokenManager` 判定「這次失敗算不算離線」的條件從「連不上」放寬為「連不上，或回應的狀態碼是 500 以上」。門檻（連續 3 次）與回到 online 的條件（任何一次成功）不變。

進入 offline 時廣播的訊息：最後一次失敗帶有伺服器的原因（D4 的函式）時是「<原因>——顯示最後成功載入的內容」，否則是既有的固定句子。

回到 online 的條件多一種：伺服器以低於 500 的狀態碼回應（403、404、409 等）。它和成功的請求一樣把連續失敗歸零，連線是 offline 時回到 online 並廣播；回應本身的錯誤照常回給呼叫端。401 不在這裡處理：它先走換發與重試，仍是 401 就進入 needs-reauth。

- 為什麼需要：分頁被拒時 worker 只輪詢，而輪詢只會得到 403。這期間伺服器若斷線又恢復，就沒有任何「成功」的請求能把連線帶回 online，畫面會一直停在離線而不是存取被拒（D8 的優先序是 offline 先於 access-denied）。
- 為什麼不只認輪詢的 403：有狀態碼而低於 500，就表示伺服器收到請求並做了裁決；這和「連不上」「不能服務」是相反的事實，對哪一種回應都成立。

### D10 示範用 Server 在心跳的間隔重新檢查權限

示範用 Server 的事件串流在等待事件的同時，每隔一個心跳間隔（設定檔的 `events.heartbeat_secs`，預設 15 秒）查一次身分儲存：訂閱的使用者是否仍是可用的帳號、是否仍是這個專案的成員。任一個答案是「否」就結束串流。身分儲存查詢失敗時不結束串流（暫時的儲存問題不該讓每條串流都斷線重連）。

不檢查 token 的有效期限：token 到期由既有的換新流程處理，串流不因此中斷。

`events.heartbeat_secs` 設成 0 時（設定載入只拒絕 45 以上），沒有間隔可以依循，計時器也不接受 0；這時權限確認改用預設的心跳間隔（15 秒）。

- 為什麼掛在心跳的間隔：不用新增設定，而且用戶端本來就以心跳判斷串流是否存活；偵測的延遲因此有明確的上限。

### D11 文件隨行為更新

- `docs/sdk-node` 中英兩份：`auth-methods` 回應的 `browser` 一定帶 `scopes`（沒有宣告時是空陣列）；用 `pushTransports` 宣告自己的串流的宿主，要在使用者失去權限時關閉串流；入口對專案底下的 `/whoami` 不要求 repo。
- `crates/adapters/speclink-node/index.d.ts`：`authMethods` 的 `browser.scopes` 的註解補上同一句。
- `docs/verb-contract` 中英兩份：`X-Speclink-Repo` 那一列補上 `/whoami` 的例外。
- `docs/remote-getting-started` 中英兩份：多個 repo 的專案怎麼連（`link --repo`）、Speclink Desktop 的「沒有存取權限」提示。
- `docs/product-status` 中英兩份：照流程設定的規則更新已知缺口，並寫明宿主要升級 `@speclink/engine`、CLI、Speclink Desktop 三樣。

## Implementation Contract

**行為**

- 入口：`GET /api/speclink/v1/projects/<專案>/whoami` 對專案在範圍清單裡的身分一律回 200，本文是 `{ user: { name, handle }, repos: [{ name, gitUrl }] }`（形狀不變），回應沒有 ETag 標頭。專案有幾個 repo、請求有沒有 `X-Speclink-Repo`、標頭的值是不是已註冊的 repo，都不影響結果。沒有身分是 401；API 版本標頭不符是 409；專案不在清單裡是 404。
- 入口：其他專案路由在多個 repo 而沒有指名時回 409、`reason` 是 `refused`，訊息是 D2 的字面。
- `speclink link <url> [--repo <key>]`（已有憑證）：依 D3 檢查。失敗時 exit code 非 0、訊息在 stderr、`.speclink.yaml` 不變。檢查通過的話，輸出與變更前相同。沒有憑證時行為不變。
- `speclink init --store remote --url <url> [--repo <key>]`：初始化的輸出不變；之後依 D3 檢查，失敗時 exit code 非 0、訊息在 stderr。
- `speclink auth status`：第一行不變；第二行是 `  Credential from: <來源>`，鑰匙圈的登入憑證顯示 D5 的文字；有設定 repo 時照舊印 repo 的檢查結果；沒有設定 repo 而專案有多個 repo 時印一行 `! ` 開頭的 D3 訊息；exit code 0。`--json` 的 payload 不變：`{ user: { name: string, handle: string }, credentialSource: string }`。
- 用戶端的錯誤訊息：依 D4。
- Speclink Desktop：依 D6 到 D9。事件 `remote-access-state` 的 payload 是 `{ key: string, denied: boolean, message: string | null }`。連線是 offline 時，伺服器以低於 500 的狀態碼（401 除外）回應任何一個請求，連線就廣播 online。
- 示範用 Server：依 D10。

**失敗情況**

- 伺服器的 403 沒有可讀的信封或訊息是空的：提示列用既有的說明文案；被拒的判定不受影響（只看狀態碼）。
- 輪詢在「被拒」期間改成連不上或 500 以上：worker 維持「被拒」；連線層另外依 D9 累計失敗，達到門檻時連線進入 offline，畫面依 D8 的優先序顯示離線。
- 身分儲存在 D10 的檢查中失敗：串流繼續，下一個間隔再查。
- 舊版的伺服器（0.14.0 以前）遇到新版的 CLI：`link` 等三個動詞在多個 repo 的專案仍然收到舊的 409。這是已知的限制，文件寫明要升級伺服端。

**驗收**

- `cargo test -p speclink-host`：專案 `/whoami` 在兩個 repo 而沒有標頭、標頭指名未註冊的 repo 兩種情況都回 200 並列出兩個 repo、回應沒有 ETag；另一條專案路由在同樣的請求回 409 而且訊息含 `X-Speclink-Repo` 與 `remote.repo`。
- `cargo test -p speclink-server --test it`：經示範用 Server 的同一組情況；事件串流在成員資格被移除後的一個心跳間隔內結束，成員仍在時不結束。
- `cargo test -p speclink-cli --test it remote_connect`：D3 的三種結果，各對 `link`、`init --store remote`、`auth status`。
- `cargo test -p speclink-remote --test it client_errors`：D4 的兩種句子與拆解函式；`cargo test -p speclink-remote` 的單元測試涵蓋 D5 的文字。
- `cargo test -p speclink-desktop --test it`：worker 在輪詢 403 時通知被拒、成功時通知解除並重新載入、原因相同時不重複通知；500 以上連續 3 次進入 offline 而且訊息帶原因；分頁被拒時寫入命令立即被拒；offline 之後伺服器回 403 即回 online，低於 500 的回應把連續失敗歸零。
- `npm test -w apps/desktop`：`remoteWorkspaceStatus` 的優先序；提示列、專案圖示列、系統匣面板與原生選單在 access-denied 下的呈現；寫入能力被關掉、解除後復原。

**範圍邊界**

- 範圍內：上面列出的入口、CLI、`speclink-remote`、Speclink Desktop、示範用 Server 與文件。
- 範圍外：Non-Goals 列的各項；`apps/server-web`；`packages/ui` 的元件；儲存接口與引擎命令；協定的 payload 形狀。

## Risks / Trade-offs

- [既有測試比對 409 的舊字面與 `/whoami` 的 ETag] → 入口的測試（`http_door.rs`）、示範用 Server 的整合測試與 CLI 的 `remote_connect.rs` 在同一批更新；`remote_verb_parity.rs` 不涉及這三個動詞，執行確認仍通過。
- [`auth status` 的人眼輸出改了一行] → 這是刻意的變更，記在提案的相容性影響；`--json` 不變，由既有的 payload 測試守住。
- [第三方用戶端讀專案 `/whoami` 的 ETag] → 已知的用戶端都不讀；文件寫明這個回應沒有 ETag。
- [500 以上計入離線後，一個專案的 503 讓同一條連線的其他分頁也顯示離線] → 連線層本來就以連線為單位；任何一個分頁的下一次成功就會回到 online。門檻維持 3 次，單次的 500 不會觸發。
- [心跳間隔的權限檢查增加身分儲存的查詢] → 每條串流每個間隔一次（預設 15 秒）；查詢是兩次以主鍵的讀取。
- [事件串流的測試依賴時間] → 測試把心跳設成 50 毫秒（既有測試的做法），並等待串流結束而不是等待固定的時間；Windows 上連線被拒的逾時較長，斷言用「在期限內結束」。
- [`init --store remote` 在多個 repo 而沒帶 `--repo` 時留下沒有 repo 的 remote 區段] → 訊息直接給出補上的指令；之後的動詞收到 D2 的 409，文字同樣指出 `remote.repo`。
- [宿主只升級 CLI 與 Speclink Desktop 而沒有升級引擎] → F1 不會修好。文件與 `docs/product-status` 寫明三樣都要升級。
- [跨平台] → 這次沒有路徑、換行或 git 行為的改動；Speclink Desktop 的提示列、狀態點與面板在三個平台共用同一份前端；系統匣面板只在 macOS 與 Windows 有，Linux 走原生選單，兩條路徑都在 D8 涵蓋。

**銳邊稽核：CLI 的 repo 檢查（任務 3.3）**

對 `remote_base` 的 `check_repo` 與三個呼叫處，照稽核清單的三個角色逐項看過。沒有需要改程式的發現；邊界的行為由 `remote_base` 的單元測試釘住。

- [沒有 `--repo`] → 專案沒有 repo 或只有一個時放行，兩個以上時失敗並列出代號。伺服器端同樣不代選，所以放行的兩種情況不會連到使用者沒選的 repo。
- [`repos` 是空清單] → 沒有宣告時放行（之後的動詞由伺服器回「專案沒有註冊 repo」）；有宣告時失敗，訊息的 `(available: )` 是空的。這是既有的字面，不改：專案沒有 repo 是管理者要處理的設定問題，訊息已經指出 repo 不在專案裡。
- [`--repo ""`、名稱前後有空白、大小寫不同] → 一律逐字比對，不相等就失敗；訊息把使用者給的名稱放在引號裡，空白看得見，旁邊列出已註冊的名稱。不做去空白或忽略大小寫：伺服器的比對是逐字的，CLI 放寬只會把錯誤延到下一個動詞。
- [參數能不能對調] → `check_repo` 的網址是 `&str`、repo 是 `Option<&str>`，型別不同，對調會編譯失敗；回傳 `Result`，呼叫處不處理會有編譯警告。
- [檢查能不能被關掉] → 沒有旗標或設定可以略過。沒有憑證時 `link` 與 `init` 本來就不查（延到第一個動詞），這是既有的設計。
- [訊息裡的外來文字] → 代號清單來自伺服器的 `/whoami`，專案網址來自使用者自己的參數或 `.speclink.yaml`，兩者都原樣印出、沒有跳脫控制字元。不修：使用者已經把憑證交給這台伺服器，既有的 `(available: …)` 訊息也是同樣的做法；`auth status --json` 不印這些文字。
- [`init --store remote` 失敗後留下沒有 repo 的 remote 區段] → 已記在上面；訊息給出補上的指令。

**同類位置排查（任務 7.1）**

四組搜尋的命中與處理方式。除了標明「新命中」的兩處，其餘都在提案時已知。

- [（一）`RemoteClient::new(` 於 `crates/adapters/speclink-cli/src`，共 5 處] → repo 參數為 `None` 的是 `remote_base.rs` 的 `validate_or_defer`、`verbs/connection.rs` 的 `cmd_link` 與 `auth_status` 三處，都問專案底下的 `/whoami`，由 1.2 修好、呼叫方式不改。另外兩處在 `remote_base.rs` 的 `connect`，帶的是設定裡的 repo 或握手裁定的 repo，問的是握手與動詞，不在這次的範圍。
- [（二）`Client::new(` 於 `apps/desktop/src-tauri/src`，正式程式 4 處] → repo 參數為 `None` 的只有 `remote.rs` 的 `list_scopes`，問 API 根的範圍清單，不是專案路由，不用改。`open_workspace` 的握手在目標沒有寫 repo 時也不帶 repo（新命中）：它問的是握手，多個 repo 時照規則回 409，訊息由 D2 改成新的字面；新增專案的對話框一律帶 repo，所以這條路只在手寫目標時走到，不用改。其餘兩處都帶 repo。
- [（三）`choose_repo(` 於 `crates/host`，共 2 處] → 入口（`speclink-host` 的 `http/mod.rs`）由 D1 在它之前回答 `/whoami`；示範用 Server 的 `identity/auth.rs` 只服務事件串流，不服務 `/whoami`，訊息跟著 D2 改變。
- [（四）500 以上的判斷於 `crates/protocol/speclink-remote/src` 與 `apps/desktop/src-tauri/src`] → `speclink-remote` 的 `translate_protocol_error` 由 2.2 改成讀信封的原因；Speclink Desktop 的 `counts_toward_offline`（原名 `is_transport_failure`）由 4.2 把 500 以上計入。沒有其他地方丟掉 500 以上的信封：`device.rs`、`events.rs`、`client.rs` 都經 `translate_protocol_error`。
- [（四）新命中：換發 access token 時伺服器回 500 以上] → 換發的失敗以沒有狀態碼的訊息回到 `TokenManager`（`RefreshFailure::Unavailable` 只帶文字），所以不計入離線的連續失敗。不修：記憶體裡有 bearer 時不會換發，伺服器全面回 500 以上的情況由一般請求計入；記憶體裡沒有 bearer 的情況是 app 剛啟動，這時握手失敗、分頁走既有的復原頁（歸類為無法連線）。要計入就得讓換發的失敗帶狀態碼，會動到 `speclink-remote` 與 CLI 共用的型別，超出這次的範圍。
- [`oauth.rs` 對登入服務的 500 以上] → 對象是外部的登入服務，不是 Speclink 的伺服器，訊息另有一套，不在這次的範圍。

**實作時定下的細節**

- [被拒期間 worker 只輪詢、不訂閱] → D6 寫「只認輪詢」。實作上被拒期間略過重新訂閱：重新訂閱若先成功，worker 會停在串流上不再輪詢，「被拒」就沒有解除的機會。解除後的下一輪才再訂閱，續傳位點照舊。
- [背景分頁的存取狀態是上次作用中時的狀態] → 只有作用中的分頁有 worker（既有的設計：切換分頁就換訂閱）。分頁切到背景時，前端保留它最後的狀態，所以專案圖示列與系統匣仍標示它；Rust 端的存取狀態隨訂閱在退訂時消失（背景分頁不會送出寫入）。分頁回到作用中時前端先清掉舊狀態、由新的 worker 重新判定：仍被拒的話，第一輪輪詢就會再標示（中間有不到一次請求往返的空檔）。
- [訂閱收束不發存取狀態事件] → 沒有 worker 的分頁，狀態是「不知道」而不是「解除」。退訂之後 worker 不再改狀態；已經在送出途中的那一則通知仍可能到達前端，它只會讓背景分頁停在上次的狀態，分頁回到作用中時會被清掉。
- [寫入守門掛在命令共用的骨架] → `lib.rs` 的 `with_remote` 與 `with_remote_settings` 經同一個函式建立 `RemoteWorkspace`，把「問事件管理器這個分頁的存取狀態」接上去，全部資料面命令都經這兩個骨架；`RemoteWorkspace` 的每個寫入、兩個批次任務寫入與三個流程設定寫入共用同一個前置檢查。把本機資料遷移到伺服器的命令不經分頁（目標分頁還不存在），不問存取狀態。
- [`speclink-server` 的 `tokio` 多開 `time` 功能] → D10 的間隔計時直接用到它。先前是靠其他相依套件間接開啟。
- [專案圖示列與系統匣共用狀態文字與狀態裁決] → 圖示列對有 session 或復原狀態的分頁呼叫 `remoteWorkspaceStatus`（和系統匣同一個函式），不再自己排一次優先序；原生選單與面板的狀態文字出自同一個函式。這是品質關卡第 1 輪的修正。
