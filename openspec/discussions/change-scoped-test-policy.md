---
topic: change 驗證只跑受影響面的測試，全量 test:all 交給 CI
slug: change-scoped-test-policy
status: promoted
promoted_to: add-improve-flow
created: 2026-08-07
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: change 驗證只跑受影響面的測試，全量 test:all 交給 CI

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：使用者跑 `npm run test:all` 一輪 20 分鐘級且有一筆紅（speclink-node render.spec.ts 的 worktree 軸斷言過時），質疑「每個 change 都跑全量」的必要性——受影響面才需要跑，全量該交給 CI；並問調整落點是否為 config.yaml。

模式：assumptions（codebase scout 找到大量相關檔案：package.json 的 test:all 鏈、openspec/config.yaml 的 rules.tasks、ci.yml 三平台全量、add-improve-flow/tasks.md 的驗證步驟、crates/speclink-node/__test__/render.spec.ts、crates/speclink-core/tests/golden 兩份 worktree on/off 快照）。

相關前情：已封存討論 2026-08-01-slow-global-test-suite 已診斷全量慢的結構性成本（Gatekeeper 掃描稅已用 Warp 豁免＋binary 合併降過一輪；殘餘大頭為 watch 計時器測試 486s、server e2e 344s 含測試內編譯、desktop e2e 122s）。進行中 changes：add-improve-flow（唯一在 tasks 明寫 test:all 的活動變更）、desktop-ui-stamp-and-overflow-polish 與 quality-skill-round-pause（worktree 中，與本題無涉）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-07)

**Focus**: 「每個 change 都跑 test:all」從哪來、該在哪調整
**Position**: 四項假設全數獲使用者確認——調整落點是 openspec/config.yaml 的 rules.tasks：
- 全量的來源是 tasks.md 撰寫慣例而非引擎強制：rules.tasks 現況只要求 golden 與 CLI 測試，是 propose 產驗證步驟時把「npm run test:all 全綠」寫進收尾 task（證據：openspec/changes/add-improve-flow/tasks.md:29）
- 調整走 /speclink-config 落 rules.tasks，一次改、之後每個 change 的 tasks 自動遵循；CI 已在三平台跑全量（ci.yml，且有 scripts 測試釘住），「全量交 CI」前提成立
- 新 rule 寫「原則＋粗對應」：動 crates/<x> 跑 cargo test -p <x>；動到輸出加跑 golden（cargo test -p speclink-core --test it render_golden::）；動前端 workspace 跑 npm test -w <該面>；全量 test:all 由 CI 守門
- 本機不保留強制全量關卡；change 橫跨三面以上時收尾前自行判斷跑一次
- 順帶定案兩個事實題：(1) 現行唯一紅燈是 crates/speclink-node/__test__/render.spec.ts:97 的過時斷言——worktree 開關實際多 4 行（兩技能行＋worktree 工作流行＋Quality stations bullet，golden diff 證實），Rust 端當時同步了、Node 端漏了；(2) 全量慢的大頭是 watch 計時器測試 486s＋server e2e 344s（含測試內編譯）＋desktop e2e 122s＋整包編譯 130s，結構性成本，正當化「全量不該每 change 付」
**Ruled out**: 改 .claude/CLAUDE.md 或 tdd-workflow 技能（不隨 spec store 走，換機器即失效）；逐 crate 細對應表（易過時，每加 crate 要回改 rule）；本機保留 archive 前強制全量（每 change 收尾多付 15–20 分鐘，CI 已守門）
**Open**: 進行中的 add-improve-flow 既有 tasks 是否連帶調整

### Round 2 — assumptions (2026-08-07)

**Focus**: add-improve-flow 的既有 tasks 是否連帶調整
**Position**: 是——但範圍精確到一行：只有 task 6.1 需要改，且改法是點名受影響面而非砍驗證：
- grep 活動 changes（含兩個 worktree 副本）確認 task 6.1 是唯一明寫 test:all 的活動任務；其他 tasks（1.1、1.2、2.1、3.2、4.1、4.3）本來就是 scoped 寫法（cargo test -p、npm test -w），只有收尾 6.1 落回全量
- 6.1 現行寫法「npm run test:all 與 cargo test --workspace 全綠」本身冗餘（test:all 已含 cargo test --workspace）
- add-improve-flow 的受影響面其實很廣（speclink-core／cli／protocol／remote／server 讀取路徑／desktop-core、packages/ui、apps/desktop 前端、speclink-node 的 skill registry 與 marker 渲染），scoped 改寫的實益是跳過「未受影響的貴測試」：src-tauri 的 watch 計時器測試（486s）與 phase3 e2e（122s）、scripts/、apps/server-web——省下約 10 分鐘級
- 建議 6.1 驗證改為：cargo test -p speclink-core -p speclink-cli -p speclink-protocol -p speclink-remote -p speclink-server -p speclink-desktop-core ＋ npm test -w packages/ui ＋ npm test -w apps/desktop ＋ speclink-node 的 napi build 與 npm test ＋ speclink validate add-improve-flow；措辭定稿交給 ingest
- 落地路徑：discuss link 本討論至 add-improve-flow，由 /speclink-ingest 摺入並 seal（add-improve-flow 無 worktree，tasks.md 就在主 checkout；另兩個 worktree 分支帶著的舊副本未被動過，merge 時 git 自動收斂）
**Ruled out**: 保持 6.1 不動（新 rule 只約束未來 change，會留下「規則說 scoped、活動 change 卻示範全量」的自相矛盾）；砍掉 6.1 的跨面驗證（該 change 真的橫跨多面，scoped ≠ 窄，是點名受影響面）
**Open**: 無

## Conclusion

**Decision**: change 的 task 驗證步驟只跑受影響面的測試，全量 npm run test:all 交給 CI 守門。落地兩處：(1) openspec/config.yaml 的 rules.tasks 增一條「驗證步驟原則＋粗對應」rule（動 crates/<x> 跑 cargo test -p <x>；動到輸出加跑 golden；動前端 workspace 跑 npm test -w <該面>；動 speclink-node 跑 napi build＋npm test；全量由 CI 守門，change 橫跨三面以上時收尾前自行判斷）；(2) 進行中的 add-improve-flow task 6.1 連帶改寫為點名受影響面（core／cli／protocol／remote／server／desktop-core 的 cargo test -p ＋ packages/ui 與 apps/desktop 的 npm test -w ＋ speclink-node，跳過未受影響的 src-tauri watch/e2e、scripts/、server-web）。
**Rationale**: 全量的結構性成本（watch 計時器 486s＋server e2e 344s＋desktop e2e 122s＋編譯 130s）已由前討論定性為不可免；CI 已在三平台跑全量，本機重複付費無增量保障。rules.tasks 是 propose 產驗證步驟的正典依據，一次改、之後每個 change 自動遵循，且隨 spec store 走。
**Rejected alternatives**: 改 .claude/CLAUDE.md 或 tdd-workflow 技能（不隨 spec store 走）；逐 crate 細對應表（易過時）；本機保留 archive 前強制全量（每 change 多付 15–20 分鐘且 CI 已守門）；add-improve-flow 6.1 保持不動（規則與活動 change 自相矛盾）。
**Deferred**: crates/speclink-node/__test__/render.spec.ts:97 的過時斷言（worktree 開關實多 4 行、測試仍期望 2 行）是既封存變更漏更新的回歸，與本題平行——以獨立小修處理，不進本討論的 change 鏈。watch 測試 486s 的計時器成本優化亦不在本題範圍。
**Capture to**: openspec/config.yaml 的 rules.tasks（經 /speclink-config）；add-improve-flow 的 tasks.md（經 /speclink-ingest）
**Next**: /speclink-config 落 rule → speclink discuss link change-scoped-test-policy add-improve-flow → /speclink-ingest add-improve-flow（摺入 6.1 改寫並 seal）
