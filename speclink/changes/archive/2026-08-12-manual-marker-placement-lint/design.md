## Context

`[M]` 標記只在 checkbox 後的前綴槽被解析（crates/speclink-core/src/tasks.rs 的前綴剝除迴圈；行為由 manual-task-marker 正典的 Example 表凍結）。誤寫成「編號在前」或 checkbox 後多打空格時，任務被靜默算成寫碼任務，codeRemaining 卡死、station 守門誤判。本設計落實討論結論的兩關防護：validate 的 error 級 lint＋propose／ingest 技能文字的對比對規則。

## Goals / Non-Goals

**In scope**：
- speclink-core 的標記誤置判斷式與 change 驗證接線（error 級）
- propose.md 與 ingest.md 兩個技能 asset 的起草指引改寫（對比對），含三連動與本 repo 技能檔再生
- manual-task-marker、propose-skill 兩份正典的對應條文

**Out of scope**：
- 解析器行為（前綴槽定義）不動；packages/ui 解析器不動
- analyze 不加同類檢查；desktop 不加專屬 UI（validate 輸出自然流至各端）
- desktop-loading-skeleton-ux 6.2 的現場修正（已在其 worktree 完成）

## Decisions

### D1: 判斷式落點

判斷式歸 tasks.rs，validate.rs 只接線。

標記誤置偵測屬標記語意，與 parse／counts 同居 crates/speclink-core/src/tasks.rs（領域演算法歸 speclink-core，無 ANSI、無儲存媒介假設）。新增公開函式接收解析後的任務清單，回傳誤置清單（任務序號＋描述），validate_change 讀 tasks.md、呼叫判斷式、格式化錯誤。單一實作落點：CLI、server、desktop 全部經 validate_change 消費，不平行實作（回歸對照 remote_verb_parity 不受影響——輸出僅在誤置存在時多出 error 行）。

### D2: 誤置判準

兩錯型的精確判準——只看描述開頭，不掃中段。

對每個解析後任務的顯示描述（前綴剝除已完成）：

- **錯型 A「編號在前」**：首個空白分隔 token 僅含 ASCII 數字與句點且至少一個數字（如 `6.2`、`3`、`1.10`），且下一個 token 為字面 `[M]` → 命中。
- **錯型 B「行首殘留」**：描述以字面 `[M]` 開頭（後接空白或即為結尾）→ 命中。對應 checkbox 後多打空格使前綴槽漏接的殘留。

只檢查描述開頭。行文中段或反引號包裹的 `[M]`（本 repo 既有 tasks.md 大量存在）不誤中。已勾任務同樣檢查——誤置是格式錯誤，與完成狀態無關。

### D3: 錯誤訊息契約

自帶修復指引的正誤例並列。

比照 Purpose 早期檢查慣例（訊息自帶修法，不只報缺失）。英文訊息（CLI 輸出不在詞彙表範圍），一行一任務，含：tasks.md 邏輯路徑（正斜線）、任務序號、描述前綴引文、正誤例並列。形如：

```
openspec/changes/<name>/tasks.md: Task 12 ("6.2 [M] 手動驗收…"): misplaced [M] marker — write "- [ ] [M] 6.2 …" (marker right after the checkbox), not "- [ ] 6.2 [M] …"
```

錯型 B 的訊息點名「checkbox 後恰一個空格」。error 級、驗證結果 invalid；既有錯誤先列、本檢查後附（凍結順序慣例同 Purpose 檢查）。tasks.md 缺席或無命中：零輸出，既有驗證結果逐位元不變。

### D4: 對比對文字與三連動

propose.md 既有 `[M]` 段落改寫、ingest.md 新增段落，共同形狀：正例 `- [ ] [M] 3.2 …` 與誤例 `- [ ] 3.2 [M] …` 並列、一句後果（引擎不認、任務被算成寫碼任務而卡住完成度）、「checkbox 後恰一個空格」規則。asset 內文變動走三連動：跑 golden 對照測試，紅燈即提升 crates/speclink-core/src/init.rs 的 MARKER_VERSION 並再生 golden 與 crates/speclink-core/tests/golden/assets.lock；claude 與 codex 兩形由正典化生成涵蓋。完成後於本 repo 再生技能檔，確認 .claude/skills 渲染副本同步。

## Risks / Trade-offs

- **誤報**：正當描述恰以「數字＋空白＋字面 [M]」開頭的機率趨近零（要提及字面標記的描述實務上用反引號，反引號使 token 不等於 `[M]`，不命中）。接受此殘餘風險換取零誤放。
- **既有誤寫浮現**：lint 上線後，主檢出的 desktop-loading-skeleton-ux tasks.md（誤寫行在 worktree 已修、main 待 merge 流回）在 merge 前跑 validate 會被點名——這是正確行為，merge 後自然消失；封存區不在 validate 範圍，舊誤寫不受影響。
- **輸出相容性**：新 error 僅在誤置存在時出現；無誤置的變更輸出逐位元不變，回歸對照無波及。

## Migration Plan

單向落地，無資料遷移。順序：引擎判斷式與接線（TDD）→ 正典條文隨 delta 落 → asset 文字與三連動 → 本 repo 技能檔再生。可獨立 revert（lint 與 asset 文字互不依賴）。

## Open Questions

(none)
