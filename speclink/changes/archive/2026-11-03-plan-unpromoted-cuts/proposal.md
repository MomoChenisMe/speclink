## Why

透過 AI agent 開發的開發者在決定「接下來做什麼」時，要知道哪些變更在跑、第幾波、依賴誰，還要知道討論結論裡規劃好的刀有哪些還沒轉出成變更。前三項 `speclink plan` 已經給得出來；最後一項目前只有桌面 app 的討論詳情頁看得到，而且要一份一份點開，判定還是前端自己解析結論文字算的，CLI、Herdr 與之後的桌面路線圖都拿不到。討論 herdr-speclink-panel 定案把這份規劃資料放進引擎：本變更（第 2 刀）讓引擎解析結論裡的刀，並由 `plan` 一起列出尚未轉出的刀。使用情境是 workflow 中 discuss 結論之後、propose 之前的「挑下一刀」那一步，以及 apply、archive 技能讀 plan 的時候。

## What Changes

- speclink-core 新增結論分刀的解析：在討論結論的 Decision 欄位裡，以粗體開頭、粗體內以 `cut <編號>`、`第 N 刀` 或 `刀…` 起頭的頂層列點為一把刀；刀的變更名取粗體內的反引號片段，範圍句取粗體之後的說明——與桌面討論詳情頁現行的判定相同
- `plan` 的計算在既有結果之外多算一份「尚未轉出的刀」：對每份未封存討論的每一把刀，粗體內有變更名、且其中至少一個不在該討論的 `promoted_to` 時列出；依討論清單順序、同一討論依結論內的刀序
- `speclink plan --json` 新增頂層鍵 `unpromotedCuts`（陣列）：每項含 `discussion`（slug）、`cut`（刀編號原文，例如 `cut 3`）、`title`（範圍句）、`changes`（粗體內的全部變更名）、`unpromoted`（其中還不在 `promoted_to` 的變更名）；沒有時為 `[]`
- `speclink plan` 人眼輸出：有尚未轉出的刀時，在 Skipped 段之後、`next:` 行之前加一段 `Unpromoted cuts`，每把刀一行；沒有時不印這一段，既有輸出位元級不變
- remote：server 的 GET /plan 回應帶同一個 `unpromotedCuts`（由同一個引擎計算）；client 讀到缺席的鍵時視為空陣列（新 client 讀舊 server），轉回引擎報告時逐欄搬運
- 相容性影響：`plan --json` 只新增一個頂層鍵，既有四鍵與 changes 九鍵的名稱、型別、順序不變，只讀既有鍵的工具（apply、archive 技能、Claude Code 外掛的 `#` 選單）不受影響；人眼輸出只在有尚未轉出的刀時多一段；JSON 回歸對照中斷言「恰為四鍵」的測試要同批更新

## Non-Goals

- 不改桌面討論詳情頁：刀卡與「N 刀尚未轉出」照舊由前端解析，改用引擎資料是第 4 刀 `discussion-cuts-from-engine`
- 不推算刀與刀之間的依賴：結論的刀序不一定代表依賴，只標「屬於哪份討論的哪一刀」
- 不納入已封存的討論：封存後不能再轉出
- 不列出粗體內沒有變更名的刀（舊式寫法把名稱放在粗體之後）：無從判定是否已轉出，與桌面現行的「unknown」一致
- 不改 `discuss list --json`、`discuss show --json` 的輸出
- 不做桌面路線圖：第 3 刀 `desktop-roadmap`

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `change-plan`: plan 輸出新增 `unpromotedCuts` 鍵與人眼段落，並新增「plan 列出討論結論中尚未轉出的刀」需求（分刀的解析規則與列出條件）
- `server-verb-api`: GET /plan 回應帶 `unpromotedCuts`，與 CLI plan --json 同形
- `client-protocol`: plan 回應 payload 新增 `unpromotedCuts` 的型別定義、缺席讀作空陣列、轉回引擎報告時逐欄搬運

## Impact

- Affected specs: change-plan、server-verb-api、client-protocol
- Affected code:
  - New:
    - crates/engine/speclink-core/src/lifecycle/discuss/cuts.rs
  - Modified:
    - crates/engine/speclink-core/src/lifecycle/discuss.rs
    - crates/engine/speclink-core/src/lifecycle/plan.rs
    - crates/adapters/speclink-cli/src/verbs/plan.rs
    - crates/adapters/speclink-cli/tests/it/plan_verbs.rs
    - crates/adapters/speclink-cli/tests/it/remote_plan.rs
    - crates/protocol/speclink-protocol/src/query.rs
    - crates/protocol/speclink-remote/src/convert.rs
    - crates/protocol/speclink-remote/tests/it/typed_client.rs
    - crates/host/speclink-server/src/api/routes.rs
    - crates/host/speclink-server/tests/it/api/plan_api.rs
    - apps/desktop/src-tauri/src/remote.rs
    - apps/desktop/core/src/manage.rs
- 影響的 crate：speclink-core（解析與 plan 計算）、speclink-cli（plan 的人眼與 JSON 輸出）、speclink-protocol 與 speclink-remote（GET /plan 型別與轉換）、speclink-server（GET /plan 回應）；apps/desktop 只改兩處（plan 報告多一個欄位，桌面只讀既有欄位）：apps/desktop/core/src/manage.rs 的 `set_depends_at` 成環複檢改用只檢查成環的 `plan::check_cycle`，不必每次讀討論；apps/desktop/src-tauri/src/remote.rs 的測試 helper 補上 `PlanResponse` 新欄位，否則測試編譯失敗
- CLI：`speclink plan`，旗標、stdin、exit code 都不變；`--json` 多 `unpromotedCuts` 鍵，人眼輸出有資料時多一段；`--no-color` 下新段落同樣無 ANSI
- 設定：無
