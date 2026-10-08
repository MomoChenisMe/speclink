## Context

桌面 app 的前端狀態在 `apps/desktop/src/store.ts`（zustand），四種 detail 抽屜（變更詳情、討論、規格、已封存）的互斥由 `openDetail`／`openDiscussion`／`openSpec`／`openArchived` 四個開啟動作在同一次 set 內清除其他抽屜欄位來保證（spec「detail 抽屜互斥」）。對話框各自掛在獨立狀態上：

| 浮層 | 狀態位置 | 性質 |
| --- | --- | --- |
| 新增 Workspace（WorkspaceChooser） | store `workspaceChooser` | 尚未送出的選擇 |
| 初始化確認 | store `pendingInit` | 尚未送出的確認 |
| 啟用確認 | store `pendingAdopt` | 尚未送出的確認 |
| 封存確認與工單三選項框 | store `pendingArchive`＋`pendingArchiveSettled`（App.tsx 由此派生 `pendingArchiveStation`） | 尚未送出的確認 |
| 封存討論確認 | store `pendingArchiveDiscussion` | 尚未送出的確認 |
| 更新日誌 | `apps/desktop/src/App.tsx` 本地 state `releaseNotes` | 純檢視 |
| 遷移對話框 | store `migrationRoot` | 已開始的寫入流程 |
| 遠端衝突對話框 | store `pendingRemoteConflict` | 等待裁決的衝突寫入 |

系統匣（`apps/desktop/src/tray.ts` 的 `openIn`：先喚起主視窗再執行 store 動作）與看板卡片都呼叫同一組 store 動作，所以疊層在每個入口都會發生。這是討論 desktop-ui-redesign 的 cut 0，獨立於換皮，先修。

## Goals / Non-Goals

**Goals:**

- 任一可取消浮層開啟中，開啟變更詳情或討論抽屜時，該浮層關閉、抽屜開啟，畫面只有一個浮層。
- 保證落在狀態層的開啟動作，所有入口（看板、系統匣選單、面板、抽屜內跳轉）共用，呼叫端不先關再開。
- 不可取消的浮層（遷移、遠端衝突）開啟中，開啟動作不改任何狀態；系統匣的交接仍把主視窗帶到前景。

**Non-Goals:**

- 不把更新日誌搬進 store（spec「更新日誌彈窗」規定純 UI 開關）。
- 不做浮層佇列（遷移結束後自動開抽屜）。
- 不動 `openSpec`／`openArchived`（宿主頁是規格頁與已封存頁，系統匣沒有這兩個入口）。
- 不動系統匣面板與原生選單的呈現、不動 Rust 端（`apps/desktop/src-tauri`、`apps/desktop/core`）。

## Decisions

### D1 互斥擴到可取消浮層，仍由 store 的開啟動作保證

`openDetail` 與 `openDiscussion` 在既有的同一次 set 內，連同其他抽屜欄位一起把可取消浮層歸零。替代方案「系統匣交接時先關浮層」被否決：spec 明定互斥不得依賴呼叫端，且看板卡片入口同樣會疊層。

### D2 可取消與不可取消的判準

判準是「使用者按取消會不會丟掉已提交的工作」：尚未送出的確認或選擇（新增 Workspace、初始化、啟用、封存、封存討論、更新日誌）可取消，歸零等同使用者按「取消」，與 `cancelInit`／`cancelAdopt`／`cancelArchive`／`cancelArchiveDiscussion`／`closeWorkspaceChooser` 的寫入相同；已開始的寫入流程（遷移）與等待裁決的衝突寫入（遠端衝突）不可取消。`pendingArchiveSettled` 隨 `pendingArchive` 一起還原為初始值，避免下一次封存入口沿用上一輪的站別處置。

### D3 更新日誌對話框跟隨抽屜狀態關閉

`releaseNotes` 留在 App.tsx。App.tsx 以一個 effect 觀察 `detailChange`／`detailDiscussion` 自 null 變為非 null 的瞬間，將 `releaseNotes` 設為 null。開啟動作仍是單一真相；更新日誌是跟著狀態反應，不是呼叫端先關。替代方案「搬進 store」違反既有 spec；「系統匣特判」違反 D1。

### D4 不可取消浮層開啟中，開啟動作為 no-op

`openDetail`／`openDiscussion` 在 `migrationRoot` 或 `pendingRemoteConflict` 非 null 時直接返回、不呼叫 set。`openIn` 不變——主視窗照常喚起，使用者看到的是遷移或衝突對話框。替代方案「排隊、結束後自動開抽屜」多一個狀態機，使用者再點一次即可，不值得。

### D5 可取消浮層的歸零集中成一個片段

store 內加一個回傳部分狀態物件的純函式（命名依既有 snake／camel 慣例，例如 `cancelableOverlaysCleared()`），`openDetail` 與 `openDiscussion` 把它展開進同一次 set。之後新增可取消浮層只改這一處。

## Implementation Contract

**Behavior**

- 可取消浮層任一開啟中，呼叫 `openDetail(name)`（name 存在於 `changes`）或 `openDiscussion(slug)`（slug 存在於 active 或 archived 討論）：同一次 set 內設定自身抽屜欄位、`boardView` 切回 `"board"`、清除其他三個抽屜欄位與 `drawerVerb`，並把 `workspaceChooser`、`pendingInit`、`pendingAdopt`、`pendingArchive`、`pendingArchiveDiscussion` 設為 null、`pendingArchiveSettled` 設回 `{ review: false, verify: false }`。
- `releaseNotes` 非 null 且 `detailChange` 或 `detailDiscussion` 自 null 變為非 null：更新日誌對話框關閉；whatsNew 模式下不寫「已看過」記錄（與使用者按關閉不同，關閉鈕才記錄；下次啟動照舊彈出）。
- `migrationRoot` 或 `pendingRemoteConflict` 非 null：`openDetail`／`openDiscussion` 不改任何狀態（含不切 `boardView`）。
- `openSpec`／`openArchived`、`closeDetail`／`closeDiscussion` 行為不變。

**Interface / data shape**

- store 動作簽名不變：`openDetail(name: string): void`、`openDiscussion(slug: string): void`。
- 不新增對外事件、IPC、CLI 或 `--json` 欄位。

**Failure modes**

- name／slug 找不到：維持既有行為（不改狀態、不報錯）。
- 不可取消浮層開啟中：靜默 no-op，不 toast、不記錄。

**Acceptance criteria**

- `apps/desktop/src/__tests__/store.test.ts` 新增：（a）五種可取消浮層各開一個再 `openDetail`／`openDiscussion`，斷言浮層欄位歸零、抽屜開啟、`boardView` 為 board；（b）`migrationRoot` 與 `pendingRemoteConflict` 各非 null 時呼叫兩個動作，斷言整個 state 物件淺比較不變。
- `apps/desktop/src/__tests__/` 的 App 層測試新增：更新日誌開啟→store `openDetail`→對話框不可見。
- 既有「detail 抽屜互斥」四組轉移測試不變且通過。
- 手動：面板開「新增 Workspace」→系統匣點「開啟此變更」→只剩抽屜。

**Scope boundaries**

- In：`apps/desktop/src/store.ts` 兩個開啟動作與一個歸零片段、`apps/desktop/src/App.tsx` 一個 effect、對應測試、`desktop-app` delta spec。
- Out：系統匣、Rust 端、其他抽屜、浮層外觀、換皮相關任何改動。

## Risks / Trade-offs

- [使用者在封存三選項框選到一半，從系統匣開抽屜 → 選擇被取消] → 這與按「取消」相同，沒有任何寫入發生；spec 以 scenario 明示。
- [更新日誌 whatsNew 被抽屜頂掉後下次啟動又彈] → 刻意如此（沒按關閉就不算看過），design 明記，避免被當成 bug 回報。
- [回歸對照] → 不動 CLI 與 golden；前端只跑 `npm test -w apps/desktop`，既有互斥測試是回歸基準。
- [跨平台] → 純狀態邏輯，macOS／Windows／Linux 無差異；系統匣面板只在 macOS，但原生選單入口在三平台走同一條 store 動作。

## Migration Plan

無資料遷移。部署隨桌面 app 一般發版；回滾即還原兩個檔案。

## Open Questions

無。
