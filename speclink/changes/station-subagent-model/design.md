## Context

品質關卡的兩站（`/speclink-review`、`/speclink-verify`）與合跑的 `/speclink-quality` 派出子代理時，技能原稿沒有任何模型相關指示，子代理一律沿用主對話的模型。討論 station-subagent-model 定案：在 `speclink/config.yaml` 新增 `subagent_model` 區塊，依站與 AI 工具指定模型，技能在執行期經 CLI 讀取。本變更是 cut 1（引擎設定、CLI 讀寫與兩站技能）；cut 2 desktop-subagent-model-page 之後另開。

現況：

- 設定模型在 speclink-core 的 `WorkflowConfig`（workflow 設定模組）。頂層寬鬆（舊 CLI 略過未知鍵），`review` 區塊以 `ReviewConfig` 從嚴解析（`deny_unknown_fields`），壞檔經 `ConfigError` fail-closed。server 的 `PUT /config` 以同一個 `WorkflowConfig::from_text` 解析拒收，回應 `invalid_config`。
- 寫入的唯一落點是純函式 `update_workflow_config_text`：呼叫端以 `ContentEdit` 指定要換的區塊，函式做行級文字手術（`rules`、`review` 以 serde_yaml 序列化整塊代換），寫完再解析回來比對目標狀態。CLI 本機臂、CLI remote 臂（讀-改-寫加 revision）與 desktop 設定頁都走它。
- CLI 的 `workflow-config show --json` 由 `WorkflowConfigJson` 輸出；`workflow-config review` 子指令把 argv 與 stdin 解析成 `WorkflowConfigWrite`，交給兩臂共用的 `plan_workflow_config_edit`。
- 技能原稿：審查與驗證兩個技能都由「station 開場片段＋`station-shared.md`＋本站原稿」三段串成；`station-shared.md` 的內容兩站共用一份。渲染變數 `{{TOOL}}` 在生成時代換成工具名稱（claude、codex、copilot 或自訂描述子名稱），與 `--agent` 的值相同。
- 查證過的工具事實：Claude Code 的 Agent 工具 `model` 參數只收 sonnet、opus、haiku、fable 四個簡稱；Codex 的 `spawn_agent` 在 `~/.codex/config.toml` 設 `expose_spawn_agent_model_overrides = true` 後才露出 `model` 欄位；pi 本身沒有子代理。

## Goals / Non-Goals

**Goals:**

- `subagent_model` 區塊的讀、寫、嚴格解析，本機與 remote 共用同一套引擎實作。
- `workflow-config show --json` 帶出區塊內容與引擎內建清單，供技能與 cut 2 的 desktop 使用。
- 審查、驗證兩站派子代理前讀設定：有值才帶 model；呈現寫明本輪子代理用的模型；工具不支援、名稱被拒、不能派子代理三種情況各有固定行為。
- `/speclink-quality` 三個子代理各帶自己站的模型。

**Non-Goals:**

- desktop 的「品質關卡 › 子代理模型」頁與分類欄改兩層——屬 cut 2；本刀只讓 desktop 跟上 `ContentEdit` 新欄位而能編譯，行為不變。
- `workflow-config show` 的人眼輸出不加 `subagent_model` 行：討論只要求 `--json`；兩個消費者（技能、desktop）都讀機器輸出；標籤欄寬 13 放不下 `subagent_model`（14 字元），加寬會改動每一行並破壞 golden 對照。
- `subagent-model` 寫入後不做技能檔過期提示：討論沒有要求；文件寫明升級後要跑 `speclink update`。
- 推理強度（effort）不納入；不加環境變數覆寫；不讀 `.speclink.yaml`。
- 不檢查模型名稱、不從各工具的 provider 取得清單（討論 Deferred）。
- pi、cursor 加成內建工具屬討論 pi-cursor-tool-support。

## Decisions

### SubagentModelConfig 的嚴格解析

在 workflow 設定模組新增 `SubagentModelConfig`，掛在 `WorkflowConfig` 的 `subagent_model: Option<SubagentModelConfig>` 欄位（`#[serde(default)]`，缺席或 null 讀作預設）：

- 結構 `deny_unknown_fields`，欄位 `review`、`verify` 為 `BTreeMap<String, String>`（AI 工具名稱 → 模型名稱），`choices` 為 `BTreeMap<String, Vec<String>>`（AI 工具名稱 → 自訂清單）。三個欄位都 `#[serde(default)]`。
- AI 工具名稱以既有的 kebab-case 規則檢查（自訂工具描述子同一個 `is_kebab_case`：2 到 50 字元、小寫字母數字與連字號、頭尾與連續連字號不可），違者整份 config 無法解析，原因點名該名稱。
- 模型名稱與清單條目只收 YAML 字串（同 `RuleEntry` 的作法，不讓 serde_yaml 把 `1`、`true` 轉成文字）；讀取時去頭尾空白。空白或 null 的模型值讀作「沒設定」（該格不出現在 map 裡）；清單比照審查規則清單：空條目丟棄、null 清單讀作空清單。
- 用 `BTreeMap` 而不是保留檔案順序：寫入本來就整塊代換、區塊內註解不保留（比照 `review`），排序後寫出的結果可預期；desktop 的列序由 Tool::ALL 決定，與檔案順序無關。
- 替代方案：頂層 `review` 區塊底下加 `model` 子鍵——否決，驗證站的設定掛在審查區塊下會誤導，討論已定案頂層區塊。

### ContentEdit 新增 subagent_model 作為唯一寫入落點

`ContentEdit` 新增 `subagent_model: Option<&SubagentModelConfig>`：`None` 不動，`Some` 整塊代換。代換在 `update_workflow_config_text` 內與 `rules`、`review` 走同一條路徑——先清理（條目去空白、空條目與空清單丟棄、空白模型值丟棄），再以 serde_yaml 序列化成頂層區塊（純量跳脫由 serde_yaml 負責，不手拼純量），三個欄位都空時移除整個 `subagent_model` 鍵；寫完照既有流程解析回來比對目標狀態。CLI 本機臂、CLI remote 臂與 cut 2 的 desktop 都只透過它寫，不另寫一套。

- desktop 在本刀只調整兩處以通過編譯：desktop core 的內容改寫函式解構 `ContentEdit` 時忽略新欄位，Tauri 殼的 `content_edit` 建構時補預設值。desktop 不寫 `subagent_model`，文字手術讓既有的 `subagent_model` 區塊逐位元保留。
- server 不改程式：`PUT /config` 已用 `WorkflowConfig::from_text` 解析，新區塊的嚴格解析自動生效。

### 內建清單常數 BUILTIN_SUBAGENT_MODELS

在 workflow 設定模組新增常數 `BUILTIN_SUBAGENT_MODELS`，只有一筆：`claude` 對應 `sonnet`、`opus`、`haiku`、`fable`（Claude Code Agent 工具收的四個簡稱，不帶版本號、不會過期）。CLI 的 `show --json` 與 cut 2 的 desktop 都讀這個常數，比照 `REVIEW_BASELINE_SMELLS` 由引擎提供單一來源。其他工具不放內建清單——名稱帶版本號、幾週就過期，靠 `choices` 擴展。

### show --json 的 subagentModel 與 builtinSubagentModels 欄位

`WorkflowConfigJson` 在 `review` 之後新增兩個欄位：

- `subagentModel`：`{"review":{…},"verify":{…},"choices":{…}}`，區塊缺席時三個都是空物件，技能不必先判斷鍵在不在。
- `builtinSubagentModels`：`{"claude":["sonnet","opus","haiku","fable"]}`。

既有欄位與順序不變；人眼輸出不變（見 Non-Goals）。remote 模式由 server 的 config 文字在 CLI 端解析，兩個欄位的形狀與本機相同。

### workflow-config subagent-model 子指令

新增子指令群 `subagent-model`，三個形式：

- `speclink workflow-config subagent-model review <tool> <model>`
- `speclink workflow-config subagent-model verify <tool> <model>`
- `speclink workflow-config subagent-model choices <tool> --stdin`

每個形式都支援 `--dry-run` 與 `--no-color`。`<model>` 為空字串（或只有空白）時移除該格，比照 `set` 以空字串移除 `locale`。`choices` 以 stdin 整段代換該工具的清單（沿用 `stdin_entries`：一行一條、去空白、略過空行），空 stdin 移除該工具的清單。CLI 先以同一條 kebab-case 規則檢查 `<tool>`，違者在讀檔前以非零 exit code 拒絕。解析後成為新的 `WorkflowConfigWrite` 變體，交給既有的 `plan_workflow_config_edit`，本機臂與 remote 臂、`--dry-run` 的差異輸出、remote 版本衝突訊息、壞檔 fail-closed 全部沿用。

- 替代方案：把 `set` 擴成 `set subagent_model.review.claude opus` 的點號鍵——否決，`set` 只管有環境變數覆寫語意的政策欄位，混進來會讓 `set --help` 的鍵清單與值域規則失真。
- 替代方案：站上的值也用 `--stdin`——否決，一格只有一個值，argv 最直接。

### 技能在執行期讀子代理模型

模型值不寫進技能檔；技能檔只寫「讀哪一格」，格子名稱用 `{{TOOL}}`。規則寫在兩站共用的 `station-shared.md` 一次（新段落「Sub-agent model」），兩站各自的派子代理步驟只指向它：

- 讀取：審查站在第 2 步已執行的 `speclink workflow-config show --json` 一併保留 `subagentModel.review`；驗證站在第 3 步之後執行同一指令並保留 `subagentModel.verify`。指令失敗時原文回報並停下，不派子代理、不寫工單。
- 套用：`subagentModel.<站>.{{TOOL}}` 有值 → 每個子代理呼叫都帶這個 model；沒值 → 不帶。審查兩軸共用一個值。
- 三種例外：工具的子代理呼叫沒有 model 參數 → 照常派、用主對話的模型，呈現寫一行說明（Codex 附 `expose_spawn_agent_model_overrides = true` 的開法）；工具拒絕這個模型名稱 → 停在派子代理那一步，原文回報工具的錯誤，指出要改 `speclink/config.yaml` 的 `subagent_model.<站>.{{TOOL}}`（或 desktop 的子代理模型頁），不寫工單、不退回主對話的模型；工具不能派子代理 → 在主對話裡跑、設定不套用，呈現寫明。
- 呈現：審查站第 6 步的開頭在規則行之後加一行模型行，驗證站第 10 步的報告開頭加同樣一行；只屬呈現，不進工單。
- `/speclink-quality`：第 2 步寫明三個子代理各帶自己站的模型；任一子代理因模型名稱被拒時本輪兩站都不記錄（守住「一輪兩站都檢查或都不檢查」），回報錯誤後停下；第 4 步的合併摘要保留兩站的模型行。
- 替代方案：生成時把模型值寫進技能檔——否決（討論 round 3）：每次改設定都要重新生成、技能檔不再等於模板、remote 模式會讀到本機生成當下的值。

### 技能原稿的版號與再生

改了 `station-shared.md`、`review.md`、`verify.md`、`quality.md`，依 skill-authoring 的三連動處理：ASSET_VERSION 升一個 minor 版號（撰寫時為 v1.53.0 → v1.54.0）、以 UPDATE_GOLDEN 重生全部 golden、以 UPDATE_ASSETS_LOCK 重生 assets.lock；接著先 `cargo build -p speclink-cli` 再跑 `speclink update`，讓本 repo 進版控的技能檔跟上。新段落不用強調字（`station-shared.md` 的強調字上限為 0），且含「cannot spawn」的退路句，以通過技能原稿的機械守門。

進行中的 add-pi-cursor-tools 也改 ASSET_VERSION、golden 與 assets.lock，並新增 pi、cursor 兩個內建工具；它已記為依賴本變更，所以本變更先做。若實際順序反過來（它先封存），本變更從它留下的版號再升一版，並一併重生它新增的 pi、cursor 兩份 golden；兩邊的版號行對撞以重生衍生物解決，不挑邊。pi、cursor 加入 Tool::ALL 後，兩站技能的共用段落自動渲染進它們的技能檔，pi 沒有子代理時走 cannot spawn 的退路。

### LANGUAGE.md 的子代理模型詞條

`speclink/LANGUAGE.md` 新增詞條「子代理模型」：definition 為「品質關卡派出子代理時用的模型，依站與 AI 工具設定（`speclink/config.yaml` 的 `subagent_model`），沒設定時沿用主對話的模型」；avoid 只列英文「subagent model（中文散文中）」，不新增中日韓守門詞，避免照出既有掃描面的存量。

## Implementation Contract

**行為**

- 沒有 `subagent_model` 的專案：所有指令與技能行為不變；`show --json` 多出 `subagentModel`（三個空物件）與 `builtinSubagentModels`。
- 設了 `subagent_model.review.claude: opus` 的專案，在 Claude Code 跑 `/speclink-review`：兩個軸的子代理都帶 `model: opus`，結果呈現的規則行之後有一行寫明子代理模型為 opus。
- 在 Codex 跑同一專案且 `subagent_model.review` 沒有 `codex` 這格：子代理不帶 model，模型行寫明沿用主對話。

**介面與資料形狀**

- config.yaml：

  ```yaml
  subagent_model:
    review:
      claude: opus
      codex: gpt-6.1-sol
    verify:
      claude: sonnet
    choices:
      codex:
        - gpt-6.1-sol
        - gpt-6-astra
  ```

- `show --json` 片段：`"subagentModel":{"review":{"claude":"opus","codex":"gpt-6.1-sol"},"verify":{"claude":"sonnet"},"choices":{"codex":["gpt-6.1-sol","gpt-6-astra"]}},"builtinSubagentModels":{"claude":["sonnet","opus","haiku","fable"]}`。
- 寫入成功訊息（stdout 單行，前綴綠色 ✓，`--no-color` 下無色碼）：

  | 指令 | stdout |
  | ---- | ------ |
  | subagent-model review claude opus | ✓ subagent_model.review.claude = opus |
  | subagent-model verify codex "" | ✓ subagent_model.verify.codex removed |
  | subagent-model choices codex --stdin（兩行） | ✓ subagent_model.choices.codex = 2 entries |
  | subagent-model choices codex --stdin（空） | ✓ subagent_model.choices.codex removed |

- 核心型別：`SubagentModelConfig { review, verify, choices }`、`WorkflowConfig::subagent_model()`（缺席時回預設）、`ContentEdit::subagent_model`、常數 `BUILTIN_SUBAGENT_MODELS`。

**失敗模式**

- 區塊內未知子鍵、非 kebab-case 的工具名稱、模型值或清單條目不是字串、`review`／`verify` 不是 mapping、`choices` 的值不是清單 → 整份 config 無法解析；所有讀 config 的指令以非零 exit code 結束，stderr 指出 `speclink/config.yaml` 並點名出錯的鍵或值；remote `PUT /config` 回應 `invalid_config`，store 不變。
- 子指令的 `<tool>` 不是 kebab-case、第一個參數不認得、`choices` 沒帶 `--stdin` → 非零 exit code、stderr 說明，config 逐位元不變。
- remote 讀到寫之間被他人改寫 → 沿用既有的版本衝突訊息，提示重跑。
- 技能：模型名稱被工具拒絕 → 該站停在派子代理那一步並原文回報；這是唯一會擋下品質關卡的情況。

**驗收**

- `cargo test -p speclink-core --lib workspace_group::config::` 涵蓋：嚴格解析的拒收面、空白值讀作沒設定、整塊代換與全空移除鍵、其他鍵逐位元保留、CRLF 檔的行尾跟隨。
- `cargo test -p speclink-cli --test it workflow_config::` 與 `config_fail_closed::` 涵蓋：`show --json` 兩個新欄位、三種寫入形式與成功訊息、`--dry-run` 只顯示 `subagent_model` 區塊的差異、拒收參數、remote 寫入與版本衝突。
- `cargo test -p speclink-server --test it api::policy_write::` 涵蓋：拼錯的 `subagent_model` 子鍵被 `invalid_config` 拒收且 revision 不變。
- `cargo test -p speclink-core --test it render_golden::` 與 `skill_authoring::` 綠：golden 與 assets.lock 已重生，技能原稿通過強調字上限與 cannot spawn 退路檢查。
- `cargo test -p speclink-desktop-core` 與 `cargo build -p speclink-desktop` 綠（desktop 只為編譯跟上）。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 綠（文件的子指令與 CLI help 一致、中英文件對齊、詞彙守門）。

**範圍**

- 在範圍內：speclink-core 的設定模型、改寫函式與四份技能原稿；speclink-cli 的 `show --json` 與 `subagent-model` 子指令；desktop 兩處編譯調整；docs/configuration 中英兩份；LANGUAGE.md 詞條；ASSET_VERSION、golden、assets.lock 與 `speclink update` 再生的技能檔。
- 不在範圍內：desktop 設定頁與分類欄、settings 快照新增欄位（cut 2）；`show` 人眼輸出；寫入後的技能檔過期提示；server 程式。

## Risks / Trade-offs

- [嚴格解析讓拼錯的子鍵擋下所有指令] → 與 `review` 區塊相同的取捨（拼錯不能默默丟掉設定）；錯誤訊息點名該鍵，文件寫明。
- [golden 與 assets.lock 回歸對照] → 技能原稿改動是刻意的；同批以 UPDATE_GOLDEN、UPDATE_ASSETS_LOCK 重生並逐份檢查差異只落在新段落與指向句。`show --json` 加欄位屬刻意變更，人眼輸出逐位元不變，由既有 CLI 測試守住。
- [跨平台] → 改寫沿用既有行尾跟隨（CRLF 檔的代換區塊用 CRLF）；新增的單元測試含一份 CRLF 文件；CLI 測試用 tempdir、不依賴路徑分隔字元。
- [舊技能檔不讀設定] → 升級後要跑 `speclink update`；文件寫明。不做寫入後過期提示（Non-Goals），代價是使用者可能以為設定已生效。
- [Codex 預設不露出 model 欄位] → 技能照常派子代理並寫出開法，不擋品質關卡。
- [工具拒絕名稱的判讀依賴 agent] → 技能把「沒有 model 參數」與「名稱被拒」分成兩種固定文字的情況；Claude Code 的四個簡稱由內建清單提供，desktop（cut 2）下拉減少手打錯字。
- [本 repo 再生的技能檔數量多] → 依記憶的收尾程序，以 git status 盤點 `speclink update` 再生的檔案一併提交。

## Migration Plan

- 不需要資料遷移：區塊缺席等同預設。
- 使用者升級 CLI 後跑 `speclink update` 讓技能檔讀到設定；舊版 CLI 讀到新區塊時略過。
- 回滾：移除 `subagent_model` 區塊即回到原行為；舊版 CLI 不受影響。
