## MODIFIED Requirements

### Requirement: improve 技能以六步骨架渲染至兩工具

內嵌 speclink-improve 技能(事實來源 crates/speclink-core/assets/skills/improve.md,經 init 與 update 渲染至 claude 與 codex 工具技能目錄)SHALL 以六步骨架規定流程:載入詞彙(speclink language show)、防重提檢查、範圍收斂、掃描、建記錄呈現 candidates、grilling 收斂;並 SHALL 標示技能僅由使用者發起、模型 SHALL NOT 自行觸發、SHALL NOT 於流程中實作程式碼。防重提檢查 SHALL 規定:開場以範圍關鍵字執行 speclink discuss search(--json)取得在途與封存討論中 Ruled out 與結論的命中,閱讀順序 SHALL 將 kind 為 improve 且同範圍的舊記錄排前;已否決方案 SHALL NOT 再列為 candidate,除非敘明當時否決理由已失效;並讀取 speclink list 的 in-flight changes,與其重疊區域的 candidate SHALL NOT 提出。技能檔 SHALL NOT 再指示以 speclink discuss list --archived 後逐筆 show 作為防重提的讀取方式。本能力屬 Speclink 自身延伸;渲染產物內容由 speclink-core 的 render_golden 測試(cargo test)保護,golden 快照更新屬刻意變更。

#### Scenario: 渲染產物含六步骨架

- **WHEN** 執行 speclink init 或 speclink update 渲染 claude 與 codex 工具的技能檔
- **THEN** 產出的 speclink-improve 技能檔 SHALL 依序含載入詞彙、防重提檢查、範圍收斂、掃描、建記錄呈現 candidates、grilling 收斂六步,且含「僅使用者發起」與「不得實作」的限定

#### Scenario: 防重提檢查涵蓋已封存討論與 in-flight changes

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的防重提段落
- **THEN** 內容 SHALL 規定以 speclink discuss search 取得已封存與在途討論的 Ruled out 與結論命中以排除已否決方案、同範圍舊 improve 記錄排前閱讀,並規定避開 in-flight changes 的重疊區域;SHALL NOT 含 speclink discuss list --archived 後逐筆 show 的指示
