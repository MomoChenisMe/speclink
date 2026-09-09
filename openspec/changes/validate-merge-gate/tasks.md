## 1. D1：驗證入口拆成兩層，archive 家族改呼叫結構層

- [ ] 1.1 在 crates/speclink-core/src/validate.rs 把既有 `validate_change` 本體改名為公開的 `validate_change_structural`（簽名相同、輸出逐位元不變），`validate_change` 改為「結構層結果加上合併守門 error」，守門判斷沿用 crates/speclink-core/src/archive.rs 的 `merge_violations`（spec archive-merge「過期判定單源共用」：validate 成為第四個共用者，不平行實作）；crates/speclink-core/src/archive.rs 的單筆 archive 驗證前置與 crates/speclink-cli/src/verbs/lifecycle.rs 的 bulk archive 驗證前置改呼叫 `validate_change_structural`。驗證：`cargo test -p speclink-core archive` 綠（Validation failed 與 merge_refusal 的凍結字串未動），且 validate.rs 測試斷言同一份會被守門拒收的 change 用 `validate_change_structural` 零守門 error、用 `validate_change` 至少一條；spec「change 驗證納入合併守門」Scenario「archive 的拒絕輸出不變」對應此 task。 <!-- speclink-task:tsk_01M22BD562QAXTHBFCEXCN5G1N -->

## 2. D2：守門 error 的組字與位置

- [ ] 2.1 在 crates/speclink-core/src/validate.rs 把每筆 `MergeViolation` 組成 `specs/<capability>/spec.md: <operation> '<requirement>': <reason> (see: speclink drift <change>)`，追加在所有既有 error（含 `[M]` 標記 error）之後，`valid` 隨 errors 為空與否。驗證：測試涵蓋 spec「change 驗證納入合併守門」的 Scenario「MODIFIED 目標不存在時 validate 報 error」與「未宣告的 scenario 移除被 validate 抓到」（含補宣告後通過），並逐列斷言 Example 表三種組字；`cargo test -p speclink-core validate` 綠。 <!-- speclink-task:tsk_01M22BD5628MGMDRAVRC8WKCD6 -->

## 3. D3：與結構層的去重規則

- [ ] 3.1 在 crates/speclink-core/src/archive.rs 為 `MergeViolation` 新增 `is_section_collision()`（與 `is_purpose_gate()` 同一模式），crates/speclink-core/src/validate.rs 略過 Purpose 類違規，並略過需求名已被 Duplicate／appears in both error 報過的撞名違規。驗證：測試涵蓋 Scenario「Purpose 與重複需求名不重複列」（errors 恰 2 條）與「RENAMED 端點撞名只有守門看得到」（恰 1 條守門 error、`<requirement>` 為 `R2`）；`cargo test -p speclink-core validate` 綠。 <!-- speclink-task:tsk_01M22BD562P558FFCBMEYKY6HB -->

## 4. D4：測試落點

- [ ] 4.1 在 crates/speclink-cli/tests/it/validate_specs.rs 加端到端案例：正典 `specs/auth/spec.md` 有 `R1`、change 的 delta MODIFIED `R9`，`speclink validate <change>` 人眼輸出含 `✗ <change> — invalid` 與該 error 行、以 `Validation failed.` 非零收尾；`--json` 的 `valid` 為 false、`errors` 含該字串、四欄 `change`／`errors`／`valid`／`warnings` 齊備；同一 change 跑 `speclink drift` 的 spec assumption 指向同一 capability 與需求名（「四處判定一致」）。驗證：`cargo test -p speclink-cli --test it validate_specs` 與 `cargo test -p speclink-cli --test it remote_verb_parity` 綠。 <!-- speclink-task:tsk_01M22BD562J7HKZTC16ZECH7RR -->
- [ ] 4.2 收尾基準：對本 repo 現存的 change 跑 `speclink validate --all` 確認仍通過（現存 change 無守門違規），並確認 archive-merge 的 delta 以 `<!-- REMOVED-SCENARIO: 三處判定一致 -->` 宣告了 scenario 改名。驗證：`speclink validate validate-merge-gate` 通過、`speclink analyze validate-merge-gate` 無 Critical／Warning、`node --test scripts/*.test.mjs` 綠。 <!-- speclink-task:tsk_01M22BD562HNMP2JNNKB2H42ZT -->
