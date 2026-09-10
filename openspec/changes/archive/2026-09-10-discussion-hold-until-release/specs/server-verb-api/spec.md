## MODIFIED Requirements

### Requirement: 討論列表回應攜帶 concluded

GET /discussions 的每筆討論 SHALL 由 server 於 route 邊緣以引擎的討論投影組裝 concluded 與 hold 兩個欄位（皆 camelCase 布林、恆填 true 或 false）：concluded 為 true 即該討論的 Conclusion 段已寫入內文（scaffold 佔位註解不算）；hold 為 true 即該討論的 frontmatter 帶 `hold: true` 行。GET /discussions/{slug} 回應的 info SHALL 攜帶同樣兩個欄位。判準 SHALL 只由引擎的 frontmatter 解析提供，server SHALL NOT 重讀檔或重複實作。引擎的討論列表結構與 CLI 的 discuss list --json 輸出 SHALL 維持逐位元不變。查詢失敗的單筆討論 SHALL 以欄位缺席容錯、列表不失敗。

#### Scenario: 已結論與未結論討論的列表欄位

- **WHEN** scope 內有一筆已寫入結論的 promoted 討論與一筆 Conclusion 仍為佔位註解的 promoted 討論，呼叫 GET /discussions
- **THEN** 前者含 concluded: true、後者含 concluded: false；本地 CLI 的 discuss list --json 輸出與改動前逐位元相同

#### Scenario: 保留中討論的列表欄位

- **WHEN** scope 內有一筆 frontmatter 帶 `hold: true` 的 promoted 討論與一筆無 hold 行的 open 討論，呼叫 GET /discussions；再對前者呼叫 GET /discussions/{slug}
- **THEN** 列表中前者含 hold: true、後者含 hold: false；單筆回應的 info 含 hold: true；本地 CLI 的 discuss list --json 輸出與改動前逐位元相同
