## ADDED Requirements

### Requirement: list --json 曝變更的 delta capability

speclink list --json SHALL 於帶 delta 規格的變更 payload 曝 deltaCapabilities 欄位：該變更 delta 規格的 capability 名稱字串陣列，升冪，取自 Store 的 delta capability 清單；沒有 worktree 映射時與 speclink show <change> --json 的 deltaSpecs 同源，主 checkout 的 worktree 觀察面生效時，映射成立的變更與 list 其他欄位一樣取自該 worktree 副本（show 仍讀主副本）。變更沒有 delta 規格時 SHALL 省略該欄位，以維持 list --json 對無 delta 變更的既有輸出逐位元不變。人眼 speclink list 輸出 SHALL 不變。remote 模式 SHALL 輸出同形欄位，值取自 GET /changes 清單項的 deltaCapabilities；server 未送該欄位時 SHALL 省略，SHALL NOT 偽造值。

#### Scenario: list 曝 deltaCapabilities 且對無 delta 變更輸出不變

- **WHEN** 對含一個帶 client-protocol 與 server-verb-api 兩份 delta 規格的變更、與一個沒有 delta 規格的變更的專案執行 speclink list --json 與 speclink list
- **THEN** 前者的 --json payload 含 "deltaCapabilities": ["client-protocol", "server-verb-api"]；後者的 --json payload 無 deltaCapabilities 鍵；人眼 speclink list 輸出不含 deltaCapabilities 字樣

#### Scenario: remote 模式同形

- **WHEN** remote 模式下 GET /changes 的清單項帶 deltaCapabilities: ["cap-a"]，執行 speclink list --json
- **THEN** 該變更 payload 含 "deltaCapabilities": ["cap-a"]，欄位名集合與 fs 模式下帶 cap-a delta 規格的同名變更一致
