# 設定說明

> **文件狀態：**本文描述目前已實作的本地 workspace 設定介面。遠端模式的 workflow policy 以具版本的 Store 為正典，本機 override 不得靜默覆寫團隊政策；目標設計見[平台架構藍圖](platform-architecture.zh-TW.md#48-workflow-policy-的歸屬)。

Speclink 的設定分散在兩個檔案與一個目錄，各自有明確的歸屬規則：

| 位置 | 承載 | 跟著誰走 |
|---|---|---|
| `openspec/config.yaml` | 工作流政策：`locale`、`spec_locale`、`tdd`、`audit`，以及 `schema`、`context`、`rules` | **spec store**（spec 文件所在之處） |
| `.speclink.yaml` | workspace 綁定：`tools`（為哪些 AI harness 生成檔案）、`spec_dir`（spec store 在哪裡） | **repo checkout** |
| `.speclink/` | 主機工作資料：touched 記錄、archive 快照、生成工具足跡 | **本機**（gitignored） |

「這個設定該放哪」的判定規則：

- **政策跟 store。** 會改變工作流產出的設定（artifact 語言、spec 語言、TDD 紀律、audit 紀律）放在 `openspec/config.yaml`。無論本地或透過遠端 store 讀 spec 的人，看到的都是同一份真相。
- **綁定跟 repo。** `.speclink.yaml` 只描述「這個 checkout」如何連上 store、接了哪些 AI 工具，不承載政策。
- **個人差異跟環境變數。** `SPECLINK_*` 環境變數在單一 shell 或單一 CI job 內覆寫一切，不動任何檔案。

## 解析順序

有效政策值經四層解析，先命中者勝：

| 優先序 | 層 | 說明 |
|---|---|---|
| 1（最高） | `SPECLINK_LOCALE`／`SPECLINK_SPEC_LOCALE`／`SPECLINK_TDD`／`SPECLINK_AUDIT` | 布林變數僅接受 `true`／`false`（不分大小寫）；其他值——`yes`、`1`、空字串——視為**未設定**，落到下一層。 |
| 2 | `.speclink.yaml` 的舊政策鍵 | Deprecated 相容層（見下節）。鍵「存在」即勝出，即使值是 `false`。 |
| 3 | `openspec/config.yaml` | 正典歸屬。 |
| 4（最低） | 內建預設 | `locale` 未設定＝English、`tdd`＝false、`audit`＝false。 |

## Deprecated：`.speclink.yaml` 中的政策鍵

較舊的專案會在 `.speclink.yaml` 留有 `locale`、`spec_locale`、`tdd`、`audit`。它們仍然有效——值依舊勝過 `openspec/config.yaml`，不會靜默改變行為——但每次執行指令都會向 stderr 輸出一行：

```
speclink: warning: deprecated policy keys in .speclink.yaml: tdd, audit (move them to openspec/config.yaml)
```

這一行有固定前綴（`speclink: warning:`）、每次執行恰出現一次、且不影響 stdout——`--json` 輸出完全不變。

### 自舊佈局遷移

1. 把用到的政策鍵從 `.speclink.yaml` 搬到 `openspec/config.yaml`，值保持不變。
2. 從 `.speclink.yaml` 刪除它們，只留 `tools` 與（若有自訂的）`spec_dir`。
3. 執行任一指令——警告消失，有效值不變（正典層接手了原本相容層提供的值）。

遷移前 → 後：

```yaml
# .speclink.yaml（前）             # .speclink.yaml（後）
locale: tw                         tools:
tdd: true                            - claude
tools:
  - claude                         # openspec/config.yaml（後）
                                   schema: spec-driven
# openspec/config.yaml（前）       locale: tw
schema: spec-driven                tdd: true
```

## 用 `workflow-config` 動詞管理 `openspec/config.yaml`

`speclink workflow-config` 管的是**工作流政策檔**；`speclink config` 管的是與專案無關的全域 KV 存放，兩者互不相干。

| 子指令 | 作用 |
|---|---|
| `show [--json]` | 顯示政策四欄、`context`（行數）與 `rules`（各節條數）。顯示**正典值**——不套用環境變數或舊鍵覆寫（有效值的四層解析屬 `speclink instructions` 的職責）。`--json` payload 欄位為 camelCase：`locale`、`specLocale`、`tdd`、`audit`、`context`、`rules`；未設定的欄位為 `null`，未設定的布林為 `false`。 |
| `set <key> <value>` | 寫入 `locale`、`spec_locale`、`tdd`、`audit` 之一。其他鍵以非 0 exit code 拒絕；`tdd`／`audit` 僅接受 `true`／`false`；`locale` 僅接受代碼 `tw`／`ja`／`en`、`spec_locale` 僅接受 `tw`／`ja`／`en`／`auto`（大小寫敏感）——顯示名稱（如「繁體中文」）會被拒絕並列出合法代碼。設為 `false`（或 locale 設為空字串）＝**移除該鍵**，維持「未設定＝預設」語意。 |
| `context --stdin` | 以 stdin 全文設定 `context`；內容僅空白時移除該鍵。 |
| `rules <artifact> --stdin` | 整節代換該 artifact 的規則（一行一條、空行忽略）；stdin 為空時移除該節。`artifact` 限目前 schema 的 artifact id，未知 id 以非 0 exit code 拒絕。 |

三個寫入子指令都支援 `--dry-run`：印出 unified diff 至 stdout、不寫入任何檔案、exit code 0。預覽與實寫走完全相同的改寫路徑，所以 diff 就是會落檔的內容。

```bash
speclink workflow-config set tdd true --dry-run   # 先看
speclink workflow-config set tdd true             # 再落檔
cat CONTEXT.md | speclink workflow-config context --stdin
```

**fs 與 remote 模式。** 模式由既有 binding 判定：fs 模式直接讀寫 `openspec/config.yaml`；remote 模式經連線讀取 server 端 config 文件與其版本、套用同一改寫、寫回時附帶該版本。版本識別不出現在指令介面——被他人並行改寫時以非 0 exit code 提示重新執行，絕不覆蓋他人寫入。離線或認證失效同樣以非 0 exit code 失敗，不暫存也不排隊。

**既知取捨：模板註解會喪失。** 寫入是 read-modify-write（讀進整份文件、改目標鍵、整份寫回），其他鍵值一律保留，但原檔的模板註解在重寫後不再存在——與桌面設定頁同一取捨。首次使用先跑 `--dry-run` 看 diff 再決定。文件無法解析時一律 fail-closed：讀寫都以非 0 exit code 拒絕（重寫壞檔會毀掉既有內容）。

內建技能 `speclink-config` 就建立在這個動詞之上——它從 codebase 的固定來源整理 `context` 與 `rules`，一律先產 diff 交使用者裁決再寫入。

## 自訂工具描述子

`tools` 清單除內建名（`claude`、`codex`）外，也接受描述子物件，用於任何其他 AI harness：

```yaml
tools:
  - claude
  - name: wad-harness
    skills_dir: .wad/skills
    instructions_file: WAD.md
    invocation: tool-call
```

| 欄位 | 必填 | 規則 |
|---|---|---|
| `name` | 是 | kebab-case，2–50 字元、限 `[a-z0-9-]`；不得與內建工具名衝突 |
| `skills_dir` | 是 | 專案根相對路徑；不得逸出專案根 |
| `instructions_file` | 是 | 專案根相對路徑；不得逸出專案根 |
| `invocation` | 否 | `cli`（預設）或 `tool-call`——決定生成文字如何指示 harness 執行 speclink 動詞：「執行 `speclink <動詞>`」vs「呼叫 speclink 工具（參數為 argv 陣列）」 |

驗證失敗（名稱衝突、大小寫違規、路徑逸出、invocation 非法值）時指令以非 0 exit code 結束，並輸出指明欄位的單行錯誤訊息。

描述子與內建工具共享完整生命週期：

- **生成**——`speclink init`／`speclink update` 在 `skills_dir` 下寫入 `speclink-*/SKILL.md` 技能檔，並在 `instructions_file` upsert `SPECLINK` marker 區塊。
- **同步**——`speclink update` 對仍在清單上的描述子全部重新生成。
- **清理**——把描述子從 `tools` 移除後，下一次 `speclink update` 會刪除其 `speclink-*` 技能目錄（因而變空的目錄一併移除）、自 `instructions_file` 剝除 marker 區塊，若剝除後檔案全空則整檔刪除。

描述子生成的內容採用**中性渲染**：無 `/speclink-` slash 前綴、無 plan mode 參照、動詞措辭由 `invocation` 決定。內建 claude 與 codex 的輸出完全不受影響。

## 參考：全部鍵值

### `openspec/config.yaml`

| 鍵 | 預設 | 意義 |
|---|---|---|
| `schema` | `spec-driven` | 新 change 使用的工作流 schema |
| `locale` | English | AI 生成 artifact 的語言（`tw`、`ja`…） |
| `spec_locale` | English | spec 檔語言；`auto` 跟隨 `locale` |
| `tdd` | `false` | 要求實作遵循 Red-Green-Refactor 紀律 |
| `audit` | `false` | 要求實作套用 sharp-edges audit 紀律 |
| `context` | — | 建立 artifact 時提供給 AI 的專案脈絡 |
| `rules` | — | 各 artifact 的撰寫規則 |

### `.speclink.yaml`

| 鍵 | 預設 | 意義 |
|---|---|---|
| `spec_dir` | `openspec` | spec store 目錄（專案根相對路徑） |
| `tools` | — | 要生成指令檔的 AI harness（內建名或描述子） |
| `locale`／`spec_locale`／`tdd`／`audit` | — | **Deprecated**——仍有效，但每次指令都會警告 |

### 環境變數

| 變數 | 值 |
|---|---|
| `SPECLINK_LOCALE` | 任意 locale 代碼 |
| `SPECLINK_SPEC_LOCALE` | 任意 locale 代碼，或 `auto` |
| `SPECLINK_TDD` | `true`／`false` |
| `SPECLINK_AUDIT` | `true`／`false` |
