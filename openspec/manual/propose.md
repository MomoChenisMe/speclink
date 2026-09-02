---
title: 提案：建立變更與產物
section: SDD 工作流
order: 110
keywords: [提案, propose, 變更, capability, 命名守門, Purpose, 手動任務]
sources: [propose-skill, capability-naming-guard, spec-validation, manual-task-marker]
generated: 2026-09-02
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

### 從文件建立

用 `--from-doc <路徑>` 時，提案的 Why 或 Impact 段會留一行 `Source doc: <路徑>`，記下出處。這是技能層的約定，不改變任何 CLI 語法。

## capability 命名守門

提案裡每個 delta 規格都掛在一個 capability 名下。agent 起草時會：

- 把掃描既有規格的結果留在提案裡。
- 對 Capabilities 區段的每個新 capability，附一句「為何既有規格不涵蓋」。

引擎在建立 delta 規格的那一刻守門。名稱不在正典規格裡、這個變更也沒有同名 delta、又沒帶 `--new` 時：

```
speclink new artifact spec <capability> --change <變更名>
```

會被拒絕：以錯誤結束、不建立任何檔案。錯誤訊息包含：

- 至多三筆近似的既有名稱。每筆標注來源（正典，或某個進行中的變更名），並附該規格 Purpose 的第一行。
- 兩條指引：修改既有 capability 就沿用它的確切名稱；確定是新 capability 就帶 `--new` 重跑。

名稱比對逐字、區分大小寫。沒有任何近似名時仍然拒絕，只是訊息裡沒有建議清單。

近似名的排序依序看：名稱 token 的完全包含關係、kebab 字段交集數、編輯距離。例如正典有 `auth` 與 `author-tools`，你用 `authentication`，建議清單第一筆會是 `auth`。另一個進行中的變更已經開了同名的 delta 時，訊息會另外點名那個變更並指路 `--new`。

確定是新 capability：

```
speclink new artifact spec <capability> --change <變更名> --new
```

`--new` 不豁免 delta 的格式驗證：內容仍然要有 ADDED、MODIFIED、REMOVED 或 RENAMED 其中一種操作區塊。名稱已在正典裡時，帶不帶 `--new` 行為相同。

## 新 capability 的 Purpose

新開的 capability，delta 檔要以 `## Purpose` 區段開頭，一兩句話說明能力邊界，去掉前後空白後至少 50 個字元（以字元計，中文一個字算一個）。

執行 `speclink validate <變更名>` 時：

- 新開的 capability 缺 Purpose 或不合格：報 error，變更驗證結果為不通過。錯誤訊息說明規則，並附含 `## Purpose` 的範例骨架。
- 修改既有 capability 的 delta：完全不檢查 Purpose，有沒有都不報。
- 新開的 capability 名稱貼近既有名：報 warning，附近似名清單。同一個 capability 就把 delta 目錄改用既有名；確定是新的可以忽略。這個 warning 不影響驗證結果。

> [!WARNING]
> Purpose 不合格的新 capability 在提案階段只是驗證錯誤，到封存時會被直接拒絕。早點補好。見[封存](archive.md)。

`speclink validate --specs` 會逐份驗證正典規格：缺 `## Purpose` 或內容為空報 error；內容不足 50 字元只在 `--strict` 時報 warning；內容仍是封存時的佔位文字報 warning。`--all` 同時驗變更與正典規格。`--specs` 不能與變更名同時給，會被拒絕並指路單獨 `--specs` 或 `--all`。

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

- 硬信號：兩個變更的 delta 目錄含同一個 capability，就必須依序。兩份 delta 重寫同一份正典規格，亂序封存可能被合併守門拒絕。
- 軟信號：讀提案與任務推測程式碼重疊或依賴。

| 變更 A 的 delta | 變更 B 的 delta | 判定 |
| --- | --- | --- |
| board-card-order | board-card-order 與 tray-status-menu | 須依序 |
| discuss-skill | archive-skill | 可平行 |

有效的 worktree 政策開啟時，結果分成「可平行」與「須依序」兩組；可平行的變更各開一個 session 走 `/speclink-apply-with-worktree`。政策關閉時給單一建議順序。只有 1 個提案中變更時不盤點。

盤點只是建議，agent 不會自動呼叫任何技能。

下一步：[實作：完成任務](apply.md)。

**出處**：`propose-skill`、`capability-naming-guard`、`spec-validation`、`manual-task-marker`
