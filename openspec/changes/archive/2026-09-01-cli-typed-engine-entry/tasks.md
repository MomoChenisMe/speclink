## 1. core 型別化轉換層（TDD 紅燈先行）

- [x] 1.1 在 crates/speclink-core/src/command/typed.rs 起檔並先寫單元測試（`#[cfg(test)]` 模組）：ListOutcome 自 CommandOutcome::List 成功轉換；對錯誤 variant 轉換回 WrongOutcome 且 Display 訊息同時含期望型別名與實際 variant 名；TaskFlipOutcome 同時接受 TaskDone 與 TaskUndone；String 同時接受 ArtifactCat 與 Language。此時實作未寫，cargo test -p speclink-core 編譯失敗即紅燈 <!-- speclink-task:tsk_01M1DYZ9RS2D1RQCT9VANTZAYQ -->
- [x] 1.2 在 crates/speclink-core/src/command/typed.rs 實作 WrongOutcome（pub struct，實作 Display 與 std::error::Error，訊息英文無 ANSI）、私有 variant 名輔助函式，與檔內 macro_rules! 表驅動生成 design D1 表列的 24 支 TryFrom<CommandOutcome> impl；在 crates/speclink-core/src/command/mod.rs 掛 mod typed 並 pub use WrongOutcome；cargo test -p speclink-core 全綠 <!-- speclink-task:tsk_01M1DYZ9RSR83YVSTVT8Z3E6P2 -->

## 2. CLI 薄泛型入口與 29 處呼叫點遷移

- [x] 2.1 在 crates/speclink-cli/src/common.rs 把 run_command 改名並泛型化為 run（簽名照 design D4：store: &dyn Store、ws: Option<&Workspace>、cmd: Command，回傳 Result<T>，約束 T: TryFrom<CommandOutcome, Error = WrongOutcome>），函式體為既有 ExecutionContext 組裝加 execute 加 T::try_from 上拋；cargo build -p speclink-cli 的編譯錯誤即待遷移呼叫點清單 <!-- speclink-task:tsk_01M1DYZ9RSMDD4QMX5GQV9PH5F -->
- [x] 2.2 遷移 crates/speclink-cli/src/verbs/query.rs（list、show、status）、crates/speclink-cli/src/verbs/checks.rs（validate、analyze）、crates/speclink-cli/src/verbs/trace.rs（trace）、crates/speclink-cli/src/verbs/instructions.rs（instructions）：每處 let-else 拆封改為一行具型別綁定的 run 呼叫，刪除該檔 let store: &dyn Store 轉型行 <!-- speclink-task:tsk_01M1DYZ9RSQVB9NJM4X5JS7CK1 -->
- [x] 2.3 遷移 crates/speclink-cli/src/verbs/lifecycle.rs（archive、discard）、crates/speclink-cli/src/verbs/new.rs（new change、new artifact）、crates/speclink-cli/src/verbs/documents.rs（artifact cat、language show）、crates/speclink-cli/src/verbs/progress.rs（task done、task undone、in-progress add、in-progress remove）：同 2.2 形狀；in-progress add 的丟棄式呼叫改為 let 底線綁定並明標 InProgressOutcome 型別 <!-- speclink-task:tsk_01M1DYZ9RSAVBBHF1AJHETKCHG -->
- [x] 2.4 遷移 crates/speclink-cli/src/verbs/discuss.rs 的 11 個 discuss 子指令（new、list、show、context、add-round、conclude、promote、link、seal、archive、discard）：同 2.2 形狀，該檔共 11 處呼叫點 <!-- speclink-task:tsk_01M1DYZ9RSFZNK9W22WDRR7BS7 -->
- [x] 2.5 遷移完成斷言：grep 確認 crates/speclink-cli/src/verbs/ 內 unreachable! 出現次數為零、let store: &dyn Store 為零、run_command 識別符為零；cargo build -p speclink-cli 零 dead-code warning <!-- speclink-task:tsk_01M1DYZ9RSVAPCEAFNF98BEBTM -->

## 3. 凍結輸出驗證

- [x] 3.1 cargo test -p speclink-cli --test it 全綠——凍結輸出整合測試位元級不變即 command-runtime 規格「覆蓋動詞輸出凍結」scenario 的驗收；不動任何 golden <!-- speclink-task:tsk_01M1DYZ9RSPHJTF969S802V8D5 -->
- [x] 3.2 cargo test -p speclink-core 全綠（含既有 golden 對照），確認 additive 轉換層未波及引擎既有行為 <!-- speclink-task:tsk_01M1DYZ9RSXB1GC37BPSZEGEDY -->
