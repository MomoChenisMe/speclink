## Why

現有彈珠檯（pinball/index.html）只有基本物理與 bumper 計分，缺乏聽覺/視覺回饋、玩法深度（新元件與連擊）、成績持久化與暫停能力，遊戲體驗單薄。本變更源自已收斂的討論 improve-pinball，一次補齊六項玩家可感知的改進；實作中途依使用者新增需求，追加第七項 Nudge+TILT（輕推檯面與防濫用鎖定）。

## What Changes

- 音效：以 WebAudio 即時合成五種音效（flipper 拍擊、bumper 命中、slingshot 命中、球落失、game over），按 M 鍵切換靜音；不使用任何外部音檔。
- 視覺回饋：bumper 與 slingshot 命中時產生粒子爆發與短暫閃光。
- 新檯面元件：左右下側各一個 slingshot（反彈並計分）、檯面中段三個 drop targets（擊中即消失並計分，三個全清獲得 +2000 bonus 並全部重置）。
- Combo 倍率：3 秒內連續命中 bumper/slingshot/drop target 時倍率由 x1 遞增至上限 x5，逾時重置為 x1；HUD 顯示目前倍率。
- 高分榜：以 localStorage 保存前 3 名分數，game over 時更新並顯示於結算畫面。
- 暫停：按 P 鍵暫停/續玩，暫停時畫面顯示 PAUSED 且物理停止推進。
- Nudge 與 TILT（中途新增）：按 N 鍵輕推檯面，給進行中的球一個固定的小水平衝量；3 秒內使用超過 3 次即觸發 TILT——本球翻板失靈直到落失，HUD 顯示 TILT。
- 既有輸入不變：ArrowLeft/ArrowRight（及 A/L）翻板、Space 發球、R 重新開始維持原行為；新增按鍵 M（靜音）、P（暫停）、N（nudge）不與其衝突。
- 物理不變：重力、固定步進（1/120s x 2 substeps）、牆/翻板/bumper 碰撞恢復係數維持既有數值；slingshot 與 drop target 重用既有線段/圓形碰撞流程。

## Capabilities

### New Capabilities

- `audio-feedback`: WebAudio 合成音效（五種遊戲事件音）與 M 鍵靜音切換。
- `hit-effects`: 命中 bumper/slingshot 時的粒子爆發與短暫閃光回饋。
- `combo-scoring`: 3 秒連擊倍率 x1→x5 的計分加成與 HUD 倍率顯示。
- `high-scores`: localStorage 前 3 名高分榜，game over 時更新並顯示。
- `pause-control`: P 鍵暫停/續玩與 PAUSED 畫面提示。

### Modified Capabilities

- `pinball-table`: 新增檯面元件需求 — slingshot x2（反彈+計分）與 drop targets x3（擊中消失、全清 +2000 bonus 並重置）；HUD 計分顯示納入倍率後的加分結果；新增 Nudge（N 鍵水平衝量）與 Tilt（3 秒內超過 3 次 nudge 觸發、本球翻板失靈、HUD 顯示 TILT）需求。

## Impact

- Affected specs: 新增 audio-feedback、hit-effects、combo-scoring、high-scores、pause-control；修改 pinball-table（含中途新增的 Nudge Impulse / Tilt Lockout）。
- Affected code:
  - Modified: pinball/index.html（單一檔案內新增：音效模組、粒子系統、slingshot/drop target 幾何與碰撞、combo 狀態、localStorage 高分榜、暫停旗標與輸入處理、nudge/TILT 狀態機、HUD 與 game over 畫面擴充）
  - New: pinball/tests.js（node 可執行的 mocked-DOM 測試，驗證上述行為）
  - Removed: 無
- 相依性：無新外部依賴；僅使用瀏覽器原生 WebAudio 與 localStorage。維持單一 HTML 檔、無建置步驟。
