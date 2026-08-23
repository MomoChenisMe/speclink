## Why

Node SDK 的 dispatch 目前在 Rust 側寫死操作者身分：fs 模式抓 workspace 的 git identity，宿主 Store 模式一律無身分——JS 宿主接自家資料庫做多人系統時，所有蓋章（created_by、review／verify 章的 _by）永遠匿名，Host 面沒有把手可以標示「是誰在操作」。host-runtime 正典要求 identity 由 Host 解析注入，而 Node 綁定的 JS 宿主正是那個 Host——它需要一個合乎正典的注入點。

## What Changes

- `createEngine` 選項新增選填 `actor` 欄位（"Name <email>" 格式字串）：兩種儲存形式（fs 與宿主 Store 物件）皆於建構期收下，存於 engine 實例——一個實例綁一個身分，多人宿主以每請求（或每身分）一個實例表達
- dispatch 的 actor 解析改為三層：建構期 actor 有給值（trim 後非空）一律優先；fs 模式未給回退 git identity（現行為）；宿主 Store 模式未給維持無章（現行為不變）
- `crates/speclink-node/index.js` 與 `index.d.ts` 的 CreateEngineOptions 契約面同步新增 actor 欄位與說明
- node-sdk spec 新增「建構期 actor 注入」requirement（delta）
- 文件對齊（createEngine 契約段歸本 change）：docs/sdk-node.zh-TW.md 與 docs/sdk-node.md 的 createEngine 段補 actor 選項的語意與多人宿主用法

## Non-Goals

- dispatch 參數帶 actor——違反 host-runtime「ExecutionContext 由 Host 解析一次、Command 輸入不含 actor」正典，等於身分偽造旁路
- 認證與權限判定——誰可以宣稱哪個身分是 JS 宿主（Host）的職責，SDK 只收結果
- 獨立 `@speclink/host` 套件（藍圖三件套是過期構想，單一 actor 選項不成套件）
- npm 發布通路（平行 change engine-npm-publish）
- CLI、Desktop、server 的身分行為——本 change 只動 Node 綁定

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `node-sdk`: 新增「建構期 actor 注入」requirement——createEngine 收選填 actor、優先序與兩種儲存形式的回退語意。近鄰 `host-runtime` 定的是 Host 邊界原則（identity 由 Host 注入、command 不帶 actor），本 delta 是該原則在 Node SDK 契約面的落地，不改 host-runtime 本身。

## Impact

- Affected specs: `node-sdk`（modified）
- Affected code:
  - New: crates/speclink-node/__test__/actor.spec.ts
  - Modified: crates/speclink-node/src/lib.rs、crates/speclink-node/index.js、crates/speclink-node/index.d.ts、docs/sdk-node.zh-TW.md、docs/sdk-node.md
  - Removed: （無）
