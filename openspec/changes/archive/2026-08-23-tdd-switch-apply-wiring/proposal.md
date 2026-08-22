## Summary

修復 TDD／audit 政策開關在 apply 消費端的四處接線漂移：instructions apply payload 直接帶引擎解析完的有效值，技能資產不再叫 agent 自讀設定檔，並移除 .speclink.yaml 舊政策鍵相容層。

## Motivation

目標使用者是透過 AI 代理跑 SDD 的開發者，情境是 apply 階段（/speclink-apply 與 worktree 版）。政策開關（tdd、audit）的正典已搬到 openspec/config.yaml，但 apply 技能資產仍叫 agent「Read `.speclink.yaml`」判斷開關——本 repo 的 .speclink.yaml 已無政策鍵，照字面執行 TDD 紀律會靜默失效，只靠專案 CLAUDE.md 的硬規定兜住。tdd.md 資產同樣殘留舊路徑字句，且自述可 standalone 呼叫（`/speclink:tdd`）但引擎從不渲染該技能檔，入口是死文字。另外 .speclink.yaml 舊鍵相容層（含 deprecation 警告）在第一個正式版發布時即無任何歷史使用者可相容，屬死重。（源自討論 tdd-switch-apply-wiring 的結論。）

## Proposed Solution

1. **payload 帶有效值**：`speclink instructions apply --json` 的 payload 新增 `tdd` 與 `audit` 兩個布林欄位（camelCase），值為引擎解析完的有效政策值，與既有 `locale` 欄位同機制；wire contract（protocol 的 ApplyInstructions）同步增欄，fs 與 remote 模式輸出一致。舊 server 缺欄位時 client 端反序列化失敗即 fail closed（比照 Progress 寫碼計數欄位「不設 serde default」的先例——預設 false 會靜默關掉 TDD，正是本次要修的病）。
2. **apply.md 改依 payload**：步驟 5「Check project preferences」改為讀 apply instructions payload 的 `tdd`／`audit` 欄位，移除「Read `.speclink.yaml`」字句；TDD 與 audit 紀律的觸發內文不變。
3. **tdd.md 清理**：移除「set in `.speclink.yaml`」過時字句，並刪除 standalone 模式描述（Usage Modes 段與 `/speclink:tdd` 入口）——TDD 定位為 apply 的內嵌紀律，經 `speclink instructions --skill tdd` 取得。
4. **移除舊鍵相容層**：政策欄位（locale、spec_locale、tdd、audit）的解析由四層縮為三層：環境變數 ＞ openspec/config.yaml ＞ 內建預設；.speclink.yaml 的政策鍵一律不生效（與既有 worktree 鍵行為一致），deprecation 警告機制整組移除。
5. **連帶字句修正**：ingest.md、propose.md、onboard.md 資產中「spec_locale 可設於 .speclink.yaml」的字句改為僅 openspec/config.yaml；docs/configuration 兩語言版的舊鍵相容說明移除。

## Non-Goals

- 不渲染 standalone tdd 技能檔（討論已裁定 YAGNI；debug 端走 TDD 交由各專案自備技能）
- 不改 TDD／audit 紀律本身的內容（Red→Green→Refactor、Example 轉首批測試、sharp-edges 檢查表皆不動）
- 不改 `instructions tasks` 端的 TDD 掛勾（該端已正確經引擎解析）
- 不動 .speclink.yaml 的應用層鍵（tools、spec_dir、remote）——移除的僅限政策鍵相容層
- 不提供舊鍵自動搬移工具（無歷史使用者可搬）

## Alternatives Considered

- **apply 技能自跑 `speclink workflow-config show` 查政策**：多一次指令往返，且 show 顯示的是正典值而非有效值（不含環境變數覆寫），技能文字仍可能再漂移——payload 帶值把解析責任收回引擎，被否決。
- **保留相容層**：第一個正式版發布時 .speclink.yaml 即無政策鍵，相容層保護的是不存在的使用者，被否決。

## 相容性影響

- `speclink instructions apply --json`：payload 新增 `tdd`、`audit` 欄位（加欄不移欄，既有欄位不變）；人眼輸出不變。
- Wire contract：protocol ApplyInstructions 增兩欄（JSON Schema 由 derive 自動導出，無獨立匯出檔）；新 client 對舊 server 反序列化失敗即報錯（刻意 fail closed，屬版本偏斜的預期行為）。
- `.speclink.yaml` 含政策鍵的專案：該鍵由「生效＋警告」變為「不生效、無警告」。第一個正式版起即無此類專案，無實際遷移對象；若有，遷移方式為把鍵搬入 openspec/config.yaml。
- stderr：deprecation 警告不再出現。
- 技能資產變更觸發 MARKER_VERSION 版號 bump 與 golden／assets.lock 再生，claude 與 codex 兩工具的渲染技能檔（apply、apply-with-worktree、ingest、propose、onboard）內容更新。

## Impact

- Affected specs: workflow-config（解析層數、deprecation 警告 requirement 移除、payload 有效值欄位新增、show 動詞的層數字句）
- Affected code:
  - Modified:
    - crates/speclink-core/src/instructions.rs
    - crates/speclink-core/src/config.rs
    - crates/speclink-core/src/init.rs
    - crates/speclink-protocol/src/query.rs
    - crates/speclink-server/src/routes.rs
    - crates/speclink-cli/src/verbs/instructions.rs
    - crates/speclink-cli/src/common.rs
    - crates/speclink-cli/src/main.rs
    - crates/speclink-cli/tests/it/instructions_policy.rs
    - crates/speclink-core/assets/skills/apply.md
    - crates/speclink-core/assets/skills/tdd.md
    - crates/speclink-core/assets/skills/ingest.md
    - crates/speclink-core/assets/skills/propose.md
    - crates/speclink-core/assets/skills/onboard.md
    - crates/speclink-core/tests/golden/claude.snapshot.md
    - crates/speclink-core/tests/golden/codex.snapshot.md
    - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
    - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
    - crates/speclink-core/tests/golden/assets.lock
    - docs/configuration.md
    - docs/configuration.zh-TW.md
  - New: （無）
  - Removed:
    - crates/speclink-cli/tests/it/deprecation_warning.rs
