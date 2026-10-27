---
topic: 專案選擇對齊 Spectra
slug: 專案選擇對齊-spectra
status: concluded
created: 2026-07-06
---

# Discussion: 專案選擇對齊 Spectra

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：使用者希望 desktop-config-multiproject 的專案選擇功能對齊 Spectra 桌面 app 的做法，並要求以 computer use 實測 Spectra 畫面。

實測（Spectra GUI，C:/Users/momoc/AppData/Local/Spectra/app.exe，v2.3.1）：①視窗頂部為「專案分頁列」——mbitek/wadpilot/speclink 三個 tab，跨啟動持久化（啟動即還原上次的 tabs）、各 tab 有關閉鈕、右上「開啟專案」按鈕新增專案；②點 tab 即切換活躍專案，內容（統計卡、專案設定）立即載入；③tab 帶數字徽章＝該專案進行中變更數（wadpilot 徽章 2 與其儀表板「2 進行中」一致）；④每專案有儀表板（進行中/規格/已封存統計）與專案設定（config.yaml 的專案說明/產出規則，可編輯）。附帶觀察（反面教材）：Spectra 對「舊版格式」專案於切入 tab 時彈「升級至 Spectra 格式」對話框並主動寫檔——實測誤觸「立即升級」造成本 repo openspec/config.yaml 被剝走 locale/tdd/audit 三行（已以 git checkout 還原）、寫入 .spectra.yaml 與 .spectra/——onboarding 主動寫檔＋確認按鈕相鄰是危險設計。

模式：assumptions——desktop-config-multiproject 的 proposal/design/tasks 已完整（D1 執行期可變 root、D3 開啟三態流程、D8 localStorage 最近清單），Spectra 目標形態已實測。

相關 changes/specs：desktop-config-multiproject（0/28，proposal 明列 Non-Goal「多視窗或同時開多個專案——一次一個專案 root」；specs/desktop-config 有「最近開啟清單存於 app 本機」需求）；desktop-app spec（啟動語境、外部變更即時反映）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-06)

**Focus**: Spectra 專案選擇的實際模型為何、對齊它需要動 desktop-config-multiproject 的哪些層
**Position**: Spectra 模型＝「持久化專案分頁列＋點擊即切＋進行中數徽章」。對齊方向定為 UI 形態對齊而非後端多活躍 context：D1（Mutex<PathBuf> 單一活躍 root）與 D3（開啟三態流程）不動，tabs 是「最近開啟清單」（D8）的形態升級——localStorage 存 tabs（路徑＋順序＋最後活躍），啟動還原、關 tab 即移除、「開啟專案」新增 tab。徽章：啟動時對各 tab 專案以既有 list_changes_at(root) 輕掃一次計數，活躍 tab 即時（watcher）、背景 tab 保留最後已知值——desktop-core 查詢層本就逐呼叫收 root，只需一個吃 path 參數的唯讀 Tauri command，無新架構縫。
**Ruled out**: 多 root 同時活躍（per-tab watcher、per-root cache、store 分 tab）——單視窗一次只呈現一個專案，Spectra 外觀上也如此；範圍爆炸且與 proposal Non-Goal 衝突，若未來真要背景即時徽章再另開刀。學 Spectra 的格式升級式 onboarding（切入 tab 即彈寫檔對話框）——實測誤觸直接改壞使用者 repo，config-multiproject 既有「未初始化經確認才 init」流程優於它，維持。
**Open**: tabs 與「最近開啟清單」是否完全合一（關掉的 tab 是否還要出現在某個「最近」選單）？背景 tab 徽章的過期容忍度（啟動掃一次夠嗎）？tab 數量上限與溢出行為（Spectra 未觀察到；D8 原設計上限 10）？

### Round 2 — assumptions (2026-07-06)

**Focus**: 基於 Spectra 實測，speclink 桌面的 UI/UX 改善機會與取捨
**Position**: 分三類。①專案選擇本體的細節優化（納入本刀）：「＋」新增鈕直接掛在分頁列尾端（瀏覽器心智模型）而非只有右上按鈕、tab 徽章加 tooltip（「N 個進行中變更」）、Ctrl+Tab／Ctrl+1..9 切 tab、零 tab 首啟動給「開啟專案」空狀態引導頁（speclink 現況是空看板，比 Spectra 更需要此頁因為我們沒有預設 tabs）、消失路徑的 tab 以錯誤態呈現且一鍵關閉。②借鏡但另行處理：看板搜尋列（active changes 已 9 張卡、已封存頁已有 query 基建，成本低——可塞本刀或下一刀）；快捷鍵系統＋側欄快捷鍵提示（Spectra 側欄列 Ctrl+E 等，質感關鍵，獨立小刀）；config.yaml 的 context／rules GUI 編輯（Spectra 專案首頁可直接編輯專案說明／產出規則——真差距，但本刀 Non-Goal 已排除 rules/context 且已有 28 任務，維持排除、後續刀）。③不學：切入 tab 即彈「格式升級」並主動寫檔的 onboarding（實測誤觸即改壞 repo）；確認對話框「取消／執行」按鈕相鄰且等視覺重量——提煉為 speclink 設計原則：任何寫入使用者 repo 的確認框，寫入鈕與安全鈕需距離或視覺重量差異、預設焦點落在安全選項。
**Ruled out**: 學 Spectra 的每專案獨立儀表板頁（統計卡首頁）——speclink 看板欄頭已有各欄計數＋已封存徽章，看板本身就是儀表板，另設首頁多一層導航（YAGNI）。
**Open**: 承 Round 1 的假設 1-4 與三個開放點仍待使用者確認；本輪新增：看板搜尋塞本刀還是另開刀？快捷鍵系統的範圍（僅 tab 切換 vs 全域系統）？

### Round 3 — assumptions (2026-07-06)

**Focus**: mockup 定稿與開放點裁決
**Position**: 使用者核可 ASCII mockup 全案與預設裁量。定案：①分頁列取代頂欄「目前專案」chip，active tab 粗框 teal 底、✕ 僅 active 與 hover 顯示；②「＋」掛分頁列尾端與右上「開啟專案」雙入口並存；③徽章＝進行中變更數，hover tooltip「N 個進行中變更」，背景快照制（啟動輕掃一次、活躍 tab 即時）；④Ctrl+Tab 循環、Ctrl+1..9 直達；⑤零 tab 首啟動顯示「開啟專案」空狀態引導頁（含「一般目錄可經確認後初始化」說明）；⑥失效路徑 tab 轉錯誤態（警示 icon＋灰字），點擊顯示錯誤與「自分頁移除」；⑦tabs 即最近清單（關 tab 即移除、不另設殘影選單）、上限沿 D8 的 10、切 tab 走 D3 三態流程；⑧寫入型確認框按鈕原則（安全鈕靠左＋預設焦點、寫入鈕靠右拉開距離）納入 init 確認框設計。
**Ruled out**: 看板搜尋列與快捷鍵系統塞進本刀——本刀已 28 任務，兩者皆獨立小刀另議（搜尋列基建現成、成本低，優先）。
**Open**: 無——全數收斂，進 conclude。

## Conclusion

**Decision**: desktop-config-multiproject 的專案選擇改為 Spectra 式「持久化專案分頁列」——UI 形態對齊、後端架構不動：D1（Mutex<PathBuf> 單一活躍 root）與 D3（開啟三態流程）原封不變，tabs 是 D8「最近開啟清單」的形態升級（localStorage 存路徑＋順序＋最後活躍，上限 10，關 tab 即移除、不另設最近選單）。細節定稿：分頁列取代頂欄「目前專案」chip；「＋」掛分頁列尾端與右上「開啟專案」雙入口；徽章＝進行中變更數（tooltip 註明、背景快照制：啟動對各 tab 以既有 list_changes_at 輕掃一次、活躍 tab 靠 watcher 即時）；Ctrl+Tab／Ctrl+1..9 切換；零 tab 首啟動顯示開啟專案空狀態引導頁；失效路徑 tab 轉錯誤態、點擊可自分頁移除；寫入型確認框按鈕原則（安全鈕靠左＋預設焦點、寫入鈕靠右拉開距離與視覺重量）納入 init 確認框。
**Rationale**: 使用者的心智模型來自 Spectra，分頁列是其專案選擇的核心體驗；選 UI 形態對齊而非多 root 同時活躍，讓既有 28 任務的架構決策全數保留，增量僅前端 tabs、徽章輕掃 command 與空狀態頁。
**Rejected alternatives**: 多 root 同時活躍（per-tab watcher／cache／store 分 tab——範圍爆炸且與 proposal Non-Goal 衝突）；學 Spectra 的格式升級式主動寫檔 onboarding（實測誤觸即改壞 repo，既有確認後 init 流程更安全）；每專案獨立儀表板首頁（看板欄頭已有計數，看板即儀表板）；tabs 之外另設最近清單（同一概念重複表達）。
**Deferred**: 看板搜尋列（獨立小刀，基建現成、優先）；快捷鍵系統＋側欄提示（獨立小刀）；config.yaml 的 context/rules GUI 編輯（後續刀，本刀 Non-Goal 維持）；背景 tab 徽章即時化（快照制實測不足再議）。
**Capture to**: proposal（Non-Goal 措辭精修＋最近清單改 tabs）、design（D8 改存 tabs、新增 tabs UI 與徽章決策、確認框按鈕原則）、specs/desktop-config（「最近開啟清單存於 app 本機」需求改寫為分頁列需求）、tasks（對應調整）——皆屬 desktop-config-multiproject，經 ingest 更新。
**Next**: /speclink-ingest desktop-config-multiproject
