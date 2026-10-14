## ADDED Requirements

### Requirement: 技能規定審查規則的第六問與規則接收

渲染產出的 speclink-config 技能檔 SHALL 規定技能也管理 config 的 review 區塊，並載明以下行為：

- **第六問**：review.baseline（extend 或 replace）與政策四欄、第五問同性質——SHALL 帶出現值逐項詢問使用者，SHALL NOT 自 repo 推斷；使用者無意見時維持原狀。
- **不自 repo 推導審查規則**：技能 SHALL NOT 從固定輸入來源或 repo 文件（CLAUDE.md、AGENTS.md、CONTRIBUTING、lint 設定等）整理出審查規則——審查站的 Standards 軸本身會讀這些文件，複製一份會成為第二份正典。
- **規則接收**：技能 SHALL 詢問使用者是否要新增或修改審查規則；使用者口述的每條規則，技能 SHALL 依語氣建議放入的清單——禁止或一律類語氣（如「不得」「一律」）建議 critical、要求類語氣（如「要」「需」）建議 warning、建議語氣（如「宜」「偏好」）建議 suggestion——並由使用者確認或改分格後才落地。
- **既有審查規則的檢查**：每條既有與新收的審查規則 SHALL 照判準檢查——baseline 為 extend（含未設定）時，與生成的 speclink-review 技能檔內嵌 smell baseline 同義的規則 SHALL 被淘汰；baseline 為 replace 時不做此項檢查；會過時的內容（版本號、計數、統計數字）不寫；引用的路徑、測試名、指令 SHALL 以靜態手段核實存在。審查規則的刪除理由限定與 rules 相同：只因不過判準或使用者明確撤回而刪。
- **落地**：審查規則與 baseline 的寫入 SHALL 經 speclink workflow-config review 子指令的 --dry-run 產出 diff、經使用者確認後才實際寫入，與其餘寫入同一核准流程；收斂驗收（同一未變動 codebase 第二次執行 diff 為空）同樣涵蓋 review 區塊。

#### Scenario: 渲染產物含第六問

- **WHEN** 檢視渲染產出的 speclink-config 技能檔（claude 與 codex 兩 flavor）的政策詢問段落
- **THEN** 段落載明 review.baseline 的第六問、帶現值詢問、不得自 repo 推斷

#### Scenario: 不自 repo 推導審查規則

- **WHEN** 檢視渲染產出的 speclink-config 技能檔關於審查規則的段落
- **THEN** 段落明定技能不從固定輸入來源或 repo 文件整理審查規則，只接收使用者口述的規則

#### Scenario: 依語氣建議分格

- **WHEN** 檢視渲染產出的 speclink-config 技能檔關於規則接收的段落
- **THEN** 段落載明依語氣建議 critical、warning、suggestion 的對應，且分格由使用者確認後才落地

##### Example: 分格建議

| 使用者口述的規則 | 技能建議的清單 |
| ---------------- | -------------- |
| 不得把金鑰或密碼寫死在程式碼裡 | critical |
| 新增的公開 API 要有對應的測試 | warning |
| 函式超過 50 行時宜拆分 | suggestion |

#### Scenario: extend 時淘汰重述 smell 的規則

- **WHEN** baseline 為 extend，使用者口述「變數名稱要能說明用途」（與 smell baseline 的 Mysterious Name 同義）
- **THEN** 技能檔的判準指引該規則被淘汰並向使用者說明理由；baseline 為 replace 時同一條規則可以落地

#### Scenario: 寫入經 dry-run 核准

- **WHEN** 檢視渲染產出的 speclink-config 技能檔的寫入段落
- **THEN** 段落列出 speclink workflow-config review 子指令的 --dry-run 用法，並明定經使用者確認後才不帶 --dry-run 重跑
