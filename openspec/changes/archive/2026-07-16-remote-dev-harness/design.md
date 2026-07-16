## Context

remote 模式手動測試缺 pnpm dev 等價物：server 要手寫 config YAML＋cargo run、desktop 另開終端 tauri dev、web /setup 靠人腦記流程。既有零件已齊：server 以 --config＋--addr 啟動、首跑於 stdout 印一次性 /setup?token 連結（ensure_setup_token）；desktop 有 tauri dev 一鍵；deploy compose 已確立「組態 YAML 不做環境變數展開，由編排層於啟動時插值」的 fail-closed 決策與 SPECLINK_PORT／SPECLINK_PUBLIC_URL 命名。缺的只是 repo root 的編排層。本刀源自討論 remote-dev-harness；desktop 現階段連不上 server（remote UI 是 Phase 3），本 harness 是 Phase 3 開發迴圈的前置基建，當下可測 server web 面（/setup、admin、invite）與 CLI remote 模式。

## Goals / Non-Goals

**Goals:**

- npm run dev 一個指令同起 server（env 驅動設定）與 desktop（tauri dev），終端可見 /setup 連結。
- .env.example 完整列出可調鍵與預設，複製為 .env 即可切 sqlite／serverfs／postgres／memory。
- .dev/ 持久化跨重啟；npm run dev:reset 顯式清空。
- 兩份正典文件反映此開發路徑。

**Non-Goals:**

- 不動 server 產品碼：不加 --dev 旗標、不讓 server 原生讀 env 或 .env（第二設定來源＋優先序歧義，違反 config fail-closed 單一來源；deploy compose 註解已釘此決策）。
- 不動 phase2-e2e-chain：e2e 本來就是真實 binary＋tempdir 的 cargo 測試，與本 harness（手動測試）分屬兩事。
- 不提供 docker 版開發迴圈、不改 deploy/ 下任何檔案。
- 不做 server-only／desktop-only 的子指令拆分——需求出現再加。

## Decisions

### 決策 1：env 插值在編排層，生成 .dev/config.yaml

scripts/dev.mjs 讀 repo root 的 .env（若存在），與 process env 合併後插值生成 .dev/config.yaml，再以 --config 啟動 server。優先序：process env 蓋過 .env 檔（dotenv 慣例——臨時 SPECLINK_STORE_DRIVER=memory npm run dev 可一次性覆寫）。.dev/config.yaml 每次啟動整檔重寫，檔頭註明由 npm run dev 生成、手改無效。替代案「server 原生吃 env」已在討論中排除（見 Non-Goals）。

### 決策 2：env 鍵與映射規則

| 鍵 | 預設 | 映射 |
| --- | --- | --- |
| SPECLINK_STORE_DRIVER | sqlite | store.driver（合法值：sqlite、serverfs、postgres、memory） |
| SPECLINK_STORE_PATH | .dev/store.db（sqlite）／.dev/store（serverfs） | store.path（sqlite/serverfs 用；postgres/memory 忽略） |
| SPECLINK_POSTGRES_URL | 無 | store.url（driver=postgres 時必填，缺值即以可讀錯誤退出、不啟動任何 process——fail-closed） |
| SPECLINK_IDENTITY_PATH | .dev/identity.db | identity.path（identity 固定 driver: sqlite——memory 重啟丟帳號/PAT，違反持久化體感） |
| SPECLINK_PORT | 8080 | server 的 --addr 127.0.0.1:{port} |
| SPECLINK_PUBLIC_URL | http://localhost:{port} | public_url |

不合法的 driver 值同樣於生成前以可讀錯誤退出（列出四個合法值），不留給 server 端報錯——script 是第一道邊界。SPECLINK_POSTGRES_PASSWORD 為 server 既有的原生機制（URL 可不含密碼、由該 env 補），script 不處理、僅在 .env.example 註明並直通給 child process。

### 決策 3：零依賴 Node script

root package.json 目前無任何 dependency，維持之：.env 解析（逐行 KEY=VALUE、跳過註解與空行、不支援多行/展開）與 YAML 生成（欄位固定的字串模板）皆手寫，不引入 dotenv／concurrently。理由：解析需求是六個已知鍵，引依賴的維護面大於三十行手寫碼；concurrently 的價值（前綴著色）用 child stdio inherit＋自加前綴即可替代。替代案「shell script」因 Windows（跨機器開發既有事實）排除。

### 決策 4：process 生命週期

dev.mjs spawn 兩個 child：cargo run -p speclink-server -- --config .dev/config.yaml --addr 127.0.0.1:{port}，與 desktop 的 tauri dev（經 npm workspace script 呼叫；Windows 上 spawn npm 需 shell 相容處理）。stdio 直通終端（server 的 /setup 連結行必須原樣可見）。SIGINT/SIGTERM 時同殺兩個 child；任一 child 先退出時連帶結束另一個並以其 exit code 退出——避免半死狀態。前置：`tauri.conf.json` 無 devUrl、tauri dev 直接載入靜態 `apps/desktop/dist`（gitignored），故 spawn 前若 `dist/index.html` 不存在需先跑一次 `npm run build -w apps/desktop`——否則全新 checkout 開窗失敗（實作時發現，非原討論假設的「tauri dev 已一鍵」）。

### 決策 5：dev:reset 的邊界

npm run dev:reset（dev.mjs 的 --reset 模式）只遞迴刪除 .dev/ 目錄；不碰 .env、不碰 deploy/、對不存在的 .dev/ 靜默成功（冪等）。postgres driver 的資料在外部資料庫，reset 不涉及——.env.example 於 postgres 段註明「重置 postgres 資料請自行 drop/recreate database」。

### 決策 6：正典文件落點

架構文件 §13.4「Server 與 Desktop 的開箱流程」段後補一小段「本地開發啟動」：native 直跑（cargo run 或 release binary）＋設定檔、同一條 /setup 流程、與 docker 部署形態的關係一句話；措辭用「本地開發啟動」而非「dev server」，避開同節「若只提供流程範例應命名為 example/dev server」的定位條款——speclink-server 仍是同一顆 production-lite server，只是啟動方式不同。roadmap §4.2 刀組表記入本刀，定位為 Phase 3 前置基建（排 phase2-e2e-chain 之後、desktop-workspace-session 之前）。

## Implementation Contract

- 觀測面：npm run dev 於全新 checkout（無 .env、無 .dev/）直接可用——全預設 sqlite 落 .dev/，終端出現 /setup?token=… 連結；Ctrl+C 後無殘留 process。
- scripts/dev.mjs 的 env→config 生成邏輯以純函式暴露，scripts/dev.test.mjs 以 node --test 覆蓋：四種 driver 的 YAML 輸出、預設值、process env 蓋 .env、postgres 缺 URL 報錯、非法 driver 報錯。
- root package.json 的 test 入口涵蓋 scripts 測試（併入 test:all 鏈）。
- .gitignore 新增 .dev/ 與 .env；.env.example 與決策 2 的表逐鍵一致。
- 驗證動線（手動）：npm run dev → 瀏覽器走 /setup 建 Admin/Project/Repo → npm run dev:reset → 再 npm run dev 應回到全新 setup token。
