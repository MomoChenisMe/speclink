## 1. 測試基礎設施（TDD 地基）

- [x] 1.1 建立 pinball/tests.js 測試 harness：mock document/canvas 2d context/localStorage/AudioContext/requestAnimationFrame，讀取 pinball/index.html 抽出 <script> 內容以 node vm 執行。完成時 node pinball/tests.js 可執行並輸出 harness 自檢 PASS（能取得 window.__pinball 掛鉤）。驗證：執行 node pinball/tests.js。
- [x] 1.2 在 pinball/index.html 的 IIFE 尾端匯出測試掛鉤 window.__pinball（含 step、addScore、playSound、serveBall、resetGame、ball/score/multiplier 等狀態存取器），瀏覽器玩家行為不變。驗證：node pinball/tests.js 的 harness 自檢 PASS；手動開啟 index.html 遊戲照常可玩。

## 2. 音效（audio-feedback）

- [x] 2.1 實作 Requirement: Synthesized Sound Effects — 依 design 決策「音效以單一 AudioContext 即時合成」實作 playSound(name)，name 為 flipper/bumper/sling/drain/gameover 五種，於對應事件（翻板按下、碰 bumper、碰 slingshot、落失、最後一球落失）呼叫；不載入任何外部音檔。驗證：tests.js 先寫失敗測試（各事件觸發後 mock AudioContext 記錄到對應合成呼叫）再實作至 PASS；手動在瀏覽器按 ArrowLeft/A 聽到拍擊音。
- [x] 2.2 實作 Requirement: Mute Toggle — keydown 處理加入 M 鍵切換 muted 旗標，muted 時 playSound 直接返回不建節點。驗證：tests.js 測試按 M 後觸發 bumper 命中不產生任何合成呼叫、再按 M 恢復；手動按 M 後打 bumper 無聲。

## 3. 視覺回饋（hit-effects）

- [x] 3.1 實作 Requirement: Particle Burst On Hit — 依 design 決策「粒子系統用單一陣列加上限」新增 spawnParticles(x,y,color) 與 step() 內的粒子更新（dt 衰減、上限 150 丟最舊），bumper/slingshot 命中時噴至少 10 顆、600ms 內消散。驗證：tests.js 測試命中後 particles.length>=10、推進 0.6s 後歸零、灌爆時不超過 150；手動觀察命中噴粒子。
- [x] 3.2 實作 Requirement: Hit Flash — bumper 與 slingshot 命中時 flash 計時器設 0.15s，render() 期間以亮色繪製、逾時恢復。驗證：tests.js 測試命中後 flash>0 且 0.3s 內回到 0；手動觀察命中閃光。

## 4. 新檯面元件（pinball-table）

- [x] 4.1 實作 Requirement: Slingshot Rebound And Scoring — 依 design 決策「slingshot 與 drop target 重用線段碰撞」新增左右兩個 slingshot 斜線段，step() 內以 collideSegment(seg, 1.1, 260) 反彈並經 addScore(75) 計分。驗證：tests.js 先寫失敗測試（把球放在 slingshot 面上，step 後速度反向且 score 增加 75）再實作至 PASS；手動觀察球撞左右下側斜面被彈開且加分。
- [x] 4.2 實作 Requirement: Drop Target Bank — 檯面中段三個 drop targets（alive 旗標），命中即消失並 addScore(150)，第三個倒下時加平坦 2000 bonus（不乘倍率）並全部重置為 standing。驗證：tests.js 測試依序擊倒三個後 score 含 2000 bonus 且三個 targets 全部 alive；手動觀察擊中消失、全清後重置。

## 5. Combo 倍率（combo-scoring）

- [x] 5.1 實作 Requirement: Combo Multiplier 與修改後的 Requirement: Bumper Scoring — 依 design 決策「combo 以秒計時、先遞增後計分」實作 addScore(base)：comboTimer>0 時 multiplier=min(multiplier+1,5)、重設 comboTimer=3.0、得分 base*multiplier；step() 內 comboTimer-=dt 歸零重置 multiplier=1；bumper 計分改走 addScore(100)。驗證：tests.js 測試 1 秒間隔三連擊得 100/200/300、逾時 3 秒後回 x1、六連擊封頂 x5。
- [x] 5.2 實作 Requirement: Multiplier HUD Display — render() 的 HUD 區塊顯示目前倍率（COMBO xN）。驗證：tests.js 測試 render 後 mock ctx 的 fillText 呼叫含 "x2"（倍率為 2 時）；手動連擊觀察 HUD 倍率變化。

## 6. 高分榜（high-scores）

- [x] 6.1 實作 Requirement: Persistent High Score Table — 依 design 決策「高分榜存 localStorage JSON」實作 loadHighScores()/saveHighScore(score)（key "pinball.highscores"，降冪、截斷 3 筆，try/catch 降級），最後一球落失進 game over 時呼叫。驗證：tests.js 測試 [5000,3000,1000] 插入 4000 得 [5000,4000,3000]、低分不改表、localStorage 拋例外時遊戲不崩潰。
- [x] 6.2 實作 Requirement: High Score Display At Game Over — render() 的 game over 面板列出前 3 名（HIGH SCORES）。驗證：tests.js 測試 game over 後 fillText 呼叫含高分數字；手動打完一局觀察榜單。

## 7. 暫停（pause-control）

- [x] 7.1 實作 Requirement: Pause Toggle — 依 design 決策「暫停跳過物理步而非停止 rAF」新增 paused 旗標與 P 鍵切換（game over 時無效），paused 時主迴圈 frame() 清空 acc 不執行 step()（球、分數、combo 計時、粒子全凍結），render() 疊 PAUSED 文字。驗證：tests.js 測試按 P 後 step 推進球位置不變、combo 倍率不因暫停超時而重置、再按 P 恢復、fillText 含 "PAUSED"；手動按 P 觀察畫面凍結與 PAUSED 字樣。

## 8. Nudge 與 TILT（pinball-table，中途新增）

- [x] 8.1 實作 Requirement: Nudge Impulse — 依 design 決策「nudge 以固定水平衝量、TILT 以滑動窗計數」實作 nudge()：N 鍵在球 inPlay、未暫停、未結束、未 TILT 時給球固定 90 px/s 水平衝量（球在左半場 +90、右半場 -90，推向檯面中心）。驗證：tests.js 測試球 inPlay 按 N 後 vx 恰變 ±90、waiting/paused 時按 N 速度不變；手動按 N 觀察球被輕推。
- [x] 8.2 實作 Requirement: Tilt Lockout — 以遊戲時間記錄 nudge 時刻（3000ms 滑動窗），第 4 次觸發 tilted=true：updateFlippers 目標鎖 rest（翻板失靈）、nudge 無效、render() HUD 顯示 TILT；drain 時 tilted=false 並清空紀錄，下一球恢復。驗證：tests.js 測試 3 秒內按 N 四次 → tilted、持 ArrowLeft 翻板角度不變、fillText 含 "TILT"、落失後下一球翻板恢復且無 TILT；手動連按 N 觀察 TILT 與翻板失靈。

## 9. 整合驗證

- [x] 9.1 全量回歸：node pinball/tests.js 全部 PASS（既有行為：發球、翻板、bumper、落失、game over、R 重開不受破壞），並手動在瀏覽器完整玩一局逐項核對七項功能（音效/M、粒子閃光、slingshot 與 drop targets、COMBO HUD、高分榜、P 暫停、N nudge 與 TILT）。驗證：測試輸出無 FAIL；手動 checklist 全數通過。
