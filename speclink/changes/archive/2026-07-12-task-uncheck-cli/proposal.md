## Why

apply 階段的任務勾選由 agent 執行 CLI 動詞完成，但引擎只有單向的「標記完成」：`speclink task done` 之外沒有反向動詞。agent 誤勾或實作回退需要取消勾選時，唯一途徑是直接編輯 tasks.md——這繞過儲存抽象，在 remote 模式（任務清單的真相在 server）下會改到錯的地方。取消勾選的邏輯其實已存在於桌面 app，但以桌面自製的行編輯實作、未進引擎，CLI 與其他入口均無法使用，也使引擎完成函式「所有工具路徑單一協作點」的設計宣稱不成立。

目標使用者：透過 AI 代理跑 SDD 的開發者——對應 apply 階段與 speclink-apply 技能；桌面 app 勾選框使用者間接受益（取消勾選改走引擎共用路徑）。

## What Changes

- speclink-core 新增「取消完成」引擎動詞：把指定任務由已勾選翻回未勾選。純狀態翻轉、無反向側效——不回收 touched 記錄、不撤 started_at 開工章（沿桌面批次動詞「取消勾選無側效」的既有裁定）。對「已是未勾選」的任務回報既定狀態且零檔案效果，與完成動詞對稱。
- speclink-cli 新增子指令 `speclink task undone <task-id>`，旗標 `--change <name>` 與 `--json`，無 stdin；與 `task done` 完全對稱。成功 exit code 0；任務已是未勾選、task id 非數字或超界、tasks.md 不存在時以非零 exit code 結束，錯誤訊息形狀對稱於 `task done`。
- remote 模式：CLI 的 remote 攔截路徑新增對應分派，speclink-remote client 新增取消完成的 endpoint 呼叫；人眼與 `--json` 輸出形狀與 fs 模式一致（欄位 camelCase 同名）。
- 桌面 app 的單發取消勾選改走引擎新動詞，移除桌面自製行編輯；可觀察行為不變（取消勾選維持冪等成功）。
- speclink-apply 技能新增取消勾選動詞的使用指引：claude 與 codex 兩工具的 repo 技能實例、內嵌資產同步修改，render golden 於乾淨樹再生。

## Non-Goals

- 不做 touched 記錄與開工章的反向回滾——trace 是歷史記錄，取消勾選不竄改它。
- 不動桌面批次動詞（全部標完成／全部取消）的實作歸屬，本次僅收斂單發取消勾選。
- 不改 `task done` 的任何既有行為與輸出。
- 不等待也不阻擋進行中的 engine-typed-core：本變更以現行 CLI handler 直呼引擎的形式落地，動詞收編進 typed runtime 由該變更處理。
- 已否決做法：維持直接改檔（remote 下錯誤）、CLI 層自做行編輯（繞過引擎協作點、與桌面邏輯分岔）、以旗標掛在 task done 上（對稱動詞對 agent 更直覺、--help 可發現性更好）。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `verb-contract`: task 動詞由單向勾選擴為勾選／取消勾選——undone 的指令輸出（人眼與 `--json`）、錯誤形狀與 remote endpoint 入約，remote 模式輸出形狀與 fs 模式一致。

## Impact

- 相容性影響：純新增動詞，既有指令的人眼與 `--json` 輸出皆不變，parity/color/twin 回歸對照不受影響；技能資產變動需於乾淨樹再生 render golden 快照。
- 影響 crate：`speclink-core`、`speclink-cli`、`speclink-remote`；另及桌面 app 的 `apps/desktop/core` 與技能資產。
- Affected specs: `verb-contract`（修改）
- Affected code:
  - Modified: `crates/speclink-core/src/tasks.rs`、`crates/speclink-cli/src/main.rs`、`crates/speclink-cli/src/commands.rs`、`crates/speclink-cli/src/remote_commands.rs`、`crates/speclink-remote/src/client.rs`、`crates/speclink-cli/tests/remote_write_path.rs`、`apps/desktop/core/src/manage.rs`、`crates/speclink-core/assets/skills/apply.md`、`.claude/skills/speclink-apply/SKILL.md`、`.agents/skills/speclink-apply/SKILL.md`、`crates/speclink-core/tests/golden`（乾淨樹再生，實際變動檔以再生結果為準）
  - New: `crates/speclink-cli/tests/task_undone.rs`（task undone 的 fs 模式整合測試）
  - Removed: （無）
