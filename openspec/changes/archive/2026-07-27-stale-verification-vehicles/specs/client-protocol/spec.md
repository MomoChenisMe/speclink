## MODIFIED Requirements

### Requirement: typed client 全面取代 raw JSON 旁路

speclink-remote 與 CLI remote 攔截層的 wire payload 處理 SHALL 全數經 protocol DTO；SHALL NOT 殘留以通用 JSON 值重組回應的路徑。ETag 與 If-Match SHALL 以型別攜帶：帶 If-Match 的寫入在 revision 不符時 SHALL 得到 revision_conflict。remote 模式全部現行動詞的人眼輸出、--json 輸出與 exit code SHALL 與重構前逐位元一致。

#### Scenario: 寫入攜 If-Match 且衝突可辨

- **WHEN** typed client 以既知 ETag 執行寫入動詞而 stub server 判定 revision 已前進
- **THEN** 請求標頭含 If-Match；client 收到 revision_conflict reason 並對映現行衝突訊息

#### Scenario: remote 輸出凍結

- **WHEN** 執行 `crates/speclink-cli/tests/remote_read_path.rs` 對 stub server 與 fs 模式雙跑同一動詞的全部對照情境
- **THEN** remote 與 fs 模式的 `--json` 欄位形狀（key 集合）一致，全部對照情境全綠
