## Context

desktop 開本機資料夾走純探測分流（apps/desktop/core/src/project.rs 的 open_project_at）：向上探索命中即判 Project 直接進看板、完全未命中判 Uninitialized 跳初始化確認框、remote marker 走 handshake。core 的向上探索（Workspace::discover）以 `.speclink.yaml` 為首要判定鍵、`openspec/` 目錄與舊版 remote 標記檔為 fallback——因此「有 openspec/ 但無 .speclink.yaml」的資料夾（自其他體系遷移、或隊友未提交 .speclink.yaml）被判 Project 靜默放行，工作區檔從未安裝。引擎側素材：init 的 store_init 以 write_if 冪等寫骨架、reconcile_builtin_tools 寫 tools 進 .speclink.yaml 後呼叫 update() 整套再生受管檔；init 本體對「openspec/ 或 .speclink.yaml 已存在」直接擋下（Already initialized）。

來源討論 desktop-workspace-auto-init 已定案判準與流程；「.speclink.yaml 在但指令檔缺」的相鄰缺口由變更 desktop-instruction-staleness-prompt 的缺失態涵蓋，與本變更互斥分工。

## Goals / Non-Goals

**Goals:**

- 「已啟用 speclink」判準＝專案根 `.speclink.yaml` 存在；未啟用的資料夾在 desktop 開啟時得到啟用提示，確認後補齊工作區檔並切入專案
- 引擎提供冪等 adopt 入口：補骨架缺件、寫 tools、生成受管檔，既有規格內容零觸碰
- 既有分流（Project／Uninitialized／RemoteBinding、舊 remote 標記檔遷移警告）行為凍結

**Non-Goals:**

- 不改 core Workspace::discover 的 openspec/ fallback——CLI 對無 .speclink.yaml 的專案照舊運作（相容性依賴）
- 不做其他體系內容格式的實質轉換（onboard 語意）——adopt 只補工作區檔，openspec/ 內文件原樣
- CLI 不加未啟用提示或新子指令
- 不處理「.speclink.yaml 在但指令檔不存在」——desktop-instruction-staleness-prompt 的缺失態涵蓋
- 不以 openspec/config.yaml 作為啟用判準（討論已否決：語意是團隊工作流政策、可合法缺席、remote 模式無本地 openspec/）

## Decisions

### 決策 1：判準＝.speclink.yaml 存在與否，第四態只加在 desktop 探測層

open_project_at 於 Workspace::discover 命中、resolve 為本地檔案模式（StoreMode::Fs）、且 ws.root 無 `.speclink.yaml` 時，回報新變體 Unadopted { root }（序列化 status: "unadopted"）；有 `.speclink.yaml` 照舊 Project。判定錨在 discover 命中的 root（非使用者所選子目錄）——從子目錄開啟時 adopt 落在正確的專案根。remote 分流不經此判定：`.speclink.yaml` 帶 remote section 者 resolve 為 Remote → RemoteBinding 照舊；舊版 remote 標記檔（無 .speclink.yaml）resolve 的既有行為維持原樣，本變更不改其路徑——Unadopted 判定僅攔「resolve 成功且為 Fs 且無 .speclink.yaml」一種組合。
替代方案：改 Workspace::discover 使 bare openspec/ 不算 workspace——破壞 CLI 相容（無 .speclink.yaml 的舊專案 CLI 會變成 not in a project），否決；沿用 Uninitialized 態帶旗標——把「全新初始化」與「補齊啟用」兩種確認語意混進同一態，前端文案與寫入行為都要靠旗標分岔，不如獨立變體清晰，否決。

### 決策 2：引擎 adopt 入口＝store_init 冪等補件＋reconcile_builtin_tools 組合

speclink-core 的 init 模組新增 pub fn adopt（root、tools），語意：對「已有 openspec/ 但無 .speclink.yaml」的目錄補齊工作區——(1) 冪等補骨架：specs/、changes/archive/ 目錄 create_dir_all，config.yaml 僅在不存在時寫入範本（write_if 語意，既有檔含使用者政策時零觸碰）；(2) reconcile_builtin_tools 寫 tools 進 .speclink.yaml 並整套再生技能檔與指令檔受管區塊。spec_dir 固定 openspec——無 .speclink.yaml 時 discover 的 fallback 就是 openspec，不存在其他可能。openspec/ 既有文件（specs、changes、discussions 及其內容）零觸碰、零改寫。tools 空清單拒絕（沿用 reconcile 既有錯誤）。
替代方案：直接放寬 init 的 Already initialized 擋板（加 flag）——init 的 store_init 語意是「建骨架」而非「補缺件」，且擋板保護的是 CLI init 的誤觸語意，放寬影響面大於新入口，否決；desktop 自行組合多個引擎呼叫——寫入序列的原子性與錯誤語意散落 desktop 層，違反單一收斂入口慣例，否決。

### 決策 3：啟用確認框沿用初始化確認框同型

前端新增 pendingAdopt 狀態（與 pendingInit 平行），對 unadopted 態開啟與初始化確認框同型的對話框：AI 工具多選（claude／codex、預設勾選 claude）、確認呼叫 adopt command、取消零寫入維持原專案、失敗單行錯誤不切換、成功後以回報的 root 切入專案（與 confirmInit 成功後語意一致）。文案為啟用語意——標題與說明表達「這個資料夾已有規格資料，尚未啟用 speclink；啟用會安裝技能與指令檔」，主動作「啟用」；遵循 openspec/LANGUAGE.md（不出現 marker、frontmatter 等工程詞；`.speclink.yaml` 檔名不入文案）。zh-TW 與 en 兩語系鍵集合維持相等。
替代方案：復用同一個 pendingInit 對話框加條件文案——兩種確認的寫入行為不同（init 建骨架 vs adopt 補件不覆蓋），共用狀態易把 confirmInit 誤接到未啟用資料夾上（init 會 bail Already initialized），獨立狀態讓錯接直接編譯期可見，否決。

### 決策 4：IPC 為獨立 adopt command

src-tauri 新增 adopt_project command（單行委派 desktop-core 的 adopt 包裝，成功後重跑探測回報命中的 Project，與 init_project 的回報形狀一致）。不復用 init_project——其引擎路徑 init() 對已有 openspec/ 的目錄必然 bail，語意不同不共用。
替代方案：init_project 加參數分流——兩種語意共用一個 command 靠參數岔開，呼叫端錯傳參數變成執行期錯誤，否決。

## Implementation Contract

**行為**：

- 以 desktop 開啟「有 openspec/、無 .speclink.yaml」的資料夾 → 出現啟用確認對話框（工具多選、預設 claude）；確認 → 專案根產生 .speclink.yaml（tools 記錄所選）、所選工具的技能檔與指令檔受管區塊、openspec/ 骨架缺件補齊（既有 config.yaml 與所有既有文件原樣）→ 切入該專案看板；取消 → 維持原專案、目標資料夾零寫入。
- 開啟有 .speclink.yaml 的專案 → 無啟用對話框，照舊直接進看板。
- 開啟完全未初始化的資料夾 → 照舊出現初始化確認框（行為凍結）。
- 從未啟用專案的子目錄開啟 → 對話框針對向上命中的專案根，adopt 寫入該根。

**介面／資料形狀**：

- speclink-core：init 模組新增 pub fn adopt(root: &Path, tools: &[Tool]) -> Result<UpdateOutcome>；tools 空清單回錯誤。adopt 另呼叫既有的 ensure_gitignore 補 `.speclink/` 條目——reconcile_builtin_tools 走的 update 路徑不含這步（只有 init 的 workspace_init 有），不補則 desktop 快取檔在使用者的版控裡冒出未追蹤檔。
- desktop-core：ProjectProbe 新增 Unadopted { root: String } 變體（serde camelCase，status: "unadopted"）；新增 adopt 包裝函式（呼叫 core adopt 後重跑 open_project_at 回報 Project）。
- desktop IPC：新增 adopt_project(path, tools) command；前端 adapter 的 probe 型別聯集加 unadopted。
- 前端：store 新增 pendingAdopt: string | null 與 confirmAdopt(tools)／cancelAdopt；i18n 新增啟用對話框文案鍵（zh-TW／en）。

**失敗模式**：

- adopt 失敗（如目錄唯讀、.speclink.yaml 寫入失敗）→ 單行錯誤呈現、不切換專案（與 init 失敗語意一致）；已寫入的部分檔案不回滾——adopt 冪等，重試即收斂。
- 探測遇壞 .speclink.yaml → 照舊 fail-closed 單行 Err（不得誤判 Unadopted 或 Uninitialized）。

**驗收準則**：

- speclink-core 單元測試：adopt 對「有 openspec/ 無 .speclink.yaml」目錄補齊工作區檔且既有 config.yaml 與規格文件位元級不變；對既有檔重跑 adopt 冪等；tools 空清單錯誤；.gitignore 缺席時建立且涵蓋 `.speclink/`，既有 .gitignore 追加而非覆寫、已涵蓋時不重複追加。
- desktop-core 單元測試：open_project_at 對「openspec/ 在、.speclink.yaml 不在」回報 unadopted 且零寫入；「.speclink.yaml 在」仍回報 project；完全未命中仍 uninitialized；adopt 包裝成功後回報 project。
- 前端 vitest：unadopted → pendingAdopt 開啟對話框；confirmAdopt 成功切入專案；cancelAdopt 清狀態零呼叫；失敗浮單行錯誤不切換。
- 真實視窗 GUI 驗證（依 CLAUDE.md 備忘）：開啟遷移資料夾 → 啟用對話框 → 確認後看板呈現既有 changes/specs、技能檔已安裝；取消路徑零寫入。

**範圍邊界**：in scope＝引擎 adopt 入口、desktop 探測第四態、啟用確認框與文案、切入語意；out of scope＝core discover 行為、CLI 指令面、remote 分流（含舊 remote 標記檔遷移警告路徑）、其他體系內容格式轉換、「已啟用但指令檔缺失」（staleness change 涵蓋）。

## Risks / Trade-offs

- [誤把舊 remote 標記檔資料夾判成未啟用] → Unadopted 條件釘死「resolve 成功且 StoreMode::Fs 且無 .speclink.yaml」；desktop-core 測試含此邊界。
- [adopt 覆蓋使用者既有 config.yaml] → write_if 語意僅在檔案不存在時寫範本；單元測試以位元級比對釘死。
- [使用者誤開別人的 openspec 專案被寫入] → 確認框擋板（與 init 同慣例，曾靠它擋下誤選使用者目錄）；取消零寫入有測試釘死。
- [workspace-chooser「本機開啟行為凍結」場景鬆動] → 凍結對象是既有專案與未初始化資料夾兩路徑，均不變；未啟用是先前被誤併入 Project 的新分流，chooser 匯流本身不動。

## Migration Plan

單向前進：合入後未啟用資料夾首次開啟即見啟用對話框；既有已啟用專案（.speclink.yaml 在）行為零變化。回退＝revert commit，未啟用資料夾回到被判 Project 的舊行為，已 adopt 的專案不受影響（其工作區檔已是正常 speclink 專案）。

## Open Questions

（無——判準、入口語意、確認框形態均由來源討論定案。）
