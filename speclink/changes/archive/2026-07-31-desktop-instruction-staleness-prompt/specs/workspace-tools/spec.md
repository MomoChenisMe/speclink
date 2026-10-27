## ADDED Requirements

### Requirement: 產物層版本戳同源

生成的技能檔 frontmatter 版本欄位 SHALL 與指令檔 SPECLINK 標記的版號同值同源（單一產物層版號）。該版號 SHALL 僅於內嵌資產的 render 內容變動時遞增，SHALL NOT 隨 app 或 CLI 發版自動變動。

#### Scenario: 生成物的版號一致

- **WHEN** tools 含 claude 與 codex，執行 speclink init 或 speclink update 後檢視 CLAUDE.md 與 AGENTS.md 的標記版號、及 .claude/skills/ 與 .agents/skills/ 下任一技能檔的 frontmatter 版本欄位
- **THEN** 四處為相同的版號字串，無任何技能檔殘留固定值 "1.0"

### Requirement: 指令檔過期探測

引擎 SHALL 提供唯讀的指令檔過期探測：依 .speclink.yaml 的 tools 清單，讀取各內建工具指令檔的 SPECLINK 標記版號並與當前產物層版號比對，回報四態之一——缺失（任一工具的指令檔不存在，即從未安裝）、過期（任一工具的標記版號與現版不等）、現版、無法判定（設定解析失敗或指令檔存在但讀取錯誤）；缺失與過期並存時 SHALL 回報缺失。版號比對 SHALL 為字串相等判定，SHALL NOT 解析版本語意。指令檔存在但不含 SPECLINK 標記時，該工具 SHALL 視為已退出受管、不計入過期與缺失；指令檔不存在 SHALL 判缺失，SHALL NOT 與退出受管或無法判定混同。過期或缺失時 SHALL 一併回報「更新將新建或改寫且內容與現版 render 不同」的受管檔清單（專案根相對路徑）；比對前 SHALL 正規化換行，僅換行形式差異的檔案 SHALL NOT 列入清單。探測 SHALL NOT 寫入任何檔案。

#### Scenario: 舊版工作區判過期並列差異檔

- **WHEN** 工作區 CLAUDE.md 的標記版號與當前產物層版號不等，執行過期探測
- **THEN** 回報過期，並列出內容與現版 render 不同的受管檔相對路徑（含技能檔與指令檔）

#### Scenario: 現版工作區不過期

- **WHEN** 工作區全部受管檔由當前版本的 init 或 update 生成，執行過期探測
- **THEN** 回報現版，差異清單為空

#### Scenario: 標記移除視為退出受管

- **WHEN** tools 清單僅含 claude 且 CLAUDE.md 不含 SPECLINK 標記（使用者整塊移除），執行過期探測
- **THEN** 回報現版（不過期），不列任何差異檔

#### Scenario: 指令檔不存在判缺失

- **WHEN** tools 清單含 claude 與 codex，CLAUDE.md 為現版而 AGENTS.md 不存在（如 clone 後指令檔未進版控），執行過期探測
- **THEN** 回報缺失，並列出更新將新建或改寫且內容與現版 render 不同的受管檔相對路徑；不與退出受管（檔案存在但無標記）或無法判定混同

#### Scenario: 設定損壞回報無法判定

- **WHEN** .speclink.yaml 無法解析，執行過期探測
- **THEN** 回報無法判定；SHALL NOT 與現版或過期混同

#### Scenario: 換行差異不誤報

- **WHEN** 工作區技能檔內容與現版 render 僅換行形式不同（CRLF 對 LF），執行過期探測
- **THEN** 該檔不出現在差異清單

### Requirement: 內嵌資產版本鎖定紀律

repo SHALL 提交記錄產物層版號與全部 render 輸出指紋的鎖定檔。鎖定測試 SHALL 於 render 指紋與鎖定檔不符而版號未變時失敗，失敗訊息 SHALL 載明修復步驟（遞增版號後以指定環境變數重生鎖定檔）。鎖定檔重生 SHALL 於指紋變動而版號未變時拒絕改寫並失敗。僅遞增版號而 render 內容未變 SHALL 通過。

#### Scenario: 改資產未遞增版號即紅燈

- **WHEN** 修改內嵌技能資產內容而未遞增產物層版號，執行 speclink-core 測試
- **THEN** 鎖定測試失敗，測試輸出含遞增版號與重生鎖定檔的修復指引

#### Scenario: 防呆重生拒絕繞過

- **WHEN** 未遞增版號即以重生環境變數執行鎖定測試，且 render 指紋已變
- **THEN** 鎖定檔不被改寫，測試失敗

#### Scenario: 遞增並重生後通過

- **WHEN** 遞增產物層版號並於乾淨樹以重生環境變數更新鎖定檔後，正常執行測試
- **THEN** 鎖定測試通過，鎖定檔記錄新版號與新指紋

## MODIFIED Requirements

### Requirement: 中性渲染目標

<!-- BEFORE: 內建 claude 與 codex 的生成內容 SHALL 與本變更前（config-system-rework）位元級一致；本次因 frontmatter 版本戳改為同源值而重新錨定於 render golden 基線 -->

描述子生成的技能與指令區塊 SHALL 使用中性渲染：內文 SHALL NOT 含 /speclink- slash 前綴與 plan mode 參照；speclink 動詞的措辭依 invocation 決定——cli 為「執行 speclink <動詞>」形式，tool-call 為「呼叫 speclink 工具（參數為 argv 陣列）」形式。內建 claude 與 codex 的生成內容 SHALL 與 render golden 基線位元級一致；基線 SHALL 僅隨提案記載的刻意變更同批更新。

#### Scenario: tool-call 措辭

- **WHEN** 描述子 invocation 為 tool-call，執行 speclink update 後檢視生成的技能檔
- **THEN** 內文以「呼叫 speclink 工具」措辭引用動詞，且不含 /speclink- 前綴與 plan mode 字樣

#### Scenario: 內建工具輸出鎖定於 golden 基線

- **WHEN** tools 僅含 claude 與 codex，執行 speclink update
- **THEN** 生成的 CLAUDE.md、AGENTS.md marker 區塊與 .claude/skills/、.agents/skills/ 技能內容與 render golden 基線完全一致
