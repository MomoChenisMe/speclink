## 1. fs 模式動詞（先紅後綠）

- [x] 1.1 撰寫 fs 模式測試（新檔 crates/speclink-cli/tests/workflow_config.rs），涵蓋規格「workflow-config show 動詞」「workflow-config set 政策欄位寫入」「workflow-config context 與 rules 寫入」的 fs 情境：show 人眼與 --json（camelCase 欄位、正典值不受 SPECLINK_TDD 影響）、set 各鍵寫入與保留其他鍵、未知 key 與非法布林拒絕、set false 移除鍵、context --stdin 多行設定與空白移除、rules 整節代換與未知 artifact 拒絕、--dry-run 印 unified diff 且檔案逐位元不變、壞 config fail-closed 拒絕讀寫。驗證：cargo test -p speclink-cli --test workflow_config 全數失敗（動詞尚不存在） <!-- speclink-task:tsk_01KYGX42WS6K0KVVK2N99EX1H6 -->
- [x] 1.2 實作 fs 模式動詞（落點依 design「決策一：動詞落點與命令執行層的關係」——周邊設定動詞、不進命令執行層）：crates/speclink-cli/src/main.rs 註冊 workflow-config 子指令樹（show／set／context／rules，旗標 --json、--dry-run、--no-color、--stdin），crates/speclink-cli/src/commands.rs 新增編排——經既有 workspace 定位讀 openspec/config.yaml、呼叫 speclink-core 的 config seam 改寫（依 design「決策四：set 的單鍵語意映射到 seam 的完整目標態」組裝完整目標態）、寫回或依 design「決策三：--dry-run 由 CLI 以同一改寫路徑產 diff」輸出 unified diff。驗證：cargo test -p speclink-cli --test workflow_config 全綠 <!-- speclink-task:tsk_01KYGX42WS5S2905Z784TNZXN2 -->
- [x] 1.3 重構：確認 CLI 側為薄編排（無自行解析 YAML 的邏輯），並依 design「決策六：git 互動與跨平台」確認 diff 生成不依賴系統 diff 工具、換行以 LF 為準、路徑經既有 workspace 定位；既有測試不受影響。驗證：cargo test -p speclink-cli 全綠 <!-- speclink-task:tsk_01KYGX42WSGM91MQS6SYFJKPR2 -->

## 2. remote 模式動詞（先紅後綠）

- [x] 2.1 撰寫 remote 模式測試（crates/speclink-cli/tests/workflow_config.rs 增節，沿既有 remote 測試的 in-process server harness）：show 輸出形狀與 fs 一致、set／context 寫入後讀回、版本衝突（寫回前由另一連線改寫 server config）以非零 exit code 提示重跑且不覆蓋、離線（server 關閉）語義化失敗不暫存。驗證：新增測試全數失敗 <!-- speclink-task:tsk_01KYGX42WSJD7HHP7EB46H1WQ0 -->
- [x] 2.2 實作 remote 分派：crates/speclink-cli/src/remote_commands.rs 依 design「決策二：remote 的版本處理——單動詞內讀-改-寫、版本不進介面」編排——經 speclink-remote 既有 config 讀寫端點取內容與版本、同一 seam 改寫、寫回帶版本；CAS 失敗與離線的錯誤訊息落地。驗證：cargo test -p speclink-cli --test workflow_config 全綠（fs＋remote） <!-- speclink-task:tsk_01KYGX42WSEDP0ZJ9AM4TAQ9S0 -->
- [x] 2.3 重構：比對 fs 與 remote 兩分支的輸出組裝無重複實作（共用呈現函式）。驗證：cargo test -p speclink-cli 全綠 <!-- speclink-task:tsk_01KYGX42WS5QYER9SPGY43139R -->

## 3. 內嵌技能 speclink-config

- [x] 3.1 撰寫渲染測試預期：依規格「內嵌 speclink-config 技能的渲染與保護」「技能規定固定輸入來源與四條內容判準」「技能規定 diff 先行與收斂驗收」，先於 crates/speclink-core/tests/ 的 render_golden 流程確認新技能將納入渲染集合（新增資產前 golden 不含 speclink-config——執行 cargo test -p speclink-core --test render_golden 記錄現況為基準） <!-- speclink-task:tsk_01KYGX42WSPCFVE1V8FXZGKKT8 -->
- [x] 3.2 依 design「決策五：技能為內嵌資產，走 commit-skill 同機制」新增技能資產 crates/speclink-core/assets/skills/config.md：內容含固定輸入來源清單（Cargo workspace 成員與 workspace 相依、關鍵邊界相依、各 package 相依、README、docs 索引、既有 config.yaml、speclink language show）、四條判準（判準一以 speclink instructions <artifact> --json 逐條反證、判準四引用存在性核實）、執行流程（政策四欄逐項詢問、--dry-run 產 diff 經使用者確認後以 workflow-config 動詞寫入）、收斂驗收（同一 codebase 連跑兩次第二次 diff 為空，否則回查判準）；並於 crates/speclink-core/src/skills.rs 註冊渲染。驗證：cargo test -p speclink-core --test render_golden 轉紅（golden 尚無新技能區段） <!-- speclink-task:tsk_01KYGX42WSMXDHKW9HMP6G462K -->
- [x] 3.3 確認工作樹乾淨（git status --porcelain 為空、僅本變更檔案）後，以 UPDATE_GOLDEN=1 cargo test -p speclink-core --test render_golden 再生四份 golden 並審視 diff 僅含 speclink-config 新增區段。驗證：再跑 render_golden 轉綠；git diff 的 golden 變動全屬新技能 <!-- speclink-task:tsk_01KYGX42WSB21ATFBWZ9H6BQ3E -->
- [x] 3.4 同步 repo 技能實例：新增 .claude/skills/speclink-config/SKILL.md 與 .agents/skills/speclink-config/SKILL.md（與 assets 渲染一致）。驗證：兩檔內容與 speclink update 渲染輸出一致 <!-- speclink-task:tsk_01KYGX42WS6WX30YP85MD0CRKN -->

## 4. 文件與收尾

- [x] 4.1 docs/configuration.zh-TW.md 與 docs/configuration.md 補 workflow-config 動詞說明（子指令、--dry-run、fs 與 remote 行為、模板註解喪失取捨），兩語版概念對等。驗證：兩檔各含 workflow-config 章節、H2 結構對稱 <!-- speclink-task:tsk_01KYGX42WSRYVW8FPFCZXA2634 -->
- [x] 4.2 全套驗證：cargo test --workspace 全綠；speclink workflow-config show --json 於本 repo 回傳現行 config.yaml 的正典值（locale tw、tdd true、audit true）。驗證：全部通過、payload 欄位 camelCase <!-- speclink-task:tsk_01KYGX42WSH7QY1YSB19JWK97N -->
- [x] 4.3 執行 speclink validate workflow-config-verb-and-skill 與 speclink analyze workflow-config-verb-and-skill。驗證：validate 通過、analyze 無 Critical 或 Warning <!-- speclink-task:tsk_01KYGX42WSJV1JFEQZX8P8GEG5 -->
