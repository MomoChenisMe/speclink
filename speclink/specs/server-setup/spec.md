# server-setup Specification

## Purpose

全新 server 的開箱流程：以一次性 bootstrap token 門禁（且僅在尚無 admin 時可用），依序完成建立第一位 Admin、顯示 Store 狀態、建立第一組 Project 與 Repo、顯示初始連線資訊四個節點，registry 持久化後由 binding 直接讀庫。本 capability 保證流程冪等可續作、token 在最後節點於同一交易邊界內耗用並入 audit，且 public URL 只來自部署組態、不由 setup 寫入。

## Requirements

### Requirement: registry 持久化且 binding 讀庫

Project/Repo registry SHALL 存於 server 自有資料庫（projects 與 repos 表），identity 層 SHALL 提供 registry 讀寫介面（列與查 project、列 repos、建 project、建 repo）；重複的 project key 或同 project 內重複的 repo key SHALL 拒絕建立。binding 裁決 SHALL 讀 registry：未註冊 project key 回 404 not_found、repo 標頭未註冊回 not_found、多 repo 缺標頭拒絕不代選、恰一 repo 綁定——錯誤分類與訊息 SHALL 與組態時代逐位元一致。schema 演進 SHALL 沿用既有守門：舊版 migrate 升級且既有資料完整保留、較新版本拒開。

#### Scenario: binding 語意在遷移後不變

- **WHEN** 以 registry 介面播種一個雙 repo project 後，分別以未註冊 key、缺 repo 標頭、正確標頭呼叫 /binding
- **THEN** 依序得到 404 not_found、多義拒絕、成功 binding——與組態播種時代的回應一致

#### Scenario: 重複 key 拒絕

- **WHEN** 對已存在的 project key 再次建立 project
- **THEN** 建立被拒絕且既有 project 不受影響

---

<!-- @trace
source: server-setup-registry
updated: 2026-07-14
code:
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/src/setup.rs
  - crates/speclink-server/src/web.rs
  - crates/speclink-server/tests/auth_device.rs
  - crates/speclink-server/tests/auth_pat.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/device_e2e.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/identity.rs
  - crates/speclink-server/tests/invite.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/refresh_rotation.rs
  - crates/speclink-server/tests/setup_flow.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-server/tests/web_device_sessions.rs
-->

---
### Requirement: bootstrap token 一次性且以無 admin 為條件

server 啟動時不存在任何 admin 使用者且無未過期 setup token 時，SHALL 生成高熵 bootstrap token：hash 落庫、帶到期（預設 24 小時）、明文僅印於 stdout 並附 /setup 指引，SHALL NOT 寫入 log 檔或組態。已存在 admin 的 server SHALL NOT 生成 token 且 /setup SHALL 回 404。setup 完成 SHALL 耗用 token；token 過期而 setup 未完成時，重啟 SHALL 生成新 token 並使舊 token 作廢。無效、過期或已耗用的 token 訪問 /setup SHALL 得到同一無效回應，SHALL NOT 區分原因。

#### Scenario: 首次啟動印 token 且完成後關門

- **WHEN** 以全新資料庫啟動 server，憑 stdout 的 token 完成 setup 流程後再次訪問 /setup
- **THEN** 首次啟動 stdout 含 token 與指引；完成後訪問回 404；重啟後不再印 token

#### Scenario: 已有 admin 不開 setup

- **WHEN** 對已完成 setup 的 server 重啟並訪問 /setup
- **THEN** stdout 無 token；/setup 回 404

---

<!-- @trace
source: server-setup-registry
updated: 2026-07-14
code:
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/src/setup.rs
  - crates/speclink-server/src/web.rs
  - crates/speclink-server/tests/auth_device.rs
  - crates/speclink-server/tests/auth_pat.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/device_e2e.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/identity.rs
  - crates/speclink-server/tests/invite.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/refresh_rotation.rs
  - crates/speclink-server/tests/setup_flow.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-server/tests/web_device_sessions.rs
-->

---
### Requirement: setup 流程完成開箱四要素

`/setup` SPA 與 `/api/speclink/v1/web/setup/*` SHALL 以 bootstrap token 門禁，流程 SHALL 涵蓋：建立第一位 Admin（email、顯示名、密碼，直接為 active 且帶 admin 旗標，不經邀請）、顯示 Store 狀態（儲存的種類、可用性檢查的結果、identity schema version；回應的 store 物件 SHALL NOT 含 `contractVersion`、`level` 與 `capabilities`）、建立第一組 Project 與 Repo（寫 registry）、顯示初始連線資訊（部署組態的 public URL 與所建 project／repo keys）。流程 SHALL 冪等可續作：token 未耗用前重入不重建已完成的節；全部 mutation SHALL 驗證同源並回相同 `{data}`／`{error}` browser JSON envelope。setup SHALL NOT 寫入 public URL，其唯一來源 SHALL 為部署組態。

完成最後節點 SHALL 在既有 setup 交易邊界內耗用 bootstrap token、記錄 audit 並建立第一位 Admin 的 Web session；成功回應 SHALL 帶 `destination: "/admin?welcome=1"` 與 `connection: {publicUrl, projectKey, repoKey}`。Session cookie SHALL 在回應送出前設定完成；若 session 建立失敗，Server SHALL 回可重試登入的 recovery error，SHALL NOT 把結果表示為已登入。

#### Scenario: 完成 setup 即可邀請與連線

- **WHEN** 完成 setup 建立 Admin 與第一組 Project／Repo 後，以 invite 子命令對該 project 邀請成員，成員接受邀請、建 PAT 並以 CLI 連線
- **THEN** setup 回應建立 session 並導向 `/admin?welcome=1` 顯示連線資訊；邀請建立成功；成員的 `/binding` 對該 project／repo 成功；CLI remote 動詞照常運作

#### Scenario: 中斷後憑同一 token 續作

- **WHEN** 完成第一位 Admin 建立後關閉瀏覽器，再憑同一 token 進入 `/setup`
- **THEN** SPA 顯示 Admin 節已完成且不重建，可繼續建立 Project／Repo 完成流程

#### Scenario: 重複完成不建立第二份資料

- **WHEN** setup completion 請求因 client 未收到回應而以相同 bootstrap token 重送
- **THEN** Server 不建立第二位 Admin 或第二組 Project／Repo；已耗用 token 回既有不可區分的無效 token 結果

#### Scenario: 跨 origin setup mutation 被拒絕

- **WHEN** 有效 bootstrap token 從不同 origin 提交任一 setup mutation
- **THEN** Server 回 403，不建立或修改 Admin、registry、session 與 audit 成功事件


<!-- @trace
source: server-on-port-examples
updated: 2026-11-16T23:24:33+08:00
-->

---
### Requirement: invite 子命令對 registry 查核

invite 子命令的 project 指派 SHALL 對 registry 查核：未註冊的 project key SHALL 以非零 exit code 拒絕並列出既有 project keys；邀請 URL 的基底 SHALL 為部署組態的 public url。

#### Scenario: 未註冊 project 拒絕邀請

- **WHEN** 對 registry 中不存在的 project key 執行 invite 子命令
- **THEN** 非零 exit code；stderr 列出既有 project keys；不建立邀請

<!-- @trace
source: server-setup-registry
updated: 2026-07-14
code:
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/src/setup.rs
  - crates/speclink-server/src/web.rs
  - crates/speclink-server/tests/auth_device.rs
  - crates/speclink-server/tests/auth_pat.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/device_e2e.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/identity.rs
  - crates/speclink-server/tests/invite.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/refresh_rotation.rs
  - crates/speclink-server/tests/setup_flow.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-server/tests/web_device_sessions.rs
-->