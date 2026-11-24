## Context

`skills.render` 的宿主限制由 speclink-core 的 `HarnessLimits`（`crates/engine/speclink-core/src/workspace/skills.rs`）表示，目前有兩個欄位：`skills`（宿主提供的技能清單）與 `checkout`（布林）。`harness_limits_note` 把它寫成技能文字開頭的 `## Harness limits` 段落；`render_skill_file_with_limits` 把段落接在既有的前言之後。唯一的呼叫端是 Node SDK 的 `resolve_matrix`（`crates/adapters/speclink-node/src/render.rs`），它把 JS 的 options 轉成 `HarnessLimits`；`crates/adapters/speclink-node/index.js` 先擋掉不認得的鍵與值是 undefined 的鍵。CLI 與入口產生技能時用預設值（沒有限制），所以不會出現這個段落。

`checkout` 的意思是「有沒有程式碼的 checkout 與 git 歷史」，把兩件事綁在一起。內嵌宿主常見的情況是「讀得到程式碼、不能用 git」：給 Agent 的是唯讀的單一 commit 副本，沒有執行 git 的工具。`checkout: false` 會讓模型連程式碼都不讀；省略則留下執行 git 的步驟。

宿主常用的六支技能（discuss、trace、propose、ingest、analyze、config）以 `specInCheckout: false` 產生時，只有 trace 有要執行的 git 步驟（`crates/engine/speclink-core/assets/skills/trace.md`）：前提那一句、evidence 是 null 時查 git log、沒有對到能力時的 git log -S 與 git blame。

## Goals / Non-Goals

**Goals:**

- 宿主能用一個選項宣告「讀得到程式碼、不能用 git」，技能文字自己告訴模型跳過 git 步驟，宿主不用在 system prompt 另外寫規則。
- 不合法的組合（沒有 checkout 卻有 git）在產生時就報錯。
- 不帶新選項的輸出逐位元不變；CLI 產生的技能檔、golden 與內嵌資產版號不變。

**Non-Goals:**

- 不改任何技能原稿的內文（不在 trace 加「不能跑 git 時」的判斷，也不加條件區塊）。
- 不改 `checkout: false` 的項目文字與它對 `specInCheckout` 的連帶效果。
- 不把 `checkout` 改成三種值（例如 `'read-only'`）。
- 不給 CLI 或入口的技能清單加宿主限制。
- 不處理 `instructions` 回應的語言與任務指示（同一份討論的 cut 1）。

## Decisions

### 選項 gitHistory 的形狀與預設

`RenderOptions` 新增選填的 `gitHistory`（布林）。解析規則，和 `specInCheckout` 跟著 `checkout` 的寫法一致：

| `checkout` | `gitHistory` | 結果 |
| --- | --- | --- |
| 省略或 true | 省略或 true | 沒有程式碼方面的限制 |
| 省略或 true | false | 讀得到程式碼、不能用 git |
| false | 省略或 false | 沒有 checkout（現有行為） |
| false | true | 拋錯：兩個選項互相矛盾 |

`gitHistory` 不影響 `specInCheckout` 的預設：只有 `checkout: false` 會讓省略的 `specInCheckout` 變成 false。

替代方案：`checkout: true | false | 'read-only'`。不採用：重點是沒有 git 而不是唯讀（這些技能本來就不改程式碼），而且 napi 的欄位要從布林改成聯合型別。

### 核心以一個三值的型別表示程式碼的存取

`HarnessLimits` 的 `checkout: bool` 換成 `code: CodeAccess`，`CodeAccess` 有三個值：`WithGit`（預設）、`WithoutGit`、`None`。這樣「沒有 checkout 卻有 git」在核心裡寫不出來，`harness_limits_note` 只需要對三個值各自決定要不要加項目；矛盾的組合只在 Node 這一層把兩個布林轉成 `CodeAccess` 時檢查一次。`HarnessLimits` 的建構只有 `resolve_matrix` 與核心的測試兩處，改欄位的影響面就是這兩處。

替代方案：在 `HarnessLimits` 再加一個 `git_history: bool`。不採用：四種組合裡有一種不合法，每個讀這個結構的地方都要記得那條規則。

### Harness limits 的新項目與順序

`CodeAccess::WithoutGit` 時，`## Harness limits` 加一個項目，文字固定為：

```
- This harness has a copy of the code that you can read and search, but it has no git history and cannot run git. Skip every step of this document that runs a git command. When a step names another way to get the same fact, use that way. In your answer, say which steps you skipped for this reason.
```

`CodeAccess::None` 的項目文字不變（它已經說明沒有 git 歷史）。一次呼叫最多只有其中一個。順序不變：技能清單的項目在前，程式碼的項目在後。項目是英文，不加全大寫強調字。

這個項目對 trace 的效果：前提的「requires git」由這個項目蓋過；evidence 是 null 的步驟改走它已經寫明的後備（用討論與提案的內容回答）；沒有對到能力的步驟照常用關鍵字搜尋程式碼，跳過 git log -S 與 git blame，並在回答裡說明。

### Node 這一層的檢查

- `crates/adapters/speclink-node/index.js`：`RENDER_OPTIONS` 加入 `gitHistory`；鍵在而值是 undefined 時拋 TypeError，訊息的形式和 `checkout` 相同（`skills.render: gitHistory must be true or false, got undefined (leave the key out to follow checkout)`）。
- `crates/adapters/speclink-node/src/render.rs`：`RenderOptions` 加 `git_history: Option<bool>`；`resolve_matrix` 把 `(checkout, gitHistory)` 轉成 `CodeAccess`，`checkout: false` 加 `gitHistory: true` 時回傳錯誤，訊息含 `checkout: false` 與 `gitHistory: true` 兩段文字並說明互相矛盾。非布林值（含 null）由和 `checkout` 相同的機制拒絕。
- `crates/adapters/speclink-node/index.d.ts`：`RenderOptions` 加 `gitHistory?: boolean` 與說明；`checkout` 的說明補一句指向 `gitHistory`。

## Implementation Contract

**行為**

- `skills.render('trace', { target: 'neutral', invocation: 'tool-call', specInCheckout: false, gitHistory: false })` 的回傳字串有 `## Harness limits` 段，段裡恰好一個項目，含 `cannot run git` 與 `Skip every step of this document that runs a git command`，不含 `reads code files`；段落前後的文字合起來等於不帶 `gitHistory` 的回傳字串。
- 同時給 `skills` 與 `gitHistory: false` 時段裡有兩個項目，技能清單在前。
- `{ checkout: false }` 與 `{ checkout: false, gitHistory: false }` 的回傳字串相同，且等於現在 `{ checkout: false }` 的輸出。
- `{ gitHistory: true }`（`checkout` 省略或 true）的回傳字串等於不帶這個鍵的輸出，沒有 `skills` 時不含 `## Harness limits`。
- `gitHistory: false` 不改變 `specInCheckout` 的預設：`{ target: 'neutral', gitHistory: false }` 的內文是 `specInCheckout: true` 的那一份。

**介面與資料形狀**

`RenderOptions.gitHistory?: boolean`。沒有新的匯出名稱、命令、旗標或設定鍵；沒有序列化格式的變更。

**失敗模式**（都同步拋出錯誤、不回傳字串）

| 選項 | 錯誤 |
| --- | --- |
| `gitHistory: 'false'`、`gitHistory: null` | 不是布林值 |
| 鍵在而值是 undefined | TypeError，訊息含 `gitHistory must be true or false` |
| `checkout: false, gitHistory: true` | 訊息指出兩個選項互相矛盾 |
| 打錯的鍵（例如 `gitHistroy`） | 不認得的選項鍵，訊息列出的合法鍵含 `gitHistory` |

**驗收**

- `cargo test -p speclink-core skills`：`harness_limits_note` 對三個 `CodeAccess` 值的單元測試，以及 trace 帶 `WithoutGit` 時內文不變的測試。
- `npm --prefix crates/adapters/speclink-node run build` 之後 `npm --prefix crates/adapters/speclink-node test`：`render.spec.ts` 涵蓋上面每一條行為與每一列失敗模式。
- `cargo test -p speclink-core --test it render_golden` 不需要更新任何 golden 就通過（CLI 產生的技能檔不變）。

**範圍**

- 範圍內：`HarnessLimits` 與 `harness_limits_note`、Node 的選項解析與型別定義、`render.spec.ts`、`docs/sdk-node.md` 與 `docs/sdk-node.zh-TW.md` 的選項表與範例、`docs/product-status.md` 與 `docs/product-status.zh-TW.md` 裡 Node SDK 那一列。
- 範圍外：Non-Goals 列的每一項；技能原稿、golden 快照、`ASSET_VERSION`、`assets.lock`；speclink-cli、speclink-host、speclink-server、apps/desktop。

## Risks / Trade-offs

- [回歸對照] 換掉 `HarnessLimits` 的欄位時，預設值寫錯會讓 CLI 產生的技能檔多出段落。→ `render_golden` 的十份快照是不帶限制的輸出，任何多出來的段落都會讓它紅；另有核心測試斷言預設值沒有限制。
- [一般性的指示] 新項目是一句通用的話，不是逐步改寫技能；模型仍可能在某個 git 步驟上做錯。→ 和 `checkout: false` 的做法一致；目前六支技能裡只有 trace 有 git 步驟，而且兩個步驟都有或部分有不用 git 的做法。之後觀察到實際的錯誤再考慮改技能內文。
- [選項變多] 兩個布林有一種不合法的組合。→ 在產生時拋錯，訊息指出矛盾；核心用三值型別，不合法的組合進不了引擎。
- [跨平台] 改動只有字串與選項的解析，不碰路徑、檔案系統與子程序，Windows、macOS、Linux 行為相同。
- [儲存解耦] 不涉及規格的儲存；這個選項讓「程式碼在哪裡、能用什麼工具」由宿主宣告，引擎不去探測環境。
