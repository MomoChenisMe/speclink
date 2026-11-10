---
topic: 我想為review、quality、verify新增設定，可以選擇指定subagent的模型
slug: station-subagent-model
status: promoted
created: 2026-11-10
created_by: MomoChen <momochenisme@gmail.com>
hold: true
promoted_to: station-subagent-model
---

# Discussion: 我想為review、quality、verify新增設定，可以選擇指定subagent的模型

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者想為品質關卡（審查、驗證，及兩站合跑的 quality）新增設定，讓派出的子代理可以指定模型；目前子代理一律沿用主對話的模型，技能原稿沒有任何模型相關指示。需求一開始就有可驗證的目標（每站可指定子代理模型、預設不變），不需要 grill 階段，直接列假設。scout 找到的相關規格為 workflow-config、review-skill、verify-skill、quality-skill、desktop-config；目前沒有進行中的變更，也沒有 in-flight delta 命中。查證事實：Claude Code 的 Agent 工具 model 參數只收 sonnet／opus／haiku／fable 簡稱；Codex（codex-cli 0.167.0-alpha）的 spawn_agent 有 model 與 reasoning_effort 欄位，名稱取自它自己的 model_catalog；兩者另有全域設定（CLAUDE_CODE_SUBAGENT_MODEL 環境變數、Codex config.toml 的 default_subagent_model），但會改到所有子代理。引擎內建工具為 claude／codex／copilot（skills.rs Tool::ALL），另可接自訂描述子（CustomTool.name，kebab-case），技能檔逐工具生成。
Prior discussions: quality-gate-speed, custom-review-rules

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-11-10)

**Focus**: 初始假設——動機、粒度、設定的位置與形狀、技能怎麼套用、範圍
**Position**: 使用者確認假設 1、2，修正假設 3：設定要提供基本模型清單、同時允許自行輸入，且要涵蓋 Claude Code、Codex、Pi 等不同 agent；這也推翻假設 5「只手改 config.yaml」的範圍。
- 決策樹：1 動機 → 2 粒度 → 3 位置與形狀（依賴 1、2）→ 4 技能套用（依賴 3）→ 5 範圍
- 1 已定：預設跟隨主 agent，設定只是保留彈性的選項；與 quality-gate-speed 否決的「子代理改用較小模型」不衝突——那次是為加速改預設，舊理由（複驗輪只要 1–1.5 分、省得少還可能漏問題）擋不到「預設不變的選項」
- 2 已定：只設 review、verify 兩個值，quality 跟隨兩站（quality.md:11 只管時序）；審查兩軸（Standards、Correctness）共用一個值
- 3 修正：值依 agent 分開寫——Claude Code 只收 sonnet／opus／haiku／fable，Codex 收自己 model_catalog 的名稱；agent 不限內建三個，自訂描述子（如 Pi，CustomTool.name 為 kebab-case）也要能設；需要基本清單＋自行輸入
- 4 未被修正，暫定：有設定才帶 model、沒設定不帶；工具拒絕該名稱時原文回報並停下、不默默退回預設；無法派子代理時註明設定不適用；報告開頭一行寫明本輪模型
- 5 受 3 牽動：「提供基本清單」代表要有選取介面，範圍不只引擎與技能
**Ruled out**: 每站只寫一個字串——同時用多個 agent 的專案會把 opus 送給 Codex；只靠工具自己的全域設定（CLAUDE_CODE_SUBAGENT_MODEL、Codex default_subagent_model）——會改到所有子代理、不能分站，也不跟著專案走
**Open**: 基本清單出現在哪裡、設定寫進團隊共用的 config.yaml 還是個人層；各 agent 的基本清單內容與由誰維護；推理強度（effort）是否一併納入（目前假設不納入）

### Round 2 — interview (2026-11-10)

**Focus**: 基本清單的選取介面放在哪裡、設定寫到哪一層
**Position**: 放在 desktop「專案設定」的審查規則處，設定寫進 speclink/config.yaml，由整個專案共用。
- 介面：兩站（審查、驗證）× 本專案啟用的 agent（.speclink.yaml 的 tools＋自訂描述子），每格一個下拉——「沿用主對話」→ 該 agent 的基本清單 → 「自行輸入…」；沒有內建清單的 agent 只有首尾兩項
- 清單由引擎提供、CLI 與 desktop 共用一份，比照審查規則的 smell 名稱（ReviewRulesPage.tsx 讀 reviewBaselineSmells）
- 接受的代價：同一個 agent 的兩個人不能各選各的；用不同 agent 的人各填各的欄，沒有衝突
**Ruled out**: 放個人層（desktop 全域設定或環境變數）——設定不跟著專案走、換機器要重設，使用者選專案共用
**Open**: 各 agent 的基本清單內容與更新方式（模型名稱會隨新版過期）；推理強度（effort）是否一併納入（目前假設不納入）；config.yaml 的鍵名與形狀、寫入指令與 remote 版本衝突

### Round 3 — interview (2026-11-10)

**Focus**: 子代理模型的設定值由 CLI 在執行期取得，還是在生成時寫進技能檔
**Position**: 由 CLI 在執行期取得（使用者提議，查證後採納）——技能檔只寫「去哪裡讀、讀哪一格」，不寫模型值本身。
- 兩站派子代理前以 `speclink workflow-config show --json` 讀設定：審查站第 2 步本來就讀這支（review.md:26），驗證站多讀一次
- 技能檔裡唯一固定的是 agent 名稱，用既有的渲染變數 `{{TOOL}}`（skills.rs:258），它跟著技能目錄走、不隨設定改變
- 基本清單同樣由引擎經 CLI 提供給 desktop，技能檔不寫清單
**Ruled out**: 生成時把模型值寫進技能檔——每次改設定都要跑 `speclink update` 重新生成（本 repo 有 87 個生成的技能檔進版控）；技能檔不再等於引擎正典模板、golden 對照失效；remote 模式會讀到生成當下的本機值而不是 server 的設定（review.md:29 已規定一律經此指令讀、不直接讀檔）；desktop 改完設定還要再生成才生效
**Open**: 基本清單是否採 B（只有 Claude Code 內建 sonnet／opus／haiku／fable，其他 agent 自行輸入）；推理強度（effort）是否納入；config.yaml 的鍵名與形狀、寫入指令與 remote 版本衝突；Codex 可能預設隱藏 spawn_agent 的 model 欄位（expose_spawn_agent_model_overrides）

### Round 4 — interview (2026-11-10)

**Focus**: 各 agent 的基本清單放哪些模型、以後怎麼更新
**Position**: 採 B——只有 Claude Code 有內建清單（sonnet／opus／haiku／fable），其他 agent 用自行輸入；使用者另外提出新分支：能否自行擴展清單，或搭配對應的 provider 取得清單。
- Claude Code 的 Agent 工具只收這四個不帶版本號的簡稱，永遠指向最新版，清單不會過期
- Codex 名稱帶版本號且由網路下載：本機 models_cache.json 列 gpt-6.1-sol、gpt-6-astra 等，Codex 程式內建清單還有 gpt-5.5，兩者已不一致
- 查證 provider 取得的可行性：Codex 有 `codex debug models`（除錯指令，輸出原始模型目錄 JSON）與 app-server 的 `model/list`（標示 experimental）；Pi 有 `pi --list-models [search]`，模型 ID 為 provider/id 形式；Copilot 本機未安裝、無資料
**Ruled out**: A 引擎內建每個 agent 的清單——Codex 清單幾週就過期，每出新模型就要發 speclink 新版；C desktop 直接讀 Codex 的本機快取檔——非公開內部檔案、格式隨時可能改，遠端模式與 server 後台讀不到
**Open**: 自行擴展清單與 provider 取得要做哪一種、還是都不做；推理強度（effort）是否納入；config.yaml 的鍵名與形狀、寫入指令與 remote 版本衝突；Codex 可能預設隱藏 spawn_agent 的 model 欄位（expose_spawn_agent_model_overrides）

### Round 5 — interview (2026-11-10)

**Focus**: 清單要不要讓使用者自行擴展，或由各 agent 的 provider 取得
**Position**: 這次只做乙——config.yaml 加一份自訂清單，每個 agent 一串名稱，desktop 下拉顯示「內建＋自訂」；provider 取得（丙）延後。
- 乙全團隊共用、本機與遠端模式都能用、不執行任何外部指令
- 丙延後的觸發條件：Codex 名稱常換、自訂清單常要改時再議；屆時決定接哪些 agent，以及列表指令放在本機哪一層（例如 .speclink.yaml）
**Ruled out**: 甲只有自行輸入——使用者要能擴展清單；丙這次就做——每個 agent 一個專用接法、依賴對方沒承諾的輸出格式（`codex debug` 是除錯指令、`model/list` 標 experimental）、只有 desktop 本機能跑、server 後台不能用，且指令若寫在 config.yaml，打開設定頁就會執行別人提交或遠端 server 送來的指令
**Open**: config.yaml 的鍵名與形狀（含自訂清單）、解析規則、寫入指令與 remote 版本衝突；推理強度（effort）是否納入；Codex 可能預設隱藏 spawn_agent 的 model 欄位（expose_spawn_agent_model_overrides）

### Round 6 — interview (2026-11-10)

**Focus**: config.yaml 裡子代理模型設定的鍵名、形狀與解析規則
**Position**: 採用頂層 `subagent_model` 區塊，底下 `review`、`verify` 兩站各以 agent 名稱對應模型，`choices` 為自訂清單。
- 形狀：`subagent_model: { review: { claude: opus, codex: gpt-6.1-sol }, verify: { claude: sonnet }, choices: { codex: [gpt-6.1-sol, gpt-6-astra], pi: [...] } }`
- 結構從嚴：只收 review／verify／choices，拼錯即報錯並點名該鍵（比照 review 區塊）；頂層維持寬鬆，舊版 CLI 略過此區塊
- agent 名稱收任何 kebab-case 名稱，不限本機啟用的工具（config.yaml 全團隊共用，同事可能用別的 agent）
- 模型名稱不檢查（清單一直變），名稱錯誤由技能派子代理被工具拒絕時回報；站上的值不必出現在 choices
- 無環境變數覆寫；`workflow-config show --json` 帶出整個區塊與引擎內建清單（claude：sonnet／opus／haiku／fable）；比照 `workflow-config review` 新增寫入子指令供 desktop 使用，remote 做版本衝突檢查
- 推理強度（effort）不納入——使用者沒有要求
- 例：Claude Code 跑 quality → 審查兩子代理用 opus、驗證子代理用 sonnet；Codex 跑 → 審查帶 gpt-6.1-sol、驗證沿用主對話；Pi 跑 → 兩站皆沿用，choices 的 pi 只出現在 desktop 下拉
**Ruled out**: 推理強度一併納入——沒被要求，屬於多加的彈性
**Open**: desktop 設定頁的 UI 設計（使用者要求以 artifact 呈現）；Codex 可能預設隱藏 spawn_agent 的 model 欄位（expose_spawn_agent_model_overrides）；工具拒絕名稱與無法派子代理時的技能行為細節

### Round 7 — interview (2026-11-10)

**Focus**: desktop 設定頁的 UI 初稿（以 artifact 呈現：https://claude.ai/artifact/DLvfwfFSHffGeDGx2hvL2w）
**Position**: 初稿把子代理模型畫成專案設定裡獨立的一頁；使用者看完提出，它應該和審查的設定集合在一起，避免被看成額外的功能——頁面歸屬改由下一輪決定，頁內元件沿用初稿。
- 表格：每列一個 agent（本機啟用的工具＋設定檔裡已出現的 agent），兩欄固定為審查（兩軸共用）與驗證；agent 數量會變、站固定兩個，所以 agent 放列不放欄
- 檢視狀態：有值顯示模型名稱＋來源小字（內建／自訂清單／自行輸入），沒值顯示「沿用主對話」
- 下拉選單：沿用主對話（預設）→ 內建（只有 Claude Code）→ 自訂清單 → 自行輸入…；兩組皆空的 agent 只剩頭尾兩項；選自行輸入時格子下方出現輸入框
- 自訂清單卡：每個 agent 一個純文字編輯區，一行一個名稱、空白行略過；Claude Code 的內建四個唯讀顯示在旁
- 自行輸入的值不自動加進清單；從清單刪掉已被選用的名稱時，格子保留原值、改顯示為自行輸入
- 對照使用者附的實機截圖，初稿少了左側圖示欄與麵包屑、分類項沒有圖示、.speclink.yaml 的組名應為「專案」，重畫時一併修正
**Open**: 子代理模型放在哪一頁、和審查規則怎麼合併；Codex 可能預設隱藏 spawn_agent 的 model 欄位（expose_spawn_agent_model_overrides）；工具拒絕名稱與無法派子代理時的技能行為細節

### Round 8 — interview (2026-11-10)

**Focus**: 子代理模型放在哪一頁、和審查規則怎麼合併
**Position**: 專案設定的分類「審查規則」改名為「品質關卡」，同一頁放審查規則（預設清單、專案規則兩卡）與子代理模型兩個區段。
- 理由：子代理模型含驗證站的設定，放進名為「審查規則」的頁會被讀成審查規則的一部分；「品質關卡」是詞彙表既有正式用詞，定義即審查＋驗證，剛好涵蓋兩者，也避免子代理模型看起來像額外功能
- 頁標題下等寬註記改為 `speclink/config.yaml · review、subagent_model`
- 編輯沿用其他頁的單一頁級編輯鈕；儲存只寫有改過的區塊，兩塊都改時依序寫 review 再寫 subagent_model，第二筆失敗時第一筆已存、頁面停在編輯態並顯示錯誤
- 詞彙變動：「審查規則」由專案設定的分類名降為頁內區段名，「品質關卡」成為分類名，須記進 LANGUAGE.md
- 規格連動：desktop-config 四處寫死的分類清單與「設定頁編輯審查規則」需求的頁名
- artifact 依實機截圖與 speclink desktop 的設計風格重畫
**Ruled out**: 子代理模型獨立成一頁——使用者擔心被看成額外功能；直接塞進「審查規則」頁——驗證站的設定掛在審查的頁名下會誤導；分成審查、驗證兩頁——驗證頁只剩一張模型卡，太薄
**Open**: Codex 可能預設隱藏 spawn_agent 的 model 欄位（expose_spawn_agent_model_overrides）；工具拒絕名稱與無法派子代理時的技能行為細節

### Round 9 — interview (2026-11-10)

**Focus**: 頁面過長：子代理模型與審查規則要不要放同一頁（重開 round 8 的單頁決定）
**Position**: 改成分類欄兩層：「品質關卡」是上層分類，底下「審查規則」「子代理模型」各一頁。
- 量測（Chrome 渲染、視窗 1280×900）：單頁版的子代理模型標題落在約 720px，只露出標題與表頭；規則多的專案（wadpilot 14 條）會推得更遠
- 分類欄：父項「品質關卡」沿用現有分類項外觀，點了開「審查規則」，本身不顯示選中底色；子項無圖示、高 28px、文字與父項文字對齊；子項永遠展開、不做收合
- 元件：@speclink/ui 的 NavItem 新增「子項」變體（不以 className 覆蓋外觀）；settingsCatalog 加父子關係；新增一個頁面路由；config.yaml 解析失敗時兩個子項各掛警示點、父項不掛
- 儲存：每頁只寫自己那一塊（審查規則頁寫 review、子代理模型頁寫 subagent_model），round 8 的「兩塊依序寫入、第二筆失敗」情況不再存在
- 頁名：「審查規則」維持頁名，desktop-config「設定頁編輯審查規則」需求的頁名不變；round 8 記下的詞彙變動縮小為「品質關卡」成為專案設定分類欄的上層分類
- 實測長度：子代理模型頁檢視約 830px、編輯約 960px；審查規則頁約 640px（本 repo 5 條規則），內容與現有頁面相同
- 重開 round 8 否決的「子代理模型獨立成一頁」：當時的理由是會被看成額外功能；上層分類「品質關卡」把兩頁綁在一起，這個理由不再成立
**Ruled out**: 維持單頁、子代理模型移到上面並把自訂清單收合——審查規則被推到下方、自訂清單多一次點擊，且仍要處理兩塊依序寫入；頁內分頁——設定模式 2026-10-10 才以分類欄取代頁籤，再加頁內分頁是走回頭路；子項可收合——收起來又把子代理模型藏起來
**Open**: 子代理模型表格的列要對應 AI 工具頁，且使用者要求新增 pi、cursor 兩個內建工具；Codex 可能預設隱藏 spawn_agent 的 model 欄位（expose_spawn_agent_model_overrides）；工具拒絕名稱與無法派子代理時的技能行為細節

### Round 10 — interview (2026-11-10)

**Focus**: 子代理模型表格的列從哪裡來，以及使用者要求新增 pi、cursor 兩個內建工具
**Position**: 表格的列等於 AI 工具頁列出的全部內建工具（Tool::ALL），順序相同；pi、cursor 加成內建工具另開討論 pi-cursor-tool-support，兩邊誰先上線都不互卡。
- 這份 checkout 的 .speclink.yaml 沒勾選的工具照樣列出，旁邊加灰色「未啟用」；遠端專案沒有 .speclink.yaml 頁，不加標記
- 取代 round 7 的「agent 清單＝本機啟用的工具＋設定檔已出現的 agent」，「自訂 agent」這種列一併拿掉
- pi 本身沒有子代理（README：skips features like sub-agents），pi 列照樣顯示並加說明「pi 本身不派子代理，要裝了子代理擴充套件才會用上這個設定」
- 技能遇到不能派子代理的工具：該站在主對話裡跑，不套用這個設定，報告寫明
- cursor 有子代理，主代理派子代理時可指定模型（未指定＝inherit）；模型名帶版本號（composer-2、gpt-5.6-sol），比照 Codex 不放內建清單，只用自訂清單與自行輸入
- 事實來源：cursor.com/docs/subagents、cursor.com/docs/skills；pi-mono coding-agent README 與 docs/skills.md
**Ruled out**: 只列這份 checkout 已勾選的工具——同事為其他工具設的模型在本機看不到，但該工具執行時仍會套用；pi、cursor 的工具支援併進本討論——目錄（pi 讀的 .agents/skills/ 已放 codex 版、叫用前綴互斥）、cursor 同名技能重複、技能子集、golden／assets.lock、init、desktop、Node SDK、文件都要各自定案，與 github-copilot-tool-support 同等規模，併進來會讓結論變成兩件不相關的事
**Open**: Codex 可能預設隱藏 spawn_agent 的 model 欄位（expose_spawn_agent_model_overrides）；工具拒絕模型名稱時的技能行為

### Round 11 — interview (2026-11-10)

**Focus**: Codex 沒開放 model 欄位、工具不認得模型名稱時，技能怎麼處理
**Position**: 兩種情況分開處理：工具的限制照常派子代理並在報告說明；設定寫錯則該站停下並原文回報。
- Codex 的 spawn_agent 沒有 model 欄位（未開 expose_spawn_agent_model_overrides）：照常派子代理、用主對話的模型，報告寫一行「Codex 沒有開放子代理指定模型，本輪用主對話的模型。要開放，請在 ~/.codex/config.toml 設 expose_spawn_agent_model_overrides = true。」
- 工具不認得模型名稱：該站停在派子代理那一步，原文回報工具的錯誤，並指出要改的地方（子代理模型頁或 config.yaml 的 subagent_model）；不自動退回主對話的模型
- 分法的依據：前者是工具的限制，使用者在專案設定裡改不了；後者是設定寫錯，使用者改得了
- 停在派子代理那一步時審查還沒開始，浪費很少
**Ruled out**: 兩種情況都停下——只因模型偏好沒套用就擋下整個品質關卡，代價太大；名稱錯誤時自動退回主對話的模型——設定錯了不會被發現，審查還會用使用者沒預期的模型跑

## Conclusion

**Decision**: 為審查、驗證兩站新增子代理模型設定：寫在 config.yaml 的 `subagent_model`，依站與 AI 工具指定模型，預設沿用主對話；技能在執行期經 CLI 讀取；desktop 專案設定新增「品質關卡 › 子代理模型」頁。分兩刀。
- 預設不變：沒設定的格子沿用主對話的模型，設定只是給需要的人的選項
- 粒度：只設 review、verify 兩站；/speclink-quality 直接沿用兩站的設定；審查站兩軸（Standards、Correctness）共用一個值；推理強度（effort）不納入
- 相關：pi、cursor 加成內建工具另由討論 pi-cursor-tool-support 處理；表格的列取自 Tool::ALL，那邊落地後自動多出兩列，兩邊誰先上線都不互卡
- **cut 1 `station-subagent-model`**: 引擎設定、CLI 讀寫與兩站技能
  - 形狀：頂層 `subagent_model: { review: { claude: opus, codex: gpt-6.1-sol }, verify: { claude: sonnet }, choices: { codex: [gpt-6.1-sol, gpt-6-astra] } }`；`choices` 是各 AI 工具的自訂清單
  - 結構從嚴：區塊內只收 review、verify、choices，拼錯即報錯並點名該鍵（比照 review 區塊）；頂層維持寬鬆，舊版 CLI 略過此區塊；沒有環境變數覆寫
  - AI 工具名稱收任何 kebab-case 名稱（自訂描述子的工具可手改 config.yaml 使用）；模型名稱不檢查；站上的值不必出現在 choices
  - `speclink workflow-config show --json` 帶出整個區塊與引擎內建清單（只有 claude：sonnet、opus、haiku、fable，這四個簡稱不帶版本號、不會過期）
  - 比照 `speclink workflow-config review` 新增寫入子指令供 desktop 使用，遠端模式做版本衝突檢查
  - 技能：review.md、verify.md 派子代理前以 `speclink workflow-config show --json` 讀 `subagent_model.<站>.<{{TOOL}}>`，有值才帶 model、沒值不帶；技能檔不寫模型值，只靠既有的渲染變數 `{{TOOL}}`；quality.md 不另設定
  - 報告開頭一行寫明本輪子代理用的模型
  - Codex 沒開放 spawn_agent 的 model 欄位時：照常派子代理、用主對話的模型，報告寫一行說明，並提示在 `~/.codex/config.toml` 設 `expose_spawn_agent_model_overrides = true`
  - 工具不認得模型名稱時：該站停在派子代理那一步，原文回報工具的錯誤，並指出要改子代理模型頁或 config.yaml 的 `subagent_model`；不自動退回主對話的模型
  - 工具不能派子代理時（例如沒裝子代理擴充套件的 pi）：該站在主對話裡跑、不套用這個設定，報告寫明
  - 改技能原稿要一併處理 ASSET_VERSION、golden 與 assets.lock
  - 例：Claude Code 跑 /speclink-quality → 審查的兩個子代理用 opus、驗證的子代理用 sonnet；Codex 跑 → 審查帶 gpt-6.1-sol、驗證沿用主對話；Codex 沒開放 model 欄位 → 兩站都用主對話的模型，報告多一行說明
- **cut 2 `desktop-subagent-model-page`**: desktop 分類欄改兩層，新增子代理模型頁（讀寫靠 cut 1 的 CLI）
  - 分類欄：「speclink/config.yaml」組改為專案說明、產出規則、品質關卡（子項：審查規則、子代理模型）、產出流程；遠端專案同樣有這兩頁
  - 父項「品質關卡」沿用現有分類項外觀，點了開審查規則，本身不顯示選中底色；子項沒有圖示、高 28px、文字與父項文字對齊，永遠展開、不做收合
  - @speclink/ui 的 NavItem 新增「子項」變體，不以 className 覆蓋外觀；settingsCatalog 加父子關係；新增一個頁面路由
  - config.yaml 解析失敗時，兩個子項各掛警示點，父項不掛
  - 審查規則頁的內容與頁名都不變
  - 子代理模型頁：頁標題下註記 `speclink/config.yaml · subagent_model`；單一頁級編輯鈕；儲存只寫 subagent_model，與審查規則頁各存各的；未觸及的鍵（含 desktop 沒顯示的 AI 工具名稱）原樣保留
  - 「各站使用的模型」表格：每列一個 AI 工具，取自 Tool::ALL、順序與 .speclink.yaml 頁相同；兩欄固定為審查（兩軸共用）與驗證
  - 這份 checkout 的 .speclink.yaml 沒勾選的工具照樣列出，旁邊加灰色「未啟用」；遠端專案不加標記
  - pi 列附說明「pi 本身不派子代理；裝了子代理擴充套件，這兩格才會用上」（pi 加成內建工具後才出現）
  - 檢視狀態：有值顯示模型名稱加來源小字（內建、自訂清單、自行輸入），沒值顯示「沿用主對話」
  - 下拉選單：沿用主對話（預設）→ 內建（只有 Claude Code）→ 自訂清單 → 自行輸入…；兩組都空時只剩頭尾兩項；選自行輸入時格子下方出現輸入框，輸入的值不自動加進清單
  - 自訂清單卡：每個 AI 工具一個純文字編輯區，一行一個名稱、空白行略過；Claude Code 的內建四個唯讀顯示在旁；從清單刪掉已被選用的名稱時，格子保留原值、改顯示為「自行輸入」
  - 頁尾說明：模型名稱不預先檢查，名稱有誤時品質關卡在派子代理那一步停下並回報工具的錯誤原文
  - 規格：desktop-config 寫死分類清單的四處（分類欄組成、遠端分類欄、解析失敗警示點的掛法）要改
  - 畫面：https://claude.ai/artifact/DLvfwfFSHffGeDGx2hvL2w （第 7 版）
**Rationale**: 使用者要的是保留彈性而不是改預設，所以預設沿用主對話、只在有設定時帶 model，這也和 quality-gate-speed 否決的「為加速改預設用小模型」不衝突。設定放 config.yaml 由整個專案共用，技能在執行期經 CLI 讀取，改設定不必重新生成技能檔，遠端模式也讀得到 server 的設定。只有 Claude Code 有內建清單（四個簡稱不會過期），其他工具靠自訂清單擴展，speclink 不必追著各家模型名稱發版。desktop 用分類欄兩層把子代理模型放在「品質關卡」底下，不會被看成額外功能，每頁也夠短、各存各的。工具的限制（Codex 沒開放 model 欄位、pi 沒有子代理）照常跑並說明，設定寫錯則停下回報，因為前者使用者改不了、後者改得了。
**Rejected alternatives**:
- 每站只寫一個模型字串 — 同時用多個 AI 工具的專案會把 opus 送給 Codex
- 只靠工具自己的全域設定（CLAUDE_CODE_SUBAGENT_MODEL、Codex 的 default_subagent_model） — 會改到所有子代理、不能分站，也不跟著專案走
- 設定放個人層（desktop 全域設定或環境變數） — 不跟著專案走，換機器要重設
- 生成時把模型值寫進技能檔 — 每次改設定都要重新生成，技能檔不再等於模板、golden 對照失效，遠端模式會讀到本機生成當下的值
- 引擎內建每個工具的模型清單 — Codex 名稱幾週就過期，每出新模型就要發版
- desktop 直接讀 Codex 的本機快取檔 — 非公開的內部檔案、格式隨時會改，遠端模式與 server 後台讀不到
- 只有自行輸入、沒有自訂清單 — 使用者要能擴展清單
- 推理強度（effort）一併納入 — 使用者沒有要求
- 審查規則與子代理模型放在同一頁 — 規則清單把子代理模型推到第一個畫面外（1280×900 視窗中標題落在約 720px），且兩塊依序寫入會有部分失敗的狀況
- 子代理模型塞進「審查規則」頁 — 驗證站的設定掛在審查的頁名下會誤導
- 拆成審查、驗證兩頁 — 驗證頁只剩一張模型卡，太薄
- 頁內分頁 — 設定模式才以分類欄取代頁籤，再加頁內分頁是走回頭路
- 分類欄子項可收合 — 收起來又把子代理模型藏起來
- 表格只列這份 checkout 已勾選的工具 — 同事為其他工具設的模型看不到，但那個工具執行時仍會套用
- pi、cursor 的工具支援併進本討論 — 目錄、叫用前綴、同名技能等都要各自定案，規模與 github-copilot-tool-support 相同
- 工具的限制也讓該站停下 — 只因模型偏好沒套用就擋下整個品質關卡，代價太大
- 模型名稱錯誤時自動退回主對話的模型 — 設定錯了不會被發現，審查還會用使用者沒預期的模型跑
**Deferred**:
- 由各工具的 provider 取得模型清單（例如 `codex debug models`、`pi --list-models`） — 每個工具要一個專用接法、依賴對方沒承諾的輸出格式、只有 desktop 本機能跑；等 Codex 名稱常換、自訂清單常要改時再議
- desktop 顯示自訂描述子工具的列 — 目前只列內建工具，自訂描述子的工具可手改 config.yaml；等有人用自訂描述子並要求在 desktop 設定時再議
**Capture to**: proposal | design | spec | tasks | LANGUAGE.md（新增詞條「子代理模型」：品質關卡派出子代理時用的模型，依站與 AI 工具設定，沒設定時沿用主對話的模型）
**Next**: /speclink-propose --from-discussion station-subagent-model
