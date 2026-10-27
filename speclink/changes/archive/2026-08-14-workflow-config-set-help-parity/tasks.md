## 1. 紅燈：先讓 help 與接受鍵的落差被測出來

- [x] 1.1 在 crates/speclink-cli/tests/it/workflow_config.rs 新增 help 與接受鍵集合的對照測試（依 design 決策 D4: 紅燈測試落在既有的 workflow_config.rs，不引入第三份清單）：測試從同一支 binary 取 speclink workflow-config set --help 的 stdout 與 speclink workflow-config set theme dark 的 stderr，抽出兩邊所列的政策鍵，斷言集合逐字相同。驗證：cargo test -p speclink-cli --test it workflow_config 跑出紅燈，且失敗訊息指出 help 缺 worktree。 <!-- speclink-task:tsk_01KZZ55VTW2NS41XNXJ77ZFMPD -->
- [x] 1.2 在同一測試檔新增布林鍵說明測試：斷言 set --help 的 <value> 參數說明同時含 tdd、audit、worktree 三個鍵名。驗證：cargo test -p speclink-cli --test it workflow_config 跑出紅燈（現行字面只有 tdd 與 audit）。 <!-- speclink-task:tsk_01KZZ55VTWF1KM19P6AZQ55BGD -->
- [x] 1.3 在同一測試檔新增父層 help 同源測試：斷言 speclink workflow-config --help 子指令一覽中 set 一列的說明字面，與 speclink workflow-config set --help 的子指令說明相同。驗證：cargo test -p speclink-cli --test it workflow_config 跑出紅燈。 <!-- speclink-task:tsk_01KZZ55VTWQ5022VTY9NYNQWZM -->

## 2. 綠燈：help 文字改由單一真相來源產生

- [x] 2.1 依 design 決策 D1: help 說明由 POLICY_KEYS 生成，在 crates/speclink-cli/src/verbs/config.rs 新增一個 crate 內私有的 LazyLock<String> static，內容為 set 子指令說明字串、由 POLICY_KEYS 組出，並沿用 crates/speclink-cli/src/main.rs 既有的表達式屬性慣例掛上 clap 的 command(about = ...)；同時移除原本手寫的 doc comment，使 set 說明只剩一個來源。交付行為：speclink workflow-config set --help 列出五個政策鍵且順序等同 POLICY_KEYS 宣告序。驗證：1.1 與 1.3 轉綠。 <!-- speclink-task:tsk_01KZZ55VTWXTZ0BEBBYG4170XN -->
- [x] 2.2 依 design D1 的第二半（<VALUE> 說明維持字面並由測試釘住）修正 crates/speclink-cli/src/verbs/config.rs 中 value 參數的說明字面，使其標明 tdd、audit 與 worktree 三者僅接受 true 或 false，且不為此新增布林鍵常數。交付行為：set --help 的 <VALUE> 說明涵蓋全部布林鍵。驗證：1.2 轉綠。 <!-- speclink-task:tsk_01KZZ55VTWRBP2T9T2EVFW8YCK -->
- [x] 2.3 逐條列出 delta 三條需求的 scenario 與測試的對應關係，確認無孤兒 scenario：workflow-config set 政策欄位寫入 的三條新 scenario（set --help 列出全部政策鍵、set --help 標明布林鍵的合法值、父層 help 的 set 說明同源）對應 1.1 至 1.3；workflow-config show 動詞 的兩條改動 scenario（fs 模式顯示正典值、--json payload 形狀）對應既有測試 show_prints_canonical_policy_context_and_rules 與 show_json_payload_is_camel_case_with_null_for_unset（依 design 決策 D6: 兩處新增正典校正的測試載體判定，show 不新增測試）；init 範本的政策寫入位置 的 scenario 對應 5.1 新增的測試。交付行為：每條 scenario 都有指名到的測試載體。驗證：cargo test -p speclink-cli --test it 全綠，並逐條對照 openspec/changes/workflow-config-set-help-parity/specs/workflow-config/spec.md 的 scenario 名稱產出對應表。 <!-- speclink-task:tsk_01KZZ55VTWETS5M45GH13CA0CH -->
- [x] 2.4 確認正典擴及 worktree 的移除語意與現行行為相符：對含 worktree: true 的 config 執行 speclink workflow-config set worktree false --dry-run，確認 diff 顯示該鍵行被移除而非改為 false。交付行為：正典的「設為 false 即移除鍵」敘述涵蓋 worktree 且與實作一致。驗證：以 --dry-run 實跑觀察 diff；行為若不符則回報而非逕行改動實作（本變更範圍不含行為變更）。 <!-- speclink-task:tsk_01KZZ55VTWBDTHNKY7YRF4Y1FK -->

## 3. 範圍與紀律把關

- [x] 3.1 依 design 決策 D2: 同類漂移掃描結論，確認實作未擴散：git diff 的檔案清單僅含 crates/speclink-cli/src/verbs/config.rs、crates/speclink-cli/tests/it/workflow_config.rs 與 crates/speclink-cli/tests/it/init_tools.rs，未動 crates/speclink-cli/src/verbs/new.rs、crates/speclink-cli/src/verbs/query.rs、crates/speclink-core/src/init.rs、docs/ 與任何技能資產。驗證：git status 與 git diff --name-only 逐檔盤點。 <!-- speclink-task:tsk_01KZZ55VTWJVSSA3JNHE3TTXTP -->
- [x] 3.2 依 design 決策 D5: config-skill 的政策逐項詢問不納入本案，確認範圍未溢出到技能面：git diff 不含 openspec/specs/config-skill/ 與 crates/speclink-core/assets/skills/config.md，speclink-config 技能的政策問答維持四欄。交付行為：本案維持純文字校正，未升級為觸發技能三連動的產物層變更。驗證：git diff --name-only 確認兩條路徑皆未出現。 <!-- speclink-task:tsk_01KZZCE9FYVTS5TAKT19B1S2SS -->
- [x] 3.3 依 design 決策 D3: worktree 專屬行為不進 --help，確認 help 輸出未混入行為敘述：speclink workflow-config set --help 的內容僅含鍵、值與旗標的機械契約，不含技能足跡同步或關閉時擋下的說明。驗證：人工檢視 help 輸出全文。 <!-- speclink-task:tsk_01KZZ55VTWT7P3ZCS2GYZWT825 -->
- [x] 3.4 依 speclink instructions --skill audit 取得 sharp-edges 檢查表，對本次改動的 CLI 參數說明面套用一輪，確認未引入新的預設值、型別混淆或靜默失敗。交付行為：改動經過安全銳角檢查且結論記錄於本 task。驗證：逐條檢查表比對本次 git diff。 <!-- speclink-task:tsk_01KZZ55VTWW3KC3R10Q4TWSPB6 -->

## 4. 正典補正的測試守門

- [x] 4.1 為正典需求 init 範本的政策寫入位置 補上目前缺席的測試載體：在 crates/speclink-cli/tests/it/init_tools.rs 新增一條測試，於隔離 temp 環境執行 speclink init 後，斷言生成的 openspec/config.yaml 註解區含 locale、spec_locale、tdd、audit、worktree 五個鍵名與 SPECLINK_LOCALE、SPECLINK_SPEC_LOCALE、SPECLINK_TDD、SPECLINK_AUDIT、SPECLINK_WORKTREE 五個覆寫名，且 .speclink.yaml 不含任何政策鍵；只斷言這些名稱出現，不比對排版、縮排與說明措辭。交付行為：init 範本的內容從此有測試守門，改壞會變紅。驗證：cargo test -p speclink-cli --test it init_tools 全綠。 <!-- speclink-task:tsk_01KZZCCBE013X00Q34BQ9G2WRF -->
- [x] 4.2 證明 4.1 的釘樁測試不是恆綠的假測試：暫時移除 crates/speclink-core/src/init.rs 範本中的 worktree 註解行，確認 4.1 的測試轉紅，隨即還原該行並確認轉回綠。交付行為：測試的有效性經過變異檢查。驗證：兩次 cargo test -p speclink-cli --test it init_tools 的結果（先紅後綠），並以 git diff 確認 init.rs 已還原、逐位元不變。 <!-- speclink-task:tsk_01KZZCCBJBJE7C4YHE3130ZA7H -->

## 5. 回歸與收尾

- [x] 5.1 確認技能三連動未被誤觸：cargo test -p speclink-core --test it render_golden:: 全綠，且 crates/speclink-core/tests/golden/ 底下無任何檔案差異、crates/speclink-core/src/init.rs 的 MARKER_VERSION 與 crates/speclink-core/tests/golden/assets.lock 皆未改動。交付行為：golden 快照維持位元級不變。驗證：跑該測試後以 git status 檢查 golden 目錄；若出現差異，停下重新評估而非直接重生快照。 <!-- speclink-task:tsk_01KZZ55VTWBQXJZR0PMVKCCKKC -->
- [x] 5.2 確認 CLI 既有契約未回歸：cargo test -p speclink-cli --test it 全綠，含 workflow_config 既有案例與 remote_verb_parity。交付行為：set 的成功訊息、錯誤訊息、diff 輸出與 exit code 維持既有位元級輸出。驗證：該指令全綠。 <!-- speclink-task:tsk_01KZZ55VTW8KPWKHR8TV4XBKRQ -->
- [x] 5.3 確認新測試不受終端寬度與換行影響：新測試只斷言鍵名出現與集合相等，不比對整段版面、空白或換行位置。交付行為：測試在窄終端與 CI 環境同樣穩定。驗證：以 COLUMNS 設為 40 與 200 各跑一次 cargo test -p speclink-cli --test it workflow_config，兩次皆綠。 <!-- speclink-task:tsk_01KZZ55VTWCDVSHZPREN7TNMG0 -->
- [x] [M] 5.4 人工實跑確認使用者可見資訊已修正：執行 ./target/debug/speclink workflow-config set --help 與 ./target/debug/speclink workflow-config --help，確認兩份輸出的 set 說明皆列出 locale、spec_locale、tdd、audit、worktree 五鍵，且 <VALUE> 說明含 worktree；另跑 ./target/debug/speclink workflow-config show 確認 worktree 一列如常顯示（正典校正未改動輸出）。驗證：使用者目視確認三份輸出。 <!-- speclink-task:tsk_01KZZ55VTWQGBQWM6FR5GKMZQY -->
