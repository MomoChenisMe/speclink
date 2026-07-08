## Context

桌面看板卡片順序目前是計算值：變更卡由桌面內嵌 core 以 sort_changes 依修改時間排序（apps/desktop/core/src/query.rs），討論卡由 speclink-core 的 list_discussions 依 slug 排序。修改時間來自本機檔案系統，不隨 git 同步——同一 repo 在兩台機器上的看板順序本來就可能不同。已結論討論「本地看板變更卡手動排序」定案：四欄欄內手動拖排、順序以稀疏 rank 存卡片自身 meta、新卡落欄頂、跨欄彈回；rank 格式與重平衡留本設計定案。

既有可借鏡的實作：任務列拖排（packages/ui/src/components/TaskList.tsx ＋ move_task command）已建立「SortableContext ＋ DragOverlay ＋ PointerSensor distance 8 ＋ 拖曳手勢期間讓路（onDragActiveChange）」的完整模式；看板封存拖放（KanbanBoard.tsx 的 ArchiveDropZone）已有 DndContext 骨架。

## Goals / Non-Goals

**Goals:**

- 看板四欄（討論／提案中／進行中／已就緒）欄內拖排，順序落檔進 repo、跨機一致。
- 一次拖排在穩態下只改被拖卡的一個檔案；兩機各拖不同卡可被 git 自動合併。
- 既有 CLI 人眼與 --json 輸出逐位元不變（parity／color／twin 回歸對照不受影響）。
- 舊 repo（無 rank 欄位）行為完全不變——排序回退現行規則。

**Non-Goals:**

- 跨欄拖曳改變變更階段（階段仍由任務完成度推導）。
- 排序模式選單、SQLite／資料庫真相、app 本機持久化、儲存層重構（討論已否決）。
- CLI 新增 reorder 子指令；web 變體的看板排序。
- 任務列拖排（已存在）。

## Decisions

### D1: rank 採字串型 fractional key，不用浮點

rank 值為小寫英文字母組成的字串（如 `n`、`ans`），以字典序比較；插入兩鄰居之間時取字典序中點，無縫隙時**延長一位**產生新鍵（`ab` 與 `ac` 之間 → `abn`）。欄位名統一為 `board_rank`：變更卡寫入 change 目錄的 .openspec.yaml，討論卡寫入討論記錄的 frontmatter。

- 替代方案：f64 浮點中點。否決——(1) 連續插同一縫隙約 50 次後精度耗盡，必須引入「重平衡」（整欄多檔重寫＝製造合併衝突的一次性大 diff）；(2) serde_yaml 浮點格式化有跨版本漂移風險（1.5 vs 1.5000000000000002），而 meta 欄位是 byte-for-byte 保留的 parity 敏感區。字串鍵以延長取代重平衡——**重平衡機制整個不需要**，鍵長僅在病態連續插入下對數成長。
- 替代方案：整數序號步進（100、200、300）。否決——縫隙仍會耗盡，回到重編號多檔 diff 的老路（討論 Round 3 已否決）。

### D2: 排序語意——rank 升冪、缺值置頂、同值以名稱決斷

每欄的顯示序為三段複合鍵：`(有無 rank, rank 字串, 現行回退序)`。無 rank 的卡一律排在有 rank 的卡**之前**（欄頂，滿足「新卡落欄頂」），彼此間維持現行回退序（變更卡＝修改時間、討論卡＝slug）；有 rank 的卡依字典序升冪；rank 相同（兩機併發蓋章後合併的殘局）以卡片名稱／slug 決斷，保證全序且跨機確定。

- 替代方案：缺值落欄底。否決——使用者明確選欄頂，且與現行「修改過的卡浮上來」的手感一致。
- 替代方案：缺值視為隱含 rank 參與中點計算。否決——回退序含本機修改時間，跨機不穩定，算出的中點在另一台機器上語意錯位。

### D3: 首次拖排時整欄補章，穩態單檔寫入

拖放落點以鄰居的 rank 計中點；若目標欄內存在缺 rank 的卡（首次使用、或新卡尚未入序），該次寫回**先對整欄依當前顯示序批次派發 rank**（等距鍵，預留縫隙），再套用本次移動。穩態（欄內全員有 rank）下一次拖排只寫被拖卡一個檔案。

- 替代方案：只補被拖卡與相鄰卡。否決——缺 rank 卡在欄內任意分布時，「相鄰」的定義依賴本機回退序，補章結果跨機不一致；整欄一次補章讓欄從此進入穩態，語意最單純。
- 取捨：首次拖排是多檔 diff（一欄的卡各改一行）。可接受——一次性遷移成本，之後永遠單檔;風險節列出兩機同時首拖的併發情形。

### D4: rank 讀寫原語歸 speclink-core，排序與中點演算歸桌面 core

speclink-core 提供 rank 的讀與寫原語，經 Store trait 操作（維持 storage 解耦）：變更側於 ChangeMetadata 增加選配欄位 `board_rank: Option<String>`（讀取、寫回時與 created_*／started_* 同機制原樣保留）；討論側於 DiscussionInfo 增加同名選配欄位並提供寫回函式（frontmatter 更新沿 conclude／promote 的既有改寫機制）。**兩者皆不進任何 CLI 序列化輸出**——DiscussionInfo 的 rank 欄位以 serde skip 排除於 discuss list --json，listing::changes_json 的項目形狀不動。

中點計算、整欄補章、欄語意（哪些卡屬哪欄）留在桌面內嵌 core（apps/desktop/core）——排序演算是看板呈現域，不是 SDD 流程域；web 變體未來需要時再上提，現在上提是為不存在的呼叫者鋪路（過度設計）。

- 替代方案：桌面 core 直接讀寫 .openspec.yaml 與討論 frontmatter。否決——欄位保留語意（byte-for-byte）是 speclink-core 的既有契約區，繞過會出現兩份 meta 序列化實作。
- 替代方案：中點演算也放 speclink-core 並加 CLI 動詞。否決——CLI 無此需求，Non-Goal。

### D5: reorder command 以鄰居識別碼表達落點

新增一支 Tauri command `reorder_card`，參數：`kind`（`change` | `discussion`）、`id`（變更名或討論 slug）、`prevId` 與 `nextId`（落點前後鄰居的識別碼，欄頂／欄底以 null 表達）。桌面 core 讀取兩鄰居現值 → 計中點（必要時先整欄補章，D3）→ 經 speclink-core 原語寫回 → 前端 refresh。

- 替代方案：沿 move_task 的序數（from/to）表達。否決——看板有搜尋過濾（visibleChanges），視覺序數與全欄序數在過濾時錯位；鄰居識別碼在過濾狀態下語意仍正確（落在 A、B 之間就是 rank 介於 A、B，被過濾隱藏的卡可能留在其間，符合使用者表達的意圖）。
- 搜尋過濾中拖排**不停用**：同一條程式路徑，語意如上。

### D6: 看板 UI 沿任務列既有拖排模式——欄內 SortableContext、跨欄彈回、封存落點保留

KanbanBoard 的三個變更欄各包一個 SortableContext（verticalListSortingStrategy），DiscussionColumn 同構第四個；沿 TaskList 的既有教訓：PointerSensor activationConstraint distance 8（單擊開詳情不被吃掉）、DragOverlay 呈現拖曳視覺（逃出欄位 overflow 裁切）、拖曳手勢期間以 onDragActiveChange 通知宿主讓外部刷新讓路。dragEnd 時：落點在同欄 → 解析前後鄰居呼叫 onReorder；落點在他欄或無效 → 不呼叫（dnd-kit 自然彈回）；落點為 archived droppable → 走既有 onArchive 路徑不變。無障礙：拖排卡片補 aria-label（i18n 鍵進 packages/ui 訊息表）。

- 替代方案：卡片加 ⠿ 把手（如任務列）。否決——卡片整體已是拖曳源（封存拖放既有行為），distance 8 已解決點擊衝突，再加把手是重複機制且改變既有卡片版面。

## Implementation Contract

**可觀察行為**：

1. 於看板任一欄內拖動卡片到新位置放開：順序立即反映，重啟 app 後順序不變；commit 後另一台機器 pull，同欄顯示同序。
2. 穩態拖排一次，git status 只出現被拖卡的一個檔案變更（變更卡＝該 change 的 .openspec.yaml、討論卡＝該討論的 .md），diff 為 board_rank 一行的增改;該檔其餘內容 byte-for-byte 不變。
3. 首次拖排（欄內有缺 rank 卡）：該欄全部卡片獲得 board_rank，欄序＝拖放後的視覺序。
4. 無 rank 的卡（新建變更／新討論）顯示於所屬欄欄頂。
5. 變更卡拖到另一個階段欄放開：彈回原位、無任何檔案寫入;拖到封存落點：既有封存確認流程不變。
6. 相容性：對無 rank 的 repo，speclink list --json、speclink discuss list --json、桌面 list payload 逐位元不變;對有 rank 的 repo，兩個 CLI 輸出仍不含 rank 欄位、逐位元等同無 rank 時（排序與欄位皆不變）。

**介面／資料形狀**：

- meta 欄位：`board_rank`（選配、小寫字母字串），落於 change 的 .openspec.yaml 與討論 frontmatter。
- Tauri command：`reorder_card { kind: "change" | "discussion", id: string, prevId: string | null, nextId: string | null }`，成功回空、失敗回錯誤字串（前端經 verbResult 呈現，不靜默）。
- SpeclinkDataSource 介面（packages/ui/src/adapter.ts）新增對應方法；KanbanBoard／DiscussionColumn props 新增 onReorder 與 onDragActiveChange 回呼。
- rank 演算契約：midpoint(a, b) 對任意 a < b 回傳嚴格介於其間的鍵；批次派發對 n 張卡回傳嚴格遞增且兩兩留有可再分縫隙的鍵列。

**失敗模式**：

- 寫回失敗（檔案鎖、權限）：錯誤浮上 verbResult 單行呈現，看板 refresh 回磁碟現況（不留假象順序）。
- 鄰居在寫回前被封存／刪除（race）：以現存鄰居重算或落欄頂／欄底，不 panic、不寫壞 meta。
- 兩機拖同一張卡：單檔單行衝突，git 標準流程解——任取一方皆為合法順序。

**驗收準則**：

- Rust（apps/desktop/core）：rank 中點與批次派發的性質測試（嚴格介於、全序、延長行為）；reorder 寫回後 meta 其餘欄位 byte-for-byte 保留（沿 archive.rs 既有斷言模式）；query 排序測試（缺值置頂、rank 升冪、同值名稱決斷）。
- Rust（speclink-core）：讀含 board_rank 的既有檔不失敗；寫回保留欄位;discuss list --json 與 list --json 對含 rank 的 fixture 輸出不含 rank 欄位。
- 前端（packages/ui）：kanban.test.tsx／discussionColumn.test.tsx 斷言 SortableContext 掛載、dragEnd 解析出正確 prevId/nextId、跨欄與封存路徑分流。
- 真實視窗驗證（CLAUDE.md 紅線）：release exe 實際拖排一張卡，截圖確認落位、重啟後序不變、git diff 只含單檔——jsdom 測不出 pointer 拖曳互動。

**範圍邊界**：

- In scope：四欄欄內拖排、board_rank 讀寫與保留、reorder_card command、缺值回退、真實視窗驗證。
- Out of scope：CLI 動詞、web 變體、階段手動覆寫、排序模式選單、任務列、封存頁順序（唯讀檢視維持現行）。

## Risks / Trade-offs

- [meta 寫回破壞 parity 敏感欄位] → rank 寫回走 speclink-core 既有欄位保留機制，測試斷言 created_*／started_* byte-for-byte 存活（沿 archive.rs:457 測試同款模式）；CLI 輸出以「含 rank fixture 的輸出不含 rank」測試釘住。
- [兩機同時首拖同一欄] → 整欄補章互撞成多檔衝突。一次性風險：衝突任取一方後看板仍為合法全序（D2 的同值決斷兜底），最壞情況欄序洗牌一次;文件不另建鎖機制（過度設計）。
- [檔案監看與拖曳手勢 race] → 沿 TaskList 的 onDragActiveChange 模式：手勢期間宿主暫緩外部刷新，放開後寫回＋refresh 無縫接手。
- [搜尋過濾中拖排的落位與直覺偏差]（隱藏卡留在鄰居之間）→ 語意上仍是「落在 A、B 之間」;於 spec scenario 明文固定此語意，不留模糊。
- [跨平台] → rank 為純 ASCII 字串比較，無平台差;meta 寫回沿既有換行／路徑處理，不新增平台假設。
- [dnd-kit 單擊被拖曳吃掉的回歸] → 維持 PointerSensor distance 8（CLAUDE.md 既有教訓），前端測試釘住 activationConstraint。

## Migration Plan

- 無資料遷移：舊 repo 無 board_rank → 全數缺值 → 現行排序，行為不變；首次拖排即自然完成該欄補章。
- 回滾：拿掉功能後 board_rank 欄位殘留無害——speclink-core 讀取容忍未知欄位（既有行為），排序退回現行規則。
- 舊版桌面 app 開含 rank 的 repo：不識 rank、照現行排序顯示，無錯誤。

## Open Questions

無——rank 格式（D1）、重平衡（D1 消除）、缺值語意（D2）、補章時機（D3）、seam（D4）、command 形狀（D5）、UI 模式（D6）皆已定案。
