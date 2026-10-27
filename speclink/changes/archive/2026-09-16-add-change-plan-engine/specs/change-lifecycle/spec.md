## MODIFIED Requirements

### Requirement: meta 新欄位向後相容

change meta 的解析 SHALL 對缺少 started_* 欄位的既有檔案維持既有行為：所有讀取 meta 的指令與查詢 SHALL 正常運作、該 change 視為未開工，SHALL NOT 產生任何警告或錯誤。change meta 的 `depends_on` 欄位（頂層逗號清單，列出前置 change 名）缺席時 SHALL 讀作無依賴；含此欄位時 list 與 status 的人眼輸出與 --json 輸出 SHALL 與不含時逐位元一致。

#### Scenario: 舊 meta 檔正常解析且視為未開工

- **WHEN** 對 meta 僅含 schema 與 created_* 欄位（無 started_*）的 change 執行 speclink list --json 與 speclink status --change 該 change
- **THEN** 兩指令輸出與遷移前版本位元級一致，exit code 為 0，無警告

#### Scenario: depends_on 不影響既有輸出

- **WHEN** 對 meta 含 `depends_on: other` 的 change 執行 speclink list --json 與 speclink status --change 該 change --json
- **THEN** 兩指令輸出與該行不存在時逐位元一致，exit code 為 0
