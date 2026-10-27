## ADDED Requirements

### Requirement: conclude 範例與討論結論條列規則同形

渲染產出的 speclink-improve 技能檔中，兩個 `speclink discuss conclude improve-<scope> --stdin` 指令範例（收斂轉出與全數否決）的 Conclusion 欄位 SHALL 與 discuss-skill「討論記錄的樹慣例與格式不變」的結論條列規則同形：`**Rejected alternatives**:` 標頭後 SHALL 緊接 `- ` 起頭的條列行；收斂轉出範例的 `**Decision**:` 行之後 SHALL 緊接 `- ` 條列行，並含多刀 bullet `**cut N \`change-name\`**：` 與其縮排子項；全數否決範例的 Decision 為一句「不做」定論、SHALL 得維持單行；`**Deferred**:` 行 SHALL 示範 `- ` 一點一行的形狀或單寫 none。改動 SHALL 只落在兩個範例，技能其餘段落 SHALL 逐位元不變；claude 與 codex 兩工具的技能實例與 render golden SHALL 同步反映。

#### Scenario: 兩個 conclude 範例為條列形

- **WHEN** 檢視 claude、codex、neutral 三種渲染目標產出的 speclink-improve 技能檔
- **THEN** 收斂轉出範例的 `**Decision**:` 與 `**Rejected alternatives**:` 行之後各緊接一行 `- ` 條列、`- **cut N \`change-name\`**：` 行之後緊接縮排子項；全數否決範例的 `**Rejected alternatives**:` 之後緊接 `- ` 條列、Decision 維持一句單行；兩個範例的 `**Deferred**:` 行示範 `- ` 形狀

#### Scenario: 全數否決仍以一句定論起頭

- **WHEN** agent 依更新後的 improve 技能為全數否決的掃描寫入結論
- **THEN** Decision 為一句「不做」定論而不強制條列；Rejected alternatives 每個 candidate 一行「方案——落敗理由」
