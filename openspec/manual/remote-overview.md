---
title: remote 模式總覽
section: Remote 模式
order: 400
keywords: [remote, server, link, 參考實作, project-scoped URL, .speclink.yaml]
sources: [user-documentation, remote-connection]
generated: 2026-09-02
---
# remote 模式總覽

Speclink 有兩種工作方式。本地模式把正典放在你的 repo 裡，remote 模式把正典放在團隊共用的 server 上。這一頁說明兩者的差別、remote 模式怎麼被啟用，以及整條 remote 旅程的順序。

## 本地與 remote 的差別

**本地模式**的產物沿用 OpenSpec 的目錄結構：`specs/<capability>/spec.md`、`changes/<名稱>/`、`changes/archive/` 與 `config.yaml`。內容是純 Markdown 與 YAML，不經 Speclink 也讀得到、改得動，每次變動都呈現在 git diff 裡。Speclink 在這個結構上多加兩樣東西：`discussions/` 目錄，以及各變更目錄裡的 `.openspec.yaml`。細節見 [認識資料：變更、討論與規格](data-layout.md)。

**remote 模式**的正典在 server 的儲存後端。本機沒有 `openspec/` 目錄，只有一份唯讀的投影，見 [remote 模式的規格投影](remote-context.md)。上述目錄相容性不適用 remote 模式。

## 官方 server 是參考實作

`speclink-server` 是官方的參考實作，用途是開箱即用與試用遠端功能。它不是 remote 模式的唯一路徑。remote 模式由兩份公開契約定義：`openspec/specs/` 裡的 `host-runtime` 與 `client-protocol`。你可以用 Speclink 引擎自行實作 server 端，接上自己的認證、資料庫與權限模型。CLI 與桌面 app 對自建 server 同樣可用。

本地與 remote 的能力對照表以官方參考 server 為量測對象，同樣的欄位適用於自建 server。

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
- remote 區段與 `openspec/` 目錄並存：remote 模式生效，並在錯誤輸出印一行並存警告。
- 環境變數 SPECLINK_STORE_URL 存在時，覆寫區段的 url。
- remote 區段存在，但區段的 url 與環境變數都沒有：指令失敗，並同時提示 remote.url 欄位與 SPECLINK_STORE_URL 兩種設定方式。它不會悄悄改用本地模式。

> [!WARNING]
> `.speclink.yaml` 存在但無法解析時，所有依賴模式判定的指令都失敗。錯誤訊息指出檔案與解析原因。指令不會當成沒有 remote 區段而讀本地 `openspec/`，也不會發出任何遠端請求。

專案根還留著舊的 `.speclink.remote.yaml` 時，CLI 只印一行遷移警告，提示你把 url 與 repo 搬進 `.speclink.yaml` 的 remote 區段並刪掉舊檔。舊檔不參與模式判定，內容不會被讀。

## 連接指令的角色

| 指令 | 作用 |
| --- | --- |
| `speclink init --store remote --url <url> [--repo <name>]` | 在空資料夾建立 remote 工作區：寫入 `.speclink.yaml` 的 tools 與 remote 區段、生成技能與 `.gitignore`。不建立 `openspec/`，也不建立任何指令檔 |
| `speclink link <url> [--repo <name>]` | 只寫入或更新 remote 區段，保留 tools 與其他欄位 |
| `speclink unlink` | 移除 remote 區段，保留其他欄位。之後指令回到本地模式 |
| `speclink auth login` | 取得憑證 |

init 與 link 當下已有可用憑證時，CLI 立即向 server 查驗 repo 是否屬於這個 Project 並回報結果。沒有憑證時提示你執行 `speclink auth login`。操作細節見 [CLI 連接 remote](cli-remote.md)。

remote 模式下每個動詞都自動帶上 remote 區段的 repo 名。你在 repo 名為 frontend 的資料夾操作一個歸屬 backend 的變更時，指令失敗，錯誤訊息同時列出 backend 與 frontend 兩個名稱與改正指引。

## remote 旅程的順序

從零到可操作的 remote 桌面與 remote CLI，依序是：

1. 啟動 server。最短路徑是 `npx @speclink/server` 一行啟動。見 [啟動 server](server-start.md)。
2. 開箱：用 server 印出的 `/setup?token=...` 建立第一位 Admin 與第一組 Project／Repo。見 [開箱：第一位 Admin 與第一組 Project／Repo](server-setup.md)。
3. 在 `/admin/users` 為實際登入的帳號授予 Project membership。第一位 Admin 為自己授予所建 Project 的 membership 是必經步驟，不會自動發生。見 [帳號、邀請、存取金鑰與 membership](accounts.md)。
4. 在 `/account` 建立存取金鑰（若要走 PAT 路徑）。
5. 桌面 app 以裝置登入連線，或以存取金鑰為備援；選擇 Project／Repo；決定開 spec-only 還是 checkout 工作區。見 [桌面連線 server 與 remote 工作區](desktop-remote.md)。
6. CLI 用 `speclink link` 與 `speclink auth login` 接上，跑一次讀寫。見 [CLI 連接 remote](cli-remote.md)。
7. 了解一般重啟、失聯與完全重置的行為。見 [失聯與恢復](remote-offline.md)。

> [!CAUTION]
> 不要在產品 repo 的根目錄寫入測試用的連接設定。找一個獨立的測試資料夾做 smoke test。

已經有本地工作區、想整個搬到 server 的人，走 [本地工作區遷移到 remote](migrate.md)。

**出處**：`user-documentation`、`remote-connection`
