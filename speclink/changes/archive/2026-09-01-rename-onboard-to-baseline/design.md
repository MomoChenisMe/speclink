## Context

onboard 技能的唯一程式碼定義在 crates/speclink-core/src/skills.rs 的 registry；內文由 crates/speclink-core/assets/skills/onboard.md 於編譯期嵌入。生成面只有 claude（.claude/skills/）、codex（.agents/skills/）與自訂描述子三種目標，目錄名一律 speclink- 前綴。update 現況只有三條刪除路徑（worktree 政策關閉、工具下架、自訂描述子移除），registry 內技能改名後的孤兒目錄沒有人清。決策脈絡與被剔除的替代方案記錄在來源討論 rename-onboard-to-baseline，本文件只承載實作面的設計。

## Goals / Non-Goals

**Goals:**

- 技能 id 由 onboard 改為 baseline，生成名 speclink-baseline，claude 與 codex 兩側一致。
- speclink update 取得「孤兒清理」能力：改名或下架的技能目錄在下一次 update 消失，對未來改名同樣生效。
- openspec/LANGUAGE.md 釘住「規格基準」與「Apply baseline」雙義，文件裸寫的 baseline 補修飾詞。

**Non-Goals:**

- 不留 deprecated alias、不同時生成兩份技能。
- 不改技能行為內容、不新增 CLI 子指令、不新建 baseline-skill capability spec。
- 不回改歷史（封存 changes 與 discussions、@trace 檔案清單、desktop 首啟 onboarding、server e2e 的 team onboarding）。
- 不採 tombstone 舊名清單。

## Decisions

1. **名稱 baseline**：requirements baseline 是「第一批核可規格、後續變更以它為基準」的業界標準詞。備案 backfill、adopt、codify、snapshot、seed 的剔除理由見來源討論的 Round 1。撞名管理：現行五處 baseline 用法皆為帶修飾詞複合詞，以 LANGUAGE.md 釘義共存。
2. **prune 採 registry 差集，落在 speclink-core 的 init 模組**：與既有 prune_footprint 同層。所有權判準沿用既有語意——目錄名前綴 speclink- 即視為生成物，不讀 frontmatter。期望集合按工具計算：claude 為 registry 全集、codex 為 for_codex 子集、worktree 政策關閉時兩側都排除兩顆 gated 技能；自訂描述子目標用同一規則。剔除 tombstone 清單：每次改名要維護一筆，而差集一次到位。
3. **prune 只掛在 update，不掛 init**：init 不帶 force 時不覆寫既有檔，維持「init 對既有工作區保守」的既有性格；update 本來就是同步動作，force 覆寫與清理同屬其語意。workflow-config 的 worktree 切換走 init::update，因此自動獲得同一行為。
4. **寫入與刪除順序**：先逐技能生成（既有 generate_tool 邏輯），全部寫完後再對該工具目錄做差集刪除。任一刪除失敗即回傳錯誤；此時已生成的檔案保留——生成具冪等性，重跑 update 收斂到同一終態，接受這種半套狀態。
5. **版本三連動**：ASSET_VERSION v1.24.0 → v1.25.0（crates/speclink-core/src/init.rs 的常數）；assets.lock 以 UPDATE_ASSETS_LOCK=1 於乾淨樹再生；五份 golden 以 UPDATE_GOLDEN=1 再生。兩個環境變數是獨立開關，缺一不可。
6. **skill-routing 的 Scenario 改名帶 REMOVED-SCENARIO 宣告**：MODIFIED 是整塊取代，改 Scenario 名在引擎眼中是未宣告刪除，validate 與 analyze 都抓不到、到 archive 才炸。delta 內以 REMOVED-SCENARIO 註解明示。
7. **LANGUAGE.md 只釘義、不加 avoid 詞**：新詞條定義「規格基準」為 baseline 技能的產出、與「Apply baseline」（品質站凍結點）分立；avoid 清單留空，避免 scripts/vocabulary-guard.test.mjs 對既有複合詞（Apply baseline、smell baseline）誤報。
8. **local／remote 邊界**：init 與 update 是本地工作區動詞，技能生成不經 Host 或 Protocol，remote 路徑無平行實作，無 parity 議題。

## Implementation Contract

**行為（可觀察）**

- registry 含 id 為 baseline 的技能、不含 onboard；description 為 baseline 情境句（先情境、後產出，涵蓋「既有專案首次建規格」入口）。
- 生成目錄為 speclink-baseline，frontmatter name 為 speclink-baseline、metadata.version 為 v1.25.0；claude 側呼叫名 /speclink-baseline、codex 側 $speclink-baseline。
- 技能內文維持既有流程（盤點、capability map 以 AskUserQuestion 確認後才寫規格、validate、交棒 propose／discuss），自指名稱與「舊稱 onboard」不出現在內文——內文只用新名。
- 在含 speclink-onboard 目錄的工作區執行 speclink update 後：speclink-onboard 目錄不存在、speclink-baseline 存在；非 speclink- 前綴的目錄（如 conventional-commit）位元不變；speclink- 前綴但不在期望集合的目錄一律清除。
- worktree 政策關閉的工作區跑 update：兩顆 worktree 技能目錄同樣不存在（與既有 skip_gated_skill 行為可觀察等價）。
- speclink --version 顯示 engine v1.25.0。

**失敗模式**

- prune 刪除失敗（權限、檔案佔用）：update 以錯誤結束，已生成檔案保留，重跑收斂。不新增靜默吞錯。

**驗收判準**

- crates/speclink-core/src/init.rs 的 tests 模組新增單元測試：(a) 預置 speclink-onboard 目錄 → update → 僅 speclink-baseline 存在；(b) 非 speclink- 前綴的使用者目錄不動；(c) speclink- 前綴的非 registry 目錄被清除。
- cargo test -p speclink-core --test it 全綠（render_golden 與 skill_verbization 含在內）；golden 與 assets.lock 依決策 5 再生。
- npm run -w scripts 或對應入口執行 vocabulary-guard 測試通過（LANGUAGE.md 新詞條不得引入誤報）。
- 隔離專案手動驗證：init 只生成 speclink-baseline；預置舊目錄後 update 不留兩份；技能執行時先盤點、確認 capability map、寫入正式 specs、不建 change、不改 code。

**範圍邊界**

- In scope：registry 與 asset 改名、update 差集 prune、版本三連動、八份文件、LANGUAGE.md 詞條、三份 specs delta、repo 自身生成物再生。
- Out of scope：技能內文流程改動、CLI 子指令、Desktop／Server／Node SDK 程式碼、歷史紀錄、其他技能的內容。

## Risks / Trade-offs

- [golden 回歸對照大量變動掩蓋非預期 diff] → 先只做改名跑 UPDATE_GOLDEN 檢視 diff 範圍，確認僅 onboard→baseline 與版本戳；prune 邏輯不影響 render 輸出。
- [assets.lock 在髒樹再生出錯誤指紋] → 依 render_golden 守則於乾淨樹跑 UPDATE_ASSETS_LOCK=1。
- [使用者自建 speclink- 前綴目錄被誤刪] → 與既有 prune_footprint 前綴語意一致；proposal 相容性影響段載明「speclink- 前綴保留給生成物」。
- [跨平台：Windows 檔案佔用使刪除失敗] → 失敗即報錯、重跑收斂；測試用 std fs 於三平台 CI 驗證，路徑組合走既有 join 慣例、不寫死分隔符。
- [全部 SKILL.md 版本戳 bump 產生大量 diff] → 收尾 commit 前以 git status 盤點生成物，確認全數入列。
