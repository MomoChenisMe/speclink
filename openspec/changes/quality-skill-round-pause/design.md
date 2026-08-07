## Context

/speclink-quality 是收尾階段的兩站編排技能，2026-08-07 隨 quality-skill-canonicalization 正典化進引擎——事實來源 crates/speclink-core/assets/skills/quality.md，經 init 與 update 渲染至 claude 與 codex 工具技能目錄。現行時序在兩站檢查完成後自動統一修正、自動複驗至淨空、自動接連補蓋，全程不徵詢使用者。討論 quality-skill-pause-and-ui-polish（2026-08-07 結論）裁定改為每輪暫停制：使用者永遠握住階段轉換。相關正典：quality-skill spec「兩站時序的編排行為」；review-skill／verify-skill 的「quality 時序例外」（兩站「先不蓋章」離場出口）本變更原樣沿用。

## Goals / Non-Goals

**Goals:**

- 每輪兩站檢查／複驗完成後停下，彙整兩站 findings 待使用者裁示；乾淨輪也停
- 補蓋與封存建議改由使用者裁示觸發；收尾補蓋機制（明示收尾補蓋呼叫、兩章接連落、中間零編輯）原樣保留
- 正典三處（asset → claude／codex 兩實例）與 golden 同批落地，版號三連動一次到位

**Non-Goals:**

- 不動 review／verify 兩站的 asset 與 spec——兩站檢查、工單、蓋章語意與「quality 時序例外」全數不變
- 不新增引擎狀態、CLI 子指令或設定欄位
- 不動單站直接呼叫的行為（修完即蓋章預設維持）
- 桌面 UI 修整屬同討論轉出的另一變更

## Decisions

### D1 暫停制只落在 quality 正典 asset，不新增引擎狀態

編排暫停是技能文字層的行為（代理讀技能檔執行），落點唯一：crates/speclink-core/assets/skills/quality.md（speclink-core 內的純文字 asset，無 ANSI、無儲存媒介假設）。兩站既有的「先不蓋章」離場出口已足以承接每輪暫停，兩站 asset 零改動。替代方案「引擎新增暫停狀態或 CLI 關卡」否決：暫停是對話互動，引擎無從裁決何時該問人，屬過度設計。

### D2 暫停協定——每輪一停、選項固定、蓋章走使用者裁示

- 檢查輪（review 檢查 → verify 檢查，皆先不蓋章）完成 → 彙整兩站 findings 一次報告，以 AskUserQuestion 詢問（無此工具則純文字詢問並等待）：全修／挑選部分修正／不修就停。
- 每輪複驗完成 → 同樣停：報告殘餘必修與新發現，選項同上。
- 乾淨輪（兩站零 findings，或必修淨空且該輪零新修正）→ 停下報告兩站皆綠，選項：進收尾補蓋（補蓋後建議封存）／先不蓋、維持現狀離場。
- 「不修就停」＝兩站以既有「先不蓋章」出口離場，工單與凍結快照留存。必修未清時不提供補蓋選項——站內「必修淨空才蓋章」正典不被繞過，技能亦不代使用者決定蓋章。
- 裁量 findings 留票未修進收尾時，該站補蓋＝帶保留章（站內 `--accept`）——補蓋選項預先載明，以停下時的裁示為明示授權；技能不得未經明示逕自蓋保留章（審查修正輪補述：站內「Never run --accept unprompted」正典同樣不被繞過）。

替代方案「只在開修前停一次」與「乾淨輪自動補蓋」皆於討論由使用者否決（破壞「技能不自作主張」的單一心智模型）。

### D3 生成物兩處敘述與 golden 斷言同步釘住暫停語意

技能檔 frontmatter description 與 CLAUDE.md／AGENTS.md 技能清單條目**並非同源**，是兩處獨立文字，且都在括號裡帶著舊時序（「兩站檢查先不蓋章、統一修正、兩章接連蓋」），必須各自改寫：crates/speclink-core/src/skills.rs 的 quality 條目 description（渲染成兩 render target 的 frontmatter），以及 crates/speclink-core/src/init.rs 的兩處 quality 清單條目（`instructions_body` 的內建工具版、`custom_instructions_body` 的中性版）。正典 requirement「品質關卡技能的生成與正典化」只要求條目敘明觸發時機（兩站都跑時使用／單站直接呼叫該站），時序括號屬額外敘述——改寫只換括號內容，觸發時機兩半原樣留著。

crates/speclink-core/tests/it/render_golden.rs 既有 quality 斷言（workflow 行、兩站觸發、單站分岔）維持，另釘住三處暫停語意：技能檔內文的每輪暫停子句與「不修就停」選項字樣、frontmatter description、清單條目括號，防止未來改寫悄悄退回自動修。紅測先行、asset 改寫後轉綠。

### D4 MARKER_VERSION v1.18.0 與乾淨樹三連動

asset 內文變更依慣例 minor +1：v1.17.4 → v1.18.0（crates/speclink-core/src/init.rs）。（審查修正輪補述：收尾保留章補句再動 asset 內文，版號依 lock 守門再推 v1.18.0 → v1.18.1，三連動與 update 落地同批重跑。）順序固定不可換：版號提升 → 以 UPDATE_GOLDEN 環境旗標再生四份 snapshot → 以 UPDATE_ASSETS_LOCK 環境旗標再生 crates/speclink-core/tests/golden/assets.lock（lock 再生有守門：指紋變而版號未動會拒寫）。golden 必須在乾淨樹再生：再生前確認除本變更的 assets 編輯外無其他影響渲染輸入的改動，再生後審視 diff 僅含 quality 相關區段與版號欄位，並不帶環境旗標重跑綠燈。最後執行 speclink update 落地本 repo 生成物（.claude/skills/ 與 .agents/skills/ 全部技能檔的 frontmatter 版號、CLAUDE.md／AGENTS.md 的 SPECLINK 標記版號、speclink-quality 兩實例內文），與 asset 編輯同批提交——golden 測試只比對照檔、測不到安裝檔漏同步。

### D5 README 分工表時序句對齊

README.md 與 README.en.md 分工表「兩站都跑 → /speclink-quality」列的時序一句改為每輪暫停語意（兩站檢查先不蓋章 → 每輪停下待裁示 → 裁示後統一修正、兩站複驗 → 使用者裁示後兩章接連蓋），中英兩版語意對齊。

## Implementation Contract

- **行為**：對任務全數完成的 change 執行 /speclink-quality——兩站檢查完成後代理停下，彙整兩站 findings 詢問使用者（全修／挑選修／不修就停），未裁示前零編輯；每輪複驗完成同樣停；乾淨輪報告兩站皆綠並詢問是否收尾補蓋；裁示補蓋後以明示的收尾補蓋呼叫接連補蓋兩章、中間零編輯，封存以建議形式提出、由使用者執行。
- **介面／資料形狀**：無 CLI、無 --json、無設定欄位變更。對外形狀是生成物文字：speclink-quality 技能檔（claude／codex 兩 render target）、CLAUDE.md／AGENTS.md 技能清單條目、四份 golden snapshot、assets.lock 指紋、SPECLINK 標記版號 v1.18.0。
- **失敗模式**：兩站的拒絕或錯誤照站內正典原樣呈報並停流程（技能不吞錯、不繞站內守門）；必修未清時無補蓋選項可選。
- **驗收**：cargo test --workspace 全綠（含 render_golden 新增的暫停語意斷言）；golden diff 審閱僅含 quality 相關區段與版號欄位；人工核對 .claude/skills/speclink-quality/SKILL.md 與 .agents/skills/speclink-quality/SKILL.md 內文含每輪暫停時序、frontmatter 版號 v1.18.0，CLAUDE.md／AGENTS.md 標記版號同步。
- **範圍界線**：in——quality asset、skills.rs quality 條目 description、init.rs 的兩處 quality 清單條目與版號、render_golden 斷言、四 snapshot、assets.lock、README 兩檔、speclink update 落地生成物。out——review／verify 兩站 asset 與 spec、引擎狀態與 CLI、桌面前端。

## Risks / Trade-offs

- [dirty 樹再生 golden 把未提交狀態烙進 snapshot、之後 main 長期紅燈] → 再生前以 git status 確認僅本變更的 assets 編輯在樹上；再生後審視 diff；不帶環境旗標重跑確認綠燈
- [版號三連動缺一即紅燈（assets.lock 守門拒寫）] → 依 D4 固定順序執行，任務逐項列明順序
- [speclink update 漏跑留下舊版安裝檔——golden 全綠也測不到] → update 落地與 asset 編輯同批提交，任務含人工核對兩實例 frontmatter 版號
- [暫停文字重述兩站正典造成語意漂移] → 沿用「先不蓋章」「收尾補蓋呼叫」「必修」等既有正典詞，guardrail 維持「不重述兩站規則、站方為準」
- [跨平台] 純文字 asset 與測試，無平台相依行為；golden 測試沿用既有框架（換行正規化已處理），Windows／macOS／Linux 一致
