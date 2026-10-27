## Context

crates/speclink-cli 的三個原始檔以 include! 文字包含拼成單一編譯單元（main.rs 尾端兩行 include!），合計 5,603 行、零模組邊界。切分方向是「本機檔／remote 檔」分層，但實際變更以動詞為單位：同一動詞的 clap 參數在 main.rs、fs 臂與渲染在 commands.rs、remote 臂與 wire→core 轉接在 remote_commands.rs，近三月 140 檔次共動。include! 源自創始 commit，屬初期便利、非有記錄理由的決策。

前兩刀已鋪平本刀的前提：cli-render-unification 使每動詞只有一份渲染函式（吃 core outcome 型別）；cli-mode-dispatch-convergence 使模式分岔集中於 dispatch 的模式形狀表（ModeFree／Dual／FsOnly／RemoteOnly，dual／fs_only／remote_only 組合子）。本刀是純搬移收官：不改任何形狀，只把既有形狀落進真模組。

來源討論：openspec/discussions/cli-verb-family-modules.md（兩輪裁定：族切表、依賴紀律）。

## Goals / Non-Goals

**Goals:**

- 拆除 include!，crates/speclink-cli 全面改真模組。
- 按動詞族重切：13 個族檔（src/verbs/），每族檔裝齊該族完整故事（clap 參數、fs 臂、remote 臂、渲染、wire→core 轉接、內嵌測試）。
- 依賴紀律成為編譯期可守的硬規則：族檔互不 import、跨族共用升底座、最小 pub(crate) 面。
- 行為零變更：輸出、exit code、錯誤訊息逐位元不變。

**Non-Goals:**

- 不改任何執行期行為、輸出格式、錯誤文字（凍結輸出測試逐位元守門）。
- 不做候選 4（wire→core 轉接收進 speclink-remote 供 desktop 共用）——族檔先收攏該族 to_* 轉接，屆時再整批遷出。
- 不動 dispatch 模式表的語意與動詞分類（cli-mode-dispatch-convergence 已裁定並落 spec）。
- 不動 color.rs；凍結輸出整合測試內容不動（僅 D6 明列的結構守門測試 no_raw_wire_json.rs 與 init_tools.rs 一處註解隨結構同步更新）。
- 不保 git blame 跨檔血緣（一檔拆多檔後 --follow 變弱）——已於討論裁定接受，未來變更以族為單位累積新血緣。
- 已否決的切法不採：一動詞一檔（31 檔，回到跳小檔反樣式）；粗切 7-8 檔（族語意稀釋）；僅 mod 化搬檔不立依賴硬規則（越界無痕跡，結構收益折半）。

## Decisions

**D1 族切表（13 族，動詞歸屬）**：init（init、update）、connection（link、unlink、auth）、query（list、show、status）、checks（validate、analyze、drift）、lifecycle（archive、discard、claim）、progress（task、in-progress）、new（new change／new artifact）、instructions（instructions 與 --skill 分流）、documents（artifact、language）、station（review、verify）、discuss（discuss 全家）、config（config、workflow-config）、toolchain（schemas、templates、schema、completion、feedback、demo）。取捨：13 族使最大族（station，約 600 行）遠小於今日 commands.rs 的 3,217 行，同時避免 31 個百行小檔。

**D2 族檔內容物（symbol 級歸屬）**：每族檔含該族的 clap Args／Subcommand 型別（自 main.rs 遷入）、cmd_* fs 臂、remote_* remote 臂、render_*／print_* 渲染、to_* wire→core 轉接、族內私有 helper 與內嵌測試模組。指標性歸屬：connection 收現住 remote_commands.rs 的 cmd_link／cmd_unlink／cmd_auth 全段（含 login_with_pat、login_with_device、CliDeviceIo、cmd_auth_logout、print_identity；原同段的 ensure_repo_registered、validate_or_defer、git_reference_warning 三支因 init 的 cmd_init_remote 也使用，依 D4 升入 remote_base.rs——落地時的跨族實證）；init 收 cmd_init、resolve_init_tools、prompt_for_tools、ask_yes_no、cmd_init_remote 與 init_tools_tests；station 收 station_dual、station_fs、render_station_show、patch_hash_chain 與 patch_hash_chain_tests、remote_station、to_station_ticket、to_station_round；query 收 render_list（含 invalid 標記渲染）、render_specs_section、to_list_change_json、render_show、remote_show_outcome、to_status_report；config 收 WorkflowConfig 系列型別、unified_diff、hunk_range、scalar_str、require_stdin_flag、remote_workflow_config 與 REMOTE_CONFIG_LABEL，workflow-config 的 argv／stdin 正規化與雙臂宣告收為家族雙臂函式 cmd_workflow_config（與 station 的 station_dual 同型，dispatch 表以「Dual（宣告於 …）」註記——分岔決策仍由該函式尾端的 dual 單點表達）；toolchain 收 completion_shell、bash_inject_positionals、cmd_demo。單族私有 helper 留族檔不升底座（unified_diff 僅 config 用、render_specs_section 僅 list 用、artifact_rel_path 僅 documents 用、patch_hash_chain 僅 station 用——盤點證據見來源討論第二輪）。

**D3 底座三模組**：main.rs 瘦身後保留 Cli／Commands enum、dispatch 模式表、dual／fs_only／remote_only 模式組合子、main()、mod 宣告；remote_base.rs 收 RemoteCtx、remote_ctx() 握手、remote_resolve_change，及 repo 歸屬驗證三件套 ensure_repo_registered／validate_or_defer／git_reference_warning（connection 與 init 兩族共用）；common.rs 收 run_command、print_json、read_stdin、read_stdin_content、require_workspace、open_project、info_if_no_changes、warn_deprecated_policy_keys、warn_leftover_remote_file。收錄準則：確有兩族以上使用者才進底座（跨族共用今日僅此五類通用管線件，其餘 helper 全單族私有）。dispatch 表留 main.rs 的理由：它是全 CLI 的總覽地圖，與 Commands enum 同檔，「加一個動詞」的宣告入口只有一個檔。

**D4 依賴紀律與可見性（硬規則）**：族檔之間 SHALL NOT 互相 import；跨族要共用的符號 SHALL 升底座。可見性最小 pub(crate) 面：族檔僅對外開放 dispatch 表要呼叫的臂函式（cmd_*／remote_*／家族雙臂函式）與 Commands enum 要引用的 clap 參數型別；渲染、轉接、helper、測試模組一律模組私有。機制：include! 時代越界零阻力零痕跡；立規則後越界必須表態（升底座或露出新 pub），review 一眼可見。

**D5 模組宣告慣例**：src/verbs/ 目錄用 mod.rs 宣告 13 個族模組（對齊 repo 既有慣例：crates/speclink-core/src/command/mod.rs 等目錄模組均用 mod.rs）。main.rs 宣告 mod verbs、mod remote_base、mod common、mod color。

**D6 純搬移的驗證方式**：不新增任何測試（純搬移無新行為可測）。守門三重：(1) 凍結輸出整合測試逐位元不動——crates/speclink-cli/tests/it 全綠且凍結輸出測試檔零修改即證明輸出未變；(2) 搬移後 cargo build 零 warning——dead-code warning 會揭露漏搬或多搬；(3) grep 確認 include! 於 crates/speclink-cli/src 零殘留。內嵌測試模組隨族檔搬移（use super::* 語意在真模組下指向所在族檔，符號集不變即編譯即證）。例外兩支非凍結輸出測試隨結構同步更新：no_raw_wire_json.rs 的結構守門原本整檔掃描 remote_commands.rs，該檔解散後重設計為 fail-closed——remote_base.rs 與 src/verbs/ 全部檔案整檔掃描，fs 側合法的 serde_json::Value 輸出組裝以（檔名、精確行）允許清單明列，未列名即紅、清單過期亦紅；init_tools.rs 一處註解的檔名指向同步改為 verbs/init.rs。

## Implementation Contract

- 行為面：speclink CLI 全部動詞的輸出、exit code、錯誤訊息與 --json 形狀逐位元不變；使用者與呼叫方觀察不到任何差異。
- 結構面：crates/speclink-cli/src 之下為 main.rs（clap 頂層＋dispatch＋模式組合子）、common.rs、remote_base.rs、color.rs、verbs/mod.rs 與 13 個族檔；commands.rs 與 remote_commands.rs 不復存在；include! 巨集於本 crate 零使用。
- 可見性面：verbs 各族模組的 pub(crate) 符號限於臂函式與 clap 參數型別；任何族檔 SHALL NOT use 另一族檔的符號（以 use crate::verbs:: 交叉引用為零驗證）。
- 驗證目標：cargo test -p speclink-cli --test it 全綠；凍結輸出測試檔零修改（僅 D6 明列的兩支結構守門／註解測試檔隨結構同步更新）；cargo build -p speclink-cli 零 warning。（原列 cargo fmt --check 一項移除：repo 無 rustfmt.toml、CI 無 fmt gate，且搬移主體以逐字保留為先，不因搬家重排既有程式碼。）
