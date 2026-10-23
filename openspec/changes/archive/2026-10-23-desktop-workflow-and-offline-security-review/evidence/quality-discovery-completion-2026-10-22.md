# 前輪未完成覆核的補查（2026-10-22）

## 本次授權與範圍

使用者選擇「補完前輪未完成的覆核」。本次接續原 discovery 的未完成項目，沒有將完整探索偽裝成第三輪 validation。沿用 trusted base `faefda7dabde01b8d90974aea63f4f7a8bdd5c24`、HEAD `46803b2b3af11719aadde6266a2370c2d7dff367` 及原 229-path frozen patch `sha256:b7aa741a6239f224efea728cda7b1fed2392552f8fec6610522b3077da47fc2a`。

原兩項 CRITICAL 與一項 SUGGESTION 的修復、測試及 Round 2 結果，仍見 [前輪紀錄](quality-ci-followup-2026-10-22.md)。兩個活動工單均保留 Round 1、Round 2；本次沒有修改工單、追加 validation、蓋章或刪除歷史。Round 2 的 clean 僅表示原問題的修復複驗通過，不能替代本份補查結論。

`$null` 是使用者既有輸出紀錄，沒有閱讀、修改或提交。生成資產按路徑與既有生成契約定位，不重新掃描整份生成內容。補查發現原 manifest 將 Node `index.d.ts` 誤分類為生成檔；實際 build 產生的是 `binding.d.ts`，`index.d.ts` 是 package 對外的手寫型別契約，已補查並確認下列問題。

逐檔重新計數並保存 [逐項追溯表](quality-spec-coverage-2026-10-22.md)：17 份規格、46 Requirements、**133 Scenarios**、7 Examples。先前 143 是加總錯誤，沒有額外十個未檢情境。需求 45 PASS／1 partial；情境 125 PASS／3 partial／5 manual（四項文件核讀、一項公司延期）；範例 6 PASS／1 partial。PASS 僅指實作與具體證據映射，不是全數實機驗收。子檢查完成逐情境表後因額度中斷，主流程另核對總數並補存七個 Example 的實際測試，沒有把中斷本身當作通過。

## Review：規則與結果

套用 `extend` 的 12 項 baseline smells，以及專案規則 CRITICAL 1、WARNING 4、SUGGESTION 0。

Standards 子檢查完成原 manifest 所列剩餘變更行與必要上下文，涵蓋 Desktop、UI、CLI、Node、Core、FS、Host、Server、Protocol、文件、技能原稿與測試。既有生成技能僅作來源／版本對照；產品程式與技能原稿分開判斷。沒有尚未檢查的非生成 frozen hunks，沒有新增確定的專案規則違反。精確 patch 對照集合由原 manifest 保存：001–020、022–031、035–098、100–136、148–159、161–211、213–228；其中 `.github/skills` 是生成技能，不當作另一套產品邏輯。

Correctness 子檢查補完 Host registry 與下載限制、FS 共用標準根／唯讀 pin／generation／互斥鎖／原子發佈、KB 準備與失敗保舊、CLI Local／Remote 深度報告及接受流程、Remote client／Host 當次來源核對與直接測試，該分工沒有新增可確認錯誤。主流程補查更新、排序、設定、檢查紀錄、提案顯示、工具生成與清理、SDK 接口及文件，確認以下公開型別問題；原已修復問題不重報。

### [WARNING] Correctness：TypeScript 無法使用 Antigravity target

位置：`crates/adapters/speclink-node/index.d.ts` 的 `RenderOptions.target`。

`package.json` 的 `types` 指向此檔，而型別值域仍只有 `claude|codex|copilot|neutral`。Rust 已接受 Antigravity，執行測試也使用它，但 TypeScript 使用者照新文件呼叫時無法編譯。

最小重現（只在 TEMP 建測試檔，沒有改產品）：

```ts
import { skills } from "D:/dev/speclink/crates/adapters/speclink-node";
skills.render("apply", { target: "antigravity" });
```

執行現有 TypeScript compiler：`node node_modules/typescript/bin/tsc --noEmit --strict --skipLibCheck --moduleResolution node --target ES2022 <TEMP case>`，exit 2，唯一錯誤為 `TS2322: Type '"antigravity"' is not assignable to type '"claude" | "codex" | "copilot" | "neutral"'`。

同一編譯指令只把 target 換為 `copilot` 的對照案例 exit 0，確認錯誤來自公開值域漏項，而非測試路徑或 compiler 設定。

建議將公開型別值域同步為五種 target，並加入消費公開宣告檔的編譯測試；Vitest 執行時轉譯並不檢查此宣告，因此先前執行測試通過不能代替這個案例。此項為 must-fix，Verify 亦不符合「Antigravity 原生渲染與 CLI 同源」對 SDK 的可用契約。

這是原變更的新發現，與 11 檔修復無因果關係，且有實際錯誤行為的失敗重現。依兩站對 unrelated late findings 的規則，本次以 **scope changed / failed** 結束：保留票據與已完成歷史，不寫入假冒的 Round 3 validation，不蓋章。建議處理下列規格選擇後，另開具明確範圍的 discovery，不能把原 Round 2 clean 當作整體通過。

本結論是品質檢查，不是另一次安全深度審查。安全政策仍 disabled，沒有自動下載、執行深度 skill 或宣稱安全認證。

## Verify：須優先決定的規格落差

### Node SDK 渲染契約仍含已不存在的介面

- 規格：`specs/node-sdk/spec.md` 的「渲染 API」要求 `instructions.render(options)` 與 `options.store=fs|remote`；第一個情境還明確傳 `store: 'remote'`，最後追加情境要求保留此語意。
- 現況：`crates/adapters/speclink-node/index.js` 只匯出 `createEngine`、`skills`；`src/render.rs::RenderOptions` 只有 `target`、`invocation`、`spec_dir`、`tool_name`，沒有 `store`。`__test__/render.spec.ts` 測技能渲染與 CLI 同源，並明確測 init 不產生指令檔；沒有提供規格描述的指令檔 API。
- 這是規格與實作的既有落差，不是本次 11 檔修復造成的回歸。不能因現有測試通過而宣稱該需求完全符合；也不能擅自復活已退役 API 或直接改寫規格。
- 建議處置：依目前 SDK 的技能檔 API 更新本變更 delta，將公開選項寫為 `target`、`invocation`、`specDir`、`toolName`，移除 `instructions.render` 與 `store` 承諾，對應情境使用實際可呼叫的參數；保留 Copilot／Antigravity、錯誤值域與 CLI 同源要求。若使用者仍需要舊 API，則應先補設計與任務再實作，不能只改測試掩蓋。

此處需要修改規格，列為本次補查的優先處置事項。沒有寫入原 Round 2，也沒有以空 validation 輪將它洗成通過；目前不建議收尾蓋章。

供使用者選擇的具體規格方向（尚未套用）：

> SDK SHALL 提供 skills.list() 與 skills.render(name, options)。options 包含 target（claude｜codex｜copilot｜antigravity｜neutral）、invocation（cli｜tool-call，預設 cli）、specDir（預設 openspec）與 toolName（neutral 專用，預設 speclink）；渲染不寫檔，與 CLI 對等技能內容同源。不再承諾指令檔渲染介面或 store 軸。未知 target 與技能仍明確拒絕。

相應情境移除 `store` 輸入；中性 tool-call 保留工具呼叫措辭與無工具專屬前綴的斷言，CLI 同源情境比較相同 specDir 下的技能檔。不得保留沒有實作依據的「Remote 不含本地規格路徑」承諾。Copilot／Antigravity 的獨立目錄及語法要求保持。此方向會變更 Requirement 的字面契約，須先經使用者決定，再依 ingest 更新提案文件並重新驗證範圍；本次沒有直接改 delta。

## 測試追溯與文件備註

這些是補查中找到的既有覆蓋／文件缺口，沒有以它們推論已發生安全、資料遺失或錯誤行為，也不當作修復直接造成的新回歸：

1. workspace-tools 的 `--tools` Example 表要求單值 `vscode`。目前未知工具測試有 `claude,vscode`，但未找到單值 `vscode` 的同輸入案例；應補精確列值，不宣稱表格所有列皆已測。
2. security-review-report「有 dirty 檔與無 Git」：已有 `commitSha=null`、實際內容指紋與來源新增／刪除／修改測試；未找到在真實 Git 專案修改已提交目標後，同時驗 HEAD 參考及 dirty 內容 digest 的完整案例。這是組合覆蓋缺口，沒有證據顯示現有程式錯讀 HEAD 內容。
3. `docs/configuration.md` 與 `docs/configuration.zh-TW.md` 的工具表在 Antigravity 列後插入散文，自訂工具列落在表格外；`skills_dir` 規則表也漏列 `.agent/skills`，雖然上段文字已說明此目錄衝突會被拒。建議把完整表格接回並同步列出第四個保留目錄。

## 本次執行的驗證

- `npm test -w packages/ui -- proposalDoc baselineSmellHelp changeDetailPage archivedDetailPage securityReviewPanel`：5 檔、184 tests passed。
- `npm test -w apps/desktop -- updatesPage updater projectRail tabs store checkRecords securityDataPage projectSettingsView workspace App`：16 檔、720 tests passed。
- 前端合計 904 tests passed，0 failed。部分既有 React 測試輸出 act 提醒，未造成測試失敗。
- `git diff --check` 通過。沒有程式修復，所以沒有重跑前輪已通過的 2744 項受影響 Rust 測試與 scripts；其範圍與跳過原因保留在前輪紀錄，未把舊 HEAD 遠端 CI 冒充本次未提交修復的 CI。
- 公司 Proxy 的 `[M]` 12.3 仍未完成：先合併 main、從明確 commit 打包，再由公司安裝驗收。實際 NAS／SMB 也未實測；本次程式與 fixture 檢查不能代簽。

補查結束時只新增證據；程式修正尚未提交／推送，沒有合併、打包、封存或產生 manual。使用者隨後明確要求換機提交與推送，交付後狀態以 [換機說明](handoff.md) 的 2026-10-22 段及 Git 為準。
