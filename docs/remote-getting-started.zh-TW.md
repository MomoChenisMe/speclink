# Remote Server、Desktop 與 CLI 入門教學

**繁體中文** · [English](remote-getting-started.md)

本教學使用 repo root 的本地開發編排，從全新資料開始建立一個可由 Desktop 與 CLI 共用的 Remote Workspace。部署到正式環境前，另請閱讀 [Server 部署](server-deployment.zh-TW.md)、[Store drivers](server-store-drivers.zh-TW.md)與[備份／還原](server-backup.zh-TW.md)。

目前支援的是單節點 Server、Remote Desktop Workspace 與 Remote CLI。MCP、Copilot Tools、SSO 與 Cluster mode 不在本教學範圍；最新交付狀態以[產品能力狀態](product-status.zh-TW.md)為準。

## 1. Before you begin / 開始前

你需要：

- stable Rust toolchain、Node.js 與 npm。
- macOS、Windows 或 Linux 上可執行 Tauri Desktop 的開發環境。
- 兩個終端：一個長時間執行 Server＋Desktop，另一個測 CLI。

以下範例固定使用：

| 名稱 | 範例值 | 用途 |
| --- | --- | --- |
| Server base URL | `http://localhost:8080` | 瀏覽器登入、帳號與管理頁 |
| Project key | `demo` | Server registry 中的 Project |
| Repo key | `backend` | `demo` 下的 Repo |
| project-scoped URL | `http://localhost:8080/api/speclink/v1/projects/demo` | CLI／Client Protocol 連線 |

這三種 URL 不可混用。`/account`、`/admin` 等瀏覽器頁面接在 Server base URL；project-scoped URL 專供 Remote CLI／Desktop client 綁定 Project。

若目前 repo 有未提交工作，請保留它；Remote CLI smoke test 要在另一個測試資料夾執行，不要在產品 repo 根目錄執行 `speclink link`。

## 2. Start a clean development server / 啟動全新開發 Server

在 Speclink repo root 清除既有本地開發資料：

```bash
npm run dev:reset
```

這只刪除 `.dev/`，不會刪除 `.env`。若要驗證完全預設的 SQLite 設定，請先確認 `.env` 沒有覆寫 `SPECLINK_*` 值。

接著啟動：

```bash
npm run dev
```

此命令會產生 `.dev/config.yaml`、建置目前 checkout 的 `speclink-cli` 與 Desktop 前端，成功後才同時啟動 `speclink-server` 與 Tauri Desktop。CLI 建置失敗時它會以非零狀態結束，且不會留下任何長時間執行的 process——這保證第 7 節用來驗證的 CLI 與 Server／Desktop 來自同一份原始碼。終端會印出只供首次設定使用的網址：

```text
http://localhost:8080/setup?token=...
```

保持此終端執行。任一 child process 結束時，編排器會收束另一個 process；正常停止可按 `Ctrl+C`。

## 3. Complete first-run setup / 完成首次設定

在瀏覽器開啟終端印出的 `/setup?token=...`，依畫面建立：

1. 第一位 Admin：email、顯示名稱與密碼。
2. 第一個 Project：本教學使用 key `demo`。
3. 第一個 Repo：本教學使用 key `backend`。

完成頁會顯示 public URL、Project key 與 Repo key。依範例組成的 project-scoped URL 是：

```text
http://localhost:8080/api/speclink/v1/projects/demo
```

一般重啟後 `/setup` 會關閉，setup token 不會重印；這代表 identity 與 Store 已持久化，不是啟動失敗。

## 4. Grant Project membership / 授予 Project membership

建立 Project／Repo registry 不等於授予帳號存取權。Server Admin 是 installation 管理權；Project membership 是 Project 資料權限。即使是第一位 Admin，也不會繞過 membership 檢查。

1. 開啟 [http://localhost:8080/admin/users](http://localhost:8080/admin/users)。
2. 若被導向登入頁，使用 `/setup` 建立的 Admin email 與密碼登入。
3. 找到 Desktop 實際要登入的帳號。
4. 在 membership 表單選 `demo`。
5. 選擇角色：
   - `editor`：可讀寫，適合本教學 smoke test。
   - `reader`：可讀，寫入操作會停用或被 Server 拒絕。
6. 按「加入／更新」。

若要測一般使用者，請由 `/admin/users` 建立 invitation，指派 Project membership，讓受邀者開啟一次性邀請連結並設定密碼。不要共用 Admin 的 PAT 或密碼。

## 5. Create a PAT safely / 安全建立 PAT

PAT（Personal Access Token，個人存取權杖）是 CLI 與 Desktop fallback 的憑證。先開啟：

[http://localhost:8080/account](http://localhost:8080/account)

登入後，在 Personal Access Tokens 表單填名稱（例如 `local-cli`），到期日可留空，再按「建立 PAT」。PAT 明文只顯示一次，請立即複製到安全位置。

`/account` 是單頁應用（SPA）頁面：在其 Personal Access Tokens 表單建立 PAT，SPA 會提交至 browser API **POST `/api/speclink/v1/web/account/tokens`**，明文只顯示一次。`/account/tokens` 本身不是可瀏覽頁面——若直接以 GET 開啟會得到 JSON 404。

不要把 PAT 放進：

- URL 或 shell argument。
- `.speclink.yaml`、repo、文件或 log。
- Desktop localStorage。

CLI 可用互動式 `speclink auth login` 從 stdin 讀取；Desktop 會把 credential 放在 OS Keychain。

## 6. Open a Remote Desktop Workspace / 開啟 Remote Desktop Workspace

在 Desktop：

1. 選「新增 Workspace」。
2. 選「Speclink Server」。
3. 輸入 Server base URL：`http://localhost:8080`，不要輸入 project-scoped URL。
4. 優先選 Device Login：
   - Desktop 會開啟瀏覽器 `/activate`。
   - 若尚未登入，先登入帳號。
   - 確認畫面中的 user code 與 Desktop 顯示一致，再核准。
5. 若 Device Login 無法使用，可選 PAT fallback 並貼上 `/account` 剛建立的 PAT。
6. 選擇 `demo`／`backend`。

若清單顯示「此帳號目前沒有任何 Project／Repo membership」，表示目前登入帳號沒有 `demo` membership；回到 `/admin/users` 補上 `reader` 或 `editor`，再回 Desktop 關閉並重新開啟 chooser，或回上一步重新載入。

接著選 workspace 類型：

- **spec-only**：略過 checkout，直接使用 Server 上的規格；適合 PM／PO。
- **remote + checkout**：選本機 Git repo。無 `.speclink.yaml` remote marker 時 Desktop 會驗證後寫入；既有 marker 必須和所選 Server origin／Repo 一致。

handshake 成功後才會建立 remote 分頁。分頁重啟恢復、角色能力與離線狀態都綁定該 Remote Workspace，不會靜默退回 local mode。

## 7. Connect and smoke-test the Remote CLI / 連接並測試 Remote CLI

第 2 節的 `npm run dev` 已經建置好目前 checkout 的 CLI。用 `npm run cli` 執行它，就不必先安裝 CLI，也不會誤用 PATH 上另一版：

- 在 Speclink repo root：`npm run cli -- <args>`。
- 在其他資料夾（本節的測試資料夾就是）：`npm --prefix /path/to/speclink run cli -- <args>`。`--prefix` 只決定用哪個 checkout 的 CLI，CLI 仍作用於你目前所在的資料夾。
- 需要直接解析 `--json` 輸出時加 `--silent`：`npm run --silent cli -- <args>`，避免 npm 的 lifecycle 訊息混進 stdout。
- `--` 之後的參數原樣傳給 CLI；漏掉 `--` 會被 npm 自己吃掉。

在另一個測試資料夾執行以下命令；不要在 Speclink 產品 repo 根目錄執行：

```bash
mkdir -p /tmp/speclink-remote-smoke
cd /tmp/speclink-remote-smoke
npm --prefix /path/to/speclink run cli -- link \
  http://localhost:8080/api/speclink/v1/projects/demo \
  --repo backend
npm --prefix /path/to/speclink run cli -- auth login
```

將 `/path/to/speclink` 換成實際 repo 絕對路徑。`auth login` 提示後再貼 PAT，避免把 token 寫進 shell history。登入後先測讀取：

```bash
npm --prefix /path/to/speclink run cli -- auth status
npm --prefix /path/to/speclink run cli -- list
npm --prefix /path/to/speclink run --silent cli -- list --json
```

再以 `editor` 身分測最小寫入與結構檢查：

```bash
npm --prefix /path/to/speclink run cli -- new change remote-smoke-test
npm --prefix /path/to/speclink run cli -- status --change remote-smoke-test
npm --prefix /path/to/speclink run cli -- validate remote-smoke-test
npm --prefix /path/to/speclink run cli -- analyze remote-smoke-test
```

回 Desktop，確認 `remote-smoke-test` 出現在同一個 `demo`／`backend` 看板。Remote CLI 會在測試資料夾寫入 remote binding 與唯讀 Context Projection；規格寫入仍由 Server Host 處理。

## 8. Verify persistence and recovery / 驗證持久化與恢復

### 一般重啟

在執行 `npm run dev` 的終端按 `Ctrl+C`，再執行：

```bash
npm run dev
```

預期：

- 不再印 setup token。
- Project、Repo、membership、帳號與 change 保留。
- Desktop 儲存的 remote tab 可恢復。

### offline／stale

保持 remote 分頁開啟，停止 Server。預期：

- remote 分頁保留最後 snapshot，呈現 offline／stale。
- snapshot 只能讀；寫入立即被拒絕，不建立隱性 local write queue。
- local 分頁不受影響。

重新執行 `npm run dev`。預期 Desktop 以 Query＋ETag 收斂並重新訂閱 SSE，資料更新後清除 stale 狀態。

### credential 失效

若 Server 回 401 或 credential family 被撤銷，Desktop 會顯示需要重新認證；從 Server 設定重新登入後，原 remote 分頁應原地恢復，不得改成本地 workspace。

## 9. Reset the development environment / 重置開發環境

完全重置前先停止 `npm run dev`，再執行：

```bash
npm run dev:reset
npm run dev
```

預期重新出現全新的 `/setup?token=...`。這會刪除 `.dev/` 內的預設 SQLite Store 與 identity，因此舊帳號、PAT、membership、Project／Repo 與 changes 都無法繼續使用。

若 `.env` 使用 PostgreSQL，`npm run dev:reset` 不會刪除外部資料庫；必須自行 drop／recreate 指定 database 才是完全重置。

## 10. Troubleshooting / 故障排除

| 症狀 | 原因 | 處理方式 |
| --- | --- | --- |
| 直接開啟 `/account/tokens` 得到 404 | 它是 browser-API 端點，不是頁面 | 開啟 `/account`，由 Personal Access Tokens 表單建立 PAT |
| Desktop 顯示沒有任何 Project／Repo membership | registry 已有資源，但目前帳號沒有 Project membership；Admin 也不會 bypass | 到 `/admin/users` 對實際登入帳號授予 `reader` 或 `editor`，再重新開啟 chooser |
| Desktop Project／Repo 清單仍是空的 | 授權加在另一個帳號，或 chooser 尚未重新載入 | 核對 `/account` 的 email，補正 membership，關閉並重新開啟 chooser；必要時登出再登入 |
| 401／需要重新認證 | PAT、access token 或 device credential family 已失效／撤銷 | 從 Desktop Server 設定重新登入，或由 `/account` 建新 PAT |
| Server offline，分頁顯示 stale | SSE／HTTP 暫時不可達 | 保留唯讀 snapshot，不嘗試離線寫入；重啟 Server 等待 Query＋ETag 自動收斂 |
| checkout marker 衝突 | `.speclink.yaml` 指向不同 Server origin 或 Repo | 不要手改掩蓋；選正確 checkout，或依 Desktop 衝突對話選擇本地、以 Server 為準或遷移 |
| CLI 顯示 not logged in | 已 link，但該 Server origin 尚無 credential | 在同一測試資料夾執行 `npm --prefix /path/to/speclink run cli -- auth login` |
| CLI 行為與 Server／Desktop 對不上 | PATH 上的 `speclink` 是另一版或另一個 checkout 建的 | 改用 `npm run cli`（或 `npm --prefix <checkout> run cli`）執行目前 checkout 的 binary，不必動 PATH |
| `npm run cli` 說無法執行 checkout CLI | 目前 checkout 尚未建置 debug binary | 執行 `npm run dev` 或 `cargo build -p speclink-cli`；訊息會同時列出 binary 路徑與 cwd 供比對 |
| 重啟後沒有 setup token | setup 已完成且資料仍存在 | 直接登入 `/account`／`/admin`；只有完全 `npm run dev:reset` 才會重新 setup |
| reset 後舊 Desktop connection 失效 | identity、Project 與 credential 已被刪除 | 完成新 setup、重授 membership，再新增或重新登入 connection |

完成以上流程後，你已驗證同一個 Remote Project／Repo 可由 Server、Desktop 與 CLI 共用。若要驗證完整角色、事件恢復與多分頁情境，再依 [Phase 3 acceptance spec](../openspec/specs/phase3-acceptance/spec.md) 執行專案測試。
