## Context

目前文件同時承擔產品介紹、上手教學、完整工作流、能力狀態與目標架構，卻沒有明確的責任分界。中英文 README 的狀態摘要均落後目前 Server／Store／Host 實作，getting-started 宣稱可呼叫尚未生成的 verify skill、把 design 寫成固定產物，且沒有說明 discuss promote、link→ingest→seal、續作前 drift 等分流。同時，README 首段既有品牌圖片、SDD Engine 標語、Local Repo／Remote Store 雙路徑與 Spectra App 2.3.1 相容性起源，是回答「專案在做什麼、為什麼存在」的必要入口，不應因文件分層而被移除。另一方面，skills 與正典 specs 對流程細節寫得很完整，但它們分別是 Agent 作業手冊與工程契約，不是面向使用者的漸進式教學。

利害關係人包含首次導入 Local Repo 的開發者、以討論與提案參與的 PO／PM、負責實作與續作的 RD、部署 Server／Desktop 的操作者，以及維護 skills、CLI、正典 specs 與架構文件的貢獻者。文件必須同時滿足「數分鐘完成第一輪」與「遇到非 happy path 時查得到完整規則」，且不得讓目標架構看起來像已交付能力。

本 change 是純文件與專案 context 整理，不修改 runtime、build、CLI、資料格式或生成 skills。`speclink-core`、`speclink-cli`、Desktop、Server、Node SDK 只作為查核來源；storage 解耦引擎與 Local／Remote 邊界仍以平台架構藍圖為準。

## Goals / Non-Goals

**Goals:**

- 以漸進揭露建立清楚的文件入口：README 導覽、getting-started happy path、workflow 完整規則、product-status 現況矩陣、architecture／roadmap 目標與交付順序。
- 在 README 首段保留 Speclink 的品牌圖片、產品目的、起源、Local Repo／Remote Store 雙路徑及 Spectra CLI 相容基線，只微調過時事實與連結。
- 為每個 workflow 階段與分支提供用途、使用時機、輸入、產物、Agent／CLI 呼叫、完成判準、下一步與恢復方式。
- 用可追溯證據區分可用、部分可用、規劃中與已棄用，並讓狀態文件具查核日期。
- 保持繁體中文與英文使用者文件的章節與事實對等，繁體中文遵循 `openspec/LANGUAGE.md` 正典詞彙。
- 修正已知的失效連結、錯誤命令、過時元件清單與 Agent Host 呼叫語法。

**Non-Goals:**

- 不修改任何 CLI／skills／runtime 行為，也不因文件稽核發現功能缺口而順手實作功能。
- 不改平台架構藍圖的目標決策與 Phase 順序，不讓 workflow 或 product-status 成為第二份架構 Roadmap。
- 不把 README 改成僅含狀態摘要與文件導覽的入口，也不以避免內容重複為由刪除專案定位與起源敘事。
- 不重寫歷史 change／discussion，不把歷史 artifact 當目前操作文件。
- 不在本 change 完整重建進階 Protocol／verb contract 文件；缺失須被誠實列出並另開後續工作。

## Decisions

### D1 — 以漸進揭露分配文件責任

- `README.md`／`README.en.md` 是產品入口，分成兩層。第一層保留並鏡像品牌圖片、標語、語言切換、Rust SDD 引擎定位、PM／PO／RD／AI Agent 共用 change／artifact／task／verify／archive 語意、Local Repo／Remote Store 雙路徑，以及以 Spectra App 2.3.1 CLI 為行為參考與 parity／golden tests 保護相容性的起源。第二層提供經 product-status 校正的目前狀態摘要、最短流程心智模型、Local Repo 開始入口與文件地圖；詳細流程、完整狀態矩陣及架構內容改以連結承接。
- `docs/getting-started.zh-TW.md`／`docs/getting-started.md` 是 Local Repo 第一輪 happy path：命令可直接複製，僅使用目前存在且已核對的 CLI／skills；複雜分流連到 workflow。
- `docs/workflow.zh-TW.md`／`docs/workflow.md` 是使用者工作流正典：完整解釋各階段、可選步驟、分流、狀態、產物與失敗恢復。
- `docs/product-status.zh-TW.md`／`docs/product-status.md` 是能力現況正典：記錄目前可用程度、證據、限制、查核日期與目標連結。
- `docs/platform-architecture.zh-TW.md` 維持唯一目標架構正典；`docs/implementation-refactor-roadmap.zh-TW.md` 維持其下的執行伴隨文件。

替代方案一是把所有內容擴寫進 README 或 getting-started；排除原因是文件會過長、現況與目標重新混在一起，且每次能力變動需要在多處同步同一張大表。替代方案二是把 README 改成純導覽頁；排除原因是會遺失專案身分、產品起源與 Local／Remote 雙路徑的第一層心智模型，使首次讀者必須先追連結才知道 Speclink 是什麼。

### D2 — 能力狀態使用證據分級而非檔案存在推論

狀態矩陣固定使用四類：

- 可用（Available）：目前存在使用者入口，且可由 CLI help、生成 skill、可執行 binary／UI 路徑、測試或現行操作文件至少兩種互相獨立證據確認。
- 部分可用（Partial）：已有可操作子集，但完整工作流、平台、封裝或限制尚未閉合；文件必須明列可用邊界與缺少部分。
- 規劃中（Planned）：只有目標架構、roadmap、未落地 change／spec 意圖，或缺少可操作入口；不得使用「目前支援」措辭。
- 已棄用（Deprecated）：仍保留相容路徑，但已明確不是目標架構；文件必須給替代方向。

每列必須附「證據」「限制／下一步」，並在文件頂部記錄最後查核日期。crate 目錄存在、canonical spec 存在或架構文件描述，單獨都不足以標成可用；以 CLI／skills／實際入口為準，specs 與 tests 用來佐證行為邊界。

替代方案是沿用 README 的「現況／目標」自由文字；排除原因是同一列容易把已存在的底層元件誤寫成完整產品交付，也無法表達 partial 與 deprecated。

### D3 — 完整工作流以主幹加決策分支呈現

workflow 先給主幹 `onboard? → discuss? → propose → apply ⇄ ingest → validate/analyze/audit → archive`，再以決策表拆解：

- 既有程式首次導入：onboard。
- 只求理解且無決策：直接問答，不開 discussion。
- 需求需收斂：discuss；結論後可直接 propose、快速轉為變更後續補 propose、link 到既有 change 再 ingest／seal、或不做而封存討論。
- 需求明確的新工作：propose。
- change 閒置後續作：先 drift；規格／需求改變：ingest；純實作：apply。
- analyze／validate 是 artifact 品質與結構檢查；audit 是安全檢查；commit 是特定 change 的 Git 工具，不誤列成必經生命週期階段。
- verify 只按目前可觀察入口與 evidence 實作描述；不存在的 Agent skill 不得列為可直接呼叫。

每個階段使用同一欄位模板：目的、何時使用、何時跳過、輸入、產物、呼叫層級、完成判準、下一步、恢復方式。Agent 呼叫範例同時列 Claude slash command、Codex `$skill` 與直接 CLI，且清楚說明 skill 是工作流知識、CLI／Host 才是執行引擎。

替代方案是依指令字母序建立 CLI reference；排除原因是新使用者以意圖與生命週期選路，不是先知道指令名稱。

### D4 — 中英文鏡像與正典詞彙共同維護

中英文成對文件使用相同 H2 章節次序、表格列與範例語意；程式識別符與 CLI 指令保持原文。繁體中文散文使用「轉為變更」「已轉出變更」「封存」「輪」「背景」等正典詞，不使用 `promote`、促轉、歸檔等避免詞，除非在 code span 中解釋引擎動詞。

README 現有 English 入口與 `README.en.md` 均予保留；兩版 README 的品牌圖片、標語、產品定位、雙部署路徑、Spectra 相容基線與目前狀態摘要保持概念對等。只有繁體中文版本的架構／Roadmap／Server 營運文件，在英文入口明確標示語言，而不是假造內容對等。

替代方案是先只維護繁體中文、之後再校正英文；排除原因是現有兩版 README 均承載產品事實，延後會讓英文入口繼續保留過時狀態。

### D5 — 將文件準確性驗證收斂成可重複清單

apply 必須以目前 checkout 重新取得 `speclink --help`、各相關子指令 `--help`、`.agents/skills/` 清單、Cargo workspace members、文件連結與正典詞彙，據此更新狀態；不得直接照 proposal 中的 2026-07-17 快照抄寫。驗證至少涵蓋：

- 所有相對 Markdown 連結目標存在，已明示為缺口的路徑不得做成可點連結。
- 中英文成對文件 H2 集合與順序一致；狀態矩陣列集合一致。
- getting-started 出現的 skill 名存在於相應 Host 生成 surface；CLI 指令與旗標可由目前 help 觀察。
- workflow 不含未註明狀態的規劃中入口，product-status 每列有證據與限制／下一步。
- 繁體中文文件不出現正典詞彙的 avoid 用法，歷史引用與 code span 除外。

替代方案是引入新的文件網站框架或外部 link checker；排除原因是本 change 不需要新依賴，現有 Markdown 與簡單 shell 查核足以完成可驗證交付。

## Risks / Trade-offs

- [狀態矩陣很快再次過時] → 集中於單一 product-status 文件、附查核日期與證據，README 只做摘要與連結。
- [README、getting-started、workflow 重複敘述] → README 固定保留品牌與起源首段，但操作細節以責任分層限制：README 提供定位、狀態摘要與最短心智模型，getting-started 承載單一路徑，workflow 才承載分支與恢復。
- [文件把底層 crate 誤當完整產品能力] → 使用 D2 的雙證據規則並強制寫限制／下一步。
- [中英文漂移] → 章節／矩陣結構對等查核，先完成繁中事實版再逐段鏡像英文。
- [更新 openspec/config.yaml 影響後續 Agent context] → 只修正現況散文與 workspace 組成，不變更 schema、policy、rules 或設定解析；回退只需還原該段文字。
- [既有進階文件缺口擴大範圍] → product-status 誠實揭露並導向後續 change，本次不以空殼或未驗證內容假裝補齊。

## Migration Plan

1. 先以 CLI／skills／workspace／tests 盤點目前能力，建立繁中 product-status 與 workflow。
2. 依繁中事實版產生英文鏡像，再收斂 getting-started 與 README，避免先在四個入口各自編寫造成漂移。
3. 只在架構藍圖、Roadmap 與 openspec/config.yaml 補定位與現況邊界，不搬動架構章節。
4. 執行連結、章節對等、命令存在、狀態證據與詞彙查核；最後以 `speclink validate unify-user-documentation` 驗證 artifacts。

回退策略是整體還原本 change 觸及的 Markdown 與 context 散文；無資料遷移、相容格式或 runtime rollback。

## Open Questions

無。文件責任、狀態分級、工作流範圍與雙語策略均已由本設計固定；執行時只需依當下 checkout 證據填入最新狀態。
