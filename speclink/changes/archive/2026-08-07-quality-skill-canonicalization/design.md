## Context

兩個品質站（review／verify）的正典預設是「修完即蓋章」，且章會凍結範圍檔內容指紋——兩站都跑時，後蓋站的修正必然把先蓋的章打黃。討論 cross-station-staleness 定案的時序解（兩站檢查先不蓋章 → 統一修正 → 各自複驗 → 兩章接連蓋）目前只存在於本機非正典技能 .claude/skills/speclink-quality，不進 golden、不隨 speclink update 同步。使用者裁定「兩站都跑」已是預設流程，啟動該討論 Deferred 的重議條款：收進引擎正典。

現況錨點：技能 body 為 crates/speclink-core/assets/skills/ 下的 markdown asset，經 skills.rs 的技能表（Skill 結構：name／description／fork／disallow_edit／for_codex／body）以 include_str 註冊；CLAUDE.md／AGENTS.md 由 init.rs 範本生成，含 workflow 行與技能使用清單；生成物由 golden snapshot（crates/speclink-core/tests/golden/）對照保護；MARKER_VERSION（init.rs）控制既有專案 speclink update 是否重新生成。

相依：verify-skill 正典（含「驗證收尾迴圈」）由進行中 change verify-station-parity 新增——本 change 的 verify 側修改以其落地封存後的正典為基準。

## Goals / Non-Goals

**Goals:**

- quality 編排成為正典技能：speclink update 生成、golden 涵蓋、MARKER_VERSION 帶動既有專案更新
- 「兩章皆過才正式蓋」成為無例外的保證：堵掉兩站「零 findings 自動蓋章」在 quality 時序下的縫
- 規則落進生成文件（CLAUDE.md／AGENTS.md workflow 行與技能清單）與 README 兩站分工表

**Non-Goals:**

- 不新增引擎狀態層支援：無 quality 工單、無 quality 章、無新 CLI 動詞；工單、章、裁決全部留在兩站
- 不改兩站的檢查內容、裁決分類與收斂邏輯；不改單站流程語意（單站自行觸發、蓋章後被後續修正打黃屬正常警示）
- 不做看板／系統匣／GUI 的「品質關卡進行中」顯示（實務有感再另開討論）
- 無設定欄位（openspec/config.yaml／.speclink.yaml）變更；無 serde、git、YAML 手術、remote 路徑變更

## Decisions

### D1 quality 以正典技能 asset 承載，不新增引擎狀態

新增 crates/speclink-core/assets/skills/quality.md，內容以現行本機 SKILL.md 為底改寫為正典行文：定位（只管時序，兩站的檢查、工單、蓋章語意由各站技能承載——不得重述站內正典標準，與 config-station-canon-guard 紅線一致）、前提（change 任務全數完成；未滿足時依該站既有前提行為呈現——review 站拒絕、verify 站中途盤點）、延後蓋章時序（review 檢查先不蓋章 → verify 檢查先不蓋章 → 兩站 findings 統一修正（回主線、依專案 TDD 慣例）→ review 複驗至必修淨空、仍不蓋章 → verify 複驗至必修淨空、仍不蓋章 → 確認零新修正後兩章接連補蓋 → 封存；任一站複驗引出新修正即修正後重跑兩站複驗，迴圈至一輪零新修正才補蓋——蓋章延後到迴圈收斂之後，任何一章都不會被後續修正打黃）、邊界情況（事後變卦：已蓋一站才加跑另一站 → 照跑、接受前章暫態變黃、封存定格回綠、不重做；單站或都不跑：不經本技能）。skills.rs 技能表註冊 name「quality」、fork false（編排與統一修正都在主線）、disallow_edit false（統一修正需改檔）、for_codex true。捨棄方案：引擎狀態層支援（quality 無自有工單／章／裁決軸，造動詞與 GUI 成本不成比例——討論已否決）。

### D2 兩站 asset 補 quality 時序例外，堵零缺失自動蓋章縫

review.md 與 verify.md 各補一段例外：於 quality 時序中（由 /speclink-quality 依序呼叫時），discovery 零 findings 與 validation 後必修淨空皆 SHALL NOT 當場蓋章，改走既有「先不蓋章」離場；蓋章一律延至 quality 的收尾補蓋。收尾補蓋機制：蓋章的唯一閘門是編排方明示的「收尾補蓋呼叫」（closing stamp call）——quality 於收尾時明示，兩站據此關閉禁蓋例外；未被告知即視為非收尾呼叫，一律不蓋。review 側沿用「末輪零 findings 時重試蓋章」的既有捷徑，但 SHALL 先分兩層：外部守門失敗留下的乾淨末輪直接重試 stamp；quality 時序刻意留下者僅於收尾補蓋呼叫中蓋——該呼叫以 review scope 確認凍結點後內容無再移動（無移動直接 stamp、有移動同呼叫內先跑 validation 輪淨空再蓋），非收尾呼叫則無論有無移動都不蓋（空 patch 回報無新內容可判、不動工單）。堵住的是兩道縫：捷徑把未經驗證的移動直接蓋掉，以及非收尾複驗經捷徑提前蓋章。verify 側同構——step 4 入口即依收尾補蓋呼叫分流（末輪乾淨且無移動→跳過檢查直接 stamp 不落空輪；有移動→同呼叫內 validation 後即蓋），step 13 例外同樣只對非收尾呼叫生效。單站直接呼叫時行為不變（零 findings 仍自動蓋章）。對應 spec delta：review-skill「審查後的迴圈與收尾」、verify-skill「驗證收尾迴圈」各 MODIFIED。捨棄方案：維持 Deferred 不堵（quality 升正典後保證帶星號，官方文件得寫「乾淨 discovery 例外章可能暫黃」，不可接受）；由 quality asset 外層攔（攔不住站內 SHALL，且屬重述他站正典）；站內蓋章制（review 複驗淨空即蓋、verify 複驗再出必修時前章必被打黃——SHALL 保證破口，quality 品質輪定案改為延後蓋章制）。

### D3 init.rs 範本的 workflow 行與技能清單條目

workflow 行由 `discuss? → propose → apply ⇄ ingest → (review? ∥ verify?) → archive` 改為 `discuss? → propose → apply ⇄ ingest → (quality? | review? ∥ verify?) → archive`；worktree 流程行的品質站段同步改為 `(quality? | review? ∥ verify?)`（quality 為 worktree_gated false，worktree 專案同樣生成該技能，流程行不同步即指引缺口；對應 workspace-tools「marker 技能指引跟隨 worktree 政策」MODIFIED）；技能使用清單加入 quality 條目（觸發時機：事前已知 review 與 verify 兩站都要跑時走 /speclink-quality；只跑一站直接呼叫該站技能）。claude、codex 與 custom neutral 三個 render target 同步（neutral 同樣生成 speclink-quality 技能檔，marker 不提及即指引不可達）。對應 spec delta：review-skill「審查技能的生成與正典化」所釘 workflow 行文字 MODIFIED 為新版本；quality-skill 生成 requirement 只斷言「行含 quality 入口」不重述整行，避免兩規格互相釘死同一字面。

### D4 相依 verify-station-parity，verify 側以其封存後正典為基準

本 change 排在 verify-station-parity 封存之後才可開工：verify 站的工單與章、verify-skill 正典（含「驗證收尾迴圈」與 verify.md asset）皆由該 change 落地，是 D2 verify 側修改的存在前提。本 change 的 verify-skill delta 以該 change 併入正典後的條文為基準撰寫；開工前若間隔久，先跑 drift 確認基準未漂移。捨棄方案：併入 verify-station-parity（其 19 任務再膨脹、關注點混雜）；先行開工只做 review 側（兩站不對稱落地，quality 保證不成立）。

### D5 MARKER_VERSION 提升與乾淨樹 golden 再生

技能 asset 與 init 範本內容變更後，提升 init.rs 的 MARKER_VERSION，使既有專案的 speclink update 重新生成技能檔與 CLAUDE.md／AGENTS.md。golden（assets.lock 與各 render target snapshot）於乾淨樹再生，與程式變更同批落地；本 repo 執行 speclink update 使 .claude/skills/ 下的 speclink-quality（由生成物取代手寫版）、speclink-review、speclink-verify 與根 CLAUDE.md／AGENTS.md 刷新。

### D6 README 兩站分工表補 quality 入口

README.md 與 README.en.md 的兩站分工表（verify-station-parity 任務落地的版本）補「兩站都跑 → /speclink-quality」列與一句時序說明（兩站檢查先不蓋章 → 統一修正 → 各自複驗 → 兩章接連蓋），並把原「兩站都跑時的蓋章時序慣例」句改指向技能入口——慣例升格為技能後，README 不再以慣例行文承載時序細節。SDD workflow 流程圖（中英兩處）的主流程行同步補 quality 入口（`(quality? | review? ∥ verify?)`），與生成之 CLAUDE.md／AGENTS.md workflow 行對齊——流程圖是讀者第一眼入口，不同步即觀感缺口。

## Implementation Contract

**行為**：
- 使用者對任務全數完成的 change 說「兩站都跑」或執行 /speclink-quality：代理依序跑 review 檢查（選「先不蓋章」離場）→ verify 檢查（同）→ 兩站 findings 合併 triage、統一修正於主線 → 重跑 review 複驗（validation 涵蓋上輪凍結點以來全部修正）至必修淨空、仍不蓋章 → 重跑 verify 複驗至必修淨空、仍不蓋章 → 一輪完整複驗零新修正後，review 章與 verify 章接連補蓋（補蓋前內容又移動則先跑 validation 淨空再蓋）→ 建議封存。全程沒有任何一章會被後續修正打黃，兩章到封存皆綠。
- quality 時序中任一站 discovery 零 findings 或 validation 淨空：該站皆走「先不蓋章」離場，不當場蓋章；蓋章一律延至收尾補蓋。單站直接呼叫：零 findings 仍當場自動蓋章，行為不變。
- 事後變卦（已蓋一站才加跑另一站）：不經 quality 重做，照跑新站、接受前章暫態變黃，封存定格回綠。
- speclink update 於已啟用專案：生成 speclink-quality 技能檔（claude／codex 依 tools 設定）、CLAUDE.md／AGENTS.md workflow 行含 `(quality? | review? ∥ verify?)`、技能清單含 quality 條目。

**介面／資料形狀**：無新 CLI 子指令、旗標、stdin、exit code；無 `--json` 欄位變更。技能檔 frontmatter 沿用既有生成格式（name／description／metadata.version＝MARKER_VERSION）。

**失敗模式**：quality 前提不滿足（任務未全完成）時，依該站既有前提行為呈現——review 站拒絕並停止；verify 站以中途盤點報告收場（其工單引擎守門本就拒絕未完成 change 落工單）——quality asset 不另設守門、不吞錯。

**驗收**：cargo test -p speclink-core --test it 全綠（render_golden 與 skill_verbization 涵蓋新技能註冊與生成內容）；乾淨樹 golden 再生後 cargo test --workspace 全綠；本 repo speclink update 後人工核對 CLAUDE.md workflow 行、技能清單、三個技能檔內容與 README 分工表。

**範圍邊界**：in scope＝speclink-core 的技能 asset／技能表／init 範本（主流程行與 worktree 流程行）／golden、本 repo 生成物落地、README 兩處分工表與 SDD workflow 流程圖。out of scope＝兩站檢查與裁決邏輯、引擎狀態與 GUI、設定欄位、remote 路徑（技能生成純屬本地 checkout 檔案，無 local／remote 雙路徑契約）、docs/ 文件集的 quality 補寫與既存殘留（另案處理）。

## Risks / Trade-offs

- [golden 與 CLI 回歸] 技能與範本變更牽動多份 snapshot 與 assets.lock，漏更新即紅測 → 乾淨樹一次性再生 golden、與程式變更同批提交；render_golden 與 skill_verbization 先行紅測驗證覆蓋
- [跨平台] 技能 asset 與範本為純文字 markdown，無路徑、換行以 LF 為準（golden 對照既有規範）→ Windows／macOS／Linux 無平台分支，沿用既有生成管線
- [相依漂移] verify-station-parity 封存前其 verify-skill delta 仍可能修訂，本 change 的 verify 側 delta 基準隨之漂移 → 開工前跑 drift；若條文名或收尾迴圈語意變動，先 ingest 校正 delta 再 apply
- [兩規格釘同一字面] workflow 行文字同時被 review-skill 與 quality-skill 斷言會互相釘死 → D3 已定：review-skill 持有整行字面，quality-skill 只斷言含 quality 入口
- [取代手寫技能檔] 本 repo speclink update 會覆寫 .claude/skills/speclink-quality/SKILL.md 手寫版 → 屬刻意取代；正典 asset 撰寫時先吸收手寫版全部語意（D1 內容清單），避免語意遺失

## Migration Plan

1. verify-station-parity 封存後開工；先紅測（golden／skill_verbization），再落 asset 與範本變更，MARKER_VERSION 提升
2. 乾淨樹再生 golden、cargo test --workspace 全綠後，本 repo 執行 speclink update 落地生成物，README 分工表同批更新
3. 回滾：revert 該批 commit 並以同版程式重生 golden 與生成物即可，無資料遷移、無設定相容問題

## Open Questions

（無——形狀、例外、相依與文件落點皆由討論 quality-skill-canonicalization 定案）
