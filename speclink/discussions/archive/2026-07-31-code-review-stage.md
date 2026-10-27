---
topic: 在 apply 與 verify/archive/commit 之間加入可選的 code review 流程（本地與 remote 皆可用），含卡片與抽屜的已審查標示
slug: code-review-stage
status: promoted
promoted_to: code-review-stage, verify-station-parity
created: 2026-07-31
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 在 apply 與 verify/archive/commit 之間加入可選的 code review 流程（本地與 remote 皆可用），含卡片與抽屜的已審查標示

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者想在 apply 與 verify/archive/commit 之間加入可選的 code review 站，參考 Matt Pocock 的 code-review skill（github.com/mattpocock/skills，skills/engineering/code-review）擬定 speclink 版本；審查過的 change 要在 desktop 卡片與抽屜顯示標示。模式：assumptions——codebase scout 找到充分脈絡（speclink-verify skill 的三維度審查、ChangeMeta 生命週期站模式 model.rs、touched 檔案追蹤 tasks.rs、ChangeCard.tsx／RichDetailDrawer.tsx）。外部參考：Matt 版為兩軸平行審查（Standards＋Spec），diff 基準 `git diff <fixed-point>...HEAD`，結果並列不合併；其 /implement 編排只寫「review 完就 commit」，發現問題後的修正流程留白。相關 changes：無直接關聯（archive-readiness-gating 已完成、desktop-instruction-staleness-prompt 進行中，皆不相干）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-31)

**Focus**: speclink 版 code review 的基本形狀——六項假設攤開後，使用者裁定了報告落地與可選性
**Position**: 骨架承襲 Matt 版（平行 sub-agent、各軸互不污染、並列呈現），但落地方式依 speclink 既有機制調整：
- 審查範圍以 change 的 touched 記錄定界（tasks.rs 的 touched-file tracking），不要求使用者指定 git fixed point
- 標示＝ChangeMeta 新生命週期站 reviewed_at/by/with，比照 started_*（model.rs:26-33）；蓋章走新引擎動詞，remote 模式下 Context Projection 唯讀、寫入必須動詞化
- CLI `list --json` 有 parity pin（listing.rs:161），reviewed_* 只進 desktop 協定不進 CLI 公開輸出
- 使用者裁定：不留存 review.md 審查報告，僅在 .openspec.yaml 蓋章——多輪增量審查的輪間狀態活在對話內，跨 session 重啟則整個 change 重審
- 使用者確認：review 為可選站，是否執行由使用者自行判斷；workflow 更新為 apply → review? → verify? → archive
- 使用者提出：審查發現問題後應詢問使用者是否繼續修正（修正迴圈的具體形狀待下輪定）
**Ruled out**: review.md 落地為 change artifact——使用者裁定標示進 metadata 即可，報告不需留存；強制所有 change 過審——簡單 change 不需要，可選性歸使用者
**Open**: 軸怎麼切（Standards＋Correctness、spec 對齊讓給 verify？）；修正迴圈具體形狀（誰修、如何只重審問題點、蓋章時機）；失效策略（審查後又改 code 是否降級標示）；蓋章動詞與欄位細節；卡片／抽屜呈現細節；「審查」正典詞

### Round 2 — assumptions (2026-07-31)

**Focus**: 乾淨 subagent／跨 session 的修正迴圈如何交接審查狀態——上輪「不落地」裁定被使用者自己的交接需求推翻的範圍
**Position**: 審查狀態落地為 change artifact `review.md`，動詞驅動、蓋章即刪：
- 推翻理由：touched 記錄在 `.speclink/`（gitignored；workspace.rs:75-79）——本機工作狀態不跨機器；對話內狀態不跨乾淨 subagent；「下一個人接手修正」需要可傳遞的工單
- ledger 仿 discussion record：固定骨架、round append-only，每輪記「本輪範圍＋分級 findings」；末輪 findings＝下個執行者的工單；某輪 findings 為空＝可蓋章
- 位置 `openspec/changes/<name>/review.md`：本地模式跟 git（與 tasks.md 同命運、speclink-commit 歸屬此 change）；remote 模式走 store 文件管道（store trait 的 read／begin_unit_of_work／commit，speclink-store/src/lib.rs:44-84——tasks.md 勾選已走同路），Context Projection 唯讀、動詞寫入
- 與上輪調和：上輪否決的是「永久留存的審查報告」；本輪引入的是「進行中的交接工單」——`review stamp` 寫 reviewed_* 後刪除 ledger，durable trace 仍僅 metadata 章，上輪裁定不變
- 動詞面 mirror discuss 家族：`review add-round`／`review show --json`／`review stamp`／`review discard`
**Ruled out**: 狀態放 `.speclink/`——gitignored 不跨機器；狀態只活在對話——不跨乾淨 subagent 與 session；per-finding ID＋resolve 動詞——末輪清單即工單，細粒度追蹤是過度設計
**Open**: ledger 設計待裁決；初審範圍在非 apply 機器的 fallback（touched 不跨機器，git diff 或詢問基準）；軸切分（round 1 遺留）；失效策略；蓋章欄位細節；卡片／抽屜呈現細節；「審查」正典詞

### Round 3 — assumptions (2026-07-31)

**Focus**: 工單設計核可後的整體漏洞審視，與 skill 內容的 Matt 版對照骨架
**Position**: 工單設計定案（使用者核可）；漏洞審視八項——三項需裁決、三項有既定緩解、兩項查證後排除：
- 需裁決①失效策略：建議蓋章時錨定任務完成快照（done/total），之後任務狀態變動 → 標示轉「已審查（其後有變動）」
- 需裁決②未結工單遇封存：archive 整目錄搬移（archive.rs:94 起，metadata 蓋章後移動）會讓未結 ledger 化石化進 archive；建議提示（蓋章／discard／照樣帶走）不硬擋
- 需裁決③軸切分最終確認（round 1 遺留）
- 緩解項：修正輪檔案歸屬（fix 不經 task done，touched 漏記新檔——archive 從 commit 重建可自癒）；平行 session（既有紀律）；非 apply 機器初審（fallback 問基準，續輪靠 ledger 記範圍免疫）
- 排除項：review.md 不破 status/validate——status 是 schema 白名單制（status.rs:36 schema.artifact(id)），sidecar 不進 artifact DAG；remote 蓋章身分走既有 auth／created_by 模式
- skill 執行形態：不能照抄 verify 的 fork+Explore——Explore 不能 spawn agents，Matt 式平行雙 sub-agent 需主線 orchestrator；fork 內也不能問互動題（verify skill fork 段落明文）
- skill 骨架對照 Matt 五步：釘範圍（touched／工單取代 fixed-point 問答）→ 蒐集 Standards 來源（CLAUDE.md 取捨規則＋LANGUAGE.md＋Fowler 基線，repo 優先）→ 平行雙 sub-agent read-only 各 <400 字 → 並列呈現不合併不重排 → 寫入 ledger ＋ 迴圈詢問 ＋ 空輪蓋章（Matt 到報告為止，speclink 補迴圈與蓋章）
**Ruled out**: review skill 用 fork+Explore 執行形態——無法 fan-out 平行 sub-agent 也無法互動詢問
**Open**: 失效錨定是否 v1 就做；archive 遇未結工單的處理；軸切分最終確認；蓋章欄位細節；卡片／抽屜呈現細節；「審查」正典詞

### Round 4 — assumptions (2026-07-31)

**Focus**: 完成度守門對失效策略的簡化，與 spec 脈絡在 review 中的角色修正
**Position**: 使用者兩項提案改變了設計——守門簡化失效、spec 脈絡回到 review：
- 採納「review 以全任務完成為前置守門」（與「已就緒」階段對齊、與流程位置 apply→review 一致）：失效規則簡化為一條——蓋章記任務總數（reviewed_tasks_total 一個欄位），失效＝「非全完成」或「總數與蓋章時不符」；ingest 重排任務後再完成也抓得到（總數變了）；總數巧合相同的極端情況接受為代價
- 修正輪只改 code 不動任務狀態，不會誤觸失效
- Spec 軸立場修正（使用者挑戰有理）：不知道意圖就無法分辨「bug」與「刻意行為」——change artifacts（proposal／design／spec deltas）餵給兩個 sub-agent 當判準脈絡；但報告軸仍為 Standards／Correctness，逐 requirement 的系統性覆蓋審查歸 verify——review＝抽查式、以 code 為中心（spec 當脈絡）；verify＝系統式、以 spec 為中心（逐條對 code）
- Matt Step 2（找 spec 來源）從「整步刪除」改為「保留但原生化」：不用四層搜尋，change 目錄的 artifacts 就是 spec 來源
**Ruled out**: 完整 Spec 合規軸進 review——與緊接其後的 verify 重工、產出兩份可能矛盾的報告；蓋章存 done/total 雙數或任務文本 hash——守門保證蓋章時全完成，只需 total 一欄
**Open**: 未結工單遇封存的處理（正向使用者解釋）；蓋章欄位定案；卡片／抽屜呈現細節；「審查」正典詞

### Round 5 — assumptions (2026-07-31)

**Focus**: review 與 verify 是否衝突／可否二選一——以 Matt Pocock 理念與 OpenSpec 上游 verify 用途查證
**Position**: 不衝突、也不是替代品——兩者是 Matt 兩軸拆成兩站：
- 外部查證①：Matt 管線只有 code-review 一個檢查站（spec→tickets→implement→review→commit），所以它必須背 Spec 軸；其核心原則＝「單一混合裁決會讓一軸遮蔽另一軸」，兩軸以平行 sub-agent 隔離（aihero.dev/skills-code-review）
- 外部查證②：OpenSpec 上游 /opsx:verify＝"Validate implementation against artifacts"（expanded profile、可選站），且上游全流程無 code review 概念（docs/opsx.md）
- 結論：speclink 的 review＋verify＝把 Matt 的兩軸拆成兩站——review 承 Standards（工藝）＋Correctness（獵 bug），verify 承 Spec（系統性合規）；Matt 的遮蔽原則反而支持拆站，隔離做得比他更徹底
- 非二選一：verify 答「建對了嗎（照 spec）」、review 答「建得好嗎（工藝）」；兩站皆可選，依 change 風險組合（簡單→都不跑；一般→verify；高風險→都跑）
- spec-as-context 不產生合規裁決；artifacts 稀薄時 Correctness 軸僅憑 code＋tests 判斷、不臆造意圖（對應 Matt「no spec available 就不發明需求」）
- 使用者提案採納：封存偵測未結工單提示三選項（回去蓋章／放棄審查刪工單／照樣帶走）；「照樣帶走」的化石工單升格為「曾審查未通過」標示的資料來源
- 標示狀態機定案：active＝無標示／審查中（工單存在）／已審查（章）／已審查·其後有變動（章＋任務狀態不符）；archived＝已審查（章）／曾審查未通過（工單無章）
**Ruled out**: review 與 verify 擇一互斥——擇一即丟失一軸；合併為單一超級檢查站——違反 Matt 遮蔽原則
**Open**: 蓋章欄位定案（reviewed_at/by/with＋reviewed_tasks_total）；卡片／抽屜樣式細節；「審查」正典詞

### Round 6 — assumptions (2026-07-31)

**Focus**: 範圍擴充——生成指令檔／文件的工作流程更新，與 verify 的蓋章＋標示對稱補齊
**Position**: 兩項擴充皆採納：
- 工作流程正式定調為並行可選站：`discuss? → propose → apply ⇄ ingest → (review? ∥ verify?) → archive`——review 管工藝、verify 管合規、互不依賴；生成的 CLAUDE.md／AGENTS.md 工作流程行與技能清單（含新 `/speclink-review`）落點在引擎模板（skills.rs 逐 skill 正典化＋instructions.rs 渲染＋tests/golden 再生），README／docs 同步；進行中的 desktop-instruction-staleness-prompt 恰好會提示既有專案更新過期指令檔
- verify 蓋章對稱補齊：`verified_at/by/with`＋`verified_tasks_total` 進 .openspec.yaml，失效規則與 review 同一條（任務狀態偏離蓋章時的全完成 → 降級）；標示狀態機 active＝已驗證／已驗證·其後有變動、archived＝已驗證；卡片可同時掛審查章＋驗證章兩顆小標
- verify 不設工單 ledger——其迴圈維持對話內（合規缺口通常即時修），只補章與標示；蓋章時機（fork 內零 CRITICAL 自動蓋 vs 回主線蓋）留待 propose 定
- 跨站失效侷限誠實記錄：任務錨點抓不到「蓋章後純 code 修改」（如 review 蓋章後 verify 驅動的修正）——v1 以使用指引緩解（兩站都跑時，修正收斂後最後再各自蓋章），機制性偵測延後
- 規模建議：扇出為兩個 change——①審查站本體（動詞家族＋skill＋工單＋標示＋文件）②verify 蓋章對稱（小而獨立）；promote 時決定
**Ruled out**: verify 也上工單 ledger——其修正即時性高，YAGNI；把文件更新排除在 change 範圍外——生成模板在引擎內，不改模板等於沒改流程
**Open**: 結論定稿待使用者點頭；蓋章時機細節（propose 階段）

### Round 7 — assumptions (2026-07-31)

**Focus**: 「蓋章後純 code 修改偵測不到」的侷限有無機制性改善——內容指紋錨定提案
**Position**: 有——蓋章時對審查範圍檔案集記內容指紋，讀取時重算比對：
- `review stamp` 時已知範圍（工單各輪記錄的檔案清單聯集），對每檔內容取雜湊，存 `reviewed_scope`（path＋hash 清單）進 .openspec.yaml；工單刪除後範圍清單由章保存
- 顯示端重算比對：任一檔內容變 → 標示降級「已審查·其後有變動」；與任務錨互補——任務錨抓「計畫變了」（ingest 增補），指紋抓「code 變了」（蓋章後修改），兩錨皆廉價、都保留
- 不依賴 git：比照 drift 的先例（drift.rs:121 從 WorkspaceFacts 快照運作、no git access）——避開「蓋章時工作樹是 dirty、HEAD 不代表被審狀態」與「無關 commit 翻動 HEAD 造成大量誤報」兩個 git 錨的致命傷
- 計算落點：有工作樹的介面（desktop 經 tauri adapter、CLI）重算；無工作樹的 server-web 顯示章但凍結度標 unknown——best-effort 誠實
- verify 章同用此機制（verified_scope＝蓋章當下 touched 檔集指紋）
- 殘餘漏網誠實列出：蓋章後「新增的、範圍外的檔案」不在指紋內（後續修改通常動既有檔，影響小）；server-web 無法斷言凍結度
- 建議納入 v1：成本＝一個欄位＋蓋章一次雜湊＋讀取端一次比對；「最後再蓋章」從必要紀律降為好習慣
**Ruled out**: git HEAD／commit 錨——dirty 工作樹使 HEAD 不代表被審狀態、無關 commit 誤報、且強迫流程重排（先 commit 才能審）；持續輪詢偵測——讀取時重算即可
**Open**: 使用者裁決是否納入 v1；結論定稿

### Round 8 — assumptions (2026-07-31)

**Focus**: verify 為何不上工單——使用者以實際體感推翻第六輪的 YAGNI 裁定
**Position**: 採納——verify 也上工單，兩站共用同一套「品質站」生命週期機制：
- 推翻理由：當初 review.md 的存在理由（乾淨 subagent／跨 session 交接、使用者回來忘了修、關 session 記錄清空）對 verify 的 findings 一字不差地成立——Completeness 之外的 Correctness／Coherence 發現（需求實作錯、偏離 design）沒有任何耐久落點，任務勾選狀態只覆蓋「沒做完」這一類
- 機制統一：品質站模式＝守門 → 工單多輪（findings 分級、append-only）→ 迴圈 → 蓋章（雙錨：任務錨＋內容指紋錨）→ 標示；引擎參數化實作一次，review／verify 各是一個實例（verify.md 工單＋verified_* 章）
- 狀態機補齊 verify 側：驗證中（工單存在）／已驗證／已驗證·其後有變動／曾驗證未通過（archived 工單無章）；封存提示同時偵測兩種未結工單
- 保留的刻意不對稱：review 的「執行」守門在全任務完成（審成品工藝）；verify 隨時可跑（中途跑＝進度盤點，報告未完成任務本來就是其 Completeness 維度的功能），但「蓋章」守門在全任務完成＋零 CRITICAL——守門位置不同、章的語意相同
- 工單內容紀律：不重複既有耐久狀態（「任務 7 未勾」不進工單——tasks.md 就是它的落點），只記無處可放的發現
- 執行形態：verify 檢查段維持 fork（read-only、可跑 Bash 寫 add-round），互動迴圈與蓋章回主線——與 review 主線 orchestrator 收斂到同一使用者體驗
**Ruled out**: 第六輪「verify 不設工單」的 YAGNI 裁定——使用者提出的是實證需求非臆測；兩站各造一套持久化機制——引擎一次實作、參數化兩實例
**Open**: 結論定稿（使用者稱此為最後一個問題）

## Conclusion

**Decision**: 在 apply 之後加入與 verify 並行的可選品質站 `/speclink-review`，workflow 定調 `discuss? → propose → apply ⇄ ingest → (review? ∥ verify?) → archive`。核心設計：

1. **分工**（Matt 兩軸拆兩站）：review＝工藝——Standards（repo 慣例＋Fowler smells 基線，repo 文件優先）＋Correctness（bug 獵捕）平行 read-only sub-agent，change artifacts 當判準脈絡、不產合規裁決；verify＝合規——既有三維度以 spec 為中心逐條。兩站皆可選、互不依賴、依風險組合。
2. **品質站生命週期**，引擎參數化實作一次、兩站各一實例：守門 → 工單多輪 → 迴圈詢問（修正後重審／接受蓋章／先不蓋）→ 空輪蓋章 → 標示。
3. **工單**＝`openspec/changes/<name>/review.md`／`verify.md`：動詞驅動（`add-round`／`show --json`／`stamp`／`discard`）、round append-only、每輪記範圍＋分級 findings（CRITICAL/WARNING/SUGGESTION）、末輪＝下個執行者的工作單；蓋章即刪；本地跟 git、remote 走 store 文件管道；紀律：不重複既有耐久狀態（任務未勾歸 tasks.md）。
4. **章**＝`reviewed_*`／`verified_*`（at/by/with）＋雙錨：任務錨（`*_tasks_total`，抓計畫變動）＋內容指紋錨（`*_scope` path+hash 清單，抓蓋章後 code 變動；不依賴 git，比照 drift 的 WorkspaceFacts 先例）；失效 → 標示降級「·其後有變動」。CLI parity pin：新欄位不進 `list --json` 公開輸出，只進 desktop 協定。
5. **守門**：review 執行守門＝全任務完成（已就緒）；verify 隨時可跑（中途＝進度盤點），蓋章守門兩站一致＝全任務完成＋零 CRITICAL。
6. **標示狀態機**（兩站同構）：active＝無標示／審查中（工單存在）／已審查／已審查·其後有變動；archived＝已審查／曾審查未通過（化石工單＝證據）；卡片行內小章、抽屜資訊列。
7. **封存**偵測未結工單（兩種）提示三選項：回去蓋章／放棄（刪工單）／照樣帶走（永久掛「曾…未通過」）。
8. **執行形態**：review＝主線 orchestrator（需 fan-out 平行 sub-agent＋互動詢問，不能 fork+Explore）；verify 檢查段維持 fork，迴圈與蓋章回主線。
9. **文件**：引擎模板更新（skills.rs 新增 /speclink-review 正典 skill、instructions.rs 的 workflow 行與技能清單、tests/golden 再生）＋README/docs；desktop-instruction-staleness-prompt 會向既有專案推送更新提示。

**Rationale**: Matt 的遮蔽原則（混合裁決互相遮蔽）貫徹到站級——他的管線僅一個檢查站所以 Spec 軸擠在 review；OpenSpec 上游有 verify 無 review；speclink 把兩軸拆兩站各得其所。工單同時滿足乾淨 subagent／跨 session 交接與「不留存審查報告」（蓋章即刪；唯一例外＝未通過化石工單升格為標示證據）。品質站參數化：一次實作、兩站受益、使用者體驗一致。

**Rejected alternatives**: 完整 Spec 合規軸進 review（與 verify 重工、兩份矛盾報告）；review/verify 互斥擇一（丟軸）；狀態放 .speclink/（gitignored 不跨機器）；狀態只活在對話（不跨乾淨 subagent）；fork+Explore 執行 review（不能 fan-out／互動）；per-finding ID＋resolve 動詞（過度設計）；git HEAD/commit 錨（dirty 樹不代表被審狀態、無關 commit 誤報、強迫流程重排）；verify 不設工單（第六輪 YAGNI 裁定，被使用者實證推翻）；永久留存審查報告（使用者裁定，章即痕跡）。

**Deferred**: ingest 重排後任務總數巧合相同的失效漏網（接受為代價）；蓋章後新增之範圍外檔案不在指紋內（影響小）；server-web 無工作樹、凍結度標 unknown；非 apply 機器初審的範圍 fallback 細節；標示視覺樣式與 verify 蓋章時機（fork 內自動 vs 回主線）——propose 階段定。

**Capture to**: proposal（扇出兩個 change：①審查站本體＝動詞家族＋skill＋工單＋標示＋文件模板；②verify 品質站對稱補齊）＋ LANGUAGE.md（「審查」「驗證」詞彙組與狀態詞）

**Next**: `speclink discuss promote code-review-stage`（轉出第一個 change：審查站本體）；第二個 change 屆時以 `/speclink-propose --from-discussion code-review-stage` 再轉出
