## Context

桌面 app 的 speclink 網址由前端處理：`apps/desktop/src/deepLink.ts` 的 `parseSpeclinkUrl` 解析網址，`openLink` 依序喚起主視窗、開專案、切畫面。現有種類是 change、discussion、roadmap；`roadmap` 不帶名稱，派送時呼叫 store 的 `openRoadmap`。Tauri 殼只註冊外掛，不看網址內容。

Herdr 面板的 Logo 在 `integrations/herdr/mods/speclink/main.js` 的 `logoButton`，它用 open 指令加 -b 參數叫出 app，所以 app 不切專案。面板的 `this.root` 由 `projectRoot` 算出，worktree 裡的 space 也會得到主專案的路徑，與桌面 app 的分頁是同一個專案。

2026-11-04 查到三個會影響舊版處理的事實：

- 網址功能的 commit（c6fd3309、f7ab4a85，2026-11-03）不在任何 tag 裡；v0.11.0 是 2026-10-27 發的。所以 v0.11.0 以前的發佈版都沒有註冊 speclink 網址。
- open 指令遇到沒有註冊的網址格式時，結束碼是 1，訊息是 No application knows how to open URL。
- 這台機器的安裝版（由 main 建置）回報版號 0.11.0，但有註冊 speclink 網址。也就是說，版號相同的 app 可能有網址功能，也可能沒有。

變更落點只有 `apps/desktop` 的前端與 `integrations/herdr`。不動任何 crate，沒有 local／remote 兩條路徑的問題：網址只由桌面 app 處理，Herdr 面板只負責送出。不新增 UI 元件，提示沿用既有的 sonner toast。

## Goals / Non-Goals

**Goals:**

- 桌面 app 認得 `speclink://project`，只切專案並顯示看板的卡片檢視。
- Logo 在讀得到本機專案時切到該專案，其餘情況維持只叫出 app。
- 舊版 app 上，Logo 一定能把 app 叫到前景；兩種舊版的結果都寫清楚。

**Non-Goals:**

- 不改現有三種網址的格式與行為，也不改「路線圖」鈕與點列的行為。
- 不放寬格式錯誤的規則：未知種類的網址維持不切換專案，不做「認不得種類就盡力切專案」。
- 不偵測桌面 app 的版本（理由見 Decisions）。
- 不給 Herdr 面板立新的正式規格。面板到目前都沒有正式規格，這次用 README 與測試記錄。
- 不做 Windows、Linux 上的 Logo。open 指令只有 macOS 有，這是 README 已寫明的既有限制。

## Decisions

### 新增 `speclink://project` 網址種類，沿用路線圖的解析規則

`project` 和 `roadmap` 一樣不帶名稱，所以兩者共用同一條解析分支：path 只能為空或單一 `/`，否則回報 `name` 錯誤；`SpeclinkLink` 的無名稱分支把 kind 擴成 `"roadmap" | "project"`。

替代做法是 `speclink://board`。沒有採用，因為這個網址的主要目的是切專案，看板只是切完的畫面，而且需求已指定 `project`。

### 新增 store 動作 `openBoard`，固定切到卡片檢視

`openBoard` 與 `openRoadmap` 同一個結構：不可取消的浮層開著時不動作；否則切回看板頁（順便關掉詳情頁）、收掉可取消的浮層、把 `boardMode` 設成 `cards`。

替代做法與不採用的原因：

- 只切回看板頁、維持上次選的檢視：上次選路線圖時按 Logo 會看到路線圖，違反「不開路線圖」。
- 在 `deepLink.ts` 裡分別呼叫切頁與切檢視：收掉可取消浮層的邏輯是 store 私有的，會漏掉這一步。

切完之後，「上次選的檢視」就是卡片。這和 `speclink://roadmap` 把它改成路線圖是同一種語意：網址明確指定檢視。

### Logo 由純函式 `logoOpenAttempts` 決定要送什麼

`integrations/herdr/mods/speclink/model.js` 新增 `logoOpenAttempts({ root, hasBoard, on })`，回傳依序嘗試的 open 參數清單：

- `hasBoard` 為真且 `on` 為空：先送 `speclink://project` 網址，再退回 -b 參數。
- 其他情況：只有 -b 參數這一組。

門檻與「路線圖」鈕相同（`Boolean(board) && !on`）。專案沒初始化或還在載入時 `board` 是 null，別台機器的 space 有 `on`，所以這兩種情況都只叫出 app。放在 `model.js` 是因為 `main.js` 依賴 Herdr API，測不到。app 的 bundle id 常數（`APP_ID`）一起搬到 `model.js`，測試才能對照。

Logo 的提示文字跟著這個結果走：會切專案時寫「在 Speclink app 開啟這個專案」，只叫出 app 時維持「開啟 Speclink app」。

### 舊版 app 靠「送不出去就退回」處理，不偵測版本

`main.js` 新增 `openFirstInApp(attempts, cx)`：依序跑 open，成功就停；全部失敗才跳既有的「開不了 Speclink」通知，內文是最後一次有內容的錯誤。`openInApp` 改成委派給它，所以現有呼叫端不用改。

兩種舊版的結果：

| 舊版種類 | open 的結果 | Logo 的結果 |
| ---- | ---- | ---- |
| 沒有註冊 speclink 網址（v0.11.0 以前的發佈版） | 結束碼 1 | 退回 -b 參數，app 叫到前景，和現在一樣 |
| 有註冊但不認得 `project`（2026-11-03 之後、本變更之前自行建置的版本） | 結束碼 0 | app 叫到前景，專案不切換，跳出「無法開啟這個 speclink 網址」 |

第二種的提示擋不掉，Herdr 面板也看不到 app 內部的結果，所以只寫進 README。因為下一個發佈版會同時帶著網址功能與 `project`，第二種只會出現在自行建置的版本上。

替代做法與不採用的原因：

- 比對 app 版號再決定送什麼：這台機器的安裝版就是反例，版號都是 0.11.0，卻可能有或沒有網址功能，比對不出來；每次按 Logo 也要多跑子程序。
- 退回時改送 `speclink://roadmap`：舊版認得，但會開路線圖，違反需求。

## Implementation Contract

**可觀察行為：**

| 狀況 | Logo 的結果 |
| ---- | ---- |
| 目前 space 是本機、已初始化的專案，app 認得 `project` | app 到前景，分頁切到該專案，主區是看板的卡片檢視，詳情頁關閉 |
| 目前 space 在 worktree 裡 | 同上，切到主專案的分頁 |
| 專案沒初始化、還在載入、讀不到專案 | app 到前景，分頁與畫面不變 |
| space 在別台機器上 | app 到前景，分頁與畫面不變 |
| app 沒有註冊 speclink 網址 | app 到前景，分頁與畫面不變，沒有錯誤通知 |
| app 有註冊但不認得 `project` | app 到前景，分頁與畫面不變，跳出網址無效的提示 |
| app 沒有安裝，或系統沒有 open 指令 | 跳出「開不了 Speclink」通知 |

帶 remote 標記的專案和「路線圖」鈕一樣，由 app 改開選擇器，確認前不寫入。

**介面：**

- 網址：`speclink://project?project=<百分比編碼的絕對路徑>`，path 可以是空或單一 `/`。
- `parseSpeclinkUrl` 的結果多一種 `{ kind: "project", project }`。
- store 多一個動作 `openBoard(): void`，`DeepLinkStore` 的 `Pick` 要一起加上。
- `speclinkUrl("project", root)` 產生網址，不用改函式本體，只更新註解。
- `logoOpenAttempts({ root, hasBoard, on })` 回傳 `string[][]`。

**失敗模式：** 解析失敗沿用既有的 toast，不切換專案。`openProjectAt` 回傳 null 時（確認框或選擇器接手、載入失敗、載入中已切走）不切畫面，也不出 toast。不可取消的浮層開著時只喚起主視窗。沒有新的文案。

**驗收條件：**

- `apps/desktop/src/__tests__/deepLink.test.ts`：解析表涵蓋規格「網址解析」新增的四列；派送依序是喚起主視窗、開專案、`openBoard`；`openProjectAt` 回傳 null 與不可取消浮層兩種情況都不呼叫 `openBoard`。
- `apps/desktop/src/__tests__/store.test.ts`：`openBoard` 從詳情頁、規格頁與路線圖檢視都回到看板頁的卡片檢視，而且收掉可取消浮層；不可取消浮層開著時不改狀態。
- `scripts/herdr/panel-model.test.mjs`：`speclinkUrl("project", ...)` 的格式與編碼；`logoOpenAttempts` 的本機專案、`hasBoard` 為假、有 `on` 三種情況。
- 手動：在真實的 Herdr 與重新安裝的桌面 app 上，照「可觀察行為」表的前四列操作 Logo。

**範圍：**

- 範圍內：網址解析與派送、`openBoard`、Logo、兩份 README、兩份 product-status。
- 範圍外：現有三種網址、「路線圖」鈕與點列、桌面 app 的 Tauri 殼、發版流程。

## Risks / Trade-offs

- [回歸] 解析分支與派送是現有三種網址共用的路徑 → `deepLink.test.ts` 的解析表與派送案例全部保留並照常跑；這次沒有 golden 或 CLI 測試受影響，因為不動 crate 與技能。
- [跨平台] Windows 路徑的網址沿用既有的絕對路徑判斷，不用新規則 → 解析表沿用同一個判斷。Logo 只有 macOS 能送網址，其他系統維持跳通知，這是既有限制。
- [發版順序] 如果下一版在本變更合併前發佈，就會有認得 `speclink://` 但不認得 `project` 的發佈版，README 寫的版本要跟著改 → 合併時對照最新的 tag。
- [取捨] 按 Logo 會關掉開著的詳情頁，並把路線圖切回卡片 → 這是「顯示看板」的需求本身，路線圖網址也會關詳情頁。
- [取捨] 切 space 的瞬間 `this.root` 還是上一個專案，這時按 Logo 會切到上一個專案 → 「路線圖」鈕有同樣的極短空窗，下一次重新讀取就更新，不另外處理。
