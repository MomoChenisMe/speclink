> 本 change 零程式碼邏輯、零 delta：只改字。替換依 design D1 按意思分流的替換對照表（修飾動作用「全部」、指一整套東西用「完整」），標題改名清單見 design D3。grep 一律排除 openspec/changes/archive、openspec/discussions、node_modules、target、dist。每批改完立刻跑該批驗證，不累積到最後。

## 1. 開工檢查（design D4 等 config-skill-rule-routing 封存後才開工）

- [x] 1.1 確認開工前提成立：config-skill-rule-routing 已封存，且沒有在途變更的 delta 碰 design D3 四個標題所在的 capability。驗證：`speclink list --json` 不再列出 config-skill-rule-routing，openspec/changes/archive/ 下有以 `-config-skill-rule-routing` 結尾的目錄；同一份輸出裡沒有任何在途變更的 `deltaCapabilities` 含 config-skill、delivery-baseline、manual-skill 或 review-skill。前提不成立時停下回報，不改任何檔案。前提成立後對範圍內 18 個檔（openspec/specs/ 下 13 份 spec.md、openspec/manual/ 四頁、openspec/config.yaml）重跑 `git grep -n 全量`，命中逐處對照 design D1；config-skill 以封存後的實際字面為準，對照表沒列到的新命中依 D1 原則判定替換詞並在回報中列出 <!-- speclink-task:tsk_01M4YSWFMT1Y92EA2YJT0TVMR5 -->

## 2. 詞條（design D2 「完整測試」詞條立在 LANGUAGE.md）

- [x] 2.1 openspec/LANGUAGE.md 詞彙末尾新增「完整測試」詞條，definition、avoid、why 三欄字面依 design D2（avoid 只放「全量」、不帶語境限定）；`speclink language show` 列出該詞條，vocabulary-guard 自動把「全量」納入守門詞且掃描面零命中。驗證：`speclink language show` 輸出含「### 完整測試」與 avoid「全量」；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全綠 <!-- speclink-task:tsk_01M4YSWFMTRVM1THAMXASWXSCB -->

## 3. 正式規格直編（design D1、D3 標題改名清單、D7 只改引用改名標題的那一行註解）

- [x] 3.1 測試範圍意思的四份規格改成「完整測試」／「完整驗證」：openspec/specs/config-skill/spec.md（含 Scenario「答全量時不寫規則」→「答完整測試時不寫規則」）、openspec/specs/delivery-baseline/spec.md（Purpose、Requirement「root 單一指令全量驗證」→「root 單一指令完整驗證」、內文對自身標題的引用與「root 全量驗證」）、openspec/specs/review-skill/spec.md（含 Scenario「專案未寫測試政策時跑全量」→「專案未寫測試政策時跑完整測試」）、openspec/specs/verify-skill/spec.md；同批把 scripts/release/delivery-gate.test.mjs 段落註解裡的舊需求標題改成「root 單一指令完整驗證」，該檔其餘內容不動。驗證：`speclink validate --specs --strict` 綠；`speclink show delivery-baseline` 的需求標題含「root 單一指令完整驗證」；這四份規格與 delivery-gate.test.mjs `git grep 全量` 零命中；`git diff --numstat` 對這五個檔的新增行數等於刪除行數（只換字、不增刪行） <!-- speclink-task:tsk_01M4YSWFMTGQ24C4MAFAQX0BV9 -->
- [x] 3.2 remote 與投影系列六份改成「全部重新載入／查詢／收斂／建立」、「發出全部失效的通知」與「完整投影」：openspec/specs/remote-resilience/spec.md、openspec/specs/remote-workspace-data/spec.md、openspec/specs/phase2-acceptance/spec.md、openspec/specs/server-event-stream/spec.md、openspec/specs/context-projection/spec.md、openspec/specs/server-context-api/spec.md，字面依 design D1 對照表的對應列；SHALL 與 WHEN／THEN 結構不變。驗證：`speclink validate --specs --strict` 綠；這六份 `git grep 全量` 零命中；`git diff --numstat` 對這六個檔的新增行數等於刪除行數 <!-- speclink-task:tsk_01M4YSWFMTEBAR0YKBQ0HPS2AF -->
- [x] 3.3 其餘三份：openspec/specs/manual-task-marker/spec.md 改成「全部任務計數」（含表頭「全部任務(完成/總數)」）、openspec/specs/manual-skill/spec.md 改成「全部重新產生」並把 Scenario「首次全量生成」改成「首次生成完整手冊」、openspec/specs/desktop-app/spec.md 改成「還原全部」（三處）。驗證：`speclink validate --specs --strict` 綠；這三份 `git grep 全量` 零命中；`git diff --numstat` 對這三個檔的新增行數等於刪除行數 <!-- speclink-task:tsk_01M4YSWFMT84G34ATJ7R4RKBZ1 -->

## 4. 手冊與設定（design D5 手冊直接改字、不重生；D6 config.yaml 的規則經 workflow-config rules 寫入）

- [x] 4.1 手冊四頁依 design D1 直接改字：openspec/manual/manual.md（全部重新產生）、openspec/manual/policy-config.md（完整測試，兩處）、openspec/manual/remote-context.md（全部重新建立、完整投影）、openspec/manual/remote-offline.md（發出全部失效的通知、全部重新查詢）；frontmatter 含 `generated` 一字不動，不執行 `/speclink-manual`。驗證：這四頁 `git grep 全量` 零命中；`git diff -U0 openspec/manual/` 的變動行全部落在 frontmatter 結束的 `---` 之後；`git diff --numstat` 對這四頁的新增行數等於刪除行數 <!-- speclink-task:tsk_01M4YSWFMTQ7XJK11R994SDG0A -->
- [x] 4.2 openspec/config.yaml 的 tasks 規則「全量 `npm run test:all` 只由 CI（.github/workflows/ci.yml）守門」改成「完整測試 `npm run test:all` 只由 CI（.github/workflows/ci.yml）守門」，其餘規則逐字不變、順序不變：以 `speclink workflow-config show --json` 取出 tasks 段規則、只換這一條的字詞，先經 `speclink workflow-config rules tasks --stdin --dry-run` 看 diff。diff 只有這一行時去掉 `--dry-run` 寫入；diff 出現其他行（例如區塊樣式或引號被重寫）時改為直接編輯 openspec/config.yaml 的這一行（只替換字詞，不需跳脫）。驗證：openspec/config.yaml `git grep 全量` 零命中；`git diff --numstat openspec/config.yaml` 為 1 增 1 刪；`speclink instructions tasks --change taiwan-full-wording --json` 的 rules 含「完整測試 `npm run test:all`」 <!-- speclink-task:tsk_01M4YSWFMT3VZJ5P3HNRP0WNSE -->

## 5. 驗收（design D8 驗收以 grep、驗證指令與抽審三道完成）

- [x] [M] 5.1 使用者抽審改動最多的句子：openspec/specs/config-skill/spec.md 的第五問段落與確認表、openspec/specs/remote-resilience/spec.md 的恢復段落、openspec/specs/manual-task-marker/spec.md 的計數段落與表格、openspec/specs/desktop-app/spec.md 的搜尋還原段落；讀起來不通順或意思分流用錯的句子提出後逐句修正，修正後重跑該批驗證。抽審通過即驗收完成 <!-- speclink-task:tsk_01M4YSWFMTV5JVXP9F2K0K2KTC -->
- [x] 5.2 收尾驗證三道全過：範圍內 18 個檔（13 份規格、4 頁手冊、openspec/config.yaml）`git grep 全量` 零命中；design D3 四個舊標題在全 repo（排除 openspec/changes/archive 與 openspec/discussions）`git grep` 零命中；`speclink validate --specs --strict` 綠；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全綠；`git status` 只含 proposal Impact 列出的檔案與本變更的 artifacts，技能原稿、三份 i18n、golden、assets.lock、封存區 git diff 為空 <!-- speclink-task:tsk_01M4YSWFMTM8DD30KAS58DW0HQ -->
