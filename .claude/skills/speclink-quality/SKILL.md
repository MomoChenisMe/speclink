---
name: speclink-quality
description: 兩個品質站（review＋verify）一次跑完的時序編排：兩站檢查都先不蓋章，findings 統一修完後各自複驗、兩章接連蓋，再封存。當使用者要對同一個 change 同時跑 review 與 verify、或說「兩站都跑」時使用。只跑一站時不要用本技能，直接呼叫該站技能。
---

# 兩站品質關卡（本機編排技能）

**定位**：本機技能、非引擎正典（不進 skills.rs、不進 golden、不三處同步）。兩站的檢查內容、工單、蓋章語意完全由 `/speclink-review` 與 `/speclink-verify` 各自承載；本技能只管**時序**。出處：討論 `cross-station-staleness`（2026-08-04 定案）。

**為什麼需要這個時序**：站的章會凍結範圍檔的內容指紋，蓋章後任何修改——包括另一站 findings 的修正——都會把章打黃（「已審查·其後有變動」）。把兩站的蓋章都壓到所有修正之後，兩章到封存都是綠的，且各站的 validation 輪會驗收到雙站全部修正（validation patch＝上輪凍結點到現值的全部差異，不分修正出自誰）。

**前提**：change 任務全數完成（兩站的成品檢查都要求這個；verify 的工單引擎守門會直接拒絕未完成的 change）。

## 步驟

1. **review 檢查、先不蓋章**：跑 `/speclink-review`。到收尾三選項時選「Stop without stamping」（先不蓋章結束），工單與凍結 snapshot 保留。
   - 若 discovery 零 findings：站內正典規定當場自動蓋章，不要攔——之後若被 verify 側修正打黃，屬已知暫態（討論記為 Deferred），封存會定格回綠。
2. **verify 檢查、先不蓋章**：跑 `/speclink-verify`，同樣選「先不蓋章結束」。
   - verify 的工單與章要等 change `verify-station-parity` 落地；在那之前 verify 只有對話報告——本步驟照跑，只是無章可留，流程照樣成立。
3. **統一修正**：兩站 findings 合併 triage，一次修完。修正一律回主線、照專案 TDD 慣例；站的 fork 不改任何檔案。
4. **review 複驗＋蓋章**：重新跑 `/speclink-review`。validation 輪自動涵蓋上輪凍結點以來的全部修正（含 verify 引發的），必修集合清空後站內迴圈自動蓋章。
5. **verify 複驗＋蓋章**：重新跑 `/speclink-verify`，同樣 validation → 蓋章。兩章接連落、中間零編輯。
6. **封存**：`/speclink-archive`。

## 邊界情況

- **事後變卦**（某站已蓋章才決定加跑另一站）：不要重做已蓋的站——照跑新站、接受舊章暫時變黃。封存側只看有章沒章、不重算凍結度，進封存頁即回綠。重蓋＝重新完整 discovery，不成比例。
- **單站或都不跑**：不經本技能；技能預設（修完即蓋）即正確，蓋完沒有他站修正會來打黃。
