## Why

add-change-plan-engine 把執行順序的判定放進引擎，apply 技能第 1 步以 `speclink plan --json` 選 change 並守門；apply-with-worktree 與 apply 本體同源，守門「自動帶到」了，但帶到的位置不對：本體第 1 步跑在 worktree 前段的產物 commit 與建 worktree 之後，而且依前段 P6 是在 worktree 內執行。linked worktree 的 .git 是檔案，引擎的 observed_facts 回空 overlay，worktree 內的 plan 只讀自己的快照——被擋的 change 會先留下空 worktree 與分支；續作舊 worktree 時主線已封存的前置在快照裡仍活著會假擋；未指名時 next 可能選到別的 worktree 正在做的 change；前段本身在沒給名字時也沒有任何選 change 的步驟。另一個缺口是 ingest：中途改需求可能讓本 change 新建立在別的提案成果上，但 ingest 收尾沒有 propose 收尾那段軟依賴判定，新依賴不落檔、apply 守門看不到。本變更是討論 worktree-apply-plan-preflight 的結論。目標使用者是透過 AI 代理跑 SDD 的開發者，對應 apply-with-worktree 的前置階段與 ingest 的收尾階段。

## What Changes

- apply-with-worktree 前段（crates/engine/speclink-core/assets/skills/apply-worktree-pre.md）在 P1 政策檢查之後新增「以 plan 選 change 並守門」步驟，在主 checkout 執行 `speclink plan --json`：未指名取 `next`（null 時列每個 change 的 `blockedBy` 並停）；指名者 `blockedBy` 非空即印前置清單並停，不 commit 產物、不建 worktree；名字不在 `changes`（已封存、打錯、列於 `skipped`）即停並列可用名單。原 P2 以 `speclink list --json` 做的存在性檢查併入此步。
- P0 多名字時先跑 `speclink plan --json`：`blockedBy` 為空的名字列成並行配方（各開 session 走 apply-with-worktree），非空的說明「等 X 落地再開」，使用者從並行名單擇一；恰有一個可並行時改為確認是否在此執行，沒有可並行者即停。
- 前段明寫：apply 本體第 1 步的 plan 在 worktree 內只是複查，結果與前段不同時以主 checkout 的判定為準，第 2 步（review prepare 與 in-progress add）照常執行；寫法沿後段「本體 Next steps 在此不適用」的先例。P2 標題點出主 checkout，與本體第 1 步區分。apply 本體（apply.md）位元不動，speclink-apply 的生成輸出除 frontmatter 的 version 行外不變。
- ingest 技能（crates/engine/speclink-core/assets/skills/ingest.md）在驗證通過後、給下一步建議之前新增「重判軟依賴」步驟：作用中 change 兩個以上時讀其他 change 的 proposal Impact，本 change 建立在某 change 成果上即對每個前置各執行一次 `speclink change depends <本 change> --on <前置>`（動詞對同一次呼叫的多個前置全寫或全不寫，逐一呼叫才不會因一個被拒而連帶遺失其他前置）；只動到同一段程式碼不記為前置（本 change 若已在進行中，等待尚未開工的 change 會卡住它），改向使用者提出重疊；只判本 change 的入邊、硬信號（delta 重疊）交引擎、僅落檔不代跑 apply。
- propose 技能（crates/engine/speclink-core/assets/skills/propose.md）收尾盤點的落檔段落改為與 ingest 逐字一致：每個前置各一次呼叫、拒絕時續行下一個前置且不重試；判定規則（建立在成果上或動到同一段程式碼）不變。兩份字面以 render_golden 的共用段落鎖定測試防止漂移，沿 archive 與 commit 共用段落的先例。
- ASSET_VERSION 自 v1.38.0 升為 v1.39.0（v1.38.0 已由先落地的 discuss-scout-in-flight-deltas 使用），claude／claude-worktree／codex／neutral-cli／neutral-tool-call 五份 golden 與 assets.lock 同批更新，`speclink update` 再生 SKILL.md。影響 claude 與 codex 兩個工具的 speclink-apply-with-worktree、speclink-ingest 與 speclink-propose 技能。
- 相容性影響：不動任何 CLI 指令與引擎行為；只有三份技能資產的文字與 golden 快照變更。

## Non-Goals

- 不改引擎讓 worktree 內的 plan 跳到主 checkout 聚合（會改 list 與 plan 共用的 observed_facts 閘門，違反「worktree 內只讀自己副本」的設計）。
- 不在 apply.md 本體加 worktree 條件分支（本體須與 apply 位元同源）。
- 不給 worktree-merge 加 Plan order hint（有依賴或重疊的配對在 apply 守門就已序列化）。
- 不讓 drift 重跑 plan（出邊 apply 已守門）。
- 不補 docs/workflow 兩語版與 README 流程圖的 plan 說明（使用者裁定文件整體過時，另案處理）。
- 不併入 add-change-plan-remote 或 discuss-scout-in-flight-deltas（兩者不碰這三份技能資產）。

## Capabilities

### New Capabilities

- `ingest-skill`: ingest 技能收尾重判本 change 的軟依賴並以 change depends 落檔。ingest 目前沒有自己的正典 capability：skill-routing 只記它的出邊（artifacts 更新完回 apply），change-lifecycle 只記 restale_from 的觀測，propose-skill 的軟依賴判定需求只約束 propose；ingest 的收尾行為沒有任何規格承載。

### Modified Capabilities

- `worktree-apply-skill`: 「apply-with-worktree 技能的前置指示」插入以 plan 選 change 並守門的步驟（在主 checkout 執行、建 worktree 之前）、改寫 (0) 多名字配方為依 plan 分可並行與要等、(2) 存在性檢查併入 plan 步驟，並加「本體第 1 步在 worktree 內為複查、以主 checkout 判定為準、第 2 步照常執行」的指示；(0) 另補恰一個可並行時改為確認。

## Impact

- Affected specs: `worktree-apply-skill`（修改）、`ingest-skill`（新增）
- Affected code:
  - New: 無
  - Modified: crates/engine/speclink-core/assets/skills/apply-worktree-pre.md、crates/engine/speclink-core/assets/skills/ingest.md、crates/engine/speclink-core/assets/skills/propose.md（落檔段落與 ingest 對齊）、crates/engine/speclink-core/src/workspace/init.rs（ASSET_VERSION）、crates/engine/speclink-core/tests/golden/assets.lock、crates/engine/speclink-core/tests/golden/claude.snapshot.md、crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md、crates/engine/speclink-core/tests/golden/codex.snapshot.md、crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md、crates/engine/speclink-core/tests/it/render_golden.rs（依規格場景新增渲染內容測試）、由 speclink update 再生的 .claude/skills 與 .agents/skills 下的 SKILL.md
  - Removed: 無
- 相關規格掃描：worktree-apply-skill（本變更修改）、change-plan（「apply 技能以 plan 挑選與守門」已要求兩份技能同含守門，不動；apply-with-worktree 的守門點在前段 P2，於主 checkout 執行，本體第 1 步在 worktree 內只是複查，不因複查結果停止）、skill-routing（交棒表 apply-with-worktree 與 ingest 兩列出邊不變，不動）、propose-skill（軟依賴判定規則的來源；ingest 只沿用「建立在成果上」一條與落檔段落；propose 落檔改為每個前置各一次呼叫，仍符合其「執行 change depends 落檔」的需求字面，不動）。
- 與在途變更的關係：add-change-plan-remote 的 delta 為 change-plan／client-protocol／desktop-app／remote-board-order／server-verb-api／verb-contract，discuss-scout-in-flight-deltas 的 delta 為 change-lifecycle／client-protocol／discuss-skill／server-verb-api；本變更只落 worktree-apply-skill 與 ingest-skill，零重疊，可並行。
