---
topic: tray 切換遠端專案分頁無反應
slug: tray-remote-project-switch
status: promoted
promoted_to: tray-remote-project-switch
created: 2026-07-20
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: tray 切換遠端專案分頁無反應

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者回報：macOS 系統匣面板的專案 tab 條上，點擊 remote 專案分頁（截圖中「Demo 專案/back…」）完全沒有反應，無法切換過去。討論以 assumptions 模式進行（掃到 apps/desktop/src/tray.ts、panel/TrayPanel.tsx、panel/main.tsx、store.ts、session.ts 共 5 個相關原始檔，脈絡充足）。相關規格：tray-status-menu（「選單專案切換」「面板樣式（macOS）」）、workspace-session（locator key 識別契約）。進行中變更 server-scope-read-api 的 artifacts 全文未涉 tray，與本題無重疊。介面深度檢查跳過：不新增模組／IPC／儲存抽象，重用既有 store 動詞與事件通道。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-20)

**Focus**: 點擊 tray 面板的 remote 專案分頁為何靜默無反應，以及修法與範圍
**Position**: 根因是 tray 快照把 remote 分頁的 root 設為空字串，切換動作被 falsy guard 靜默吃掉；修法為 tray 切換改以 locator key 走 store.activateTab。使用者確認全部假設：
- 根因鏈：TrayPanel.tsx:228 以 `onOpenProject(tab.root)` 發動作 → tray.ts:242 remote 分頁 `root: ""`（註明「remote 本刀無建構路徑」——tray 建置時 remote 僅存在於型別，屬已知缺口如期引爆，非回歸）→ tray.ts:381 `if (kind === "open-project" && id)` 空字串 falsy → 靜默落空
- 修法：面板與原生選單的專案切換一律改走 `activateTab(locatorKey)`（store.ts:683 已完整處理 remote：有 session 直切、重啟後重走 handshake；local：目錄失效轉 tabErrors）；tray 模型每個 tab 本已攜帶 key（tray.ts:127）；`TraySnapshot.root` 欄位隨之退場
- 規格依據：workspace-session spec 明寫「tray 選單識別 SHALL 一律經 locator key，SHALL NOT 再以裸 root 字串比對」——切換動作用 root 是殘留違例；tray-status-menu「選單專案切換」需求本無 remote 例外，修好即回歸規格字面
- 原生選單（Windows/Linux）同根因（tray.ts:299 `openProjectAt("")` 拋錯吐 toast 於可能隱藏的主視窗）、一併修
- 範圍：獨立小變更（bug-fix delta 掛 tray-status-menu spec），不併入進行中的 server-scope-read-api
**Ruled out**: 只為 remote 加分支、local 續走 openProjectAt——形成兩條切換路徑，且 openProjectAt 的 pendingInit 初始化對話框語意不該出現在 tray 切既有分頁；tray 內新增切換失敗的錯誤 UI——使用者裁定失敗靜默可接受（tabErrors 由看板呈現，沿用 spec 既有語意，面板維持薄渲染層定位）
**Open**: 無——待 propose 落地任務與測試（含真實視窗驗證）

## Conclusion

**Decision**: tray 的專案切換動作（macOS 面板 tab 條與非 macOS 原生選單皆然）改以 locator key 呼叫 store 既有的 `activateTab(key)`，取代現行 `openProjectAt(root)`；`TraySnapshot.root` 欄位退場。修復後點擊 remote 專案分頁即完成原地切換（有 session 直切、重啟後重走 handshake），local 分頁行為不變。
**Rationale**: 根因是 tray 建置時 remote 分頁尚不存在，快照將 remote root 設空字串、被面板 handler 的 falsy guard 靜默吃掉。`activateTab` 已完整承載兩型分頁的切換語意，且 workspace-session spec 本就要求 tray 一律經 locator key 識別——此修法同時消除殘留違例，是單一路徑、最小改動的解。
**Rejected alternatives**: (1) 只為 remote 加分支、local 續走 openProjectAt——兩條切換路徑會語意分歧，且 openProjectAt 的 pendingInit 對話框語意不屬於 tray 切既有分頁；(2) tray 內新增切換失敗錯誤 UI——使用者裁定失敗靜默可接受，錯誤沿用看板 tabErrors 呈現（spec 既有語意），面板維持薄渲染層。
**Deferred**: 無
**Capture to**: proposal（新變更；delta 掛 tray-status-menu spec）
**Next**: /speclink-propose --from-discussion tray-remote-project-switch
