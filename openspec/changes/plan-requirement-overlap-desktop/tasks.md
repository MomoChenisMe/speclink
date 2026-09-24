## 1. 桌面清單六欄同進同出

- [ ] 1.1 apps/desktop/core/src/query.rs 在 plan 列存在時多寫 `requirementOverlap` 與 `archiveAfter` 兩鍵；成環或壞 meta 時與 wave 等一起缺席，dependsOn 成環例外維持。驗證：query.rs 測試 PLAN_KEYS 改為六鍵、新增 list_change_carries_requirement_overlap_and_archive_after 與 cycle_omits_all_plan_keys_but_depends_on，`cargo test -p speclink-desktop-core query::`（規格：變更清單的排程欄位）。 <!-- speclink-task:tsk_01M38JQ5S614Y30SQRHAM1BEGD -->
- [ ] 1.2 apps/desktop/src-tauri/src/remote.rs 的排程欄組加 requirementOverlap 與 archiveAfter，merge_plan 逐欄搬運；舊 server 缺欄位時為空陣列（沿 wire 型別的 default）。驗證：apps/desktop/src-tauri/tests/it/remote_data.rs 新增 merge_plan_carries_six_plan_fields 與 merge_plan_old_server_yields_empty_new_fields，`cargo test -p speclink-desktop --test it remote_data`（規格：remote 變更清單的排程欄位）。 <!-- speclink-task:tsk_01M38JQ5S6FZH0T51P8PWEZ4ME -->
- [ ] 1.3 packages/ui/src/adapter.ts 的 ChangeItem 加 `requirementOverlap?: ChangeRequirementOverlap[]` 與 `archiveAfter?: string[]` 並定義 ChangeRequirementOverlap 型別；packages/ui/src/stage.ts 新增 planRequirementOverlap(c) 與 planArchiveAfter(c)（wave 缺席即空陣列）。驗證：packages/ui/src/__tests__/planBadge.test.tsx 追加兩個入口的缺席與存在測試，`npm test -w packages/ui -- planBadge`（規格：詳情抽屜的排程分頁）。 <!-- speclink-task:tsk_01M38JQ5S62XT9TKNH0YNTS53T -->

## 2. 排程分頁卡片化，阻擋併入前置卡的狀態徽章

- [ ] 2.1 packages/ui/src/components/RichDetailDrawer.tsx 的 PlanTab 改為四張邊框卡片（data-plan-section 為 wave、depends、overlaps、archive，外觀沿 TicketView 的 `rounded-md border border-border/60 bg-muted/20`、標題列與內容區以 border-t 分格）；移除 data-plan-section="blocked" 段；前置卡標題列右側渲染 Badge：blockedBy 空為「可以開工」（secondary）、非空為「等 N 項」（outline）且 title 為 planBlockedLabel 的前置名。驗證：packages/ui/src/__tests__/planTab.test.tsx 改寫為四卡錨定，新增 depends_card_badge_ready_and_waiting 與 no_blocked_section_rendered，`npm test -w packages/ui -- planTab`（規格：詳情抽屜的排程分頁）。 <!-- speclink-task:tsk_01M38JQ5S6P90FNPBD4QHRT34D -->
- [ ] 2.2 重疊卡改列 requirementOverlap：每列對方 PlanName、`capability › requirement` 文字、ownOperation 與 otherOperation 兩枚小標籤（`rounded border border-border/60 px-1 py-0.5 text-[10px]`）、conflict 為 true 時加 destructive 色「同名衝突」標籤；不再渲染目錄級 overlaps；空時「無重疊」。驗證：planTab.test.tsx 新增 overlaps_card_lists_requirement_level_rows 與 conflict_row_shows_name_conflict_tag，`npm test -w packages/ui -- planTab`（規格：詳情抽屜的排程分頁）。 <!-- speclink-task:tsk_01M38JQ5S6SAEDYZGNZTFP3Z9G -->
- [ ] 2.3 新增封存順序卡：archiveAfter 每項 PlanName，下方提示句「先封存它們，再對照正典重寫同名 requirement 後封存本變更」；空時「可直接封存」；wave 缺席的兩種說明句與成環時只渲染前置卡（無徽章、無新增下拉）維持。驗證：planTab.test.tsx 新增 archive_card_lists_names_and_hint、archive_card_empty_state、old_server_missing_fields_show_empty_states、cycle_renders_only_depends_card_without_badge，`npm test -w packages/ui -- planTab`（規格：詳情抽屜的排程分頁）。 <!-- speclink-task:tsk_01M38JQ5S6AWR99PF7188H2ZXB -->

## 3. i18n 詞條

- [ ] 3.1 packages/ui/src/i18n.tsx 新增 tw／en 的 plan.archiveAfter、plan.archiveHint、plan.noArchiveAfter、plan.conflict、plan.blockedCount，移除 plan.blocked，plan.canStart 改用於徽章；tw 與 en 鍵集合一致。驗證：既有 i18n 鍵一致性測試綠，planTab 測試以 tw 文案斷言，`npm test -w packages/ui`（規格：詳情抽屜的排程分頁）。 <!-- speclink-task:tsk_01M38JQ5S6MT6CXG0RFJVQPP0T -->

## 4. 桌面整合與收尾驗證

- [ ] 4.1 apps/desktop/src/__tests__/App.test.tsx 與 apps/desktop/src/__tests__/remoteCapabilities.test.tsx 對排程分頁的斷言改為四卡錨定：remote 資料源（setDepends 為 false）四卡照常顯示且無移除鈕與下拉；local 清單項六欄注入後抽屜顯示重疊卡與封存順序卡內容。驗證：`npm test -w apps/desktop -- App remoteCapabilities`（規格：詳情抽屜的排程分頁、remote 變更清單的排程欄位）。 <!-- speclink-task:tsk_01M38JQ5S6QRV3YDJA266R67ME -->
- [ ] 4.2 全面回歸：`cargo test -p speclink-desktop-core query::`、`cargo test -p speclink-desktop --test it remote_data`、`npm test -w packages/ui`、`npm test -w apps/desktop`；最後執行 `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 確認詞彙守門與連結檢查綠。 <!-- speclink-task:tsk_01M38JQ5S6B52DEWW6JTZ3JEZC -->
