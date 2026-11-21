## Why

內嵌宿主以 `speclink` CLI 0.14.0 與 Speclink Desktop 0.14.0 實測後回報兩項不足，查證後都成立。

- 專案有兩個以上 repo 時，`speclink link`（已登入）、`speclink auth status`、`speclink init --store remote` 被 409 拒絕。這三個動詞問專案底下的 `/whoami` 而不帶 repo，入口卻要求先選 repo 才回答。這個路由的用途就是列出專案有哪些 repo，所以「列 repo 要先選 repo」是循環；錯誤訊息還要求 CLI 使用者指定他不能設的標頭。
- 看板開著時帳號被停用或被移出專案，Speclink Desktop 的背景輪詢每次都收到 403，畫面卻沒有任何訊息，繼續顯示舊內容。使用者要等到自己送出某個請求才知道原因。

目標使用者：透過 AI agent 開發的開發者，以及把引擎內嵌進自己系統的宿主開發者。使用情境：把 checkout 連到遠端專案（`link`、`init`、`auth status`），以及用 Speclink Desktop 看遠端專案的看板。宿主會用修正後的版本重新實測，所以兩項放在同一個變更。

## What Changes

**入口（`speclink-host`，示範用 Server 的專案路由也經過它）**

- 專案底下的 `/whoami` 不再選 repo：專案有多個 repo 而請求沒有 `X-Speclink-Repo` 時回 200，`repos` 是整個專案的清單。請求帶了標頭也不拿它來選 repo。身分、專案是否在這個人的範圍清單裡、API 版本標頭三項檢查照舊。
- 這個回應不再帶 ETag（不選 repo 就沒有範圍的版本號）。
- 其他專案路由不變：多個 repo 而沒有指名時仍回 409。409 的訊息改成同時寫出 `X-Speclink-Repo` 標頭與 `.speclink.yaml` 的 `remote.repo`，候選清單照舊。

**CLI（`speclink-cli`、`speclink-remote`）**

- `speclink link <url>`：已有憑證、專案有多個 repo、沒帶 `--repo` 時，以非 0 exit code 結束，stderr 請使用者加 `--repo` 並列出代號，不寫入 remote 區段。旗標、stdin 都沒有新增。
- `speclink init --store remote --url <url>`：同樣的情況下，初始化照既有順序先完成，接著以非 0 exit code 結束，stderr 列出代號並指引執行 `speclink link <url> --repo <代號>`。
- `speclink auth status`：第二行的憑證來源文字不再指名登入方式（鑰匙圈不記是瀏覽器登入還是裝置登入）。
- 伺服器以錯誤信封回 `unavailable`（500 以上）時，錯誤訊息顯示信封的原因，不再請使用者檢查網址。連不上、或 500 以上而沒有可讀的信封時，訊息不變。

**Speclink Desktop（`apps/desktop`、`apps/desktop/src-tauri`）**

- 新增「存取被拒」狀態，跟著分頁（連線＋專案＋repo）。背景輪詢（`/sync-state`）收到 403 時進入，下一次輪詢成功時離開並重新載入。
- 狀態期間：已載入的內容照常可以看，寫入操作全部停用，Rust 端的寫入命令立即拒絕（和離線相同）。
- 三個地方一起顯示，都用錯誤的紅色與警示三角形：看板上方的提示列（顯示伺服器回的原因）、專案圖示列的狀態點、系統匣（面板與原生選單）。
- 伺服器有回應但不能服務（500 以上）也計入離線的連續失敗次數。離線提示列在伺服器有回原因時顯示它的原因。
- 伺服器以低於 500 的狀態碼回應（例如 403）也算連線恢復：離線的連線回到 online。沒有這一條的話，分頁被拒期間伺服器斷線又恢復，畫面會一直停在離線（品質關卡第 1 輪補上）。

**示範用 Server（`speclink-server`）**

- 事件串流在送出心跳的間隔重新檢查帳號仍可用、仍是專案成員；不成立就關閉串流。Speclink Desktop 重新訂閱被拒後，就經輪詢進入「存取被拒」。

**文件**

- SDK 文件：`auth-methods` 回應的 `browser` 一定帶 `scopes`；自己提供串流的宿主要在使用者失去權限時關閉串流。
- 遠端專案的使用者文件與 `docs/product-status`：多個 repo 的專案怎麼連、Speclink Desktop 的新提示；並寫明宿主要升級 `@speclink/engine`、CLI、Speclink Desktop 三樣。

**相容性影響**

- 專案底下的 `/whoami`：多個 repo 而沒有標頭時由 409 變成 200；標頭指名不存在的 repo 時由 404 變成 200；回應少了 ETag 標頭。本文的形狀不變。CLI 與 Speclink Desktop 都不讀這個 ETag。
- 409 的訊息文字改變（CLI 原樣顯示它）。比對這句文字的腳本要更新；狀態碼與 `reason`（`refused`）不變。
- `speclink link`：多個 repo 而沒帶 `--repo` 的情況，變更前就會失敗（409），變更後仍以非 0 結束，只是訊息改成可照做的指引。專案只有一個 repo、或有帶 `--repo` 的用法，輸出不變。
- `speclink auth status` 的人眼輸出第二行文字改變；`--json` 的 `credentialSource` 值不變。
- 伺服器回 `unavailable` 信封時的錯誤訊息改變；exit code 不變。
- 這個變更沒有新增設定欄位，也不影響生成的技能。

**三種形態下的結果**

- 本機專案：不適用。`auth status` 需要遠端連線；`link` 與 `init --store remote` 在執行前還沒有遠端連線，行為由下面兩種遠端的形態決定。Speclink Desktop 的新狀態只出現在遠端專案的分頁。
- 連到示範用 Server 的 checkout：可用。三個 CLI 動詞在多個 repo 的專案正常；Speclink Desktop 經串流關閉或輪詢偵測到存取被拒。
- 內嵌宿主：可用，條件是宿主升級 `@speclink/engine`。宿主只用輪詢時，Speclink Desktop 最長 30 秒內顯示提示；宿主自己提供串流時，要由宿主在使用者失去權限時關閉串流，這一點寫進 SDK 文件。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `host-kit`: 專案底下的身分查詢不選 repo、不帶 ETag；多個 repo 而沒有指名時的 409 訊息寫出兩種指定方式。
- `reference-server`: 「所有路由先裁決 repo」的規則加上專案底下身分查詢的例外。
- `remote-connection`: `link` 與 `init --store remote` 在多個 repo 而沒帶 `--repo` 時的行為；伺服器回 `unavailable` 時顯示它的原因。
- `remote-auth`: `auth status` 的憑證來源文字不指名登入方式。
- `remote-resilience`: 500 以上的回應計入離線、離線提示列顯示伺服器的原因；新增跟著分頁的「存取被拒」狀態。
- `server-event-stream`: 使用者失去權限時關閉串流。
- `tray-status-menu`: 系統匣的面板與原生選單標示「存取被拒」。

## Impact

- Affected specs: `host-kit`、`reference-server`、`remote-connection`、`remote-auth`、`remote-resilience`、`server-event-stream`、`tray-status-menu`
- 掃描過但不需要改的規格：`desktop-app`「介面狀態語意色分層」已經把遠端專案的 access-denied 列為紅色的錯誤，新提示照它上色；`client-protocol` 的 payload 形狀不變。
- Affected code:
  - Modified:
    - `crates/host/speclink-host/src/http/mod.rs`
    - `crates/host/speclink-host/src/http/routes.rs`
    - `crates/host/speclink-host/src/http/router.rs`
    - `crates/host/speclink-host/tests/http_door.rs`
    - `crates/host/speclink-server/src/api/routes.rs`
    - `crates/host/speclink-server/Cargo.toml`
    - `crates/host/speclink-server/tests/it/api/sse_events.rs`
    - `crates/host/speclink-server/tests/it/api/read_api.rs`
    - `crates/adapters/speclink-cli/src/verbs/connection.rs`
    - `crates/adapters/speclink-cli/src/remote_base.rs`
    - `crates/adapters/speclink-cli/tests/it/remote_connect.rs`
    - `crates/protocol/speclink-remote/src/lib.rs`
    - `crates/protocol/speclink-remote/src/auth.rs`
    - `crates/protocol/speclink-remote/tests/it/client_errors.rs`
    - `crates/protocol/speclink-remote/tests/it/typed_client.rs`
    - `apps/desktop/src-tauri/src/event_manager.rs`
    - `apps/desktop/src-tauri/src/lib.rs`
    - `apps/desktop/src-tauri/src/remote.rs`
    - `apps/desktop/src-tauri/tests/it/event_manager.rs`
    - `apps/desktop/src-tauri/tests/it/remote_runtime.rs`
    - `apps/desktop/src/session.ts`
    - `apps/desktop/src/store.ts`
    - `apps/desktop/src/App.tsx`
    - `apps/desktop/src/tray.ts`
    - `apps/desktop/src/components/ProjectRail.tsx`
    - `apps/desktop/src/panel/TrayPanel.tsx`
    - `apps/desktop/src/i18n/messages.ts`
    - `apps/desktop/src/__tests__/remoteResilience.test.tsx`
    - `apps/desktop/src/__tests__/remoteDataSource.test.ts`
    - `apps/desktop/src/__tests__/projectRail.test.tsx`
    - `apps/desktop/src/__tests__/tray.test.ts`
    - `apps/desktop/src/__tests__/trayPanel.test.tsx`
    - `crates/adapters/speclink-node/index.d.ts`
    - `docs/sdk-node.md`
    - `docs/sdk-node.zh-TW.md`
    - `docs/verb-contract.md`
    - `docs/verb-contract.zh-TW.md`
    - `docs/remote-getting-started.md`
    - `docs/remote-getting-started.zh-TW.md`
    - `docs/product-status.md`
    - `docs/product-status.zh-TW.md`
  - New: （無）
  - Removed: （無）
