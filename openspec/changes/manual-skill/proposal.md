## Why

正式規格（`openspec/specs/`）的 Scenario 密度足以支撐一份給人讀的操作手冊——本輪已在 speclink（有驗收劇本）與 wadpilot（無劇本、343 份規格）兩個專案實測成功——但這條產線目前只存在於一次性的對話裡，沒有可重複的入口。使用者（透過 AI 代理跑 SDD 的開發者、PO、PM；以及第一天加入、只想知道「系統怎麼操作」的新人）需要一個技能：一句話就能從規格產出 wiki 式的新人手冊，或直接被 AI 帶著導覽系統。

## What Changes

- 新增對外工具型技能 `manual`（渲染為 `/speclink-manual`、Codex 的 `$speclink-manual`）。技能本文承載兩條動線：
  - **生成模式**（預設）：讀正式規格，寫出 `openspec/manual/*.md`。技能本文內嵌讀取策略（先以 Purpose 分流使用者面向與引擎內部規格；旅程優先取劇本型規格，無則從能力規格重建）、頁格式契約（frontmatter 即 manifest）、過期報告（既有頁的來源規格更新、與尚無頁涵蓋的使用者面向能力）與只重生受影響頁的規則。
  - **導覽模式**（引數觸發，如「導覽」或「tour」）：不產檔，AI 在對話中以手冊 frontmatter 為索引帶使用者走旅程並隨問隨答，每個回答附規格出處；無手冊時退回直接掃規格並明示。
- 新增正典 capability `manual-pages`：手冊頁的落點、檔名、frontmatter 欄位、內文慣例、必產頁、過期判定基準與重生保留規則。此契約由本變更建立，後續 desktop「手冊」頁的變更引用同一份契約。
- 修改 `skill-routing`：入口情境聯集加入「需要一份人類操作手冊、或想被導覽如何操作系統」；工具技能清單加入 manual。
- 修改 `archive-skill`：技能結尾在 `openspec/manual/` 存在時另帶一條提醒句（可跑 `/speclink-manual` 檢查手冊是否過期），明文僅提醒、不代跑。
- 詞彙：`openspec/LANGUAGE.md` 新增「手冊」「導覽」「可能過期」三條，避免與「說明書／指南／文件」混用。
- 使用者文件：`docs/workflow.zh-TW.md` 與 `docs/workflow.md` 的工具技能段加入 manual 一列（用途、技能名、完成判準、下一步），維持 user-documentation 正典「文件只寫已驗證入口」的約束。

不新增任何 CLI 子指令、旗標或設定欄位；引擎邏輯零改動。程式碼側僅有技能註冊表新增一筆與資產檔新增。

**相容性影響**：
- 渲染產物變動：`speclink init` 與 `speclink update` 會多產出一個技能目錄（claude／codex／neutral 三種目標各一份），archive 技能檔結尾多一段提醒。golden 快照（`crates/speclink-core/tests/golden/`）與 `assets.lock` 隨本變更刻意更新，`ASSET_VERSION`（`crates/speclink-core/src/init.rs`）升版；既有工作區執行 `speclink update` 即取得新技能。
- 人眼與 `--json` 輸出：`speclink list`、`speclink validate` 對新出現的 `openspec/manual/` 目錄 SHALL 無感——本變更以測試釘住此點，輸出逐位元不變。
- remote 模式：生成模式只寫本機 checkout 的 `openspec/manual/`，不上 server；技能於 remote 綁定的專案明示不支援生成（導覽模式仍可用）。

## Capabilities

### New Capabilities

- `manual-pages`: 手冊頁的格式與落點契約——`openspec/manual/` 下的 kebab-case 檔名、frontmatter 六欄（title／section／order／keywords／sources／generated）、GitHub Alert 內文慣例、必產的首頁與「本手冊的來源」頁、過期判定基準、重生時保留既有 section／order 的規則。掃描相關規格後無既有 capability 承載此契約：`user-documentation` 管的是 speclink 自身手寫文件集的結構與準確性，不管由規格衍生的手冊；`workspace-tools` 管 AI 工具指令檔的生成，與手冊頁無關。
- `manual-skill`: `/speclink-manual` 技能的內容契約——雙動線與引數形狀、生成模式的讀取策略與輸出、過期報告、導覽模式行為、remote 模式的限制敘述，以及技能檔渲染到 claude／codex／neutral 三目標。與既有 `trace-skill`、`improve-skill` 同為「per-skill 內容契約」的模式；沒有任何既有 skill capability 涵蓋手冊生成或導覽。

### Modified Capabilities

- `skill-routing`: 入口情境聯集與工具技能清單加入 manual。
- `archive-skill`: 收尾段在手冊存在時增加手冊過期檢查的提醒句（僅提醒）。

## Impact

- Affected specs: 新增 `manual-pages`、`manual-skill`；修改 `skill-routing`、`archive-skill`
- Affected code:
  - New:
    - crates/speclink-core/assets/skills/manual.md（技能資產，事實來源）
    - crates/speclink-cli/tests/it/manual_dir_ignored.rs（釘住 list／validate 對 openspec/manual/ 無感；並於 crates/speclink-cli/tests/it/main.rs 登錄 mod）
  - Modified:
    - crates/speclink-core/src/skills.rs（技能註冊表新增 manual 一筆：名稱、觸發情境描述、渲染目標）
    - crates/speclink-core/assets/skills/archive.md（結尾提醒句）
    - crates/speclink-core/src/init.rs（ASSET_VERSION 升版）
    - crates/speclink-core/tests/golden/claude.snapshot.md、codex.snapshot.md、neutral-cli.snapshot.md、neutral-tool-call.snapshot.md、claude-worktree.snapshot.md、assets.lock（渲染產物快照刻意更新）
    - openspec/LANGUAGE.md（三條新詞）
    - docs/workflow.zh-TW.md、docs/workflow.md（工具技能段加 manual）
  - Removed: 無
- 影響的 crate／app：speclink-core（資產、註冊表、golden）；speclink-cli 無程式碼改動但渲染產物變動；apps/desktop 本變更不動（desktop 手冊頁另立變更引用 `manual-pages`）。
- 影響的技能與工具：新增 manual（claude／codex／neutral 三目標）；archive 技能檔結尾變動（三目標）。
