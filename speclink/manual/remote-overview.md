---
title: remote 模式總覽
section: Remote 模式
order: 400
keywords: [remote, server, link, 示範與試用, 內嵌, 宿主, 登入方式, project-scoped URL, .speclink.yaml]
sources: ["user-documentation#使用者文件載明本地產物的純文字可攜性", "user-documentation#官方 server 定位為示範與試用", "user-documentation#Remote Getting Started 提供可重複的完整操作路徑", "user-documentation#遠端專案的登入與技能來源有說明", remote-connection, "remote-auth#登入方式的探索與選擇"]
generated: 2026-11-19T16:40:41+08:00
---
# remote 模式總覽

Speclink 有兩種工作方式。本地模式把正式規格放在你的 repo 裡，remote 模式把正式規格放在團隊共用的 server 上。這一頁說明兩者的差別、remote 模式怎麼被啟用，以及整條 remote 旅程的順序。

## 本地與 remote 的差別

**本地模式**的產物放在規格資料夾（預設 `speclink/`）底下：`specs/<capability>/spec.md`、`changes/<名稱>/`、`changes/archive/` 與 `config.yaml`。內容是純 Markdown 與 YAML，不經 Speclink 也讀得到、改得動，每次變動都呈現在 git diff 裡。Speclink 另外多加兩樣東西：`discussions/` 目錄，以及各變更目錄裡的 `.meta.yaml`。細節見 [認識資料：變更、討論與規格](data-layout.md)。

**遠端專案**（remote 模式）的正式規格由 Server 持有，checkout 裡沒有規格檔，也沒有 `speclink/` 目錄。本機另有一份唯讀的投影，見 [remote 模式的規格投影](remote-context.md)。上述的純文字可攜性不適用遠端專案。

## 官方 server 用於示範與試用

官方 server 是示範與試用用的。它讓你不寫程式就能試遠端專案。正式使用請把引擎內嵌進自己的系統。

這裡的官方 server 指 `speclink-server`。把 Speclink 引擎內嵌進去的那套系統，叫做宿主。官方 server 與內嵌的宿主用同一套宿主套件與同一份儲存契約組成，CLI 與桌面 app 對兩者同樣可用。

本機與遠端的能力對照表，遠端一欄以官方的示範用 server 為量測對象。內嵌的宿主行為相同，差異列在已知缺口。

## 三種網址

remote 模式會碰到三種網址，用途不同：

| 網址 | 用途 | 例子 |
| --- | --- | --- |
| Server base URL | server 本體的位置 | `http://localhost:8080` |
| 瀏覽器帳號與管理頁 URL | 人在瀏覽器裡操作的頁面 | `/setup?token=...`、`/account`、`/admin/users` |
| project-scoped API URL | CLI 與桌面 app 連接某個 Project 用的網址 | `http://localhost:8080/api/speclink/v1/projects/demo` |

CLI 的 `speclink link` 要給 project-scoped API URL，不是 base URL。

## CLI 怎麼決定用哪種模式

CLI 看 `.speclink.yaml` 裡有沒有 remote 區段。

- 有 remote 區段：remote 模式。區段的 url 是 project-scoped API URL，repo 是這個 repo 在 Project 內的註冊名。
- 沒有 remote 區段：本地模式。
- remote 區段與 `speclink/` 目錄並存：remote 模式生效，並在錯誤輸出印一行並存警告。
- 環境變數 SPECLINK_STORE_URL 存在時，覆寫區段的 url。
- remote 區段存在，但區段的 url 與環境變數都沒有：指令失敗，並同時提示 remote.url 欄位與 SPECLINK_STORE_URL 兩種設定方式。它不會悄悄改用本地模式。

> [!WARNING]
> `.speclink.yaml` 存在但無法解析時，所有依賴模式判定的指令都失敗。錯誤訊息指出檔案與解析原因。指令不會當成沒有 remote 區段而讀本地 `speclink/`，也不會發出任何遠端請求。

專案根還留著舊的 `.speclink.remote.yaml` 時，CLI 只印一行遷移警告，提示你把 url 與 repo 搬進 `.speclink.yaml` 的 remote 區段並刪掉舊檔。舊檔不參與模式判定，內容不會被讀。

## 連接指令的角色

| 指令 | 作用 |
| --- | --- |
| `speclink init --store remote --url <url> [--repo <name>]` | 在空資料夾建立 remote 工作區：寫入 `.speclink.yaml` 的 tools 與 remote 區段、生成技能與 `.gitignore`。不建立 `speclink/`，也不建立任何指令檔 |
| `speclink link <url> [--repo <name>]` | 只寫入或更新 remote 區段，保留 tools 與其他欄位 |
| `speclink unlink` | 移除 remote 區段，保留其他欄位。之後指令回到本地模式 |
| `speclink auth login` | 取得憑證 |

init 與 link 當下已有可用憑證時，CLI 立即向 server 查驗 repo 是否屬於這個 Project 並回報結果。沒有憑證時提示你執行 `speclink auth login`。操作細節見 [CLI 連接 remote](cli-remote.md)。

登入有三種方式：瀏覽器登入、裝置登入、存取金鑰。Server 宣告了自己支援哪幾種時，CLI 與桌面 app 依這個順序選第一個；Server 沒有宣告時走裝置登入。連 HTTPS 的 Server 時，信不信任它的 HTTPS 憑證由作業系統的信任庫決定，組織自己的 CA 裝進信任庫就能連，不必另外設定。兩件事的細節都在 [CLI 連接 remote](cli-remote.md)。

remote 模式下每個動詞都自動帶上 remote 區段的 repo 名。你在 repo 名為 frontend 的資料夾操作一個歸屬 backend 的變更時，指令失敗，錯誤訊息同時列出 backend 與 frontend 兩個名稱與改正指引。

## remote 旅程的順序

從零到可操作的 remote 桌面與 remote CLI，依序是：

1. 啟動 server。最短路徑是 `npx @speclink/server` 一行啟動。見 [啟動 server](server-start.md)。
2. 開箱：用 server 印出的 `/setup?token=...` 建立第一位 Admin 與第一組 Project／Repo。見 [開箱：第一位 Admin 與第一組 Project／Repo](server-setup.md)。
3. 在 `/admin/users` 為實際登入的帳號授予 Project membership。第一位 Admin 為自己授予所建 Project 的 membership 是必經步驟，不會自動發生。見 [帳號、邀請、存取金鑰與 membership](accounts.md)。
4. 在 `/account` 建立存取金鑰（若要走 PAT 路徑）。
5. 桌面 app 以裝置登入連線，或以存取金鑰為備援；選擇 Project／Repo；決定開 spec-only 還是 checkout 工作區。見 [桌面連線 server 與 remote 專案](desktop-remote.md)。
6. CLI 用 `speclink link` 與 `speclink auth login` 接上，執行 `speclink update` 取得 Server 提供的技能，再跑一次讀寫。見 [CLI 連接 remote](cli-remote.md)。
7. 了解一般重啟、失聯與完全重置的行為。見 [失聯與恢復](remote-offline.md)。

> [!CAUTION]
> 不要在產品 repo 的根目錄寫入測試用的連接設定。找一個獨立的測試資料夾做 smoke test。

已經有本地工作區、想整個搬到 server 的人，走 [本地工作區遷移到 remote](migrate.md)。

**出處**：`user-documentation`、`remote-connection`、`remote-auth`
