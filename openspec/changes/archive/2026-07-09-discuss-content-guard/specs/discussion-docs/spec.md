## ADDED Requirements

### Requirement: 討論內容寫入動詞拒絕空內容

討論內容寫入動詞（context、add-round、conclude）SHALL 在其內容去除前後空白後為空時以錯誤中止，SHALL NOT 靜默寫入空的 Context／Round／Conclusion；conclude 遇空內容 SHALL NOT 將討論 status 翻為 concluded。錯誤訊息 SHALL 指出內容為空並提醒可能漏帶 --stdin。此空內容檢查 SHALL 置於引擎（core::discuss）以覆蓋所有前門（本地 CLI、遠端 CLI、桌面）。CLI SHALL 於標準輸入為管線（非互動終端）時讀取其內容作為動詞內容，不論是否帶 --stdin 旗標；--stdin 旗標 SHALL 維持被接受以相容既有腳本。add-round SHALL 維持純附加，SHALL NOT 提供改寫既有輪的動詞；context 與 conclude SHALL 維持重跑即覆寫該區段的既有行為。

#### Scenario: add-round 空內容中止且不新增輪

- **WHEN** 對某討論執行 add-round，而提供的內容去除前後空白後為空（例如漏帶 --stdin 且無管線輸入）
- **THEN** 指令以錯誤中止、不新增任何 Round，錯誤訊息指出內容為空並提醒 --stdin

#### Scenario: conclude 空內容中止且不翻狀態

- **WHEN** 對某討論執行 conclude，而內容去除前後空白後為空
- **THEN** 指令以錯誤中止、不寫入空 Conclusion，且該討論 status 不翻為 concluded

#### Scenario: context 空內容中止且不覆寫

- **WHEN** 對某討論執行 context，而內容去除前後空白後為空
- **THEN** 指令以錯誤中止、不以空內容覆寫既有 Context 區段

#### Scenario: 管線內容於未帶 --stdin 時仍被寫入

- **WHEN** 以管線將非空內容送入 add-round，但未帶 --stdin 旗標
- **THEN** 該內容被當作輪內容寫入，不因漏帶旗標而靜默成空

#### Scenario: 空內容 guard 覆蓋桌面前門

- **WHEN** 桌面經內嵌引擎對某討論以空內容執行任一內容寫入動詞
- **THEN** 該寫入以與 CLI 相同的 core guard 被拒，不產生空區段
