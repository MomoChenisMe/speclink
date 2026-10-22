## ADDED Requirements

### Requirement: 變更 schema 寫入端點

server SHALL 提供 `POST /changes/{name}/schema`，body 為 `{ schema: string }`，經 Command gateway 直通引擎的 schema 切換命令，editor role 限定（reader 回 403）。change 不存在 SHALL 回 404。schema 解析不到、要切到沒有 specs 產出的 schema 而變更含 delta spec、變更有未結的驗證工單時 SHALL 回 HTTP 409、reason 為 refused（沿用封閉 error reason registry，SHALL NOT 新增 reason 值），message SHALL 為引擎原文，逐字與 fs 模式 `speclink change schema` 的 stderr 相同。schema 解析 SHALL 只看內建（server 不讀任何 client 或 server 本機的 user 層目錄）。守門失敗 SHALL 零寫入。實際改動 meta 時 SHALL 發布領域事件、scope revision 前進並發布 invalidate；與現值相同時 SHALL HTTP 200、零寫入、零事件、revision 不前進。回應 SHALL 為 `{ change: string, schema: string, previous: string, changed: boolean, verifyStampCleared: boolean }`，`verifyStampCleared` 表示這次寫入移除了驗證章，CLI 遠端模式據此印出與 fs 模式相同的第二行。

#### Scenario: editor 切換 schema

- **WHEN** editor 對 schema 為 spec-driven、沒有 delta 的 tidy 呼叫 `POST /changes/tidy/schema`，body `{ schema: "no-spec-change" }`
- **THEN** HTTP 200，body 為 `{ change: "tidy", schema: "no-spec-change", previous: "spec-driven", changed: true, verifyStampCleared: false }`，meta 的 `schema:` 行改變、其餘逐字元保留，事件發布且 revision 前進

#### Scenario: reader 被拒

- **WHEN** reader 呼叫同一端點
- **THEN** HTTP 403，meta 不變

#### Scenario: 不存在的 change

- **WHEN** editor 呼叫 `POST /changes/ghost/schema`，scope 內沒有 ghost
- **THEN** HTTP 404，reason 為 not_found，revision 不前進

#### Scenario: 守門失敗 409

- **WHEN** 變更 c 含 auth 的 delta spec，editor 分別以 `{ schema: "no-spec-change" }` 與 `{ schema: "no-such-flow" }` 呼叫 `POST /changes/c/schema`
- **THEN** 兩次皆 HTTP 409、reason 皆為 refused，message 與 fs 模式 `speclink change schema` 對應情境的 stderr 逐字相同，meta 皆不變、revision 不前進

#### Scenario: 冪等切換

- **WHEN** 對 schema 已是 spec-driven 的 c 以 `{ schema: "spec-driven" }` 呼叫
- **THEN** HTTP 200、`changed` 為 false，零寫入、零事件、revision 不前進
