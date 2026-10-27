## Why

討論記錄的定案——各輪的 Ruled out 與結論的 Decision／Rejected alternatives／Deferred——藏在記錄內文，封存已達 121 筆，卻沒有任何引擎動詞能依關鍵字把它們找出來。/speclink-discuss 開場只接續在途討論，封存記錄一律不讀；/speclink-improve 雖規定讀封存的 Ruled out，卻只能 list 之後逐筆 show。實測 drawer、golden、sse 三個關鍵字在 topic 命中 0 筆，在決定行分別命中 6、15、12 筆：只比 topic 找不到定案，而 remote 模式與 sqlite／postgres 後端下本機 grep 又碰不到記錄。結果是透過 AI 代理跑 SDD 的開發者、PO 與 PM 在開討論時，會把已否決或已延後的方向重新拿出來討論。

本變更承接討論 discuss-recall-archived-discussions 的結論：引擎提供 discuss search 動詞，discuss 技能開場加入舊討論查核，improve 技能的防重提檢查改用同一動詞。

## What Changes

- **新增引擎查詢動詞 `speclink discuss search`**（影響 speclink-core、speclink-cli）：
  - 子指令：`speclink discuss search <關鍵字>... [--json] [--no-color]`。位置參數至少一個關鍵字；不吃 stdin。
  - 比對：不分大小寫的子字串比對；多個關鍵字任一命中即算命中。比對範圍為記錄的 topic、slug，以及四種決定行——各輪的 `**Ruled out**` 行與 Conclusion 的 `**Decision**`、`**Rejected alternatives**`、`**Deferred**` 行。其他行（Evidence、Focus、Position、Open 與散文）不比對。
  - 範圍：預設在途與封存記錄皆搜，不設旗標。
  - 輸出：每筆命中回 slug、topic、status、archived、created、kind（記錄有才出）與命中行清單；每個命中行帶種類（topic／slug／ruled-out／decision／rejected／deferred）、所屬位置（輪號或 conclusion）與該行原文。排序：topic 或 slug 命中者排前，其餘依 created 由新到舊。
  - exit code：零命中 exit 0 且回空清單（--json 為空陣列）；未帶任何關鍵字 exit 非零並於 stderr 說明。
- **命令層覆蓋**（speclink-core）：新增 `Command::DiscussSearch` 與對應 `CommandOutcome`，覆蓋表加入 discuss search，CLI 本機模式經此層執行。
- **server 與 remote**（speclink-server、speclink-protocol、speclink-remote）：新增唯讀端點 `GET /discussions/search`，關鍵字以可重複的 `q` query 參數傳入，走同一 Command；protocol 新增搜尋回應型別；remote client 新增 typed 方法；CLI remote 模式輸出與本機同形。
- **discuss 技能**（claude 與 codex 兩工具，事實來源 crates/speclink-core/assets/skills/discuss.md）：偵察漏斗在正典掃描之後、程式碼掃描之前加入「舊討論查核」——以使用者題目的關鍵字與正典轉譯出的英文詞執行 discuss search；命中的決定行全數列出，整份 Conclusion 最多讀 3 份、topic 命中者優先。假設清單在既有三分對照之外加入第四類「舊討論已定案」，細分曾否決（附當時理由；重開須說明該理由已失效）、曾延後（可接手）、已落地（正典會照出，不重列）；不得以此擋下討論方向。Context 段加一行 `Prior discussions: <slug 清單>`。
- **improve 技能**（同兩工具，事實來源 crates/speclink-core/assets/skills/improve.md）：第一步防重提檢查改為執行 discuss search（關鍵字取自範圍），取代「list --archived 後逐筆 show」；同範圍的舊 improve 記錄排前閱讀。
- **技能 asset 三連動**：ASSET_VERSION 升版、render golden 快照與 assets.lock 同批更新。

**相容性影響**：既有 `speclink discuss list` 與 `speclink discuss show` 的人眼輸出與 `--json` 逐位元不變；新動詞與新端點皆為新增，不改既有回歸對照；desktop 與 Node SDK 不消費新型別（Node dispatch 現不覆蓋任何 discuss 動詞，維持不變）。不涉及 openspec/config.yaml 與 .speclink.yaml 欄位。

## Non-Goals (optional)

見 design.md 的 Goals / Non-Goals。

## Capabilities

### New Capabilities

（無。掃描到的鄰近規格：discussion-docs 已涵蓋討論動詞語意、server-read-api 已涵蓋 workspace 全文搜尋與已封存內容瀏覽、client-protocol 已涵蓋討論 payload 形狀、command-runtime 已涵蓋動詞覆蓋表；本變更全部落在既有 capability 的新增或修改要求。）

### Modified Capabilities

- `discussion-docs`：新增要求——discuss search 動詞的比對範圍、輸出形狀、排序、exit code，以及 remote 模式同語意。
- `command-runtime`：修改「動詞覆蓋與跨入口一致性」——查詢動詞覆蓋表加入 discuss search。
- `server-read-api`：新增要求——`GET /discussions/search` 唯讀端點，與既有 workspace 全文搜尋端點分工（後者只搜在途記錄且每卡回首個命中，語意綁桌面，不改）。
- `client-protocol`：新增要求——討論搜尋回應 payload 的欄位形狀。
- `discuss-skill`：新增要求——開場舊討論查核的位置、時間盒與第四類對照。
- `improve-skill`：修改「improve 技能以六步骨架渲染至兩工具」——防重提檢查改用 discuss search。

## Impact

- Affected specs: discussion-docs、command-runtime、server-read-api、client-protocol、discuss-skill、improve-skill
- Affected code:
  - Modified:
    - crates/speclink-core/src/discuss.rs（搜尋函式與命中型別）
    - crates/speclink-core/src/command/mod.rs（DiscussSearch 命令與結果）
    - crates/speclink-cli/src/verbs/discuss.rs（Search 子指令、人眼與 --json 輸出、remote 分支）
    - crates/speclink-protocol/src/query.rs（搜尋回應型別）
    - crates/speclink-remote/src/client.rs（typed 搜尋方法）
    - crates/speclink-server/src/app.rs（路由表）
    - crates/speclink-server/src/routes.rs（端點 handler）
    - crates/speclink-core/assets/skills/discuss.md（舊討論查核與第四類對照）
    - crates/speclink-core/assets/skills/improve.md（防重提改用 search）
    - crates/speclink-core/src/init.rs（ASSET_VERSION）
    - crates/speclink-core/tests/golden/claude.snapshot.md
    - crates/speclink-core/tests/golden/claude-worktree.snapshot.md
    - crates/speclink-core/tests/golden/codex.snapshot.md
    - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
    - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
    - crates/speclink-core/tests/golden/assets.lock
    - crates/speclink-server/tests/it/discussion_routes.rs（端點測試）
    - crates/speclink-remote/tests/it/typed_client.rs（typed 方法測試）
  - New:
    - crates/speclink-cli/tests/it/discuss_search.rs（CLI 動詞測試）
  - Removed: （無）
