# 設定說明

> **文件狀態**：本文描述目前已實作的本地 workspace 設定介面。遠端模式的 workflow policy 以具版本的 Store 為正典，本機 override 不得靜默覆寫團隊政策。規則的正典有兩份：`openspec/specs/workflow-config` 管欄位與解析順序，`openspec/specs/server-policy-write` 管遠端讀寫與授權。

Speclink 的設定分散在兩個檔案與一個目錄，各自有明確的歸屬規則：

| 位置 | 承載 | 跟著誰走 |
|---|---|---|
| `openspec/config.yaml` | 工作流政策：`locale`、`spec_locale`、`tdd`、`audit`、`worktree`，以及 `schema`、`context`、`rules` | **spec store**（spec 文件所在之處） |
| `.speclink.yaml` | workspace 綁定：`tools`（為哪些 AI harness 生成檔案）、`spec_dir`（spec store 在哪裡） | **repo checkout** |
| `.speclink/` | 主機工作資料：touched 記錄、archive 快照、生成工具足跡 | **本機**（gitignored） |

「這個設定該放哪」的判定規則：

- **政策跟 store。** 會改變工作流產出的設定（artifact 語言、spec 語言、TDD 紀律、audit 紀律、worktree 流程開關）放在 `openspec/config.yaml`。無論本地或透過遠端 store 讀 spec 的人，看到的都是同一份真相。
- **綁定跟 repo。** `.speclink.yaml` 只描述「這個 checkout」如何連上 store、接了哪些 AI 工具，不承載政策。
- **個人差異跟環境變數。** `SPECLINK_*` 環境變數在單一 shell 或單一 CI job 內覆寫一切，不動任何檔案。

## Remote 模式的設定歸屬

接上 Remote Store 之後，歸屬規則不變，只是「store 在哪裡」換了地方：

| 位置 | 承載 | 備註 |
|---|---|---|
| server 的 `config.yaml` | 工作流政策：`locale`、`spec_locale`、`tdd`、`audit`、`worktree`、`schema`、`context`、`rules` | 政策跟著 store 走，所以遠端模式下這份才是團隊真相。`speclink workflow-config` 是 Dual 動詞，遠端模式作用於這一份。 |
| `.speclink.yaml` 的 `remote:` 區段 | `url`（project-scoped 連線網址）、`repo`（這個 checkout 在遠端的註冊名稱） | 由 `speclink link` 寫入、`speclink unlink` 移除。`url` 也可以改由 `SPECLINK_STORE_URL` 提供，讓進版本控制的檔案不必寫死網址。 |
| OS 的 Keychain | 存取憑證（device 憑證族與 PAT） | 以連線 origin 為鍵，**不落在任何專案檔案裡**——所以 `.speclink.yaml` 可以安心進版本控制。 |
| `.speclink/` | 唯讀的 Context Projection 與本機工作資料 | gitignored。不要手動編輯投影：那不算遠端寫入，下一個命令會判定它已被改動而拒絕。 |

有兩條紅線。

第一，**本機 override 不得靜默覆寫團隊政策**。遠端模式的政策正典是具版本的 Store，寫入一律經 CAS，而且先由引擎檢查（見 `openspec/specs/server-policy-write`）。

第二，`remote:` 區段存在但缺 `url` 時，不會靜默退回本地模式，而是在下游明確失敗。

怎麼建立連線、怎麼登入、失聯怎麼回來，都不在本文。見[Remote 入門教學](remote-getting-started.zh-TW.md)。

## 解析順序

有效政策值經三層解析，先命中者勝：

| 優先序 | 層 | 說明 |
|---|---|---|
| 1（最高） | `SPECLINK_LOCALE`／`SPECLINK_SPEC_LOCALE`／`SPECLINK_TDD`／`SPECLINK_AUDIT`／`SPECLINK_WORKTREE` | 布林變數僅接受 `true`／`false`（不分大小寫）；其他值——`yes`、`1`、空字串——視為**未設定**，落到下一層。 |
| 2 | `openspec/config.yaml` | 正典歸屬。 |
| 3（最低） | 內建預設 | `locale` 未設定＝English、`tdd`＝false、`audit`＝false、`worktree`＝false。 |

政策鍵（`locale`、`spec_locale`、`tdd`、`audit`、`worktree`）寫在 `.speclink.yaml` 裡一律不生效、也不產生警告——檔案照常解析，這些鍵單純被忽略。若你的檔案帶有這些鍵，把它們以相同的值搬進 `openspec/config.yaml` 即可。

## 用 `workflow-config` 動詞管理 `openspec/config.yaml`

`speclink workflow-config` 管的是**工作流政策檔**。`speclink config` 管的是與專案無關的全域 KV 存放。兩者互不相干。

| 子指令 | 作用 |
|---|---|
| `show [--json]` | 顯示政策五欄、`context`（行數）與 `rules`（各節條數）。顯示**正典值**——不套用環境變數覆寫（有效值的三層解析屬 `speclink instructions` 的職責）。`--json` payload 欄位為 camelCase：`locale`、`specLocale`、`tdd`、`audit`、`worktree`、`context`、`rules`；未設定的欄位為 `null`，未設定的布林為 `false`。 |
| `set <key> <value>` | 寫入 `locale`、`spec_locale`、`tdd`、`audit`、`worktree` 之一。其他鍵以非 0 exit code 拒絕；`tdd`／`audit`／`worktree` 僅接受 `true`／`false`；`locale` 僅接受代碼 `tw`／`ja`／`en`、`spec_locale` 僅接受 `tw`／`ja`／`en`／`auto`（大小寫敏感）——顯示名稱（如「繁體中文」）會被拒絕並列出合法代碼。設為 `false`（或 locale 設為空字串）＝**移除該鍵**，維持「未設定＝預設」語意。 |
| `context --stdin` | 以 stdin 全文設定 `context`；內容僅空白時移除該鍵。 |
| `rules <artifact> --stdin` | 整節代換該 artifact 的規則（一行一條、空行忽略）；stdin 為空時移除該節。`artifact` 限目前 schema 的 artifact id，未知 id 以非 0 exit code 拒絕。 |

**`worktree` 的寫入比其他四個鍵多兩件事。** 它會連動技能足跡：開啟時生成兩個 worktree 技能，關閉時清掉，範圍等同 `speclink update`。

而且**由 `true` 改成 `false` 時，若還有活躍的 linked worktree，指令會拒絕寫入**——政策一關，收尾用的 merge 技能就沒了，那些 worktree 會卡在半路。拒絕時它逐列印出每個 worktree 的變更名、分支與路徑，並要你先跑 `speclink-worktree-merge` 收乾淨；`openspec/config.yaml` 位元組不動、技能足跡也不動。政策本來就是關的時候不擋（那是 no-op 寫入）。

另一個邊界：config 寫進去了但技能同步失敗時，**寫入仍然成立**（config 才是正典），錯誤會浮到 stderr 並要你重跑 `speclink update` 重建足跡。

三個寫入子指令都支援 `--dry-run`。它會印出 unified diff 至 stdout、不寫入任何檔案、以 0 結束。預覽與實寫走完全相同的改寫路徑，所以 diff 就是會落檔的內容。

```bash
speclink workflow-config set tdd true --dry-run   # 先看
speclink workflow-config set tdd true             # 再落檔
cat CONTEXT.md | speclink workflow-config context --stdin
```

**fs 與 remote 模式。** 模式由既有 binding 判定。fs 模式直接讀寫 `openspec/config.yaml`。remote 模式經連線讀取 server 端 config 文件與其版本，套用同一改寫，寫回時附帶該版本。

版本識別不出現在指令介面。被他人並行改寫時，指令以非 0 exit code 提示你重新執行，絕不覆蓋他人的寫入。離線或認證失效同樣以非 0 exit code 失敗，不暫存也不排隊。

**既知取捨：模板註解會喪失。** 寫入是 read-modify-write：讀進整份文件、改目標鍵、整份寫回。其他鍵值一律保留，但原檔的模板註解在重寫後不再存在。桌面設定頁是同一個取捨。

首次使用先跑 `--dry-run` 看 diff 再決定。文件無法解析時一律 fail-closed，讀寫都以非 0 exit code 拒絕——重寫壞檔會毀掉既有內容。

內建技能 `speclink-config` 就建立在這個動詞之上。它從 codebase 的固定來源整理 `context` 與 `rules`，一律先產 diff 交使用者裁決，再寫入。

## 自訂工具描述子

`tools` 清單除內建名（`claude`、`codex`）外，也接受描述子物件，用於任何其他 AI harness：

```yaml
tools:
  - claude
  - name: wad-harness
    skills_dir: .wad/skills
    invocation: tool-call
```

| 欄位 | 必填 | 規則 |
|---|---|---|
| `name` | 是 | kebab-case，2–50 字元、限 `[a-z0-9-]`；不得與內建工具名衝突 |
| `skills_dir` | 是 | 專案根相對路徑；不得逸出專案根 |
| `instructions_file` | 否 | **已棄用**——引擎不再往它生成任何東西。保留只為兩件事：舊設定檔仍能解析，以及 `speclink update` 知道去哪裡剝除遺留的 `SPECLINK` 區塊。欄位存在時仍會驗證（專案根相對、不得逸出）；留著它會在 stderr 得到一行棄用提示 |
| `invocation` | 否 | `cli`（預設）或 `tool-call`——決定生成文字如何指示 harness 執行 speclink 動詞：「執行 `speclink <動詞>`」vs「呼叫 speclink 工具（參數為 argv 陣列）」 |

四種情況會讓指令以非 0 exit code 結束：名稱衝突、大小寫違規、路徑逸出、`invocation` 值非法。每一種都輸出指明欄位的單行錯誤訊息。

描述子與內建工具共享完整生命週期：

- **生成**——`speclink init`／`speclink update` 在 `skills_dir` 下寫入 `speclink-*/SKILL.md` 技能檔。就這樣：指令檔已退出受管集合。
- **同步**——`speclink update` 對仍在清單上的描述子全部重新生成。
- **清理**——把描述子從 `tools` 移除。下一次 `speclink update` 會刪除它的 `speclink-*` 技能目錄，並一併移除因此變空的目錄。描述子若仍寫著 `instructions_file`，該檔的遺留 `SPECLINK` 區塊也會被剝除；剝除後全空的話整檔刪除。先拿掉 `instructions_file` 才移除描述子的話，引擎就不知道那個檔案在哪——請自行手動刪除。

描述子生成的內容採用**中性渲染**：沒有 `/speclink-` slash 前綴，沒有 plan mode 參照，動詞措辭由 `invocation` 決定。內建 claude 與 codex 的輸出完全不受影響。

## 參考：全部鍵值

### `openspec/config.yaml`

| 鍵 | 預設 | 意義 |
|---|---|---|
| `schema` | `spec-driven` | 新 change 使用的工作流 schema |
| `locale` | English | AI 生成 artifact 的語言（`tw`、`ja`…） |
| `spec_locale` | English | spec 檔語言；`auto` 跟隨 `locale` |
| `tdd` | `false` | 要求實作遵循 Red-Green-Refactor 紀律 |
| `worktree` | `false` | 開啟 worktree 平行實作流程；開關同時決定是否生成兩個 worktree 技能 |
| `audit` | `false` | 要求實作套用 sharp-edges audit 紀律 |
| `context` | — | 建立 artifact 時提供給 AI 的專案脈絡 |
| `rules` | — | 各 artifact 的撰寫規則 |

### `.speclink.yaml`

| 鍵 | 預設 | 意義 |
|---|---|---|
| `spec_dir` | `openspec` | spec store 目錄（專案根相對路徑） |
| `tools` | — | 要生成技能檔的 AI harness（內建名或描述子） |
| `locale`／`spec_locale`／`tdd`／`audit`／`worktree` | — | 不生效且不警告——政策一律讀 `openspec/config.yaml` |

### 環境變數

| 變數 | 值 |
|---|---|
| `SPECLINK_LOCALE` | 任意 locale 代碼 |
| `SPECLINK_SPEC_LOCALE` | 任意 locale 代碼，或 `auto` |
| `SPECLINK_TDD` | `true`／`false` |
| `SPECLINK_AUDIT` | `true`／`false` |
| `SPECLINK_WORKTREE` | `true`／`false` |
