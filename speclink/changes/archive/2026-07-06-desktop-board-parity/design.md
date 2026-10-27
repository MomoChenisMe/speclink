## Context

桌面 app（desktop-shell-and-browser 產出）的看板資料流：App.tsx 掛載時與 app 內操作後呼叫 store 的 refresh，經 SpeclinkDataSource 走 Tauri command 到 speclink-desktop-core，逐次以 root 重建 ProjectContext 即時讀檔。沒有檔案監看——外部寫者（CLI、agent、人手）的變更直到下次 app 內操作才反映。看板欄位由 packages/ui 的 stage 派生（0 任務＝提案中、全完成＝已就緒、其餘＝進行中），純任務數近似。

引擎的 in-progress 標記存於 .git/speclink-app/speclink.db（Spectra 同款 DDL 逐字複製，含 .migrate.lock 與舊 in_progress.json 遷移），由 CLI 的 in-progress add 寫入；inprogress 模組只有 add 一個 pub 函式，**全 repo 零讀取者**，且 Spectra 用不同路徑（.git/spectra-app/）——parity 純儀式、無互通，卻繼承「狀態不隨 repo 走」的跨機缺陷。change meta（.openspec.yaml）已有 created／created_by／created_with／from_discussion，archive 時蓋 archived_by／archived_at——生命週期歸屬已完成三分之二。

封存側：文件實體完整保留於 changes/archive/<dated-name>/；Store trait 有 read_archived_meta／write_archived_meta 對，但無封存 artifact 讀取；active change meta 亦無原文讀寫對（list_changes 回傳解析後的 meta、create_change 寫初始 meta）。桌面的封存清單經 SQLite 衍生快取（.speclink/desktop-cache.db，CACHE_VERSION 1，僅存 meta），ArchivedRow 只渲染日期＋名稱＋複製鈕。

本設計實作討論「桌面即時刷新與封存瀏覽」的第一刀結論（parity 四項）。

## Goals / Non-Goals

**Goals:**

- 外部與 app 內的 openspec/ 變更雙向即時反映到看板、抽屜與封存頁。
- 已封存變更可展開唯讀檢視（提案／設計／任務／規格），列帶任務數徽章。
- in-progress 標記真相遷入 change meta（started_at／started_by／started_with），冪等、歸檔保留；SQLite 寫入端退役。
- 看板欄位改由標記驅動；CLI 一切輸出位元級不變。

**Non-Goals:**

- 封存的復原與刪除；討論看板（第二刀）；web／remote 端實作；SpeclinkDataSource 訂閱介面；逐任務歸屬與 event log；CLI 輸出任何欄位變更。

## Decisions

### D1 標記真相遷入 change meta

Store trait 新增 active change meta 的原文讀寫對（與 read_archived_meta／write_archived_meta 對稱），FsStore 實作為讀寫 changes/<name>/.openspec.yaml。inprogress::add 改為經儲存介面讀-改-寫 meta：追加 started_at（當日 ISO 日期）、started_by（與 created_by 同一身分來源）、started_with（agent 名）；已有 started_* 時冪等不覆寫（保留首次開工蓋章），stdout 與 exit code 兩種情形皆與現行一致。函式簽名由收 Workspace 改為收儲存介面——CLI 呼叫點一行跟隨，輸出不變。SQLite bootstrap／遷移機制整段退役刪除；既有機器的 .git/speclink-app/ 殘檔留置無害，不做遷移（標記從未被消費過，遷移無對象）。change meta 解析結構加三個 Option 欄位（serde 預設，舊檔缺欄位即 None，向後相容）。archive 流程既有的 meta 讀-改-寫天然保留新欄位，以測試釘住。
替代方案：(a) 維持 SQLite 並補讀取端——host-local 不隨 repo 走（跨機失聯）、remote 三情境需另開 store 狀態縫，且 parity 純儀式（路徑不同無互通），否決；(b) SQLite 檔案進版控——二進位不可合併、page 重寫致工作樹無故髒、Windows 檔案鎖阻擋 git 操作、remote 模式文件真相在 store 不在 repo 仍無解，否決；(c) 裸布林 in_progress 欄位——升級需 migration，歸屬資訊寫入時免費且 remote 情境（PO 看誰在做）必需，否決。儲存解耦：標記騎在 meta 文件上，store 同步到哪標記就到哪——remote store 屆時實作同一對 meta 方法即得，無新狀態抽象。

### D2 桌面清單疊加標記欄位

speclink-desktop-core 的 list_changes_at 在既有 CLI 同形 payload 之上，為每個 change 疊加 startedAt／startedBy／startedWith（camelCase，未開工為 null）——資料來自 model::list_changes 已解析的 meta，不多讀檔。speclink list --json 的 CLI 輸出**位元級不變**（parity 紅線：store-abstraction spec 明定 31＋16＋8 對照為驗證載體）；桌面 payload 是桌面自己的契約，允許超集。manage 的 change_meta 查詢隨 meta 結構自然帶出新欄位供抽屜顯示。
替代方案：改 CLI 的 changes_json 加欄位——破壞位元級 parity 基線，否決；桌面每 change 另發 meta 查詢——N 次 IPC 往返換取零收益（meta 已在 list 解析路徑上），否決。

### D3 openspec 檔案監看

apps/desktop/src-tauri 新增 watch 模組：以 notify（官方 debouncer 伴隨件）遞迴監看 <root>/openspec/，事件合併去抖（數百毫秒級）後向前端發送單一 Tauri 事件（workspace-changed，不帶 payload——前端一律整批 refresh，不做細粒度 diff）。前端在 App 掛載時訂閱該事件呼叫既有 refresh；app 內操作後的主動 refresh 保留，與監看事件重複由去抖吸收。監看不含專案根的 .speclink/（快取寫入不在 openspec/ 下，天然無自迴圈）。wiring 全在宿主層：SpeclinkDataSource 與 packages/ui 不知道信號源存在——未來 web 端以 SSE 對接自家 store，介面零變更。前瞻註記：watcher 綁定啟動時的 root，desktop-config-multiproject 的執行期換 root 落地時需隨切換重掛（屬該刀範圍）。
替代方案：輪詢——常駐浪費且延遲固定，否決；視窗聚焦時刷新——切窗前的變更不反映、與「即時」要求不符，可作為 watcher 失效時的保底但不是主方案，否決為主案；subscribe 進 SpeclinkDataSource——重演逼 web adapter 表態的介面污染（延續 desktop-config-multiproject 的 D6 結論），否決。

### D4 Store trait 封存讀取擴充

Store trait 新增兩個方法：封存 artifact 原文讀取（以 dated_name＋output path 定址，如 proposal.md、specs/<cap>/spec.md）與封存 delta capability 名列舉——皆為 read_archived_meta 的對稱擴充，帶預設實作（None／空清單）使既有其他 Store 實作不需即刻跟隨；FsStore 覆寫為讀 changes/archive/<dated-name>/ 下實體。speclink-desktop-core 據此提供封存文件查詢（含與 document_at 同款的路徑穿越防護），Tauri 層對應新增唯讀 command。
替代方案：以 document_at 拼 "archive/<dated-name>" 相對路徑 hack——把 fs 目錄佈局假設外漏到呼叫端、remote store 直接斷裂，否決；封存文件全量進快取——文件本體大且低頻存取，快取只放清單級欄位（見 D5），否決。

### D5 封存快取升版帶任務計數

desktop-cache.db 的 CACHE_VERSION 1→2，archived_changes 表加 tasks_total 與 tasks_done 欄位；版本不符即整表重建（既有機制），首次收斂時經 store 讀封存 tasks.md 解析計數入快取，之後清單讀取零解析。快取失敗的退回路徑（直接以目錄資料回應）維持，徽章欄位缺席時前端不顯示徽章。
替代方案：展開時才懶解析單筆——列表收合狀態顯示不了徽章（Spectra 有），否決；每次清單全量現場解析——違背此快取存在的理由（歸檔量無上限成長），否決。

### D6 封存列展開檢視

ArchivedRow 擴為可展開（chevron 切換）：展開後呈現唯讀分頁（提案／設計／任務／規格＋N），內容經 SpeclinkDataSource 新增的封存文件讀取與封存 capability 列舉方法懶載入，渲染復用既有 Markdown 與唯讀模式的任務清單元件（不接 onToggle——封存不可互動）。列上顯示任務數徽章（來自 D5 快取欄位）。ArchivedItem 型別加計數欄位。封存瀏覽屬「change 瀏覽」抽象本體（web 版同需），進 SpeclinkDataSource 不算介面污染——與 workspace 管理操作（開專案／設定）的判準不同。
替代方案：點列開 RichDetailDrawer 共用抽屜——抽屜深度綁互動任務（勾選、拖曳、動詞列），唯讀化的條件分支多於復用收益，且 Spectra 的封存互動就是行內展開（截圖對照），否決。

### D7 看板 stage 標記驅動

packages/ui 的 stage 派生規則改為（優先序由上而下）：任務全完成（totalTasks > 0 且 completed == total）＝已就緒；有 startedAt＝進行中；其餘＝提案中。與現行規則的可見差異：有任務但未開工的 change 由「進行中」移回「提案中」（修正剛 propose 完即被錯置的問題）。詳情抽屜標頭顯示「誰於何時開工」（startedBy＋startedAt，未開工不顯示）。stage 單元測試矩陣更新並補齊四象限（0 任務／有任務未標記／已標記未完成／全完成）。
替代方案：started 缺席但任務全完成不算已就緒——手動勾完全部任務的 change 會卡在提案中，違反直覺且結論已定全完成優先，否決。

## Implementation Contract

**行為（使用者可觀察）：**

- CLI（或 agent、或手動編輯器）修改 openspec/ 下任何文件後，執行中的桌面 app 於一兩秒內自行更新看板卡片、任務數、抽屜與封存頁——無需重啟、無需 app 內操作。app 內勾任務／跑動詞的即時反映維持既有行為。
- speclink in-progress add 某 change 後：該 change 的 .openspec.yaml 含 started_at 與 started_by（git 身分可得時）；started_with 依 created_with 同機制——呼叫端具 agent 識別時寫入（CLI 現無此來源、缺席；寫入縫為引擎函式的 agent 參數）。再次執行同指令，欄位值不變（冪等）；兩次執行的 stdout 與 exit code 與現行版本一致；.git/speclink-app/ 目錄不再被建立。
- 桌面看板：剛 propose 完（有任務、無標記）的 change 顯示於「提案中」；in-progress add 後移入「進行中」；任務全勾後移入「已就緒」。抽屜標頭可見開工者與開工日。
- 已封存頁：每列顯示日期、名稱、任務數徽章（如 48/48）；點擊展開唯讀分頁檢視提案／設計／任務／規格內容；復原與刪除不提供。
- speclink archive 一個已開工的 change 後，封存目錄的 .openspec.yaml 同時含 created_*、started_*、archived_* 三站欄位。

**介面／資料形狀：**

- Store trait 新增四方法：active change meta 原文讀寫對（與 archived 對稱）、封存 artifact 原文讀取（dated_name＋output path 定址）、封存 delta capability 列舉；後兩者帶預設實作（None／空）。
- inprogress::add 改收儲存介面與身分參數，寫 meta；meta 解析結構加 started_at／started_by／started_with 三個 Option 欄位。
- 桌面 list_changes payload 的 change 項疊加 startedAt／startedBy／startedWith（camelCase，null＝未開工）；CLI list --json 位元級不變。
- 新 Tauri command（唯讀）：封存文件讀取與封存 capability 列舉；Tauri 事件 workspace-changed（無 payload）。
- SpeclinkDataSource 新增封存文件讀取與封存 capability 列舉兩方法；ChangeItem 加 started 三欄位、ArchivedItem 加任務計數欄位；stage 派生規則如 D7。
- desktop-cache.db schema v2：archived_changes(dated_name, meta, tasks_total, tasks_done)。

**失敗模式：**

- watcher 建立失敗（如權限）：app 照常運作（僅失去自動刷新），錯誤記錄於 stderr／log，不彈窗轟炸。
- 封存文件讀取對不存在的 dated_name 或 artifact 回 None → 前端該分頁顯示空狀態；路徑穿越參數一律拒絕。
- 快取重建失敗退回目錄直讀（既有機制），徽章缺席不阻擋清單。
- in-progress add 對不存在的 change：維持現行行為——遷移前實測基線為靜默成功（無輸出、exit 0、名稱不驗證），遷移後同形且不寫任何檔案。

**驗收條件：**

- cargo test --workspace：inprogress 的 meta 寫入／冪等／不存在 change 錯誤、meta 結構向後相容（舊檔無新欄位）、archive 保留 started_*、FsStore 新方法（含穿越拒絕）、快取 v2 重建與計數。
- 既有 parity／color 對照套件與 twin harness 照常通過；speclink list --json 輸出與基線位元級一致；in-progress add 的 stdout snapshot 不變。
- npm test -w packages/ui：stage 四象限矩陣、ArchivedRow 展開與唯讀分頁、徽章顯示、ChangeItem/ArchivedItem 型別消費。
- npm test -w apps/desktop：workspace-changed 事件觸發 refresh 的 wiring（模擬事件）、抽屜開工者顯示。
- 真實視窗驗證（依 CLAUDE.md 備忘）：外部終端跑 speclink task done 勾一項任務→看板數秒內更新；in-progress add→卡片移欄；封存列展開檢視實際內容；jsdom 測不出的互動一律實點。

**範圍邊界：**

- In scope：上述 trait 四方法、inprogress 改寫與 SQLite 退役、桌面 payload 疊加、watcher＋事件、快取 v2、封存展開 UI、stage 規則、抽屜開工者顯示。
- Out of scope：封存復原／刪除、討論看板（第二刀）、web／remote 實作、SpeclinkDataSource 訂閱介面、CLI 輸出變更、執行期換 root 的 watcher 重掛（desktop-config-multiproject 範圍）。

## Risks / Trade-offs

- [改動 list 路徑週邊誤傷 CLI 輸出位元級 parity] → CLI 的 changes_json 完全不碰（疊加在 desktop-core），parity／color 套件為驗收硬條件；in-progress add 輸出以 snapshot 釘住。
- [watcher 跨平台差異（Windows ReadDirectoryChangesW vs inotify/FSEvents 的事件粒度與重複）] → notify 官方 debouncer 合併事件；前端一律整批 refresh 不依賴事件內容，粒度差異被吸收；三平台皆以真實視窗驗證。
- [app 自寫觸發 watcher 的重複刷新] → 去抖窗口內與主動 refresh 合併；refresh 為唯讀全量重讀，重複執行冪等無害。
- [快取 v2 首次收斂需解析全部封存 tasks.md，大歸檔庫一次性成本] → 僅版本升級時發生一次，之後零解析；失敗退回目錄直讀。
- [meta 結構加欄位影響既有解析] → Option＋serde 預設，舊檔與新檔互通；以向後相容測試釘住。
- [SQLite 退役後留在使用者機器的殘檔造成困惑] → 殘檔無讀取者、無害；design 記載即可，不寫清理邏輯（避免誤刪他人 .git 內容物）。
