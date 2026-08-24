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

---
### Requirement: createEngine 的建構期 actor 注入

createEngine SHALL 接受選填 actor 欄位（"Name <email>" 格式字串），兩種儲存形式皆於建構期收下並綁定於該 engine 實例；dispatch 的操作者身分解析 SHALL 為：建構期 actor 有給值（trim 後非空）一律優先，fs 形式未給時回退 workspace 的 git identity（現行為），宿主 Store 形式未給時維持無章（現行為）。trim 後為空字串的 actor SHALL 視同未給。actor SHALL NOT 能經 dispatch 的 argv 或其他呼叫期參數傳入或覆寫——一個實例一個身分，多身分宿主以多個 engine 實例表達。

#### Scenario: fs 形式明給 actor 優先於 git identity

- **WHEN** 在設有 git user.name 與 user.email 的 fixture 專案，以 createEngine({ store: { type: 'fs', root }, actor: 'Alice <alice@example.com>' }) 建構並 dispatch(['new', 'change', 'demo'])
- **THEN** demo 的 metadata created_by 為 Alice <alice@example.com>，而非 git identity

#### Scenario: fs 形式未給 actor 維持現行回退

- **WHEN** 同一 fixture 專案以不帶 actor 的 createEngine fs 形式建構並 dispatch(['new', 'change', 'demo2'])
- **THEN** demo2 的 created_by 與在該專案執行 CLI speclink new change 的蓋章逐位元一致（git identity 回退不變）

#### Scenario: 宿主 Store 形式帶 actor 落章

- **WHEN** 以 JS Store 物件＋actor: 'Bob <bob@example.com>' 建構引擎並 dispatch(['new', 'change', 'demo3'])
- **THEN** 宿主 Store 收到的 metadata 寫入含 created_by: Bob <bob@example.com>；同引擎後續的 review／verify 蓋章動詞亦以同值落 _by 欄位

#### Scenario: 宿主 Store 形式未給 actor 維持無章

- **WHEN** 以 JS Store 物件建構引擎（不帶 actor）並 dispatch(['new', 'change', 'demo4'])
- **THEN** 寫入的 metadata 不含 created_by（與現行無章行為一致）

#### Scenario: 呼叫期無從覆寫身分

- **WHEN** 檢視 dispatch 的輸入契約並嘗試以任意 argv 影響蓋章身分
- **THEN** dispatch 不存在 actor 參數，蓋章內容只隨建構期 actor（或其回退）改變

<!-- @trace
source: node-host-actor
updated: 2026-08-24
-->

---
### Requirement: dispatch 的蓋章動詞

dispatch SHALL 認得 `review add-round`、`review stamp`、`verify add-round`、`verify stamp` 四個動詞，argv 沿用 CLI 詞彙（`--accept`、`--agent <tool>`、`--stdin`）。add-round 的輪次內容 SHALL 由 dispatch 的 stdin 參數帶入；stamp 的 scope 指紋與 missing 清單 argv 承載不了，SHALL 由 stdin 參數以 JSON 帶入（`{ "scope": [{ "path", "hash" }], "missing": [] }`，兩欄缺席讀作空清單）。蓋章落下的 `reviewed_by`／`verified_by` SHALL 為該 engine 建構期綁定的 actor（未給時依儲存形式回退，與 created_by 同一條解析）。引擎既有的守門（任務未全完成、末輪未解必修 findings、scope ∪ missing 與工單聯集的分割）SHALL 原封傳遞，拒絕時以語義化例外呈現。蓋章會刪除工單文件並改寫 change 的 metadata 原文，宿主 Store 因此 SHALL 可提供三個選填的前置方法：`deleteArtifact(change, artifact)`、`readChangeMeta(name)`、`writeChangeMeta(name, content)`；缺任何一個時蓋章 SHALL 在動手前以語義化訊息拒絕（工單與 metadata 皆不動），其餘動詞不受影響。

#### Scenario: review 蓋章鏈落 actor

- **WHEN** 以 actor: 'Rev <rev@example.com>' 建構引擎，對一個任務全完成的 change 依序 dispatch(['review', 'add-round', 'beta', '--stdin'], { stdin: 只含 SUGGESTION 的輪次內容 }) 與 dispatch(['review', 'stamp', 'beta', '--stdin'], { stdin: JSON.stringify({ missing: [輪次 Scope 的檔路徑] }) })
- **THEN** add-round 解析為 { change: 'beta', round: 1 }；stamp 解析為 { change: 'beta' }，且 change 的 metadata 落下 reviewed_by: Rev <rev@example.com>

#### Scenario: verify 蓋章鏈落 actor

- **WHEN** 同引擎（同一 actor）對同一 change 走 verify add-round 與 verify stamp
- **THEN** metadata 落下 verified_by 為同一個 actor 值

#### Scenario: 蓋章守門的拒絕原封傳遞

- **WHEN** 對末輪帶 CRITICAL finding 的工單 dispatch(['review', 'stamp', ...]) 而不帶 --accept
- **THEN** dispatch 以 Error 拒絕，message 為引擎的語義化守門訊息，未落任何 reviewed_* 欄位

#### Scenario: 宿主 Store 未實作 deleteArtifact 時蓋章失敗

- **WHEN** 以未實作 deleteArtifact 的 JS Store 建構引擎，走完 add-round 後 dispatch(['review', 'stamp', ...])
- **THEN** 以 Error 拒絕，訊息指名 deleteArtifact 是蓋章所需的方法；同一個 store 的 list／status／new 動詞不受影響

#### Scenario: 未支援的子動詞明確拒絕

- **WHEN** dispatch(['review', 'show', 'beta'])
- **THEN** 以 code 為 invalid_argv 的 Error 拒絕，訊息指出 review 只支援 add-round 與 stamp

<!-- @trace
source: node-host-actor
updated: 2026-08-24
-->