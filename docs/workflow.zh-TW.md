# Speclink 完整 SDD 工作流

**繁體中文** · [English](workflow.md)

本文是 Speclink 使用流程的使用者正典。它逐站回答五件事：做什麼、對應哪個技能、何時跳過、完成判準是什麼、完成後去哪裡。

第一次只想完成 Local Repo 一輪，先讀[入門教學](getting-started.zh-TW.md)。要判斷某項能力是否已可用，查[專案能力狀態](product-status.zh-TW.md)。

## Mental model / 心智模型

```text
baseline? → discuss?/improve? → propose → apply ⇄ ingest → (quality? | review? ∥ verify?) → archive
                                            ↑
                                    閒置後續作：先 drift

worktree：apply-with-worktree ⇄ ingest → (quality? | review? ∥ verify?) → worktree-merge → archive

工具：validate / analyze / audit / commit / config / manual
```

- `baseline` 只用於既有程式首次建立「目前行為」正典 specs。
- `discuss` 與 `improve` 都是選用的收斂入口，差別在題目誰帶：**你帶題目走 `discuss`，要模型幫你找題目走 `improve`**。
- `propose → apply ⇄ ingest → archive` 是變更的主生命週期。
- 兩道品質關卡（`review`、`verify`）互不依賴，依風險自由組合。低風險變更兩道都跳過也是正當選擇。
- `drift` 是閒置後續作的條件式前置。
- `validate`、`analyze`、`audit`、`commit`、`config`、`manual` 是工具或守門，不是每個變更都會依序經過的狀態。

看板把這條路線畫成三欄——提案中、進行中、已封存——每張卡片就是一個變更目前站在哪裡：

![Speclink 桌面 app 的變更看板，三欄分別是討論、提案中與進行中](assets/screenshots/desktop-board.png)

## Choose the entry / 選擇入口

依序問六個問題，第一個符合者就是推薦入口：

| Question / 問題 | Answer / 判斷 | Recommended entry / 推薦入口 |
| --- | --- | --- |
| 只求理解，沒有待決事項嗎？ | 是 | 直接問答；不要建立討論。 |
| 已有相關變更嗎？ | 是 | 若只是繼續實作走 `apply`；若新背景會改 artifacts，走 `ingest`。 |
| 變更曾閒置或規劃時的假設可能已過時嗎？ | 是 | 先 `drift`，再依結果回 `apply` 或 `ingest`。 |
| 實作中需求或外部背景改變嗎？ | 是 | `ingest`，更新 artifacts 後再回 `apply`。 |
| 想改善程式碼但講不出要改哪裡嗎？ | 是 | `improve`，讓模型掃描並提出候選。 |
| 新需求已明確嗎？ | 是／否 | 明確就 `propose`；仍需取捨就 `discuss`。 |

既有 codebase 尚無正典 specs 時，在上述流程前先做一次 `baseline`；它不會建立變更，也不描述未來理想。

## Lifecycle and utilities / 生命週期與工具

| Kind / 類型 | Stages / 階段 | Meaning / 意義 |
| --- | --- | --- |
| Main lifecycle / 主生命週期 | `propose`、`apply`、`ingest`、`archive` | 變更從規劃、實作、需求更新到合併正典。 |
| Conditional / 條件式 | `baseline`、`discuss`、`improve`、`drift`、worktree 流程 | 只有既有程式初始建規格、需求需收斂、閒置續作，或要平行推多個變更時使用。 |
| Quality stations / 品質關卡 | `review`、`verify`、`quality` | 封存前的兩道選用關卡：工藝與合規，各自落工單並蓋章。 |
| Utilities / 工具 | `validate`、`analyze`、`audit`、`commit`、`config`、`manual` | 檢查結構、artifact 一致性、安全 sharp edges、變更範圍提交、工作流設定與從規格生成的手冊。 |

## Stage reference / 階段參考

每一站的格式一致：用途、使用時機、跳過條件、輸入、產物、各介面的呼叫方式、完成判準、下一站與恢復路徑。

### baseline

- **Purpose / 目的**：從現有 code 與 tests 建立當前行為的正典 specs，作為後續變更的規格基準（舊稱 onboard）。
- **Use / 使用**：採用 Speclink 的既有 codebase 尚無 specs，或只需補未覆蓋能力。
- **Skip / 跳過**：已有足夠正典 specs，或要描述的是新需求而非現況。
- **Input / 輸入**：README、entry points、source、tests、以 `speclink workflow-config show --json` 取得的 workflow config（專案說明 `context`、`specLocale` 與 specs 產出規則 `rules.specs`）與使用者確認的 capability map。
- **Outputs / 產物**：直接寫入 `openspec/specs/<capability>/spec.md`；不建立變更。
- **Claude**：`/speclink-baseline [scope]`。
- **Codex**：`$speclink-baseline [scope]`。
- **CLI/Host**：沒有 `speclink baseline` 子指令。Agent 盤點後寫正典 specs，再以 `speclink validate --specs --all --strict` 檢查。
- **Done / 完成**：能力邊界已由使用者確認，specs 有可溯源的行為證據且 strict validation 通過。
- **Next / 下一步**：新需求走 `propose`；模糊的新需求先 `discuss`。
- **Recover / 恢復**：發現既有 spec 需修改時，不在 baseline 重寫，另開變更。

### discuss

- **Purpose / 目的**：把需要取捨的問題逐輪收斂，保存可溯源的結論。
- **Use / 使用**：需求模糊、設計有多個合理方向、需形成決策。
- **Skip / 跳過**：只是理解問題且沒有裁決，或需求已明確可直接提案。
- **Input / 輸入**：一個聚焦主題、目前 code／spec 背景與需要決定的問題。主題也可以是文件路徑（自寫計劃、plan mode 產出或任何可讀文件）——其主張會逐條對 codebase 分診。
- **Outputs / 產物**：`openspec/discussions/<slug>.md` 的背景、輪與結論。
- **Claude**：`/speclink-discuss <topic>`。
- **Codex**：`$speclink-discuss <topic>`。
- **CLI/Host**：`speclink discuss new/context/add-round/conclude`；結論後依[討論結論分流](#discussion-outcomes--討論結論分流)選 `promote`、`link`、`seal` 或 `archive`。
- **Done / 完成**：結論含決策、理由、已否決方案、暫緩事項、落點與下一步。
- **Next / 下一步**：建立完整新變更、快速建立骨架、併入既有變更，或決定不做並封存。
- **Recover / 恢復**：有實質輪的討論應結論＋封存；沒有形成任何內容才用 `discuss discard`。

### improve

- **Purpose / 目的**：掃描 codebase、提出結構改進的候選，寫進同一套討論記錄。
- **Use / 使用**：想改善 codebase 但講不出具體要改哪裡的時候。
- **Skip / 跳過**：你已經知道要改什麼——那是 `discuss` 或直接 `propose` 的題目。
- **Input / 輸入**：你點名的方向（優先），或以 git log 熱點推斷的範圍。掃描前一定先收斂範圍，全 repo 漫掃只會得到泛泛的候選。
- **Outputs / 產物**：與 `discuss` 同一份討論記錄，以 `--kind improve` 標記（看板卡片與討論詳情面板顯示小章）；每個候選附 Files／Problem／Solution／Wins／建議強度。
- **Claude**：`/speclink-improve [scope]`。
- **Codex**：`$speclink-improve [scope]`。
- **CLI/Host**：`speclink discuss new <topic> --kind improve`；之後的輪、結論、轉為變更、封存與 `discuss` 完全相同。
- **Done / 完成**：候選已逐項列出，你挑一個深入盤問，結論也寫下了。**候選全數不採納時也要寫結論並封存**——否決理由本身就是下次掃描的防重提依據。
- **Next / 下一步**：採納的候選走 `propose` → `apply`；全數否決則封存討論。
- **Recover / 恢復**：開場會讀已封存討論的已否決項與進行中的變更，避免重提已否決或正在做的事；發現重提時直接指出來源討論。

兩條限制寫在這裡以免誤用。第一，`improve` **只由你發起**，模型不會自己跑。第二，它**只產討論記錄，不寫程式碼**；改進要落地一樣走 `propose` → `apply`。

### propose

- **Purpose / 目的**：建立可交付給實作者的變更與 schema 所需 artifacts。
- **Use / 使用**：需求已清楚的新工作，或已結論討論要形成完整提案。
- **Skip / 跳過**：純問答、只建立現況 specs，或既有變更只需吸收新背景。
- **Input / 輸入**：明確需求、已結論討論的 slug，或以 `--from-doc` 指定的文件路徑。
- **Outputs / 產物**：變更 metadata、proposal、delta specs、tasks，以及符合條件時的 design；實際集合由 schema DAG 與 `applyRequires` 決定。
- **Claude**：`/speclink-propose <change>`、`/speclink-propose --from-discussion <slug>` 或 `/speclink-propose --from-doc <path>`。
- **Codex**：同名的 `$speclink-propose ...`。
- **CLI/Host**：`speclink new change`、`speclink instructions <artifact> --json`、`speclink new artifact ... --stdin`、`speclink analyze`、`speclink validate`。
- **Done / 完成**：`speclink status --change <name> --json` 顯示所有 `applyRequires` artifacts 完成，analyze 無 Critical／Warning 且 validate 通過。
- **Next / 下一步**：由你決定何時呼叫 `apply`。
- **Recover / 恢復**：`discuss promote` 只建立骨架時，對同一變更再執行 propose 補齊；需求不清楚則回 `discuss`。

### apply

- **Purpose / 目的**：依 tasks 與實作契約修改 code／docs，逐項檢查並記錄完成。
- **Use / 使用**：變更的 `applyRequires` artifacts 已完整。
- **Skip / 跳過**：artifacts 尚缺、需求正在變更，或變更閒置後尚未做 `drift`。
- **Input / 輸入**：proposal、design（若有）、delta specs、tasks 與目前 workspace。
- **Outputs / 產物**：實作變更、測試與檢查結果、已勾選 tasks，以及 touched-file evidence。evidence 落在 `openspec/changes/<name>/.evidence.json`，隨變更目錄一起提交。
- **Claude**：`/speclink-apply <change>`。
- **Codex**：`$speclink-apply <change>`。
- **CLI/Host**：開工前先跑 `speclink review prepare <change>` 記下品質關卡要用的 Apply 基準，再跑 `speclink in-progress add <change>`。Agent 用 `speclink instructions apply --change <name> --json` 取得脈絡，每項完成後跑 `speclink task done --change <name> <id>`。
- **Done / 完成**：每個 task 的行為、契約與該過的檢查都通過，且 apply instructions 回 `state: all_done`。標著 `[M]` 的任務由你手動確認，模型不會代打勾。
- **Next / 下一步**：依風險決定要不要跑品質關卡，再 `archive`；需求改變時先 `ingest`。
- **Recover / 恢復**：回滾 task 實作後用 `speclink task undone`。誤開工且零工作痕跡時，用 `speclink in-progress remove` 退回提案中。遠端 Context Projection 過期或被改動時，重新取得 apply instructions 刷新。

變更詳情面板是 apply 期間的主要視角。提案、設計、任務與規格四個分頁對應同一組 artifacts，任務分頁的進度就是 `speclink task done` 的結果：

![變更詳情面板，顯示提案內容與任務、規格分頁的進度](assets/screenshots/desktop-change-drawer.png)

### worktree（平行實作）

- **Purpose / 目的**：同時推進多個互相獨立的變更，各自在自己的 git worktree 裡實作，互不干擾。
- **Use / 使用**：手上有兩個以上彼此不衝突的變更要一起做。
- **Skip / 跳過**：單一變更，或多個變更會改到同一批檔案——那種情況排隊做比較快。
- **Prerequisite / 前置**：`worktree` 政策要先開（`speclink workflow-config set worktree true`）。兩個 worktree 技能只在政策開啟時才生成；政策關閉時它們不存在。
- **Input / 輸入**：已 apply-ready 的多個變更。
- **Outputs / 產物**：每個變更一個 worktree 與對應分支；實作、品質關卡與提交都在該 worktree 內完成。
- **Claude**：`/speclink-apply-with-worktree <changes>`，收尾 `/speclink-worktree-merge <change>`。
- **Codex**：同名的 `$speclink-apply-with-worktree`、`$speclink-worktree-merge`。
- **CLI/Host**：`speclink list` 以 `[worktree]` 標示哪些變更在 worktree 中。
- **Done / 完成**：worktree 內任務完成、你選擇要跑的品質關卡已蓋章、變更也已提交。接著 `worktree-merge` 把分支併回主分支並清掉 worktree。
- **Next / 下一步**：回到主 checkout 執行 `archive`——**封存只能在主 checkout 跑**，在 linked worktree 內會被引擎拒絕。
- **Recover / 恢復**：品質關卡要在 worktree 內跑，因為 Apply 基準記在那裡。worktree 內的 `tasks.md` 與主 checkout 是兩份，要改只改 worktree 那份。

### ingest

- **Purpose / 目的**：把新對話、計畫、外部文件或討論決策合併到既有變更的 artifacts。
- **Use / 使用**：實作中需求／背景改變，或已結論討論要併入已存在的變更。
- **Skip / 跳過**：純實作沒有 artifact 變更，或尚無變更（應 `propose`）。
- **Input / 輸入**：既有變更與新的外部背景；討論路徑先執行 `discuss link`。
- **Outputs / 產物**：合併更新後的 proposal／design／specs／tasks；已完成 tasks 保持不變。
- **Claude**：`/speclink-ingest <change>`。
- **Codex**：`$speclink-ingest <change>`。
- **CLI/Host**：逐 artifact 取得 `speclink instructions ... --json`，更新後執行 `speclink analyze` 與 `speclink validate`。討論內容落地之後，再跑 `speclink discuss seal <slug> <change>`。
- **Done / 完成**：新背景已映射到所有受影響 artifacts，已完成的 tasks 沒被改寫，analyze 與 validate 都通過。有 link 的話也已經 seal。
- **Next / 下一步**：回 `apply`。
- **Recover / 恢復**：若 ingest 顯示既有假設已失效，先補齊 artifacts 再續作。不要只 seal 而沒把內容反映進去。

### drift

- **Purpose / 目的**：判斷閒置變更與目前 codebase、design anchors、touched files 及規劃時假設是否漂移。
- **Use / 使用**：變更暫停後恢復，或懷疑外部 commits 已碰到同一範圍。
- **Skip / 跳過**：連續工作的短期 apply 且基準未變。
- **Input / 輸入**：變更 artifacts、Git 歷史、目前 code 與 evidence。
- **Outputs / 產物**：Light／Moderate／Heavy 漂移報告與單一建議下一步。
- **Claude**：`/speclink-drift <change>`。
- **Codex**：`$speclink-drift <change>`。
- **CLI/Host**：`speclink drift <change> --json`。
- **Done / 完成**：報告已指出時間、broken anchors、任務衝突與建議路徑。
- **Next / 下一步**：Light 通常回 `apply`；需求／delta 假設過時走 `ingest`；Heavy 先更新 artifacts。
- **Recover / 恢復**：無法判斷的外部修改先保留，不以重置或覆寫使用者 worktree 解決。

### quality（兩道一起跑）

- **Purpose / 目的**：同一個變更要跑兩道品質關卡時的編排。兩道都先檢查、先不蓋章，每輪停下等你裁示。
- **Use / 使用**：改動大且既在意工藝也在意合規，兩道都想跑。
- **Skip / 跳過**：只想跑一道時，直接呼叫 `/speclink-review` 或 `/speclink-verify`，維持該站修完即蓋的預設。
- **Input / 輸入**：任務全部完成的變更。
- **Outputs / 產物**：兩份工單（`review.md`、`verify.md`）與兩枚章。
- **Claude**：`/speclink-quality <change>`。
- **Codex**：`$speclink-quality <change>`。
- **CLI/Host**：底層是 `speclink review`／`speclink verify` 各自的 `scope`、`add-round`、`show`、`stamp`。
- **Done / 完成**：你說可以了，兩章才接連蓋——審查在前、驗證在後。
- **Next / 下一步**：`archive`（worktree 流程則先 `worktree-merge`）。
- **Recover / 恢復**：乾淨輪也會停，不會自己蓋章或封存。複驗迴圈中途不要提交，那會靜默離開審查面。

兩道關卡的分工：

| | `review` 審查 | `verify` 驗證 |
| --- | --- | --- |
| 回答的問題 | 程式碼寫得好不好（工藝） | 交付是否符合規格（合規） |
| 判準 | repo 慣例文件＋Fowler smells 基線（repo 文件優先）＋bug 獵捕 | 變更的 specs 逐條三維度 |
| artifacts 的角色 | 判準脈絡，不產合規裁決 | 檢查的中心 |
| 執行前提 | 全任務完成 | 檢查隨時可跑（中途執行＝進度盤點）；收尾落工單要求全任務完成 |
| 產出 | `review.md` 工單多輪，零必修後蓋章 | `verify.md` 工單多輪，零必修後蓋章 |
| 蓋章順序 | 前 | 後 |

兩道都跑時的時序分四段：

1. 兩道都先檢查，先不蓋章。
2. 每輪停下等你裁示：全修、挑著修，或不修就停。
3. 你裁示要修的統一落地，兩道複驗，再停一次。
4. 你說可以了，兩章才接連蓋。

之所以要這樣排，是因為**站章凍結的是範圍內檔案的內容指紋**：先蓋的章會被另一站的修正打成「其後有變動」。先修完再一起蓋就沒這個問題。

**蓋章會消耗工單。**章欄位寫入與工單（`review.md`／`verify.md`）刪除發生在同一個原子寫入內，不存在「章已寫入而工單仍在」的狀態。因此封存的已蓋章變更不含 `review.md` 與 `verify.md`；只有未結工單會經 `--carry-review` 或 `--carry-verify` 隨封存移動。fs 模式下被刪工單的文字僅存於 git 歷史；remote 模式的 store 不保留已刪文件內容，蓋章後工單文字不可回讀。

### review

- **Purpose / 目的**：以工藝標準審查實作，findings 分級記入工單。
- **Use / 使用**：改動大、跨子系統、或會被長期維護的程式碼。
- **Skip / 跳過**：低風險小改；跳過是正當選擇，不是欠帳。
- **Input / 輸入**：Apply 基準凍結出來的變更範圍（`speclink review prepare` 在開工前記下的 HEAD 與起始髒檔）。
- **Outputs / 產物**：`review.md` 工單，findings 分級 CRITICAL／WARNING／SUGGESTION。
- **Claude**：`/speclink-review <change>`。
- **Codex**：`$speclink-review <change>`。
- **CLI/Host**：`speclink review prepare/scope/add-round/show/stamp/discard`。
- **Done / 完成**：全任務完成且最後一輪的必修集合為空即可蓋章——**SUGGESTION 不擋章**。蓋章會在同一個原子寫入內寫入 reviewed 欄位並刪除 `review.md`；fs 模式下工單文字僅存於 git 歷史，remote 模式蓋章後不可回讀。
- **Next / 下一步**：`verify`（若也要跑）或 `archive`。
- **Recover / 恢復**：蓋章後範圍內檔案再被修改時，卡片標示降級為「已審查·其後有變動」。封存時偵測到未結工單會被攔下，此時有三個選擇：回去蓋章、放棄審查，或照樣帶走。另外，工單裡的 finding 路徑不得帶行號，且必須逐字落在凍結快照的檔案集內。

### verify

- **Purpose / 目的**：逐條對照變更的 specs，判定交付是否符合規格。
- **Use / 使用**：規格條款多、或合規性本身就是交付重點時。
- **Skip / 跳過**：低風險小改；同樣是正當選擇。
- **Input / 輸入**：變更的全部 artifacts 與凍結的變更 patch。
- **Outputs / 產物**：`verify.md` 工單多輪。
- **Claude**：`/speclink-verify <change>`。
- **Codex**：`$speclink-verify <change>`。
- **CLI/Host**：`speclink verify scope/add-round/show/stamp/discard`。
- **Done / 完成**：全任務完成且最後一輪必修集合為空；**SUGGESTION 同樣不擋章**。蓋章會在同一個原子寫入內寫入 verified 欄位並刪除 `verify.md`；fs 模式下工單文字僅存於 git 歷史，remote 模式蓋章後不可回讀。
- **Next / 下一步**：`archive`。
- **Recover / 恢復**：任務全完成後，第一輪是唯一一次完整盤查——讀全部 artifacts，程式碼證據限定在凍結的變更 patch。之後每一輪只看兩樣東西：上輪未解的 findings，以及修正 patch 直接造成的回歸；不重掃未修改的區域。**必修集合每輪必須嚴格變少**才允許再修一次。第一次沒進展就以「未通過」停下，保留工單、不蓋章。

卡片與系統匣面板上，審查章與驗證章並排，審查在前、驗證在後。兩張工單並存時，要對兩道關卡分別處置才封存得掉。

### archive

- **Purpose / 目的**：將 delta specs 合併到正典 specs，封存完成的變更與關聯討論。
- **Use / 使用**：任務全部完成、artifacts valid、假設未過時，且你選擇要跑的品質關卡已結案。
- **Skip / 跳過**：有未完成任務、stale delta、`validate` 未過，或需求還在變。
- **Input / 輸入**：ready 的變更、完整 final-state deltas 與完成證據。
- **Outputs / 產物**：更新後的正典 specs、`openspec/changes/archive/` 記錄；最後一個存活變更封存時，關聯討論一併封存。已蓋章的變更封存時不含工單檔；只有未結工單會經 `--carry-review`／`--carry-verify` 隨封存移動。
- **Claude**：`/speclink-archive <change>`。
- **Codex**：`$speclink-archive <change>`。
- **CLI/Host**：`speclink archive <change>`；不要用 `--no-validate` 或 `--mark-tasks-complete` 規避未完成工作。
- **Done / 完成**：CLI 成功、正典 spec delta 統計正確、變更已移入 archive。
- **Next / 下一步**：需要時以變更範圍的提交把封存結果留下來。
- **Recover / 恢復**：delta 不完整時先正規化。假設過時就回 `drift` 或 `ingest`，不要強制封存。還有一個常見地雷：MODIFIED 區塊是整塊取代，所以改了 scenario 名稱等於未宣告的刪除。validate 與 analyze 都抓不到，要到封存才炸——補一則 `REMOVED-SCENARIO` 註解明示。

### validate

- **Purpose / 目的**：檢查變更／spec 的結構、必要欄位與 schema 規則。
- **Use / 使用**：提案完成、artifact 更新後、封存前與文件驗收時。
- **Skip / 跳過**：不應在交付前跳過；探索性閱讀可不執行。
- **Input / 輸入**：變更名稱、spec 或 `--all` 範圍。
- **Outputs / 產物**：valid／invalid 結果，可選 `--json`。
- **Claude／Codex**：無獨立技能；由 propose／ingest／archive 流程呼叫。
- **CLI/Host**：`speclink validate <change>`；全規格用 `speclink validate --specs --all --strict`。
- **Done / 完成**：exit code 0 且目標顯示 valid。
- **Next / 下一步**：再跑 analyze 或實作面的檢查，或進入 `archive`。
- **Recover / 恢復**：依錯誤修正 artifacts 後重跑；不要用 `--no-validate` 掩蓋問題。

### analyze

- **Purpose / 目的**：跨 proposal、design、specs、tasks 檢查 Coverage、Consistency、Ambiguity、Gaps。
- **Use / 使用**：提案／ingest 完成後與最終 artifact 回歸。
- **Skip / 跳過**：單純查詢現有規格時可跳過；不可把它誤當 code test。
- **Input / 輸入**：一個進行中的變更。
- **Outputs / 產物**：四維度 findings，含 severity、location、recommendation。
- **Claude**：`/speclink-analyze <change>`。
- **Codex**：尚未生成對應技能；直接使用 CLI。
- **CLI/Host**：`speclink analyze <change> --json`。
- **Done / 完成**：至少無 Critical／Warning；Suggestion 需明確評估是否影響交付。
- **Next / 下一步**：修 artifacts、`apply` 或最終驗收。
- **Recover / 恢復**：Critical 先修正 artifact 契約，不應直接開始實作。

### audit

- **Purpose / 目的**：從危險預設、型別混淆與靜默失敗角度稽核已變更的 code。
- **Use / 使用**：安全敏感 API、設定、認證、Store／Server 邊界，或專案設定 `audit: true`。
- **Skip / 跳過**：純文件且沒有新增介面／安全語意時可跳過。
- **Input / 輸入**：特定變更的 diff、design 與 specs。
- **Outputs / 產物**：按嚴重度排序的 sharp-edge findings。這一站本身不改變生命週期狀態。
- **Claude**：`/speclink-audit <change>`。
- **Codex**：`$speclink-audit <change>`。
- **CLI/Host**：沒有 `speclink audit` 子指令；技能讀取 artifacts 與 diff 執行稽核。
- **Done / 完成**：每項發現都有具體位置、誤用方式與修正方向，或明確回報無 findings。
- **Next / 下一步**：修正後回測試／`apply`；無問題則可進封存準備。
- **Recover / 恢復**：不要把「呼叫者責任」當成忽略危險介面的理由。

### commit

- **Purpose / 目的**：只選取並提交一個變更的 artifacts 與相關實作檔。
- **Use / 使用**：需要可稽核的、範圍限定在單一變更的 Git commit。
- **Skip / 跳過**：你有其他提交策略，或尚未確認提交範圍。
- **Input / 輸入**：變更名稱、Git status、touched files 與任務進度。
- **Outputs / 產物**：經你確認的選擇性 stage 與 Git commit。
- **Claude**：`/speclink-commit <change>`。
- **Codex**：`$speclink-commit <change>`。
- **CLI/Host**：技能結合 `speclink status/artifact` 與 Git；不使用 `git add .`／`git add -A`。
- **Done / 完成**：commit 只含確認過的變更檔案，並回報 hash 與訊息。
- **Next / 下一步**：可繼續 `apply` 或在完成後 `archive`；提交不是封存的替代。
- **Recover / 恢復**：發現無關檔案時排除並重新確認，不覆寫或清除它們。平行 session 同時動同一批檔案時，提交前重新盤點 `git status`。

### config

- **Purpose / 目的**：從 codebase 組出工作流設定的脈絡與規則（`openspec/config.yaml`）。
- **Use / 使用**：要讓 Agent 產出的 artifacts 貼合這個 repo 的慣例時。
- **Skip / 跳過**：預設設定已夠用。
- **Input / 輸入**：codebase 慣例、既有文件與測試。
- **Outputs / 產物**：經你核可的 diff 落成 `openspec/config.yaml`。
- **Claude**：`/speclink-config`。
- **Codex**：`$speclink-config`。
- **CLI/Host**：`speclink workflow-config`。
- **Done / 完成**：diff 已核可並套用。
- **Next / 下一步**：回到任一站；設定影響之後所有 artifact 產出。
- **Recover / 恢復**：設定寫錯只要再跑一次改回來，不影響既有變更。

### manual

- **Purpose / 目的**：從正典規格生成 `openspec/manual/` 的 wiki 式操作手冊，或在對話中導覽系統怎麼操作。
- **Use / 使用**：需要一份給人讀的操作手冊，或新人想被帶著走一遍系統時；封存後想確認手冊是否可能過期時也用它。
- **Skip / 跳過**：專案尚無使用者面向的規格，或沒有人要讀手冊。
- **Input / 輸入**：正典規格（`openspec/specs/`）與既有手冊頁的 frontmatter；不讀 README、docs 或程式碼。
- **Outputs / 產物**：生成模式寫出 `openspec/manual/*.md`（含首頁與來源頁），並回報可能過期的頁與未入冊能力；導覽模式零寫檔。
- **Claude**：`/speclink-manual`（生成）、`/speclink-manual 導覽`（導覽）。
- **Codex**：`$speclink-manual`。
- **CLI/Host**：沒有 `speclink manual` 子指令；技能以 `speclink list --specs` 與 `speclink show` 讀規格。
- **Done / 完成**：摘要列出新增、重生、未動的頁數與可能過期頁／未入冊能力清單；無異動時明示手冊已是最新。
- **Next / 下一步**：以一般提交收尾手冊異動（僅建議）。remote 綁定的專案生成模式尚不支援，導覽照常。
- **Recover / 恢復**：手冊頁是普通檔案，刪掉或還原即可；重生只碰可能過期的頁，既有順序不變。

## Discussion outcomes / 討論結論分流

| Outcome / 結論去向 | Use when / 使用時機 | Command or skill / 呼叫 | Result / 結果 | Required next step / 必要下一步 |
| --- | --- | --- | --- | --- |
| 新變更、完整提案 | 結論明確，想直接得到全部必要 artifacts | `/speclink-propose --from-discussion <slug>`（Codex 為 `$speclink-propose`） | 建立並連結變更，執行完整 artifact workflow | artifacts 綠燈後由你決定 `apply` |
| 新變更、快速骨架 | 只需立刻建立變更身分，稍後再補完整提案 | `speclink discuss promote <slug> [--name <change>]` | 建立變更、以結論預填 proposal 的 Why、連結兩側並把討論標成已轉出變更；不是 apply-ready | 對該變更再執行 propose，補齊 schema 必要 artifacts |
| 併入既有變更 | 結論要修正進行中的變更，不應另開新變更 | `speclink discuss link <slug> <change>` → `/speclink-ingest <change>` → `speclink discuss seal <slug> <change>` | `link` 只建立變更側來源鏈；ingest 反映內容；`seal` 才標記已轉出變更 | 回 `apply` |
| 決定不實作 | 討論有實質推理，但結論是不做 | `speclink discuss archive <slug>` | 保存結論與推理，不建立空變更 | 無；未來新議題可開新討論 |

一份討論可以轉出多個變更，`promoted_to` 會累積這些名稱。最後一個仍存活的關聯變更封存時，這份討論會自動一起封存。

`link` 之後不可以先 `seal` 再補內容。seal 代表決策已經實際反映到 artifacts，先蓋等於說謊。

## Recovery paths / 恢復路徑

| Symptom / 症狀 | Route / 恢復路徑 |
| --- | --- |
| 轉為變更後只有 proposal 骨架 | 對同一變更執行 propose；不要直接 apply。 |
| 討論結論要進既有變更 | `link → ingest → seal`；缺一不可。 |
| 變更暫停一段時間 | 先 drift；Light 回 apply，假設過時回 ingest。 |
| 實作中需求改變 | ingest 更新 artifacts，重新 analyze／validate，再回 apply。 |
| apply 顯示缺 artifact | 回 propose 完成 `applyRequires` 鏈。 |
| 任務被誤勾或實作回滾 | `speclink task undone --change <name> <id>`。 |
| 變更誤開工、想退回提案中 | `speclink in-progress remove <change>`；僅零工作痕跡時可行。 |
| Context Projection 為 STALE／被改動 | 不直接編輯 projection；重新取得 instructions 刷新。 |
| analyze 有 Critical | 先修 artifacts 的 coverage／consistency／gap，再實作。 |
| 品質關卡蓋章後檔案又被改 | 卡片降級為「其後有變動」；回該站再跑一輪重新蓋。 |
| 封存被未結工單攔下 | 回去蓋章、放棄該站、或明確選擇照樣帶走。 |
| 在 worktree 裡執行封存被拒 | 先 `worktree-merge` 回主分支，封存只在主 checkout 執行。 |
| archive 指出 stale delta 或不完整 final state | 回 drift／ingest 並正規化 delta，重新 validate。 |

## Call layers / 呼叫層級

| Layer / 層級 | Responsibility / 責任 | Example / 範例 |
| --- | --- | --- |
| Speclink 技能 | 告訴 Agent 何時讀背景、如何產生與檢查 artifacts、何時停下。它是流程知識。 | Claude `/speclink-propose`、Codex `$speclink-propose` |
| `speclink` CLI | Local／Remote 的命令列 adapter，執行 status、instructions、artifact、task 與生命週期動詞。 | `speclink status --change demo --json` |
| Speclink Host/Runtime | 組合 Engine、Store、認證、binding、revision、交易與事件。它才是執行語意的 application boundary。 | Embedded Host 或 `speclink-server` |

不要把技能當成 runtime，也不要假設每個 Host 使用同一種呼叫字面。Claude 使用斜線指令。Codex 以 `$技能名` 明確呼叫技能，也可以打 `/skills` 從清單裡挑同一個技能。CLI 是另一個較低層入口。

## Current limitations / 目前限制

- `validate` 與 `analyze` 只檢查 artifacts，不等同於 code tests，也不等同於完整的實作符合度。實作面由品質關卡處理。
- Desktop Server Connections 已可用。完整的 Desktop Remote Workspace 仍是部分可用：在桌面遠端看板勾任務不會回報 touched files（CLI 路徑會存）。
- Legacy remote REST v1 已棄用。新工作以目前的 Client Protocol／Host 路徑為準。
- 逐項證據與最後查核日期以[專案能力狀態](product-status.zh-TW.md)為準，本文不重複維護狀態矩陣。

## Related documents / 相關文件

- [Local Repo 入門教學](getting-started.zh-TW.md)
- [Remote Server、Desktop 與 CLI 入門教學](remote-getting-started.zh-TW.md)
- [專案能力狀態](product-status.zh-TW.md)
- [動詞與旗標契約](verb-contract.zh-TW.md)
- [專案路線圖](roadmap.zh-TW.md)
