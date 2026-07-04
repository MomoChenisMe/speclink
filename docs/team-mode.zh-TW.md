# 團隊模式（Remote Store）

團隊模式下，規格文件與 change 狀態存放在**團隊系統**（內嵌 Speclink 引擎的 server），你的 code 與 git 留在本地。`speclink` CLI 成為[動詞契約](verb-contract.zh-TW.md)的薄 client：你熟悉的每個動詞——`list`、`status`、`instructions`、`task done`、`discuss …`——輸出形狀完全不變，只有背後的儲存搬了家。

本文涵蓋 client 端：連接 repo、認證、repo 識別的運作、錯誤訊息的含義。server 端實作屬團隊系統，由動詞契約規範。

## 連接檔與模式解析

一個檔案決定模式：

| repo 的狀態 | 模式 |
|---|---|
| 無 `.speclink.remote.yaml` | **fs**——經典本地 `openspec/` 佈局，一切照舊 |
| 有 `.speclink.remote.yaml` | **remote**——動詞呼叫團隊系統的契約端點 |
| 連接檔與 `openspec/` 並存 | **remote 勝出**；每個指令在 stderr 印一行並存警告（通常是遷移殘留——移除其中一邊） |

檔案只有兩個欄位：

```yaml
# .speclink.remote.yaml — committed，如 .lfsconfig：每個 clone 拿到同一份綁定
url: https://team.example.com/api/speclink/v1/projects/erp   # 必填，含專案範疇
repo: backend    # 單 repo 專案可缺省——本 repo 在專案內的註冊名
```

- `url` 攜帶**專案範疇**——一台團隊 server 可承載多個專案；你的 repo 恰綁定一個。
- `SPECLINK_STORE_URL` 為單一 shell 或 CI job 覆寫 url（staging server、port-forward）。它只覆寫既有連接的 url——絕不會把 fs 工作區變成 remote。
- 憑證永不放在這個檔案（也不放在 repo 內任何地方）。

## 建立連接

### 新 repo：`speclink init --store remote`

```bash
speclink init --store remote --url https://team.example.com/api/speclink/v1/projects/erp --repo backend
```

只執行 **workspace init**：`CLAUDE.md`／`AGENTS.md` marker 區塊（remote 措辭）、技能、`.claude/settings.json`、`.gitignore` 條目、連接檔。刻意**不**建立 `openspec/` 樹——文件活在 server 上。

### 既有 repo：`speclink link`／`speclink unlink`

```bash
speclink link https://team.example.com/api/speclink/v1/projects/erp --repo backend
speclink unlink   # 移除連接檔；回到 fs 模式
```

`link` 寫入連接檔。已登入時立即驗證（見下）；未登入時照樣寫檔並提醒執行 `speclink auth login`——離線 link 絕不阻塞，改由第一個動詞驗證。

### 認證：`speclink auth login`／`speclink auth status`

```bash
speclink auth login             # 互動貼上 PAT
speclink auth login --token-stdin   # 腳本化（CI）
speclink auth status            # 我是誰、repo 是否註冊、有無 fork 警告
```

- PAT 在團隊系統 UI 簽發。視同 SSH 私鑰管理：疑似洩漏立即到那裡撤銷。
- `login` **先**向 server 驗證 token 再儲存——被拒絕的 token 絕不落檔。
- 憑證依 server origin 存於**使用者層級**設定目錄（`credentials.yaml`，Unix 權限 0600；Windows 沿使用者目錄 ACL）。同一台 server 的所有專案與所有 clone 共用一次登入。
- `SPECLINK_TOKEN` 供 CI／headless 覆寫憑證檔。空值視為未設定。
- 未登入執行 remote 動詞？一行訊息——`not logged in to <origin> — run `speclink auth login``——與非 0 exit code。不猜測、不快取。

## Repo 識別——宣告一次、自動攜帶、逐動詞驗證

三層把你的 repo 接到專案（只有出錯時才會感覺到它們）：

1. **Repo → 專案**：連接 `url` 含專案範疇。
2. **Repo 身分**：`repo:` 欄位是本 repo 在專案 repos 註冊表（server 端管理）的註冊名。`init`／`link` 在有憑證時立即驗證——名字不在註冊表即大聲失敗並列出可用名單，且不寫連接檔。單 repo 專案可缺省，由 server 解析。
3. **Change 歸屬**：每個 change **恰歸屬一個 repo**（v1 規則）。建立 change 時歸屬你的 repo；`speclink list` 只列你的 repo 的 changes；在錯的 repo 執行 change 範疇動詞會失敗，訊息同時指名兩個 repo。跨 repo 需求拆成多個 change，一 repo 一個。

每個請求自動攜帶你的 repo 名（`X-Speclink-Repo`）；不需逐指令傳遞。

**輔助 fork 警告**：server 註冊表記錄了本 repo 的 git URL 參考值、且本地 `git remote origin` 與之不符時，`link` 與 `auth status` 在 stderr 印一行提示你可能在 fork 或鏡像上工作。它絕不影響結果與 exit code；無參考值或非 git 目錄時靜默跳過。

## 錯誤訊息速查

每個 server 錯誤都翻譯為單行語義化訊息＋建議動作——CLI 絕不輸出裸 HTTP 狀態碼。完整正典目錄見[動詞契約 §4](verb-contract.zh-TW.md#4-錯誤-reason-目錄)；實際會遇到的：

| 你看到 | 含義 | 這樣做 |
|---|---|---|
| `not logged in to <origin> — run \`speclink auth login\`` | 沒有此 server 的憑證 | 登入 |
| `credentials expired/revoked — run \`speclink auth login\`` | PAT 已失效 | 重新簽發並登入；CLI 絕不重試失效 token |
| `repo is not registered in this project (available: …)` | `repo:` 不在註冊表 | 修正 `repo:` 或重跑 `speclink link` |
| `change belongs to repo 'backend' but you are 'frontend' — run this verb from the owning repo` | 對錯誤 repo 的 change 執行動詞 | 到歸屬 repo 執行 |
| `change is held by <user> — coordinate, or re-claim if it was released` | 被他人認領（或先一步認領） | 協調／釋放後 `speclink claim` |
| `content changed since you read it — re-read it and re-apply your edit` | artifact 寫入的樂觀並行衝突 | 重新讀取（`speclink artifact cat`）後再改 |
| `change is <state> — wait for the in-flight operation to finish, then retry` | server 端操作（如 ingest 合併）持有 change | 等待後重試——你的認領仍在 |
| `waiting for <gate> approval in the team system — ask the approver` | gate（proposal 或 archive）待核准 | 核准在團隊系統 UI 進行 |
| `N task(s) still open — finish them before archiving` | 尚有未勾 task 就 archive | 先 `speclink task done …` |
| `server unreachable — check the connection url in .speclink.remote.yaml (or SPECLINK_STORE_URL)` | 連不上 | 修連線；**沒有離線模式、沒有快取答案** |
| `server does not support this CLI's API version — upgrade the CLI or the server` | 契約版本不符 | 升級落後的那一側 |

## 從純本地專案升級

`speclink store push`（把既有 `openspec/` 樹整批遷入空的遠端專案）已規劃但**尚未提供**。在那之前的手動路徑：

1. 在團隊系統建立專案並註冊 repo(s)；請 PM 在那裡重建**進行中**的 changes（或對 remote store 重跑 `/speclink-propose`——通常更乾淨，proposal 會被重新驗證）。
2. 正典規格：把每個 `openspec/specs/<capability>/spec.md` 貼進團隊系統（或其匯入功能）。
3. 在 repo 裡：`speclink link <url> --repo <name>`，然後 `speclink auth login`。
4. 用 `speclink list` 驗證——你應該看到 server 上的 changes，而非本地樹。
5. 移除本地 `openspec/` 目錄（內容已完整存在於 server）——在移除前，每個指令都會提醒兩者並存且 remote 勝出。
6. 若重視本地稽核軌跡，`openspec/changes/archive/` 歷史可留在 git；server 端歷史自遷移起算。

回退是對稱的：`speclink unlink` 就回到 fs 模式，以 repo 內尚存的 `openspec/` 內容為準。

## Remote 模式下仍留在本地的東西

- `.speclink.yaml`（tools 清單、workspace 選項）——工作區仍自行決定生成哪些 AI harness 檔案。
- `.speclink/` 工作資料（touched 記錄）、`.gitignore`、生成的技能與 marker 區塊。
- Workflow **政策**（`locale`、`tdd`、`audit`、schema、context、rules）由團隊系統提供（`GET /config`）——remote 模式沒有本地 `openspec/config.yaml`，政策因此不可能在網頁端與你的 repo 之間分岔。
