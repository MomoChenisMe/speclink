## Why

第一刀 station-subagent-model 已讓審查、驗證兩站在執行期讀 `speclink/config.yaml` 的 `subagent_model`，但設定只能手改 config.yaml 或打 `speclink workflow-config subagent-model` 指令。透過 AI agent 開發的開發者（團隊情境下含 PO、PM）在桌面 app 調整品質關卡時，看不到這個設定，也不知道各 AI 工具能填哪些名稱。本變更是討論 station-subagent-model 結論的第二刀，也是最後一刀：在桌面 app 的專案設定新增「品質關卡 › 子代理模型」頁，讓使用者用下拉選模型、維護專案共用的自訂清單。

子代理模型頁放在上層分類「品質關卡」底下，與審查規則頁並列，避免它被看成額外的功能；兩頁各自一頁，頁面才不會太長，儲存也各存各的。

## What Changes

- 專案設定分類欄改成兩層：「工作流程」組依序為專案說明、產出規則、品質關卡、產出流程；「品質關卡」是上層分類，底下有兩個子項「審查規則」「子代理模型」，子項永遠展開、不做收合。
  - 點「品質關卡」開審查規則頁；「品質關卡」本身不顯示選中底色，選中底色只落在子項上。子項沒有圖示、高 28px、文字與上層分類的文字對齊。
  - config.yaml 解析失敗時，兩個子項各掛警示點，「品質關卡」不掛。
  - 遠端專案的分類欄同樣有這兩個子項。
  - 麵包屑在兩個子項的頁面為「專案名 / 專案設定 / 品質關卡 / 頁名」，「品質關卡」一段點了開審查規則頁。
  - 審查規則頁的內容與頁名都不變。
- 新增「子代理模型」頁（頁標題下等寬註記 `speclink/config.yaml · subagent_model`，遠端專案顯示 policy revision）：
  - 「各站使用的模型」表格：每列一個內建 AI 工具，順序與 AI 工具頁相同（claude、codex、copilot、pi、cursor）；兩欄固定為審查（兩軸共用）與驗證。本機專案的 `.speclink.yaml` 沒勾選的工具照樣列出，旁邊加灰色「未啟用」；遠端專案不加標記。pi 列附說明「pi 本身不派子代理；裝了子代理擴充套件，這兩格才會用上」。
  - 檢視時，有值的格子顯示模型名稱與來源小字（內建、自訂清單、自行輸入），沒值顯示「沿用主對話」。
  - 編輯時，每格是一個下拉：沿用主對話（預設）→ 內建（只有 claude）→ 自訂清單 → 自行輸入…；選自行輸入時格子下方出現輸入框，輸入的值不加進清單。
  - 「自訂清單」卡：每個工具一個純文字編輯區，一行一個名稱、空白行略過；claude 的四個內建名稱唯讀顯示在旁。從清單刪掉已被選用的名稱時，格子保留原值、改顯示為「自行輸入」。
  - 單一頁級編輯鈕（編輯／取消／儲存 ⌘S）。儲存只寫 `subagent_model` 鍵，其他鍵逐字元不變；頁面沒顯示的 AI 工具名稱（手改 config.yaml 寫進去的）原樣保留；內容與現值相同時不寫檔。遠端專案儲存帶 expected revision，衝突與檢視者唯讀的處理與其他頁相同。
  - 頁尾說明：模型名稱不預先檢查，名稱有誤時品質關卡在派子代理那一步停下並回報工具的錯誤原文。
- `@speclink/ui` 的 NavItem 新增「子項」變體；SettingsChoice 下拉新增可選的分組（分組標籤加分隔線，Select 原語補一個分隔線元件），既有呼叫端不受影響。
- desktop 的設定快照新增 `subagentModel` 與 `builtinSubagentModels` 兩個欄位（取自第一刀的引擎型別與常數 `BUILTIN_SUBAGENT_MODELS`）；本機與遠端的內容寫入指令新增 `subagentModel` 參數，經第一刀的同一個寫入落點（`ContentEdit` 的 `subagent_model`）改寫，不另寫一套。
- `speclink/LANGUAGE.md`：「品質關卡」詞條補上「專案設定分類欄的上層分類」用法，「審查規則」改為「品質關卡」底下的頁名。
- docs/configuration（中英兩份）補一句：桌面 app 的「專案設定 › 品質關卡 › 子代理模型」頁改的是同一個 `subagent_model` 區塊。

不涉及 CLI 指令、設定欄位或技能檔：`subagent_model` 的形狀、預設值（缺席＝沿用主對話）、CLI 讀寫與技能套用都在第一刀定案，本變更只加桌面 app 的畫面。

三種形態下的結果：

- 本機 repo：可用，讀寫 checkout 裡的 `speclink/config.yaml`。
- 桌面 app 連到遠端的 workspace：可用，內容來自 server 的 policy 文件，儲存帶 revision。
- 沒有本機規格檔的內嵌宿主：不適用——本變更只改桌面 app 的設定頁；內嵌宿主要怎麼呈現設定，屬討論 embedded-host-needs 的範圍。

## Capabilities

### New Capabilities

（無）step 3 掃描找到的相關規格為 desktop-config（專案設定各頁）、desktop-app（側欄導覽結構）與 workflow-config（子代理模型區塊的格式，第一刀已定案）；本變更的行為全落在前兩個既有 capability 內。

### Modified Capabilities

- `desktop-config`: 「設定頁圖形化讀寫兩層設定」的分類欄組成、遠端分類欄、解析失敗警示點的掛法與遠端頁數改為兩層；新增需求「設定頁編輯子代理模型」；「設定頁編輯審查規則」的遠端處理改為「與同需求所述其他各頁一致」（不再寫死三頁）。
- `desktop-app`: 「側欄導覽結構」的專案設定分類欄清單與麵包屑改為兩層。

## Impact

- 目標使用者與情境：用桌面 app 管理專案、跑品質關卡（`/speclink-review`、`/speclink-verify`、`/speclink-quality`）前想先指定子代理模型的開發者；設定由專案共用，團隊中 PO、PM 不直接操作。
- Affected specs: desktop-config、desktop-app
- 影響的 crate 與 app：apps/desktop（desktop core 的設定快照與寫入驗證、Tauri 殼的兩個寫入指令、前端設定頁）、packages/ui（NavItem 子項變體、SettingsChoice 分組）。speclink-core、speclink-cli、speclink-server 不改：第一刀已提供型別、常數與嚴格解析，server 的 `PUT /config` 以同一套解析拒收壞內容。
- 影響的技能與工具：無。不改技能原稿，不動 ASSET_VERSION、golden 與 assets.lock。
- Affected code:
  - New:
    - apps/desktop/src/views/project-settings/SubagentModelPage.tsx
    - apps/desktop/src/views/project-settings/subagentModel.ts
    - apps/desktop/src/__tests__/subagentModel.test.ts
  - Modified:
    - apps/desktop/core/src/settings.rs
    - apps/desktop/src-tauri/src/lib.rs
    - apps/desktop/src/adapter/workspace.ts
    - apps/desktop/src/session.ts
    - apps/desktop/src/settingsCatalog.ts
    - apps/desktop/src/builtinTools.ts
    - apps/desktop/src/components/ProjectSettingsColumn.tsx
    - apps/desktop/src/views/ProjectSettingsView.tsx
    - apps/desktop/src/views/project-settings/useProjectSettings.ts
    - apps/desktop/src/App.tsx
    - apps/desktop/src/i18n/messages.ts
    - apps/desktop/src/__tests__/projectSettingsView.test.tsx
    - apps/desktop/src/__tests__/App.test.tsx
    - apps/desktop/src/__tests__/settingsColumn.test.tsx
    - apps/desktop/src/__tests__/session.test.ts
    - packages/ui/src/components/NavItem.tsx
    - packages/ui/src/components/ui/settings.tsx
    - packages/ui/src/components/ui/select.tsx
    - packages/ui/src/__tests__/navItem.test.tsx
    - packages/ui/src/__tests__/settingsParts.test.tsx
    - docs/configuration.md
    - docs/configuration.zh-TW.md
    - speclink/LANGUAGE.md
  - Removed: (none)
