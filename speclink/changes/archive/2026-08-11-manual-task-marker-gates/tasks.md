## 1. 解析器與 payload

- [x] 1.1 依 design D1、D2 以 TDD 落實「任務行的手動測試標記與解析」與「寫碼任務全完成預測子」:crates/speclink-core/src/tasks.rs 的前綴剝除迴圈同時接受 `[P]` 與 `[M]`(順序不敏感、各至多一次)、Task 增 manual 旗標、描述剝除標記;進度統計同時回傳全量與寫碼兩組計數(寫碼任務全完成預測子的單一實作)。行為結果:manual-task-marker spec 的解析 Example 表逐列成立;無標記任務行為逐位元不變。驗證:cargo test -p speclink-core 之 tasks 單元測試綠燈(含 [M]/[P] 組合、剝離、空真計數案例)。 <!-- speclink-task:tsk_01KZN47CJN7AR5CMDM406YSJKV -->
- [x] 1.2 依 design D6 以 TDD 落實「任務 payload 的 manual 欄位與寫碼進度」,並以 verb-contract「動詞 --json 輸出形狀凍結」的加欄基線釘住形狀:crates/speclink-core/src/instructions.rs 任務項增 manual 欄位,progress 增 codeTotal/codeComplete/codeRemaining;crates/speclink-protocol/src/query.rs 的 wire 形狀同步(加欄不改名)。行為結果:instructions apply --json 對含 `[M]` 的 change 輸出 manual=true 與三個 code 欄位;無 `[M]` 時 code 欄位值與全量一致、既有欄位逐位元不變。驗證:cargo test -p speclink-core 與 -p speclink-protocol 的 payload 形狀測試綠燈。 <!-- speclink-task:tsk_01KZN47CJN53GD5F0C6WEXHA56 -->

## 2. 引擎守門

- [x] 2.1 依 design D3 以 TDD 改判「驗證工單的建立與追加」的落工單守門:crates/speclink-core/src/station.rs 的 add-round 任務守門改用寫碼任務全完成預測子,拒絕訊息點名寫碼任務計數。行為結果:寫碼任務未完成拒絕(stderr 述寫碼計數)、僅餘 `[M]` 未勾放行落工單。驗證:cargo test -p speclink-core 之 station/verify 單元測試綠燈(新增僅餘手動任務放行案例)。 <!-- speclink-task:tsk_01KZN47CJN58W0RR9EAQW714Q7 -->
- [x] 2.2 依 design D3 以 TDD 改判兩站蓋章守門——review 站「蓋章守門與蓋章效果」與 verify 站「驗證蓋章守門與蓋章效果」:station.rs 的 stamp 任務條件改用同一預測子,拒絕訊息同步;蓋章寫入的 reviewed_tasks_total/verified_tasks_total 仍記全任務總數。行為結果:寫碼任務全完成即可蓋章(`[M]` 未勾不擋),錨欄位含 `[M]` 計數。驗證:cargo test -p speclink-core 之 review/verify stamp 測試綠燈(含僅餘手動任務蓋章、錨值為全任務總數案例)。 <!-- speclink-task:tsk_01KZN47CJN2HADDJ1TAWPH57TP -->
- [x] 2.3 依 design D4 以 TDD 改判失效判定的任務錨——review 站「內容指紋錨與失效判定」與 verify 站「驗證指紋錨與失效判定」:station.rs 的 freshness 判定式改為「全任務總數等於蓋章時總數 且 寫碼任務全完成」,門面簽名(review.rs/verify.rs)增寫碼計數參數。行為結果:補勾 `[M]` → fresh;改 scope 檔 → stale;新增任務 → stale;取消勾寫碼任務 → stale。驗證:cargo test -p speclink-core 之 freshness 四情境測試綠燈。 <!-- speclink-task:tsk_01KZN47CJNBS4CS97SVTHH67MM -->
- [x] 2.4 依 design D5 以 TDD 實作封存的章失效守門:crates/speclink-core/src/archive.rs 於任務完成度守門之後對兩章各判 freshness(本機讀工作樹算內容錨;remote 通道僅判任務錨),stale 拒絕並點名站別與破錨原因、指路重跑該站;Unknown 與無章放行且行為逐位元不變;批次封存沿既有 fail-fast 樣式因 stale 章中止並點名該 change。行為結果:change-lifecycle delta 的守門判定 Example 表逐列成立。驗證:cargo test -p speclink-core 之 archive 守門測試綠燈(stale 拒絕/放行/順序/remote 各案例)。 <!-- speclink-task:tsk_01KZN47CJN0F7V9DNXT35XMPDZ -->

## 3. 技能文字與衍生物

- [x] 3.1 依 design D7 更新五技能 asset 文字(crates/speclink-core/assets/skills 下 claude 與 codex 兩形):review 開跑守門讀 codeRemaining、僅餘 `[M]` 時繼續並點名手測與時序;verify 分流改判 codeRemaining;quality 前提句改寫碼任務全完成;propose 起草教 `[M]` 前綴;apply 不代勾 `[M]`、寫碼任務全勾即回報完成。行為結果:「審查流程的技能行為」「驗證技能的工單落地」「兩站時序的編排行為」「手動測試任務的起草標記」「apply 技能的手動任務處理」五條文的技能文字情境一致。驗證:對照各 delta spec 逐項核讀;golden 於 3.2 收斂。 <!-- speclink-task:tsk_01KZN47CJNK06YT3R52F5NENKY -->
- [x] 3.2 完成技能衍生物三連動:crates/speclink-core/src/skills.rs 的 MARKER_VERSION 提升、.claude/skills 五個 SKILL.md 與 assets 同步、golden snapshot 再生與 crates/speclink-core/tests/golden/assets.lock 更新。行為結果:speclink update 產出的技能檔與 golden 一致。驗證:cargo test -p speclink-core 之 render_golden 綠燈。 <!-- speclink-task:tsk_01KZN47CJN0NAGJC7JG9JNH1EG -->

## 4. 整體驗收

- [x] 4.1 新增 CLI 整合測試 crates/speclink-cli/tests/it/manual_task_gates.rs,覆蓋 design Implementation Contract 行為 1–5:payload 曝光、verify add-round 放行/拒絕、兩站 stamp 放行/拒絕、freshness 四情境、archive 章失效守門(含任務守門先拒的順序)。驗證:cargo test -p speclink-cli --test it manual_task_gates 全綠。 <!-- speclink-task:tsk_01KZN47CJN9B2KGEJC8R4A2TAR -->
- [x] 4.2 調整既有守門測試至新訊息與新條件:crates/speclink-cli/tests/it/review_verbs.rs、verify_verbs.rs、archive_readiness_gate.rs 中引用「任務全數完成」訊息與 4/5 拒絕條件的案例改判寫碼任務。行為結果:既有測試反映新守門語意、無殘留舊訊息斷言。驗證:cargo test -p speclink-cli --test it 全綠。 <!-- speclink-task:tsk_01KZN47CJN9NBSYNCPBF7HFGZP -->
- [x] 4.3 全套回歸與收尾:workspace 全測試與 lint,確認無因本 change 孤兒化的 imports 或死碼。驗證:cargo test 與 cargo clippy 全綠。 <!-- speclink-task:tsk_01KZN47CJNDYVG8S2MQFRKEEZH -->
