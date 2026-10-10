## Context

討論已裁定 Copilot CLI／VS Code 共用 Codex 的 .agents/skills 與內容。現有 Tool enum、ToolSelection、SyncPlan 以工具為單位生成與清理，Codex 本文又寫死 $ 前綴及 --agent codex；單純增加枚舉會讓同一目錄被寫兩次並在取消其中一個時被清掉。Node SDK 直接調用 core renderer，桌面工具寫入也已走 core reconcile_builtin_tools，應擴充這些既有落點。

## Goals / Non-Goals

**Goals:** `locale`／`spec_locale` 未設定時各自採用作業系統語言；App 只提供未設定及三種明確語言，舊 auto 可讀；內建 copilot 可由桌面、CLI 與 SDK 使用；Codex／Copilot 在三種非空組合下生成位元級一致的共享技能；工具選集、清理、探測與降級守門一致；相容性拒絕可理解且不寫入。

**Non-Goals:** 不新增 .github/skills 生成、MCP／SDK 工具層、常駐 agent、Server 端工具偏好或認證機制；不注入 AGENTS.md／Copilot 指令檔、不改使用者設定或工具權限；不把其他 custom descriptor 改為共享工具。不更動討論 agent-tool-sdk-layering 的工作。

## Decisions

### D1 工具解析與相容性

在 speclink-core 的 Tool 增 Copilot，name 為 copilot、skills_dir 為 .agents/skills，保留 agents → codex 別名。ToolSelection 保存實際選到的工具，不能因目錄相同就把 copilot 轉成 codex。CLI parse_tools／互動列表、桌面 inspect 與設定快照共用這個解析；UI 列表加 copilot，預設仍沿既有選擇，不自動替既有專案啟用。

ToolDescriptor::validate 將 copilot 納入保留名。原自訂物件 name: copilot 在 update／reconcile 的 guard 前以單行錯誤拒絕，訊息指明 name 衝突並提示「改名 copilot-custom 保留原 skills_dir／invocation；或手動改為內建字串 copilot」，exit 1、整個 checkout 零寫入。不自動遷移，因 tool-call 描述子未必代表 Copilot CLI。其他合法描述子、unknown entry、remote、spec_dir 與頂層未知鍵保留，目錄衝突與逸出驗證維持。

### D2 共享渲染與執行者標記

在現有 RenderTarget::Builtin 的 Codex／Copilot 分支共用一份 substitutions 與共用執行前言，不增加一個轉發 module 或新資產集合。前綴 /speclink: 與殘留 /speclink- 引用轉成 speclink- 技能名稱，而非 speclink 空白加動詞；有效 CLI 命令區塊維持。共享本文去除工具專屬 Plan Mode 提醒，採非 Claude 的既有技能集合與 worktree 條件。

共享前言明示：技能名稱是技能入口，不是 CLI 子指令；動詞仍由 shell 執行；需要互動時用當前代理能提供的詢問方式，無專用詢問工具時直接問使用者並等待，不寫死 Claude 的 API。

propose／review／verify 目前的 --agent {{TOOL}} 片段在共用渲染中移除，命令範例因此仍可直接執行；共享前言再明示「執行 new change 或 review／verify stamp 時，Codex 加 --agent codex、Copilot 加 --agent copilot；不能確認自身工具就省略 --agent」。不得從 .speclink.yaml 勾選推斷執行者，也不得輸出未解析模板。核心既有可選 agent 參數接受字串，不新增 agent API 或環境變數；既有 created_with 與 stamp agent 欄位由實際呼叫記錄。

就 Copilot 整合而言，Claude 及既有 Custom target 的 substitutions／本文不改（資產版號戳同步變動除外）；本輪 D9 的語言說明變更另明示適用所有工具，避免順手改掉 custom neutral 的歷史行為。Codex 及 Copilot 從同一 RenderTarget 行為產出，SDK 也沿用；不採「只勾 Codex 時舊內容、勾 Copilot 時新內容」，因切換勾選不該改共享檔案。

### D3 共享目錄同步與清理

維持 SyncPlan 為唯一計畫來源。在 resolve 裡保留真實選集與報告工具名，但 Codex／Copilot 對 .agents/skills 只建立一個實際寫入 target；直接在現有迴圈按已知共享目錄去重，不引入通用可配置所有權框架。target 的報告可攜帶該目錄選取的工具名集合，UpdateOutcome.updated 仍依 claude、codex、copilot、描述子順序各列一次。

清理按目錄的啟用者集合，而非逐 deselected tool：任一共享工具仍選取就不 prune .agents/skills，兩者都未選才清其中 Speclink 受管技能；空選集仍拒絕，因此清理案例用 tools=[claude]。使用者技能與配置檔不動。探測 tools 仍有選取的 codex／copilot 各一筆，但相同內容與版本來自同一共享讀取結果；differingFiles 路徑去重。降級 guard 對共享目錄只檢查一次，任何更新路徑都不能因只選 copilot 而繞過。

沿用既有寫入順序：解析新配置及所有描述子、建計畫、降級 guard 均在寫入前；reconcile 接著原子寫 .speclink.yaml，SyncPlan 依序剝除已知舊 marker、清理移除描述子的足跡、寫入各實際目錄／清孤兒、清理未使用的內建目錄、記錄 custom 足跡。guard／驗證失敗零寫入；I/O 中途失敗保留已成功寫入檔案，不跨檔回滾，回傳階段與路徑，重試同一選集收斂（既有契約）。新程式只改一次性選集／共享目錄判斷。

### D4 桌面與 CLI 完整入口

CLI init 的 --tools 及互動列表加入 copilot，兩種儲存模式共用；非互動缺 --tools、空選集或未知名仍 exit 1、stdout 空白、stderr 說明、零寫入。成功 exit 0；工具名稱清單新增 copilot，Generated files 路徑去重，其他輸出格式與 --no-color 保護維持。update 仍無新增 stdin 或 JSON，沿用 --allow-downgrade 的顯性例外。

桌面 ProjectSettingsView、WorkspaceChooser 與 App 的初始化／啟用工具選擇都接受 copilot；SettingsSnapshot.app.tools 與 ProjectProbe.tools 按原 string array 表示真實勾選，不新增 IPC 形狀。messages 補共用提示：勾選管理的是 Speclink 生成／保留哪些技能，不是控制各代理能否讀到目錄。write_app_tools／init_project／adopt_project 既有 Tauri command 僅委派 speclink-desktop-core，core 再調同一 workspace 同步；不另寫一套桌面 writer。AssetUpdatePrompt 沿用既有安裝／更新動作，讀取核心共享探測結果，不新增工具選取。

remote 分頁的專案設定仍只有 Workflow，tools 不寫遠端 policy；有本機 checkout 時在現有 checkout 綁定選擇／CLI init remote 接入 copilot。revision 衝突、離線、認證失效維持 Host／Protocol 既有錯誤，拒絕時不降格本地模式、不新建一份可寫 openspec。無 checkout 不提供工具安裝，沿用現有唯讀入口。

### D5 SDK 渲染一致性

speclink-node RenderOptions.target 的 TS union 加 copilot，resolve_matrix 接受它並委派 core builtin renderer；codex、copilot 的相同 specDir 渲染逐位元一致，等同 CLI 生成。同一 unknown target error 的有效名清單加入 copilot。既有 claude／neutral、invocation 解析與 toolName 的 neutral 用法維持，不新增 instruction marker API，亦不修正 node-sdk 規格內與這次無關的歷史欄位。

### D6 資產鎖與操作文件

共享 render 的產物變更須遞增 ASSET_VERSION，使用現有 assets.lock 與 UPDATE_GOLDEN 流程更新 Codex snapshot；Copilot 整合對 Claude／neutral snapshot 只接受版本戳變化；本輪 D9 另將系統語系說明的本文變更納入所有工具 snapshot。golden 加「三種共享選集逐檔一致、共選不重複、切換不刪／不變、worktree 開關不漂」斷言，不為 Copilot 複製一套內容 snapshot。腳本的 Claude 技能群組守門對照核心 registry 的對外技能清單，避免本 checkout 只選 Codex／Copilot 就把合法技能誤判成已移除；仍檢查缺項與殘留，不恢復停用的 Claude 生成物。

configuration、getting-started、sdk-node 的中英版本說明內建名、共用目錄、停用規則、自訂 copilot 遷移、兩工具各自技能入口與真實執行者。不改專案使用者自己已編輯的生成檔，直到 apply 完成程式及版本鎖、確認本 checkout 原工具選集後，才按原選集再生 .agents/skills。使用者原有 .claude 技能刪除與 .speclink.yaml 修改不納入本變更或還原。

### D7 系統語系預設與明確設定優先

Host 在 `process_env_overrides`、`resolve_effective_policy` 及 server bridge 的執行 context 讀取 sys-locale 的 OS 語言。Core 的 `EnvOverrides.system_locale: Option<String>` 只接受注入值，系統 I/O 不進入 core。`resolve_policy` 仍是有效政策的唯一判定入口：Local 的 SPECLINK_* 明確覆寫優先於 config.yaml，未設定語言才使用系統值；Remote 維持 server 政策邊界，不接受 client 的本機覆寫，未設定語言使用執行命令的 server 語言。CLI 與 Node 已經由 Host 組裝輸入，不增加另一套偵測邏輯或 Protocol 欄位。

系統 tag 的語言段不分大小寫，支援連字號、底線、encoding／modifier 分隔；zh 對應 tw，ja 對應 ja，英文、其他語言或無法取得值均使用 en。`locale` 與 `spec_locale` 各自取系統預設，例如 locale 明確設 ja、系統 zh-TW、spec_locale 未設定時，artifact 為日文而 spec 為繁中。舊 spec_locale:auto 保持跟隨有效 locale；spec_locale:en 維持英文及既有內部 None 表示。資料格式不新增政策鍵，不把偵測值寫回檔案，不依 UI localStorage 偏好決定產出語言。

不以只改 App 文案或強制寫入 tw 代替偵測：那會讓 CLI／Node／Remote 仍使用不同預設。採用跨平台 sys-locale，避免自行維護平台 shell 指令；新增相依僅在 Host。壞 config.yaml 仍 fail-closed，系統語系不繞過 YAML 錯誤；偵測不到或不支援的語言靜默落 en，不新增警告。

### D8 App 移除 auto 選項與舊值保留

`ProjectSettingsView` 的兩個下拉提供未設定、tw、ja、en。移除 spec_locale 的 auto SelectItem；中英 help 與未知值修復提示均不推薦 auto。讀到既有 auto 時，SelectValue 顯示「跟隨 locale（舊設定）」且不新增 auto 可選項，不於 hydrate 改寫值；使用者只儲存其他政策時保留原值，改選未設定後才送 specLocale:null，由既有 writer 移除 config 鍵。未知值仍顯性呈現，其他三個合法碼與明確設定不受影響。

不移除後端對 auto 的讀寫支援，也不把舊 auto 靜默轉為未設定，因使用者要求的是 App 選項簡化。Local config.yaml 簽與 Remote Workflow 簽共用同一 UI；遠端 CAS／離線／角色權限與寫入形狀沿用原契約。

### D9 語言說明、資產與既有證據回填

init 的 workflow-config 範本、workflow-config show 的人眼提示、configuration 中英說明，以及 config／baseline／propose／ingest 技能與內建 specs 指引，統一描述未設定使用系統語系。Baseline 仍先取 canonical show JSON；null 的語言指引改為系統語言，legacy auto 在 locale:null 時也採系統語言，rules.specs 與失敗停止規則維持。CLI canonical show JSON 仍保留 null，instructions 注入有效語言；非英文 specs 指引以 Resolved spec_locale 說明，避免把 OS 預設誤稱使用者已寫入的設定。

語言指引是所有工具的共同資產，故本輪 Claude、Custom、Codex／Copilot 生成本文皆接受這個明示變更，並非僅共享 target。ASSET_VERSION 已遞增，golden 與 assets.lock 已再生並在關閉再生開關下通過；本 checkout 依當時真實工具選集同步 .agents/skills，不恢復原本刪除的 .claude 技能或改寫工具選擇。

此 ingest 沿用已通過的 core／CLI／Host／桌面／Node／scripts 測試及使用者實機驗收；原 15 個已完成任務逐字保留，追加已實作範圍的核對與文件同步記錄。版本與測試總數留在 tasks 驗證記錄，不寫死於行為規格。

## Implementation Contract

In scope：上述五個 delta capability 的工具解析、生成／清理／probe、桌面入口、SDK 與文件，以及工作流語言預設、Baseline 語言指引和 App 選項相容。Out of scope：Server API／資料庫、MCP、Agent runtime，以及 unrelated custom neutral render 修正。

| 可觀察面 | 完成條件 |
| --- | --- |
| tools 設定 | 字串 copilot 可選，agents 仍是 codex 別名；custom copilot 在任何寫入前被拒，錯誤附兩種人工遷移 |
| 共享檔案 | [codex]、[copilot]、[codex,copilot] 的 .agents/skills 逐檔相同；切換任何非空共享組合不刪檔；只有 [claude] 清理共享受管技能 |
| CLI | init／update 成功 stdout 沿現有格式；錯誤 exit 1、stderr 說明；無新增 init JSON／stdin；--no-color 無 ANSI |
| 桌面 JSON | app.tools／probe.tools 為 string array；AssetProbe.status／currentVersion／tools／differingFiles 欄位及型別不變；選取兩共享工具時 tools 各一筆、differingFiles 無重複 |
| 共享本文 | 無 $speclink-、無 /speclink- 固定引用、無未替換模板、無技能轉成不存在的 CLI 動詞；CLI 程序照既有格式執行 |
| metadata | Codex 建變更／蓋章記 codex，Copilot記 copilot，無法辨識則省略；不是由勾選或生成目標冒認執行者 |
| 語言預設 | locale／spec_locale 各自使用 OS 預設，明確值優先；zh→tw、ja→ja、其他／缺值→en；不落檔；Remote 採 server 語言；壞 config 不繞過 |
| App 語言選單 | 只有未設定／tw／ja／en，舊 auto 顯示舊設定且保留，改選未設定後移除鍵；local／remote 同形 |
| 技能與 canonical JSON | 所有工具語言說明與 init／show 提示一致；show JSON 的 locale／specLocale 保留字串或 null，不把 OS 預設寫入正典 |
| SDK | skills.render 的 codex 與 copilot 相同，符合 CLI；未知 target／skill 仍明確失敗，無檔案寫入 |

驗收先由 Rust／CLI／桌面 Vitest／Node 渲染測試涵蓋上述矩陣，再由手動任務於具有效 Copilot 權限的 CLI 與 VS Code agent chat 查看已發現的技能路徑、讀入共享 propose 內容、在暫存 fixture 建立變更並確認 .openspec.yaml 的 created_with 為 copilot。勾選 Claude 共存時重做載入來源核對；不以聊天回答「成功」取代檔案內容與 JSON 檢查。人工作業記錄實際工具版本、技能來源、操作與結果，不在 specs 寫死版本。

## Risks / Trade-offs

- Codex 輸出與既有回歸鎖變更 → 同批更新 golden／assets.lock，依 CLI 整合與 SDK 比對確認只改明示範圍；Copilot 整合的 Claude／neutral 只變版本戳；本輪 D9 的共同語言說明變更另在所有工具 golden 明示。
- Windows／macOS／Linux 的路徑與換行差異 → 使用現有 Path 與詞法正規化，差異比對保持 LF／CRLF 正規化；CLI、core 以 fixture 測試，共用受管目錄不假設單一平台。
- Copilot 也發現 .claude/skills 同名技能 → 人工確認 CLI／VS Code 實際選到共享本文；若載入 Claude 特化本文且無法選共享版本，驗收未完成，不改寫 Claude 生成物或宣稱通過。
- 生成檔案無法控制每個工具的載入權限 → UI 明說勾選是產物管理，不承諾取消 Copilot 就讓它讀不到 Codex 的共享檔。
- 共享本文無法靜態知道實際代理 → 前言明定正確 --agent 值與未知省略，代理行為由雙入口實機檢查，不新增不可靠的工具偵測 API。
- 自訂 copilot 名稱變為保留名 → 零寫入拒絕與人工遷移說明；不按猜測把 tool-call 改為 shell。

## Migration Plan

先落核心與回歸測試，再落 CLI／桌面／SDK，同批遞增資產版本與鎖，最後文件及已設定工具的生成物。Codex 專案使用 update 轉為共用本文。舊自訂 copilot 使用者先備份 .speclink.yaml，按錯誤提示改名或改為內建；若換內建，成功 update 清理原受管足跡，使用者自有檔不動。新版共享技能在舊引擎上觸發既有較新版本 guard；回退時使用者選擇恢復備份或顯性 allow-downgrade，不自動降低資產。

## Open Questions

設計決策已定。實機工具版本與權限由 apply 的手動驗收記錄，無環境時保持 [M] 未完成；尚未驗收不視為完成支援。


### 本輪回填的驗收與遷移補充

D7 的驗證目標為 `unset_languages_use_the_injected_system_locale`、`unset_spec_language_uses_system_independently_of_artifact_language`、`explicit_languages_override_system_defaults`、CLI 的 `unset_languages_reach_instructions_without_being_written_to_config` 與 `missing_workflow_config_still_runs_with_defaults`；Host 測試維持注入／policy 與 bridge 契約。D8 使用桌面設定頁的「舊 auto 設定可讀；選單僅有系統語系與明確語言，改選未設定後送 null」互動測試。D9 以 core render_golden、CLI workflow-config／instructions、Node SDK 逐位元渲染與 scripts 守門核對。既有測試紀錄及使用者在系統語系為繁中的電腦確認預設語言，均沿用於本輪。

原來依賴固定英文預設的使用者，可明確寫 locale:en、spec_locale:en 保持英文。新語言預設不用遷移檔案；舊 auto 直到使用者改選未設定並儲存才移除。Unsupported OS 語言仍回英文，App 不擴充更多語言代碼。Claude 共存／清理的實機驗收依使用者決定略過，原功能契約與自動測試保留；此限制已記入 tasks，不以勾選狀態冒稱該部分實測通過。

## 品質補救：Baseline 取得具體語言

Host 的 workflow_languages 驗證正典語言值後，透過既有 Core resolve_policy 搭配 OS 預設解析，不套用 SPECLINK_*。`workflow_languages` 將結果轉為共用 WorkflowLanguages DTO，locale 是顯示名稱、specLocale 是 tw／ja／en（內部英文 None 正規化為 en）。只讀 CLI `workflow-config languages [--json]` 不需建立 change，不落檔。

Remote 經既有 GET /config 取得可選 languages，與正典 content 同一次回應；Server 用自己的 OS 解析。Protocol 對舊回應缺欄仍可反序列化，原 show 照舊輸出 canonical null；新 languages 查詢遇到舊 Server 則明確要求升級，不退回 Client 計算。無新增 API 路由；這個回應資料擴充是為修補 Review／Verify 的資訊缺口。

Baseline 仍由 show 讀取 context／rules.specs，再由 languages.specLocale 取得具體散文語言；兩個查詢任一失敗即停止。Bridge 測試注入 Server OS tag，驗證 proposal／specs 均為日文、client policy／env 不被轉送、config／revision／staged writes 不動。CLI remote mock 覆蓋 null／auto 與 server metadata，API typed-client 測試確認真實 Server 提供資料。政策 Example writer 測試逐列加入 ja，保留 auto 相容測試。

### 品質第三輪：未知舊語言值

只讀 languages 要提供 tw／ja／en 的具體代碼，因此在這個新查詢邊界驗證正典 locale／spec_locale（auto 仍合法），未知值以錯誤停止、不輸出 payload、不改寫設定。既有 show 與設定頁仍寬容讀取舊未知值，以供使用者修復。GET /config 已先驗證 YAML；語言值未知時僅省略可選 languages，content／schema／revision 仍照原讀取。Remote languages 先檢查正典值，故未知設定回報合法代碼，而不是誤報需要升級；合法設定卻缺 metadata 時仍提示 upgrade。Client 邊界也檢查回應的 specLocale，拒絕 Server 回傳不在三個合法碼內的資料。

測試同時涵蓋 spec_locale:zh-Hant、locale:zh-Hant＋auto、locale:JA＋ja；Local／Remote 的 languages 均停止，show 保留原值，config／revision 不變。實際 Server API 從含未知語言的既有 Store 讀取，確認 content 仍可取得、languages 缺席及 revision 不動。
