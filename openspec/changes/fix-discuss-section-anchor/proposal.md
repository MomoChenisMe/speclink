## Why

外部協作測試發現討論記錄輪次錯亂：Round 1 標題下空白、兩輪內文全部落在 Round 2 之下且順序顛倒。根因是章節掃描把輪內文裡任何以「## 」開頭的行都誤認為結構標題——add_round 的插入點、set_context 與 conclude 的區段替換、conclusion_text 的讀取、UI 的區段切分全部共用同一條錯誤規則。此病不限 remote 模式，本機同樣可觸發；更危險的是偽結論會經 conclusion_text 餵進 promote 的提案前置內容。（源討論：remote-fix-plan-gaps，刀 1）

## What Changes

- 引擎 discuss 模組的 section_body_range 改為只認固定結構標題（行首整行為「## Context」「## Rounds」「## Conclusion」）作為區段終點；輪內文的其他「## 」行不再截斷區段。
- 寫入端跳脫：add_round、set_context、conclude 落盤前，把內容中與結構撞名的行（整行為結構標題，或行首為「### Round 」「## Round 」前綴）以 markdown 反斜線跳脫，關閉殘餘歧義，同時杜絕 count_rounds 計數膨脹與跳號。
- count_rounds 收緊為僅認合法輪標題形狀（「### Round <編號> — <mode> (<日期>)」；保留 pre-scaffold 的「## Round 」容忍）。
- UI 的 splitDiscussionSections 同步改為結構標題白名單，與引擎同一條規則。
- 回歸測試：引擎補「輪內文含 ## 行」的多輪順序、conclude 不落輪內、計數不膨脹三類；UI 測試檔補區段切分同場景。
- 正典 discussion-docs 的 add-round 純附加要求現況只有敘述、無可驗場景——補釘住 scenario。

## Non-Goals

- 不做既有損壞記錄的修復工具（沿 remote-task-evidence「不回填、不追溯」先例）；已損壞的記錄人工重建。
- 不改 splitRounds 對非法「### 」行整篇退回單一檢視的既有行為——那是刻意的降級路徑，不是資料損壞。
- 不做「所有 # 開頭內容行全面跳脫」——只跳脫會撞結構的行，維持內容最小改動。
- 不動討論文件骨架（三段結構與 HTML 註解照舊），不需資料遷移。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `discussion-docs`: add-round 的純附加語意補上可驗場景——輪內文含結構撞名行時，新輪仍落在 Rounds 區段尾端、先前輪的內文留在原標題下、結論永不寫進輪內；並明定寫入端跳脫行為。

## Impact

- Affected specs: discussion-docs
- Affected code:
  - Modified: crates/speclink-core/src/discuss.rs, packages/ui/src/components/DiscussionDrawer.tsx, packages/ui/src/__tests__/discussionDrawer.test.tsx
  - New: (none)
  - Removed: (none)
