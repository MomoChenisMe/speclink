## Why

`speclink workflow-config set --help` 印出的政策鍵只有四個（locale、spec_locale、tdd、audit），少了 `worktree`。但同一支指令的未知鍵錯誤訊息印的是五個，而且 speclink workflow-config set worktree false --dry-run 實測正常輸出 diff、exit code 0。功能一直都在，只有 help 在騙人——照 help 讀的使用者會以為 worktree 政策不能用 CLI 改。

根因是**同一份清單有兩個真相來源**：`crates/speclink-cli/src/verbs/config.rs` 裡的 `POLICY_KEYS` 常數（五個，正確）負責錯誤訊息，而 clap 的 doc comment 是另一份手寫字面（四個，過時）負責 help。兩者各自為政，沒有任何機制把它們綁在一起，所以下一次增刪政策鍵會再漂一次。

這條錯字面還有兩個會放大它的下游消費者：`openspec/specs/config-skill/spec.md` 明文規定 speclink-config 技能核實 CLI 子指令時要「對照 --help 輸出」，`openspec/specs/user-documentation/spec.md` 規定文件裡的 CLI 旗標必須可由對應子指令 help 觀察——錯的 help 會被當成核實依據反向污染。

正典自己也漂了，而且是**三條需求同時落後**：`openspec/specs/workflow-config/spec.md` 的「工作流政策的正典歸屬與四層解析順序」需求明列五欄、且規定 set SHALL 接受 worktree 鍵，但另外三條停在四欄——

1. **set 需求**寫「key SHALL 限 locale、spec_locale、tdd、audit 四者」，正典插入序也只列四鍵（實作早已五鍵序）。過時的 help 字面極可能就是照這條抄的。
2. **show 需求**寫「政策四欄」、`--json` 欄位清單漏 worktree；實測人眼輸出已有 worktree 一列、payload 鍵序已是 locale、specLocale、tdd、audit、worktree、context、rules。更明顯的矛盾是同一份 spec 的 worktree 專屬需求寫著「show 的人眼與 --json 輸出形狀不變（worktree 欄位既已存在）」——它預設 show 需求寫了 worktree，但 show 需求根本沒寫。
3. **init 範本需求**只規定四鍵註解示例；實測乾淨 temp repo 跑 speclink init 產出的範本已含 worktree 註解示例與 SPECLINK_WORKTREE 覆寫提示。

三處方向一致（都少同一個 worktree），必須同批收：分批修等於在同一份 spec 上開平行 change，版號行會對撞。只補程式碼不補正典，等於留著同一個錯誤來源。

目標使用者是透過 AI 代理跑 SDD 的開發者，以及直接用 CLI 的人；使用情境是設定工作流政策（尤其是開關 worktree 平行流程）與 speclink-config 技能的引用核實。

## What Changes

- **help 文字改由 `POLICY_KEYS` 生成**：`workflow-config set` 子指令的說明字串以 `POLICY_KEYS` 組出，掛在 clap 的 `#[command(about = ...)]`，取代手寫 doc comment。寫法沿用同 crate `crates/speclink-cli/src/main.rs` 既有的 `LazyLock<String>` ＋ 表達式屬性慣例，不是新發明。這讓「help 少一個鍵」在結構上不可能再發生。
- **`<VALUE>` 參數說明補上 worktree**：目前寫「Policy value (tdd/audit take true or false)」，`worktree` 同樣只收 true/false 卻沒被列出。此處不引入新常數（布林鍵子集沒有既有常數，只為一行 help 立一個常數是過度設計），改為修正字面並由測試釘住。
- **新增 CLI 整合測試釘住 help 與接受鍵集合的一致性**：測試從同一支 binary 取兩份輸出——set --help 印的鍵集合、以及未知鍵錯誤訊息（Unknown key ... Use one of: ...）印的鍵集合——斷言兩者逐字相同；並斷言 `<VALUE>` 說明涵蓋全部布林鍵。測試不硬寫第三份清單，所以它自己不會變成第三個真相來源。
- **修正正典 workflow-config 的三條落後需求**（純文字校正，實作不動）：
  - **set 需求**：接受鍵由「四者」更正為五者（含 worktree）、正典插入序補上 worktree、「設 false 移除鍵」擴及 worktree，並新增一條規定——set 的 help 文字所列鍵集合 SHALL 與實際接受的鍵集合一致。
  - **show 需求**：「政策四欄」更正為五欄、`--json` payload 欄位清單補上 worktree，並聲明本次為文字校正、輸出維持位元級不變。
  - **init 範本需求**：註解示例區由四鍵更正為五鍵，並明文規定覆寫提示行須列出五個 SPECLINK_* 環境變數名（含 SPECLINK_WORKTREE）。
- **新增一條 init 範本的回歸釘樁測試**：全 repo 核實後確認**沒有任何測試守住 init 產出的 config.yaml 範本註解區**——範本內容目前正確卻無人看守，改壞不會有測試變紅。正典既然要明文宣告範本內容，就一併給它載體。此測試對現行實作一寫即綠（非紅燈），並以一次性變異檢查證明它不是恆綠的假測試。show 需求則**不新增測試**：既有的 `show_prints_canonical_policy_context_and_rules` 與 `show_json_payload_is_camel_case_with_null_for_unset` 已分別斷言人眼的 worktree 一列與 payload 的 worktree 欄位。

### 相容性影響

- 人眼輸出：speclink workflow-config set --help 的說明行與 `<VALUE>` 說明行字面改變（各補上 worktree）；speclink workflow-config --help 的 set 那一列同步改變（clap 用同一個 about）。這是**刻意修正**的錯誤資訊，不是行為變更。
- `--json`：完全不變。help 沒有 json 面，`--json` payload 欄位名與 shape 一字不動——包含 `workflow-config show --json`（該處只有正典文字追上既有 payload，payload 本身不動）。
- `speclink workflow-config show` 與 `speclink init`：兩者的產出**一個位元都不變**。這兩條需求交付的是正典文字校正與測試守門，不是行為變更。
- 回歸對照：help 字面**未被任何技能資產或 golden 快照引用**（已逐檔核實），因此 `MARKER_VERSION`／golden 快照／`assets.lock` 三連動**不觸發**。
- 遷移：使用者無須任何動作，也不必跑 speclink update。

### 影響的 crate 與 app

`speclink-cli`（唯一）。不動 speclink-core、speclink-host、speclink-server、speclink-node，也不動 apps/desktop 與 apps/server-web。

### 技能與工具影響

不改任何技能資產內文，不影響 claude／codex 任一工具的生成足跡。間接受益者是 speclink-config 技能：它的核實依據（--help 輸出）從此與正典一致。

### CLI 介面

子指令、旗標、stdin 與 exit code **全部不變**：speclink workflow-config set <KEY> <VALUE> 仍收 --dry-run 與 --no-color、無 stdin、成功 0、未知鍵與非法值非 0。本變更只改說明文字與其產生方式。

### 設定欄位

不新增也不變更任何設定欄位。`openspec/config.yaml` 的政策五欄與預設值（worktree 預設 false）維持原樣。

## Non-Goals

- **不改 `set worktree` 的行為**：技能足跡同步、以及由 true 改 false 時遇活躍 linked worktree 的擋下，程式碼與正典都不動。
- **不把 worktree 的專屬行為寫進 --help**：詳細取捨見 design。
- **不動 `docs/configuration.md` 與 `docs/configuration.zh-TW.md`**：兩份文件的政策鍵清單已經是正確的五個，沒有要修的東西。（另註：兩份文件都沒有描述「關閉 worktree 時遇活躍 worktree 會被擋下」，那是既有的文件缺口，與本變更無關，也與已完成但未封存的 user-docs-overhaul 檔案面重疊，不在此併修。）
- **不重構 `new.rs` 與 `query.rs` 的同類寫法**：兩處手寫枚舉（new artifact 的 TYPE 說明、list 的 --sort 說明）已逐一比對過實際接受值，目前**沒有漂移**，因此不在範圍內。詳見 design 的掃描結論。
- **不引入 clap 的 `value_parser` 白名單或 enum 化鍵參數**：那會改變未知鍵的錯誤訊息與 exit path（clap 自己報錯，取代目前的 bail!），屬行為變更，超出「修正錯誤資訊」的範圍。
- **不改 show 與 init 的實際輸出**：這兩處是正典追上實作，不是實作追上正典。`crates/speclink-core/src/init.rs` 的範本字面一字不動。
- **不動正典的舊政策鍵 deprecation 警告需求**：它只列四鍵是**正確的**——同一份 spec 已明文規定 worktree 無歷史舊鍵、寫在 `.speclink.yaml` 不生效且不警告。補上 worktree 反而會把對的正典改錯。
- **不把 worktree 加進 speclink-config 技能的政策逐項詢問**：`openspec/specs/config-skill/spec.md` 的「政策四欄 SHALL 逐項詢問使用者」維持四欄。這不是漂移而是尚未做過的設計判斷，理由與替代處置見 design。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workflow-config`: 三條需求同批校正——**set 需求**的接受鍵由「四者」更正為五者（含 worktree）、正典插入序補上 worktree，並新增「help 文字所列鍵集合須與實際接受鍵集合一致」的規定；**show 需求**的「政策四欄」更正為五欄、`--json` 欄位清單補上 worktree；**init 範本需求**的註解示例區由四鍵更正為五鍵並明文規定五個 SPECLINK_* 覆寫名。

## Impact

- Affected specs: workflow-config
- Affected code:
  - Modified:
    - crates/speclink-cli/src/verbs/config.rs
    - crates/speclink-cli/tests/it/workflow_config.rs
    - crates/speclink-cli/tests/it/init_tools.rs
  - New: (none)
  - Removed: (none)
- 不影響任何 API、wire contract、相依套件或 store driver。
