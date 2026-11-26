## 1. 引擎：tasks 指示與範本

- [ ] 1.1 schema 測試（Red）：在 crates/engine/speclink-core/src/workspace/schema.rs 的測試模組，為需求「tasks 指示說明短標題的任務形狀」加斷言——`SPEC_DRIVEN_SCHEMA_YAML` 與 `NO_SPEC_CHANGE_SCHEMA_YAML` 的 tasks instruction 都含「驗證：」「検証：」「Verify: 」三個標記與繁體中文、日文、英文三種形狀各一行；兩份 instruction 的 Example 區塊裡每個任務行都含「: 」與「Verify: 」；`A_TASKS_TMPL` 含四個新的佔位文字 `<!-- Short title: behavior/contract delivered. Verify: verification target -->`、不含舊的佔位文字；no-spec-change 的 instruction 仍含「Unchanged behavior (REQUIRED)」。驗證：`cargo test -p speclink-core --lib schema` 因指示與範本還沒改而失敗。 <!-- speclink-task:tsk_01M8AMCY7A937G8EXW0JKDWVAB -->
- [ ] 1.2 spec-driven 的指示（Green，design D1）：在 crates/engine/speclink-core/assets/schema/spec-driven/fork.schema.yaml 的 tasks instruction，於「Behavior + verification」那一條之後加「Short title first」一條，內容是任務的形狀、三種語言各一行的寫法、依 `locale` 選一組且其他語言用英文那一組、標題是短語且不含冒號、標記不翻譯；Example 區塊的四個任務改成英文的形狀。驗證：1.1 裡 spec-driven 的斷言通過。 <!-- speclink-task:tsk_01M8AMCY7A69K7F7NT7VJS74H3 -->
- [ ] 1.3 no-spec-change 的指示（Green，design D1）：在 crates/engine/speclink-core/assets/schema/no-spec-change/fork.schema.yaml 的 tasks instruction 加同一條，Example 區塊的任務改成英文的形狀；「Unchanged behavior (REQUIRED)」與「不引用 requirement 名稱」兩段逐字不動。驗證：1.1 裡 no-spec-change 的斷言通過。 <!-- speclink-task:tsk_01M8AMCY7AHGFJ3EFR771D22FT -->
- [ ] 1.4 任務範本（Green，design D3）：crates/engine/speclink-core/assets/schema/spec-driven/tasks.template.md 開頭的說明加一句任務的形狀並指向 tasks 指示，四個任務的佔位文字改成新文字。驗證：`cargo test -p speclink-core --lib schema` 全部通過。 <!-- speclink-task:tsk_01M8AMCY7ASCWD52NFZHWWF292 -->
- [ ] 1.5 不成為格式守門：在 crates/adapters/speclink-cli/tests/it/new_artifact.rs 加一個測試，寫入只有「- [ ] 1.1 一句沒有冒號也沒有驗證標記的長句」的任務清單後跑 validate，斷言兩個指令的 exit code 為 0、validate 的輸出不含 title、標題、標記這些字；另在 crates/adapters/speclink-cli/tests/it/instructions_policy.rs 加一個測試，對 spec-driven 與 no-spec-change 的變更各取一次 tasks 指示的 `--json`，斷言 payload 有字串欄位 `instruction` 與 `template`、`instruction` 含「Verify: 」、`template` 含新的佔位文字。驗證：`cargo test -p speclink-cli --test it new_artifact` 與 `cargo test -p speclink-cli --test it instructions_policy` 通過。 <!-- speclink-task:tsk_01M8AMCY7AF2SM3F3Y5VKT86FV -->
- [ ] 1.6 既有輸出的回歸：確認 schema fork 仍與正典 YAML 逐位元組相同、render 的 golden 不需要更新、`ASSET_VERSION` 與 crates/engine/speclink-core/tests/golden/assets.lock 沒有變動。驗證：`cargo test -p speclink-core` 全部通過且 crates/engine/speclink-core/tests/golden 沒有任何檔案被改；`cargo test -p speclink-cli --test it schema_commands` 通過。 <!-- speclink-task:tsk_01M8AMCY7A3ESNRMY8FBD2MDCE -->

## 2. packages/ui：任務切分

- [ ] 2.1 切分測試（Red）：在 packages/ui/src/__tests__/docModel.test.ts 的 `splitTaskText` 區塊，為需求「任務群組標題與章節標籤同款式」的新情境加 design Implementation Contract 表裡的六列——繁體中文、日文（「検証：」）、英文（「: 」與「Verify: 」）、有全形冒號時不看半形冒號（含 D 籤 `D2`）、行內程式碼裡的半形冒號不切、沒有任何標記。驗證：`npm test -w @speclink/ui -- docModel` 的日文與英文兩列失敗，既有「三層切分」的四個例子通過。 <!-- speclink-task:tsk_01M8AMCY7ANBXXZQFEY58AERN4 -->
- [ ] 2.2 三組標記的切分（Green，design D2）：packages/ui/src/docModel.ts 的 `splitTaskText` 取「驗證：」「検証：」「Verify: 」最先出現的一個當驗證標記；前段有全形「：」時照舊在首個全形「：」切，沒有時才在首個「半形冒號加一個空白」切；「Verify: 」與半形冒號落在成對反引號之間時不算；函式簽名與 `TaskTextParts` 不變，函式上方的說明註解改成新規則。驗證：`npm test -w @speclink/ui -- docModel` 全部通過。 <!-- speclink-task:tsk_01M8AMCY7A6XYGNPSA8BK2H9H7 -->
- [ ] 2.3 任務列顯示英文任務的三層：在 packages/ui/src/__tests__/taskList.test.tsx 加一個測試，渲染未勾選的任務「1.3 Export CSV: the command writes a file with a header row. Verify: cargo test -p demo export_csv」，斷言標題層是「Export CSV」、列尾箭頭鈕的 aria-expanded 為 true、內文與驗證各自顯示。驗證：`npm test -w @speclink/ui -- taskList` 通過。 <!-- speclink-task:tsk_01M8AMCY7AJ46M2429YWFNT4VG -->

## 3. 文件

- [ ] 3.1 切分規則的章節（design D4）：依需求「動詞參考文件載明任務行的切分規則」，在 docs/verb-contract.zh-TW.md 與 docs/verb-contract.md 各加一節（錨點 `task-line-shape`），寫明勾選框與 `[M]`、行尾 stable ID 註解先移除、三個驗證標記取最先出現的一個、全形冒號優先於「半形冒號加空白」、行內程式碼的例外、切不到時整段為標題，附三種語言的例子表，並說明這是 tasks 指示給寫入端的建議、既有的任務不保證符合；在講 `show --json` 欄位的段落加一個連到這一節的連結。兩份的例子表列數相同、標記逐字相同。驗證：`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過（中英對照、連結與詞彙守門）。 <!-- speclink-task:tsk_01M8AMCY7AJRJ29QDJG00E2HXM -->

## 4. 收尾

- [ ] 4.1 受影響面的全部測試：確認這個變更沒有改到任何 CLI 指令、`--json` 欄位與技能檔，三個受影響面都通過。驗證：`cargo test -p speclink-core`、`cargo test -p speclink-cli --test it instructions_policy`、`npm test -w @speclink/ui`、`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全部通過。 <!-- speclink-task:tsk_01M8AMCY7APD6MBNB629WAENSH -->
