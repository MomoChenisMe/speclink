## Purpose

/speclink-baseline 技能的行為契約：技能檔的渲染與保護、盤點前取得 workflow config 並把 specs 產出規則套用到第一批正式 specs（含寫入前與報告時的揭露），以及基準盤點只記錄現況、不建 change、不改 code、只補缺、確認後才寫、最後 strict validation 的六項邊界。本 capability 只管 baseline 技能的內文行為；入口 description 與出口交棒句屬 skill-routing，workflow-config show 動詞本身屬 workflow-config。

## ADDED Requirements

### Requirement: 內嵌 speclink-baseline 技能的渲染與保護

內嵌 speclink-baseline 技能（事實來源 crates/speclink-core/assets/skills/baseline.md）SHALL 經 init 與 update 渲染至各工具技能目錄（claude 與 codex），與既有內嵌技能同機制；兩側渲染產物 SHALL 源自同一份資產，SHALL NOT 存在工具別的平行版本。渲染產物內容由 speclink-core 的 render_golden 測試（cargo test）保護，golden 快照更新屬刻意變更。本 change 對五份 golden 快照的 baseline 段落與資產版本戳的變更屬刻意變更；其他技能的渲染產物 SHALL 僅版本戳改變，內文 SHALL 維持既有位元級輸出。本 capability 不新增任何 CLI 子指令、旗標、人眼輸出或 --json payload；檔案系統效果限技能目錄下 speclink-baseline/SKILL.md 的生成與更新。

#### Scenario: init 與 update 渲染技能

- **WHEN** 執行 speclink init 或 speclink update 且工具含 claude 與 codex
- **THEN** .claude/skills/speclink-baseline/SKILL.md 與 .agents/skills/speclink-baseline/SKILL.md 各生成，內容源自 baseline.md 資產的渲染，exit code 0

#### Scenario: golden 保護渲染產物

- **WHEN** baseline.md 資產變更後執行 cargo test -p speclink-core --test it 的 render_golden 測試
- **THEN** 快照不符時測試失敗；刻意變更以快照再生落地並可審視 diff

#### Scenario: 其他技能只變版本戳

- **WHEN** 本 change 落地後於既有 workspace 執行 speclink update
- **THEN** 全部 speclink-* 技能檔的 frontmatter 版本戳為 v1.27.0；除 speclink-baseline 外，各技能檔的內文與 v1.26.1 的渲染產物逐位元相同

### Requirement: 盤點前取得 workflow config 並套用 specs 產出規則

渲染產出的 speclink-baseline 技能檔 SHALL 規定：Step 1 檢查現況時 MUST 執行 speclink workflow-config show --json 取得正典 workflow config（openspec/config.yaml 或 remote store 的 config 文件所載的值；SHALL NOT 套用 SPECLINK_* 環境變數覆寫，與 workflow-config capability 的 show 語意一致），並從 payload 讀取三個欄位——context（專案說明，作為盤點與每份 spec 的背景）、specLocale（null 時 spec 散文以英文撰寫；auto 時以同一 payload 的 locale 為準；其他值為語系代碼 tw、ja 或 en，spec 散文以該語言撰寫，結構標記與 SHALL/MUST 關鍵字維持英文）、rules.specs（字串陣列，或不存在）。技能檔 SHALL NOT 指示直接讀取 openspec/config.yaml，SHALL NOT 指示自行解析 YAML。

rules.specs 存在且非空時，技能檔 SHALL 規定本輪 Step 4 產生的每一份正式 spec MUST 遵守每一條規則，規則原文照套、不翻譯、不篩選適用性。rules.specs 不存在或為空清單時，技能檔 SHALL 規定 spec 內容規則與現行相同。

技能檔 SHALL 規定兩處揭露：Step 3 的 capability map 確認訊息與 Step 5 的最終報告各帶同一段固定文字——有規則時首行為「Specs rules applied this run (from rules.specs, N entries):」，其後逐條編號列出規則原文；無規則時為單行「Specs rules applied this run: none (no rules.specs configured)」。技能檔 SHALL 明文：這些規則是 Agent 產生內容時必須遵守的指令，speclink validate --specs --all --strict 只檢查結構（checks structure only），不機械式驗證自由文字規則。

speclink workflow-config show --json 以非零 exit code 結束時（openspec/config.yaml 無法解析的 fail-closed、remote 模式離線或認證失效），技能檔 SHALL 規定 Agent 回報該錯誤並停止，SHALL NOT 寫入任何 spec，SHALL NOT 退回手讀 config.yaml（never fall back to reading）。remote 模式下 workflow-config show 的 payload 形狀與 fs 模式一致（由 workflow-config capability 保證），本 capability 對兩模式不做差別規定；Baseline 直接寫檔到 openspec/specs/ 的本機行為維持既有範圍，SHALL NOT 因本 change 擴張。

#### Scenario: 技能檔載明取得 workflow config 的入口

- **WHEN** 檢視 .claude/skills/speclink-baseline/SKILL.md 或 .agents/skills/speclink-baseline/SKILL.md 的 Step 1
- **THEN** 內文含 speclink workflow-config show --json，並點名 context、specLocale 與 rules.specs 三個欄位；內文不含指示直接讀取 openspec/config.yaml 的句子

#### Scenario: 設定了 rules.specs 時的套用與揭露

- **WHEN** openspec/config.yaml 的 rules.specs 含兩條規則，Agent 依技能執行 baseline 並產生兩份正式 spec
- **THEN** capability map 確認訊息含「Specs rules applied this run (from rules.specs, 2 entries):」與兩條規則原文；兩份 spec 各遵守兩條規則；最終報告含同一段兩條規則的揭露

##### Example: 揭露段依 rules.specs 狀態的字面

| rules.specs 狀態 | map 確認訊息與最終報告的揭露段 |
| --- | --- |
| 兩條規則 | Specs rules applied this run (from rules.specs, 2 entries): 其後 1. 與 2. 各列一條原文 |
| 空清單 | Specs rules applied this run: none (no rules.specs configured) |
| 鍵不存在 | Specs rules applied this run: none (no rules.specs configured) |

#### Scenario: 未設定 rules.specs 時行為不變

- **WHEN** openspec/config.yaml 沒有 rules.specs（或 rules 節不存在），Agent 依技能執行 baseline
- **THEN** capability map 確認訊息與最終報告各含單行「Specs rules applied this run: none (no rules.specs configured)」；spec 內容規則與現行相同，仍寫入 openspec/specs/<capability>/spec.md

#### Scenario: specLocale 決定 spec 散文語言

- **WHEN** payload 的 specLocale 分別為 null、auto、tw
- **THEN** 技能檔規定 spec 散文分別以英文、locale 所指語言、繁體中文撰寫；三種情況下結構標記與 SHALL/MUST 關鍵字均維持英文

#### Scenario: workflow config 讀取失敗即停止

- **WHEN** openspec/config.yaml 含 YAML 語法錯誤（或 remote 模式離線、認證失效），Agent 依技能執行 baseline
- **THEN** speclink workflow-config show --json 以非零 exit code 結束，Agent 回報 stderr 的錯誤並停止；openspec/specs/ 下無任何新檔；Agent 不改讀 config.yaml

#### Scenario: 規則屬 Agent 指令而非機械驗證

- **WHEN** 檢視渲染產出的 speclink-baseline 技能檔的 Step 4 規則段
- **THEN** 內文含「MUST honour every entry」要求每份 spec 遵守全部 rules.specs，並含「checks structure only」說明 speclink validate 不驗證自由文字規則

### Requirement: 基準盤點的行為邊界

渲染產出的 speclink-baseline 技能檔 SHALL 規定六項邊界：(1) 只記錄系統目前已存在的行為，每條 requirement 追溯到實際讀過的 code 或 tests，無法驗證的推論標記或省略；(2) SHALL NOT 修改任何程式碼；(3) SHALL NOT 建立 change，正式 specs 直接寫入 openspec/specs/<capability>/spec.md，SHALL NOT 在 openspec/changes/ 下建立任何目錄或檔案；(4) 已有 specs 時只做 gap filling，SHALL NOT 重寫既有 spec，修改走 change；(5) capability map 經使用者確認前 SHALL NOT 寫入任何 spec；(6) 寫入後 MUST 執行 speclink validate --specs --all --strict 並修正結構性發現。此六項為既有行為的首次正典化，技能檔對應段落的語意 SHALL 維持既有；本 change 對技能檔的刻意變更僅限「盤點前取得 workflow config 並套用 specs 產出規則」所述的段落。

#### Scenario: 已有 specs 時只補缺

- **WHEN** openspec/specs/ 已有三個 capability，Agent 依技能執行 baseline
- **THEN** 技能進入 gap-filling 模式，只盤點未覆蓋的行為區域；既有三份 spec.md 逐位元不變

#### Scenario: 寫入前等待 capability map 確認

- **WHEN** Agent 提出 capability map 而使用者尚未確認
- **THEN** openspec/specs/ 下無任何新檔；使用者確認（含合併、拆分或刪除 capability）後才依確認後的清單寫入

#### Scenario: 不建 change 不改 code

- **WHEN** Agent 依技能完成一次 baseline
- **THEN** openspec/changes/ 無新增檔案；repo 中除 openspec/specs/ 下新增的 spec.md 外無其他檔案異動

#### Scenario: 最後執行 strict validation

- **WHEN** Agent 寫完全部確認的 spec
- **THEN** 執行 speclink validate --specs --all --strict；有結構性發現時修正後重跑；報告列出建立的 capability（含 requirement 與 scenario 數）、標記為未驗證的行為、刻意略過的區域，以及揭露段
