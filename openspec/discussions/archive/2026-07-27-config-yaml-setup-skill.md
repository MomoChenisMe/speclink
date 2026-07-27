---
topic: config.yaml 的 context 寫法優化與設定技能
slug: config-yaml-setup-skill
status: promoted
promoted_to: workflow-config-verb-and-skill
created: 2026-07-27
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: config.yaml 的 context 寫法優化與設定技能

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者要新增一支 speclink 專屬技能，用於自動整理 `openspec/config.yaml`：搭配 CLI 時依 remote 或 local 模式決定設定存放，locale / spec_locale / tdd / audit 等政策逐項詢問，並以本專案的 config.yaml 為範本——但範本本身要先優化。追加需求：技能每次呼叫都能依當下 codebase 做一次迭代優化（codebase 會持續演進），且本專案現有的 context 寫得不夠好，需先考據 OpenSpec 對 context 的既定寫法。

模式：assumptions——codebase scout 找到 `crates/speclink-core/src/config.rs`（WorkflowConfig / update_workflow_config_text / WorkflowPolicyFields）、`crates/speclink-core/src/instructions.rs`（context 與 rules 的注入點）、`crates/speclink-core/src/init.rs`（config.yaml 範本）、`crates/speclink-cli/src/commands.rs`（config 動詞）、`apps/desktop/src-tauri/src/lib.rs`（本機／遠端寫入 command）、`apps/desktop/src/views/ProjectSettingsView.tsx`（既有設定頁），足以直接形成立場。

相關規格：workflow-config（正典規格對 `context` 一字未提，只規範四層解析、deprecation 警告、init 範本政策寫入位置）、desktop-config、command-runtime。目前無進行中的變更。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-27)

**Focus**: 一支「搭配 CLI、依 local/remote 決定儲存」的設定技能，現有基建撐得住嗎？
**Position**: 撐不住——core 有改寫器但 CLI 沒有任何動詞可寫 `openspec/config.yaml`，remote 路徑更只能經動詞：
- `speclink config path` 實測回 `~/Library/Application Support/speclink/config.yaml`：`config get/set/list/unset/reset/edit` 全部管全域設定，完全不碰專案的 `openspec/config.yaml`。
- core 的 `config.rs:503 update_workflow_config_text` 是完整的 text→text 改寫器（政策四欄為完整目標態、context 三態、rules 整份代換，未知鍵原樣保留），但目前唯一消費者是 desktop 的 Tauri command（`apps/desktop/src-tauri/src/lib.rs:263` 本機／`:644` 遠端）。
- remote 模式不可能靠寫檔：`remote_write_workflow_config` 要求 `expected_revision: u64` 樂觀鎖，且 remote checkout 根本不建本機 `openspec/`（`connections.rs:681` 測試斷言）。故 local/remote 的分流必須在 CLI/Host 層裁決，技能側不該有分支。
- 逐項詢問的四欄正好是 `WorkflowPolicyFields`（locale / spec_locale / tdd / audit），與 desktop `ProjectSettingsView` 的政策區塊同源；`context` 與 `rules` 則屬可從 codebase 推導的內容層，問使用者無意義。
**Ruled out**: 技能直接以檔案工具寫 `openspec/config.yaml`——local 可行但 remote 永久無解，兩條路徑不對等。
**Open**: CLI 動詞的形狀（擴充 `speclink config` 加 `--project` 面，或另立動詞）？context/rules 的產出如何避免每次呼叫都全量改寫（迭代收斂判準）？

### Round 2 — assumptions (2026-07-27)

**Focus**: OpenSpec 對 `context` 有無既定寫法？本專案這份該怎麼優化才算對？
**Position**: 上游只給定位不給長度，但注入機制本身就是硬判準——context 進四個 artifact、rules 只進對應 artifact，據此本專案的 context 可砍掉三分之二：
- 專案內既有說明只有三處且都極簡：`init.rs:38-45` 範本註解舉的例子是四行（Tech stack / conventional commits / Domain）；`docs/configuration.md:100` 一句「Project context shown to AI when creating artifacts」；正典規格 `openspec/specs/workflow-config/spec.md` 對 context **一字未提**（只規範四層解析、deprecation 警告、init 範本政策寫入位置）。
- 上游 OpenSpec 官方 customization 文件明載：「Context appears in ALL artifacts」對比「Rules ONLY appear for the matching artifact」，範例同樣是四行技術棧＋慣例；**未給任何長度或精簡建議**。
- 程式碼比文件更硬：`instructions.rs:178` 每次 `build_artifact` 都以 `wf.context_text()` 整份帶入，`rules` 則是 `wf.rules_for(&artifact.id)` 分 artifact。context 長度＝proposal/specs/design/tasks 每次產生都要付四遍的固定成本。
- 本專案 context（約 60 行實體內容）有三類該砍：(a) **與自動注入重複**——`tdd: true` 已讓 tasks instruction 自動帶 Red-Green-Refactor（`instructions.rs:150-157`）、`audit: true` 同理（:158-165）、`spec_locale: tw` 已自動注入「散文用繁中／結構標記與 SHALL 保留英文」且多帶中文弱語言偵測（:120-135），而「紅線約束」把 TDD 與語言兩條又複述一遍；(b) **只對單一 artifact 咬合、該降級為 rules**——「分層邊界」與 rules.design 既有條目重疊，「`--json` 欄位一律 camelCase」在 rules.specs 與 rules.tasks 各已有一條；(c) **對 AI 無作用的 meta**——「精確版本一律以各 Cargo.toml 為準（會過時）」是寫給人看的免責聲明。
- 該留下的是四個 artifact 都需要的專案身分：Speclink 是什麼、Local Repo 與 Remote Store 雙路徑、crate 地圖、雙輸出路徑（人眼＋`--json`）、正典 vs delta、設定分兩層，以及三份 docs 的指路（product-status / platform-architecture / roadmap）——指路成本低、收益高，保留。
**Ruled out**: 把 context 當「專案知識全集」寫——注入機制決定它是四份 artifact 共同前言，不是知識庫；artifact 專屬內容有 rules 這個更精準的位置。
**Open**: 「每次呼叫依 codebase 迭代優化」如何收斂而非每跑一次就重寫一遍？技能是否應只產 diff 交使用者裁決、不自動寫入？優化後的 config.yaml 要不要反過來成為技能內嵌的參考範例（三處內嵌技能同步的維護成本）？

### Round 3 — assumptions (2026-07-27)

**Focus**: 使用者裁定（產 diff、config.yaml 與技能同一變更、補 CLI 動詞且 local/remote 都要）之後，新動詞該落在哪一層？
**Position**: wire 面已經齊備，這刀只缺 CLI 側的薄編排——但絕不能掛進既有 `speclink config`：
- 使用者裁定三項：技能產 diff 交人裁決（不自動寫）；先優化本專案 config.yaml、再拿改完的當技能內嵌範例，兩者同屬一個變更；CLI 動詞要補，local 與 remote 都要。
- **好消息**：remote 的傳輸面已存在——`speclink-protocol` 已有 `client.config()`（GET，回 content＋revision）與 `client.put_config(text, expected_revision)`（PUT＋CAS），desktop 的 `apps/desktop/src-tauri/src/remote.rs:1285-1305` 就是靠這兩支。文字改寫 seam 也已在 core（`config.rs:503 update_workflow_config_text`）。CLI 要補的只有「取現況文字 → 過 seam → 依模式寫回」這段編排。
- **深度檢查（新 CLI 動詞、跨 CLI↔Host↔Store 流）**：seam 位置＝`speclink-core::config` 的文字改寫器（已在正確位置）；adapter 數＝目前 desktop 自帶一套（`apps/desktop/core/src/settings.rs` 的窄化包裝＋`remote.rs` 的模式分派），`Store` trait 只有 `read_workflow_config`、**沒有任何 write**，寫入從未進過 Store contract；深度＝分派層真正藏的是「local 讀寫檔 vs remote GET/PUT＋CAS＋離線拒絕」；刪除測試＝若 CLI 自建第二套分派，revision 與離線語意會在兩個 client 各自漂移。
- 結論：這刀 CLI 走薄編排、**單一動詞內完成讀-改-寫**（remote 由 CLI 自己先 GET 取 revision，不對使用者暴露 revision），衝突交 server CAS 擋下並提示重跑。不為此新建 Host 抽象層——消費者目前只有二，硬拉一層違反專案的禁過度設計紅線；但 CLI 的分派要寫在 desktop 未來可收斂過去的位置。
- **不可掛進既有 `speclink config`**：`commands.rs:1353-1430` 實測是無 schema 的自由 key-value 存放（任何 key 都能 set，`allow_unknown` 參數根本被忽略），為 Spectra parity 而存在，且 `set` 只吃單行 scalar——多行 `context` 塞不進去，語意也與有 schema 的政策改寫衝突。
**Ruled out**: `speclink config --project` 之類的旗標擴充——會讓同一組動詞同時是自由 KV 存放與 schema 化政策改寫；另外也排除「為此新建 Host command 層」，消費者數量未達門檻。
**Open**: 新動詞的具體形狀與命名？寫入介面確定走 `--stdin`（仿 `discuss context --stdin`，因 context 是多行）；`--dry-run` 的 diff 由 CLI 產（與實寫共用同一改寫路徑，杜絕技能算的 diff 與實際寫入不符）待確認；技能每次呼叫的掃描範圍如何界定才不會在大 repo 上失控？

### Round 4 — assumptions (2026-07-27)

**Focus**: 動詞形狀定案，以及技能每次呼叫要掃什麼才會收斂？
**Position**: 動詞取 `speclink workflow-config`；技能的輸入集合必須固定成結構性事實來源，收斂性才有可測的驗收：
- 使用者選定 `speclink workflow-config`（與 `language` 對稱——`Store` trait 的 `read_workflow_config` 與 `read_language` 本就並列，兩份 store 文件各有自己的動詞；kebab-case 與既有 `in-progress` 一致）。形狀：`show [--json]`、`set <key> <value>`（四欄政策）、`context --stdin`、`rules <artifact> --stdin`，寫入一律可加 `--dry-run` 印 unified diff。模式由 binding 判定：local 讀寫 `openspec/config.yaml`，remote 走 `client.config()` → 同一 seam → `client.put_config` ＋ CAS。
- 技能的掃描範圍**不做全 repo 掃描**，固定讀四類結構性來源：workspace 清單（Cargo.toml members / package.json workspaces）、README、docs/ 索引、既有 config.yaml，外加 `speclink language show`（若有）。理由：context 該寫的是專案身分（crate 地圖、雙路徑、輸出契約），這些全部可從結構與索引推得；深入讀原始碼得到的是實作細節，而實作細節本就違反第三條判準（會過時的不寫）。輸入集合固定，同一 codebase 兩次呼叫才會產出相同結果。
- **收斂性的驗收條件**：對同一個未變動的 codebase 連跑兩次，第二次的 diff 為空。這是可測的，直接寫進規格。
- **已知取捨（新發現）**：`update_workflow_config_text` 明載會丟失模板註解（與 `init::write_remote_section` 同一取捨）。本專案的 `openspec/config.yaml` 開頭已直接是 `schema: spec-driven`、init 範本那些 `# Workflow policy (optional)` 說明註解早已不存在——證明這條路徑已被 desktop 設定頁走過。對新專案的首次使用者，跑完技能即失去範本註解，需在規格中明述而非留給使用者踩。
**Ruled out**: 技能自行全 repo 掃描——輸入集合不固定則多次呼叫必然震盪，且與「不寫會過時的內容」判準衝突。
**Open**: 技能名稱（`speclink-config`？）；本刀是一個變更還是拆「先動詞、後技能」兩刀。

### Round 5 — assumptions (2026-07-27)

**Focus**: 把三條判準實際套到本專案的 config.yaml，會刪掉什麼、判準本身撐不撐得住？
**Position**: 定稿產出於 scratchpad（`config.optimized.yaml`，YAML 解析通過、政策四欄未動），context 從 40 行／2787 字降到 27 行／1603 字（−42%）；套用過程逼出判準一的兩處缺陷與 context 的一個事實錯誤：
- **判準一（已自動注入者不得重述）刪除**：「TDD 無例外：紅→綠→重構」（`tdd: true` 已注入 tasks instruction）、「語言：UI 與散文用繁中／結構標記保留英文」（`spec_locale: tw` 已注入 specs instruction，且更完整——多帶中文弱語言偵測詞表）。
- **判準二（單一 artifact 咬合者降 rules）刪除**：「分層邊界」整條併入 rules.design 首條並從只提 core/cli 擴充為完整分層（core／fs／store／host／adapters）；「`--json` camelCase」細節（rules.specs／design／tasks 已各有一條）；「技能相容 init 產生 claude/codex」（rules.proposal 已有）；「serde／chrono／git」（rules.design 已有兩條涵蓋）。
- **判準三（會過時者不寫）刪除**：「精確版本以各 Cargo.toml 為準」meta 免責；回歸對照的「parity_suite 31 項／color_suite 16 項／twin harness 8 情境」數字（原則保留、數字刪——記憶亦載 scratchpad 基建會消失）。
- **修正 context 的事實錯誤**：原文「設定分兩層：`.speclink.yaml`（應用層：locale、spec_locale、tdd、audit、tools）」與正典相反——`docs/configuration.md:109` 明載這四欄在 `.speclink.yaml` 已 **deprecated（still honored, warns on every command）**，正典歸屬是 `openspec/config.yaml`。改寫為三層：config.yaml（政策，隨 spec store）／.speclink.yaml（tools、spec_dir，隨 checkout）／`SPECLINK_*`（個人／CI）。crate 地圖亦補上遺漏的 `apps/server-web` 與 `packages/ui`，並修正 rules.proposal「影響的 crate（speclink-core / speclink-cli）」這個只列兩個 crate 的過時寫法。
- **判準一需擴大（本輪發現）**：範圍不只 context，rules 同樣適用——rules.tasks 的「嚴格遵循 TDD 順序」與 `tdd: true` 的自動注入重複，已刪；且不只政策開關，**schema 內建 instruction 也算**——`assets/schema/spec-driven/specs.instruction.md:15-18,92-93` 已載明 SHALL/MUST 用法、WHEN/THEN 格式與「結構標記與關鍵字保留英文」，故 rules.specs 兩條隨之改寫（保留其中本專案特有的「只用可觀察 CLI 行為、不得出現 Rust 型別」）。
- **判準一的執行方式因此可機械化**：以 `speclink instructions <artifact> --json` 取得實際 payload（instruction ＋ context ＋ rules），逐條比對是否已被 instruction 覆蓋——不靠印象判斷重複。本輪即因印象誤刪「語言相關行為須分別敘明 tw／ja／en」（那是要求規格涵蓋多語言**被測行為**，與「用什麼語言書寫」無關），查證 instruction 後補回。
- rules 條數變化：proposal 9→9、specs 10→10、design 7→9（多行 YAML 區塊拆為獨立條目）、tasks 8→7。
**Ruled out**: 憑印象判斷「這條是否已被自動注入」——本輪已誤刪一條，證明必須以實際 payload 為準。
**Open**: 定稿是否還有該砍未砍（27 行仍高於上游四行範例，但 crate 地圖是四個 artifact 的定位基礎）；技能名稱；一刀或兩刀。

### Round 6 — assumptions (2026-07-27)

**Focus**: crate 地圖依最新 codebase 校正，並把優化直接落檔（使用者裁定不開 propose）。
**Position**: `openspec/config.yaml` 已直接寫入優化版並經引擎實測驗證；crate 地圖以 14 個 workspace member 的 doc comment 重核，補回三項對設計有約束力、舊版遺漏的事實：
- 使用者裁定：crate 地圖保留帶職責的形式；config.yaml 的優化**不開 propose、直接落檔**（它是設定調整而非程式碼變更，走變更流程過重，且不落檔會使後續 apply 產生 drift）；技能名 `speclink-config`；CLI 動詞與技能合為一刀。
- **crate 地圖校正（依 `Cargo.toml` members 與各 crate `//!` doc comment，非沿用舊文）**：`speclink-server` 補「axum adapter，**async 邊界僅在此 crate**」；`speclink-protocol` 補「Rust 型別為正典、JSON Schema 是匯出」；`speclink-store` 補 TeamStore contract 的實質（Project／Repo 定址、CAS 寫入、不可變歷史、transactional outbox）；`speclink-node` 補「引擎核心維持同步、dispatch 走 worker thread」；`apps/desktop/core` 補「純邏輯、不依賴 Tauri」。另刪去「14 個 crate」這類計數（判準三：會過時）。
- **桌面分層升格為 design 規則**：`apps/desktop/core` 的 doc comment 載明「Tauri 殼的每個 `#[tauri::command]` 是對 speclink-desktop-core 的單行委派，真正的邏輯與測試在 core」——這是規範而非詞彙，故進 rules.design 而非 context。
- **落檔後的實測驗證**（在 scratchpad 隔離 workspace 內 init ＋ 套用本檔 ＋ 建 probe change，跑 `speclink instructions <artifact> --json`）：proposal payload 讀到 `locale=Traditional Chinese (繁體中文)`、context 1736 字元、rules 9 條——證明 serde_yaml 解析成功、政策未靜默退回預設（CLAUDE.md 記載的地雷）。
- **判準一的刪除經 payload 反證無誤**：tasks instruction 仍含 `Red-Green-Refactor` 與 sharp-edges audit 紀律；specs instruction 仍含繁中書寫要求、中文弱語言詞表、`Use SHALL/MUST` 與結構標記保留英文——四項被我從 context／rules 刪去的內容，引擎全部照舊自動提供。
- **最終數字**：context 40 行／2787 字 → 29 行／1737 字（−38%）；rules proposal 9→9、specs 10→10、design 7→10、tasks 8→7；政策四欄與 schema 未動。
**Open**: 使用者將檢視落檔版本後再定 A 與 B 的後續方向；技能是否要把「以 `speclink instructions <artifact> --json` 反證重複」寫成必跑步驟（本輪證明這是唯一可靠的判準一執行方式）。

### Round 7 — assumptions (2026-07-27)

**Focus**: context 該不該提 Spectra、專案敘述是否過長、上游意圖的技術棧是否被漏掉？
**Position**: 使用者三點質疑中兩點成立、一點成立一半——已據此再修一輪並重驗：
- **「專案敘述過長」成立**：刪去「Speclink 另擴充 discussion、Desktop、Store abstraction、Node SDK、Client Protocol 與 Remote Platform」——這串功能列表與下方 crate 地圖完全重疊（protocol／remote／store／node／desktop 全在地圖裡）。
- **「context 應保留技術棧」成立，且是我優化時的實質偏差**：上游 OpenSpec 的 context 範例核心就是 `Tech stack:` 一行，而我上一輪以「rules.design 已有 serde 與 git 條目」為由刪去技術棧，留下專案敘述——方向反了。已補回一行技術棧，內容查自 `Cargo.toml` workspace dependencies 與各 crate／package.json 實際相依（非憑印象）：Rust 的 serde／chrono／anyhow／clap／ulid、HTTP client ureq、async tokio ＋ axum、store driver rusqlite／postgres／檔案系統、Node SDK napi-rs、桌面 Tauri；前端 React ＋ TypeScript ＋ Vite ＋ Tailwind v4 ＋ Radix UI ＋ zustand、測試 vitest；drift 與 archive 呼叫 git。
- **「不需提 Spectra」只成立一半**：`## 專案` 段的「Local CLI 以 Spectra App 2.3.1 為相容基準」確實冗餘（約束段已載明），連同 crate 地圖抄自 doc comment 的「Spectra 相容行為」一併刪去，context 內 Spectra 從 3 次降為 2 次。但**約束段的兩處必須留**：「`--json` 欄位與 Spectra 對齊」與「對 Spectra 2.3.1 的一致性是既成基線」是硬約束——它決定 `--json` 欄位名與人眼輸出不得隨意更動，對 proposal（相容性影響）、specs（parity 預期）、design（欄位命名）、tasks（回歸驗證）四者皆咬合，屬 context 的正當內容。
- **重驗**：隔離 workspace 實測 `speclink instructions proposal --change probe --json` 回 `locale=Traditional Chinese (繁體中文)`、rules 9 條，序列化解析正常、政策未退回預設。最終 context 40 行／2787 字 → **31 行／1837 字（−34%）**，政策四欄與 schema 未動。
**Ruled out**: 把約束段的 Spectra 一併刪除——那會使「既有 CLI 輸出是回歸保護對象」失去判準對象，等同放棄相容基線。
**Open**: 使用者檢視本版後再定 A 與 B 的後續方向。

### Round 8 — assumptions (2026-07-27)

**Focus**: 對齊 Spectra 的約束對現在的專案還有意義嗎？
**Position**: 使用者論點成立，且查證後發現底下藏著更嚴重的問題——那條規則本身已無法執行：
- 使用者主張：speclink 已完成，不需再對齊 Spectra；新維護者不一定安裝 Spectra。
- **查證結果比主張更嚴重**：規則要求的「parity／color／twin 對照」在 repo 中**完全不存在**——`find` 全樹無 parity_suite／color_suite／twin harness，`crates/speclink-cli/tests/` 下的 `remote_verb_parity.rs` 是遠端動詞對照、與 Spectra 無關。CLAUDE.md 記載的那三套 suite 位於 scratchpad，早已隨環境消失（記憶亦載明「scratchpad 基建會消失，勿依賴」）。也就是說，這條規則要求 AI 去跑一組不存在的測試。
- **實際存在的回歸保護**：`crates/speclink-core/tests/render_golden.rs` ＋ `tests/golden/*.snapshot.md`（技能渲染輸出）與 `crates/speclink-cli/tests/` 的 23 個整合測試檔。約束本身有效，錯的是基準來源與驗證手段。
- **修法：保約束、換基準**。context 改為「既有輸出是回歸保護對象：人眼輸出與 `--json` shape 都是既成契約；改動前後須以 golden（`cargo test -p speclink-core --test render_golden`）與 CLI 測試確認未意外變動，刻意變更則同批更新 golden 並在提案記載」，`--json` camelCase 改述為「欄位名本身是契約的一部分」。連帶修 rules 四處：specs 的「parity 敏感行為須標明對 Spectra 2.3.1 的預期」→「動到既有輸出的規格須標明維持既有位元級輸出或屬刻意變更」、specs 的「parity/color fixture」→「golden fixture」、design 的「欄位名與 Spectra 對齊」→「欄位名是對外契約」、tasks 兩條改指向 `crates/speclink-core/tests/golden` 與 `crates/speclink-cli/tests/`。
- **Spectra 已自 config.yaml 完全移除**（grep 零殘留）；正典規格中仍有 12 份、18 處提及，那是既成行為的歷史記載，不在本次範圍。
- **重驗**：隔離 workspace 實測 `speclink instructions specs --change probe --json` 回 `locale=Traditional Chinese (繁體中文)`、specs rules 10 條。最終 context 40 行／2787 字 → **32 行／1869 字（−33%）**，政策四欄與 schema 未動。
**Ruled out**: 只刪 Spectra 字樣而保留「須確認 parity／color／twin 對照」——那會留下一條指向不存在基建的死規則，比提及 Spectra 更有害。
**Open**: 使用者檢視本版後再定 A 與 B 的後續方向；CLAUDE.md 的開發備忘同樣寫著 parity_suite 31 項／color_suite 16 項／twin harness 8 情境，該一併校正（不在 config.yaml 範圍，另記）。

### Round 9 — assumptions (2026-07-27)

**Focus**: CLAUDE.md 是否同樣殘留指向不存在基建的回歸對照描述？
**Position**: 不需校正——上一輪的推斷有誤，經 grep 核實後更正：
- `CLAUDE.md:41` 實際寫的是「CLI 輸出是回歸保護對象：重構前先保存 baseline exe 做**自我基線**雙沙盒對照（scratchpad 基建會消失，勿依賴）」——基準早已是 speclink 自身前一版 exe 而非 Spectra，且已明載 scratchpad 基建不可依賴。
- `.claude/CLAUDE.md` 與 `AGENTS.md` 對 parity／color／twin／Spectra 零提及。
- 「parity_suite 31 項／color_suite 16 項／twin harness 8 情境」只存在於 `openspec/config.yaml`，已於上一輪清除。上一輪記錄中「CLAUDE.md 同樣寫著」的陳述作廢。
- CLAUDE.md 的 baseline exe 手段與 config.yaml 新寫的 golden／CLI 測試不衝突：前者是重構時的額外雙沙盒對照，後者是常規回歸驗證，兩層並存。
**Ruled out**: 為求一致而改寫 CLAUDE.md 第 41 行——它本身正確，改動只會引入噪音。

### Round 10 — assumptions (2026-07-27)

**Focus**: 依最終落檔的 config.yaml（含 Spectra 移除與回歸指向修正）回看技能設計，哪些要調整？
**Position**: 決策主體不變，三處要調——全部源自第 7–9 輪實作優化時踩到的實證：
- **固定輸入集合要擴充相依 manifest**。原清單（Cargo.toml members／package.json workspaces、README、docs 索引、既有 config.yaml、language show）只有結構資訊，查不出技術棧行——而技術棧正是上游 context 範例的核心、也是使用者裁定要保留的內容。補入：`[workspace.dependencies]`、關鍵 crate 的 Cargo.toml 相依（async／HTTP／driver 類）、各 package.json dependencies。第 7 輪的技術棧行就是這樣查出來的（tokio／axum／ureq／rusqlite／napi-rs／Tauri／React／Tailwind v4／zustand／vitest）。
- **新增第四判準：引用必須可執行**。context 與 rules 中引用的驗證手段（指令、測試名、路徑）必須實際存在於 repo，技能每次迭代都要核實。實證：本專案 config.yaml 曾要求跑不存在的 parity／color／twin suite——死規則會讓 AI 編造「已確認通過」或卡住，比內容過時更有害。核實成本低（glob／`cargo test --list` 即可）。
- **payload 反證從 Deferred 升為必跑步驟**。判準一（已自動注入者不得重述）的唯一可靠執行方式是 `speclink instructions <artifact> --json` 取實際 payload 逐條比對——本討論兩度證明憑印象會錯（誤刪語言涵蓋條、漏看 schema 內建 instruction 已含 SHALL 用法）。
- 內嵌參考範例即為最終落檔版 config.yaml（含技術棧行、Spectra 零提及、回歸指向 render_golden 與 crates/speclink-cli/tests/），無需另製。
**Ruled out**: 為技術棧掃全部相依樹——只取 workspace 層與關鍵邊界相依，版本號一律不寫（判準三）。

## Conclusion

**Decision**: 補一支 `speclink workflow-config` CLI 動詞（`show [--json]`、`set <key> <value>`、`context --stdin`、`rules <artifact> --stdin`，寫入皆可 `--dry-run` 印 unified diff），local 讀寫 `openspec/config.yaml`、remote 走既有 `client.config()` → core 的 `update_workflow_config_text` seam → `client.put_config` ＋ CAS，模式由 binding 判定、revision 不對使用者暴露（單一動詞內完成讀-改-寫，衝突交 server CAS 擋下並提示重跑）。在其上做技能 `speclink-config`：每次呼叫掃固定的結構性來源——Cargo.toml members 與 `[workspace.dependencies]`、關鍵 crate 的邊界相依（async／HTTP／driver）、package.json workspaces 與 dependencies、README、docs 索引、既有 config.yaml、`speclink language show`——產 diff 交人裁決，不自動寫入。context 與 rules 的分工由四條判準釘死：(1) 已由政策開關或 schema 內建 instruction 自動注入的，context 與 rules 皆不得重述——必跑 `speclink instructions <artifact> --json` 取實際 payload 逐條反證，禁止憑印象（本討論兩度實證印象會錯）；(2) 只對單一 artifact 咬合的降 rules；(3) 會過時的（版本號、計數、統計）不寫；(4) context 與 rules 引用的驗證手段（指令、測試名、路徑）必須實際存在於 repo，每次迭代核實（實證：本檔曾指向不存在的 parity／color／twin suite 成為死規則）。收斂驗收：同一未變動 codebase 連跑兩次，第二次 diff 為空。CLI 動詞與技能合為一刀。

**Rationale**: 關鍵不是「能不能寫入」而是「多次呼叫會不會震盪」。`context` 進四個 artifact、`rules` 只進對應 artifact 這個注入機制本身就是判準的來源；把它寫死成四條規則、且判準一與判準四都以可執行的核實步驟落地，迭代才會收斂。傳輸面（`client.config()` / `put_config` ＋ CAS）與文字改寫 seam（`config.rs:503`）皆已存在，這刀只補 CLI 側的薄編排。

**Rejected alternatives**: 技能直接寫檔（remote 永久無解）；掛進既有 `speclink config`（無 schema 自由 KV、單行 scalar 塞不下多行 context）；為此新建 Host 抽象層（消費者僅二，過度設計）；技能全 repo 掃描（輸入集合不固定必然震盪）；憑印象判斷重複（兩度實證不可靠）；為技術棧掃全部相依樹（只取 workspace 層與關鍵邊界相依，版本號不寫）。

**已完成（經使用者裁定直接落檔，不走變更流程）**: `openspec/config.yaml` 優化已寫入並經隔離 workspace 實測（引擎解析正常、政策未退回預設、自動注入內容經 payload 反證無缺）。最終：context 40 行／2787 字 → 32 行／1869 字（−33%）；rules proposal 9→9、specs 10→10、design 7→10、tasks 8→7；政策四欄與 schema 未動。內容：刪與自動注入重複者、降 rules 者、會過時者；補技術棧行（查自實際 manifest）；crate 地圖依 14 個 member 的 doc comment 重核；桌面薄包裝約束升格 design 規則；修正「政策四欄歸 .speclink.yaml」的事實錯誤；Spectra 零提及、回歸對照改指向真實存在的 render_golden 與 crates/speclink-cli/tests/。此檔即為技能的內嵌參考範例，無需另製。

**Deferred**: 正典規格中的 Spectra 提及清理——已由獨立討論 spectra-legacy-cleanup 承接。

**Capture to**: proposal（新變更，範圍為 CLI 動詞 ＋ `speclink-config` 技能；config.yaml 優化已完成不納入）

**Next**: /speclink-propose --from-discussion config-yaml-setup-skill
