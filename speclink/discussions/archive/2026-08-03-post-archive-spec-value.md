---
topic: 封存後正典規格的後續價值:外部 AI 討論結論的驗證與 archive 治理裁決
slug: post-archive-spec-value
status: promoted
promoted_to: archive-fail-closed-merge, evidence-home-and-trace-slim
created: 2026-08-03
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 封存後正典規格的後續價值:外部 AI 討論結論的驗證與 archive 治理裁決

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者貼上一份與另一個 AI（Codex）的完整討論逐字稿，主題是封存後規格的長期價值、正典規格與 @trace 的現行邏輯、以及與 OpenSpec（https://github.com/Fission-AI/openspec）的比較，請本場對其結論逐條評估。模式:assumptions（document-input triage——把外部結論當成別人的假設清單，逐條對照本地程式碼與新 clone 的 OpenSpec 原始碼驗證）。codebase scout 命中:crates/speclink-core/src/archive.rs、drift.rs、tasks.rs、skills.rs、crates/speclink-host/src/evidence.rs；相關正典規格:verify-evidence、desktop-app；OpenSpec 對照物:src/core/specs-apply.ts、templates/workflows/sync-specs.ts、repo 根目錄的 openspec-parallel-merge-plan.md。逐字稿為對話內貼文、非 repo 內文件，故無 Source doc。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-03)

**Focus**: 外部 AI 結論的逐條驗證——哪些屬實、哪些要修正
**Position**: 大多數宣稱屬實且引用行號正確，兩處判斷需修正、一處重要情報遺漏:
- 屬實:archive 合併的靜默跳過（ADDED 撞名 archive.rs:451、MODIFIED 缺目標 :459）、RENAMED 保留舊 trace（:479-487）、trace 空清單整塊不注入（:383-387）、新正典 Purpose 固定 TBD（:426-429）、fresh canonical 會物化 MODIFIED（:417-419）
- 屬實:trace 來源的 spec/code drift——verify-evidence/spec.md:136 要求 v2 entries 或 v1 檔案清單，實作在 entries 為空時掃 Git dirty worktree（archive.rs:178-181）忽略 v1；有 v2 時 all_files() 為 v1＋v2 聯集（tasks.rs:577）
- 屬實:drift 的 spec_assumptions 只供 drift 與 bulk readiness check，單筆 archive 無守門（drift.rs:334 註解自承 silently skips）；sync 不在公開 registry（skills.rs:93 registry() 無 sync）
- 屬實:OpenSpec specs-apply.ts 全部嚴格保護（ADDED 撞名 throw :398、MODIFIED 缺目標 throw :361、scenario 遺失 throw :372、新 capability 禁 MODIFIED/RENAMED :234、RENAMED 已實作含 near-miss 提示）；sync-specs.ts:16 自述 agent-driven 智慧合併，分裂屬實
- 修正一:「本地與遠端統一 evidence gate」不是修 bug——verify-evidence/spec.md:136 明文「本地 archive SHALL NOT 強制該檢查」，是規格明定的刻意分層；要改是規格變更，不該與 v1 fallback 這種實作偏離同籃同級
- 修正二:「OpenSpec 對並行修改也沒有解法」情報過時——repo 根目錄 openspec-parallel-merge-plan.md 完整規劃 base fingerprint（SHA-256）→ archive 驗證 → 不符 abort 要求 rebase → 3-way merge → scenario 級 delta；與外部 AI 建議的「基準摘要」獨立收斂到同一設計，反而強化該建議並提供現成參考
- 遺漏:OpenSpec 的 MODIFIED scenario superset check 是 fail-closed 中最有價值的一條——正是 parallel-merge-plan 記錄的實際資料遺失事故（Windsurf scenario 被蓋掉）的第一道防線；與 fingerprint 是同一問題的兩層防禦（superset 擋無意漏抄、fingerprint 擋基礎過期）
- 數字:125 封存、58 正典、383 需求、58/58 TBD Purpose、89 snapshots 全對；@trace 實測 374（外部說 364，小誤差）
**Ruled out**: 照單全收外部結論（兩處判斷錯誤）；「archive 丟向量庫做聊天機器人」優先（沿用外部討論的否決——會混淆過時設計與目前真相，本場驗證未推翻）
**Open**: 使用者對修正後優先序的裁決；本地 evidence gate 是否統一為強制

### Round 2 — assumptions (2026-08-03)

**Focus**: 治理裁決三題——gate 歸屬、本地強制與否、sync 去留
**Position**: 使用者三項裁決確立修正後的工作包形狀:
- 裁決一:「本地/遠端 gate 統一」不另案，併入第三點成單一「trace 證據可信化」包——與 v1 fallback 修正同落 verify-evidence 規格與 archive 路徑，一起動刀
- 裁決二:本地 archive 也要強制 evidence gate——推翻 verify-evidence/spec.md:136 現行「本地 SHALL NOT 強制」明文；查證 check_archive_evidence（evidence.rs:163）目前全 repo 無生產呼叫端（remote Phase 2 也未落地），故本地將成為第一個實際強制點，remote 之後接同一函式即天然統一
- 裁決二的連動:gate 強制後 v1-only change 會被 EvidenceMissing 拒絕，v1 fallback 修正只剩逃生口路徑需要——若允許旗標放行無 v2 evidence 的封存，trace 應按規格用 v1 清單、徹底移除 dirty worktree 猜測
- 裁決三:不做 sync，archive 維持唯一物化點——OpenSpec 的 early-sync no-op 分支（內容相同視為已同步）不需移植，fail-closed 可更嚴格單純:ADDED 撞名一律報錯，不留 idempotent 例外；內部 sync.md asset 原樣留置（既有物，不因本次孤兒化）
**Ruled out**: sync 作為公開動詞（agent 智慧合併與機械合併語意分裂，使用者明示不要）；本地 gate 維持 advisory-only（使用者裁決推翻）；gate 統一另立獨案（與 trace 修正同域，拆開反而兩案互踩）
**Open**: 無 v2 evidence 變更（純規格/文件改動、未走 apply 流程）的逃生口設計；工作如何拆 change；archive 的 discuss 連結是否受影響

### Round 3 — assumptions (2026-08-03)

**Focus**: archive 的 discuss 產物連結對後續規格價值的意義，及本次修正對 discuss 的影響
**Position**: 連結鏈完整且是後續價值的關鍵原料，本次修正對 discuss 零結構影響:
- 現行鏈:正典需求 @trace 的 source → 封存 change 的 .openspec.yaml 保留 from_discussion → 封存討論保留 promoted_to 與結論四欄（Decision/Rationale/Rejected alternatives/Deferred），實例驗證於 2026-07-31-revert-in-progress-to-proposed 兩側
- 價值:rejected alternatives 與 rationale 只存在於討論紀錄——正典規格永遠不帶「為什麼、否決過什麼」；未來做「speclink explain <requirement>」類歷史查詢時，這條鏈是唯一原料，故連結行為應原樣保留
- 限制:@trace 是 last-writer，需求只能反查到最後一次物化它的 change；更早的討論要靠封存區與 snapshot 搜尋（第 1 輪已確認搜尋目前不涵蓋封存區）
- @trace 不需要加 discussion 欄位:可由 change meta 推導，寫進正典是重複真相（衍生索引原則）
- 修正的影響只有時間性:fail-closed 與 evidence gate 拒絕封存時，change 封存延後，連結討論的自動封存（archive.rs:277-293 隨最後一個引用 change 走）跟著延後；討論紀錄格式、promote/link/seal 生命週期全不動
**Ruled out**: 在 @trace 直寫 discussion slug（重複可推導的真相，且 last-writer 語意下只會保存最後一段，誤導大於幫助）
**Open**: 逃生口旗標形狀（建議循 --mark-tasks-complete/--carry-review 既有模式，propose 階段定案）；change 拆分（建議兩刀:archive fail-closed＋Purpose 帶入為一刀，trace 證據可信化＋本地 gate 為一刀；fingerprint/CAS 留待多 session 並行成為痛點時另議）

### Round 4 — assumptions (2026-08-03)

**Focus**: OpenSpec 對「缺 evidence 的封存」怎麼處理——逃生口設計的參考
**Position**: OpenSpec 無 evidence 概念、問題在它那裡不存在，但其守門採兩級分層，直接回答逃生口形狀:
- OpenSpec 沒有 touched/evidence/@trace 任何對應物，archive 只守「驗證」與「任務完成度」兩關（archive.ts:501-537），純文件改動只要任務勾完就直接過——evidence gate 是 Speclink 自己的發明，逃生口也得自己定
- 兩級分層明確:advisory 級（任務未完成、略過驗證）可旁路——人機模式互動確認且預設 No，agent/json 模式不互動、擲型別化錯誤（archive_tasks_incomplete）並印出帶 --yes 的精確重跑指令（rerunCommand，archive.ts:145-170）
- correctness 級（specs-apply.ts 的合併錯誤:ADDED 撞名、MODIFIED 缺目標、scenario 遺失）完全無旁路旗標——--no-validate 只略過文件驗證、不解鎖合併錯誤，唯一出路是 --skip-specs 整段不合併或修好 delta
- 映射到 Speclink:evidence gate 屬 advisory 級 → 專用旗標放行（循 --mark-tasks-complete/--carry-review 既有模式，拒絕訊息點名旗標與原因，Speclink 任務守門已是此風格）；放行時 trace 按規格退用 v1 清單。fail-closed 合併錯誤屬 correctness 級 → 不設旗標，與 OpenSpec 同層
- 採專用旗標而非泛用 --yes:Speclink 非互動、每道守門已各有具名旗標，泛用 yes 會讓一支旗標同時解鎖多道守門、語意稀釋
- 值得順手抄的 UX:OpenSpec 被擋時一律印出可直接複製的完整重跑指令（含當下已帶的旗標），Speclink 拒絕訊息可比照
**Open**: 旗標命名與 conclude（拆分兩刀已在第 3 輪建議，待使用者確認後收斂）

### Round 5 — assumptions (2026-08-03)

**Focus**: @trace 的目的、實際用途與必要性——逐欄位盤點消費端
**Position**: source 欄位必要，code 欄位以現在形式不必要——建議瘦身 @trace 並把檔案證據搬家到封存區:
- 目的溯源:@trace 的 UI 呈現由 2026-07-09-spec-source-footer 引入，提案自述價值是「這條規格出自哪個變更」的 SDD 溯源
- 實際消費端盤點:source 被 SpecDrawer footer（去重顯示、不可點）與 trace_count 溯源變更數（query.rs:135）消費；updated 沒有任何讀者（UI 明文刻意不顯示）；code 沒有任何讀者（trace.ts 只抽 source，query.rs 解析時跳過整塊，skills 也未指示 agent 消費）
- code 欄位同時是最多操作痛的來源:dirty worktree 猜測、bulk archive 的 clean worktree 守門（commands.rs:809 存在理由就是防 dirty 檔污染每個 change 的 @trace）、平行 session 污染事故（使用者記憶有案）
- 耐久性反轉:touched.json 在 commit 後刪除、封存 change 目錄不含 evidence，@trace code 是檔案歸屬在 repo 內的唯一耐久紀錄——但這是把歷史證據塞進正典的結果，違反第 1 輪確立的分層原則（正典＝現況、archive＝歷史證據）
- 建議:@trace 收斂為 source（updated 順手留或刪皆可）；archive 時把 touched/evidence 記錄複製進封存 change 目錄，成為誠實的 change 級檔案歸屬耐久家；374 個既有 @trace 的 code 清單可一次性機械瘦身或原樣留置（propose 定）
- 對已裁決計畫的影響:第二刀 trace-evidence-trust 大幅縮小——v1 fallback、dirty worktree、change 級誤導全部因「正典不再寫檔案清單」而消失；evidence gate（裁決 2）不受影響，它守封存紀律、與正典寫什麼無關；bulk 的 clean worktree 守門可望放鬆
- 未來不受阻:spec health／失效路徑檢查可改讀封存區 evidence；requirement 級譜系本來就規劃住在 spec.md 之外
**Ruled out**: 整支 @trace 移除（source 有現役消費者且是 explain 類查詢的 O(1) 錨點；OpenSpec 式「全靠 archive＋Git」要掃 125 份封存才能反查，且正典檔對 agent 不再自我描述）；code 原樣保留並繼續修可信度（為沒有讀者的欄位付出 v1/dirty/聯集三項修正成本，不划算）
**Open**: 使用者對瘦身方案的裁決；既有 374 個 code 清單遷移與否

### Round 6 — assumptions (2026-08-03)

**Focus**: touched 的家——隨 change 目錄移動取代 .speclink 集中存放，與 @trace 最終形狀
**Position**: 使用者提案 touched.json 搬進 change 目錄，查證後優於第 5 輪的「archive 時複製」方案:
- 決定性事實:.speclink/ 整個 gitignored（.gitignore:10）——evidence 今天是 local-only、不進 git、跨機器不存在、commit 後即刪；@trace code 因此曾是唯一被 commit 的檔案歸屬紀錄
- 搬進 openspec/changes/<change>/ 後:evidence 隨 change 被 commit 與分享、封存時隨目錄自然移入 archive（零複製步驟）、discard 隨目錄消失（.speclink/touched 現存 demo.json 等孤兒即脫鉤證據）、archive skill 的「commit 後刪 touched」步驟雷（使用者記憶有案）結構性消滅
- 前例支撐:change 目錄已有機器寫入的 .openspec.yaml；OpenSpec parallel-merge-plan 同樣提議 changes/<id>/meta.json 放機器 metadata
- 附帶收益:remote 可改經 Store trait 讀 evidence（現行 host drift.rs:148 直讀本機檔案系統），本地/遠端天然統一——範圍 propose 定
- @trace 最終形狀:source＋updated（updated 留作未來用，成本一行）、不再含 code、不再依檔案清單決定注入與否——一律注入，連「無 code 檔連 source 都消失」的舊缺陷一起解
- 設計調整確認:第二刀重塑為「evidence 隨行＋trace 瘦身＋本地 gate」，v1 fallback 與 dirty worktree 修正因 code 欄位移除而整項消滅；第一刀不變
**Ruled out**: archive 時複製 touched 到封存區（第 5 輪原案——搬家更徹底:committed、隨行、無刪除步驟、無孤兒）；touched 留在 .speclink 僅修可信度（evidence 不進 git 的根本問題不動）
**Open**: 無——其餘全數移交 propose（檔名/格式、validate 與 watcher 影響、逃生口旗標命名、374 既有 code 清單遷移、remote Store 統一範圍）

## Conclusion

**Decision**: 分兩刀落地封存治理修正，並確立 evidence 與 trace 的新家。刀一「archive fail-closed」:合併引擎改硬性守門（ADDED 撞名、MODIFIED/REMOVED/RENAMED 缺目標、同需求多操作區段一律報錯，含 MODIFIED scenario superset check），全部 capability 驗證成功後才一次寫入，新 capability 的 Purpose 從 delta 帶入；correctness 級錯誤不設旁路旗標。刀二「evidence 隨行與 trace 瘦身」:touched/evidence 記錄從 gitignored 的 .speclink/touched/ 搬進 openspec/changes/<change>/ 隨 change 生命週期移動（commit 分享、封存隨行、discard 隨滅）；@trace 瘦身為 source＋updated、一律注入、不再含 code 清單；本地 archive 強制 evidence gate（修訂 verify-evidence 的「本地 SHALL NOT」為 SHALL，advisory 級、專用逃生口旗標放行，拒絕訊息點名旗標並附完整重跑指令）；bulk archive 的 clean worktree 守門檢討放鬆。不做 sync，archive 維持唯一物化點；archive 的 discuss 連結（from_discussion ↔ promoted_to）原樣保留。fingerprint＋CAS、穩定 requirement ID、UI 時間線依序後置。
**Rationale**: 逐條驗證外部 AI 結論後確認:@trace 的 code 欄位無任何程式讀者，卻是 dirty worktree 猜測、bulk clean-tree 守門、平行 session 污染三項操作痛的共同根源；檔案證據屬歷史層，其家應在封存 change 目錄而非正典（.speclink gitignored 使 evidence 連 git 都進不去，@trace code 曾是唯一被 commit 的歸屬紀錄——這個唯一性正是分層錯置的產物）。正典只留 O(1) 溯源錨點 source（SpecDrawer 與 trace_count 的現役消費欄位、未來 explain 類查詢的地基）。fail-closed 直接借 OpenSpec specs-apply.ts 的錯誤清單當驗收清單；守門採其 advisory/correctness 兩級分層，旗標循 speclink 既有具名模式（--mark-tasks-complete/--carry-review）而非泛用 --yes。
**Rejected alternatives**: 整支 @trace 移除（source 有現役消費者，拔除後反查需掃 125 份封存、正典對 agent 不再自我描述）；code 欄位保留並續修可信度（為無讀者欄位付 v1 fallback、dirty worktree、change 級誤導三項修正成本）；archive 時複製 touched 到封存區（搬家進 change 目錄更徹底）；sync 公開化（agent 智慧合併與機械合併語意分裂，OpenSpec 同樣分裂中）；泛用 --yes 旁路（一支旗標解多道守門、語意稀釋）；@trace 直寫 discussion slug（可由 change meta 推導，重複真相）；向量庫聊天機器人優先（混淆過時設計與目前真相）；本地 gate 維持 advisory-only（使用者裁決推翻，且 check_archive_evidence 全 repo 本無生產呼叫端，本地將是第一個強制點）。
**Deferred**: 逃生口旗標命名；evidence 檔在 change 目錄內的檔名/格式與 validate、desktop watcher 影響；374 個既有 @trace code 清單一次性瘦身或原樣留置；remote 經 Store trait 統一讀 evidence 的範圍；requirement 基準 fingerprint＋CAS（OpenSpec parallel-merge-plan Phase 0 為現成參考，待多 session 並行成痛點）；穩定 requirement ID 與 trace event 模型；UI 時間線與封存深連結；封存區全文搜尋。
**Capture to**: proposal（兩個新變更，各自獨立出貨）
**Next**: /speclink-propose --from-discussion post-archive-spec-value（先刀一後刀二）
