## ADDED Requirements

### Requirement: 預設十二條的問號說明

「查看12條」展開後，十二個引擎提供的名稱 SHALL 各有可點擊的問號，呈現用途、例子及預設只列建議的效果；英文名稱不翻譯，說明隨zh-TW／en。相同名稱在審查工單的套用規則區 SHALL 使用同一說明。問號 SHALL 可用鍵盤開啟、Escape關閉且返回原焦點；未知名稱 SHALL 顯示無此版本說明，不顯示別條內容。extend／replace的原設定及嚴重度行為 SHALL 不變。

#### Scenario: 十二條逐一查看

- **WHEN** 使用者展開清單並以滑鼠或鍵盤開啟任一問號，再切換介面語言
- **THEN** 十二條均有對應用途與例子，名稱不變，說明依語言切換；關閉後焦點返回該問號

#### Scenario: 整份換掉仍能理解預設清單

- **WHEN** 使用者選replace並查看十二條
- **THEN** 說明明示該預設清單未套用，不改寫專案規則或工單

### Requirement: 納入安全驗收與封存紀錄的設定

Desktop SHALL 在安全主開關下提供預設未勾選的「納入驗收條件」checkbox，保存security_verification.required_for_acceptance；主開關關閉時停用但保留勾選值並顯示暫不生效。勾選說明 SHALL 明列中／高／嚴重風險須修復或記錄接受理由、LOW不阻擋、必要檢查未完成不能通過。SHALL NOT 在儲存時自動執行審查。產出流程 SHALL 提供審查／驗證／安全審查三個預設勾選的「封存時保留紀錄」選項，保存check_records.archive，並說明封存前皆保存、關閉不刪舊歷史、安全驗收摘要仍保存。唯讀、編輯、取消、快捷儲存及未知欄位保值 SHALL 沿用既有設定行為。

Local與Remote SHALL 共用政策；Remote reader唯讀，editor保存帶expected revision。衝突、離線、401或寫入失敗 SHALL 不採用新值，保留草稿並提供既有衝突／重試流程。UI SHALL 提示required但全部標準或三種審查皆關閉時不能通過安全驗收，不替使用者勾選標準。

#### Scenario: 參考模式改驗收模式

- **WHEN** 使用者啟用主開關、勾選納入驗收並儲存後重開
- **THEN** checkbox仍勾選、顯示通過條件，沒有自動掃描；取消編輯則原值不變

#### Scenario: 封存保留設定與遠端衝突

- **WHEN** 使用者只取消security的封存保留並儲存，或Remote保存遇revision衝突
- **THEN** 前者只有security欄位變false且既有報告仍可查，後者草稿保留而政策不被覆蓋
