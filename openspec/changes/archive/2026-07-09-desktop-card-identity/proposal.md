## Why

桌面的 discuss 卡與 change 卡在身分與 meta 呈現上有落差：discuss 卡看不到也無法複製英文檔名（slug，是執行 CLI 動詞的把手）、change 卡缺少抽屜已有的建立者資訊；且討論記錄未蓋建立者章，本機小團隊與遠端模式無從得知討論由誰發起。

## What Changes

- discuss 卡改以檔名（slug）為標題、topic 降為卡身描述、並加複製鈕（比照 change 卡以名稱為題＋複製）。
- change 卡加建立者（createdBy）頭像；「來自討論」與「同源」關係於 hover 以提示呈現對應資訊。
- 引擎於 discuss new 蓋建立者章（created_by，取自 git 身分，比照 change 的既有機制），經 list／show --json 曝露，DiscussionItem 帶 createdBy，討論抽屜與卡片顯示「誰發起」。
- 詞彙例外：discuss 卡以 slug 為題屬「slug 不出現於使用者可見文案」原則的受控例外，記入 openspec/LANGUAGE.md（比照 config.yaml 頁簽先例）。

## Non-Goals

見 design.md 的 Goals/Non-Goals。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `discussion-docs`：新增「討論記錄蓋建立者章」需求（discuss new 以 git 身分蓋 created_by、經 --json 曝露）。
- `desktop-app`：修改「討論於看板第 0 欄兩級呈現」（discuss 卡改以 slug 為題、topic 為描述、加複製鈕與建立者）；新增「看板變更卡呈現建立者與關係提示」需求。

## Impact

- Affected specs: discussion-docs（modified）、desktop-app（modified）
- Affected code:
  - Modified:
    - crates/speclink-core/src/discuss.rs
    - crates/speclink-core/src/model.rs
    - apps/desktop/core/src/query.rs
    - apps/desktop/src/adapter/tauriDataSource.ts
    - packages/ui/src/adapter.ts
    - packages/ui/src/components/DiscussionColumn.tsx
    - packages/ui/src/components/DiscussionDrawer.tsx
    - packages/ui/src/components/ChangeCard.tsx
    - packages/ui/src/i18n.tsx
    - openspec/LANGUAGE.md
    - packages/ui/src/__tests__/discussionColumn.test.tsx
    - packages/ui/src/__tests__/changeListItem.test.tsx
  - New: (none)
  - Removed: (none)
