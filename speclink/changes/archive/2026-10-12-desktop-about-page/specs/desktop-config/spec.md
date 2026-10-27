## MODIFIED Requirements

### Requirement: 設定頁圖形化讀寫兩層設定

設定 SHALL 拆分為兩種模式：**專案設定**（跟隨 active 專案，自專案欄底部「專案設定」進入）與**應用程式設定**（與任何專案無關，自圖示列齒輪進入）。兩種模式 SHALL 以左側分類欄取代頁簽：點分類即切換主區為該分類的單一頁，分類欄 SHALL 記住上次所在分類；每頁 SHALL 以共用設定零件排版——24px 頁標題＋灰字說明＋右側頁級動作、14px 小標＋16px 圓角卡、卡內一列一筆細線分隔（左標籤與說明、右控制項）、列右側下拉。

專案設定的分類欄 SHALL 分兩組：「openspec/config.yaml」組依序為專案說明、產出規則、產出流程；「專案」組為 .speclink.yaml；預設落在專案說明。

- **專案說明**頁 SHALL 含專案說明卡、**產出規則**頁 SHALL 含產出規則卡（行為見需求「設定頁編輯專案說明與產出規則」）。
- **產出流程**頁 SHALL 含「產出政策」卡——locale、spec_locale（列右側下拉）與 tdd、audit、worktree（列右側開關）——與「產出流程」卡（行為見需求「產出流程頁」）。
- **.speclink.yaml** 頁 SHALL 含「檔案」列（等寬路徑，唯讀）與「AI 工具」卡——內建工具 claude／codex／copilot 每個一列附開關，自訂工具描述子原樣呈現為不可編輯列。

應用程式設定的分類欄 SHALL 為「應用程式」組：一般、更新、CLI、伺服器，其後為「關於」組：關於 Speclink（行為見 desktop-app 需求「關於頁」）；預設落在一般；零專案時 SHALL 照常可進入且分類欄照常呈現。

- **一般**頁 SHALL 含「語言與外觀」卡（顯示語言列、外觀列；行為見需求「UI 介面語言支援 zh-TW 與 en」「外觀偏好」）與「鍵盤快捷鍵」卡，並 SHALL 以頁標題下的註記說明其內容僅存於此裝置、不寫入版本庫；SHALL NOT 含軟體更新卡與 CLI 指令卡。
- **更新**頁行為見 desktop-app 需求「桌面自動更新」「更新日誌彈窗」。
- **CLI**頁行為見 desktop-app 需求「安裝 CLI 指令到 PATH」。
- **伺服器**頁行為見 desktop-connections 需求「伺服器管理最小面」。

應用程式設定的同一個設定列 SHALL 只出現在一個分類頁，SHALL NOT 在其他分類頁重複。

專案設定每頁的頁標題下 SHALL 以等寬字註記對應檔案路徑與鍵（如 `openspec/config.yaml · context`）。讀取時未設定的欄位 SHALL 呈現為預設值狀態；寫入時 SHALL 僅代換目標鍵——未觸及的鍵（remote、spec_dir、自訂工具描述子等）SHALL 原樣保留；政策欄位設回預設值時 SHALL 移除該鍵而非寫入明值。tools 寫入成功後 app SHALL 同步技能檔（新選工具生成、取消工具清理殘留）。自訂工具描述子 SHALL 寫入後保留。任一層設定檔解析失敗時，分類欄中掛該檔的分類項（config.yaml 掛專案說明、產出規則、產出流程；.speclink.yaml 掛 .speclink.yaml）SHALL 帶警示點、頁內 SHALL 浮出解析失敗說明且該頁表單 SHALL 停用；應用程式設定 SHALL NOT 受任何專案設定檔解析失敗影響。

遠端 workspace 為 active 專案時，專案設定的分類欄 SHALL 只有「openspec/config.yaml」組的三項（無 .speclink.yaml——tools 屬本機 checkout 概念），三頁內容來自 server 的 policy 文件，頁標題下的註記 SHALL 以等寬字顯示 policy revision；鍵保留語意與本地一致（未觸及鍵原樣保留、設回預設移除鍵）。儲存 SHALL 帶 expected revision；收到 revision 衝突時 SHALL 原樣保留使用者輸入並於頁頂浮出逐欄位對照（server 現值｜我的輸入），僅提供「以 server 版重載」與「檢視後以最新 revision 重新提交」兩出口——SHALL NOT 提供未經對照的強制覆寫。policy 寫入權為假（reader）時三頁 SHALL 唯讀、儲存停用附繁中角色說明。應用程式設定不受 active 專案種類影響。

#### Scenario: 兩種設定模式與預設分類

- **WHEN** 使用者於 local 專案分別點專案欄「專案設定」與圖示列齒輪
- **THEN** 專案設定的分類欄為「openspec/config.yaml」組（專案說明、產出規則、產出流程）與「專案」組（.speclink.yaml），預設在專案說明頁（頁標題下等寬註記 `openspec/config.yaml · context`）；切到產出流程頁見產出政策卡與產出流程卡、切到 .speclink.yaml 頁見檔案列與 AI 工具卡；應用程式設定的分類欄為「應用程式」組（一般、更新、CLI、伺服器）與「關於」組（關於 Speclink），預設在一般頁（含語言與外觀、鍵盤快捷鍵兩卡與「僅存於此裝置」註記，無軟體更新卡與 CLI 指令卡）

#### Scenario: 遠端專案的專案設定分類

- **WHEN** active 專案為遠端 workspace 且使用者具 policy 寫入權，開啟專案設定修改產出流程頁的產出政策並儲存
- **THEN** 分類欄只有專案說明、產出規則、產出流程三項（頁標題下等寬顯示 policy revision），儲存成功後 revision 前進，server 端 config 反映新政策且未觸及鍵原樣保留

#### Scenario: 遠端儲存遇 revision 衝突

- **WHEN** 兩個 client 同時編輯同一 scope 的 policy，後儲存者收到 revision 衝突
- **THEN** 後儲存者的輸入原樣保留，頁頂浮出 server 現值與我的輸入的逐欄位對照，可選以 server 版重載或以最新 revision 重新提交；無任何未經對照的強制覆寫路徑

#### Scenario: reader 唯讀

- **WHEN** role 為 reader 的使用者於遠端專案開啟專案設定
- **THEN** 三頁呈現現值但唯讀，儲存停用並附角色說明

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
- **THEN** 分類欄的專案說明、產出規則、產出流程三項帶警示點；進入任一頁可見解析失敗說明，產出政策卡表單與專案說明、產出規則兩頁的編輯鈕停用；應用程式設定的顯示語言列仍可正常使用

### Requirement: UI 介面語言支援 zh-TW 與 en

app 的 UI 介面語言 SHALL 支援 zh-TW 與 en:未設定偏好時 SHALL 跟隨系統語言(系統語言以 zh 開頭判為 zh-TW,其餘判為 en);應用程式設定「一般」頁的「顯示語言」列 SHALL 提供「跟隨系統／zh-TW／en」三選下拉,切換 SHALL 即時對全介面生效並持久化於 app 本機;macOS 的原生選單列 SHALL 同樣跟隨 UI 語言,切換時 SHALL 立即重建。UI 語言偏好與 config.yaml 的 locale(AI artifacts 產出語言)SHALL 互不影響。兩語言字典的 key 集合 SHALL 相等;查無 key 時 SHALL 顯示 key 本身而非另一語言的字串。

#### Scenario: 未設定偏好時跟隨系統語言

- **WHEN** app 於 UI 語言偏好未設定的狀態下啟動
- **THEN** 系統語言以 zh 開頭時全介面(含 macOS 選單列)呈現 zh-TW,否則呈現 en

##### Example: 系統語言判定

| 系統語言 | UI 語言 |
| -------- | ------- |
| zh-TW | zh-TW |
| zh-CN | zh-TW |
| en-US | en |
| ja-JP | en |

#### Scenario: 手動切換即時生效並持久化

- **WHEN** 使用者於一般頁的顯示語言列將 UI 語言由 zh-TW 切為 en
- **THEN** 全介面(標題列、圖示列、專案欄、看板、對話框、macOS 選單列)即時改為英文,重啟 app 後仍為英文,且 config.yaml 內容未被此操作改動

#### Scenario: UI 語言與 artifacts 產出語言互不影響

- **WHEN** config.yaml 設定 locale: tw,使用者將 UI 語言切為 en
- **THEN** UI 呈現英文,而 config.yaml 的 locale 仍為 tw(引擎產出 artifacts 的語言政策不受 UI 語言影響)

### Requirement: 外觀偏好

應用程式設定的「一般」頁 SHALL 提供「外觀」列三選：跟隨系統、淺色、深色。偏好 SHALL 存於 app 本機（與 UI 語言偏好同層），SHALL NOT 寫入任何專案檔案；切換 SHALL 即時生效於主視窗與系統匣面板，重啟後維持。跟隨系統時 SHALL 依系統的淺色／深色偏好；明示淺色時即使系統為深色 SHALL 呈現淺色，明示深色反之。淺色與深色的全部色值 SHALL 取自共用 theme 的同一組 token（深色 token 對系統偏好與明示深色兩種觸發 SHALL 相同）。server-web SHALL NOT 受此偏好影響。

#### Scenario: 明示深色即時生效並持久化

- **WHEN** 系統為淺色，使用者於一般頁把外觀改為「深色」
- **THEN** 主視窗與系統匣面板立即改為深色 token；重啟 app 後仍為深色；config.yaml 與 .speclink.yaml 未被改動

#### Scenario: 跟隨系統

- **WHEN** 外觀為「跟隨系統」且使用者把系統外觀自淺色切為深色
- **THEN** 主視窗與系統匣面板隨之改為深色，無需重啟

#### Scenario: 明示淺色壓過系統深色

- **WHEN** 系統為深色，使用者把外觀改為「淺色」
- **THEN** 主視窗與系統匣面板呈現淺色 token

##### Example: 有效主題判定

| 偏好 | 系統 | 有效主題 |
| ---- | ---- | -------- |
| 跟隨系統 | 淺色 | 淺色 |
| 跟隨系統 | 深色 | 深色 |
| 淺色 | 深色 | 淺色 |
| 深色 | 淺色 | 深色 |
