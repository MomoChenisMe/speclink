## MODIFIED Requirements

### Requirement: 單 change 讀取回應攜帶 show 組合欄位

GET /changes/{name} 的回應 SHALL 攜帶七個選填欄位供 client 端 show 讀取與詳情呈現組合（皆 camelCase、serde default）：created SHALL 僅於該 change meta 同時具有 schema 與 created 時出現（引擎 ShowChange 的成對回報規則）；fromDiscussions SHALL 為 meta 的 from_discussion 鏈（空清單即省略）；deltaCapabilities SHALL 為該 change 的 delta spec capability 名單（空清單即省略）；createdBy、createdWith、startedAt、startedBy SHALL 各為 meta 的 created_by、created_with、started_at、started_by（缺席即省略）。七欄皆由 server 於既有路由自 meta 與 scope 文件組裝；舊 server 不送這些欄位時，client 的對應區塊 SHALL 維持缺席、SHALL NOT 偽造預設值。

#### Scenario: show 組合欄位隨單 change 讀取上 wire

- **WHEN** 對 meta 含 schema、created 與 from_discussion，且帶有一個 delta spec 的 change 呼叫 GET /changes/{name}
- **THEN** 回應含 created（等於 meta 的 created）、fromDiscussions（含該討論 slug）與 deltaCapabilities（含該 capability）；對 meta 無 created 且無鏈、無 delta spec 的 change，三個鍵皆不出現於回應

#### Scenario: meta 歸屬欄位隨單 change 讀取上 wire

- **WHEN** 對 meta 含 created_by、created_with 且已蓋開工章（started_at 與 started_by）的 change 呼叫 GET /changes/{name}，再對四欄皆缺的 change 呼叫同端點
- **THEN** 前者回應含 createdBy、createdWith、startedAt、startedBy 四鍵且值等於 meta；後者四鍵皆不出現

##### Example: 欄位組裝

| change meta 與文件 | GET /changes/{name} 額外欄位 |
| ------------------ | ---------------------------- |
| `schema: spec-driven`＋`created: 2026-07-29`＋`from_discussion: auth-scope`＋`specs/auth/spec.md` | `"created":"2026-07-29","fromDiscussions":["auth-scope"],"deltaCapabilities":["auth"]` |
| 上列再加 `created_by: Demo <d@e.com>`＋`started_at: 2026-08-25T00:00:00Z`＋`started_by: Demo <d@e.com>` | 額外再含 `"createdBy":"Demo <d@e.com>","startedAt":"2026-08-25T00:00:00Z","startedBy":"Demo <d@e.com>"` |
| `schema: spec-driven`（無 created、無鏈、無 delta spec、無歸屬欄位） | （七鍵皆缺席） |

## ADDED Requirements

### Requirement: 變更清單回應攜帶建立者與來源討論欄位

GET /changes 的清單項 SHALL 沿既有的逐筆 meta 組裝路徑（startedAt 的同一條）補三個選填欄位：createdBy、created 與 fromDiscussions，語意與 wire 欄位定義一致（缺席或空清單即省略）。meta 解析失敗的 change SHALL 維持既有 metaError 容錯路徑，三欄不出現、清單不失敗。

#### Scenario: 清單項攜帶建立者與來源討論

- **WHEN** scope 內有一個 meta 含 created_by、created 與 from_discussion 的 change 與一個三者皆缺的 change，呼叫 GET /changes
- **THEN** 前者清單項含 createdBy、created 與 fromDiscussions；後者三鍵缺席；回應整體成功

### Requirement: 討論列表回應攜帶 promotedTo

GET /discussions 的每筆討論 SHALL 由 server 於 route 邊緣以引擎的 promoted_to 查詢函式組裝 promotedTo 欄位（空清單即省略）；引擎的討論列表結構與 CLI 的 discuss list --json 輸出 SHALL 維持逐位元不變。查詢失敗的單筆討論 SHALL 以欄位缺席容錯、列表不失敗。

#### Scenario: 已轉出與未轉出討論的列表欄位

- **WHEN** scope 內有一筆 promoted_to 含兩個 change 名的討論與一筆未轉出的討論，呼叫 GET /discussions
- **THEN** 前者含 promotedTo 且順序沿 frontmatter 累加順序；後者無 promotedTo 鍵；本地 CLI 的 discuss list --json 輸出與改動前逐位元相同
