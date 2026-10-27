## Context

手冊過期判定的契約在 `manual-pages`，生成端是 manual 技能（asset 在 speclink-core，渲染到 claude／codex），讀取端是桌面 app 的手冊索引（`apps/desktop/core/src/manual.rs`，純邏輯、不依賴 Tauri）。現況：頁的 `sources` 只記 capability 名，讀取端把該 capability 正典規格內全部 `@trace updated` 時戳拿來比對；技能規定重生後內文逐位元相同即不寫檔。`desktop-app` 一份規格餵四頁手冊，任何桌面封存都讓四頁亮燈，而內文未受影響的頁時戳永遠不動，燈永遠不消。

正典規格的結構固定：每條 `### Requirement: <名稱>` 之後接描述、scenario、該條的 `<!-- @trace … -->` 區塊，再以 `---` 分隔下一條；引擎的 trace 動詞已按此結構把 @trace 歸屬到單條 Requirement（`crates/engine/speclink-core/src/lifecycle/trace.rs`）。Requirement 名稱也是引擎合併 delta 的鍵，是規格內唯一穩定的段落識別。

來源討論：manual-stale-capability-granularity。

## Goals / Non-Goals

**Goals:**

- 讀取端：`sources` 項可帶 `#<Requirement 名>` 錨定，判定只看該段落內的時戳；無錨定維持整份規格語意。
- 生成端：內文逐位元相同的過期頁只推進 `generated`，讓燈能消；新寫或重生的頁寫錨定。
- 錨定失效（Requirement 改名或刪除）可見：視為過期，不靜默退回。
- 同一次進版把技能 asset、契約、桌面讀取端一起改，只燒一次 ASSET_VERSION。

**Non-Goals:**

- 不回填既有 41 頁手冊的 `sources`；不在本變更內重生手冊。
- 不改封存寫 `@trace` 的格式、不改 trace 動詞。
- 不拆 `desktop-app` 規格。
- 不做 Requirement 改名的自動追蹤（RENAMED delta 不回寫手冊）。

## Decisions

### 錨定語法為 capability 名加井號加 Requirement 名，寫在既有 sources 欄

`sources: ["desktop-app#看板與任務", policy-config]`。井號前是 capability 目錄名，井號後是正典規格 `### Requirement:` 之後的標題原文（去頭尾空白）。一項只能帶一個錨定；同一 capability 的多條 Requirement 寫成多項。

替代方案：另開 `anchors` 欄——否決，frontmatter 契約明文「不含其他欄位」，且錨定與來源本是同一件事；用 Requirement 序號當錨定——否決，序號在插入或刪除 Requirement 後全部位移，比名稱更脆。

YAML 面：帶錨定的項一律用雙引號包住（Requirement 名可含空白、冒號、逗號），生成端負責；讀取端用 serde_yaml 解析，任何合法 YAML 字串都收。

### 過期判定改為逐項判定，錨定項只取該 Requirement 段落內的時戳

讀取端對每個 `sources` 項：

1. 以第一個 `#` 切成 capability 與錨定；無 `#` 即錨定缺席。
2. capability 部分走既有的路徑守門（單一路徑段、無 `..`、無 `:`）；不合規整項丟棄，與現行行為相同。
3. 錨定缺席：取整份規格的全部 `@trace updated`（現行行為）。
4. 錨定存在：把規格全文依 `### Requirement:` 標題切段（一段從標題起到下一個 `### Requirement:` 標題或檔尾），找標題文字去空白後與錨定相等的段，取該段內的 `@trace updated`。找不到相符段，或規格不存在，該項視為過期。
5. 任一項判為過期即頁過期；三段式時戳比較（Stamp::counts_as_after）不變。

「錨定找不到即過期」是刻意選擇：改名或刪除代表那段規格變了，手冊必然要人看一眼；靜默退回整份規格會讓失聯永遠不可見。

替代方案：錨定找不到退回整份規格——否決，理由如上。

### 未入冊、出處行、來源消失都只看井號前的 capability 名

未入冊判定「不在任何頁的 sources」以 capability 名比對；頁尾出處行仍只列 capability 名（不列錨定）；「來源全部消失」的孤兒頁判定也只看 capability 是否存在。錨定只影響過期判定，不擴散到其他規則。桌面前端的出處列（`packages/ui` 的手冊頁元件）直接讀索引 `sources` 畫可點 chip，屬同一條規則的讀取端：取每項第一個 `#` 前的名稱後去重，再與正典 capability 清單比對決定可不可點。技能文字把這條規則集中寫成一段「Source anchors」，其他提到 `sources` 的地方都引用它，不各自複述。

### 切段沿用引擎的 parse_canonical

桌面讀取端不自己切 `### Requirement:` 段。引擎封存合併已有唯一的切段實作 `speclink_core::archive::parse_canonical`（回傳標題與整段文字），把它從 crate 內部改為公開，桌面端呼叫後對每段跑同一個 @trace 抽取。標題語法將來一動只改引擎那一處，符合 config.yaml「領域演算法歸 speclink-core」的分層規則。

### 內文逐位元相同的過期頁只推進 generated

生成端重生一頁後，比對去掉 frontmatter `generated` 行之後的內容（含重新依取材範圍推導的 `sources`）：全部相同時，只把 `generated` 換成本次時戳，其餘位元不動；不同時整頁重寫。因此錨定失效（Requirement 改名或刪除）而過期的頁，重新推導的 `sources` 必然不同，一律整頁重寫並帶上現行標題，不會卡在「只換時戳但錨定仍失效」的迴圈。摘要新增「只換時戳：N 頁」一列。index 與 about 兩頁維持既有規則（任一頁新增、重生或只換時戳時隨之重生）。

替代方案：完全不動（現行）——否決，燈永遠不消；整頁重寫——否決，會造成無意義的措辭漂移。

### 生成端寫錨定的規則

新頁或重生頁的每個 `sources` 項：該頁只取材該 capability 的部分 Requirement 時寫錨定（每條一項）；取材該 capability 全部 Requirement 時寫不帶錨定的名稱。只換時戳的頁不改 `sources`（既有寫法保留，下次真重生才換）。

### ASSET_VERSION 進版與衍生物同批再生

manual 技能 asset 內文改動觸發 ASSET_VERSION 進版（`crates/engine/speclink-core/src/workspace/init.rs`），golden 五份與 assets.lock 用既有再生流程更新；`.claude/` 與 `.agents/` 下的技能檔以 speclink update 再生，不手改。先 cargo build 再跑 update，避免 binary 帶舊版號。

## Implementation Contract

**Behavior（讀取端，桌面 app）**

- `list_manual_pages` 回傳每頁的 `sources` 為 frontmatter 原字串陣列（含錨定原樣），`stale` 為布林；欄位名與型別不變。
- 一頁 `generated: 2026-09-05`、`sources: ["desktop-app#桌面上的品質關卡"]`，規格 `desktop-app` 中「桌面上的品質關卡」段的 `@trace updated` 為 `2026-08-14`，其他段有 `2026-09-10T17:15:24+08:00`：`stale` 為 false。
- 同一頁改為 `sources: [desktop-app]`：`stale` 為 true。
- 錨定「不存在的段名」：`stale` 為 true。
- 混合項 `["desktop-app#A", other-cap]`：任一項過期即 true。
- 未入冊：capability `x` 只出現在某頁的 `x#某段` 項時，`x` 不列入未入冊。
- 路徑守門：`../evil#段`、`a/b#段` 整項丟棄，行為與現行相同。
- 邊界：`"cap#"`（空錨定）視同裸名 `cap`；capability 與錨定兩半都去頭尾空白（`" desktop-app # 看板與任務 "` 可解析）。
- 前端出處列：`sources: ["desktop-app#看板與任務", "desktop-app#系統匣選單", policy-config]` 的頁，出處列恰有一顆可點的 `desktop-app` chip 與一個不可點的 `policy-config`（不在正典時），文字中不出現 `#`。

**Interface / data shape**

- frontmatter `sources` 項：`<capability>` 或 `<capability>#<Requirement 名>`；帶錨定的項由生成端以雙引號包住。
- 段落切分規則：以行首 `### Requirement:` 為段界；標題文字去頭尾空白後與錨定精確比對（區分大小寫）。

**Behavior（生成端，manual 技能）**

- 過期頁重生後內文相同：只改 `generated` 一行，摘要「只換時戳」計入該頁；內文不同：整頁重寫並計入「重生」。
- 新頁與整頁重寫的頁：`sources` 依「部分取材寫錨定、全部取材寫名稱」規則寫出。
- 技能的過期報告與讀取端採同一錨定規則（契約明文「生成端與讀取端同基準」）。

**Failure modes**

- 錨定找不到、規格不存在：該項過期（可見）。
- `sources` 項不合路徑守門：整項丟棄（靜默，與現行同）。
- frontmatter 無法解析：頁列為 malformed，行為不變。

**Acceptance criteria**

- `cargo test -p speclink-desktop-core manual` 新增測試通過：錨定段落級判定、錨定找不到即過期、無錨定維持整份規格、未入冊只看 capability 名、混合項、守門丟棄、空錨定與去空白。
- `npm test -w packages/ui -- manualPage` 新增測試通過：錨定頁的出處列只列井號前名稱、去重、可點。
- `cargo test -p speclink-core --test it render_golden::` 綠，golden 與 assets.lock 已隨新版號再生。
- speclink update 後 `.claude/skills/speclink-manual/SKILL.md` 含新 stale page 定義、`sources` 錨定說明與「只換時戳」摘要列。
- 三份 delta spec 經 speclink validate 通過。

**Scope boundaries**

- In：manual-pages 契約、manual-skill 規格、desktop-manual-page 規格的 delta；`apps/desktop/core/src/manual.rs`；`packages/ui` 手冊頁出處列；`speclink_core::archive::parse_canonical` 公開；`openspec/LANGUAGE.md` 詞條；manual 技能 asset 與 ASSET_VERSION 進版及衍生物再生。
- Out：既有手冊頁回填、手冊重生、封存 @trace 格式、trace 動詞、桌面前端出處列以外的畫面、server。

## Risks / Trade-offs

- [golden 回歸對照] ASSET_VERSION 進版讓全部 golden 的版號行變動 → 用既有 UPDATE_ASSETS_LOCK 流程一次再生，diff 只應含版號行與 manual 技能內文。
- [跨平台] 段落切分以行首 `### Requirement:` 為準，須容忍 `\r\n` → 切分前逐行 trim 行尾 `\r`（與既有 split_frontmatter 同做法）；測試用兩種換行各跑一次。
- [Requirement 改名] 封存 RENAMED 後手冊錨定失效，該頁持續亮燈直到重生 → 這是刻意的可見失效；下次跑 manual 技能重生時寫新名稱。
- [YAML 逗號與冒號] Requirement 名含 `,`、`:`、`]` 時未加引號會解析失敗 → 生成端一律雙引號；讀取端遇解析失敗走既有 malformed 路徑並列入索引的 malformed 清單。
- [既有 41 頁] 不回填，行為與現在完全相同（整份規格語意），假警報要等各頁下次重生才收斂 → about 頁已知侷限段在下次重生時載明。
- [兩端漂移] 技能文字與 Rust 讀取端各自實作同一規則 → 契約 Example 判定表為共同真相，兩端測試／檢查都對照它。
