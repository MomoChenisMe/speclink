---
topic: SDD engine as SDK with pluggable document storage for team scenarios
slug: sdd-engine-as-sdk-with-pluggable-document-storage-for-team-scenarios
status: promoted
promoted_to: store-trait-and-fs-adapter
created: 2026-07-03
---

# Discussion: SDD engine as SDK with pluggable document storage for team scenarios

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

Speclink 目前（承襲 openspec/spectra）的 SDD 文件、狀態、流程、產物全部跟隨 git 儲存庫（`openspec/` 目錄），偏向個人使用。目標：保持引擎邏輯不變，讓引擎可作為 SDK 供團隊系統（含 AI Agent 系統）串接，而文件的存放/管理/UI 呈現由團隊自決。需要重新思考 `init`（CLAUDE.md/AGENTS.md 寫入、skills 安裝）在各情境下的行為。

四種目標情境：
1. PO/PM 在客製化 AI Agent 系統中執行 discuss/propose/ingest/archive，change 文件存在該系統（看板呈現）；RD 在本地 git repo 用 Claude Code 執行 apply/drift/verify。
2. PO/PM/RD/QA 全部在客製化 AI Agent 系統中執行完整 SDD 流程。
3. RD/QA 在本地 git repo 用 Claude Code 執行完整 SDD 流程，但規格文件存在遠端看板系統。
4. 完全本地（現狀）：所有角色在本地 git repo 執行，change 跟隨版本控制。

模式：assumptions — codebase scout 找到大量相關原始碼（crates/speclink-core 的 paths.rs / init.rs / model.rs / skills.rs），且存在空殼 crate `crates/speclink-fs`（不在 workspace members），顯示儲存層拆分已有伏筆。相關 changes/specs：`speclink list` 與 `speclink discuss list` 皆為空，無 openspec/LANGUAGE.md。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-03)

**Focus**: 引擎與文件儲存的縫線該切在哪一層，以及 init 如何因應四種部署情境
**Position**: 四種情境可分解為兩個正交軸（引擎執行位置：本地 CLI vs 內嵌於團隊系統；文件存放位置：本地 FS/git vs 遠端系統），因此只需要一個抽象：領域層級的 `Store` trait（語彙為 change/artifact/discussion/spec，非路徑）。fs adapter 落在現成的空殼 crate `speclink-fs`，遠端走單一 HTTP 儲存契約（`speclink-remote` client + 團隊自行實作 server 端）。init 拆成 workspace init（CLAUDE.md/skills，永遠本地）與 store init（依後端而定），`.speclink.yaml` 增加 `store:` 設定。skills 已以 CLI 為介面故大致免改，但需消除殘留的直接讀檔指示。證據：crates/speclink-core/src/{paths.rs,model.rs,init.rs,skills.rs}，空的 crates/speclink-fs。
**Ruled out**: VFS 式 path-based trait — 會強迫看板/DB 後端模擬 openspec/ 目錄樹與 mtime 語意，介面太淺；多語言 FFI 綁定作為 SDK 主形式 — 維護矩陣爆炸，且情境 1/3（本地 CLI 連遠端文件）無論如何都需要網路協定，HTTP 契約一份可服務所有非 Rust 消費者。
**Open**: 遠端 store 下規格的版本控制/稽核歸屬（git 不再是版本來源）；drift 在 specs 不進 git 時的基準取得；tasks.md 勾選狀態的併發寫回；HTTP 契約粒度（檔案級 blob vs 結構化欄位）；認證機制；store=remote 時 CLAUDE.md marker 區塊內容是否要變體。

### Round 2 — assumptions (2026-07-03)

**Focus**: SDK 語言範圍（Node.js + Rust）、兩個設定檔的歸屬、SDK 情境下 CLAUDE.md/AGENTS.md/skills 的處理
**Position**: (1) SDK 首發 Rust + Node.js 合理：Rust 即 speclink-core crate 本身；Node 走 napi-rs 原生綁定（prebuild 平台矩陣成熟，支援 JS 實作 Store 的 callback 橋接），核心維持同步、無 async runtime（守住現有紅線），async 橋接放在綁定層。CLI 改為公開 core API 的第一個消費者，SDK 回傳型別直接沿用既有 --json payload（已是 camelCase）。(2) 設定檔分工規則：`openspec/config.yaml` 是「store 內的文件」（團隊共享 workflow 設定：schema/context/rules/locale fallback），透過 Store 讀取即可，遠端情境自然共享；`.speclink.yaml` 是「引擎執行端 bootstrap」（store 選擇與連線、tools、個人覆寫），SDK 消費者不需要它——host 以程式參數建構引擎。spec_dir 降級為 fs adapter 選項（移入 store: 區段）。locale 的兩層解析（app 勝、workflow 為 fallback）現有程式碼已支援，團隊情境的指引改為「團隊值放 config.yaml、個人覆寫放 .speclink.yaml」。(3) SDK 不「安裝」而是「輸出內容」：skills.rs 的 registry/render/substitute 與 init.rs 的 instructions_body/upsert_marker 都是純字串函式，SDK 直接公開為 API（render 出 SKILL.md/marker 區塊字串），安裝位置由 host 決定；workspace install（現行 init 的寫檔行為）保留為選用 helper，CLI 與管理本地 repo 的 host 系統共用。情境 2 的務實路徑：agent sandbox 裝 CLI + store: remote，skill 本體原樣可用；MCP server（基於 Node SDK）列為未來通道。
**Ruled out**: WASM 作為 Node 綁定主形式 — JS 實作 Store 的 async callback 橋接體操較差，napi-rs 的平台預編譯已是主流成熟路徑（swc/Prisma/oxc）；Store trait 改 async — 會把 tokio 拖進 core，違反「無 async runtime」紅線，且 CLI 情境不需要。
**Open**: tdd/audit 是否比照 locale 移為 config.yaml 團隊值 + .speclink.yaml 個人覆寫；skills 是否需要第三種 render 目標（Tool::Neutral，供非 claude/codex 的 agent 系統）；遠端 store 的版本控制/稽核歸屬、drift 基準、tasks 併發寫回、HTTP 契約粒度、認證（延續前輪）。

### Round 3 — assumptions (2026-07-03)

**Focus**: skill 在非 CLI 環境（如 Copilot SDK 後端）如何運作、RD 本地 init 與 OAuth 流程、尚未想到的細節盤點
**Position**: (1) 核心領悟——skill 手冊裡的 `speclink ...` 是「動詞詞彙」而非「必須開子程序」。SDK 提供 `dispatch(argv)` 入口，CLI 執行檔與 host 註冊的 tool 都是它的薄殼：Claude Code 環境 shell 跑 CLI；Copilot SDK 後端註冊名為 `speclink` 的 Tool（參數即 argv）直達 Node SDK → 自家 DB Store，skill 內容零修改即可用。Copilot SDK 另可走 MCP server（Node SDK 包裝）原生掛載。skill 需改的仍只有：消除直接讀檔殘留（前輪假設 4）＋ v2 增加中性渲染變體（去 slash 前綴/plan mode 語彙）。(2) RD 本地初始化：`speclink init --store remote --url <project-scoped-url>` 做 workspace init + store 設定（不建 openspec/ 樹）；`.speclink.yaml` 進 git（url 如 git remote），憑證永不落 repo——token 存 global_config_dir()（config.rs:61 既有機制）或 OS keyring，CI 用 SPECLINK_TOKEN 環境變數；401 提示重新登入。v1 先做 PAT（server 成本極低），device flow OAuth 後補。(3) 細節盤點（design 階段待辦）：樂觀並行控制（version/ETag + If-Match + 409，一併解決 tasks 併發寫回）、archive 交易性操作、契約的 project 範疇（多 repo 對一 store）、看板欄位必須讀引擎推導狀態而非獨立 status 欄、actor 身分從 OAuth token 傳入 Store、API 版本協商、`speclink store push/pull`（fs⇄remote 遷移/備份，雙 adapter 的免費副產品）、離線 v1 明確失敗不做快取。
**Ruled out**: 為 tool-calling 環境維護第二份 skill 本體 — 單一來源＋動詞詞彙綁定即可，雙份必然漂移；OAuth device flow 作為 v1 — server 端要多實作 OAuth endpoint，PAT 先行；離線快取 — 一致性深坑，無真實痛點前不做。
**Open**: tdd/audit 是否移為 config.yaml 團隊值＋個人覆寫（延續）；中性渲染變體的具體形狀（v2）；HTTP 契約粒度與遠端規格版本稽核歸屬（歸入 design 待辦 1/5）。

### Round 4 — assumptions (2026-07-03)

**Focus**: 驗證「Copilot SDK 自訂 Tool 承載 speclink」的可行性，並以 wadpilot 實際設計文件對齊架構
**Position**: 可行性獲雙重確認。(a) 官方：@github/copilot-sdk 已 GA（2026-06-02，npm v1.0.5，Node ^20.19||>=22.12），`defineTool("speclink", { parameters: z.object({ argv: z.array(z.string()) }), skipPermission: true, handler })` 搭 `createSession({ tools })` 是官方標準用法；tool 名限 /^[a-zA-Z0-9_-]+$/（speclink 合法）、handler 收結構化物件、回傳任意 JSON 可序列化值、無文件記載的回傳大小上限；`skillDirectories` 原生支援 SKILL.md 目錄載入——speclink 渲染出的 skills 幾乎直接對接。(b) 實證：wadpilot 生產碼已用同模式（CopilotTool 介面 + createSession({tools})，packages/server/src/agents/shared/docs/doc-tools.ts、session-runtime/session-pool.ts:407-433）。兩項修正：① MCP 路徑對 wadpilot 不成立——其自家研究（docs/sdd-research/notes/feasibility-copilot-skill.md）已否決：MCP host 建立後凍結、要冷重建才能改，另有官方 Issue #947（sub-agent 白名單靜默忽略 defineTool 自訂工具）；in-process tool 是定案路徑。② 網路契約層級修正——wadpilot 的 04-speclink-final-design.md 是一份完整的 SpecLink 嵌入設計（三端架構：規格與狀態真相在 server、code 與 git 在本地、REST+PAT 接縫；引擎在 server、本地 CLI 是 pull-only 薄 client + outbox），與本討論先前「引擎到處跑、Store 縫線即網路契約」不同。調和為**雙縫線模型**：Store trait 是引擎內部縫線（部署無關、fs adapter 用它）；團隊情境的**網路契約切在領域動詞層**（claim/bundle/done/ingest…），由團隊系統以 Node SDK 內嵌引擎提供 server 端——PM gates、原子 archive、權限與一致性必須中央執行，文件級 CRUD 契約無法承載。speclink CLI 的 remote 模式因此是動詞契約的薄 client。新缺口：wadpilot 04 的引擎範圍（PM 雙 gate、claim、單號規則、plan_ref、outbox）超出 Rust core 的 Spectra-parity 集合，需決定分層——core 提供 SDD 原語（DAG/delta/validate/tasks 解析），團隊 workflow 層由 host 自包，或 core 增設選用的 team-workflow 模組。
**Ruled out**: MCP 作為 wadpilot 的接入形式（host 凍結 + Issue #947）；文件級 CRUD 作為團隊網路契約（承載不了 server 端 gates 與原子性）；用 TS 重刻引擎（雙引擎漂移，正是本討論要避免的）。
**Open**: @speclink/engine 的分層切法（Rust core 原語 vs team-workflow 歸屬）；wadpilot 04 假設「純 Node 領域邏輯」而 napi-rs 是 native module——需與 04 的部署前提核對；.speclink.json（04）vs .speclink.yaml（Rust CLI）命名調和。

### Round 5 — assumptions (2026-07-03)

**Focus**: 三個送達管道缺口——SDK 模式下指令區塊（CLAUDE.md/AGENTS.md）與 workflow 設定（config.yaml）如何取得、.speclink.yaml 跟 repo 走時其他系統如何取得
**Position**: 內容分三類、各有載體，互不混用。(1) 流程知識（指令區塊模板 + skill 本體）**跟引擎版本走**：內嵌在發行物中（include_str!，skills.rs:63-76），不是 store 文件；本地模式由 init 寫成 CLAUDE.md/AGENTS.md marker 與 .claude/skills/，SDK 模式由 host 呼叫 instructions.render()/skills.render() 取字串後注入自己的環境——wadpilot 為 systemMessage.sections.custom_instructions（append，其自家研究證實自創 section 被 SDK 靜默丟棄）與 skillDirectories 目錄。CLAUDE.md 與 system prompt section 是同一份內容的兩種載體；init 不需網路。版本漂移由已遞延的 API 版本協商防護。(2) workflow 設定的領域物件是 WorkflowConfig，經 Store 介面 well-known 讀取；openspec/config.yaml 只是 fs adapter 的序列化細節，wadpilot Store 接 speclink_project_config 表；RD remote 模式不讀本地檔，團隊設定經動詞契約（bundle/config 端點）sidecar 帶下，保證永遠拉最新。(3) .speclink.yaml 的服務對象只有「在該 repo 啟動的 CLI」——它是 CLI 建構參數的序列化，SDK 宿主的等價物是 createEngine({...}) 建構參數而非檔案；進 git 的意義類似 lockfile（每個 clone 拿到同一份 store 綁定），個人差異用 SPECLINK_STORE_* 環境變數覆寫，遠端系統不需要也不應該讀它。
**Ruled out**: 把 skills/指令區塊做成 store 文件（會讓流程知識與引擎版本脫鉤、init 需要網路、且 store 後端得理解引擎內部資產）；SDK 宿主讀 .speclink.yaml（server 端沒有 repo，bootstrap 本來就該是建構參數）；RD remote 模式讀本地 config.yaml 副本（與團隊真相分岔，sidecar 帶下已解）。
**Open**: SPECLINK_STORE_* 環境變數覆寫的具體鍵名與優先序（design 細節）；skills export 到 skillDirectories 的落地時機（部署時 vs 執行時 materialize，design 細節）。

### Round 6 — assumptions (2026-07-03)

**Focus**: .speclink.yaml 是否該整個跟著 store 走——其中 tdd/audit/locale 是不是 SDD 專案屬性
**Position**: 設定值搬家、檔案留下。tdd/audit/locale/spec_locale 是 SDD 專案的 workflow 政策與共享產物語言，正確的家是 store 側 WorkflowConfig（fs＝openspec/config.yaml、wadpilot＝speclink_project_config 表）——這同時解掉先前遞延的「tdd/audit 歸屬」。但 .speclink.yaml 本身不能跟 store 走：bootstrap 悖論——「怎麼連到 store」不能存在 store 裡。瘦身後 .speclink.yaml 只剩 store:（type/url/dir）與 tools（宿主屬性：各工作區裝的 AI 工具不同）。純本地模式看不出問題是因為工作區==SDD 專案；團隊模式撕開這條縫（PO 網頁改 tdd、RD 本地檔不同值時，store 側團隊值必須是真相）。相容遷移：WorkflowConfig 已有 locale/spec_locale 欄位（config.rs:82-91），resolve_locale 的「app 勝、workflow fallback」順序使舊 .speclink.yaml 值繼續有效（視為 deprecated override），init 範本改寫進 config.yaml；tdd/audit 需補 WorkflowConfig 讀取＋同覆寫順序。個人/CI 覆寫一律走 SPECLINK_* 環境變數（committed 檔裡放個人覆寫自相矛盾）。與 wadpilot 04 對齊（speclink_project_config + sidecar 帶下）；04 的 tdd 兩層細分（server 管 verify gate、本地管 apply 先寫測試）留給 design 決定。Spectra 把 tdd/audit 放 app config 是單機情境的歷史包袱，此處為刻意分歧。
**Ruled out**: .speclink.yaml 整檔移入 store（bootstrap 悖論）；tools 移入 store（宿主屬性，server 端與各 RD 工作區各不相同）；個人覆寫留在 .speclink.yaml（committed 檔與個人設定語意衝突）。
**Open**: wadpilot 04 的 tdd 兩層細分（verify gate vs apply 紀律）是否採納為通用設計（design 階段）。

### Round 7 — assumptions (2026-07-03)

**Focus**: store 的 type/url 該放 committed .speclink.yaml、.git/config（如 git remote），還是其他設定層
**Position**: 維持 committed .speclink.yaml。理由：(1) 零設定 clone——store 綁定是「這個 repo 的規格真相在哪」的團隊事實，性質同 git-lfs 的 .lfsconfig 與 npm 專案 .npmrc（committed 專案級綁定），而非 git remote 那種「這個 clone 的視角」；(2) git 不同步 .git/config，放那裡等於每個 RD 手動設定且漂移無從察覺；(3) bootstrap 不能依賴 git——Paths::discover 的 walk-up 不需要 git，SDK 宿主與非 git 目錄也成立。覆寫維持兩層：committed 團隊預設 → SPECLINK_STORE_* 環境變數（個人/CI/staging）。邊界行為：外部 clone 連不上 store 時明確失敗＋提示（不靜默 fallback 到 fs——本地無規格文件，fallback 製造假真相）；團隊視內網 URL 敏感時可選擇不 commit、全隊 env 注入，機制天然支援。
**Ruled out**: .git/config 作為 store 綁定的家（不同步、加 git 依賴、clone 後需手動設定）；git config 作為第三層覆寫（env 已覆蓋所有實際情境，YAGNI）；連不上時靜默 fallback 到 fs（製造分岔假真相）。

### Round 8 — assumptions (2026-07-03)

**Focus**: bootstrap 檔改名——避免新世代瘦 bootstrap 與舊世代胖 .speclink.yaml（含 tdd/locale）同名混淆
**Position**: 使用者拍板改名 **speclink.link.yaml**（repo 根、進 git、可見檔）。名字直說「綁定/連結」本質（store 綁定 + tools，非設定值），呼應品牌；與 gitignored 的 .speclink/ 工作目錄形成「可見=團隊共享進版控、隱藏=本機私有」對比。釘死原則：fs 與 remote 模式**同一個檔、同一個 schema**（只差 store.type），絕不本地/遠端分用不同檔名——兩套 schema 才是混淆根源。遷移：pre-release 無外部使用者，硬切換——`Paths::discover` 改認 speclink.link.yaml，舊 .speclink.yaml 僅觸發明確遷移訊息（不再解析，避免半殘相容）。此決定同時解掉遞延的「wadpilot 04 `.speclink.json` vs `.speclink.yaml` 命名調和」：canonical 為 speclink.link.yaml，wadpilot 實作時對齊。
**Ruled out**: 維持 .speclink.yaml（同名混淆仍在，正是本輪要解的問題）；speclink.yaml（可見慣例佳但語意不如 link 直白，敗於使用者偏好）；speclink.config.yaml（config 暗示設定值，而設定值恰好全搬去 store 側，語意誤導）；本地與遠端分用不同檔名（schema 分岔製造真混淆）；舊檔名長期雙軌相容（pre-release 背這債不值得）。

### Round 9 — assumptions (2026-07-03)

**Focus**: speclink.link.yaml 的 tools 是否需要遠端，以及三個設定檔（config.yaml / .speclink.yaml / speclink.link.yaml）在本地與遠端的最終形狀與合理性重審
**Position**: (1) tools 不需要遠端，且第 6 輪「宿主屬性：各人工具不同」的理由修正為「repo 層級屬性」——init 生成物（CLAUDE.md/AGENTS.md/.claude/skills/）是 committed 共享檔，個人工具選擇不需設定（多套指令檔共存、各工具只讀自己的）；tools 的真正作用是讓 update 的同步/清理有確定性（init.rs:180-216 的 regenerate + prune），若為個人設定則 RD-A 的 update 會清掉 RD-B 的 AGENTS.md。server 宿主不讀 link 檔（skill 走 skillDirectories），store 無理由知道 code repo 的工具組合（一個 SDD 專案可綁多個 repo、各 repo 工具不同）。(2) 最終體系＝兩檔一目錄：speclink.link.yaml（bootstrap，committed；fs 模式可整檔缺省——walk-up 靠 openspec/ 目錄、tools 靠 detect_tools 足跡偵測；remote 模式必要因 url 無預設）、openspec/config.yaml（WorkflowConfig 的 fs 序列化，僅存在於 fs 模式；remote 模式本地無此檔、無 openspec/ 目錄，政策活在 server 的 speclink_project_config 表並經 sidecar 帶下）、.speclink/（gitignored 工作資料：本地 touched/snapshots，remote 快取/outbox）；.speclink.yaml 已退役。SDK 宿主三者皆無（建構參數 + 自家 DB）。(3) 判定規則通過逐欄位驗證：影響共享產物與流程政策→WorkflowConfig（store）；描述 repo 怎麼連線與生成哪些工具檔→speclink.link.yaml；個人/機器差異→SPECLINK_* 環境變數。弱點重審：本地雙 committed 檔之重由 link 檔可缺省化解；remote 無本地 config.yaml 是特性（留副本即與團隊真相分岔，第 5 輪已排除）。
**Ruled out**: tools 移入 store/遠端（store 與 repo 工具組合無關、server 不讀 link 檔）；tools 作為個人設定（update prune 會互相清掉對方工具檔）；remote 模式保留本地 config.yaml 副本（分岔假真相，重申第 5 輪）。
**Open**: 若採納 wadpilot 04 tdd 兩層細分，本地 knob 的落點（link 檔 vs .speclink/）——併入既有 design 待決項。

### Round 10 — assumptions (2026-07-03)

**Focus**: 反事實檢驗——若 locale/spec_locale/tdd/audit 回歸 bootstrap 檔（speclink.link.yaml），三種模式下的形狀與後果
**Position**: 回歸作為「取代」不可行，維持第 6 輪決定（canonical 在 store 側 WorkflowConfig）。逐模式檢驗：本地模式下回歸看似無問題（即 Spectra 原始佈局）——因 workspace==SDD 專案，問題被掩蓋；遠端模式下 server 端無論如何需要自己的一份（PO/PM 網頁端產 artifact 需 locale/tdd/audit，server 讀不到任何 RD repo 的 link 檔），故回歸=複製出雙真相：locale 分岔（同一 change 的網頁端與本地端 artifact 語言不一致）、tdd 分岔（多 repo 政策不一致、PM 改政策需逐 repo 發 commit）；SDK 模式下 link 檔不存在，直接不成立。結論：store 側的家省不掉，「回歸」只會複製到真相邊界兩側。但本輪挖出合法殘餘需求——repo 層級刻意分歧（如 legacy repo 暫無法跑 TDD 而專案預設 tdd:true），解法是**窄覆寫**而非搬回整組欄位：speclink.link.yaml 允許 `overrides:`（僅限 apply 階段紀律如 tdd/audit），恰為 wadpilot 04 tdd 兩層細分（server 管 verify gate、repo 管 apply 紀律）的具體形狀候選，併入既有 design 待決項。locale/spec_locale 不在可覆寫之列——remote 模式下規格與 artifact 皆活在 store，per-repo 語言分歧無合法場景，個人需求走 SPECLINK_* 環境變數。
**Ruled out**: 四欄位整組回歸 bootstrap 檔（remote 模式產生雙真相、SDK 模式不成立、store 側的家無論如何省不掉）；locale/spec_locale 的 repo 覆寫（無合法場景）。
**Open**: overrides: 窄覆寫的確切欄位集與解析順序（design 階段，與 wadpilot 04 tdd 兩層細分一併定案）。

### Round 11 — assumptions (2026-07-03)

**Focus**: 設定檔重排（取代第 8 輪的整檔改名）——.speclink.yaml 保留原名原角色，speclink.link.yaml 降為 remote 專屬連接檔，tools 回歸 .speclink.yaml
**Position**: 採納使用者提出的重排，優於第 8 輪方案。理由：(1) 政策欄位（locale/spec_locale/tdd/audit）第 6 輪已移居 store 側後，.speclink.yaml 剩下的 schema 本來就退回近原始形狀（tools + spec_dir）——第 8 輪擔心的新舊同名混淆已被搬家消解，整檔改名屬過度反應；(2) 檔案存在即模式訊號：有 speclink.link.yaml=remote、無=fs，store.type 欄位不再需要；(3) link 語意更準——連接檔只在有東西可連時存在；(4) 關注點分離——視 url 敏感的團隊可單獨 gitignore link 檔。最終形狀：.speclink.yaml（所有模式，committed，可缺省）＝tools + spec_dir(fs 佈局選項) + 未來 overrides 窄覆寫；speclink.link.yaml（僅 remote，init --store remote 生成，committed 如 .lfsconfig，round 7 決定沿用）＝url；模式解析＝walk-up 發現 link 檔→remote、否則 fs，兩者並存（遷移殘留）→link 勝出+doctor 警告。tools 回歸 .speclink.yaml（workspace 屬性，與連接無關，round 9 的 repo 層級判定不變）。不再需要硬切換——舊檔殘留政策鍵出 deprecation 警告指向 config.yaml 即可。衍生 UX：speclink link <url>／unlink 指令對，搭配 store push 完成情境 4→3 遷移閉環；bootstrap 悖論精確化為只套在 link 檔。
**Ruled out**: 第 8 輪的單一 bootstrap 檔（store: type/url/dir + tools 全在 speclink.link.yaml）＋硬切換——被本輪取代：存在即訊號更簡潔、.speclink.yaml 角色連續性更高、遷移壓力更小；link 檔內放 tools（workspace 屬性非連接屬性）。
**Open**: speclink link/unlink 指令是否進 v1（design）；link 檔 url 之外的欄位（project 範疇、API 版本提示——與動詞契約 design 一併定）。

### Round 12 — assumptions (2026-07-03)

**Focus**: tools 遇到客製 AI Agent/harness 的擴充方式，與連接檔最終定名
**Position**: (1) tools 的客製化分兩種情境，答案不同：server 端 SDK agent（情境 2）完全不經 tools——送達走 render API→systemMessage+skillDirectories（第 5 輪既定）；本地跑的客製 harness 則把 tools 從封閉枚舉（claude|codex）開放為「內建名＋自訂描述子」——`{name, skills_dir, instructions_file, invocation: cli|tool-call}`，init/update 對描述子與內建工具一視同仁（生成/同步/清理），渲染基底為 Tool::Neutral 中性變體（因此從 v2 提前為描述子的前置需求）。(2) 連接檔定名 **.speclink.remote.yaml**（使用者拍板）——隱藏檔與 .speclink.yaml 風格一致；「remote」比「link」精確說出「僅遠端模式存在」的語意；speclink link <url>/unlink 指令名保留（動詞），生成/移除該檔。模式解析不變：有 .speclink.remote.yaml→remote、無→fs、並存→remote 勝＋doctor 警告。(3) 以最終定名輸出全系統 ASCII 總覽（分層架構×兩縫線、檔案與設定體系、動詞×四情境矩陣、指令面與情境 4→3 遷移路徑），作為 propose 的視覺基礎。
**Ruled out**: speclink.remote.yaml 可見檔（我的建議，敗於與 .speclink.yaml 的風格一致性——使用者偏好）；維持 speclink.link.yaml（「link」未說出僅遠端語意）；speclink.origin.yaml（origin 在 git 指預設 remote 名而非檔案概念，誤導）；為客製 harness 等上游逐一內建 Tool 枚舉值（描述子讓工具矩陣開放，不必等上游）。
**Open**: 自訂描述子的欄位定案與驗證規則（design）；invocation: tool-call 時 skill 文字的具體措辭（與 Tool::Neutral 一併設計）。

### Round 13 — assumptions (2026-07-03)

**Focus**: 反駁檢驗——server 端（Copilot SDK）跑 propose 是否需要 .speclink.yaml 的資訊
**Position**: 需要「資訊」但不需要「檔案」——.speclink.yaml 在本地扮演的資訊角色在 server 端拆到兩個新家，且此拆解正是第 6 輪搬家的因果核心。逐項對照：propose 消耗的 context/rules/schema/locale/spec_locale/tdd/audit 全屬 WorkflowConfig 領域物件，兩端同源經 Store 讀取（本地=openspec/config.yaml、server=speclink_project_config 表）——若這些還留在 .speclink.yaml，「server 讀不到 repo 檔案」會使 server 端 propose 直接不成立，本輪反駁即為致命；skill 渲染參數（措辭/invocation）本地由 tools 落檔（供 init/update 重放生成），server 由 createEngine/render 建構參數直接供給（host 自控部署、無重放需求故不落檔）；spec_dir 在 server 無意義（Store ports 即佈局）。修正第 12 輪措辭：「server 不經 tools」精確含義=不需要檔案但需要其中的渲染參數類資訊，改走建構參數通道。三個家一句話：政策跟 store（兩端同源）、渲染參數本地落檔/server 進程式碼、佈局只存在於 fs adapter——propose 所需資訊無一失落。
**Ruled out**: server 端引入 .speclink.yaml 或其等價檔（server 無 repo、宿主 bootstrap 本應為建構參數——第 5 輪既定，且所需政策資訊已在 store 側同源可得）。

### Round 14 — assumptions (2026-07-03)

**Focus**: remote 模式下本地 workspace init 的內容正確性、情境 1/2 客製系統取得指引的明確路徑、特例掃描
**Position**: (1) 確認情境 1/3 的 RD 本地照常由 init 生成 skills 與 CLAUDE.md/AGENTS.md（workspace init 永遠本地，第 3 輪既定），但發現新缺口：instructions_body（init.rs:74-76）的路徑句「Specs live in openspec/specs/…」在 remote 模式為誤導（該路徑不存在）——**渲染矩陣須加 store 維度**：（claude|codex|自訂描述子）×（fs|remote），remote 變體的路徑句改為「文件在團隊系統，一律走 speclink CLI 動詞、絕不本地讀寫規格檔」；skills 本體動詞化後單一來源不隨 store 模式變，{{SPEC_DIR}} 替換僅 fs 有意義、動詞化後逐步移除。具名漏網案例：LANGUAGE.md 是 store 文件，discuss skill Step 0 的「直接讀 openspec/LANGUAGE.md」在 remote 讀不到，需動詞（如 speclink language show）。(2) 情境 1/2 的「CLAUDE.md 等價物」取得路徑明確化：server 無此檔案，同一份內容經 instructions.render({target,store}) → systemMessage.custom_instructions（append，不可自創 section）與 skills.render → 部署時 materialize 到磁碟 → skillDirectories 兩通道；動詞接線 defineTool("speclink")→dispatch。檔案 vs prompt section 僅載體差異。(3) 特例掃描新發現四項：monorepo/巢狀綁定（v1 明定一 repo 一綁定、最近者勝，巢狀留 v2）；一專案綁多 repo 的 change↔repo 歸屬（動詞契約需歸屬欄位，04 list_source_repos 為雛形，design 定案）；store push 衝突與歷史遷移（v1 僅允許 push 進空專案 fail loud；archive 歷史全量遷、保 @trace 血緣）；remote marker 變體（即本輪 (1)）。已覆蓋確認：PAT 失效中途（401 fail-loud + outbox）、claim 搶佔（原子、409）、版本漂移（API 版本協商同時保護 CLI 內嵌 skills 時效）。
**Ruled out**: remote 模式沿用 fs 版 marker 內容（路徑句誤導 agent 去讀不存在的檔案）；巢狀綁定 v1 就支援（複雜度不值，先明定單一綁定）。
**Open**: speclink language show 等 store 文件動詞的完整清單（design 盤點所有 skill 的直接讀檔點）；change↔repo 歸屬的契約形狀（design）。

### Round 15 — assumptions (2026-07-03)

**Focus**: 通用性審計（非 wadpilot 專案的適用性）與遠端 store 下的專案↔repo 識別
**Position**: (1) 通用性審計：Store trait／動詞契約／render 矩陣／設定三分皆通用（wadpilot 僅為證據來源）；「in-process 優於 MCP」降級為 wadpilot 特例——通用規則為「引擎出 render+dispatch，綁定方式由宿主選（CLI 子程序/in-process tool/MCP 皆合法殼）」；sidecar 通用化為「動詞契約必須有 config/bundle 端點供薄 client 拉政策」。抓到兩個通用性缺口：**缺口一＝gate 必須 per-project 政策可配置**（零 gate/一 gate/自訂角色都要成立，餵給已遞延的引擎分層切法）；**缺口二＝非 Node/非 Rust 團隊系統無內嵌路徑**——解法為 `speclink serve` 參考伺服器（headless 執行檔，內嵌引擎＋可插拔儲存 fs/Postgres，對外即動詞契約；非 Node 系統當 sidecar 跑、自家 UI 讀同一契約；紅利＝無客製系統的團隊可直接跑情境 3），列 roadmap（幾乎是 dispatch＋契約＋既有 store 的免費副產品）。(2) 專案↔repo 識別三層：層 1 repo→專案＝連接檔 url 含專案範疇（既定）；層 2 repo 身分＝連接檔新增 `repo:` 欄位（此 repo 在專案內的註冊名，server 維護 repos 註冊表並驗證 repo∈專案，單 repo 專案可缺省；git remote URL 僅供 doctor 比對警告——鏡像/fork 使 URL 推斷不可靠，不做強制）；層 3 change→repo＝**v1 一 change 一 repo**（propose 時宣告、claim/bundle 帶 repo 範疇、repo 不符時 CLI fail loud 防呆），跨 repo 需求走 discuss 拆分計畫產多 change（通用採納 wadpilot 04 的 plan_ref 藍圖）——一併解掉第 14 輪遞延的 change↔repo 歸屬。
**Ruled out**: 把「in-process 優於 MCP」寫成通用規則（僅 Copilot SDK 的 host 凍結約束）；非 Node 系統自行實作動詞契約（＝重刻引擎，早已排除）；change 跨 repo 的複雜歸屬模型（用一 change 一 repo＋plan 拆分消掉問題）；以 git remote URL 推斷 repo 身分作為強制驗證（鏡像/fork 不可靠，僅作 doctor 警告）。
**Open**: speclink serve 的儲存後端範圍（fs 之外是否 v1 就含 Postgres）與其認證中介（roadmap 定案）；repos 註冊表的管理動詞（design）。

### Round 16 — assumptions (2026-07-03)

**Focus**: speclink serve 降級為「遇到再做」，與 repo 識別三層的具體運作流程
**Position**: (1) speclink serve 參考伺服器從 roadmap 降級為明確遞延（YAGNI）——現有具體消費者只有 wadpilot（Node 內嵌路徑已足），等真實的非 Node/非 Rust 需求出現再做；設計上仍保留其可行性（dispatch＋動詞契約＋既有 store 使其近乎免費副產品），但不進任何版本承諾。(2) repo 識別的運作定案：設定期——init/link 時以 --repo 宣告註冊名寫入 .speclink.remote.yaml，CLI 當下即向 server 驗證 repo∈專案註冊表（不在→失敗並列可用名單；單 repo 專案可省略、server 回填）；propose 期——change 於 proposal Impact 宣告 repo 歸屬（單 repo 自動預設、多 repo 由 PM 選或 AI 推斷、server 驗證）；apply 期——每個動詞自動攜帶 repo，server 驗證鏈＝PAT→repo∈註冊表→change.repo==宣告 repo→原子 claim，list 依 repo 過濾，跑錯 repo 時 fail loud 附語義化訊息；跨 repo 需求以 discuss 拆分計畫產出各自單 repo 的 changes（plan 血緣追蹤）；git remote URL 僅 doctor 輔助警告（fork/鏡像不擋）。使用者體感：身分宣告一次、之後每動詞自動攜帶、僅走錯 repo 時被擋。
**Ruled out**: speclink serve 進 roadmap/版本承諾（無真實消費者，遇到再做）。

### Round 17 — assumptions (2026-07-03)

**Focus**: 遞延項定案——@speclink/engine 分層切法（team-workflow 歸 core 還是 host）
**Position**: 採「選用模組」而非二選一，判定規則：多人一致性所需的裁決邏輯歸引擎、持久化/身分/通知/呈現歸 host。具體：speclink-core 維持 SDD 原語不動（情境 4 只需這些）；新增選用 crate speclink-team（未來第⑤刀）承載團隊狀態機（狀態與轉移裁決）、gate 政策評估（per-project 設定決定哪些轉移需核准）、claim/ownership 規則（原子語意與 409 reason 判定），持久化經擴充的 team store ports 由 host 實作，單號經 IdGen port、通知/outbox/看板呈現全歸 host。理由：(1) 動詞契約承諾的語意（409 reason、claim 原子、狀態轉移）若由各 host 重刻必逐家漂移——裁決進引擎、host 只養資料；(2) 純本地 CLI 不載入 team 模組，情境 4 零負擔；(3) gate 政策化自然落點——政策是資料、狀態機是邏輯，為「看板讀引擎推導狀態」原則的延伸；(4) 通過深度/刪除測試（藏住轉移裁決與競態語意，非 pass-through），並滿足 wadpilot 04 對 engine 含狀態機的期待（其 PG adapter 僅多實作幾個 ports）。時序：不現在開第⑤刀——契約文件（change ③ 任務 1.1）先把狀態與 reason 語意寫成正典，speclink-team 等 wadpilot server 端開工時以其為第一個消費者共同成形。附帶：本輪前已依驗證核對結果對 change ③ 執行 ingest，補上 git remote 參考值輔助警告（remote-connection spec）與 v1 一 change 一 repo 歸屬規則（verb-contract spec）兩條明文需求。
**Ruled out**: team-workflow 全歸 host 各自實作（契約語意逐家漂移，引擎唯一理念破功）；team-workflow 直接長進 core（情境 4 被迫背上狀態機、違反核心精簡）；現在即開 speclink-team change（無消費者先蓋抽象＝過度設計，等 wadpilot server 端開工）。

## Conclusion

**Decision**: 將 speclink 重構為「引擎—Store—呈現」三層分離，並採**雙縫線模型**：(1) `Store` trait 是引擎內部縫線（領域語彙 change/artifact/discussion/spec/WorkflowConfig，同步、非 async）——`speclink-fs` 實作現行 openspec/ 佈局（預設，情境 4 行為不變）；(2) 團隊情境的**網路契約切在領域動詞層**（claim/bundle/done/ingest 等 REST + PAT，含 config/bundle 端點供薄 client 拉政策），由團隊系統以 SDK 內嵌引擎提供 server 端——**gate 為 per-project 政策可配置**（零/一/多 gate 與角色皆成立，非寫死 wadpilot 雙 gate）；中央治理（gates／原子 archive／權限）是動詞契約存在的理由；speclink CLI 的 remote 模式是契約的 pull-only 薄 client。SDK 首發 Rust（core crate）+ Node.js（napi-rs），統一入口 `dispatch(argv)`；**綁定方式由宿主自選**——CLI 子程序、in-process tool（Copilot SDK 已雙重驗證，wadpilot 因 host 凍結約束選此）、MCP server 皆為合法殼；非 Node/非 Rust 宿主的 `speclink serve` 參考伺服器**明確遞延（YAGNI，遇到真實需求再做）**。**內容送達三分**：流程知識內嵌發行物跟引擎版本走（本地 init 寫檔、SDK 宿主 render API 注入 systemMessage 與 skillDirectories）；指令區塊渲染矩陣＝（claude|codex|自訂描述子）×（fs|remote），remote 變體路徑句改為「文件在團隊系統，一律走 speclink 動詞」，skills 動詞化後單一來源（含 LANGUAGE.md 等 store 文件動詞化）；workflow 政策全數歸 store 側 WorkflowConfig（schema/context/rules/locale/spec_locale/tdd/audit；本地不留副本）；bootstrap 跟宿主。**設定檔（兩檔一目錄）**：`.speclink.yaml`＝workspace 設定（tools 開放自訂描述子＋spec_dir＋未來 overrides），可缺省；`.speclink.remote.yaml`＝remote 專屬連接檔——url（含專案範疇）＋ `repo:` 註冊名，檔案存在即模式訊號，並存時 remote 勝＋doctor 警告；`.speclink/`＝gitignored 工作資料。**repo 識別三層運作**：init/link 時 --repo 宣告並即時向 server 註冊表驗證（單 repo 專案可缺省、server 回填）；change 於 propose 宣告 repo 歸屬（**v1 一 change 一 repo**）；每個動詞自動攜帶 repo，server 驗證鏈＝PAT→repo∈註冊表→change.repo 相符→原子 claim，list 依 repo 過濾，跑錯 repo fail loud；跨 repo 走 discuss 拆分計畫（採納 04 plan_ref 藍圖）；git remote URL 僅 doctor 警告。多 repo 綁定規則：v1 一 repo 一綁定（巢狀留 v2）。`init` 拆 workspace init（永遠本地）與 store init（僅 fs）；link/unlink＋store push/pull 完成情境 4→3 遷移（v1 僅 push 進空專案，archive 歷史全量遷、保 @trace）。認證 v1 PAT（keyring；CI 用 SPECLINK_TOKEN），device flow 後補；憑證永不落 repo。
**Rationale**: 四種部署情境分解為兩正交軸（引擎執行位置 × 文件存放位置），單一 Store 抽象覆蓋文件端；「speclink 指令是動詞詞彙而非子程序」讓流程知識與執行環境解耦；網路契約切動詞層因團隊情境必然需要 server 端治理（wadpilot 04 為證據）。通用性以「引擎出 render+dispatch、綁定宿主自選、gate 政策化」保證——wadpilot 是第一個參考實作，不是依賴。設定歸屬三分：政策跟 store、綁定跟 repo、個人差異跟環境變數。repo 身分「宣告一次、動詞自動攜帶、逐次驗證」，使用者僅在走錯 repo 時感知。
**Rejected alternatives**: VFS 式 path-based trait；文件級 CRUD 契約；FFI 多語言綁定為 SDK 主形式；WASM 為 Node 主形式；async Store trait；TS 重刻引擎；第二份 skill 本體；skills/指令區塊做成 store 文件；SDK 宿主讀 bootstrap 檔；連接設定移入 store；store 綁定放 .git/config；tools 移入 store 或連接檔；逐一內建 Tool 枚舉值；locale/tdd/audit 回歸 workspace 檔；locale/spec_locale 的 repo 覆寫；個人覆寫留 committed 檔；RD remote 讀本地 config.yaml 副本；連不上靜默 fallback；單一 bootstrap 檔＋整檔改名；連接檔名 link/remote 可見/origin（定名 .speclink.remote.yaml）；remote 沿用 fs 版 marker；巢狀綁定 v1；「in-process 優於 MCP」作為通用規則（僅 wadpilot 特例）；非 Node 系統自行實作動詞契約（重刻引擎）；speclink serve 進 roadmap/版本承諾（YAGNI，遇到再做）；change 跨 repo 複雜歸屬模型（一 change 一 repo＋plan 拆分）；git remote URL 強制驗證 repo 身分；device flow 作 v1；離線快取。
**Deferred**: design 待辦——@speclink/engine 分層切法（含 gate 政策化形狀，對照 04）；napi-rs 與 04「純 Node」前提核對；動詞契約完整動詞集與 response 形狀＋連接檔欄位＋repos 註冊表管理動詞；store 文件動詞完整清單（盤點 skill 直接讀檔點）；樂觀並行控制；archive 交易性；看板 UI 讀推導狀態；actor 身分傳入；API 版本協商；store push/pull 與 link/unlink 是否進 v1；離線明確失敗；overrides 窄覆寫欄位集（與 04 tdd 兩層一併）；tools 描述子驗證與 tool-call 措辭（與 Tool::Neutral 一併）；SPECLINK_* 鍵名優先序；skills export 落地時機；monorepo 巢狀（v2）；speclink serve（明確遞延，遇到真實非 Node 需求再評估）。
**Capture to**: proposal（範圍、四情境、非目標）＋ design（雙縫線、動詞契約、SDK 三層、送達三分、兩檔一目錄、渲染矩陣、tools 描述子、repo 識別三層、gate 政策化、init/auth；與 wadpilot 04 對齊）＋ 中英雙語使用文件（README 引用）
**Next**: /speclink-propose --from-discussion sdd-engine-as-sdk-with-pluggable-document-storage-for-team-scenarios
