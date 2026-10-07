# Node SDK（@speclink/engine）

**繁體中文** · [English](sdk-node.md)

`@speclink/engine` 讓你在 Node.js 程式裡直接執行 Speclink 的動詞、把規格存進自己的資料庫，並為任何 AI 工具產生技能檔。它就是 CLI 用的那顆 Rust 引擎，透過 [napi-rs](https://napi.rs) 接到 Node，不是另外重寫的版本，所以動詞行為、`--json` 輸出形狀與技能內容都和 CLI 一致。

常見的兩種用法：

- 把 Speclink 接進既有流程，例如腳本、內部工具，或執行 AI agent 的服務。
- 拿它當**自建 server 的引擎**。官方的 `speclink-server` 只是參考實作；遠端模式由 `openspec/specs/` 底下的 `host-runtime` 與 `client-protocol` 兩份公開契約定義。你可以照這兩份契約，用這個引擎寫自己的 server，接上自家的認證、資料庫與權限模型，CLI 與桌面 app 一樣接得上。官方 server 的用法見 [Remote 入門](remote-getting-started.zh-TW.md)。

## <a id="install"></a>安裝

```bash
npm install @speclink/engine
```

0.2.0 起可從 npm 安裝，版號跟著 Speclink 的 release 走。需要 Node.js 18 以上。Windows x64、macOS x64 與 arm64、Linux x64 與 arm64（glibc）五個平台都有預先編譯好的套件，安裝時只下載你的平台那一份，不需要 Rust。Alpine 這類 musl Linux 沒有預編譯套件。

```js
const { createEngine, skills } = require('@speclink/engine')
```

### 替代路徑：自 repo 建置

要改引擎本身，或你的平台沒有預編譯套件時，從原始碼建置（需要 Rust 工具鏈）：

```bash
git clone https://github.com/MomoChenisMe/speclink.git
cd speclink/crates/adapters/speclink-node
npm ci
npm run build          # 為目前的平台建出 .node 檔
```

然後用路徑引用：

```js
const { createEngine } = require('/path/to/speclink/crates/adapters/speclink-node')
```

`npm run build` 只建出目前平台的檔案，所以要在部署的目標平台上建置。

## <a id="create-engine"></a>建立引擎

`createEngine` 有兩種儲存方式。

**本機檔案**：指向含 `openspec/` 的專案根目錄，適合本機工具與測試。

```js
const engine = createEngine({ store: { type: 'fs', root: '/path/to/project' } })
// 選填：specDir（預設 "openspec"）
```

**自己的 Store**：傳入一個實作 Store 介面的物件（見[自己實作 Store](#store)），引擎透過它讀寫文件。

```js
const engine = createEngine({ store: myStore })
```

物件缺少必要方法時，`createEngine` 會立刻拋錯，並列出所有缺少的方法名。

**`actor`（選填）**：這顆引擎的操作者，格式是 `"Name <email>"`。引擎記下的 `created_by`（`new change`）、`reviewed_by`（`review stamp`）與 `verified_by`（`verify stamp`）都是它。

```js
const engine = createEngine({ store: myStore, actor: 'Alice <alice@example.com>' })
```

- 一個實例只有一個身分。`dispatch` 沒有身分參數，呼叫端無法冒用別人。多人系統請每個請求（或每個使用者）建一個實例；建立成本只是一個物件。
- 沒給 `actor` 時，本機檔案形式用該 workspace 的 git 身分（和 CLI 相同）；自己的 Store 則不記身分。只有空白也視同沒給。
- 誰可以用哪個身分，由你的系統決定。SDK 只接收結果。

## <a id="dispatch"></a>執行動詞

```js
const list = await engine.dispatch(['list', '--json'])
const status = await engine.dispatch(['status', '--change', 'add-auth', '--json'])
await engine.dispatch(
  ['new', 'artifact', 'proposal', '--change', 'add-auth', '--stdin'],
  { stdin: '## Why\n…' },
)
```

- **輸入**：字串陣列，和 CLI 的寫法一樣，只是去掉程式名。CLI 從 stdin 讀內容的動詞，改用第二個參數 `{ stdin }` 傳入。
- **輸出**：Promise，結果和 CLI `--json` 的輸出相同（欄位是 camelCase）。沒有 `--json` 形式的動詞（例如 `new change`）回傳 `{ output: string }`。TypeScript 型別是 `Promise<unknown>`，欄位形狀見[動詞與旗標契約](verb-contract.zh-TW.md)。
- **不會卡住事件迴圈**：每次 dispatch 都在背景執行緒執行，可以同時發出多個。

目前接上的動詞：`list`、`status`、`new change`、`new artifact`、`claim`、`review add-round`、`review stamp`、`verify add-round`、`verify stamp`。其他動詞會以 `invalid_argv` 拒絕。

失敗時 Promise 以 `Error` 拒絕。`message` 和 CLI 印出的訊息相同，可以直接交給使用者或 agent；`code` 是分類：

| `code` | 意思 |
| --- | --- |
| `invalid_argv` | 參數有誤，或這個動詞還沒接上 SDK |
| `not_found` | 找不到 change 或討論 |
| `invalid_config` | 設定檔存在但無法解析（不會改用預設值） |
| `refused` | 前置條件不成立 |
| `error` | 其他引擎錯誤。在本機檔案形式或沒實作 `claim` 的 Store 上執行 `claim`，也會得到這個 |
| Store 自訂的 code | Store 方法拋出帶 `code` 的錯誤時原樣傳回，例如 `ownership_lost` |
| `store_error` | Store 方法拋出沒有 `code` 的錯誤 |
| `panic` | 引擎內部錯誤 |

## <a id="store"></a>自己實作 Store

Store 是一個物件，引擎透過它讀寫 change、artifact、規格與討論。資料要怎麼存（資料表、檔案、物件儲存）由你決定。完整的方法簽名見 [`index.d.ts`](../crates/adapters/speclink-node/index.d.ts)。每個方法可以直接回傳值，也可以回傳 Promise。

**必要方法**（共 31 個）：

| 分組 | 方法 |
| --- | --- |
| Change | `listChanges`、`findChange`、`changeExists`、`createChange`、`updatedAtSecs` |
| Artifact | `readArtifact`、`writeArtifact`、`artifactExists` |
| Delta 規格 | `deltaCapabilities`、`hasCapabilityDirs` |
| 正式規格 | `listCanonicalCapabilities`、`canonicalSpecExists`、`readCanonicalSpec`、`writeCanonicalSpec`、`canonicalSpecPath` |
| 封存 | `archivedChangeExists`、`archiveChange`、`readArchivedMeta`、`writeArchivedMeta` |
| 討論 | `liveDiscussionExists`、`archivedDiscussionExists`、`liveDiscussionPath`、`readLiveDiscussion`、`writeLiveDiscussion`、`deleteLiveDiscussion`、`readDiscussion`、`listLiveDiscussions`、`listArchivedDiscussions`、`archiveDiscussion` |
| 設定與詞彙 | `readWorkflowConfig`、`readLanguage` |

**選配方法**：

| 方法 | 什麼時候需要 |
| --- | --- |
| `readChangeMeta`、`writeChangeMeta`、`deleteArtifact` | `review stamp`、`verify stamp`。缺任何一個，蓋章會在動手前拒絕 |
| `claim` | `claim` 動詞。見下方 |
| `deleteChange`、`readEvidence`、`writeEvidence` | 目前接上的動詞用不到，可以先不實作 |

實作時要注意：

- **change 的 meta 兩邊命名不同。** `createChange(name, metaText)` 收到的是 YAML 文字，鍵名用底線，例如 `created_by: Alice <alice@example.com>`。`listChanges` 與 `findChange` 要回傳物件，meta 的鍵名用駝峰，例如 `createdBy`。
- **回傳型別要對。** 布林、數字、字串型別不對時，引擎不會報錯，而是當成預設值（例如 `changeExists` 回傳非布林會被當成 `false`）。`listChanges`、`findChange` 的形狀不對則會報錯。
- **`path`、`dir` 這類回傳值只是顯示用的字串**，引擎不會拿它去開檔案。
- **artifact 的識別碼**是相對於 change 的路徑：`proposal.md`、`design.md`、`tasks.md`、`specs/<capability>/spec.md`。空文件也算存在。
- **只能用內建 schema。** 自己的 Store 沒有本機 workspace，所以專案層與使用者層的自訂 schema 都讀不到。
- **錯誤處理**：Store 方法拋錯或 reject 時，進行中的 `dispatch` 會以 `Error` 拒絕，訊息前面加上方法名，例如 `listChanges: connection refused`。
- **不要在 Store 方法裡同步等待同一顆引擎的另一個 `dispatch`**，兩邊會互相等待而卡住。Store 方法回傳之後再發新的 dispatch 沒有問題。

下面是用 `Map` 存在記憶體的片段，示範 meta 的兩種命名：

```js
const changes = new Map() // name → { metaText, artifacts: Map }

const store = {
  createChange(name, metaText) {
    // metaText 例如 "schema: spec-driven\ncreated: 2026-10-07\ncreated_by: Alice <alice@example.com>\n"
    changes.set(name, { metaText, artifacts: new Map() })
    return `changes/${name}`
  },
  findChange(name) {
    const c = changes.get(name)
    if (!c) return null
    const meta = parseYaml(c.metaText) // 用你慣用的 YAML 套件
    return { name, meta: { schema: meta.schema, created: meta.created, createdBy: meta.created_by } }
  },
  listChanges() {
    return [...changes.keys()].sort().map((name) => store.findChange(name))
  },
  changeExists: (name) => changes.has(name),
  readArtifact: (change, artifact) => changes.get(change)?.artifacts.get(artifact) ?? null,
  writeArtifact(change, artifact, content) {
    changes.get(change).artifacts.set(artifact, content)
    return `changes/${change}/${artifact}`
  },
  // ……其餘必要方法依此類推
}
```

### `claim`（選配）

認領（claim）是團隊系統的概念，由你的 Store 決定誰可以認領。實作 `claim(name)` 之後，`dispatch(['claim', '<name>'])` 會呼叫它：

- 成功時回傳你自己的資料，例如 `{ claimed: true, claimedBy: 'alice' }`，SDK 原樣交給呼叫端。
- 衝突時拋出帶 `code` 的 `Error`（例如 `ownership_lost`），訊息寫明誰持有這個 change、該怎麼做。SDK 也原樣傳回。

沒有實作 `claim` 時，這個動詞以 `error` 失敗。

## <a id="stamp"></a>品質關卡：`review` 與 `verify`

兩個品質關卡各接上兩個動詞，參數和 CLI 相同：

```js
// 新增一輪：內容用 stdin 參數傳入
await engine.dispatch(['review', 'add-round', 'add-auth', '--stdin'], { stdin: round })
// → { change: 'add-auth', round: 1 }

// 蓋章：檔案指紋放進 stdin 的 JSON
await engine.dispatch(['review', 'stamp', 'add-auth', '--agent', 'claude', '--stdin'], {
  stdin: JSON.stringify({
    scope: [{ path: 'src/auth.ts', hash: '<sha256>' }],
    missing: [],
  }),
})
// → { change: 'add-auth' }
```

- `scope` 是**你算好的檔案指紋**。引擎看不到你的工作目錄，所以不會替你計算。要和 CLI 相容，就用檔案內容的 SHA-256（十六進位）；文字檔先把 CRLF 換成 LF。
- `missing` 列出工單範圍裡已經不存在的檔案。`scope` 與 `missing` 合起來必須剛好等於工單涵蓋的檔案、而且不重複，否則拒絕。兩個欄位都可以省略（視為空清單），多出其他欄位會以 `invalid_argv` 拒絕。
- 任務還沒做完（標 `[M]` 的手動任務不算），或最後一輪還有 CRITICAL／WARNING 時，蓋章會被拒絕。`--accept` 可以略過必修發現；SUGGESTION 本來就不擋。
- `reviewed_by`／`verified_by` 是建立引擎時的 `actor`；`--agent` 記在 `reviewed_with`／`verified_with`。
- 蓋章需要 `readChangeMeta`、`writeChangeMeta`、`deleteArtifact` 三個方法，缺一個就在動手前拒絕，工單與 meta 都不會被改。
- 同一個引擎實例裡的蓋章會自動排隊，不會互相覆蓋。跨實例、跨程序對同一個 change 蓋章時，由你的 Store 負責協調。

## <a id="render"></a>產生技能檔

`skills` 用的是和 `speclink init`／`speclink update` 同一份產生程式，內容不會和 CLI 有落差。

```js
skills.list() // [{ name: 'analyze', description: '…' }, …]

const skillMd = skills.render('propose', { target: 'neutral', invocation: 'tool-call' })
```

| 選項 | 值 | 說明 |
| --- | --- | --- |
| `target` | `claude`、`codex`、`neutral` | `neutral` 給自訂工具用：沒有 `/speclink-` 斜線指令，也不提 plan mode |
| `invocation` | `cli`（預設）、`tool-call` | `tool-call` 把動詞寫成「呼叫 speclink 工具，參數是 argv 陣列」，搭配以 `dispatch` 實作的工具；`cli` 寫成 shell 指令 |
| `specDir` | 字串 | 技能內文裡的規格目錄名，預設 `openspec` |
| `toolName` | 字串 | 只用於 `neutral`：技能內文裡的工具名稱，預設 `speclink` |

把產生的 `SKILL.md` 寫進一個目錄，交給你的 agent 載入。每個技能的 `description` 說明什麼時候用它，結尾的 Next steps 說明之後建議做什麼，不需要另外的系統提示。前提是你的 agent 會讀技能的 description。

## <a id="example"></a>完整範例：接上 Copilot SDK

一個名為 `speclink`、參數是 argv 陣列的工具，加上產生好的技能檔。Copilot SDK 的 API 以它的官方文件為準；這個範例對照的是 `@github/copilot-sdk` 1.0.16。

```js
const { createEngine, skills } = require('@speclink/engine')
const { CopilotClient, defineTool } = require('@github/copilot-sdk')
const { mkdirSync, writeFileSync } = require('node:fs')
const { join } = require('node:path')

const engine = createEngine({ store: myDatabaseStore, actor: 'Alice <alice@example.com>' })

// 1. speclink 工具：argv 進、結果出；錯誤以文字交回 agent。
const speclinkTool = defineTool('speclink', {
  description:
    'Run a speclink verb. Pass the argv array exactly as the skill says, ' +
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
      return { error: err.message, code: err.code }
    }
  },
})

// 2. 產生技能檔。
const skillsRoot = join(process.cwd(), '.my-agent', 'skills')
for (const { name } of skills.list()) {
  const dir = join(skillsRoot, `speclink-${name}`)
  mkdirSync(dir, { recursive: true })
  writeFileSync(join(dir, 'SKILL.md'), skills.render(name, { target: 'neutral', invocation: 'tool-call' }))
}

// 3. 建立 session 時交給它工具與技能目錄。
async function startSession() {
  const client = new CopilotClient()
  return client.createSession({ tools: [speclinkTool], skillDirectories: [skillsRoot] })
}
```

技能用工具呼叫的方式描述動詞，工具把呼叫送進同一個程序裡的引擎，引擎再透過你的 Store 存檔。整條路上沒有 CLI、沒有子程序，也不需要本機的 `openspec/` 目錄。

## <a id="limits"></a>目前的限制

- 只接上九個動詞（見[執行動詞](#dispatch)）；`archive`、`task`、`discuss` 等其他動詞還不能透過 SDK 執行。
- 自己的 Store 只能用內建 schema。
- 沒有 musl Linux 的預編譯套件。
- Store 方法拋錯時，stderr 可能會多印一行 Rust 的 panic 訊息；Promise 仍會照上面的規則拒絕。
- `dispatch` 的回傳值沒有個別的 TypeScript 型別，形狀以[動詞與旗標契約](verb-contract.zh-TW.md)為準。
