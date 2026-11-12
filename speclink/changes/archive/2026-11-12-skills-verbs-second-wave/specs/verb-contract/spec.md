## MODIFIED Requirements

### Requirement: store 文件讀取動詞
<!-- BEFORE: 「改寫經動詞」只規定 discuss、propose、ingest 三支技能；沒有規定規格不在 checkout 時產生的技能內容 -->

CLI SHALL 提供雙模式的文件讀取動詞：speclink artifact cat <artifact> --change <name> 輸出該 artifact 的內容；speclink language show 輸出共用詞彙文件內容（fs 模式讀本地檔、remote 模式經契約端點）；文件不存在時 SHALL 以非 0 exit code 與語義化訊息結束。生成技能資產中的文件閱讀指示 SHALL 使用這些動詞，SHALL NOT 指示直接讀取規格目錄下的檔案路徑。讀取正式規格的指示 SHALL 以 speclink show <capability> --item-type spec 表述（不指定類型時，同名的作用中變更優先），SHALL NOT 指示以 speclink list --specs --json 回傳的 path 欄位開啟檔案——該欄位在 remote 模式是儲存的鍵，不是本機路徑。全部產生的技能中改寫既有 artifact 的指示 SHALL 以「speclink artifact cat 讀出全文、修改後以 speclink new artifact 帶 --stdin --force 整份寫回」表述，SHALL NOT 留下未指名動詞的讀取或改寫步驟。讀取 evidence 的指示 SHALL 以 speclink evidence show 表述；判斷變更有哪些 delta 的指示 SHALL 以 speclink list --json 的 deltaCapabilities 表述；判斷變更閒置多久的指示 SHALL 以 speclink list --json 的 created 與 modifiedAt 表述，SHALL NOT 以 git 歷史判斷。以「規格資料夾不在這個 checkout」產生的技能 SHALL NOT 含規格資料夾的路徑，SHALL NOT 含 `.evidence.json` 或 `.meta.yaml`，SHALL NOT 含對規格檔的 git 步驟。以「規格資料夾在這個 checkout」產生的技能中，含規格資料夾路徑的句子 SHALL 只用於 git 步驟與路徑的顯示，SHALL NOT 是讀取或改寫的指示。直接寫入規格資料夾的技能（baseline 與 manual）不受這兩句限制，豁免 SHALL 明列於守門測試；這兩支技能以「規格資料夾不在這個 checkout」產生時 SHALL 在動手寫入之前先說明不支援並結束（見 baseline-skill 與 manual-skill）。

#### Scenario: artifact cat 於兩模式輸出一致
- **WHEN** 分別於 fs 模式與 remote 模式（server 存有相同內容）執行 speclink artifact cat proposal --change 某 change
- **THEN** 兩模式 stdout 輸出相同的 artifact 內容，exit code 為 0

#### Scenario: 詞彙文件缺失
- **WHEN** 專案無共用詞彙文件時執行 speclink language show
- **THEN** exit code 非 0，stderr 訊息說明詞彙文件不存在（技能據此靜默跳過詞彙載入）

#### Scenario: 技能資產不含直接讀檔指示
- **WHEN** 執行 speclink update 後掃描生成的全部 SKILL.md 內容
- **THEN** 不存在指示直接開啟規格目錄檔案路徑的語句，文件閱讀一律以 speclink 動詞表述

#### Scenario: discuss 以 show 讀正式規格

- **WHEN** 檢視生成的 discuss 技能中讀取候選規格 Purpose 的步驟
- **THEN** 該步驟以 speclink show <capability> --item-type spec 表述；全文不存在把 path 欄位與 spec.md 寫在同一句的指示

#### Scenario: 三支技能的改寫經動詞

- **WHEN** 檢視生成的 propose 與 ingest 技能中修正或更新既有 artifact 的步驟
- **THEN** 每個這樣的步驟都指向「speclink artifact cat 讀出、speclink new artifact 帶 --stdin --force 寫回」的做法；兩支技能的全文不含 config.yaml

#### Scenario: remote 模式下 discuss 讀得到正式規格

- **WHEN** 於 remote 模式（checkout 內沒有規格資料夾）依 discuss 技能的步驟讀取某個 capability 的 Purpose
- **THEN** 執行的是 speclink show <capability> --item-type spec，stdout 含該規格的 Purpose，exit code 為 0

#### Scenario: 同名的作用中變更不會被誤讀成正式規格

- **WHEN** 專案有一個與某 capability 同名的作用中變更，依 discuss 技能的步驟讀那個 capability 的 Purpose
- **THEN** 執行的指令帶 --item-type spec，stdout 是正式規格的內容，不是那個變更的內容

#### Scenario: remote 專案的技能檔不含規格路徑

- **WHEN** 以規格資料夾名稱 openspec、「規格資料夾不在這個 checkout」渲染 baseline 與 manual 以外的全部技能（Claude 與中性 tool-call 兩種目標）
- **THEN** 結果不含 `openspec/`，不含 `.evidence.json`，不含 `.meta.yaml`

#### Scenario: 本機專案的技能檔不以路徑讀寫

- **WHEN** 以「規格資料夾在這個 checkout」渲染同一批技能
- **THEN** 含 `openspec/` 的每一段（到下一個空白行為止的連續幾行；以冒號結尾的引導句和緊接在後的那一段算同一段）都不含 read、open、glob、grep、cat、check for、check whether、compare … against、edit、write、rewrite 這些字，也不含它們加了字首或字尾的寫法（例如 reads、re-read、editing、rewritten、overwrite）

#### Scenario: commit 與 worktree 開工前以動詞讀 evidence

- **WHEN** 檢視產生的 commit 與 apply-with-worktree 技能中取得「這個變更改了哪些檔」的步驟
- **THEN** 該步驟以 speclink evidence show 表述

#### Scenario: 閒置的判斷不用 git 歷史

- **WHEN** 檢視產生的 apply 與 drift 技能中判斷變更閒置多久的步驟
- **THEN** 該步驟以 speclink list --json 的 created 與 modifiedAt 表述，並寫明沒有 modifiedAt 時只看建立時間；apply 在把變更標成進行中之前讀取這兩個欄位，apply-with-worktree 在建立 worktree 之前讀取（這兩個動作都會讓 modifiedAt 變成現在）；全文不含對規格資料夾的 git log

#### Scenario: 守門抓到新加入的路徑讀取

- **WHEN** 在一份未豁免的原稿加入一句以路徑讀取規格檔的指示
- **THEN** 守門測試失敗並指名該技能
