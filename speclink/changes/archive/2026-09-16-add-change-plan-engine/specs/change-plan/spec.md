## Purpose

change 層的執行順序：以看板排序鍵與建立日期為基底、以宣告依賴與 delta capability 重疊做拓樸修正，算出波次（同波可並行）與每個 change 的阻擋清單；涵蓋 `plan` 查詢動詞、`change depends` 寫入動詞、rank 移動的依賴檢查，以及 apply 與 archive 技能消費 plan 的規定。看板卡片 rank 的持久化屬 board-card-order，任務層並行不在此範圍。

## ADDED Requirements

### Requirement: 執行順序的基底與拓樸修正

引擎 SHALL 對所有未封存的 change 算出一個全序的配置順序與波次。基底順序 SHALL 依三鍵排列：(1) 階段——已就緒（任務全勾且總數大於零）在前、進行中（有 started_at 或任一已勾任務）次之、提案中最後；(2) 同階段內有 board_rank 者排前、以位元組字典序升冪，缺 board_rank 者排後、彼此以 created 升冪、缺 created 者殿後；(3) 同值以 change 名字典序決斷。拓樸修正 SHALL 沿基底順序反覆配置「所有作用中宣告前置皆已配置」的第一個 change；宣告前置 SHALL 只計 depends_on 中仍未封存且 metadata 可解析的名稱，指到不存在、已封存或 metadata 壞掉（列入 skipped）的名稱 SHALL 視為已滿足。change 的波次 SHALL 為 1 加上其作用中宣告前置與已配置且 delta capability 重疊的 change 的最大波次；無前置無重疊者 SHALL 為第 1 波。delta 重疊 SHALL NOT 構成有向邊，重疊的兩個 change 的先後 SHALL 由基底順序決定。壞 metadata 的 change SHALL 排除於配置與重疊判定之外並列入 skipped。

#### Scenario: 無依賴無重疊時全部同波

- **WHEN** 三個提案中 change 皆無 depends_on、delta 目錄互不重疊
- **THEN** 三者皆為第 1 波，配置順序依基底順序

#### Scenario: 宣告依賴推後波次

- **WHEN** change c 的 depends_on 含 a，a 為提案中且未封存
- **THEN** c 的波次為 a 的波次加 1，c 的 blockedBy 含 a

#### Scenario: 重疊互斥依基底順序定先後

- **WHEN** change a 與 b 皆含 specs/desktop-app/ 的 delta，基底順序 a 在 b 前，兩者皆無 depends_on
- **THEN** a 為第 1 波、b 為第 2 波，b 的 blockedBy 為 [a]，a 的 blockedBy 為空

#### Scenario: 指向已封存的依賴視為滿足

- **WHEN** change c 的 depends_on 含已封存的 old-change，其餘無前置
- **THEN** c 為第 1 波，blockedBy 為空，dependsOn 仍原文列出 old-change

#### Scenario: 已開工者排在未開工者之前

- **WHEN** change a 為進行中、b 為提案中，兩者重疊且 b 的 created 早於 a
- **THEN** a 配置在 b 之前，b 的 blockedBy 為 [a]

##### Example: 五個 change 的基底順序與波次

| change | 階段 | board_rank | created | depends_on | delta | 基底序 | 波次 | blockedBy |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| r1 | 已就緒 | 無 | 2026-09-01 | 無 | tray-status-menu | 1 | 1 | 無 |
| p2 | 進行中 | n | 2026-09-03 | 無 | desktop-app | 2 | 1 | 無 |
| n5 | 提案中 | f | 2026-09-02 | 無 | archive-skill | 3 | 1 | 無 |
| n3 | 提案中 | 無 | 2026-09-10 | 無 | desktop-app | 4 | 2 | p2 |
| n4 | 提案中 | 無 | 2026-09-12 | n3 | discuss-skill | 5 | 3 | n3 |

- **GIVEN** 上表五個未封存 change
- **WHEN** 引擎計算 plan
- **THEN** waves 為 [[r1, p2, n5], [n3], [n4]]，next 為 n5（第一個提案中且 blockedBy 為空者）

### Requirement: plan 動詞輸出波次與阻擋清單

`speclink plan` SHALL 為唯讀查詢。`--json` 時 stdout SHALL 為單一 JSON 物件：`waves`（陣列，每項含 `index` 整數與 `changes` 字串陣列，陣列內為該波配置順序）、`changes`（陣列，依整體配置順序，每項含 `name` 字串、`wave` 整數、`stage` 字串三值 proposed／in-progress／ready、`dependsOn` 字串陣列、`overlaps` 陣列（每項 `change` 字串與 `capabilities` 字串陣列升冪）、`blockedBy` 字串陣列、`ready` 布林）、`next`（字串或 null）、`skipped`（陣列，每項 `change` 與 `reason` 字串，空時為 `[]`）。人眼輸出 SHALL 逐波印 `Wave N` 標題（同波兩個以上附 `(parallel)`）、每個 change 一行含名稱與階段、有阻擋時附 `blocked by: …`，末行 `next: <name>` 或 `next: none`；`--no-color` 下 SHALL 無 ANSI 序列。無作用中 change 時 `waves` 與 `changes` SHALL 為空陣列、`next` 為 null。depends_on 成環時 SHALL 以非零 exit code 結束，stderr 為 `dependency cycle: <a> -> <b> -> … -> <a>`，stdout SHALL 為空。在主 checkout 且 worktree 政策開啟時，plan 的階段判定 SHALL 讀取各 worktree 內的開工標記與任務進度，與 list 動詞同一聚合面。remote 模式下的臂 SHALL 依 verb-contract「模式分岔的單點宣告」對 plan 的歸屬決定。

#### Scenario: JSON 輸出形狀

- **WHEN** 於含兩個提案中 change 的 fs 專案執行 speclink plan --json
- **THEN** stdout 為含 waves、changes、next、skipped 四鍵的物件，changes 每項含 name、wave、stage、dependsOn、overlaps、blockedBy、ready 七鍵，exit code 為 0

#### Scenario: 人眼輸出與 --no-color

- **WHEN** 執行 speclink plan --no-color
- **THEN** stdout 含 `Wave 1` 行與 `next:` 行，不含 ANSI 序列，exit code 為 0

#### Scenario: 依賴成環

- **WHEN** change a 的 depends_on 含 b、b 的 depends_on 含 a，執行 speclink plan --json
- **THEN** exit code 非零，stderr 含 `dependency cycle: a -> b -> a`，stdout 為空

#### Scenario: 壞 metadata 列入 skipped

- **WHEN** 某 change 的 .openspec.yaml 無法解析，執行 speclink plan --json
- **THEN** 該 change 不在 waves 與 changes 內，skipped 含其名稱與原因，其餘 change 照常配置，exit code 為 0

### Requirement: change depends 動詞寫入宣告依賴

`speclink change depends <name> --on <other>... [--remove] [--json]` SHALL 把前置寫進該 change 的 `.openspec.yaml` 頂層 `depends_on` 欄位（逗號清單，以 `, ` 串接），其餘位元組逐字保留、每行沿用自身行尾。守門 SHALL 依序：`<name>` 非作用中 change 即錯；非 `--remove` 時任一 `<other>` 已封存或不存在即錯（訊息分辨兩者），`--remove` 時 SHALL 允許移除指向已封存或不存在名稱的殘留項、只拒絕不合法名稱（空白、含路徑分隔符或 `..`）；`<other>` 等於 `<name>` 即錯；非 `--remove` 時加邊後若有經過 `<name>` 的環即錯並印環（自 `<name>` 起讀），別處既有的環 SHALL NOT 阻擋、由 plan 回報。作用中 change 的判定 SHALL 以 change 名逐字比對，SHALL NOT 依賴檔案系統的存在探針（大小寫不分的檔案系統、`archive` 目錄與空字串皆不得放行）。任一守門失敗 SHALL 以非零 exit code 結束且零寫入。多個 `--on` SHALL 於單次寫入完成；加已存在或移除不存在的邊 SHALL 不改檔且成功。`--remove` 後清單為空時 SHALL 整行移除。成功時 stdout SHALL 為一行 `✓ <name> depends on: …`（`--remove` 時 `✓ <name> no longer depends on: …`）；`--json` 時 stdout SHALL 為 `{ "change": string, "dependsOn": string[] }`（寫入後的完整清單）。壞 metadata 的 `<name>` SHALL 拒絕寫入。在主 checkout 且 worktree 政策開啟時，`<name>` 已映射到 worktree 者 SHALL 以該 worktree 副本為準——守門、成環檢查與寫入皆作用於副本，與 plan 同一聚合面；無映射時作用於 CLI 所在 checkout。remote 模式下的臂 SHALL 依 verb-contract「模式分岔的單點宣告」對 change 的歸屬決定。

#### Scenario: 寫入單一前置

- **WHEN** 對無 depends_on 的 change c 執行 speclink change depends c --on a（a 為作用中）
- **THEN** c 的 .openspec.yaml 多一行 `depends_on: a`，其餘行逐位元不變，stdout 為 `✓ c depends on: a`，exit code 為 0

#### Scenario: 追加與移除

- **WHEN** c 已有 `depends_on: a`，依序執行 --on b 再 --on a --remove
- **THEN** 第一次後該行為 `depends_on: a, b`，第二次後為 `depends_on: b`，兩次 exit code 皆為 0

#### Scenario: 移除至空即整行移除

- **WHEN** c 只有 `depends_on: a`，執行 speclink change depends c --on a --remove
- **THEN** c 的 .openspec.yaml 不再含 depends_on 行，其餘行逐位元不變

#### Scenario: 守門拒絕零寫入

- **WHEN** 分別執行 --on 指向自己、指向不存在的名稱、指向已封存的名稱，以及會成環的邊
- **THEN** 四次皆 exit code 非零、stderr 說明原因，.openspec.yaml 逐位元不變

#### Scenario: 移除指向已封存名稱的殘留項

- **WHEN** c 有 `depends_on: old, ghost`，old 已封存、ghost 不存在，依序執行 speclink change depends c --on old --remove 再 --on ghost --remove
- **THEN** 兩次 exit code 皆為 0，第一次後該行為 `depends_on: ghost`，第二次後整行移除；再執行 --on old（不帶 --remove）仍以 exit code 非零拒絕且零寫入

#### Scenario: 名稱逐字比對

- **WHEN** 作用中 change 為 add-auth 與 c，分別執行 speclink change depends c --on Add-Auth、--on 空字串、--on archive
- **THEN** 三次皆 exit code 非零、stderr 說明沒有該名稱的作用中 change，c 的 .openspec.yaml 逐位元不變

#### Scenario: 目標 change 在 worktree 內時寫入副本

- **WHEN** 主 checkout 且 worktree 政策開啟，add-dark-mode 已映射到 worktree，執行 speclink change depends add-dark-mode --on add-auth
- **THEN** worktree 副本的 .openspec.yaml 多一行 `depends_on: add-auth`，主副本不變，隨後 speclink plan --json 的 add-dark-mode 之 dependsOn 與 blockedBy 皆含 add-auth

#### Scenario: CRLF meta 檔保留行尾

- **WHEN** c 的 .openspec.yaml 以 CRLF 換行，執行 speclink change depends c --on a
- **THEN** 新增的 depends_on 行以 CRLF 結尾，既有各行的位元組不變

### Requirement: rank 移動的依賴檢查

引擎 SHALL 提供 rank 移動檢查：給定 change 名與擬寫入的 board_rank，以新 rank 重算同階段的基底順序，若任一作用中宣告前置會排在該 change 之後、或任一宣告依賴它的 change 會排在它之前，SHALL 回拒絕結果並列出兩組名稱；僅因 delta 重疊而須依序的 change SHALL NOT 構成拒絕理由。檢查 SHALL 不寫入任何檔案。

#### Scenario: 拖到宣告前置之前被拒

- **WHEN** c 的 depends_on 含 a，兩者同為提案中，擬把 c 的 rank 改為排在 a 之前
- **THEN** 檢查回拒絕，must_follow 含 a

#### Scenario: 重疊夥伴翻轉允許

- **WHEN** a 與 b 只因 delta 重疊而依序、無宣告依賴，擬把 b 的 rank 改為排在 a 之前
- **THEN** 檢查通過

### Requirement: apply 技能以 plan 挑選與守門

內嵌 apply 技能（事實來源 crates/engine/speclink-core/assets/skills/apply.md，apply-with-worktree 與其同源）SHALL 於選擇 change 的第一步規定：先執行 `speclink plan --json`；未指名時 SHALL 取 `next`，`next` 為 null 時 SHALL 列出每個 change 的 blockedBy 並停止；指名時 SHALL 讀該 change 的 blockedBy，非空時 SHALL 印出前置清單並停止、SHALL NOT 執行 review prepare 與 in-progress add；指名的 change 不在 plan 內時沿既有錯誤處理。技能檔 SHALL NOT 再以「只有一個 change 就自動選」繞過 plan。

#### Scenario: 技能檔含 plan 守門指示

- **WHEN** 技能再生後讀取 speclink-apply 與 speclink-apply-with-worktree 的 SKILL.md
- **THEN** 兩者皆含執行 `speclink plan --json`、取 next、blockedBy 非空即停止的指示

### Requirement: archive 技能以 plan 建議封存順序

內嵌 archive 技能（事實來源 crates/engine/speclink-core/assets/skills/archive.md）SHALL 於封存前規定：執行 `speclink plan --json`，目標 change 的 blockedBy 非空時提醒使用者 plan 建議先封存那些前置；此為建議，SHALL NOT 阻擋封存，SHALL NOT 依賴引擎新增守門。

#### Scenario: 技能檔含封存順序建議

- **WHEN** 技能再生後讀取 speclink-archive 的 SKILL.md
- **THEN** 內文含執行 `speclink plan --json` 並於 blockedBy 非空時提醒先封存前置的指示，且明載僅建議不阻擋
