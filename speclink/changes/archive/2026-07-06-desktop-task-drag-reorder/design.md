## Context

任務排序互動的現況鏈：packages/ui 的 TaskList 以上下箭頭按鈕發 onMove(ordinal, dir)，RichDetailDrawer 轉成 onMoveTask(change, from, to)（一次一格），App.tsx 經 SpeclinkDataSource.moveTask 走 Tauri command 到 apps/desktop/core/src/manage.rs 的 move_task_at——僅搬 checkbox 行本身、群組標題不動、越界回 Err；寫回後前端全量重讀 tasks.md。任務文字的編號前綴（1.1、2.3…）在搬移後不會更新，順序與編號立即脫節。

看板卡片已用 @dnd-kit/core 拖曳（PointerSensor activationConstraint distance 8——否則單純點擊被拖曳監聽吃掉；拖曳視覺用 DragOverlay 逃出 overflow 裁切——皆為 desktop-shell-and-browser 的實戰教訓，記於 CLAUDE.md）。封存唯讀檢視（desktop-board-parity 交付）以 TaskList readOnly 呈現：核取方塊 disabled、無排序鈕。

Spectra 桌面版的對照互動：每任務列左側 ⠿ 把手、拖曳放開即完成排序（截圖對照）。

**修訂脈絡（2026-07-06，首輪實作後使用者實測回報）**：首輪以「僅任務入 sortable、標題為靜態元素」實作——拖曳中 dnd-kit 的讓位 transform 只位移 sortable item，導致被越過的任務（如 2.1）視覺上穿越其群組標題滑進上一群組區域，預覽與放開後結果不一致（使用者截圖佐證）；且 over 只能是任務、放開一律落在目標任務前/後，「放到群組開頭」這個組界兩義槽位（群組 1 尾 vs 群組 2 首）無法表達。D6/D7 為此修訂新增，D1/D4 隨之修訂。

## Goals / Non-Goals

**Goals:**

- 任務排序改為拖放把手手勢，一次到位；上下箭頭移除。
- 搬移寫回時自動重寫受影響任務的編號前綴，文字編號恆與順序一致。
- 點擊勾選與拖曳互不干擾；封存唯讀檢視維持不可互動。
- 拖曳中讓位視覺不穿越群組標題；群組標題可作為「組首」落點（修訂新增）。

**Non-Goals:**

- 群組標題的拖放與重寫；拖入空群組的落點；無「數字.數字」前綴任務的自動補編號；引擎/CLI 任務動詞變更；看板卡片拖曳行為變更；觸控最佳化。

## Decisions

### D1 把手拖曳

TaskList 改用 @dnd-kit/sortable（verticalListSortingStrategy）：每任務列左側渲染 ⠿ 把手（GripVertical，aria-label「拖曳任務 N」），sortable 的 listeners **只綁把手**——核取方塊與文字的點擊事件不經過拖曳監聽；sensors 用 PointerSensor（activationConstraint: { distance: 8 }，沿用看板教訓）＋ KeyboardSensor（鍵盤可近性免費取得）。拖曳中以 DragOverlay 渲染浮起列（逃出抽屜 overflow 裁切），原位置以半透明佔位。readOnly 時不渲染把手、不掛 DndContext。**修訂**：sortable 序列不只任務——群組標題也入列（見 D6），讓位位移對齊群組邊界。
替代方案：整列可拖——與核取方塊點擊衝突（8px 位移內尚可，但把手更明確且與 Spectra 對照一致），否決；保留上下箭頭與拖放並行——雙軌互動維護兩套邏輯、畫面雜訊，否決。

### D2 重編號語意

搬移寫回時對整份 tasks.md 重算編號前綴，規則：(1) 群組編號取自標題自身的「數字.」前綴（如 ## 3. 整合驗證 → 3）；標題無數字前綴的群組，其下任務不重編號。(2) 群組內第 k 個 checkbox 行，若其文字以「數字.數字」＋空白開頭，前綴重寫為「群組編號.k」；不符樣式的任務行逐字元保留。(3) 群組標題本身與所有非 checkbox 行逐字元保留。(4) 不在任何群組下（首個 ## 前）的任務不重編號。跨群組搬移天然取得新群組編號。
替代方案：以群組出現序（1、2、3…）為群組編號——標題數字與重寫前綴可能不一致（畫面自相矛盾），否決；無前綴任務自動補編號——樣式猜測（N. 單層？N.M？縮排子彈？），否決（proposal Non-Goal）。

### D3 重編號落點

重編號實作為 apps/desktop/core/src/manage.rs 內的純函式（吃行陣列、回傳改寫後行陣列），move_task_at 搬行成功後呼叫再寫回——一次寫檔完成搬移＋重編號。set_task_done 不觸發重編號（勾選不改順序）。引擎（speclink-core）與 CLI 零變更：speclink task done 以 ordinal 定址 checkbox 行，與文字編號無關。
替代方案：重編號下沉 speclink-core——引擎現無任何排序動詞，為桌面專屬操作開引擎縫過度設計（web 端屆時經 SpeclinkDataSource.moveTask 的 server 實作重用同語意即可），否決；前端算好整份文字回傳——前端持有寫入真相、繞過管理層邊界，否決。

### D4 onReorder 收斂

（本決策經 2026-07-06 修訂：增加側別參數。）TaskList 的 onMove(ordinal, "up"|"down") 回呼改為 onReorder(from, to, before?)（from/to 皆 1-based ordinal、一次到位；before 為可選側別）；RichDetailDrawer 的 handleMove 逐格邏輯刪除，直接把落點轉給 onMoveTask(change, from, to, before?)。SpeclinkDataSource.moveTask 增加**可選**第四參數 before?: boolean——省略時後端維持方向推斷（見 D7），既有呼叫端與 web/remote adapter 零改動。手勢層變化仍不外漏成新介面方法。
替代方案：新增批次排序介面（一次傳整個新順序）——單次拖放只產生一組 from/to，批次介面無消費者，否決；為組首落點另開專用方法（moveTaskToGroupStart）——同一動詞拆兩個方法、介面碎片化，否決。

### D5 寫回重載

拖放放開 → onMoveTask resolve → 既有 reloadTasks() 全量重讀 tasks.md——重編號後文字已變，僅前端 arrayMove 樂觀排序會顯示舊編號，必須以檔案真相刷新。寫入期間沿用既有 busy 鎖（opacity 降低＋pointer-events 鎖定）防止連續拖放競態。
替代方案：前端同步模擬重編號避免重讀——重複實作 D2 規則、兩處真相，否決。

### D6 標題入讓位序列與組首落點

群組標題以**不可拖的 sortable item**（useSortable disabled，id 如 `g-<序>`）加入 SortableContext——dnd-kit 的讓位 transform 把標題視為一格：任務被越過時與標題保持相對順序，讓位視覺不再穿越群組邊界（修正使用者回報的「2.1 看似被換進群組 1」假象）。over 為標題時，標題是「組界槽」、依 active 相對標題的位置雙向解析（修訂）：active 在標題上方 → 成為該群組組首（to＝組首任務 ordinal、before=true）；active 在標題下方 → 移到標題之前、成為上一群組末任務（to＝標題前最近任務的 ordinal、before=false）——否則組首任務永遠拖不回上一群組末位（使用者實測回報）。標題該側無任務可錨定（空群組、檔首）或錨即 active 自己時不提供落點。
替代方案：per-group 多容器模式（dnd-kit multiple containers）——視覺與語意最完整，但 TaskList 需重構為巢狀容器、與單一 ordinal 序列的落點對映複雜度不成比例，否決；插入線 indicator 模型（無讓位動畫）——放棄 sortable 生態自帶的讓位回饋、自繪 indicator 工程更大，否決；標題維持靜態＋僅文件說明——不修使用者實際踩到的視覺誤導，否決。

### D7 moveTask 側別

manage.rs 的 move_task_at 加 `before: Option<bool>`：None＝維持既有方向推斷（向上移插目標前、向下移插目標後——首輪修正後的行為）；Some(true)＝明確插在目標任務**行之前**（跨標題即成為目標所屬群組的組首）；Some(false)＝明確插在目標任務行之後。Tauri command move_task 與 tauriDataSource、SpeclinkDataSource.moveTask 對應加可選參數。前端規則：over=任務 → 省略 before（方向推斷）；over=群組標題 → to=組首任務 ordinal、before=true。
替代方案：to 改為槽位索引（0..n）——槽位在群組邊界仍兩義（同一行間隙分屬上組尾/下組首），未解決根本問題，否決；負數/哨兵值編碼側別——型別欺詐，否決。

## Implementation Contract

**行為（使用者可觀察）：**

- 任務分頁每列左側有 ⠿ 把手（唯讀檢視沒有）；按住把手拖曳，浮起列跟隨游標、原位半透明；放開後清單依新順序呈現且受影響任務的編號前綴已重寫（如把 1.1 拖到 1.3 之後：原 1.2→1.1、原 1.3→1.2、原 1.1→1.3）。
- 跨群組拖放到任務上：任務落入新群組並取得新群組編號（拖到 2.1 之後 → 2.2，原 2.2 後移 2.3；群組 1 剩餘重排）。
- **拖放到群組標題上：任務成為該群組的第一個任務**（拖群組 1 的任務到「## 2」標題 → 前綴 2.1，原 2.1 → 2.2）。
- **拖曳中，被越過任務的讓位視覺不穿越其群組標題**——標題與任務一起讓位，群組歸屬在預覽中保持可讀。
- 文字不以「數字.數字」開頭的任務：拖放僅改變位置，文字逐字元不變。
- 在核取方塊上按下並於 8px 內放開：勾選切換、順序不變、無拖曳啟動。
- 上下箭頭按鈕不再存在。

**介面／資料形狀：**

- TaskListProps：onMove 移除，新增 onReorder?: (from: number, to: number, before?: boolean) => void；readOnly 語意不變。
- RichDetailDrawerProps.onMoveTask 與 SpeclinkDataSource.moveTask：既有 (change, from, to) 之上增加可選 before?: boolean——省略時行為與修訂前一致。
- packages/ui 新依賴 @dnd-kit/sortable（@dnd-kit/core 已在）。
- manage.rs：move_task_at 簽名加 before: Option<bool>；重編號純函式不變。
- Tauri command move_task 加可選 before 參數。

**失敗模式：**

- move_task_at 越界（from/to 超出 checkbox 行數、0、無 tasks.md）：維持既有 Err 行為，檔案不動（含帶 before 的呼叫）。
- 重編號遇到不符樣式的行：逐字元保留（重編號永不產生資料遺失）。
- 拖放中 moveTask reject：busy 解鎖、重讀 tasks.md 回到檔案真相、錯誤沿既有 verbResult 呈現路徑。
- over=空群組標題：前端不觸發 onReorder（落點無效，畫面回彈）。

**驗收條件：**

- cargo test -p speclink-desktop-core：重編號矩陣（組內移動、跨群組、無前綴保留、無數字標題群組不動、群組標題逐字元不變、勾選不重編號）＋側別矩陣（before=true 跨標題成組首、before 省略維持方向推斷、越界帶 before 仍 Err）。
- npm test -w packages/ui：把手渲染與 aria-label、readOnly 無把手、箭頭按鈕不存在、onReorder 接線（含標題落點→組首＋before=true 的對映）。
- npm test -w apps/desktop：既有測試不破。
- 真實視窗驗證：重現原報告操作（1.6 拖向群組 2）——讓位中 2.1 不穿越標題；放開於標題上成 2.1 組首、放開於 2.1 上成 2.2（檔案 diff 佐證）；核取方塊點擊回歸。

**範圍邊界：**

- In scope：TaskList 拖放化（含標題讓位與組首落點）、RichDetailDrawer 接線、manage.rs 重編號與側別、SpeclinkDataSource/Tauri command 可選參數、相關測試。
- Out of scope：群組拖放、空群組落點、引擎/CLI 變更、SpeclinkDataSource 新增方法、看板拖曳、封存唯讀檢視的任何互動。

## Risks / Trade-offs

- [拖曳監聽吃掉點擊（勾選失效）] → listeners 只綁把手＋PointerSensor distance 8（雙保險）；真實視窗實點驗證。
- [重編號誤傷使用者文字（前綴誤判）] → 僅重寫「數字.數字＋空白」開頭且位於有數字標題群組下的行；純函式單元測試矩陣釘住；其餘一律逐字元保留。
- [拖放與 busy 競態（連續快速拖放）] → 沿用既有 busy 鎖；寫回期間清單鎖定互動。
- [DragOverlay 與抽屜捲動容器的定位] → 沿用看板 DragOverlay 模式；真實視窗驗證長清單拖曳（跨捲動）。
- [標題入讓位序列後，讓位動畫中標題短暫位移可能造成新的視覺困惑] → 標題讓位與任務同幅同向、群組相對順序恆保持；真實視窗以原報告操作驗收，若動畫觀感不佳以縮短 transition 調整，不回退語意。
- [disabled sortable item 的 over 判定依 dnd-kit 版本行為而異] → 以 useSortable disabled（仍註冊 droppable）實作並以真實視窗驗證 over=標題可觸發；若該版本 disabled 不可為 over，改為標題外掛 useDroppable 同 id 的相容作法。
