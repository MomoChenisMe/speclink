## Context

正式規格（`openspec/specs/`）是系統行為的唯一真相，且 Scenario 逐字寫下按鈕文案、確認框內容與錯誤出路。本輪在兩個專案實測證明：只讀規格就能寫出給新人的 wiki 式操作手冊——speclink 有驗收劇本與交棒表，旅程屬「轉寫」；wadpilot 無劇本、343 份規格，旅程屬「重建」，品質仍可用但要誠實標出規格內部的新舊矛盾。討論 `manual-generation-skill` 定案：把這條產線固化成技能 `manual`，產出純 Markdown、呈現交給 desktop 的「手冊」頁（另立變更）。

現況：技能資產為 `crates/speclink-core/assets/skills/` 下 22 個單一 `.md`，經 `skills.rs` 的註冊表以 `include_str!` 內嵌、渲染成三種目標（claude／codex／neutral）的 SKILL.md；沒有附檔機制。渲染產物由 golden 快照與 `assets.lock` 保護，資產內文變動須升 `ASSET_VERSION`。

## Goals / Non-Goals

**Goals:**

- 一句話（`/speclink-manual`）從正式規格產出 `openspec/manual/*.md`，或（帶「導覽」引數）在對話中被 AI 帶著導覽系統。
- 手冊頁格式成為正典契約 `manual-pages`，讓後續 desktop 手冊頁與任何第三方讀取端（SSG、pandoc）都以 frontmatter 為唯一索引。
- 重生只碰受影響頁：既有頁的 section／order 逐字保留，未動頁逐位元不變。
- 觸發一律人工；archive 技能結尾在手冊存在時多一句提醒。

**Non-Goals:**

- 不產 HTML、不帶模板、不擴充技能附檔機制（討論已排除 A／C1／C2 三案）。
- 不新增引擎動詞、旗標或設定欄位；過期判定由技能讀規格 `@trace updated` 自行比對。
- 不做 validate 級 frontmatter lint（漂移實際發生再補）。
- 不處理 remote 模式的手冊投影：remote 綁定的專案生成模式明示不支援；導覽模式照常。
- 不做 desktop「手冊」頁——另立變更 `desktop-manual-page` 引用 `manual-pages`。
- 不接 LLM Wiki／DeepWiki 類工具；手冊來源只有正式規格，SHALL NOT 讀 README、docs 或程式碼作為內容來源。

## Decisions

### 技能資產與註冊表新增 manual 一筆

新增資產 `crates/speclink-core/assets/skills/manual.md`，在 `crates/speclink-core/src/skills.rs` 的技能註冊表加入 `manual` 一筆：description 以觸發情境句開場（「需要一份人類操作手冊、或想被導覽如何操作系統時」）、`for_codex` 為真、不 fork、不禁編輯（生成模式要寫檔）。與既有 `trace`、`audit` 同為工具技能。替代方案「只在單一專案手寫 `.claude/skills/`」被排除：不進正典、其他專案用不到、不受 golden 保護。

### 格式契約內嵌技能本文而非附檔

frontmatter 六欄、內文慣例、必產頁、過期判定與重生規則，約三十行文字，直接寫在技能本文。與 discuss 的 round 模板、verify／review 的工單輪格式同一慣例。替代方案「附檔 references」被排除：現有機制一技能一檔，且擴機制只為一個客戶違反專案慣例。

### 過期判定以規格 @trace updated 對頁 generated

頁過期 ⇔ 任一 `sources` 規格的最新 `@trace updated` 日期晚於該頁 `generated`。未入冊 ⇔ 技能分流為使用者面向的 capability 不在任何頁的 `sources`。兩種都列入生成模式的啟動報告。替代方案「比對封存清單」被排除：@trace updated 就在規格檔內、`speclink show` 可讀，不需多一層對映。

### archive 提醒句以 openspec/manual/ 目錄存在為條件

archive 技能是純文字資產，判斷不了「本次封存是否動到使用者面向規格」。條件簡化為「工作區有 `openspec/manual/`」即提醒可跑 `/speclink-manual` 檢查過期，過期與否交給 manual 技能報告。提醒明文僅建議、SHALL NOT 代跑，符合 skill-routing 的交棒契約。

### 手冊落點 openspec/manual/ 且 list 與 validate 無感

手冊是規格的衍生物，與源頭同住 `openspec/`；desktop 與系統匣以 `openspec/` 為資料根、watcher 已盯著它，將來 remote 投影也有既有通道。風險是引擎動詞對多出的目錄敏感——以整合測試釘住：建立 `openspec/manual/index.md` 後 `speclink list --json` 與 `speclink validate --specs` 的輸出逐位元不變、exit code 0。替代方案 `docs/manual/` 被排除：desktop 要另接一條路徑、watcher 要另跟。

### 不新增引擎動詞

生成、過期比對、導覽全在技能層以既有 CLI（`speclink list --specs --json`、`speclink show <capability>`）與檔案讀寫完成。引擎級 `manual` 動詞列入 Deferred；有第二個消費者（如 desktop 要引擎算過期）再立案。

## Implementation Contract

**可觀察行為（生成模式）**

- 使用者呼叫 `/speclink-manual`（無引數）→ 技能讀規格、寫 `openspec/manual/*.md`，結束時在對話中輸出摘要：新增頁數、重生頁數、未動頁數、過期頁清單、未入冊能力清單、about 頁記錄的矛盾數。
- 已有手冊時預設只重生過期頁；使用者要求「全部重生」時全量重寫但既有 section／order 逐字保留。
- 專案綁定 remote store（`.speclink.yaml` 有 remote 區段）時，生成模式輸出「remote 模式尚不支援手冊生成」並停止，零檔案寫入。

**可觀察行為（導覽模式）**

- 引數含「導覽」或「tour」→ 技能不寫任何檔案；有手冊時以 frontmatter 為索引，先問一題角色，再依 section／order 帶旅程，每個回答附 capability 名出處；無手冊時明示「尚無手冊，改以規格直接導覽」。

**檔案形狀**：依 `manual-pages` 規格——kebab-case 檔名、六欄 frontmatter（title／section／order／keywords／sources／generated）、GitHub Alert 內文、頁尾出處行、必產 `index.md` 與 `about.md`。

**渲染產物**：`speclink init` 與 `speclink update` 於三種目標各多一個 `speclink-manual` 技能目錄；archive 技能檔結尾多一段手冊提醒。golden 五份快照與 `assets.lock` 同批更新、`ASSET_VERSION` 升一個 minor。

**失敗模式**：規格無任何使用者面向 capability 時，生成模式仍產 `index.md` 與 `about.md`，about 頁載明「尚無可入冊的使用者面向能力」；`openspec/manual/` 內有無法解析 frontmatter 的頁時，技能列入報告並跳過該頁、不覆寫。

**驗收**：
- `cargo test -p speclink-core --test it render_golden::` 綠（快照已更新）。
- 新測試 `crates/speclink-cli/tests/it/manual_dir_ignored.rs`：有 `openspec/manual/` 時 list／validate 輸出與無此目錄時逐位元一致。
- 手動：於 speclink 專案跑 `/speclink-manual`，檢查 `openspec/manual/` 兩必產頁與 frontmatter 六欄；再跑一次確認未動頁逐位元不變；跑 `/speclink-manual 導覽` 確認零寫檔。

**範圍邊界**：in scope＝技能資產、註冊表、archive 提醒句、golden 與版號、`manual-pages` 契約、LANGUAGE.md 三詞、docs/workflow 工具技能段。out of scope＝desktop 手冊頁、引擎動詞、lint、remote 投影、HTML 產出。

## Risks / Trade-offs

- [golden 快照與 assets.lock 漏更新導致 CI 紅] → 同批升 `ASSET_VERSION`、重生五份快照與 lock；記憶中的三連動慣例列為獨立任務。
- [技能檔內文靠模型紀律維持格式，可能漂移] → frontmatter 規則在技能本文以表格釘住；desktop 讀取端寬容；漂移實際出現再補 validate lint。
- [大型專案首次生成 token 高] → 讀取策略先以 Purpose 分流、只讀使用者面向規格；二次起只讀過期頁的來源規格。
- [跨平台：Windows 路徑與換行] → 技能只寫相對於工作區的 `openspec/manual/`，檔名固定 kebab-case ASCII；測試以 tempdir 建目錄、不假設分隔符。
- [`speclink update` 在既有工作區覆蓋 archive 技能檔] → 屬既有機制（資產版本升級即覆蓋），相容性影響已在 proposal 記載。

## Migration Plan

1. 合入後既有工作區執行 `speclink update` 取得 `speclink-manual` 技能與新的 archive 技能檔。
2. 回滾：還原資產與註冊表、golden 與 `ASSET_VERSION` 同批還原；已生成的 `openspec/manual/` 為普通檔案，刪除即可。

## Open Questions

- 無。desktop 手冊頁的細節在變更 `desktop-manual-page` 處理。
