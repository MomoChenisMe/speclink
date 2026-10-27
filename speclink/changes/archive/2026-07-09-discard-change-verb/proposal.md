## Why

移除 change 目前沒有動詞——只能手動刪目錄，繞過所有生命週期機制，留下三個相連的漏洞：死掉的變更名殘留在討論的 promoted_to、promoted 孤兒討論在看板上沒有收尾入口（GUI 僅 concluded 卡有封存動詞）、「砍掉另開」沒有正規流程。廢棄動詞讓刪除與討論側清理在同一步完成（源自討論 discard-change-verb 的結論）。

## What Changes

- 新增引擎頂層動詞 speclink discard，接受變更名（拼寫鏡射頂層 archive）：刪除變更目錄與該變更的 touched 紀錄（若存在）。
- 討論側解鏈：對變更 from_discussion 清單中的每份討論，自 promoted_to 移除該變更名；清單因此變空時移除 promoted_to 行並回退狀態——記錄有結論回 concluded、無結論回 open（link 允許 open 討論併入）。
- 守衛：變更有動工痕跡（meta 的 started_at 或 tasks.md 任何已勾任務）時拒絕，--force 放行；變更不存在時報錯。使用者確認砍 change 幾乎都在動工前；動工後異動走 discuss＋ingest 或 archive 後新開，守衛不會儀式化。
- 輸出：報告已刪除的變更與每份解鏈討論及其回退後狀態；remote store 模式下報「不支援」（鏡射 discuss discard 的既有行為）。
- 文件：README.md 與 README.en.md 的「指令參考——變更生命週期」表補 discard 一列，「SDD 工作流」節補「砍掉另開」流程一句。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `change-lifecycle`: 新增「變更以 discard 動詞廢棄」requirement——動詞行為、守衛與輸出。
- `discussion-docs`: 新增「討論隨變更廢棄解鏈」requirement——promoted_to 移除與狀態回退。

## Impact

- Affected specs: `change-lifecycle`（修改）、`discussion-docs`（修改）
- Affected code:
  - New:
    - `crates/speclink-core/src/discard.rs` — 廢棄流程（守衛、刪除、解鏈編排），鏡射 archive 的頂層動詞模組模式
  - Modified:
    - `crates/speclink-core/src/store.rs` — Store trait 新增變更刪除方法（沿 delete_live_discussion 先例）
    - `crates/speclink-core/src/discuss.rs` — 解鏈函式：promoted_to 移除該變更名與空清單時的狀態回退
    - `crates/speclink-fs/src/lib.rs` — 檔案系統 store 實作變更刪除
    - `crates/speclink-node/src/store_bridge.rs` — bridge store 對映新 trait 方法
    - `crates/speclink-cli/src/main.rs` — clap 子指令 discard 與 --force 旗標
    - `crates/speclink-cli/src/commands.rs` — handler 與人眼／--json 輸出
    - `crates/speclink-cli/src/remote_commands.rs` — remote 模式報不支援（鏡射 discuss discard）
    - `README.md`、`README.en.md` — 指令參考表與 SDD 工作流補述
  - Removed: （無）
- 實作順序相依：`rediscuss-promoted-change`（from_discussion 改累積器）須先實作——解鏈邏輯逐 slug 走累積器讀取。
- 不受影響：GUI（捨棄動詞比照討論 discard 排除於 GUI，屬 agent/CLI 領域）、`docs/verb-contract.md` 與 `docs/verb-contract.zh-TW.md`（remote 不支援故遠端契約不動）、內嵌技能資產與 render golden。
