## ADDED Requirements

### Requirement: 封存前的讀取與 delta 改寫經動詞

archive 技能在封存前的每一個讀取 SHALL 指名動詞：任務的完成數取自 speclink list --json；變更有哪些 delta 取自同一輸出的 deltaCapabilities；delta 的內容以 speclink artifact cat specs/<capability> 讀取；正式規格以 speclink show <capability> --item-type spec 讀取。需要改寫 delta 時 SHALL 以共用的寫回做法（讀出全文、以 speclink new artifact spec 帶 --stdin --force 整份寫回）進行，SHALL NOT 指示就地編輯檔案；技能 SHALL 維持「只改 delta、不改正式規格」的規則。封存完成後 SHALL 以封存動詞回傳的封存名稱告知使用者。封存名稱衝突時改名或刪除既有封存資料夾的步驟、收尾的 git commit 提醒與手冊資料夾的提醒 SHALL 只在規格資料夾位於這個 checkout 時產生；不位於時技能 SHALL 指示停下來並告訴使用者這一步要在持有規格的 Server 上處理。

#### Scenario: delta 的檢查不用路徑

- **WHEN** 檢視產生的 archive 技能中檢查 delta 與正式規格是否一致的步驟
- **THEN** 該步驟以 deltaCapabilities、speclink artifact cat 與 speclink show 表述，不含規格資料夾的路徑

#### Scenario: 改寫 delta 經整份寫回

- **WHEN** 檢視產生的 archive 技能中修正 delta 的步驟
- **THEN** 該步驟指向共用的寫回做法，並保留「不改正式規格」的規則

#### Scenario: remote 專案封存後的告知

- **WHEN** 以「規格資料夾不在這個 checkout」產生 archive 技能，檢視封存成功後的輸出範本
- **THEN** 範本以封存名稱告知結果，不含規格資料夾的路徑，也不含 git commit 的提醒

#### Scenario: remote 專案遇到封存名稱衝突

- **WHEN** 以「規格資料夾不在這個 checkout」產生 archive 技能，檢視封存名稱已存在時的步驟
- **THEN** 步驟指示停下來並告訴使用者要在 Server 上處理，不含改名或刪除資料夾的指示
