## Context

- 四個詳情頁（變更、討論、規格、已封存）的左欄「狀態」列都由 `packages/ui` 的 `DetailStatus`（容器，`DetailShell.tsx`）包住多個 `StatusRow`（列，`DetailHeader.tsx`）。`StatusRow` 的標籤欄寫死 `w-14`，容器是 `flex-col divide-y`。
- 兩個 app 的介面文字都經 `packages/ui` 的 `I18nProvider`：內建字典 `MESSAGES` 與 app 字典合併後提供 `t(key)`。數量由呼叫端以 `.replace("{n}", …)` 代入。
- 三份字典各有一條「中英鍵集合相等」測試（`packages/ui`、`apps/desktop`、`apps/server-web`）。
- Tailwind 為 4.3.2，`grid-cols-subgrid` 可用，`apps/desktop/src/views/project-settings/parts.tsx` 已在用。WKWebView、WebView2 與 WebKitGTK 都支援 subgrid。
- 不動 crate；本變更只落在前端。

## Goals / Non-Goals

**Goals:**

- 英文介面的狀態列標籤不與數值重疊，數值欄在同一頁內左緣對齊。
- 英文計數在數量為 1 時用單數形。
- 用守門測試擋住新的「只寫複數形」計數字串。

**Non-Goals:**

- 不支援英文以外的複數規則，不引入 ICU MessageFormat 或執行期 i18n 套件。
- 不改序數與分數字串（Wave {n}、Round {n}、{n}/{m}）。
- 不改繁中文案，也不改狀態列的內容與順序。
- 不重拍文件截圖（歸 docs-tray-and-mod-screenshots）。

## Decisions

### D1 狀態列以 subgrid 共用標籤欄

`DetailStatus` 改為 `grid grid-cols-[max-content_minmax(0,1fr)]`，保留 `divide-y` 的列間細線。`StatusRow` 改為 `col-span-2 grid grid-cols-subgrid`，標籤 `span` 從 `w-14` 改為 `min-w-14 max-w-28`，並加 `break-words`。結果是：

- 標籤欄寬度＝該頁最長標籤的寬度，下限 56px、上限 112px。
- 繁中標籤都不超過 56px，欄寬維持 56px，版面與現在相同。
- 英文的「Requirements」約 75px，欄寬變 75px，各列數值左緣對齊。
- 超過 112px 的標籤在字間折行，不溢出。

`StatusRow` 只出現在 `DetailStatus` 之內（四個詳情頁各一處），所以 subgrid 永遠有格線父層。

替代方案：
- 標籤欄改成固定 80px。被否決：繁中版面會多出空白，未來更長的英文標籤仍會溢出。
- 標籤寬度依各列內容（`w-max`）。被否決：各列數值的左緣會不對齊。

### D2 單複數由 `tn(key, n)` 與 `.one` 鍵處理

`I18nContextValue` 新增 `tn: (key: string, n: number) => string`，這是唯一的實作點。規則：

- `n === 1`，且合併後字典有 `<key>.one` → 回傳 `<key>.one`。
- 否則回傳 `<key>`。
- 兩種情況都把 `{n}` 換成 `String(n)`。其他佔位符（`{cap}`、`{mode}`、`{key}`）仍由呼叫端代入。

`.one` 只寫在英文字典。繁中沒有單複數之分，不寫 `.one`，`tn` 自然回傳原鍵。Provider 外的預設 context 也提供同樣的 `tn`，只查繁中字典。

替代方案：
- 在同一個字串裡用 `|` 分隔單複數。被否決：還沒改的呼叫端用 `t()` 會直接把 `|` 顯示在畫面上。
- 改用 `Intl.PluralRules`。被否決：英文只需要「1 或非 1」，不值得多一層。

### D3 呼叫端逐一改用 `tn`

本變更範圍內的計數訊息共 34 個：`packages/ui` 內建字典 31 個、server 後台 app 字典 3 個（完整清單見 proposal 的 Problem 與下方 Implementation Contract）。它們的每一個呼叫端，都從 `t(key).replace("{n}", …)` 改成 `tn(key, n)`。如果呼叫端還要代入其他佔位符，就在 `tn` 的結果上接著 `.replace`。

英文單數形照文法寫，包括動詞一致：

- 「1 user has not activated yet.」
- 「1 requirement has no source change」
- 「The last round still lists 1 open item」

### D4 守門測試與鍵集合測試

- **新增守門測試**：放在 `packages/ui/src/__tests__/i18n.test.tsx`，`apps/server-web` 的字典放在它自己的 i18n 測試。測試掃描英文字典，凡是值符合「`{n}` 之後、隔零到兩個單字、接一個以 s 結尾的單字」的訊息，都要有 `<key>.one`。失敗訊息點名缺的鍵。
- **調整三份鍵集合相等測試**：比對前先排除以 `.one` 結尾的鍵。另外斷言每個 `.one` 鍵在同一語言都有基礎鍵，而且 `.one` 只出現在英文。

### D5 狀態列結構以 DOM 結構測試

jsdom 量不到版面，所以 `detailStatusLayout.test.tsx` 檢查結構：

- `DetailStatus` 的根元素帶 `grid-cols-[max-content_minmax(0,1fr)]`。
- 每個 `StatusRow` 帶 `grid-cols-subgrid` 與 `col-span-2`。
- 標籤帶 `min-w-14` 與 `max-w-28`，不再有 `w-14`。

畫面上是否真的不重疊，由使用者在英文介面實機確認（[M] 任務）。docs-tray-and-mod-screenshots 重拍時也會再看一次。

## Implementation Contract

**行為：**

- 英文介面任一詳情頁的狀態列，標籤與數值不重疊，同一頁的數值欄左緣對齊。繁中介面的標籤欄寬度維持 56px。
- 英文計數在 n＝1 時為單數，n＝0 或 n≥2 時為複數。繁中不變。

**介面：**

- `useI18n()` 回傳 `{ locale, t, tn }`。`tn(key, n)` 的回傳值已代入 `{n}`。
- 字典鍵 `<key>.one` 只存在英文。

**34 個訊息鍵：**

- `packages/ui`：common.tasksCount、common.rounds、kanban.hitCount、plan.calloutBlocked、plan.calloutArchiveWait、ticket.scopeFiles、ticket.findings、tasks.manualCallout、archived.specCount、archived.promotedCount、ddetail.priorDiscussions、conclusion.cutChanges、conclusion.tileCuts、conclusion.afterRounds、rounds.modeRounds、rounds.convergedOpenLeft、promoted.groupChanges、doc.capabilityRewritten、doc.points、doc.moreParagraphs、doc.moreScenarios、doc.scenarioCount、doc.removedScenarios、doc.removedScenariosChip、time.daysAgo、sdetail.metaRequirements、sdetail.metaScenarios、sdetail.untraced、specs.requirementCount、specs.traceCount、manual.uncoveredNew
- `apps/server-web`：overview.metricProjectsHint、overview.todoPendingInvitations、registry.projectMeta

**失敗模式：**

- 鍵不存在時，`tn` 和 `t` 一樣回傳鍵名本身（可見的失敗）。
- 守門測試會點名缺 `.one` 的鍵。

**驗收：**

- `npm test -w packages/ui`、`npm test -w apps/desktop`、`npm test -w apps/server-web` 全綠。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全綠。
- [M] 英文介面實機確認。

**範圍：**

- 範圍內：上述字典、`tn`、34 個訊息的呼叫端、狀態列兩個元件、三份字典測試。
- 範圍外：截圖、繁中文案、序數字串、crate 與 CLI。

## Risks / Trade-offs

- [漏改呼叫端，英文仍顯示複數] → 用 `grep` 對 34 個鍵逐一列出呼叫端，改完再搜一次 `.replace("{n}"` 確認只剩非計數鍵。守門測試只管字典，呼叫端靠這次排查。
- [subgrid 在舊版 WebKitGTK 不支援] → Linux AppImage 內建的 WebKitGTK 版本支援 subgrid，`project-settings` 已在三平台使用同一寫法，風險與既有程式相同。
- [Windows／macOS／Linux 字型寬度不同] → 標籤欄依實際內容寬度計算，不依賴固定像素值。上限 112px 留有餘裕。
- [回歸對照] → 不動 CLI 與 golden。前端三個 workspace 的既有測試是回歸面。
