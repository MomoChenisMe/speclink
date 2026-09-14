---
title: 提案：建立變更與產物
section: SDD 工作流
order: 110
keywords: [提案, propose, 變更, capability, 命名守門, Purpose, 手動任務, validate, 最後一刀, 分期]
sources: [propose-skill, capability-naming-guard, spec-validation, manual-task-marker]
generated: 2026-09-14T16:01:31+08:00
---

# 提案：建立變更與產物

需求清楚之後，呼叫 `/speclink-propose` 建立變更。agent 依需求來源寫出提案、任務清單與 delta 規格。過程中有兩道守門：capability 的名稱不能跟既有規格撞名或走音，新開的 capability 要寫合格的 Purpose。任務清單裡需要你親手做的事，會標上手動任務標記。

## 需求從哪裡來

agent 依這個優先序決定需求來源：

1. 你在呼叫時明確給的需求描述。
2. `--from-doc <路徑>`：直接以一份文件為需求來源，不需要既存討論。
3. `--from-discussion <slug>`：以討論記錄為來源。
4. plan 檔偵測：plan mode 對話觸發時，從 plan 檔目錄讀。
5. 對話上下文。

### 從討論建立

用 `--from-discussion <slug>` 時，agent 讀討論記錄。記錄的 Context 如果有 `Source doc: <路徑>` 一行，agent 會連原始文件一起讀，並用疊加的方式合成提案：

- 討論有決定的，以討論為準。
- 討論沒觸及的內容，用文件補位。
- 討論 Ruled out 的內容，不會出現在提案裡。即使原始文件主張作法 X，討論否決了 X 並決定作法 Y，提案就採 Y，X 不會以任何形式復活。

記錄沒有 Source doc 行時，流程與一般相同。

#### 最後一刀帶 --last

討論的結論可能把一件事切成幾刀依序立案，並用 `speclink discuss conclude --hold` 把記錄保留在途（見[討論：需求還模糊時](discuss.md)）。用 `--from-discussion` 建立變更時，agent 會判定這次立的是不是最後一刀：讀結論 Decision 段列出的刀清單（刀一／刀二／…或 cut A／cut B 等任一寫法），對照記錄 frontmatter 已轉出的變更清單。

- 是最後一刀：建立變更時併帶 `--last`。
- 立案時把結論的一刀拆成多個變更：只在拆出的最後一段帶 `--last`。
- 結論沒有規劃分期（單刀）：不帶。記錄沒有 hold 行時帶了也不會有任何效果，但技能以「不帶」為規定。

```
speclink new change <變更名> --from-discussion <slug> --last
```

| Decision 的刀清單 | 已轉出的變更 | 本次立的變更 | 帶 --last？ |
| --- | --- | --- | --- |
| 刀一、刀二、刀三 | （空） | 刀一 | 否 |
| 刀一、刀二、刀三 | cut-a、cut-b | 刀三 | 是 |
| 刀一、刀二、刀三 | cut-a、cut-b | 刀三拆出的前半 | 否 |
| 刀一、刀二、刀三 | cut-a、cut-b、cut-c1 | 刀三拆出的後半 | 是 |
| 單刀（未規劃分期） | （空） | 唯一的變更 | 否 |

帶 `--last` 的效果：記錄的保留在途旗標解除，之後最後一個轉出變更封存時，記錄自動隨行封存，不用再手動執行 `speclink discuss archive`。

> [!WARNING]
> 不是最後一刀卻帶了 `--last`，記錄會在最後一個在途變更封存時被收走。再立下一刀之前，把記錄檔從 `openspec/discussions/archive/` 搬回 `openspec/discussions/`。

### 從文件建立

用 `--from-doc <路徑>` 時，提案的 Why 或 Impact 段會留一行 `Source doc: <路徑>`，記下出處。這是技能層的約定，不改變任何 CLI 語法。

## capability 命名守門

提案裡每個 delta 規格都掛在一個 capability 名下。agent 起草時會：

- 把掃描既有規格的結果留在提案裡。
- 對 Capabilities 區段的每個新 capability，附一句「為何既有規格不涵蓋」。

引擎在建立 delta 規格的那一刻守門。名稱不在正式規格裡、這個變更也沒有同名 delta、又沒帶 `--new` 時：

```
speclink new artifact spec <capability> --change <變更名>
```

會被拒絕：以錯誤結束、不建立任何檔案。錯誤訊息包含：

- 至多三筆近似的既有名稱。每筆標注來源（正式規格，或某個進行中的變更名），並附該規格 Purpose 的第一行。
- 兩條指引：修改既有 capability 就沿用它的確切名稱；確定是新 capability 就帶 `--new` 重跑。

名稱比對逐字、區分大小寫。沒有任何近似名時仍然拒絕，只是訊息裡沒有建議清單。

近似名的排序依序看：名稱 token 的完全包含關係、kebab 字段交集數、編輯距離。例如正式規格有 `auth` 與 `author-tools`，你用 `authentication`，建議清單第一筆會是 `auth`。另一個進行中的變更已經開了同名的 delta 時，訊息會另外點名那個變更並指路 `--new`。

確定是新 capability：

```
speclink new artifact spec <capability> --change <變更名> --new
```

`--new` 不豁免 delta 的格式驗證：內容仍然要有 ADDED、MODIFIED、REMOVED 或 RENAMED 其中一種操作區塊。名稱已在正式規格裡時，帶不帶 `--new` 行為相同。

## 新 capability 的 Purpose

新開的 capability，delta 檔要以 `## Purpose` 區段開頭，一兩句話說明能力邊界，去掉前後空白後至少 50 個字元（以字元計，中文一個字算一個）。

執行 `speclink validate <變更名>` 時：

- 新開的 capability 缺 Purpose 或不合格：報 error，變更驗證結果為不通過。錯誤訊息說明規則，並附含 `## Purpose` 的範例骨架。
- 修改既有 capability 的 delta：完全不檢查 Purpose，有沒有都不報。
- 新開的 capability 名稱貼近既有名：報 warning，附近似名清單。同一個 capability 就把 delta 目錄改用既有名；確定是新的可以忽略。這個 warning 不影響驗證結果。

> [!WARNING]
> Purpose 不合格的新 capability 在提案階段只是驗證錯誤，到封存時會被直接拒絕。早點補好。見[封存](archive.md)。

`speclink validate --specs` 會逐份驗證正式規格：缺 `## Purpose` 或內容為空報 error；內容不足 50 字元只在 `--strict` 時報 warning；內容仍是封存時的佔位文字報 warning。`--all` 同時驗變更與正式規格。`--specs` 不能與變更名同時給，會被拒絕並指路單獨 `--specs` 或 `--all`。

## validate 會提早抓出封存守門的違規

delta 與正式規格對不上的問題（例如 MODIFIED 的目標需求已經不在正式規格、ADDED 的名字正式規格已有、MODIFIED 漏掉正式規格既有的 scenario 又沒宣告移除），以前要到封存才被拒絕。現在 `speclink validate <變更名>` 就會對每個 delta capability 做同一套判斷，每條違規化成一條 error，變更驗證結果為不通過，`--strict` 與否都一樣。無參數、`--all`、`--changes`、remote 模式與桌面的結構驗證都適用。

| 違規 | error 長相 |
| --- | --- |
| MODIFIED 的目標不存在 | `specs/auth/spec.md: MODIFIED 'R9': target requirement no longer exists in the canonical spec (see: speclink drift demo)` |
| ADDED 的名字正式規格已有 | `specs/auth/spec.md: ADDED 'R1': already exists in the canonical spec — archive would refuse it (see: speclink drift demo)` |
| RENAMED 缺目標名 | `specs/auth/spec.md: RENAMED 'R1': RENAMED operation names no TO: target (see: speclink drift demo)` |

守門 error 排在結構 error 之後，結構檢查已報過的不重複報（新 capability 的 Purpose 只報 Purpose error；同一份 delta 內的重複需求名只報重複）。完整的守門清單與補救路線見[封存](archive.md)。

## 手動任務標記

任務清單裡，agent 做不到、需要你親手操作的任務，要標 `[M]`。範圍不限於測試：人工驗收、到外部服務建立帳號、放置金鑰，都算。agent 做得到的寫碼與自動化測試不標。

標記的位置有嚴格規則：`[M]` 緊接在 checkbox 之後，checkbox 後恰好一個空格，編號在標記之後。

| 任務行 | 判定 |
| --- | --- |
| `- [ ] [M] 3.2 手測匯入` | 正確 |
| `- [ ] 3.2 [M] 手測匯入` | 錯誤：編號在前 |
| `- [ ]  [M] 手測` | 錯誤：checkbox 後兩個空格 |
| ``- [ ] 說明 `[M]` 剝除規則`` | 正確：中段提到 `[M]` 不算標記 |

寫錯位置的後果是引擎不認得這個標記。任務會被算成寫碼任務，完成度卡住降不下來。`speclink validate <變更名>` 會抓出這兩種錯型並報 error，訊息含任務序號、描述引文與正誤例。

舊版的 `[P]` 前綴仍會被剝掉，但不帶任何意義。

手動任務在實作階段怎麼被處理，見[實作：完成任務](apply.md)。

## 收尾：盤點提案中變更的順序

propose 完成、給下一步建議之前，agent 會列出所有還沒開工的變更。開工與否看變更狀態檔的開工章，不看任務數。

還沒開工的變更有 2 個以上時，agent 會判定執行順序：

- 硬信號：兩個變更的 delta 目錄含同一個 capability，就必須依序。兩份 delta 重寫同一份正式規格，亂序封存可能被合併守門拒絕。
- 軟信號：讀提案與任務推測程式碼重疊或依賴。

| 變更 A 的 delta | 變更 B 的 delta | 判定 |
| --- | --- | --- |
| board-card-order | board-card-order 與 tray-status-menu | 須依序 |
| discuss-skill | archive-skill | 可平行 |

有效的 worktree 政策開啟時，結果分成「可平行」與「須依序」兩組；可平行的變更各開一個 session 走 `/speclink-apply-with-worktree`。政策關閉時給單一建議順序。只有 1 個提案中變更時不盤點。

盤點只是建議，agent 不會自動呼叫任何技能。

下一步：[實作：完成任務](apply.md)。

**出處**：`propose-skill`、`capability-naming-guard`、`spec-validation`、`manual-task-marker`
