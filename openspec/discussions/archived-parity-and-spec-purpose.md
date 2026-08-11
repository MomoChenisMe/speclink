---
topic: 桌面已封存頁與看板的呈現對齊（tray 落頁、卡片雙行、抽屜標頭）＋正典規格 Purpose 全為 TBD 的根因
slug: archived-parity-and-spec-purpose
status: promoted
promoted_to: desktop-archived-parity, spec-purpose-gates, spec-purpose-backfill
created: 2026-08-10
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 桌面已封存頁與看板的呈現對齊（tray 落頁、卡片雙行、抽屜標頭）＋正典規格 Purpose 全為 TBD 的根因

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：使用者以四張截圖提出 desktop 的四個調整點——(1) 自系統匣開啟變更／討論時，底層頁面停在原頁（截圖為已封存頁），未回到看板；(2) 已封存的變更卡與討論卡呈現不對稱（變更只有英文名、討論只有中文 topic），希望兩者同步為「標題＋描述」；(3) 已封存抽屜的標頭缺複製鈕與固定資訊，對照進行中變更抽屜；(4) 規格頁上每一份規格都標「Purpose 待補」。

模式：assumptions。掃到的相關源碼遠超三檔（apps/desktop/src/store.ts、tray.ts、packages/ui/src/components/{ArchivedList,ArchivedDrawer,RichDetailDrawer,DiscussionColumn,DiscussionDrawer}.tsx、apps/desktop/core/src/query.rs、crates/speclink-core/src/archive.rs），足以直接列假設讓使用者修正。

相關規格：client-protocol（清單 payload 欄位）、archive-merge（封存合併與新 capability 的 Purpose 帶入）、desktop 系列規格。相關變更：無進行中的相關 change（目前唯一在途為 manual-task-marker-gates，主題無關）。相關既有裁定：LANGUAGE.md 的明文例外「討論識別錨點以 slug 直出、topic 降為描述」（desktop-card-identity 起，數次擴充範圍）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-10)

**Focus**: 四個議題各自的根因與落點——先把「是什麼壞了」釘死，才談怎麼修。

**Position**: 四題都定位到單一落點，且議題 2／3 本質是「已封存頁沒跟上看板既有設計」，不是新設計；使用者確認五條假設全數成立。
- 議題 1（tray 落頁）：`store.ts:1040` 的 `openDetail` 與 `1057` 的 `openDiscussion` 只設抽屜狀態、不碰 `boardView`，於是抽屜浮在任何頁之上。修在這兩個函式補 `boardView: "board"` 即涵蓋全部四個呼叫端（KanbanBoard 點卡、同源變更互跳、討論抽屜跳衍生變更、封存前「去蓋章」）。
- 目的地無分支：`tray.ts:371-376` 的快照只餵 `state.changes` 與 `discussions.active`，系統匣上出現的一定是活躍項，不存在「該落在已封存頁」的情境。
- 議題 2（卡片雙行）：看板側早已是雙行——變更卡 `ChangeCard.tsx:155` 標題＋`whyExcerpt` 描述、討論卡 `DiscussionColumn.tsx:104-132` slug 標題＋topic 描述；已封存頁停在單行（`ArchivedList.tsx:62` 只有 name、`:171` 只有 topic）。要做的是把既有 anatomy 補到封存側。
- 議題 2 的討論側純前端可完成（DiscussionItem 已同時帶 slug 與 topic），且與 LANGUAGE.md 的受控例外一致；變更側則缺資料——`ArchivedItem`（adapter.ts:79-98）無 `whyExcerpt`，需照 `query.rs:207` 的 `purposeExcerpt` 先例在封存清單 payload 疊加欄位。
- 議題 3（抽屜標頭）：封存抽屜只有純文字標題（`ArchivedDrawer.tsx:152`），對照活躍變更抽屜的四層標頭（`RichDetailDrawer.tsx:355-470`）。該補標題複製鈕與出身列（建立者／建立日期／封存日期）；不補進度條與動作列——封存是唯讀定格。出身列的建立／開工日期需新增封存側 meta 查詢（`query.rs` 目前僅有 `archived_document_at`／`archived_capabilities_at`）。
- 議題 4（Purpose 待補）：非顯示問題。實測 `openspec/specs/*/spec.md` 67 份中 66 份的 Purpose 首行即 archive 佔位文字（`archive.rs:858-864`），UI 只是忠實標示（`query.rs:207-218`）。

**Ruled out**:
- 只在 `tray.ts:551` 的 dispatch 修落頁——涵蓋面不足，其他跨頁入口仍留下同樣的錯頁狀態。
- 已封存兩種卡都改用中文 topic 當標題——要反推翻 LANGUAGE.md 受控例外與看板側既有設計，範圍遠大於本次訴求。
- 把「Purpose 待補」的橘字標示拿掉當解——治標，且等於承認正典規格不需要「這個能力是幹嘛的」這一層。

**Open**: 議題 4 的 Purpose 到底被誰消費、為何全數落在 TBD；存量 66 份補寫的做法與時機；四題是收成一個 change 還是拆開。

### Round 2 — assumptions (2026-08-10)

**Focus**: Purpose 的實際用途是什麼、以及為何 67 份正典規格中 66 份都停在 TBD 佔位。

**Position**: Purpose 唯一的機器消費者是 propose 的 capability 歸屬判斷；全數 TBD 的成因是「預設路徑從創始日就是寫 TBD、帶入機制上線 7 天且只被用過一次、技能層從未要求寫、寫進正典後無任何守門複查」四件事疊加。
- 消費面一（唯一機器消費）：`.claude/skills/speclink-propose/SKILL.md:103` 要求 propose 對候選 capability 跑 `speclink show <spec-id>` 讀頂端 Purpose，用以決定新需求歸入哪個既有 capability 或另開新的。67 個 capability 全為 TBD 時，這步只能靠 capability 名稱與 requirement 內文猜——capability 邊界漂移的直接來源。
- 消費面二、三（人眼）：規格卡描述列（`query.rs:207` 的 `purposeExcerpt`）與 `speclink show` 輸出頂端，實測 `speclink show verb-contract` 首段即該句 TBD。
- 成因一（預設路徑）：`archive.rs:858-864`——建立新 capability 時 delta 未帶 `## Purpose` 即寫入佔位句。`git log -S` 顯示該佔位文字源自創始 commit `5d7fa5c`，即專案第一天起的預設行為。
- 成因二（機制新且未被沿用）：讓 delta 自帶 Purpose 的 `delta_purpose`（`archive.rs:790`）由 `e902eaa`（2026-08-03，change archive-fail-changed-merge／archive-fail-closed-merge）引入，距今 7 天。283 份封存 delta 中僅 1 份寫了 `## Purpose`，正是該機制自身的 change，產出唯一一份有真 Purpose 的規格 `archive-merge`——上線後只被 dogfood 過一次。
- 成因三（技能層未接）：`speclink-propose` 的步驟從未要求「新 capability 的 delta 需帶 `## Purpose`」；僅 `speclink-archive/SKILL.md:82-83` 以說明語氣提及此機制存在。agent 讀不到要求就不會寫。
- 成因四（無守門複查）：analyzer 的模糊詞檢查（`analyzer.rs:208-235`，含 TBD／TODO）作用於 change 的 delta，正典規格寫入後不再受任何檢查。
- 判讀：「Purpose 待補」不是 UI 出錯，而是誠實標示——66 個能力從未有人寫過邊界說明。

**Ruled out**:
- 「UI 誤判／解析壞掉」的可能——`speclink show` 直出同一句 TBD，資料端即如此。
- 「delta 帶 Purpose 的機制沒做」的可能——機制已存在且有測試釘住（`archive.rs:1945-1996`），缺的是技能層的要求與守門。

**Open**: 存量 66 份怎麼補（一次全補／用到才補／只擋新的）；要不要讓 archive 對新 capability 缺 Purpose 直接拒絕放行；四題收成一個 change 還是拆開。

### Round 3 — assumptions (2026-08-10)

**Focus**: 使用者裁定走 A（硬擋）；上游 OpenSpec（Fission-AI/OpenSpec）對同一問題的做法是什麼、我們該抄哪些。

**Position**: 上游用「三道防線」而非單一硬擋——正典指引寫死規則、validate 把 Purpose 列為必填並檢查長度、帶入時做品質檢查；其中「validate 檢查正典 spec 的 Purpose」是我們完全沒有的一層，且它正是存量 66 份的追蹤機制，應與使用者裁定的 archive 硬擋一起做。
- 上游第一道（給 agent 的規則）：`schemas/spec-driven/schema.yaml:86-95` 明文要求「新 capability 的 delta 以 `## Purpose` 開頭，一兩句、50+ 字元，否則 `openspec validate --strict` 報 too brief」，並反向規定「既有 capability 的 delta 不要加 `## Purpose`（會被忽略）；要改既有 Purpose（含殘留 TBD 佔位）直接編輯正典檔」，附完整範例。我們的對應位置是 propose／archive 技能文字，目前只有 archive 技能以說明語氣提過機制存在。
- 上游第二道（validate 守門）：`src/core/schemas/spec.schema.ts:7` 把 spec 的 overview（＝Purpose 內容）以 zod `.min(1)` 設為必填非空，訊息 `SPEC_PURPOSE_EMPTY`；`validation/constants.ts:7,40` 定 `MIN_PURPOSE_LENGTH = 50` 與警告 `PURPOSE_TOO_BRIEF`；另有缺 `## Purpose`／`## Requirements` 區段的引導訊息。這一層作用於正典 spec 本身，我們的 analyzer 模糊詞檢查只跑 change delta，正典寫入後零複查。
- 上游第三道（帶入時的品質檢查）：`src/core/specs-apply.ts:302-352`——delta Purpose 只餵尚未存在的 spec；目標已有不同 Purpose 時發警告說明「delta Purpose 被忽略」；帶入後若使 spec 不可讀則退回佔位並警告；帶入的 Purpose 短於 50 字元亦警告（註解明說「佔位句總是清過這個門檻」）。我們的 `delta_purpose`（archive.rs:790）只有「有就用、沒有就佔位」，無警告、無長度概念。
- 上游沒有做的：archive 不因缺 Purpose 拒絕放行，仍寫佔位。使用者裁定的 A（硬擋）比上游嚴，但與我們 archive 既有的 fail-closed 守門風格一致（archive-fail-closed-merge）。
- 效果對照（實證）：上游 36 個 capability 的正典 spec 抽查 5 份（artifact-graph／cli-validate／opsx-archive-skill／telemetry／schema-resolution），Purpose 全為真內容、一至三句、講清楚治理邊界；`gh` 全庫搜尋顯示佔位句「Update Purpose after archive」不出現在其 `openspec/specs/` 任一檔。我們是 66/67 為佔位。
- 落地推論：A 只能擋新的、對存量 66 份零作用；上游第二道正好是存量的追蹤清單，補上後存量可走「用到才補」而不失控。
- 守門嚴格度推論：若守門只檢查「有沒有 `## Purpose` 段」，代理寫兩個字即可繞過；應同時要求非空且達最低長度，長度值直接對齊上游的 50 字元。

**Ruled out**:
- 照抄上游「不硬擋、只靠 validate 事後抓」——使用者已裁定 A；且我們的 archive 本就是 fail-closed 風格，靠事後抓正是過去 7 天失效的路徑。
- 只做 archive 硬擋而不做 validate 檢查——存量 66 份將永遠沒有追蹤面，UI 橘字是唯一提示。

**Open**: 守門的最低長度是否採 50 字元；存量 66 份的補寫策略（一次全補／用到才補／不補只標示）；四題收成一個 change 還是拆開。

### Round 4 — assumptions (2026-08-10)

**Focus**: 使用者裁定「門檻與上游一致」「存量一次全補」後，Purpose 這條線的落點與工單切法。

**Position**: validate 的 `--specs` 旗標已存在但完全空轉，正典規格驗證的接線位置與 UI 把手都是現成的；三件事（守門／驗證／存量）依性質切成兩個 change 較乾淨。
- 實證（旗標空轉）：`crates/speclink-cli/src/verbs/checks.rs:26-28` 宣告 `--specs`，但 `cmd_validate`（同檔 43-59）建 `Command::Validate` 時只傳 `item／all／changes／strict`，`a.specs` 從未被讀取。實跑 `speclink validate --specs` 與 `--specs --strict`，輸出的仍是 change 驗證結果（`✓ manual-task-marker-gates — valid`），67 份正典規格完全沒被碰。
- 實證（核心無 spec 驗證器）：`speclink-core` 只有 `validate.rs:17` 的 `validate_change`，不存在任何正典 spec 驗證入口——空轉旗標的成因。
- 因此第二道防線不是新開動詞，而是把既有的 `--specs` 接上真正的正典規格驗證；與上游 `openspec validate` 同時驗 changes 與 specs 的形狀一致。
- 門檻解讀（與上游一致）：`MIN_PURPOSE_LENGTH = 50` 同值採用；validate 分級照上游——缺 `## Purpose` 區段或內容為空＝error，內容不足 50 字元＝warning（strict 下報）；額外加一條「內容仍為 archive 佔位句」＝warning，因佔位句長度恆超過 50、上游那道長度門檻抓不到它（`specs-apply.ts:348` 註解自承）。
- archive 守門（使用者裁定的 A，比上游嚴）：新建 capability 的 delta 缺 `## Purpose`／內容為空／不足 50 字元三者皆拒絕放行。不擋「不足 50」等於留後門——代理寫兩個字即可繞過硬擋。
- 存量補寫的落地：正典規格只有 archive 會寫入（`archive.rs:638` 是 `write_canonical_spec` 的唯一非測試呼叫端），但檔案就在 repo 內、無任何機制阻止直接編輯，與上游「要改既有 Purpose 直接編輯 `openspec/specs/<capability>/spec.md`」的指引一致；66 份補寫是純文件工作，不產生任何 delta。
- 工單切法：Purpose 線的程式碼（守門＋validate 接線＋技能文字）與 66 份文件補寫性質不同——後者無程式碼、且可用前者產出的 `validate --specs` 當驗收（補完後全綠），分成兩個 change 可避免品質站的審查面被 66 份文件稀釋。

**Ruled out**:
- 為正典規格驗證新開一支 CLI 動詞——既有 `--specs` 旗標與 `--strict` 已在位，新開等於製造第二個把手。
- 存量補寫與守門合併為一個 change——66 份文件改動會把 review／verify 的審查面撐爆，且驗收工具正是同 change 內才剛寫好的東西。

**Open**: 四個議題的 change 切法（desktop 三題是否收成一批）；一次全補的品質控管方式（逐份人審／抽審）。

### Round 5 — assumptions (2026-08-10)

**Focus**: change 2 改完之後，propose 出來的 delta 就會帶 Purpose 嗎——「產生」的來源到底在哪個檔案。

**Position**: 引擎不生成 Purpose 文字，產生的唯一來源是代理寫 delta 時讀到的指引檔；我們的指引檔正是上游 schema.yaml 的對應物，且獨獨漏掉了 Purpose 那一段——283 比 1 的直接成因就在這裡。
- 落點確認：`crates/speclink-core/assets/schema/spec-driven/specs.instruction.md`（97 行，經 `schema.rs:60` 的 `include_str!` 編進引擎，由 `instructions` 動詞注入給代理）與上游 `schemas/spec-driven/schema.yaml` 逐段對應——「Create one spec file per capability」「Delta operations」「Format requirements」「CRITICAL: Scenarios MUST use exactly 4 hashtags」「MODIFIED requirements workflow」四段幾乎逐句相同。
- 唯一的結構差異：上游在 Format requirements 與 MODIFIED workflow 之間有整段 Purpose 規則（新 capability 才寫、50+ 字元、既有 capability 不要寫、要改既有 Purpose 直接編輯正典檔），我們這份**完全沒有這一段**，全檔 97 行零次出現 Purpose。代理每次寫 delta 讀的就是這份，讀不到就不會寫。
- 因此三個機制的作用時點各不相同：指引檔補上規則＝propose 當下就寫出來（唯一的產生來源）；archive 守門＝忘了寫時封存被擋（兜底）；`validate --specs`＝事後抓正典殘留。缺了第一項，另外兩項只會讓工單卡在最後一哩。
- 適用範圍限定：只有 change 新開 capability 時才寫 Purpose；既有 capability 的 delta 不寫（寫了引擎也會忽略，`archive.rs` 測試 `delta_purpose_never_rewrites_an_existing_canonical_purpose` 釘住此行為）。「propose 後就有 Purpose」僅在新開能力的 change 上成立。
- 更早的回饋點（建議納入 change 2）：`validate.rs:17` 的 `validate_change` 可加一條「delta 對應的 capability 在正典中不存在（＝新開）且未帶 `## Purpose`」的檢查，使 apply 階段即報，不必等到 archive 被拒。
- 實作波及面提醒：`specs.instruction.md` 是 `include_str!` 的 asset，改內文須連動 MARKER_VERSION／golden 檔／assets.lock（既有正典紀律），propose 時需把這三處寫進任務。

**Ruled out**:
- 「引擎自動生成 Purpose 文字」的期待——引擎只搬運 delta 既有的 `## Purpose` 內容（`archive.rs:790` 的 `delta_purpose`），不產生語句；能力邊界說明只能由寫 delta 的人／代理寫。
- 只改技能 SKILL.md 而不改 specs.instruction.md——後者才是代理寫 delta 時實際讀到的正典指引，且是上游放規則的同一位置。

**Open**: 一次全補 66 份的品質控管方式（逐份人審／抽審）。

### Round 6 — assumptions (2026-08-10)

**Focus**: Purpose 規則要不要也寫進 propose 技能；上游 OpenSpec 的 propose 技能實際怎麼處理。

**Position**: 上游技能刻意零次提 Purpose——規則單一正典放在 schema、技能只負責去拿；我們技能同構但多一步收尾 validate，那才是比文字提醒更硬的著力點，技能層只該放檢查點、不複製規則內文。
- 上游實證：`skills/openspec-propose/SKILL.md` 全檔 148 行，`grep -i purpose` **零命中**。其步驟 5 對每個 artifact 跑 `openspec instructions <artifact-id> --change "<name>" --json`，取回的 JSON 含 `instruction`（schema-specific guidance）、`template`、`rules`、`context`，技能明寫「create the artifact file using template as the structure」「Apply context and rules as constraints」。內容規則的唯一正典是 schema，技能不重複任何一條。
- 我方同構：`.claude/skills/speclink-propose/SKILL.md`（419 行）同樣走 `speclink instructions <artifact-id>`（132-135、263-267 行），第 401 行明寫「Follow the `instruction` field from `speclink instructions` for each artifact type」。因此補在 `specs.instruction.md` 的規則會自動流進 propose 的寫作流程，機制上已接通。
- 我方獨有的著力點：我們的 propose 技能第 381 行收尾跑 `speclink validate "<name>"`、失敗即修再驗；上游 propose 技能全篇不跑 validate（僅在 store 說明中列名）。因此 change 二 的 `validate_change` 早期檢查一旦加上，propose 收尾當場就會擋下「新開 capability 缺 Purpose」，代理立刻補——不必等到 archive。此檢查的地位由「建議納入」升級為主力機制。
- 技能層的形狀（回應使用者訴求）：在 specs artifact 那一步加一句硬性檢查點——「本 change 若新開 capability，delta 必須以 `## Purpose` 開頭；規則內文見 instruction 欄位」——只標示條件與指路，不複製 50 字元、既有 capability 不寫等規則內文。
- 反對複製內文的依據：同一條規則若同時存在於 asset 與技能，改規則需同步兩處，本專案已有「三處技能同步」漂移的前例；單一正典＋指路可同時滿足「明確提醒」與「不漂移」。

**Ruled out**:
- 照上游完全不在技能提 Purpose——使用者明確要求技能要有明確提醒，且我們的技能本就比上游厚（419 vs 148 行），放檢查點與既有風格一致。
- 在技能複製完整 Purpose 規則內文——兩份同源文字必然漂移，且 asset 才是代理寫 delta 時實際讀到的正典。

**Open**: 一次全補 66 份的品質控管方式（逐份人審／抽審）。

### Round 7 — assumptions (2026-08-10)

**Focus**: 技能檢查點還要不要——既然上游技能零提 Purpose，是否單靠 validate 早期檢查就夠。

**Position**: 拿掉技能檢查點，改抄上游的「錯誤訊息自帶修復指引」——validate 早期檢查＋會教人怎麼修的錯誤訊息，已使迴路自癒，技能層第三次提及的邊際價值趨近於零。
- 寫作當下的提醒已存在：propose 技能命令代理逐 artifact 跑 `speclink instructions` 並遵循其 `instruction` 欄位（SKILL.md:401），規則補進 `specs.instruction.md` 後，代理寫 delta 時本來就會讀到——技能檢查點是同一時點的第二次提醒，不是唯一提醒。
- 失敗成本實際很小：propose 收尾的 validate 失敗迴圈已內建（SKILL.md:381「fix errors and re-validate」）；漏寫 Purpose 的修復動作只是在既有 delta 頂端補一段一兩句話，不是重寫。
- 上游的關鍵細節（本輪新抄）：`validation/constants.ts:45-53` 的 GUIDE_* 修復指引直接附在錯誤訊息上——`GUIDE_MISSING_SPEC_SECTIONS` 連完整範例骨架都印出來。錯誤訊息即教材，代理看到失敗當場知道怎麼補，這才是取代技能提醒的實體。我們的早期檢查錯誤訊息應照此模式：說明「新開 capability 的 delta 須以 `## Purpose` 開頭（一兩句、50+ 字元）」並附範例。
- 維護面：技能檢查點雖只一行，仍是第三處提及 Purpose 的位置，且技能檔案跨 `.claude/skills`／`.agents/skills` 多份同步；兩處（asset＋錯誤訊息）已閉環，第三處是純負債。
- 修正第六輪的判斷：當時保留檢查點的理由是「使用者要求技能明確提醒」，使用者本輪主動重議；以機制取代文字提醒後，validate-only 與上游「技能零提規則」的形狀完全一致。

**Ruled out**:
- 保留技能一行檢查點——寫作時點已有 instruction 承載提醒、失敗時點由帶指引的錯誤訊息承載修復，第三處提及無新增價值、多一處同步負擔。
- 錯誤訊息只報「缺 Purpose」不附修法——上游經驗是指引直接附在訊息上才自癒；乾錯誤訊息會把代理推回翻文件，重製摩擦。

**Open**: 一次全補 66 份的品質控管方式（逐份人審／抽審）。

## Conclusion

**Decision**: 四個議題切成三個 change 落地。

一、desktop 已封存頁與看板對齊（議題 1／2／3 合一批，同一批檔案）：
- `store.ts` 的 `openDetail`／`openDiscussion` 補 `boardView: "board"`，抽屜開啟時底層頁面回到看板（系統匣只列活躍項，目的地無分支）。
- 已封存變更卡與討論卡改為「標題＋描述」雙行，與看板既有 anatomy 一致：討論卡以 slug 為標題、topic 為描述（純前端，且符合 LANGUAGE.md 受控例外）；變更卡以 name 為標題、新增描述列，資料源需在 Rust 端封存清單 payload 疊加「封存 proposal.md 的 Why 首句」欄位（比照 `query.rs:207` 的 `purposeExcerpt` 先例）。
- 封存抽屜標頭補標題複製鈕與出身列（建立者／建立日期／封存日期）；不補進度條與動作列——封存是唯讀定格。出身列的建立／開工日期需新增封存側 meta 查詢。

二、Purpose 守門（引擎＋指引），依作用時點由早到晚：
1. `specs.instruction.md` 補上上游對應的 Purpose 規則段（新 capability 才寫、50+ 字元、既有 capability 不要寫、要改既有 Purpose 直接編輯正典檔）——這是 Purpose 得以產生的唯一來源，也是規則的單一正典；技能層不再另設檢查點（validate-only，與上游「技能零提規則」同形）。改 asset 須連動 MARKER_VERSION／golden／assets.lock。
2. `validate_change` 加早期檢查（主力機制）：delta 對應的 capability 不存在於正典（＝新開）且未帶合格 Purpose 即報 error，且錯誤訊息照上游 GUIDE_* 模式自帶修復指引與範例骨架（validation/constants.ts:45-53 的作法）——propose 技能收尾本就跑 `speclink validate` 並帶「fix errors and re-validate」迴圈（SKILL.md:381），錯誤訊息即教材，迴路自癒。
3. archive 對新建 capability 硬擋：delta 缺 `## Purpose`／內容為空／不足 50 字元三者皆拒絕放行（比上游嚴，與 archive 既有 fail-closed 風格一致），作為最後一道網。
4. 接上空轉的 `validate --specs`：正典規格缺 `## Purpose` 或內容為空＝error；不足 50 字元＝warning（strict 下報）；內容仍為 archive 佔位句＝warning（佔位句長度恆超過 50，長度門檻抓不到它）。門檻值 50 與上游 `MIN_PURPOSE_LENGTH` 同值。

三、存量 66 份 Purpose 一次全補（純文件、零程式碼）：以 change 二 產出的 `validate --specs` 全綠為驗收。

**Rationale**: 議題 1／2／3 不是新設計，而是已封存頁沒跟上看板既有 anatomy，收成一批可避免同批檔案互踩。議題 4 的根因鏈完整可證：佔位句自創始 commit 即為預設路徑、delta 帶入機制 2026-08-03 才上線且 283 份 delta 僅 1 份用過（機制自身的 dogfood）、代理實際讀的 `specs.instruction.md` 從未寫過這條規則、正典寫入後零複查。上游 OpenSpec 的 propose 技能刻意零次提 Purpose——規則單一正典在 schema、技能只負責跑 `instructions` 去拿、錯誤訊息自帶修復指引；其 36 個 capability 無一殘留佔位，我們 66/67 殘留。我方技能與上游同構且多一步收尾 validate 迴圈，故早期檢查＋帶指引的錯誤訊息即閉環：寫作時點由 instruction 承載提醒、失敗時點由錯誤訊息承載修復，技能層第三次提及為純同步負債（技能檔跨多處同步的漂移前例已有）。程式碼與 66 份文件分開成兩個 change，因後者的驗收工具正是前者的產出，且混在一起會撐爆品質站的審查面。

**Rejected alternatives**:
- 只在 tray dispatch 修落頁——涵蓋面不足,其他跨頁入口仍留錯頁。
- 已封存兩種卡都以中文 topic 為標題——需推翻 LANGUAGE.md 受控例外與看板既有設計。
- 拿掉「Purpose 待補」橘字標示當解——治標，等於承認正典不需要能力邊界說明。
- 照抄上游「不硬擋、只靠 validate 事後抓」——正是過去 7 天失效的路徑。
- 技能層加一行檢查點（第六輪曾納入，第七輪撤回）——寫作時點已有 instruction 承載、失敗時點由帶指引的錯誤訊息承載，第三處提及無新增價值、多一處跨檔同步負擔。
- 錯誤訊息只報缺不附修法——乾錯誤訊息把代理推回翻文件，上游經驗是指引附在訊息上才自癒。
- 在技能複製完整 Purpose 規則內文——兩份同源文字必然漂移，asset 才是代理實際讀到的正典。
- 為正典規格驗證新開一支 CLI 動詞——`--specs` 與 `--strict` 已在位（且 `--specs` 目前空轉），新開等於製造第二個把手。
- 存量與守門合併為一個 change——66 份文件會稀釋審查面，且驗收工具在同 change 內才產出。

**Deferred**: 一次全補 66 份的品質控管方式（逐份人審／抽審）留待 change 三 的 propose 決定。上游另有兩項帶入期品質檢查（目標已有不同 Purpose 時警告忽略、帶入後不可讀則退回佔位）未納入本次範圍。

**Capture to**: proposal（三個 change 各自）

**Next**: /speclink-propose --from-discussion archived-parity-and-spec-purpose
