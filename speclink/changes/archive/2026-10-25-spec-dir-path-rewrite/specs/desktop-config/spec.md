## MODIFIED Requirements

### Requirement: 執行期切換專案 root

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

桌面 app SHALL 提供「開啟專案」操作：經原生資料夾選擇器選定目錄後，app SHALL 以該目錄為起點向上探索 speclink 專案根（與啟動時的探索語意一致），命中即於執行期切換專案 root——看板、已封存與設定頁 SHALL 全數改為呈現新專案內容，SHALL NOT 要求重啟；頂欄分頁列 SHALL 以 active 分頁標示目前專案（root 目錄名）。開啟失敗（路徑不存在或不可讀）時 app SHALL 顯示單行錯誤訊息並維持原專案不變。

#### Scenario: 選定既有專案後全視圖切換

- **WHEN** 使用者於專案 A 中經「開啟專案」選定含 speclink/ 的專案 B 根目錄
- **THEN** 看板顯示專案 B 的 change、分頁列新增專案 B 的分頁並標示為 active，過程中 app 未重啟，且專案 A 與 B 的檔案內容均未被此操作改動

#### Scenario: 自子目錄向上探索至專案根

- **WHEN** 使用者選定專案 B 根目錄下的子目錄（子目錄本身不含 speclink/）
- **THEN** app 切換至向上探索命中的專案 B 根目錄，行為與直接選定專案根一致

#### Scenario: 開啟失敗維持原專案

- **WHEN** 使用者嘗試開啟一個已被刪除或不可讀的路徑
- **THEN** app 顯示單行錯誤訊息，目前專案與畫面內容維持不變

### Requirement: 專案分頁列存於 app 本機

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

app SHALL 以圖示列的專案方塊呈現目前開著的專案(每個專案一個 32px 方塊,內容為顯示名首字母;上限 10 個):開著的專案 SHALL 跨啟動持久化於 app 本機狀態(含順序與最後作用中專案),SHALL NOT 寫入任何專案目錄。點擊方塊 SHALL 以該 locator 執行與「開啟專案」相同的切換語意;同一專案再次開啟 SHALL 去重並移至既有方塊;關閉專案(標題列「⋯」或方塊右鍵選單的「關閉專案」)SHALL 將其自持久化清單移除,但 SHALL NOT 改變最近開啟記錄(曾開啟過的專案由 workspace-chooser 的「最近開啟清單」需求另行記憶,專案方塊 SHALL NOT 作為唯一的最近開啟記憶)。方塊 SHALL NOT 顯示待收尾數或其他計數徽章——待收尾狀態由看板欄位計數與系統匣面板分區計數承載。方塊 hover SHALL 以提示顯示完整顯示名與路徑(remote 專案顯示 checkout 路徑,無 checkout 時顯示連線與 Project/Repo)。方塊指向已不存在的路徑時 SHALL 以錯誤態呈現(方塊角落紅點、提示附錯誤原因),點擊 SHALL 開啟含錯誤訊息與「自專案列移除」的選單,SHALL NOT 切換專案;背景 local 專案 SHALL 於 app 啟動時各探測一次路徑有效性,失效即轉錯誤態。remote 專案離線或需重新登入時方塊角落 SHALL 為琥珀點。無任何專案時 app SHALL 不渲染專案欄,主區 SHALL 顯示「開啟一個專案開始」空狀態(字標、標題、說明、「新增專案」與「連線 Server」兩鈕、最近開啟卡)而非空看板。app SHALL 支援 Ctrl+Tab 循環切換與 Ctrl+1..9 直達第 N 個專案。

#### Scenario: 成功開啟後記入分頁並去重上移

- **WHEN** 使用者依序開啟專案 A、B,再次開啟專案 A
- **THEN** 圖示列僅含 A、B 各一個方塊且 A 為作用中(teal 底白字),A 與 B 的專案目錄內均無因此新增的檔案;重啟 app 後圖示列還原為相同內容與順序

#### Scenario: 點擊分頁切換專案

- **WHEN** 使用者於專案 A 為作用中時點擊專案 B 的方塊(或按 Ctrl+Tab 循環至 B)
- **THEN** 看板、已封存、規格、手冊與專案設定頁改為呈現專案 B 內容,B 的方塊轉為作用中、左側標題列與麵包屑的專案名改為 B,行為與經「開啟專案」選定 B 一致

#### Scenario: 分頁不顯示計數徽章

- **WHEN** 圖示列含專案 A(作用中,2 個已就緒變更與 1 份已結論未轉出討論)與背景專案 B
- **THEN** A 與 B 的方塊均只顯示首字母與必要的角落狀態點,無任何計數徽章或待收尾數 hover 說明

#### Scenario: 分頁路徑已消失時轉錯誤態

- **WHEN** 圖示列中專案 B 的目錄已被刪除,使用者點擊該方塊
- **THEN** 該方塊呈現錯誤態(角落紅點),點擊彈出選單首列為單行錯誤訊息、其後只有「自專案列移除」,app 維持原專案;執行移除後該方塊自圖示列與持久化清單消失

#### Scenario: 方塊提示顯示全名與路徑

- **WHEN** 滑鼠停在本機專案 B 的方塊上;另停在無 checkout 的 remote 專案 R 的方塊上
- **THEN** 前者提示顯示 B 的顯示名與完整路徑;後者顯示 R 的顯示名與連線名稱加 Project/Repo

#### Scenario: 零分頁時顯示空狀態引導頁

- **WHEN** app 於無任何持久化專案的狀態下啟動(如首次使用)
- **THEN** 左側只有圖示列(品牌標記、「＋」、設定),無專案欄;主區置中顯示字標、「開啟一個專案開始」標題、說明(含「資料夾還沒有 speclink/ 時會先問你要不要初始化」)、「新增專案」與「連線 Server」兩鈕,而非空白看板

### Requirement: 設定頁圖形化讀寫兩層設定

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

設定 SHALL 拆分為兩種模式：**專案設定**（跟隨 active 專案，自專案欄底部「專案設定」進入）與**應用程式設定**（與任何專案無關，自圖示列齒輪進入）。兩種模式 SHALL 以左側分類欄取代頁簽：點分類即切換主區為該分類的單一頁，分類欄 SHALL 記住上次所在分類；每頁 SHALL 以共用設定零件排版——24px 頁標題＋灰字說明＋右側頁級動作、14px 小標＋16px 圓角卡、卡內一列一筆細線分隔（左標籤與說明、右控制項）、列右側下拉。

專案設定的分類欄 SHALL 分兩組：「speclink/config.yaml」組依序為專案說明、產出規則、審查規則、產出流程；「專案」組為 .speclink.yaml；預設落在專案說明。

- **專案說明**頁 SHALL 含專案說明卡、**產出規則**頁 SHALL 含產出規則卡（行為見需求「設定頁編輯專案說明與產出規則」）。
- **審查規則**頁 SHALL 含預設清單卡與專案規則卡（行為見需求「設定頁編輯審查規則」）。
- **產出流程**頁 SHALL 含「產出政策」卡——locale、spec_locale（列右側下拉）與 tdd、audit、worktree（列右側開關）——與「產出流程」卡（行為見需求「產出流程頁」）。
- **.speclink.yaml** 頁 SHALL 含「檔案」列（等寬路徑，唯讀）與「AI 工具」卡——內建工具 claude／codex／copilot 每個一列附開關，自訂工具描述子原樣呈現為不可編輯列。

應用程式設定的分類欄 SHALL 為「應用程式」組：一般、更新、CLI、伺服器，其後為「關於」組：關於 Speclink（行為見 desktop-app 需求「關於頁」）；預設落在一般；零專案時 SHALL 照常可進入且分類欄照常呈現。

- **一般**頁 SHALL 含「語言與外觀」卡（顯示語言列、外觀列；行為見需求「UI 介面語言支援 zh-TW 與 en」「外觀偏好」）與「鍵盤快捷鍵」卡，並 SHALL 以頁標題下的註記說明其內容僅存於此裝置、不寫入版本庫；SHALL NOT 含軟體更新卡與 CLI 指令卡。
- **更新**頁行為見 desktop-app 需求「桌面自動更新」「更新日誌彈窗」。
- **CLI**頁行為見 desktop-app 需求「安裝 CLI 指令到 PATH」。
- **伺服器**頁行為見 desktop-connections 需求「伺服器管理最小面」。

應用程式設定的同一個設定列 SHALL 只出現在一個分類頁，SHALL NOT 在其他分類頁重複。

專案設定每頁的頁標題下 SHALL 以等寬字註記對應檔案路徑與鍵（如 `speclink/config.yaml · context`）。讀取時未設定的欄位 SHALL 呈現為預設值狀態；寫入時 SHALL 僅代換目標鍵——未觸及的鍵（remote、spec_dir、自訂工具描述子等）SHALL 原樣保留；政策欄位設回預設值時 SHALL 移除該鍵而非寫入明值。tools 寫入成功後 app SHALL 同步技能檔（新選工具生成、取消工具清理殘留）。自訂工具描述子 SHALL 寫入後保留。任一層設定檔解析失敗時，分類欄中掛該檔的分類項（config.yaml 掛專案說明、產出規則、審查規則、產出流程；.speclink.yaml 掛 .speclink.yaml）SHALL 帶警示點、頁內 SHALL 浮出解析失敗說明且該頁表單 SHALL 停用；應用程式設定 SHALL NOT 受任何專案設定檔解析失敗影響。

遠端 workspace 為 active 專案時，專案設定的分類欄 SHALL 只有「speclink/config.yaml」組的四項（無 .speclink.yaml——tools 屬本機 checkout 概念），四頁內容來自 server 的 policy 文件，頁標題下的註記 SHALL 以等寬字顯示 policy revision；鍵保留語意與本地一致（未觸及鍵原樣保留、設回預設移除鍵）。儲存 SHALL 帶 expected revision；收到 revision 衝突時 SHALL 原樣保留使用者輸入並於頁頂浮出逐欄位對照（server 現值｜我的輸入），僅提供「以 server 版重載」與「檢視後以最新 revision 重新提交」兩出口——SHALL NOT 提供未經對照的強制覆寫。policy 寫入權為假（reader）時四頁 SHALL 唯讀、儲存停用附繁中角色說明。應用程式設定不受 active 專案種類影響。

#### Scenario: 兩種設定模式與預設分類

- **WHEN** 使用者於 local 專案分別點專案欄「專案設定」與圖示列齒輪
- **THEN** 專案設定的分類欄為「speclink/config.yaml」組（專案說明、產出規則、審查規則、產出流程）與「專案」組（.speclink.yaml），預設在專案說明頁（頁標題下等寬註記 `speclink/config.yaml · context`）；切到產出流程頁見產出政策卡與產出流程卡、切到 .speclink.yaml 頁見檔案列與 AI 工具卡；應用程式設定的分類欄為「應用程式」組（一般、更新、CLI、伺服器）與「關於」組（關於 Speclink），預設在一般頁（含語言與外觀、鍵盤快捷鍵兩卡與「僅存於此裝置」註記，無軟體更新卡與 CLI 指令卡）

#### Scenario: 遠端專案的專案設定分類

- **WHEN** active 專案為遠端 workspace 且使用者具 policy 寫入權，開啟專案設定修改產出流程頁的產出政策並儲存
- **THEN** 分類欄只有專案說明、產出規則、審查規則、產出流程四項（頁標題下等寬顯示 policy revision），儲存成功後 revision 前進，server 端 config 反映新政策且未觸及鍵原樣保留

#### Scenario: 遠端儲存遇 revision 衝突

- **WHEN** 兩個 client 同時編輯同一 scope 的 policy，後儲存者收到 revision 衝突
- **THEN** 後儲存者的輸入原樣保留，頁頂浮出 server 現值與我的輸入的逐欄位對照，可選以 server 版重載或以最新 revision 重新提交；無任何未經對照的強制覆寫路徑

#### Scenario: reader 唯讀

- **WHEN** role 為 reader 的使用者於遠端專案開啟專案設定
- **THEN** 四頁呈現現值但唯讀，儲存停用並附角色說明

#### Scenario: 寫入政策欄位且未觸及鍵原樣保留

- **WHEN** config.yaml 原含 rules 區塊與 context 文字，使用者於產出流程頁將 tdd 開關切為開啟並儲存
- **THEN** 重新讀取 config.yaml 可見 tdd: true，且 rules 與 context 內容與寫入前逐字元一致

#### Scenario: 設回預設值即移除鍵

- **WHEN** config.yaml 原含 locale: tw，使用者於產出流程頁將 locale 改回「未設定（English）」並儲存
- **THEN** 重新讀取 config.yaml 已無 locale 鍵，且引擎解析該檔的有效 locale 為預設 English

##### Example: 政策欄位寫入效果

| 操作前檔案狀態 | 表單操作 | 寫入後檔案效果 |
| -------------- | -------- | -------------- |
| 無 tdd 鍵 | tdd 切開啟 | 新增 tdd: true |
| tdd: true | tdd 切關閉 | tdd 鍵被移除（預設即 false） |
| locale: tw、含 rules 區塊 | spec_locale 選 auto | 新增 spec_locale: auto，locale 與 rules 原樣保留 |

#### Scenario: tools 變更後技能同步

- **WHEN** .speclink.yaml 原 tools 僅 claude，使用者於 .speclink.yaml 頁開啟 codex 的開關並儲存
- **THEN** .speclink.yaml 的 tools 記錄 claude 與 codex，且專案根新增 .agents/skills/ 技能檔而無 AGENTS.md

#### Scenario: 加選與取消 copilot 後技能同步

- **WHEN** .speclink.yaml 原 tools 僅 claude，使用者開啟 copilot 並儲存，之後再關閉 copilot 並儲存
- **THEN** 第一次儲存後 tools 記錄 claude 與 copilot、專案根新增 .github/skills/ 技能檔；第二次儲存後 tools 僅含 claude，.github/skills/ 下 speclink- 前綴目錄全數移除，.github/ 下其他檔案位元級不變

#### Scenario: 自訂工具描述子原樣保留

- **WHEN** .speclink.yaml 的 tools 含一個自訂描述子物件，使用者於 .speclink.yaml 頁變更內建工具開關並儲存
- **THEN** 寫入後的 tools 清單仍含該描述子且欄位內容不變，頁內將其呈現為不可編輯列

#### Scenario: 解析失敗分類級警示

- **WHEN** config.yaml 被外部改壞為無法解析，使用者開啟專案設定
- **THEN** 分類欄的專案說明、產出規則、審查規則、產出流程四項帶警示點；進入任一頁可見解析失敗說明，產出政策卡表單與專案說明、產出規則、審查規則三頁的編輯鈕停用；應用程式設定的顯示語言列仍可正常使用

### Requirement: 產出政策的 worktree 開關

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

local workspace 的產出流程頁產出政策卡 SHALL 顯示 worktree 開關（列右側開關），文案於 zh-TW 與 en 介面語言下均直出「worktree」一詞（LANGUAGE.md 明文例外）。存檔 SHALL 以開關的畫面實值寫入 speclink/config.yaml 的 worktree 鍵（汰換「UI 無此欄位時恆回填原值」的保值行為），寫入成功後 SHALL 觸發與 CLI workflow-config set 同一技能足跡同步；設定頁載入時 SHALL 反映 config 現值。由開改關存檔且存在活躍 linked worktree 時，SHALL 拒絕寫入並浮出擋下訊息——列出各活躍 worktree 的 change 名、分支與路徑，提示先執行 worktree-merge 收尾——config 維持不變。remote 工作區的設定頁 SHALL NOT 顯示此開關。

#### Scenario: 開關顯示且存檔生效

- **WHEN** local workspace 於產出流程頁將 worktree 開關切為開啟並存檔
- **THEN** speclink/config.yaml 的 worktree 鍵為 true，技能足跡出現兩顆 worktree 技能，設定頁重新載入後開關維持開啟

#### Scenario: 關閉遇活躍 worktree 浮出擋下

- **WHEN** 存在活躍 linked worktree 時於設定頁將 worktree 開關切為關閉並存檔
- **THEN** 存檔失敗浮出，訊息列出各 worktree 的 change 名、分支與路徑及先收尾的指引，speclink/config.yaml 不變，開關回復為開啟

#### Scenario: CLI 先行寫入後設定頁不吃鍵

- **WHEN** 以 CLI 將 worktree 設為 true 後，開啟設定頁再存檔
- **THEN** 設定頁載入時開關即呈現開啟，存檔後 speclink/config.yaml 的 worktree 鍵維持 true

#### Scenario: remote 工作區不顯示開關

- **WHEN** remote 工作區開啟產出流程頁的產出政策卡
- **THEN** 不顯示 worktree 開關，其餘政策欄位照常

### Requirement: 產出流程的切換寫入

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

產出流程頁的產出流程卡 SHALL 提供下拉切換專案 schema：選定後把 schema 鍵寫入 speclink/config.yaml，寫入 SHALL 複用引擎的 byte-preserving setter（set_workflow_schema_text）——其餘內容逐位元組保留、無法解析的文件拒寫。local 模式直寫檔案；remote 模式 SHALL 走既有 revision 守門的 config 寫入通道，revision 落後時顯性失敗。切換成功後產出規則分節的固定鍵 SHALL 隨新 schema 的 artifact 圖更新——例外：產出規則正在編輯中時，編輯面 SHALL 凍結在開編輯當下的分節（草稿不因換集而丟棄或清空），固定鍵於該次編輯儲存或取消後才跟上；編輯期間換入的新固定鍵其既有規則 SHALL 在儲存時原樣保留。

#### Scenario: 切換寫入且其餘內容保留
- **WHEN** 使用者把專案 schema 從 spec-driven 切到自訂 schema
- **THEN** config.yaml 的 schema 鍵更新為該名稱，檔內其餘既有內容逐位元組不變，產出規則分節改列新 schema 的 artifact id

#### Scenario: 壞檔拒寫顯性失敗
- **WHEN** config.yaml 無法解析且使用者嘗試切換
- **THEN** 寫入被拒、錯誤浮出於表單，檔案一個位元組不變

### Requirement: 產出流程的客製 fork

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

產出流程頁的產出流程卡 SHALL 提供 fork 動作（僅 local 模式顯示）：把選中的 schema 複製到專案 speclink/schemas/ 下，複用引擎既有的 fork 函式（複本名為引擎預設 <source>-custom，不收自訂名）；成功後清單 SHALL 即時反映新的專案層 schema。同名跨層時 fork 動作 SHALL 只出現在引擎解析會命中的那一層（project→user→內建的第一命中，含壞檔——引擎的層命中只看檔案存在）：被 shadow 的清單項不提供 fork，避免複製到前層內容。remote 模式 SHALL 不顯示 fork 動作。

#### Scenario: fork 產出專案層複本
- **WHEN** 使用者在 local 模式對 spec-driven 按下 fork
- **THEN** speclink/schemas/spec-driven-custom/ 建立（schema.yaml 與 templates 目錄），清單新增該專案層項目

#### Scenario: remote 模式無 fork 入口
- **WHEN** 工作區連線 remote store 且使用者開啟產出流程頁
- **THEN** 介面不出現 fork 動作

### Requirement: 產出流程的建立

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

產出流程頁的產出流程卡 SHALL 提供建立動作（僅 local 模式顯示）：收 kebab-case 名稱，呼叫引擎既有的 init_schema 在專案 speclink/schemas/ 下產出預設骨架（schema.yaml 與 templates/ 內每個 artifact 的範本檔）；成功後清單 SHALL 即時反映新的專案層 schema。名稱驗證 SHALL 由引擎承擔（前端不重複規則）：名稱不合法或目標已存在時 SHALL 浮出引擎的錯誤訊息且磁碟不變。建立 SHALL NOT 提供 artifact 佈局輸入——骨架佈局用引擎預設，內容客製交外部編輯器。remote 模式 SHALL 不顯示建立動作。

#### Scenario: 建立產出專案層骨架
- **WHEN** 使用者在 local 模式輸入名稱 my-flow 並送出建立
- **THEN** speclink/schemas/my-flow/ 建立（schema.yaml 與 templates/ 內引擎預設 artifact 的範本檔），清單新增該專案層項目

#### Scenario: 不合法名稱顯性失敗
- **WHEN** 使用者輸入非 kebab-case 名稱（如 My Flow）並送出建立
- **THEN** 引擎的名稱錯誤訊息浮出於表單，speclink/schemas/ 無任何新目錄

##### Example: 建立輸入與結果
| 輸入名稱 | 結果 |
|----------|------|
| my-flow | speclink/schemas/my-flow/ 骨架建立，清單新增專案層項目 |
| My Flow | 拒絕：引擎 kebab-case 錯誤浮出，磁碟不變 |
| my-flow（已存在） | 拒絕：引擎 already exists 錯誤浮出，磁碟不變 |

#### Scenario: remote 模式無建立入口
- **WHEN** 工作區連線 remote store 且使用者開啟產出流程頁
- **THEN** 介面不出現建立動作

### Requirement: 產出流程的編輯入口

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

產出流程頁的清單項 SHALL 對有磁碟路徑的 schema（專案層與 user 層）提供「開啟所在資料夾」動作（僅 local 模式顯示）：按下後在系統檔案管理器顯示該 schema 的目錄（schema.yaml 與 templates/ 所在處），內容編輯交外部編輯器。內建 schema（內嵌於程式、無磁碟檔案）SHALL 不顯示此動作。快照的每個清單項 SHALL 帶其 schema 目錄的絕對路徑（內建為空）——user 層路徑由快照組裝端解析，前端不自行拼路徑。remote 模式 SHALL 不顯示此動作。

#### Scenario: 專案層項目開啟所在資料夾
- **WHEN** 使用者在 local 模式對建立出的專案層 schema 按下開啟所在資料夾
- **THEN** 系統檔案管理器顯示 speclink/schemas/<name>/ 目錄

#### Scenario: 內建項無編輯入口
- **WHEN** 使用者在 local 模式檢視內建 spec-driven 的清單項
- **THEN** 該項不出現開啟所在資料夾動作

### Requirement: 產出流程的刪除

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

產出流程頁的產出流程卡 SHALL 對專案層項目提供刪除動作（僅 local 模式顯示；內建無檔案、user 層跨專案共用，均不提供）：按下 SHALL 先開確認對話框，取消 SHALL 零變動；確認後移除專案 speclink/schemas/<name>/ 整個目錄，成功後清單 SHALL 即時反映。刪除目標 SHALL 由名稱固定解析為專案層目錄（不接受任意路徑）。config 的 schema 鍵正指著的 schema（使用中）SHALL 拒刪並浮出顯性錯誤、磁碟不變。remote 模式 SHALL 不顯示刪除動作。

#### Scenario: 刪除經確認後移除專案層目錄
- **WHEN** 使用者對非使用中的專案層 schema 按刪除並在確認對話框按下確認
- **THEN** speclink/schemas/<name>/ 整個目錄移除，清單不再列出該項

#### Scenario: 取消確認零變動
- **WHEN** 使用者按刪除後在確認對話框取消
- **THEN** 磁碟與清單皆無任何變動

#### Scenario: 使用中的 schema 拒刪
- **WHEN** config.yaml 的 schema 鍵指著 my-flow 且使用者確認刪除 my-flow
- **THEN** 錯誤浮出於表單、speclink/schemas/my-flow/ 原封不動

### Requirement: 設定頁編輯審查規則

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

專案設定的「審查規則」頁 SHALL 呈現兩張卡——預設清單卡與專案規則卡——預設唯讀並有自身的編輯態，一頁進入編輯 SHALL NOT 影響其他專案設定頁的唯讀呈現與可用性；頁標題下 SHALL 以等寬字註記 `speclink/config.yaml · review`。介面文案 SHALL 隨 UI 介面語言（zh-TW、en）切換，嚴重度籤 CRITICAL、WARNING、SUGGESTION 與 smell 名稱維持英文。

唯讀呈現：預設清單卡 SHALL 顯示目前模式（「沿用並追加」或「整份換掉」）與一句說明，並 SHALL 提供「查看 12 條」展開鈕，展開後列出 12 條 smell 名稱——名稱 SHALL 取自引擎提供的清單，desktop SHALL NOT 另存一份。專案規則卡 SHALL 依 CRITICAL、WARNING、SUGGESTION 三節列出規則，每節帶嚴重度籤、一行效果說明與條數；效果說明 SHALL 為：CRITICAL「擋蓋章；要接受放行須逐條再確認」、WARNING「擋蓋章；偶有正當例外可接受放行」、SUGGESTION「只記在工單，不擋蓋章」。三節皆空時 SHALL 顯示空狀態。

編輯：頁標題右側的編輯鈕 SHALL 切換本頁為編輯態並將按鈕列改為取消與儲存（儲存鈕標示 ⌘S／Ctrl+S，該快捷鍵於編輯器內與頁內皆觸發儲存）。預設清單 SHALL 為列右側的下拉（與產出流程頁 locale 同一共用元件），選項為「沿用並追加」與「整份換掉」，列說明 SHALL 顯示所選模式的一句說明。專案規則 SHALL 以 CRITICAL、WARNING、SUGGESTION 三個固定節各一張小卡呈現（嚴重度籤＋即時「N 條」計數），小卡內為與產出規則頁同一個程式碼編輯器元件（純文字模式、行號、自動換行、底部狀態列）；SHALL NOT 提供自由新增節的輸入。一行即一條規則，行序 SHALL 即寫入檔案的條目順序；儲存時 SHALL 逐行去除頭尾空白、空行 SHALL NOT 寫入。選擇「整份換掉」且三節皆空時，頁內 SHALL 顯示提示「Standards 軸只依 repo 文件審查」，該提示 SHALL NOT 阻擋儲存。

寫入：儲存 SHALL 僅寫入 review 鍵——context、rules 與政策欄位逐字元不變；移除語意與 CLI 的 workflow-config review 寫入一致：「沿用並追加」不寫 baseline 鍵、某節清空移除該清單鍵、baseline 缺席且三節皆空時移除整個 review 鍵。寫入 SHALL 經與產出規則頁相同的寫入路徑與雙重解析驗證，以 YAML 保留字元開頭的條目自動加引號、讀回逐字元一致。取消 SHALL 還原唯讀呈現且 SHALL NOT 寫入檔案。config.yaml 解析失敗時本頁 SHALL 依分類的解析失敗呈現停用編輯（見需求「設定頁圖形化讀寫兩層設定」）。遠端專案時本頁內容來自 server 的 policy 文件，儲存 SHALL 帶 expected revision，revision 衝突與 reader 唯讀的處理 SHALL 與同需求所述其他三頁一致。

#### Scenario: 唯讀呈現

- **WHEN** config.yaml 的 review 為 critical 兩條、warning 一條、suggestion 一條且未寫 baseline，使用者開啟專案設定的審查規則頁
- **THEN** 預設清單卡顯示「沿用並追加」；專案規則卡依 CRITICAL（2 條）、WARNING（1 條）、SUGGESTION（1 條）三節列出規則與各自的效果說明；頁標題下等寬註記 `speclink/config.yaml · review`

#### Scenario: 展開 12 條名稱

- **WHEN** 使用者於審查規則頁按「查看 12 條」
- **THEN** 預設清單卡列出 Mysterious Name、Duplicated Code、Feature Envy、Data Clumps、Primitive Obsession、Repeated Switches、Shotgun Surgery、Divergent Change、Speculative Generality、Message Chains、Middle Man、Refused Bequest 十二個名稱；遠端專案時同樣可展開，名稱與本機一致

#### Scenario: 編輯並儲存僅寫 review

- **WHEN** config.yaml 原含 context、rules 與 review，使用者於審查規則頁進入編輯，在 WARNING 小卡新增一行規則後按 ⌘S
- **THEN** 重新讀取 config.yaml 的 review.warning 多出該條，context、rules 與政策欄位與寫入前逐字元一致；本頁回唯讀並顯示新內容

#### Scenario: 整份換掉且無規則時提示但可儲存

- **WHEN** 使用者於編輯態選擇「整份換掉」並清空三個小卡
- **THEN** 頁內顯示「Standards 軸只依 repo 文件審查」的提示；按儲存後 config.yaml 的 review 只含 baseline: replace

#### Scenario: 清空即移除鍵

- **WHEN** 使用者於編輯態清空某節或切回「沿用並追加」後儲存
- **THEN** config.yaml 依移除語意更新，其餘鍵原樣保留

##### Example: 移除語意

| 操作前 review 區塊 | 編輯操作 | 寫入後檔案效果 |
| ------------------ | -------- | -------------- |
| critical 一條、warning 一條 | 清空 WARNING 小卡並儲存 | review 只餘 critical 一條 |
| baseline: replace、critical 一條 | 選「沿用並追加」並儲存 | review 只餘 critical 一條（無 baseline 鍵） |
| suggestion 一條 | 清空 SUGGESTION 小卡並儲存 | review 鍵整個被移除 |

#### Scenario: 固定三節不可自由新增

- **WHEN** 使用者進入審查規則頁的編輯
- **THEN** 編輯器小卡恰為 CRITICAL、WARNING、SUGGESTION 三節各一（各帶「N 條」計數），介面不提供新增節的輸入

#### Scenario: 解析失敗時停用編輯

- **WHEN** config.yaml 的 review 區塊含未知子鍵 critcal，使用者開啟專案設定
- **THEN** 分類欄的審查規則項帶警示點；進入審查規則頁可見解析失敗說明（含 critcal），編輯鈕停用

#### Scenario: 遠端專案儲存帶 revision

- **WHEN** active 專案為遠端 workspace 且使用者具 policy 寫入權，於審查規則頁新增一條 CRITICAL 規則並儲存
- **THEN** 儲存帶 expected revision，成功後 revision 前進、server 端 config 的 review.critical 含該條且其他鍵原樣保留；遇 revision 衝突時輸入原樣保留並浮出逐欄位對照
