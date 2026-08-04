## Why

多個無相依的 changes 想並行 apply 時，多個 agent 共改同一份工作樹會互踩——本 repo 已實際發生過 archive 的 @trace 記錄被平行 session 髒檔污染的事故。討論 worktree-parallel-apply 定案引入 git worktree 隔離：每個並行 change 一個 worktree、desktop 維持單一專案身分並即時觀察進度。本刀為拆分後的第一刀，落地引擎與技能面（聚合讀、worktree 政策欄位、兩個技能與詞彙例外）；desktop 的即時 overlay 與卡片標示屬第二刀（worktree-live-board，將另行轉出）。

目標使用者是透過 AI 代理跑 SDD 的開發者。情境對應 apply 階段（以 speclink-apply-with-worktree 技能在隔離 worktree 中並行實作）與收尾階段（以 speclink-worktree-merge 技能把完工分支合併回主分支）。

## What Changes

- **config 新增 worktree 政策欄位**：`openspec/config.yaml` 新增布林欄位 worktree（預設缺席＝關閉），納入既有工作流政策的四層解析——SPECLINK_WORKTREE 環境變數覆寫 ＞ .speclink.yaml 舊鍵相容層（不適用此新欄位，該處的 worktree 鍵不生效）＞ config.yaml ＞ 內建預設 false。workflow-config set 受理 worktree 鍵、workflow-config show（人眼與 `--json`）呈現該欄位。政策 wire 為整份文件流通（content 字串＋revision），protocol 與 server 端點零改動。
- **引擎聚合讀（僅 local workspace 的主 checkout）**：workspace 概念擴為「主 checkout ＋ linked worktrees」。discovery 由 Host 層以 git worktree list --porcelain 取得名冊，依分支命名慣例 speclink/<change名> 對應到未封存的 change；映射成立的 change，list 的任務計數、狀態與開工戳記改讀該 worktree 副本，使主資料夾的看板與 CLI 列表即時反映各 worktree 進度。git 本身是唯一登記簿——不新增任何持久化儲存；worktree 移除後聚合讀自動退場；政策關閉時完全不介入。remote workspace 不套用（task 狀態本就集中於 TeamStore、看板天生即時）。
- **新技能 speclink-apply-with-worktree**：生成期組裝——以 worktree 前置段（讀政策拒跑、preflight、以分支 speclink/<change名> 在 repo 外 sibling 巢 <repo資料夾名>.worktrees/<change名>/ 建立或沿用 worktree、印建置成本提示）與收尾段（在 worktree 內 commit、停在合併之前並指向 worktree-merge 技能）包住共用的 apply 本體模板，輸出自包含的 SKILL.md。claude 與 codex 兩種工具目標都生成。
- **新技能 speclink-worktree-merge**：人觸發、agent 執行的收尾配對技能——preflight（主樹乾淨、worktree 分支已 commit）→ 合併回主分支 → 衝突即停等人裁決 → 成功後移除 worktree 並刪除分支。
- **詞彙明文例外**：`openspec/LANGUAGE.md` 記入「worktree」直出為工程詞明文例外（先例：config.yaml 頁簽、討論 slug），供第二刀的使用者可見文案引用。
- **技能清單與契約文件同步**：CLAUDE.md／AGENTS.md 的 SPECLINK 注入區塊新增兩個技能的使用時機條目（注入內容源於引擎的 init 模組，隨技能再生機制落地）；`docs/verb-contract.md` 增列 list 的 worktree 欄位契約。

**相容性影響**：

- 無新增 CLI 子指令。既有指令的刻意擴充有二：list 的人眼輸出對映射成立的 change 行尾追加「 [worktree]」標示、`--json` 條目新增可空 worktree 物件欄位（camelCase，含 path 與 branch）；workflow-config set 受理新鍵 worktree、show 輸出新增該欄位——golden 與 CLI 整合測試同批更新。
- 未啟用政策、無任何合乎慣例的 worktree、或於 remote 模式時，list 輸出與既有行為位元級一致（聚合讀零介入）；remote 條目恆無 worktree 欄位，形狀一致性以「可空且缺席不序列化」維持。
- `.speclink.yaml` 不受理新欄位（政策正典歸 `openspec/config.yaml`，與 tdd／audit 的棄用方向一致）；protocol 與 server 端點零改動，舊 client 不受影響。

## Non-Goals

- 不做 desktop 端任何改動：watch 擴充、卡片 worktree 標示、抽屜分支資訊、產出政策區段的 GUI toggle——全部屬第二刀 worktree-live-board。
- 不做並行時機的自動偵測（lockfile／heartbeat）：何時並行由人以呼叫技能表達。
- 不做每次 apply 都開 worktree：worktree 只在人決定並行時使用。
- 不新增 worktree 名冊的持久化儲存：git worktree list 是唯一 discovery 來源。
- 不做 remote workspace 的聚合讀：remote 的 task 狀態集中於 TeamStore、SSE 天生即時，worktree 僅屬程式碼隔離。
- 不動 archive／snapshots 流程：archive 永遠在主 checkout 執行，本刀不改其行為。
- 不做 list 以外動詞（status、show、drift 等）的 overlay，也不做 desktop 卡片上的 merge 按鈕（後續視需求）。

## Capabilities

### New Capabilities

- `worktree-overlay`: 引擎聚合讀——worktree discovery 慣例（分支命名 speclink/<change名>、git 名冊、三條件映射與 fail-open）、有活躍 worktree 的 change 改讀 worktree 副本、list 人眼與 `--json` 的可觀察輸出、僅 local 主 checkout 適用的邊界。
- `worktree-apply-skill`: speclink-apply-with-worktree 技能——生成與組合（包住 apply 本體）、政策拒跑與 worktree 建立慣例的前置指示、停在合併前的收尾指示。
- `worktree-merge-skill`: speclink-worktree-merge 技能——生成，與 preflight／合併／衝突即停／清理交棒的流程指示。

### Modified Capabilities

- `workflow-config`: 工作流政策欄位清單增 worktree——四層解析、SPECLINK_WORKTREE 環境覆寫、workflow-config set／show 的受理與呈現。
- `verb-contract`: list 動詞 `--json` payload 新增可空 worktree 物件欄位的形狀契約（僅 fs 模式主 checkout 出現、remote 恆缺席）。

## Impact

- Affected specs: 新增 worktree-overlay、worktree-apply-skill、worktree-merge-skill；修改 workflow-config、verb-contract
- Affected code:
  - New: `crates/speclink-core/assets/skills/apply-worktree-pre.md`、`crates/speclink-core/assets/skills/apply-worktree-post.md`、`crates/speclink-core/assets/skills/worktree-merge.md`、`crates/speclink-host/src/worktree.rs`
  - Modified: `crates/speclink-core/src/config.rs`、`crates/speclink-core/src/listing.rs`、`crates/speclink-core/src/skills.rs`、`crates/speclink-core/src/init.rs`、`crates/speclink-host/src/lib.rs`、`crates/speclink-cli/src/commands.rs`、`crates/speclink-core/tests/golden`、`crates/speclink-cli/tests`、`docs/verb-contract.md`、`openspec/LANGUAGE.md`、`CLAUDE.md`、`AGENTS.md`
  - Removed: （無）
