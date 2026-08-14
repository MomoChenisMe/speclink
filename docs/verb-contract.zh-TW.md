# 動詞與旗標契約

> **誰需要讀這份**：要自己串接 Speclink 遠端 API、寫客戶端，或要確認某個動詞在遠端模式會怎麼表現的人。
> 只用桌面 app 或 CLI 的話，這份可以完全跳過——日常操作看[完整 SDD 工作流](workflow.zh-TW.md)就夠。

本文件回答動詞層的契約。它涵蓋兩塊：CLI 動詞在本機與 remote 兩模式的歸屬與輸出規則，以及正典 `verb-contract` spec 指定的端點、payload 與錯誤形狀。

端點那一塊目前涵蓋動詞補全（verb-parity）端點：validate、analyze、刪除變更、任務搬移、討論建立帶 slug、討論 discard、討論 link、討論 seal，以及變更開工標記。其餘動詞的契約仍以 canonical specs 為準：

- [正典動詞契約](../openspec/specs/verb-contract/spec.md)
- [Client Protocol spec](../openspec/specs/client-protocol/spec.md)

以下端點都位於 project base `/api/speclink/v1/projects/{key}` 之下。每個請求都要帶標準契約 headers：`Authorization: Bearer …`、`X-Speclink-Api-Version`，以及已選定時的 `X-Speclink-Repo`。所有成功回應都附 scope ETag header，值是 project revision。

## CLI 動詞的模式歸屬

每個頂層動詞歸屬四種模式形狀之一，分岔決策集中宣告於 dispatch 層，不散在各動詞函式裡：

| 形狀 | 動詞 | 語意 |
| --- | --- | --- |
| **ModeFree** | `init`、`update`、`link`、`unlink`、`auth`、`schemas`、`templates`、`feedback`、`schema`、`config`、`completion` | 不觸發 store 模式解析。不讀專案設定的動詞（`completion`、`config`）不受壞掉的 `.speclink.yaml` 影響。 |
| **Dual** | `list`、`show`、`validate`、`analyze`、`drift`、`archive`、`discard`、`artifact`、`language`、`status`、`instructions`、`new`、`workflow-config`、`task`、`in-progress`、`discuss`、`review`、`verify` | 本機模式作用於本機 store，remote 模式作用於 remote store，**不會**在 remote 模式靜默改作用於本機。缺任一臂構成建置失敗，而非執行期靜默回退。 |
| **FsOnly** | `demo` | remote 模式以非零 exit code 明確拒絕，且不發出任何 server 請求——離線環境同樣拒絕。 |
| **RemoteOnly** | `claim` | 本機模式以非零 exit code 明確拒絕，並於 stderr 說明需要 remote store。 |

模式判定是惰性的：只有宣告形狀需要時才解析模式，只有 remote 臂將執行時才建立連線。

## 兩模式的輸出同形

Dual 動詞的人眼輸出（stdout 文本，含 `--no-color`）在兩模式下逐位元一致，只有五項明文分歧：

1. `new change` 的 Path 行——本機印、remote 不印（server 端路徑對本機使用者無意義）。
2. `list` 的 worktree 標示——remote 恆缺席（worktree 是本機主 checkout 的觀察面）。
3. `status` 的 schema 覆寫旗標——remote 明確拒絕（server 的工作流設定決定 schema）。
4. `workflow-config` 的文件標籤——remote 以 `config.yaml` 為標籤。
5. `discuss promote` 的 Path 行與其後的提示行——本機印、remote 不印（兩行綁在一起去留）。

清單以外的任何輸出差異都是缺陷。模式差異只存在於資料取得與守門拒絕，不存在於輸出文本的組版。

`--json` 的欄位集合與 camelCase 命名是凍結契約：不改名、不移除既有欄位，工單原文不出現在任何 `--json` 輸出。

想知道每個動詞「什麼時候用、完成判準是什麼」不在本文——那是[完整 SDD 工作流](workflow.zh-TW.md)的責任。

## 錯誤封套

所有非 2xx 回應皆為 protocol 錯誤封套：

```json
{ "status": 409, "reason": "refused", "message": "…人類可讀的引擎凍結文字…" }
```

`reason` 為機器可判的註冊表值（`not_found`、`permission_denied`、`refused`、`invalid_argument`、`invalid_config`、`revision_conflict`、`unavailable`、`internal`）。

## GET /changes/{name}/validate

唯讀衍生查詢，**reader 與 editor 皆可用**。它經 Command gateway 執行與 fs 模式 `speclink validate` 相同的引擎運算：單 change、spec-driven schema、非 strict。它不寫入、不發事件，scope revision 也不前進。

回應 `200`：

```json
{ "change": "demo", "valid": false, "errors": ["…"], "warnings": ["…"] }
```

錯誤：change 不存在時 `404 not_found`。

**聚合規則**：端點固定單 change。CLI 的聚合語意——無參數、`--all`、`--changes`——由 **client 組合**：先 list，再逐 change 呼叫本端點。聚合輸出形狀與 fs 模式一致。任一 change invalid 時，CLI 以非零 exit code 結束。

## GET /changes/{name}/analyze

唯讀衍生查詢，**reader 與 editor 皆可用**。回傳引擎完整的 `AnalyzeReport`。不寫入、不發事件。

回應 `200`：

```json
{
  "changeId": "demo",
  "dimensions": [{ "dimension": "Coverage", "status": "Clean", "findingCount": 0 }],
  "findings": [{
    "id": "AMB-1", "dimension": "Ambiguity", "severity": "Suggestion",
    "location": "specs/auth/spec.md", "summary": "…", "recommendation": "…",
    "summaryMsg": { "key": "…", "params": { "scenario": "…" } },
    "recommendationMsg": { "key": "…", "params": {} }
  }],
  "artifactsAnalyzed": ["proposal.md"],
  "artifactsMissing": ["design.md"]
}
```

錯誤：change 不存在時 `404 not_found`。

## DELETE /changes/{name}?force={bool}

**editor 限定**，reader 收 `403 permission_denied`。它經 Command gateway 執行 discard 的全部語意：fail-closed metadata 檢查、started-work guard、來源討論 unlink、change 全部文件的原子刪除，以及 touched 記錄清理。commit 發布 `change-discarded` 事件，SSE 訂閱端收到 invalidate。

query 參數 `force` 預設 `false`。

- `force=false` 對帶開工痕跡的 change（`started_at` 已蓋或任一任務已勾）→ `409 refused`，message 為引擎的凍結 needs-force 文字。**在本端點上，`reason: "refused"` 即機器可判的 needs-force 訊號。** 零寫入。
- `force=true` 無視開工痕跡刪除。metadata 損壞時即使 `force=true` 也拒絕（`invalid_config`）。

回應 `200`：

```json
{ "change": "demo", "unlinkedDiscussions": [{ "slug": "auth-flow", "status": "concluded" }] }
```

**兩個入口的 force 語意不同。** CLI 直通使用者的 `--force` 旗標，與本地 discard 的 guard 行為 parity。桌面的 remote 刪除固定送 `force=true`，與本地桌面無 guard 直刪同模式——確認對話框在 UI 層。

## POST /changes/{name}/tasks/move

**editor 限定**，reader 收 `403 permission_denied`。它搬移一個 checkbox 任務並重算「數字.數字」編號前綴，結果與本地拖排逐位元一致——引擎只有這一份搬移實作。

請求：

```json
{ "from": 1, "to": 3, "before": null }
```

`from`／`to` 為 1-based checkbox ordinal（與任務勾選／取消勾選同一定址域）。`before` 為可省略的明確側別：`true` 插於錨任務行之前（跨過群組標題即成為錨所屬群組的組首）、`false` 插於錨任務行之後、省略／`null` 依方向推斷（向上插前、向下插後）。

回應 `200`：

```json
{ "change": "demo", "description": "2.2 甲" }
```

`description` 為搬移**後**的任務描述（前綴已重編號）。commit 發布 `task-moved` 事件 → SSE invalidate。

錯誤：

- `from`／`to` 越界時回 `409 refused`，message 為 `task index out of range (1..=N)`。他人同時編輯造成的過期索引是可預期的競態，SSE invalidate 會矯正 client 視圖。零寫入。
- 該 change 無 `tasks.md` 時 `404 not_found`。

## POST /discussions——選填 slug 覆寫

建立討論請求接受選填 `slug` 欄位（camelCase、缺席即省略——舊 client 的 body 逐位元不變）：

```json
{ "topic": "看板搜尋列", "slug": "board-search-bar" }
```

檢查規則只住在引擎裡：ASCII kebab-case，小寫字母與數字，以單一連字號分隔。非法值回 `400 invalid_argument`，message 為引擎凍結文本，零寫入。未帶 `slug` 時，server 照舊由 topic 推導。

回應 `200`（形狀不變；帶覆寫時 `slug` 回覆寫值）：

```json
{ "slug": "board-search-bar", "topic": "看板搜尋列", "path": "discussions/board-search-bar.md" }
```

## DELETE /discussions/{slug}?force={bool}

**僅 editor**，reader 收 `403 permission_denied`。它直通引擎的討論 discard。0 輪的記錄直接刪除。已有輪時引擎拒絕，需要 `force=true`：回 `409 refused`，message 為凍結的 needs-force 文本，記錄逐位元保留。commit 發布 `discussion-discarded`。

query 參數 `force` 預設 `false`。回應 `200`：

```json
{ "slug": "board-search-bar" }
```

錯誤：該 slug 無 live 討論時 `404 not_found`；封存記錄拒絕刪除（`409 refused`——封存記錄留存、不 discard）。

## POST /discussions/{slug}/link

鑄變更側 `from_discussion` 鏈（引擎 link 語意：逗號累加、同一配對冪等）。請求／回應：

```json
{ "change": "add-auth" }
```

```json
{ "slug": "auth-scope", "change": "add-auth" }
```

commit 發布 `discussion-linked`。錯誤：討論或 change 不存在時 `404 not_found`（引擎凍結 message 指名缺席主體）。

## POST /discussions/{slug}/seal

內容落地後，它把討論標記為已轉出：status 變 `promoted`，`promoted_to` 累加變更名，並清除該 change 對本 slug 的 re-ingest 旗標。請求與回應形狀同 link。

守衛：change 的 `from_discussion` 鏈必須先含該 slug，否則回 `409 refused`，message 為引擎的先跑 link 文本。commit 發布 `discussion-sealed`。討論或 change 缺席時回 `404 not_found`。

## POST /changes/{name}/in-progress

經 Command gateway 的靜默生命週期蓋章。對存在且未開工的 change 首次呼叫時，它把 `started_at` 與 `started_by` 寫進 change meta，發布 `change-marked-in-progress`，並讓 scope revision 前進。`started_by` 是呼叫者的認證身分，與 `created_*` 同一歸屬機制。

重複呼叫或未知 change 名稱，維持引擎凍結的靜默成功：`200`、零寫入、零事件、revision 不前進。兩種情形的 body 都是空物件：

```json
{}
```

## 變更清單的 `startedAt` 欄位

`GET /changes` 清單項攜帶選填 `startedAt`（camelCase），值來自 change meta 的 `started_at`；未開工的 change 省略該欄位。消費端以其做欄位推導（「已開工即進行中」，完成數 fallback 保留以涵蓋繞過工具的寫入路徑）：

```json
{ "name": "demo", "status": "in-progress", "completedTasks": 0, "totalTasks": 15, "startedAt": "2026-07-30" }
```

## `speclink list --json`——僅本機的 `worktree` 欄位

`list --json` 的變更項目可帶一個選填的 `worktree` 物件——那是「這個變更正在某個 linked git worktree 裡實作」的本機觀察面：

```json
{ "name": "add-dark-mode", "completedTasks": 3, "totalTasks": 5, "worktree": { "path": "/repos/speclink.worktrees/add-dark-mode", "branch": "speclink/add-dark-mode" } }
```

- `path`——字串，worktree 目錄的絕對路徑。`branch`——字串，完整分支名（`speclink/<change>`）。
- **只有**在 fs 模式、由**主 checkout**（workspace 根目錄的 `.git` 是目錄）、`worktree` 工作流政策開啟、且該變更的對應關係成立時才出現。其餘情況一律缺席，而且缺席時**不序列化**該欄位。
- remote 模式的 `list` 項目**恆不帶**此欄位：它描述的是呼叫端的本機 checkout，server 對此一無所知。因此在沒有 worktree 的情況下，fs 與 remote 的 payload 逐欄位完全相同。
- 欄位存在時，該項目的 `completedTasks`／`totalTasks`／`status`／`metaError` 取自**那個 worktree 內**的變更副本，不是主 checkout 的。欄位名稱與型別不變。

## GET /changes/{name}——show 組合的 meta 欄位

單 change 讀取另攜帶三個選填欄位，餵 CLI remote `show` 的讀取組合：`created`（僅 meta 的 schema+created 成對時出現——引擎的成對回報規則）、`fromDiscussions`、`deltaCapabilities`（空清單即省略）。舊 server 不送、舊 client 忽略。

```json
{ "changeName": "demo", "schemaName": "spec-driven", "…": "…", "created": "2026-07-29", "fromDiscussions": ["auth-scope"], "deltaCapabilities": ["auth"] }
```

## capability 宣告

`GET /binding` handshake 依 membership role 宣告這些動詞：

```json
"capabilities": { "validate": true, "analyze": true, "deleteChange": true, "moveTask": true, … }
```

`validate`／`analyze` 對全 role 為 `true`；`deleteChange`／`moveTask` 僅 editor 為 `true`。capability 為 `false` 時 client 停用對應 affordance；server 的 request-time role 檢查仍是最終權限防線。
