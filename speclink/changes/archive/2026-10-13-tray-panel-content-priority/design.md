## Context

macOS 系統匣面板是三段式版面：固定頁首（專案 tab 條卡）、可捲中段（內容卡）、固定頁尾（動作區塊卡），高度依內容自適應、上限 640px。現況中段順序為「討論 → 提案中 → 進行中 → 已就緒」，頁尾為四列動作。討論多時（wadpilot 實測 8 則），討論卡約 330px、可捲中段約 410px，三個生命週期分區被擠到第一屏之外。來源討論 `tray-panel-content-priority` 已定案三件事：重排、頁尾單列、內容卡收合，並以設計畫布的 TrayCrowdedNow／TrayCrowdedProposal 兩張畫板確認外觀。之後使用者回報 macOS 毛玻璃幾乎看不見：tray-panel-restyle 把根層補光由 bg-background/60 改為 bg-sidebar/90、卡片由 bg-foreground/5 改為不透明 bg-card，毛玻璃只剩一成自卡片縫隙透出；Rust 端 apply_vibrancy（HudWindow）仍正常。使用者選擇「補光層調回約六成、白卡不變」併入本變更。

變更只落在 `apps/desktop` 的面板前端（React）：面板是主視窗 store 的薄渲染層，資料由 TraySnapshot 事件送進來；speclink-core、speclink-host、speclink-desktop-core 與 Tauri 殼都不動，因為沒有新的領域邏輯、IPC command 或資料欄位。收合狀態是純呈現偏好，與既有「已轉出」收合列的記憶同類，留在面板視窗的 localStorage。

## Goals / Non-Goals

**Goals:**

- 打開面板時，三個生命週期分區落在第一屏內（以 wadpilot 的 5／3／0 筆資料為準）。
- 使用者可以收起不想看的內容卡，收合狀態跨專案、跨重開面板、跨重啟 app 保留。
- 四個動作的行為與可及性名稱不變，只改排法。

**Non-Goals:**

- 不改高度上限 640、每區前 5 筆的溢出門檻、討論列兩行（slug＋說明）。
- 不改非 macOS 原生選單（區段順序本來就是生命週期在前；選單本身是子選單層級，不需要收合）。
- 不做分段切換（中段一次只顯示一個清單）——面板的用途是一眼看全部狀態。
- 不做每個專案各自記收合狀態。
- 不讓專案 tab 條卡、動作區塊卡、復原卡、stale 狀態列可收合。
- 不改看板主視窗的欄位或收合行為。
- 不改共用元件 `SectionHeader`（@speclink/ui）的 API。
- 不讓卡片變半透明——會推翻主視窗與面板共用的「白底細框卡」規則，且深色背景下整體偏暗的問題會回來。
- 不改 Rust 端的 vibrancy 材質與視窗設定。

## Decisions

### 中段重排：生命週期卡在前、討論卡在後

在面板元件中，把討論卡的渲染移到三張生命週期卡之後；stale 狀態列仍在中段最上方，復原卡分支（取代討論與生命週期卡）不受影響。三階段的相對順序（提案中 → 進行中 → 已就緒）不變，與看板左到右一致。

- 替代方案「已就緒排最上」：與看板順序不一致，被否決。
- 替代方案「維持討論在前、只靠收合」：使用者第一次打開仍要捲，被否決。

### 動作區塊卡改為單列：文字鈕＋三顆圖示鈕

動作區塊卡內改成一個橫向列：左側「開啟 Speclink」保留圖示＋文字並佔滿剩餘寬度；右側依序為「專案設定」（SlidersHorizontal）、「設定」（Settings）、「結束」（Power）三顆圖示鈕。圖示鈕為原生 `button`（`type="button"`、`tabIndex={-1}`，與複製鈕同樣退出 Tab 順序），`aria-label` 與 `title` 取既有 i18n 詞條（`app.navProjectSettings`、`tray.settings`、`tray.quit`），不新增詞條。「設定」與「結束」之間放一條細直線分隔符（約 16px 高、`bg-border`，兩側各留一段外距），間距因此比其他鈕之間寬，並標出「結束」屬另一組，降低誤按。分隔符是卡內元素，不違反「區塊之間只以間距分隔」。驗收時只留加寬空白的版本看起來像排版出錯，故改為分隔符。hover 為半透明前景色灰底，與面板列同語彙。

- 名稱提示用 `title`（原生 tooltip），沿用面板站章的做法：面板是無焦點的一瞥介面，掛 Radix Tooltip 會多一層 portal 與焦點管理。
- 替代方案「保留四列」：佔約 130px，被否決。
- 替代方案「四顆都改圖示」：「開啟 Speclink」是最常用的動作，保留文字最好認。

### 收合觸發沿用已轉出收合列的模式

每張內容卡的分區標題外包一層可點容器：`role="button"`、`aria-expanded`、`tabIndex={-1}`、Enter／Space 也能切換——與現有「已轉出」收合列（`panel-promoted-toggle`）同一模式。方向箭頭用 `SectionHeader` 既有的 `action` 插槽放 ChevronRight，展開時旋轉 90 度（朝下）、收合時朝右；計數徽章維持在箭頭左側。

- 替代方案「替 SectionHeader 加 collapsible 屬性」：共用元件另一個使用者（DocumentTree）用不到，為單一呼叫端擴大共用 API，被否決。

### 收合狀態記在面板本機、所有專案共用一份

新增 `apps/desktop/src/panel/sectionCollapse.ts`，形狀比照 `promotedExpanded.ts`：

- localStorage 鍵 `speclink.tray.collapsedSections`，值為 JSON 字串陣列，元素為收合中的分區 id：`proposed`、`in-progress`、`ready`、`discussions`（前三者即 @speclink/ui 的 Stage 值）。
- 讀取：鍵缺席、JSON 壞掉、不是陣列、或 localStorage 不可用 → 回傳空集合（全部展開），不拋錯；陣列中的未知 id 忽略。
- 寫入：以目前收合集合整份覆寫；失敗靜默（只是下次回到上次能讀到的狀態）。
- 單一鍵、不分專案：切換作用中專案不重讀也不改狀態。

替代方案「每個專案各記一份」：要依分頁 locator key 分存並在關閉專案時清理，而收合是個人閱讀習慣、與專案無關，被否決。替代方案「每張卡一個布林鍵」：四個鍵各自讀寫，沒有好處。

### 收合中的卡只留標題一行、不自動展開

收合的卡只渲染分區標題（圖示、名稱、計數、箭頭）；其下的列、骨架、失敗提示、「還有 N 個…」溢出列、已轉出列與「已轉出」收合列都不渲染。收合的卡不套用空狀態卡的最小高度，標題下也不畫細線。計數照常由快照更新（載入中時與現況相同、不出徽章）。

- 收合狀態只由使用者點擊改變；快照更新（項目增減）不改收合狀態，所以收合中的卡不會自動展開。
- 卡重新展開時，溢出列回到未展開（溢出列的展開是元件內暫態，收合時隨內容卸載）；「已轉出」收合列維持它自己的記憶狀態。
- 面板高度由既有的 ResizeObserver（監看中段內容 wrapper）重新貼合，不需新程式碼。

### 毛玻璃找回：補光層調回約六成、白卡不變

面板根層（panel-root）的補光層由 `bg-sidebar/90` 改為 `bg-sidebar/60`；三類卡（專案 tab 條卡、各內容卡、動作區塊卡）維持不透明 `bg-card`。毛玻璃因此自面板外框（根層 8px 內距）與卡片之間的間距透出約四成，可見背後內容的色調；卡內文字與列的亮度由卡片面色決定，不受背後內容影響。六成是 tray-panel-scroll-and-vibrancy 當初針對深色背景實測調出的濃度（當時判準：深色背景下亮度明顯上提、淺色背景下毛玻璃仍可辨）。

- 替代方案「維持九成」：毛玻璃幾乎不可見，使用者回報為問題，被否決。
- 替代方案「卡片也半透明」：要讓卡片底下看得到毛玻璃，補光層與卡片都要調得很透，等於推翻白卡規則，深色背景偏暗的老問題也會回來，被否決。

### 收合箭頭統一規則：收合 ›、展開時指向展開出來的內容

面板與看板所有收合控制只用一條箭頭規則：收合時一律 ›；展開時箭頭指向展開出來的內容。內容卡標題的收合（內容在標題下方）展開時朝下；系統匣討論卡底部與看板討論欄底部的「已轉出 N」收合列（展開的細列在該列上方）展開時朝上。實作上兩處收合列的 ChevronRight 由展開時旋轉 90 度改為旋轉 -90 度（apps/desktop/src/panel/TrayPanel.tsx 與 packages/ui/src/components/DiscussionColumn.tsx），收合時不旋轉；版面與展開位置不變。

- 替代方案「收合 ⌄、展開 ⌃」（顯示更多／顯示較少的慣例）：同一張討論卡上，標題的 ⌄ 代表「展開中」、收合列的 ⌄ 卻代表「未展開」，同一個符號兩種意思，被否決。
- 替代方案「系統匣的已轉出細列改展開在該列下方、維持 › → 朝下」：要改版面與規格；看板的收合列固定在欄底、內容只能往上長，兩處做法會分岔，被否決。

### 專案 tab 條：「＋」固定右端、邊緣淡出、捲到作用中

現況：tab 寬 64px、tab 條可用約 290px，3 個專案加「加入專案」就塞滿；第 4 個專案起「加入專案」被擠出、第 5 個起專案也被擠出；捲軸隱藏、也不會捲到作用中的 tab，使用者看不出還有東西。決策三件事一起做：

- 「加入專案」移出可捲區，固定在 tab 條卡的右端，與專案 tab 同構：虛線框「＋」方塊、其下為「加入專案」文字（不截斷、寬度至少與專案 tab 同寬 64px，英文 Add project 會更寬；`aria-label` 與 `title` 為「加入專案」，取既有詞條），點擊行為不變。驗收時只留「＋」圖示的版本在一排帶名稱的 tab 旁顯得突兀，故保留文字。位置選右端：與主視窗圖示列「專案在前、＋在後」同序，且最常做的是切換專案，不該先看到很少用的動作。
- 可捲區依捲動位置在左右兩側套漸層遮罩（mask-image），只在該側還有未完整顯示的 tab 時淡出；於捲動事件、可捲區尺寸改變與 tab 清單改變時重算。捲軸維持隱藏。
- 面板開啟與作用中專案改變時，把可捲區的 scrollLeft 設到作用中 tab 完整可見的位置（已完整可見、且不落在淡出區內則不動）；直接設 scrollLeft，不用 scrollIntoView，避免連帶捲動外層。
- 兩個計算寫成純函式：淡出判斷（捲動位置、內容總長、可視長度 → 起點側與終點側是否淡出，容差 1px）放 `apps/desktop/src/panel/edgeFade.ts`，與中段上下緣共用（見下一個決策）；露出目標（tab 左緣、tab 寬、目前 scrollLeft、可捲區寬 → 新的 scrollLeft）放 `apps/desktop/src/panel/tabStrip.ts`。jsdom 沒有版面，元件測試以注入尺寸驗證接線、純函式測試驗證計算。

- 替代方案「固定在左邊」：先看到很少用的動作，且與主視窗圖示列順序相反，被否決。
- 替代方案「不固定」：專案一多就看不到，也不知道它還在，被否決。
- 替代方案「只固定『＋』、不做漸層與自動捲動」：第 5 個以後的專案仍像不存在，作用中的專案也可能不在畫面上，被否決。

### 中段上下緣淡出：與 tab 條共用邊緣淡出

驗收時（[M] 4.1）使用者指出：可捲中段捲到一半時，卡片在頁首與頁尾邊緣被硬切，看起來像被截斷；希望比照 WadPilot 側欄，捲動區上下緣淡出。做法與 tab 條的左右淡出同一套：

- 淡出判斷抽成軸向無關的純函式放 `apps/desktop/src/panel/edgeFade.ts`（捲動位置、內容總長、可視長度 → 起點側／終點側是否淡出，容差 1px），連同遮罩漸層字串（橫向 to right、縱向 to bottom，淡出寬 24px）由 tab 條與中段共用；`tabStrip.ts` 只留露出目標，並沿用同一個淡出寬度當餘裕。
- 面板元件以同一個監看函式套用兩處：監聽捲動事件，並以 ResizeObserver 觀察可捲區本身與其直接子節點——中段的內容 wrapper 隨收合、快照增減改變高度時會觸發重算；tab 條的 tab 尺寸固定，改以 tab 清單改變時重新監看。中段只監看一次，同一組量測同時供上下緣淡出與捲動指示條；捲動時只重繪中段外殼，不重繪卡片。
- 遮罩掛在中段捲動容器（`panel-scroll`）本身，不隨內容捲動；捲動指示條是它的兄弟節點，不被淡出；固定頁首與頁尾在容器之外，也不被淡出。淡出區落在頁首／頁尾與中段之間的 8px 間距上方，內容是漸淡而不是被切斷。

- 替代方案「常駐上下淡出（不判斷方向）」：捲到最上方時第一張卡的標題也被淡掉，被否決。
- 替代方案「中段另寫一份縱向判斷」：與 tab 條的判斷只差軸向，兩份會各自漂移，被否決。

## Implementation Contract

**Behavior**

- ready 或已有 session 的 offline 狀態下，中段由上而下為：（stale 狀態列，若有）、提案中卡、進行中卡、已就緒卡、討論卡。
- 動作區塊卡為單列：「開啟 Speclink」文字鈕在左；「專案設定」「設定」「結束」圖示鈕在右，各帶 `aria-label` 與 `title`（中文介面分別為「專案設定」「設定」「結束」），「設定」與「結束」之間有一條細直線分隔符，點擊行為與現況相同（開主視窗、開專案設定頁、開設定頁、結束 app）。
- 提案中、進行中、已就緒、討論四張卡的標題可點擊切換收合；`aria-expanded` 反映狀態；收合時只剩標題一行（含計數與朝右箭頭）。
- 收合狀態跨「關閉再開啟面板」「切換作用中專案」「重啟 app」保留；沒有記憶時全部展開。
- 收合中的卡在快照更新後仍收合，計數更新為新值。
- 專案 tab 條卡、動作區塊卡、復原卡、stale 狀態列沒有收合控制。
- 面板根層補光層為約六成不透明的側欄面色（`bg-sidebar/60`），三類卡維持不透明白底；毛玻璃自外框與卡片間距透出。
- 「已轉出 N」收合列（系統匣討論卡底部、看板討論欄底部）收合時箭頭為 ›，展開時朝上；內容卡標題的收合箭頭收合時 ›、展開時朝下。
- 專案 tab 條：「加入專案」為固定在右端、「＋」方塊下帶「加入專案」文字的動作項，不隨 tab 捲動；有 tab 未完整顯示的一側淡出；面板開啟與作用中專案改變時捲到作用中 tab 完整可見。
- 可捲中段：還有內容被捲出可視範圍的上緣／下緣淡出，該側沒有被遮住的內容時不淡出；固定頁首與頁尾不被淡出。
- 收合觸發與圖示鈕都不在 Tab 順序內；開啟面板時沒有元素自動取得焦點（既有規格不變）。

**Interface / data shape**

- localStorage 鍵 `speclink.tray.collapsedSections`；值 `JSON.stringify(string[])`，元素 ∈ {`proposed`, `in-progress`, `ready`, `discussions`}。
- `sectionCollapse.ts` 匯出：鍵常數、讀取函式（回傳收合中的 id 集合）、寫入函式（接收完整集合）。
- 沒有新的 TraySnapshot 欄位、IPC command、i18n 詞條。

**Failure modes**

- localStorage 讀取丟例外或值不合法 → 全部展開，不顯示錯誤。
- 寫入丟例外 → 靜默；當次面板內的收合仍生效，只是不保證下次記得。

**Acceptance criteria**（`apps/desktop/src/__tests__/trayPanel.test.tsx`，以 vitest 執行）

- 區塊順序測試：提案中、進行中、已就緒三張卡在討論卡之前，「開啟 Speclink」在最末的動作區塊卡內。
- 動作區塊測試：動作區塊卡內有「開啟 Speclink」文字與三個 `aria-label` 分別為「專案設定」「設定」「結束」的按鈕，各帶同名 `title`；點擊各觸發對應回呼；三顆按鈕的 tabIndex 為 -1；「設定」與「結束」之間有一個 aria-hidden 的分隔符，四顆鈕都不帶額外左外距。
- 收合測試：點「進行中」標題後該卡的變更列消失、`aria-expanded` 為 false、計數仍顯示；再點恢復。
- 記憶測試：收合「討論」後重新掛載面板（模擬重開），討論卡仍收合；換一個作用中專案的快照，仍收合；localStorage 寫入的值為含 `discussions` 的 JSON 陣列。
- 容錯測試：localStorage 存壞值（非 JSON、非陣列）時四張卡全部展開。
- 不自動展開測試：收合「已就緒」後送入多一筆已就緒變更的快照，卡仍收合、計數由 0 變 1。
- 既有測試中以可見文字找「專案設定」「設定」「結束」的斷言，改以按鈕名稱（aria-label）查找。
- 箭頭方向測試：系統匣「已轉出」收合列展開後，箭頭的 class 含 `-rotate-90`、不含單獨的 `rotate-90`，收合時兩者皆無（apps/desktop/src/__tests__/trayPanel.test.tsx 既有「展開後含 rotate-90」斷言改寫）；看板討論欄底部收合列同樣斷言（packages/ui/src/__tests__/discussionColumn.test.tsx 既有斷言改寫）。
- tab 條測試：`panel-add-project` 不在 `panel-project-tabs` 可捲區內、`aria-label` 與 `title` 為「加入專案」、方塊下有「加入專案」文字；以注入 scrollWidth／clientWidth／scrollLeft 驗證捲動事件後可捲區的遮罩隨左右狀態改變；作用中 tab 在可視範圍外時，開啟後可捲區 scrollLeft 改為讓它完整可見。edgeFade.ts 純函式測試涵蓋 Example「淡出的側邊」四列（最左端、中間、最右端、未超出）與 Example「中段淡出的邊緣」四列（最上方、中間、最下方、未超限）；tabStrip.ts 純函式測試涵蓋露出目標（已可見不動、在右側外、在左側外）。
- 中段淡出測試：以注入 scrollTop／scrollHeight／clientHeight 觸發 `panel-scroll` 的捲動事件後，其淡出狀態隨上下是否還有內容改變；`panel-header`、`panel-footer` 不帶遮罩。
- 補光層測試：panel-root 的 class 含 `bg-sidebar/60`、不含 `bg-sidebar/90`；專案 tab 條卡、動作區塊卡與各內容卡仍含 `bg-card`。既有「根層不疊漸層、以側欄面色約九成補光」測試改為約六成。

**Scope boundaries**

- In scope：面板元件（apps/desktop/src/panel/TrayPanel.tsx）的中段順序、動作區塊卡排法、內容卡收合、根層補光層濃度、「已轉出」收合列箭頭方向、專案 tab 條（「＋」固定右端、邊緣淡出、捲到作用中）；新增 apps/desktop/src/panel/tabStrip.ts 與 apps/desktop/src/panel/edgeFade.ts（tab 條與中段共用的邊緣淡出）；可捲中段的上下緣淡出；看板討論欄（packages/ui/src/components/DiscussionColumn.tsx）「已轉出」收合列的箭頭方向；新增 apps/desktop/src/panel/sectionCollapse.ts；對應 vitest 測試；`tray-status-menu` 與 `desktop-app` 規格 delta。
- Out of scope：Rust 端 vibrancy 材質與面板視窗設定（apps/desktop/src-tauri/src/panel.rs）、卡片透明度、非 macOS 原生選單（apps/desktop/src/tray.ts 的選單建構）、看板主視窗、@speclink/ui 中 DiscussionColumn 收合列箭頭以外的共用元件、Rust crates、高度上限與溢出門檻。

## Risks / Trade-offs

- [圖示鈕比文字難認，首次使用者要滑過才知道是什麼] → `title` 顯示名稱、`aria-label` 給輔助技術；三個圖示沿用主視窗側欄的同款圖示，使用者已見過。
- [「結束」是圖示鈕，誤按會直接關掉 app] → 放在最右、以細直線分隔符與其他鈕隔開；行為與現況相同（現況的「結束」列也是一點即結束）。
- [使用者收起「已就緒」後忽略了待收尾的變更] → 收合後計數仍在標題上，0→1 會變；這是使用者自己的選擇，不自動展開是刻意的。
- [既有 vitest 斷言以可見文字找動作列，改為圖示鈕後會失敗] → 本變更同步改寫那些斷言（見 Acceptance criteria）。
- [在途變更 clickable-cursor-hover-rule 會新增守門，掃描 apps/desktop 原始碼中寫死的 cursor-pointer 與 cursor-not-allowed] → 收合觸發與三顆圖示鈕都不寫游標 class，游標交給共用樣式的全域規則（原生 button 與 role="button" 元素）決定；兩個變更修改 tray-status-menu 的不同需求，沒有需求層級的重疊。
- [補光層變薄後，深色桌布下面板外框與卡片縫隙會偏灰暗] → 卡片維持不透明，內容亮度不受影響；六成是當初針對深色背景實測過的值，spec 寫「約六成」容許真機微調（55–65）。
- [「＋」固定後永遠佔約一個 tab 寬（64px 以上），可捲區少一個 tab] → 換來「加入專案」永遠可見且名稱一眼可讀。
- [tray-panel-windows 的 delta 以 REMOVED＋ADDED 複製了「面板樣式」全文] → 本次 tab 條的規格修改已同步套用到 tray-panel-windows 的 ADDED「面板樣式」，兩邊文字一致。
- 回歸對照：沒有 CLI 輸出、golden 檔或 Rust 測試受影響；回歸面只在 desktop 的 vitest。
- 跨平台：面板只在 macOS 使用；Windows／Linux 走原生選單，本變更不碰。localStorage 行為在各平台的 webview 相同。
