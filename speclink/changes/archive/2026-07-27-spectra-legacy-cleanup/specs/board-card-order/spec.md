## MODIFIED Requirements

### Requirement: board_rank 不進 CLI 輸出且既有輸出逐位元不變

`board_rank` SHALL 為桌面看板專用欄位：speclink list --json、speclink discuss list --json 及對應人眼輸出 SHALL NOT 出現 rank 相關欄位，且對含 `board_rank` 的 repo，上述輸出 SHALL 與同一 repo 移除全部 `board_rank` 欄位後的輸出逐位元一致（項目順序與欄位皆不變）。本需求為輸出凍結敏感：既有輸出基線 SHALL 維持位元級一致。

#### Scenario: 含 rank 的 repo 之 CLI 輸出不變

- **WHEN** repo 內數個 change 的 .openspec.yaml 與討論 frontmatter 含 `board_rank`，執行 speclink list --json 與 speclink discuss list --json
- **THEN** 兩者輸出與移除全部 `board_rank` 後執行的輸出逐位元相同，payload 不含 rank 欄位
