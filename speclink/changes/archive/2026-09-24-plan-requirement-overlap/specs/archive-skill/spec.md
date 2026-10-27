## MODIFIED Requirements

### Requirement: 未指名時的候選清單依 plan 順序

<!-- BEFORE: 候選標 blockedBy（空時「無阻擋」） -->

內嵌 speclink-archive 技能 SHALL 於未指名 change 時以 speclink plan --json 的 `changes` 陣列列出候選（依配置順序），每個候選 SHALL 標出其 `archiveAfter`（空時標「可封存」，非空時標「等 <名稱> 封存」），`blockedBy` 非空時 SHALL 再標「前置 <名稱> 未封存」；SHALL 仍由使用者選擇、SHALL NOT 自動選。plan 失敗（依賴成環）時 SHALL 退回 speclink list --json 列候選。其他技能（commit、review、verify、drift、analyze）的候選清單 SHALL 維持 list --json，不受本需求影響。

#### Scenario: 候選依 plan 順序並標阻擋

- **WHEN** 未指名執行 speclink-archive 技能，plan 的 changes 依序為 add-a（archiveAfter 空）、add-b（archiveAfter ["add-a"]）
- **THEN** 候選清單依 add-a、add-b 順序呈現，add-a 標「可封存」、add-b 標「等 add-a 封存」，並由使用者選擇

#### Scenario: plan 失敗退回 list

- **WHEN** 未指名執行 speclink-archive 技能且 plan 回依賴成環錯誤
- **THEN** 候選清單改以 speclink list --json 的順序呈現，不標阻擋
