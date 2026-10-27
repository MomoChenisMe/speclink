---
topic: 多個無相依 changes 並行 apply：引入 git worktree 的隔離策略，與 desktop 單一專案身分的呈現
slug: worktree-parallel-apply
status: promoted
promoted_to: worktree-parallel-apply
created: 2026-08-04
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 多個無相依 changes 並行 apply：引入 git worktree 的隔離策略，與 desktop 單一專案身分的呈現

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：多個無相依的 changes 想並行 apply，等同多個 agent 共改同一份 repo——使用者環境已實際發生過平行 session 互踩事故（archive @trace 記錄被另一 session 的髒檔污染）。想引入 git worktree 隔離，但 desktop 以資料夾路徑為專案身分，多 worktree ＝ 多分頁，使用者認為反直覺。

模式：assumptions——掃描找到充足脈絡（apps/desktop/src/tabs.ts、session.ts、store.ts、components/ProjectTabs.tsx；引擎端 crates/speclink-core/src/drift.rs、crates/speclink-cli/src/commands.rs 的 provenance 補救指引）。

相關 changes／specs：看板現有四個 in-progress changes（evidence-home-and-trace-slim、config-station-canon-guard、verify-station-parity、add-improve-flow），即並行需求的實例。相關機制：config 三層（openspec/config.yaml 政策層，tdd／audit 同款 toggle）、in-progress 開工戳記、touched／provenance 記錄。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-04)

**Focus**: 隔離手段與呈現方向定調——五項假設攤給使用者校正
**Position**: git worktree 定為並行隔離的正解，desktop 維持單一專案身分、worktree 降為執行細節：
- 引擎的 provenance 補救指引本就把 isolated worktree 當正規手段（crates/speclink-cli/src/commands.rs:2361）
- speclink 的 git-tracked 狀態（tasks 勾選、開工戳記）隨分支分岔——若每個 worktree 一個分頁，等於 N 份互相過期的看板
- desktop local 分頁身分＝資料夾路徑（apps/desktop/src/session.ts 的 locatorKey：`local:{root}`），worktree 天生被當成不相干專案
- 使用者確認：desktop 只留一個專案身分，多 worktree 共用；worktree 啟用與否放 openspec/config.yaml（GUI「產出政策」區段，比照 tdd／audit toggle）
- 開／收 worktree 由 agent 側（skill 層）負責，引擎頂多在開工戳記延伸記錄工作位置
**Ruled out**: 同資料夾排隊執行——放棄並行，需求不成立；每 worktree 一個 desktop 分頁——使用者明言反直覺，且各分頁互相過期
**Open**: worktree 觸發時機（人決定／每次都開／自動偵測）；remote 模式的對應；主看板是否需要各 worktree 的即時進度（overlay）；.speclink/ 為 gitignored 的每-checkout 狀態，worktree 裡的 touched／evidence 不隨 merge 回家（與 in-flight change evidence-home-and-trace-slim 交疊）

### Round 2 — assumptions (2026-08-04)

**Focus**: 即時進度同步是否為硬需求，與觸發機制的形態
**Position**: 主看板即時同步定為硬需求，走引擎層 overlay；觸發點改為專用技能 speclink-apply-with-worktree：
- 使用者裁定：一個 desktop、多 worktree、多 agent 勾 task done，主看板要即時更新——最小版（merge 後才更新）出局
- overlay 建議落點：引擎／host 層聚合讀（workspace ＝ 主 checkout ＋ linked worktrees），CLI 與 desktop 共用同一實作（呼應 config 規則「同一契約唯一實作落點」）
- change↔worktree 映射建議用分支命名慣例（如 speclink/<change>），discovery 走 git worktree list——git 本身就是登記簿，免新增儲存
- desktop watch 需擴充：監看各 worktree 的 openspec/changes/<change>/，並偵測 worktree 增減（.git/worktrees/ 目錄）
- touched 坑證實已由在途 change 解決：evidence 搬家至 openspec/changes/<change>/.evidence.json（受版控，evidence-home-and-trace-slim proposal「What Changes」），隨分支 merge 回家
- snapshots 疑慮解除：僅 archive 時寫入（crates/speclink-core/src/archive.rs:472 起，.speclink/snapshots/ 存正典 spec 合併前位元組備份）；archive 永遠在主 checkout 執行，worktree 流程碰不到它
- config 的 worktree 欄位建議比照 tdd／audit：技能於執行期讀取（關閉時技能拒跑），config.rs:65 註明政策欄位本就是「Consumed by the skills」
**Ruled out**: 最小版呈現（進度 merge 後才更新）——使用者裁定即時同步為硬需求；自動偵測觸發——由專用技能的顯式呼叫取代，偵測機制（lockfile／heartbeat）複雜度不成比例
**Open**: merge 回主分支由誰負責（技能收尾自動 merge vs 留給人裁決）；worktree 位置慣例與每-worktree 建置成本的提示；主看板 overlay 呈現細節（worktree 標示、待收尾時點、worktree 存在時 desktop 動詞的防護）

### Round 3 — assumptions (2026-08-04)

**Focus**: 技能生成機制、merge 收尾形態、remote 相容性、資料夾樣貌與卡片標示——五個落地細節
**Position**: 全部有解，架構收攏成「兩個技能＋引擎聚合讀＋卡片標示」：
- 技能生成走「生成期組裝」：skills.rs 現況是一技能一模板（assets/skills/*.md 經 include_str! 內嵌、substitute 渲染），apply-with-worktree 於生成期以 worktree 前置／收尾段落包住共用的 apply 本體模板（B_APPLY），輸出自包含的 SKILL.md——維持「逐 skill 正典化＋golden 釘住」的既有紀律，不做執行期跨技能委派
- merge 回主分支：人觸發、agent 執行——配對的小技能（暫名 speclink-worktree-merge）做 preflight（主樹乾淨、分支已 commit）→ merge → 衝突即停等人裁決 → 收 worktree 刪分支；worktree 一消失，聚合讀與卡片標示自動退場（同一 discovery 機制）
- remote 模式 OK：task 狀態寫 server（TeamStore、revision 衝突處理），看板走 SSE 本就即時——聚合讀規則只適用 local workspace，remote 下 worktree 純屬程式碼隔離；.speclink.yaml 受版控、worktree checkout 自動帶 remote marker（上輪已證）
- 資料夾樣貌建議 sibling 巢：主資料夾旁 <repo>.worktrees/<change名>/，每個 worktree 是完整原始碼副本（.git 為指回主 .git 的檔案；自備 node_modules 與 cargo target，開啟時技能應提示建置成本）
- 卡片標示：走 worktree 的 change 於卡片加識別標示、抽屜顯示分支與路徑——標示詞屬詞彙缺口（vocabulary drift），候選：直出「worktree」（工程詞例外，先例：config.yaml 頁簽、討論 slug）vs 白話詞（如「平行工作區」），待使用者裁定後記入 openspec/LANGUAGE.md
**Ruled out**: 執行期跨技能委派（skill 內再呼叫 /speclink-apply）——cwd 與技能上下文的脆弱點多，且違背逐 skill 自包含的既有紀律；repo 內 .worktrees/ 巢（初步傾向排除）——watcher 遞迴、掃描工具、巢狀 node_modules 都要另行排除，且破壞「主資料夾＝專案」的心智模型
**Open**: worktree 巢位置最終裁定（sibling vs repo 內）；卡片標示的使用者可見詞裁定；兩技能的最終命名（propose 時定）

### Round 4 — assumptions (2026-08-04)

**Focus**: 最後兩個裁定——worktree 巢位置與使用者可見用詞
**Position**: sibling 巢定案、「worktree」直出定案：
- 巢位置：repo 外 sibling（<repo>.worktrees/<change>）。使用者確認三個支撐事實：repo 外是 git worktree 原生用法；引擎偵測不受位置影響（git worktree list --porcelain 回報 .git/worktrees/ 名冊中的絕對路徑，引擎不掃磁碟）；提交更乾淨（各 worktree 自有 index 與分支，主樹 git status 不受污染，speclink-commit 檔案歸屬不見外人檔案）
- 用詞：「worktree」直出，記入 openspec/LANGUAGE.md 為工程詞明文例外（先例：config.yaml 頁簽、討論 slug——開發者工具中 git 使用者的原生心智模型，翻譯多一層對應）
**Ruled out**: repo 內 .worktrees/ 巢——主樹 status 污染＋工具排除成本（本輪正式定案）；白話譯詞（如「平行工作區」）——每次閱讀需在腦中翻譯回 worktree

## Conclusion

**Decision**: 引入 git worktree 並行 apply 流程，desktop 維持單一專案身分並即時同步各 worktree 進度。四個面向：
- 隔離：每個並行 change 一個 worktree，巢在 repo 外 sibling（<repo>.worktrees/<change名>），分支命名慣例（如 speclink/<change名>）作為 change↔worktree 的映射鍵——git 本身是登記簿（git worktree list --porcelain），零新增儲存
- 觸發：專用技能 speclink-apply-with-worktree（生成期組裝——worktree 前置／收尾段落包住共用的 apply 本體模板 B_APPLY，輸出自包含 SKILL.md）；呼叫技能＝人決定並行；openspec/config.yaml 新增 worktree 政策欄位（bool，比照 tdd／audit 由技能執行期讀取，關閉時技能拒跑；GUI「產出政策」區段加 toggle）
- 收尾：配對技能 speclink-worktree-merge——人觸發、agent 執行：preflight（主樹乾淨、分支已 commit）→ merge → 衝突即停等人裁決 → 收 worktree 刪分支；worktree 消失即令聚合讀與卡片標示自動退場
- 呈現：引擎／host 層聚合讀（workspace＝主 checkout＋linked worktrees；有活躍 worktree 的 change 改讀 worktree 內的 tasks／stamps／evidence），CLI list 與 desktop 看板共用同一實作；desktop watch 擴充監看各 worktree 的 openspec/changes/<change>/ 與 .git/worktrees/（偵測增減）；卡片加 worktree 標示、抽屜顯示分支與路徑
- remote：聚合讀僅適用 local workspace；remote 的 task 狀態寫 TeamStore、看板 SSE 天生即時，worktree 僅程式碼隔離；.speclink.yaml 受版控、worktree 自動帶 remote marker
**Rationale**: 核心張力是「並行隔離」與「單一看板即時可觀察」不可兼得的表象——以 git 自身當登記簿（worktree list＋分支命名）讓兩者兼得且零新儲存；即時同步為使用者硬需求，故聚合讀落引擎層（同一契約唯一實作落點，CLI 與 desktop 共享）。觸發交給顯式技能呼叫，把「何時並行」還給人、免去不可靠的偵測機制。
**Rejected alternatives**: 每 worktree 一個 desktop 分頁（N 份互相過期的看板，使用者明言反直覺）；同資料夾排隊執行（放棄並行）；每次 apply 都開 worktree（每 worktree 自備 node_modules／cargo target，建置成本不成比例）；自動偵測並行時機（in-progress ≠ 執行中，lockfile／heartbeat 複雜度不成比例）；執行期跨技能委派（cwd 與技能上下文脆弱，違背逐 skill 自包含＋golden 釘住的紀律）；repo 內 .worktrees/ 巢（主樹 git status 污染、watcher／掃描工具排除成本、破壞「主資料夾＝專案」心智模型）；最小版呈現（進度 merge 後才更新——使用者裁定即時同步為硬需求）；白話譯詞（「平行工作區」等——需在腦中翻譯回 worktree）
**Deferred**: worktree 存在時 desktop 動詞的防護（如對 worktree 中的 change 執行 archive／discard）——design 階段定；「待收尾」欄位時點與 overlay 呈現細節——design 階段定；desktop 卡片上的 merge 按鈕——後續視需求；兩技能最終命名——propose 時定；apply-with-worktree 是否內用 Claude Code 原生 worktree 能力——技能模板實作細節（分支命名慣例必須由技能自控）
**Capture to**: proposal（promote 本討論；建議拆兩刀——①引擎聚合讀＋config 欄位＋兩技能，②desktop 即時 overlay＋卡片標示，亦可一刀全包，promote 時裁定）＋ openspec/LANGUAGE.md（「worktree」直出記為工程詞明文例外，隨 change 落地）。前置相依：evidence-home-and-trace-slim 須先落地（.evidence.json 隨 change 目錄進版控，worktree 證據才能隨 merge 回家）
**Next**: /speclink-propose --from-discussion worktree-parallel-apply（或 speclink discuss promote worktree-parallel-apply）
