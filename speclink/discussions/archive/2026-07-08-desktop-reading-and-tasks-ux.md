---
topic: 桌面閱讀體驗與任務互動打磨
slug: desktop-reading-and-tasks-ux
status: promoted
promoted_to: desktop-reading-experience, desktop-task-interactions, desktop-specs-view
created: 2026-07-08
---

# Discussion: 桌面閱讀體驗與任務互動打磨

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者列出七項桌面 app 問題：①字體換 Noto Sans TC ②討論記錄的 HTML 註解顯示在抽屜 ③抽屜內容字級 16px ④markdown 段落/列表在抽屜塌成文字牆 ⑤任務分頁缺 Spectra 式工具列（全部已完成/下一個未完成/重置）⑥勾選任務卡頓 ⑦規格 nav 頁簽無功能（參考 Spectra 規格頁）。

模式：assumptions——相關程式碼充足（packages/ui 的 Markdown.tsx、TaskList.tsx、RichDetailDrawer.tsx、DiscussionDrawer.tsx；apps/desktop 的 index.css、App.tsx、store.ts、tauriDataSource.ts）。

相關變更：desktop-window-and-settings-polish（同為桌面 UI 打磨、進行中）、desktop-acp-agent（桌面互動深化、提案中）。規格資料管道（listSpecs/getSpecDocument/store.specs）已存在但無 View。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-08)

**Focus**: 七項問題的根因裁決與 markdown 樣式方案走向
**Position**: ①字體——index.css:74 的 font-family 把 Noto Sans TC 排最後、永遠輪不到，傾向 @fontsource 打包（+5-10 MB）而非只調順序；②HTML 註解——scaffold 錨點功能性存在不能刪，改在 Markdown.tsx 以 react-markdown skipHtml 統一過濾；③16px——現況 .markdown 14px（index.css:79）＋TaskList 13px（TaskList.tsx:66）兩處要動；④「無段落感」真根因＝Tailwind v4 preflight 重置 ul/ol 的 list-style，.markdown 只恢復 margin/padding 沒恢復 list-style（bullet 全滅）＋ p 段距 0.5em 過弱——proposal.md 原文段落與 bullet 皆正常，來源無罪；⑤任務工具列需後端批次指令（set_all_tasks 一次寫檔），前端迴圈 N 次寫檔＝卡頓放大版；⑥勾選卡頓＝受控 checkbox 要等 set_task_done→全量 refresh→抽屜五文件重載整條鏈才回饋，且 busy 鎖 pointer-events（TaskList.tsx:198），走樂觀更新＋不鎖指標；⑦規格頁——資料管道已在、缺 SpecsView，Spectra 式搜尋＋卡片展開，mtime 需 Rust 端補。使用者未逐項糾正，但把 ③④ 的樣式做法導向「有無現成套件可直接套」——手工維護 .markdown 這次踩的 list-style 坑即是論據。
**Open**: 字體打包 vs 只調順序；④ 是否還要 soft-break=換行（remark-breaks）；markdown 樣式套件選型；拆刀建議（閱讀體驗 1-4／任務互動 5-6／規格頁 7）；規格頁做到什麼程度。

### Round 2 — assumptions (2026-07-08)

**Focus**: markdown 樣式套件選型——查證使用者自家 wadpilot chat panel 的前例
**Position**: wadpilot 樣式層正是 @tailwindcss/typography（packages/web/package.json ^0.5.19）；StreamingText.tsx 的 PROSE_CLASSES 以 `prose prose-sm max-w-none` 為基底＋一串 prose-* modifier 覆寫＋`chat-prose` 掛鉤集中暗色規則於 app.css——與本討論提議的「typography 接管基礎排版＋薄覆寫」同型。渲染引擎不同：wadpilot 用 Streamdown（聊天串流需容忍未閉合 markdown、帶 caret 動畫），speclink 桌面抽屜是靜態完整文件，react-markdown 續用即可，Streamdown 無必要。字級差異：wadpilot 聊天用 prose-sm（14px，密度取捨），speclink 要 16px＝prose 出廠預設。
**Ruled out**: Streamdown 引擎移植——僅為串流場景而生，靜態文件用不到其複雜度。
**Open**: typography 方案定案確認；字體打包 vs 只調順序；soft-break 要不要；拆刀分群。

### Round 3 — assumptions (2026-07-08)

**Focus**: 字體與 markdown 樣式方案定案
**Position**: ①字體採打包——@fontsource/noto-sans-tc 隨 app 帶字體檔，font-family 改 Noto Sans TC 優先，離線與未安裝字體的機器皆生效，接受安裝包 +5-10 MB。②markdown 樣式採 @tailwindcss/typography——prose 接管基礎排版（出廠 16px 同時滿足字級需求）＋薄覆寫保留 Spectra 特調（teal code chip、任務清單懸掛縮排），渲染引擎續用 react-markdown。皆為使用者明示裁決。
**Ruled out**: 只調 font-family 順序（依賴使用者機器有無安裝，不確定）；github-markdown-css（自帶配色與 shadcn token 打架）；手工修補 .markdown（list-style 坑證明長期維護成本）；Streamdown 引擎（串流場景特化，靜態文件無需求）。
**Open**: soft-break 是否採 remark-breaks——新事證：discuss add-round 的記錄格式本身就以單換行分隔 Focus/Position/Ruled out/Open 各行，CommonMark 下塌成同一段，正是截圖文字牆的另一半成因；拆刀分群；規格頁範圍。

### Round 4 — assumptions (2026-07-08)

**Focus**: 殘餘開放點清零（soft-break、拆刀、規格頁範圍）
**Position**: 三項建議使用者全數同意：①remark-breaks 採用——單換行=硬換行；事證為 discuss 記錄格式自身以單換行分隔各行，不開修不掉文字牆另一半。②拆三刀——閱讀體驗（1-4，純前端）／任務互動（5-6，前端＋Rust 批次 IPC）／規格頁（7，新 View＋Rust mtime），互相獨立交付。③規格頁走 Spectra 基準款——搜尋＋卡片展開縮合＋prose 全文＋修改於 N 天前；requirement 計數 badge 與全螢幕抽屜首版不做。
**Ruled out**: 一刀全包（純樣式快改被 Rust 端進度拖住）；規格頁全套 Spectra 功能（範圍膨脹，需要再加）。
**Open**: 無——收斂，進 conclude。

## Conclusion

**Decision**: 七項桌面問題拆三刀處理。①閱讀體驗刀（desktop-reading-experience）：@fontsource/noto-sans-tc 打包、font-family 改 Noto Sans TC 優先；Markdown.tsx 開 skipHtml 過濾 HTML 註解；@tailwindcss/typography 的 prose 取代手工 .markdown（出廠 16px 即滿足字級）＋薄覆寫（teal code chip、GFM 任務清單懸掛縮排、--tw-prose-* 映射既有 oklch token 使深色自動跟隨）；remark-breaks 使單換行=硬換行；TaskList 任務文字 13px→16px 對齊。②任務互動刀（desktop-task-interactions）：TaskList 頂部 Spectra 式工具列——全部已完成／下一個未完成（n 快捷鍵）／重置任務，Rust 端新增批次 IPC（一次寫檔）；勾選改樂觀更新——UI 先翻轉、寫回失敗回滾，busy 不再鎖 pointer-events，refresh 世代單一資料流保留。③規格頁刀（desktop-specs-view）：SpecsView 基準款——搜尋列＋規格卡片展開縮合＋prose 全文渲染＋複製名稱＋「修改於 N 天前」（Rust list_specs 補 mtime），boardView 加 specs。
**Rationale**: 排版交給官方 typography 插件而非手工維護——本次 list-style 被 Tailwind preflight 重置的坑即手工成本的實證，且使用者自家 wadpilot chat panel 同款方案已驗證；三刀拆分讓純樣式快改不被 Rust 端進度拖住，各刀獨立交付驗證。
**Rejected alternatives**: 只調 font-family 順序（依賴機器有無安裝字體）；github-markdown-css（自帶配色與 shadcn token 打架）；手工修補 .markdown（維護成本實證）；Streamdown 引擎（串流特化，靜態文件無需求）；改 CLI scaffold 移除 HTML 註解（agent/CLI 功能性錨點）；前端迴圈實作批次勾選（N 次寫檔＝卡頓放大）；一刀全包。
**Deferred**: 規格頁進階功能（requirement 計數 badge、全螢幕抽屜）——需要再加。
**Capture to**: proposal（三個變更）
**Next**: speclink discuss promote desktop-reading-and-tasks-ux --name desktop-reading-experience / desktop-task-interactions / desktop-specs-view
