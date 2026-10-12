# Review — desktop-spec-manual-pages

## Round 1

**Phase**: discovery
**Patch**: sha256:ddf8fc3e4d4b20582f2856499303c8abdb3ef193656216912c1c3e62418df36a
**Scope**: apps/desktop/core/src/manual.rs, apps/desktop/src-tauri/src/lib.rs, apps/desktop/src/App.tsx, apps/desktop/src/__tests__/App.test.tsx, apps/desktop/src/__tests__/fsActions.test.ts, apps/desktop/src/adapter/fsActions.ts, packages/ui/src/__tests__/deltaSpecView.test.tsx, packages/ui/src/__tests__/manualPage.test.tsx, packages/ui/src/__tests__/specDetailPage.test.tsx, packages/ui/src/__tests__/specSections.test.ts, packages/ui/src/__tests__/specSummary.test.ts, packages/ui/src/__tests__/ticketView.test.tsx, packages/ui/src/components/DetailHeader.tsx, packages/ui/src/components/ManualPage.tsx, packages/ui/src/components/ManualTree.tsx, packages/ui/src/components/Markdown.tsx, packages/ui/src/components/RequirementBlock.tsx, packages/ui/src/components/SpecDetailPage.tsx, packages/ui/src/components/TicketView.tsx, packages/ui/src/i18n.tsx, packages/ui/src/index.ts, packages/ui/src/manualDoc.ts, packages/ui/src/specSections.ts, packages/ui/src/specSummary.ts

- [WARNING] packages/ui/src/components/ManualPage.tsx — Correctness: 過期提示框的 `staleSources` 只用 slug 當鍵，切換專案時沒有清掉。讀內文的 effect 在 `!currentSlug` 時會 `setDoc(null)` 並作廢在途載入（註解還特別寫了「新 workspace 首頁多半同名」要防這件事），但新增的 stale effect 在 `!staleSlug` 時只遞增 `staleSeq`，沒有 `setStaleSources(null)`。觸發路徑：在專案 A 看過一頁過期頁（例如 `tray`）→ 切到專案 B（`s.manual` 先變 null，再載入 B 的索引）→ 在 B 開同名而且也過期的頁。B 的內文通常比 N 份規格先讀完，`staleLatest = staleSources.latest`（slug 相同）就會先把 A 專案的出處日期和變更名畫進提示框，要等 B 的 `getSpecDocument` 全部回來才換掉。建議在 `!staleSlug` 時（至少在索引清空時）一併清空 `staleSources`。
- [WARNING] packages/ui/src/components/SpecDetailPage.tsx — Correctness: `docPath` 寫死成 `` `openspec/specs/${capability}/spec.md` ``，並且代進 `sdetail.purposeTbdBody`，告訴使用者「直接編輯 `{path}`」。可是這次 change 的 Rust 測試明確支援自訂 `spec_dir`（`spec_dir: docs/specs` 時路徑是 `docs/specs/...`）。觸發路徑：專案的 `.speclink.yaml` 設了 `spec_dir`，規格的 Purpose 還是佔位文字 → 提示框指示去改一個不存在的路徑，出處行顯示的路徑也是錯的。「開啟 ▾」走 core 解析，所以不受影響。
- [SUGGESTION] packages/ui/src/components/ManualPage.tsx — Correctness: 出處行也寫死 `` `openspec/manual/${current.slug}.md` ``，自訂 `spec_dir` 時會和「開啟 ▾」選單首列顯示的 `location.relative` 不一致。可以改顯示 core 解析後的相對路徑，或者至少不要寫死根目錄名。
- [SUGGESTION] packages/ui/src/components/RequirementBlock.tsx — Correctness: 收合模式下，收起的需求段不會渲染內文，所以內文裡的 `.markdown h2/h3` 在 DOM 裡不存在。展開後這些標題才出現，但 SpecDetailPage 的 `tocKey`（tab＋doc＋needle）不會因此改變，本頁目錄不會重掃，展開後新出現的子標題不會進目錄。實際上很少見（需求內文極少帶 `##`／`###`），列作強化建議。
- [SUGGESTION] packages/ui/src/components/SpecDetailPage.tsx — Standards: 可能有重複程式碼（Duplicated Code）。`const calendarDay = (updated: string) => updated.slice(0, 10)` 和 specSummary.ts 裡沒匯出的同名函式一字不差，ManualPage.tsx 又直接寫了一次 `latest.updated.slice(0, 10)`。「只取日期部分」這條規則散在三個地方，建議由 specSummary.ts 匯出一份大家共用。
- [SUGGESTION] packages/ui/src/specSummary.ts — Standards: 可能是用不到的彈性（Speculative Generality，這裡是參數收得比實際需要的寬）。`specStats(sections, sources)` 只用到 `sources.length`，卻要求傳入整個 `{ name; archived? }[]` 陣列。結果 ManualPage 只好傳一個空陣列湊數：`specStats(parseSpecSections(md), []).latest`。改成收 `sourceCount: number`，或另外抽一個只算最近時間點的函式給手冊頁用，會比較直白。
- [SUGGESTION] packages/ui/src/components/ManualPage.tsx — Standards: 可能是用基本型別硬撐（Primitive Obsession）。`staleCaps = sourceNames.join("\n")` 先把陣列串成字串當 effect 的依賴，effect 裡再 `split("\n").filter(Boolean)` 拆回來。這招背後有個沒寫出來的前提：capability 名稱裡不會有換行，旁邊的註解也沒交代為什麼要這樣做。建議補一句註解，或改用 memo 起來的穩定陣列。
- [SUGGESTION] packages/ui/src/index.ts — Standards: 可能是用不到的彈性（Speculative Generality）。這次新增的公開匯出 `specStats`、`specTiles`、`traceTiles`、`PURPOSE_TBD_PREFIX`、`countHeadings`，在 apps/ 底下沒有任何地方用到，測試也是直接 import 模組路徑。CLAUDE.md 寫明「不加沒被要求的彈性」；雖然 design D10 有指定要匯出，還是可以考慮等真的有外部使用者再開。
- [SUGGESTION] packages/ui/src/specSections.ts — Standards: 可能有跨語言的重複程式碼（Duplicated Code）。新增的 `PURPOSE_TBD_PREFIX` 加上 specSummary 裡的 `purposeTbd` 判斷，等於在前端重做一次 Rust 端 `apps/desktop/core/src/query.rs` 已經替規格清單算好的 `purposeTbd`。對齊測試能擋住兩邊字串漂移，但同一條規則還是有兩份實作。design D4 已經選了這條路，這裡只是提醒維護成本。
- [SUGGESTION] packages/ui/src/components/ManualTree.tsx — Standards: 可能造成散彈式修改（Shotgun Surgery，改一處就得連帶改好幾處）。這次把 `h-8` 改成 `h-[30px]`，等於手動照抄 `ui/tabs.tsx` 文件樹節點的 class 字串（`h-[30px] … rounded-lg px-2.5 text-[13px] … hover:bg-foreground/5`，作用中為 `bg-primary/12`）來做到「同款」。以後調整文件樹節點樣式就得兩邊一起改；NavItem 是原本就有的第三份，不在這次範圍內。

## Round 2

**Phase**: validation
**Patch**: sha256:a6316ecaac3f023d0028eb16095def59193fa8ef2112839af1323020f43a4ed7
**Scope**: apps/desktop/src/App.tsx, apps/desktop/src/__tests__/App.test.tsx, packages/ui/src/__tests__/manualPage.test.tsx, packages/ui/src/__tests__/specDetailPage.test.tsx, packages/ui/src/__tests__/specSummary.test.ts, packages/ui/src/components/ManualPage.tsx, packages/ui/src/components/ManualTree.tsx, packages/ui/src/components/RequirementBlock.tsx, packages/ui/src/components/SpecDetailPage.tsx, packages/ui/src/components/ui/tabs.tsx, packages/ui/src/i18n.tsx, packages/ui/src/index.ts, packages/ui/src/specSummary.ts

- [WARNING] packages/ui/src/components/SpecDetailPage.tsx — Correctness: 把 `toggles` 放進 `tocKey` 後，每次展開或收合需求段，ManualToc 都會重跑掃描 effect，而那個 effect 會無條件執行 `setActiveId(heads[0]?.id)`，把「目前段」的高亮重設到第一項。觸發方式：捲到頁面中段（例如第 8 條需求），點開它或收起它 → 右欄目錄的高亮跳回 Purpose。展開或收合不會改變 scrollTop，所以不會觸發 scroll 事件；WKWebView 也沒有 scroll anchoring，不會順手修正。高亮會一直停在錯的地方，直到使用者下一次捲動，違反 spec 要求的「捲動時 SHALL 高亮目前段」。以前只有搜尋或換文件時才會重設，現在變成每次點需求列都會發生。建議重掃後依目前捲動位置算出作用中的標題，不要直接退回第一項。
- [SUGGESTION] packages/ui/src/specSections.ts — Standards: 可能有跨語言的重複程式碼（Duplicated Code）。新增的 `PURPOSE_TBD_PREFIX` 加上 specSummary 裡的 `purposeTbd` 判斷，等於在前端重做一次 Rust 端 `apps/desktop/core/src/query.rs` 已經替規格清單算好的 `purposeTbd`。對齊測試能擋住兩邊字串漂移，但同一條規則還是有兩份實作。design D4 已經選了這條路，這裡只是提醒維護成本。

## Round 3

**Phase**: validation
**Patch**: sha256:7f0715e1eea90cb09799cc982959e1c7c67a5cf28f12a54b5d9f90d9f0be0e3c
**Scope**: packages/ui/src/__tests__/specDetailPage.test.tsx, packages/ui/src/components/ManualToc.tsx

- [SUGGESTION] packages/ui/src/specSections.ts — Standards: 可能有跨語言的重複程式碼（Duplicated Code）。新增的 `PURPOSE_TBD_PREFIX` 加上 specSummary 裡的 `purposeTbd` 判斷，等於在前端重做一次 Rust 端 `apps/desktop/core/src/query.rs` 已經替規格清單算好的 `purposeTbd`。對齊測試能擋住兩邊字串漂移，但同一條規則還是有兩份實作。design D4 已經選了這條路，這裡只是提醒維護成本。
