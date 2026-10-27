## Context

討論側 `promoted_to` 已是逗號累積器（一份討論可扇出多個變更），但 change 側 `from_discussion` 是單值：`speclink discuss link` 遇到已連結其他討論的變更直接拒絕。re-discussion 情境（出身自討論的變更需要再次討論）揭示關係本質是多對多，單值連結是模型缺口——新討論 link 不回目標變更，無法自動封存，推理鏈斷裂。

現行消費鏈：core 的 link 守衛與封存共行（變更封存時，其 `from_discussion` 指向的討論若不再被任何存活變更引用則隨行封存）→ CLI 的 archive 人眼輸出 → desktop bridge 以 `fromDiscussion`（string|null）送 GUI → GUI 的變更卡徽章、詳情抽屜來源討論、同源變更清單。

## Goals / Non-Goals

**Goals:**

- 出身自討論的變更可再連結後續討論：`discuss link` 對已連結變更改為追加，維持冪等。
- 封存共行在多討論下語意不變：每份連結討論仍「隨最後一個引用它的存活變更封存」。
- GUI 呈現多值：徽章以出身討論為代表，抽屜列出全部來源討論，同源判定改集合交集。
- 單一連結情境（絕大多數既有資料）的行為與 CLI 輸出逐位元不變。

**Non-Goals:**

- 不重開已轉出的討論（rounds append-only、狀態機單向，見討論記錄 rediscuss-promoted-change 的 Ruled out）。
- 不抽取舊討論內容搬移到新討論——上下文自足由新討論的 Context 區摘要引用達成，屬 skill 既有慣例，無引擎改動。
- 不動內嵌技能資產與 render golden（技能文字未文件化單值限制，引擎放行後文字自然成立）。
- 不動 node SDK 的 store bridge（讀原始字串，天然相容）與 CLI `speclink list --json`（不含 fromDiscussion 欄位）。

## Decisions

### D1 — from_discussion 以逗號累積字串存於 meta，讀取端分割

`ChangeMeta.from_discussion` 維持 `Option<String>` 承載原始逗號字串（`from_discussion: d1, d2`），core 提供分割讀取（trim 後的 slug 清單），所有成員判定改走清單。與討論側 `promoted_to` 的儲存與讀取模式（原始字串＋獨立分割函式）完全同款。

- 落點：`speclink-core`（model、discuss、archive）；serde 面不變——欄位仍是字串，既有檔案零遷移，單值即一元清單。
- 分隔安全性：slug 經 kebab-case 驗證或衍生規則滌除 ASCII 標點，不可能含 ASCII 逗號。
- 替代方案：欄位改 `Vec<String>`（自訂 serde）——meta 為手寫 YAML 行式讀寫、inprogress 等處要求既有欄位原樣保留，型別改動會波及全部 meta round-trip 與 node store bridge 的欄位對映，成本高而無行為增益。拒絕。

### D2 — link 守衛改為追加且維持冪等

`discuss link` 的守衛表僅改一格：「變更 meta 已有 from_discussion 指向其他討論」從拒絕改為在既有值後追加 `, <slug>`；「已含本 slug」維持冪等成功不改檔；其餘守衛（討論不存在、討論已封存、變更不存在）與寫入順序（change 側先寫、討論側後寫）不變。

- 落點：`speclink-core` discuss 模組；CLI 成功／失敗訊息形狀不變（僅原「已連結其他討論」錯誤路徑消失）。
- 替代方案：保留拒絕＋新增 `--force` 旗標——使用者與 agent 都要學新旗標、skill 文字被迫改動三處同步；且「追加」在 M↔N 模型下是正確預設而非危險操作，無需門檻。拒絕。

### D3 — 封存共行逐 slug 判定，ArchiveOutcome 改複數

變更封存時，對其 from_discussion 清單的每個 slug 各自檢查「是否仍被任何存活變更的清單引用」，不被引用者隨行封存。`ArchiveOutcome.archived_discussion` 由 `Option<(String, String)>` 改為 `archived_discussions: Vec<(String, String)>`。

- 落點：`speclink-core` archive 模組；`speclink-cli` 的 archive 人眼輸出改為逐討論一行，單一討論情境的輸出與現行逐位元一致（回歸保護）；desktop bridge 的 archive 結果 camelCase 組裝同步改複數。
- 替代方案：維持 Option、多討論時只取第一個——其餘討論靜默漏封存，正是本變更要消滅的「板上永遠掛著」。拒絕。

### D4 — bridge 欄位改名 fromDiscussions 陣列，GUI 多值呈現

desktop bridge 送 GUI 的變更項欄位由 `fromDiscussion: string|null` 改為 `fromDiscussions: string[]`（空陣列＝非討論而來），首元素即出身討論。GUI 端：變更卡徽章顯示維持單徽章、以首元素為代表且 title 列出全部；詳情抽屜「來源討論」列出全部並可互跳；同源變更清單判定改為「雙方 fromDiscussions 交集非空」。

- 落點：desktop bridge 的變更清單與詳情查詢、`packages/ui` 型別與元件（ChangeCard、RichDetailDrawer、i18n 多值文案）、desktop 前端的同源計算。bridge 與 GUI 同艙出貨，無外部消費者，一次換形不留雙欄位。
- 替代方案 A：保留 `fromDiscussion` 鍵、值改陣列——同名不同型，對照舊測試與舊碼時易誤讀。拒絕（改名即文件）。
- 替代方案 B：雙欄位並存（string 首項＋陣列全量）——冗餘無消費者。拒絕。

## Implementation Contract

**行為**（實作完成後可觀察）：

- 對已有 `from_discussion: d1` 的變更 c1 執行 `speclink discuss link d2 c1`：exit 0、stdout 單行成功訊息（形狀同現行）；c1 的 meta 變為 `from_discussion: d1, d2`；d2 標記 `status: promoted` 且 `promoted_to` 累加 c1。
- 同一組合重跑：exit 0，兩側檔案內容不變（冪等）。
- 其餘守衛不變：討論不存在／已封存、變更不存在 → 非零 exit code、stderr 說明、兩側檔案逐位元不變。
- c1 封存時：d1、d2 各自檢查存活引用，皆無引用則兩者皆移入 `openspec/discussions/archive/`，CLI 逐討論輸出一行共行訊息；僅一份連結討論的變更封存輸出與現行逐位元一致。
- GUI：多討論變更卡帶單一討論徽章（title 列全部來源）；詳情抽屜列出全部來源討論可互跳；兩變更任一來源討論相同即互為同源。

**介面／資料形狀**：

- meta：`from_discussion: <slug>[, <slug>]*`（逗號＋空格分隔，追加於尾端）。
- core：ArchiveOutcome 的共行結果為 `Vec<(slug, archived_file)>`。
- bridge JSON：變更項 `fromDiscussions: string[]`（camelCase，空陣列＝無來源討論），取代 `fromDiscussion`。

**失敗模式**：link 守衛失敗即整體失敗、零寫入；封存共行對單一討論的封存失敗沿現行行為（不阻斷變更本身的封存）。

**驗收**：

- Rust：cargo test -p speclink-core --lib 通過（含新測試：追加、冪等、多討論逐 slug 共行）；本 Windows 機器須帶 --lib（mingw cdylib 連結問題）。
- 前端：npm test -w packages/ui 與 npm test -w apps/desktop 通過（含多值徽章、抽屜清單、交集同源的案例）。
- 回歸：archive 單一討論情境與 link 成功訊息以變更前 baseline exe 對照輸出一致。
- GUI 依開發備忘以真實視窗檢視徽章與抽屜呈現。

**Scope 邊界**：

- In：core 的 link／封存共行、CLI archive 輸出、desktop bridge 欄位、packages/ui 與 desktop 前端多值呈現、對應 spec delta（discussion-docs、desktop-app）。
- Out：技能資產與 golden、node SDK store bridge、CLI `speclink list --json`、討論記錄文件格式、重開／抽取討論的任何機制。

## Risks / Trade-offs

- [CLI 輸出回歸破口：archive 共行輸出改迴圈後單一情境走樣] → 以變更前 baseline exe 對單一討論情境做輸出對照；新測試鎖定單行格式字串。
- [守衛移除屬行為變更，既有 spec 明文「拒絕」] → discussion-docs 以 MODIFIED delta 同步守衛表與情境，封存時併入正典，spec 與行為不脫鉤。
- [GUI 靜默降級：漏改某個 fromDiscussion 消費點致徽章／同源消失] → 改名（fromDiscussion→fromDiscussions）使漏改點成為型別錯誤而非靜默 null；TypeScript 編譯即檢出。
- [跨平台：meta 追加寫入的換行處理] → 沿用 link 現行「無結尾換行則補」邏輯，不新增平台假設；Rust 測試覆蓋無結尾換行的 meta。
