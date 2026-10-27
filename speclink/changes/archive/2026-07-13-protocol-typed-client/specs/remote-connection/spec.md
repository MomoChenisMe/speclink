## ADDED Requirements

### Requirement: remote 動詞經 handshake 建立的連線語境執行

remote 模式的動詞執行 SHALL 以 binding handshake 建立的連線語境為前置：handshake 成功後動詞請求 SHALL 自動攜帶確認過的 project 與 repo 身分；handshake 因 API version 不相容、binding 缺失、無權限或多義而失敗時，動詞 SHALL 以非零 exit code 停止並輸出指向原因的錯誤，SHALL NOT 回退為未驗證的逐 verb 呼叫。連線設定（.speclink.yaml 的 remote 區段）的欄位與語意 SHALL 不變。

#### Scenario: handshake 失敗動詞即停

- **WHEN** stub server 的 handshake 回應為 binding 多義（兩個候選 repo），隨後執行任一 remote 動詞
- **THEN** 動詞以非零 exit code 結束、stderr 指出 binding 多義與候選清單；無動詞請求被送出

#### Scenario: 設定欄位不變

- **WHEN** 以現行 .speclink.yaml 的 remote 區段（url 與 repo key）啟動 remote 動詞且 handshake 成功
- **THEN** 動詞行為與輸出與現行一致；設定檔無需任何修改
