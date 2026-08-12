---
topic: desktop/tray 切換 workspace 與讀取時加上 skeleton／loading 的 UX 效果
slug: desktop-loading-skeleton-ux
status: promoted
promoted_to: desktop-loading-skeleton-ux
created: 2026-08-11
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: desktop/tray 切換 workspace 與讀取時加上 skeleton／loading 的 UX 效果

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

已封存的 2026-08-11-desktop-async-commands 把所有觸及檔案系統／子進程的 Tauri command 改為 async＋spawn_blocking，解掉整窗凍結，但其 Success Criteria 所述「資料以載入中狀態呈現」前端並未實作——store 的 `loaded` 旗標（apps/desktop/src/store.ts:158）有寫入但全 repo 零 UI 消費者。使用者在 tray 切換 workspace 時延遲感特別明顯，希望 desktop 與 tray 在切換與讀取時都有 skeleton／loading 回饋。

模式：assumptions——codebase scout 找到大量相關碼（store.ts 的 activateTab/refresh、tray.ts、panel/TrayPanel.tsx、packages/ui 元件群），足以先列假設再由使用者糾正。

相關 change／spec：來源 change 為已封存的 2026-08-11-desktop-async-commands（desktop-app spec 的 command 執行緒契約）；進行中的 spec-purpose-backfill 與本題無關。skeleton 基元候選落點 packages/ui/src/components/ui/（shadcn 基元群，現無 skeleton；apps/server-web 亦依賴 @speclink/ui）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-11)

**Focus**: 切換延遲的歸因與治法分工——五項假設全數獲使用者確認
**Position**: 延遲是兩段成因不同的等待，分開治；skeleton 只補「首訪無快取」的空窗。
- A 段（切換前）：activateTab 先 await openProject probe（store.ts:1824-1827，spawn 子進程、macOS 首抓可秒級），成功才翻 activeKey——期間畫面與 tray 分頁高亮完全不動，即 tray「點了沒反應」的主因
- B 段（切換後）：activeKey 翻轉即顯示該 workspace 舊快取、首訪為空清單（workspaceActivationState → visibleWorkspaceSnapshot，store.ts:528），refresh() 四路並發完成才有真資料；首訪空清單與「真的沒東西」無法區分
- 治法：A 段加「切換中」旗標做即時回饋（不改切換順序）；B 段 skeleton 條件＝既有 `!loaded`（store.ts:158/488/956），有舊快取則照現狀顯示舊資料、refresh 後靜默更新
- skeleton 基元落 packages/ui/src/components/ui/（shadcn Skeleton）；apps/server-web/package.json:13 亦依賴 @speclink/ui，web 端未來直接受益
- tray 原生選單做不了 skeleton（選單項僅能改字／停用），重點做 React 面板；「切換中／載入中」狀態進 TraySnapshot，維持面板薄渲染同源（TrayPanel.tsx:2、panel/main.tsx:28）
- 範圍擴充（使用者裁定）：抽屜內文件內文載入也納入 skeleton 範圍，不只清單層
**Ruled out**: 樂觀翻頁（probe 失敗的回滾與錯誤歸屬複雜化，現有 tabErrors 依賴不翻 activeKey——store.ts:1829-1833）；每次 refresh 都閃 skeleton（watcher 事件與切換共用 refresh，會頻繁閃爍）；原生選單載入動畫（平台能力做不到）；skeleton 基元放 apps/desktop（server-web 共用不到）
**Open**: 文件內文（DocumentViewer 等）的載入路徑現況與 skeleton 掛點；A 段「切換中」在各 surface 的具體呈現形式

### Round 2 — assumptions (2026-08-12)

**Focus**: 文件內文層的載入現況與 skeleton 掛點；A 段「切換中」的呈現形式
**Position**: 文件層已有「undefined＝載入中」三態慣例但消費不全——統一消費並把載入文字換成 skeleton；A 段回饋＝目標分頁小 spinner。
- Doc 三態慣例已存在：undefined＝載入中、null＝檔不存在、string＝內容（RichDetailDrawer.tsx:85、SpecDrawer.tsx:21、DiscussionDrawer.tsx:266）
- 消費不一致：proposal 分頁／spec 抽屜／已封存抽屜討論頁顯示 common.loading 文字（RichDetailDrawer.tsx:571、SpecDrawer.tsx:90、ArchivedDrawer.tsx:323）；design、tasks 等分頁把載入中直接渲染成「沒有文件」空態（RichDetailDrawer.tsx:574）——載入期間短暫呈現假的「無文件」
- 掛點：所有抽屜分頁統一消費 undefined 態，渲染文件 skeleton（數行灰條），取代載入文字與假空態
- A 段呈現：目標分頁上小 spinner——主視窗 ProjectTabs 與 tray 面板分頁條同款；原生選單不加（點擊即關、無處呈現回饋）
**Ruled out**: tray 圖示忙碌變體（YAGNI——分頁 spinner 已覆蓋回饋需求，圖示變體需多套圖資與狀態機）；載入文字改遍全站（既然引 skeleton，文字態一併汰換而非兩制並存）
**Open**: 無——進入結論

## Conclusion

**Decision**: desktop 與 tray 補上三層載入回饋——(1) A 段切換中：store 加「切換中分頁」旗標，主視窗 ProjectTabs 與 tray 面板分頁條在目標分頁顯示小 spinner，不改 activateTab 的 probe-先-翻頁順序；(2) B 段清單首訪：skeleton 條件＝既有 `!loaded`，看板欄與 tray 面板分區以佔位卡呈現，有舊快取則照現狀顯示舊資料、refresh 後靜默更新；(3) 文件內文：統一所有抽屜消費 Doc 三態的 undefined 載入態，渲染文件 skeleton，取代 common.loading 文字與假「無文件」空態。skeleton 基元（shadcn Skeleton）落 packages/ui/src/components/ui/；tray 面板的載入／切換狀態進 TraySnapshot 維持薄渲染同源。
**Rationale**: desktop-async-commands 解了整窗凍結但「載入中呈現」從未實作（loaded 旗標零 UI 消費者）；切換延遲是 A（probe 擋在翻頁前）、B（refresh 未完）兩段成因不同的等待，須分開給回饋；沿用既有狀態語意（loaded、Doc 三態）不建平行狀態，是最小且不易出同步 bug 的路。
**Rejected alternatives**: 樂觀翻頁（probe 失敗回滾與 tabErrors 錯誤歸屬複雜化）；每次 refresh 都閃 skeleton（watcher 與切換共用 refresh，頻繁閃爍）；原生選單載入動畫（平台能力不可行）；tray 圖示忙碌變體（YAGNI）；skeleton 基元放 apps/desktop（server-web 共用不到）。
**Deferred**: none
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion desktop-loading-skeleton-ux
