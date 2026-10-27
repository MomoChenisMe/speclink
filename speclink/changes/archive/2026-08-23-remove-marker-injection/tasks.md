## 1. 引擎面——拆注入與遺留剝除（TDD：先讓既有 marker 測試轉紅再改實作）

- [x] 1.1 依 design D1: 注入拆除面，實作規格需求「init 內建 Agent 工具選擇」「工作區補齊入口」「工具檔生成不寫入 AI 工具的使用者設定檔」與 remote-connection「remote 初始化與連接指令」的無 marker 生成物集合，並落地 REMOVED 需求「marker 技能指引跟隨 worktree 政策」「指令區塊的 remote 變體」：crates/speclink-core/src/init.rs 移除 instructions_body、custom_instructions_body、upsert_marker 及 fs init、remote init、update、tools 收斂、adopt 各路徑的 marker 寫入；受管生成物縮為技能檔、.speclink.yaml、.gitignore 條目。同批移除 Node SDK 的 instructions.render 公開面（crates/speclink-node/src/render.rs 的 instructions_render 匯出、index.js／index.d.ts 條目、__test__/render.spec.ts 對應測項）——兩支 body 函式的唯一外部消費者。驗證：cargo test -p speclink-core 中原斷言「marker 生成」的單元測試改斷言「指令檔不存在或位元級不變」後全綠。 <!-- speclink-task:tsk_01M0P57HK2XHNG02RJ6AB1NFRJ -->
- [x] 1.2 依 design D2: 遺留剝除（update 一律剝），實作規格需求「built-in tools 權威收斂」與「描述子的同步與清理生命週期」的剝除語意：對內建工具（CLAUDE.md、AGENTS.md）與仍帶 instructions_file 欄位的描述子，偵測 SPECLINK:START..END 區塊即以既有 remove_marker 語意剝除（保留使用者內容、全空刪檔、無區塊零觸碰），stdout 摘要列出剝除檔案。驗證：新增單元測試涵蓋「含使用者內容剝除後保留」「純 marker 檔剝除後刪除」「無 marker 位元級不變」三情境全綠。 <!-- speclink-task:tsk_01M0P57HK2NST9BAJFPG33ZNYZ -->
- [x] 1.3 依 design D3: instructions_file 欄位棄用，實作規格需求「tools 自訂描述子的接受與驗證」：crates/speclink-core/src/config.rs 將該欄位由 require_field 轉選填（存在時仍驗證專案根相對與不逸出），workspace 檢查對殘留欄位輸出一行棄用提示（非錯誤）。驗證：cargo test -p speclink-cli --test it tools_descriptor 更新後全綠，含「無 instructions_file 的描述子被接受」與「殘留欄位得棄用提示」兩案。 <!-- speclink-task:tsk_01M0P57HK2W5K3A85PVN10NKV2 -->
- [x] 1.4 依 design D6: 版號探測與降版拒絕改基準，以新規格需求「技能檔過期探測」取代 REMOVED 的「指令檔過期探測」，並同步實作「受管檔再生的降級守門」「產物層版本戳同源」「引擎版號查詢面」的技能檔版號語意：instruction_status／differing_files／refuse_downgrade 的版本來源改為各工具 skills 目錄技能檔 frontmatter 的 version 欄位（缺失＝目錄無任何 speclink- 技能檔；五態與聚合優先序、數值比較規則不變），MARKER_VERSION 常數更名 ASSET_VERSION。驗證：init.rs 探測單元測試依新基準改寫後全綠，cargo test -p speclink-cli --test it update_downgrade_guard 與 engine_version 全綠。 <!-- speclink-task:tsk_01M0P57HK2DZTPFD5RC20HMENV -->

## 2. 資產面——入口 description 與出口交棒句（skill-routing spec 落地）

- [x] 2.1 依 design D4: 入口路由——description 觸發情境句，實作 skill-routing 規格需求「入口路由由技能描述承載」：改寫 crates/speclink-core/src/skills.rs registry() 的 18 個對外技能 description 為觸發情境句（先情境後產出，英文），逐句對照原 marker 路由表 bullet 確認全部入口情境被涵蓋。驗證：內容審閱對照 spec 需求列出的入口情境清單逐項命中，無情境落空。 <!-- speclink-task:tsk_01M0P57HK2J8HBX2E4DGQZQARM -->
- [x] 2.2 依 design D5: 出口路由——交棒句邊集，實作 skill-routing 規格需求「出口交棒由技能結尾承載」「去中心化路由不留集中總表」「內部技能不參與路由」：於 crates/speclink-core/assets/skills/ 的流程鏈技能資產（propose、apply、apply-worktree-post、worktree-merge、drift、ingest、review、verify、quality、onboard、discuss、improve）補齊或統一結尾的下一步建議段，明文僅建議不代跑；工具技能與 archive 不加固定出邊；onboard 出口僅兩條邊、不帶命令總表。驗證：逐檔審閱對照 design D5 表，邊集完整且無技能自動呼叫字句。 <!-- speclink-task:tsk_01M0P57HK2RCX5X76HA46VNMET -->

## 3. 三連動與 golden 重整

- [x] 3.1 依 design D7: golden 面重整落地三連動並覆蓋規格需求「中性渲染目標」「worktree 技能的政策條件式生成」的新基線：ASSET_VERSION bump（minor 位）；刪除 crates/speclink-core/tests/golden/remote-claude.marker.md 並移除 crates/speclink-core/tests/it/render_golden.rs 的 marker 專屬測項；以刻意更新流程再生 claude.snapshot.md、claude-worktree.snapshot.md、codex.snapshot.md、neutral-cli.snapshot.md、neutral-tool-call.snapshot.md（再生後不含任何指令檔段）與 assets.lock。驗證：cargo test -p speclink-core 全綠，含 render_golden 與資產鎖定測試。 <!-- speclink-task:tsk_01M0P57HK26ZBGFMHMKFY5Q2P4 -->
- [x] 3.2 調整 CLI 整合測試面：crates/speclink-cli/tests/it/ 中斷言 marker 生成或內容的測試（remote_connect、workflow_config 等）改斷言指令檔不生成與遺留剝除行為，覆蓋 remote-connection 規格需求「remote 初始化與連接指令」的無指令檔場景。驗證：cargo test -p speclink-cli --test it 全綠。 <!-- speclink-task:tsk_01M0P57HK2BTAMQBECVVERRMPX -->

## 4. desktop 面

- [x] 4.1 依 design D6 調整 apps/desktop/core/src/project.rs：過期探測改讀技能檔 frontmatter 版本，UI 過期提示措辭由指令檔改為技能檔（apps/desktop/src/i18n/messages.ts 對應詞條）。驗證：desktop core 探測單元測試依新基準改寫後全綠。 <!-- speclink-task:tsk_01M0P57HK2KCX0MP43KRC6MKS9 -->
- [x] 4.2 依 desktop-config 規格需求「未初始化目錄經確認後自動初始化」與「設定頁圖形化讀寫兩層設定」的新生成物集合，調整 apps/desktop/core/src/settings.rs 與 apps/desktop/src-tauri/src/connections.rs：初始化與 tools 同步不再生成 marker、同步時剝除遺留區塊，測試斷言由「AGENTS.md 含 marker」改為「AGENTS.md 不生成／遺留區塊被剝除」。驗證：speclink-desktop 相關 crate cargo test 全綠（依 macOS 環境慣例先補 sidecar 與 server-web dist）。 <!-- speclink-task:tsk_01M0P57HK2VVQ13D93EFZP82PG -->

## 5. 文件面

- [x] 5.1 更新 docs/getting-started.md、docs/getting-started.zh-TW.md、docs/configuration.md、docs/configuration.zh-TW.md、docs/verb-contract.md、docs/verb-contract.zh-TW.md、docs/platform-architecture.zh-TW.md、docs/server-store-drivers.zh-TW.md、docs/implementation-refactor-roadmap.zh-TW.md、docs/sdk-node.md、docs/sdk-node.zh-TW.md：marker 注入敘述改為技能路由敘述（init／update 產物清單、instructions_file 棄用說明、update 自動剝除遺留區塊的行為）；sdk-node 兩份的 Render API 段與 Copilot 整合範例移除 instructions.render，改述自建 harness 由技能檔 description 承載路由。驗證：grep 上列檔案無「SPECLINK:START」與「marker 區塊會生成」類殘句（歷史段落與 roadmap 回顧除外，逐檔人工確認語境）。 <!-- speclink-task:tsk_01M0P57HK2SKRQ44G6WJ6ACSRX -->

## 6. 收尾回歸

- [x] 6.1 逐 crate 全量驗證與提交面盤點：cargo test -p speclink-core、cargo test -p speclink-cli --test it、desktop 相關 crate 測試全綠；speclink validate remove-marker-injection 通過（delta 依 design D8: delta 宣告紀律可乾淨套用，REMOVED 與 REMOVED-SCENARIO 宣告齊備）；git status 盤點——golden 快照、assets.lock、speclink update 再生的全部渲染 SKILL.md、本 repo 的 CLAUDE.md／AGENTS.md 剝除結果均納入提交清單，工作樹無未認領檔案。驗證：上列測試面全綠且盤點清單與 proposal Impact 一致。 <!-- speclink-task:tsk_01M0P57HK22V6YQRB3D6S9GEP4 -->
