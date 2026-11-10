## ADDED Requirements

### Requirement: 設定頁編輯子代理模型

專案設定「品質關卡」底下的「子代理模型」頁 SHALL 讀寫 config.yaml 的 subagent_model 鍵，呈現兩張卡——「各站使用的模型」與「自訂清單」——預設唯讀並有自身的編輯態，一頁進入編輯 SHALL NOT 影響其他專案設定頁的唯讀呈現與可用性；頁標題下 SHALL 以等寬字註記 `speclink/config.yaml · subagent_model`。介面文案 SHALL 隨 UI 介面語言（zh-TW、en）切換，AI 工具的顯示名（Claude Code、Codex、Copilot、pi、Cursor）、工具名稱與模型名稱維持原文。

表格：「各站使用的模型」卡 SHALL 為一張表格，每列一個內建 AI 工具，順序與 AI 工具頁相同（claude、codex、copilot、pi、cursor），工具欄顯示顯示名與等寬的工具名；其餘兩欄固定為「審查」（附註 Standards、Correctness 兩軸共用）與「驗證」。本機專案時，.speclink.yaml 沒勾選的工具 SHALL 照樣列出並在旁加灰色「未啟用」；遠端專案 SHALL NOT 加此標記。pi 列 SHALL 附說明「pi 本身不派子代理；裝了子代理擴充套件，這兩格才會用上」。表格與自訂清單卡 SHALL NOT 列出內建工具以外的工具名稱。

唯讀呈現：有值的格子 SHALL 顯示模型名稱與來源小字——值在該工具的內建清單內為「內建」，否則在該工具的自訂清單內為「自訂清單」，否則為「自行輸入」；沒值的格子 SHALL 顯示「沿用主對話」。內建清單 SHALL 取自引擎提供的清單（只有 claude：sonnet、opus、haiku、fable），desktop SHALL NOT 另存一份。自訂清單卡 SHALL 每個內建工具一列，列出該工具的自訂名稱與「自訂 N 個」計數，沒有名稱時顯示「沒有自訂名稱」；claude 列另顯示四個內建名稱。

編輯：頁標題右側的編輯鈕 SHALL 切換本頁為編輯態並將按鈕列改為取消與儲存（儲存鈕標示 ⌘S／Ctrl+S，該快捷鍵於編輯器內與頁內皆觸發儲存）。每格 SHALL 為一個下拉（與產出流程頁 locale 同一共用元件），選項依序為「沿用主對話」（預設）、「內建」組（只有 claude 有）、「自訂清單」組（取該工具編輯中的自訂清單）、「自行輸入…」；兩組都空時只剩頭尾兩項；同一名稱同時在內建清單與自訂清單時只列在「內建」組。選「自行輸入…」時格子下方 SHALL 出現輸入框，輸入的值 SHALL NOT 自動加進自訂清單。自訂清單卡 SHALL 每個內建工具一個純文字編輯區（與審查規則頁同一個程式碼編輯器元件），一行一個名稱；claude 的四個內建名稱 SHALL 唯讀顯示在旁。從自訂清單刪掉某格已選用的名稱時，該格 SHALL 保留原值並改顯示為「自行輸入」。頁尾 SHALL 說明模型名稱不預先檢查，名稱有誤時品質關卡在派子代理那一步停下並回報工具的錯誤原文。

寫入：儲存 SHALL 僅寫入 subagent_model 鍵——context、rules、review 與政策欄位逐字元不變。模型名稱與清單條目 SHALL 去除頭尾空白；選「沿用主對話」或自行輸入留空的格子 SHALL 從檔案移除，清單的空行 SHALL NOT 寫入、清空的清單移除該清單鍵，整塊沒有內容時移除 subagent_model 鍵（移除語意與 CLI 的 workflow-config subagent-model 一致）。config.yaml 裡內建工具以外的工具名稱的格子與清單 SHALL 原樣保留。儲存內容與現值相同時 SHALL NOT 寫入檔案，頁面直接回到唯讀。寫入 SHALL 經與審查規則頁相同的寫入路徑與雙重解析驗證；寫入失敗時 SHALL 顯示指明檔案與階段的單行錯誤，頁面停在編輯態且輸入保留，檔案維持原內容。取消 SHALL 還原唯讀呈現且 SHALL NOT 寫入檔案。本頁儲存 SHALL NOT 寫入 review 鍵，審查規則頁的儲存也不寫入 subagent_model 鍵。config.yaml 解析失敗時本頁 SHALL 依分類的解析失敗呈現停用編輯（見需求「設定頁圖形化讀寫兩層設定」）。遠端專案時本頁內容來自 server 的 policy 文件，頁標題下的註記顯示 policy revision，儲存 SHALL 帶 expected revision；revision 衝突時的逐欄位對照 SHALL 含 subagent_model 的各格與各清單，衝突與 reader 唯讀的處理 SHALL 與同需求所述其他各頁一致。

#### Scenario: 唯讀呈現

- **WHEN** config.yaml 的 subagent_model 為 review.claude: opus、review.codex: gpt-6.1-sol、verify.claude: sonnet、choices.codex: [gpt-6.1-sol, gpt-6-astra]，.speclink.yaml 只勾 claude 與 codex，使用者開啟專案設定的子代理模型頁
- **THEN** 表格依序列出 Claude Code、Codex、Copilot、pi、Cursor 五列；Claude Code 的審查格為 opus 與「內建」、驗證格為 sonnet 與「內建」；Codex 的審查格為 gpt-6.1-sol 與「自訂清單」、驗證格為「沿用主對話」；Copilot、pi、Cursor 三列旁有「未啟用」且各格為「沿用主對話」；pi 列附子代理擴充套件的說明；自訂清單卡的 Codex 列為 gpt-6.1-sol、gpt-6-astra 與「自訂 2 個」；頁標題下等寬註記 `speclink/config.yaml · subagent_model`

#### Scenario: 下拉選項依工具的清單而定

- **WHEN** 使用者於子代理模型頁進入編輯，打開各工具審查格的下拉
- **THEN** 選項依該工具的內建清單與編輯中的自訂清單組成，依序為「沿用主對話」、內建組、自訂清單組、「自行輸入…」，空的組不出現

##### Example: 各工具的下拉選項

| 工具 | 內建清單 | 自訂清單 | 下拉選項（依序） |
| ---- | -------- | -------- | ---------------- |
| claude | sonnet、opus、haiku、fable | （空） | 沿用主對話；內建：sonnet、opus、haiku、fable；自行輸入… |
| codex | （無） | gpt-6.1-sol、gpt-6-astra | 沿用主對話；自訂清單：gpt-6.1-sol、gpt-6-astra；自行輸入… |
| copilot | （無） | （空） | 沿用主對話；自行輸入… |
| claude | sonnet、opus、haiku、fable | opus、claude-x | 沿用主對話；內建：sonnet、opus、haiku、fable；自訂清單：claude-x；自行輸入… |

#### Scenario: 自行輸入的模型名稱

- **WHEN** 使用者進入編輯，把 copilot 的驗證格選「自行輸入…」，在出現的輸入框輸入「 gpt-5.6 」（前後有空白）後按 ⌘S
- **THEN** config.yaml 的 subagent_model.verify.copilot 為 gpt-5.6，subagent_model.choices 不變；本頁回唯讀，該格顯示 gpt-5.6 與「自行輸入」

#### Scenario: 編輯並儲存僅寫 subagent_model

- **WHEN** config.yaml 原含 context、rules、review 與 subagent_model，使用者於子代理模型頁把 codex 的驗證格選成 gpt-6-astra 後按 ⌘S
- **THEN** 重新讀取 config.yaml 的 subagent_model.verify 多出 codex: gpt-6-astra，context、rules、review 與政策欄位與寫入前逐字元一致；本頁回唯讀並顯示新內容

#### Scenario: 清空即移除鍵

- **WHEN** 使用者於編輯態把格子選回「沿用主對話」或清空某個工具的自訂清單後儲存
- **THEN** config.yaml 依移除語意更新，其餘鍵原樣保留

##### Example: 移除語意

| 操作前 subagent_model 區塊 | 編輯操作 | 寫入後檔案效果 |
| -------------------------- | -------- | -------------- |
| review.claude: opus、verify.claude: sonnet | claude 的驗證格選「沿用主對話」並儲存 | 只餘 review.claude: opus（verify 鍵移除） |
| review.claude: opus、choices.codex 兩個名稱 | 清空 Codex 的自訂清單並儲存 | 只餘 review.claude: opus（choices 鍵移除） |
| 只有 review.claude: opus | 該格選「自行輸入…」並清空輸入框後儲存 | subagent_model 鍵整個被移除 |

#### Scenario: 刪掉已選用的名稱改顯示為自行輸入

- **WHEN** codex 的審查格為 gpt-6.1-sol 且它在 Codex 的自訂清單內，使用者於編輯態從 Codex 的自訂清單刪掉 gpt-6.1-sol 並儲存
- **THEN** 編輯中該格的下拉改為「自行輸入…」、輸入框帶 gpt-6.1-sol；寫入後 subagent_model.review.codex 仍為 gpt-6.1-sol、choices.codex 不再含它；唯讀時該格的來源小字為「自行輸入」

#### Scenario: 內建工具以外的工具名稱原樣保留

- **WHEN** config.yaml 的 subagent_model 含 review.my-agent: model-a 與 choices.my-agent: [model-a]（my-agent 不是內建工具），使用者於子代理模型頁把 claude 的審查格選成 opus 並儲存
- **THEN** 表格與自訂清單卡沒有 my-agent 的列；寫入後檔案的 subagent_model 含 review.claude: opus，且仍含 review.my-agent: model-a 與 choices.my-agent: [model-a]

#### Scenario: 內容未變時不寫入

- **WHEN** config.yaml 的 subagent_model 區塊內有註解，使用者於子代理模型頁進入編輯、未做任何改動就按儲存
- **THEN** config.yaml 逐位元不變（註解仍在），本頁回唯讀

#### Scenario: 取消不寫入

- **WHEN** 使用者於編輯態改了兩格與一份自訂清單後按取消
- **THEN** 本頁回唯讀並顯示原值，config.yaml 逐位元不變

#### Scenario: 解析失敗時停用編輯

- **WHEN** config.yaml 的 subagent_model 區塊含未知子鍵 reveiw，使用者開啟專案設定
- **THEN** 分類欄的審查規則與子代理模型兩個子項帶警示點、上層分類「品質關卡」不帶；進入子代理模型頁可見解析失敗說明（含 reveiw），編輯鈕停用

#### Scenario: 遠端專案儲存帶 revision

- **WHEN** active 專案為遠端 workspace 且使用者具 policy 寫入權，於子代理模型頁把 claude 的審查格選成 opus 並儲存
- **THEN** 表格不顯示「未啟用」標記；儲存帶 expected revision，成功後 revision 前進、server 端 config 的 subagent_model.review.claude 為 opus 且其他鍵原樣保留；遇 revision 衝突時輸入原樣保留，頁頂的逐欄位對照含 subagent_model.review.claude 一列

## MODIFIED Requirements

### Requirement: 設定頁圖形化讀寫兩層設定
<!-- BEFORE: 「工作流程」組為平的四項（專案說明、產出規則、審查規則、產出流程），遠端與 reader 皆為四頁 -->

設定 SHALL 拆分為兩種模式：**專案設定**（跟隨 active 專案，自專案欄底部「專案設定」進入）與**應用程式設定**（與任何專案無關，自圖示列齒輪進入）。兩種模式 SHALL 以左側分類欄取代頁簽：點分類即切換主區為該分類的單一頁，分類欄 SHALL 記住上次所在分類；每頁 SHALL 以共用設定零件排版——24px 頁標題＋灰字說明＋右側頁級動作、14px 小標＋16px 圓角卡、卡內一列一筆細線分隔（左標籤與說明、右控制項）、列右側下拉。

專案設定的分類欄 SHALL 分兩組，組名與分類名一律用概念名、SHALL NOT 用設定檔檔名（與應用程式設定的分類欄相同）：「工作流程」組依序為專案說明、產出規則、品質關卡、產出流程，其中「品質關卡」是上層分類，底下依序有「審查規則」「子代理模型」兩個子項；「專案」組為 AI 工具；預設落在專案說明。上層分類「品質關卡」SHALL 與其他分類項同一外觀（含圖示），點擊 SHALL 開啟審查規則頁，其本身 SHALL NOT 顯示選中底色——選中底色只落在子項上；子項 SHALL 沒有圖示、高 28px，文字左緣與上層分類的文字左緣對齊，選中與 hover 的配色與其他分類項相同；子項 SHALL 永遠展開，SHALL NOT 提供收合。設定檔的路徑只出現在各頁頁標題下的等寬註記。

- **專案說明**頁 SHALL 含專案說明卡、**產出規則**頁 SHALL 含產出規則卡（行為見需求「設定頁編輯專案說明與產出規則」）。
- **審查規則**頁 SHALL 含預設清單卡與專案規則卡（行為見需求「設定頁編輯審查規則」）。
- **子代理模型**頁 SHALL 含「各站使用的模型」卡與「自訂清單」卡（行為見需求「設定頁編輯子代理模型」）。
- **產出流程**頁 SHALL 含「產出政策」卡——locale、spec_locale（列右側下拉）與 tdd、audit、worktree（列右側開關）——與「產出流程」卡（行為見需求「產出流程頁」）。
- **AI 工具**頁（頁標題「AI 工具」，頁標題下等寬註記 `.speclink.yaml · tools`）SHALL 含「AI 工具」卡——內建工具 claude／codex／copilot／pi／cursor 每個一列附開關，自訂工具描述子原樣呈現為不可編輯列；SHALL NOT 含「設定檔」卡或「檔案」列。

應用程式設定的分類欄 SHALL 為「應用程式」組：一般、更新、CLI、伺服器，其後為「關於」組：關於 Speclink（行為見 desktop-app 需求「關於頁」）；預設落在一般；零專案時 SHALL 照常可進入且分類欄照常呈現。

- **一般**頁 SHALL 含「語言與外觀」卡（顯示語言列、外觀列；行為見需求「UI 介面語言支援 zh-TW 與 en」「外觀偏好」）與「鍵盤快捷鍵」卡，並 SHALL 以頁標題下的註記說明其內容僅存於此裝置、不寫入版本庫；SHALL NOT 含軟體更新卡與 CLI 指令卡。
- **更新**頁行為見 desktop-app 需求「桌面自動更新」「更新日誌彈窗」。
- **CLI**頁行為見 desktop-app 需求「安裝 CLI 指令到 PATH」。
- **伺服器**頁行為見 desktop-connections 需求「伺服器管理最小面」。

應用程式設定的同一個設定列 SHALL 只出現在一個分類頁，SHALL NOT 在其他分類頁重複。

專案設定每頁的頁標題下 SHALL 以等寬字註記對應檔案路徑與鍵（如 `speclink/config.yaml · context`）。讀取時未設定的欄位 SHALL 呈現為預設值狀態；寫入時 SHALL 僅代換目標鍵——未觸及的鍵（remote、spec_dir、自訂工具描述子等）SHALL 原樣保留；政策欄位設回預設值時 SHALL 移除該鍵而非寫入明值。tools 寫入成功後 app SHALL 同步技能檔（新選工具生成、取消工具清理殘留）。自訂工具描述子 SHALL 寫入後保留。任一層設定檔解析失敗時，分類欄中掛該檔的分類項（config.yaml 掛專案說明、產出規則、審查規則、子代理模型、產出流程，上層分類「品質關卡」SHALL NOT 掛；.speclink.yaml 掛 AI 工具）SHALL 帶警示點、頁內 SHALL 浮出解析失敗說明且該頁表單 SHALL 停用；應用程式設定 SHALL NOT 受任何專案設定檔解析失敗影響。

遠端 workspace 為 active 專案時，專案設定的分類欄 SHALL 只有「工作流程」組（專案說明、產出規則、品質關卡及其子項審查規則與子代理模型、產出流程；無「專案」組與 AI 工具——tools 屬本機 checkout 概念），五頁內容來自 server 的 policy 文件，頁標題下的註記 SHALL 以等寬字顯示 policy revision；鍵保留語意與本地一致（未觸及鍵原樣保留、設回預設移除鍵）。儲存 SHALL 帶 expected revision；收到 revision 衝突時 SHALL 原樣保留使用者輸入並於頁頂浮出逐欄位對照（server 現值｜我的輸入），僅提供「以 server 版重載」與「檢視後以最新 revision 重新提交」兩出口——SHALL NOT 提供未經對照的強制覆寫。policy 寫入權為假（reader）時五頁 SHALL 唯讀、儲存停用附繁中角色說明。應用程式設定不受 active 專案種類影響。

#### Scenario: 兩種設定模式與預設分類

- **WHEN** 使用者於 local 專案分別點專案欄「專案設定」與圖示列齒輪
- **THEN** 專案設定的分類欄為「工作流程」組（專案說明、產出規則、品質關卡及其子項審查規則與子代理模型、產出流程）與「專案」組（AI 工具），組名與分類名都不含檔名，預設在專案說明頁（頁標題下等寬註記 `speclink/config.yaml · context`）；切到產出流程頁見產出政策卡與產出流程卡、切到 AI 工具頁見頁標題「AI 工具」、等寬註記 `.speclink.yaml · tools` 與 AI 工具卡，沒有檔案列；應用程式設定的分類欄為「應用程式」組（一般、更新、CLI、伺服器）與「關於」組（關於 Speclink），預設在一般頁（含語言與外觀、鍵盤快捷鍵兩卡與「僅存於此裝置」註記，無軟體更新卡與 CLI 指令卡）

#### Scenario: 點上層分類開審查規則頁

- **WHEN** 使用者於 local 專案的專案設定點分類欄「品質關卡」，再點「子代理模型」，接著點「專案說明」
- **THEN** 第一次點擊後主區為審查規則頁，選中底色落在「審查規則」子項、「品質關卡」沒有選中底色；第二次點擊後主區為子代理模型頁、選中底色移到「子代理模型」子項；第三次點擊後兩個子項仍展開在「品質關卡」之下、都沒有圖示

#### Scenario: 遠端專案的專案設定分類

- **WHEN** active 專案為遠端 workspace 且使用者具 policy 寫入權，開啟專案設定修改產出流程頁的產出政策並儲存
- **THEN** 分類欄只有專案說明、產出規則、品質關卡（子項審查規則、子代理模型）、產出流程（頁標題下等寬顯示 policy revision），儲存成功後 revision 前進，server 端 config 反映新政策且未觸及鍵原樣保留

#### Scenario: 遠端儲存遇 revision 衝突

- **WHEN** 兩個 client 同時編輯同一 scope 的 policy，後儲存者收到 revision 衝突
- **THEN** 後儲存者的輸入原樣保留，頁頂浮出 server 現值與我的輸入的逐欄位對照，可選以 server 版重載或以最新 revision 重新提交；無任何未經對照的強制覆寫路徑

#### Scenario: reader 唯讀

- **WHEN** role 為 reader 的使用者於遠端專案開啟專案設定
- **THEN** 五頁呈現現值但唯讀，儲存停用並附角色說明

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

- **WHEN** .speclink.yaml 原 tools 僅 claude，使用者於 AI 工具頁開啟 codex 的開關並儲存
- **THEN** .speclink.yaml 的 tools 記錄 claude 與 codex，且專案根新增 .agents/skills/ 技能檔而無 AGENTS.md

#### Scenario: 加選與取消 copilot 後技能同步

- **WHEN** .speclink.yaml 原 tools 僅 claude，使用者開啟 copilot 並儲存，之後再關閉 copilot 並儲存
- **THEN** 第一次儲存後 tools 記錄 claude 與 copilot、專案根新增 .github/skills/ 技能檔；第二次儲存後 tools 僅含 claude，.github/skills/ 下 speclink- 前綴目錄全數移除，.github/ 下其他檔案位元級不變

#### Scenario: 加選與取消 pi 後技能同步

- **WHEN** .speclink.yaml 原 tools 僅 codex，使用者開啟 pi 並儲存，之後再關閉 pi 並儲存
- **THEN** 第一次儲存後 tools 記錄 codex 與 pi、專案根新增 .pi/skills/ 技能檔；第二次儲存後 tools 僅含 codex，.pi/skills/ 下 speclink- 前綴目錄全數移除，.agents/skills/ 下的 Codex 技能檔仍在

#### Scenario: 自訂工具描述子原樣保留

- **WHEN** .speclink.yaml 的 tools 含一個自訂描述子物件，使用者於 AI 工具頁變更內建工具開關並儲存
- **THEN** 寫入後的 tools 清單仍含該描述子且欄位內容不變，頁內將其呈現為不可編輯列

#### Scenario: 解析失敗分類級警示

- **WHEN** config.yaml 被外部改壞為無法解析，使用者開啟專案設定
- **THEN** 分類欄的專案說明、產出規則、審查規則、子代理模型、產出流程五項帶警示點，上層分類「品質關卡」不帶；進入任一頁可見解析失敗說明，產出政策卡表單與專案說明、產出規則、審查規則、子代理模型四頁的編輯鈕停用；應用程式設定的顯示語言列仍可正常使用

### Requirement: 設定頁編輯審查規則
<!-- BEFORE: 遠端時的 revision 衝突與 reader 唯讀處理寫「與同需求所述其他三頁一致」 -->

專案設定的「審查規則」頁 SHALL 呈現兩張卡——預設清單卡與專案規則卡——預設唯讀並有自身的編輯態，一頁進入編輯 SHALL NOT 影響其他專案設定頁的唯讀呈現與可用性；頁標題下 SHALL 以等寬字註記 `speclink/config.yaml · review`。介面文案 SHALL 隨 UI 介面語言（zh-TW、en）切換，嚴重度籤 CRITICAL、WARNING、SUGGESTION 與 smell 名稱維持英文。

唯讀呈現：預設清單卡 SHALL 顯示目前模式（「沿用並追加」或「整份換掉」）與一句說明，並 SHALL 提供「查看 12 條」展開鈕，展開後列出 12 條 smell 名稱——名稱 SHALL 取自引擎提供的清單，desktop SHALL NOT 另存一份。專案規則卡 SHALL 依 CRITICAL、WARNING、SUGGESTION 三節列出規則，每節帶嚴重度籤、一行效果說明與條數；效果說明 SHALL 為：CRITICAL「擋蓋章；要接受放行須逐條再確認」、WARNING「擋蓋章；偶有正當例外可接受放行」、SUGGESTION「只記在工單，不擋蓋章」。三節皆空時 SHALL 顯示空狀態。

編輯：頁標題右側的編輯鈕 SHALL 切換本頁為編輯態並將按鈕列改為取消與儲存（儲存鈕標示 ⌘S／Ctrl+S，該快捷鍵於編輯器內與頁內皆觸發儲存）。預設清單 SHALL 為列右側的下拉（與產出流程頁 locale 同一共用元件），選項為「沿用並追加」與「整份換掉」，列說明 SHALL 顯示所選模式的一句說明。專案規則 SHALL 以 CRITICAL、WARNING、SUGGESTION 三個固定節各一張小卡呈現（嚴重度籤＋即時「N 條」計數），小卡內為與產出規則頁同一個程式碼編輯器元件（純文字模式、行號、自動換行、底部狀態列）；SHALL NOT 提供自由新增節的輸入。一行即一條規則，行序 SHALL 即寫入檔案的條目順序；儲存時 SHALL 逐行去除頭尾空白、空行 SHALL NOT 寫入。選擇「整份換掉」且三節皆空時，頁內 SHALL 顯示提示「Standards 軸只依 repo 文件審查」，該提示 SHALL NOT 阻擋儲存。

寫入：儲存 SHALL 僅寫入 review 鍵——context、rules 與政策欄位逐字元不變；移除語意與 CLI 的 workflow-config review 寫入一致：「沿用並追加」不寫 baseline 鍵、某節清空移除該清單鍵、baseline 缺席且三節皆空時移除整個 review 鍵。寫入 SHALL 經與產出規則頁相同的寫入路徑與雙重解析驗證，以 YAML 保留字元開頭的條目自動加引號、讀回逐字元一致。取消 SHALL 還原唯讀呈現且 SHALL NOT 寫入檔案。config.yaml 解析失敗時本頁 SHALL 依分類的解析失敗呈現停用編輯（見需求「設定頁圖形化讀寫兩層設定」）。遠端專案時本頁內容來自 server 的 policy 文件，儲存 SHALL 帶 expected revision，revision 衝突與 reader 唯讀的處理 SHALL 與同需求所述其他各頁一致。

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
