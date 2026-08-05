## 1. 前置確認與紅測

- [ ] 1.1 依 design「D4 相依 verify-station-parity，verify 側以其封存後正典為基準」確認開工前置：verify-station-parity 已封存、verify-skill 正典（含「驗證收尾迴圈」）已併入 openspec/specs/verify-skill/spec.md、crates/speclink-core/assets/skills/verify.md 已存在；本 change 的 verify-skill delta 與正典條文比對無漂移（條文名或收尾語意有異則先以 /speclink-ingest 校正 delta 再續行）。驗證：speclink show verify-skill 輸出含「驗證收尾迴圈」且與 delta 的 BEFORE 基準一致 <!-- speclink-task:tsk_01KZ5RY8HX0C3YHF163WMEXPAW -->
- [ ] 1.2 撰寫技能註冊與生成紅測（規格「品質關卡技能的生成與正典化」；design「D1 quality 以正典技能 asset 承載，不新增引擎狀態」）：skill_verbization 增列 quality 技能案例（技能表含 name quality、fork false、disallow_edit false、for_codex true、body 非空）；render_golden 斷言 claude 與 codex snapshot 含 speclink-quality 技能檔生成段、CLAUDE.md／AGENTS.md 範本段 workflow 行含 quality 入口。檔案 crates/speclink-core/tests/it/skill_verbization.rs、crates/speclink-core/tests/it/render_golden.rs。驗證：cargo test -p speclink-core --test it 新增測試全紅 <!-- speclink-task:tsk_01KZ5RY8HXP0ANMXCVA37Z09VC -->

## 2. quality 正典技能

- [ ] 2.1 新增 quality 技能 asset 並註冊技能表（規格「兩站時序的編排行為」；design「D1 quality 以正典技能 asset 承載，不新增引擎狀態」）：asset 吸收 .claude/skills/speclink-quality/SKILL.md 手寫版全部語意——定位（只管時序，SHALL NOT 重述兩站檢查、工單與蓋章語意）、前提（change 任務全數完成，未完成依站內守門拒絕、不另設守門）、六步時序（review 檢查先不蓋章 → verify 檢查先不蓋章 → 兩站 findings 統一修正回主線依 TDD → review 複驗蓋章 → verify 複驗蓋章 → 建議封存，兩章接連落中間零編輯）、邊界情況（事後變卦照跑新站接受前章暫黃封存回綠；單站或都不跑不經本技能）。檔案 crates/speclink-core/assets/skills/quality.md、crates/speclink-core/src/skills.rs。驗證：1.2 紅測轉綠 <!-- speclink-task:tsk_01KZ5RY8HX316RGXPMXYXFM022 -->

## 3. 兩站 asset 的 quality 時序例外

- [ ] 3.1 review 技能 asset 補 quality 時序例外（規格「審查後的迴圈與收尾」；design「D2 兩站 asset 補 quality 時序例外，堵零缺失自動蓋章縫」）：於 quality 時序中零 findings 的 discovery 不當場蓋章、改走既有「先不蓋章」離場，蓋章延至 quality 複驗階段；單站直接呼叫時零 findings 仍自動蓋章。檔案 crates/speclink-core/assets/skills/review.md。驗證：golden 差異審閱僅含新例外段與版本欄位，任務 5.1 全綠後覆蓋 <!-- speclink-task:tsk_01KZ5RY8HXX03TGQ5VE10HQEW0 -->
- [ ] 3.2 verify 技能 asset 補同構例外（規格「驗證收尾迴圈」；design「D2 兩站 asset 補 quality 時序例外，堵零缺失自動蓋章縫」）：quality 時序中零 findings 記錄空 discovery round 後以「先不蓋章」結束；單站直接呼叫維持記錄後執行 verify stamp。檔案 crates/speclink-core/assets/skills/verify.md。驗證：同 3.1，golden 差異審閱僅含新例外段與版本欄位 <!-- speclink-task:tsk_01KZ5RY8HX8H2P6AESW75M326H -->

## 4. init 範本的 workflow 行與技能清單

- [ ] 4.1 更新 init 範本（規格「審查技能的生成與正典化」「品質關卡技能的生成與正典化」；design「D3 init.rs 範本的 workflow 行與技能清單條目」）：生成之 CLAUDE.md／AGENTS.md workflow 行改為 discuss? → propose → apply ⇄ ingest → (quality? | review? ∥ verify?) → archive，技能使用清單加入 quality 條目（觸發時機：事前已知兩站都跑時使用；只跑一站直接呼叫該站技能），claude 與 codex 兩 render target 同步。檔案 crates/speclink-core/src/init.rs。驗證：1.2 的 render_golden workflow 行斷言轉綠 <!-- speclink-task:tsk_01KZ5RY8HXD549CPFYWE8GY0ND -->

## 5. 版本提升與 golden 落地

- [ ] 5.1 提升 MARKER_VERSION 並乾淨樹再生 golden（design「D5 MARKER_VERSION 提升與乾淨樹 golden 再生」）：版本提升使既有專案 speclink update 重新生成技能檔與 CLAUDE.md／AGENTS.md；乾淨樹再生 assets.lock 與各 render target snapshot。檔案 crates/speclink-core/src/init.rs、crates/speclink-core/tests/golden/assets.lock、crates/speclink-core/tests/golden/claude.snapshot.md、crates/speclink-core/tests/golden/codex.snapshot.md、crates/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md。驗證：cargo test --workspace 全綠 <!-- speclink-task:tsk_01KZ5RY8HXA1YTHBSZQ3VAH3SH -->
- [ ] 5.2 本 repo 執行 speclink update 落地生成物：.claude/skills/speclink-quality/SKILL.md（引擎生成物取代手寫版）、.claude/skills/speclink-review/SKILL.md、.claude/skills/speclink-verify/SKILL.md、CLAUDE.md、AGENTS.md 刷新。驗證：人工核對 CLAUDE.md workflow 行含 (quality? | review? ∥ verify?)、技能清單含 quality 條目、三個技能檔 frontmatter version 為新 MARKER_VERSION 且內容與 asset 一致 <!-- speclink-task:tsk_01KZ5RY8HXRPFZ42T395Y8A63M -->

## 6. README 說明文件

- [ ] 6.1 README 兩站分工表補 quality 入口（design「D6 README 兩站分工表補 quality 入口」）：README.md 與 README.en.md 分工表補「兩站都跑 → /speclink-quality」列與時序一句（兩站檢查先不蓋章 → 統一修正 → 各自複驗 → 兩章接連蓋），原「兩站都跑時的蓋章時序慣例」句改指技能入口。檔案 README.md、README.en.md。驗證：內容審閱——分工表與 quality-skill 規格及討論 quality-skill-canonicalization 結論一致，中英兩版語意對齊 <!-- speclink-task:tsk_01KZ5RY8HXP2AYZ8XNV2XPHC3Y -->
