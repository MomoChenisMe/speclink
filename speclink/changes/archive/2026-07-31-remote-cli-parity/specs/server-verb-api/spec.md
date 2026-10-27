## ADDED Requirements

### Requirement: 討論寫入動詞端點補齊

server SHALL 補齊討論的寫入動詞端點：POST /discussions 的請求 SHALL 接受選填 slug 欄位並轉傳引擎（未帶時行為與現行完全相同，非法值由引擎拒絕並映射為語義化錯誤、不落檔）；DELETE /discussions/{slug} SHALL 以 force query 參數直通引擎 discard（0 輪即刪、有輪無 force 拒絕），並比照 change 刪除做 editor 限定（reader 收 403、scope 零改動）；POST /discussions/{slug}/link 與 POST /discussions/{slug}/seal SHALL 以 body 攜帶 change 名稱直通對應引擎命令。四者皆為 unit of work：成功寫入時 scope revision 前進、事件照引擎 outcome 發布；討論或 change 不存在時 SHALL 回 404 與語義化訊息。

#### Scenario: 建立討論轉傳 slug

- **WHEN** 以合法 slug 欄位呼叫 POST /discussions（topic 為中文）
- **THEN** HTTP 200，回應的 slug 為覆寫值、topic 為原文，server store 以該 slug 建檔；非法 slug 時回語義化錯誤且不落檔

#### Scenario: 討論 discard 的 guard 經端點生效

- **WHEN** 對 0 輪討論呼叫 DELETE /discussions/{slug}，再對有輪討論呼叫同端點（無 force）
- **THEN** 前者刪除成功、scope revision 前進；後者被拒且記錄完整保留、revision 不前進；帶 force=true 重呼叫則刪除成功

#### Scenario: reader 的討論刪除被拒

- **WHEN** 以 reader role 憑證呼叫 DELETE /discussions/{slug}
- **THEN** HTTP 403、reason 機器可判為權限不足，該討論完整保留

#### Scenario: link 與 seal 直通引擎

- **WHEN** 依序呼叫 POST /discussions/{slug}/link 與 POST /discussions/{slug}/seal，body 帶既有 change 名稱
- **THEN** 兩者 HTTP 200：link 後 change meta 的 from_discussion 鏈含該 slug，seal 後討論標記 promoted；不存在的討論或 change 回 404 與語義化訊息

### Requirement: 單 change 讀取回應攜帶 show 組合欄位

GET /changes/{name} 的回應 SHALL 攜帶三個選填欄位供 client 端 show 讀取組合（皆 camelCase、serde default）：created SHALL 僅於該 change meta 同時具有 schema 與 created 時出現（引擎 ShowChange 的成對回報規則）；fromDiscussions SHALL 為 meta 的 from_discussion 鏈（空清單即省略）；deltaCapabilities SHALL 為該 change 的 delta spec capability 名單（空清單即省略）。三欄皆由 server 於既有路由自 meta 與 scope 文件組裝；舊 server 不送這些欄位時，client 的 show 對應區塊 SHALL 維持缺席、SHALL NOT 偽造預設值。

#### Scenario: show 組合欄位隨單 change 讀取上 wire

- **WHEN** 對 meta 含 schema、created 與 from_discussion，且帶有一個 delta spec 的 change 呼叫 GET /changes/{name}
- **THEN** 回應含 created（等於 meta 的 created）、fromDiscussions（含該討論 slug）與 deltaCapabilities（含該 capability）；對 meta 無 created 且無鏈、無 delta spec 的 change，三個鍵皆不出現於回應

##### Example: 欄位組裝

| change meta 與文件 | GET /changes/{name} 額外欄位 |
| ------------------ | ---------------------------- |
| `schema: spec-driven`＋`created: 2026-07-29`＋`from_discussion: auth-scope`＋`specs/auth/spec.md` | `"created":"2026-07-29","fromDiscussions":["auth-scope"],"deltaCapabilities":["auth"]` |
| `schema: spec-driven`（無 created、無鏈、無 delta spec） | （三鍵皆缺席） |

### Requirement: 變更開工標記端點

server SHALL 提供 POST /changes/{name}/in-progress，經 Command gateway 直通引擎的開工標記命令：change 存在且未開工時以呼叫者認證身分蓋 started_at 與 started_by 進 meta、發布領域事件、scope revision 前進；change 不存在或已開工時 SHALL 維持引擎的靜默成功語意——HTTP 200、零文件寫入、零事件、revision 不前進。

#### Scenario: 首次蓋章寫入與事件

- **WHEN** 對未開工的 change 呼叫 POST /changes/{name}/in-progress
- **THEN** HTTP 200，meta 新增 started_at 與 started_by（呼叫者認證身分）、既有欄位逐字元保留，事件發布且 revision 前進

#### Scenario: 重複與未知名稱皆靜默成功

- **WHEN** 對已開工的 change 與不存在的 change 名稱各呼叫一次該端點
- **THEN** 兩者皆 HTTP 200，server 零文件寫入、零事件、revision 不前進，已開工者的首章逐字元保留
