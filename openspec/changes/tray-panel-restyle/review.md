# Review — tray-panel-restyle

## Round 1

**Phase**: discovery
**Patch**: sha256:3a0a7293738dd8167b711c824eeca1408f43ef47a19974e1ee9cffba76c254ea
**Scope**: apps/desktop/src/__tests__/trayPanel.test.tsx, apps/desktop/src/components/ProjectRail.tsx, apps/desktop/src/components/projectTile.ts, apps/desktop/src/panel/TrayPanel.tsx, apps/desktop/src/panel/promotedExpanded.ts, packages/ui/src/__tests__/copyButton.test.tsx, packages/ui/src/components/CopyButton.tsx
- [WARNING] apps/desktop/src/__tests__/trayPanel.test.tsx — Correctness: 「localStorage 不可用時讀回收合、寫入不拋錯」這題其實沒測到東西。它攔的是 `vi.spyOn(Storage.prototype, ...)`，但 vitest.setup.ts 掛上的全域 localStorage 來自另一個 JSDOM，原型不是同一個，所以攔不到。我用 node 實測過：兩個 JSDOM 的 Storage.prototype 不相等，改其中一個不會影響另一個。App.test.tsx:2186 的註解也記錄了同一個坑。結果是 beforeEach 已清掉鍵，讀回 false；寫入真的寫進 "true"，也不會拋錯，所以測試永遠綠。promotedExpanded.ts 的兩個 catch 分支完全沒被執行。修法：改攔 `Object.getPrototypeOf(localStorage)`，或直接 `vi.spyOn(localStorage, "getItem"/"setItem")`。
- [WARNING] apps/desktop/src/panel/TrayPanel.tsx — Correctness: 專案 tab 的寬度從原本依內容伸縮改成固定 `w-16`，專案名的上限也從 `max-w-24`（96px）縮成 `max-w-14`（56px）。11px 字約只放得下 8～9 個英文字，常見的名稱（如 `speclink-desktop`、`Demo/backend`）都會被截成「…」，而且 tab 沒有 `title`，看不到完整名稱。原本這些名稱放得下。spec 要求每個 tab 顯示「其下的專案名」，名稱一長就辨認不出是哪個專案。
- [SUGGESTION] apps/desktop/src/panel/TrayPanel.tsx — Correctness: `panel-promoted-toggle` 是 `role="button"`、`tabIndex={-1}` 的 div，只綁了 `onClick`，沒有 `onKeyDown`。旁邊真正的 `<button>`（複製鈕、復原卡按鈕）都能用 Enter／Space 觸發，只有這一列不行。輔助技術使用者看到的是一顆按鈕，卻沒辦法用鍵盤操作。其他列（變更列、討論列、溢出列）也一樣，但它們沒有宣告 `role="button"`。
- [SUGGESTION] apps/desktop/src/panel/TrayPanel.tsx — Correctness: ChangeRow 的 `cn(rowClass, blockedBy.length > 0 && "opacity-60")` 會讓整列變淡，連 rowClass 新加的分隔線 `border-b` 也一起變淡。結果是被擋的變更列下方那條線比其他列淡。只影響外觀。
- [SUGGESTION] apps/desktop/src/panel/TrayPanel.tsx — Standards: 這次改寫 `rowClass` 時還留著 `group`，但反白用的 group-hover 已全部拿掉，列裡也沒用到 REVEAL_ON_HOVER，這個 class 已經沒有用處。依 CLAUDE.md「因本次改動而孤兒化的要清掉」，建議一起刪。條文只點名 import、變數、函式，class 不在內，所以不列 CRITICAL。
- [SUGGESTION] apps/desktop/src/components/ProjectRail.tsx — Standards: possible Middle Man。`export { RAIL_SQUARE } from "./projectTile"` 只是為了讓 UpdateRailButton 和它的測試沿用舊的匯入路徑，把這兩處的 import 改掉就能省掉這層轉手。
- [SUGGESTION] apps/desktop/src/panel/promotedExpanded.ts — Standards: possible Duplicated Code。讀寫加 try/catch 的寫法和 store.ts 的 readPromotedExpanded／setPromotedExpanded 幾乎一樣，只差鍵名。面板是獨立 bundle，分開寫說得過去，記一筆就好。
- [SUGGESTION] apps/desktop/src/panel/TrayPanel.tsx — Standards: possible Middle Man。拿掉 inverted 後，RowCopyButton 只剩固定 tabIndex=-1 和把 text 改名成 value。
- [SUGGESTION] apps/desktop/src/__tests__/trayPanel.test.tsx — Standards: possible Duplicated Code。同一個 describe 裡兩題各寫了一次一樣的 `header` helper，`follows` 也多了一份，現在共三份。
- [SUGGESTION] apps/desktop/src/panel/TrayPanel.tsx — Standards: possible Duplicated Code——`<DiscussionRow key={d.slug} d={d} onOpen={onOpenDiscussion} onCopy={onCopy} copyLabel={t("tray.copySlug")} />` 在討論中列與 `panel-promoted-rows` 展開區兩處 `OverflowGroup` 的 `rows` 中逐字重複；可在 `TrayPanel` 內宣告 `const discussionRow = (d) => <DiscussionRow … />` 後兩處 `.map(discussionRow)`。
- [SUGGESTION] apps/desktop/src/panel/TrayPanel.tsx — Standards: possible Duplicated Code——專案 tab 與「加入專案」兩個容器 class 幾乎相同（`flex w-16 shrink-0 flex-col items-center gap-1 rounded-lg px-1 py-1.5 hover:bg-foreground/5` 對 `group flex min-w-16 shrink-0 flex-col items-center gap-1 rounded-lg px-1 py-1.5 hover:bg-foreground/5`），兩者應同步調整的間距／hover 分散兩處；可抽 `const TAB_ITEM = "shrink-0 flex-col items-center gap-1 rounded-lg px-1 py-1.5 hover:bg-foreground/5"` 共用。

## Round 2

**Phase**: validation
**Patch**: sha256:fd2f3f0cb0e13588ac94b395aaece3ef581ad33978c31f946f242f905083fdd9
**Scope**: apps/desktop/src/__tests__/trayPanel.test.tsx, apps/desktop/src/__tests__/updateRailButton.test.tsx, apps/desktop/src/components/ProjectRail.tsx, apps/desktop/src/components/UpdateRailButton.tsx, apps/desktop/src/panel/TrayPanel.tsx
- [SUGGESTION] apps/desktop/src/panel/promotedExpanded.ts — Standards: possible Duplicated Code。讀寫加 try/catch 的寫法和 store.ts 的 readPromotedExpanded／setPromotedExpanded 幾乎一樣，只差鍵名。面板是獨立 bundle，分開寫說得過去，記一筆就好。
