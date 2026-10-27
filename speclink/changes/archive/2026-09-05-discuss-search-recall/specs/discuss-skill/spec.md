## ADDED Requirements

### Requirement: 開場舊討論查核與第四類對照

內嵌 speclink-discuss 技能（事實來源 crates/speclink-core/assets/skills/discuss.md，經 init 與 update 渲染至 claude 與 codex 工具技能目錄）SHALL 將偵察漏斗規定為「正典 → 舊討論查核 → 程式碼」三段。舊討論查核 SHALL 規定：以使用者題目的關鍵字加正典掃描轉譯出的英文詞執行 speclink discuss search（--json），命中的決定行 SHALL 全數列出；整份 Conclusion 以 speclink discuss show 讀取 SHALL 最多 3 份、topic 命中者優先；此查核 SHALL NOT 依討論的 kind 過濾。假設清單 SHALL 在既有三分對照之外加入第四類「舊討論已定案」，細分曾否決（附當時理由；重開該方向 SHALL 說明當時理由已失效）、曾延後（可接手）、已落地（正典會照出，不重列）；技能檔 SHALL 明文不得以此擋下討論方向。Context 段 SHALL 規定加一行 `Prior discussions: <slug 清單>`，零命中時寫 none。既有討論記錄格式與 Context／Rounds／Conclusion 骨架 SHALL 不變。本能力屬 Speclink 自身延伸；渲染產物內容由 speclink-core 的 render_golden 測試（cargo test）保護，golden 快照更新屬刻意變更。

#### Scenario: 渲染產物含舊討論查核

- **WHEN** 執行 speclink init 或 speclink update 渲染 claude 與 codex 工具的技能檔
- **THEN** 產出的 speclink-discuss 技能檔 SHALL 將偵察描述為「正典 → 舊討論查核 → 程式碼」，舊討論查核 SHALL 指示執行 speclink discuss search、命中決定行全列、整份 Conclusion 最多讀 3 份且 topic 命中優先，並 SHALL NOT 含依 kind 過濾的指示

#### Scenario: 曾否決方向重開須說明理由失效

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的假設清單對照規則
- **THEN** 技能檔 SHALL 含第四類「舊討論已定案」及曾否決、曾延後、已落地三種細分，規定重開曾否決方向須說明當時理由已失效，且 SHALL 明文不得以此擋下討論方向

#### Scenario: Context 記錄舊討論來源

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的 Context 撰寫規定
- **THEN** 技能檔 SHALL 規定 Context 加一行 `Prior discussions: <slug 清單>`，零命中時寫 none
