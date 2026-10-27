---
topic: capability 命名不一致導致重複規格：OpenSpec 上游後續作法與 Speclink 防護計畫
slug: capability-naming-dedup
status: promoted
promoted_to: capability-naming-guard
created: 2026-08-20
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: capability 命名不一致導致重複規格：OpenSpec 上游後續作法與 Speclink 防護計畫

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

Day 09 文章查核引出的問題：archive 依資料夾路徑合併規格，capability 取名靠 AI 判斷。AI 判斷失準時（既有 `auth` 卻另建 `authentication`），會產出兩份語意重複的正典規格，路徑比對式的合併不會發現。本討論盤點 OpenSpec 上游的後續作法與 Speclink 現況，並定出 Speclink 的防護計畫。

模式：assumptions——codebase scout 找到多個直接相關的原始檔（crates/speclink-core/src/newcmd.rs、validate.rs、archive.rs、assets/skills/propose.md、.claude/skills/speclink-propose/SKILL.md）。

相關規格／討論：正典規格 spec-validation、archive-merge（新開 capability 的 Purpose 守門即出自此二處）；已轉出討論 schema-engine-openspec-parity（題目相鄰：schema 引擎對齊上游）。

上游研究基準：OpenSpec repo main（commit 1ebddd1，2026-08-19），Issue #901、#1689，PR #902、#700、#1700，releases v1.0.0→v1.10.0。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-20)

**Focus**: 對「新 capability 撞名既有規格」，OpenSpec 上游與 Speclink 目前各有哪些防護？
**Position**: 兩邊都沒有確定性的命名防護；上游主線是強化 AI 指令，Speclink 引擎已有可掛防護的偵測點，但目前只拿來守 Purpose。
- 上游指令現況：schema.yaml 對查既有規格只有一句 "Research existing specs before filling this in"，沒教 AI 用哪個指令盤點（schemas/spec-driven/schema.yaml line 16-21）。
- 上游動向：PR #1700（作者已 approve、未合併）把 `openspec list --specs` 逐字寫進指令；spec catalog 提案鏈（#901→#902→#700）掛 design-review 標籤四個多月未決；v1.9.0 反而移除自動載入的 spec 索引 openspec/AGENTS.md，讓問題短期更糟（#1689）。
- 上游確定性防護：MODIFIED delta 打錯名時 archive 硬擋（specs-apply.ts:324-328）；ADDED-only 的重複命名暢行無阻——`authentication/` 撞 `auth/` 的典型情境正是全 ADDED。Levenshtein 工具存在（utils/match.ts）但只用於 show/validate 的 "Did you mean"，不用於 archive。
- 上游唯一的網：archive 非 --yes 時列出每個 capability 的 create/update 預覽給人工確認。
- Speclink 指令層：propose skill 第 3 步已要求掃描既有規格（`speclink list --specs --json` → 讀候選 Purpose），但結果只顯示、不擋、不留痕（SKILL.md:98-108）——比上游現況強，本質仍靠 AI 自覺。
- Speclink 建立點：`speclink new artifact spec <cap>` 收什麼名字建什麼資料夾，不查正典、不驗格式（newcmd.rs:47-53）。
- Speclink 引擎偵測點：validate（design D2）與 archive（design D3）都會判斷「正典無此 capability」，目前只守新 capability 的 Purpose 品質與 ADDED-only，不做命名比對。程式庫無任何相似度比對程式碼。
- 規模風險已在門口：本專案自有 71 個正典 capability，正是 #901 描述「capability 一多 AI 掃描就漏」的區間；且命名家族大量共用 token（archive-merge/archive-skill、commit-skill/config-skill）。
**Ruled out**: 「等上游解決」——上游確定性防護是空白，路線圖只走指令強化（PR #1700），catalog 提案懸而未決，時程不可依賴。
**Open**: 防護掛哪些點（建立點／validate／archive）？擋法用「新 capability 確認制」還是「相似度門檻硬擋」？比對範圍要不要含平行 in-flight change 的 delta capabilities？AI 指令側要不要要求 New Capabilities 每項附「為何既有規格不涵蓋」並留痕？語意重複（login vs auth）字串比不出來，如何處置？

### Round 2 — assumptions (2026-08-20)

**Focus**: Day 09 草稿（archive 段落）的敘述與研究事實是否相符？
**Position**: 草稿論述全部成立，僅一處建議精確化，另有兩個可選補強；文章未修改。
- 「archive 會詢問要不要先同步規格」成立：main 的 archive.ts:1497 確認提示 "Proceed with spec updates?"，非 --yes 必問。
- 「capability 名稱在 proposal 階段決定」「現行 schema 要求沿用既有完整路徑」「propose Skill 提醒保留 path」皆對到原文（schema.yaml、skills/openspec-propose/SKILL.md:22）。
- 「還沒有內建機制判斷 auth 與 authentication 是否同一 capability」「#901 仍在提案階段」與研究一致。
- 精確化建議：「只按路徑處理」有一例外——delta 含 MODIFIED/RENAMED 而目標規格不存在時 archive 報錯（specs-apply.ts:324-328）；只有全 ADDED 的 delta 靜默通過。MFA 例子是 ADDED，草稿結論不變，建議補一句。
- 可選補強一：archive 確認前印出 "Specs to update:" 並逐 capability 標 update/create（archive.ts:1417-1422）——修改既有功能卻看到 create 即警訊，是上游唯一的人工攔截點。
- 可選補強二：「現在有什麼不同？」框可補時間線——v1.9.0 移除自動載入的規格索引（#1689）、PR #1700（已 approve 未合併）才首次把 `openspec list --specs` 寫進指令。
**Open**: 沿用第 1 輪——Speclink 防護計畫的五條假設（建立點確認制、validate warning、平行 delta 比對、AI 指令留痕、archive 不加）待使用者裁定。

### Round 3 — assumptions (2026-08-20)

**Focus**: 五條假設全數確認後，整體防護機制的形狀是什麼？
**Position**: 三層機制——AI 指令層（留痕＋說理）、建立點確認制（主閘）、validate 相似警告（第二網）；核心原則是「擋在『正典沒有這個名字』的二元事實上，相似度只用來排序建議、不當擋人的門檻」。
- 使用者裁定：五條假設全對；Day 09 的精確化與兩個補強也已落檔（iTHome2026-challenge/articles/day-09/article.md，三處修改、未 commit）。
- 建立點主閘：`speclink new artifact spec <cap>` 遇正典沒有的名字時預設拒絕；錯誤訊息列出最接近的既有名（含其他進行中 change 的 delta capabilities，標注來源與 Purpose 首行）；確定要開新 capability 必須帶 `--new` 重跑。
- 近似名單排序：token 完全包含 > kebab token 交集 > 編輯距離，取前三；不設硬門檻，避免 archive-merge/archive-skill 這類命名家族誤殺。
- validate 第二網：新 capability（正典無同名）且與既有名相似 → warning 級 lint；propose 收尾自動觸發。已驗證只有 propose 指令走 `new artifact spec`，ingest 是直接編修檔案、可能徒手開新目錄——validate 是唯一涵蓋所有入口的網。
- archive 不加新防護：維持既有 Purpose 守門與 ADDED-only。
- AI 指令側：propose asset 的 New Capabilities 每項附一句「為何既有規格不涵蓋」、掃描結果留痕、寫明 `--new` 語意；ingest asset 補「新增 delta capability 時先對照既有名」。改 assets 觸發 MARKER_VERSION／golden／assets.lock 三連動。
- 波及面：newcmd.rs（主閘）、validate.rs（lint）、相似度小工具（speclink-core 內、不另立 crate）、store 的跨 change delta 列舉（雙 store＋conformance）、propose/ingest 兩份 asset。
**Ruled out**: 相似度門檻硬擋（命名家族誤殺，正式排除）；`--new` 另留 metadata 痕跡（新 capability 已被 Purpose 守門強制說明用途，重複記錄無收益）。
**Open**: worktree 內新建的 delta 目錄對主 checkout 不可見的邊界（接受為已知限制或在 propose 階段處理）；語意重複（login vs auth）維持暫緩。結論待寫，寫完可 promote。

### Round 4 — assumptions (2026-08-20)

**Focus**: 範圍更正——Day 09 文章要不要由本討論代改？
**Position**: 不改。文章屬使用者自管，第 3 輪落檔的三處修改已全數還原（git diff 確認與提交版一致）；本討論自此只專注 speclink 的防護機制。查核結論（一處精確化＋兩個補強）仍留在第 2 輪供使用者自行取用。
**Open**: 沿用第 3 輪——機制形狀待使用者最終確認後寫結論；worktree delta 可見性邊界與語意重複暫緩項不變。

## Conclusion

**Decision**: 為 Speclink 加上三層 capability 命名防護，核心原則是「擋在『正典沒有這個名字』的二元事實上，相似度只用來排序建議、不當擋人的門檻」。
- 建立點主閘：`speclink new artifact spec <cap>` 遇正典沒有的名字時預設拒絕，錯誤訊息列出前三個近似既有名（來源含正典規格與其他進行中 change 的 delta capabilities，各附來源標注與 Purpose 首行；排序：token 完全包含 > kebab token 交集 > 編輯距離）；確定開新 capability 必須帶 `--new` 重跑。
- validate 第二網：新 capability（正典無同名）且與既有名相似時報 warning 級 lint；propose 收尾自動觸發，並涵蓋 ingest 徒手開目錄這類繞過 CLI 的入口。
- AI 指令側：propose asset 加掃描留痕、New Capabilities 每項附一句「為何既有規格不涵蓋」、寫明 `--new` 語意；ingest asset 補「新增 delta capability 前先對照既有名」。
- archive 不加新防護，維持既有的 Purpose 守門與 ADDED-only 限制。
**Rationale**: 上游 OpenSpec 的確定性防護是空白（全 ADDED 的 delta 撞名靜默通過），路線圖只走 AI 指令強化（PR #1700）且時程不可依賴；而 Speclink 引擎已有「新開 capability」偵測點（validate design D2、archive design D3）與 propose 收尾自動 validate，掛防護成本低。用二元事實擋、相似度只排序，是因為本專案 71 個正典 capability 有大量共用字根的命名家族（archive-*、*-skill），任何相似度門檻都會誤殺。
**Rejected alternatives**: 等上游解決（CLI 防護空白，catalog 提案 #901/#902/#700 掛 design-review 四個月未決）；相似度門檻硬擋（命名家族誤殺）；archive 加防護（實作已完成才擋、代價最高，且引擎直跑無互動確認）；`--new` 另留 metadata（Purpose 守門已強制新 capability 說明用途，重複）；純 AI 指令強化（#901 底下的實戰回報證明 AI 會跳過步驟，不可單獨依賴）。
**Deferred**: 純語意重複（login vs auth，字面不像）字串比對抓不到，Purpose 文字留作未來語意比對素材；worktree 內新建的 delta 目錄對主 checkout 不可見造成的跨 change 比對漏洞，propose 階段再決定接受為已知限制或處理。
**Capture to**: proposal（新 change；波及 newcmd.rs、validate.rs、相似度小工具、雙 store 跨 change delta 列舉與 conformance、propose/ingest 兩份 asset → MARKER_VERSION／golden／assets.lock 三連動）
**Next**: /speclink-propose --from-discussion capability-naming-dedup（或 speclink discuss promote capability-naming-dedup 走快速路徑）
