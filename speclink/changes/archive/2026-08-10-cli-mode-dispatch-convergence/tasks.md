## 1. 凍結對照先行（characterization 測試，重構前即應全綠）

- [x] 1.1 依規格「模式分岔的單點宣告」新增 crates/speclink-cli/tests/it/mode_dispatch.rs 並於 crates/speclink-cli/tests/it/main.rs 登錄模組，落地宣告層三類邊界行為的凍結對照：(a) ModeFree——壞 .speclink.yaml 專案目錄下執行 completion 與 config（兩者不讀專案設定、現行即免疫；schemas 等因 workspace 探索讀檔的既有失敗不入對照），斷言 exit 0 且 stderr 不含 .speclink.yaml；(b) FsOnly——remote 模式設定且 server 不可達的專案執行 demo，斷言非零 exit、stderr 逐字含現行拒絕文案（demo is not available in remote mode 起頭）、未發出任何 server 請求；(c) RemoteOnly——fs 專案執行 claim，斷言非零 exit、stderr 逐字含現行拒絕文案（claim requires a remote store 起頭）。驗證：cargo test -p speclink-cli --test it mode_dispatch 於重構前全綠（凍結現行行為）。 <!-- speclink-task:tsk_01KZH1FC3S4DR0E7E3VYS28A07 -->

## 2. 形狀組合子與宣告層

- [x] 2.1 於 commands.rs 建立形狀組合子：dual（本機臂與 remote 臂皆為必填參數）、fs_only（僅解析 store 模式即拒絕、不建立連線）、remote_only（fs 即拒絕、remote 解析後連線派臂）；ModeFree 為 dispatch 直呼不經組合子。模式解析與連線握手分離（design D2 觸發矩陣）。驗證：cargo build -p speclink-cli 通過，組合子簽名使「缺臂」構成編譯錯誤。 <!-- speclink-task:tsk_01KZH1FC3T2DP4F37BFFFBC6NM -->
- [x] 2.2 dispatch 的 31 個 Commands variant 照 design D5 分類表逐一改為組合子宣告；多子指令的 Dual 動詞（task、new、artifact、language、in-progress、discuss、review、verify、workflow-config）拆出本機家族函式與 remote 家族函式，兩臂各自對子指令 enum 窮盡 match 且無 catch-all；review 與 verify 的 clap 到 StationVerb 正規化收為一支共用函式，兩臂各自呼叫。驗證：cargo build -p speclink-cli 通過，且任選一 Dual 動詞暫時移除 remote 臂可觀察到編譯錯誤（驗後還原）。 <!-- speclink-task:tsk_01KZH1FC3TS9SRBYFX3B1R820M -->
- [x] 2.3 移除 commands.rs 內 22 處函式開頭的 remote_ctx() 分岔與 demo 函式內的模式檢查；remote_ctx 僅由組合子層呼叫。驗證：grep -n "remote_ctx(" crates/speclink-cli/src/commands.rs crates/speclink-cli/src/remote_commands.rs 的呼叫點僅存在於組合子定義與 remote_ctx 自身定義處。 <!-- speclink-task:tsk_01KZH1FC3TASEW2F1MCQ6R6MGS -->

## 3. 行為凍結驗證

- [x] 3.1 全量整合測試零修改通過：cargo test -p speclink-cli --test it 全綠，其中 remote_verb_parity、remote_read_path、remote_write_path、config_fail_closed、no_raw_wire_json 與第 1 組新對照皆不因重構改動任何斷言或對照文字。驗證：測試輸出零 failed、git diff 顯示上列既有測試檔零變更。 <!-- speclink-task:tsk_01KZH1FC3TYHD14Q798JB0PQCQ -->
- [x] 3.2 拒絕文案與輸出零漂移總驗：claim（fs）與 demo（remote）的 stderr 文案、兩模式任一 Dual 動詞的人眼與 --json 輸出，與重構前基準逐字一致。驗證：以第 1 組凍結對照與既有整合測試全綠為準，另抽查 speclink list --json 於 fs 模式的輸出與 main 分支基準 diff 為零。 <!-- speclink-task:tsk_01KZH1FC3T5EE8T4TNGSFAGKZX -->
