## Why

propose 收尾的「執行順序盤點」今天只活在 AI 腦中：propose-skill 規格要求代理人每次讀完全部提案判定誰可並行、誰要依序，但判定結果不落檔，apply 技能也不讀，下一個 session 只能再讀一次全部 change、再燒一次 token。看板與系統匣則只照時間排，使用者看不出誰該先做。本變更是討論 change-execution-order 的第一刀（共三刀）：把順序判定搬進引擎、把軟依賴落檔、給 apply 一個零 token 的查詢入口。目標使用者是透過 AI 代理跑 SDD 的開發者，對應 propose 收尾、apply 開工、archive 收尾三個技能階段。

## What Changes

- change meta（`.openspec.yaml`）新增 `depends_on` 欄位：逗號清單，列出本 change 的前置 change 名稱。缺席讀作無依賴，既有 meta 檔不受影響；`list --json` 與 `status --json` 輸出逐位元不變。
- 新增唯讀動詞 `speclink plan [--json] [--no-color]`：只算未封存的 change，輸出波次（同一波可並行）、每個 change 的階段、宣告依賴、delta capability 重疊、尚未封存的前置（`blockedBy`）與可開工旗標，以及第一個可開工的 change（`next`）。依賴成環時以非零 exit code 回錯誤並點名環上的 change。
- 新增寫入動詞 `speclink change depends <name> --on <other>... [--remove]`：把前置寫進 `depends_on`。目標不存在、自我依賴、寫入後成環、前置已封存皆拒絕且零寫入；`--remove` 可清掉指向已封存或不存在名稱的殘留項。名稱以作用中 change 逐字比對，不靠檔案系統探針。
- 引擎新增排序基底與拓樸修正的計算：先照 `board_rank`（缺 rank 者以建立日期補位、同值以名稱決斷），再以 `depends_on` 與 delta 重疊修正。另提供「rank 移動是否違反依賴」的檢查函式，供第二刀的桌面拖排接上；本刀不動桌面程式碼。
- 兩個新動詞於 remote 模式明確拒絕（FsOnly），第三刀再升為 Dual。
- 三個技能改寫並 bump ASSET_VERSION：propose 收尾改為「判定本次 change 的軟依賴 → 以 `speclink change depends` 寫入 → 跑 `speclink plan --json` 呈現波次」，不再每次重讀全部提案；apply 第一步一律先跑 `speclink plan --json`——未指名時取 `next`，指名且 `blockedBy` 非空時停止並列出前置；archive 收尾加一句「plan 顯示尚有未封存的前置時，建議先封存前置」，僅建議不阻擋。apply-with-worktree 與 apply 同源，自動帶到。影響 claude 與 codex 兩個工具的生成技能。
- 相容性影響：既有指令的人眼輸出與 `--json` 形狀皆不變；新增的是兩個新動詞與一個可缺席的 meta 欄位。技能生成的 golden 快照（claude、claude-worktree、codex、neutral-cli、neutral-tool-call 五份）與 assets.lock 於同批刻意更新。

## Capabilities

### New Capabilities

- `change-plan`: change 層的執行順序——排序基底、依賴與重疊的拓樸修正、`plan` 與 `change depends` 兩個動詞的契約、rank 移動的依賴檢查，以及 apply 與 archive 技能消費 plan 的規定。掃描過的鄰近規格：`board-card-order` 只管看板卡片的 rank 持久化，不算執行順序；`propose-skill` 的盤點條只規範代理人的口頭判定；`worktree-apply-skill` 只管單一 change 在 worktree 內的執行；三者都沒有「誰先做、誰可並行」的引擎契約。

### Modified Capabilities

- `propose-skill`: 「收尾盤點提案中變更的執行順序」改為判定後以 `change depends` 落檔、再以 `plan` 呈現，移除「不依賴引擎新增指令」的限制。
- `verb-contract`: 「模式分岔的單點宣告」的 FsOnly 清單加入 `plan` 與 `change`（並補上正典漏列的既有 FsOnly 動詞 `trace`）。
- `change-lifecycle`: 「meta 新欄位向後相容」補上 `depends_on` 缺席讀作無依賴、既有輸出逐位元不變。

## Impact

- Affected specs: `change-plan`（新增）、`propose-skill`、`verb-contract`、`change-lifecycle`（修改）
- Affected code:
  - New: crates/engine/speclink-core/src/lifecycle/plan.rs、crates/adapters/speclink-cli/src/verbs/plan.rs、crates/adapters/speclink-cli/tests/it/plan_verbs.rs
  - Modified: crates/engine/speclink-core/src/lifecycle/model.rs（ChangeMeta 的 depends_on 欄位與讀取器、階段判定）、crates/engine/speclink-core/src/lifecycle/mod.rs（模組註冊）、crates/engine/speclink-core/src/lifecycle/listing.rs（list --json 不變的回歸測試）、crates/engine/speclink-core/src/command/mod.rs 與 crates/engine/speclink-core/src/command/typed.rs（Plan 與 ChangeDepends 兩個 Command 與 outcome）、crates/adapters/speclink-cli/src/main.rs（兩個 FsOnly 動詞的宣告）、crates/adapters/speclink-cli/src/verbs/mod.rs、crates/adapters/speclink-cli/tests/it/main.rs、crates/adapters/speclink-cli/tests/it/mode_dispatch.rs、crates/engine/speclink-core/assets/skills/propose.md、crates/engine/speclink-core/assets/skills/apply.md、crates/engine/speclink-core/assets/skills/archive.md、crates/engine/speclink-core/src/workspace/init.rs（ASSET_VERSION）、crates/engine/speclink-core/tests/golden/assets.lock、crates/engine/speclink-core/tests/golden/claude.snapshot.md、crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md、crates/engine/speclink-core/tests/golden/codex.snapshot.md、crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md、crates/host/speclink-host/src/commit.rs（新增 DomainEvent 變體後 `event_record_of` 的窮舉 match 補臂）、docs/verb-contract.md 與 docs/verb-contract.zh-TW.md（模式表加入 plan 與 change）、由 speclink update 再生的 .claude/skills 與 .agents/skills 下的 SKILL.md
  - Removed: 無
- 不動的部分：apps/desktop（第二刀）、crates/speclink-server 與 crates/speclink-remote（第三刀）、`list --json` 與 `status --json` 的輸出形狀。
