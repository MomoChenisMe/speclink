## MODIFIED Requirements

### Requirement: 未初始化目錄經確認後自動初始化

所選目錄向上探索未命中任何 speclink 專案時，app SHALL NOT 逕行寫入，而 SHALL 顯示初始化確認對話框（含 AI 工具多選 claude／codex／copilot／pi／cursor，預設只勾選 claude；說明文字寫明會建立 `speclink/`）。使用者確認後 app SHALL 執行與 speclink init 等效的初始化（speclink/ 骨架含 specs/、changes/archive/ 與 config.yaml、專案根的 .speclink.yaml 記錄所選 tools、為每個所選工具生成 skills 檔），隨即切換至該專案；使用者取消時 app SHALL 維持原專案，且目標目錄 SHALL NOT 產生任何寫入。初始化失敗時 app SHALL 顯示單行錯誤訊息且 SHALL NOT 切換 root。

#### Scenario: 確認後初始化並切入新專案

- **WHEN** 使用者選定不含任何 speclink 標記的空目錄，於確認對話框保持預設（claude）並確認
- **THEN** 該目錄產生 speclink/（含 specs/、changes/archive/、config.yaml）、.speclink.yaml（tools 含 claude）與 .claude/skills/ 技能檔，不產生 openspec/ 與 CLAUDE.md，且 app 切換至該專案並於看板顯示空清單

#### Scenario: 勾選 codex 時生成對應工具檔

- **WHEN** 使用者於確認對話框加勾 codex 後確認
- **THEN** 目標目錄除 claude 對應檔案外，另產生 .agents/skills/ 技能檔而無 AGENTS.md，.speclink.yaml 的 tools 同時記錄 claude 與 codex

#### Scenario: 取消初始化則零寫入

- **WHEN** 使用者於確認對話框取消
- **THEN** app 維持原專案，所選目錄內容與選擇前完全相同（無任何新檔案或目錄）

#### Scenario: 勾選 copilot 時生成對應工具檔

- **WHEN** 使用者於確認對話框加勾 copilot 後確認
- **THEN** 目標目錄除 claude 對應檔案外，另產生 .github/skills/ 技能檔，不產生 .github/copilot-instructions.md、AGENTS.md，.speclink.yaml 的 tools 同時記錄 claude 與 copilot

#### Scenario: 勾選 pi 與 cursor 時生成對應工具檔

- **WHEN** 使用者於確認對話框加勾 pi 與 cursor 後確認
- **THEN** 目標目錄除 claude 對應檔案外，另產生 .pi/skills/ 與 .cursor/skills/ 技能檔，不產生 .cursor/agents/、.cursor/rules/、AGENTS.md，.speclink.yaml 的 tools 依 claude、pi、cursor 記錄

### Requirement: 未啟用資料夾經確認後補齊啟用

所選目錄向上探索命中 workspace、store mode 為本地檔案、且該 workspace root 不存在 `.speclink.yaml` 時，app SHALL 判定為未啟用 speclink，SHALL NOT 逕行寫入亦 SHALL NOT 直接以既有專案開啟，而 SHALL 顯示啟用確認對話框（含 AI 工具多選 claude／codex／copilot／pi／cursor，預設只勾選 claude；文案為啟用語意，遵循 LANGUAGE.md、不出現工程詞）。規格資料夾為舊佈局的 `openspec/` 時，對話框 SHALL 另寫明確認後資料夾會改名為 `speclink/`、之後 OpenSpec CLI 讀不到這個專案。判定與寫入 SHALL 錨定向上命中的 workspace root，而非使用者所選的子目錄。

使用者確認後 app SHALL 經引擎的工作區補齊入口執行啟用（必要時先把 `openspec/` 匯入為 `speclink/`、補骨架缺件、專案根 `.speclink.yaml` 記錄所選 tools、為每個所選工具生成 skills 檔，不生成指令檔），既有規格文件的內容 SHALL 不被改寫，隨即切換至該專案；使用者取消時 app SHALL 維持原專案，目標目錄 SHALL NOT 產生任何寫入。啟用失敗（包含因有其他 worktree 或未提交修改而拒絕匯入）時 app SHALL 顯示單行錯誤訊息說明原因且 SHALL NOT 切換 root。`.speclink.yaml` 存在的專案 SHALL 照舊直接開啟，SHALL NOT 出現啟用對話框；向上探索完全未命中的目錄 SHALL 照舊走初始化確認流程。

#### Scenario: OpenSpec 專案確認啟用後匯入並切入

- **WHEN** 使用者選定含 openspec/（內有既有規格文件）但無 .speclink.yaml、已 commit 的資料夾，啟用確認對話框寫明資料夾會改名為 speclink/，使用者保持預設（claude）並確認
- **THEN** openspec/ 改名為 speclink/、內容位元級不變，專案根產生 .speclink.yaml（tools 含 claude）與 .claude/skills/ 技能檔、不產生 CLAUDE.md，app 切換至該專案並於看板呈現既有內容

#### Scenario: 取消啟用則零寫入

- **WHEN** 使用者於啟用確認對話框取消
- **THEN** app 維持原專案，所選資料夾內容與選擇前完全相同

#### Scenario: 已啟用專案不出現啟用對話框

- **WHEN** 使用者選定專案根含 .speclink.yaml 的資料夾開啟
- **THEN** app 直接開啟該專案進看板，無啟用對話框

#### Scenario: 子目錄開啟錨定專案根

- **WHEN** 使用者選定未啟用專案的子目錄開啟並確認啟用
- **THEN** .speclink.yaml 與工具檔產生於向上命中的專案根，app 切入該根

#### Scenario: 既有工作流設定不被覆蓋

- **WHEN** 未啟用資料夾的 openspec/config.yaml 已存在且含使用者自訂政策，使用者確認啟用
- **THEN** 匯入後 speclink/config.yaml 與原檔位元級相同，僅補齊其餘缺件

#### Scenario: 啟用時勾選 copilot

- **WHEN** 使用者選定含 openspec/ 但無 .speclink.yaml、已 commit 的資料夾，於啟用確認對話框改為只勾 copilot 並確認
- **THEN** 專案根產生 .speclink.yaml（tools 僅含 copilot）與 .github/skills/ 技能檔，不產生 .claude/、.agents/、CLAUDE.md、AGENTS.md，speclink/ 內既有文件與原 openspec/ 位元級相同

#### Scenario: 啟用時勾選 cursor

- **WHEN** 使用者選定含 openspec/ 與 .cursor/rules/、但無 .speclink.yaml、已 commit 的資料夾，於啟用確認對話框改為只勾 cursor 並確認
- **THEN** 專案根產生 .speclink.yaml（tools 僅含 cursor）與 .cursor/skills/ 技能檔，.cursor/rules/ 下既有檔案位元級不變，不產生 .claude/、.agents/、CLAUDE.md、AGENTS.md

#### Scenario: 有未提交修改時啟用失敗

- **WHEN** 使用者選定的 OpenSpec 資料夾中 openspec/ 有未提交修改，於啟用確認對話框確認
- **THEN** app 顯示單行錯誤說明 openspec/ 有未提交修改，維持原專案，所選資料夾內容不變

### Requirement: 設定頁圖形化讀寫兩層設定

設定 SHALL 拆分為兩種模式：**專案設定**（跟隨 active 專案，自專案欄底部「專案設定」進入）與**應用程式設定**（與任何專案無關，自圖示列齒輪進入）。兩種模式 SHALL 以左側分類欄取代頁簽：點分類即切換主區為該分類的單一頁，分類欄 SHALL 記住上次所在分類；每頁 SHALL 以共用設定零件排版——24px 頁標題＋灰字說明＋右側頁級動作、14px 小標＋16px 圓角卡、卡內一列一筆細線分隔（左標籤與說明、右控制項）、列右側下拉。

專案設定的分類欄 SHALL 分兩組，組名與分類名一律用概念名、SHALL NOT 用設定檔檔名（與應用程式設定的分類欄相同）：「工作流程」組依序為專案說明、產出規則、審查規則、產出流程；「專案」組為 AI 工具；預設落在專案說明。設定檔的路徑只出現在各頁頁標題下的等寬註記。

- **專案說明**頁 SHALL 含專案說明卡、**產出規則**頁 SHALL 含產出規則卡（行為見需求「設定頁編輯專案說明與產出規則」）。
- **審查規則**頁 SHALL 含預設清單卡與專案規則卡（行為見需求「設定頁編輯審查規則」）。
- **產出流程**頁 SHALL 含「產出政策」卡——locale、spec_locale（列右側下拉）與 tdd、audit、worktree（列右側開關）——與「產出流程」卡（行為見需求「產出流程頁」）。
- **AI 工具**頁（頁標題「AI 工具」，頁標題下等寬註記 `.speclink.yaml · tools`）SHALL 含「AI 工具」卡——內建工具 claude／codex／copilot／pi／cursor 每個一列附開關，自訂工具描述子原樣呈現為不可編輯列；SHALL NOT 含「設定檔」卡或「檔案」列。

應用程式設定的分類欄 SHALL 為「應用程式」組：一般、更新、CLI、伺服器，其後為「關於」組：關於 Speclink（行為見 desktop-app 需求「關於頁」）；預設落在一般；零專案時 SHALL 照常可進入且分類欄照常呈現。

- **一般**頁 SHALL 含「語言與外觀」卡（顯示語言列、外觀列；行為見需求「UI 介面語言支援 zh-TW 與 en」「外觀偏好」）與「鍵盤快捷鍵」卡，並 SHALL 以頁標題下的註記說明其內容僅存於此裝置、不寫入版本庫；SHALL NOT 含軟體更新卡與 CLI 指令卡。
- **更新**頁行為見 desktop-app 需求「桌面自動更新」「更新日誌彈窗」。
- **CLI**頁行為見 desktop-app 需求「安裝 CLI 指令到 PATH」。
- **伺服器**頁行為見 desktop-connections 需求「伺服器管理最小面」。

應用程式設定的同一個設定列 SHALL 只出現在一個分類頁，SHALL NOT 在其他分類頁重複。

專案設定每頁的頁標題下 SHALL 以等寬字註記對應檔案路徑與鍵（如 `speclink/config.yaml · context`）。讀取時未設定的欄位 SHALL 呈現為預設值狀態；寫入時 SHALL 僅代換目標鍵——未觸及的鍵（remote、spec_dir、自訂工具描述子等）SHALL 原樣保留；政策欄位設回預設值時 SHALL 移除該鍵而非寫入明值。tools 寫入成功後 app SHALL 同步技能檔（新選工具生成、取消工具清理殘留）。自訂工具描述子 SHALL 寫入後保留。任一層設定檔解析失敗時，分類欄中掛該檔的分類項（config.yaml 掛專案說明、產出規則、審查規則、產出流程；.speclink.yaml 掛 AI 工具）SHALL 帶警示點、頁內 SHALL 浮出解析失敗說明且該頁表單 SHALL 停用；應用程式設定 SHALL NOT 受任何專案設定檔解析失敗影響。

遠端 workspace 為 active 專案時，專案設定的分類欄 SHALL 只有「工作流程」組的四項（無「專案」組與 AI 工具——tools 屬本機 checkout 概念），四頁內容來自 server 的 policy 文件，頁標題下的註記 SHALL 以等寬字顯示 policy revision；鍵保留語意與本地一致（未觸及鍵原樣保留、設回預設移除鍵）。儲存 SHALL 帶 expected revision；收到 revision 衝突時 SHALL 原樣保留使用者輸入並於頁頂浮出逐欄位對照（server 現值｜我的輸入），僅提供「以 server 版重載」與「檢視後以最新 revision 重新提交」兩出口——SHALL NOT 提供未經對照的強制覆寫。policy 寫入權為假（reader）時四頁 SHALL 唯讀、儲存停用附繁中角色說明。應用程式設定不受 active 專案種類影響。

#### Scenario: 兩種設定模式與預設分類

- **WHEN** 使用者於 local 專案分別點專案欄「專案設定」與圖示列齒輪
- **THEN** 專案設定的分類欄為「工作流程」組（專案說明、產出規則、審查規則、產出流程）與「專案」組（AI 工具），組名與分類名都不含檔名，預設在專案說明頁（頁標題下等寬註記 `speclink/config.yaml · context`）；切到產出流程頁見產出政策卡與產出流程卡、切到 AI 工具頁見頁標題「AI 工具」、等寬註記 `.speclink.yaml · tools` 與 AI 工具卡，沒有檔案列；應用程式設定的分類欄為「應用程式」組（一般、更新、CLI、伺服器）與「關於」組（關於 Speclink），預設在一般頁（含語言與外觀、鍵盤快捷鍵兩卡與「僅存於此裝置」註記，無軟體更新卡與 CLI 指令卡）

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
- **THEN** 分類欄的專案說明、產出規則、審查規則、產出流程四項帶警示點；進入任一頁可見解析失敗說明，產出政策卡表單與專案說明、產出規則、審查規則三頁的編輯鈕停用；應用程式設定的顯示語言列仍可正常使用

### Requirement: 畫面依實際規格資料夾與資料檔名顯示

桌面 app 畫面上提到規格資料夾或變更資料檔的地方 SHALL 依專案實際使用的資料夾與檔名顯示，SHALL NOT 寫死 `openspec/` 或 `.openspec.yaml`：專案設定各頁標題下的檔案註記（例如 `speclink/config.yaml · context`；舊佈局專案為 `openspec/config.yaml · context`；明寫 `spec_dir` 時為該路徑）、討論詳情顯示的記錄路徑。專案設定分類欄的組名是概念名（「工作流程」），不帶規格資料夾路徑。遷移到遠端與來源選擇對話框 SHALL 以「本機規格資料夾」稱呼本機的規格資料夾。變更資料檔解析失敗的卡片提示 SHALL 以「變更資料檔」稱呼，不帶寫死的檔名。新專案的說明文字（初始化對話框、空狀態與來源選擇）SHALL 寫 `speclink/`。文案 SHALL 跟隨 UI 語言。

#### Scenario: 新佈局專案的檔案註記

- **WHEN** 使用者在規格資料夾為 speclink/ 的專案打開專案設定的專案說明頁
- **THEN** 頁標題下的註記為 `speclink/config.yaml · context`，分類欄組名為「工作流程」

#### Scenario: 暫緩遷移的舊佈局專案

- **WHEN** 使用者在仍是 openspec/ 的專案打開專案設定的產出規則頁
- **THEN** 頁標題下的註記為 `openspec/config.yaml · rules`

#### Scenario: 討論詳情的記錄路徑

- **WHEN** 使用者在 speclink/ 專案打開討論 board-search-bar 的詳情
- **THEN** 顯示的記錄路徑為 `speclink/discussions/board-search-bar.md`

#### Scenario: 資料檔解析失敗的提示

- **WHEN** 某變更的資料檔無法解析，使用者在看板看到該卡片
- **THEN** 卡片提示以「變更資料檔」稱呼並附解析原因，不出現 `.openspec.yaml` 字樣
