## 1. 核心 meta 欄位（model.rs）

- [x] 1. 於 ChangeMeta（crates/speclink-core/src/model.rs）加 restale_from: Option<String> 欄位，from_text 解析之；既有欄位解析與預設不變。驗證：新增欄位不破壞既有 from_text 測試。覆蓋需求：restale_from 記錄變更待重新反映的討論並經 CLI 觀測
- [x] 2. 先寫失敗 unit 測試再實作 ChangeMeta::restale_from() → Vec<String>（缺席回空、逗號值 trim 後分割），仿 from_discussions() 既有測試形態。驗證：cargo test -p speclink-core --lib restale_from 三案（缺席/單值/逗號多值）綠。覆蓋需求：restale_from 記錄變更待重新反映的討論並經 CLI 觀測

## 2. conclude 蓋章（discuss.rs）

- [x] 3. 先寫失敗測試：conclude 對 promoted_to 含一個 active change 的討論，使該 change meta 的 restale_from 累加本 slug；討論記錄除 Conclusion 外逐位元不變。覆蓋需求：討論重新結論標記已反映變更待重新反映
- [x] 4. 先寫失敗測試：conclude 對 promoted_to 同含 active 與已歸檔 change 的討論，僅 active 被蓋、已歸檔目錄逐位元不變（判存活：openspec/changes/<name>/ 存在 vs archive/）。覆蓋需求：討論重新結論標記已反映變更待重新反映
- [x] 5. 先寫失敗測試：conclude 對 promoted_to 空的討論不寫任何 change meta（既有輸出不變）；及重複 conclude 冪等（restale_from 已含本 slug 不重複）。覆蓋需求：討論重新結論標記已反映變更待重新反映
- [x] 6. 實作 conclude（crates/speclink-core/src/discuss.rs）蓋章步：寫結論後讀 promoted_to、走訪 active change（鏡像 unlink_discarded）、restale_from 累加 slug 冪等寫回；回傳被標記 active change 清單供 CLI 用。驗證：任務 3–5 測試全綠。覆蓋需求：討論重新結論標記已反映變更待重新反映

## 3. seal 清除（discuss.rs）

- [x] 7. 先寫失敗測試：seal 自目標 change meta restale_from 移除本 slug（其餘值保留）；唯一值時移除整行；不含時冪等不改；既有 seal 守衛/promoted/輸出不變。覆蓋需求：seal 清除變更的 restale 旗標
- [x] 8. 實作 seal（crates/speclink-core/src/discuss.rs）清除步：mark_promoted 後移除 restale_from 中本 slug、變空移除行。驗證：任務 7 測試全綠、既有 discuss_seal 測試不回歸。覆蓋需求：seal 清除變更的 restale 旗標

## 4. CLI 浮現（commands.rs + analyzer.rs）

- [x] 9. conclude 指令（crates/speclink-cli/src/commands.rs）於蓋章後 stdout 報告被標記 active change 清單（無則不報）、--json payload 帶被標記變更名陣列；promoted_to 空時輸出逐位元不變。驗證：CLI 整合測試 re-conclude 後 stdout/JSON 含被標記變更。覆蓋需求：討論重新結論標記已反映變更待重新反映
- [x] 10. show 與 list 的 --json（crates/speclink-cli/src/commands.rs）曝 restaleFrom（camelCase 陣列，缺席空陣列），仿 fromDiscussions。驗證：CLI 整合測試對帶 restale_from 的 change 見 restaleFrom。覆蓋需求：restale_from 記錄變更待重新反映的討論並經 CLI 觀測
- [x] 11. analyze（crates/speclink-core/src/analyzer.rs）對 restale_from 非空的 change 出一條資訊性 finding（指明反映討論已重新結論、需 re-ingest）；為空時無此 finding。驗證：analyze 整合測試出/不出該 finding 兩案。覆蓋需求：restale_from 記錄變更待重新反映的討論並經 CLI 觀測

## 5. desktop 看板徽章

- [x] 12. Rust 側變更序列化於 apps/desktop/core/src/query.rs 的看板清單路徑疊加 restale_from 為資料欄（Tauri invoke 傳前端）。驗證：desktop core 測試變更物件含 restale_from。覆蓋需求：看板卡片浮現待重新反映徽章
- [x] 13. tauriDataSource（apps/desktop/src/adapter/tauriDataSource.ts）型別與映射帶 restaleFrom。驗證：npm test -w apps/desktop dataSource 映射測試綠。覆蓋需求：看板卡片浮現待重新反映徽章
- [x] 14. 先寫失敗元件測試再實作 packages/ui 看板卡片：restale_from 非空渲染「待重新反映」徽章（主題化、與既有卡片視覺一致）、為空不渲染；不影響欄位派生。驗證：npm test -w packages/ui kanban 徽章有/無兩案綠。覆蓋需求：看板卡片浮現待重新反映徽章
- [x] 15. 真實視窗驗證徽章（release exe ＋ 截圖）：造一個 restale_from 非空的 change，確認卡片顯示徽章、無者不顯示。操作前先確認使用者未在使用螢幕。覆蓋需求：看板卡片浮現待重新反映徽章

## 6. 技能指引與 golden

- [x] 16. ingest 技能（crates/speclink-core/assets/skills/ingest.md）加指引：目標 change 帶 restale_from 時，re-ingest 折入新結論後執行 seal 清除該 slug。同步 repo 技能實例（.claude/skills、.agents/skills 經 speclink update）。覆蓋需求：restale_from 記錄變更待重新反映的討論並經 CLI 觀測
- [x] 17. 於乾淨樹跑 UPDATE_GOLDEN=1 cargo test -p speclink-core --test render_golden 再生四份 golden，審視 diff 僅限 ingest 技能的 restale 指引段。驗證：render_golden 測試以新基準綠。覆蓋需求：restale_from 記錄變更待重新反映的討論並經 CLI 觀測

## 7. 驗證收尾

- [x] 18. 跑全套：cargo test -p speclink-core --lib、CLI 整合測試、npm test -w packages/ui、npm test -w apps/desktop；speclink analyze reconclude-restale 與 validate 通過。驗證：全綠、無 Critical/Warning。

## 設計決策涵蓋（design ↔ tasks 對應）

各設計決策落在的任務（散文對應，非額外任務）：

- D1 觸發鍵綁 promoted_to 非空 → 任務 5、6
- D2 蓋章跳過已歸檔變更 → 任務 4
- D3 restale_from 記錄哪份討論 → 任務 2、6、7
- D4 實作鏡像 unlink_discarded → 任務 6
- D5 seal 清除 per-slug → 任務 7、8
- D6 meta 欄位機制 → 任務 1、2
- D7 CLI 與看板四處浮現 → 任務 9、10、11、12、13、14、15
- D8 技能指引 → 任務 16、17
