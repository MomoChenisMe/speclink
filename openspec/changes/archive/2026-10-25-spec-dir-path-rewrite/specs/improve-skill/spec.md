## MODIFIED Requirements

### Requirement: 掃描精髓段逐字保留

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

渲染產出的 speclink-improve 技能檔 SHALL 完整保留源自 improve-codebase-architecture 的掃描精髓,SHALL NOT 濃縮:(1) 範圍收斂——使用者點名方向(模組/子系統/痛點)時 SHALL 直接採用並跳過推斷;否則 SHALL 以 git log 熱點推斷並加權近期常變區域,輔以 speclink/changes/archive 的 touched 記錄;熱點分散無焦點時 SHALL 放寬網。(2) 掃描 SHALL 列出全部六條 friction 訊號:概念理解需跳多個小模組、interface 複雜度逼近實作的 shallow module、為測試抽純函式但 bug 藏在呼叫端、緊耦合跨 seam 洩漏、難以透過現行 interface 測試的區域、以及分組只靠檔名猜(同一概念的檔案散在平鋪目錄、讀者靠前綴或字母序重拼分組,或目錄與既有設計文件的分層對不上);並 SHALL 保留「有機探索、不逐條打勾」的精神敘述。(3) deletion test SHALL 為前五條訊號的 candidate 准入判準:刪除後複雜度集中才成立,僅搬家不成立。(4) 掃描機制 SHALL 規定 inline 為預設,僅於未指定方向或範圍跨 crate 時派 Explore subagent,且數量硬上限為 2。(5) 第六條訊號 SHALL 帶自己的准入測試,緊接 deletion test 段落敘明:改問讀者預測測試(第一次打開該目錄的人能否不靠搜尋預測功能住哪、哪些檔案是一組),且 candidate 須同時滿足三條件——分組有客觀依據(模組引用方向、既有設計文件的分層、對外連結)、對呼叫者不可見(既有路徑經根層 re-export 維持,無法 re-export 的面以守門測試斷言無殘留舊路徑)、搬移本身即整個變更(需要每個呼叫端改路徑的只是搬家,SHALL NOT 列為 candidate);candidate 同時命中前五條訊號與第六條時 SHALL 先以 deletion test 裁決,刪除後複雜度集中者屬程式碼 candidate、第六訊號只涵蓋原樣搬移者。

#### Scenario: 六條 friction 訊號逐條在列

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的掃描段落
- **THEN** 六條 friction 訊號 SHALL 逐條出現,且 deletion test 以「複雜度集中才算、搬家不算」的判準敘述並限定於前五條

#### Scenario: 第六訊號帶讀者預測測試

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔 deletion test 段落之後的內容
- **THEN** 內容 SHALL 敘明第六訊號不適用 deletion test、改問讀者預測測試,並列出分組有客觀依據、對呼叫者不可見、搬移即整個變更三條件;claude 與 codex 兩工具的技能實例 SHALL 同步含此段

#### Scenario: subagent 上限與觸發判準

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的掃描機制段落
- **THEN** 內容 SHALL 規定 inline 為預設、Explore subagent 僅於未指定方向或跨 crate 時派出、硬上限 2
