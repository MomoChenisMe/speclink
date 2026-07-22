# Speclink 完整 SDD 工作流

**繁體中文** · [English](workflow.md)

本文是 Speclink 使用流程的使用者正典，回答每個階段「做什麼、何時用、何時跳過、完成後去哪裡」。第一次只想完成 Local Repo 一輪，先讀[入門教學](getting-started.zh-TW.md)；要判斷某項產品能力是否已可用，查[產品能力狀態](product-status.zh-TW.md)。

## Mental model / 心智模型

```text
onboard? → discuss? → propose → apply ⇄ ingest → archive
                         ↑
                 resume after pause: drift first

utilities: validate / analyze / audit / commit / verify and evidence
```

- `onboard` 只用於既有程式首次建立「目前行為」正典 specs。
- `discuss` 是需要收斂決策時的選用步驟；單純詢問不建立 discussion。
- `propose → apply ⇄ ingest → archive` 是 change 主生命週期。
- `drift` 是閒置後續作的條件式前置；`validate`、`analyze`、`audit`、`commit` 與 verify/evidence 是工具或守門，不是每個 change 都會依序經過的狀態。

## Choose the entry / 選擇入口

依序問五個問題，第一個符合者就是推薦入口：

| Question / 問題 | Answer / 判斷 | Recommended entry / 推薦入口 |
| --- | --- | --- |
| 只求理解，沒有待決事項嗎？ | 是 | 直接問答；不要建立 discussion。 |
| 已有相關 change 嗎？ | 是 | 若只是繼續實作走 `apply`；若新背景會改 artifacts，走 `ingest`。 |
| change 曾閒置或基準可能改變嗎？ | 是 | 先 `drift`，再依結果回 `apply` 或 `ingest`。 |
| 實作中需求或外部背景改變嗎？ | 是 | `ingest`，更新 artifacts 後再回 `apply`。 |
| 新需求已明確嗎？ | 是／否 | 明確就 `propose`；仍需取捨就 `discuss`。 |

既有 codebase 尚無正典 specs 時，在上述 change 流程前先做一次 `onboard`；它不會建立 change，也不描述未來理想。

## Lifecycle and utilities / 生命週期與工具

| Kind / 類型 | Stages / 階段 | Meaning / 意義 |
| --- | --- | --- |
| Main lifecycle / 主生命週期 | `propose`、`apply`、`ingest`、`archive` | change 從規劃、實作、需求更新到合併正典。 |
| Conditional / 條件式 | `onboard`、`discuss`、`drift` | 只有既有程式初始建規格、需求需決策、或閒置續作時使用。 |
| Quality and safety / 品質與安全 | `validate`、`analyze`、`audit`、verify/evidence | 檢查結構、artifact 一致性、安全 sharp edges 與實作證據。 |
| Git utility / Git 工具 | `commit` | 只提交特定 change 的相關檔案；不改變 change 生命週期。 |

## Stage reference / 階段參考

### onboard

- **Purpose / 目的：**從現有 code 與 tests 建立當前行為的 canonical specs。
- **Use / 使用：**採用 Speclink 的既有 codebase 尚無 specs，或只需補未覆蓋能力。
- **Skip / 跳過：**已有足夠 canonical specs，或要描述的是新需求而非現況。
- **Input / 輸入：**README、entry points、source、tests、設定與使用者確認的 capability map。
- **Outputs / 產物：**直接寫入 `openspec/specs/<capability>/spec.md`；不建立 change。
- **Claude：**`/speclink-onboard [scope]`。
- **Codex：**`$speclink-onboard [scope]`。
- **CLI/Host：**沒有 `speclink onboard` 子指令；Agent 盤點後寫 canonical specs，並以 `speclink validate --specs --all --strict` 驗證。
- **Done / 完成：**能力邊界已由使用者確認，specs 有可追溯行為證據且 strict validation 通過。
- **Next / 下一步：**新需求走 `propose`；模糊的新需求先 `discuss`。
- **Recover / 恢復：**發現既有 spec 需修改時，不在 onboard 重寫，另開 change。

### discuss

- **Purpose / 目的：**把需要取捨的問題逐輪收斂，保存可追溯結論。
- **Use / 使用：**需求模糊、設計有多個合理方向、需形成決策。
- **Skip / 跳過：**只是理解問題且沒有 verdict，或需求已明確可直接提案。
- **Input / 輸入：**一個聚焦主題、目前 code／spec 背景與需要決定的問題。
- **Outputs / 產物：**`openspec/discussions/<slug>.md` 的 Context、Rounds、Conclusion。
- **Claude：**`/speclink-discuss <topic>`。
- **Codex：**`$speclink-discuss <topic>`。
- **CLI/Host：**`speclink discuss new/context/add-round/conclude`；結論後依下一節選 `promote`、`link`、`seal` 或 `archive`。
- **Done / 完成：**Conclusion 有 Decision、Rationale、Rejected alternatives、Deferred、Capture to 與 Next。
- **Next / 下一步：**建立完整新 change、快速建立骨架、併入既有 change，或決定不做並封存。
- **Recover / 恢復：**有實質 rounds 的討論應 conclude＋archive；沒有形成任何內容才用 `discuss discard`。

### propose

- **Purpose / 目的：**建立可交付給實作者的 change 與 schema 所需 artifacts。
- **Use / 使用：**需求已清楚的新工作，或已結論 discussion 要形成完整提案。
- **Skip / 跳過：**純問答、只建立現況 specs，或既有 change 只需吸收新背景。
- **Input / 輸入：**明確需求，或 concluded discussion slug。
- **Outputs / 產物：**change metadata、proposal、delta specs、tasks，以及符合條件時的 design；實際集合由 schema DAG 與 `applyRequires` 決定。
- **Claude：**`/speclink-propose <change>` 或 `/speclink-propose --from-discussion <slug>`。
- **Codex：**`$speclink-propose <change>` 或 `$speclink-propose --from-discussion <slug>`。
- **CLI/Host：**`speclink new change`、`speclink instructions <artifact> --json`、`speclink new artifact ... --stdin`、`speclink analyze`、`speclink validate`。
- **Done / 完成：**`speclink status --change <name> --json` 顯示所有 `applyRequires` artifacts 完成，analyze 無 Critical／Warning 且 validate 通過。
- **Next / 下一步：**由使用者決定何時呼叫 `apply`。
- **Recover / 恢復：**`discuss promote` 只建立骨架時，對同一 change 再執行 propose 補齊；需求不清楚則回 discuss。

### apply

- **Purpose / 目的：**依 tasks 與實作契約修改 code／docs，逐項驗證與記錄完成。
- **Use / 使用：**change 的 `applyRequires` artifacts 已完整。
- **Skip / 跳過：**artifacts 尚缺、需求正在變更，或 change 閒置後尚未做 drift。
- **Input / 輸入：**proposal、design（若有）、delta specs、tasks 與目前 workspace。
- **Outputs / 產物：**實作變更、測試／驗證結果、已勾選 tasks 與 touched-file evidence。
- **Claude：**`/speclink-apply <change>`。
- **Codex：**`$speclink-apply <change>`。
- **CLI/Host：**Agent 用 `speclink instructions apply --change <name> --json` 取得 context；每項完成後用 `speclink task done --change <name> <id>`。
- **Done / 完成：**每個 task 的行為、契約與驗證目標均通過，apply instructions 回 `state: all_done`。
- **Next / 下一步：**完成品質／實作檢查後 `archive`；需求改變時先 `ingest`。
- **Recover / 恢復：**回滾 task 實作後用 `speclink task undone`；遠端 Context Projection 過期或被改動時重新取得 apply instructions 刷新。

### ingest

- **Purpose / 目的：**把新對話、計畫、外部文件或 discussion 決策合併到既有 change artifacts。
- **Use / 使用：**實作中需求／背景改變，或 concluded discussion 要併入已存在 change。
- **Skip / 跳過：**純實作沒有 artifact 變更，或尚無 change（應 propose）。
- **Input / 輸入：**既有 change 與新的外部背景；discussion 路徑先執行 `discuss link`。
- **Outputs / 產物：**合併更新後的 proposal／design／specs／tasks；已完成 tasks 保持不變。
- **Claude：**`/speclink-ingest <change>`。
- **Codex：**`$speclink-ingest <change>`。
- **CLI/Host：**逐 artifact 取得 `speclink instructions ... --json`，更新後執行 `speclink analyze`、`speclink validate`；discussion 內容已落地後 `speclink discuss seal <slug> <change>`。
- **Done / 完成：**新背景已映射到所有受影響 artifacts，完成 tasks 未被改寫，analysis／validation 通過；有 link 時已 seal。
- **Next / 下一步：**回 `apply`。
- **Recover / 恢復：**若 ingest 顯示既有假設已失效，先補齊 artifacts 再續作；不要只 seal 而未反映內容。

### drift

- **Purpose / 目的：**判斷閒置 change 與目前 codebase、design anchors、touched files 及基準是否漂移。
- **Use / 使用：**change 暫停後恢復，或懷疑外部 commits 已碰到同一範圍。
- **Skip / 跳過：**連續工作的短期 apply 且基準未變。
- **Input / 輸入：**change artifacts、Git 歷史、目前 code 與 evidence。
- **Outputs / 產物：**Light／Moderate／Heavy drift 報告與單一建議下一步。
- **Claude：**`/speclink-drift <change>`。
- **Codex：**`$speclink-drift <change>`。
- **CLI/Host：**`speclink drift <change> --json`。
- **Done / 完成：**報告已指出時間、broken anchors、task collision 與建議路徑。
- **Next / 下一步：**Light 通常回 `apply`；需求／delta 假設過時走 `ingest`；Heavy 先更新 artifacts。
- **Recover / 恢復：**無法判斷的外部修改先保留，不以重置或覆寫使用者 worktree 解決。

### validate

- **Purpose / 目的：**檢查 change／spec 的結構、必要欄位與 schema 規則。
- **Use / 使用：**提案完成、artifact 更新後、封存前與文件驗收時。
- **Skip / 跳過：**不應在交付前跳過；探索性閱讀可不執行。
- **Input / 輸入：**change 名稱、spec 或 `--all` 範圍。
- **Outputs / 產物：**valid／invalid 結果，可選 `--json`。
- **Claude：**無獨立生成 skill；由 propose／ingest／archive 流程呼叫。
- **Codex：**無獨立生成 skill；直接使用 CLI。
- **CLI/Host：**`speclink validate <change>`；全規格可用 `speclink validate --specs --all --strict`。
- **Done / 完成：**exit code 0 且目標顯示 valid。
- **Next / 下一步：**再做 analyze／實作驗證，或進入 archive。
- **Recover / 恢復：**依錯誤修正 artifacts 後重跑；不要用 `--no-validate` 掩蓋問題。

### analyze

- **Purpose / 目的：**跨 proposal、design、specs、tasks 檢查 Coverage、Consistency、Ambiguity、Gaps。
- **Use / 使用：**提案／ingest 完成後與最終 artifact 回歸。
- **Skip / 跳過：**單純查詢現有規格時可跳過；不可把它誤當 code test。
- **Input / 輸入：**一個 active change。
- **Outputs / 產物：**四維度 findings，含 severity、location、recommendation。
- **Claude：**無獨立生成 skill；由 artifact workflow 呼叫。
- **Codex：**無獨立生成 skill；直接使用 CLI。
- **CLI/Host：**`speclink analyze <change> --json`。
- **Done / 完成：**至少無 Critical／Warning；Suggestion 需明確評估是否影響交付。
- **Next / 下一步：**修 artifacts、`apply` 或最終驗收。
- **Recover / 恢復：**Critical 先修正 artifact 契約，不應直接開始實作。

### audit

- **Purpose / 目的：**從危險預設、型別混淆與靜默失敗角度稽核已變更 code。
- **Use / 使用：**安全敏感 API、設定、認證、Store／Server 邊界，或專案 `audit: true`。
- **Skip / 跳過：**純文件且沒有新增介面／安全語意時可跳過。
- **Input / 輸入：**特定 change 的 diff、design 與 specs。
- **Outputs / 產物：**按嚴重度排序的 sharp-edge findings；本身不改生命週期狀態。
- **Claude：**`/speclink-audit <change>`。
- **Codex：**`$speclink-audit <change>`。
- **CLI/Host：**沒有 `speclink audit` 子指令；skill 讀取 artifacts 與 diff 執行稽核。
- **Done / 完成：**每項發現都有具體位置、誤用方式與修正方向，或明確回報無 findings。
- **Next / 下一步：**修正後回 tests／apply；無問題則可進封存準備。
- **Recover / 恢復：**不要把「呼叫者責任」當成忽略危險介面的理由。

### verify and evidence

- **Purpose / 目的：**將實作逐項對照 tasks、delta specs 與 design，並保存 completion evidence。
- **Use / 使用：**apply 完成後、archive 前。
- **Skip / 跳過：**沒有 code checkout 的環境只能執行 artifact／server 可觀察檢查，必須標示限制。
- **Input / 輸入：**change artifacts、diff、tests 與 task evidence。
- **Outputs / 產物：**實作符合度結論、tests 結果、`task done` 的 evidence；遠端 evidence 目前有已知缺口。
- **Claude：**此 repo 沒有生成 `/speclink-verify`。
- **Codex：**此 repo 沒有生成 `$speclink-verify`。
- **CLI/Host：**目前以 `speclink task done`、`validate`、`analyze` 加上專案 tests 組合；引擎內有 verify asset，但不等於已安裝入口。
- **Done / 完成：**每個 Requirement／Scenario 與 task 契約都有可觀察證據，限制已記錄。
- **Next / 下一步：**通過後 `archive`；發現需求差異回 `ingest`，實作缺口回 `apply`。
- **Recover / 恢復：**不可把 validate/analyze 的 artifact 綠燈宣稱成 code correctness。

### commit

- **Purpose / 目的：**只選取並提交一個 Speclink change 的 artifacts 與相關實作檔。
- **Use / 使用：**需要可稽核的 change-scoped Git commit。
- **Skip / 跳過：**使用者有其他提交策略，或尚未確認 commit 範圍。
- **Input / 輸入：**change 名稱、Git status、touched files 與 tasks 進度。
- **Outputs / 產物：**經使用者確認的 selective stage 與 Git commit。
- **Claude：**`/speclink-commit <change>`。
- **Codex：**`$speclink-commit <change>`。
- **CLI/Host：**skill 結合 `speclink status/artifact` 與 Git；不使用 `git add .`／`git add -A`。
- **Done / 完成：**commit 只含確認過的 change 檔案，並回報 hash／message。
- **Next / 下一步：**可繼續 apply 或在完成後 archive；commit 不是 archive 的替代。
- **Recover / 恢復：**發現 unrelated changes 時排除並重新確認，不覆寫或清除它們。

### archive

- **Purpose / 目的：**將 delta specs 合併到 canonical specs，封存完成的 change 與關聯 discussion。
- **Use / 使用：**tasks 全部完成、artifacts valid、假設未過時且必要驗證通過。
- **Skip / 跳過：**有未完成 tasks、stale delta、驗證失敗或仍在變更需求。
- **Input / 輸入：**ready change、完整 final-state deltas 與 completion evidence。
- **Outputs / 產物：**更新後 canonical specs、`openspec/changes/archive/` 記錄；最後一個存活 change 封存時，關聯 discussion 一併封存。
- **Claude：**`/speclink-archive <change>`。
- **Codex：**`$speclink-archive <change>`。
- **CLI/Host：**`speclink archive <change>`；不要用 `--no-validate` 或 `--mark-tasks-complete` 規避未完成工作。
- **Done / 完成：**CLI 成功、canonical spec delta 統計正確、active change 已移入 archive。
- **Next / 下一步：**需要時使用 change-scoped commit 提交封存結果。
- **Recover / 恢復：**delta 不完整時先正規化；stale assumptions 回 `drift`／`ingest`，不要強制封存。

## Discussion outcomes / 討論結論分流

| Outcome / 結論去向 | Use when / 使用時機 | Command or skill / 呼叫 | Result / 結果 | Required next step / 必要下一步 |
| --- | --- | --- | --- | --- |
| New change, complete proposal / 新 change、完整提案 | 結論明確，想直接得到全部必要 artifacts | `$speclink-propose --from-discussion <slug>`（Claude 對應 `/speclink-propose`） | 建立並連結 change，執行完整 artifact workflow | artifacts 綠燈後由使用者決定 `apply` |
| New change, fast scaffold / 新 change、快速轉為變更骨架 | 只需立刻建立 change 身分，稍後再補完整提案 | `speclink discuss promote <slug> [--name <change>]` | 建立 change、以 Conclusion 預填 proposal 的 Why、連結兩側並把 discussion 標成已轉出變更；不是 apply-ready | 對該 change 再執行 propose，補齊 schema 必要 artifacts |
| Existing change / 既有 change | 結論要修正進行中的 change，不應另開新 change | `speclink discuss link <slug> <change>` → `$speclink-ingest <change>` → `speclink discuss seal <slug> <change>` | `link` 只建立 change 側來源鏈；ingest 反映內容；`seal` 才標記已轉出變更 | 回 `apply` |
| Do not implement / 決定不實作 | 討論有實質推理，但結論是不做 | `speclink discuss archive <slug>` | 保存結論與推理，不建立空 change | 無；未來新議題可開新 discussion |

一份 discussion 可轉出多個 change；`promoted_to` 會累積名稱。它會在最後一個仍存活的關聯 change 封存時自動一起封存。`link` 後不可先 `seal` 再補內容，因為 seal 表示決策已實際反映到 artifacts。

## Recovery paths / 恢復路徑

| Symptom / 症狀 | Route / 恢復路徑 |
| --- | --- |
| 轉為變更後只有 proposal 骨架 | 對同一 change 執行 propose；不要直接 apply。 |
| discussion 結論要進既有 change | `link → ingest → seal`；缺一不可。 |
| change 暫停一段時間 | 先 drift；Light 回 apply，假設過時回 ingest。 |
| 實作中需求改變 | ingest 更新 artifacts，重新 analyze／validate，再回 apply。 |
| apply 顯示缺 artifact | 回 propose 完成 `applyRequires` 鏈。 |
| task 被誤勾或實作回滾 | `speclink task undone --change <name> <id>`。 |
| Context Projection 為 STALE／被改動 | 不直接編輯 projection；重新取得 instructions 刷新。 |
| analyze 有 Critical | 先修 artifacts 的 coverage／consistency／gap，再實作。 |
| archive 指出 stale delta 或不完整 final state | 回 drift／ingest 並正規化 delta，重新 validate。 |

## Call layers / 呼叫層級

| Layer / 層級 | Responsibility / 責任 | Example / 範例 |
| --- | --- | --- |
| Speclink Skill | 告訴 Agent 何時讀背景、如何產生／驗證 artifacts 與何時停下；它是 workflow knowledge。 | Claude `/speclink-propose`、Codex `$speclink-propose` |
| `speclink` CLI | Local／Remote 的命令列 adapter，執行 status、instructions、artifact、task 與 lifecycle verbs。 | `speclink status --change demo --json` |
| Speclink Host/Runtime | 組合 Engine、Store、auth、binding、revision、transaction 與 event；它才是執行語意的 application boundary。 | Embedded Host 或 `speclink-server` |

不要把 skill 當成 runtime，也不要假設每個 Host 使用同一種呼叫字面：Claude 使用 slash command，Codex 使用 `$skill`，CLI 是另一個較低層入口。

## Current limitations / 目前限制

- 此 repo 的生成 skills 沒有 `$speclink-verify`／`/speclink-verify`；目前用專案 tests、`task done` evidence、`validate` 與 `analyze` 組合驗證，詳細狀態見[產品能力狀態](product-status.zh-TW.md)。
- `validate`／`analyze` 驗證 artifacts，不等同於 code tests 或完整實作符合度。
- Desktop Server Connections 已可用，但完整 Desktop Remote Workspace 仍是 Partial。
- Legacy remote REST v1 已棄用；新工作以目前 Client Protocol／Host 路徑為準。

## Related documents / 相關文件

- [Local Repo 入門教學](getting-started.zh-TW.md)
- [產品能力狀態](product-status.zh-TW.md)
- [平台架構藍圖](platform-architecture.zh-TW.md)
- [實作重構路線圖](implementation-refactor-roadmap.zh-TW.md)
