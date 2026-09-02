---
title: CLI 連接 remote
section: Remote 模式
order: 460
keywords: [link, auth login, 裝置登入, 存取金鑰, 金鑰圈, logout, .speclink.yaml]
sources: [remote-connection, remote-auth, server-device-auth, user-documentation]
generated: 2026-09-02
---

# CLI 連接 remote

要讓 CLI 對著 server 工作，你需要兩件事：一個連接設定，以及一把憑證。連接設定寫在專案的 `.speclink.yaml`。憑證存在你自己的機器上，不進 repo。

## 連接：`init`、`link`、`unlink`

新專案直接以 remote 模式初始化：

```
speclink init --store remote --url <project URL> --repo <repo> --tools claude
```

- 寫入前 CLI 先要一個 Claude／Codex 的選集。帶 `--tools` 就直接用。互動終端沒帶時，CLI 在 stderr 詢問。非互動終端沒帶時，CLI 什麼都不寫、以非零 exit code 結束，stderr 指引 `--tools claude`、`codex` 或 `claude,codex`。
- 成功後，專案裡有 Skills、`.gitignore`，以及含 tools 與 remote 區段的 `.speclink.yaml`。
- 不會建立 `openspec/` 目錄，不會建立獨立連接檔，也不會建立 `CLAUDE.md` 或 `AGENTS.md`。

既有專案接上 server：

```
speclink link http://localhost:8080/api/speclink/v1/projects/demo --repo backend
```

- `link` 只寫入或更新 `.speclink.yaml` 的 remote 區段。tools 與其他欄位保留。
- URL 用 project-scoped API URL，不是 server 首頁。三種網址的分別見 [啟動 server](server-start.md)。

> [!WARNING]
> 不要在產品 repo 的根目錄誤寫測試用的連接設定。連測試 server 時用一個獨立的資料夾。

`speclink unlink` 移除 remote 區段，其他欄位保留。之後的指令回到本地模式。

`init` 與 `link` 共同的行為：

- 當下已有可用憑證時，CLI 立刻向 server 查這個 repo 是否屬於該專案，並回報結果。repo 不在專案裡：非零 exit code，stderr 指出並列出可用的儲存庫代號，remote 區段不寫入。
- 沒有憑證時，CLI 提示執行 `speclink auth login`。
- server 提供本 repo 的 git 位址參考值，而它與本地 git remote 不一致時：stderr 一行警告，提示你可能在 fork 或鏡像上工作。這只是警告，不影響結果。

## 模式怎麼判定

- `.speclink.yaml` 有 remote 區段：remote 模式。沒有：本地模式。
- remote 區段與 `openspec/` 目錄並存：remote 生效，stderr 一行並存警告。
- 環境變數 `SPECLINK_STORE_URL` 存在時，覆寫區段裡的 url。
- remote 區段存在，但區段的 url 與環境變數都缺：非零 exit code，訊息同時提示 `remote.url` 欄位與 `SPECLINK_STORE_URL` 兩種設法。CLI 不會偷偷改用本地模式。
- `.speclink.yaml` 無法解析：一律失敗，stderr 指出檔案與解析原因。CLI 不讀本地資料，也不發任何遠端請求。這是刻意的設計。
- 專案根殘留舊的 `.speclink.remote.yaml`：stderr 一行遷移警告，指引把 url 與 repo 搬進 `.speclink.yaml` 的 remote 區段並刪掉舊檔。舊檔不參與模式判定。

## 每個 remote 動詞怎麼跑

- 動詞執行前，CLI 先向 server 確認你要操作的專案與 repo（規格稱為 handshake）。確認失敗時，動詞以非零 exit code 停止，錯誤指向原因。原因有四種：API 版本不相容、找不到對應的專案或 repo、無權限、專案有多個候選 repo。
- 動詞自動帶上 remote 區段的 repo 名。你操作的變更屬於別的 repo 時，CLI 以非零 exit code 結束。訊息一行同時列出變更所屬的 repo、當前 repo 名，以及改正指引。

## 登入：`speclink auth login`

登入有三條路：

1. 裝置登入。在互動終端不帶旗標執行 `speclink auth login`。
2. 存取金鑰登入。執行 `speclink auth login --pat`，互動輸入存取金鑰。
3. 腳本登入。執行 `speclink auth login --token-stdin`，從 stdin 讀存取金鑰。CI 用這條。

會擋下的情況：

- `--pat` 與 `--token-stdin` 同時給：非零 exit code，stderr 說明兩個旗標互斥。
- 非互動終端且不帶旗標：非零 exit code，stderr 指引 `--token-stdin`，不發任何網路請求。
- server 不支援裝置登入：非零 exit code，指引 `--pat`。
- 你的機器沒有 OS 金鑰圈，或金鑰圈存取被拒：非零 exit code，說明長效憑證不能存成明文檔，指引 `--pat` 或環境變數 `SPECLINK_TOKEN`。

裝置登入的流程：

1. CLI 在 stdout 印出驗證網址與裝置碼。裝置碼的格式是 `XXXX-XXXX`。可以開瀏覽器的環境會同時開啟核准頁。
2. 你可以在任何一部裝置開啟核准頁。核准頁在 `/activate`，需要已登入的瀏覽器工作階段。還沒登入時，瀏覽器先導向登入頁，登入後回到核准頁，裝置碼已預填。
3. 按下一步。這時才看到核准與拒絕兩個選項。
4. 選核准或拒絕。結果頁兩種情況都提示你回到 Speclink app 繼續。
5. CLI 在這段期間依 server 宣告的間隔輪詢。核准後，CLI 把長效的續約憑證與短效的通行證存進 OS 金鑰圈，顯示登入身分，exit code 0。憑證檔不寫入。

裝置登入失敗時，訊息可區分兩種：核准頁上被拒絕，或逾期沒人核准。輸入不存在、已用或逾期的裝置碼，核准頁都給同一個無效回應。

存取金鑰登入時，CLI 依 server 位址把金鑰存進使用者設定目錄的憑證檔。Unix 上檔案權限為 0600。專案 repo 內不會有任何新增或變更的檔案。

## 憑證從哪裡來：四層階梯

remote 動詞與 `speclink auth status` 依固定順序找憑證：

1. 環境變數 `SPECLINK_TOKEN`。
2. OS 金鑰圈裡的續約憑證。桌面 app 或 CLI 裝置登入都會建立它，同一台機器、同一個 server 共用一份。
3. OS 金鑰圈裡的存取金鑰。桌面 app 的存取金鑰登入會建立它。
4. 憑證檔裡的存取金鑰。CLI 的 `--pat` 或 `--token-stdin` 會建立它。

- 某一層不可用時（沒有金鑰圈服務、存取被拒、條目不存在），CLI 靜默探下一層。金鑰圈不可用不會讓動詞失敗。
- 四層都沒有：非零 exit code 報未登入，指引 `speclink auth login`。
- 桌面 app 登入過的 server，CLI 免登入就能用。

`speclink auth status` 顯示目前解析到的身分、repo 驗證結果，以及憑證來自哪一層。未登入時 exit code 非 0，並附登入指引。

## 憑證失效時

- 憑證來自金鑰圈的續約憑證：CLI 靜默續約一次再重試。成功就完成動詞，你不會看到任何登入提示。續約被 server 拒絕（這組憑證已被撤銷）時，CLI 清除該 server 的金鑰圈條目，以非零 exit code 結束，提示 `speclink auth login`。同一次執行不會改用其他來源的憑證。
- 憑證來自環境變數或存取金鑰：非零 exit code，提示重新登入，不重試。
- 同一台機器多個 speclink 程序同時需要續約：只會向 server 續約一次，兩邊都成功。等待逾時時，CLI 報錯指出疑似有其他 speclink 程序長時間持鎖，不會無限期停住。
- 桌面 app 與 CLI 交錯使用同一個 server，不會互相把對方登出。

## 登出：`speclink auth logout`

- 金鑰圈有續約憑證時，CLI 先呼叫 server 撤銷這組裝置憑證。
- 然後清除該 server 的所有本機憑證：金鑰圈的續約憑證、通行證快取、存取金鑰條目，以及憑證檔裡該 server 的條目。
- server 端的存取金鑰不會被撤銷。要撤銷它，到 `/account` 操作。見 [帳號](accounts.md)。
- 成功時 exit code 0，顯示已登出的 server 位址。該 server 沒有任何本機憑證時，非零 exit code 報未登入。
- server 不可達時，本機憑證仍被清除，exit code 0，stderr 警告 server 端的裝置憑證未撤銷。
- 登出後，桌面 app 對同一個 server 的下一次操作也回到未登入狀態。

## 連上之後

登入後，用讀取動詞（例如 `speclink list`）確認連線正常，再試寫入動詞。規格未載試跑用的具體動詞清單。動詞在 remote 模式與本地模式的輸出同形。

**出處**：`remote-connection`、`remote-auth`、`server-device-auth`、`user-documentation`
