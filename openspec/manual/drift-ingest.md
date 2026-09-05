---
title: 續作與需求變更：drift 與 ingest
section: SDD 工作流
order: 130
keywords: [drift, ingest, 漂移, 閒置, 需求變更, 過期]
sources: [drift-computation, archive-merge, skill-routing, user-documentation]
generated: 2026-09-03
---

# 續作與需求變更：drift 與 ingest

兩種情況會用到這一頁：

- **變更閒置了一陣子，現在要撿回來做**：先跑 drift，看程式碼和規格離當初的假設有多遠。
- **實作到一半需求變了**：走 ingest，把新的需求併進這個變更的產物，再回去實作。

> [!NOTE]
> drift 與 ingest 兩個技能的內文行為規格未載。本頁只寫規格有寫的部分：drift 報告的內容、它與封存守門的關係，以及兩個技能的交棒方向。

## drift：看漂移了多少

```
speclink drift <變更名>
```

drift 只是診斷。它不會寫回正典規格，也不會動任何規格文件。同一個工作區狀態跑兩次，結果一樣。

報告分五個維度：

| 維度 | 看什麼 |
| --- | --- |
| Specs | 規格面：這個變更的 delta 相對正典規格有沒有過期 |
| Time | 工作區面 |
| Structure | 工作區面 |
| Tasks | 工作區面 |
| Environment | 工作區面 |

規格面只看規格事實。後面四個維度看你本機的工作區，需要有 code checkout。

沒有 checkout 時，四個工作區維度標為「unavailable」，不給分數，也不算乾淨。整份報告的涵蓋範圍標為 spec-only。有 checkout 但 git 不能用，是另一種狀態，報告沿用既有的 git 不可用回退文字與分數。

drift 以一份固定的基準運算。如果運算到合併前，你又改了這個變更的 tasks.md 之類的檔案，報告會標 stale，並列出哪些基準對不上。它不會靜默給你一份混用兩個時點的報告。

## 規格面過期，代表封存會被拒

drift 的 Specs 維度、批次封存的預檢、單筆封存的合併守門，三處用同一套過期判定。同一條過期的 delta 操作，三處會指向同一個 capability 與需求名。drift 與批次預檢的文字寫的是「archive 將拒絕」，不是「跳過」。

兩類過期各有不同的補救路線：

- **一般過期**（delta 裡的需求名對不上正典，例如 ADDED 的名字已存在、MODIFIED 的來源不存在）：先 drift，再以 ingest 更新 delta。
- **新 capability 缺 Purpose**（delta 新開的 capability 沒有合格的 `## Purpose` 區段）：drift 的主建議指向 validate 的 Purpose 指引，不是 ingest。補寫 `## Purpose`，再跑 `speclink validate` 拿完整指引。

完整的拒絕清單見 [封存](archive.md)。

## drift 之後往哪走

drift 技能結尾依結果給建議，只建議、不代跑：

- 檢出 delta 的假設已過期：走 `/speclink-ingest`。
- 沒有漂移：回 `/speclink-apply` 繼續。

## ingest：把新需求併進變更

實作途中收到會改變產物的新需求時，用 `/speclink-ingest`。它把新的脈絡併進這個變更既有的提案、設計、規格與任務，讓 apply 可以接著做。做完之後，ingest 建議回 `/speclink-apply`。檢查結果可能讓你回 apply，也可能需要再 ingest 一次。

有一種 ingest 來自討論：討論的結論要修正一個已存在的變更時，依序做三件事。

1. `speclink discuss link <slug> <變更名>`：在變更這一側建立來源鏈。
2. `/speclink-ingest`：把討論的結論反映進變更的產物。
3. 執行討論的 seal 動詞：內容反映完之後，把討論標成已轉出。

link 只建立變更側的來源鏈；seal 只在內容已經反映後才做。討論的完整流程與 seal 的指令寫法見 [討論：需求還模糊時](discuss.md)。

## 分清楚兩個入口

| 你的情況 | 走哪裡 |
| --- | --- |
| 變更閒置後要續作 | 先 drift |
| 實作中需求改變 | ingest |

變更曾反映的討論又重新下了結論，是第三種情況，見 [實作：完成任務](apply.md) 的「需求中途變更」一節。

**出處**：`drift-computation`、`archive-merge`、`skill-routing`、`user-documentation`
