# SDD 最終實測報告：簡易人事系統（Spectra 2.3.1 vs Speclink）

- 日期：2026-07-03
- 題目：單檔 HTML 簡易人事系統（員工 CRUD／部門管理／請假追蹤／localStorage 持久化／CSV 匯出）
- 方法：twin 沙盒（`hr-sp` 跑 Spectra 2.3.1、`hr-sl` 跑 Speclink release build），從 `init` 與 config 設定開始，
  走完 discuss → propose → apply → ingest → drift → verify → archive 全流程；第二輪在 Speclink 側以
  `spec_locale: tw` 驗證本次所有新調整（中文規格、CJK 弱語言、promote 共同歸檔、BEFORE 註記、bulk archive、
  update sync/prune、init 自動偵測、onboard）。
- 對照原則：brand 正規化（spectra↔speclink 字樣）後逐 byte 比對；已登錄的刻意分歧經由 `cmp.py`
  正規化層明示，不混入未知差異。

## 1. 九階段對照結果

| 階段 | Spectra | Speclink | 對照結果 |
| --- | --- | --- | --- |
| init + config | `--tools claude` 明示 | 無參數自動偵測（`.claude/` → claude），並寫回 `tools: [claude]` | 產物樹等值；CLAUDE.md 區塊 v1.0.2 vs v1.1.0（刻意）；config 模板差異全屬移除功能與 `spec_locale` 新增 |
| discuss | 純 skill，無 CLI 產物，結論只活在對話 | `discuss new/context/add-round/conclude`，3 輪蘇格拉底問答落地成文件 | Speclink 獨有；文件含固定骨架與 document rules |
| propose | `new change` + 4 artifacts | 同左 | `instructions` payload 四個 artifact JSON 全 PASS |
| validate/analyze | `--strict` 通過；analyzer 檢出 design D1 未被 tasks 引用 | 同左 | 修正 tasks 引用後兩邊同步歸零；**本輪修復 conDesignNotInTasks 訊息格式**（見 §3） |
| apply | 8 tasks done；touched 歸因 task 1（2 檔） | 同左 | `task done` 輸出與 touched JSON 逐 byte 相同；**本輪修復 preflight missingFiles**（見 §3） |
| ingest | csv-export capability 中途加入 | 同左 | validate/touched 增量歸因一致；tests 17→20 全綠 |
| drift | LIGHT(0) → 建議 `apply` | HEAVY(7) → 建議 `ingest` | **刻意分歧的核心展示**（見 §4） |
| verify | skill（fork）；tests 20/20 + validate + analyze | 同左 | `instructions --skill verify` 輸出等值；SKILL.md **本輪補上 fork-context 段後全等** |
| archive | `added: 8`、canonical 5 caps、snapshot | 同左 + `Discussion archived`（第二輪） | canonical specs、archive dir、created_specs.json **遞迴逐 byte 相同**；歸檔後 `list` 同為空 |

## 2. 功能品質（QA）

- `hr/index.html` 單檔實作：員工 CRUD（必填驗證、級聯刪除）、部門（唯一性、成員保護）、
  請假（含首尾天數、範圍驗證、特休 14 天年額）、versioned localStorage（損毀回退）、CSV 匯出（引號跳脫）。
- `hr/tests.js` mocked-DOM 測試：主輪 17、ingest 後 20、第二輪（降冪排序＋部門人數）21 — **全數通過**，
  覆蓋四份（後為六份）delta spec 的每一個 scenario。

## 3. 本輪抓到並修復的 parity 缺口（Speclink 修正，parity suite 31/31 維持全綠）

1. **analyzer `conDesignNotInTasks`**：Spectra 將 design topic **全小寫**後做大小寫不敏感比對，
   訊息為 `Design topic '<小寫>' not referenced in tasks`、參數鍵 `keyword`、recommendation 為固定文案且無參數。
   Speclink 原本保留原大小寫、大小寫敏感比對、自創文案 — 已全面對齊（probe 驗證含大小寫變體）。
2. **preflight `missingFiles`**：Spectra 從 proposal 第一個含 "affected code"（不分大小寫）的行掃到下一個
   `## ` 標題，取行內反引號引用，過濾（需含 `/`、不得以 `/` 結尾、副檔名 ∈
   {md, html, js, ts, tsx, jsx, css, json, yaml, rs, toml, svelte}，大小寫敏感），保序去重後檢查存在性，
   缺檔以 `{path, referencedIn: "proposal"}` 列出且 status 轉 `critical`。Speclink 原為恆空 stub — 已實作，
   五組 fixture 交叉驗證 PASS。附帶發現：`staleness` 在 `created` 缺漏或無法解析時**整段省略**（已對齊）；
   `driftedFiles` 在純 CLI 環境經 dirty／backdate／commit 時序／touched 各種攻擊皆不觸發，判定為桌面 app
   資料驅動（與 `.started`、13 表 db 同類），CLI 恆空即為正確 parity。
3. **SKILL.md fork-context 段**：Spectra 的 fork 型 skill（analyze/verify/drift）在 frontmatter 後有
   「## Claude fork context」自動選擇規則段，Speclink 漏植 — 已補（analyze/verify 現與 Spectra 全等）。
   同時修正所有產出 skill 檔尾多一空行的問題。
4. **audit 定位**：Spectra 的 audit 是 fork 精簡版；Speclink 的 audit 是刻意重寫的 Two Modes
   （standalone 三代理平行 + apply 內嵌 discipline），與 fork（Explore 無 Agent 工具）矛盾 —
   改為非 fork skill，`disallowedTools: [Edit, Write]` 保留。
5. Speclink 品質修正：BEFORE 註記緊貼 requirement header 時剝除後保留 header 分隔空行；
   `update` prune 後清除空的 `.agents/skills/`、`.agents/` 目錄。

## 4. 刻意分歧實測（同一情境、兩種結果）

**Drift**（外部 hotfix commit＋未完成 task＋canonical spec 碰撞同時存在）：

| | Spectra | Speclink |
| --- | --- | --- |
| Environment | `0 commits`（bare-date approxidate 吃掉當日 commits） | `8 commits (3 touching this change's files)`（midnight 錨定＋關聯計數） |
| Structure | `0/17 broken`（corpus 含已 commit 的 design.md → 永久 self-hit，維度失效） | `8/17 broken`（排除 change 目錄；散文大寫詞 false positive 浮現，見建議） |
| Specs | 無此維度 | `1 stale assumptions`：ADDED 'Roster CSV export' 已存在 —「archive would skip it」 |
| 結論 | **LIGHT → 建議 `/spectra-apply`**（直奔 silent skip 事故） | **HEAVY → 建議 `/speclink-ingest`**（正確攔截） |

**碰撞歸檔實證**：帶著未解決碰撞直接 archive，兩邊輸出**逐 byte 相同**（`added: 7`），
第 8 條 requirement 被無聲跳過、delta 的三個 scenario 靜默丟失，訊息隻字未提 — merge 行為 parity 成立，
而 Speclink 的 Specs 維度正是在事前把這件事變成 HEAVY drift。

**Discuss／promote**：討論文件 conclude 後 `promote` 自動建 change（proposal 預填結論、frontmatter 記
`status: promoted` + `promoted_to`）；歸檔該 change 時討論**自動共同歸檔**（`Discussion archived: … →
discussions/archive/2026-07-03-improve-hr.md`）。未 promote 的討論（hr-system）以 `discuss archive` 獨立收檔。
Spectra 的討論無任何持久痕跡。

**spec_locale: tw ＋ CJK 弱語言**：`instructions specs` 動態附上「以繁體中文寫規格、結構標記與
SHALL/MUST 維持英文」及 CJK 弱詞警示（僅 tw/zh* 附）。故意寫入弱詞的規格被 analyzer 檢出
應該/也許/考慮/待定（行號正確），「不可能發生」未誤報「可能」；中文 MODIFIED requirement 歸檔後正確併入
英文結構的 canonical spec，BEFORE 註記剝除乾淨。

**Bulk archive**：dirty tree 拒絕並列出會汙染 @trace 的檔案；clean 後 `--all` 依 created 順序歸檔
2 個 ready、skip 1 個 tasks 未完（附原因）＋ `Bulk archive: 2 archived, 1 skipped` 總結；
Spectra 的 archive 接多參數直接被 CLI 拒絕。

**update sync/prune**：`tools` 加 codex → 生成 `.agents/skills/speclink-*`（不含 fork-only 的
analyze/verify）＋ AGENTS.md v1.1.0；移除 codex → prune 提示、AGENTS.md 與空目錄全部收乾淨。

## 5. 順帶記錄的 Spectra 自身缺陷（Speclink 不複製）

- drift 的 bare-date `--since` 使同日 commits 隨時刻消失（Environment 恆 0 的假象）
- drift corpus 含 change 自身 → Structure 維度對已 commit 的 design 永久靜音
- 碰撞歸檔無任何警示（上述 silent skip）
- commit skill 的示例路徑 `docs/specs/archived/<name>` 與實際結構 `openspec/changes/archive/<name>` 不符
- preflight `driftedFiles` 在 CLI 世界是永不觸發的死欄位

## 6. 評分（滿分 10）

| 面向 | Spectra 2.3.1 | Speclink | 備註 |
| --- | --- | --- | --- |
| CLI 輸出 parity（以 Spectra 為基準） | — | 9.8 | 本輪 4 缺口修畢；殘餘差異全屬登錄分歧 |
| SDD 流程完整度 | 8.0 | 9.0 | Speclink 補 discuss 落地、promote、onboard、verify 前置 |
| drift 可信度 | 5.5 | 8.5 | Spectra 三個維度失真；Speclink 修正並新增 Specs 維度（散文 anchor 噪音扣分） |
| 歸檔安全 | 7.0 | 8.5 | merge 行為相同，但 Speclink 有事前攔截與 bulk 三語義 |
| 規格語言治理 | 6.5（英文硬規） | 9.0 | spec_locale＋CJK 弱語言閉環 |
| 工具鏈生成／同步 | 7.5 | 9.0 | 自動偵測、config 驅動 sync/prune、無足跡移除 |
| **總評** | **7.4** | **8.9** | |

## 7. 建議（後續）

1. **drift 散文 anchor 降噪**：把 Symbol anchor 擷取收斂到反引號 span 與 code-like token
   （camelCase／snake_case／路徑），排除句首大寫散文詞 — 本輪 8/17 broken 中 8 個全是散文詞。
2. `analyze` 的 scenario Example 建議（SUGGEST 級）在中文情境下對「數字即具體」的判定已生效，
   維持現狀即可。
3. 下輪可測：`schema fork` 自訂 workflow 在雙工具下的 parity、`demo` 指令對照。
