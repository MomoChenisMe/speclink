## Context

審查關卡的 Standards 軸目前由兩個來源決定要檢查什麼：寫死在 speclink-core 技能範本（審查技能範本）裡的 12 條 code smell，以及子代理自行翻找的 repo 文件。專案沒有明確、保證生效、可決定嚴重度的規則入口。討論 custom-review-rules 定案在 openspec/config.yaml 新增頂層 `review` 區塊，並分兩刀交付；本刀（第 1 刀）涵蓋設定區塊、CLI 寫入、審查技能、config 技能與 desktop 設定頁，第 2 刀才處理工單記錄與詳情頁顯示。

現況中與本刀相關的接縫：

- 設定解析：speclink-core 的 WorkflowConfig 以 serde 從整份 config 文件解析；文件存在但無法解析時 fail-closed（ConfigError），不退回預設值。server 的 PUT /config 也以同一個解析把關（invalid_config）。
- 設定寫入：speclink-core 的 update_workflow_config_text 是唯一的文字手術接縫——政策欄位、context、rules 都經它改寫，只動目標頂層區塊、其他行逐位元保留，改寫後再解析並與目標狀態比對（不符即拒寫）。CLI 的 workflow-config 寫入子指令（fs 與 remote 兩臂共用 plan_workflow_config_edit）與 desktop-core 的 context／rules 寫入都呼叫它。
- desktop：前端 adapter 的 writeWorkflowRules → Tauri 指令 write_workflow_content（本機）或 remote_write_workflow_content（遠端，帶 expected revision）→ desktop-core 的 write_workflow_content_at／遠端 workspace 的 write_workflow_content → 上述接縫。產出規則頁（RulesPage）用 CodeEditor 元件與 CardEditControls。
- 技能：審查技能與 config 技能的範本內文改變時，ASSET_VERSION、golden 快照與 assets.lock 三者連動更新。

## Goals / Non-Goals

**Goals:**

- config.yaml 新增 `review` 區塊，嚴格解析，local 與 remote 同語意。
- 一個 CLI 子指令族負責寫入，與既有 workflow-config 寫入共用同一接縫與 remote 讀-改-寫。
- 審查技能依設定套用專案規則與預設清單模式，並補上接受 CRITICAL 的確認與呈現開頭的規則摘要。
- config 技能能詢問模式、接收與檢查審查規則。
- desktop 新增「審查規則」設定頁，編輯體驗與寫入路徑和產出規則頁相同。

**Non-Goals:**

- 工單記錄首輪套用的規則、desktop 審查分頁顯示（第 2 刀）。
- Correctness 軸與審查流程規則的自訂；驗證站的自訂規則。
- 改變引擎蓋章守門（CRITICAL 與 WARNING 照舊都擋章、都可 --accept）；章上標示「帶保留」。
- 改規則讓既有章失效。

## Decisions

### review 區塊以 typed 結構嚴格解析

WorkflowConfig 新增選填欄位 `review`，型別為 ReviewConfig：`baseline`（ReviewBaseline 列舉，序列化為小寫 extend／replace，預設 extend）與 `critical`、`warning`、`suggestion` 三個字串清單（預設空）。ReviewConfig 標上 serde 的 deny_unknown_fields，因此未知子鍵、非法 baseline 值、非字串陣列的清單都由 serde 直接報錯，錯誤訊息點名出錯的鍵或值；這些錯誤經既有 ConfigError 路徑成為 fail-closed。讀取時清單條目即去除頭尾空白並丟掉空白條目（與寫入相同的整理），手寫的空白條目不會進審查；改寫某一個清單時，原樣寫回的其他清單值也不會變。WorkflowConfig 頂層不加 deny_unknown_fields，舊版 CLI 讀到 `review` 只會略過。

- 替代方案：寬鬆解析加警告——打錯鍵時整組規則默默失效，正是討論 config-context-與-rules-gui-編輯 否決自由鍵的理由。
- 替代方案：放在 `rules.review`——會與自訂 schema 中 id 為 review 的 artifact 撞名、被 rules 寫入動詞的 artifact 檢查擋下，且放不下模式開關（討論已否決）。

### 寫入接縫擴充 review 參數

update_workflow_config_text 的內容參數收成一個 ContentEdit（context、rules、review 三個欄位，各自預設為不動），呼叫端只寫要改的區塊（`ContentEdit { review: Some(&r), ..Default::default() }`），日後再加區塊不必改動所有呼叫端。review 為 None 時不動；Some 為整塊代換——先依移除語意正規化（baseline 為 extend 時不寫 baseline 鍵、空清單不寫該鍵、全部為空則移除整個 review 鍵），再以既有的頂層區塊序列化（serialize_top_level_block，經 serde_yaml 產生，保留字元自動加引號）寫入，最後把 review 納入改寫後解析比對的目標狀態。手術沿用 rules 的區塊代換：只換 review 的行範圍，其他行逐位元保留，review 區塊內的註解不保留。CRLF 檔裡，context、rules、review 三種代換區塊都沿用檔案的行尾。

- 替代方案：另立 update_review_text 函式——兩個接縫各自做手術與驗證，日後行為容易分岔；既有接縫已具備整塊代換與驗證，擴充一個欄位即可。
- 替代方案：維持三個獨立參數——每加一個區塊，CLI、desktop-core、遠端寫入與約三十處單元測試都要補一個 None（品質關卡指出的 Data Clumps）。
- Tauri 指令的 IPC 參數維持 context、rules、review 三個鍵（前端契約不變），由 lib.rs 的一個轉換函式組成 ContentEdit。

### CLI 子指令 workflow-config review 的形狀

`speclink workflow-config review` 底下以 clap 巢狀子指令實作四個動作：`critical`、`warning`、`suggestion`（各帶 --stdin 與 --dry-run）與 `baseline <extend|replace>`（帶 --dry-run；值以 ValueEnum 限定）。clap 對未知子指令與非法值本身就以非零 exit code 結束並列出合法值。清單子指令未帶 --stdin 時沿用既有 require_stdin_flag 的用法訊息。WorkflowConfigWrite 新增 Review 變體，由 plan_workflow_config_edit 以目前文件的 review 為底組出完整目標 ReviewConfig 再交給接縫，因此 fs 與 remote 兩臂自動共用同一語意。成功訊息為單行：`✓ review.<list> = <n> entries`、`✓ review.<list> removed`、`✓ review.baseline = replace`、`✓ review.baseline = extend (default)`。

- 替代方案：用 `workflow-config set review.baseline replace`——set 是政策欄位，帶環境變數覆寫與三層解析語意，review 不屬政策欄位，混在一起會讓 set 的語意變得不一致。
- 替代方案：`workflow-config review --baseline replace --critical`等旗標組合——一次指令改多處，dry-run 的 diff 與成功訊息都變複雜；一個子指令改一個鍵與既有 `rules <artifact>` 一致。

### show 輸出帶出 review

`workflow-config show --json` 的 payload 新增 `review` 物件（baseline 字串與三個陣列），未寫 review 時輸出預設值，欄位一律存在，呼叫端不必處理缺欄。人眼輸出於 rules 行之後新增 review 行，值為 none 或「<baseline>, critical <n>, warning <n>, suggestion <n>」。這是刻意的輸出變更，其餘行逐位元不變，CLI 測試同批更新。

### 12 條預設清單由引擎單一提供

speclink-core 在技能模組旁定義公開常數 REVIEW_BASELINE_SMELLS（12 個名稱，順序同技能範本），並以單元測試確認審查技能範本內每個名稱都以 smell 清單項目的形式出現、且清單項目恰為 12 條。desktop-core 的設定快照帶出這份清單，前端只顯示、不另存。遠端專案時快照同樣由本機引擎提供這份清單。

- 替代方案：前端寫一份常數——兩份正典會漂移，改技能範本時沒有東西擋。

### 審查技能套用專案規則

審查技能範本（英文）修改五處：

- 守門步驟後以 `speclink workflow-config show --json` 讀取 review 物件；讀取失敗即原樣回報並停止。選這個動詞的理由：它已輸出正典值、remote 模式可用、不必改引擎的任何 payload；把規則塞進 review scope 的輸出會改到品質站的 payload 與 golden，留給第 2 刀判斷。
- Standards 軸簡報依嚴重度分組帶入規則原文；baseline 為 replace 時不帶 smell baseline 段。Correctness 簡報不帶規則。
- 嚴重度由規則所在清單決定、不得另行升降級；finding 描述原文引用被違反的規則；專案規則與 repo 文件衝突時專案規則勝出（它是專案為審查站明確寫下的清單）。Standards 軸原本「只用 CRITICAL 與 SUGGESTION 兩級」的說明改為：warning 清單的違反記 WARNING。裁量分類的必修定義補上 critical／warning 清單的違反。
- 結果呈現開頭先印一行規則摘要（模式與三個清單條數；replace 且皆空時寫明只依 repo 文件），依 locale 撰寫，不寫入工單。
- 第 9 步的三個接受途徑（修正後重審時不修、接受現狀蓋章、依推薦 --accept）在所接受的必修項含 CRITICAL 時，先逐條列出 CRITICAL 原文並以互動工具確認一次；不確認則回到選擇節點。

### config 技能管理審查規則

config 技能範本新增：review.baseline 列為第六問（帶現值、不推斷）；明定不從固定輸入來源或 repo 文件推導審查規則；詢問是否新增或修改審查規則，依語氣建議分格（「不得」「一律」→critical、「要」「需」→warning、「考慮」「盡量」「宜」「偏好」→suggestion）並由使用者確認；既有與新收規則照判準檢查（extend 時對照生成的審查技能檔淘汰重述 smell 的規則、replace 時不做此項；不寫會過時內容；引用靜態核實）；寫入列出 `speclink workflow-config review ... --dry-run` 的用法，同意後才去掉 --dry-run 重跑。範本開頭對文件的描述補上 review 區塊。

### desktop 沿用產出規則的寫入路徑新增審查規則頁

- desktop-core：WorkflowSettings 新增 `review`（camelCase：baseline、critical、warning、suggestion）與 `reviewBaselineSmells`；write_workflow_content_at 與 rewrite_workflow_content_text 改收一個 ContentEdit，傳給同一接縫，寫後回讀驗證涵蓋 review。
- Tauri：write_workflow_content 與 remote_write_workflow_content 新增選填參數 `review`，仍為單行委派；遠端 workspace 的 write_workflow_content 同步改收 ContentEdit。不另立 Tauri 指令：本機與遠端維持同一對指令，既有的 revision 衝突與 reader 處理自動涵蓋新頁。
- 前端：adapter 新增 writeWorkflowReview（本機與遠端兩實作，context 與 rules 傳 null）；useProjectSettings 新增 review 的唯讀值、編輯態、草稿、儲存與「整份換掉且三節皆空」判斷；新檔 ReviewRulesPage.tsx 以 @speclink/ui 的 Card、SettingsPageHeader、SettingsRow、SettingsChoice（預設清單為列右側下拉，與產出流程頁 locale 同一元件）、Button（「查看 12 條」與產出流程頁「檢視內容」同一種 outline 小按鈕）與既有的 CodeEditor、CardEditControls、PageNote、PageAlerts、FieldHelp 組成；嚴重度籤改用 @speclink/ui 新增的共用 SeverityTag，工單（TicketView）與分析面板（AnalyzePanel）原本各寫一份的色表一併改用它，三處同色同尺寸；草稿存回時沒編輯的節原樣沿用現值，手寫的多行規則不會被拆開（產出規則頁同一組轉換一併修正）；遠端衝突重送成功或重讀快照後，不在編輯中的各區一併採納 server 最新值，產出流程動作（切換、fork、建立、刪除）進行中則停用各頁的寫入入口，等待期間的改動不會被重讀蓋回；settingsCatalog 在 rules 與 workflow 之間新增 id 為 review 的分類（group config、remote true、ShieldCheck 圖示）；ProjectSettingsView 依分類 id 渲染新頁；i18n 新增 zh-TW 與 en 文案。分類的解析失敗警示點沿用 config 群組的既有判斷。
- 畫面依討論定稿的草稿：https://claude.ai/artifact/RNqumZRVKjC7AWox64Z2pu （設定頁唯讀與編輯中兩張）。

### desktop-app 的側欄需求以進行中變更的版本為底

desktop-app 的「側欄導覽結構」同時被進行中的 desktop-swipe-back-finger-lift 修改（觸控板返回），而它的文字列出專案設定分類。本刀的 MODIFIED 區塊以該變更的版本為底，只在兩處分類清單加上審查規則。封存順序因此必須是 desktop-swipe-back-finger-lift 先、本刀後；若該變更在本刀封存前再改這條需求，本刀封存前需以新版重做這個區塊。

### 詞彙與文件

openspec/LANGUAGE.md 新增「審查規則」詞條（definition：config.yaml 的 review 區塊——審查關卡 Standards 軸的預設清單模式與依嚴重度分組的專案規則；avoid：review rules（中文散文中）、自訂規則（指此概念時）；why：與「產出規則」同族、字面即可推出用途）。docs/configuration.md 與 docs/configuration.zh-TW.md 的設定檔表、workflow-config 子指令表與欄位表補上 review。

### 技能版號與生成物

審查與 config 技能範本內文改變，ASSET_VERSION 由 v1.44.0 升為 v1.45.0。審查技能新增的段落必須寫出 CRITICAL 嚴重度籤（清單對應、呈現開頭行範例、接受確認與自我檢查清單各一處），而技能原稿守門把全大寫的 CRITICAL 計入強調字，review.md 原本恰在登記上限 16；其餘不必要的全大寫字已改寫掉，登記表中 review.md 的上限依規範改為 20（crates/engine/speclink-core/tests/it/skill_authoring.rs）。以 UPDATE_GOLDEN 再生六份 golden 快照、以 UPDATE_ASSETS_LOCK 再生 assets.lock；先 cargo build 再執行 `speclink update`，讓本 repo 自身的技能檔與指令檔換成新版，收尾以 git status 盤點這些再生檔一起提交。

## Implementation Contract

**行為：**

- 使用者在 config.yaml 寫 `review: { critical: [...], warning: [...], suggestion: [...] }`（可加 `baseline: replace`）後，`/speclink-review` 的 Standards 軸依這些規則檢查，違反時以規則所在清單的嚴重度入單、描述原文引用規則；replace 時不檢查 12 條 smell。
- `speclink workflow-config review <critical|warning|suggestion> --stdin` 與 `speclink workflow-config review baseline <extend|replace>` 寫入 review；`speclink workflow-config show [--json]` 帶出 review。
- `/speclink-config` 詢問 baseline、接收並檢查審查規則、經 dry-run 核准寫入。
- desktop 專案設定多一個「審查規則」頁，可唯讀檢視、展開 12 條名稱、編輯三個清單與模式、儲存只改 review。

**介面與資料形狀：**

- config.yaml：`review: { baseline?: extend|replace, critical?: string[], warning?: string[], suggestion?: string[] }`；寫出時 extend 不寫 baseline、空清單不寫、全空移除 review。
- `workflow-config show --json`：新增 `review: { baseline: "extend"|"replace", critical: string[], warning: string[], suggestion: string[] }`，恆存在。
- desktop 設定快照：`workflow.review`（同上形狀）與 `workflow.reviewBaselineSmells: string[]`（12 個名稱）。
- Tauri：`write_workflow_content(root, context, rules, review)`、`remote_write_workflow_content(..., context, rules, review, expectedRevision)`，review 為 null 時不動。

**失敗模式：**

- review 區塊寫錯（未知子鍵、非法 baseline、非字串清單）：所有讀 config 的指令 fail-closed，stderr 點名鍵或值；desktop 該群組分類帶警示點、頁內顯示解析原因、編輯停用；server 回 invalid_config。
- CLI 寫入：未帶 --stdin、未知子指令、非法 baseline 值皆非零 exit code 且無檔案效果；壞檔拒寫；remote 版本衝突非零並提示重跑。
- 審查技能讀不到設定即停止，不在缺規則下開審。

**驗收：**

- speclink-core 單元測試：review 解析（預設、合法、未知子鍵、非法 baseline、清單型別錯、讀取時整理條目）、接縫寫入（代換、移除語意、保留字元引號、其他行逐位元保留、CRLF 文件的三種區塊）、REVIEW_BASELINE_SMELLS 與各工具生成的審查技能檔一致。
- CLI 整合測試（workflow_config.rs）：review 各子指令的寫入、dry-run、錯誤路徑、show 的人眼 review 行與 --json review 欄位；remote 對等測試（remote_verb_parity.rs）涵蓋 review 寫入。
- server 測試（policy_write.rs）：review 區塊寫錯的 PUT /config 回 invalid_config 且不落盤。
- golden：`cargo test -p speclink-core --test it render_golden::` 在再生後通過，快照含審查技能的新段落與 config 技能的第六問。
- desktop-core 測試：快照帶出 review 與 12 條名稱、write_workflow_content_at 只寫 review 並保留其他鍵。
- desktop 前端測試：projectSettingsView 測新頁的唯讀、展開、編輯儲存、整份換掉提示、解析失敗停用；remoteDataSource 與 session 測 adapter 的參數。
- 手動：在 desktop 實際開審查規則頁編輯並儲存，確認檔案效果與畫面。

**範圍：**

- 在內：上述設定、CLI、兩份技能範本、desktop 設定頁、LANGUAGE.md 詞條、docs/configuration 兩份、ASSET_VERSION 與生成物。
- 在外：工單格式、review show 的輸出、desktop 審查分頁、驗證站、蓋章守門與章欄位、Correctness 軸。

## Risks / Trade-offs

- [review 區塊寫錯會讓所有讀 config 的指令拒絕執行] → 錯誤訊息點名鍵或值；desktop 頁內顯示解析原因；docs 寫明格式。這與現行 config 壞檔的行為一致，是討論刻意選的嚴格度。
- [回歸對照：show 人眼輸出新增一行、技能內文變更] → 只新增 review 行、其餘逐位元不變，CLI 測試同批更新；golden 與 assets.lock 依 UPDATE_GOLDEN／UPDATE_ASSETS_LOCK 再生並以 render_golden 測試確認。
- [接縫簽名變更牽動多處呼叫端] → 呼叫端皆在本 repo；測試補 None，編譯器會列出所有漏改處。
- [desktop-app 需求與 desktop-swipe-back-finger-lift 重疊] → 本刀區塊以其版本為底，封存排在它之後；它若再改這條需求，本刀封存前重做區塊。
- [平行變更同時提升 ASSET_VERSION] → 版號行對撞時以重生衍生物解決（重跑 UPDATE_GOLDEN、UPDATE_ASSETS_LOCK 與 speclink update），不挑邊。
- [跨平台：Windows 的 CRLF config] → 接縫的行尾保留邏輯涵蓋 context、rules、review 三種代換區塊，單元測試以 CRLF 文件驗證寫入後每一行都是 CRLF、沒有重複的 CR；路徑與指令行為不涉平台差異。
- [舊版 CLI 不套用規則] → 舊版只會略過 review、不會失敗；docs 註明需要新版 CLI 與 `speclink update` 後的技能檔。

## Migration Plan

- 沒有寫 review 的專案行為不變，不需遷移。
- 升級後執行 `speclink update` 取得新版技能檔；desktop 會依既有機制提示技能檔過期。
- 回滾：移除 config.yaml 的 review 區塊即回到預設行為。

## Open Questions

(none)
