## ADDED Requirements

### Requirement: 討論鏈結動詞對壞 change metadata 拒絕

speclink discuss link 與 speclink discuss seal 在對象 change 的 `.openspec.yaml` 存在但 YAML 解析失敗時 SHALL 以帶檔案位置與解析原因的錯誤拒絕：SHALL NOT 寫入該 change 的 metadata，也 SHALL NOT 改動討論記錄。壞 metadata SHALL NOT 被解讀為「無 from_discussion 鏈」或「無 restale 旗標」——拒絕原因 SHALL 是 metadata 損壞，而非既有守衛的「鏈不存在」誤導訊息。

#### Scenario: link 對壞 metadata 拒絕且兩側皆不寫

- **WHEN** 執行 speclink discuss link 給定一份未封存討論的 slug 與一個 `.openspec.yaml` 為壞 YAML 的既有變更名
- **THEN** 以非零 exit code 結束；該 `.openspec.yaml` 與 discussions/ 下該討論記錄皆逐位元不變

#### Scenario: seal 對壞 metadata 拒絕且不誤報鏈缺失

- **WHEN** 對壞 metadata 的變更執行 speclink discuss seal（兩個位置參數：討論 slug 與該變更名）
- **THEN** 以非零 exit code 結束；stderr 指出 metadata 檔損壞（而非 from_discussion 不含該 slug）；兩側檔案逐位元不變
