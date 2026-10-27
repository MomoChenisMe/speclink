## 1. 更新 discuss 技能事實來源（assets）

- [x] 1.1 於 crates/speclink-core/assets/skills/discuss.md 加入文件輸入紀律（對應 Requirement: 文件作為預填樹來源逐條分診）：topic 指定文件路徑（自寫 markdown、plan mode 產出、repo 內 docs、任意可讀路徑）時，讀取文件並萃取主張作為決策樹節點，逐條對 codebase 分診為證實（附程式碼證據）／牴觸（逐條指出文件與程式碼實況差異並附證據）／真決策（送使用者裁定）三類，並明定文件不得僅作背景素材一次性閱讀。驗證：檢視該檔含三類分診規則與逐條牴觸呈現的規定，且既有決策樹遍歷、Evidence 硬規則、事實分診段落未被改動。 <!-- speclink-task:tsk_01KYTXDXCYWHANHBVGKM95KQSD -->
- [x] 1.2 於 crates/speclink-core/assets/skills/discuss.md 的記錄規則加入 Source doc 慣例（對應 Requirement: Source doc 記錄慣例）：以文件為輸入的討論，Context 固定含一行 Source doc: <路徑>；輪 Evidence 引用文件時以段落標題或短句為之；記錄只存討論結果、不內嵌整份規劃文件；不修改使用者的原始文件。驗證：檢視記錄規則段落含上述四點，且明示未給文件時流程照舊（Context 無 Source doc 行、無額外讀檔步驟）。 <!-- speclink-task:tsk_01KYTXDXCYSDZ9ZW67HH5MHPJC -->

## 2. 更新 propose 技能事實來源（assets）

- [x] 2.1 於 crates/speclink-core/assets/skills/propose.md 的 from-discussion 流程加入文件跟隨與疊加語意（對應 Requirement: from-discussion 跟隨 Source doc 引用與疊加語意）：討論記錄的 Context 含 Source doc: <路徑> 行時讀取原始文件，以「文件為底層、討論為勝出層」合成——討論有決定的以討論為準、討論未觸及的文件內容補位採用、討論 Ruled out 的內容不得出現於提案。驗證：檢視該檔 from-discussion 段落含疊加語意三規則與 Ruled out 不復活的明文規定，並明示記錄無 Source doc 行時流程照舊。 <!-- speclink-task:tsk_01KYTXDXCYMM0PQ8RP715FW8MW -->
- [x] 2.2 於 crates/speclink-core/assets/skills/propose.md 的需求來源判定加入 --from-doc 入口（對應 Requirement: from-doc 直接文件入口）：認得 --from-doc <路徑> 引數慣例，讀取指定文件作為需求來源建立提案、無需既存討論；優先序更新為明確引數 → --from-doc → 討論記錄 → plan 檔偵測 → 對話上下文，既有 plan 檔偵測段落保留。驗證：檢視該檔需求來源段落含 --from-doc 慣例與五級優先序，plan 檔偵測（步驟 1c）內容未刪，且明示未帶 --from-doc 時判定照舊。 <!-- speclink-task:tsk_01KYTXDXCYTDK1ZJGTFG9MC8RB -->

## 3. 再生 render golden（刻意更新）

- [x] 3.1 以 git status 確認除本變更的 assets 編輯外，無其他未提交改動會影響渲染輸入，然後帶 UPDATE_GOLDEN=1 執行 cargo test -p speclink-core --test render_golden 再生 crates/speclink-core/tests/golden/claude.snapshot.md、crates/speclink-core/tests/golden/codex.snapshot.md、crates/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md。驗證：git diff 顯示四份 snapshot 僅含 speclink-discuss 與 speclink-propose 技能文字的預期變更，無其他技能內容或未提交狀態滲入。 <!-- speclink-task:tsk_01KYTXDXCY94A2C2NGX02KZHQC -->
- [x] 3.2 不帶 UPDATE_GOLDEN 重跑 cargo test -p speclink-core --test render_golden。驗證：測試綠燈，golden 與渲染輸出一致。 <!-- speclink-task:tsk_01KYTXDXCYC5BAK88CQ8E1WE5G -->

## 4. 同步工具技能實例

- [x] 4.1 將任務 1.1–1.2 的相同內容變更套用至 .claude/skills/speclink-discuss/SKILL.md 與 .agents/skills/speclink-discuss/SKILL.md，任務 2.1–2.2 套用至 .claude/skills/speclink-propose/SKILL.md 與 .agents/skills/speclink-propose/SKILL.md（各檔與 assets 版僅 frontmatter／工具殼差異；涵蓋 Requirement: 文件作為預填樹來源逐條分診、Source doc 記錄慣例、from-discussion 跟隨 Source doc 引用與疊加語意、from-doc 直接文件入口 的相同文字）。驗證：以 diff 比對兩技能各三處檔案的文件輸入紀律、Source doc 慣例、疊加語意、--from-doc 段落，內文一致。 <!-- speclink-task:tsk_01KYTXDXCYQMZ0QPS2SMTBGPBD -->

## 5. 收尾走查

- [x] 5.1 對照兩份 delta spec（openspec/changes/discuss-propose-from-docs/specs/discuss-skill/spec.md、openspec/changes/discuss-propose-from-docs/specs/propose-skill/spec.md）逐場景走查：四個 Requirement（文件作為預填樹來源逐條分診、Source doc 記錄慣例、from-discussion 跟隨 Source doc 引用與疊加語意、from-doc 直接文件入口）的九個 Scenario 均能在渲染產物（golden snapshot）與六處技能檔中指認對應文字。驗證：執行 speclink validate discuss-propose-from-docs 通過，走查清單九項全數指認成功。 <!-- speclink-task:tsk_01KYTXDXCYP1HNM1K3RSEESPAR -->

## 6. 折入 verify 發現：from-doc 出處記錄與使用者文件補記

- [x] 6.1 於 crates/speclink-core/assets/skills/propose.md 的 --from-doc 段（步驟 1b）加入出處記錄規則（對應 Requirement: from-doc 直接文件入口／Scenario: from-doc 提案留存出處）：以 --from-doc 建立的提案，proposal 的 Why 或 Impact SHALL 含一行 Source doc: <路徑> 留存來源文件出處，並明示此為技能文字約定、引擎零改動；同批將相同文字套用至 .claude/skills/speclink-propose/SKILL.md 與 .agents/skills/speclink-propose/SKILL.md。驗證：diff 比對三處檔案的 --from-doc 段均含出處記錄規則且內文一致，段內其餘既有文字未被改動。 <!-- speclink-task:tsk_01KYTZ0GKMJJCKFXZKKG7YKVVE -->
- [x] 6.2 帶 UPDATE_GOLDEN=1 執行 cargo test -p speclink-core --test render_golden 再生 crates/speclink-core/tests/golden/claude.snapshot.md、codex.snapshot.md、neutral-cli.snapshot.md、neutral-tool-call.snapshot.md，再不帶 UPDATE_GOLDEN 重跑同測試。驗證：git diff 顯示四份 snapshot 相對前一版僅新增 --from-doc 出處記錄文字、無其他技能內容滲入，重跑測試綠燈。 <!-- speclink-task:tsk_01KYTZ0GKMRRMX5S2P9PP3971P -->
- [x] 6.3 於 docs/workflow.md 與 docs/workflow.zh-TW.md 補記兩個新輸入入口：discuss 段的 Input 行補「topic 可為文件路徑（自寫計劃、plan mode 產出等）」；propose 段的 Input 行與 Claude/Codex 呼叫行補 --from-doc <path> 變體。不動 docs/getting-started 系列（--from-doc 非引擎旗標，寫入違反正典「Getting Started 僅使用已驗證入口」）。驗證：兩檔的 discuss 與 propose 段均含新入口且中英文語意對等；grep docs/getting-started* 無 --from-doc。 <!-- speclink-task:tsk_01KYTZ0GKM495BJ4CSEKZ3XN5T -->
- [x] 6.4 收尾走查：speclink validate discuss-propose-from-docs 通過；新增 Scenario（from-doc 提案留存出處）可於三處 propose 技能檔（assets／.claude／.agents）與四份 golden snapshot 中指認對應文字。驗證：validate exit 0，指認清單全數命中。 <!-- speclink-task:tsk_01KYTZ0GKM8HWJ0W49JK6E83V3 -->
