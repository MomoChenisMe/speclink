## Why

內嵌宿主給 Agent 的程式碼常是唯讀的副本：讀檔與搜尋的工具都有，但沒有執行 git 的工具，副本也只有一個 commit。`skills.render` 的 `checkout` 只有兩個值，兩個都不合這種宿主：`checkout: false` 讓模型連程式碼都不讀（實測 discuss 技能三次回答「沒有程式碼 checkout，因此略過程式碼搜尋」），省略 `checkout` 則留下 trace 技能裡執行 git 的步驟。宿主目前只能靠自己的 system prompt 蓋過技能文字，用語和技能對不上。

目標使用者：把 `@speclink/engine` 內嵌進自己後端、並把 speclink 技能交給自己的 Agent 的開發者。使用情境：宿主啟動時用 `skills.render` 產生技能文字的階段；受益的是在宿主的對話裡請 Agent 討論或溯源的使用者。

來源：討論 embedded-host-needs-8 的 cut 2（G3）。相關的正式規格掃描結果：node-sdk（「渲染 API 的宿主限制選項」「渲染 API 拒絕不認得的選項鍵」）、trace-skill（溯源技能的 git 步驟）、workspace-tools（CLI 產生技能檔，不受影響）、host-kit（入口回傳的技能清單，不帶宿主限制，不受影響）。

## What Changes

- `skills.render` 的 options 新增選填的 `gitHistory`（布林）：使用這份技能的環境能不能使用 git 與它的歷史。省略時跟著 `checkout`（`checkout` 省略或 true 時是 true，`checkout: false` 時是 false）。
- `gitHistory: false` 而 `checkout` 不是 false 時，`## Harness limits` 多一個項目：可以讀與搜尋程式碼，但沒有 git 歷史、不能執行 git；跳過執行 git 的步驟；步驟寫了其他做法時改用它；在回答裡說明跳過了哪些步驟。
- `checkout: false` 的項目與行為不變（它已經涵蓋 git）；`checkout: false` 加 `gitHistory: true` 拋錯（兩個選項互相矛盾）；`gitHistory` 不是布林值、或鍵在而值是 undefined 時拋錯，和 `checkout` 的規則相同。
- `gitHistory` 加入合法的選項鍵清單；不認得的鍵的錯誤訊息會列出它。
- 技能原稿的內文不改：`## Harness limits` 之後的內容和不帶這個選項時相同。
- 這是推翻討論 embedded-host-needs-7「檔案搜尋與 git 歷史合成 checkout 一個開關」的決定：那時沒有「有程式碼、沒有 git」的宿主，現在有了。
- 不新增或變更任何 CLI 子指令、旗標、stdin 行為與 exit code；不涉及 speclink/config.yaml 或 .speclink.yaml 的欄位。

### 相容性影響

- 不帶 `gitHistory` 的呼叫，輸出與現在逐位元相同；`gitHistory: true` 加上 `checkout` 省略或 true 也相同。
- CLI 產生的技能檔（claude、codex、copilot、pi、cursor 與自訂工具）不受影響：CLI 不帶宿主限制。技能的 golden 快照與內嵌資產版號不變。
- 型別定義 `RenderOptions` 多一個選填欄位，既有的 TypeScript 呼叫端不用改。
- 既有使用者不需要遷移；想要新行為的宿主在自己的 render 呼叫加上 `gitHistory: false`，並拿掉 system prompt 裡替代的規則。

### 三種形態

- 本機 repo：不適用。本機的技能檔由 CLI 產生，CLI 一定有 checkout 與 git，不帶宿主限制。
- 連到遠端的 checkout：不適用。遠端專案的技能由 Server 的入口提供，入口不帶宿主限制，使用技能的一端是有 git 的 checkout。
- 沒有本機規格檔的內嵌宿主：可用。宿主用 `skills.render(name, { …, gitHistory: false })` 產生給自己 Agent 的技能文字。

### 受影響的技能

每一支技能、每一個 target 都能帶這個選項。實際有差別的是有 git 步驟的技能；宿主常用的六支（discuss、trace、propose、ingest、analyze、config）裡只有 trace 有要執行的 git 步驟：前提那一句、evidence 是 null 時查 git log 的步驟（已有後備：改用討論與提案的內容）、沒有對到能力時的 git log -S 與 git blame（沒有替代，跳過並只根據現在的程式碼回答）。

## Non-Goals (optional)

設計文件會建立，範圍排除寫在 design 的 Goals / Non-Goals。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `node-sdk`: 「渲染 API 的宿主限制選項」加入 gitHistory 選項、它的預設、新的 Harness limits 項目與不合法的組合；「渲染 API 拒絕不認得的選項鍵」的合法鍵清單加入 gitHistory。

## Impact

- Affected specs: node-sdk
- Affected crates: speclink-core（宿主限制的型別與前言的文字）、speclink-node（選項的解析、型別定義與測試）。speclink-cli、speclink-host、speclink-server、apps/desktop 沒有程式改動。
- Affected code:
  - Modified:
    - crates/engine/speclink-core/src/workspace/skills.rs
    - crates/adapters/speclink-node/src/render.rs
    - crates/adapters/speclink-node/index.js
    - crates/adapters/speclink-node/index.d.ts
    - crates/adapters/speclink-node/__test__/render.spec.ts
    - docs/sdk-node.md
    - docs/sdk-node.zh-TW.md
    - docs/product-status.md
    - docs/product-status.zh-TW.md
  - New: (none)
  - Removed: (none)
