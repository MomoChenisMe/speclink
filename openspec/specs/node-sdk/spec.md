# node-sdk Specification

## Purpose

Node 綁定對外的 SDK 契約：createEngine 的兩種儲存建構形式、dispatch 的輸入與輸出契約，以及把引擎人眼輸出取回 JS 端的渲染 API（含 worktree 軸）。本 capability 保證 JS 端不必自行拼裝引擎內部結構，且渲染結果與 CLI 同源、不另立一套。

## Requirements

### Requirement: createEngine 的雙形式儲存建構
Node SDK 的 createEngine SHALL 接受兩種儲存形式：內建 fs（指定專案根與選填 spec 目錄名），或宿主實作的 Store 物件（方法回傳值或 Promise 皆可）。以 Store 物件建構且缺少必要方法時，createEngine SHALL 於建構當下拋出錯誤並列出缺少的方法名。

#### Scenario: fs 形式與 CLI 行為對等
- **WHEN** 以 createEngine 的 fs 形式指向某 fixture 專案，執行 await engine.dispatch(['list', '--json'])
- **THEN** 回傳物件與在同一專案執行 speclink list --json 的輸出 JSON 完全一致

#### Scenario: 宿主 Store 物件生效
- **WHEN** 以模擬兩個 change 的 JS Store 物件（方法回傳 Promise）建構引擎並 dispatch(['list', '--json'])
- **THEN** 回傳兩個 change 且欄位名（camelCase）與 CLI 輸出一致

#### Scenario: 缺方法建構即失敗
- **WHEN** 以缺少 artifact 寫入方法的 Store 物件呼叫 createEngine
- **THEN** 同步拋出錯誤，訊息列出缺少的方法名，不產生引擎實例


<!-- @trace
source: node-sdk
updated: 2026-07-05
code:
  - .github/workflows/node-sdk.yml
  - Cargo.lock
  - Cargo.toml
  - README.md
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-node/.gitignore
  - crates/speclink-node/Cargo.toml
  - crates/speclink-node/__test__/engine.spec.ts
  - crates/speclink-node/__test__/helpers.ts
  - crates/speclink-node/__test__/render.spec.ts
  - crates/speclink-node/__test__/store-bridge.spec.ts
  - crates/speclink-node/__test__/stress.spec.ts
  - crates/speclink-node/__test__/write-path.spec.ts
  - crates/speclink-node/build.rs
  - crates/speclink-node/index.d.ts
  - crates/speclink-node/index.js
  - crates/speclink-node/package-lock.json
  - crates/speclink-node/package.json
  - crates/speclink-node/src/lib.rs
  - crates/speclink-node/src/render.rs
  - crates/speclink-node/src/store_bridge.rs
  - docs/sdk-node.md
  - docs/sdk-node.zh-TW.md
-->

---
### Requirement: dispatch 的輸入輸出契約
engine.dispatch SHALL 接受 argv 字串陣列（與 CLI 動詞詞彙一對一）與選填第二參數（stdin 內容），回傳 Promise；成功時解析為與 CLI --json 對齊的結構化物件（無 --json 形式的動詞回傳含 output 字串的物件）；失敗時以 Error 拒絕——message 為與 CLI 相同的語義化訊息並附 code 欄位。dispatch SHALL 於背景工作執行，SHALL NOT 阻塞 JS 事件迴圈。

dispatch SHALL 由與 CLI 共用的引擎命令層執行：argv 詞彙、回傳形狀與既有錯誤碼值域維持不變；對相同 workspace 狀態，dispatch 的成功結果與錯誤 SHALL 與 CLI 對應動詞語意一致，錯誤碼 SHALL 出自命令層的封閉註冊表（含 invalid_config 與 refused）。宿主 Store 提供的工作流設定文字存在但無法解析時，讀取政策的 dispatch 呼叫 SHALL 以 Error 拒絕且 code 為 invalid_config，SHALL NOT 以預設政策繼續執行。

#### Scenario: 寫入動詞經 stdin 參數
- **WHEN** 執行 await engine.dispatch(['new', 'artifact', 'proposal', '--change', 'demo', '--stdin'], { stdin: 內容字串 })
- **THEN** 宿主 Store 收到該 artifact 的寫入呼叫，dispatch 解析為成功結果

#### Scenario: 錯誤以語義化例外傳遞
- **WHEN** Store 於認領時回報該 change 已被他人持有，執行 dispatch(['claim', 'x'])
- **THEN** Promise 以 Error 拒絕，message 為語義化訊息（含持有情境與建議動作）、code 反映衝突類別，宿主可將 message 直接回給 agent

#### Scenario: 並發 dispatch 不死結
- **WHEN** 對同一引擎並發發出多個 dispatch 呼叫（宿主 Store 方法為 async）
- **THEN** 全部呼叫在有限時間內完成（無互等死結），事件迴圈期間可持續處理其他工作

#### Scenario: 壞工作流設定經 dispatch 拒絕
- **WHEN** 宿主 Store 的工作流設定讀取方法回傳無法解析的 YAML 文字，執行 dispatch(['new', 'change', 'demo'])
- **THEN** Promise 以 Error 拒絕，code 為 invalid_config，message 指出工作流設定無法解析與原因

---
### Requirement: 渲染 API
SDK SHALL 提供 skills.list()（回傳技能名與描述清單）、skills.render(name, options) 與 instructions.render(options)——options 涵蓋渲染矩陣：target（claude｜codex｜neutral）、invocation（cli｜tool-call）、store（fs｜remote）；回傳字串內容 SHALL 與 CLI 以對等參數生成的內容一致。

#### Scenario: 中性 tool-call 渲染
- **WHEN** 執行 skills.render('propose', { target: 'neutral', invocation: 'tool-call', store: 'remote' })
- **THEN** 回傳字串以「呼叫 speclink 工具」措辭表述動詞、不含 /speclink- 前綴與本地規格路徑句

#### Scenario: 與 CLI 生成一致
- **WHEN** 以 target claude、store fs 呼叫 skills.render('apply', …)，並與 speclink init 於 fs 專案生成的 .claude/skills/speclink-apply/SKILL.md 比對
- **THEN** 兩者內容一致

<!-- @trace
source: node-sdk
updated: 2026-07-05
code:
  - .github/workflows/node-sdk.yml
  - Cargo.lock
  - Cargo.toml
  - README.md
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-node/.gitignore
  - crates/speclink-node/Cargo.toml
  - crates/speclink-node/__test__/engine.spec.ts
  - crates/speclink-node/__test__/helpers.ts
  - crates/speclink-node/__test__/render.spec.ts
  - crates/speclink-node/__test__/store-bridge.spec.ts
  - crates/speclink-node/__test__/stress.spec.ts
  - crates/speclink-node/__test__/write-path.spec.ts
  - crates/speclink-node/build.rs
  - crates/speclink-node/index.d.ts
  - crates/speclink-node/index.js
  - crates/speclink-node/package-lock.json
  - crates/speclink-node/package.json
  - crates/speclink-node/src/lib.rs
  - crates/speclink-node/src/render.rs
  - crates/speclink-node/src/store_bridge.rs
  - docs/sdk-node.md
  - docs/sdk-node.zh-TW.md
-->

---
### Requirement: 渲染 API 的 worktree 軸

instructions.render(options) 的 options SHALL 涵蓋 worktree 布林軸（未給定時視為 false），用以選擇 marker 區塊是否含兩行 worktree 技能指引；回傳內容 SHALL 與 CLI 於對等 worktree 政策下生成的 marker 一致。skills.render 不受此軸影響——技能檔內容與政策無關，政策只決定該技能是否被生成。

#### Scenario: worktree 軸切換 marker 內容

- **WHEN** 以同組 target 與 store 分別呼叫 instructions.render({ worktree: true }) 與 instructions.render({ worktree: false })
- **THEN** 前者的 marker 含 apply-with-worktree 與 worktree-merge 兩行，後者不含，其餘內容逐字相同

#### Scenario: 未給定 worktree 時取預設

- **WHEN** 呼叫 instructions.render 而未給定 worktree 選項
- **THEN** 回傳內容與 worktree: false 的結果相同

<!-- @trace
source: worktree-toggle-and-guards
updated: 2026-08-05
-->