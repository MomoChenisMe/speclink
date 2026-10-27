## Why

目標使用者：透過 AI 代理跑 SDD 的開發者（走 discuss → propose/ingest → apply 流程者）。使用情境：ingest 階段把討論結論折入既有變更，以及跨 session 續作既有變更。

問題有二。其一，`discuss link` 在鑄造變更側 from_discussion 鏈的同時，就把討論標記 `status: promoted`（進看板「已轉出變更的討論」組），但內容折入靠後續 ingest；中途停手即留下「假完成」——變更帶 from_discussion、proposal 仍是舊的、看板卻顯示已轉出。此坑對 link 特別嚴重：link 併的是既有變更，其舊 proposal 看似完整，過期是隱形的。其二，`ingest` 技能從不讀被連結討論的結論（只靠對話脈絡），跨 session 續作時無從得知要折什麼，與 propose 的 `--from-discussion` 播種不對稱。

## What Changes

- **link 停止預先標記**：`discuss link` 僅鑄造變更側 from_discussion 鏈，SHALL NOT 再標記討論 promoted 或寫 promoted_to；討論停在 concluded／open，直到內容經 seal 落地。link 時機不變（維持中斷仍自動封存的安全網）——只把「標記已轉出」移出。
- **新增 `discuss seal` 動詞**：`speclink discuss seal <slug> <change>` 標記討論 `status: promoted` 並累加 `promoted_to`，冪等；由 ingest 於內容落地完成時呼叫。子指令吃兩個位置參數（討論 slug 與變更名），旗標僅 `--json`／`--no-color`，不吃 stdin；成功 exit 0、stdout 單行訊息，守衛失敗（討論不存在／已封存、變更不存在、變更 from_discussion 未含該 slug）非零 exit＋stderr。
- **`show <change> --json` 暴露 fromDiscussions**（camelCase 陣列），供 ingest 技能發現連結討論、不再硬讀 .openspec.yaml。
- **discussion-aware ingest**：ingest 技能於目標變更帶 from_discussion 時，經 `discuss show <slug>` 讀其結論為一等來源、併入既有脈絡／plan（不取代），並於結尾呼叫 `discuss seal`。

## Non-Goals

- **不改 `discuss promote` 與 `new change --from-discussion`**：兩者維持建立時即標 promoted。新變更的 change 卡本就是「提案中／TBD」的可見未完成訊號，不構成 link 那種隱形假完成；且 promote 為 desktop「轉為變更」按鈕與 Node SDK 共用，不動以免非預期漣漪。propose 技能因此不需呼叫 seal（promote 已標）。
- **re-conclude stale 偵測**（對已 promoted 討論再 conclude 時標記其連結變更待 re-ingest）：link-seal-timing 結論的第三支柱，本刀不做，另刀處理。
- 不引入 hash 指紋或 per-load 掃描（結論已否決）。
- 不動 desktop 看板的視覺呈現（徽章等）。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `discussion-docs`: `discuss link` 不再預先標記 promoted；新增 `discuss seal` 動詞承接「標記已轉出」（服務 ingest 路徑）；from_discussion 鏈可經 `show --json` 觀察；ingest 技能指示改為 discussion-aware ＋ 結尾 seal。

## Impact

- 影響 crate：speclink-core（discuss.rs 的 link 移除 mark_promoted、新增 seal）、speclink-cli（discuss seal 子指令接線、show --json 增欄位）。
- 相容性影響：
  - **BREAKING** `discuss link`：討論側 frontmatter 不再於 link 時變動（status／promoted_to）；斷言「link 即 promoted」的既有測試 SHALL 更新。
  - `discuss promote`、`new change --from-discussion`：行為不變（仍即標 promoted）。
  - `show <change> --json`：純增 fromDiscussions 欄位，既有欄位（created／deltaSpecs／design／name／proposal／schema／tasks）逐位元不變。
  - 內嵌技能三處同步（core assets、repo 技能實例、render golden）：ingest 技能文字變更後，須於乾淨樹重生 golden 並審視 diff。
- 影響技能與工具：ingest 技能（claude／codex 兩套注入）。
- 影響程式碼：
  - Modified: crates/speclink-core/src/discuss.rs, crates/speclink-cli/src/commands.rs, crates/speclink-cli/src/main.rs, crates/speclink-core/assets/skills/ingest.md, crates/speclink-core/tests/golden, .claude/skills, .agents/skills
