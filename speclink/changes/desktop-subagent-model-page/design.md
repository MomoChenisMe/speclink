## Context

討論 station-subagent-model 的第一刀（已封存）在 speclink-core 加了 `subagent_model` 區塊：`SubagentModelConfig { review, verify, choices }`（三張以 `ToolName` 為鍵的 map）、常數 `BUILTIN_SUBAGENT_MODELS`（只有 claude：sonnet、opus、haiku、fable）、`ContentEdit::subagent_model`（整塊代換的唯一寫入落點），以及 CLI 的 `workflow-config show --json` 與 `workflow-config subagent-model`。第一刀刻意讓 desktop 只做到能編譯：desktop core 的內容改寫函式解構 `ContentEdit` 時忽略 `subagent_model`，文字手術讓既有區塊原樣保留。本變更是第二刀，也是最後一刀：桌面 app 的「專案設定 › 品質關卡 › 子代理模型」頁。

現況：

- 專案設定分類欄由 `settingsCatalog` 的 `PROJECT_SETTINGS_CATEGORIES` 決定，「工作流程」組四項（專案說明、產出規則、審查規則、產出流程）、「專案」組一項（AI 工具，遠端專案沒有）。`ProjectSettingsColumn` 逐項渲染 `@speclink/ui` 的 `NavItem`；config.yaml 解析失敗時「工作流程」組每一項都掛警示點。
- 各頁共用 `useProjectSettings` 的狀態：快照載入、各卡獨立的編輯態、`guardWrite` 寫入守門、遠端 revision 衝突對照（`PendingRemoteWrite`、`buildConflictRows`、`retryConflict`）、未存修改判定（`dirty`）。審查規則頁的寫入走 `writeWorkflowReview` → Tauri `write_workflow_content`（本機）或 `remote_write_workflow_content`（遠端）→ desktop core 的 `rewrite_workflow_content_text`（內部呼叫 core 的 `update_workflow_config_text`），寫前寫後各驗一次。
- 設定快照 `WorkflowSettings` 由 desktop core 從 config 文字建立，本機與遠端共用同一個入口（`read_workflow_settings_from_text`）。
- 麵包屑在 `App` 組成：專案名 / 專案設定 / 分類名。
- 內建工具清單在前端是 `BUILTIN_TOOLS`（claude、codex、copilot、pi、cursor，與引擎 `Tool::ALL` 同序；pi、cursor 已由 add-pi-cursor-tools 加入）。

討論結論的分類欄寫法「『speclink/config.yaml』組」沿用了畫面稿的組名；desktop-config 規格已規定組名用概念名「工作流程」，本變更依規格寫「工作流程」組。

## Goals / Non-Goals

**Goals:**

- 分類欄改兩層：上層分類「品質關卡」底下有「審查規則」「子代理模型」兩個子項，本機與遠端都有。
- 子代理模型頁：表格選各站各工具的模型、維護自訂清單、只寫 `subagent_model` 鍵，本機與遠端共用同一條寫入路徑。
- 審查規則頁的內容、頁名與寫入行為不變。

**Non-Goals:**

- 不改 speclink-core、speclink-cli、speclink-server：`subagent_model` 的格式、解析、CLI 讀寫與技能套用都在第一刀定案。
- 不檢查模型名稱、不從各工具的 provider 取得清單（討論 Deferred）。
- 不顯示自訂描述子工具的列（討論 Deferred）：手改 config.yaml 寫進去的這類格子與清單在儲存時原樣保留。
- 不做推理強度（effort）、不做個人層設定、不做頁內分頁、子項不做收合（討論已否決）。
- 應用程式設定的分類欄不變。
- server-web 後台沒有專案設定頁，不在範圍內。

## Decisions

### 設定目錄的父子關係與子項 id

`ProjectSettingsCategory` 新增可選欄位 `parent?: "quality"`；`review` 與新的 `subagent` 兩項帶 `parent: "quality"`，兩者都 `remote: true`、都屬 `config` 組，且在目錄裡相鄰（審查規則在前）。子項不渲染圖示，所以 `icon` 改為可選、兩個子項不帶。另以常數 `PROJECT_SETTINGS_PARENTS` 定義上層分類：`quality` 的標籤鍵 `settings.projectSection.quality`、圖示沿用審查規則原本的 `ShieldCheck`、點擊開 `review`。上層分類不是一個 section（沒有自己的頁），所以 `ProjectSettingsSection` 只多 `"subagent"`，store 的 `projectSettingsSection` 與「記住上次所在分類」不必改。

- 替代方案：讓「品質關卡」成為有自己頁面的 section——否決，討論定案點了開審查規則，沒有總覽頁。
- 替代方案：目錄改成巢狀樹——否決，只有一個上層分類、兩個子項，平的清單加 `parent` 欄位就夠，搜尋與遠端過濾的既有寫法不必改。

### 分類欄渲染兩層與警示點

`ProjectSettingsColumn` 照目錄順序渲染；遇到某個 `parent` 的第一個子項時，先渲染上層分類（`NavItem` 預設變體、帶圖示、`active` 恆為 false、點擊呼叫 `onSelect("review")`），再渲染該 `parent` 的子項（`NavItem` 子項變體）。上層分類與子項包在 `role="group"`、`aria-label` 為上層分類名的容器裡。解析失敗的警示點掛在每個子項與其他 config 分類項上，上層分類不掛。

### NavItem 的子項變體

`@speclink/ui` 的 `NavItem` 新增 `variant?: "default" | "sub"`，預設 `"default"`，既有呼叫端外觀不變。`"sub"`：不渲染圖示、高 28px、文字左緣與預設變體的文字左緣對齊（預設變體的文字左緣＝左內距 10px＋圖示 16px＋間距 8px＝34px），選中與 hover 的配色與預設變體相同。`icon` 改為可選，子項變體不傳。外觀只在 `NavItem` 內定義，呼叫端不以 className 覆蓋。

### 麵包屑帶上層分類

`settingsCatalog` 新增 `projectSettingsTrail(section)`，回傳該頁麵包屑在「專案設定」之後的段（標籤鍵，上層分類那段另帶點了要開的頁）：有 `parent` 的頁為 `[上層分類, 頁名]`，其他頁為 `[頁名]`。`App` 的麵包屑在「專案設定」之後接這些段：上層分類那段點了開審查規則頁，最後一段不可點。

### 快照帶出子代理模型與內建清單

desktop core 的 `WorkflowSettings` 新增兩個欄位（camelCase 序列化）：

- `subagent_model: SubagentModelConfig`：取 `WorkflowConfig::subagent_model()`，缺席時三個空 map；解析失敗時給預設（表單停用，不呈現猜測值）。
- `builtin_subagent_models: BTreeMap<String, Vec<String>>`：由 `BUILTIN_SUBAGENT_MODELS` 組成，與 `review_baseline_smells` 一樣由引擎單一提供，前端不另存一份。

本機與遠端都經 `workflow_settings_from_text` 建立快照，兩個欄位的形狀相同。desktop core 再匯出 `SubagentModelConfig`，Tauri 殼與前端型別都從這裡取。

### 內容寫入接上 subagent_model 與雙重驗證

desktop core 的 `rewrite_workflow_content_text_for` 不再忽略 `ContentEdit::subagent_model`：期望值為「`None` → 現值 `current.subagent_model()`；`Some` → 清理後的副本」，清理與 core 寫入相同（模型值去頭尾空白、空白值丟棄；清單條目去頭尾空白、空條目丟棄、空清單丟棄）。`verify_workflow_content_text` 多比對 `cfg.subagent_model()`；寫後回讀的驗證以寫前解析出的 `expected.subagent_model()` 為期望值，與 `review` 相同。改寫本身仍只呼叫 core 的 `update_workflow_config_text`，不另寫一套。

Tauri 殼的 `write_workflow_content` 與 `remote_write_workflow_content` 各加一個參數 `subagent_model: Option<SubagentModelConfig>`，共用的 `content_edit` 把它放進 `ContentEdit`；兩個 command 仍是一行委派。`SubagentModelConfig` 的反序列化經 `ToolName::parse` 檢查工具名稱，非 kebab-case 的名稱在進入 desktop core 前就被拒收。

- 替代方案：desktop 呼叫 CLI 子指令 `workflow-config subagent-model` 寫入——否決，desktop 的其他設定都直接呼叫引擎函式並自帶雙重驗證與遠端 revision；改走 CLI 要多一個子行程，遠端路徑也要另接。討論說的「讀寫靠 cut 1」指的是第一刀的引擎型別與寫入落點，本設計照此實作。

### 前端寫入介面與遠端衝突

`WorkspaceSettingsProvider` 新增 `writeWorkflowSubagentModel(config)`：本機呼叫 `write_workflow_content`（context、rules、review 皆 null），遠端呼叫 `remote_write_workflow_content` 並帶 `expectedRevision`，成功回新 revision。`useProjectSettings` 的 `PendingRemoteWrite` 新增 `{ kind: "subagentModel" }`，`guardWrite`、`handleWriteError`、`adoptFresh`、`retryConflict`、`reloadServerVersion` 與 `dirty` 都納入這一種。衝突對照的列為 `subagent_model.review.<tool>`、`subagent_model.verify.<tool>`、`subagent_model.choices.<tool>`，工具取 server 現值與我的輸入兩邊出現過的名稱聯集，沒值顯示「—」。

### 子代理模型的草稿與儲存

純函式放在 project-settings 目錄新增的 `subagentModel.ts`，頁面與 hook 共用、可單獨測試：

- 草稿：每格一個 `{ mode: "inherit" | "pick" | "custom", value }`；每個工具的自訂清單一份「一行一個」文字。由現值建草稿時：沒值 → inherit；值在該工具的內建清單或自訂清單裡 → pick；其他 → custom（輸入框帶原值）。
- 來源判定：值在內建清單 → 內建；否則在自訂清單（編輯中以草稿清單為準）→ 自訂清單；否則 → 自行輸入。所以從清單刪掉已選用的名稱時，格子保留原值、改顯示為「自行輸入」。
- 存回：以快照的 `subagent_model` 為底，只覆寫 `BUILTIN_TOOLS` 各工具的兩格與清單——inherit 移除該格、pick 寫選的名稱、custom 寫去頭尾空白後的輸入（空白即移除）；清單以「一行一條、去空白、略過空行」轉回，沒改過的清單原樣沿用現值（與審查規則頁同一個做法）。不在 `BUILTIN_TOOLS` 的工具名稱的格子與清單原樣帶回。
- 存回結果與現值相同時不呼叫寫入，直接回唯讀：整塊代換會丟掉區塊內的註解，沒改內容就不該動檔案（與 CLI 的 `unchanged` 一致）。
- 「一行一條」的轉換（`entriesToText`、`textToEntries`、`keepUnedited`）從 `useProjectSettings` 移到同目錄新增的 `lines.ts`，產出規則、審查規則與本頁共用：`subagentModel.ts` 要用它們，而 hook 又要用 `subagentModel.ts`，放在 hook 裡會變成兩個模組互相引用。

### 子代理模型頁的版面

頁面元件 `SubagentModelPage` 沿用既有零件：`SettingsPageHeader`、`SettingsCard`、`PageAlerts`、`PageNote`、`CardEditControls`、`FieldHelp`、`useSaveShortcut`（project-settings 的共用零件）、`@speclink/ui` 的 `Table` 系列與 `Input`，自訂清單的編輯區沿用 desktop 的 `CodeEditor` 純文字模式（與審查規則頁相同）。

- 表格「各站使用的模型」：每列一個 `BUILTIN_TOOLS` 工具；工具欄顯示工具的顯示名（claude → Claude Code、codex → Codex、copilot → Copilot、pi → pi、cursor → Cursor）與等寬的工具名。顯示名放在 `builtinTools` 旁的對照表，依討論定案的畫面稿；產品名不翻譯。
- 「未啟用」只在本機專案、且工具不在快照的 `app.tools` 時顯示。
- 下拉用 `SettingsChoice`。值的哨兵：`__inherit__`（沿用主對話）、`__custom__`（自行輸入…）；同一個名稱同時在內建與自訂清單時只出現在內建那組（Radix Select 的值不得重複）。
- 頁尾說明在檢視與編輯時都顯示；檢視時另加一句說明「未啟用」的意思。

### SettingsChoice 的分組

`SettingsChoice` 的 `options` 可放分組項 `{ label, options }`：每組渲染成 `SelectGroup`＋`SelectLabel`，組與組、組與單項之間以分隔線隔開。`select.tsx` 新增 `SelectSeparator`（包 Radix 的 Separator）。只放單項的既有呼叫端渲染結果不變。

### LANGUAGE.md 與文件

`speclink/LANGUAGE.md`：「品質關卡」的 definition 補「也是桌面 app 專案設定分類欄的上層分類，底下有審查規則、子代理模型兩頁」；「審查規則」的「（專案設定的分類名）」改為「（專案設定『品質關卡』底下的頁名）」；「子代理模型」補「（專案設定『品質關卡』底下的頁名）」。三處只改 definition，不改 avoid 欄，不擴大詞彙守門的掃描面。docs/configuration 中英兩份在「桌面 app 的專案設定改的是同一份檔案」那段補一句子代理模型頁。

## Implementation Contract

**行為**

- 本機專案開專案設定：分類欄「工作流程」組為專案說明、產出規則、品質關卡（子項審查規則、子代理模型）、產出流程；「專案」組為 AI 工具。點「品質關卡」進審查規則頁，選中底色在「審查規則」子項上；麵包屑「專案名 / 專案設定 / 品質關卡 / 審查規則」。
- config.yaml 為 `subagent_model: { review: { claude: opus }, choices: { codex: [gpt-6.1-sol] } }`、`.speclink.yaml` 只勾 claude：子代理模型頁的 claude 列審查格顯示「opus · 內建」、驗證格「沿用主對話」；codex、copilot、pi、cursor 四列旁有「未啟用」，各格「沿用主對話」；自訂清單卡的 codex 顯示 gpt-6.1-sol。
- 編輯時把 codex 的審查格選成 gpt-6.1-sol 並儲存：config.yaml 的 `subagent_model.review` 變為 `{ claude: opus, codex: gpt-6.1-sol }`，其他鍵逐字元不變。
- 遠端專案：分類欄同樣有兩個子項；子代理模型頁的註記顯示 policy revision；儲存帶 expected revision，成功後 revision 前進；檢視者唯讀。

**介面與資料形狀**

- 設定快照 `workflow` 新欄位：`"subagentModel":{"review":{"claude":"opus"},"verify":{},"choices":{"codex":["gpt-6.1-sol"]}}`（三個鍵恆在）、`"builtinSubagentModels":{"claude":["sonnet","opus","haiku","fable"]}`。
- Tauri command：`write_workflow_content` 多收 `subagentModel`（可為 null）；`remote_write_workflow_content` 同。null 表示不動 `subagent_model`。
- 前端：`SettingsSnapshot.workflow.subagentModel: SubagentModelConfig`、`builtinSubagentModels: Record<string, string[]>`；`SubagentModelConfig = { review: Record<string, string>; verify: Record<string, string>; choices: Record<string, string[]> }`；`WorkspaceSettingsProvider.writeWorkflowSubagentModel(config): Promise<number | void>`。
- 目錄：`ProjectSettingsSection` 多 `"subagent"`；`ProjectSettingsCategory.parent?: "quality"`；`PROJECT_SETTINGS_PARENTS`；`projectSettingsTrail(section)`。
- `@speclink/ui`：`NavItem` 的 `variant?: "default" | "sub"`、`icon` 改可選；`SettingsChoice` 的分組項；`SelectSeparator`。
- i18n 新鍵（zh-TW、en 各一份）：上層分類「品質關卡／Quality gates」、子項「子代理模型／Sub-agent models」、頁說明、表格標題與欄名、「沿用主對話」、三種來源標籤、「自行輸入…」、輸入框標籤與說明、「未啟用」、pi 說明、自訂清單卡標題與說明、「自訂 N 個」計數（英文單複數兩形）、頁尾說明。

**失敗模式**

- config.yaml 解析失敗（含拼錯的 `subagent_model` 子鍵）：審查規則、子代理模型兩個子項與其他 config 分類項帶警示點，「品質關卡」不掛；子代理模型頁浮出解析失敗說明，編輯鈕停用。
- 本機寫入失敗：單行錯誤（指明檔案與階段）顯示在頁標題的按鈕列旁，頁面停在編輯態、草稿保留，磁碟檔案維持原內容。
- 遠端 revision 衝突：草稿保留，頁頂浮出逐欄位對照（含 `subagent_model.*` 列），只有「以 server 版重載」與「以最新 revision 重新提交」兩個出口。
- 檢視者：頁面唯讀、編輯鈕停用並附角色說明。
- 存回內容與現值相同：不寫檔，直接回唯讀。

**驗收**

- `cargo test -p speclink-desktop-core`：快照帶出兩個新欄位（含缺席、壞檔）；只寫 `subagent_model` 時 context、rules、review、政策欄位逐字元不變；`subagent_model: None` 時既有區塊（含註解）原樣保留；寫後回讀驗證比對 `subagent_model`。
- `cargo build -p speclink-desktop` 綠（Tauri 殼只加參數）。遠端路徑由兩處涵蓋：desktop core 的 `rewrite_workflow_content_text`（遠端與本機共用的純文字改寫）帶 `subagent_model` 的測試，與前端 `session.test.ts` 斷言 `remote_write_workflow_content` 收到 `subagentModel` 與 `expectedRevision`。
- `npm test -w packages/ui`：`NavItem` 子項變體（無圖示、`aria-current`）、`SettingsChoice` 分組（分組標籤、既有單項呼叫端不變）。
- `npm test -w apps/desktop`：`subagentModel.test.ts` 的純函式（草稿、來源判定、存回合併、相同內容判定）；分類欄兩層與警示點、點上層分類開審查規則頁、麵包屑四段、子代理模型頁的檢視與編輯（來源標籤、未啟用、pi 說明、自行輸入輸入框、刪掉已選用名稱改顯示自行輸入、只寫 subagent_model、未顯示的工具名稱保留、相同內容不寫檔）、遠端衝突列、未存修改的返回確認；i18n 字典測試（zh-TW、en 鍵一致、英文計數有單數形）。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 綠（中英文件對齊、詞彙守門）。

**範圍**

- 在範圍內：desktop core 的快照與內容寫入驗證；Tauri 殼兩個寫入 command 的參數；前端設定目錄、分類欄、麵包屑、子代理模型頁、設定狀態 hook、i18n；`@speclink/ui` 的 NavItem、SettingsChoice、SelectSeparator；LANGUAGE.md 三個詞條的 definition；docs/configuration 中英兩份各一句。
- 不在範圍內：speclink-core、speclink-cli、speclink-server、技能原稿與 golden；應用程式設定的分類欄；server-web；自訂描述子工具的列；模型名稱檢查。

## Risks / Trade-offs

- [`useProjectSettings` 已經很長，新增一種寫入要穿過每個分支] → 草稿轉換、來源判定、存回合併放進獨立的 `subagentModel.ts` 純函式並各自測試；hook 只接上與既有四種寫入相同的守門、衝突與 dirty 流程。
- [整塊代換會丟掉 `subagent_model` 區塊內的註解] → 與 CLI、審查規則頁相同的取捨；內容沒變時不寫檔，避免只按儲存就洗掉註解。
- [頁面沒顯示的工具名稱可能被覆寫] → 存回以快照為底、只覆寫 `BUILTIN_TOOLS` 的格子與清單，測試釘住「手寫的自訂工具名稱在儲存後仍在」。
- [Radix Select 的值不得為空字串、不得重複] → 沿用 `SettingsChoice` 的哨兵慣例；同名項只放內建組。
- [回歸對照] → 不碰技能原稿、golden、assets.lock 與 CLI 輸出；desktop 快照多兩個欄位屬刻意變更，既有前端測試的快照 fixture 補上欄位。
- [跨平台] → 改動在前端與 desktop core 的純文字改寫，沿用既有行尾跟隨（CRLF 檔的代換區塊用 CRLF）；WKWebView 的焦點差異不影響本頁（沒有靠失焦判斷的互動，自行輸入在儲存時才讀）。

**Audit 判定**（apply 期依 `speclink instructions --skill audit` 的三個角色檢查；沒有找到要修的問題，補了一條邊界測試）

- 壞蛋（注入與繞過）：Tauri 的 `subagentModel` 參數經 `SubagentModelConfig` 的嚴格反序列化，工具名稱只收 kebab-case（`ToolName::parse`，`../x` 這類鍵進不來），拼錯的子鍵與非字串的值整份拒收，由 desktop core 測試 `subagent_model_payload_from_the_frontend_takes_the_strict_parse` 釘住；模型名稱依 spec 不檢查，寫入由 serde_yaml 序列化並經寫前、寫後兩次解析比對，名稱裡的 YAML 特殊字元改不到其他鍵；與下拉哨兵值（`__inherit__`、`__custom__`）同名的清單名稱不進選項，選不到「看起來是名稱、實際被當成沿用或自行輸入」的項目。
- 懶惰的開發者（危險預設）：缺席與空值都等於「沿用主對話」；自行輸入留空與清單空行一律丟棄，不會寫出空字串的模型；只按儲存、內容沒變時不寫檔，不會順手洗掉區塊內的註解。
- 搞混的開發者（型別混淆、靜默失敗）：其他三種內容寫入不帶 `subagentModel`，Tauri 收到 `None`＝不動，本頁的寫入明確送 `review: null`，兩頁不會互相覆寫；頁面沒列出的工具名稱以快照為底原樣帶回，不會因為只列內建工具而被靜默刪掉；遠端衝突對照列出兩邊出現過的所有工具名稱（含頁面沒列出的），重送前看得到差異。

## Migration Plan

- 不需要資料遷移：`subagent_model` 缺席等同預設，舊檔照常顯示「沿用主對話」。
- 舊版 desktop 讀到含 `subagent_model` 的檔案時（第一刀之後的版本）原樣保留該區塊；回滾本變更只會少掉頁面，檔案不受影響。
