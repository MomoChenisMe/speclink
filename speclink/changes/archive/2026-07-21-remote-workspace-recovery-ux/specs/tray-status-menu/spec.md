## MODIFIED Requirements

### Requirement: 選單專案切換
<!-- BEFORE: remote 切換失敗只在看板分頁呈錯誤，系統匣明定不另設錯誤呈現。 -->

選單專案區 SHALL 列出全部已開啟的專案分頁。ready 專案 SHALL 以 check item 呈現且作用中專案帶勾選；restoring remote 專案 SHALL 呈現不可重複觸發的「正在連線」狀態；error 或 needs-reauth remote 專案 SHALL 呈現含 workspace 名稱與繁中狀態摘要的復原 submenu，並於 submenu 內標示該專案是否作用中。點選非作用中的 ready 專案 SHALL 使桌面 app 切換至該專案（看板與選單一致更新），且 SHALL NOT 將主視窗帶到前景或奪取焦點。

切換與復原動作 SHALL 以分頁的 locator key 識別目標，SHALL NOT 以 root 路徑為切換把手。local 與 remote ready 分頁點選 SHALL 一視同仁完成切換，SHALL NOT 因 remote 分頁無本機路徑而靜默無反應。remote handshake 失敗時，分頁 SHALL 成為作用中復原目的地，原生選單 SHALL 轉為復原 submenu，至少提供重新連線或重新登入、開啟問題詳情與伺服器設定；直接重新連線 SHALL NOT 喚起主視窗，只有使用者明確選取開啟詳情、伺服器設定或重新登入 SHALL 顯示主視窗並取得焦點。local 專案目錄失效 SHALL 沿用主視窗既有分頁錯誤處理且 app SHALL NOT 崩潰。

#### Scenario: 點選非作用中專案完成切換且不奪焦

- **WHEN** 使用者於系統匣選單點選一個非作用中的 ready 專案
- **THEN** 看板切換至該專案、選單作用中標記移至該專案，主視窗的前景與焦點狀態不變

#### Scenario: 點選 remote 專案分頁完成切換

- **WHEN** 已開啟的分頁中含一個 ready remote 專案分頁，使用者於系統匣選單點選該非作用中 remote 專案
- **THEN** 看板切換至該 remote 專案、選單作用中標記移至該專案，主視窗的前景與焦點狀態不變

#### Scenario: remote 切換失敗轉為復原 submenu

- **WHEN** 使用者於原生系統匣選單點選 remote 專案，而 handshake 因 server 不可達而失敗
- **THEN** 該專案成為作用中復原目的地，選單項轉為含「無法連線」摘要、重新連線、開啟問題詳情與伺服器設定的 submenu，app 未崩潰

#### Scenario: 原生選單直接 retry 不奪焦

- **WHEN** 主視窗隱藏或位於背景，使用者於 error workspace submenu 選取重新連線
- **THEN** workspace 轉為 restoring 並重走 handshake，主視窗維持原顯示與焦點狀態；失敗時 submenu 原位更新，成功時回到 ready 專案項

#### Scenario: 原生選單顯式詳情動作取得焦點

- **WHEN** 使用者於 error workspace submenu 選取開啟問題詳情、伺服器設定或重新登入
- **THEN** 主視窗顯示並取得焦點，分別開啟該 workspace 復原頁、對應 server 設定或對應 connection 登入流程

### Requirement: 面板樣式（macOS）
<!-- BEFORE: remote tab 切換失敗只沿用看板錯誤且面板明定不另設錯誤呈現。 -->

於 macOS，點擊系統匣圖示 SHALL NOT 顯示原生下拉選單，而 SHALL 於圖示下方彈出貼齊圖示的面板視窗，再次點擊 SHALL 收合——無需任何偏好設定。點擊 SHALL 不分滑鼠按鍵：主鍵（左鍵）與次要鍵（右鍵）點擊圖示 SHALL 完全等價，皆為開閉面板。面板內容 SHALL 與原生選單同源（同一前端 store 投影：專案與復原狀態、生命週期分區的變更與進度、討論——討論比照原生選單分「討論」與「已轉出」兩分區），SHALL NOT 為面板另建第二條資料查詢路徑或狀態機。

專案區 SHALL 呈現為橫向 tab 條：每個 tab SHALL 顯示專案名首字母的圓角方塊 avatar 與專案名；remote tab SHALL 另以圖示與文字可辨識 ready、restoring、offline、needs-reauth 或 error，SHALL NOT 只用顏色表意。作用中 ready 專案的 tab SHALL 以實心主色底＋反白文字標示；作用中 restoring／error 專案 SHALL 維持同等清楚的 selected 狀態且不呈 disabled 外觀。tab 總寬超出面板時 SHALL 可橫向捲動且 SHALL NOT 顯示捲軸。

點擊 tab SHALL 原地切換作用中專案，SHALL NOT 喚起主視窗、SHALL NOT 收合面板；切換 SHALL 以分頁的 locator key 為把手（與「選單專案切換」需求同語意），local 與 remote 專案 tab 一視同仁，SHALL NOT 因分頁無本機路徑而靜默無反應。active remote tab 尚無 session且為 restoring／error 時，面板 SHALL 以一張精簡復原卡取代討論與生命週期分區；復原卡 SHALL 顯示 workspace、server、繁中狀態摘要及重新連線或重新登入、在 Speclink 中查看詳情／設定的動作，SHALL NOT 顯示上一個 workspace 的資料。直接重新連線 SHALL 由面板動作回流主視窗 store、面板保持開啟且 SHALL NOT 喚起主視窗；使用者明確選取詳情、設定或重新登入 SHALL 顯示主視窗並取得焦點。active remote session 為 offline 時，面板 SHALL 顯示 stale 狀態列並保留最後成功的變更與討論內容，等待既有 worker 自動收斂。

tab 條尾端 SHALL 有「加入專案」動作項：點擊 SHALL 先顯示主視窗（含切換至其所在桌面——確保後續對話框於使用者眼前可見）再開啟資料夾選擇器（與主視窗「開啟專案」同語意）——選定即以分頁加入該專案並成為作用中專案，取消則無任何變化。資料夾選擇器等系統原生對話框 SHALL 跟隨系統語言呈現（app SHALL 宣告繁體中文在地化，不得固定英文介面）。

active workspace 為 ready 或已有 session 的 offline 時，面板內容 SHALL 依區塊排列：專案 tab 條之下依序為討論區塊（「討論」分區常駐呈現，其後「已轉出」分區有料才現）、生命週期區塊（提案中→進行中→已就緒）、動作區塊（「開啟 Speclink」「設定」「結束」）。專案 tab 條與討論區塊之間、討論區塊與生命週期區塊之間、生命週期區塊與動作區塊之間 SHALL 各有一條分割線（共三條）；區塊內部（分區卡之間）SHALL NOT 出現分割線。此區塊順序為面板刻意設計；原生選單的區段順序仍依「系統匣圖示與原生選單」需求（生命週期分區在前、討論區在後），不受本段影響。無 session 的 restoring／error 狀態 SHALL 以「tab 條、分割線、復原卡、分割線、動作區塊」排列，不渲染討論與生命週期空卡。

生命週期分區與討論分區 SHALL 各自以半透明圓角卡片容器呈現（面板毛玻璃底 SHALL 可透出），分區標題 SHALL 含主色上色的分區圖示，並 SHALL 顯示該分區的項目計數（徽章樣式與看板欄計數同語彙）。生命週期三個階段分區（提案中／進行中／已就緒）SHALL 常駐呈現：零筆階段 SHALL 以「分區標題＋計數 0」的空狀態卡呈現，SHALL NOT 因該階段無變更而整卡消失；全無變更時 SHALL NOT 顯示佔位卡（原「尚無進行中變更」），而以三張計數 0 的分區卡呈現，分區順序固定為提案中→進行中→已就緒。「已轉出」分區 SHALL 維持有料才現——零筆時 SHALL NOT 呈現（與「討論列表」需求一致）。空狀態卡（討論零筆、生命週期零筆階段）SHALL 維持最小高度、內容垂直置中，不得塌陷成細條。有任務的變更列，其進度條填色 SHALL 依階段套用與看板同源的主色深淺階梯（提案中最淺、進行中次之、已就緒最深）。

變更與討論列 SHALL 於列尾常駐複製鈕（複製內容與原生選單的複製動作一致：變更為 name、討論為 slug）；複製鈕點擊後 SHALL 短暫顯示成功回饋（勾號圖示，與看板複製鈕同模式）後自行復原。點擊變更或討論列本體 SHALL 顯示主視窗並開啟對應詳情。

面板開啟時 SHALL NOT 有任何互動元素自動取得焦點（不得出現系統焦點框）；複製鈕 SHALL NOT 可經 Tab 鍵取得焦點。新增的復原 tab、card 與動作 SHALL 提供語意化 label 與清楚 pointer hit area，但 SHALL NOT 將面板改為會奪取前景 app 焦點的 key window。面板高度 SHALL 自適應內容（隨內容增減貼合，達上限高度後面板內部捲動、不得於內容未超限時出現多餘捲動與空白）。面板開啟 SHALL NOT 奪取目前前景 app 的焦點；面板失焦時 SHALL 自動收合。面板視窗建立失敗時 app SHALL 以原生選單樣式運作（選單實作跨平台保留、兼作 macOS 失敗後備）並於設定頁本機設定簽浮出單行錯誤。

#### Scenario: 面板樣式下點擊圖示彈出貼齊面板

- **WHEN** 使用者於 macOS 點擊系統匣圖示
- **THEN** 圖示下方彈出貼齊圖示的面板，頂部為專案 tab 條，其下依作用中 workspace 狀態呈正常資料分區或復原卡，尾端為動作區塊，未出現原生下拉選單

#### Scenario: 右鍵點擊圖示與左鍵等價

- **WHEN** 使用者於 macOS 以滑鼠次要鍵（右鍵）點擊系統匣圖示
- **THEN** 面板於圖示下方彈出且貼齊位置與左鍵點擊一致；面板已開啟時再以右鍵點擊圖示則面板收合——與左鍵行為完全相同

#### Scenario: ready workspace 的區塊順序與分割線

- **WHEN** 面板開啟，作用中 ready 專案存在討論中討論、已轉出討論與各階段變更
- **THEN** 由上而下依序為：專案 tab 條、分割線、「討論」分區、「已轉出」分區、分割線、「提案中」「進行中」「已就緒」分區、分割線、「開啟 Speclink」「設定」「結束」；分割線恰為三條且僅出現於區塊之間、分區卡之間無分割線

#### Scenario: 點擊專案 tab 原地切換

- **WHEN** 面板開啟且有兩個以上 ready 專案分頁，使用者點擊非作用中專案的 tab
- **THEN** 該 tab 轉為實心主色的作用中標示，面板下方的變更與討論內容切換為該專案，主視窗未被喚起、面板保持開啟

#### Scenario: 點擊 remote 專案 tab 原地切換

- **WHEN** 面板開啟且分頁中含一個非作用中的 ready remote 專案，使用者點擊該 remote 專案的 tab
- **THEN** 該 tab 轉為作用中標示，面板下方的變更與討論內容切換為該 remote 專案，主視窗未被喚起、面板保持開啟

#### Scenario: remote handshake 失敗顯示復原卡

- **WHEN** 面板中點擊一個無 session 的 remote tab，handshake 因 server 不可達而失敗
- **THEN** 該 tab 維持作用中 error 狀態，面板以 workspace／server／繁中摘要與復原動作卡取代討論及生命週期分區，未顯示上一 workspace 資料，主視窗未被喚起且面板保持開啟

#### Scenario: 面板 retry 原地恢復

- **WHEN** 使用者於 error 復原卡選取重新連線且 handshake 成功
- **THEN** tab 先呈 restoring，成功後轉 ready 並恢復該 workspace 的討論與生命週期分區，面板全程保持開啟且主視窗未被喚起

#### Scenario: 面板顯式開啟詳情或重新登入

- **WHEN** 使用者於復原卡選取在 Speclink 中查看詳情、伺服器設定或重新登入
- **THEN** 主視窗顯示並取得焦點，開啟對應 workspace 復原頁、server 設定或 connection 登入流程

#### Scenario: 已建立 session 離線保留 Panel stale 內容

- **WHEN** 作用中 remote session 已載入內容後進入 offline
- **THEN** 面板顯示 offline／stale 狀態列並保留最後成功的討論與生命週期內容，未改為無 session 復原卡

#### Scenario: tab 條尾端快速加入專案

- **WHEN** 主視窗位於另一個桌面或未在前景，使用者點擊 tab 條尾端的「加入專案」項並於資料夾選擇器選定一個專案目錄
- **THEN** 主視窗先被喚起（桌面切換至其所在處）、資料夾選擇器於前景出現；選定後該專案以分頁加入並成為作用中專案；於選擇器按取消則分頁無任何變化

#### Scenario: 分區標題顯示項目計數

- **WHEN** ready workspace 的面板列出提案中 1 筆變更、討論 0 筆
- **THEN** 「提案中」分區標題帶計數徽章 1；討論空狀態卡顯示計數 0 且維持最小高度、內容垂直置中

#### Scenario: 全無變更時三個生命週期分區常駐

- **WHEN** 作用中 ready 專案沒有任何變更，使用者於 macOS 開啟面板
- **THEN** 面板依序呈現「提案中」「進行中」「已就緒」三張分區卡，各帶計數徽章 0、維持最小高度且內容垂直置中，未出現「尚無進行中變更」佔位卡

#### Scenario: 部分有資料時空階段分區仍常駐

- **WHEN** 作用中 ready 專案僅有 1 個進行中變更，無提案中與已就緒變更
- **THEN** 「進行中」分區卡帶計數徽章 1 並列出該變更；「提案中」與「已就緒」分區卡仍呈現且各帶計數徽章 0，三張分區卡依提案中→進行中→已就緒順序排列

#### Scenario: 進度條依階段深淺

- **WHEN** ready workspace 的面板同時列出提案中與進行中各一個有任務的變更
- **THEN** 兩列進度條填色同為主色但深淺不同——提案中較淺、進行中較深，與看板欄位的階段配色同階梯

#### Scenario: 開啟面板無預設焦點

- **WHEN** 使用者點擊系統匣圖示開啟面板
- **THEN** 面板中無任何元素帶系統焦點框（含第一顆複製鈕或復原動作），可點擊動作仍有語意 label 與可見回饋

#### Scenario: 面板不搶焦點且失焦自動收合

- **WHEN** 使用者於其他 app 位於前景時點擊系統匣圖示開啟面板，隨後點擊面板外任意處
- **THEN** 面板開啟期間原前景 app 保持焦點；點擊面板外後面板自動收合

#### Scenario: 面板內以常駐複製鈕複製

- **WHEN** 使用者點擊面板中某討論列列尾的複製鈕（無需 hover 顯示、常駐可見）
- **THEN** 系統剪貼簿內容等於該討論的 slug，複製鈕短暫轉為勾號回饋後復原，面板保持開啟、未開啟主視窗

#### Scenario: 面板高度自適應內容

- **WHEN** 面板開啟且正常內容或復原卡高度少於一屏
- **THEN** 面板高度貼合內容（下方無大片空白），內容增加超過上限高度後面板內部出現捲動

#### Scenario: 面板建立失敗退回原生選單

- **WHEN** macOS 上面板視窗建立失敗
- **THEN** 系統匣以原生選單樣式運作，remote error workspace 仍可經復原 submenu 操作，設定頁本機設定簽浮出單行面板建立錯誤
