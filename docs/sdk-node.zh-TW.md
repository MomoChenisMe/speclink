# Node SDK（@speclink/engine）

> **文件狀態**：本文描述目前已實作的 Node SDK surface。Typed Command Runtime、TeamStore 契約與 Host 邊界的正典是 `openspec/specs/` 底下的 `command-runtime`、`teamstore-contract`、`host-runtime` 與 `node-sdk`；Copilot Tool 封裝尚未實作，方向見[專案路線圖](roadmap.zh-TW.md)。

`@speclink/engine` 讓你把 Speclink 引擎內嵌進 Node.js 行程：伺服器（或 AI agent 宿主）在行程內 dispatch speclink 動詞、以自家資料庫透過 `Store` 物件儲存規格文件，並為任何 harness 渲染流程知識（技能檔）。

它就是 CLI 隨附的那顆 Rust 引擎，以 [napi-rs](https://napi.rs) 綁定，不是重新實作一份。所以動詞行為、`--json` payload 形狀與渲染內容，從結構上就保證一致。Rust SDK 則是 `speclink-core` crate 本身。

這個 SDK 有兩種用法。一是把 Speclink 接進既有流程，例如寫腳本或做內部工具。二是拿它當**自建 server 端的引擎**——官方的 `speclink-server` 只是 Host 契約的參考實作，你可以照 `openspec/specs/` 的 `host-runtime` 與 `client-protocol` 做自己那一份，配上自家的認證、資料庫與權限模型，CLI 與桌面 app 照樣接得上。

## 取得方式與平台注意事項

> **尚未發布至 npm。**`@speclink/engine` 目前只能從本 repo 建置取得，npm registry 上沒有這個套件。目前狀態見[專案能力狀態](product-status.zh-TW.md)的 Node SDK 一列；npm 通路要解決什麼、目前到哪、可觀察的下一步見[專案路線圖](roadmap.zh-TW.md)。

從 repo 建置並載入：

```bash
git clone https://github.com/MomoChenisMe/speclink.git
cd speclink/crates/speclink-node
npm ci
npm run build          # napi 建置出本機平台的 .node
```

在你的專案中以路徑引用建置產物：

```js
const { createEngine } = require('/path/to/speclink/crates/speclink-node')
```

- 這是一個 **native module**。引擎是編譯後的 Rust，以 Node addon 載入。上述 `npm run build` 只產出**當前平台**的二進位，所以要在部署目標平台上執行，或針對該平台交叉建置。
- 建置需要 Rust 工具鏈（`rustup`）。
- 引擎支援五個平台：Windows x64、macOS x64 與 arm64，以及 Linux x64 與 arm64（皆為 glibc）。發布至 npm 之後，這些平台會以預編譯子套件提供，屆時不再需要工具鏈。

## createEngine——兩種儲存形式

```js
const { createEngine } = require('@speclink/engine')
```

**內建 fs 儲存**——把引擎指向本地專案根目錄（含 `openspec/` 的目錄）。零橋接成本，適合本地工具與測試：

```js
const engine = createEngine({ store: { type: 'fs', root: '/path/to/project' } })
// 選填：specDir（預設 "openspec"）
```

**宿主 Store 物件**——自行實作儲存介面（例如接 Postgres），引擎透過它讀寫文件：

```js
const engine = createEngine({ store: myStore })
```

每個 `Store` 方法可以回傳值**或 Promise**，橋接層兩者都接受。物件缺少必要方法時，`createEngine` 會同步拋錯並列出所有缺少的方法名。這是 fail fast，不會產生引擎實例。

**`actor`（選填）——這顆引擎的操作者身分。** 兩種儲存形式都收，格式是 `"Name <email>"`：

```js
const engine = createEngine({ store: myStore, actor: 'Alice <alice@example.com>' })
```

它決定引擎蓋下的每個章歸誰：`created_by`（`new change`）、`reviewed_by`／`verified_by`（`review stamp`／`verify stamp`）。

- **一個實例一個身分。** 身分在建構時綁定，`dispatch` 刻意沒有身分參數——呼叫端無從冒用別人。多人系統的用法是每個請求（或每個身分）開一顆 engine 實例；建構成本只是一個物件，不是連線池。
- **沒給的時候**：fs 形式回退到該 workspace 的 git identity（與 CLI 蓋章逐位元一致）；宿主 Store 形式沒有本地 workspace，就不蓋身分（維持匿名）。trim 後為空字串視同沒給。
- **誰可以宣稱哪個身分，是你的事。** 認證與權限判定屬於宿主，SDK 只收結果。

> **警告——絕不要在 Store 方法內同步回呼引擎。** `dispatch` 在背景工作執行緒上等待你的 store 方法解決。若某個 store 方法同步阻塞等待同一顆引擎的另一個 `engine.dispatch(...)`，會形成互等循環。在 store 方法回傳*之後*（或無關的程式碼中）發起新的 dispatch 沒有問題——並發 dispatch 是支援且被測試覆蓋的。

## Store 介面——實作指南

這個介面與引擎核心的儲存縫線一對一，也就是 `speclink-core` 的 `Store` trait，命名採 camelCase。引擎只講領域詞彙：change、artifact、delta 與 canonical spec、討論、workflow config。實體佈局由你的實作決定。

完整簽名見 [`index.d.ts`](../crates/speclink-node/index.d.ts)。`path` 與 `dir` 的回傳值是**呈現在 payload 裡的字串**，不是引擎會去開的檔案路徑。

| 分組 | 方法 | 說明 |
|---|---|---|
| Changes | `listChanges`、`findChange`、`changeExists`、`createChange`、`updatedAtSecs` | `listChanges` 回傳 `{name, dir?, meta?}` 且按名稱排序；`meta` 對應 `.openspec.yaml`（`schema`、`created`、`createdBy`、`createdWith`、`fromDiscussion`）。`updatedAtSecs` 是「最近更新」排序鍵（整數秒；change 不存在 → 0）。 |
| Artifacts | `readArtifact`、`writeArtifact`、`artifactExists`、`deleteArtifact`（選配） | artifact 識別碼是 schema 定義、相對於 change 的輸出路徑：`proposal.md`、`design.md`、`tasks.md`、`specs/<capability>/spec.md`。空文件也算存在。`deleteArtifact` 只有 review／verify 蓋章會用到（蓋章會刪掉工單），沒實作就只有蓋章路徑失敗。 |
| Delta specs | `deltaCapabilities`、`hasCapabilityDirs` | change 內含 delta spec 的 capability 名稱，排序後回傳。 |
| Canonical specs | `listCanonicalCapabilities`、`canonicalSpecExists`、`readCanonicalSpec`、`writeCanonicalSpec`、`canonicalSpecPath` | 專案層級的正典規格，archive 時 delta 併入之處。 |
| Archive | `archivedChangeExists`、`archiveChange`、`readArchivedMeta`、`writeArchivedMeta` | `archiveChange(name, datedName)` 把使用中的 change 移到含日期的封存名下（`YYYY-MM-DD-<name>`）。 |
| Discussions | `liveDiscussionExists`、`archivedDiscussionExists`、`liveDiscussionPath`、`readLiveDiscussion`、`writeLiveDiscussion`、`deleteLiveDiscussion`、`readDiscussion`、`listLiveDiscussions`、`listArchivedDiscussions`、`archiveDiscussion` | 文件以原始文字儲存；解析（輪、結論）是引擎邏輯。`readDiscussion` 先找 live，再找最新的封存候選。 |
| Config／詞彙 | `readWorkflowConfig`、`readLanguage` | `config.yaml` 原文（或 null）與 LANGUAGE 文件（或 null——沒有共用詞彙是正常狀態）。 |
| 選配 | `claim` | 團隊系統的所有權裁決——見下文。 |

wadpilot 式的資料庫映射示意：

```js
// 資料表：changes(name PK, meta JSONB, updated_at)、
//        artifacts(change_name, path, content, PRIMARY KEY (change_name, path))、
//        canonical_specs(capability PK, content)、
//        discussions(slug, text, archived, stored_name)
const store = {
  async listChanges() {
    const rows = await db.query('SELECT name, meta FROM changes ORDER BY name')
    return rows.map((r) => ({ name: r.name, dir: `changes/${r.name}`, meta: r.meta }))
  },
  async readArtifact(change, artifact) {
    const row = await db.maybeOne(
      'SELECT content FROM artifacts WHERE change_name = $1 AND path = $2',
      [change, artifact],
    )
    return row ? row.content : null
  },
  async writeArtifact(change, artifact, content) {
    await db.query(
      `INSERT INTO artifacts (change_name, path, content) VALUES ($1, $2, $3)
       ON CONFLICT (change_name, path) DO UPDATE SET content = $3`,
      [change, artifact, content],
    )
    await db.query('UPDATE changes SET updated_at = now() WHERE name = $1', [change])
    return `changes/${change}/${artifact}`
  },
  async deltaCapabilities(change) {
    const rows = await db.query(
      `SELECT DISTINCT split_part(path, '/', 2) AS cap FROM artifacts
       WHERE change_name = $1 AND path LIKE 'specs/%/spec.md' ORDER BY cap`,
      [change],
    )
    return rows.map((r) => r.cap)
  },
  // ……其餘方法依此類推。
}
```

store 方法拋錯或 reject 時，進行中的 `dispatch` 會以 `Error` 拒絕。message 帶 store 方法名前綴，例如 `readArtifact: connection refused`。`code` 承載 JS 錯誤自己的 `code`，沒有的話就是 `store_error`。

### `claim`（選配）

所有權是團隊系統的概念，引擎不做裁決。若你的 store 實作了 `claim(name)`，`dispatch(['claim', '<name>'])` 會路由過去。

成功時它 resolve 你的 payload，例如 `{ claimed: true, claimedBy: 'you' }`。衝突時它 reject 一個 `Error`：`code` 用動詞契約的 409 reason（`ownership_lost`、`change_busy`、`gate_pending`），message 說明誰持有該 change、該怎麼做。SDK 把兩者原樣傳給呼叫端。

沒有 `claim` 方法時，該動詞就像在 fs store 上一樣直接失敗。

## dispatch——統一入口

```js
const result = await engine.dispatch(['list', '--json'])
const status = await engine.dispatch(['status', '--change', 'add-auth', '--json'])
await engine.dispatch(
  ['new', 'artifact', 'proposal', '--change', 'add-auth', '--stdin'],
  { stdin: '## Why\n…' },
)
```

- **輸入**：字串陣列，與 CLI 動詞詞彙一對一，等同 shell argv 去掉程式名。它不支援互動式輸入。CLI 中讀 stdin 的動詞，改由第二參數傳內容：`{ stdin }`。
- **輸出**：Promise，解析為與 CLI `--json` 完全一致的結構化物件（camelCase 欄位名）。沒有 `--json` 形式的動詞解析為 `{ output: string }`。目前 TypeScript shape 以 [`index.d.ts`](../crates/speclink-node/index.d.ts) 為準；未來遠端 Command/Query payload 由平台藍圖與版本化 Protocol 工作定義。
- **錯誤**：Promise 以 `Error` 拒絕——`message` 是 CLI 的語義化訊息（可直接回給 agent），`code` 分類失敗：`invalid_argv`（argv 有誤）、`not_found`（change／討論查找）、`error`（引擎失敗，即 CLI 的 exit-1 類別）、宿主 store 的 409 reason 原樣傳遞（`ownership_lost`……）、`store_error`（無 code 的 store 失敗）、`panic`。
- **絕不阻塞事件迴圈**：每次 dispatch 都在背景工作執行緒上執行；支援並發 dispatch。

目前已路由的動詞：`list`、`status`、`new change`、`new artifact`、`claim`、`review add-round`、`review stamp`、`verify add-round`、`verify stamp`。詞彙會朝完整 CLI 對等擴充；未支援的動詞以 `invalid_argv` 拒絕。

### 蓋章動詞——`review` 與 `verify`

兩個品質關卡各有兩個動詞，argv 沿用 CLI 詞彙：

```js
// 開一輪：內容走 stdin 參數（與 new artifact --stdin 同一個機制）
await engine.dispatch(['review', 'add-round', 'add-auth', '--stdin'], { stdin: round })
// → { change: 'add-auth', round: 1 }

// 落章：scope 指紋 argv 塞不下，走 stdin 的 JSON
await engine.dispatch(['review', 'stamp', 'add-auth', '--accept', '--agent', 'claude', '--stdin'], {
  stdin: JSON.stringify({
    scope: [{ path: 'src/auth.ts', hash: '0f9c' }],
    missing: [],
  }),
})
// → { change: 'add-auth' }
```

- `scope` 是**你算好的指紋**——宿主沒有工作樹，引擎不會替你重算；`missing` 是工單範圍裡已經不存在的檔。引擎驗「scope ∪ missing ＝工單聯集且不相交」，不合就拒。兩個欄位都可省略（讀作空清單），不帶 `--stdin` 等同兩者皆空。
- 落下的 `reviewed_by`／`verified_by` 就是建構期的 `actor`（見上面 createEngine 段）；`--agent` 落 `reviewed_with`／`verified_with`。
- **守門原封傳遞**：任務沒做完、末輪還有未解的 CRITICAL／WARNING（`--accept` 可豁免必修條件、SUGGESTION 本來就不擋章），都會以引擎的語義化訊息 reject。
- 蓋章成功會**刪掉工單**，所以宿主 Store 需要實作選配的 `deleteArtifact`。

## 渲染 API

為你的 harness 取得流程知識——與 `speclink init`／`update` 共用同一份生成程式碼，內容不會與 CLI 漂移：

```js
const { skills } = require('@speclink/engine')

skills.list() // [{ name: 'propose', description: '…' }, …]

// 渲染矩陣：target（claude|codex|neutral）× invocation（cli|tool-call）
const skillMd = skills.render('propose', {
  target: 'neutral',
  invocation: 'tool-call',
})
```

- `target: 'neutral'` 為自訂 harness 渲染：沒有 `/speclink-` 斜線前綴、沒有 plan-mode 措辭；`toolName`（預設 `"speclink"`）代入 `{{TOOL}}`。
- `invocation: 'tool-call'` 把動詞表述為「以 argv 陣列呼叫 speclink 工具」——對應以 `dispatch` 為後端的 tool；`'cli'` 則表述為 shell 指令。
- 把 `skills.render(...)` 的檔案餵給 agent（例如寫到一個目錄後以 `skillDirectories` 傳入）。路由就在這些檔案裡：每個技能的 `description` 說明何時該用它，結尾的 **Next steps** 段說明跑完之後建議做什麼——不需要、也不再生成任何獨立的 instructions 區塊。

## 完整整合範例——Copilot SDK

一個名為 `speclink`、參數是 argv 陣列的 tool，加上落地的 skills：

```js
const { createEngine, skills } = require('@speclink/engine')
const { defineTool, CopilotClient } = require('@github/copilot-sdk') // 示意 import
const { mkdirSync, writeFileSync } = require('node:fs')
const { join } = require('node:path')

const engine = createEngine({ store: myDatabaseStore })

// 1. speclink tool：argv 進、結構化 payload 出；錯誤以文字回給 agent。
const speclinkTool = defineTool('speclink', {
  description:
    'Run a speclink verb. Pass the argv array exactly as the skill documents say, ' +
    "e.g. ['status', '--change', 'add-auth', '--json'].",
  parameters: {
    type: 'object',
    properties: {
      argv: { type: 'array', items: { type: 'string' } },
      stdin: { type: 'string', description: 'Content for verbs that take --stdin' },
    },
    required: ['argv'],
  },
  async handler({ argv, stdin }) {
    try {
      return await engine.dispatch(argv, stdin === undefined ? undefined : { stdin })
    } catch (err) {
      // err.message 是語義化訊息——直接交給 agent。
      return { error: err.message, code: err.code }
    }
  },
})

// 2. 生成 skill 檔案（一次性或開機時），以 skillDirectories 餵入。
const skillsRoot = join(process.cwd(), '.wad', 'skills')
for (const { name } of skills.list()) {
  const dir = join(skillsRoot, `speclink-${name}`)
  mkdirSync(dir, { recursive: true })
  writeFileSync(
    join(dir, 'SKILL.md'),
    skills.render(name, { target: 'neutral', invocation: 'tool-call' }),
  )
}

// 3. 接進 agent session。路由不需要額外的 system prompt：每個技能的
//    description 就寫著什麼情境該用它。
const client = new CopilotClient({
  tools: [speclinkTool],
  skillDirectories: [skillsRoot],
})
```

生成的 skills 以 speclink tool 呼叫表述動詞，tool 把它們路由進行程內的引擎，引擎再透過你的 store 持久化——沒有 CLI、沒有子行程、沒有本地 `openspec/` 樹。

前提是你的 harness 會載入這些技能的 description；不載入的 harness 等於沒有流程路由。

## 延伸閱讀

- [`index.d.ts`](../crates/speclink-node/index.d.ts)——目前發布的 Node API 與 payload types。
