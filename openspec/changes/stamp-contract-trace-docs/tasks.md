## 1. 文件修正（雙語同步）

- [ ] 1.1 docs/workflow.md 與 docs/workflow.zh-TW.md：品質站段落（review、verify、quality）補「蓋章消耗工單」敘述——同一原子寫入刪工單並寫章、封存的已蓋章 change 不含 review.md 與 verify.md、僅未結工單經 --carry-review 或 --carry-verify 隨封存移動、fs 模式工單文字僅存於 git 歷史、remote 模式蓋章後不可回讀；archive 段落同步一句。兩語內容一致、僅語言不同 <!-- speclink-task:tsk_01M111W65KJ3TCDY4ZHKK29WAQ -->
- [ ] 1.2 docs/verb-contract.md 與 docs/verb-contract.zh-TW.md：模式分岔表的 FsOnly 列補 trace，拒絕語意敘述與 demo 同格式（remote 拒絕、離線亦拒絕、零 server 請求） <!-- speclink-task:tsk_01M111W65KTNC0ZF0KKRJERGDW -->
- [ ] 1.3 docs/remote-getting-started.md 與 docs/remote-getting-started.zh-TW.md：已登入非成員讀取專案資源的敘述自 404 改為 403（permission_denied），移除與 404 綁定的「無法推知專案存在」推論句，對齊 server-identity 正典 <!-- speclink-task:tsk_01M111W65K1TY33VDWZV0P4HJW -->

## 2. 釘住測試與註解

- [ ] 2.1 crates/speclink-cli/tests/it/mode_dispatch.rs：新增 trace 的 FsOnly 拒絕釘住測試，與 demo 既有釘住同形——remote 模式執行 speclink trace 非零 exit、stderr 含「trace is not available in remote mode」拒絕訊息、零 server 請求。以 `cargo test -p speclink-cli --test it` 確認新測試通過（釘住既有行為，預期直接綠） <!-- speclink-task:tsk_01M111W65KRHAV5KFA5SQTRGPH -->
- [ ] 2.2 crates/speclink-cli/src/main.rs：修正 dispatch 表的過期動詞計數註解（現寫 31，Commands enum 實為 32），或改為不含寫死計數的敘述 <!-- speclink-task:tsk_01M111W65KHTCZS6QEDTQH6A72 -->

## 3. 收尾

- [ ] 3.1 跑 node --test scripts/remote-docs.test.mjs 文件查核腳本；斷言與本刀新敘述衝突時依實況更新斷言（歸文件查核面，非行為改動） <!-- speclink-task:tsk_01M111W65KTEJ39MZ8DB3E0KC1 -->
- [ ] 3.2 逐一核對四份 delta 與落地內容對應——review-station「蓋章守門與蓋章效果」、verify-station「驗證蓋章守門與蓋章效果」、verb-contract「動詞契約的涵蓋面與 payload 形狀」、user-documentation「品質站蓋章效果與非成員錯誤碼的文件揭露」；執行 speclink validate stamp-contract-trace-docs 通過 <!-- speclink-task:tsk_01M111W65KD03P3TTTSZ1GKZ1T -->
