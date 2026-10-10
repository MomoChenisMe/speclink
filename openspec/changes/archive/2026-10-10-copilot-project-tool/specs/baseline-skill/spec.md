## MODIFIED Requirements

### Requirement: 盤點前取得 workflow config 並套用 specs 產出規則
<!-- BEFORE: specLocale:null 的技能指引固定寫英文，其他 canonical config 及 rules.specs 契約維持。 -->

渲染產出的 speclink-baseline 技能檔 SHALL 規定：Step 1 檢查現況時 MUST 執行 speclink workflow-config show --json 取得正典 workflow config（openspec/config.yaml 或 remote store 的 config 文件所載的值；SHALL NOT 套用 SPECLINK_* 環境變數覆寫，與 workflow-config capability 的 show 語意一致），並另外執行 speclink workflow-config languages --json 取得具體語言；Agent MUST 使用該查詢的 specLocale 代碼決定 spec 散文語言，SHALL NOT 從 client、對話或介面語言猜測 server 預設。show 的正典 payload 維持不變，並從 payload 讀取三個欄位——context（專案說明，作為盤點與每份 spec 的背景）、specLocale（null 時 spec 散文採執行 Speclink 的作業系統語言（中文 tw、日文 ja、其他或無法取得時 en）；auto 時以同一 payload 的 locale 為準，locale 亦為 null 時採系統語言；其他值為語系代碼 tw、ja 或 en，spec 散文以該語言撰寫，結構標記與 SHALL/MUST 關鍵字維持英文）、rules.specs（字串陣列，或不存在）。技能檔 SHALL NOT 指示直接讀取 openspec/config.yaml，SHALL NOT 指示自行解析 YAML。

rules.specs 存在且非空時，技能檔 SHALL 規定本輪 Step 4 產生的每一份正式 spec MUST 遵守每一條規則，規則原文照套、不翻譯、不篩選適用性。rules.specs 不存在或為空清單時，技能檔 SHALL 規定 spec 內容規則與現行相同。

技能檔 SHALL 規定兩處揭露：Step 3 的 capability map 確認訊息與 Step 5 的最終報告各帶同一段固定文字——有規則時首行為「Specs rules applied this run (from rules.specs, N entries):」，其後逐條編號列出規則原文；無規則時為單行「Specs rules applied this run: none (no rules.specs configured)」。技能檔 SHALL 明文：這些規則是 Agent 產生內容時必須遵守的指令，speclink validate --specs --all --strict 只檢查結構（checks structure only），不機械式驗證自由文字規則。

speclink workflow-config show --json 或 languages --json 以非零 exit code 結束時（openspec/config.yaml 無法解析的 fail-closed、remote 模式離線、認證失效或舊 server 缺少語言資料），技能檔 SHALL 規定 Agent 回報該錯誤並停止，SHALL NOT 寫入任何 spec，SHALL NOT 退回手讀 config.yaml（never fall back to reading）。remote 模式下 workflow-config show 的 payload 形狀與 fs 模式一致（由 workflow-config capability 保證），本 capability 對兩模式不做差別規定；Baseline 直接寫檔到 openspec/specs/ 的本機行為維持既有範圍，SHALL NOT 因本 change 擴張。

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
- **THEN** 技能檔規定 spec 散文分別以系統語言、locale 所指語言（locale 為 null 時使用系統語言）、繁體中文撰寫；三種情況下結構標記與 SHALL/MUST 關鍵字均維持英文

#### Scenario: workflow config 讀取失敗即停止

- **WHEN** openspec/config.yaml 含 YAML 語法錯誤（或 remote 模式離線、認證失效），Agent 依技能執行 baseline
- **THEN** speclink workflow-config show --json 以非零 exit code 結束，Agent 回報 stderr 的錯誤並停止；openspec/specs/ 下無任何新檔；Agent 不改讀 config.yaml

#### Scenario: 規則屬 Agent 指令而非機械驗證

- **WHEN** 檢視渲染產出的 speclink-baseline 技能檔的 Step 4 規則段
- **THEN** 內文含「MUST honour every entry」要求每份 spec 遵守全部 rules.specs，並含「checks structure only」說明 speclink validate 不驗證自由文字規則

#### Scenario: 未設定語言的 baseline 指引

- **WHEN** workflow-config show --json 的 specLocale 與 locale 都為 null，檢視生成的 baseline 技能
- **THEN** 技能要求執行 workflow-config languages --json 並按回傳 specLocale 撰寫 spec 散文；Remote 採 Server 語言，中文對應 tw、日文對應 ja、其他或無法取得對應 en；仍經 show 取得正典欄位與 rules.specs，SHALL NOT 為語言預設直接讀取或改寫 config.yaml
