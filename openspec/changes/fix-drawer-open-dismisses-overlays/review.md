# Review — fix-drawer-open-dismisses-overlays

## Round 1

**Phase**: discovery
**Patch**: sha256:b8819d3b3401eff4587788553580e56e4561048ea5e09a463e149872d1f84988
**Scope**: apps/desktop/src/App.tsx, apps/desktop/src/__tests__/App.test.tsx, apps/desktop/src/__tests__/store.test.ts, apps/desktop/src/store.ts
- [WARNING] apps/desktop/src/store.ts — Correctness: 這個 patch 讓 openDetail/openDiscussion 會在 `confirmArchiveDiscardTicket` 的 `await discard(name)` 期間（例如從系統匣觸發）把 `pendingArchiveSettled` 歸零，`pendingArchiveCarry` 卻保留原值。`settleStation` 在 await 之後才從 `get()` 讀這兩個值來合併，會出兩種錯：(1) 雙站流程裡，先處理過的那站旗標會遺失，`openTicketStation` 判定該站還沒處理就提早 return。結果第二站工單已經刪掉且無法復原，使用者確認過的封存卻默默沒執行，也沒有 toast。(2) 如果 `changes` 已刷新，封存照常進行，但 `failKey` 少算已放棄的站，失敗訊息會從「兩站」錯成「單站」。另外，await 期間如果使用者重新 `requestArchive(Y)`，舊流程會把 X 那站寫進 Y 的 settled，還會 `set({pendingArchive:null})` 關掉 Y 的對話框。建議在 await 前先快照 settled/carry 再傳給 `settleStation`，或在 await 之後檢查 `pendingArchive !== name`，再明確決定要繼續還是中止。
- [SUGGESTION] apps/desktop/src/store.ts — Correctness: `cancelableOverlaysCleared` 沒清 `pendingArchiveCarry`，但註解寫的是「站別處置隨封存守門一起還原」。平常靠 `requestArchive` 重設所以沒事，碰上上一條的 race 就會出現 settled 和 carry 不一致。建議把它一起歸零，`apps/desktop/src/__tests__/store.test.ts` 也補上對 carry 的斷言。
- [SUGGESTION] apps/desktop/src/App.tsx — Correctness: 更新日誌 effect 只在抽屜由關轉開的那一刻收掉對話框。whatsNew 要等 `appVersion()` 非同步回來才決定要不要彈出；如果抽屜先開了，之後彈出的 whatsNew 會疊在抽屜上面，而且不會被收掉。觸發的時間窗很窄，但規格裡的互斥在這裡沒有保住。
- [SUGGESTION] apps/desktop/src/store.ts — Standards: possible Data Clumps：cancelableOverlaysCleared 只把 pendingArchiveSettled 歸零，沒有一起歸零 pendingArchiveCarry。檔內既有三處（換 workspace 時的清除、初始 state、requestArchive）都是兩欄成對歸零，這裡把這組欄位拆開了。另外 `{ review: false, verify: false }` 這個字面值又多了一份。
- [SUGGESTION] apps/desktop/src/__tests__/store.test.ts — Standards: possible Duplicated Code：斷言先手動挑出 9 個欄位，再用 toEqual 比對同一份清單，加上 helper 本身，歸零清單一共寫了三份。日後新增可取消浮層時要改三處（Shotgun Surgery）。同檔已經用過 3 次 toMatchObject，改成 `expect(s).toMatchObject({...})` 可以少一份。
- [SUGGESTION] apps/desktop/src/__tests__/store.test.ts — Standards: possible Duplicated Code：withTopicA 的 topic-a discussions fixture，和同檔既有兩個測試裡內嵌的 fixture 內容相同，現在變成第三份。既有慣例本來就是內嵌重複，所以屬低優先。

## Round 2

**Phase**: validation
**Patch**: sha256:5031394536ab0059786f3efa43474ab7c51bad111d24f6dca805df22a36f32ac
**Scope**: apps/desktop/src/App.tsx, apps/desktop/src/__tests__/App.test.tsx, apps/desktop/src/__tests__/store.test.ts, apps/desktop/src/store.ts
