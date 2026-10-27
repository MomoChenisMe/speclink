## Why

/speclink-discuss 開場偵察的規格段只讀已封存的正式規格（speclink list --specs --json），進行中變更帶的 delta 規格（openspec/changes/<name>/specs/<capability>/spec.md）完全不在偵察範圍；技能檔只在「Speclink Awareness」段跑 speclink list --json 列出變更名稱，而該 JSON 每筆只有 name／status／summary／completedTasks／totalTasks，沒有這個變更改了哪些 capability；speclink show <name> --json 雖帶 deltaSpecs 欄位，技能檔從未規定偵察時用它。結果是透過 AI 代理跑 SDD 的開發者、PO 與 PM 在開討論時，假設清單可能建立在一條正被某個進行中變更改掉的正式規格上，要到 propose 或 drift 階段才撞到。本 repo 此刻就有三個進行中變更、12 個 delta capability 目錄，偵察一個都看不到。

本變更承接討論 discuss-scout-in-flight-deltas 的結論：偵察順序不動，規格段擴大為「正式規格＋進行中 delta 規格」。討論結論原為零引擎改動；品質關卡的 review 指出逐一 show 每個變更的三動詞串接（possible Message Chains），使用者裁定一併修正，因此加一個小的引擎改動：speclink list --json 每筆變更帶 deltaCapabilities 選填欄位，技能檔改讀開場已跑的這支指令。

## What Changes

- **discuss 技能的偵察規格段納入進行中 delta 規格**（影響 speclink-core 的技能 asset；渲染至 claude 與 codex 兩個工具的 speclink-discuss 技能檔，事實來源 crates/engine/speclink-core/assets/skills/discuss.md）：
  - Step 2 的 Canon pass 命中 capability 名後，技能檔規定以開場已執行的 speclink list --json 各變更項的 deltaCapabilities 欄位與命中的 capability 名交集（不以 status 過濾、不逐一執行 speclink show）；交集即「進行中 delta 命中」，並記下該變更名。不以檔案路徑比對：正典 verb-contract 禁止技能檔指示直接開啟規格目錄的檔案（skill_verbization 守門測試）。
  - 時間盒：delta 命中以 speclink artifact cat specs/<capability> --change <name> 取內容，只讀 Requirement 標題與 ADDED／MODIFIED／REMOVED／RENAMED 標記，至多 3 份，不讀全文；變更在 worktree 進行時（清單項帶 worktree 物件）於該 worktree 路徑下執行，讀到的標題才與清單的命中同一副本；與既有「正式規格讀 Purpose 至多 3 份、原始碼至多 5 檔」並列。兩動詞皆為 Dual，remote 模式同樣可用。
  - 三段順序「正式規格 → 舊討論查核 → 程式碼」不變；delta 規格屬規格段的一部分，不另開一段。正式規格零命中或沒有進行中變更時 delta 比對跳過。
- **假設清單的對照標記**：既有四類對照不增列；「Covered by canon」與「Conflicts with canon」兩類在 delta 命中時附加標記「（進行中變更 <name> 將改動）」，讓使用者一眼看出該假設踩在別人正在改的地方。不得以此擋下討論方向。
- **Context 段**：不加新行；既有「related changes/specs」句的撰寫規定改為一併列出命中的 delta capability 與其變更名。`Prior discussions:` 行與 Context／Rounds／Conclusion 骨架不變。
- **speclink list --json 每筆變更帶 deltaCapabilities**（影響 speclink-core 的清單組裝、speclink-protocol 的 ChangeSummary、speclink-server 的 GET /changes、CLI 的 remote 清單映射）：有 delta 規格的變更帶該欄位（capability 名稱、升冪，值與 show --json 的 deltaSpecs 同源），沒有 delta 的變更省略；人眼輸出不變；remote 模式同形。
- **技能 asset 三連動**：ASSET_VERSION 升一個 minor 版、render golden 快照（claude、claude-worktree、codex、neutral-cli、neutral-tool-call）與 assets.lock 同批更新，speclink update 再生 .claude/skills 與 .agents/skills 下的 SKILL.md。

**相容性影響**：不新增或改動任何 CLI 指令、旗標、stdin 或 exit code。speclink list --json 對帶 delta 規格的變更多一個選填欄位 deltaCapabilities（只加不刪，既有消費者照常解析）；沒有 delta 的變更、speclink list --specs --json 與全部人眼輸出逐位元不變。wire 的 ChangeSummary 增同名選填欄位（serde default），舊 server 不送時 client 視為空；既有討論記錄格式不變、不需遷移。不涉及 openspec/config.yaml 與 .speclink.yaml 欄位。影響的生成技能：claude 與 codex 兩工具的 speclink-discuss；propose 與 improve 的偵察段不動。

## Non-Goals (optional)

見 design.md 的 Goals / Non-Goals。

## Capabilities

### New Capabilities

（無。掃描到的鄰近規格：discuss-skill 已承載偵察漏斗、時間盒與四類對照的全部要求，本變更是對它的修改；skill-routing 只管技能路由、不管偵察內容；propose-skill 的偵察段本題明確不動。）

### Modified Capabilities

- `discuss-skill`：修改「事實與決策分診及逐節點查證」——漏斗的規格段從「正式規格」擴為「正式規格＋進行中 delta 規格」，含 delta 比對規則與時間盒；修改「正式規格接地與三分對照」——delta 命中時兩類對照附加「進行中變更將改動」標記；新增要求——Context 的相關變更句列出命中的 delta capability。
- `change-lifecycle`：新增要求「list --json 曝變更的 delta capability」。
- `client-protocol`：新增要求「變更清單的 delta capability 欄位」。
- `server-verb-api`：新增要求「變更清單回應攜帶 delta capability 欄位」。

## Impact

- Affected specs: discuss-skill, change-lifecycle, client-protocol, server-verb-api
- Affected code:
  - Modified:
    - crates/engine/speclink-core/assets/skills/discuss.md（Step 2 Canon pass 的 delta 比對與時間盒、Canon triage 的標記規則、Context 範本的相關變更句）
    - crates/engine/speclink-core/src/workspace/init.rs（ASSET_VERSION）
    - crates/engine/speclink-core/src/lifecycle/listing.rs（ListChangeJson 增 deltaCapabilities）
    - crates/protocol/speclink-protocol/src/query.rs（ChangeSummary 增 deltaCapabilities 與序列化測試）
    - crates/host/speclink-server/src/api/routes.rs（GET /changes 清單項映射）
    - crates/host/speclink-server/tests/it/api/query_routes.rs（GET /changes 測試）
    - crates/adapters/speclink-cli/src/verbs/query.rs（remote 清單映射）
    - crates/adapters/speclink-cli/tests/it/remote_read_path.rs（list 的 fs／remote 同形測試）
    - crates/adapters/speclink-cli/tests/it/main.rs（登記新測試模組）
    - apps/desktop/src-tauri/src/remote.rs（測試 helper 的 ChangeSummary literal）
    - apps/desktop/src-tauri/tests/it/remote_data.rs（測試 helper 的 ChangeSummary literal）
    - crates/engine/speclink-core/tests/golden/claude.snapshot.md
    - crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md
    - crates/engine/speclink-core/tests/golden/codex.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md
    - crates/engine/speclink-core/tests/golden/assets.lock
    - 由 speclink update 再生的 .claude/skills 與 .agents/skills 下的 SKILL.md
  - New:
    - crates/adapters/speclink-cli/tests/it/list_delta_capabilities.rs（fs 模式 list --json 的 deltaCapabilities 與人眼輸出測試）
  - Removed: （無）
