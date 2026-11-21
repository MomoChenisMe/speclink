---
title: CLI 連接 remote
section: Remote 模式
order: 460
keywords: [link, auth login, auth status, 瀏覽器登入, 裝置登入, 存取金鑰, 登入方式, 金鑰圈, logout, .speclink.yaml, 多個 repo, "--repo", 信任庫, CA, certificate, server unreachable, server unavailable, 暫時無法服務, Pi, Cursor]
sources: [remote-connection, remote-auth, server-device-auth, "user-documentation#Remote Getting Started 提供可重複的完整操作路徑", "user-documentation#遠端專案的登入與技能來源有說明", "workspace-tools#init 內建 Agent 工具選擇", "workspace-tools#remote 專案的技能取自 Server", "host-kit#登入方式由宿主宣告", "host-kit#範圍與能力由宿主的函式決定"]
generated: 2026-11-21T17:53:22+08:00
---

# CLI 連接 remote

要讓 CLI 對著 server 工作，你需要兩件事：一個連接設定，以及一把憑證。連接設定寫在專案的 `.speclink.yaml`。憑證存在你自己的機器上，不進 repo。

## 連接：`init`、`link`、`unlink`

新專案直接以 remote 模式初始化：

```
speclink init --store remote --url <project URL> --repo <repo> --tools claude
```

- 寫入前 CLI 先要一個工具選集：`claude`、`codex`、`copilot`、`pi`、`cursor` 任選一個或多個，多個用逗號分隔。帶 `--tools` 就直接用。互動終端沒帶時，CLI 在 stderr 依序問 Claude、Codex、Copilot、Pi、Cursor 五題。非互動終端沒帶時，CLI 什麼都不寫、以非零 exit code 結束，stderr 指出 `--tools`、五個工具名與逗號組合的寫法。
- 成功後，專案裡有 Skills、`.gitignore`，以及含 tools 與 remote 區段的 `.speclink.yaml`。
- 不會建立 `speclink/` 目錄，不會建立獨立連接檔，也不會建立 `CLAUDE.md` 或 `AGENTS.md`。
- 輸出的結尾多一行提示：登入後執行 `speclink update`，取得 Server 提供的技能。見下方「連上之後」。

> [!NOTE]
> 可選的工具在規格裡有兩種說法。較早的 remote 連接規格只列 Claude 與 Codex；較新的工具選擇規格寫所有 `speclink init`（含 remote）都接受五個工具。上面照較新的說法寫，差異記在 [本手冊的來源](about.md)。

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

- 當下已有可用憑證時，CLI 立刻向 server 查這個 repo 是否屬於該專案，並回報結果。
- 沒有憑證時，CLI 提示執行 `speclink auth login`。
- server 提供本 repo 的 git 位址參考值，而它與本地 git remote 不一致時：stderr 一行警告，提示你可能在 fork 或鏡像上工作。這只是警告，不影響結果。

查驗的結果，看專案註冊了幾個 repo，以及你有沒有給 `--repo`：

| 專案註冊的 repo | 你給的 `--repo` | `link` 的結果 | `init` 的結果 |
| --- | --- | --- | --- |
| 一個 | 沒給 | 和以前相同 | 和以前相同 |
| 一個或多個 | 已註冊的名稱 | 和以前相同，連接成功 | 和以前相同 |
| 兩個以上 | 沒給 | 非零 exit code，remote 區段不寫入 | 初始化照常完成，之後以非零 exit code 結束 |
| 任何 | 沒註冊的名稱 | 非零 exit code，remote 區段不寫入 | 規格未載 |

專案有多個 repo，這件事本身不會讓查驗失敗。指名一個已註冊的 repo，就照常連上。兩種失敗的 stderr 訊息：

- **專案有兩個以上 repo，而你沒有指名**：訊息指引你用 `speclink link <url> --repo <名稱>` 指名一個，並列出全部可用的註冊名。`init` 與 `link` 印的是同一則訊息。`init` 時，Skills、`.gitignore` 與含 remote URL 的 `.speclink.yaml` 都已建立，照訊息再執行一次 `speclink link` 即可。`--no-color` 下訊息的文字相同。
- **指名的 repo 沒有註冊**：訊息列出可用的註冊名。例如專案註冊 erp 與 crm，你給了 `--repo nope`：

  ```
  Error: repo 'nope' is not registered in this project (available: erp, crm)
  ```

## 模式怎麼判定

- `.speclink.yaml` 有 remote 區段：remote 模式。沒有：本地模式。
- remote 區段與 `speclink/` 目錄並存：remote 生效，stderr 一行並存警告。
- 環境變數 `SPECLINK_STORE_URL` 存在時，覆寫區段裡的 url。
- remote 區段存在，但區段的 url 與環境變數都缺：非零 exit code，訊息同時提示 `remote.url` 欄位與 `SPECLINK_STORE_URL` 兩種設法。CLI 不會偷偷改用本地模式。
- `.speclink.yaml` 無法解析：一律失敗，stderr 指出檔案與解析原因。CLI 不讀本地資料，也不發任何遠端請求。這是刻意的設計。
- 專案根殘留舊的 `.speclink.remote.yaml`：stderr 一行遷移警告，指引把 url 與 repo 搬進 `.speclink.yaml` 的 remote 區段並刪掉舊檔。舊檔不參與模式判定。

## 每個 remote 動詞怎麼跑

- 動詞執行前，CLI 先向 server 確認你要操作的專案與 repo（規格稱為 handshake）。確認失敗時，動詞以非零 exit code 停止，錯誤指向原因。原因有四種：API 版本不相容、找不到對應的專案或 repo、無權限、專案有多個候選 repo。
- 專案有多個候選 repo，而 remote 區段沒有 repo：stderr 指出這件事並列出候選。Server 拒絕的訊息另外指出兩種指名方式：`X-Speclink-Repo` 標頭，或 `.speclink.yaml` 的 `remote.repo`。用 CLI 時，執行 `speclink link <url> --repo <名稱>`，把 repo 寫進 remote 區段。
- 動詞自動帶上 remote 區段的 repo 名。你操作的變更屬於別的 repo 時，CLI 以非零 exit code 結束。訊息一行同時列出變更所屬的 repo、當前 repo 名，以及改正指引。

## HTTPS 連線：信不信任 Server 的憑證

這一節的「憑證」指 Server 用來證明自己身分的 HTTPS 憑證（certificate），不是你登入用的憑證。

- CLI 與桌面 app 以 HTTPS 連到 Server 或登入服務時，信不信任對方的憑證，由作業系統的信任庫決定。
- 簽發憑證的 CA（憑證簽發單位）在作業系統的信任庫裡，就連得上。公開的 CA 與組織自己的 CA 都一樣。你不需要為此設定任何旗標、設定檔欄位或環境變數。
- 每一種連線都用同一條規則：查詢登入方式、登入、一般指令、事件串流與狀態查詢。CLI 與桌面 app 對同一台 Server 的信任結果相同。
- 作業系統沒有提供任何 CA 時（例如沒有安裝 CA 憑證套件的精簡 Linux 容器），改用程式內建的公開 CA 清單，不會因此讓全部 HTTPS 連線失敗。
- 以 `http://` 開頭的連線不做任何憑證檢查，例如本機的示範用 server。

Server 的憑證被拒絕時（簽發的 CA 不在信任庫、憑證過期、主機名稱不符等），CLI 把它當成連線失敗：不讀快取、不重試，也不會改當成本機專案來跑。

- CLI 以非零 exit code 結束，stderr 印一行英文訊息。
- 訊息以 `server unreachable` 開頭，含 `certificate` 一詞與被拒絕的原因，並指出處理方法：把簽發憑證的 CA 加進作業系統的信任庫。
- 原因是作業系統回報的名稱。同一種情況在不同作業系統、或 Server 送出的憑證鏈不同時，名稱可能不同；作業系統沒給具體名稱時是 `Other`。
- 訊息固定是英文，不隨語言設定改變，`--no-color` 下文字相同。
- 登入服務的憑證被拒絕時，登入指令同樣以非零 exit code 結束，訊息同樣含 `certificate`、被拒絕的原因與處理方法。

| 失敗的原因 | stderr 訊息的開頭 | 訊息另外含有 |
| --- | --- | --- |
| 簽發的 CA 不在信任庫，Server 只送出自己的憑證（macOS、Linux） | `Error: server unreachable — ` | `certificate`、`UnknownIssuer`、`trust store` |
| 簽發的 CA 不在信任庫，Server 連根憑證一起送出（macOS） | `Error: server unreachable — ` | `certificate`、`Other`、`trust store` |
| Server 沒有在聽（拒絕連線） | `Error: server unreachable — check the connection url` | 不含 `certificate` |

憑證以外的連線失敗（拒絕連線、逾時、DNS 解析失敗）是另一句訊息，不含 `certificate`：

```
server unreachable — check the connection url (`remote.url` in .speclink.yaml or SPECLINK_STORE_URL)
```

桌面 app 把憑證被拒絕呈現為連線中斷，寫入動作停用，與 Server 沒有回應時相同。見 [失聯與恢復](remote-offline.md)。

## Server 暫時無法服務時

Server 以 500 以上的狀態碼回應時，CLI 以非零 exit code 結束。stderr 的訊息看 Server 有沒有說明原因：

- Server 回的是可讀的錯誤說明，標明暫時無法服務，而且附上原因文字：訊息是 `server unavailable — <原因>`。原因照 Server 寫的原文顯示，不翻譯、不截短，也沒有檢查連接網址的提示。
- 其他情況：訊息和以前相同，提示你檢查連接網址：

```
server unavailable — check the connection url (`remote.url` in .speclink.yaml or SPECLINK_STORE_URL)
```

| Server 的回應 | 訊息 |
| --- | --- |
| 503，標明暫時無法服務，原因是「資料庫切換中」 | `server unavailable — 資料庫切換中` |
| 503，標明暫時無法服務，但原因是空的 | `server unavailable — check the connection url (…)` |
| 500，標明是內部錯誤 | `server unavailable — check the connection url (…)` |
| 502，回的是 HTML 網頁 | `server unavailable — check the connection url (…)` |
| 連不上 Server | `server unreachable — check the connection url (…)` |

例如 Server 維護期間對登入方式的查詢回 503，原因是「系統維護中，預計 10 分鐘後恢復」。這時執行 `speclink auth login --browser`，stderr 是：

```
Error: server unavailable — 系統維護中，預計 10 分鐘後恢復
```

桌面 app 進入離線時，離線橫幅也會顯示 Server 回報的原因。見 [失聯與恢復](remote-offline.md)。

## 登入：`speclink auth login`

登入有三種方式：瀏覽器登入、裝置登入、存取金鑰。

| 指令 | 用哪一種 |
| --- | --- |
| `speclink auth login` | 在互動終端執行。CLI 先問 Server 支援哪些方式，再自動選一種 |
| `speclink auth login --browser` | 指定瀏覽器登入 |
| `speclink auth login --device` | 指定裝置登入 |
| `speclink auth login --pat` | 存取金鑰，互動輸入 |
| `speclink auth login --token-stdin` | 存取金鑰，從 stdin 讀。CI 與腳本用這條 |

### CLI 怎麼選登入方式

不帶 `--pat` 與 `--token-stdin` 時，CLI 在登入前先向 Server 發一個不帶憑證的查詢，問它宣告支援哪些登入方式：瀏覽器登入、裝置登入、存取金鑰。

- 沒有指定旗標：在 Server 宣告的方式裡，依固定順序選第一個：瀏覽器登入、裝置登入、存取金鑰。這個順序不隨 Server 回應裡的排列改變。
- 選到存取金鑰（Server 只宣告這一種）：非零 exit code，stderr 指引 `--pat` 或 `--token-stdin`。CLI 不會發起裝置登入。
- Server 沒有宣告登入方式（查詢回 404 或 405）：維持原本的行為，走裝置登入；Server 不支援裝置登入時，非零 exit code 並指引 `--pat`。
- 查詢回其他錯誤，或 Server 連不上：非零 exit code 報連線錯誤。CLI 不會改用任何登入方式。
- CLI 不認得的方式名稱直接略過，不報錯。
- Server 宣告了瀏覽器登入，但缺少發行者網址或用戶端識別：當成沒有宣告瀏覽器登入。

官方的示範用 server 宣告裝置登入與存取金鑰兩種，所以對它不帶旗標登入時，走的是裝置登入。

要改用另一種方式，就帶對應的旗標。會擋下的情況：

- `--browser`、`--device`、`--pat`、`--token-stdin` 四個旗標任兩個同時給：非零 exit code，不進入任何登入流程。
- 非互動終端且不帶旗標：非零 exit code，stderr 指引 `--token-stdin`，不發任何網路請求。
- 帶 `--browser` 或 `--device`，而 Server 有宣告但不含這一種：非零 exit code，stderr 列出 Server 支援的方式。不會開啟瀏覽器。
- 帶 `--browser`，而 Server 沒有宣告登入方式：非零 exit code，說明這台 Server 沒有宣告瀏覽器登入。
- 你的機器沒有 OS 金鑰圈，或金鑰圈存取被拒：瀏覽器登入與裝置登入都以非零 exit code 結束，說明長效憑證不能存成明文檔，指引 `--pat` 或環境變數 `SPECLINK_TOKEN`。瀏覽器登入會在開啟瀏覽器之前就結束。

### 瀏覽器登入的流程

瀏覽器登入用的是 Server 宣告的登入服務，CLI 印出與開啟的是那個登入服務的授權網址。

1. CLI 向登入服務取得它的說明文件並檢查。檢查不過時，CLI 以非零 exit code 結束、指出是哪一項，不開啟瀏覽器。
2. CLI 在 stdout 印出授權網址。可以開瀏覽器的環境會同時開啟它。
3. CLI 在本機的 `127.0.0.1` 開一個由作業系統指定的埠等結果，回呼位址是 `http://127.0.0.1:<埠>/callback`，不用 `localhost`。
4. 你在瀏覽器完成登入。
5. CLI 把長效的續約憑證與短效的通行證存進 OS 金鑰圈，清除這個 Server 在金鑰圈裡的存取金鑰，顯示登入身分，exit code 0。憑證檔不寫入。

第 1 步的檢查會擋下這些情況：

- 說明文件裡的發行者（issuer，登入服務公告的自己的網址）與 Server 宣告的發行者網址不完全相同：stderr 指出 issuer 不符。
- 說明文件缺授權端點或 token 端點。
- 說明文件列出的 PKCE 方式不含 S256（例如只有 plain）：stderr 指出登入服務不支援 S256。PKCE 是登入流程裡防止授權碼被別人攔走的機制。
- 發行者、授權端點或 token 端點不是 https：stderr 指出必須是 https。主機是本機位址時例外。

登入失敗時，三種情況的訊息可以區分，金鑰圈都不會新增任何條目：

- 你在登入服務拒絕。
- 300 秒內沒有完成登入。這時本機的埠已關閉。
- 登入服務拒絕換發。

登入之後：

- 通行證到期時，CLI 靜默向登入服務換新，你不會看到任何登入提示。規則與裝置登入相同，見下方「憑證失效時」。
- 登入服務沒有發續約憑證時，登入當下仍然成功。通行證到期後的指令以非零 exit code 結束，提示 `speclink auth login`。

### 裝置登入的流程

1. CLI 在 stdout 印出驗證網址與裝置碼。裝置碼的格式是 `XXXX-XXXX`。可以開瀏覽器的環境會同時開啟核准頁。
2. 你可以在任何一部裝置開啟核准頁。核准頁在 `/activate`，需要已登入的瀏覽器工作階段。還沒登入時，瀏覽器先導向登入頁，登入後回到核准頁，裝置碼已預填。
3. 按下一步。這時才看到核准與拒絕兩個選項。
4. 選核准或拒絕。結果頁兩種情況都提示你回到 Speclink app 繼續。
5. CLI 在這段期間依 server 宣告的間隔輪詢。核准後，CLI 把長效的續約憑證與短效的通行證存進 OS 金鑰圈，清除這個 Server 在金鑰圈裡的存取金鑰，顯示登入身分，exit code 0。憑證檔不寫入。

裝置登入失敗時，訊息可區分兩種：核准頁上被拒絕，或逾期沒人核准。輸入不存在、已用或逾期的裝置碼，核准頁都給同一個無效回應。

### 存取金鑰登入

`--pat` 與 `--token-stdin` 不會先問 Server 支援哪些登入方式。

1. CLI 先向 Server 確認這把金鑰有效。金鑰有效但你不是這個專案的成員時，登入仍然成功；權限不足由之後的動詞回報。
2. CLI 依 Server 位址把金鑰存進 OS 金鑰圈。金鑰圈不可用（沒有金鑰圈服務，或存取被拒）時，改存進使用者設定目錄的憑證檔，Unix 上檔案權限為 0600。
3. 登入成功時，CLI 清除這個 Server 在金鑰圈裡的續約憑證與通行證。

- 專案 repo 內不會有任何新增或變更的檔案。
- Server 不接受這把金鑰：非零 exit code，stderr 說明認證失敗。金鑰圈與憑證檔都不會新增內容。
- 以前存在憑證檔裡的存取金鑰照常可用，不會被搬走。
- 在有金鑰圈的機器用 `--pat` 登入後，桌面 app 伺服器頁的這個 Server 顯示為已登入。

## 憑證從哪裡來：四層階梯

remote 動詞與 `speclink auth status` 依固定順序找憑證：

1. 環境變數 `SPECLINK_TOKEN`。
2. OS 金鑰圈裡的續約憑證。桌面 app 或 CLI 的瀏覽器登入、裝置登入都會建立它，同一台機器、同一個 server 共用一份。
3. OS 金鑰圈裡的存取金鑰。桌面 app 的存取金鑰登入，以及 CLI 的 `--pat` 或 `--token-stdin` 會建立它。
4. 憑證檔裡的存取金鑰。金鑰圈不可用時，CLI 的 `--pat` 或 `--token-stdin` 把金鑰存在這裡；以前存進來的金鑰也還在這裡。

- 某一層不可用時（沒有金鑰圈服務、存取被拒、條目不存在），CLI 靜默探下一層。金鑰圈不可用不會讓動詞失敗。
- 四層都沒有：非零 exit code 報未登入，指引 `speclink auth login`。
- 桌面 app 登入過的 server，CLI 免登入就能用。

`speclink auth status` 顯示目前解析到的身分、憑證來自哪一層，以及 repo 的查驗結果：

- 第一行是身分。第二行是兩個空白加 `Credential from: <來源說明>`。`--no-color` 下這兩行的文字相同。
- 憑證是金鑰圈裡的續約憑證時，來源說明是 `system keychain (login session, shared with the desktop app)`。它不寫是瀏覽器登入還是裝置登入，因為金鑰圈的條目不記錄這件事。以前這一行寫的是 `device login`。
- 其他三層（環境變數、金鑰圈的存取金鑰、憑證檔）的來源說明不變。
- 專案註冊多個 repo 時照常完成，exit code 0。remote 區段有 repo 時，印出它的查驗結果，例如 `Repo 'crm' is registered in this project`；沒有註冊時，改印一行以 `! ` 開頭的提示，列出可用的註冊名。remote 區段沒有 repo 時，印一行以 `! ` 開頭的提示，指引你用 `speclink link <url> --repo <名稱>` 指名，並列出全部可用的註冊名。
- 加 `--json` 時只輸出身分與憑證來源，不輸出 repo 的查驗結果或提示。
- 未登入、憑證被拒、或你不是這個專案的成員時，exit code 非 0。未登入時附登入指引。不是成員時，stderr 是存取被拒的訊息，stdout 不印身分與憑證來源。

## 憑證失效時

- 憑證來自金鑰圈的續約憑證：CLI 靜默續約一次再重試。成功就完成動詞，你不會看到任何登入提示。續約被 server 拒絕（這組憑證已被撤銷）時，CLI 清除該 server 的金鑰圈條目，以非零 exit code 結束，提示 `speclink auth login`。同一次執行不會改用其他來源的憑證。
- 瀏覽器登入的續約憑證向登入服務換新，規則相同：靜默換新一次再重試；登入服務拒絕換新時，CLI 清除該 server 在金鑰圈裡的續約憑證與通行證，提示 `speclink auth login`。
- 憑證來自環境變數或存取金鑰：非零 exit code，提示重新登入，不重試。
- 同一台機器多個 speclink 程序同時需要續約：只會向 server 續約一次，兩邊都成功。等待逾時時，CLI 報錯指出疑似有其他 speclink 程序長時間持鎖，不會無限期停住。
- 桌面 app 與 CLI 交錯使用同一個 server，不會互相把對方登出。

## 登出：`speclink auth logout`

- 金鑰圈有裝置登入的續約憑證時，CLI 先呼叫 server 撤銷這組裝置憑證。
- 金鑰圈有瀏覽器登入的續約憑證，而且登入服務列有撤銷端點時，CLI 向那個端點送出撤銷請求。登入服務沒有撤銷端點時不送任何撤銷請求，本機憑證照樣清除，exit code 0。
- 然後清除該 server 的所有本機憑證：金鑰圈的續約憑證、通行證快取、存取金鑰條目，以及憑證檔裡該 server 的條目。
- server 端的存取金鑰不會被撤銷。要撤銷它，到 `/account` 操作。見 [帳號](accounts.md)。
- 成功時 exit code 0，顯示已登出的 server 位址。該 server 沒有任何本機憑證時，非零 exit code 報未登入。
- server 不可達時，本機憑證仍被清除，exit code 0，stderr 警告 server 端的裝置憑證未撤銷。
- 登出後，桌面 app 對同一個 server 的下一次操作也回到未登入狀態。

## 連上之後

登入後，執行 `speclink update` 取得技能。遠端專案的技能由 Server 提供，版本跟著 Server；連到一台 Server，等於信任它提供的技能。細節見 [建立工作區與指令檔](init-workspace.md)。

接著用讀取動詞（例如 `speclink list`）確認連線正常，再試寫入動詞。規格未載試跑用的具體動詞清單。動詞在 remote 模式與本地模式的輸出同形。

**出處**：`remote-connection`、`remote-auth`、`server-device-auth`、`user-documentation`、`workspace-tools`、`host-kit`
