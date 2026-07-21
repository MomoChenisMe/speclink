## MODIFIED Requirements

### Requirement: 分頁身分為 WorkspaceLocator 而非 root 路徑

Desktop 分頁 SHALL 以 WorkspaceLocator 為身分：local 變體攜帶 root 路徑，remote 變體（connectionId／projectId／repoId／可選 checkoutRoot）經 chooser 或 remote marker 探測的 handshake 成功路徑建構，checkoutRoot 由 checkout 綁定流程寫入且不參與分頁身分（locator key 不含 checkoutRoot——同 scope 重綁不同 checkout 為同一分頁、新值覆寫舊值）。分頁去重、活躍分頁記錄與 tray 選單識別 SHALL 一律經 locator key（local 為 local:{root}），SHALL NOT 再以裸 root 字串比對。local 分頁的 UI 可觀察行為（分頁列呈現、切換、關閉、上限淘汰、tray 顯示）SHALL 與 root 字串時代一致。

#### Scenario: 同一專案重複開啟仍去重

- **WHEN** 使用者對已在分頁列的資料夾再次執行開啟
- **THEN** 分頁列不新增條目，既有分頁更新顯示名並成為活躍分頁，與重構前行為一致

#### Scenario: 同 scope 重綁 checkout 不分裂分頁

- **WHEN** 對已開啟的 remote 分頁以另一資料夾重新完成 checkout 綁定
- **THEN** 分頁列仍為同一分頁，checkoutRoot 更新為新資料夾
