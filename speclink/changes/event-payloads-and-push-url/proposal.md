## Why

宿主（內嵌了引擎的系統）與用戶端靠事件知道「有東西變了」。這條路今天有三個接不上的地方：

1. 蓋章事件沒有程式碼的版本。審查或驗證蓋章時，事件 `review-stamped`、`verify-stamped` 的內容只有變更名稱。蓋章的請求其實帶著當時的 commit，引擎也把它記進了 metadata，但事件沒有帶出來。宿主在儲存的 `commit` 裡把事件寫進自己的事件表時，還要另外讀一次才知道是哪個版本。同類的落地事件 `change-landed` 就有帶 commit。
2. 改流程設定沒有事件。當初把它留給「提供掛鉤的變更」決定，後來宿主套件只在執行前掛鉤給了變動的鍵。執行前掛鉤發生在提交之前；提交成功之後，宿主的事件表與連著的用戶端都收不到任何通知。
3. Speclink Desktop 不理會宿主宣告的推送網址。握手可以宣告 SSE 串流（SSE：Server 經一條長連線持續推送事件）的網址，可是 Desktop 只檢查「有沒有宣告 SSE」，然後連自己寫死的路徑。宿主照文件用 `pushTransports` 宣告了自己的串流網址，Desktop 也連不到，只能輪詢。正式規格 server-event-stream 寫的是「宣告的 url 即事件端點」。另外，文件沒有寫串流的心跳間隔與用戶端的斷線規則，自己做串流的宿主無從得知。

受影響的人：寫宿主程式的整合者（事件表、自己的 SSE 串流），以及用 Speclink Desktop 連到宿主的工程師（畫面不是即時的）。來源討論 embedded-host-needs-2 裁定：事件維持「只指出哪裡變了、收到後重新讀取」的設計並寫進文件；蓋章事件帶 commit；新增流程設定變動的事件；Desktop 改用宣告的 SSE 網址。

掃過的相關正式規格：command-runtime（「變更型動詞的領域事件」）、server-event-stream（「binding 宣告 sse transport」「審查與驗證事件歸到變更」）、remote-workspace-data（Desktop 的事件訂閱與斷線收斂）、workflow-config（「流程設定的寫入經命令執行層」）、host-kit（提交後掛鉤拿到的事件）、store-port-contract（事件與寫入同一次提交）。

## What Changes

- 蓋章事件帶 commit：`review-stamped` 與 `verify-stamped` 的內容在章有記下 commit 時多一個 `commit`；沒有記下時維持只有變更名稱。
- 新增事件 `workflow-config-changed`：流程設定的寫入真的寫入時發出，內容是變動的鍵 `keys`（字串陣列，可以是空的：只改了排版或註解）。沒有寫入（內容相同、dry-run、被拒絕）時不發。
  - 它和寫入在同一次提交，所以宿主在儲存的 `commit` 與提交後掛鉤都收得到。
  - 推送給用戶端時歸在「其他」，用戶端重新讀取全部。
- Speclink Desktop 連宣告的 SSE 網址：
  - 宣告的網址沒有協定與主機時，接在專案的 API 位址後面（和輪詢網址 `/sync-state` 的寫法一致；示範用 server 宣告的 `/events` 因此照常可用）。
  - 宣告的網址是完整的網址時，只在它和 Server 是同一個 origin（協定、主機、埠都相同）時使用，避免把登入憑證送到別的主機。不是同一個 origin 時不連線，改用輪詢。
  - 只宣告 WebSocket 時照舊輪詢。
- 使用者文件寫明：
  - 事件的設計：事件只指出哪個資源變了，內容刻意精簡，收到後重新讀取。
  - 推送網址的兩種寫法與同 origin 的規則。
  - 串流的心跳：示範用 server 預設每 15 秒送一次；用戶端 45 秒沒有收到任何資料就斷線重連。自己做串流的宿主要在 45 秒內送出心跳。
  - 兩種事件的新內容。

相容性影響：

- 不新增或變更任何 CLI 子指令、旗標、stdin、exit code；動詞的人眼輸出與 `--json` 輸出都不變。
- 事件的內容是新增欄位與新增一種事件：只認得舊內容的宿主忽略新欄位即可；用戶端收到的推送格式（哪個資源變了）不變。
- 流程設定的寫入以前沒有事件；有測試或宿主程式假設「這種提交的事件清單是空的」時要調整。repo 內這類斷言同批更新。
- Speclink Desktop 連到示範用 server 的行為不變（宣告的 `/events` 解析出來就是原本的路徑）。連到宣告了別的同 origin 網址的宿主時，從「連不到、只輪詢」變成「連得到」。

三種形態：

- 本機 repo：事件在本機專案不保存也不推送；命令層回報的事件同樣多出 commit 與新的事件種類，沒有可觀察的差別。推送網址不適用。
- 連到遠端的 checkout：CLI 不訂閱事件，不受影響。Speclink Desktop 連到遠端專案時使用宣告的 SSE 網址。
- 沒有本機規格檔的內嵌宿主：可用。宿主在儲存的 `commit` 與提交後掛鉤收到的事件多出 commit 與新的事件種類；宿主宣告的 SSE 網址會被 Speclink Desktop 使用。

不涉及設定欄位，不涉及生成的技能與 Agent 指令。

## Non-Goals

- 用戶端不實作 WebSocket。
- 推送給用戶端的訊息不帶事件的內容（commit、鍵）：它維持只指出哪個資源變了。
- 不為流程設定新增一個推送的資源類別；歸在「其他」已經會讓用戶端重新讀取。
- 不改輪詢的網址與規則。
- 宿主套件不提供現成的 SSE 串流；跨實例的推送維持在已知缺口。
- 其他事件（丟棄工單、認領等）的內容不改。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `command-runtime`: 新增一條需求——蓋章事件帶 commit，以及流程設定變動的事件。
- `remote-workspace-data`: 新增一條需求——Speclink Desktop 的事件串流網址取自握手的宣告，含解析與同 origin 的規則。
- `server-event-stream`: 新增一條需求——宣告的串流網址的寫法與心跳的契約。

## Impact

- Affected specs: command-runtime、remote-workspace-data、server-event-stream
- Affected crates: speclink-core（兩種事件的內容與發出條件）、speclink-remote（串流網址的解析與訂閱）、apps/desktop（把握手宣告的網址交給訂閱）；speclink-host 與 speclink-node 只更新測試
- Affected code:
  - New: (none)
  - Modified:
    - crates/engine/speclink-core/src/command/mod.rs
    - crates/engine/speclink-core/src/command/tests.rs
    - crates/engine/speclink-core/src/command/workflow_config.rs
    - crates/host/speclink-host/src/commit.rs
    - crates/host/speclink-host/tests/kit.rs
    - crates/host/speclink-host/tests/station_wire.rs
    - crates/host/speclink-host/tests/http_door.rs
    - crates/protocol/speclink-remote/Cargo.toml
    - Cargo.lock
    - crates/protocol/speclink-remote/src/events.rs
    - crates/protocol/speclink-remote/tests/it/events_sse.rs
    - apps/desktop/src-tauri/src/remote.rs
    - apps/desktop/src-tauri/src/lib.rs
    - crates/adapters/speclink-node/__test__/host.spec.ts
    - crates/adapters/speclink-node/__test__/workflow-config-verbs.spec.ts
    - crates/adapters/speclink-node/index.d.ts
    - docs/verb-contract.md
    - docs/verb-contract.zh-TW.md
    - docs/sdk-node.md
    - docs/sdk-node.zh-TW.md
    - docs/product-status.md
    - docs/product-status.zh-TW.md
  - Removed: (none)
