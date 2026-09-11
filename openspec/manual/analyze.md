---
title: 分析：交叉檢查變更的產物
section: SDD 工作流
order: 115
keywords: [analyze, 分析, 交叉檢查, 覆蓋度, 一致性, 模糊度, 缺漏, 弱語氣]
sources: [change-analysis]
generated: 2026-09-11T10:03:08+08:00
---

# 分析：交叉檢查變更的產物

提案寫完、還沒開始實作之前，可以先讓引擎把一個變更的提案、設計、delta 規格與任務清單互相對照一遍，看有沒有對不上的地方。指令是：

```
speclink analyze <變更名>
```

它只讀不寫，什麼檔案都不會動。技能入口是 `/speclink-analyze`。桌面 app 詳情面板的「分析」按鈕跑的也是這個檢查，畫面呈現見[看板與任務](desktop-board.md)。

> [!NOTE]
> analyze 只管「哪些情況該提醒你」。產物的格式對不對，歸 `speclink validate`；delta 能不能併進正典，歸封存守門（見[封存](archive.md)）；程式碼與規格離當初假設多遠，歸 drift（見[續作與需求變更](drift-ingest.md)）。

## 四個面向

報告分四個面向，依序是覆蓋度（Coverage）、一致性（Consistency）、模糊度（Ambiguity）、缺漏（Gaps）。每個面向要有夠用的產物才會跑；不夠時該面向標為 `Skipped (insufficient artifacts)`，其餘照常。有沒有產物只看檔案存不存在，空檔也算存在。

| 面向 | 需要的產物 |
| --- | --- |
| 覆蓋度 | proposal.md，加上 specs 目錄或 tasks.md 至少一個 |
| 一致性 | design.md 與 tasks.md |
| 模糊度 | 至少一份 delta 規格 |
| 缺漏 | 任一產物存在 |

每個面向的狀態是三選一：`Skipped (insufficient artifacts)`、`Clean`（零發現）、`N issue(s) found`。人眼輸出逐面向列狀態與發現數。

每筆發現有：編號（面向前綴 COV／CON／AMB／GAP 加流水號）、嚴重度（Critical／Warning／Suggestion）、位置、摘要與建議。發現依覆蓋度、一致性、模糊度、缺漏的順序排列。

| 你看到的情況 | 意思 |
| --- | --- |
| 有 proposal.md、tasks.md 與 delta 規格，沒有 design.md | 一致性標為 Skipped，其餘三個面向照常 |
| tasks.md 存在但是空檔 | 覆蓋度與一致性照常跑；設計裡每個小節都會被報「任務沒提到」 |

## 覆蓋度：提案說的，規格與任務有沒有跟上

兩種發現：

- **提案列了 capability，delta 規格卻不存在**（Critical）。引擎讀 proposal.md 的 Capabilities 區段（含底下的 New Capabilities、Modified Capabilities 小節），每一行第一個反引號包起來的名字當作 capability，找不到對應的 delta 檔就報一筆。
- **delta 需求沒有任務去做**（Warning）。每條 delta 需求的名字，要以連續子字串出現在至少一個 checkbox 任務的描述裡，不分大小寫。只出現在群組標題不算。REMOVED 區塊的需求也要有任務去拆。tasks.md 不存在時不報。

| 需求名 | 任務描述 | 結果 |
| --- | --- | --- |
| `CSV Export` | `Implement csv export` | 命中（不分大小寫） |
| `CSV Export` | `Implement csv-export` | 未命中（連字號打斷了連續子字串） |
| `CSV Export` | `Export CSV` | 未命中（順序不同） |
| `CSV Export` | 只出現在標題 `## 2. CSV Export` | 未命中，報 `Requirement 'CSV Export' has no matching task` |

## 一致性：設計裡的每個題目，任務有沒有提到

設計文件（design.md）每個 `###` 小節都算一個題目。tasks.md 沒提到它時報一筆 Warning，摘要是 `Design topic '<小寫整串標題>' not referenced in tasks`。

判定先把標題拆成「編號」與「本文」。編號只認三種樣式：`D<數字>`、`決策<數字或連續的中文數字一到十>`、`Decision <數字>`（數字只認半形，全形數字不算；不分大小寫；多個空白當一個）。編號後面可以接冒號（全形或半形都行）。剩下的是本文。

tasks.md 任何一行（群組標題、checkbox、散文都算）滿足下列任一條，就算提到：

- 含本文（不分大小寫）。
- 含編號，而且編號前後都不是英數字，後面也不是中文數字。夾在任務 ID 註解或 `card1` 這類識別符裡的 `d1` 不算。

本文為空（標題只有編號）就只比編號。標題沒有編號時，退回整串標題的子字串比對。

| 設計標題 | tasks.md 裡有 | 結果 |
| --- | --- | --- |
| `決策一：整個移除 listDepthLimit 擴充` | `## 2. 實作：整個移除 listDepthLimit 擴充` | 不報 |
| `D1 違規清單與聚合錯誤形狀` | `- [ ] 1.1 彙整違規（design D1）` | 不報 |
| `D1 違規清單與聚合錯誤形狀` | 只有 `D12` | 報 |
| `D1 違規清單與聚合錯誤形狀` | 只在任務 ID 註解裡有 `d1` | 報 |
| `決策十一：整個移除 listDepthLimit 擴充` | `拆除擴充（design 決策十一）` | 不報 |
| `決策十：拆分模組` | 只有 `決策十二` | 報 |
| `D4` | 只有 `D42` | 報；有 `(design D4)` 時不報 |
| `索引 JSON 的形狀與推導規則` | 只有 `索引 JSON 的形狀` | 報；整串都在時不報 |
| `D１ 全形編號` | — | 沒有編號，整串比對 |

## 模糊度：規格寫得夠不夠具體

### 需求要有 scenario

REMOVED 以外區塊的每條需求，至少要有一個 `#### Scenario:`，沒有就報 Warning：`Requirement '<name>' has no scenarios`。

REMOVED 區塊的需求不查 scenario（有也不違規），改查需求本文（`### Requirement:` 之後、第一個 scenario 之前）有沒有各一行以 `**Reason**` 與 `**Migration**` 開頭（冒號在粗體內或外都可以，`**Reasoning**` 不算）。缺任一個報 Warning，摘要是 `REMOVED requirement '<name>' has no **Reason**`、`... has no **Migration**` 或 `... has no **Reason** and **Migration**`，建議是 `Add **Reason**: and **Migration**: lines under '<name>'`。這兩行要寫在本文，寫進 scenario 內文不算。

### scenario 要有具體值

一個 scenario 沒有 `##### Example:`，內文又沒有任何「具體值字元」時，報 Suggestion：`Scenario '<name>' has no concrete examples`。具體值字元是：半形數字、反引號、半形雙引號、全形引號「」『』、全形數字。中文數字（一、二、三）與單引號不算。

| scenario 內文 | 結果 |
| --- | --- |
| `回傳 3 筆結果` | 具體 |
| `顯示「已封存」` | 具體 |
| `第１頁` | 具體（全形數字） |
| `回傳三筆結果` | 抽象（中文數字不算） |
| `顯示 '完成'` | 抽象（單引號不算） |
| `顯示成功訊息` | 抽象 |

### 弱語氣詞

每一行（標題行除外）最多報一筆 Suggestion：`Vague language '<pattern>' found`，位置帶行號。檢查順序是：五個英文詞 should、may、might、consider、possibly，再 `TBD`／`TODO`／`???`／`TKTK`，再中文詞（可能、應該、考慮）。

英文五詞以字邊界比對：整行轉小寫後，命中的前後都不是英文字母才算；`n't` 縮寫算字邊界，所以 `shouldn't` 仍報 `should`。代價是 `maybe`、`considered` 這類含字的完整單字不再命中。`TBD`／`TODO`／`TKTK` 維持不分大小寫的子字串比對，`???` 維持原樣子字串比對。中文詞維持子字串比對；「不可能」豁免：去掉這三個字後仍含「可能」才報。

| 規格行 | 結果 |
| --- | --- |
| `The strap should lock` | 報 `should` |
| `The shoulder strap SHALL lock` | 不報 |
| `a considerable delay` | 不報 |
| `the mayor` | 不報 |
| `it shouldn't lock` | 報 `should` |
| `maybe lock` | 不報（字邊界的代價） |
| `outbdoor` | 報 `TBD`（子字串比對維持） |
| `系統盡可能鎖定` | 報 `可能` |
| `這不可能發生` | 不報 |

## 缺漏：東西有沒有少

四種發現：

- **討論被重新結論**（Suggestion）：這個變更反映的討論後來又重新下了結論，每份討論一筆，位置是變更的狀態檔。處理方式見[實作：完成任務](apply.md)的需求中途變更一節。
- **有 delta 規格卻沒有提案**（Critical）：proposal.md 不存在。
- **MODIFIED 指向的 capability 沒有正典規格**（Warning）：每個 capability 只報一筆，例如 `MODIFIED requirements reference capability 'auth' but no main spec found`。
- **正典有這個 capability，但找不到同名需求**（Warning）：例如 `MODIFIED requirement 'R9' not found in main spec`。名字要與正典的 `### Requirement:` 逐字相同。

REMOVED 與 RENAMED 的目標存不存在，不由 analyze 判定，那是封存守門的範圍，`speclink validate` 也會提早報出來。見[封存](archive.md)。

## 之後怎麼走

analyze 是隨叫隨用的工具，沒有固定的下一步。發現若指向產物本身，回頭改提案、設計、規格或任務；若指向 delta 與正典對不上，先跑 drift 再 ingest（見[續作與需求變更](drift-ingest.md)）。沒有發現就進[實作：完成任務](apply.md)。

**出處**：`change-analysis`
