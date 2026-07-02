## Context

pinball/index.html 是單一自足 HTML 檔：IIFE 內含固定步進物理（DT=1/120s、SUBSTEPS=2，等效 60fps 畫面下每幀 4 個子步）、線段碰撞（closestOnSeg/collideSegment）與圓形 bumper 碰撞（collideBumper）、canvas HUD。既有物理常數：GRAVITY=1400 px/s^2、WALL_RESTITUTION=0.78、BUMPER_RESTITUTION=1.05、BUMPER_BOOST=120 px/s、FLIPPER_IMPULSE=780 px/s、MAX_SPEED=2400 px/s、BUMPER_SCORE=100 分。本設計在不動這些常數與既有輸入（ArrowLeft/Right、A/L、Space、R）的前提下加入六項能力；實作中途追加第七項 Nudge+TILT。

## Goals / Non-Goals

**Goals:**

- 七項能力（音效、粒子回饋、slingshot/drop targets、combo、高分榜、暫停、nudge+TILT）全部落在 pinball/index.html 單檔內。
- 重用既有碰撞流程與固定步進迴圈；所有計時類邏輯以 dt 秒累加（3000ms = 180 frames @60fps）。
- 行為可被 node + mocked DOM 測試（pinball/tests.js）驗證。

**Non-Goals:**

- 不改既有物理常數、翻板手感、檯面外框幾何。
- 不引入建置步驟、外部資產（音檔/圖檔）、框架或模組系統。
- 不做多球（multiball）、觸控支援、音量滑桿、nudge 的畫面震動特效。

## Decisions

### 音效以單一 AudioContext 即時合成

每個音效 = OscillatorNode(+GainNode 包絡) 即時建立、播完即棄；不預先產 buffer、不用音檔。音色以 type/頻率掃描區分：flipper（square 短促 240→180Hz, 0.06s）、bumper（triangle 660Hz, 0.08s）、slingshot（sawtooth 330Hz, 0.07s）、drain（sine 300→80Hz 下滑, 0.4s）、game over（sine 三連降音 392/330/262Hz, 0.9s）。替代方案：AudioBuffer 預生成 — 被拒，程式碼更長且無感知差異。AudioContext 於第一次按鍵手勢時建立/resume（autoplay policy）。muted 旗標為 true 時直接 return，不建節點。

### 粒子系統用單一陣列加上限

particles 陣列存 {x,y,vx,vy,life,maxLife,color}，命中時 spawn 12 顆（速度隨機放射、life 0.35~0.6s），每個物理步以 dt 衰減、重力減半作用；陣列滿 150 顆時丟棄最舊。替代方案：物件池 — 被拒，數量級小、GC 壓力可忽略，可讀性優先。閃光重用既有 bumper flash 模式（flash 計時器 0.15s，繪製時切亮色）。

### slingshot 與 drop target 重用線段碰撞

- slingshot x2：左右下側 inlane 上方的斜線段（左：(M+18,H-252)→(cx-pivotSep-26,flipperY-64)；右：鏡像 (370,408)→(284,500)）。以 collideSegment(seg, 1.1, 260) 處理 — 恢復係數 1.1、固定外推衝量 260 px/s，命中得分 base 75。與牆的差異僅在恢復係數、衝量與計分掛勾。
- drop targets x3：檯面中段 y=300 三個短水平線段（寬 26px、間距 16px）。alive=true 時參與 collideSegment(seg, 0.9, 0) 並在命中時 alive=false、得分 base 150；三個皆倒下時立即 +2000（不吃倍率）並全部 alive=true。替代方案：矩形 AABB 碰撞 — 被拒，會引入第二套碰撞幾何，線段已足夠。

### combo 以秒計時、先遞增後計分

comboTimer（秒）與 multiplier（1..5）。任一 bumper/slingshot/drop-target 命中：若 comboTimer>0 則 multiplier=min(multiplier+1,5)；重設 comboTimer=3.0；該次得分 = base * multiplier。每個物理步 comboTimer-=dt，歸零時 multiplier=1。加分統一走 addScore(base) helper，bonus 2000 直接加在 score 上不乘倍率。HUD 於 BALLS 下方顯示 COMBO xN。

### 高分榜存 localStorage JSON

key = "pinball.highscores"，值為降冪數字陣列（<=3 筆）。讀寫皆包 try/catch（隱私模式/檔案協定可能丟例外），失敗時以空陣列運作、不中斷遊戲。game over 時 push 當局分數、sort 降冪、slice(0,3) 寫回，並在 game over 面板列出（HIGH SCORES 1./2./3.）。替代方案：存物件含名字/日期 — 被拒，範圍只要求分數前 3 名。

### 暫停跳過物理步而非停止 rAF

paused 旗標；P 鍵在遊戲未結束時切換。主迴圈照常 rAF，但 paused 時將 acc 清零且不執行 step()（物理、combo 計時、粒子、flash 全部凍結；step() 開頭亦加 paused 守衛以利測試直接驅動），render() 照跑並疊半透明 PAUSED 文字。替代方案：cancelAnimationFrame — 被拒，恢復時要重建時戳且無法畫暫停疊層。暫停中 Space/翻板鍵/N 鍵不影響遊戲狀態。

### nudge 以固定水平衝量、TILT 以滑動窗計數

N 鍵在球 inPlay、未暫停、未結束、未 TILT 時觸發 nudge：ball.vx += NUDGE_IMPULSE(90 px/s)，方向推向檯面水平中心（球在左半 → +90，右半 → -90），確定性便於測試。TILT 判定：以遊戲時間（step 內 dt 累加的 gameTime）記錄每次 nudge 時刻，保留最近 3000ms 內的紀錄，第 4 次（超過 3 次）觸發 tilted=true。tilted 期間 updateFlippers 目標鎖定 rest（翻板失靈）、nudge 無效、HUD 顯示 TILT；球落失（drain）時 tilted=false、nudge 紀錄清空。替代方案：即時時鐘 Date.now() — 被拒，暫停時會誤計時且測試不可控；用 step 累加的遊戲時間與暫停凍結天然一致。

## Implementation Contract

- 行為總覽：玩家聽到五種事件音（M 靜音切換）、看到命中粒子與閃光、可打 2 個 slingshot 與 3 個 drop targets（全清 +2000 並重置）、3 秒連擊倍率 x1→x5 顯示於 HUD、game over 顯示 localStorage 前 3 名、P 暫停顯示 PAUSED、N 輕推檯面且 3 秒內超過 3 次觸發 TILT（本球翻板失靈、HUD 顯示 TILT、落失解除）。
- 介面/資料形狀：
  - addScore(base) — 命中計分入口，回傳實得分數 base*multiplier 並維護 comboTimer/multiplier。
  - playSound(name) — name ∈ {"flipper","bumper","sling","drain","gameover"}；muted 時無副作用。
  - particles 陣列元素 {x,y,vx,vy,life,maxLife,color}；上限 150。
  - slingshots：[{x1,y1,x2,y2,flash}] x2；targets：[{x,y,w,alive}] x3。
  - nudge()：施加 ±90 px/s 水平衝量並記錄時刻；tilted 布林 + nudge 時刻陣列（3000ms 滑動窗）。
  - localStorage key "pinball.highscores" = JSON 降冪數字陣列（長度 <=3）。
  - 全域測試掛鉤：IIFE 尾端輸出 window.__pinball = { state 存取與 step/addScore/nudge 等函式 }，供 node 測試驅動（單檔內、瀏覽器行為不受影響）。
- 失敗模式：AudioContext 建構失敗或 localStorage 拋例外 → 靜默降級（無聲/不持久化），遊戲照常。粒子超限 → 丟最舊，不丟幀。TILT 中的輸入 → 靜默忽略（不報錯）。
- 驗收標準：node pinball/tests.js 全 PASS（mock document/canvas/localStorage/AudioContext/rAF）；瀏覽器手動驗證各鍵行為與畫面。
- 範圍界線：in scope = pinball/index.html 與 pinball/tests.js；out of scope = 物理常數調整、檯面外框改動、其他任何檔案。

## Risks / Trade-offs

- [autoplay policy 使首個音效被瀏覽器擋下] → AudioContext 延遲到第一次 keydown 手勢建立並 resume。
- [localStorage 在隱私模式拋例外] → 所有讀寫包 try/catch，降級為當次 session 記憶。
- [粒子過多拖慢固定步進] → 150 顆硬上限 + 丟最舊策略。
- [slingshot 衝量把球打出界] → 既有 MAX_SPEED=2400 px/s 夾制仍然生效，外推衝量 260 px/s 遠低於上限。
- [nudge 連按濫用改變玩法平衡] → TILT 滑動窗鎖定（超過 3 次/3 秒即翻板失靈），與實體彈珠檯慣例一致。
- [測試掛鉤 window.__pinball 洩漏內部狀態] → 僅唯讀引用與函式，瀏覽器玩家不受影響；換取 node 可測性。
