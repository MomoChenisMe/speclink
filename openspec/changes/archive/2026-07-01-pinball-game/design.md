## Context

彈珠檯需要一個以固定時間步進（fixed timestep）驅動的即時模擬，在 Canvas 2D 上以每秒 60 幀繪製。所有邏輯放在單一 `pinball/index.html`，無框架、無建置。座標系以左上為原點、y 向下為正，重力為正 y。

## Goals / Non-Goals

**Goals:**
- 穩定可玩的重力物理與碰撞手感。
- 兩翻板、三緩衝器、發球、扣球、Game Over/重開、HUD。

**Non-Goals:**
- tilt、音效資產、多關卡、連線功能。

## Decisions

### Physics Integration

每一幀對球速加上重力（`vy += GRAVITY * dt`），再以 `pos += vel * dt` 更新位置；牆壁碰撞以反射法向速度並乘上反彈係數 `WALL_RESTITUTION`（約 0.8）。以子步進（sub-steps）降低高速穿牆。

### Flipper Model

每個翻板有 `restAngle` 與 `raisedAngle` 兩個目標角，按鍵按下時角度朝 raised 內插、放開朝 rest 內插。碰撞偵測用「點到線段距離」判斷球是否接觸翻板線段；接觸時沿翻板法向反彈並加上與翻板角速度相關的向上衝量 `FLIPPER_IMPULSE`。

### Bumper Collision

緩衝器為圓；當球心與緩衝器圓心距離小於半徑和時，沿兩心連線方向把球推出並施加額外速度 `BUMPER_BOOST`，同時 `score += BUMPER_SCORE`（100）並觸發短暫閃光計時器。

### Launch And Serve

球初始停在右側發球道（`state = 'waiting'`）；按 Space 給予向上初速 `LAUNCH_VELOCITY` 並切為 `state = 'inPlay'`。落袋後若尚有球則重新 serve 到發球道。

### Game Loop

以 `requestAnimationFrame` 驅動，累積時間並以固定 `DT` 呼叫 `update()`，再 `render()`。輸入以 `keydown/keyup` 維護一組 `keys` 旗標，`update()` 讀取旗標驅動翻板與發球。

### HUD Rendering

在 Canvas 上以 `fillText` 繪製分數與剩餘球數；Game Over 時覆蓋半透明面板與提示文字。

## Implementation Contract

- **可觀察行為**：載入 `pinball/index.html` 後顯示球檯、HUD（Score/Balls）與待發球；Space 發球、方向鍵打翻板、撞緩衝器加分、落袋扣球、0 球顯示 Game Over、R 重開。
- **介面/資料形狀**：全域 `state` 物件含 `ball{pos,vel,radius,state}`、`flippers[2]`、`bumpers[3]`、`score`、`balls`、`over`。
- **失敗模式**：球高速時不得穿牆（用子步進）；發球道待發時忽略重複 Space；Game Over 後不再 serve。
- **驗收標準**：可於瀏覽器完成「發球→打擊→加分→落三球→Game Over→R 重開」完整循環。
- **範圍邊界**：僅 `pinball/index.html`；不改動 speclink 引擎程式碼。

## Risks / Trade-offs

- 高速穿牆 → 以固定 DT 子步進與連續碰撞近似緩解。
- 翻板手感調校 → 將常數集中於檔頭方便微調。
