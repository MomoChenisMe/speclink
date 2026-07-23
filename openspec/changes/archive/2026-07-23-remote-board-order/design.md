## Context

本地看板順序真相＝卡片 meta 的 board_rank（變更卡在 .openspec.yaml、討論卡在 frontmatter），拖排以鄰居中點鍵單檔寫回、欄內缺 rank 先整欄補章——演算法（spread／midpoint）在桌面 core 的 rank 模組，編排在 apps/desktop/core/src/manage.rs。remote 模式此路不通：架構文件「不建議的做法」明文不把 board/card 呈現狀態混入共享規格 revision，並指定共享順序走「獨立、具 CAS 的 board resource」。store 側的地基已備：三 driver 對 DocumentId 走各自的字串編碼（crates/speclink-store-fs/src/layout.rs、crates/speclink-store-sqlite/src/lib.rs、crates/speclink-store-postgres/src/lib.rs），export 以泛用文件列舉組 Bundle（新種類自動隨行）；server 的 PUT /config 已立下「非引擎 Command 的 scope 文件直寫＋CAS」handler 先例。桌面側 UI 的欄分組（changeStage 同構）與拖排 affordance 皆由 capability 管線驅動，remote ChangeItem 已帶 stage 推導所需欄位（totalTasks／completedTasks／startedAt）。

範圍內：DocumentId 新種類與三 driver＋conformance、server 兩端點、protocol／client、桌面 remote 排序 overlay 與拖排直達、capability 翻正。範圍外：本地順序真相搬遷、per-user 順序視圖、board resource 歷史 UI、server 端語意校驗。

## Goals / Non-Goals

- Goals：remote 看板拖排與本地同手感、順序全團隊共享且跨重啟持久；規格 revision 與文件歷史不被呈現層寫入汙染；capability 停用清單清空。
- Non-Goals：不動本地路徑任何行為；不引入 per-user 呈現狀態的新儲存；不讓 server 理解 board 內容。

## Decisions

### 決策 1：共享順序＝獨立 CAS board resource，卡片 meta 不動

remote 拖排寫入獨立的 board resource 文件，變更卡與討論卡的 meta／frontmatter 在 remote 模式下 SHALL NOT 因拖排被觸碰。替代案 A：照本地把 rank 寫進卡片 meta——每次拖卡 mutate 共享文件 revision、汙染 append-only 歷史、觸發全員 invalidate，架構明文禁止，否決。替代案 B：per-user 本機順序（localStorage）——與本地「rank 進 git 即團隊共享」語意分家，同團隊兩人看到不同欄序，否決（記為未來偏好需求）。本地模式維持 meta 真相不動——兩模式真相不同但排序語意同構，規格以分模式改寫釘住。

### 決策 2：BoardOrder 為 scope 層級單文件、內容不透明、泛用列舉自動隨行

DocumentId 新增無參數的 board order 種類（同 WorkflowConfig／Language 的 scope 單文件形狀）；內容為 JSON 兩段圖（changes：變更名→rank、discussions：slug→rank，rank 沿既有小寫字母字典序鍵）。server 視內容為不透明文本：不經引擎解析、不校驗語意（呈現資源非政策文件），僅設大小上限（拒絕異常巨量 payload）。三 driver 各補一組編碼／解碼字串；conformance suite 補 round-trip 與 export 涵蓋案例——export／import 與 backup 走既有泛用列舉，零額外程式。替代案：結構化 DTO＋server 端校驗——server 因此理解呈現語意、跨層耦合，且校驗失敗會把看板整個 fail，否決（容錯歸桌面，見決策 6）。

### 決策 3：PUT 全文＋If-Match CAS 沿 put_config 先例

GET /board-order 回內容（缺席為正常態、回 null 內容）＋scope ETag；PUT /board-order 帶 If-Match 與全文，handler 沿 put_config 的直寫形狀（role 檢查→snapshot revision 比對→UoW commit CAS），不新增引擎 Command——board 順序不是 workflow 語意，進 Command enum 是詞彙汙染。PUT editor 限定（reader 403 reason 機器可判）；commit 後既有 notify 使訂閱端收 invalidate、他端看板數秒內反映新序。409＝informed resubmit 語意（重讀重算後重試），非盲目覆寫。

### 決策 4：排序 overlay 在桌面 Rust 側，UI 零改動

remote 清單指令（changes 與 discussions）同時取 board resource，於 Rust 側合併排序後回傳：具 rank 卡依 rank 字典序升冪、缺 rank 卡置頂並維持 server 回傳序（remote 的確定回退序）、同 rank 以名稱／slug 字典序決斷——與本地 board-card-order 排序語意同構。UI 元件與 TS 層不做排序、不知道 board resource 存在。替代案：TS 側 overlay——remoteDataSource 要多打一個 invoke 並在前端合併，把順序邏輯劈成兩層，否決。

### 決策 5：拖排寫回＝讀清單＋board resource→補章／中點→PUT 全文，409 重試一次

remote reorder 指令流程：取當下清單與 board resource → 依 stage 同構推導被拖卡所在欄成員 → 欄內有缺 rank 卡時依當前顯示序整欄補章（重用桌面 core rank 模組的 spread；等距鍵只落在 board resource 圖內，不觸碰任何卡片文件）→ 以落點鄰居中點鍵（重用 midpoint；消失的鄰居視為開放端、現值逆序棄上界保底——沿本地既有語意）更新被拖卡條目 → 修剪不在現行清單的孤兒條目 → PUT 全文帶 If-Match。409 時重讀重算重試一次；再敗回單行錯誤、UI 刷新至 server 現況（不留假象順序——沿 board-card-order 的失敗語意）。替代案：PUT 差分（單條目 patch 端點）——server 得理解內容結構，違決策 2 的不透明原則，否決。

### 決策 6：損壞容錯與孤兒條目歸桌面

board resource 內容非法（壞 JSON、非預期形狀）時，桌面視為全員缺 rank——回退序照常渲染、看板不 fail；下一次拖排的補章＋PUT 全文自然重建文件。已封存／刪除卡的殘留條目在排序時無害（查無此卡即忽略）、在每次 PUT 重寫全圖時修剪。替代案：server 端校驗擋壞內容——見決策 2 否決理由；桌面容錯讓壞文件的爆炸半徑=退回預設序。

### 決策 7：capability 依 role 翻真，停用清單清空

RemoteCapabilities 的 reorderCard 依 role 翻真（editor 真、reader 假——寫入面沿 remote-verb-parity 的 role 呈現模式）；remoteDataSource.ts 的 reorderCard 由 unsupported 拒絕改 invoke 直達。本刀後 remote-workspace-data 的 capability 停用清單清空；規格修訂以 remote-verb-parity 修訂後文本為基準——apply 與 archive 順序排在該刀之後（共同熱點檔：remote.rs、remoteDataSource.ts 與其測試，依平行 session 提交衛生合流）。

## Risks / Trade-offs

- 拖排多一次 GET＋PUT 往返：拖排是顯式低頻操作，可接受；409 重試一次上限避免活鎖。
- 兩人同時拖同欄：後者 409 重讀重算後通常仍能落位（鄰居仍在則中點語意成立）；同張卡互搶為 last-write-wins——順序資料無正確性風險，SSE invalidate 數秒內全員收斂。
- 清單與 board resource 兩次讀非同一時點：極端下欄成員判定過期→補章名單偏差，PUT 的 CAS 與 invalidate 收斂矯正；不為呈現資源引入 snapshot 綁定複雜度。

## Migration Plan

純新增文件種類與端點：既有 scope 無 board resource 時一切走回退序（與本刀前行為一致），第一次拖排時建立文件。export／import／backup 泛用列舉自動涵蓋，無資料遷移。回滾＝revert（殘留的 board resource 文件被舊版桌面忽略——舊版不讀此種類）。

## Open Questions

（無——per-user 視圖與本地真相搬遷已明確記為範圍外。）
