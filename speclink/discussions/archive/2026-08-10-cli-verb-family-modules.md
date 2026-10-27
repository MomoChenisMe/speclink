---
topic: CLI include! 文字包含改真模組——按動詞族重切檔案
slug: cli-verb-family-modules
status: promoted
promoted_to: cli-verb-family-modules
created: 2026-08-10
created_by: MomoChen <momochenisme@gmail.com>
kind: improve
---

# Discussion: CLI include! 文字包含改真模組——按動詞族重切檔案

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

前因：improve-cli-command-layer（2026-08-07，已轉出封存）盤出四個候選；候選 1（cli-render-unification）已落地封存、候選 2（cli-mode-dispatch-convergence）已完成待封存，使用者指示接續候選 3「include! 文字包含改真模組，按動詞族重切檔案」。

偵察（候選 1、2 落地後的前提複核）：include! 仍在（main.rs:916–917），三檔合計 5,603 行單一編譯單元（main.rs 917／commands.rs 3,217／remote_commands.rs 1,469），零模組邊界。include! 源自創始 commit（5d7fa5c），屬初期便利、非有記錄理由的決策——無重提衝突。近三月動到 CLI 三檔的 commit 檔次共 140，共動摩擦持續。候選 1、2 落地後的形狀是重切的有利起點：dispatch 已是 31 動詞的模式形狀表（commands.rs:6–75，ModeFree／Dual／FsOnly／RemoteOnly），每動詞渲染已收斂單份（吃 core outcome 型別），wire→core 轉接（to_*）集中於 remote_commands.rs。跨檔散佈實例：ModeFree 的 link／unlink／auth 實作住在 remote_commands.rs:370–692（與檔名語意脫鉤）；clap 參數定義全在 main.rs、實作在另兩檔。commands.rs 內嵌兩個單元測試模組（patch_hash_chain_tests、init_tools_tests）以 use super::* 依賴所在檔，改真模組後隨族檔搬移即可。

相關 change：cli-mode-dispatch-convergence（候選 2）done 未封存、程式碼已 commit、工作樹乾淨——無程式碼撞車，封存先後不構成前置。候選 4（wire→core 轉接收進 speclink-remote）仍留在原討論待後續，與本候選相容：族檔先收攏該族 to_*，候選 4 若做再整批遷出。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — interview (2026-08-10)

**Focus**: 族切表——31 個動詞如何分檔
**Position**: 使用者照提案裁定：13 個族檔（src/verbs/）＋底座。族檔原則：一族一檔裝齊該族完整故事——clap 參數定義（自 main.rs 遷入）、fs 臂、remote 臂、渲染、wire→core 轉接、內嵌測試。族表：init（init、update）、connection（link、unlink、auth——現住 remote_commands.rs:370–692 的那批歸位）、query（list、show、status）、checks（validate、analyze、drift）、lifecycle（archive、discard、claim）、progress（task、in-progress）、new、instructions、documents（artifact、language）、station（review、verify）、discuss、config（config、workflow-config）、toolchain（schemas、templates、schema、completion、feedback、demo）。底座：main.rs 瘦身後保留 Cli／Commands enum、dispatch 模式表、dual／fs_only／remote_only 組合子、main()；remote_base.rs 收 RemoteCtx、remote_ctx() 握手、remote_resolve_change；common.rs 收 run_command、print_json、read_stdin*、require_workspace、open_project、info_if_no_changes、warn_*；color.rs 不動。規模檢核：最大族 station 約 600 行，遠小於今日 commands.rs 的 3,217。
**Ruled out**: 一動詞一檔（31 檔）——每檔僅百餘行，複製「理解一個概念要跳好幾個小檔」的反樣式，是訊號 1 的另一極端；粗切 7–8 檔——族語意被稀釋，單檔仍大，共動摩擦只減半不消解。
**Open**: 族檔間依賴紀律與 pub 可見性；純搬移的驗證方式（行為零變更如何守）

### Round 2 — interview (2026-08-10)

**Focus**: 族檔間依賴紀律與 pub 可見性
**Position**: 使用者照提案裁定硬規則：族檔之間禁止互相 import，跨族共用件一律升底座；可見性最小 pub 面——族檔僅對外開放 dispatch 表要呼叫的臂函式（cmd_*／remote_*）與 Commands enum 要引用的 clap 參數型別，渲染／wire→core 轉接／helper／內嵌測試全私有。證據：規則今天零成本——盤點全部共用 helper，真正跨族共用僅五支通用管線件（run_command、print_json、read_stdin_content、info_if_no_changes、remote_resolve_change），本就歸底座；其餘 helper 全為單族私有（unified_diff／scalar_str／require_stdin_flag 僅 config 用、render_specs_section 僅 list 用、artifact_rel_path 僅 documents 用、patch_hash_chain 僅 station 用）。機制意義：include! 時代任何東西看得到任何東西、越界零阻力零痕跡；立規則後越界必須表態——升底座或露出新 pub——review 一眼可見，這是「模組邊界讓越界依賴看得見」這個 win 的具體機制。
**Ruled out**: 不立硬規則（僅 mod 化搬檔、族檔間允許互相引用）——邊界靠習慣維持，越界依舊無痕跡，重切的結構收益折半
**Open**: 無——介面深度四項檢查與純搬移驗證方式於結論總整理

## Conclusion

**Decision**: CLI 的 include! 文字包含改為真模組，按動詞族重切檔案。13 個族檔（src/verbs/：init、connection、query、checks、lifecycle、progress、new、instructions、documents、station、discuss、config、toolchain），每族檔裝齊該族完整故事——clap 參數定義、fs 臂、remote 臂、渲染、wire→core 轉接、內嵌測試；底座留 src/（main.rs 瘦身保留 Cli／Commands enum、dispatch 模式表、模式組合子、main()；remote_base.rs 收 RemoteCtx／remote_ctx()／remote_resolve_change；common.rs 收 run_command／print_json／read_stdin*／require_workspace／open_project／info_if_no_changes／warn_*；color.rs 不動）。依賴紀律為硬規則：族檔互不 import、跨族共用一律升底座、最小 pub 面（僅臂函式與 clap 型別）。純搬移、行為零變更——凍結輸出整合測試逐位元不動即是守門，搬移後零 dead-code warning 為輔助斷言。
**Rationale**: 三檔 5,603 行單一編譯單元、零模組邊界，同一動詞的完整故事散在三個檔（參數在 main.rs、fs 臂與渲染在 commands.rs、remote 臂與轉接在 remote_commands.rs），近三月 140 檔次共動為實證。候選 1（渲染單份化）、候選 2（dispatch 模式表）落地後，「本機檔/remote 檔」的分層切法已無結構意義——重切為動詞族是把散在三處的同一概念集中，改一個動詞開一族檔。介面深度四項檢查：(1) 接縫位置——模組邊界定在動詞族，通用管線件沉底座，與 dispatch 的模式表宣告層正交；(2) 轉接數量——零新增執行期轉接，純編譯期邊界，不疊任何 wrapper；(3) 深度——每族檔以二、三個 pub 入口藏住渲染／轉接／helper 的全部實作，pub 面遠小於檔內行為；(4) 刪除測試——刪掉重切回到三檔分層切，每動詞改動重新橫跨 2–3 檔；重切是集中複雜度（同族一處可整體理解）而非搬移。依賴紀律今天零成本（跨族共用僅五支通用管線件，其餘 helper 全單族私有），立規則只是把現狀升級成守得住的邊界。
**Rejected alternatives**: 一動詞一檔（31 檔，每檔百餘行——複製「跳好幾個小檔」反樣式）；粗切 7–8 檔（族語意稀釋，共動摩擦只減半）；不立依賴硬規則、僅 mod 化搬檔（越界依舊無痕跡，結構收益折半）。
**Deferred**: 候選 4（wire→core 轉接收進 speclink-remote 供 CLI 與 desktop 共用）——維持原討論的 speculative 定位，與本刀相容：族檔先收攏該族 to_*，屆時再整批遷出；git blame 跨檔追溯成本（一檔拆多檔後 --follow 變弱）——接受，共動證據顯示未來變更以族為單位，新血緣自然累積。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion cli-verb-family-modules
