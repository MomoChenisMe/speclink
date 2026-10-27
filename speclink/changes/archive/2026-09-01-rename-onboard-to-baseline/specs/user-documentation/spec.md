## MODIFIED Requirements

### Requirement: 完整工作流指南說明用途與使用時機
<!-- BEFORE: 涵蓋清單中的站名為 onboard -->

中英文 workflow SHALL 涵蓋 baseline、discuss、propose、apply、ingest、drift、analyze、validate、audit、commit、archive，以及目前可觀察的 verify／evidence 能力；每個階段 SHALL 說明目的、使用與跳過時機、輸入、產物、Agent skill 與底層 CLI／Host 的呼叫層級、完成判準、下一步與常見恢復方式。workflow SHALL 明確區分必經生命週期階段、條件式階段與 utility skill，並 SHALL 說明 skill 是工作流知識、CLI／Host 是執行引擎。

#### Scenario: 需求明確與需求模糊採不同入口

- **WHEN** 使用者比較一項已明確需求與一項仍需取捨的需求
- **THEN** workflow 指示前者直接 propose，後者先 discuss；若只是理解問題且沒有待決事項，SHALL 指示直接問答且不建立 discussion 記錄

#### Scenario: 續作與需求改變採不同入口

- **WHEN** 使用者要恢復一個閒置 change，或實作途中收到會改變 artifacts 的新需求
- **THEN** workflow 分別指示閒置 change 先 drift、需求改變走 ingest，且列出檢查結果如何回到 apply 或再次 ingest

#### Scenario: utility skill 不被誤列為生命週期必經步驟

- **WHEN** 使用者查詢 audit 或 commit 在流程中的位置
- **THEN** workflow 將 audit 說明為安全檢查、commit 說明為限定特定 change 檔案的 Git 工具，且 SHALL NOT 把兩者畫成每個 change 必經的狀態轉移

### Requirement: 工作流正典逐站列出技能與完成判準
<!-- BEFORE: 站別清單中的站名為 onboard；無舊稱補註要求 -->

`docs/workflow.zh-TW.md` 與 `docs/workflow.md` SHALL 以單一結構列出 SDD 全部站別——baseline、discuss、improve、propose、apply、ingest、quality、review、verify、archive 與 worktree 流程——每站 SHALL 載明用途、對應的 `/speclink-*` 技能名稱、完成判準與下一站。讀者 SHALL NOT 需要跨文件拼湊任一站的上述四項資訊。baseline 站 SHALL 於兩語言載明舊稱 onboard，使循舊名而來的讀者可對上新站名。

#### Scenario: 逐站資訊完整

- **WHEN** 讀者在任一語言的工作流文件查找任一站
- **THEN** 該站的用途、對應技能名稱、完成判準與下一站四項均可在該文件內找到

#### Scenario: 循舊名可找到 baseline 站

- **WHEN** 讀者以舊稱 onboard 在任一語言的工作流文件搜尋
- **THEN** baseline 站的段落含「舊稱 onboard」字樣，讀者由此對上新站名
