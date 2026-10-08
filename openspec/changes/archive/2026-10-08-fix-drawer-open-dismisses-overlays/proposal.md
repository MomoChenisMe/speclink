## Problem

桌面 app（apps/desktop）的使用者——透過 AI 代理跑 SDD 的開發者／PO／PM——在主視窗開著「新增 Workspace」對話框（或更新日誌、初始化／啟用／封存／刪除／退回確認框）時，從系統匣面板或原生選單點「開啟此變更」「開啟此討論」，變更詳情抽屜或討論抽屜會疊在對話框上面：對話框沒關、抽屜也開了，兩個浮層同時可見、焦點與遮罩互相打架。對應 workflow 階段：apply 與 archive 之間的看板操作、系統匣的「開啟此變更／開啟此討論」快捷動作。

## Root Cause

狀態層的互斥只涵蓋四種 detail 抽屜。`apps/desktop/src/store.ts` 的 `openDetail` 與 `openDiscussion` 在設定自身選定狀態時只清除其他三個抽屜欄位與 `drawerVerb`；對話框各自掛在獨立狀態上——`workspaceChooser`、`pendingInit`、`pendingAdopt`、`pendingArchive`（含其衍生的工單三選項框）、`pendingArchiveDiscussion`、`pendingDelete`、`pendingRevert`、`revertBlocked` 在 store，`releaseNotes` 在 `apps/desktop/src/App.tsx` 的本地 state——沒有任何開啟動作會碰它們。系統匣的交接（`apps/desktop/src/tray.ts` 的 `openIn`）直接呼叫同一組 store 動作，所以不管入口是看板還是系統匣，結果都一樣。

## Proposed Solution

把「開啟抽屜時收掉可取消浮層」做成狀態層開啟動作的保證，與既有的 detail 抽屜互斥同一條規則：

- `openDetail` 與 `openDiscussion` 在同一次 set 內，連同清除其他抽屜欄位，一併把可取消浮層歸零：`workspaceChooser`、`pendingInit`、`pendingAdopt`、`pendingArchive`（連帶 `pendingArchiveSettled` 等衍生欄位）、`pendingArchiveDiscussion`、`pendingDelete`、`pendingRevert`、`revertBlocked`。取消語意與使用者按「取消」相同——這些對話框都是尚未送出的確認、選擇或純說明，沒有任何已提交的工作被中斷。刪除與退回確認從變更詳情抽屜內開啟，不收的話換抽屜後「刪除 A？」會疊在 B 的抽屜上，比原問題更容易按錯。
- `releaseNotes` 是 App.tsx 的純 UI 開關（spec「更新日誌彈窗」規定不進 store）：App.tsx 以一個 effect 觀察開著的是哪個變更詳情或討論抽屜，抽屜開啟或換開另一個時即關閉更新日誌對話框。開啟動作仍是單一真相，更新日誌只是跟著反應，呼叫端不必先關再開。
- 遷移進行中（`migrationRoot` 非 null）或遠端衝突等待裁決（`pendingRemoteConflict` 非 null）不收：遷移是已開始的寫入流程，遠端衝突是等待裁決的衝突寫入，都不可取消。此時 `openDetail`／`openDiscussion` 不改變任何狀態；系統匣的交接仍把主視窗帶到前景，使用者看到的是遷移或衝突對話框。
- 已送出的封存守門處置（三選項框按下「放棄」後、CLI 刪工單途中）不被收掉中斷：處置照按下當下的快照走完，沒有下一站就照常封存。
- `openSpec` 與 `openArchived` 不在此列：它們的宿主頁是規格頁與已封存頁，系統匣沒有這兩個入口，且既有行為不切頁。

不新增或變更任何 CLI 指令、設定欄位、技能或 Agent 指令；不影響 `speclink-core` 與其他 crate。

## Non-Goals

- 不把 `releaseNotes` 搬進 store（spec 明定純 UI 開關）。
- 不做浮層佇列（例如遷移結束後自動開抽屜）——遷移中的系統匣點擊視為「帶到前景」即可。
- 不改系統匣面板或原生選單的任何呈現。
- 不動 `openSpec`／`openArchived` 的互斥範圍。

## Success Criteria

- 「新增 Workspace」對話框開啟中，自系統匣面板點「開啟此變更」：對話框關閉、變更詳情抽屜開啟、看板為底層，畫面上只有一個浮層。自原生選單與看板卡片開啟結果相同。
- 更新日誌、初始化確認、啟用確認、封存確認（含工單三選項框）、封存討論確認、刪除確認、退回確認、退回被擋說明任一開啟中，自系統匣開啟變更或討論：該對話框關閉、抽屜開啟。
- 遷移對話框或遠端衝突對話框開啟中，自系統匣點「開啟此變更」：該對話框維持、抽屜不開、主視窗到前景。
- 既有 detail 抽屜互斥的四組狀態轉移不變；`apps/desktop/src/__tests__/store.test.ts` 既有的互斥測試全部通過，並新增「open 動作清除可取消浮層」「遷移中 open 動作不改狀態」兩組測試。
- `speclink validate` 通過，`npm test -w apps/desktop` 通過。

## Impact

- Affected specs: `desktop-app`（MODIFIED「detail 抽屜互斥」——互斥範圍自抽屜之間擴及可取消浮層，並定義遷移中的例外）
- Affected code:
  - Modified: `apps/desktop/src/store.ts`（`openDetail`、`openDiscussion` 清除可取消浮層；遷移或遠端衝突中提前返回；封存守門處置改用按下當下的快照）
  - Modified: `apps/desktop/src/App.tsx`（更新日誌對話框跟隨抽屜開啟而關閉的 effect）
  - Modified: `apps/desktop/src/__tests__/store.test.ts`（新增兩組測試）
  - Modified: `apps/desktop/src/__tests__/App.test.tsx`（新增更新日誌跟隨抽屜關閉的案例）
  - New: （無）
  - Removed: （無）
