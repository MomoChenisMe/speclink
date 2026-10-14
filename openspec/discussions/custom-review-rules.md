---
topic: 我想要讓review這個關卡的規則是可以讓使用者自訂的，除了預設的規則（目前設計的review規則）外，也可以自己定義
slug: custom-review-rules
status: promoted
created: 2026-10-14
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: review-rules-config, review-ticket-applied-rules
---

# Discussion: 我想要讓review這個關卡的規則是可以讓使用者自訂的，除了預設的規則（目前設計的review規則）外，也可以自己定義

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者希望審查關卡（/speclink-review）的檢查規則可由使用者自訂：保留目前設計的預設規則，也能自己定義。需求已有可驗證目標（預設規則之外可加自訂規則），未進入拷問階段，直接以假設清單開場。偵察命中的正式規格：review-skill（Standards 軸的 smell baseline、審查結果的裁量分類）、review-station、workflow-config（context 與 rules 寫入：rules 鍵限目前 schema 的 artifact id）、quality-skill；目前沒有進行中的變更。現況：12 條 smell 清單逐字寫死在技能範本 crates/engine/speclink-core/assets/skills/review.md；Standards 軸會自行翻 CLAUDE.md／AGENTS.md／CONTRIBUTING／lint 設定當 repo 標準，但不保證讀到、remote 模式也不在 server 上。
Prior discussions: review-skill-smell-baseline, config-skill-rule-alignment, config-context-與-rules-gui-編輯, code-review-stage, code-review-convergence-boundary, stamp-blocking-set-alignment, schema-engine-openspec-parity

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-14)

**Focus**: 開場五條假設——自訂規則接在哪、能做到什麼程度、存哪、多嚴重、怎麼讀
**Position**: 假設 1、4、5 使用者未異議而成立；假設 2 被修正為「加規則與整份換掉都要」；假設 3 的 schema 影響經查證後動搖。
- 成立 1：自訂規則只接 Standards 軸，Correctness 軸（抓 bug）不開放（review.md:71、:95）
- 成立 4：違反自訂規則照「文件寫明的 repo 標準」處理，明確違反記 CRITICAL（review.md:93、:113）
- 成立 5：技能用動詞讀規則、不直接讀檔，remote 同語意
- 修正 2：使用者要求除了加規則，也要能整份換掉預設規則
- 查證 3（rules.review 的 schema 影響）：載入不受影響——rules 是任意鍵對照表、載入不比對 schema（config.rs:353）；注入不受影響——只依 schema artifact id 注入（instructions.rs:177）
- 查證 3：CLI 寫入會擋——workflow-config rules review 回 Unknown artifact（speclink-cli verbs/config.rs:447-456）
- 查證 3：desktop「產出規則」只畫 schema 分節，看不到 review；存檔會保留 schema 外的鍵、不會刪（useProjectSettings.ts:399-402）
- 查證 3：自訂 schema 可定義 id 為 review 的 artifact（只要求小寫 kebab-case、無保留字，schema.rs:486-493），rules.review 會同時代表兩件事
- 查證 3：config.yaml 不拒絕未知頂層鍵（WorkflowConfig 無 deny_unknown_fields，config.rs:337-354），舊版 CLI 遇新區塊只會忽略
**Ruled out**: 只能加、不能整份換掉——使用者要求兩者都要
**Open**: 整份換掉的範圍（只換 12 條 smell，或含 Correctness 軸與流程規則）；儲存位置（rules.review 或新的頂層 review 區塊）；換掉模式下自訂規則的嚴重度是否要分硬性／建議；換掉模式配空清單怎麼處理；desktop 設定頁的編輯入口與用詞；verify 是否比照

### Round 2 — interview (2026-10-14)

**Focus**: 整份換掉的範圍與規則的儲存位置
**Position**: 「整份換掉」只限 Standards 軸的 12 條 smell 清單，規則存在 config.yaml 新的頂層 `review:` 區塊（`baseline: extend|replace`＋`rules` 清單），desktop 設定頁也要能編輯。
- 可換：12 條 smell 清單；不可換：Correctness 軸（是工作方向不是清單，換掉＝關掉抓 bug）與流程規則（首輪／複驗、三級嚴重度、必修／可裁界線、工單格式——蓋章、擋封存、章失效都靠它們，review-station 規格）
- `baseline` 省略＝extend：12 條照舊、自訂規則加在上面；replace：自訂規則取代 12 條
- review-skill 規格「Standards 軸的 smell baseline」要改為條件套用；前討論 review-skill-smell-baseline 只裁定逐字照抄、未否決關閉，不算推翻
- 獨立區塊避開三個問題：與自訂 schema 的 review artifact 撞名、LANGUAGE.md「產出規則」定義衝突、rules 清單放不下模式開關
- 舊版 CLI 遇新區塊只會略過（config.rs:337-354）
- 前討論 config-skill-rule-alignment 否決「另開新欄位」的理由是 rules 管道已足夠；本案 rules 會撞名且放不下開關，該理由不成立
- desktop 設定頁加審查規則的編輯入口（使用者明示）
**Ruled out**: 存在 `rules.review`——會與自訂 schema 中 id 為 review 的 artifact 撞名、CLI 寫入動詞擋下、放不下 extend／replace 開關；`rules.review`＋保留字＋另開政策欄位——規則與開關分居兩處還要加保留字，比獨立區塊更散；開放換掉 Correctness 軸或流程規則——章的意義會因專案而變
**Open**: config 技能如何處理 review 區塊（使用者新提）；replace 模式下自訂規則的嚴重度是否分硬性／建議；replace 配空清單怎麼處理；寫入動詞的形狀；desktop 用詞；verify 是否比照

### Round 3 — interview (2026-10-14)

**Focus**: config 技能如何處理新的 review 區塊
**Position**: config 技能升級——extend／replace 開關列為第六問、每次帶現值問使用者；審查規則只收使用者口述的，技能不從 repo 推導（使用者未異議，直接轉問嚴重度寫法）。
- 開關是答案不是發現，比照 locale／tdd／audit 不推論（config.md Step 3），否則第二次執行的 diff 不收斂
- 既有審查規則照判準檢查：判準 1 在 extend 模式下擋重述 12 條 smell（已在 speclink-review 技能內），replace 模式不檢查此點；判準 3 不寫會過時內容；判準 4 引用必須存在
- 新規則：技能詢問要不要加或改，使用者口述的規則過判準後經 --dry-run 落地
- 寫入走新的 workflow-config 子指令，同意後才寫；remote 照舊做版本衝突檢查
- 規格：config-skill MODIFIED「技能規定固定輸入來源與四條內容判準」（判準 1 依模式），ADDED 審查規則第六問與規則接收
**Ruled out**: 技能從 CLAUDE.md／CONTRIBUTING／lint 設定推導審查規則——Standards 軸本來就自己讀這些檔，抄一份會成兩份正典而漂移；lint 已強制的規則審查本來就略過
**Open**: 規則嚴重度的寫法（使用者新提）；replace 配空清單；寫入動詞形狀；desktop 用詞；verify 是否比照

### Round 4 — interview (2026-10-14)

**Focus**: 自訂規則的嚴重度怎麼寫，以及 WARNING 是否已有擋章機制
**Position**: `review:` 區塊依嚴重度分 `critical`／`warning`／`suggestion` 三個清單，規則在哪一格就以該級入單；WARNING 擋章的機制引擎現在就有，要改的只有審查技能的文字。
- 結構：`review: { baseline: extend|replace, critical: [...], warning: [...], suggestion: [...] }`，各清單為純字串
- 分格用法：critical＝違反必錯無例外；warning＝通常要修、偶有正當例外（可用接受放行留紀錄）；suggestion＝偏好，不擋章
- 審查者照規則所在清單定級、不自行重判；工單記為 `- [CRITICAL] path — Standards: 違反專案審查規則「…」：…`
- 12 條 smell 照舊 SUGGESTION；要讓某條 smell 擋章就在 warning 寫對應規則
- config 技能依語氣建議分格（不得／一律→critical、要／應→warning、考慮／盡量→suggestion），使用者確認
- desktop 設定頁「審查規則」為三個清單編輯器，各附一行是否擋章的說明
- 查證 WARNING 擋章：review-station 規格「蓋章守門與蓋章效果」已規定 CRITICAL 與 WARNING 為必修、擋乾淨蓋章、--accept 只豁免必修；引擎 stamp_gate 以「非 SUGGESTION」計數（station.rs:513-520），審查與驗證兩站共用；CLI 測試 review_verbs.rs:245-258、verify_verbs.rs:329 已釘住；desktop 工單分頁已有 WARNING 琥珀色章與「CRITICAL、WARNING 皆 0 才可蓋章」提示（TicketView.tsx:14-19、:115）
- 要改的只有審查技能文字：Standards 軸目前寫明只用 CRITICAL／SUGGESTION 兩級（review.md:93），必修定義未涵蓋自訂規則的 warning（review.md:113）；review-skill 規格「審查結果的裁量分類」隨之 MODIFIED
**Ruled out**: 每行前綴 `[CRITICAL] …`——實測 YAML 解析失敗，手改會讓整份 config.yaml 讀不了；`CRITICAL: …`——實測被解析成物件；每條寫成 rule／severity 物件——每條多兩行，手改冗長
**Open**: verify 是否比照；replace 配空清單；寫入動詞形狀；desktop 用詞

### Round 5 — interview (2026-10-14)

**Focus**: 「接受」放行機制的現況，與驗證站是否比照開放
**Position**: 接受放行現在就有，但不分 CRITICAL 與 WARNING——兩級都能接受；第 4 輪分格說明中「warning 可接受放行」被誤寫成 warning 專屬，更正為兩級機制相同、差別只在用法指引；驗證站這次不開放自訂規則。
- 接受途徑一：修正迴圈中選擇不修的必修項，以 `(accepted)` 標記原文帶入下一輪，子代理收到不得重報清單（review.md 第 8、9 步；review-skill「已接受事項的續輪前饋」）
- 接受途徑二：`speclink review stamp --accept` 帶保留蓋章；引擎只在未帶 --accept 時以「非 SUGGESTION」計數擋章（station.rs:513-525）
- 帶保留蓋章與乾淨蓋章寫入同一組 reviewed_* 欄位，meta 不標示「帶保留」；工單於蓋章時刪除，接受項的文字在 fs 模式只留在 git 歷史，remote store 不保留（review-station「蓋章守門與蓋章效果」）——既有限制，非本案引入
- 驗證站維持現狀：它的規則就是變更自己的規格，要求規格多寫什麼已可用 rules.specs／rules.design
**Ruled out**: 驗證站本次比照開放自訂規則——使用者裁定先不用
**Open**: CRITICAL 是否改為不可接受（讓三級在機制上有差別）；replace 配空清單；寫入動詞形狀；desktop 用詞

### Round 6 — interview (2026-10-14)

**Focus**: CRITICAL 是否改為不可接受
**Position**: 維持現行機制——CRITICAL 與 WARNING 都擋章、都可經接受放行，三個清單的差別只在給寫規則者的指引。
- 理由：AI 審查必有誤判，不可接受的 CRITICAL 會讓誤判變成死路（只能照錯的判斷改碼或丟工單重審）
- 理由：蓋章檢查由兩站共用（station.rs:490 stamp_gate），改動會波及 Correctness 軸與驗證站的所有 CRITICAL，遠超自訂規則範圍
- 使用者邀請提出可優化之處，下一輪處理
**Ruled out**: CRITICAL 不可用 --accept 放行——誤判成死路，且波及兩站全部 CRITICAL
**Open**: 優化候選的取捨；replace 配空清單；寫入動詞形狀；desktop 用詞

### Round 7 — interview (2026-10-14)

**Focus**: 優化候選的取捨
**Position**: 納入三項優化（接受 CRITICAL 多一道確認、review 區塊嚴格解析、審查報告寫明套用規則），章標示「帶保留」另開變更，改規則不讓既有章失效。
- 接受 CRITICAL：使用者要接受的項目含 CRITICAL 時，技能逐條列出並要求明確確認一次；只接受 WARNING 時流程照舊；只改審查技能文字，引擎與工單格式不動
- 嚴格解析：review 區塊出現未知子鍵或 baseline 值不是 extend／replace 時，讀設定即報錯並點名該鍵，比照 locale 值檢查（config.rs:328-333）；代價為設定有錯時所有指令拒絕執行，與現行壞檔行為一致
- 報告寫明規則：報告開頭一行列出 baseline 模式與三個清單的條數；replace 且三清單皆空時寫明「Standards 軸只依 repo 文件」，不另設寫入檢查（解掉「replace 配空清單」待解項）
- 改規則不失效：章綁被審查的程式碼內容（reviewed_scope），規則只影響之後的審查；複驗階段改規則也不產生新發現，因為複驗只判前輪發現
**Ruled out**: replace 配空清單在寫入時擋下——「不要 smell 只看 repo 文件」是正當選擇，報告寫明即可；章綁規則指紋、改規則即失效——兩站失效判定都要改，且規則調整本意是影響之後的審查
**Open**: 第 3 項的「套用規則」desktop 是否也要顯示（使用者新提）；寫入動詞形狀；desktop 用詞

### Round 8 — interview (2026-10-14)

**Focus**: 套用規則是否要在 desktop 顯示，以及 desktop 的畫面怎麼呈現
**Position**: desktop 要顯示（使用者確認）；套用規則寫進工單首輪的選填 `**Rules**:` 行，desktop 在設定頁新增「審查規則」分類、在變更詳情頁審查分頁的第 1 輪顯示套用規則；畫面草稿在新畫布 https://claude.ai/artifact/RNqumZRVKjC7AWox64Z2pu ，待使用者確認。
- 寫進工單的理由：報告只在執行審查的對話裡，工單才是隊友、其他電腦與 session 都看得到的記錄；desktop 工單分頁只讀解析後的欄位（adapter.ts:163-169），解析器略過不認得的行（station.rs:725-770），所以引擎必須認得這一行
- 工單：首輪多一行選填 `**Rules**: extend; critical=2; warning=1; suggestion=1`，只記模式與條數；複驗不重新套用規則所以不寫；解析器比照 `**Phase**:` 檢查格式；`review show --json` 每輪新增 `rules` 欄位，舊工單為 null；驗證站共用解析器但不寫此行
- 設定頁：「openspec/config.yaml」分組在產出規則與產出流程之間新增「審查規則」；唯讀狀態顯示預設清單模式（可展開 12 條 smell 名稱）與三個嚴重度清單；編輯狀態為預設清單的兩個單選（沿用並追加／整份換掉）＋三個清單文字框（即時條數）；整份換掉且三清單皆空時頁內提示「只依 repo 文件審查」
- 詳情頁：審查分頁第 1 輪展開後、範圍檔案之前多一列「套用規則」——預設 12 條＋專案規則與三級條數籤；整份換掉時寫明沒有套用預設 12 條；整份換掉且無規則時以琥珀底提示；舊工單沒有記錄時不顯示此列；列尾註明記錄的是首輪當時的設定
**Ruled out**: 只在審查報告寫套用規則——報告只存在於執行審查的對話，隊友與其他 session 看不到；工單只多寫一行文字、引擎不解析——desktop 只讀解析後的欄位，看不到
**Open**: 畫面草稿是否需要調整；是否分兩刀（設定＋技能＋設定頁／工單記錄＋詳情頁顯示）；寫入動詞形狀

### Round 9 — interview (2026-10-14)

**Focus**: 審查規則的編輯方式，以及套用規則展開時 12 條與專案規則的來源
**Position**: 編輯沿用產出規則的同一套編輯器與寫入路徑；套用規則可展開，預設 12 條從引擎內建清單引用、不寫進工單，專案規則全文寫進工單首輪——修改第 8 輪「工單只記模式與條數」。
- 編輯器：與產出規則頁同一個 CodeEditor（CodeMirror 6 純文字模式、行號、自動換行、⌘S 儲存、底部狀態列），同一組編輯／取消／儲存控制；每個嚴重度一張小卡（嚴重度籤＋即時條數），取代原草稿的普通文字框（apps/desktop/src/views/project-settings/RulesPage.tsx、apps/desktop/src/components/CodeEditor.tsx）
- 寫入路徑：沿用 config.yaml 的文字手術寫入（只改目標區塊、其他內容與註解逐位元保留）、寫入前後解析驗證、壞檔停用表單、remote 版本衝突檢查（config.rs:536-548）；預設清單模式仍為單選
- 12 條引用引擎：清單寫死在引擎內建的審查技能範本，只隨引擎版本改變；desktop-core 直接相依 speclink-core（apps/desktop/core/Cargo.toml），本機與 remote 模式都拿得到；工單只要記 extend／replace 就知道有沒有套用
- 專案規則寫進工單：config 之後可能被改，展開時若讀目前設定，會把新規則誤標成「首輪當時的設定」；規則清單通常只有幾行，工單在蓋章時刪除，膨脹只是暫時的
- 工單格式改為 `**Rules**: extend` 一行＋每條規則一行 `**Rule**: CRITICAL <規則原文>`；每行都以 `**Rule**:` 開頭，不會撞到工單禁止的 `## ` 輪次分隔；`review show --json` 每輪 `rules` 為 {baseline, critical[], warning[], suggestion[]} 或 null，條數由清單推得
- 詳情頁：套用規則列加「展開／收合」；展開後先列預設 12 條名稱（extend 時）或「沒有套用預設 12 條」（replace 時），再依嚴重度列專案規則全文，末尾註明之後改規則這裡不會跟著變
**Ruled out**: 工單只記條數、展開時讀目前 config——規則改過後會把新規則誤標成首輪當時的設定；12 條也寫進工單——引擎內建且只隨版本改變，寫進去只是重複；記規則指紋、比對不符才警告——比直接存全文複雜，且仍看不到當時的規則
**Open**: 畫面草稿是否還要調整；是否分兩刀；寫入動詞形狀

### Round 10 — interview (2026-10-14)

**Focus**: 畫面草稿確認與分刀
**Position**: 使用者確認畫面草稿與分兩刀——第 1 刀做設定、兩個技能與設定頁，第 2 刀做工單記錄套用規則與詳情頁顯示，第 2 刀依賴第 1 刀。
- 12 條 smell 名稱由引擎提供給 desktop 這件事落在第 1 刀，因為設定頁的「查看 12 條」已經要用；第 2 刀的詳情頁沿用
- 寫入動詞的確切形狀交給第 1 刀的 design 決定
**Open**: 無——收斂到結論

## Conclusion

**Decision**: 審查關卡開放專案自訂規則——config.yaml 新增頂層 `review:` 區塊，Standards 軸的預設 12 條 code smell 可沿用並追加或整份換掉，專案規則依 critical／warning／suggestion 三個清單決定嚴重度；desktop 設定頁可編輯、審查分頁顯示首輪套用的規則；分兩刀交付。
- 設定形狀：`review: { baseline: extend|replace, critical: [...], warning: [...], suggestion: [...] }`，各清單為純字串；`baseline` 省略＝extend
- 例：`review: { critical: ["所有 SQL 一律用參數化查詢，不得用字串拼接", "不得把金鑰或密碼寫死在程式碼裡"], warning: ["新增的公開 API 要有對應的測試"], suggestion: ["函式超過 50 行時，考慮拆分"] }`
- 可自訂範圍：只有 Standards 軸——自訂規則與 repo 文件標準同等；extend＝12 條照舊（只記 SUGGESTION）再加專案規則，replace＝不套用 12 條；Correctness 軸（抓 bug）與流程規則（首輪／複驗、三級嚴重度、必修／可裁界線、工單格式）不開放
- 嚴重度：規則所在清單即違反時的入單級別，審查者不自行重判；critical 與 warning 違反皆為必修、擋蓋章，suggestion 不擋；工單記為 `- [CRITICAL] path — Standards: 違反專案審查規則「…」：…`；想讓某條 smell 擋章就在 warning 寫對應規則
- 擋章與接受機制維持現狀：引擎蓋章檢查以「非 SUGGESTION」計數（station.rs stamp_gate，兩站共用），CRITICAL 與 WARNING 都可經 `--accept` 放行；三個清單的差別在給寫規則者的指引（critical＝違反就該修；warning＝通常要修、偶有正當例外；suggestion＝偏好）
- 嚴格解析：review 區塊出現未知子鍵或 baseline 值不是 extend／replace 時，讀設定即報錯並點名該鍵（比照 locale 值檢查）；舊版 CLI 遇到此區塊只會略過
- 改規則不讓既有章失效：章綁被審查的程式碼內容；規則只影響之後的審查，複驗階段改規則也不產生新發現
- 接縫：唯一實作落點在 speclink-core（config 解析與文字手術寫入、工單解析）；CLI、server、desktop 只經既有 workflow-config 寫入接縫與 `review show --json` 共用，不平行實作；審查技能以 `speclink workflow-config show --json` 讀規則，不直接讀檔
- 驗證站不比照開放
- **cut 1 `review-rules-config`**: 設定區塊、審查技能、config 技能與 desktop 設定頁
  - 引擎：WorkflowConfig 解析 review 區塊（嚴格）、`workflow-config show --json` 帶出 review、新增寫入子指令（--stdin、--dry-run、remote 版本衝突檢查）；12 條 smell 名稱由引擎提供給 desktop
  - 審查技能：Standards 軸 brief 帶入專案規則與 baseline 模式；review.md 原「Standards 軸只用 CRITICAL／SUGGESTION」與必修定義改為涵蓋 warning 清單；接受項含 CRITICAL 時逐條列出再確認一次（只接受 WARNING 照舊）；報告開頭一行寫明 baseline 模式與三清單條數，replace 且三清單皆空時寫明「Standards 軸只依 repo 文件」
  - config 技能：baseline 列為第六問、每次帶現值問、不推論；既有審查規則照判準 3、4 檢查，判準 1 在 extend 時擋重述 12 條、replace 時不檢查此點；技能不從 repo 推導審查規則，只接收使用者口述的規則並依語氣建議分格（不得／一律→critical、要／應→warning、考慮／盡量→suggestion），經 --dry-run 落地
  - desktop 設定頁：openspec/config.yaml 分組在產出規則與產出流程之間新增「審查規則」；唯讀顯示預設清單模式（可展開 12 條名稱）與三個清單；編輯沿用產出規則的 CodeEditor 與同一組編輯／取消／儲存控制與寫入路徑，預設清單為兩個單選（沿用並追加／整份換掉），整份換掉且三清單皆空時頁內提示
  - LANGUAGE.md 新增「審查規則」詞條
  - 規格：workflow-config（review 區塊與寫入）、review-skill（smell baseline 改為條件套用、審查結果的裁量分類、專案規則套用）、config-skill（四條判準、第六問與規則接收）、desktop-config（設定頁編輯審查規則）
- **cut 2 `review-ticket-applied-rules`**: 工單記錄首輪套用的規則，desktop 審查分頁可展開顯示；依賴 cut 1
  - 工單：首輪寫 `**Rules**: extend|replace` 一行＋每條專案規則一行 `**Rule**: <SEVERITY> <規則原文>`；複驗不寫；解析器比照 `**Phase**:` 檢查格式；驗證站共用解析器但不寫這些行
  - 輸出：`review show --json` 每輪新增 `rules`＝{baseline, critical[], warning[], suggestion[]}，舊工單為 null；remote 協定同步
  - 審查技能：首輪記錄時寫入上述行
  - desktop 審查分頁：第 1 輪展開後、範圍檔案之前一列「套用規則」（模式＋三級條數籤），可展開——12 條名稱取自引擎（extend 時）或寫明沒有套用（replace 時），專案規則全文取自工單；replace 且無規則時以琥珀底提示；舊工單沒有記錄時不顯示；末尾註明之後改規則這裡不會跟著變
  - 規格：review-station（工單的建立與追加、讀取）、review-skill（首輪記錄套用規則）、desktop-app（詳情頁的工單分頁）
- 畫面草稿：https://claude.ai/artifact/RNqumZRVKjC7AWox64Z2pu （設定頁唯讀、編輯中、變更詳情頁審查分頁三張）
**Rationale**: 規則要能被團隊共享、被 remote 存取、也要能放下「沿用或換掉」的開關，所以落在 config.yaml 的獨立區塊，而不是塞進以 schema artifact 為鍵的 rules 或依賴子代理自己翻 CLAUDE.md；嚴重度以「放在哪一格」表達，因為手改 YAML 時行首標記會讓整份設定讀不了；擋章機制不動，因為 AI 審查必有誤判，CRITICAL 若不可接受會變成死路，且蓋章檢查由兩站共用。
**Rejected alternatives**:
- 只能加規則、不能整份換掉 — 使用者要求兩者都要
- 存在 `rules.review` — 會與自訂 schema 中 id 為 review 的 artifact 撞名、CLI 寫入動詞擋下、放不下 extend／replace 開關
- `rules.review`＋保留字＋另開政策欄位 — 規則與開關分居兩處還要加保留字
- 開放換掉 Correctness 軸或流程規則 — 章的意義會因專案而變
- config 技能從 CLAUDE.md／CONTRIBUTING／lint 設定推導審查規則 — Standards 軸本來就會讀這些檔，抄一份會成兩份正典而漂移
- 每行前綴 `[CRITICAL] …` — 實測 YAML 解析失敗，手改會讓整份 config.yaml 讀不了
- 每行寫 `CRITICAL: …` — 實測被解析成物件
- 每條寫成 rule／severity 物件 — 每條多兩行，手改冗長
- CRITICAL 不可用 --accept 放行 — 誤判成死路，且波及兩站全部 CRITICAL
- replace 配空清單在寫入時擋下 — 「不要 smell 只看 repo 文件」是正當選擇，報告與設定頁寫明即可
- 章綁規則指紋、改規則即失效 — 兩站失效判定都要改，且規則調整本意是影響之後的審查
- 只在審查報告寫套用規則 — 報告只存在於執行審查的對話，隊友與其他 session 看不到
- 工單多寫一行但引擎不解析 — desktop 只讀解析後的欄位
- 工單只記條數、展開時讀目前 config — 規則改過後會把新規則誤標成首輪當時的設定
- 12 條也寫進工單 — 引擎內建且只隨版本改變，寫進去只是重複
- 記規則指紋、比對不符才警告 — 比直接存全文複雜，且仍看不到當時的規則
**Deferred**:
- 章上標示「帶保留」 — 帶保留與乾淨蓋章目前寫入同一組欄位、工單蓋章即刪；要改章欄位、desktop、remote 協定與兩站，與自訂規則無關，另開變更
- 驗證站自訂規則 — 使用者裁定先不用；驗證站的規則就是變更自己的規格，已可用 rules.specs／rules.design 要求規格多寫什麼
- 寫入子指令的確切形狀 — 交給 cut 1 的 design；限制為沿用 workflow-config 動詞族、--stdin 與 --dry-run、remote 版本衝突檢查，不走 set 的四層解析（無環境變數覆寫）
**Capture to**: proposal | design | spec | tasks | LANGUAGE.md
**Next**: /speclink-propose --from-discussion custom-review-rules
