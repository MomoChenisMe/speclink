# 完整品質重跑結果 — 2026-10-22

結論：目前不可蓋乾淨章。Review 有 4 WARNING，Verify 有 2 WARNING（其中兩項與 Review 重疊），共 4 個不同的必修問題。沒有接受或跳過任何 finding；本輪不修新 findings、不蓋章、不提交、不合併、不打包、不封存。

## 已完成第 1 步

技能原稿格式守門已修正，既有上限未放寬；資產版本 v1.50.5、golden、assets.lock、生成技能與 Node binding 已同步。詳見 [技能格式修正記錄](quality-full-recheck-2026-10-21.md)。新增 task 18.1 已經 task done 記錄；原 58 個完成任務逐字保留。現在 59/60 任務完成，59/59 程式任務完成，僅人工 12.3 開放。

本步實際通過：CLI build；Core 1081 單元＋94 整合（1 ignored）；其中 render_golden 63、skill_authoring 11；CLI init 27、Remote verb parity 49；Node build 後 57；scripts 386；validate；git diff --check。中途的格式／共用字句及資產鎖失敗均已修正後轉綠，沒有隱藏失敗。

## 第 2 步的備份與範圍

- 原 review.md / verify.md 完整備份到 quality-before-full-recheck-2026-10-21，manifest 記錄位元數與 SHA-256，備份內容一致後才依 CLI discard 清除活動工單。備份 SHA：review 00cf6bce1f1c71d26f3c0a24228160de02cc40c027954c226dfc600c145edb1b；verify 366bb0fcd97573b070346d6f9920776c4a30a2d81000ed77439d311ff6da0a8b。
- trusted base：faefda7dabde01b8d90974aea63f4f7a8bdd5c24；HEAD f5f5179c805590e94fdf08083905874a6bb0507b 加上未提交變更。
- 兩站 discovery 使用同一 frozen patch：sha256:4c0dd687fd4463f80da883a23d6c0935aa76ead30597c2b32dd4062019e4b882，共 228 paths。不是沿用只涵蓋原 13 項修正的舊 Round 2。
- 10/21 代理因額度中斷；10/22 恢復後，逐檔 SHA-256 核對 228 份 afterHash，全部一致，再接續同一輪，不新增空白輪次。記錄工單前後再次核對來源未動。
- 依 speclink-quality 以三個唯讀代理檢查 Standards、Correctness、Verify，兩站已依 CLI add-round 寫入新 Round 1，順序為 review 然後 verify，且 show JSON 已回讀 4／2 findings。

## Review 規則與結果

審查規則：extend（12 項預設建議清單）＋專案規則 CRITICAL 1、WARNING 4、SUGGESTION 0。規則原文已隨 discovery 記入 review 工單。

### Standards

0 CRITICAL／0 WARNING／0 SUGGESTION。完成原九項、原 13 修正、Antigravity／標籤、技能格式與跨層共用接線檢查；四工具標籤、歷史紀錄、十二條說明、CLI Local／Remote 查詢輸出沿既有共用實作，UI 元件未見違反外觀覆蓋規則。無未完成的 Standards 必查項；生成副本只按 brief 核對路徑與生成證據。

### Correctness（代理 finding 原文）

- [WARNING] apps/desktop/core/src/updater.rs — Correctness: `select_desktop_release` 對兩個來源一律執行 `tag.strip_prefix("desktop-v")`；原作者正式桌面版 `v0.10.0`、`v0.9.0` 因此前綴不符而全部略過。使用者點「檢查原作者版本」後，即使網路正常、Release 含安裝包，仍回傳空結果，顯示「未取得版本」。
- [WARNING] apps/desktop/src/adapter/updater.ts — Correctness: 新增備援以 `compareVersions(release.version, installed) <= 0` 排除舊版，但比較器使用 `split(".").map(Number)`；實際安裝版 `0.10.0-build.3` 產生 NaN。候選 `0.10.0.2` 與它比較亦回 NaN，條件為 false，導致較舊版本仍顯示手動更新。主線已用實際比較器重現。
- [WARNING] crates/host/speclink-host/src/security/deep.rs — Correctness: `build_submission_context` 對 `manifest.files` 每項呼叫 `reader.read_optional_text` 再雜湊文字；正常下載 CWE 會保留 `raw/cwec_v4.20.xml.zip`，`expand_cwe` 僅新增 XML，`knowledge::prepare` 亦將 ZIP 寫入知識庫清單。底層讀取器拒絕 NUL／非 UTF-8，因此選用下載的 CWE 後，準備成功仍無法建立深度審查提交內容；Remote checkout 共用此路徑。現有 XML-only fixture 未涵蓋。
- [WARNING] crates/engine/speclink-core/src/quality/station.rs — Correctness: `stamp_gate` 將缺少 metadata 轉成空字串，但新交易預期固定為 `Some(sha256(gate.raw_meta.as_bytes()))`。既有模型與提案探索明確接受沒有 `.openspec.yaml` 的提案；此時 FS 實際值為 None，clean review／verify 蓋章一律得到 `revision_conflict`。應保留原始缺檔狀態作比較。

主線獨立載入實際 whatsNew.ts 比較器重現：0.10.0.2、0.10.0.3、0.10.0.4、0.10.1 對 installed=0.10.0-build.3 全回 NaN，<=0 全為 false。CWE 的 ZIP→manifest→文字讀取不相容與缺檔 metadata 的 None→Some(empty hash) CAS 不相等亦由主線核對直接程式路徑。

## Verify 結果

Completeness：59/59 程式任務完成，46/46 Requirements 有實作落點。Correctness／Coherence：全部 133 Scenarios、7 Examples 已完成條件→實作／必要直接測試映射；確認兩條執行路徑偏差如下，不代表全部情境都重新執行。

- [WARNING] apps/desktop/core/src/updater.rs — Correctness: select_desktop_release 對本分支與原作者一律只接受 desktop-v 前綴及三至四段純數字版本，因此會略過原作者已發布的桌面 v0.10.0／v0.9.0，以及本分支較舊的正式桌面 desktop-0.8.0-build.1；即使 GitHub metadata 查詢成功、不是 draft／prerelease 且附桌面安裝檔，更新頁仍顯示未取得版本。這不符合「本分支更新與原作者版本」要求取得各固定專案的已發布桌面版本。請按兩個專案各自實際桌面 tag 格式解析，保留安裝資產、來源、draft／prerelease 與 CLI-only 過濾，並補上述真實 tag 的成功回歸測試。
- [WARNING] crates/host/speclink-host/src/security/deep.rs — Correctness: build_submission_context 將知識庫 manifest 的每個檔案都當 UTF-8 文字讀取，但正常官方 CWE 安裝會同時保留原始 raw/cwec_v4.20.xml.zip 與解壓 XML，KB prepare 也會將兩者完整複製並登錄指紋。因此選用正常完整 CWE 資料包後，即使 security kb prepare 成功，深度審查在建立 submission 時仍會因 ZIP 的 NUL 或非 UTF-8 位元組回 binary／unsupported_encoding，無法取得提交上下文或完成 add-round。這偏離完整離線資料包可供深度審查、準備成功後可使用該快照及深度報告提交的契約。請以受控的原始位元組驗證 manifest 檔案 SHA，或明確區分二進位封裝與可讀參考內容；保留完整資料及指紋驗證，並補官方 CWE ZIP＋XML 經安裝、KB prepare、submission／add-round 的串接回歸測試。

### 規格映射範圍

| Delta | Requirements | Scenarios | Examples |
| --- | ---: | ---: | ---: |
| change-lifecycle | 2 | 4 | 0 |
| client-protocol | 2 | 4 | 0 |
| desktop-app | 6 | 20 | 2 |
| desktop-config | 4 | 10 | 0 |
| desktop-network-diagnostics | 2 | 4 | 0 |
| node-sdk | 1 | 6 | 0 |
| propose-skill | 1 | 1 | 0 |
| review-skill | 1 | 2 | 0 |
| review-station | 3 | 10 | 1 |
| security-review | 5 | 9 | 1 |
| security-review-report | 5 | 10 | 0 |
| security-standard-data | 3 | 9 | 0 |
| ui-copy-vocabulary | 1 | 2 | 0 |
| verify-skill | 2 | 5 | 0 |
| verify-station | 2 | 9 | 1 |
| workflow-config | 3 | 8 | 0 |
| workspace-tools | 3 | 20 | 2 |
| 合計 | 46 | 133 | 7 |

映射覆蓋：引擎歷史與封存、八種保留組合、Host／Protocol／Remote／CLI、Desktop 排序與更新、共用文件樹與設定、Node 渲染、各 skill 原稿及 golden、離線 KB 與報告、acceptance、安全設定、工具初始化與清理。保留的 MODIFIED 原契約必要測試證據有部分在 frozen hunks 外，僅作相容上下文，不把未改舊問題列成本輪 findings。

## 分類與處置

四個不同問題均有具體觸發，全部列必修。無選修、無 accepted finding。修正可沿既有需求實作，沒有需要使用者先決定的 artifact 意圖變更。下一步由使用者選全部修正、指定項目或先停止；修正後必須先 build 與受影響測試轉綠，再對兩份活動工單進行 remediation validation，仍不自行蓋章。

## 限制及交付邊界

- 本輪代理沒有重跑測試；沿主線已通過的受影響驗證與既有證據。Correctness 未逐字複核所有生成副本／測試檔，不把這些範圍宣稱已逐字審查。全 repo test:all／跨平台 CI 仍由 CI 守門。
- 未進行 Windows／UNC／NAS、Antigravity 或公司 Proxy 實機驗收；目前安全政策 disabled，沒有另外啟動 OWASP 深度審查。
- 人工 12.3 的公司 Proxy 驗收依使用者要求等待 main 明確 commit 打包後執行；它不阻擋程式品質章，但封存須等人工完成。這次不能蓋乾淨章的原因是上述程式必修項。

## 使用者追加三問的查核

- GitHub API 查 mosutw/speclink 的 default_branch=main，contributors 已含 mosutw（前次查詢 38 次）；截圖若為原作者 repository，只統計該 repository。README 中英已列維護者與 Codex AI 工具署名。Contributors 不能由 README 手動指定；共同作者要使用可連結到實際 GitHub 帳號的 email，不偽造 Codex bot。來源：https://docs.github.com/en/repositories/viewing-activity-and-data-for-your-repository/viewing-a-projects-contributors 、https://docs.github.com/en/pull-requests/how-tos/commit-changes/creating-a-commit-with-multiple-authors 。
- 本分支可採獨立正式版號，例如 desktop-v0.10.1，Cargo workspace version 與 tauri.conf.json 同設 0.10.1。現有 desktop-release-version 將四段 tag 轉為 -build.N，Desktop Build 依含連字號標 prerelease，因此目前 0.10.0-build.3 是預先發行版。版本查詢來源固定 mosutw，原作者只列資訊；沒有修改版號、遠端 release 或 tag。自用打包流程沒有 updater 簽章包/latest.json，維持手動更新。
- 前次 GitHub API：本分支 desktop-v0.10.0.3/.2/.1 為 prerelease；舊正式桌面標籤 desktop-0.8.0-build.1；原作者正式桌面標籤 v0.10.0/v0.9.0。排除 prerelease 符合現有政策，但兩個專案 tag 不相容是本輪必修。截圖 fork 列沒有時間，亦可能尚未按頂端檢查更新；未查詢和查無合格版目前同顯示「未取得版本」，不能僅由截圖認定 Proxy 根因。
