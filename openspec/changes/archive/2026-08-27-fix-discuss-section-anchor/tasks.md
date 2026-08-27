## 1. 引擎回歸測試（先紅）

- [x] 1.1 在 crates/speclink-core/src/discuss.rs 的 `#[cfg(test)]` 模組新增三個回歸測試：(a) Round 1 內文含「## 背景」行時 add_round 追加第二輪，斷言文件順序為 Round 1 標題→Round 1 完整內文→Round 2 標題→Round 2 內文→結構 Conclusion；(b) 某輪內文含整行「## Conclusion」時執行 conclude，斷言結論寫入結構 Conclusion 區段、該內容行以反斜線跳脫形式留在輪內、既有輪內文不被改寫；(c) 輪內文含「### Round 」前綴行時，斷言 count_rounds 不膨脹、下一輪編號連續。以 `cargo test -p speclink-core discuss` 確認三個新測試如預期失敗（紅燈基準） <!-- speclink-task:tsk_01M111DM6V6BB500AYGXJFVH04 -->

## 2. 引擎實作（轉綠）

- [x] 2.1 crates/speclink-core/src/discuss.rs：section_body_range 改為結構標題白名單——只有整行為「## Context」「## Rounds」「## Conclusion」的行終止區段；add_round 插入點、replace_section（set_context 與 conclude 共用）、conclusion_text 隨之取得正確邊界。驗證：1.1 的 (a)(b) 測試轉綠 <!-- speclink-task:tsk_01M111DM6VGFKFBS1H2M8YAQS7 -->
- [x] 2.2 crates/speclink-core/src/discuss.rs：寫入端跳脫——add_round、set_context、conclude 落盤前，把撞名內容行（整行為結構標題，或行首為「### Round 」「## Round 」前綴）加上 markdown 反斜線；count_rounds 收緊為合法輪標題形狀（保留 pre-scaffold「## Round 」容忍）。驗證：1.1 的 (c) 測試轉綠 <!-- speclink-task:tsk_01M111DM6VF8TRGW42QX9GQHEY -->
- [x] 2.3 跑 `cargo test -p speclink-core` 全綠：既有討論測試與 crates/speclink-core/tests/golden 不回歸（scaffold 骨架未動，golden 不應有差異；若有差異即為實作外溢，回頭修） <!-- speclink-task:tsk_01M111DM6V2JKTQMPM90BYM9BA -->

## 3. UI 同步修（先紅後綠）

- [x] 3.1 packages/ui/src/__tests__/discussionDrawer.test.tsx 新增失敗測試：輪內文含「## 」開頭行時，splitDiscussionSections 的 rounds 區段完整涵蓋至結構 Conclusion、該輪內文不遺失；以 `npm test -w packages/ui` 確認新測試失敗 <!-- speclink-task:tsk_01M111DM6VHXQDPQKWVD2Q8P76 -->
- [x] 3.2 packages/ui/src/components/DiscussionDrawer.tsx：splitDiscussionSections 改為與引擎同一結構標題白名單。驗證：`npm test -w packages/ui` 全綠 <!-- speclink-task:tsk_01M111DM6VQM2NAH7V517977Y1 -->

## 4. 整合與收尾

- [x] 4.1 跑 `cargo test -p speclink-cli --test it` 確認討論相關 CLI 整合測試（含 discuss_content_guard）不回歸 <!-- speclink-task:tsk_01M111DM6V49CA6ZR37H7CANCZ -->
- [x] 4.2 執行 speclink validate fix-discuss-section-anchor 通過；核對實作與 specs/discussion-docs delta「討論記錄結構錨定與撞名內容跳脫」requirement 的四個 scenario 逐一對得上 <!-- speclink-task:tsk_01M111DM6V0CX9EP7HPWB2TDWJ -->
