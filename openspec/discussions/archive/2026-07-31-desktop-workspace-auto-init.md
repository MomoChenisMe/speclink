---
topic: desktop 開啟已有 openspec/ 的資料夾不會補裝 speclink 指令檔——如何判定專案已啟用 speclink
slug: desktop-workspace-auto-init
status: promoted
promoted_to: desktop-instruction-staleness-prompt, desktop-enable-speclink-prompt, workflow-config-surgical-write
created: 2026-07-31
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: desktop 開啟已有 openspec/ 的資料夾不會補裝 speclink 指令檔——如何判定專案已啟用 speclink

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在 desktop 以「開啟專案」選擇本機資料夾後，發現既不會自動 init 也不會補裝指令檔——資料夾是從其他體系（Spectra／上游 OpenSpec）遷移而來，`openspec/` 目錄已存在但 speclink 技能與 CLAUDE.md/AGENTS.md 受管區塊從未安裝。探測（`apps/desktop/core/src/project.rs` 的 `open_project_at`）只要看到 `openspec/` 目錄就判為 project 直接進看板，零檢查零寫入；init 確認框僅在 Uninitialized 態觸發。

模式：assumptions——掃到充分的相關原始碼（`apps/desktop/src/store.ts` 的 openProjectAt 分流、`apps/desktop/core/src/project.rs`、`crates/speclink-core/src/workspace.rs` 的 discover、`crates/speclink-core/src/init.rs` 的 init/store_init/workspace_init/reconcile_builtin_tools、`apps/desktop/src-tauri/src/connections.rs` 的 checkout reconciliation）。

相關 change：`desktop-instruction-staleness-prompt`（in-progress，0/19 未動工）——涵蓋「已安裝但過期」的偵測與提示；其 spec 明文「指令檔不含 SPECLINK 標記＝視為退出受管、跳過不提示」，「檔案從未存在」無明文 scenario。本討論處理的是更上游的「從未啟用」缺口。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-31)

**Focus**: 使用者遇到的實際情境走的是哪條分流，缺口的真正位置在哪
**Position**: 缺口確認＝probe 把「有 `openspec/` 的資料夾」一律判為 project 直接進看板，遷移中的專案被靜默吞掉：
- 使用者情境：資料夾從其他體系（Spectra／上游 OpenSpec）遷移，`openspec/` 已在、技能與指令檔從未安裝
- `Workspace::discover`（crates/speclink-core/src/workspace.rs:46）：`openspec/` 目錄存在即命中 workspace，即使無 `.speclink.yaml`
- `openProjectAt`（apps/desktop/src/store.ts:1420）：project 態直接 enterProject，對指令檔零檢查
- 全新資料夾其實會跳 init 確認框（Uninitialized → pendingInit → init_project_at）——「不會自動 init」的真正主體是 project 態
- 進行中的 desktop-instruction-staleness-prompt 只涵蓋「已裝但過期」，且無標記＝視為退出受管，接不住「從未安裝」
**Ruled out**: 「init 確認框壞掉」——確認框僅在 Uninitialized 態觸發，此情境屬規格缺口而非 bug；「remote checkout 選集為空」——使用者走的是本機開啟流程，與 remote chooser 無關
**Open**: 以什麼標記判定「已啟用 speclink」（使用者提議 openspec/config.yaml）；補裝流程的形狀（提示確認 vs 靜默寫入）；與 staleness change 的關係（ingest 擴充 vs 獨立 change）；「已啟用但指令檔缺失」的 clone 情境是否一併處理

### Round 2 — assumptions (2026-07-31)

**Focus**: 「已啟用 speclink」的判定標記該用哪個檔
**Position**: 判準定案＝根目錄 `.speclink.yaml` 是否存在，使用者同意：
- `.speclink.yaml` 已是 `Workspace::discover` 首要判定鍵、記錄 tools 清單（補裝所需的唯一資訊）、fs／remote 兩模式通用
- 分流設計：`.speclink.yaml` 在 → project 照舊；`openspec/` 在但 `.speclink.yaml` 不在 → desktop probe 新增第四態「未啟用」→ 跳「啟用 speclink」確認框 → 確認後只補工作區檔（`.speclink.yaml`＋技能＋指令檔受管區塊），不動既有 openspec/ 內容
- 不改 core `discover` 的 openspec/ fallback（CLI 相容性依賴），第四態只加在 desktop probe 層
- 引擎素材已存在：`store_init` 冪等（write_if）、`reconcile_builtin_tools`＝寫 tools＋整套再生受管檔；只缺一個繞過「Already initialized」bail 的 adopt 入口
**Ruled out**: `openspec/config.yaml` 作為判準——語意是團隊工作流政策而非「工作區已接上 speclink」，可合法缺席（政策退回預設），remote 模式無本地 openspec/；改 core discover 語意——破壞無 `.speclink.yaml` 舊專案的 CLI 相容性
**Open**: 與 staleness change 的關係（ingest 擴充 vs 獨立 change）；「`.speclink.yaml` 已 commit 但技能檔沒進版控」的 clone 情境是否一併處理；使用者新提出 config.yaml 政策鍵被附加在檔尾的寫入位置問題（本輪後展開）

### Round 3 — assumptions (2026-07-31)

**Focus**: config.yaml 政策鍵被附加在檔尾——修「插入位置」還是修整個寫入接縫
**Position**: 選 B＝把 `update_workflow_config_text`（crates/speclink-core/src/config.rs:527，desktop 設定頁與 CLI config set 共用的唯一接縫）改為文字層手術，使用者同意：
- 成因：現行把整份文件 parse 成 serde_yaml::Mapping 再 insert，缺鍵一律附加尾端；且整份重新序列化會抹掉全檔註解與空行（doc comment 明載「Template comments are lost」）——使用者「不動自加內容」的要求在含註解檔案上只有文字層做得到
- 缺鍵插入位置＝`schema:` 行之後，按範本正典序（locale、spec_locale、tdd、audit）成連續區塊，區塊與下一內容間空一行；`schema:` 不存在時插檔案最頂端
- 既有鍵不搬家、原位改值——重排使用者檔案佈局也算「動到」
- 可行性：四個政策鍵都是頂層單行 scalar；context／rules 編輯語意本來就是整塊替換（Set／Remove），以 `^\S` 頂層鍵為界分段拼接即可，不需通用 YAML 註解保留引擎
**Ruled out**: 方案 A（Mapping 重排序＋後處理塞空行）——位置能修但註解仍全滅，違反「不動使用者內容」；維持現狀——就是使用者回報的缺陷
**Open**: 三個缺口怎麼打包成 changes（未啟用偵測、config 寫入接縫、staleness 探測的「缺失」態）；clone 情境（.speclink.yaml 已 commit 但指令檔沒進版控）要不要現在處理

### Round 4 — assumptions (2026-07-31)

**Focus**: 三個缺口怎麼打包成 changes
**Position**: 按接縫歸屬打包為「兩個新 change＋一個 ingest」，使用者同意：
- 未啟用偵測＋啟用確認＋adopt 入口 → 新 change：判定機制（.speclink.yaml 存在與否）與 staleness 的版號比對完全不同，架構上靠近既有 init 確認框而非 staleness 非阻斷提示
- config.yaml 寫入接縫文字層手術 → 新 change：不同 capability（設定寫入 vs 開專案流程），與其他兩者零依賴，可先落地
- 「已啟用但指令檔缺失」（clone 情境）→ ingest 進 desktop-instruction-staleness-prompt：staleness 探測加「缺失」回報態，「檔案不存在（從未安裝→提示補裝）」與「檔案在但標記被移除（退出受管→不提示）」明文區分；該 change 未動工，spec 措辭此時改零成本，等實作完再改就是二次 drift
- 本輪修正第 1 輪的假設 4（「擴充 staleness change 而非開新 change」）——該假設形成於遷移情境揭露之前，設計移到 probe 第四態後接縫重疊縮小，獨立 change 更乾淨
**Ruled out**: 把未啟用偵測塞進 staleness change——兩個可獨立驗收的機制被綁死、該 change 已 19 tasks 且明文排程靠後；defer 缺失態——錯過 spec 未動工的零成本修改窗口
**Open**: （無——全數收斂，進結論）

## Conclusion

**Decision**: 三個缺口、三個去處：
1. 「已啟用 speclink」判準＝根目錄 `.speclink.yaml` 是否存在。desktop probe（open_project_at）新增第四態「未啟用」（`openspec/` 在、`.speclink.yaml` 不在，涵蓋自 Spectra／上游 OpenSpec 遷移的專案）→ 跳「啟用 speclink」確認框（沿用 init 確認框慣例）→ 確認後 adopt：只補工作區檔（`.speclink.yaml`＋技能＋指令檔受管區塊），不動既有 openspec/ 內容。core `discover` 的 openspec/ fallback 不動（CLI 相容）。→ 新 change
2. `update_workflow_config_text` 改文字層手術：缺鍵按範本正典序（locale、spec_locale、tdd、audit）插在 `schema:` 行之後成連續區塊、與下一內容空一行、schema 缺席時插檔案頂端；既有鍵原位改值不搬家；全檔註解、空行、使用者內容原樣保留。→ 新 change
3. staleness 探測新增「缺失」回報態：tools 清單宣告的指令檔不存在＝從未安裝→提示補裝；與「檔案在但標記被移除＝退出受管→不提示」明文區分。→ ingest 進 desktop-instruction-staleness-prompt

接縫深度（interface depth check）：probe 第四態隱藏「啟用判定」、adopt 入口隱藏「繞過 Already initialized bail 的冪等組合」（store_init＋reconcile_builtin_tools 素材已存在）、文字層手術隱藏「保註解的 YAML 編輯」——各接縫皆單一 adapter；刪除測試：無第四態則遷移專案靜默進看板無技能，無文字層則設定頁每次存檔毀註解。
**Rationale**: 判準必須能區分「speclink 自己的工作區」與「同名 openspec/ 目錄的他家體系」——`.speclink.yaml` 是唯一 speclink 專屬、記錄 tools 清單（補裝所需的全部資訊）、fs／remote 通用的標記。三缺口分屬三個不同接縫（probe 分流、config 寫入、staleness 探測），按接縫歸屬打包，避免把可獨立驗收的機制綁進同一個 change。
**Rejected alternatives**: `openspec/config.yaml` 作為啟用判準（語意是團隊工作流政策、可合法缺席、remote 模式無本地 openspec/）；改 core discover 語意（破壞無 .speclink.yaml 舊專案的 CLI 相容）；方案 A Mapping 重排序（位置能修但註解仍全滅）；未啟用偵測併入 staleness change（判定機制不同、綁死驗收、該 change 已 19 tasks 排程靠後）；缺失態 defer（錯過 spec 未動工的零成本窗口）
**Deferred**: 遷移專案的內容格式相容性（Spectra／上游 OpenSpec 的 changes/specs 實質轉換，屬 onboard 語意，adopt 只補工作區檔不碰內容）；CLI 端的未啟用提示（CLI 在 bare openspec/ 下照舊運作，僅 desktop 加第四態）
**Capture to**: proposal ×2（本討論轉出兩個新 change）＋ desktop-instruction-staleness-prompt 的 spec（經 ingest）
**Next**: /speclink-ingest desktop-instruction-staleness-prompt → /speclink-propose --from-discussion desktop-workspace-auto-init（依序兩次）
