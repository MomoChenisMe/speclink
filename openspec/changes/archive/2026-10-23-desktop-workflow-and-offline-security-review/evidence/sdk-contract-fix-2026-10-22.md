# CI 核對與 SDK 契約修復（2026-10-22）

## 授權與範圍

使用者要求先完成交接清單第 1、2 步：核對最新提交 CI，採現行 SDK 方向修訂舊 API 規格，修正 Antigravity 公開型別並補測試。本輪基於 `c8b77ece886d3db0b0883ba9b70b0b66a34e344d`，只更新規劃、SDK 型別／測試與雙語 SDK 說明；不將這次實作當成正式品質重跑。原始九項需求與原有 59 項完成任務逐字保留，新增 19.1、19.2。公司 Proxy 的 `[M]` 12.3 保持未完成。

## 第 1 步：實際 CI 狀態

透過 GitHub CLI 查詢 Actions runs、最新提交 check-runs 與 PR #12：

- 目前功能分支 HEAD 是 `c8b77ece886d3db0b0883ba9b70b0b66a34e344d`。該 SHA 的 Actions 查詢為空，check-runs 的 total_count 為 0，**沒有最新提交的 CI 成功證據**。
- [PR #12](https://github.com/mosutw/speclink/pull/12) 已於 `2026-10-22T02:41:26Z` 合併，其 head 為 `46803b2b3af11719aadde6266a2370c2d7dff367`，不是目前 HEAD。該提交的 [CI](https://github.com/mosutw/speclink/actions/runs/39702058887) 與 [Node SDK](https://github.com/mosutw/speclink/actions/runs/39702058902) 均成功。
- `origin/main` 的合併提交為 `9774b7a27121e886e19186f2cd1cb89a064bc6e0`，其 [CI](https://github.com/mosutw/speclink/actions/runs/39712765459) 與 [Node SDK](https://github.com/mosutw/speclink/actions/runs/39712765475) 均成功。
- 本機 `git log origin/main..HEAD` 只有 `c8b77ece`。`.github/workflows/ci.yml` 與 `node-sdk.yml` 的 push 僅匹配 main／master；Node SDK 另有 paths 篩選，兩者都支援 pull_request。合併後只推功能分支不會由 push 觸發它們。本輪沒有修改觸發規則、建立 PR 或觸發遠端寫入。

核對已完成，結論為「main 與先前 PR 通過，最新功能提交未觸發」，不能表述為 c8b77ece 已通過 CI。本輪未提交修改也沒有遠端 CI；後續提交並開 PR 後，須核對新 SHA 的結果。

## 第 2 步：規格方向與修復

- 規格採 `skills.list()`、`skills.render(name, options)`；選項為 target／invocation／specDir／toolName。移除 delta 中已退役的 instructions.render 與 store 渲染承諾，並明確 REMOVED「渲染 API 的 worktree 軸」，避免封存後留下相互矛盾的 canonical 要求。canonical specs 此時不直接改寫。
- `index.d.ts` 的 RenderOptions.target 補上 antigravity，保留 claude／codex／copilot／neutral。
- 新增 `__test__/types.spec.ts` 與 consumer fixture，從套件入口讀取 package.json 指向的手寫 index.d.ts，執行 strict／noEmit 編譯；五種 target 與完整選項通過，未知 target／invocation、store／worktree 及 instructions 匯入須產生預期編譯錯誤。不是只測 Vitest 轉譯或生成 binding.d.ts。
- Node 套件新增 TypeScript 開發依賴，鎖定版本 5.9.3；原有 npm test 自動收集此測試，既有 Node SDK CI 無需另建工作。實際 compiler 解析至該套件自己的 node_modules/typescript，沒有借用根 workspace 的版本。
- 補渲染回歸：預設值等同明給 cli／openspec／speclink；自訂 specDir／toolName 於 verify 技能替換成 specs-local/changes/ 與 --agent project-tool；未知 invocation shell 拒絕。工具呼叫前言仍是 speclink，toolName 是技能中的工具身分替換，沒有擅改既有生成內容。
- 同步雙語 SDK 文件的 API 邊界與 toolName 語意，清除 src/render.rs 中已不存在的 store 渲染軸註解。JavaScript 匯出、Rust 渲染邏輯、CLI 人眼／JSON／exit、資產原稿與生成內容均未改動。

## 測試結果

- `cargo build -p speclink-cli --locked`：成功，後續使用此 checkout 新建置 CLI。
- `npm --prefix crates/adapters/speclink-node run build`：成功；最後文件／註解修改後也已重建。
- RED：新增公開型別測試在修正前確實失敗，compiler 回 TS2322，只有 antigravity 不屬於四值 target 的診斷。
- GREEN：補型別後同一測試通過；五個預期錯誤指令也由 compiler 核對。
- `npm --prefix crates/adapters/speclink-node ci --offline --ignore-scripts --no-audit --no-fund`：53 packages 安裝成功，確認新 manifest 與 lock 可供獨立安裝。首次加入 TypeScript 時快取缺件，經核准下載後才完成安裝；這是開發依賴下載，未啟動安全審查或取得安全參考資料。
- 新增的三項執行回歸先核對實際程式行為；暫時將 JS wrapper 的 invocation／specDir／toolName 傳遞改壞時，三項全部失敗（另 10 項因測試篩選跳過），隨後以原始位元組還原 index.js。該突變沒有留在工作目錄。
- `npm --prefix crates/adapters/speclink-node test`：**9 檔、61 項全部通過**，涵蓋公開型別、CLI parity、Antigravity／Codex 獨立生成、無指令檔及既有 engine／Store 測試。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs`：**386 passed，0 failed，0 skipped**。
- `speclink analyze`：0 Critical、0 Warning；68 項 Suggestion 為情境範例建議，沒有將它們當成程式審查結果。
- `speclink validate`、`git diff --check`：通過。

## 同類位置與風險核對

以 rg 檢查 index.js、index.d.ts、src/render.rs 及 docs/sdk-node 兩語版的 instructions.render／store／toolName／target：

- index.d.ts 是此次實際漏值，已修並由公開編譯測試守住。
- src/render.rs 已支持五種 target；保留實作，只改舊註解。生成 binding.d.ts 不手改。
- index.js 只匯出 createEngine 與 skills，與新 delta 一致，保留原樣；突變測試後確認還原。
- createEngine、Store bridge 與文件中的 store 屬於真實儲存介面，保留，不能因移除渲染軸而一併刪除。
- 雙語文件本已列五種 target；補 API 邊界與工具身分說明。舊品質工單、checks 歷史與歷史證據中的退役 API 描述是當時事實，原樣保留。
- audit 的三個觀點：未新增可關閉安全控制的選項；預設渲染與既有值相同且不寫檔；錯誤 target／invocation 有編譯與執行拒絕測試，明確區分渲染選項與 createEngine 儲存選項。

## 接續工作

本輪不執行第 3、4 步。完整品質檢查前，仍有交接補查列出的單值 vscode、真 Git dirty／HEAD 指紋組合測試與 configuration 中英表格修補，尚未在本輪處理。兩份活動品質工單與 checks 完成歷史均保留，沒有新增 validation 輪、discard 或蓋章；下一輪 discovery 仍須遵循既有快照／票據處置，可信基準保持 faefda7dabde01b8d90974aea63f4f7a8bdd5c24，不能換成目前 HEAD。

本輪完成後程式任務為 61／61，總進度 61／62；這只是任務完成，並非整體品質通過。公司 Proxy 仍待後續包含全部修復的 main 明確提交打包後人工驗收；本次不合併、不打包、不封存、不產生 manual，也未自動提交或推送。
