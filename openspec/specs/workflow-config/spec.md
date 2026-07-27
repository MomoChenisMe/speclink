# workflow-config Specification

## Purpose

TBD - created by archiving change 'config-system-rework'. Update Purpose after archive.

## Requirements

### Requirement: 工作流政策的正典歸屬與四層解析順序

工作流政策欄位（locale、spec_locale、tdd、audit）的正典值 SHALL 儲存於 openspec/config.yaml（經儲存介面讀取）。有效值 SHALL 依下列順序解析，先命中者勝：SPECLINK_LOCALE／SPECLINK_SPEC_LOCALE／SPECLINK_TDD／SPECLINK_AUDIT 環境變數 ＞ .speclink.yaml 的同名舊鍵（相容層）＞ openspec/config.yaml ＞ 內建預設（locale 未設定＝English、tdd 與 audit＝false）。布林環境變數僅接受 true 或 false，其他值 SHALL 視為未設定並落到下一層。

openspec/config.yaml 檔案存在但無法解析（YAML 語法錯誤或型別不符）時，讀取政策的指令 SHALL 以非零 exit code 失敗，stderr SHALL 指出該檔的 workspace 相對路徑與解析原因；SHALL NOT 以內建預設或解析順序中其他層的值繼續執行。檔案不存在時 SHALL 沿用內建預設。此 fail-closed 行為為刻意設計。

#### Scenario: 正典值生效

- **WHEN** .speclink.yaml 不含政策鍵、openspec/config.yaml 設定 tdd: true，執行 speclink instructions tasks --change 某 change --json
- **THEN** payload 反映 tdd 開關為開啟（tasks 指引含 TDD 紀律內容），stderr 無 deprecation 警告

#### Scenario: 舊鍵相容層勝過正典值

- **WHEN** .speclink.yaml 設定 locale: tw 且 openspec/config.yaml 設定 locale: ja，執行 speclink instructions proposal --change 某 change --json
- **THEN** payload 的 locale 欄位為 Traditional Chinese (繁體中文)，且 stderr 出現一行 deprecation 警告

#### Scenario: 環境變數覆寫一切

- **WHEN** 設定環境變數 SPECLINK_TDD=false，而 .speclink.yaml 與 openspec/config.yaml 均設定 tdd: true，執行 speclink instructions tasks --change 某 change --json
- **THEN** payload 反映 tdd 開關為關閉

#### Scenario: 非法布林環境變數落到下一層

- **WHEN** 設定 SPECLINK_AUDIT=yes（非法值），openspec/config.yaml 設定 audit: true，執行任一讀取政策的指令
- **THEN** 有效 audit 值為 true（環境變數被忽略，不輸出錯誤）

#### Scenario: 壞 config.yaml 一律 fail-closed

- **WHEN** openspec/config.yaml 含 YAML 語法錯誤，執行 speclink instructions tasks --change 某 change --json
- **THEN** exit code 非 0，stderr 指出 openspec/config.yaml 與解析原因，stdout 不輸出 instructions payload

#### Scenario: 環境變數不得繞過壞檔

- **WHEN** openspec/config.yaml 含 YAML 語法錯誤，且設定 SPECLINK_TDD=true，執行 speclink instructions tasks --change 某 change --json
- **THEN** 指令仍以非零 exit code 失敗（環境變數不使壞檔被忽略）

#### Scenario: 缺檔沿用內建預設

- **WHEN** openspec/config.yaml 不存在，執行 speclink instructions proposal --change 某 change --json
- **THEN** payload 以內建預設政策生成，exit code 為 0


<!-- @trace
source: spectra-legacy-cleanup
updated: 2026-07-27
code:
  - README.en.md
  - README.md
  - apps/desktop/src/App.tsx
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/index.css
  - crates/speclink-cli/src/color.rs
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/tests/discuss_promote_snapshot.rs
  - crates/speclink-cli/tests/task_done_stamps.rs
  - crates/speclink-core/assets/skills/archive.md
  - crates/speclink-core/src/analyzer.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/demo.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/drift.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/preflight.rs
  - crates/speclink-core/src/schema.rs
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/src/status.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-core/src/validate.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-host/src/context.rs
  - docs/platform-architecture.zh-TW.md
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/index.ts
  - packages/ui/src/theme.css
-->

---
### Requirement: 舊政策鍵的 deprecation 警告
當 .speclink.yaml 含有 locale、spec_locale、tdd、audit 任一鍵時，CLI SHALL 於每次指令執行時向 stderr 輸出恰一行警告，內容 SHALL 列出偵測到的鍵名並指引搬移至 openspec/config.yaml。stdout（含 --json 輸出）SHALL NOT 受警告影響。無舊鍵時 SHALL NOT 輸出警告。

#### Scenario: 含舊鍵時單行警告
- **WHEN** .speclink.yaml 含 tdd: true 與 audit: true，執行 speclink list --json
- **THEN** stderr 恰有一行警告且同時列出 tdd 與 audit 兩個鍵名，stdout 的 JSON 與無警告情境完全相同，exit code 為 0

#### Scenario: 無舊鍵時無警告
- **WHEN** .speclink.yaml 僅含 tools 與 spec_dir，執行 speclink list
- **THEN** stderr 無任何 deprecation 警告


<!-- @trace
source: config-system-rework
updated: 2026-07-04
code:
  - AGENTS.md
  - CLAUDE.md
  - README.md
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/tests/deprecation_warning.rs
  - crates/speclink-cli/tests/instructions_policy.rs
  - crates/speclink-cli/tests/tools_descriptor.rs
  - crates/speclink-core/assets/skills/archive.md
  - crates/speclink-core/assets/skills/commit.md
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/render_golden.rs
  - docs/configuration.md
  - docs/configuration.zh-TW.md
-->

---
### Requirement: init 範本的政策寫入位置
speclink init SHALL 將政策欄位的示例寫入 openspec/config.yaml 範本（locale、spec_locale、tdd、audit 的註解示例區），且 .speclink.yaml 範本 SHALL NOT 含任何政策鍵（僅 tools 與 spec_dir 相關內容）。

#### Scenario: 新專案初始化的範本內容
- **WHEN** 於空目錄執行 speclink init
- **THEN** 生成的 openspec/config.yaml 含 locale、spec_locale、tdd、audit 的註解示例；生成的 .speclink.yaml 不含此四鍵；exit code 為 0

#### Scenario: 既有專案不受範本變更影響
- **WHEN** 於已初始化且 .speclink.yaml 含舊政策鍵的專案執行 speclink update
- **THEN** 兩個設定檔內容不被改寫（update 不觸碰設定檔），僅 stderr 出現 deprecation 警告

<!-- @trace
source: config-system-rework
updated: 2026-07-04
code:
  - AGENTS.md
  - CLAUDE.md
  - README.md
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/tests/deprecation_warning.rs
  - crates/speclink-cli/tests/instructions_policy.rs
  - crates/speclink-cli/tests/tools_descriptor.rs
  - crates/speclink-core/assets/skills/archive.md
  - crates/speclink-core/assets/skills/commit.md
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/render_golden.rs
  - docs/configuration.md
  - docs/configuration.zh-TW.md
-->

---
### Requirement: workflow-config show 動詞

CLI SHALL 提供 speclink workflow-config show（旗標 --json 與 --no-color，無 stdin），顯示 openspec/config.yaml 的正典內容：政策四欄（locale、spec_locale、tdd、audit——未設定的欄位標示為未設定與其預設語意）、context（有無與行數）與 rules（各 artifact 節的條數）。show SHALL 顯示正典值，SHALL NOT 應用環境變數或 .speclink.yaml 舊鍵的覆寫（有效值的四層解析屬 instructions payload 職責）。帶 --json 時 SHALL 輸出 camelCase payload：locale、specLocale（未設定為 null）、tdd、audit（布林）、context（字串或 null）、rules（artifact id 對規則字串陣列的物件）。fs 模式讀取 openspec/config.yaml；remote 模式 SHALL 經既有連線讀取 server 端 config 文件，人眼與 --json 輸出形狀與 fs 模式一致。config 文件無法解析時 SHALL 沿用既有 fail-closed 行為：非零 exit code、stderr 指出檔案與解析原因。本動詞為周邊設定動詞（與 config 同類），SHALL NOT 進入命令執行層。成功 exit code 0、輸出至 stdout；--no-color 下無 ANSI 色彩。

#### Scenario: fs 模式顯示正典值

- **WHEN** openspec/config.yaml 含 locale: tw、tdd: true 與 context，執行 speclink workflow-config show
- **THEN** stdout 顯示 locale 為 tw、tdd 開啟、audit 未設定（預設關閉）、context 存在；exit code 0

#### Scenario: --json payload 形狀

- **WHEN** 執行 speclink workflow-config show --json
- **THEN** stdout 為 JSON，欄位 locale、specLocale、tdd、audit、context、rules 一律 camelCase；未設定的 specLocale 為 null、未設定的 tdd 為 false

#### Scenario: show 不應用環境變數覆寫

- **WHEN** 設定 SPECLINK_TDD=false 且 openspec/config.yaml 含 tdd: true，執行 speclink workflow-config show --json
- **THEN** payload 的 tdd 為 true（正典值，非解析後有效值）

#### Scenario: remote 模式輸出形狀一致

- **WHEN** 於 remote 綁定的 workspace 執行 speclink workflow-config show --json
- **THEN** stdout 的 JSON 欄位名與 fs 模式完全一致，值來自 server 端 config 文件

#### Scenario: 壞 config fail-closed

- **WHEN** openspec/config.yaml 含 YAML 語法錯誤，執行 speclink workflow-config show
- **THEN** exit code 非 0，stderr 指出該檔與解析原因，stdout 無 payload

<!-- @trace
source: workflow-config-verb-and-skill
updated: 2026-07-27
code:
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/workflow_config.rs
  - crates/speclink-core/assets/skills/config.md
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - docs/configuration.md
  - docs/configuration.zh-TW.md
-->

---
### Requirement: workflow-config set 政策欄位寫入

CLI SHALL 提供 speclink workflow-config set <key> <value>（旗標 --dry-run 與 --no-color，無 stdin），寫入政策欄位：key SHALL 限 locale、spec_locale、tdd、audit 四者，其他 key SHALL 以非零 exit code 拒絕且無檔案效果；tdd 與 audit 的 value SHALL 僅接受 true 或 false，其他值 SHALL 以非零 exit code 拒絕。寫入 SHALL 為 read-modify-write：僅變動目標鍵，其餘鍵值 SHALL 保留；tdd 或 audit 設為 false、locale 或 spec_locale 設為空字串時 SHALL 移除該鍵（維持未設定＝預設語意）。原檔的模板註解於重寫後喪失（read-modify-write 取捨）。帶 --dry-run 時 SHALL 於 stdout 印出變更前後的 unified diff、SHALL NOT 寫入任何檔案、exit code 0；無變更時 diff 為空。fs 模式寫入 openspec/config.yaml；remote 模式 SHALL 先經連線讀取 server 端 config 現行內容與版本，套用同一改寫、寫回時附帶讀得的版本——server 端版本已前進（他人並行改寫）時 SHALL 以非零 exit code 失敗、stderr 提示重新執行，SHALL NOT 覆蓋他人寫入；版本識別 SHALL NOT 出現在指令介面。連線離線或認證失效時 SHALL 以非零 exit code 失敗並輸出語義化訊息，SHALL NOT 暫存或排隊寫入。config 文件無法解析時 SHALL fail-closed 拒絕寫入（重寫壞檔會毀掉使用者內容）。成功 exit code 0、stdout 單行成功訊息。

#### Scenario: 設定 locale 保留其他鍵

- **WHEN** openspec/config.yaml 含 schema、tdd: true 與 context，執行 speclink workflow-config set locale tw
- **THEN** exit code 0；config.yaml 的 locale 為 tw，schema、tdd、context 的值不變

#### Scenario: 未知 key 拒絕

- **WHEN** 執行 speclink workflow-config set theme dark
- **THEN** exit code 非 0，stderr 指出 key 限 locale、spec_locale、tdd、audit；openspec/config.yaml 逐位元不變

#### Scenario: 非法布林值拒絕

- **WHEN** 執行 speclink workflow-config set tdd yes
- **THEN** exit code 非 0，stderr 指出 tdd 僅接受 true 或 false；無任何檔案效果

#### Scenario: 設 false 移除鍵

- **WHEN** openspec/config.yaml 含 audit: true，執行 speclink workflow-config set audit false
- **THEN** exit code 0；config.yaml 不含 audit 鍵（未設定＝預設關閉）

#### Scenario: dry-run 印 diff 不落檔

- **WHEN** 執行 speclink workflow-config set locale ja --dry-run
- **THEN** exit code 0；stdout 為 unified diff 顯示 locale 行的變更；openspec/config.yaml 逐位元不變

#### Scenario: remote 版本衝突提示重跑

- **WHEN** 於 remote 模式執行 speclink workflow-config set tdd true，且 server 端 config 在讀取後、寫回前已被他人改寫
- **THEN** exit code 非 0，stderr 說明設定已被他人更新、請重新執行；server 端內容維持他人寫入的版本

#### Scenario: 壞 config 拒絕寫入

- **WHEN** openspec/config.yaml 含 YAML 語法錯誤，執行 speclink workflow-config set locale tw
- **THEN** exit code 非 0，stderr 指出該檔無法解析、寫入已拒絕；檔案逐位元不變

<!-- @trace
source: workflow-config-verb-and-skill
updated: 2026-07-27
code:
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/workflow_config.rs
  - crates/speclink-core/assets/skills/config.md
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - docs/configuration.md
  - docs/configuration.zh-TW.md
-->

---
### Requirement: workflow-config context 與 rules 寫入

CLI SHALL 提供 speclink workflow-config context --stdin 與 speclink workflow-config rules <artifact> --stdin（各支援 --dry-run 與 --no-color）：context SHALL 以 stdin 全文設定 config.yaml 的 context 鍵，內容僅空白時 SHALL 移除該鍵；rules SHALL 整節代換指定 artifact 的規則清單——stdin 一行一條、空行忽略，stdin 為空時 SHALL 移除該 artifact 節；artifact SHALL 限目前 schema 的 artifact id（如 proposal、specs、design、tasks），未知 id SHALL 以非零 exit code 拒絕且無檔案效果。兩個子指令未帶 --stdin 時 SHALL 以非零 exit code 說明用法。寫入的保留語意、--dry-run 行為、remote 模式的讀-改-寫與版本衝突處理、離線與認證失效、壞檔 fail-closed，SHALL 與 workflow-config set 政策欄位寫入一致。成功 exit code 0、stdout 單行成功訊息。

#### Scenario: context 設定多行內容

- **WHEN** 執行 speclink workflow-config context --stdin 並經 stdin 提供多行專案說明
- **THEN** exit code 0；config.yaml 的 context 為該多行內容（YAML block scalar），政策四欄與 rules 不變

#### Scenario: 空白 stdin 移除 context

- **WHEN** config.yaml 含 context，執行 speclink workflow-config context --stdin 且 stdin 僅含空白
- **THEN** exit code 0；config.yaml 不含 context 鍵

#### Scenario: rules 整節代換

- **WHEN** 執行 speclink workflow-config rules design --stdin 並經 stdin 提供三行規則
- **THEN** exit code 0；config.yaml 的 rules.design 恰為該三條（原節整份代換），其他 artifact 節不變

#### Scenario: 未知 artifact 拒絕

- **WHEN** 執行 speclink workflow-config rules blueprint --stdin
- **THEN** exit code 非 0，stderr 指出 artifact id 不在目前 schema；無任何檔案效果

#### Scenario: rules dry-run 印 diff

- **WHEN** 執行 speclink workflow-config rules tasks --stdin --dry-run 並提供新規則
- **THEN** exit code 0；stdout 為 unified diff 顯示 rules.tasks 節的變更；openspec/config.yaml 逐位元不變

#### Scenario: remote context 寫入經版本檢查

- **WHEN** 於 remote 模式執行 speclink workflow-config context --stdin 且無並行改寫
- **THEN** exit code 0；server 端 config 的 context 更新為 stdin 內容；後續 workflow-config show 讀回同一內容

<!-- @trace
source: workflow-config-verb-and-skill
updated: 2026-07-27
code:
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/workflow_config.rs
  - crates/speclink-core/assets/skills/config.md
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - docs/configuration.md
  - docs/configuration.zh-TW.md
-->