## 1. 引擎 lint（speclink-core）

- [x] 1.1 依 design「D1: 判斷式落點」與「D2: 誤置判準」以 TDD 落實 delta「標記位置的 change 驗證檢查」的判斷式：crates/speclink-core/src/tasks.rs 新增公開函式，接收解析後任務清單、回傳誤置清單（任務序號＋描述）——錯型 A（首個空白分隔 token 僅含 ASCII 數字與句點且至少一數字、次 token 為字面 `[M]`）與錯型 B（描述以字面 `[M]` 開頭）。行為結果：delta 的「誤置判定」Example 表逐列成立；描述中段或反引號提及 `[M]` 不命中；已勾與未勾同等檢查。驗證：cargo test -p speclink-core 之 tasks 單元測試綠燈（先紅後綠）。 <!-- speclink-task:tsk_01KZSTNHYXX716XEXYGTMPCV8K -->
- [x] 1.2 依 design「D3: 錯誤訊息契約」以 TDD 接線 change 驗證：crates/speclink-core/src/validate.rs 的 validate_change 讀 tasks.md、呼叫 1.1 判斷式、逐命中報 error——訊息含 tasks.md 邏輯路徑（正斜線）、任務序號、描述前綴引文與正誤例並列，錯型 B 訊息點名 checkbox 後恰一個空格；既有錯誤先列、本檢查後附。行為結果：delta 三個 Scenario（編號在前、行首殘留、正確與中段提及不報）成立；tasks.md 缺席或無命中時驗證輸出逐位元不變。驗證：cargo test -p speclink-core 之 validate 單元測試綠燈（先紅後綠）。 <!-- speclink-task:tsk_01KZSTNHYYQPQJ1HS9HVJPCNXS -->
- [x] 1.3 CLI 整合測試釘住端到端行為：crates/speclink-cli/tests/it 新增案例——含「- [ ] 6.2 [M] …」的 change 跑 validate 結果 invalid 且 stderr/報告含正誤例訊息；格式正確的 change 輸出與本變更前逐位元一致。驗證：cargo test -p speclink-cli --test it（選跑新增案例）綠燈。 <!-- speclink-task:tsk_01KZSTNHYYRX7M2P4BN5VEBW4D -->

## 2. 技能 asset 對比對（assets 三連動）

- [x] 2.1 依 design「D4: 對比對文字與三連動」改寫兩個技能 asset 的 `[M]` 起草指引，落實 delta「手動測試任務的起草標記」（propose-skill）與「ingest 技能的起草標記指引」（manual-task-marker）：crates/speclink-core/assets/skills/propose.md 既有段落改為對比對（正例 `- [ ] [M] 3.2 …` 與誤例 `- [ ] 3.2 [M] …` 並列、一句後果、checkbox 後恰一個空格）；crates/speclink-core/assets/skills/ingest.md 新增同形狀段落。行為結果：兩份 delta 的「對比對指引呈現」與「ingest 補任務時的指引」情境成立。驗證：人工核讀兩檔含正誤例並列；golden 於 2.2 收斂。 <!-- speclink-task:tsk_01KZSTNHYY294YBTNP24ZJGHC0 -->
- [x] 2.2 asset 三連動收斂：跑 golden 對照測試，紅燈即提升 crates/speclink-core/src/init.rs 的 MARKER_VERSION 並再生 golden 與 crates/speclink-core/tests/golden/assets.lock。行為結果：兩份 delta 的「技能模板生成」情境成立（claude 與 codex 兩形含新文字）。驗證：cargo test -p speclink-core --test it 全綠。 <!-- speclink-task:tsk_01KZSTNHYYRZE1HWVYXDA08CS3 -->
- [x] 2.3 本 repo 技能檔再生同步：以更新後引擎執行 speclink update 再生本 repo 技能檔。行為結果：.claude/skills/speclink-propose/SKILL.md 與 ingest 對應技能檔含對比對文字，與 asset 一致。驗證：grep 兩檔含誤例行「- [ ] 3.2 [M]」字樣；git diff 僅涉技能檔。 <!-- speclink-task:tsk_01KZSTNHYYA4DN7QYF8PFBBXBZ -->

## 3. 收尾驗證

- [x] 3.1 受影響面測試全綠且無既有測試語意變動：cargo test -p speclink-core 與 cargo test -p speclink-cli --test it 全數通過；diff 檢視確認未修改任何既有測試的斷言語意（僅允許新增測試與 fixture）。驗證：兩個測試指令全綠、diff 核讀通過。 <!-- speclink-task:tsk_01KZSTNHYYBC4GNHWWM53PMSJ4 -->
