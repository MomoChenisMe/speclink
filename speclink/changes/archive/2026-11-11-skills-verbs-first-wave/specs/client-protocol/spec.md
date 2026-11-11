## ADDED Requirements

### Requirement: 排序請求與回應

Client Protocol SHALL 定義對單一變更的排序請求：對專案底下 changes/<name>/rank 的 POST。請求 payload 為 camelCase：`anchor`（string，錨點變更的名稱）、`position`（string，值為 `before` 或 `after`）與選填的 `force`（boolean，缺席讀作 false）。成功回應的 payload 為 `change`（string）、`anchor`（string）、`position`（string，與請求相同），SHALL NOT 含順序鍵的值；回應 SHALL 帶新的範圍版本。這是寫入請求，SHALL 受與其他寫入相同的權限檢查。守門失敗（錨點不是作用中的變更、兩者相同、不同階段、已有順序鍵而未帶 force、跨越宣告依賴）SHALL 以既有的 refused 回報並帶引擎的訊息；變更不存在 SHALL 以 not_found 回報；看板順序文件被同時改寫 SHALL 以 revision_conflict 回報。這兩個型別 SHALL 在 protocol 的單一定義處宣告。`position` 是其他值時 SHALL 以 invalid_argument 拒絕。

#### Scenario: 排序成功

- **WHEN** 對 changes/c/rank 送出 `{"anchor":"a","position":"before"}`，c 與 a 同為提案中且 c 沒有順序鍵
- **THEN** 回應狀態 200，payload 為 `{"change":"c","anchor":"a","position":"before"}`，並帶新的範圍版本

#### Scenario: force 缺席讀作 false

- **WHEN** 對已有順序鍵的 c 送出不含 force 的排序請求
- **THEN** 回應的 reason 為 refused，訊息含 `already has a board rank`

#### Scenario: 不合法的 position

- **WHEN** 送出 `{"anchor":"a","position":"first"}`
- **THEN** 回應的 reason 為 invalid_argument；看板順序文件不變

#### Scenario: 沒有編輯權限

- **WHEN** 只有讀取權限的成員送出排序請求
- **THEN** 回應是權限錯誤；看板順序文件不變

#### Scenario: 變更不存在

- **WHEN** 對不存在的變更送出排序請求
- **THEN** 回應的 reason 為 not_found
