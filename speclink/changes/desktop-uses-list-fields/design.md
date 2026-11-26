## Context

前置變更 change-and-discussion-list-fields 做完之後，引擎有三樣新東西可以用：不讀儲存的階段函式與就緒日的顯示規則（speclink-core）、metadata 的 `created_by_id` 與 `ready_at`、HTTP 回應的 `stage`、`createdById`、`readyAt` 與討論的 `modifiedAt`。Desktop 目前的狀況：

- 階段：`packages/ui/src/stage.ts` 的 `changeStage` 用任務數與 `startedAt` 自己算，9 個檔案呼叫它；Tauri 殼的 `remote.rs` 另有一份 `change_stage`，遠端專案拖排時用來判斷卡片在哪一欄。兩份都和引擎的規則相同。
- 建立者篩選：`packages/ui/src/search.ts` 的 `BoardFilters.createdBy` 存一個「名字 <信箱>」字串，`matchesFilters` 做全等比對；`KanbanBoard.tsx` 把看板上出現過的名稱去重成選單。
- 卡片：變更卡的 meta 列是進度條與「完成數/總數」，討論卡的 meta 列是「N 輪」。變更的清單資料已經有 `modifiedAt`，但畫面沒有用；討論的清單資料沒有修改時間。
- 詳情頁：出身行由 `ChangeDetailPage.tsx` 組出，開工日取自單一變更的資料（本機是 desktop-core 的 `change_meta_at`，遠端是 Server 的單一變更回應）。

限制：Desktop 連到舊版 Server 時拿不到新欄位，每一項都要有退路。卡片的順序以拖排的順序為準（board-card-order）。使用者可見的文案不出現工程詞（LANGUAGE）。

## Goals / Non-Goals

**Goals:**

- 詳情頁看得到就緒日；看板卡片看得到最後更新。
- 階段的規則之後只改引擎一處：新版 Server 與本機專案的畫面跟著引擎走。
- 建立者篩選用識別認人，改名與同名不再出錯。
- 沒有新欄位的資料來源（舊版 Server、本機專案的建立者識別）行為和今天相同。

**Non-Goals:**

- 不加「我建立的」篩選：whoami 的回應沒有識別，討論已否決。
- 不依最後更新排序卡片。
- 不拿掉 Desktop 自己的階段判斷：它是連到舊版 Server 時的退路。
- 系統匣面板的卡片、已轉出的細列、已封存頁與規格頁不加最後更新（規格頁本來就有）。
- 討論詳情頁的出身行不加最後更新。
- 不改 CLI、HTTP 回應與引擎的任何行為。
- 不修正本機檔案時間不可靠的問題；沿用規格頁同樣的來源。

## Decisions

### D1：階段函式改成「有值就用、沒有才算」，呼叫端不改

`ChangeItem` 加選填的 `stage`。`changeStage(c)`：`c.stage` 是 `proposed`、`in-progress`、`ready` 三個值之一時直接回它；否則照既有的規則算。其他值（未來的引擎多出新階段）視同沒有，照既有規則算，不讓看板出現放不進任何一欄的卡片。9 個呼叫端不動。

Tauri 殼的 `change_stage` 同理：回應帶 `stage` 時用 speclink-core 的 `Stage::parse` 轉成欄的序號；沒有時改成呼叫前置變更的階段函式，不再自己寫一次規則。這樣引擎以外的 Rust 那一份規則消失，TypeScript 那一份留作退路。

desktop-core 的 `list_changes_at` 用引擎的階段函式在每個清單項疊上 `stage`（metadata 損壞的項目不疊）。遠端清單把 Server 的回應原樣攤平，`stage` 不用另外對應。

替代方案「前端一律自己算、不讀 `stage`」是現況，使用者已決定改。替代方案「拿掉前端的規則」會讓連到舊版 Server 的看板沒有階段，否決。

### D2：建立者篩選的選項以「人」為單位

把「卡片屬於哪一個人」做成 `search.ts` 的一個純函式，輸入看板上全部的變更卡與討論卡，輸出每張卡的歸屬鍵與選單的選項：

1. 有 `createdById` 的卡片，歸屬鍵是識別。
2. 沒有識別、只有名稱的卡片：這個名稱在有識別的卡片裡只對應到一個識別時，歸到那個識別；對應到零個或兩個以上時，歸屬鍵是名稱本身（今天的行為）。
3. 識別與名稱都沒有的卡片沒有歸屬，選了任何建立者時都不顯示（今天的行為）。
4. 選項的標籤是名稱：一個識別有好幾個名稱時，取清單順序裡最先出現的那一個。兩個選項的標籤相同時（同名的兩個人），各自在標籤後接識別的最後 4 個字元來區分。
5. 選項的順序維持今天的規則：依卡片在清單裡出現的順序。

`BoardFilters.createdBy` 改存歸屬鍵；`matchesFilters` 比對卡片的歸屬鍵。第 2 點是提案時加的：遠端專案在升級前建立的舊卡片沒有識別，同一個人會同時有「有識別」與「沒有識別」的卡片，不合併的話選單會出現兩個同名的選項。

本機專案的卡片都沒有識別，每一步都落在「歸屬鍵是名稱」，結果和今天逐一相同。

desktop-core 的 `list_changes_at` 在清單項疊上 `createdById`（取自 metadata）；討論清單項的 `createdById` 由引擎的討論資訊帶出；`ChangeItem` 與 `DiscussionItem` 各加選填的 `createdById`。

### D3：卡片的最後更新放在 meta 列尾端

`ChangeCard` 與 `DiscussionColumn` 的全尺寸討論卡，在 meta 列的尾端加一段：`relativeDays(modifiedAt)` 的結果（今天、昨天、N 天前），沿用 `packages/ui/src/time.ts` 既有的函式與字典。這一段帶主題化提示，內容是「最後更新」加上以使用者時區呈現的日期與時間。`modifiedAt` 缺席或解析不出來時整段不出現（不顯示原始字串）。`DiscussionItem` 加選填的 `modifiedAt`；`ChangeItem` 補上型別裡漏掉的 `modifiedAt`（清單資料本來就有）。拖曳時浮起的卡片與系統匣面板的卡片不加。

### D4：就緒日接在開工日之後

`ChangeMetaInfo` 加選填的 `readyAt` 與 `createdById`。`ChangeDetailPage` 在出身行的開工段之後加一段「就緒 YYYY-MM-DD」，字典加一個詞條（三種語言）；`readyAt` 缺席時整段不出現。desktop-core 的 `change_meta_at` 用引擎的顯示規則決定要不要帶 `readyAt`（目前至少一個任務而且全部勾完才帶），並帶出 `createdById`。遠端的單一變更資料取自 Server 的回應，欄位同名。

「就緒」和階段名「已就緒」同一個詞根，和「開工 YYYY-MM-DD」同一種寫法；不出現 `readyAt` 這類工程詞。

## Implementation Contract

**行為**

- 階段：

  | 清單項 | `changeStage` 的結果 |
  | --- | --- |
  | `stage: "ready"`、任務 1/3 | `ready`（以清單的值為準） |
  | 沒有 `stage`、任務 3/3 | `ready`（自己算） |
  | 沒有 `stage`、任務 0/3、有 `startedAt` | `in-progress` |
  | `stage: "paused"`（不認得的值）、任務 0/3、沒有 `startedAt` | `proposed`（自己算） |

- 建立者選單（卡片依清單順序）：

  | 卡片的 `createdBy`／`createdById` | 選單的選項 | 選第一個選項時留下 |
  | --- | --- | --- |
  | A：`Ann`／`u_1`；B：`Ann Lee`／`u_1` | `Ann` | A、B |
  | A：`Ann`／`u_1`；C：`Ann`／（無） | `Ann` | A、C |
  | A：`Ann`／`u_1`；D：`Ann`／`u_2`；C：`Ann`／（無） | 三個選項：兩個識別各一個（標籤後接識別的最後 4 個字元）、一個只有名稱的 `Ann` | A |
  | E：`Bob`／（無）；F：`Bob`／（無） | `Bob` | E、F |

- 最後更新：`modifiedAt` 是今天的時間時，卡片的 meta 列尾端顯示「今天」；3 天前顯示「3 天前」；缺席時沒有這一段。
- 就緒日：單一變更的資料帶 `readyAt: "2026-11-25"` 時，出身行在開工段之後顯示「就緒 2026-11-25」；缺席時沒有這一段，出身行其餘各段不變。

**介面與資料形狀**

- `ChangeItem` 新增選填欄位 `stage`、`createdById`、`modifiedAt`；`DiscussionItem` 新增 `createdById`、`modifiedAt`；`ChangeMetaInfo` 新增 `readyAt`、`createdById`。全部 camelCase、缺席即未知。
- desktop-core 的本機清單 payload 每項多 `stage` 與 `createdById`（沒有識別時是 null，和同一段程式疊的其他欄位一致）；`change_meta_at` 多 `readyAt` 與 `createdById`。
- `changeStage`、`matchesFilters` 的簽名不變；`BoardFilters.createdBy` 的型別不變、意思改成歸屬鍵。
- Tauri 殼的 command 不新增、不改名。

**失敗模式**

- 沒有新欄位：每一項都退回今天的行為，不顯示錯誤。
- `modifiedAt` 解析不出來：那一段不出現。
- `stage` 是不認得的值：照既有規則算。

**驗收**

- `npm test -w @speclink/ui` 的 `stage`、`search`、`kanban`、`discussionColumn`、`changeDetailPage` 測試涵蓋上面每一列。
- `cargo test -p speclink-desktop-core`：本機清單 payload 帶 `stage` 與 `createdById`、`change_meta_at` 帶 `readyAt` 的條件。
- `cargo test -p speclink-desktop --test it remote_data`：`change_stage` 有 `stage` 時用它、沒有時的結果與既有測試相同。

**範圍**

- 範圍內：D1–D4。
- 範圍外：Non-Goals 列的每一項；引擎、協定與 HTTP 回應（屬於前置變更）。

## Risks / Trade-offs

- [回歸對照：既有的看板與詳情頁測試] → `changeStage`、`matchesFilters` 的簽名不變；本機專案的資料沒有識別，建立者篩選的既有測試不改斷言就要通過。desktop-core 的清單 payload 多兩個鍵，釘住鍵集合的測試同批更新。CLI 的輸出與 render 的 golden 不受影響。
- [跨平台] → 相對時間用 JavaScript 的時間運算，日期與時間的提示用使用者的時區與語系格式化，三個平台的 WebView 行為相同；Rust 端只呼叫引擎的純函式，沒有路徑或換行的差異。
- [本機的最後更新偏新] → 檔案時間會被 checkout 或複製改到。沿用規格頁同樣的來源；討論的結論已把它列為延後處理。
- [階段的兩份規則仍在] → TypeScript 的退路只在清單沒有 `stage` 時生效；引擎之後改規則時，連到舊版 Server 的 Desktop 會顯示舊規則的結果。這是使用者已知並接受的限制。
- [合併沒有識別的卡片會認錯人] → 只在名稱對應到唯一一個識別時才合併；同名的兩個人各有識別時不合併。一個從來沒有識別的人和一個有識別的人同名時會被合併，機率低，而且只影響篩選結果，不改任何資料。
- [卡片的 meta 列變擠] → 新的一段在列尾、不換行，寬度不夠時由既有的溢出規則裁切；卡片高度不變。
- [前置變更沒有先做完] → 用 `speclink change depends` 記下相依；apply 開工先跑 `validate`，確認 delta 引用的正式規格需求還在。

## Migration Plan

不需要資料遷移。前置變更封存之後再開工。回滾時還原這個變更即可；資料形狀只有新增選填欄位，舊的前端會忽略它們。

## Open Questions

(none)
