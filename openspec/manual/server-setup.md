---
title: 開箱：第一位 Admin 與第一組 Project／Repo
section: Remote 模式
order: 420
keywords: [setup, bootstrap token, Admin, Project, Repo, membership]
sources: [server-setup, user-documentation]
generated: 2026-09-02
---

# 開箱：第一位 Admin 與第一組 Project／Repo

全新的 server 只有一次開箱流程。入口是首次啟動時印在畫面上的 setup 連結。開箱完成後，`/setup` 就關門了。

## 拿到 setup token

- server 啟動時，若還沒有任何 admin，也沒有未過期的 setup token，它會生成一個 token。
- token 明文只印在 stdout，並附 `/setup` 指引。它不寫進 log 檔，也不寫進設定檔。
- token 預設 24 小時到期。
- 已有 admin 的 server 不生成 token，`/setup` 回 404。
- token 過期而開箱還沒完成：重啟 server 會生成新 token，舊 token 作廢。
- 無效、過期、已用的 token 訪問 `/setup` 都得到同一個無效回應。server 不告訴你是哪一種原因。

取 token 的方式依啟動方式而定，見 [啟動 server](server-start.md)。

## 四個節點

開箱流程依序有四個節點：

1. 建立第一位 Admin。填 email、顯示名、密碼。這個帳號直接啟用並帶 admin 旗標，不經邀請。
2. 看儲存後端狀態。頁面顯示驅動與能力、健康檢查結果、資料結構版本。
3. 建立第一組 Project 與 Repo。填專案代號、儲存庫代號。
4. 看初始連線資訊。頁面顯示部署設定的 public URL，以及剛建立的專案代號與儲存庫代號。

流程的行為：

- 可以中斷再續作。token 還沒耗用前，憑同一個 token 回到 `/setup`，已完成的節顯示完成，不會重建。
- 完成最後一個節點時，server 耗用 token、寫一筆稽核紀錄、建立第一位 Admin 的登入工作階段，然後導向 `/admin?welcome=1` 顯示連線資訊。
- 若瀏覽器沒收到回應而重送完成請求：server 不會建立第二位 Admin 或第二組 Project／Repo。已耗用的 token 回無效。
- 登入工作階段建立失敗時，server 回一個可重試登入的錯誤。它不會假裝你已登入。
- public URL 只來自部署設定。setup 不寫入 public URL。
- 從不同來源網站送出的 setup 動作一律 403，不建立任何資料。

## 開箱後的必經一步：給自己 membership

> [!WARNING]
> 建立 Project／Repo 不會授予成員資格（membership）。Admin 身分也不會繞過成員資格。

第一位 Admin 必須做這件事，桌面 app 才看得到專案：

1. 到 `/admin/users`。
2. 對自己的帳號授予所建 Project 的 `editor` 成員資格。
3. 回到桌面 app，重新載入 scope 清單。

規格明示這是必經步驟，不是只有邀請他人時才需要。細節見 [帳號](accounts.md)。

## registry 的規則

- 專案代號重複，或同一專案內儲存庫代號重複：拒絕建立，既有資料不受影響。
- 專案代號與儲存庫代號建立後不可變更。顯示名可以改。
- CLI 連接時用了未註冊的專案代號：得到 404 not_found。
- 專案有多個 Repo 而連接時沒指定：拒絕，不代選。

## 用 invite 子命令邀請成員

- 在 server 主機上用 invite 子命令建立邀請。
- 對未註冊的專案代號邀請：非零 exit code，stderr 列出既有的專案代號，不建立邀請。
- 邀請 URL 的基底是部署設定的 public URL。

邀請的完整行為見 [帳號](accounts.md)。管理員也可以在 [瀏覽器後台](web-console.md) 的使用者頁送出邀請。

**出處**：`server-setup`、`user-documentation`
