## Context

技能檔提示：`store.ts` 於本地分頁活躍、更新完成與 workspace-changed 時探測，結果存 `assetPrompt: AssetPromptState | null`（`kind: stale | missing | newer`、`fileCount`、`version`），`applyAssetUpdate()` 走引擎再生入口、`dismissAssetPrompt()` 把「專案路徑 → 版號」寫進 `speclink.instructionSkips`；`AssetUpdatePrompt.tsx` 是主區頂部的橫幅（標題、說明、主動作、保留現狀、錯誤原位），`App.tsx` 用 `sticky top-0` 包裹層實作捲動釘選，看板等視圖為它扣高度。版本更新：`core/updater.ts` 純 reducer（idle／checking／available／downloading／restartPending／upToDate／checkFailed／error），`store.ts` 有 `checkForUpdates(manual)`、`acceptUpdate()`、`dismissUpdate()`、`relaunchToUpdate()`；`UpdateBanner.tsx` 在視窗最上方呈現 available／downloading／restartPending／error；`AppSettingsView.tsx` 的「軟體更新」卡有檢查更新、更新日誌、目前版本與 `UpdaterInlineStatus` 行內狀態。

desktop-shell-redesign 落地後：圖示列（`ProjectRail`）與專案欄（`ProjectColumn`）存在，方塊角落已有狀態點機制（紅＝錯誤、琥珀＝remote 離線／需重新登入），`ProjectColumn` 的專案設定項在底部。互動原型第 1 張（看板，專案欄底部提示卡與圖示列更新鈕）、第 9 張（技能檔提示確認框三情境）、第 10 張（設定更新頁的狀態）是視覺基準；第 10 張的四卡結構屬 cut 4，本塊只取「下載與安裝」列的狀態切換。

約束：探測時機、略過記憶、更新狀態機事件集不變；remote 分頁不探測；「徵得同意才下載」「簽章失敗拒裝」「檢查失敗靜默」「前景重檢節流」全部維持。

## Goals / Non-Goals

**Goals:**

- 技能檔提示落在專案欄底部的常駐卡，主區不再為它扣高度或釘選；確認框承載細節與動作。
- 版本更新落在圖示列的狀態鈕與設定頁的軟體更新卡，視窗頂端不再有橫幅；啟動發現新版本以 toast 提醒一次。
- 其他專案的技能檔狀況以方塊角落琥珀點提示。

**Non-Goals:**

- 設定模式「更新」頁與分類欄（cut 4）。
- 真正的下載取消、下載進度百分比（更新外掛的下載事件未接；進度列為不確定狀態）。
- remote stale 橫幅、系統匣面板。

## Decisions

### D1 技能檔提示卡與確認框

- `apps/desktop/src/components/AssetNoticeCard.tsx`：`AssetNoticeCard({ prompt, onOpen })`，`prompt` 為 null 時回 null；渲染一顆 `button`（`w-full rounded-xl border border-status-warning/40 bg-status-warning/10 p-2.5 text-left`），內容為 `AlertCircle`（`text-status-warning`）＋兩行文字（標題 12px 中粗、說明 11px 灰字）＋右端 `ChevronRight`；aria-label＝標題；`data-testid="asset-notice-card"`。文案鍵：`assets.card.staleTitle`／`missingTitle`／`newerTitle`、`assets.card.staleDesc`（含 `{count}`）／`missingDesc`／`newerDesc`。
- `apps/desktop/src/components/AssetNoticeDialog.tsx`：以 cut 1 的 `ConfirmDialog` 承載（兩鍵式，`icon` 槽放琥珀圖示；`ConfirmDialog` 若無 `children` 槽則在本塊加一個 `children?: ReactNode` 放描述下方的內容卡——這是對 ui 原語的最小擴充，形狀：`description` 之後渲染 `children`）；內容卡為 `Card size="nested"` 三列（標籤左、值右等寬）：「這個專案的技能檔版本」＝探測回報的專案標記版號、「你的 Speclink 版本」＝`appVersion`、「會改寫的檔案數」＝`fileCount`（較新態第三列改「會被換回舊內容的檔案數」）。按鈕對照：stale→ cancel「稍後」／confirm「更新技能檔」；missing→「稍後」／「安裝技能檔」；newer→ cancel「保留現狀」／confirm「更新 Speclink」。confirm 走 `applyAssetUpdate()`（newer 則 `onOpenSettingsUpdate()`：切到設定頁並捲到軟體更新卡）；「稍後」只關框；「保留現狀」呼叫 `dismissAssetPrompt()`。`assetUpdating` 時兩鍵 disabled；`assetUpdateError` 顯示於內容卡下方 `text-destructive`，confirm 可重試。
- 探測結果需多帶專案標記版號：`AssetPromptState` 新增 `projectVersion: string | null`（探測回報的專案指令檔標記版號；`missing` 為 null、顯示「—」），由 `store.ts` 自既有探測結果 `AssetProbeResult.tools[].workspaceVersion` 取第一個非 null 值填入（全部 null 即 null）；Rust 端與 Tauri command 不變。
- 落點：`ProjectColumn` 新增 `notice?: ReactNode` 槽，渲染在彈性空白與專案設定項之間（`mb-2`）；`App.tsx` 傳 `<AssetNoticeCard prompt={s.assetPrompt} onOpen={…}/>`，對話框開關為 `App.tsx` 本地 state（與 releaseNotes 同型），`openDetail`／`openDiscussion` 的浮層互斥規則（cut 0）一併清掉它：對話框是可取消浮層。主區的 `sticky` 包裹層與各視圖為提示扣高度的 class 移除。
- 其他專案的狀況：`store.ts` 只對作用中分頁探測（現狀）；本塊把探測結果改為以 locator key 為鍵的 `assetPrompts: Record<string, AssetPromptState | null>`，切換分頁時保留其他分頁上次探測結果，`assetPrompt` 以 selector 取作用中分頁的值（既有呼叫端不改語意）；`ProjectRail` 讀 `assetPrompts[key]` 非 null 時方塊角落亮琥珀點（與 remote 離線點同一顆：任一條件成立即亮，tooltip 附一句「技能檔需要處理」）。背景分頁不主動探測（維持 remote 不探測與「活躍時探測」的時機），所以點是「上次看過時的狀態」；對話框只對作用中專案開。

替代方案「維持頂部橫幅」「只用 toast」在討論中否決；「背景分頁也探測」——會讓每個分頁各開一次引擎探測，與既有「分頁活躍時探測」的時機衝突，不做。

### D2 「有新版本」圖示鈕

- `apps/desktop/src/components/UpdateRailButton.tsx`：`UpdateRailButton({ state, onClick })`，`state.phase` 不在 available／downloading／restartPending／error 時回 null；32×32 `rounded-lg bg-transparent text-muted-foreground hover:bg-foreground/5`；圖示：available＝`Download`＋右上 8px 琥珀點（`bg-status-warning border-2 border-sidebar`）、downloading＝`LoaderCircle animate-spin`（無點）、restartPending＝`RotateCw`＋琥珀點、error＝`AlertTriangle text-destructive`（無點）；tooltip 與 aria-label：`updater.rail.available`「有新版本 {version} — 到設定 › 軟體更新」、`updater.rail.downloading`「正在下載 {version}」、`updater.rail.restartPending`「{version} 已就緒，到設定安裝並重新啟動」、`updater.rail.error`「更新失敗 — 到設定查看」。點擊 `onClick` ＝ `setBoardView("settings")` 並以 `scrollIntoView` 捲到 `data-testid="updater-card"`（`App.tsx` 以 ref 傳給 `AppSettingsView` 的 `focusUpdater` 布林，進頁後捲一次）。
- `ProjectRail` 新增 `updateButton?: ReactNode` 槽（齒輪上方 `mb-1`）；`App.tsx` 傳入。

### D3 軟體更新卡的「下載與安裝」列

- `apps/desktop/src/components/UpdateInstallRow.tsx`：`UpdateInstallRow({ state, onAccept, onCancel, onRelaunch, onRetry })`，依 `state.phase`：available＝左「有新版本 {version}」右 `Button`「下載」（`onAccept`）；downloading＝左「正在下載 {version}」＋不確定進度列（`role="progressbar"` 無 `aria-valuenow`，`h-1.5 rounded-full bg-primary/15` 內一段 `animate-pulse bg-primary`）右 ghost「取消」（`onCancel`＝`dismissUpdate()`；store 的 `acceptUpdate` 在收到下載完成時若狀態已非 downloading 則忽略——本塊在 `acceptUpdate` 內以 phase 檢查實作）；restartPending＝左「更新已就緒」右 `Button`「安裝並重新啟動」（`onRelaunch`）；error＝左錯誤訊息（`text-destructive`）右 outline「重試」（`onRetry`＝`checkForUpdates(true)`）；其他 phase 回 null。文案鍵：`updater.row.download`、`updater.row.cancel`、`updater.row.installRestart`、`updater.row.retry`、`updater.row.ready`。
- `AppSettingsView.tsx`：軟體更新卡 `CardContent` 內、既有按鈕列下方渲染 `UpdateInstallRow`；`AppSettingsUpdaterProps` 加 `onAccept`、`onCancel`、`onRelaunch`；`UpdaterInlineStatus` 在 available／downloading／restartPending／error 時不再重複顯示（改由列承載），其他狀態（checking／upToDate／checkFailed）照舊。
- `App.tsx` 移除 `UpdateBanner`；`UpdateBanner.tsx` 與其測試刪除，案例搬到 `updateInstallRow.test.tsx`。

### D4 啟動 toast

- `store.ts` 的 `checkForUpdates(manual)` 在 `updateFound` 且 `manual === false` 時，若 `notifiedVersion !== version`（執行期變數，模組層 `let`，不持久化）則 `toast(t("updater.toast.available", {version}), { action: { label: t("updater.toast.view"), onClick: () => openSettingsUpdate() }, id: "updater-available" })` 並記 `notifiedVersion = version`；手動檢查不發；同版本第二次背景檢查（前景重檢）不再發。`openSettingsUpdate` 為 store 動作：`setBoardView("settings")` 並設 `focusUpdater: true`（`AppSettingsView` 捲到卡後清回 false）。
- toast 的動作鈕與圖示列鈕是同一個目的地；toast 關閉不影響鈕。

### D5 spec 字面

- 「指令檔過期提示」：呈現段改寫為專案欄提示卡＋確認框（三列內容卡、依狀態的兩鍵）；刪除「提示 SHALL 只佔用自身高度…」整段與欄高 Example、「提示存在時看板欄完整可見」「提示存在時清單頁的換頁控制列可見」「提示存在時設定頁維持整頁捲動」三個 scenario（以 REMOVED-SCENARIO 宣告），其餘探測、略過記憶、動作語意 scenario 字面只改「提示」的位置與開啟方式；新增「提示卡常駐跨頁」「其他專案的琥珀點」scenario。
- 「指令檔過期提示捲動釘選」：改為「提示卡位於專案欄、不屬主區捲動容器，主區任何捲動下 SHALL 維持可見且位置不變」，scenario 改寫。
- 「桌面自動更新」：檢查時機、節流、同意、簽章、靜默條款不動；呈現條款改為「發現新版本時圖示列 SHALL 出現更新鈕（tooltip 帶版號）、背景檢查發現時另發一次 toast、下載與安裝的控制在設定頁軟體更新卡的『下載與安裝』列、視窗頂端 SHALL NOT 出現更新橫幅」；scenario「發現新版徵求同意後套用」改經設定卡「下載」；新增「圖示鈕依狀態」「啟動 toast 只發一次」scenario；Example 表格不動。

## Implementation Contract

**Behavior**

- 本地專案技能檔過期、缺失或較新且未略過時：專案欄底部出現琥珀提示卡（標題、說明、›），所有專案頁面都在、不隨主區捲動、主區不為它扣高度；點卡開確認框（三列資訊、依狀態兩鍵）；更新／安裝成功後卡消失；「保留現狀」後同版不再出現；「稍後」只關框。失敗時錯誤在框內原位、可重試。remote 專案無卡。
- 其他開著的本地專案上次探測有狀況時，其圖示列方塊角落亮琥珀點、tooltip 附「技能檔需要處理」。
- 有新版本時：視窗頂端無橫幅；圖示列齒輪上方出現更新鈕（下載箭頭＋琥珀點、tooltip 帶版號），下載中轉圈、待重啟為重啟箭頭、失敗為紅色警示；點鈕進設定頁並捲到軟體更新卡；卡內「下載與安裝」列依狀態顯示「下載」／進度＋「取消」／「安裝並重新啟動」／錯誤＋「重試」；背景檢查發現新版本時 toast 一次（含「查看」）。
- 既有語意不變：徵得同意才下載、簽章失敗拒裝並顯示錯誤、檢查失敗靜默、手動檢查結果行內呈現、前景重檢節流。

**Interface / data shape**

- `AssetPromptState` 新增 `projectVersion: string | null`；store 新增 `assetPrompts: Record<string, AssetPromptState | null>`（`assetPrompt` 維持為作用中分頁的派生值）、`openSettingsUpdate()`、`focusUpdater: boolean`。
- `ProjectColumn` 新增 `notice?: ReactNode`；`ProjectRail` 新增 `updateButton?: ReactNode` 與方塊 `assetFlag` 判定（`assetPrompts[key] != null`）。
- `AppSettingsUpdaterProps` 新增 `onAccept`、`onCancel`、`onRelaunch`、`focus?: boolean`。
- `ConfirmDialog` 新增選配 `children?: ReactNode`（描述之後、按鈕之前）。
- i18n 新鍵（兩語系）：`assets.card.*`（六個）、`assets.dialog.title.*`（三個）、`assets.dialog.desc.*`（三個）、`assets.dialog.projectVersion`、`assets.dialog.appVersion`、`assets.dialog.fileCount`、`assets.dialog.revertCount`、`assets.dialog.later`、`assets.dialog.updateApp`、`assets.railHint`、`updater.rail.*`（四個）、`updater.row.*`（五個）、`updater.toast.available`、`updater.toast.view`。移除鍵：`assets.update`、`assets.install`、`assets.keep`、`updater.accept`、`updater.later`、`updater.restartNow`、`updater.close` 改名或併入上述鍵，兩語系鍵集合維持相等。
- 持久化：`speclink.instructionSkips` 格式不變；無新鍵。

**Failure modes**

- 更新技能檔失敗：錯誤文字顯示在確認框內容卡下方、兩鍵恢復可按、卡片仍在。
- 下載失敗／簽章失敗：狀態機進 error；圖示鈕變紅色警示、設定卡列顯示錯誤與「重試」；不自動重檢。
- 取消下載：狀態回 idle，鈕與列消失；之後下載完成事件到達時忽略。
- 探測失敗或無法判定：無卡、無點（既有語意）。

**Acceptance criteria**

- `npm test -w apps/desktop` 全綠；新測試 `assetNotice.test.tsx`（卡三態文案、點卡開框、框內三列、依狀態兩鍵文案、稍後只關框、保留現狀呼叫 dismiss、更新呼叫 apply、錯誤原位、busy 兩鍵 disabled）、`updateRailButton.test.tsx`（四態圖示與 tooltip、其他態不渲染、點擊呼叫 onClick）、`updateInstallRow.test.tsx`（四態控制與 callback、其他態不渲染）、`projectRail.test.tsx` 加「assetPrompts 有值時琥珀點」、`App.test.tsx` 改「提示在專案欄而非主區、主區無 sticky 包裹層、無 update-banner」、`store.test.ts` 加「背景檢查發現新版本 toast 一次、同版本不重發、手動不發、取消後下載完成被忽略」、`appSettingsView.test.tsx` 加「下載與安裝列四態」。
- `grep -rn "update-banner\|asset-prompt\"" apps/desktop/src` 零命中（舊 testid 退場）。
- 手動：開一個技能檔舊版的專案看卡與框；以測試更新端點或版本降號看圖示鈕、設定卡列與 toast。

**Scope boundaries**

- In：兩種提示的呈現、入口與落點；`assetPrompts` 多分頁記憶；`ConfirmDialog` 的 `children` 槽；軟體更新卡一列；啟動 toast；spec 三條需求字面；對應測試。
- Out：設定「更新」頁、真正的下載取消與進度百分比、remote stale 橫幅、系統匣、探測時機與略過規則。

## Risks / Trade-offs

- [刪除欄高條款與 scenario 改變既有 fix-board-height-under-banners 的保證] → 提示離開主區後條款失去對象；看板欄仍以 `h-full min-h-0` 填滿主區，`App.test.tsx` 既有「欄底不裁切」案例改為在無提示包裹層下驗證。
- [取消下載只是放棄同意，外掛仍在背景下載] → 下載完成事件被忽略、不會進 restartPending；documented in design；真正取消待外掛支援。
- [背景分頁的琥珀點是舊資訊] → tooltip 文案不含版號只說「需要處理」；切過去會重新探測刷新。
- [toast 與圖示鈕雙重提醒] → toast 只發一次且只在背景檢查；手動檢查無 toast。
- [回歸對照] → 不動 CLI、golden、`--json`；desktop 測試是基準，舊測試案例逐條搬到新檔。
- [跨平台] → 純前端；sonner 與圖示列在三平台一致。

## Migration Plan

無資料遷移；回滾即還原本刀 commit。

## Open Questions

無。
