## ADDED Requirements

### Requirement: 討論隨變更廢棄解鏈

speclink discard 廢棄變更時，對變更 meta 的 from_discussion 清單中每份討論 SHALL 解鏈：自記錄 frontmatter 的 promoted_to 逗號清單移除該變更名、其餘值保留；清單仍有值時狀態 SHALL 維持 promoted；清單因此變空時 SHALL 移除 promoted_to 行並回退狀態——記錄的 Conclusion 區非空時回 concluded、為空時回 open。記錄的 Context、Rounds 與 Conclusion 區內容 SHALL 逐位元不變。slug 無對應記錄時 SHALL 跳過且不視為錯誤。解鏈 SHALL 於刪除變更目錄前完成；對已解鏈的討論重跑 SHALL 冪等（變更名已不在 promoted_to 即不改檔）。

#### Scenario: 最後連結死亡回退 concluded

- **WHEN** 廢棄的變更名是某份有結論討論 promoted_to 的唯一值
- **THEN** 該記錄的 promoted_to 行消失、status 回 concluded；Context、Rounds 與 Conclusion 逐位元不變

##### Example: 回退前後的 frontmatter

- **GIVEN** 討論 alpha-search 的 frontmatter 含 status: promoted 與 promoted_to: cut-a，Conclusion 區非空
- **WHEN** 執行 speclink discard cut-a
- **THEN** frontmatter 變為 status: concluded 且無 promoted_to 行

#### Scenario: 仍有其他變更時維持 promoted

- **WHEN** 廢棄的變更名只是某討論 promoted_to 逗號清單的其中一員
- **THEN** 清單移除該名、其餘值保留；status 維持 promoted

#### Scenario: 無結論的討論回退 open

- **WHEN** 廢棄的變更名是某份 Conclusion 區為空的討論（open 狀態經 link 併入）promoted_to 的唯一值
- **THEN** 該記錄的 promoted_to 行消失、status 回 open

#### Scenario: 多來源討論逐一解鏈

- **WHEN** 廢棄 from_discussion 清單含兩份討論的變更
- **THEN** 兩份記錄各自依上述規則處理：一份因清單空而回退、另一份仍被其他變更引用則僅縮減清單並維持 promoted

#### Scenario: 缺失記錄跳過

- **WHEN** 廢棄的變更 from_discussion 指向的某 slug 無對應記錄（live 與 archive 皆無）
- **THEN** 該 slug 跳過、其餘討論照常解鏈；指令不因此失敗
