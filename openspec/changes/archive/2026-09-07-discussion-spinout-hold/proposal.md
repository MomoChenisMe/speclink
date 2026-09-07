## Why

一份討論可以分期轉出多個變更（例：improve-workspace-sync 的結論規劃刀 A、刀 B 兩刀，刀 B 要等刀 A 封存後才立案）。今天引擎在「討論已有結論、且無在途變更引用它」時就把討論隨行封存——這正是 conclusion-gated-discussion-archive 定下的規則，引擎沒做錯；缺口是「還欠一個尚未建立的變更」只存在於結論的 Deferred 散文，沒有任何機器可讀的訊號。結果是刀 A 封存時討論被收進封存區，刀 B 無從在原討論上 promote（promote 對封存記錄拒絕），只能手動搬檔救援。

目標使用者：透過 AI 代理跑 SDD 的開發者，情境是 discuss／improve 技能收斂後分期立案（先立一刀、封存後再回同一討論轉出下一刀）。

## What Changes

- **`speclink discuss conclude` 新增 `--hold` 旗標**：寫入結論的同時在討論記錄 frontmatter 寫入 `hold: true`，表示「本討論還欠一個尚未建立的變更」。不帶旗標時行為與輸出逐位元不變。帶旗標時人眼輸出多一行告知記錄保留在途；`--json` 增 `held: true`（僅旗標生效時出現）。
- **兩個自動封存點都守旗標**：變更封存的隨行封存（引擎 archive 的來源討論過濾）與 conclude 的閉環封存，在記錄帶 `hold: true` 時 SHALL NOT 封存討論，其餘條件照舊。
- **旗標由轉出清除**：`mark_promoted`（promote、`new change --from-discussion`、seal 三條路徑的共同寫入點）累加 `promoted_to` 時一併移除 `hold` 行。此後生命週期照舊——下一刀封存時討論隨行封存。
- **re-conclude 重述意圖**：不帶 `--hold` 的再次 conclude 清掉既有旗標；帶 `--hold` 的再次 conclude 保留旗標（第三刀情境）。
- **手動 `speclink discuss archive` 無視旗標**：明示動詞是放棄後續刀的出口，不需新動詞。
- **wire 與 server 同語意**：討論結論請求增選填 `hold` 布林（缺席即 false），回應增 `held` 布林（僅 true 時出鍵）；server 結論端點轉傳旗標並回填 `held`。CLI remote 模式的 `--hold` 經 wire 生效。
- **技能文字同步**：discuss 技能的中途轉出段與 improve 技能的扇出段明說——結論規劃「之後回本討論再轉出」時，conclude 必帶 `--hold`；否則後續刀走新討論。內嵌資產、repo 技能實例（claude 與 codex 兩工具）與 render golden 同批更新，ASSET_VERSION 隨之遞增。

### 相容性影響

- 不帶 `--hold` 的 conclude：人眼與 `--json` 輸出逐位元不變；既有討論記錄無 `hold` 行，兩個自動封存點的判斷結果與改動前完全相同。
- 帶 `--hold` 的 conclude：人眼輸出多一行、`--json` 多 `held: true` 鍵。
- wire：請求的 `hold` 缺席即 false，舊 client 打新 server 行為不變；新 client 帶 `hold: true` 打未升級的舊 server 時欄位被忽略、旗標靜默遺失——與其他選填請求欄位的既有取捨相同，記於 design。
- 技能檔：ASSET_VERSION 遞增波及 `.claude/skills/` 與 `.agents/skills/` 全部受管 SKILL.md 的版號行，需在同一提交以 `speclink update` 再生。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `discussion-docs`：新增「conclude 以 --hold 保留討論在途」requirement；「討論以 link 動詞併入既有變更」的隨行封存條件與「conclude 於全數轉出變更已封存時順手封存討論」的閉環條件各多一條「記錄未帶 hold」。
- `client-protocol`：討論結論請求型別增選填 `hold`、回應型別增 `held`。
- `server-verb-api`：討論結論端點轉傳 `hold` 並回填 `held`。
- `discuss-skill`：「中途轉出教學」補分期轉出時 conclude 帶 `--hold` 的指示。
- `improve-skill`：「candidates 以討論記錄承載」的扇出段補同一指示。

## Impact

- Affected specs: discussion-docs、client-protocol、server-verb-api、discuss-skill、improve-skill
- Affected code:
  - Modified:
    - crates/speclink-core/src/discuss.rs（hold 讀寫、conclude 帶旗標與閉環守門、mark_promoted 清旗標）
    - crates/speclink-core/src/archive.rs（隨行封存過濾多一條 hold 守門）
    - crates/speclink-core/src/command/mod.rs（DiscussConclude 命令增 hold、outcome 增 held）
    - crates/speclink-cli/src/verbs/discuss.rs（--hold 旗標、人眼與 --json 呈現、remote 分支轉傳）
    - crates/speclink-protocol/src/command.rs（ConcludeDiscussionRequest 增 hold、ConcludeDiscussionResponse 增 held）
    - crates/speclink-remote/src/client.rs（discussion_conclude 增 hold 參數）
    - crates/speclink-server/src/routes.rs（結論端點轉傳與回填）
    - crates/speclink-core/assets/skills/discuss.md（中途轉出段與 Lifecycle 段）
    - crates/speclink-core/assets/skills/improve.md（扇出段）
    - crates/speclink-core/src/init.rs（ASSET_VERSION 遞增）
    - crates/speclink-core/tests/golden/claude.snapshot.md、crates/speclink-core/tests/golden/claude-worktree.snapshot.md、crates/speclink-core/tests/golden/codex.snapshot.md、crates/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md、crates/speclink-core/tests/golden/assets.lock（再生）
    - .claude/skills/ 與 .agents/skills/ 下全部受管 SKILL.md（speclink update 再生版號行；speclink-discuss 與 speclink-improve 兩份另含內文變動）
    - crates/speclink-cli/tests/it/discuss_conclude_auto_archive.rs、crates/speclink-cli/tests/it/remote_write_path.rs、crates/speclink-server/tests/it/command_routes.rs（測試補齊）
  - New: (none)
  - Removed: (none)
- 影響的 crate／app：speclink-core、speclink-cli、speclink-protocol、speclink-remote、speclink-server。desktop 與 node 不動：desktop 不呼叫結論命令，討論留在途本身就是看板可觀察的結果。
