## Context

桌面 app 的四種詳情（變更、討論、規格、已封存）目前各是一個以 `Sheet` 原語實作的右側面板：`RichDetailDrawer`（989 行）、`DiscussionDrawer`、`SpecDrawer`、`ArchivedDrawer`，由 store 的 `detailChange`／`detailDiscussion`／`detailSpec`／`detailArchived` 四個欄位驅動，`openDetail`／`openDiscussion` 開啟時把 `boardView` 切回看板並清掉可取消浮層（cut 0 定案）。外殼刀已把主區頂列做成麵包屑（`MainTitleBar` 吃 `Crumb[]`），看板刀讓 `KanbanBoard` 自持搜尋與篩選狀態並以 `title`／`description` 走 `PageHeader`。cut 1 的 `Tabs variant="card"`（含 `actions` 槽）、`Card size="nested"`、`ConfirmDialog`、`CopyButton`、語意 token 都已就位。

本 change 處理看板宿主的兩種頁（變更、討論）與共用的頁層、導覽、「開啟 ▾」；規格與已封存兩種頁由 `desktop-readonly-detail-pages` 接續，依賴本 change 的 D1、D2、D4。

## Goals / Non-Goals

**Goals**

- 點卡片後主區換成該項目的詳情頁；Esc 與麵包屑回看板，看板捲動、搜尋、篩選不變。
- ‹ n／m › 與 ⌥↑／⌥↓ 在看板呈現順序內切換。
- 變更詳情頁與討論詳情頁依原型版面：標題列、出身列、右側動作、狀態卡、卡片標頭式分頁、「開啟 ▾」、任務整列可點。
- `StatusPill` 原語落地；`RichDetailDrawer`、`DiscussionDrawer` 刪除。
- 既有互斥、落回看板、浮層收掉、skeleton、工單分頁、排程分頁、任務寫回等行為全部保留。

**Non-Goals**

- 規格詳情頁與已封存詳情頁（後半 change）。
- 瀏覽歷史堆疊。
- 分頁內容渲染與資料流的任何變動。
- 從 packages/ui 移除 `Sheet`（server-web 仍用）。

## Decisions

### D1 頁層：宿主頁保留掛載，詳情頁覆蓋主區

`<main>` 改為 `relative`；其內永遠渲染宿主頁（看板、規格、已封存、手冊、設定），任一 `detail*` 欄位非 null 時再渲染 `<div data-testid="detail-page" className="absolute inset-0 flex flex-col overflow-y-auto bg-background">`，宿主頁的外層 `<div>` 加 `inert`、`aria-hidden="true"` 與 `invisible`（`visibility:hidden`——保留版面與捲動位置，不會像 `display:none` 那樣重算）。看板的四欄捲動、`KanbanBoard` 自持的搜尋字串與篩選面板因此原樣保留，不另序列化。

App 以 `activeDetail` 派生目前開啟的詳情種類（`change`／`discussion`／`spec`／`archived`，後兩者本 change 只保留既有欄位、頁由後半 change 掛上；在那之前 `detailSpec`／`detailArchived` 仍由現有 `SpecDrawer`／`ArchivedDrawer` 承接）。

為什麼不卸載看板再還原：四欄各自的捲動位置與篩選面板開合都要另存；保留掛載零狀態。為什麼不是 `display:none`：隱藏後捲動容器會重置到頂。

### D2 麵包屑、‹ n／m › 與快捷鍵

`MainTitleBar` 加 `trailing?: ReactNode` 槽（麵包屑之後、Windows 視窗鈕之前）。詳情開啟時 App 給三段麵包屑：專案名（點＝關頁並回看板）、頁名（點＝關頁）、項目名（等寬、不可點）。`trailing` 放新元件 `DetailNav`（apps/desktop/src/components/DetailNav.tsx）：‹ 鈕、`n / m` 等寬 11px、› 鈕；到頭時對應鈕 `disabled`；`m` 為 0 或目前項不在序列中（例如項目剛被外部刪除）時整個元件缺席。

序列由宿主清單元件回報：`KanbanBoard` 新增 `onVisibleOrderChange?: (order: { changes: string[]; discussions: string[] }) => void`，於 `visibleChanges`／`visibleDiscussions` 變動時以 effect 回報——變更依欄序（提案中 → 進行中 → 已就緒）再依欄內順序；討論為上區全卡依序，其後接欄底「已轉出」細列（不論收合）。App 把回報存於 `visibleOrder` state，`DetailNav` 與快捷鍵共用 `stepDetail(delta)`：以目前項目在序列中的索引加 delta，越界不動；封存、刪除、退回的確認在途（`detailConfirmInFlight`）時不動——成功後 store 關掉的是「目前」詳情頁，切走的新頁會被誤關。頁層是同一個捲動容器（不重掛），切換項目時以 layout effect 把 `scrollTop` 歸零。

快捷鍵在 App 既有的 window keydown 處理器內加三條：`Escape` 關頁；`Alt+ArrowUp`／`Alt+ArrowDown`（macOS 為 ⌥）為上一個／下一個。三者在 `event.defaultPrevented`（Radix 已用 Esc 關掉自己的選單、對話框或提示時會先 preventDefault）、或 `event.target` 為 `input`／`textarea`／`[contenteditable]` 時不作用。

### D3 store：單一關頁動作與欄位改名

- 新增 `closeDetailPage()`：一次把四個 `detail*` 欄位與 `detailVerb` 清空，Esc、麵包屑、`DetailNav` 失效時共用；`closeSpec`／`closeArchived` 保留給 `SpecDrawer`／`ArchivedDrawer` 的關閉呼叫點。`closeDetail`／`closeDiscussion` 唯一的呼叫點是兩個被刪除的 Drawer，一併刪除。
- `setBoardView` 切換主區頁面時一併清空 `detailChange`／`detailDiscussion`／`detailVerb`：頁層疊在宿主上，專案欄、應用程式選單、系統匣切頁若不關頁，新頁會被蓋住（舊抽屜是 modal，擋住了專案欄，這條路徑是頁層才開的）。
- `drawerVerb`／`clearDrawerVerb` 改名 `detailVerb`／`clearDetailVerb`（它是開啟中變更的分析結果；識別符隨元件改名，不留「drawer」字面）。
- `openDetail`／`openDiscussion`：行為不變（切回看板、互斥、收浮層、不可取消浮層開啟中不開）。注記更新為頁語意。
- `boardView` 型別不變：詳情不是一個 view，而是疊在 view 上的一層。

### D4 「開啟 ▾」文件動作與 Tauri 命令

- `apps/desktop/src/adapter/fsActions.ts` 的 `FsActionsAdapter` 加 `openDefault: (path) => invoke("open_path", { path })`；`apps/desktop/src-tauri/src/lib.rs` 新增 `open_path` 命令：`app.opener().open_path(&path, None::<&str>)`，與既有 `reveal_in_folder` 同一條 opener 路徑、不經前端 scope。
- 文件的磁碟位置由 core 解析，與讀內容同一套邏輯：新增 `document_location(root, change, artifact)`（走 `context_for_change`——綁 worktree 時為 worktree 副本，spec 目錄名取自專案設定）與 `discussion_location(root, slug)`（live 優先、封存後備，封存檔名帶日期前綴），皆回 `{ path, relative }`（`relative` 相對所在 checkout 根）或 null。前端不拼路徑——寫死 `openspec/changes/<name>` 會在 worktree、自訂 spec 目錄、已封存討論三種情況指錯檔。`FsActionsAdapter` 加 `locate(root, doc)`，原 `copyPath` 更名 `copyText`（它寫入的可能是路徑或文件原文）。
- `open_path` 先 canonicalize 再判：傳入名稱與實際目標都必須是 `.md`，且目標是一般檔案——名為 `proposal.md`、實際指向 `.app`／`.command` 的符號連結不得混過；殼層依傳入名稱的副檔名選程式，名為 `x.bat`、指向 `.md` 的連結同樣拒絕。解析結果只用來檢查，開啟仍傳原路徑：Windows 的解析結果帶 `\\?\` 前綴，殼層與不少預設 App 不認。
- store 加 `locateDocument(doc)`（活躍專案無本機目錄或查不到時回 null）與三個作用於已定位文件的動作：`openDocumentDefault(location)`、`revealDocument(location)`、`copyDocumentPath(location)`，以及 `copyDocumentContent(text)`；四者的錯誤皆以既有失敗 toast 語意呈現（主詞為相對路徑，複製內容為「複製內容」）。
- `DocumentActionsMenu`（apps/desktop/src/components/DocumentActionsMenu.tsx）：`DropdownMenu`，觸發鈕為 ghost 小鈕「開啟 ▾」（`FileText` 圖示＋文字＋`ChevronDown`）；展開時才向宿主查位置（文件可能剛被封存搬走），內容首列為等寬 11px 灰字的相對路徑標籤，其後「以預設 App 開啟」「在 Finder 顯示」（標籤沿用 `fs.revealIn.<platform>`）「複製路徑」，位置未回或查不到時三項停用；無本機目錄（remote 無 checkout）時只剩「複製內容」且首列標籤缺席，分頁不載原文（工單）時元件缺席；分頁沒有文件時元件缺席。
- 頁元件回報「哪份文件」而非路徑：`ChangeDetailPage` 與 `DiscussionDetailPage` 接受 `renderDocumentActions?: (target: DocumentTarget | null) => ReactNode`，`DocumentTarget = { doc: DocumentRef; content?: string | null }`、`DocumentRef = { change, artifact } | { discussion }`（型別在 `DetailHeader.tsx`）。由頁依作用中分頁呼叫——提案 `proposal.md`、設計 `design.md`、任務 `tasks.md`、規格 `specs/<capability>/spec.md`（規格分頁目前選定的 capability）、審查 `review.md`、驗證 `verify.md`、排程 null；討論各分頁皆為 `{ discussion: slug }`。`content` 為該分頁已載入的文件原文（未載完為 null）；工單分頁只載解析後的回合，`content` 缺席。

### D5 `StatusPill` 原語

`packages/ui/src/components/ui/status-pill.tsx`：`StatusPill({ tone, icon?, children, className? })`，`tone` 為 `"neutral" | "primary" | "progress" | "success" | "warning" | "destructive" | "stamp" | "improve"`，渲染 `inline-flex h-5 items-center gap-1 rounded-full px-2 text-[11px] font-medium whitespace-nowrap`，底色為該 tone token 12% 淡底、字色為 tone 本色（`bg-<token>/12 text-<token>`；neutral 為 `bg-muted text-muted-foreground`）。審查／驗證章籤以 `REVIEW_TONE`／`VERIFY_TONE` 既有對照換成 tone 名（對照表新增 `*_PILL_TONE` 欄位，顏色語意不變：已審查／已驗證 stamp、進行中 progress、過期 warning、未通過 destructive）。消費者：D6 標題列章籤、D6 狀態卡品質關卡列、D7 討論狀態籤與衍生變更籤。看板卡片與系統匣的既有章不在本 change 範圍。

### D6 變更詳情頁（`ChangeDetailPage`）

兩頁共用的骨架收在 `DetailHeader.tsx`：`DetailHeader`（標題列＋出身列）、`DetailPageFrame`（置中 768px 內容欄）、`StatusCard`／`StatusRow`（狀態卡與列；`wrap` 讓衍生變更籤折行）、`DetailContentCard`（分頁列下的內容卡與閱讀欄），以及 D4 的 `DocumentRef`／`DocumentTarget`。

取代 `RichDetailDrawer`，props 同其現有介面去掉 `open`／`onOpenChange`、`drawerVerb` 改 `detailVerb`，加 D4 的 `renderDocumentActions`。版面（原型「詳情頁」畫板）：

- 外層 `mx-auto w-full max-w-[768px] px-6 pt-7 pb-10`。
- **標題列**：`h1` 等寬 22px semibold 名稱單行截斷＋`CopyButton`＋章籤（審查在前、驗證在後；非 none 才渲染；蓋章日期與含 email 的完整識別收進提示）；同列右端動作區：「分析」outline 鈕（分析中 disabled 轉圈）、依派生階段的鈕——進行中「退回提案中」outline、已就緒「封存」主要鈕、有 `onClaim` 時「認領」outline——與「⋯」圖示鈕。
- **出身列**：單行 `text-xs text-muted-foreground`，以「·」分隔：建立者頭像＋名字、產生工具、建立相對時間、「開工 YYYY-MM-DD」（有 startedAt）、「來自」＋來源討論籤（首顆直出、其餘「+N」浮層，沿用現有元件）、「同源」＋同源變更籤；缺席段整段缺席；email 在提示。
- **「⋯」選單**：「封存」「刪除」兩項；守門沿「詳情頁的封存與刪除依階段守門」——不可按時項目 `disabled` 並在項目下方以 12px 灰字列出原因（取代 tooltip，選單項不接 hover 提示）；「刪除」為 `text-destructive`、置底、前有分隔線。remote 能力缺口原因優先於階段原因。
- **分析結果**：`detailVerb` 與本變更相符時，在狀態卡之下渲染既有 `AnalyzePanel` 於一張 16px 圓角卡內，卡頭右端關閉鈕走 `onClearVerb`。
- **「狀態」卡**：`SectionHeader`「狀態」＋`Card`，三列細線分隔、左標籤右內容：任務進度（既有進度條＋`n/m · p%` 等寬）、品質關卡（章籤各帶日期，兩站皆 none 時灰字「尚未經過品質關卡」）、排程（`planWave` 有值時波次徽章＋「第 N 波」＋`planBlockedLabel` 的前置說明；無 wave 時灰字沿用排程分頁的說明詞條）。
- **內容卡**：`Tabs variant="card"`——`TabsList actions={renderDocumentActions?.(…)}`；分頁與順序不變（提案／設計／任務／規格／排程／審查／驗證，後兩者條件式）；`TabsContent` 在 `Card className="rounded-t-none border-t-0"` 內，內文 15px 行高 1.75、`max-w-[72ch]` 行寬上限沿用。各分頁內容元件（`SectionedDoc`、`TaskList`、規格 delta、排程四卡、`TicketView`）原樣。
- **任務分頁**：D8。
- **寬度約束**：標題列、出身列、動作列任何資料組合下不換行、不撐寬、頁不出現水平捲軸（既有需求字面改頁）。
- 全螢幕切換鈕刪除（頁本身就是全寬）。

### D7 討論詳情頁（`DiscussionDetailPage`）

取代 `DiscussionDrawer`，props 去掉 `open`／`onOpenChange`、加 `renderDocumentActions`。版面（原型「討論詳情頁」畫板）：

- 標題列：slug 等寬 22px＋`CopyButton`＋狀態籤（`StatusPill`：討論中 neutral、已結論 primary、已轉出・尚無結論 warning、已轉出・保留中 warning、已轉出 primary）＋改進籤（kind 為 improve 時，詞條沿用）；右端「封存」outline（concluded、或已轉出且保留在途，且宿主提供 `onArchiveDiscussion` 才渲染；保留在途時 disabled、提示「結論保留在途；最後一刀轉出後自動封存」）。
- topic 15px 一行；出身列：建立者頭像＋名字、「N 輪」、建立日。
- 狀態卡三列：**階段**——步驟列「討論中 → 已結論 → 已轉出 → 已封存」，現站 `StatusPill primary`、已過站 ✓ 灰字、未到站 ○ 更淡（現站由 status 派生：promoted → 已轉出、concluded → 已結論、其餘討論中；已封存站保留給後半 change 的已封存討論頁共用同一元件）；**保留在途**——`hold` 為 true 才渲染，灰字說明；**衍生變更**——`promotedTo` 非空才渲染，每個子變更一顆可點等寬籤＋階段 `StatusPill`（提案中／進行中／已就緒依 `STAGE_*` tone、已封存 neutral、已刪除 destructive 加刪除線），點擊走 `onOpenChangeCard`。
- 內容卡：`Tabs variant="card"`，結論／討論過程 N／背景／衍生變更 N，預設分頁與區段切分退回規則不變；衍生變更分頁維持唯讀清單（與狀態卡籤列同資料，分頁多列出現況說明）。

### D8 任務列整列可點

`TaskList` 的每個任務列外層改為 `<label>` 包住勾選框與任務文字，點列任一處即切換；拖曳把手（`GripVertical` 鈕）以 `onClick` 阻止冒泡、`onPointerDown` 不觸發勾選（任務文字為純文字，列內沒有連結）；唯讀（封存檢視）與批次或拖排寫回中（busy）列的 `label` 不帶 `htmlFor` 且游標為預設——單一任務的勾選寫回不鎖列，維持「勾選後立即取消不遺失」。頁面有選取文字時 label 的單擊不轉送（拖選文字不等於勾選；雙擊選字的第二下照常放行，與尚無選取時就已切換的第一下互相抵銷；直接點勾選框照常）。拖曳進行中以捕獲階段把 Esc 標為已處理：PointerSensor 不標記，否則 Esc 會冒到詳情頁的關頁快捷鍵。勾選框保留 `role="checkbox"` 與「任務 N」標籤、空白鍵切換；拖曳仍只由把手啟動（既有「點擊核取方塊不誤觸拖曳」場景不變）。工具列上方加一行 12px 灰字提示「點整列即可勾選；拖曳把手改順序」（有寫入能力時才顯示；無拖排能力、不顯示把手時為「點整列即可勾選」）。

### D9 進出動畫與 reduced-motion

詳情頁層掛上時 `animate-in fade-in slide-in-from-bottom-1 duration-150 ease-out motion-reduce:animate-none`（4px 位移、150ms）；關閉時直接移除節點，無退場動畫。popover／tooltip／對話框的既有進出動畫不變。

### D10 詞彙與文案

- `openspec/LANGUAGE.md`：「詳情面板」條目改名「詳情頁」——definition「點卡片或列後主區整個換成該項目的頁（變更／討論／規格／已封存四種），麵包屑與 Esc 回原頁」，avoid「抽屜、詳情面板」，why 補 cut 3b 的裁定與日期；文件內其餘「詳情面板」字樣（受控例外段、衍生變更、背景、換頁、首輪、複驗、改進討論條目）同步改「詳情頁」。
- i18n：`board.pageDesc` 改「依生命週期分欄；拖曳卡片調整順序，點卡片開詳情頁」；新增 `detail.*`（`back`、`prev`、`next`、`position`、`open`、`openDefault`、`copyContent`、`status`、`statusTasks`、`statusQuality`、`statusPlan`、`noQuality`、`archiveMenu`、`deleteMenu`、`tasksHint`）與 `ddetail.*`（`stationArchived`、`holdLabel`、`holdHint`、`promotedLabel`）；既有 `drawer.*`／`ddrawer.*` 鍵名保留、只改會被看見的文案。兩語系鍵集合維持相等。

## Risks / Trade-offs

- **宿主頁 `inert` 在 jsdom 無行為**：測試只能斷言屬性存在；真實阻擋靠瀏覽器。WKWebView 自 Safari 15.5 起支援 `inert`。
- **看板保留掛載的成本**：看板在詳情頁底下仍訂閱 store 並重算；資料量為單專案數十張卡，可接受。
- **「⋯」選單內的停用原因改為項目下方文字**：與既有 tooltip 字面不同，需求字面同步改；資訊不減。
- **序列回報的時序**：外部刪除目前項目後 `DetailNav` 缺席、頁內容由既有 refresh 邏輯清空（`detailChange` 找不到即 null，頁關閉）——與面板時代一致。
- **i18n 鍵保留舊前綴 `drawer.*`**：鍵名不是使用者可見文案，改名只增加 diff；後續清理另議。

## Migration Plan

1. 先落 D5、D2 的 `trailing` 槽、D4 的命令與選單（可獨立測）。
2. 建 `ChangeDetailPage` 與 `DiscussionDetailPage`，把 `richDrawer.test.tsx`／`discussionDrawer.test.tsx` 的案例逐條搬到新測試檔（去掉 Sheet 的 `role="dialog"` 與開關斷言）。
3. App 接線頁層、麵包屑、`DetailNav`、快捷鍵；刪除兩個 Drawer 元件與匯出；`SpecDrawer`／`ArchivedDrawer` 暫留（後半 change 處理）。
4. 規格、LANGUAGE.md 與文案最後對齊，跑詞彙守門。

## Open Questions

- 無。宿主頁隱藏方式、瀏覽歷史、停用原因位置三題於本 design 定案。
