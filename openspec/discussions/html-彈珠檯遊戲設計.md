---
topic: HTML 彈珠檯遊戲設計
slug: html-彈珠檯遊戲設計
status: concluded
created: 2026-07-01
---

# Discussion: HTML 彈珠檯遊戲設計

<!-- Rounds are appended below as the discussion evolves. -->

## Round 1 — assumptions (2026-07-01)

掃描程式庫：專案內尚無任何遊戲程式碼（只有 speclink 引擎），因此採 Assumptions 模式列出設計假設。

1. **單一 HTML 檔、Canvas 2D**：整個遊戲放在 `pinball/index.html`，用 `<canvas>` + 原生 JS，無建置步驟、無外部相依。
   - 證據：openspec/config.yaml 的 context 指定「single self-contained HTML file / no build step」。
   - 若錯：改用框架會增加相依與建置複雜度，違背專案約束。
2. **重力 + 速度積分的簡易剛體物理**：每幀對球施加重力、更新速度與位置，與牆壁/擋板/緩衝器碰撞用反彈係數（restitution）。
   - 若錯：純事件式碰撞無法呈現真實彈珠手感。
3. **兩個底部翻板（flippers）**：ArrowLeft/ArrowRight 控制左右翻板上抬打擊球。
   - 若錯：少了翻板就不是彈珠檯。
4. **發球道 + Space 發球、3 顆球、落袋（drain）扣球、計分**。
   - 若錯：缺少關卡循環，玩家無目標。

開放問題：翻板要用固定角度切換還是連續轉動？緩衝器幾顆、加多少分？

## Round 2 — interview (2026-07-01)

針對開放問題收斂：

- **翻板**：採「固定兩角度即時切換」（休息角 / 上抬角），按鍵按下切到上抬角、放開回休息角；碰撞時對球施加額外向上衝量，手感足夠且實作單純（不需完整轉動慣量）。
- **緩衝器**：3 顆圓形緩衝器置於上半場，撞擊反彈並 +100 分、播放簡短閃光。
- **計分/球數**：起始 3 顆球，落袋 -1 球，歸零則 Game Over 並可按 R 重新開始；分數即時顯示於 HUD。
- **發球**：右側發球道，Space 給球一個向上初速射入場地。
- **tilt/傾斜**：本版不做（YAGNI），列為 Non-Goal。

介面深度檢查：本題為單檔遊戲、無跨模組接縫、無新儲存抽象，跳過深度檢查。

## Conclusion

**Decision**: 以單一 `pinball/index.html`（Canvas 2D + 原生 JS）實作彈珠檯：重力物理、兩個即時切換角度的翻板、3 顆緩衝器（+100 分）、發球道、3 顆球與 Game Over/重開，HUD 顯示分數與剩餘球數。
**Rationale**: 符合 config 的單檔無相依約束；固定角度翻板與反彈係數物理在複雜度與手感間取得平衡；tilt 列為 Non-Goal 以控制範圍。
**Capture to**: proposal（範圍與能力）、design（物理與遊戲迴圈決策）、spec（玩家可觀察行為）、tasks（實作步驟）。
**Next**: /speclink:propose --from-discussion html-彈珠檯遊戲設計
