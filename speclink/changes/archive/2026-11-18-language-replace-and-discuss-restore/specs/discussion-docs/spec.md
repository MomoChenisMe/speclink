## ADDED Requirements

### Requirement: 討論以 restore 動詞搬回進行中

speclink discuss restore SHALL 接受一個位置參數（討論 slug），把該 slug 已封存的討論記錄搬回進行中：記錄離開封存、成為以 slug 命名的進行中記錄，內容 SHALL 逐字元不變（frontmatter、Context、Rounds、Conclusion 都不改寫）。同一個 slug 有多份封存的記錄時 SHALL 搬回最新的一份——封存名稱的日期最新者；同一天重複封存（名稱帶 -N 序號）時取序號最大者，也就是 discuss show 在沒有進行中記錄時讀到的那一份——其餘 SHALL 留在封存裡。搬回之後，discuss show、add-round、conclude、promote、link、seal 等動詞 SHALL 把它當成進行中的記錄處理，discuss list SHALL 列出它，discuss list --archived SHALL 不再列出被搬回的那一份。

拒絕條件：slug 不是單一名稱（空字串、. 與 ..，或含路徑分隔字元）時 SHALL 在任何讀寫之前拒絕；該 slug 已有進行中的記錄時 SHALL 拒絕且零寫入，訊息 SHALL 指出進行中的記錄已存在；該 slug 沒有任何封存的記錄時 SHALL 以 not-found 拒絕。搬回 SHALL 是一次原子的寫入：不是「進行中有、封存沒有」，就是什麼都沒變；SHALL NOT 留下兩邊都有或兩邊都沒有的狀態。成功時 SHALL 發布事件 `discussion-restored`，歸到該討論。

動詞 SHALL 於本機、remote 與宿主套件三種執行方式同語意：本機專案把檔案自 speclink/discussions/archive/ 搬回 speclink/discussions/ 並拿掉檔名的日期前綴；remote 模式經契約端點；宿主套件經引擎命令（穩定名稱 `discuss-restore`），執行前 SHALL 先問執行前掛鉤。

輸出：成功時 stdout SHALL 是兩行——`✓ Restored discussion: <slug>` 與 `  From: <封存時的名稱>`；exit code 0；`--no-color` 下文字相同、不含色碼。`--json` SHALL 輸出一個物件：`slug`（字串）與 `restoredFrom`（字串，封存時的名稱，不含資料夾）。失敗時訊息走 stderr、exit code 非零。人眼輸出在三種執行方式 SHALL 逐位元一致，且 SHALL 不隨 `locale`（`tw`、`ja`、`en` 或未設定）改變。remote 路徑撞上 revision 衝突、離線或認證失效時 SHALL 與其他討論寫入動詞相同地失敗。

討論已封存時拒絕的四個動詞（add-round 等續寫動詞、promote、link、seal）的訊息 SHALL 改為指向本動詞：訊息 SHALL 含 `speclink discuss restore <slug>`，SHALL NOT 再指示使用者搬動 discussions/archive/ 底下的檔案。這是刻意變更的輸出：變更前訊息的後半句是「move it out of discussions/archive/ to …」，變更後是「run `speclink discuss restore <slug>` to …」；exit code 與拒絕的條件 SHALL 不變。

#### Scenario: 搬回被隨行封存的討論

- **WHEN** 討論 demo-topic 已被封存為 `2026-11-10-demo-topic.md`，沒有同 slug 的進行中記錄，執行 `speclink discuss restore demo-topic`
- **THEN** exit code 0，stdout 是 `✓ Restored discussion: demo-topic` 與 `  From: 2026-11-10-demo-topic.md` 兩行；`speclink discuss show demo-topic` 的內容與封存時逐字元相同；`speclink discuss list` 列出它，`speclink discuss list --archived` 不再列出它；之後 `speclink discuss add-round demo-topic` 成功

#### Scenario: 多份封存時搬回最新的一份

- **WHEN** 同一個 slug demo-topic 有兩份封存：`2026-09-01-demo-topic.md` 與 `2026-11-10-demo-topic.md`，執行 `speclink discuss restore demo-topic --json`
- **THEN** stdout 是 `{"slug":"demo-topic","restoredFrom":"2026-11-10-demo-topic.md"}`（鍵的順序不拘）；`2026-09-01-demo-topic.md` 仍在封存裡

#### Scenario: 已有進行中的記錄時拒絕

- **WHEN** demo-topic 有一份封存的記錄，同時有一份進行中的同 slug 記錄，執行 `speclink discuss restore demo-topic`
- **THEN** 指令以非零 exit code 結束，stderr 指出進行中的記錄已存在；兩份記錄都逐位元不變

#### Scenario: 沒有封存的記錄

- **WHEN** 對一個沒有任何封存記錄的 slug 執行 `speclink discuss restore ghost`
- **THEN** 指令以非零 exit code 結束，stderr 指出找不到 ghost 的封存記錄，零寫入

#### Scenario: remote 模式與宿主套件同語意

- **WHEN** Server 上有一份封存的討論 demo-topic，分別以連到該 Server 的 CLI 執行 `speclink discuss restore demo-topic`，以及在另一個內容相同的範圍經宿主套件執行同一個動詞
- **THEN** 兩者的 stdout 逐位元相同；兩邊的討論都回到進行中的清單；各有一筆 discussion-restored 事件

#### Scenario: 已封存的拒絕訊息指向 restore

- **WHEN** 討論 demo-topic 已封存，執行 `speclink discuss promote demo-topic`
- **THEN** 指令以非零 exit code 結束，stderr 含 `speclink discuss restore demo-topic`，不含 `move it out of`
