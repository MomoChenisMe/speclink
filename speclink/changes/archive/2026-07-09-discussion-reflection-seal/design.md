## Context

討論 `link-seal-timing` 定案：看板「已轉出」的「假完成」根因，是連結動詞在鑄鏈的同時就宣告討論「已被反映」（翻 `status: promoted`），但內容折入靠後續動詞；中途停手即留下不誠實狀態。實作偵察進一步發現風險與消費者面不對稱：`core::discuss::link` 只有 CLI 用，而 `core::discuss::promote` 為 desktop「轉為變更」按鈕（apps/desktop/src-tauri/src/lib.rs 委派至 apps/desktop/core/src/discussions.rs）與 Node SDK 共用。並列問題：ingest 技能從不讀被連結討論的結論（只靠對話脈絡），與 propose 的 `--from-discussion` 播種不對稱、跨 session 續作失效。

現況佐證：`crates/speclink-core/src/discuss.rs` 的 `mark_promoted` 於 `link` 與 `promote` 內被呼叫；`show <change> --json` payload 不含 from_discussion；既有 spec `discussion-docs` 的「討論以 link 動詞併入既有變更」需求明訂 link 會標記 promoted。

## Goals / Non-Goals

**Goals:**

- 修掉 link 路徑的隱形假完成——`discuss link` 併既有變更後，其舊 proposal 看似完整卻標 promoted。
- ingest 與 propose 對稱：ingest 能自被連結討論播種、並於內容落地封印。
- 零 recurring 執行成本：狀態是實際工作的副產物，非另設的掃描檢查。

**Non-Goals:**

- 不改 `discuss promote` 與 `new change --from-discussion`（維持即標 promoted；理由見 D6）。
- re-conclude stale 偵測（第三支柱，另刀）。
- hash 指紋或 per-load 掃描（結論已否決）。
- desktop 看板視覺呈現（徽章）。

## Decisions

**D1. 新增 `discuss seal` 動詞，而非把 `link` 拆成 forge＋seal。**
seal 承接原 `mark_promoted` 的討論側行為（翻 promoted、累加 promoted_to）。`link` 移除對 `mark_promoted` 的呼叫，只保留變更側鑄鏈。
- 替代：拆 `link` 為 `forge`＋`seal`。取捨：會重命名穩定動詞、動到更多既有呼叫端與 spec；新增動詞的改動面較外科。

**D2. 保留 link 先鑄鏈的時機，seal 在內容落地完成時呼叫。**
封存協同由變更側 from_discussion 驅動（既有機制），故即使 seal 未跑，鏈仍在、封存安全網不失。中斷於 seal 前 → 討論停在 concluded（誠實可回復，重跑 seal 即成）。
- 替代：link-last（把整個 link 挪到最後）。取捨：一旦「標記已轉出」外移，link 時機已無關緊要，重排反而失去中斷自動封存的安全網。

**D3. `seal` 於執行前驗證鏈已存在（變更 meta 的 from_discussion 含該 slug），否則守衛失敗。**
確保「先鑄鏈才封印」的不變量，杜絕在無鏈狀態下憑空 promoted。
- 替代：seal 獨立標記、不驗證。取捨：可製造「promoted 但無 from_discussion」的孤兒，破壞封存協同與看板連結。

**D4. `show <change> --json` 新增 `fromDiscussions`（camelCase 陣列），供 ingest 技能發現連結討論。**
- 替代：讓 ingest 技能硬讀 `.openspec.yaml` grep from_discussion。取捨：脆弱、繞過引擎契約、遠端 store 下失效——改由引擎暴露為契約欄位。

**D5. 「讀討論＋合併」的智慧留在技能（SKILL.md），引擎只供資料。**
沒有 `speclink ingest` 引擎動詞——ingest 本身是技能；讀取用既有 `discuss show <slug>`（已回結論），合併是 agent 判斷。ingest 技能結尾呼叫 `discuss seal`。
- 替代：把「讀＋合併」做成引擎 ingest-context 指令。取捨：無 ingest 引擎動詞可掛、合併需 agent 判斷，屬過度設計。

**D6. 只改 `link`，不改 `promote`／`new change --from-discussion`。**
兩者風險與消費者面不對稱：(a) link 併既有變更，舊 proposal 看似完整、過期隱形 → 標 promoted 真騙人；promote 建新變更＋TBD 骨架，未完成可見於 change 卡（提案中／TBD）→ 標 promoted 不構成假完成。(b) `core::discuss::link` 僅 CLI 用（實測 desktop 與 Node SDK 皆無 link 消費者），改它零漣漪；`core::discuss::promote` 為 desktop「轉為變更」按鈕與 Node SDK 共用，改它會漣漪至 desktop UX 與其測試。原則與務實同向：改 link、留 promote。
- 替代：全面一致（link＋promote＋new-change 皆外移）。取捨：需連改 desktop/Node 及其測試、desktop 促轉 UX 變為 concluded-until-sealed，收益（promote 的低嚴重度假完成）不抵成本。

## Implementation Contract

**行為（使用者可觀察）：**

- `speclink discuss link <slug> <change>`：鑄造變更側 from_discussion（既有累加規則不變）；討論記錄 frontmatter（status／promoted_to）SHALL 逐位元不變。成功 exit 0＋單行訊息。
- `speclink discuss seal <slug> <change>`：討論 frontmatter 變 `status: promoted`、`promoted_to` 累加該變更名（既有值保留、冪等）。守衛失敗（討論不存在／已封存、變更不存在、變更 from_discussion 未含 slug）SHALL 非零 exit＋stderr 說明，且兩側檔案逐位元不變。`--json` 時 payload 含 `slug` 與 `change`（camelCase）。不吃 stdin。
- `speclink discuss promote` 與 `speclink new change --from-discussion`：行為不變（仍即標 promoted）。
- `speclink show <change> --json`：payload 新增 `fromDiscussions`（字串陣列，空鏈為 `[]`）；既有欄位不變。
- 生成的 ingest 技能：指示「目標變更帶 from_discussion 時，經 discuss show 讀結論為一等來源、併入既有脈絡、結尾呼叫 discuss seal」。

**介面／資料形態：** 新 CLI 子指令 `discuss seal`（兩位置參數）；`show --json` 增 `fromDiscussions: string[]`；`discuss seal --json` 回 `{ slug, change }`。

**失敗模式：** seal 守衛失敗一律非零 exit＋stderr、不落檔；鏈不含 slug 時明確拒絕（非靜默）。

**驗收：** `cargo test -p speclink-core --lib` 涵蓋 link 不再翻 promoted、seal 翻 promoted 且驗鏈、冪等、守衛；`speclink show <c> --json` 斷言含 `fromDiscussions`；乾淨樹重生 render golden 並審 diff；`discuss seal --json` payload 斷言 camelCase。

**Crate 邊界：** 流程與狀態語意（link 移除標記、seal、fromDiscussions 派生）歸 speclink-core；子指令解析、人眼／色彩輸出歸 speclink-cli；core 不含 ANSI。

**範圍界線：** 本刀＝link 停止標記＋seal 動詞＋show 暴露 fromDiscussions＋discussion-aware ingest。出範圍：promote/new-change 行為、re-conclude stale、看板徽章、hash／掃描。

## Risks / Trade-offs

- **回歸對照風險（中）**：`discuss link` 的既有單元測試假設「即 promoted」，需更新（link_writes_change_meta_and_marks_discussion、link_accepts_open_discussion、link_appends_… 等）。緩解：逐一改為斷言討論記錄逐位元不變。
- **內嵌技能三處同步**：ingest 技能文字改動須同步 core assets、repo 技能實例、render golden，漏一處即長期紅燈。緩解：動 assets 後於乾淨樹跑 `UPDATE_GOLDEN=1` 重生並審視。
- **link 與 promote 的行為不對稱**：link 不標、promote 標——需在 ingest 技能與文件清楚說明「promote 已標、ingest 才需 seal」，避免使用者誤以為 propose 也要 seal。緩解：D6 記載理由；技能指示明述。
- **跨平台**：seal／link 不涉 git，無新跨平台面；show payload 純序列化。
