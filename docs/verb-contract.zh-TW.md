# 動詞契約——端點參考

本文件是正典 `verb-contract` spec 指定的動詞契約端點、payload 與錯誤形狀參考。目前涵蓋動詞補全（verb-parity）端點（validate／analyze／刪除變更／任務搬移／討論建立帶 slug／討論 discard／討論 link／討論 seal／變更開工標記）；其餘動詞的契約仍以 canonical specs 為準：

- [正典動詞契約](../openspec/specs/verb-contract/spec.md)
- [Client Protocol spec](../openspec/specs/client-protocol/spec.md)

以下端點皆位於 project base `/api/speclink/v1/projects/{key}` 之下，需帶標準契約 headers（`Authorization: Bearer …`、`X-Speclink-Api-Version`、已選定時的 `X-Speclink-Repo`）。所有成功回應皆附 scope ETag header（project revision）。

## 錯誤封套

所有非 2xx 回應皆為 protocol 錯誤封套：

```json
{ "status": 409, "reason": "refused", "message": "…人類可讀的引擎凍結文字…" }
```

`reason` 為機器可判的註冊表值（`not_found`、`permission_denied`、`refused`、`invalid_argument`、`invalid_config`、`revision_conflict`、`unavailable`、`internal`）。

## GET /changes/{name}/validate

唯讀衍生查詢，**reader 與 editor 皆可用**。經 Command gateway 執行與 fs 模式 `speclink validate` 相同的引擎運算（單 change、spec-driven schema、非 strict）。不寫入、不發事件、scope revision 不前進。

回應 `200`：

```json
{ "change": "demo", "valid": false, "errors": ["…"], "warnings": ["…"] }
```

錯誤：change 不存在時 `404 not_found`。

**聚合規則**：端點固定單 change。CLI 的聚合語意（無參數、`--all`、`--changes`）由 **client 組合**：先 list 再逐 change 呼叫本端點；聚合輸出形狀與 fs 模式一致，任一 change invalid 時 CLI 以非零 exit code 結束。

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

**editor 限定**（reader 收 `403 permission_denied`）。經 Command gateway 執行 discard 全語意：fail-closed metadata 檢查、started-work guard、來源討論 unlink、change 全部文件的原子刪除、touched 記錄清理。commit 發布 `change-discarded` 事件，SSE 訂閱端收到 invalidate。

query 參數 `force` 預設 `false`。

- `force=false` 對帶開工痕跡的 change（`started_at` 已蓋或任一任務已勾）→ `409 refused`，message 為引擎的凍結 needs-force 文字。**在本端點上，`reason: "refused"` 即機器可判的 needs-force 訊號。** 零寫入。
- `force=true` 無視開工痕跡刪除。metadata 損壞時即使 `force=true` 也拒絕（`invalid_config`）。

回應 `200`：

```json
{ "change": "demo", "unlinkedDiscussions": [{ "slug": "auth-flow", "status": "concluded" }] }
```

**兩入口的 force 語意**：CLI 直通使用者的 `--force` 旗標（與本地 discard 的 guard 行為 parity）；桌面 remote 刪除固定送 `force=true`（與本地桌面無 guard 直刪同模式，確認對話框在 UI 層）。

## POST /changes/{name}/tasks/move

**editor 限定**（reader 收 `403 permission_denied`）。搬移一個 checkbox 任務並重算「數字.數字」編號前綴，與本地拖排逐位元一致（引擎唯一的搬移實作）。

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

- `from`／`to` 越界時 `409 refused`，message 為 `task index out of range (1..=N)`（他人同時編輯下的過期索引是可預期的競態；SSE invalidate 會矯正 client 視圖）。零寫入。
- 該 change 無 `tasks.md` 時 `404 not_found`。

## POST /discussions——選填 slug 覆寫

建立討論請求接受選填 `slug` 欄位（camelCase、缺席即省略——舊 client 的 body 逐位元不變）：

```json
{ "topic": "看板搜尋列", "slug": "board-search-bar" }
```

驗證只住在引擎（ASCII kebab-case：小寫字母／數字、單一連字號分隔）。非法值 → `400 invalid_argument`，message 為引擎凍結文本；零寫入。未帶 `slug` 時 server 照舊由 topic 推導。

回應 `200`（形狀不變；帶覆寫時 `slug` 回覆寫值）：

```json
{ "slug": "board-search-bar", "topic": "看板搜尋列", "path": "discussions/board-search-bar.md" }
```

## DELETE /discussions/{slug}?force={bool}

**僅 editor**（reader 收 `403 permission_denied`）。引擎討論 discard 直通：0 輪記錄直接刪除；有輪時引擎拒絕（需 `force=true`）——`409 refused`、message 為凍結的 needs-force 文本、記錄逐位元保留。commit 發布 `discussion-discarded`。

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

內容落地後把討論標記已轉出（status `promoted`、`promoted_to` 累加變更名；並清除該 change 對本 slug 的 re-ingest 旗標）。請求／回應形狀同 link。守衛：change 的 `from_discussion` 鏈須先含該 slug——否則 `409 refused`，message 為引擎的先跑 link 文本。commit 發布 `discussion-sealed`。錯誤：討論或 change 缺席 `404 not_found`。

## POST /changes/{name}/in-progress

經 Command gateway 的靜默生命週期蓋章。對存在且未開工的 change 首次呼叫：`started_at` 與 `started_by`（呼叫者認證身分——與 `created_*` 同一歸屬機制）寫進 change meta、發布 `change-marked-in-progress`、scope revision 前進。重複呼叫或未知 change 名稱維持引擎凍結的靜默成功：`200`、零寫入、零事件、revision 不前進。兩種情形 body 皆為空物件：

```json
{}
```

## 變更清單的 `startedAt` 欄位

`GET /changes` 清單項攜帶選填 `startedAt`（camelCase），值來自 change meta 的 `started_at`；未開工的 change 省略該欄位。消費端以其做欄位推導（「已開工即進行中」，完成數 fallback 保留以涵蓋繞過工具的寫入路徑）：

```json
{ "name": "demo", "status": "in-progress", "completedTasks": 0, "totalTasks": 15, "startedAt": "2026-07-30" }
```

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
