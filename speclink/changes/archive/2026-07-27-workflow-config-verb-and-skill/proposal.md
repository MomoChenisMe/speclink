## Why

`openspec/config.yaml`（工作流政策、context、rules）目前只有 desktop 設定頁能安全寫入：CLI 的 `speclink config` 管的是全域自由 KV 存放（單行 scalar，塞不下多行 context），而 remote 模式的寫入需要 GET→改寫→PUT＋CAS 的樂觀鎖編排，靠檔案工具完全無解。core 已有完整的 text→text 改寫 seam 與 remote 傳輸面（config 讀寫端點），缺的只是 CLI 側的薄編排。同時，config.yaml 的內容層（context／rules）該寫什麼沒有可依循的紀律——已實證憑印象整理會與引擎自動注入重複、留下指向不存在測試的死規則、且多次呼叫會震盪。

目標使用者：透過 AI 代理跑 SDD 的開發者（在 local 或 remote 模式維護專案政策與 context），以及初次導入 Speclink、需要從 codebase 生成 config.yaml 內容的維護者。使用情境：onboard 後的設定初始化、codebase 演進後的 context／rules 迭代整理（對應 propose 前的政策維護），搭配技能 `speclink-config` 使用 CLI 動詞落檔。

## What Changes

1. 新增 CLI 動詞 `speclink workflow-config`，管理 `openspec/config.yaml`（與既有 `speclink config` 的全域 KV 存放分離）：
   - `show [--json]`：顯示正典政策四欄（locale、spec_locale、tdd、audit）、context 與 rules；`--json` 輸出 camelCase payload。
   - `set <key> <value>`：寫入政策四欄之一；未知 key 或非法布林值以非零 exit code 拒絕。
   - `context --stdin`：以 stdin 全文設定 context（空白內容＝移除鍵）。
   - `rules <artifact> --stdin`：整節代換指定 artifact 的規則（一行一條；空 stdin＝移除該節）。
   - 三個寫入子指令皆支援 `--dry-run`：印 unified diff 至 stdout、不寫入、exit code 0。
   - 模式由既有 binding 判定：fs 模式直接讀寫 `openspec/config.yaml`；remote 模式經連線讀取 server 端 config、同一改寫 seam、寫回帶 CAS——revision 不對使用者暴露，衝突時以非零 exit code 提示重跑。
   - stdin 僅 `context` 與 `rules` 子指令讀取；成功 exit code 0、輸出至 stdout，失敗非零、語義化訊息至 stderr。
2. 新增內嵌技能 `speclink-config`（與既有內嵌技能同機制：core assets 為事實來源，init／update 渲染至 claude 與 codex 技能目錄）：規定固定輸入來源掃描、四條內容判準（含以 instructions payload 反證重複、引用存在性核實）、先 `--dry-run` 產 diff 交使用者裁決再寫入、以及收斂驗收（同一未變動 codebase 連跑兩次，第二次 diff 為空）。

## Non-Goals

- 不動既有 `speclink config`（全域 KV）的任何行為與輸出。
- 不進命令執行層、不擴充 Node SDK dispatch——本動詞為周邊設定動詞（與 config、init 同類）。
- 不新建 Host 抽象層（消費者僅 desktop 與 CLI 兩個，各自薄編排）。
- 不改變政策的四層解析語意（環境變數、舊鍵相容層不受影響）；`show` 顯示正典值、不做解析。
- 不含 `openspec/config.yaml` 本檔的內容優化（已於討論階段直接完成落檔）。
- 技能不自動寫入——一律先 diff 經使用者確認。

## Capabilities

### New Capabilities

- `config-skill`: 內嵌 speclink-config 技能——從 codebase 固定來源整理 config.yaml 的 context 與 rules，依四判準收斂、產 diff 交使用者裁決。

### Modified Capabilities

- `workflow-config`: 新增 workflow-config 動詞的 CLI 契約（show／set／context／rules、--dry-run、fs 與 remote 兩模式）。

## Impact

- Affected specs: workflow-config（MODIFIED——新增動詞需求）、config-skill（新能力）
- Affected code:
  - New: crates/speclink-core/assets/skills/config.md、.claude/skills/speclink-config/SKILL.md、.agents/skills/speclink-config/SKILL.md、crates/speclink-cli/tests/workflow_config.rs
  - Modified: crates/speclink-cli/src/main.rs、crates/speclink-cli/src/commands.rs、crates/speclink-cli/src/remote_commands.rs、crates/speclink-core/src/skills.rs、crates/speclink-core/tests/golden/claude.snapshot.md、crates/speclink-core/tests/golden/codex.snapshot.md、crates/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md、docs/configuration.zh-TW.md、docs/configuration.md
  - Removed: （無）

影響的 crate 或 app：speclink-cli（動詞編排）、speclink-core（技能資產與渲染註冊）、speclink-remote（僅使用既有 client，無修改）；文件面 docs/configuration 兩語版。

相容性影響：純新增——既有指令的人眼與 `--json` 輸出零變更；golden 四份因新增技能區段而更新（刻意變更，乾淨樹再生）；`speclink init`／`update` 的產出新增 speclink-config 技能目錄（claude 與 codex 兩工具）。

設定欄位：無新增欄位——本動詞管理既有的 locale、spec_locale、tdd、audit、context、rules 鍵；寫入採 read-modify-write，其他鍵值保留、模板註解喪失（與 desktop 設定頁同一取捨，規格明述）。
