## Context

官方 `speclink-server`（crate speclink-server）的專案範圍路由，以 `Binding` extractor 解析呼叫者身分，`Binding.editor` 表示 membership 角色是否為 editor。角色檢查目前散落在 12 個處理函式（以 `binding.editor` 或同值的 `binding.policy_write` 判斷），其餘 17 個寫入處理函式沒有檢查。來源討論 `docs-cleanup-findings` 已裁定：reader 是檢視者，所有寫入一律擋下、不設例外，檢查集中在一個位置。

內容格式錯誤的部分：speclink-core 命令層以 `classify` 把 anyhow 錯誤分成五個錯誤碼；artifact 內容檢查（`new artifact` 流程的內容檢查函式）與品質關卡的輪內容解析用 `bail!` 產生一般錯誤，落入 `error`。server 的單一對照點（`From<CommandError> for ApiError`）把 `error` 轉成 500 `internal`。

桌面 app 的遠端 capability 由 Tauri 殼的 `RemoteCapabilities::from_binding` 從 handshake 推導：`deleteChange`、`moveTask`、`claim`、`reorderCard`、`setDepends`、`policyWrite` 依角色；`setTaskDone`、`setAllTasks`、`archive`、`promoteDiscussion`、`archiveDiscussion` 寫死為 true。前端在這些 capability 為 false 時顯示 `remote.writeUnavailable`（「目前為 stale 唯讀狀態」），這段文字原本只為離線遮罩而寫。

## Goals / Non-Goals

**Goals:**

- server 對專案範圍的每一個寫入請求檢查 editor 角色，檢查只有一個落點，之後新增的寫入端點預設受保護。
- 內容格式檢查失敗在每個入口都歸類為 `invalid_argv`；server 回 400 `invalid_argument`。
- 桌面遠端看板對 reader 停用全部寫入，說明文字指出角色原因。
- docs/verb-contract 與 docs/sdk-node 反映新規則。

**Non-Goals:**

- repo 級或逐動作的權限、新角色。
- reader 可寫的例外。
- 新的 handshake capability 欄位或 Protocol 型別變更。
- 本地模式的任何行為；CLI 人眼輸出、`--json` 與 exit code。
- 把 `RemoteCapabilities` 從 Tauri 殼搬進 speclink-desktop-core（現有落點，本變更只改推導值）。

## Decisions

### D1：寫入檢查放在專案路由的 route layer，以 HTTP 方法判斷

在專案範圍 router 註冊完所有路由後，呼叫 `route_layer` 掛一個中介層：請求方法是 GET 或 HEAD 就直接放行；其他方法先解析 `Binding`，`editor` 為 false 就回 `ApiError::forbidden`（403、reason `permission_denied`），不進處理函式。

- 唯讀但用 POST 的 `POST /context` 在 `route_layer` 之後才註冊，因此不受這道檢查。這是唯一的例外，以註冊順序表達，不比對路徑字串。
- `route_layer` 只作用在已匹配的路由：不存在的路徑照舊回 404。

替代方案：
- **寫入專用 extractor（例如 `EditorBinding`），逐一換掉寫入處理函式的參數型別**——檢查規則只有一份，但新增寫入端點時仍要記得換型別，漏了就預設開放（fail-open），與這次要修的問題同型。
- **把讀寫路由拆成兩個 router 再合併**——同一路徑同時有 GET 與 POST（例如 `/changes`）時必須拆到兩邊，router 合併規則增加維護負擔。

### D2：同一請求只做一次身分解析

中介層解析出的 `Binding` 放進 request extensions；`Binding` 的 `FromRequestParts` 實作先查 extensions，有就直接回傳複本，沒有才做完整解析。寫入請求因此不會查兩次 identity store，也不會重複更新存取金鑰的最後使用時間。未通過認證的請求在中介層就收到與原本相同的 401／403 錯誤。

### D3：移除分散的角色檢查

12 個處理函式內的 `binding.editor`／`binding.policy_write` 檢查改由 D1 承擔，全部移除。`Binding.policy_write` 與 `Binding.editor` 欄位保留，handshake 的 capability 宣告仍從它們推導。

### D4：內容格式錯誤以型別標記歸類

speclink-core 命令層新增一個內容格式錯誤的型別標記（比照既有的 `Refusal`：包一段訊息字串、Display 輸出原訊息）。artifact 內容檢查與品質關卡輪內容解析改回傳這個標記；`classify` 遇到它歸為 `ErrorCode::InvalidArgv`。

- 訊息文字逐字不變，CLI 的 stderr 輸出位元級相同；CLI 對所有錯誤碼用同一個 exit code，所以 exit code 也不變。
- server 端不需修改：既有對照把 `InvalidArgv` 轉成 400 `invalid_argument`。
- Node SDK 的錯誤碼從 `error` 變成 `invalid_argv`（刻意變更，文件同步）。
- 標記只加在呼叫端送入內容的入口：`add_round` 解析輪內容、檢查輪的 `**Phase**:` 與工單狀態是否相符時標記。解析 store 裡已存在的工單失敗時仍是 `error`，因為那不是呼叫端的輸入。

替代方案：在 server 端依訊息字串判斷改回 400——字串比對脆弱，且 Node SDK 與其他入口的錯誤碼仍不一致，違反 command-runtime「同一失敗情境的錯誤碼不因入口而異」。

### D5：桌面以既有角色位推導全部寫入 capability

`RemoteCapabilities::from_binding` 以 `deleteChange`（「這個成員可寫變更」的既有宣告，認領已沿用這個作法）推導 `setTaskDone`、`setAllTasks`、`archive`、`promoteDiscussion`、`archiveDiscussion`。前端新增角色版停用說明（例：「你的角色為檢視者，只能查看」）；連線狀態為 online 而寫入 capability 為 false 時顯示角色版，離線時維持 `remote.writeUnavailable`。

替代方案：新增 Protocol capability 欄位（例如 `write`）——需要 Protocol 版本與舊 server 相容處理，對兩值角色沒有額外好處。

### D6：CLI 的 403 訊息帶出 server 原因

speclink-remote 把錯誤封套轉成 CLI 訊息時，403 `permission_denied` 目前輸出固定句子「access denied — your account has no access to this project; ask a project admin」。D1 之後 reader 寫入也會收到 403，固定句子會讓 reader 以為自己看不到專案。改為 `access denied — <封套 message>; ask a project admin`：server 對非成員回「actor is not a member of project '<代號>'」，對 reader 寫入回「your role in project '<代號>' is reader; this action needs the editor role」。401 的訊息不變。

替代方案：CLI 依 HTTP 方法自行判斷是不是 reader 寫入——CLI 拿不到角色資訊，只能猜；server 已經知道原因，直接轉述最準。

### 與「storage 解耦的規格驅動引擎」的關係

D4 讓錯誤分類留在引擎命令層，各入口（CLI、server、Node、desktop）共用同一個分類結果，不在任何 adapter 各自判斷；D1 是 server adapter 的存取控制，不進引擎。

## Implementation Contract

**行為**
- 以 reader 存取金鑰或裝置登入呼叫專案範圍任何非 GET／HEAD 端點（`POST /context` 除外）：回 HTTP 403，body 為錯誤封套，`reason` 為 `permission_denied`、`message` 為 `your role in project '<代號>' is reader; this action needs the editor role`；store 的 scope revision 不變、不產生任何領域事件。
- 以 reader 呼叫 `POST /context` 與所有 GET 端點：結果與 editor 相同。
- editor 的所有寫入行為不變。
- 未認證或非成員的請求：錯誤與變更前相同。
- 寫入沒有 `- [ ]` 的 tasks.md、缺 `## Why`／`## Problem`／`## Summary` 的 proposal、缺 `## Context` 的 design、沒有操作區段的 delta spec，或內容不合格式的審查／驗證輪：server 回 400、`reason` 為 `invalid_argument`，`message` 與 CLI 印出的訊息逐字相同；Node SDK `dispatch` 以 `code: 'invalid_argv'` 拒絕；CLI 輸出與 exit code 不變。
- CLI 遠端模式收到 403：stderr 為 `Error: access denied — <封套 message>; ask a project admin`，exit code 非 0；收到 401 的訊息不變。
- 桌面遠端看板以 reader 開啟：勾任務、全勾、封存、轉為變更、封存討論、刪除、任務搬移、看板排序、認領、前置編輯全部停用；停用說明為角色版文字。同一 reader 分頁離線時，說明改為離線版文字。editor 分頁行為不變。

**介面**
- 不新增或修改任何 HTTP 路徑、Protocol 型別或 capability 欄位。
- 錯誤封套沿用既有形狀。

**驗收**
- 新的 server 整合測試以 reader 存取金鑰逐一呼叫全部寫入端點（方法與路徑列表涵蓋 router 上每個非 GET 路由），斷言 403、`permission_denied` 與 scope revision 不變；另斷言 reader 的 `POST /context` 成功。
- server 整合測試斷言格式錯誤的 tasks.md 寫入與審查輪寫入回 400 `invalid_argument`。
- speclink-remote 的 typed client 測試斷言 403 訊息為 `access denied — <封套 message>; ask a project admin`。
- speclink-core 單元測試斷言內容格式錯誤經 `classify` 歸為 `invalid_argv`、訊息不變。
- 既有 golden（`cargo test -p speclink-core --test it render_golden::`）與 CLI 整合測試不需更新即通過。
- 桌面：`from_binding` 對 reader 與 editor 的推導結果有單元測試；前端 vitest 斷言 reader 線上時顯示角色版說明、離線時顯示離線版說明。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過（文件守門）。

**範圍**
- 範圍內：speclink-remote 的 403 訊息、speclink-server 的路由層與 `Binding` extractor、12 個處理函式的檢查移除、speclink-core 的錯誤分類與兩處內容檢查、Tauri 殼的 capability 推導、桌面前端停用說明與 i18n、docs/verb-contract 與 docs/sdk-node 兩語版。
- 範圍外：Protocol 型別、server-web 後台、本地模式、CLI 文案、更細的權限模型。

## Risks / Trade-offs

- [既有 server 整合測試以 reader 呼叫寫入端點、預期成功] → 實作時以 `reader` 搜尋測試，逐一改成預期 403 或改用 editor；新增測試涵蓋全部寫入路由。
- [Node SDK 使用者依賴內容錯誤的 `error` 碼] → 文件明列錯誤碼變更；變更屬修正，影響面只有錯誤分支。
- [回歸對照：CLI 錯誤訊息被意外改動] → 型別標記的 Display 輸出原訊息；golden 與 CLI 測試不更新即須通過。
- [桌面測試前置條件] → speclink-desktop crate 的測試需要先佈 sidecar 與 server-web dist；實作時依 docs/development 的測試前置步驟執行。
- [跨平台] → 只動 HTTP 層與錯誤分類，沒有檔案路徑或平台差異；CI 三平台照常執行。
- [舊 server 沒宣告 `deleteChange`] → 桌面以 `deleteChange` 推導寫入面，欄位缺席時 editor 也會被停用，並看到角色說明。這個宣告自 2026-07-23（remote-verb-parity）起就存在，v0.1.0 以來的 server 都有；更早的 server 尚未開放遠端寫入動詞。明確接受。
- [`POST /context` 以外未來新增的唯讀 POST 會被擋] → 預設擋下是刻意的 fail-closed；新增唯讀 POST 時，需要在 `route_layer` 之後註冊，並在設計中說明。

## Migration Plan

1. 部署新版 server。
2. 需要寫入、但目前是 reader 的成員，由管理員在 `/admin/users` 改成 editor。
3. 回退：部署上一版 server 即可，沒有資料格式變更。
