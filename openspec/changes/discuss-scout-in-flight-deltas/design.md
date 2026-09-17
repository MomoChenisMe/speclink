## Context

/speclink-discuss 的開場偵察是三段漏斗：正式規格（speclink list --specs --json，候選至多 5、讀 Purpose 至多 3）→ 舊討論查核（speclink discuss search）→ 程式碼（至多 5 檔）。技能檔另在「Speclink Awareness」段規定跑 speclink list --json 列出進行中變更，並把變更名寫進 Context 的「related changes/specs」句。

現況缺口：進行中變更的 delta 規格（openspec/changes/<name>/specs/<capability>/spec.md）不在任何一段被讀到。speclink list --json 每筆只有 name／status／summary／completedTasks／totalTasks，沒有 capability 清單；speclink show <name> --json 雖帶 deltaSpecs 欄位，技能檔從未規定偵察時用它，代理人因此無從得知某個正式規格正被哪個變更改動。實測本 repo 此刻三個進行中變更共 12 個 delta capability 目錄，偵察一個都看不到；假設清單因此可能建立在一條正被改掉的正式規格上，要到 propose 或 drift 才撞到。

限制：技能 asset 改動連動 ASSET_VERSION、render golden 五份快照與 assets.lock；既有 CLI 指令的人眼輸出、既有 --json 欄位與討論記錄格式不得變（只許加選填欄位）；CLI list 的 fs 與 remote 共用渲染、輸出須逐位元一致，wire 欄位要兩模式同形；正典 verb-contract「技能資產不含直接讀檔指示」禁止技能檔指示直接開啟規格目錄的檔案路徑（守門：speclink-core 的 skill_verbization 測試），文件閱讀一律以 speclink 動詞表述；同期進行中變更 add-plan-handoff-after-archive 也升 ASSET_VERSION（排定 v1.36.0 → v1.37.0），兩者合併時版號行會對撞。

利害關係人：透過 AI 代理跑 SDD 的開發者、PO 與 PM（開討論時看得到「有人正在改這條規格」）；引擎維護者（清單加一個選填欄位）。

## Goals / Non-Goals

**Goals:**

- 偵察的規格段看得到進行中 delta 規格：正式規格命中的 capability，若有進行中變更帶同名 delta，代理人在列假設前就知道。
- 順序不變：正式規格（含進行中 delta）→ 舊討論查核 → 程式碼。
- 引擎只加一個選填欄位：speclink list --json 每筆變更帶 deltaCapabilities，技能檔從開場已跑的清單直接比對，不再逐一 show；其餘為技能文字加 asset 三連動。

**Non-Goals:**

- 不改 speclink list --specs --json 的輸出；不改任何人眼輸出；speclink list --json 只加選填欄位、不改既有欄位。
- 不另開「第四段：進行中變更」；delta 規格屬規格段。
- 不在假設清單對照表加第五類。
- 不動 propose 與 improve 的偵察段（留待 discuss 這一刀落地後視實務再議）。
- 不改討論記錄格式；既有記錄不需遷移。

## Decisions

### D1 delta 命中以 speclink list --json 的 deltaCapabilities 比對

引擎在 speclink list --json 每筆變更加選填欄位 deltaCapabilities：該變更 delta 規格的 capability 名稱，升冪，值取自 Store 的 delta_capabilities（沒有 worktree 映射時與 show --json 的 deltaSpecs 同源），沒有 delta 時省略。技能檔規定：Canon pass 命中 capability 名後，以開場「Speclink Awareness」已執行的 speclink list --json 各變更項的 deltaCapabilities 與命中的 capability 名取交集；交集即「進行中 delta 命中」，變更名即該項的 name。理由：比對所需的「變更 → capability」對照一次到手，不必逐一呼叫；欄位名沿用 wire 既有的 ChangeStatus.deltaCapabilities 與 plan 的 overlaps.capabilities（皆為 capability 名），不沿用 show --json 為 OpenSpec 相容保留的 `<capability>/spec.md` 形狀。

決策演進：propose 期原定零引擎改動，以 Glob 比對變更目錄下的 delta 路徑；apply 期踩到 skill_verbization 守門，改為逐一執行 speclink show <name> --json 只取 deltaSpecs；品質關卡的 review 將「list → 每個變更 show → 命中再 artifact cat」記為 possible Message Chains（一個事實串三個動詞、大半 payload 丟棄），使用者裁定修正，改為本決策。替代方案「維持逐一 show」被否決：N 個變更要 N 次呼叫，每次 payload 帶 proposal／design／tasks 全文卻只用一個欄位。替代方案「list --specs --json 的每個 capability 帶進行中變更名」被否決：規格清單要反查全部變更，把變更的屬性掛到規格上；變更清單本來就逐筆讀變更。替代方案「Glob 比對 delta 路徑」被否決：違反 verb-contract（skill_verbization 守門），且 remote 模式沒有本機目錄。替代方案「技能檔自己讀每個變更的 proposal Impact 段」被否決：Impact 是散文，比對不穩且讀量大。

已知邊界：speclink list --json 回傳全部未封存變更、含 status 為 done 者；done 變更的 delta 同樣尚未併入正式規格，故不以 status 過濾。remote 模式走同一支動詞，值取自 GET /changes 清單項的 deltaCapabilities；舊 server 不送時欄位缺席，技能檔視同沒有 delta、靜默跳過（接受：server 與 CLI 同版發行）。worktree 觀察面生效時，覆蓋層的 Store 把 delta_capabilities 委派給 worktree 副本，值與任務計數同源；speclink show 與 speclink artifact cat 則只讀主副本。因此技能檔規定：清單項帶 worktree 物件時，到該 worktree.path 下執行 artifact cat，讓 Requirement 標題與清單的命中來自同一副本（review 第三輪指出：否則只存在 worktree 的 delta 會被算成命中卻讀不到，兩邊都有的 delta 會讀到主副本的舊標題）。替代方案「讓 show 與 artifact cat 也走覆蓋層」被否決：改變兩支讀取動詞在主 checkout 的行為，範圍超出本變更。

### D2 時間盒：只讀標題與標記，至多 3 份

delta 命中以 speclink artifact cat specs/<capability> --change <name> 取內容，只保留 Requirement 標題（`### Requirement:` 行）與 ADDED／MODIFIED／REMOVED／RENAMED 區段標記，不讀全文，至多 3 份（超過時依候選順序取前 3 個命中：Canon pass 的候選依名稱命中關鍵字數多者優先、同數依 speclink list --specs --json 的列出順序；同一 capability 有多個變更時依 speclink list --json 的回傳順序）；超出時間盒的命中沒讀過，標記不帶 Requirement 名（見 D3）；artifact cat 為 Dual 動詞，remote 模式同樣可用。順序必須明定：review 指出若只寫「正式規格命中順序」而技能檔未定義，命中超過 3 份時不同代理人會取到不同的 delta。理由：discuss-skill 規格「開場 scout 維持淺掃」要求偵察為接地與需求清晰度判定，不是調查；深入查證沿樹逐節點進行。替代方案「delta 全文讀」被否決：一份 delta 常有數百行，三份就超過原始碼 5 檔的量。

### D3 呈現：既有兩類對照附加標記，不加第五類

Canon triage 表維持四類；「Covered by canon」與「Conflicts with canon」兩類在 delta 命中時，在對照類別標籤後附加標記「（進行中變更 <name> 將改動：<Requirement 名>）」——每個變更一個標記，Requirement 名為自該 delta 讀到的標題、逗號分隔；超出時間盒未讀的命中寫「（進行中變更 <name> 將改動）」，不猜 Requirement 名——Evidence 併列正式規格與該 delta。標記格式只在技能檔的 In-flight marker 段定義一次，表格兩列指回該段。技能檔重申不得以此擋下討論方向。理由：delta 是「即將成為正式規格的承諾」，性質仍屬規格對照；表格已從三類擴到四類，再加一類讓每條假設的分類負擔變重，而使用者真正要的只是「這條踩在別人正在改的地方」這一個訊號。替代方案「加第五類『進行中變更將改動』」被否決：與前兩類必然重疊（一條規格既被涵蓋又將被改），分類會二選一而丟資訊。

### D4 Context 段不加新行，擴寫既有相關變更句

Context 範本的「related changes/specs the scout surfaced」句改為規定同時列出命中的 delta capability 與其變更名（格式 `<name>: <capability>`，逗號分隔）；`Prior discussions:` 行、Source doc 慣例與 Context／Rounds／Conclusion 骨架不變。理由：這句已規定記進行中變更，加一段資訊即可；propose 目前沒有機械讀這句的行為，不需要新的機械標記行。替代方案「加一行 `In-flight deltas:`」被否決：多一個機械標記卻沒有消費者。

### D5 asset 三連動與版號對撞處理

ASSET_VERSION 自當時 main 的值升一個 minor 版；五份 golden 快照（claude、claude-worktree、codex、neutral-cli、neutral-tool-call）與 assets.lock 同批再生；在本 repo 跑 speclink update 再生 .claude/skills 與 .agents/skills 下的 SKILL.md 並以 git status 盤點納入同批 commit。同期 add-plan-handoff-after-archive 排定升 v1.37.0：本變更以該變更先落地為前提（landscape check 記為 depends_on），開工時以當時 main 的值再升一版（預期 v1.38.0）；若合併時版號行對撞，處理方式是重生衍生物（golden、assets.lock、SKILL.md），不是挑邊。

## Implementation Contract

**行為**（渲染後的 speclink-discuss 技能檔，claude 與 codex 兩工具）：

- Step 2 Canon pass 段落規定：候選依名稱命中關鍵字數多者優先、同數依指令列出順序；命中 capability 名後以開場已執行的 speclink list --json 各變更項的 deltaCapabilities 比對（不以 status 過濾、不逐一執行 speclink show）；含該 capability 者列為「進行中 delta 命中」並記變更名；以 speclink artifact cat specs/<capability> --change <name> 只讀 Requirement 標題與 ADDED／MODIFIED／REMOVED／RENAMED 標記、至多 3 份且依候選順序取前 3 個命中，清單項帶 worktree 物件時於其 worktree.path 下執行 artifact cat；正式規格零命中、沒有進行中變更或沒有變更的 deltaCapabilities 含命中的 capability 時靜默跳過；技能文字不含直接開啟 openspec/changes/ 路徑的讀檔指示（skill_verbization 守門綠）。
- 三段順序描述仍為「正式規格 → 舊討論查核 → 程式碼」；規格段的措辭涵蓋正式規格與進行中 delta 規格，不出現第四段。
- Canon triage 表仍為四列；「Covered by canon」與「Conflicts with canon」兩列的做法欄規定 delta 命中時加上進行中標記；表下 In-flight marker 段定義標記：寫在對照類別標籤後、帶該 delta 動到的 Requirement 名（逗號分隔）、超出時間盒未讀的命中不帶 Requirement 名、Evidence 併列正式規格與該 delta，並含「不得以此擋下討論方向」字句。
- Context 範本的相關變更句規定列出 `<name>: <capability>` 形式的 delta 命中；`Prior discussions:` 行不變。
- speclink list --json：帶 delta 規格的變更項含 deltaCapabilities（capability 名稱字串陣列、升冪），沒有 delta 的變更項無此鍵；人眼輸出不變；remote 模式同形（取自 GET /changes 清單項）。

**具體例**：題目關鍵字命中正式規格 client-protocol；speclink list --json 中 add-change-plan-remote 的 deltaCapabilities 含 client-protocol → 假設清單該條標為「Covered by canon（進行中變更 add-change-plan-remote 將改動：<該 delta 的 Requirement 名>）」；Context 的相關變更句含 `add-change-plan-remote: client-protocol`。

**介面／資料形狀**：無新增或改動的 CLI 指令、旗標、stdin、exit code。speclink list --json 的變更項增選填鍵 deltaCapabilities（string[]，空則省略）；speclink-core 的 ListChangeJson 增 delta_capabilities（serde rename deltaCapabilities、空清單省略）；speclink-protocol 的 ChangeSummary 增 delta_capabilities（camelCase、serde default、空清單省略）。其餘改動為 discuss.md 文字、ASSET_VERSION 常數值、五份 golden 快照與 assets.lock 的內容。

**失敗模式**：技能文字的指示不產生執行期錯誤；正式規格零命中、沒有進行中變更、沒有變更的 deltaCapabilities 含命中的 capability 三種情況都是靜默跳過，不提示、不阻斷；命中超過時間盒時，未讀的 delta 標記不帶 Requirement 名，不猜；remote 模式走同一組 Dual 動詞，行為與 fs 模式一致。清單組裝讀不到某變更的 delta 目錄時，該變更視為沒有 delta（欄位省略），清單不失敗；舊 server 不送 deltaCapabilities 時同樣視為沒有 delta。

**驗收**：

- `cargo test -p speclink-core --test it render_golden` 綠，五份快照含上述四項新內容。
- assets.lock 鎖定測試綠。
- `cargo test -p speclink-core --test it skill_verbization` 綠（技能文字無直接讀檔指示）。
- 內容審閱：渲染後的 claude 與 codex 技能檔逐條對照 discuss-skill delta 規格各 scenario 的 THEN。
- `cargo test -p speclink-cli --test it` 全綠（含 list --json 的 deltaCapabilities 與 remote 同形測試、既有 discuss 系列）。
- `cargo test -p speclink-protocol` 綠（ChangeSummary 的 deltaCapabilities 序列化、空值省略、舊 payload 反序列化）。
- `cargo test -p speclink-server --test it` 綠（GET /changes 清單項帶 deltaCapabilities、沒有 delta 者無此鍵）。
- `cargo test -p speclink-desktop` 綠（protocol struct 加欄位後 desktop 的 struct literal 仍編譯）。
- Node SDK 的 vitest 綠（`crates/adapters/speclink-node`：SDK 與 CLI 的 list --json 逐位元比對，兩邊共用 listing.rs）。

**範圍**：in scope＝discuss.md 的 Step 2、Canon triage、Context 範本三處文字，asset 三連動，SKILL.md 再生；speclink list --json 的 deltaCapabilities 選填欄位（core 清單組裝、protocol ChangeSummary、server GET /changes、CLI remote 映射，desktop 測試 helper 的 struct literal 跟著補欄位）。out of scope＝list --specs --json、任何人眼輸出、propose.md、improve.md、討論記錄格式、desktop 與 server-web 的前端呈現。

## Risks / Trade-offs

- **回歸對照**：五份 golden 快照與 assets.lock 必須同批再生，漏一份 render_golden 就紅；list --json 的既有回歸對照只在夾具帶 delta 規格時多出 deltaCapabilities 鍵（如 remote 同形測試的 fs 雙胞胎帶 cap-a），這類期望值同批更新。緩解：tasks 明列五份檔名與驗證指令。
- **版號行對撞**：add-plan-handoff-after-archive 同期升 ASSET_VERSION；兩者不同順序合併時 init.rs 的版號行與 assets.lock 會衝突。緩解：depends_on 記錄先後；對撞時重生衍生物。
- **verb-contract 守門**：技能文字若出現 read／glob／open 與規格目錄路徑同一行，skill_verbization 測試即紅；本變更全走動詞，守門本身就是回歸保護，收尾任務的 `cargo test -p speclink-core --test it` 含它。
- **與 add-change-plan-remote 重疊**：本變更新增 client-protocol 與 server-verb-api 兩份 delta，與 add-change-plan-remote 共用 capability；plan 以互斥處理，本變更 stage 為 ready、排序在前，對方在本變更封存前被擋。兩邊的 delta 皆只用 ADDED、Requirement 名不同，封存合併不互撞；程式碼層 ChangeSummary 結構可能有文字合併衝突，處理方式是保留兩邊欄位。
- **desktop 編譯**：ChangeSummary 加欄位會讓 apps/desktop/src-tauri 內以 struct literal 建構的測試 helper 編譯失敗（E0063）。緩解：同一任務補欄位並跑 cargo test -p speclink-desktop。
- **跨平台**：不涉及路徑比對與 OS 路徑拼接，動詞輸出三平台同形。
- **remote 模式**：list 與 artifact cat 皆為 Dual 動詞；list 的 deltaCapabilities 取自 GET /changes 清單項，fs 與 remote 共用渲染、輸出逐位元一致；舊 server 不送時欄位缺席，技能檔視同沒有 delta。
