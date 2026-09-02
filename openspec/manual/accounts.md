---
title: 帳號、邀請、存取金鑰與 membership
section: Remote 模式
order: 430
keywords: [邀請, 登入, 存取金鑰, PAT, membership, reader, editor, 403]
sources: [server-identity, server-policy-write, user-documentation]
generated: 2026-09-02
---

# 帳號、邀請、存取金鑰與 membership

remote 模式的每個人都需要一個帳號、至少一個專案的成員資格（membership），以及一把讓 CLI 或桌面 app 連線的憑證。本頁依序說明這三樣東西怎麼來、怎麼失效。

## 邀請與建立帳號

邀請由管理員建立，受邀者開連結設定密碼就成為正式帳號。

- 管理員在 server 主機上用 invite 子命令建立邀請：填 email、顯示名、要加入的專案、可選的 admin 旗標、到期時限。子命令輸出一次性的邀請 URL。
- 管理員也可以在 [瀏覽器後台](web-console.md) 的使用者頁送出邀請，得到可複製的邀請連結。
- 同一個 email 已有啟用帳號，或已有未過期的邀請：拒絕重複建立。子命令以非零 exit code 結束並說明原因。
- 受邀者開啟 `/invite/<token>`：頁面顯示邀請摘要（不含任何祕密），受邀者設定密碼後提交。
- 提交成功：帳號啟用並帶上指派的成員資格，邀請耗用，登入工作階段建立。帶 admin 旗標的邀請導向 `/admin`，一般邀請導向 `/account`。
- 已用、過期、未知的邀請連結都得到同一個「邀請無效」，不建立登入工作階段。server 不區分原因。
- 登入工作階段建立失敗時，server 回可重試登入的錯誤，不會假裝已登入。

## 登入與登入工作階段

- 在 `/login` 用 email 與密碼登入。密碼在 server 只存雜湊。
- 登入失敗時，不論 email 存不存在，回應的狀態與訊息都相同。
- 登入後去哪，由 server 依序決定：
  1. 網址帶有效的裝置碼：先進裝置核准流程。
  2. 網址帶安全的返回路徑：只接受以 `/account`、`/activate` 或 `/admin` 開頭的站內路徑。
  3. 都沒有：回角色首頁。admin 到 `/admin`，一般成員到 `/account`。
- 一般成員用 `/admin` 當返回路徑：403。
- 返回路徑是外部網址：server 忽略它，改回角色首頁。
- 未登入時開受保護的頁：導向 `/login?returnTo=...`，只保留安全的站內路徑。
- 登出會撤銷 server 端的登入工作階段。之後用同一個 cookie 請求，回 401，畫面回到登入頁。

## 存取金鑰（PAT）

存取金鑰是你自己建立、給 CLI 或桌面 app 用的長效憑證。規格裡叫 PAT。

- 到 `/account`，按頁面的建立動作，面板出現後填名稱與到期日。
- 明文只在建立時顯示一次，附複製動作。重新載入後，清單只顯示開頭幾碼、名稱、到期、上次使用。沒有任何途徑再讀回明文。
- 撤銷即時生效。撤銷後再用這把金鑰呼叫，回 401。
- 直接在網址列開 `http://localhost:8080/account/tokens`：得到 HTTP 405 Method Not Allowed。那個路徑只接受表單送出。回到 `/account` 登入後，用 Personal Access Tokens 表單建立。

> [!CAUTION]
> 不要把存取金鑰放進 URL、`.speclink.yaml`、repo、文件範例或 shell history。把它貼進應用程式，或經 stdin 交給 CLI。

帳號頁還有這些區塊：

- 我的專案：你隸屬的每個專案，顯示名與角色。這個區塊唯讀。沒有任何隸屬時，區塊顯示「由管理員授予」的說明，不會消失。admin 看到的也是自己的隸屬，不是全部專案。
- 登入工作階段：唯讀清單，顯示建立與到期。不能逐一撤銷。
- 裝置憑證：桌面 app 或 CLI 裝置登入產生的憑證，可逐一撤銷。撤銷即時生效，其他裝置與存取金鑰不受影響。

## 成員資格與角色

- 角色有兩種：`editor` 與 `reader`。預設是 `editor`。
- 邀請建立的成員資格固定為 `editor`。
- `reader` 與 `editor` 都能讀工作流政策。只有 `editor` 能寫。`reader` 繞過介面直接呼叫寫入，得到 403，文件不變。
- 桌面 app 依角色停用寫入按鈕只是介面提示。server 才是最終防線。
- 建立 Project／Repo 不會授予成員資格。Admin 旗標也不是 scope 的通行證。
- 管理員在 `/admin/users` 對帳號授予或更新 `reader`／`editor`。之後桌面 app 的 scope 清單才顯示該 Project 及其 Repos。
- 管理員調整成員資格會記入稽核紀錄，含新的角色值。

桌面 app 顯示「此帳號目前沒有任何 Project／Repo membership」，但管理面已有 Project 與 Repo 時：

1. 管理員到 `/admin/users`。
2. 對桌面 app 實際登入的帳號授予該 Project 的 `reader` 或 `editor`。
3. 回到桌面 app，重新載入工作區選擇器。

> [!NOTE]
> 唯讀角色的名稱在規格裡有兩種寫法。角色模型與 `/admin/users` 的說明寫 `reader`；帳號頁的一個較新範例寫 viewer。本頁採用 `reader`。差異記在 [本手冊的來源](about.md)。

## 憑證檢查怎麼失敗

server 對每個 API 請求都重新查驗憑證，沒有快取：

- 憑證無效、過期、已撤銷，或帳號被停權：401 permission_denied。server 不區分是哪一種。
- 憑證有效，但你不是該專案的成員：403 permission_denied。這與 401 可區分。
- 停權或移除成員資格，在下一個請求就生效。
- 成功的請求會更新該存取金鑰的上次使用時間。

> [!NOTE]
> 已登入的非成員讀取專案資源得到的是 403，不是 404。404 只出現在專案代號根本不存在時。

## 密碼與憑證怎麼存

- 帳號、成員資格、邀請、存取金鑰、登入工作階段存在 server 自己的 identity 資料庫，與規格資料的儲存後端分開。
- 密碼、存取金鑰、邀請 token 與登入工作階段識別只以雜湊落庫，也不出現在 log。
- identity 資料庫的結構版本比 server 新，或不是本 server 建立的：server 拒絕啟動並印出原因，不寫入。

**出處**：`server-identity`、`server-policy-write`、`user-documentation`
