---
topic: 設定中增加深色／淺色畫面選擇，並於專案設定顯示專案目錄
slug: desktop-theme-and-project-directory
status: promoted
created: 2026-10-12
created_by: mosutw <luke1974@gmail.com>
promoted_to: desktop-theme-and-project-directory
---

# Discussion: 設定中增加深色／淺色畫面選擇，並於專案設定顯示專案目錄

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者要求：（1）在設定中選擇深色或淺色畫面；（2）在專案設定顯示專案目錄。需求已有可驗證的目標，因此省略逐題釐清，先根據現況提出假設；使用者回覆「不需調整」，確認全部建議。
現況：packages/ui/src/theme.css 已有深淺色票，但以 prefers-color-scheme 跟隨系統；AppSettingsView.tsx 已有裝置級介面語言偏好；ProjectSettingsView.tsx 僅顯示設定檔相對路徑；session.ts 的本機 locator 有 root，遠端 locator 有可選 checkoutRoot。
相關正式規格為 desktop-config（兩種設定頁的分工、裝置級介面偏好、專案切換）與 desktop-app（深淺配色、文件及通知呈現）。目前無作用中變更。既有討論已確立共用色票與系統匣配色，此次新增手動選擇的優先順序，沿用既有色彩角色。
Prior discussions: tray-panel-scroll-and-vibrancy, copilot-project-tool, plan-overlap-granularity, discuss-scout-in-flight-deltas, release-assets-trim, improve-layout-signal, improve-repo-layout, improve-workspace-sync, spec-drawer-trace-links, rename-onboard-to-baseline, propose-apply-handoff-updates, node-sdk-completion-and-doc-alignment, capability-naming-dedup, schema-engine-openspec-parity, card-drawer-header-colors, evidence-gate-false-blocks, post-archive-spec-value, slow-global-test-suite, desktop-workspace-auto-init, admin-console-redesign, desktop-feedback-surface, tray-rich-panel, discard-change-verb, desktop-reading-and-tasks-ux, discuss-lifecycle-document-created-on-start-manual-cleanup-when-abandoned, sdd-engine-as-sdk-with-pluggable-document-storage-for-team-scenarios

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-12)

**Focus**: 外觀選擇的選項、作用範圍與保存方式
**Position**: 外觀需求已定案，使用者以「不需調整」接受提出的行為。
- 決策範圍分成三個節點：外觀偏好、本機專案目錄、遠端專案目錄；本輪先記錄外觀，後兩輪記錄同次回覆已確認的目錄行為。
- 應用程式設定的本機設定增加「外觀」，選項為跟隨系統／淺色／深色，預設跟隨系統。
- 選擇立即生效並保存於此裝置，重開後恢復；全 app 的所有專案與系統匣面板共用同一偏好，不寫入專案設定。
- 沿用既有深淺配色。依據：packages/ui/src/theme.css、apps/desktop/src/views/AppSettingsView.tsx；desktop-app 目前以系統偏好描述配色，需擴充為有效外觀偏好，手動選擇優先於系統。
- 既有 desktop-reading-and-tasks-ux、card-drawer-header-colors、tray-panel-scroll-and-vibrancy 的色票及角色裁決沿用；自動跟隨的行為保留為預設選項。
**Ruled out**: 僅跟隨系統而無手動選項——不滿足需求；每個專案各選配色——使用者確認裝置級共用偏好。
**Open**: 待記錄同次確認的本機專案目錄呈現，以及遠端有／無本機副本時的呈現。

### Round 2 — assumptions (2026-10-12)

**Focus**: 本機專案目錄的位置與呈現行為
**Position**: 本機目錄需求已定案，記錄使用者同次「不需調整」的確認；外觀需求已於前輪定案。
- 專案設定頁頂端、各設定頁籤共用的區域顯示「專案目錄」與目前專案的完整根目錄路徑。
- 路徑唯讀、可選取複製，過長換行；切換作用中專案時隨之更新。
- 例：目前專案為 speclink 時顯示 /Users/mosu/project/ai_dev/speclink。
- 依據：apps/desktop/src/views/ProjectSettingsView.tsx 目前只顯示設定檔相對路徑；apps/desktop/src/session.ts 已提供本機 locator.root。desktop-config 已規範設定頁隨專案切換，但未規範完整目錄展示，此次補入新需求。
**Ruled out**: 僅顯示 openspec/config.yaml 等相對路徑——無法辨識專案所在位置；從此欄更換專案目錄——使用者確認此次為唯讀顯示。
**Open**: 待記錄同次確認的遠端有／無本機副本時的呈現。

### Round 3 — assumptions (2026-10-12)

**Focus**: 遠端專案的目錄語意
**Position**: 遠端目錄需求已定案，記錄使用者同次「不需調整」的確認；兩項原始需求均已完整決定。
- 遠端專案有連結本機副本時，顯示該副本完整目錄；沒有本機副本時顯示「未連結本機目錄」。
- 路徑同樣唯讀、可選取複製、過長換行，跟隨作用中專案更新。
- 依據：apps/desktop/src/session.ts 的遠端 locator.checkoutRoot 為可選欄位，已有資料足以區分兩種情境。
**Ruled out**: 顯示 Server 上的實體目錄——使用者確認目標為本機副本位置，且現有 locator 提供的是本機路徑。
**Open**: 無。

## Conclusion

**Decision**: 新增裝置級外觀選擇，並在專案設定顯示作用中專案的本機目錄。
- 應用程式設定的「本機設定」增加「外觀」，提供「跟隨系統／淺色／深色」三選，預設跟隨系統。
- 選擇立即套用整個桌面 app，所有專案與系統匣面板共用；保存於此裝置，重開仍記住，不寫入專案設定。
- 跟隨系統時配合系統深淺變更；手動選淺色或深色時以使用者選擇為準。沿用現有色票與色彩角色，文件、通知與面板均應使用有效外觀偏好。
- 本機專案的專案設定頁頂端顯示「專案目錄」與完整根目錄路徑；顯示位於各設定頁籤共用的區域。
- 路徑唯讀、可選取複製，過長換行；切換作用中專案時立即顯示對應位置。
- 遠端專案有本機副本時顯示該副本完整目錄；沒有時顯示「未連結本機目錄」。
- 驗收例：系統為深色時選擇淺色，桌面介面與系統匣面板立即呈現淺色，切換專案或重開 app 仍維持淺色；改回跟隨系統才恢復深色。
- 驗收例：開啟 speclink 專案時顯示 /Users/mosu/project/ai_dev/speclink；切換另一專案後顯示該專案完整目錄；切換至無本機副本的遠端專案後顯示「未連結本機目錄」。
**Rationale**: 深淺色票與專案位置資料皆已存在，新增偏好入口與清楚的路徑展示即可滿足需求；外觀沿用裝置級介面偏好的分工，目錄沿用作用中專案的 root／checkoutRoot，避免新增重複的設定來源。
**Rejected alternatives**:
- 只跟隨系統——無法讓使用者手動選擇。
- 每個專案各自保存外觀——使用者確認所有專案共用裝置級偏好。
- 只展示設定檔相對路徑——無法辨識專案實際位置。
- 目錄欄提供更換位置操作——此次需求為唯讀展示。
- 顯示 Server 的實體目錄——此次確認的目錄語意為本機專案或本機副本。
**Deferred**: none
**Capture to**: proposal、design、tasks；desktop-config 補入外觀偏好與專案目錄展示，desktop-app 更新目前僅描述系統偏好的配色行為；提案時一併核對 tray-status-menu 的系統匣面板配色敘述。
**Next**: speclink-propose --from-discussion desktop-theme-and-project-directory
