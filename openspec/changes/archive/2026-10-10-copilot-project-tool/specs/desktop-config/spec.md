## MODIFIED Requirements

### Requirement: 未初始化目錄經確認後自動初始化
<!-- BEFORE: 初始化對話框僅提供 claude／codex，預設 claude。 -->

所選目錄向上探索未命中任何 speclink 專案時，app SHALL NOT 逕行寫入，而 SHALL 顯示初始化確認對話框（含 AI 工具多選 claude／codex／copilot，預設勾選 claude）。使用者確認後 app SHALL 執行與 speclink init 等效的初始化（openspec/ 骨架含 specs/、changes/archive/ 與 config.yaml、專案根的 .speclink.yaml 記錄所選 tools、為每個所選工具生成 skills 檔），隨即切換至該專案；使用者取消時 app SHALL 維持原專案，且目標目錄 SHALL NOT 產生任何寫入。初始化失敗時 app SHALL 顯示單行錯誤訊息且 SHALL NOT 切換 root。

#### Scenario: 確認後初始化並切入新專案

- **WHEN** 使用者選定不含任何 speclink 標記的空目錄，於確認對話框保持預設（claude）並確認
- **THEN** 該目錄產生 openspec/（含 specs/、changes/archive/、config.yaml）、.speclink.yaml（tools 含 claude）與 .claude/skills/ 技能檔，不產生 CLAUDE.md，且 app 切換至該專案並於看板顯示空清單

#### Scenario: 勾選 codex 時生成對應工具檔

- **WHEN** 使用者於確認對話框加勾 codex 後確認
- **THEN** 目標目錄除 claude 對應檔案外，另產生 .agents/skills/ 技能檔而無 AGENTS.md，.speclink.yaml 的 tools 同時記錄 claude 與 codex

#### Scenario: 取消初始化則零寫入

- **WHEN** 使用者於確認對話框取消
- **THEN** app 維持原專案，所選目錄內容與選擇前完全相同（無任何新檔案或目錄）

#### Scenario: 新專案僅選 Copilot

- **WHEN** 選空目錄、取消預設 claude、勾選 copilot 並確認初始化
- **THEN** 建立 .speclink.yaml 的 tools=[copilot]、既有 openspec 骨架與 .agents/skills 的共享技能，切換至新專案；不生成 Copilot 指令檔

### Requirement: 設定頁圖形化讀寫兩層設定
<!-- BEFORE: AI 工具只含 claude／codex，取消工具按各自目錄清理；語言未設定顯示 English，spec_locale 有 auto 選項。 -->

設定 SHALL 拆分為兩頁：**專案設定頁**（跟隨 active 專案分頁）與**應用程式設定頁**（與任何專案分頁無關）。

專案設定頁 SHALL 以兩頁簽組織，標籤依序為 config.yaml、.speclink.yaml，預設 SHALL 落在 config.yaml 簽：

- **config.yaml** 簽 SHALL 含「專案說明」卡與「產出規則」卡（行為見需求「設定頁編輯專案說明與產出規則」），及「產出政策」卡——locale、spec_locale（下拉）與 tdd、audit（開關）。
- **.speclink.yaml** 簽 SHALL 含「AI 工具」卡——內建工具 claude／codex／copilot 多選，自訂工具描述子原樣呈現為不可編輯項。

應用程式設定頁 SHALL 以兩頁簽組織，標籤依序為本機設定、伺服器，預設 SHALL 落在本機設定簽：

- **本機設定** 簽 SHALL 含「介面語言」卡（行為見需求「UI 介面語言支援 zh-TW 與 en」），並 SHALL 註記其內容僅存於此裝置、不寫入版本庫。
- **伺服器** 簽行為見 desktop-connections 能力的需求「伺服器管理最小面」。

config.yaml 與 .speclink.yaml 簽首 SHALL 以等寬字註記對應檔案路徑。讀取時未設定的語言欄位 SHALL 顯示「未設定（系統語系）」，兩個語言下拉的新選項 SHALL 僅有未設定、tw、ja、en，不提供 auto；其他未設定欄位 SHALL 呈現既有預設值狀態；寫入時 SHALL 僅代換目標鍵——未觸及的鍵（remote、spec_dir、自訂工具描述子等）SHALL 原樣保留；政策欄位設回預設值時 SHALL 移除該鍵而非寫入明值。tools 寫入成功後 app SHALL 按 workspace-tools 的共享同步規則生成／清理技能：codex 或 copilot 任一仍選取就保留 .agents/skills，兩者都未選才清理共享受管檔；SHALL 保留真實工具勾選，不以相同目錄互相代換。AI 工具提示 SHALL 說明勾選管理技能生成與保留，不保證阻止代理發現共享目錄。自訂工具描述子 SHALL 寫入後保留。任一層設定檔解析失敗時，專案設定頁對應頁簽（config.yaml 簽掛工作流層、.speclink.yaml 簽掛應用層）的標籤 SHALL 帶警示點、簽內 SHALL 浮出解析失敗說明且該簽表單 SHALL 停用；應用程式設定頁 SHALL NOT 受任何專案設定檔解析失敗影響。

遠端 workspace 為 active 分頁時，專案設定頁 SHALL 呈現單一 **Workflow** 簽（無 config.yaml／.speclink.yaml 兩簽——tools 屬本機 checkout 概念）：含與 config.yaml 簽同形的專案說明、產出規則、產出政策三卡，內容來自 server 的 policy 文件，簽首 SHALL 以等寬字顯示 policy revision；鍵保留語意與本地一致（未觸及鍵原樣保留、設回預設移除鍵）。儲存 SHALL 帶 expected revision；收到 revision 衝突時 SHALL 原樣保留使用者輸入並浮出逐欄位對照（server 現值｜我的輸入），僅提供「以 server 版重載」與「檢視後以最新 revision 重新提交」兩出口——SHALL NOT 提供未經對照的強制覆寫。policy 寫入權為假（reader）時三卡 SHALL 唯讀、儲存停用附繁中角色說明。應用程式設定頁不受 active 分頁種類影響。

#### Scenario: 兩頁分工與預設簽

- **WHEN** 使用者於 local 專案分頁開啟專案設定頁與應用程式設定頁
- **THEN** 專案設定頁頁簽依序為 config.yaml、.speclink.yaml 且預設落在 config.yaml 簽（含專案說明、產出規則、產出政策三卡，簽首等寬字註記檔案路徑），切至 .speclink.yaml 簽見 AI 工具卡；應用程式設定頁頁簽依序為本機設定、伺服器且預設落在本機設定簽（含介面語言卡與「僅存於此裝置」註記）

#### Scenario: 遠端分頁的 Workflow 簽編輯

- **WHEN** active 分頁為遠端 workspace 且使用者具 policy 寫入權，開啟專案設定頁修改產出政策並儲存
- **THEN** 頁面僅含 Workflow 簽（簽首等寬字顯示 policy revision），儲存成功後 revision 前進，server 端 config 反映新政策且未觸及鍵原樣保留

#### Scenario: 遠端儲存遇 revision 衝突

- **WHEN** 兩個 client 同時編輯同一 scope 的 policy，後儲存者收到 revision 衝突
- **THEN** 後儲存者的輸入原樣保留，浮出 server 現值與我的輸入的逐欄位對照，可選以 server 版重載或以最新 revision 重新提交；無任何未經對照的強制覆寫路徑

#### Scenario: reader 唯讀

- **WHEN** role 為 reader 的使用者於遠端分頁開啟專案設定頁
- **THEN** Workflow 簽三卡呈現現值但唯讀，儲存停用並附角色說明

#### Scenario: 寫入政策欄位且未觸及鍵原樣保留

- **WHEN** config.yaml 原含 rules 區塊與 context 文字，使用者於專案設定頁將 tdd 切為開啟並儲存
- **THEN** 重新讀取 config.yaml 可見 tdd: true，且 rules 與 context 內容與寫入前逐字元一致

#### Scenario: 設回預設值即移除鍵

- **WHEN** config.yaml 原含 locale: tw，使用者於專案設定頁將 locale 改回「未設定（系統語系）」並儲存
- **THEN** 重新讀取 config.yaml 已無 locale 鍵，且引擎解析該檔的有效 locale 依 workflow-config 的系統語系規則決定

##### Example: 政策欄位寫入效果

| 操作前檔案狀態 | 表單操作 | 寫入後檔案效果 |
| -------------- | -------- | -------------- |
| 無 tdd 鍵 | tdd 切開啟 | 新增 tdd: true |
| tdd: true | tdd 切關閉 | tdd 鍵被移除（預設即 false） |
| locale: tw、含 rules 區塊 | spec_locale 選 ja | 新增 spec_locale: ja，locale 與 rules 原樣保留 |

#### Scenario: tools 變更後技能同步

- **WHEN** .speclink.yaml 原 tools 僅 claude，使用者加選 codex 並儲存
- **THEN** .speclink.yaml 的 tools 記錄 claude 與 codex，且專案根新增 .agents/skills/ 技能檔而無 AGENTS.md

#### Scenario: 自訂工具描述子原樣保留

- **WHEN** .speclink.yaml 的 tools 含一個自訂描述子物件，使用者於專案設定頁變更內建工具勾選並儲存
- **THEN** 寫入後的 tools 清單仍含該描述子且欄位內容不變，專案設定頁將其呈現為不可編輯項

#### Scenario: 解析失敗簽級警示

- **WHEN** config.yaml 被外部改壞為無法解析，使用者開啟專案設定頁
- **THEN** config.yaml 頁簽標籤帶警示點；切至該簽可見解析失敗說明，產出政策卡表單與專案說明、產出規則兩卡的編輯鈕停用；應用程式設定頁的介面語言三選仍可正常使用

#### Scenario: 勾選與取消共享工具

- **WHEN** 原 tools=[codex]，加勾 copilot 儲存，再取消 codex並儲存
- **THEN** 第一次 tools=[codex,copilot]、第二次 tools=[copilot]，共用 .agents/skills 始終只有一套且本文不變；重載設定只勾實際保存的工具

#### Scenario: 保留使用者技能

- **WHEN** .agents/skills 含共享受管技能及使用者 my-skill，原勾 copilot，改為只勾 claude並儲存
- **THEN** 共享受管技能清除，my-skill 保留，原 remote、spec_dir及其他自訂描述子不變

#### Scenario: copilot 描述子衝突保存失敗

- **WHEN** 原 tools 含自訂 name: copilot 物件，使用者保存工具選擇
- **THEN** 浮出單行名稱衝突及人工遷移訊息，不自動轉成內建；原配置及技能檔不變，勾選輸入保留供修正

#### Scenario: 遠端設定維持 checkout 邊界

- **WHEN** active 為遠端分頁，開啟專案設定
- **THEN** 仍只有 Workflow，無伺服器 Copilot 選項；revision 衝突、離線或認證失效沿既有錯誤處理，不把本機 tools 寫入 policy

### Requirement: 未啟用資料夾經確認後補齊啟用
<!-- BEFORE: 既有工作區啟用的工具多選只列 claude／codex。 -->

所選目錄向上探索命中 workspace、store mode 為本地檔案、且該 workspace root 不存在 `.speclink.yaml` 時，app SHALL 判定為未啟用 speclink，SHALL NOT 逕行寫入亦 SHALL NOT 直接以既有專案開啟，而 SHALL 顯示啟用確認對話框（含 AI 工具多選 claude／codex／copilot，預設勾選 claude；文案為啟用語意，遵循 openspec/LANGUAGE.md、不出現工程詞）。判定與寫入 SHALL 錨定向上命中的 workspace root，而非使用者所選的子目錄。

使用者確認後 app SHALL 經引擎的工作區補齊入口執行啟用（補 openspec/ 骨架缺件、專案根 `.speclink.yaml` 記錄所選 tools、生成所選工具的 skills 檔（Codex／Copilot 共用一份，不生成指令檔受管區塊）），既有 openspec/ 內容 SHALL 零觸碰，隨即切換至該專案；使用者取消時 app SHALL 維持原專案，目標目錄 SHALL NOT 產生任何寫入。啟用失敗時 app SHALL 顯示單行錯誤訊息且 SHALL NOT 切換 root。`.speclink.yaml` 存在的專案 SHALL 照舊直接開啟，SHALL NOT 出現啟用對話框；向上探索完全未命中的目錄 SHALL 照舊走初始化確認流程。

#### Scenario: 遷移資料夾確認啟用後補齊並切入

- **WHEN** 使用者選定含 openspec/（內有既有規格文件）但無 .speclink.yaml 的資料夾，於啟用確認對話框保持預設（claude）並確認
- **THEN** 專案根產生 .speclink.yaml（tools 含 claude）、與 .claude/skills/ 技能檔，不產生 CLAUDE.md 的受管區塊，openspec/ 內既有文件位元級不變，app 切換至該專案並於看板呈現既有內容

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
- **THEN** 該檔位元級不變，僅補齊其餘缺件

#### Scenario: 既有規格僅啟用 Copilot

- **WHEN** 資料夾已有 openspec 而無 .speclink.yaml，在啟用對話框只選 copilot並確認
- **THEN** .speclink.yaml 記 tools=[copilot]，生成共享技能；既有規格、討論、變更與 config.yaml 位元級不變

### Requirement: 設定頁政策下拉的未知值顯性呈現
<!-- BEFORE: 兩個下拉的代碼集包含可新選的 auto；本次只移除 App 新選項，保留舊值讀取。 -->

專案設定頁的 locale 與 spec_locale 下拉（local 專案的 config.yaml 簽與遠端 workspace 的 Workflow 簽皆適用）在儲存值非空且不在合法可讀代碼集（locale：tw／ja／en；spec_locale：tw／ja／en，以及舊 auto）時，SHALL 於下拉顯示該原始值並帶無效標註與警示樣式，且該欄位下方 SHALL 顯示引導改選合法代碼的提示文字；SHALL NOT 呈現為空白，SHALL NOT 於讀取時自動清空或改寫儲存值（寫入嚴格、讀取寬容）。使用者改選合法選項並儲存後，SHALL 以所選代碼覆蓋原值，下拉 SHALL 恢復正常呈現且提示文字 SHALL 消失。儲存值為空（未設定）或在合法選項集內時，本需求 SHALL NOT 改變既有呈現。

兩個下拉的新選項 SHALL 僅含「未設定（系統語系）」及 tw、ja、en。讀到舊 spec_locale:auto 時 SHALL 顯示「跟隨 locale（舊設定）」，SHALL NOT 把 auto 列為可新選的項目，也 SHALL NOT 自動清空或寫回；使用者改選未設定並儲存後才 SHALL 移除 spec_locale 鍵。修復提示 SHALL 引導未設定或三個明確語言，不推薦 auto；中英文文案 SHALL 同義。

#### Scenario: 未知儲存值顯性呈現且不被改寫

- **WHEN** 專案的 locale 儲存值為「繁體中文」（合法選項集外的字串），使用者開啟專案設定頁
- **THEN** locale 下拉顯示「繁體中文」並帶無效標註與警示樣式，欄位下方出現改選合法代碼的提示文字；未執行任何儲存動作前，儲存端的值仍為「繁體中文」

#### Scenario: 改選合法代碼即修復

- **WHEN** 於上述狀態，使用者將 locale 下拉改選 tw 並儲存
- **THEN** 儲存端的 locale 值成為 tw，下拉正常顯示 tw 選項，無效標註與提示文字消失

#### Scenario: 合法值與未設定不受影響

- **WHEN** 專案的 locale 儲存值為 tw、spec_locale 未設定，使用者開啟專案設定頁
- **THEN** locale 下拉正常顯示 tw、spec_locale 顯示未設定預設狀態，無任何無效標註或提示文字

#### Scenario: 舊 auto 可讀且改選未設定才移除

- **WHEN** 儲存的 spec_locale 為 auto，使用者開啟 Local config.yaml 或 Remote Workflow 設定，展開 spec_locale 選單，改選未設定並儲存
- **THEN** 改選前 trigger 顯示跟隨 locale 的舊設定且無任何寫入；選單僅四項：未設定、tw、ja、en；儲存的目標 JSON specLocale 為 null，由既有 writer 移除該鍵，其他欄位保留，Remote 寫入仍經原 revision 檢查

## ADDED Requirements

### Requirement: checkout 工具選擇與共享技能驗收

桌面已有的本機 checkout 初始化／綁定工具選擇與技能缺失時的啟用入口 SHALL 提供 copilot，保存 string array 的真實選集並使用 workspace-tools 的唯一同步流程。遠端沒有 checkout 時 SHALL 不提供技能安裝；有 checkout 的工具選擇 SHALL 不更動 server policy、認證或 revision。UI 文字 SHALL 延用介面 zh-TW／en，工作流 tw／ja／en 或未設定與中文弱偵測 SHALL 不改變 tools 值或共享目錄。

#### Scenario: checkout 預選保存的 Copilot

- **WHEN** 遠端開啟流程讀取 checkout 的既有 tools=[copilot] 並顯示工具選擇
- **THEN** copilot 勾選、codex 不勾選，確認後仍保存 copilot，生成或補齊 .agents/skills 的共享檔；沒有第二份可寫 openspec

#### Scenario: 本機工具保存的回應型別

- **WHEN** 桌面取得已保存的工具快照／專案探測回應
- **THEN** app.tools／tools 是 string array，包含實際選到的 copilot；既有 camelCase 欄位名與型別不變，不增加伺服器工具設定

#### Scenario: 工具選項不隨產出語言改值

- **WHEN** 介面切換 zh-TW／en，或產出語言為 tw／ja／en／未設定
- **THEN** 同一選集仍寫 copilot／codex 的原始值、共享目錄不變，介面說明依既有語系選擇，中文弱偵測維持

#### Scenario: 雙入口確認實際載入技能

- **WHEN** 在同時啟用 Codex／Copilot 的 fixture，以 Copilot CLI 與 VS Code agent chat 使用 speclink-propose
- **THEN** 兩入口均能讀入共享中性本文，在 fixture 建立變更並觀察 .openspec.yaml 的 created_with=copilot；再啟用 Claude 時核對載入內容仍符合共享契約，不能只以目錄存在或技能清單有名稱宣稱驗收通過
