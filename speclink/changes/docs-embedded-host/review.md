# Review — docs-embedded-host

## Round 1

**Phase**: discovery
**Patch**: sha256:f91353b988ddec8d98baf478bf9d3bbeba1adaad9323cca5a54ae4c9b431da65
**Rules**: extend
**Rule**: CRITICAL 呼叫端不得以 className 覆蓋 @speclink/ui 元件變體的顏色、圓角與陰影，需要的外觀改由變體提供
**Rule**: WARNING 全系統共通的行為（例如返回導覽、快捷鍵、提示框）要在共用層實作一次，各頁面只接上共用實作；新增的頁面要接上既有的共用實作，不要在頁面裡各自重寫
**Rule**: WARNING 新增功能時，repo 已有相同用途的元件、編輯器或工具函式（例如 @speclink/ui 的元件、設定頁共用的程式碼編輯器）要直接沿用，不要另做一套平行實作
**Rule**: WARNING 修正 bug 的變更：同一種寫法或同一類元件在 repo 其他位置若有同樣的問題，要在這次一併修正，或在 tasks.md 寫明不修的理由
**Rule**: WARNING 同一種操作在不同頁面要用同一個元件與同一種變體呈現（例如按鈕、選單、分頁工具、空狀態），不要同一種操作在各頁長得不一樣
**Scope**: README.md, README.zh-TW.md, docs/assets/figures/en/embedded-shape.svg, docs/assets/figures/en/sdk-contents.svg, docs/assets/figures/en/shared-host-kit.svg, docs/assets/figures/en/skill-path-vs-verb.svg, docs/assets/figures/en/verb-four-steps.svg, docs/assets/figures/zh-TW/embedded-shape.svg, docs/assets/figures/zh-TW/sdk-contents.svg, docs/assets/figures/zh-TW/shared-host-kit.svg, docs/assets/figures/zh-TW/skill-path-vs-verb.svg, docs/assets/figures/zh-TW/verb-four-steps.svg, docs/assets/screenshots/en/server-setup.png, docs/assets/screenshots/zh-TW/server-setup.png, docs/configuration.md, docs/configuration.zh-TW.md, docs/development.md, docs/development.zh-TW.md, docs/getting-started.md, docs/getting-started.zh-TW.md, docs/product-status.md, docs/product-status.zh-TW.md, docs/remote-getting-started.md, docs/remote-getting-started.zh-TW.md, docs/roadmap.md, docs/roadmap.zh-TW.md, docs/sdk-node.md, docs/sdk-node.zh-TW.md, docs/server-backup.md, docs/server-backup.zh-TW.md, docs/server-deployment.md, docs/server-deployment.zh-TW.md, docs/server-store-drivers.md, docs/server-store-drivers.zh-TW.md, docs/verb-contract.md, docs/verb-contract.zh-TW.md, docs/workflow.md, docs/workflow.zh-TW.md, scripts/docs/docs-figures.test.mjs, scripts/docs/docs-parity.test.mjs, scripts/docs/remote-docs.test.mjs
- [SUGGESTION] scripts/docs/docs-figures.test.mjs — Standards: possible Duplicated Code。檔尾的 `docLocale`、`referenced` Set 與 `orphans` 那一段（「每張圖在每個語言都被同語言的…文件引用」），和 scripts/docs/docs-links.test.mjs 的截圖版幾乎逐字相同，只差清單與資料夾。可把「依語言收集被引用的路徑」抽到 docs-links.mjs，兩邊共用。
- [SUGGESTION] scripts/docs/docs-parity.test.mjs — Standards: possible Duplicated Code。`figures`／`outsideFigureDir` 與新增的圖解測試，和 `shots`／`outsideDir` 及截圖測試同形，只差資料夾名與副檔名（註解自己也寫「圖解和截圖同一套規則」）。另外 README 測試的 `!anchors.includes('self-hosting')`，已被 `RETIRED_PHRASES` 的 `/self-hosting/i` 對同一對 README 檢查過一次。
- [SUGGESTION] scripts/docs/remote-docs.test.mjs — Standards: possible Mysterious Name（命名不一致）。`localeOf` 與另外兩個檔的 `docLocale` 是同一個判斷，名字不同，是第三份複本。
- [SUGGESTION] docs/remote-getting-started.zh-TW.md — Standards: 新寫的「schema 由 Server 上的流程設定決定」在散文裸用 schema；LANGUAGE.md「產出流程」的 avoid 列了「schema（使用者可見文案中）」。同一批新段落的「流程設定」出現 3 次，其他文件一律寫「工作流設定」，不合「一個概念一個詞」。純英文詞不在機械守門內，verb-contract 也有同句的存量，所以只報建議。
- [SUGGESTION] docs/getting-started.zh-TW.md — Standards: 改寫後的句子「只適用 Local 模式，不適用遠端模式」（README.zh-TW.md 有同一句）；LANGUAGE.md「本機專案」「遠端專案」兩條都寫「新寫的段落用這個詞」。不是 avoid 詞。
- [SUGGESTION] README.zh-TW.md — Standards: 「內嵌」一節同一段裡「桌面 app」與「Speclink Desktop」並用；「你的路由自己驗 token」在 verb-contract.zh-TW.md 寫成「驗憑證」。LANGUAGE.md 原則「一個概念一個詞」。
- [SUGGESTION] docs/sdk-node.zh-TW.md — Standards: possible Duplicated Code。約 40 行的 `memoryStore` 範例在「實作三個儲存函式」與「完整範例」各貼一次，中英共四份，沒有守門讓它們和 `__test__/memory-port.ts` 保持一致，之後契約一改要手動改四處。
- [WARNING] scripts/docs/docs-figures.test.mjs — Correctness: 外部 `url()` 檢查 `/url\(\s*['"]?(?!#)/i` 會因正規式回溯誤判。合法的內部參照 `url("#arrow")`、`url('#arrow')`、`url( #arrow)` 都被報成「含外部的 url()」（已用 node 實測三者皆為 true），與註解「只擋不是 `#id` 的」不符，屬於為錯的理由而失敗。目前十張圖都寫成不帶引號的 `url(#arrow)`，所以還沒踩到。
- [WARNING] scripts/docs/docs-parity.test.mjs — Correctness: `RETIRED_PHRASES` 的 `/remote store\b/i` 多了 `\b`。`remote storage` 本來就不會命中，這個 `\b` 唯一的效果是放過複數「Remote Stores」（實測為 false），守門會靜默放行它要擋的詞。
- [WARNING] docs/sdk-node.md — Correctness: Express 範例 `app.all('/api/speclink/v1/*', …)` 在 Express 5（npm 目前預設安裝的版本）註冊路由時就會丟 `Missing parameter name`，要寫成 `/api/speclink/v1/*splat`。這條是依 Express 5 的已知行為判斷，沒有實跑。文件沒註明只適用 Express 4；`docs/sdk-node.zh-TW.md` 是同一段程式。
- [SUGGESTION] docs/verb-contract.md — Correctness: 「Four subcommands … `drift`, `review prepare`, `review scope`, `verify scope`」接著說沒有 checkout 的宿主會拒絕它們。`docs/sdk-node.md` 與 `docs/product-status.md` 列的被拒清單是五個，多一個 `instructions --skill`。讀者只看這頁會以為只有四個，中文版相同。
- [SUGGESTION] scripts/docs/docs-figures.test.mjs — Correctness: 三個可補強的漏洞。`href` 檢查與 `attr()` 只認雙引號，`href='other.svg'` 會通過（實測）。背景矩形只看 `fill`，`fill-opacity="0"` 或 `opacity="0"` 的透明背景會通過。`viewBox` 起點不是 `0 0` 時，仍把 x、y 為 0 的矩形當成鋪滿。

## Round 2

**Phase**: validation
**Patch**: sha256:709c65d0ac412895ad62b54b88bea0d830c40717ffb0f5163236a4d8ca9088a2
**Scope**: README.md, README.zh-TW.md, docs/getting-started.md, docs/getting-started.zh-TW.md, docs/product-status.md, docs/product-status.zh-TW.md, docs/remote-getting-started.zh-TW.md, docs/sdk-node.md, docs/sdk-node.zh-TW.md, docs/verb-contract.md, docs/verb-contract.zh-TW.md, scripts/docs/docs-figures.test.mjs, scripts/docs/docs-links.mjs, scripts/docs/docs-links.test.mjs, scripts/docs/docs-parity.test.mjs, scripts/docs/docs-surface.test.mjs, scripts/docs/remote-docs.test.mjs
- [WARNING] scripts/docs/docs-figures.test.mjs — Correctness: 新的 `url()` 正規式 `/url\(\s*(["']?)([^"')]*)\1\s*\)/gi` 遇到解析不了的寫法會整個不匹配，`urls` 是空陣列，`every` 回 true，等於靜默放行。實測 `url("a)b.svg")`、`url("it's.svg")`、`url("data:image/svg+xml,<svg xmlns='…'/>")` 三者都是 0 筆匹配、不報問題；舊正規式三者都會點名。內容帶 `)` 或另一種引號的外部 url（data URI 很常見）現在會漏掉。
- [SUGGESTION] scripts/docs/docs-figures.test.mjs — Correctness: 新的 `attr` 與 `href` 用 `(.*?)`，不跨換行；舊的 `[^"]*` 會跨。實測 `href="\no.svg"` 現在 0 筆匹配而放行（舊的會點名）；折行的 `aria-label="a\nb"` 現在讀成 undefined，會被誤報成「缺 aria-label」。
- [SUGGESTION] scripts/docs/docs-figures.test.mjs — Correctness: `opaque` 用 `Number(value) > 0`，百分比寫法 `opacity="50%"` 會變 NaN，被當成透明而報「不是鋪滿的背景矩形」，與測試名稱「全透明的不算背景」不符。

## Round 3

**Phase**: validation
**Patch**: sha256:4b5a3cdc48670b470ded4a5b570bd9b2b37fc749c4f4e15697c9d2abeb1fb85f
**Scope**: scripts/docs/docs-figures.test.mjs
- [SUGGESTION] scripts/docs/docs-figures.test.mjs — Correctness: `INTERNAL_URL` 在引號內不容許空白，`url(" #a ")` 現在報「含外部的 url()」，上一輪（先取值再 `trim()`）判成內部。`INTERNAL_HREF` 在引號後有 `\s*`，兩條規則不對稱。手寫或工具產生的圖幾乎不會這樣寫，屬冷門輸入。
- [SUGGESTION] scripts/docs/docs-figures.test.mjs — Correctness: `href` 的總數改用 `/\b(?:xlink:)?href\s*=/`，不再要求後面接引號，所以文字內容或註解裡出現 `href=` 字樣（例如 `<text>href= x</text>`）會報「含外部的 href」，上一輪不報。只有圖上的標籤文字剛好在示範 HTML 屬性時才會碰到，屬冷門輸入。
