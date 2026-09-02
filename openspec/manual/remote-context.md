---
title: remote 模式的規格投影
section: Remote 模式
order: 490
keywords: [投影, context, refresh, stale, 唯讀, gitignore]
sources: [context-projection]
generated: 2026-09-02
---
# remote 模式的規格投影

remote 模式的正典在 server 上。為了讓 AI 工具在本機讀得到規格，Speclink 會把 server 的內容投影成一份唯讀的本機副本。這一頁說明投影放在哪裡、什麼時候更新、以及哪些情況會被拒絕。本地模式不會建立投影。

## 投影長什麼樣

投影位於工作區裡 Speclink 專用工作目錄下的 context 子目錄，內容有三部分：

- 一份清單檔（manifest.json）：記錄這次快照的識別、政策版本，以及每份文件的摘要值與版本。
- 一份索引（INDEX.md）。
- 一組鏡像 `openspec/` 佈局的文件：設定、共用詞彙、正典規格與變更文件。

投影可以隨時整個目錄刪掉重建。重建後內容等價。

投影一定在 gitignore 的涵蓋範圍內。工作區的 gitignore 沒涵蓋時，Speclink 會補寫 gitignore 並在錯誤輸出印一行警告。它不會把投影悄悄寫成未被忽略的檔案。

## 投影是唯讀的

技能只讀投影，不寫回。在 remote 模式下，apply 與 verify 技能會指示 AI 從投影讀規格，任何規格修改都必須經 speclink 動詞。直接編輯投影不算遠端寫入。

> [!WARNING]
> 直接改投影裡的檔案，之後的驗證會拒絕，錯誤說「投影已被修改或不完整、需要 refresh」，並指出摘要值不符的文件。遠端正典不會被這次修改觸及。清單檔缺失時同樣拒絕。

## 什麼時候更新

投影的來源是 server 的一致快照，不是分次請求拼湊出來的。remote 動詞執行時：

- server 沒有新提交，投影不重寫，檔案不變動。
- server 有新提交，投影更新為新快照。
- server 的 context 服務失敗（例如回 503），動詞照常完成，錯誤輸出印一行「投影未刷新」的警告，既有投影被標記 stale。

更新一律先在暫存目錄產生完整快照，再一次切換成現行投影，不逐檔覆寫。產生或切換失敗時，既有投影完整保留，錯誤指出失敗的階段。

### stale 標記與 refresh

投影被標記 stale 時，Speclink 只寫一個固定名稱的標記檔，投影文件本身不變。讀取端看到標記會提示 refresh。refresh 以新快照全量重建投影並清除標記。清單檔裡的快照識別與 server 現值相同時，refresh 不重寫投影。

## 依流程縮小投影

投影可以依流程只包含需要的文件。沒指定流程時為全量。

| 流程 | 投影內容 |
| --- | --- |
| discuss | 設定、共用詞彙、正典規格索引 |
| propose | 討論、相關正典規格、產出流程與 template |
| apply | 該變更的提案、設計、任務、delta 規格與對應的正典規格 |
| verify | apply 的集合，加最新任務與驗證規則 |
| archive | delta 規格、正典規格、任務、版本 |

remote 模式的 instructions 會把要讀的檔案指向投影下的對應路徑。本地模式的 instructions 輸出不受影響。

remote 模式的整體說明見 [remote 模式總覽](remote-overview.md)。

**出處**：`context-projection`
