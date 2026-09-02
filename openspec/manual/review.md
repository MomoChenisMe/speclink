---
title: 審查站
section: SDD 工作流
order: 150
keywords: [審查, review, 工藝品質, code smell, 工單, 蓋章]
sources: [review-skill, review-station]
generated: 2026-09-02
---

# 審查站

審查站看實作的工藝品質：專案慣例有沒有守、有沒有 code smell、有沒有 bug。它不裁決「有沒有符合規格」，那是 [驗證站](verify.md) 的事。技能是 `/speclink-review`，底下的工單指令是 `speclink review ...`。審查面怎麼決定、章怎麼失效，見 [品質關卡總覽](quality-stations.md)。

## 技能怎麼跑

1. 選定要審的變更。
2. 檢查寫碼任務完成度。還有寫碼任務沒完成就停止，不算範圍、不派審查員、不寫工單。只剩 `[M]` 手動任務時繼續，結果會點名剩餘的手動任務，並說明「章可以先蓋、手動測試完成後再封存」。
3. 用 `speclink review scope` 取得凍結的差異。沒有工單時是第一輪 discovery；有工單時是續輪 validation。第一輪範圍有歧義時，技能等你提供可信的基準、明確的段落挑選、或改用隔離的 worktree，不會拿整個檔案代替。
4. 讀變更的產物（提案、設計、規格、任務）當判斷脈絡。產物很薄時，審查員只憑程式碼與測試判斷，不編需求。
5. 依輪次分流：第一輪完整檢查，續輪只驗收修正。
6. 並列呈現結果，並對每筆發現給處置分類。
7. 用 `speclink review add-round` 把這一輪寫進工單。

## 第一輪：兩軸平行檢查

同一份凍結差異交給兩個只讀的審查員平行看，各自用 400 字內回報，發現分 CRITICAL、WARNING、SUGGESTION 三級：

- **Standards**：專案慣例文件加 code smell 基準。專案文件優先：專案明文允許的做法不報。smell 一律是「possible X」的提示，永遠不當硬性違規；工具已經自動強制的規則不報。基準的 12 條 smell 名稱維持英文原文：Mysterious Name、Duplicated Code、Feature Envy、Data Clumps、Primitive Obsession、Repeated Switches、Shotgun Surgery、Divergent Change、Speculative Generality、Message Chains、Middle Man、Refused Bequest。
- **Correctness**：找 bug。

兩軸只看變更的差異段落，加上判斷所需的呼叫端與測試。兩份報告原樣並列，不合併、不重排。

發現的描述用工作區設定的語言寫；工作區沒設語言就用英文。嚴重度標籤、`Standards:`／`Correctness:` 前綴、檔案路徑維持英文。

## 處置分類與三選項

技能在問你之前，先把每筆發現分成兩類：

- **必修**：CRITICAL 級；Correctness 軸判定有現實觸發路徑的 bug（含 WARNING 級）；專案明文標準的明確違反。
- **可裁**：「possible X」的 smell 判斷與其他 SUGGESTION 級事項，每筆附一行修繕成本與效益的理由。可裁事項一律以 SUGGESTION 級記錄，不會用 WARNING。

有必修時，技能問你三選項，推薦「修正後重審」並列出必修清單：

- 修正後重審。
- 接受現狀蓋章。
- 先不蓋章。

只有可裁項時不問。單站直接呼叫時，技能記錄這一輪後直接蓋乾淨章。在 `/speclink-quality` 合跑時，改成「先不蓋章」離場，章留到合跑的收尾補蓋。

沒有互動選單工具的環境，技能用純文字問並等你回覆。

## 修正與續輪

修正一律由主線依專案的 TDD 慣例做，審查員不改檔案。修正完、派下一輪之前，技能先跑專案的完整建置與測試，全綠才進下一輪。

續輪只做兩件事：逐筆判定上一輪的發現已解或未解，以及回報修正本身直接引入的問題。未解的發現以原文寫進新一輪；已解的移除；沒改到的區域裡的新 smell、既存問題都不加進來。修正差異裡如果有「相鄰」段落（沒被點名但被改到的檔案），審查員會逐段確認確實屬於這次修正，不是的話當作退化回報。範圍外的變動只轉知你，不進工單。

每輪之後，技能比較「未接受的必修數」與上一輪：

| 情況 | 結果 |
| --- | --- |
| 必修清空，且沒有已接受的必修 | 執行 `speclink review stamp`，結果 passed clean。殘留的 SUGGESTION 不擋章 |
| 必修清空，但有已接受的必修 | 推薦你明示執行 `speclink review stamp --accept`，結果 passed with reservations |
| 必修數減少但沒清空 | 再問一次三選項 |
| 必修數沒有減少 | 記錄這一輪後以 failed 結束，工單留著、不蓋章、不自動重試 |

必修數減少只決定能不能繼續，不是品質分數。技能沒有固定的最大輪數。

## 已接受的事項

你裁定「接受、不修」的必修發現，續輪會兩軌處理：審查員收到不重報清單，不再報同一件事；工單新一輪照樣列出這些事項，行末加 `(accepted)` 標記，讓最後一輪忠實反映保留事項。換一個 session 接手時，技能從標記行重建不重報清單。接受機制只適用必修級；SUGGESTION 不需接受。

## 續輪遇到重大新問題

續輪偶然看到與修正無關的新問題時，技能不會把它塞進目前的發現，也不會重開第一輪。只有同時滿足「有現實觸發路徑」、「有重現方式、失敗測試或明確不變量破壞」、「影響安全、資料損失或錯誤行為」時，技能以 scope changed／failed 結束本站，工單留著、不蓋章，建議另開一輪或衍生一個變更。證據不足的事項只列為後續提示。

## 工單指令

| 指令 | 做什麼 |
| --- | --- |
| `speclink review add-round <變更名> --stdin` | 從標準輸入讀一輪內容，追加到工單。工單不存在就建立 |
| `speclink review show <變更名>` | 印出工單原文。無工單時報錯「該 change 無審查工單」 |
| `speclink review stamp <變更名> [--accept]` | 蓋章。條件與效果見品質關卡總覽 |
| `speclink review discard <變更名>` | 刪掉工單，不寫任何章。無工單時報錯 |

add-round 會拒絕的內容：缺少範圍行；輪次階段與差異指紋兩欄只出現其中一個；第二個 discovery（第一輪之後只能是 validation）；變更不存在。拒絕時工單零寫入。舊格式的工單（沒有階段與指紋）仍可讀可追加。

stamp 會拒絕的情況：寫碼任務沒全完成（列出計數）；最後一輪還有未解必修且沒帶 `--accept`（提示 `--accept` 或先修正重審）。最後一輪只剩 SUGGESTION 時照常蓋章。

蓋章後工單刪除，fs 模式只剩 git 歷史，remote 模式不可回讀。

## 兩種特殊收尾

如果先前蓋章因為別的守門失敗而留下了零發現的工單，守門恢復後技能直接重試蓋章，不重審。舊格式工單有發現、卻沒有對應的快照時，技能無法精確重建修正差異，會保留工單，等你明示放棄後再重跑第一輪。

蓋章之後往哪走，見 [工作流總覽](workflow-overview.md) 的交棒表；封存的規則見 [封存](archive.md)。

**出處**：`review-skill`、`review-station`
