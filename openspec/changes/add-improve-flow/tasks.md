## 1. 引擎與 CLI:kind 欄位與 --kind 白名單旗標

- [ ] 1.1 撰寫 kind 欄位測試(規格「討論記錄以 --kind 標記改進討論」):crates/speclink-core/src/discuss.rs 的 #[cfg(test)] 增 new_discussion 帶 kind 寫入 frontmatter kind: improve、非白名單值拒絕不落檔、無 kind 舊記錄讀取視為一般討論;crates/speclink-cli/tests/ 整合測試增 speclink discuss new --kind improve 建檔且 --json payload 含 kind: "improve"(camelCase、字串型別)、--kind 非法值非零 exit 且 stderr 說明僅接受 improve 且 openspec/discussions/ 無新檔、未帶 --kind 人眼與 --json 輸出逐位元不變、discuss list --json 與 discuss show --json 有 kind 曝露無 kind 省略鍵。驗證:cargo test 新測試全紅 <!-- speclink-task:tsk_01KYWGZ9WKNVQ3NM3XTE1RWJRH -->
- [ ] 1.2 實作 design 決策「kind 落在討論 frontmatter,discuss new 增 --kind 白名單旗標」:白名單常數單點定義於 speclink-core,new_discussion 增 kind 參數寫 frontmatter,讀取端組裝 kind;speclink-cli 的 discuss new 增 --kind 旗標接線,驗證失敗走非零 exit 與 stderr。檔案 crates/speclink-core/src/discuss.rs、crates/speclink-cli/src/main.rs。驗證:1.1 測試全綠,cargo test -p speclink-core --test it render_golden:: 與既有 CLI 測試不受影響 <!-- speclink-task:tsk_01KYWGZ9WK3Z0901VAFGVR4AT9 -->

## 2. 協定:DiscussionInfo 選填 kind

- [ ] 2.1 實作 design 決策「DiscussionInfo 增選填 kind,protocol 單一正典流至 desktop」(規格「討論資訊 payload 增選填 kind 欄位」):crates/speclink-protocol/src/query.rs 的 DiscussionInfo 增 Option 欄位、無值省略序列化;serde 測試斷言有值時鍵名為 camelCase 的 kind 且值為 improve、無值時 payload 不含該鍵與既有形狀一致;server 讀取路徑與型別化 client 以既有測試模式各驗一筆改進討論曝露 kind。檔案 crates/speclink-protocol/src/query.rs、crates/speclink-remote/tests/。驗證:cargo test --workspace 全綠 <!-- speclink-task:tsk_01KYWGZ9WKYQ77W4S2RRGPEEN2 -->

## 3. 技能模板:improve.md

- [ ] 3.1 撰寫 design 決策「improve.md 模板自包含六步骨架,精髓段逐字保留」的模板(規格「improve 技能以六步骨架渲染至兩工具」「掃描精髓段逐字保留」「candidates 以討論記錄承載」):新增 crates/speclink-core/assets/skills/improve.md,含六步骨架、僅使用者發起與不得實作限定、防重提檢查(已封存討論 Ruled out 與 in-flight changes 迴避)、範圍收斂(方向優先/git log 熱點加權近期變更輔以 openspec/changes/archive 的 touched 記錄/分散放寬網)、五條 friction 訊號逐條、有機探索精神、deletion test 准入(集中才算搬家不算)、inline 預設與 Explore subagent 硬上限 2、discuss new --kind improve 與 Round 1 mode 標籤 scan、candidate 五欄位(Files/Problem/Solution/Wins/建議強度三級)與首選建議、grilling 一次一題帶證據與 depth check 無條件、全數否決走 conclude 加 archive 禁止 discard。驗證:內容審閱逐項對照三條規格的場景點名內容全數命中 <!-- speclink-task:tsk_01KYWGZ9WKQ5DTVQ5M4DSQ2PMM -->
- [ ] 3.2 接線渲染並再生 golden:improve 模板納入 init/update 渲染清單(claude 與 codex 兩份),乾淨樹再生 crates/speclink-core/tests/golden 快照。驗證:cargo test -p speclink-core --test it render_golden:: 全綠,兩份生成檔均含五條 friction 訊號與 subagent 硬上限 2 字樣 <!-- speclink-task:tsk_01KYWGZ9WK8955CYSEDT4E42S2 -->

## 4. desktop:卡片與抽屜的改進標示

- [ ] 4.1 撰寫 UI 測試(規格「看板討論卡片的改進標示」「討論抽屜的改進標示」):kind 為 improve 的討論卡片渲染行內小章且 tooltip 為「改進討論」(tw)、一般討論無新增元素、已封存側維持標示;DiscussionDrawer 於 improve 顯示標示、一般討論不顯示;en 詞條對應。檔案 packages/ui/src/__tests__/。驗證:npm test -w packages/ui 新測試全紅 <!-- speclink-task:tsk_01KYWGZ9WKGS0HKE8A5SPP86MS -->
- [ ] 4.2 實作 design 決策「desktop 小章鏡像審查章樣式」:packages/ui/src/adapter.ts 增列 kind 欄位,DiscussionColumn 卡片行內小章(lucide 既有 icon 家族＋Tooltip,不加文字列),DiscussionDrawer 同步標示,標示隨 kind 恆定不隨生命週期變化。檔案 packages/ui/src/adapter.ts、packages/ui/src/components/DiscussionColumn.tsx、packages/ui/src/components/DiscussionDrawer.tsx。驗證:4.1 測試全綠 <!-- speclink-task:tsk_01KYWGZ9WKA469J4BW7BDN6H73 -->
- [ ] 4.3 增列 i18n 詞條:tw「改進討論」、en 對應詞條。檔案 packages/ui/src/i18n.tsx、apps/desktop/src/i18n/messages.ts。驗證:npm test -w packages/ui 與 npm test -w apps/desktop 全綠 <!-- speclink-task:tsk_01KYWGZ9WKB5J2NR5X1Y9WMGMT -->

## 5. 詞彙與文件同步

- [ ] 5.1 openspec/LANGUAGE.md 增正典詞「改進討論」條目(definition:kind 為 improve 的討論——由 /speclink-improve 掃描 codebase 主動提出 candidates 的討論記錄;avoid:improve 討論、架構討論;why:名詞直說內容,與卡片 tooltip 及抽屜標示文案一致)。驗證:內容審閱——詞條與 4.3 的 tw 文案一致 <!-- speclink-task:tsk_01KYWGZ9WKMPX5D7CC4ACVGPBC -->
- [ ] 5.2 實作 design 決策「文件同步經 speclink update 落地,README 手改」前半:CLAUDE.md 與 AGENTS.md 注入區塊模板的 workflow 段更新為 discuss?/improve? → propose 並補 improve 觸發時機一行,乾淨樹再生 golden 後於本 repo 執行 speclink update 落地 .claude/skills/speclink-improve/ 與兩注入區塊。驗證:cargo test -p speclink-core --test it render_golden:: 全綠;CLAUDE.md 與 AGENTS.md 注入區塊含 improve 入口 <!-- speclink-task:tsk_01KYWGZ9WK5V8H0XCF49403Q5K -->
- [ ] 5.3 實作 design 決策「文件同步經 speclink update 落地,README 手改」後半:README.md 與 README.en.md 的 workflow 圖加入 improve、新增 improve 一節(何時用、與 discuss 的分工——使用者帶題走 discuss、要模型提案走 improve)。驗證:內容審閱——分工敘述與討論 architecture-improve-flow 結論一致 <!-- speclink-task:tsk_01KYWGZ9WKXCM0BY4SV71N8EXY -->

## 6. 端到端與收尾驗證

- [ ] 6.1 E2E 走查:speclink discuss new 帶 --kind improve 建立改進討論 → discuss list --json 與 show --json 曝露 kind → dev harness 或 desktop 確認卡片小章與抽屜標示(tw 文案「改進討論」)→ 未帶 --kind 的既有流程輸出不變。驗證(只跑受影響面):cargo test -p speclink-core -p speclink-cli -p speclink-protocol -p speclink-remote -p speclink-desktop-core 全綠、cargo test -p speclink-server --test it discussion_routes:: 全綠(server 讀取面)、npm test -w packages/ui 與 npm test -w apps/desktop 全綠、crates/speclink-node 的 napi build 與 npm test 全綠(skill registry 與注入區塊渲染面)、speclink validate add-improve-flow 通過;全量 npm run test:all 由 CI 守門 <!-- speclink-task:tsk_01KYWGZ9WKXHJYX686EYAB3QEA -->
