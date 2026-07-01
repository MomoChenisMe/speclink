## 1. 場景與遊戲迴圈

- [x] 1.1 建立 `pinball/index.html` 骨架與 Game Loop（requestAnimationFrame 固定步進 update/render）。驗證：瀏覽器開啟後 Canvas 有球檯與待發球，主迴圈穩定執行。
- [x] 1.2 實作 Physics Integration：重力、速度積分與牆壁反彈（含子步進防穿牆）。驗證：發球後球受重力下墜、撞牆反彈不穿出。

## 2. 玩法互動

- [x] 2.1 實作 Launch And Serve 與 Ball Launch：右側發球道待發、Space 給予向上初速、發球中重複 Space 忽略。驗證：按 Space 球射入場地，再按無效。
- [x] 2.2 實作 Flipper Model 與 Flipper Control：ArrowLeft/ArrowRight 切換翻板角度並對接觸球施加向上衝量。驗證：按方向鍵翻板上抬並能把下墜球打回。
- [x] 2.3 實作 Bumper Collision 與 Bumper Scoring：三顆緩衝器反彈球、每次 +100 分並短暫閃光。驗證：撞緩衝器球彈開且 HUD 分數增加。

## 3. 關卡循環與介面

- [x] 3.1 實作 Ball Drain And Lives：球落袋扣一球、尚有球則重新 serve。驗證：讓球落下，剩餘球數減一並回到發球道。
- [x] 3.2 實作 Game Over And Restart：0 球顯示 Game Over 面板、按 R 重置分數與球數並重新發球。驗證：落完三球出現 Game Over，按 R 可重玩。
- [x] 3.3 實作 HUD Rendering 與 Score Display：即時顯示分數與剩餘球數。驗證：分數/球數變動時 HUD 同步更新。
