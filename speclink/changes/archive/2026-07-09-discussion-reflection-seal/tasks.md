## 1. `discuss seal` 動詞（core 狀態語意 + cli 接線）

- [x] 1.1 撰寫 `discuss::seal` 單元測試（crates/speclink-core/src/discuss.rs 的 #[cfg(test)]）：涵蓋（a）成功——concluded 討論且變更 meta from_discussion 含該 slug → 討論 status 變 promoted、promoted_to 累加變更名；（b）守衛——討論不存在／已封存、變更不存在、變更 from_discussion 未含 slug → 回 Err 且兩側檔案不變；（c）冪等——promoted_to 已含變更名 → 不改檔。預期：`cargo test -p speclink-core --lib seal` 紅（函式未實作）
- [x] 1.2 實作 `discuss::seal`（crates/speclink-core/src/discuss.rs）：先驗鏈（讀變更 meta，`ChangeMeta::from_discussions()` 含 slug 否則 Err），通過後委派既有 `mark_promoted` 完成討論側標記。預期：1.1 測試綠。覆蓋需求：內容落地以 seal 動詞標記已轉出
- [x] 1.3 接線 CLI `discuss seal` 子指令（crates/speclink-cli/src/main.rs 新增子指令變體＋兩位置參數 slug/change、旗標 --json/--no-color；crates/speclink-cli/src/commands.rs 新增 dispatch 呼叫 `core::discuss::seal`，人眼單行成功訊息、守衛失敗非零 exit＋stderr）。預期：`speclink discuss seal <slug> <change>` 成功 exit 0、單行訊息
- [x] 1.4 撰寫 CLI `discuss seal` 輸出測試（crates/speclink-cli/tests/）：`--json` payload 含 slug 與 change（camelCase）、`--no-color` 無 ANSI、守衛失敗非零 exit＋stderr 說明。預期：測試綠、payload 欄位斷言通過
- [x] 1.5 重構：確認「討論側標記（mark_promoted）」與「變更側鑄鏈」為分離函式，`seal` 只用前者、`link` 只用後者，命名清楚。預期：`cargo test -p speclink-core --lib` 綠

## 2. link 停止預先標記

- [x] 2.1 撰寫測試：`discuss::link` 後討論記錄逐位元不變（crates/speclink-core/src/discuss.rs test）——link 一份 concluded 討論到變更後，討論 frontmatter 的 status 與 promoted_to 不變、變更 meta from_discussion 累加該 slug。預期：紅（現行 link 仍翻 promoted）
- [x] 2.2 從 `discuss::link` 移除 `mark_promoted` 呼叫（crates/speclink-core/src/discuss.rs），僅保留變更側 from_discussion 鑄鏈。預期：2.1 測試綠。覆蓋需求：討論以 link 動詞併入既有變更
- [x] 2.3 更新既有斷言「link 即 promoted」的單元測試（crates/speclink-core/src/discuss.rs 的 link_writes_change_meta_and_marks_discussion、link_accepts_open_discussion、link_appends_to_from_discussion_when_change_already_linked 等）為新行為：斷言 link 後討論記錄逐位元不變、變更 meta 累加。預期：`cargo test -p speclink-core --lib` 全綠
- [x] 2.4 確認 promote／new change --from-discussion 行為不變（其既有測試仍綠，未受本刀影響）。預期：`cargo test -p speclink-core --lib promote` 綠

## 3. show --json 暴露 fromDiscussions

- [x] 3.1 撰寫測試：`speclink show <change> --json` payload 含 `fromDiscussions`——有連結時為有序字串陣列（順序同 meta）、無連結時為 `[]`（crates/speclink-cli/tests/）。預期：紅
- [x] 3.2 實作 show payload 新增 `fromDiscussions`（crates/speclink-cli/src/commands.rs 的 show 指令 --json 組裝處，派生自 `ChangeMeta::from_discussions()`），既有欄位（created／deltaSpecs／design／name／proposal／schema／tasks）不變。預期：3.1 測試綠、既有 show --json 斷言不破。覆蓋需求：from_discussion 鏈可經 show --json 觀察
- [x] 3.3 重構：`fromDiscussions` 命名遵循 camelCase 序列化慣例（`#[serde(rename)]` 或等效），與其他 --json 欄位一致。預期：測試綠

## 4. ingest 技能指示三處同步（assets → golden → repo 實例）

- [x] 4.1 更新 ingest 技能資產（crates/speclink-core/assets/skills/ingest.md）：新增步驟「目標變更 meta 帶 from_discussion 時，經 `speclink discuss show <slug>` 讀結論作為一等來源併入既有脈絡／plan（不取代）」與「artifacts 更新完成時執行 `speclink discuss seal <slug> <change>`」；既有「link 鑄鏈」提示調整為 link 先鑄鏈、seal 後封印。覆蓋需求：技能指示引導 ingest 型結論先鑄鏈
- [x] 4.2 檢視 discuss 技能資產（crates/speclink-core/assets/skills/discuss.md）：確認「Capture to 指向既有變更 → link → /speclink-ingest」指引維持，補「promoted 由 seal 於內容落地時標記、link 不再即標」的語意說明
- [x] 4.3 於乾淨樹重生 render golden：`UPDATE_GOLDEN=1 cargo test -p speclink-core --test render_golden`，審視 diff 僅含 4.1–4.2 的預期技能文字變更（crates/speclink-core/tests/golden/claude.snapshot.md、codex.snapshot.md）。預期：重生後 `cargo test -p speclink-core --test render_golden` 綠
- [x] 4.4 同步 repo 技能實例：以 `speclink update` 重生 .claude/skills 與 .agents/skills 的 ingest／discuss 技能，確認內容與 assets 一致（三處同步）。預期：repo 技能檔含新指示、與 golden 一致

## 5. 整合驗證

- [x] 5.1 端到端手動驗證狀態轉移：`speclink discuss new`→`conclude`→`link`（討論停 concluded、`discuss list --json` status 為 concluded）→`discuss seal`（status 變 promoted、promoted_to 含變更名）；`speclink show <change> --json` 的 fromDiscussions 正確。預期：各步觀察值符合 spec
- [x] 5.2 全量驗證：`cargo test -p speclink-core --lib` 綠、`cargo test -p speclink-cli` 綠、`cargo test -p speclink-core --test render_golden` 綠、`cargo build --release -p speclink-cli` 成功。預期：全綠、無回歸
