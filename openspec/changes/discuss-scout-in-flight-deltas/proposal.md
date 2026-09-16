## Why

/speclink-discuss 開場偵察的規格段只讀已封存的正式規格（speclink list --specs --json），進行中變更帶的 delta 規格（openspec/changes/<name>/specs/<capability>/spec.md）完全不在偵察範圍；技能檔只在「Speclink Awareness」段跑 speclink list --json 列出變更名稱，而該 JSON 每筆只有 name／status／summary／completedTasks／totalTasks，沒有這個變更改了哪些 capability；speclink show <name> --json 雖帶 deltaSpecs 欄位，技能檔從未規定偵察時用它。結果是透過 AI 代理跑 SDD 的開發者、PO 與 PM 在開討論時，假設清單可能建立在一條正被某個進行中變更改掉的正式規格上，要到 propose 或 drift 階段才撞到。本 repo 此刻就有三個進行中變更、12 個 delta capability 目錄，偵察一個都看不到。

本變更承接討論 discuss-scout-in-flight-deltas 的結論：偵察順序不動，規格段擴大為「正式規格＋進行中 delta 規格」，零引擎改動。

## What Changes

- **discuss 技能的偵察規格段納入進行中 delta 規格**（影響 speclink-core 的技能 asset；渲染至 claude 與 codex 兩個工具的 speclink-discuss 技能檔，事實來源 crates/engine/speclink-core/assets/skills/discuss.md）：
  - Step 2 的 Canon pass 命中 capability 名後，技能檔規定對 speclink list --json 回傳的每個進行中變更執行 speclink show <name> --json，只取其 deltaSpecs 欄位與命中的 capability 名交集；交集即「進行中 delta 命中」，變更名即該次 show 的 <name>。不以檔案路徑比對：正典 verb-contract 禁止技能檔指示直接開啟規格目錄的檔案（skill_verbization 守門測試）。
  - 時間盒：delta 命中以 speclink artifact cat specs/<capability> --change <name> 取內容，只讀 Requirement 標題與 ADDED／MODIFIED／REMOVED／RENAMED 標記，至多 3 份，不讀全文；與既有「正式規格讀 Purpose 至多 3 份、原始碼至多 5 檔」並列。兩動詞皆為 Dual，remote 模式同樣可用。
  - 三段順序「正式規格 → 舊討論查核 → 程式碼」不變；delta 規格屬規格段的一部分，不另開一段。正式規格零命中或沒有進行中變更時 delta 比對跳過。
- **假設清單的對照標記**：既有四類對照不增列；「Covered by canon」與「Conflicts with canon」兩類在 delta 命中時附加標記「（進行中變更 <name> 將改動）」，讓使用者一眼看出該假設踩在別人正在改的地方。不得以此擋下討論方向。
- **Context 段**：不加新行；既有「related changes/specs」句的撰寫規定改為一併列出命中的 delta capability 與其變更名。`Prior discussions:` 行與 Context／Rounds／Conclusion 骨架不變。
- **技能 asset 三連動**：ASSET_VERSION 升一個 minor 版、render golden 快照（claude、claude-worktree、codex、neutral-cli、neutral-tool-call）與 assets.lock 同批更新，speclink update 再生 .claude/skills 與 .agents/skills 下的 SKILL.md。

**相容性影響**：不新增或改動任何 CLI 指令、旗標、stdin 或 exit code；speclink list --json 與 speclink list --specs --json 的人眼與 --json 輸出逐位元不變，回歸對照不受影響；既有討論記錄格式不變、不需遷移。不涉及 openspec/config.yaml 與 .speclink.yaml 欄位。影響的生成技能：claude 與 codex 兩工具的 speclink-discuss；propose 與 improve 的偵察段不動。

## Non-Goals (optional)

見 design.md 的 Goals / Non-Goals。

## Capabilities

### New Capabilities

（無。掃描到的鄰近規格：discuss-skill 已承載偵察漏斗、時間盒與四類對照的全部要求，本變更是對它的修改；skill-routing 只管技能路由、不管偵察內容；propose-skill 的偵察段本題明確不動。）

### Modified Capabilities

- `discuss-skill`：修改「事實與決策分診及逐節點查證」——漏斗的規格段從「正式規格」擴為「正式規格＋進行中 delta 規格」，含路徑比對規則與時間盒；修改「正式規格接地與三分對照」——delta 命中時兩類對照附加「進行中變更將改動」標記；新增要求——Context 的相關變更句列出命中的 delta capability。

## Impact

- Affected specs: discuss-skill
- Affected code:
  - Modified:
    - crates/engine/speclink-core/assets/skills/discuss.md（Step 2 Canon pass 的 delta 比對與時間盒、Canon triage 的標記規則、Context 範本的相關變更句）
    - crates/engine/speclink-core/src/workspace/init.rs（ASSET_VERSION）
    - crates/engine/speclink-core/tests/golden/claude.snapshot.md
    - crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md
    - crates/engine/speclink-core/tests/golden/codex.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md
    - crates/engine/speclink-core/tests/golden/assets.lock
    - 由 speclink update 再生的 .claude/skills 與 .agents/skills 下的 SKILL.md
  - New: （無）
  - Removed: （無）
