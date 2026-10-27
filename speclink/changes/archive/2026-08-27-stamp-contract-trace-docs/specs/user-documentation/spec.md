## ADDED Requirements

### Requirement: 品質站蓋章效果與非成員錯誤碼的文件揭露

中英文 workflow 文件 SHALL 說明品質站蓋章的工單消耗效果：蓋章於同一原子寫入內寫入章欄位並刪除工單（review.md 與 verify.md）、封存的已蓋章 change 不含工單檔、僅未結工單經 carry 旗標隨封存移動、fs 模式的工單文字僅存於 git 歷史、remote 模式蓋章後工單文字不可回讀。中英文 remote-getting-started SHALL 以 403（permission_denied）描述已登入非成員讀取專案資源的結果，SHALL NOT 寫成 404。中英文 verb-contract 參考文件的本質本機動詞列 SHALL 含 demo 與 trace 兩者。

#### Scenario: 讀者可自 workflow 查得蓋章後工單去向

- **WHEN** 讀者於 workflow 文件的品質站段落查詢蓋章效果
- **THEN** 文件說明蓋章刪除工單、封存不含工單檔、remote 模式工單文字不可回讀，讀者不需翻正典即可得知 show 於蓋章後回報無工單為預期行為

#### Scenario: 非成員錯誤碼敘述與正典一致

- **WHEN** 讀者依 remote-getting-started 的 membership 段落驗證非成員讀取行為
- **THEN** 文件敘述的狀態碼為 403，與 server-identity 正典及實際 server 行為一致

#### Scenario: 本質本機動詞列表完整

- **WHEN** 讀者於 verb-contract 參考文件查閱動詞模式分岔表
- **THEN** FsOnly 列同時含 demo 與 trace，且與 CLI dispatch 的實際宣告一致
