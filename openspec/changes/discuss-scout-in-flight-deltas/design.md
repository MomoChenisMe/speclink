## Context

/speclink-discuss 的開場偵察是三段漏斗：正式規格（speclink list --specs --json，候選至多 5、讀 Purpose 至多 3）→ 舊討論查核（speclink discuss search）→ 程式碼（至多 5 檔）。技能檔另在「Speclink Awareness」段規定跑 speclink list --json 列出進行中變更，並把變更名寫進 Context 的「related changes/specs」句。

現況缺口：進行中變更的 delta 規格（openspec/changes/<name>/specs/<capability>/spec.md）不在任何一段被讀到。speclink list --json 每筆只有 name／status／summary／completedTasks／totalTasks，沒有 capability 清單，代理人無從得知某個正式規格正被哪個變更改動。實測本 repo 此刻三個進行中變更共 12 個 delta capability 目錄，偵察一個都看不到；假設清單因此可能建立在一條正被改掉的正式規格上，要到 propose 或 drift 才撞到。

限制：技能 asset 改動連動 ASSET_VERSION、render golden 五份快照與 assets.lock；既有 CLI 指令輸出與討論記錄格式不得變；同期進行中變更 add-plan-handoff-after-archive 也升 ASSET_VERSION（排定 v1.36.0 → v1.37.0），兩者合併時版號行會對撞。

利害關係人：透過 AI 代理跑 SDD 的開發者、PO 與 PM（開討論時看得到「有人正在改這條規格」）；引擎維護者（零引擎改動）。

## Goals / Non-Goals

**Goals:**

- 偵察的規格段看得到進行中 delta 規格：正式規格命中的 capability，若有進行中變更帶同名 delta，代理人在列假設前就知道。
- 順序不變：正式規格（含進行中 delta）→ 舊討論查核 → 程式碼。
- 零引擎改動、零 JSON 契約改動：純技能文字加 asset 三連動。

**Non-Goals:**

- 不改 speclink list --json 或 speclink list --specs --json 的輸出（不加 capability 欄位）。
- 不另開「第四段：進行中變更」；delta 規格屬規格段。
- 不在假設清單對照表加第五類。
- 不動 propose 與 improve 的偵察段（留待 discuss 這一刀落地後視實務再議）。
- 不為「引擎日後改 delta 目錄佈局會讓路徑假設靜默失效」設守門；屬已知風險，記於 Risks。
- 不改討論記錄格式；既有記錄不需遷移。

## Decisions

### D1 delta 命中以檔案路徑比對，不加引擎輸出

技能檔規定：Canon pass 命中 capability 名後，以 Glob 工具（或等價的檔案列舉）比對 openspec/changes/*/specs/<capability>/spec.md 是否存在；存在即為「進行中 delta 命中」，變更名取自路徑的 <name> 段。理由：delta 目錄本來就以 capability 命名，拿正式規格的命中名字對路徑就找得到；為一支技能的偵察改 speclink list --json 的 payload 是改對外契約，代價高於路徑比對。替代方案「引擎 list --json 加 capability 欄位」被否決：改 JSON 契約、動 protocol／server／remote 三層，只為省一次 Glob。替代方案「技能檔自己讀每個變更的 proposal Impact 段」被否決：Impact 是散文，比對不穩且讀量大。

已知邊界：remote 模式或資料庫後端下本機沒有 openspec/changes/ 目錄可比對，此時技能檔規定靜默跳過 delta 比對（與「正式規格零命中靜默略過」同一紀律）；本題不為 remote 另闢查詢路徑。

### D2 時間盒：只讀標題與標記，至多 3 份

delta 命中只讀該 delta 檔的 Requirement 標題（`### Requirement:` 行）與 ADDED／MODIFIED／REMOVED／RENAMED 區段標記，不讀全文，至多 3 份（超過取正式規格命中順序前 3 個 capability 的 delta）。理由：discuss-skill 規格「開場 scout 維持淺掃」要求偵察為接地與需求清晰度判定，不是調查；深入查證沿樹逐節點進行。替代方案「delta 全文讀」被否決：一份 delta 常有數百行，三份就超過原始碼 5 檔的量。

### D3 呈現：既有兩類對照附加標記，不加第五類

Canon triage 表維持四類；「Covered by canon」與「Conflicts with canon」兩類在 delta 命中時，該條假設的 Evidence 附加標記「（進行中變更 <name> 將改動）」，並列出該 delta 動到的 Requirement 名。技能檔重申不得以此擋下討論方向。理由：delta 是「即將成為正式規格的承諾」，性質仍屬規格對照；表格已從三類擴到四類，再加一類讓每條假設的分類負擔變重，而使用者真正要的只是「這條踩在別人正在改的地方」這一個訊號。替代方案「加第五類『進行中變更將改動』」被否決：與前兩類必然重疊（一條規格既被涵蓋又將被改），分類會二選一而丟資訊。

### D4 Context 段不加新行，擴寫既有相關變更句

Context 範本的「related changes/specs the scout surfaced」句改為規定同時列出命中的 delta capability 與其變更名（格式 `<name>: <capability>`，逗號分隔）；`Prior discussions:` 行、Source doc 慣例與 Context／Rounds／Conclusion 骨架不變。理由：這句已規定記進行中變更，加一段資訊即可；propose 目前沒有機械讀這句的行為，不需要新的機械標記行。替代方案「加一行 `In-flight deltas:`」被否決：多一個機械標記卻沒有消費者。

### D5 asset 三連動與版號對撞處理

ASSET_VERSION 自當時 main 的值升一個 minor 版；五份 golden 快照（claude、claude-worktree、codex、neutral-cli、neutral-tool-call）與 assets.lock 同批再生；在本 repo 跑 speclink update 再生 .claude/skills 與 .agents/skills 下的 SKILL.md 並以 git status 盤點納入同批 commit。同期 add-plan-handoff-after-archive 排定升 v1.37.0：本變更以該變更先落地為前提（landscape check 記為 depends_on），開工時以當時 main 的值再升一版（預期 v1.38.0）；若合併時版號行對撞，處理方式是重生衍生物（golden、assets.lock、SKILL.md），不是挑邊。

## Implementation Contract

**行為**（渲染後的 speclink-discuss 技能檔，claude 與 codex 兩工具）：

- Step 2 Canon pass 段落規定：命中 capability 名後比對 openspec/changes/*/specs/<capability>/spec.md；存在者列為「進行中 delta 命中」並記變更名；只讀 Requirement 標題與 ADDED／MODIFIED／REMOVED／RENAMED 標記、至多 3 份；正式規格零命中時跳過 delta 比對；本機沒有 openspec/changes/ 目錄（remote 模式）時靜默跳過。
- 三段順序描述仍為「正式規格 → 舊討論查核 → 程式碼」；規格段的措辭涵蓋正式規格與進行中 delta 規格，不出現第四段。
- Canon triage 表仍為四列；「Covered by canon」與「Conflicts with canon」兩列的做法欄規定 delta 命中時附加「（進行中變更 <name> 將改動）」標記與該 delta 動到的 Requirement 名，並含「不得以此擋下討論方向」字句。
- Context 範本的相關變更句規定列出 `<name>: <capability>` 形式的 delta 命中；`Prior discussions:` 行不變。

**具體例**：題目關鍵字命中正式規格 client-protocol；openspec/changes/add-change-plan-remote/specs/client-protocol/spec.md 存在 → 假設清單該條標為「Covered by canon（進行中變更 add-change-plan-remote 將改動：<該 delta 的 Requirement 名>）」；Context 的相關變更句含 `add-change-plan-remote: client-protocol`。

**介面／資料形狀**：無新增或改動的 CLI 指令、旗標、stdin、exit code、--json 欄位。改動的只有 crates/engine/speclink-core/assets/skills/discuss.md 的文字、ASSET_VERSION 常數值、五份 golden 快照與 assets.lock 的內容。

**失敗模式**：技能文字的指示不產生執行期錯誤；delta 目錄不存在、正式規格零命中、remote 模式無本機目錄三種情況都是靜默跳過，不提示、不阻斷。

**驗收**：

- `cargo test -p speclink-core --test it render_golden` 綠，五份快照含上述四項新內容。
- assets.lock 鎖定測試綠。
- 內容審閱：渲染後的 claude 與 codex 技能檔逐條對照 discuss-skill delta 規格各 scenario 的 THEN。
- 既有 `cargo test -p speclink-cli --test it discuss` 全綠（CLI 輸出未動）。

**範圍**：in scope＝discuss.md 的 Step 2、Canon triage、Context 範本三處文字，asset 三連動，SKILL.md 再生。out of scope＝引擎程式碼、propose.md、improve.md、討論記錄格式、desktop。

## Risks / Trade-offs

- **回歸對照**：五份 golden 快照與 assets.lock 必須同批再生，漏一份 render_golden 就紅；CLI 測試不受影響（無指令改動）。緩解：tasks 明列五份檔名與驗證指令。
- **版號行對撞**：add-plan-handoff-after-archive 同期升 ASSET_VERSION；兩者不同順序合併時 init.rs 的版號行與 assets.lock 會衝突。緩解：depends_on 記錄先後；對撞時重生衍生物。
- **路徑假設靜默失效**：引擎日後改 delta 目錄佈局（例如 specs 改名），技能檔的 Glob 比對會靜默零命中，行為退回今天的樣子。接受此風險，不設守門；archive-merge 與 spec-validation 規格已把 delta 路徑釘為 changes/<name>/specs/<capability>/spec.md，改佈局本身就是一個 change，屆時 grep 技能 asset 即可找到。
- **跨平台**：Glob 比對由代理人工具執行，路徑一律用正斜線寫在技能文字中；Windows 下代理人工具自行處理分隔符，技能文字不涉及 OS 路徑拼接。
- **remote 模式看不到 delta**：技能靜默跳過，行為與今天相同，不比今天差。
