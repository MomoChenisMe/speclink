---
topic: improve-pinball
slug: improve-pinball
status: promoted
promoted_to: improve-pinball
created: 2026-07-02
---

# Discussion: improve-pinball

<!-- Rounds are appended below as the discussion evolves. -->

## Round 1 — assumptions (2026-07-02)

Round 1 — 範圍界定（scope）

Assumptions（依 codebase scout：pinball/index.html 為唯一相關源檔，IIFE、Canvas 2D、固定步進 1/120s）：
1. **改進範圍鎖定六項玩法/回饋功能**：音效、粒子視覺回饋、新檯面元件（slingshot x2 + drop targets x3）、combo 倍率、localStorage 高分榜、暫停鍵。
   Evidence: pinball/index.html 目前僅有 bumper 計分與基本 HUD，無音效/粒子/暫停/持久化。
   If wrong: 範圍過大會拖累單檔可維護性，過小則玩法深度不足。
2. **既有操作不可變**：ArrowLeft/Right、A/L、Space、R 保持原義；新增鍵 M（靜音）、P（暫停）不與其衝突。
   Evidence: index.html keydown handler 現用 arrowleft/arrowright/a/l/Space/r。
   If wrong: 破壞既有玩家肌肉記憶，違反正典 spec 的 Flipper Control / Ball Launch 需求。
3. **一切必須留在單一 HTML 檔內、無外部依賴**：音效用 WebAudio 合成，不用音檔；粒子用 Canvas 2D。
   Evidence: openspec/config.yaml 的 design rule「no build step, no external asset」。
   If wrong: 引入資產或建置步驟會違反專案約束。
4. **正典 spec pinball-table 已存在**：新能力應以獨立 capability spec 表述，僅計分/檯面行為的變化以 MODIFIED pinball-table 表述。
   Evidence: openspec/specs/pinball-table/spec.md 有 6 個 Requirement 且 @trace 指向 index.html。
   If wrong: delta 錯放會讓 archive 時正典 spec 更新錯亂。

Open questions → 留待 Round 2：音效合成拓撲、粒子上限、combo 計時基準、高分榜資料形狀、暫停對固定步進主迴圈的介入點。

## Round 2 — assumptions (2026-07-02)

Round 2 — 技術取捨（technical trade-offs）

1. **音效：單一 AudioContext + 每音效即時建 Oscillator/GainNode**，不預先生成 buffer。首次使用者手勢時 resume()（瀏覽器 autoplay policy）。M 鍵切 muted 旗標，靜音時直接略過合成。
   取捨：即時合成最簡、零資產；缺點是音色簡單 — 對街機音效足夠。
2. **粒子：單一 particles 陣列 + 上限（~150），每幀 dt 衰減**；命中時噴 10~14 顆 + 短暫 flash 疊圈。取捨：物件池 vs 簡單陣列 — 選簡單陣列，60fps 下數量級足夠、單檔可讀性優先。
3. **新元件：slingshot 以現有線段碰撞（collideSegment）重用 + 高恢復係數與固定外推衝量；drop targets 以矩形/線段碰撞、hit 後 active=false**。全清 +2000 bonus 後全部重置。取捨：重用既有碰撞 helper，不引入新幾何系統。
4. **Combo：以秒為單位的 comboTimer（3.0s），每次 bumper/slingshot/target 命中 → multiplier=min(multiplier+1,5) 並重置計時器**；逾時 multiplier=1。加分一律 base*multiplier。HUD 右上顯示 xN。取捨：frame 計數 vs 秒 — 用秒（dt 累加），與固定步進一致（180 frames @60fps = 3s）。
5. **高分榜：localStorage key "pinball.highscores"，JSON 陣列前 3 名（數字）**。game over 時插入排序、截斷 3 筆並顯示於 game over 面板。讀寫皆 try/catch（隱私模式可能丟例外）。
6. **暫停：P 鍵切 paused 旗標；step() 不執行但 render() 照跑並疊 PAUSED 文字**。取捨：停 rAF vs 跳過 step — 選跳過 step，rAF 續跑可維持 last 時戳處理與畫面疊字，恢復時不會累積 dt 尖峰（恢復時重設 acc）。

結論方向：六項全部可在單檔內以既有架構（固定步進 + collideSegment 重用）落地，無需新抽象層（interface depth check：無新 seam，皆掛在既有 game loop 上）。

## Conclusion

**Decision**: 對 pinball/index.html 做六項改進，全部維持單一 HTML 檔、無外部依賴、Canvas 2D、60fps 固定步進，既有操作（ArrowLeft/Right、A/L、Space、R）不變：
1. 音效 — WebAudio 即時合成（flipper 拍擊、bumper 命中、slingshot、球落失、game over），M 鍵切換靜音，不使用外部音檔。
2. 視覺回饋 — bumper/slingshot 命中時粒子爆發（上限陣列）與短暫閃光。
3. 新檯面元件 — slingshot x2（左右下側，反彈+計分）、drop targets x3（擊中消失，全清 +2000 bonus 並重置）。
4. Combo 倍率 — 3 秒內連續命中 bumper/slingshot/target 遞增 x1→x5，逾時重置；HUD 顯示目前倍率。
5. 高分榜 — localStorage 保存前 3 名，game over 時更新並顯示（try/catch 防隱私模式）。
6. 暫停 — P 鍵切換，暫停時跳過物理 step、畫面顯示 PAUSED。

**Rationale**: 全部功能重用既有固定步進迴圈與 collideSegment/collideBumper helper，無需新架構 seam；WebAudio 合成與 localStorage 皆為瀏覽器原生能力，符合零依賴約束。計時類行為以秒（dt 累加）表述，對應 60fps 基準。

**Capture to**: proposal（範圍）、specs delta（新能力各一 spec + pinball-table MODIFIED）、design（合成拓撲/粒子上限/碰撞重用/計時基準）、tasks。

**Next**: /speclink-propose --from-discussion improve-pinball
