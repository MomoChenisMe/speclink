# Verify — fix-drawer-open-dismisses-overlays

## Round 1

**Phase**: discovery
**Patch**: sha256:b8819d3b3401eff4587788553580e56e4561048ea5e09a463e149872d1f84988
**Scope**: apps/desktop/src/App.tsx, apps/desktop/src/__tests__/App.test.tsx, apps/desktop/src/__tests__/store.test.ts, apps/desktop/src/store.ts
- [WARNING] apps/desktop/src/__tests__/App.test.tsx — Correctness: Example 表「更新日誌（whatsNew）｜開啟討論」這一列沒有測試用 openDiscussion 當輸入。App.test.tsx:1589 的新案例只透過看板卡片走 openDetail。建議補一個案例：whatsNew 開啟中觸發 openDiscussion，斷言對話框關閉、討論抽屜開啟、KEY 未寫入。
- [SUGGESTION] apps/desktop/src/__tests__/store.test.ts — Correctness: 「無封存發生」「工單維持未結」「change A 未被刪除」「無專案被開啟」這幾個結果，只靠浮層欄位歸零間接成立，測試沒斷言對應的 adapter 未被呼叫。另外「封存確認」這一列只由 settled review:true 的三選項框案例覆蓋，可補一個 settled 為預設值的案例。
- [SUGGESTION] apps/desktop/src/App.tsx — Correctness: App.tsx:332-336 的 effect 只看兩個布林值。抽屜已開著時換開另一個 change 或討論，effect 不會觸發。若 whatsNew 非同步晚於抽屜才彈出，之後再從系統匣開抽屜，更新日誌不會被收掉。這符合 D3「自 null 變非 null」的字面，但 spec Scenario 沒有「抽屜原本關著」這個前提，兩者有落差。
- [SUGGESTION] apps/desktop/src/store.ts — Coherence: proposal 只寫了遷移這個例外，沒提 pendingRemoteConflict。design D4、spec 與實作都包含遠端衝突，建議把 proposal 同步。

## Round 2

**Phase**: validation
**Patch**: sha256:5031394536ab0059786f3efa43474ab7c51bad111d24f6dca805df22a36f32ac
**Scope**: apps/desktop/src/App.tsx, apps/desktop/src/__tests__/App.test.tsx, apps/desktop/src/__tests__/store.test.ts, apps/desktop/src/store.ts
