## MODIFIED Requirements

### Requirement: 設定頁圖形化讀寫兩層設定
<!-- BEFORE: 正式規格的應用程式分類列一般、更新、CLI、伺服器與關於，尚未列網路與診斷；程式已有安全標準資料頁。 -->

設定 SHALL 拆分為兩種模式：**專案設定**（跟隨 active 專案，自專案欄底部「專案設定」進入）與**應用程式設定**（與任何專案無關，自圖示列齒輪進入）。兩種模式 SHALL 以左側分類欄取代頁簽：點分類即切換主區為該分類的單一頁，分類欄 SHALL 記住上次所在分類；每頁 SHALL 以共用設定零件排版——24px 頁標題＋灰字說明＋右側頁級動作、14px 小標＋16px 圓角卡、卡內一列一筆細線分隔（左標籤與說明、右控制項）、列右側下拉。

專案設定的分類欄 SHALL 分兩組：「openspec/config.yaml」組依序為專案說明、產出規則、審查規則、產出流程；「專案」組為 .speclink.yaml；預設落在專案說明。

- **專案說明**頁 SHALL 含專案說明卡、**產出規則**頁 SHALL 含產出規則卡（行為見需求「設定頁編輯專案說明與產出規則」）。
- **審查規則**頁 SHALL 含預設清單卡與專案規則卡（行為見需求「設定頁編輯審查規則」）。
- **產出流程**頁 SHALL 含「產出政策」卡——locale、spec_locale（列右側下拉）與 tdd、audit、worktree（列右側開關）——與「產出流程」卡（行為見需求「產出流程頁」）。
- **.speclink.yaml** 頁 SHALL 含「檔案」列（等寬路徑，唯讀）與「AI 工具」卡——內建工具 claude／codex／copilot 每個一列附開關，自訂工具描述子原樣呈現為不可編輯列。

應用程式設定的分類欄 SHALL 為「應用程式」組：一般、更新、CLI、伺服器、網路與診斷、安全標準資料，其後為「關於」組：關於 Speclink（行為見 desktop-app 需求「關於頁」）；預設落在一般；零專案時 SHALL 照常可進入且分類欄照常呈現。

- **一般**頁 SHALL 含「語言與外觀」卡（顯示語言列、外觀列；行為見需求「UI 介面語言支援 zh-TW 與 en」「外觀偏好」）與「鍵盤快捷鍵」卡，並 SHALL 以頁標題下的註記說明其內容僅存於此裝置、不寫入版本庫；SHALL NOT 含軟體更新卡與 CLI 指令卡。
- **更新**頁行為見 desktop-app 需求「桌面自動更新」「更新日誌彈窗」。
- **CLI**頁行為見 desktop-app 需求「安裝 CLI 指令到 PATH」。
- **伺服器**頁行為見 desktop-connections 需求「伺服器管理最小面」。
- **網路與診斷**頁 SHALL 提供 desktop-network-diagnostics 規定的裝置網路偏好、Debug mode與診斷匯出，分類搜尋 SHALL 同時比對Proxy、Debug mode、匯出診斷與測試更新連線等列標籤。
- **安全標準資料**頁 SHALL 沿用既有資料管理，並提供 security-standard-data 規定的匯入及匯出操作。

應用程式設定的同一個設定列 SHALL 只出現在一個分類頁，SHALL NOT 在其他分類頁重複。

專案設定每頁的頁標題下 SHALL 以等寬字註記對應檔案路徑與鍵（如 `openspec/config.yaml · context`）。讀取時未設定的欄位 SHALL 呈現為預設值狀態；寫入時 SHALL 僅代換目標鍵——未觸及的鍵（remote、spec_dir、自訂工具描述子等）SHALL 原樣保留；政策欄位設回預設值時 SHALL 移除該鍵而非寫入明值。tools 寫入成功後 app SHALL 同步技能檔（新選工具生成、取消工具清理殘留）。自訂工具描述子 SHALL 寫入後保留。任一層設定檔解析失敗時，分類欄中掛該檔的分類項（config.yaml 掛專案說明、產出規則、審查規則、產出流程；.speclink.yaml 掛 .speclink.yaml）SHALL 帶警示點、頁內 SHALL 浮出解析失敗說明且該頁表單 SHALL 停用；應用程式設定 SHALL NOT 受任何專案設定檔解析失敗影響。

遠端 workspace 為 active 專案時，專案設定的分類欄 SHALL 只有「openspec/config.yaml」組的四項（無 .speclink.yaml——tools 屬本機 checkout 概念），四頁內容來自 server 的 policy 文件，頁標題下的註記 SHALL 以等寬字顯示 policy revision；鍵保留語意與本地一致（未觸及鍵原樣保留、設回預設移除鍵）。儲存 SHALL 帶 expected revision；收到 revision 衝突時 SHALL 原樣保留使用者輸入並於頁頂浮出逐欄位對照（server 現值｜我的輸入），僅提供「以 server 版重載」與「檢視後以最新 revision 重新提交」兩出口——SHALL NOT 提供未經對照的強制覆寫。policy 寫入權為假（reader）時四頁 SHALL 唯讀、儲存停用附繁中角色說明。應用程式設定不受 active 專案種類影響。

#### Scenario: 兩種設定模式與預設分類

- **WHEN** 使用者於 local 專案分別點專案欄「專案設定」與圖示列齒輪
- **THEN** 專案設定的分類欄為「openspec/config.yaml」組（專案說明、產出規則、審查規則、產出流程）與「專案」組（.speclink.yaml），預設在專案說明頁（頁標題下等寬註記 `openspec/config.yaml · context`）；切到產出流程頁見產出政策卡與產出流程卡、切到 .speclink.yaml 頁見檔案列與 AI 工具卡；應用程式設定的分類欄為「應用程式」組（一般、更新、CLI、伺服器、網路與診斷、安全標準資料）與「關於」組（關於 Speclink），預設在一般頁（含語言與外觀、鍵盤快捷鍵兩卡與「僅存於此裝置」註記，無軟體更新卡與 CLI 指令卡）

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

- **WHEN** config.yaml 原含 locale: tw，使用者於產出流程頁將 locale 改回「未設定（系統語言）」並儲存
- **THEN** 重新讀取 config.yaml 已無 locale 鍵，且引擎解析該檔的有效 locale 為執行主機的系統語言

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

#### Scenario: 網路與診斷可搜尋且零專案可進入
- **WHEN** 使用者沒有開啟專案，在應用程式設定搜尋Proxy或匯出診斷，再進入命中分類
- **THEN** 顯示網路與診斷頁的對應設定，安全標準資料分類仍存在；切頁重回時分類記憶沿既有規則


### Requirement: 專案分頁列存於 app 本機
<!-- BEFORE: 預設作用中方塊的範例為teal底白字。 -->

app SHALL 以圖示列的專案方塊呈現目前開著的專案(每個專案一個 32px 方塊,內容為顯示名首字母;上限 10 個):開著的專案 SHALL 跨啟動持久化於 app 本機狀態(含順序與最後作用中專案),SHALL NOT 寫入任何專案目錄。點擊方塊 SHALL 以該 locator 執行與「開啟專案」相同的切換語意;同一專案再次開啟 SHALL 去重並移至既有方塊;關閉專案(標題列「⋯」或方塊右鍵選單的「關閉專案」)SHALL 將其自持久化清單移除,但 SHALL NOT 改變最近開啟記錄(曾開啟過的專案由 workspace-chooser 的「最近開啟清單」需求另行記憶,專案方塊 SHALL NOT 作為唯一的最近開啟記憶)。方塊 SHALL NOT 顯示待收尾數或其他計數徽章——待收尾狀態由看板欄位計數與系統匣面板分區計數承載。方塊 hover SHALL 以提示顯示完整顯示名與路徑(remote 專案顯示 checkout 路徑,無 checkout 時顯示連線與 Project/Repo)。方塊指向已不存在的路徑時 SHALL 以錯誤態呈現(方塊角落紅點、提示附錯誤原因),點擊 SHALL 開啟含錯誤訊息與「自專案列移除」的選單,SHALL NOT 切換專案;背景 local 專案 SHALL 於 app 啟動時各探測一次路徑有效性,失效即轉錯誤態。remote 專案離線或需重新登入時方塊角落 SHALL 為琥珀點。無任何專案時 app SHALL 不渲染專案欄,主區 SHALL 顯示「開啟一個專案開始」空狀態(字標、標題、說明、「新增專案」與「連線 Server」兩鈕、最近開啟卡)而非空看板。app SHALL 支援 Ctrl+Tab 循環切換與 Ctrl+1..9 直達第 N 個專案。

#### Scenario: 成功開啟後記入分頁並去重上移

- **WHEN** 使用者依序開啟專案 A、B,再次開啟專案 A
- **THEN** 圖示列僅含 A、B 各一個方塊且 A 為作用中(預設深灰#374151底白字),A 與 B 的專案目錄內均無因此新增的檔案;重啟 app 後圖示列還原為相同內容與順序

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
- **THEN** 左側只有圖示列(品牌標記、「＋」、設定),無專案欄;主區置中顯示字標、「開啟一個專案開始」標題、說明(含「資料夾還沒有 openspec/ 時會先問你要不要初始化」)、「新增專案」與「連線 Server」兩鈕,而非空白看板
