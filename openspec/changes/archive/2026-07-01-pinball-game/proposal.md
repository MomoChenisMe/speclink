## Why

我們要提供一個可直接在瀏覽器開啟、零安裝的彈珠檯小遊戲，展示 speclink SDD 流程能從討論一路走到可玩成品。來源為討論 `html-彈珠檯遊戲設計` 的結論。

## What Changes

- 新增單一自包含檔案 `pinball/index.html`（Canvas 2D + 原生 JS，無建置步驟、無外部相依）。
- 實作重力物理與反彈係數碰撞（球對牆壁、翻板、緩衝器）。
- 實作兩個底部翻板，以 ArrowLeft/ArrowRight（或 A/L）即時切換休息／上抬角度打擊球。
- 實作 3 顆緩衝器，撞擊 +100 分並短暫閃光。
- 實作發球道與 Space 發球、3 顆球、落袋扣球、Game Over 與 R 重新開始。
- HUD 即時顯示分數與剩餘球數。

## Non-Goals (optional)

- 不做 tilt／傾斜偵測（YAGNI）。
- 不做多關卡、音效資產或連線排行榜。

## Capabilities

### New Capabilities

- `pinball-table`: 玩家可觀察的彈珠檯行為——發球、翻板打擊、緩衝器計分、落袋扣球與遊戲結束/重開。

### Modified Capabilities

(none)

## Impact

- **Code**: `pinball/index.html`（新增）
- **Dependencies**: 無（僅使用瀏覽器內建 Canvas API）
- **Behavior**: 玩家於瀏覽器開啟即可用鍵盤遊玩完整一局彈珠檯
