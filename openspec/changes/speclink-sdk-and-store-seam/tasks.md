## 1. dispatch 動詞擴充

- [ ] 1.1 撰寫 dispatch 動詞路由失敗測試（Red）：於 `crates/speclink-node/__test__` 對 mock JS Store 斷言 `dispatch covers the remote-hostable verb set`——`archive`、`task done`、`artifact cat`、`instructions`、`language`、`config`、`spec`、`discuss` 全套各回與 fs `--json` 對齊的 camelCase payload；測試先紅
- [ ] 1.2 於 `crates/speclink-node/src/lib.rs` 的 `run_dispatch` 增補上述動詞路由（design D1: dispatch 動詞擴充），複用 `speclink-core` 既有函式與 `Store` 既有方法（含 task done 蓋 `started_*` 用的 meta 讀寫對）、不改語意；`crates/speclink-node/index.d.ts` 補記 host 可選 meta 方法；1.1 測試轉綠
- [ ] 1.3 驗證：`npm test`（crates/speclink-node）全綠，且 `dispatch(['task','done',...])` 與 `dispatch(['discuss','add-round',...])` payload 與 fs `--json` 一致

## 2. analyze/validate/drift 運算路徑

- [ ] 2.1 撰寫運算失敗測試（Red）：斷言 `Engine computes analyze/validate/drift over a host store`——`dispatch(['analyze','demo','--json'])` 回四維報告、`validate` 回 valid/errors、`drift` 回報告，形狀與 fs `--json` 一致，且過程只讀不寫 host store；測試先紅
- [ ] 2.2 於 `run_dispatch` 路由 analyze/validate/drift 為唯讀運算（design D2: analyze/validate/drift server 端運算），複用 `speclink-core` 既有 analyzer/validate/drift 邏輯；2.1 測試轉綠
- [ ] 2.3 驗證：三動詞 dispatch 報告與 fs `--json` 逐欄一致，且 mock store 僅收讀取呼叫

## 3. 契約端點與 CLI 遠端路由

- [ ] 3.1 撰寫 CLI 遠端路由失敗測試（Red）：斷言 `遠端模式 analyze/validate/drift 由 server 端運算`——CLI remote 模式對假 server（複用 `crates/speclink-remote/tests` 的 tiny_http 模式）呼叫 analyze/validate/drift 端點取報告，stdout 與 fs 模式逐位元一致；測試先紅
- [ ] 3.2 實作端點 client 方法與遠端路由（design D2）：`crates/speclink-remote/src/client.rs` 加 analyze/validate/drift 方法、`crates/speclink-cli/src/remote_commands.rs` 於 remote 模式改路由至端點；`docs/verb-contract.md` §6 涵蓋圖與 §7 對照修訂（三動詞由 client-side 改 server 端）；3.1 測試轉綠
- [ ] 3.3 驗證：fs 模式 analyze/validate/drift 仍本地運算不變；remote 模式輸出與 fs 逐位元一致

## 4. 推播通道宣告欄

- [ ] 4.1 撰寫宣告欄失敗測試（Red）：斷言 `可選推播通道宣告`——client 讀到 `events:{url,transport}` 能發現通道、欄位缺席時視為無推播且不報錯、`transport` 為不支援值時忽略並退回地基同步；測試先紅
- [ ] 4.2 於 `docs/verb-contract.md` 明載可選、傳輸無關的宣告欄 `events:{url,transport}`（掛 whoami/config metadata、push 在請求/回應契約外），並於 client metadata 讀取實作發現/退回邏輯（design D3: 推播宣告欄與引擎零推播）；4.2 測試轉綠
- [ ] 4.3 驗證：引擎本體零推播機制（grep 確認無推播傳輸碼入引擎）；宣告欄於契約文件明載為可選傳輸無關

## 5. 遠端 agent 經動詞讀文件

- [ ] 5.1 稽核並補齊遠端技能與 marker 資產（design D4: 遠端 agent 經動詞讀文件）：確認 remote 變體導引 agent 一律經 `artifact cat`／`language show`／`discuss show`／`show` 讀文件（涵蓋補完後的動詞集）、禁指本地路徑；若需改資產則三處同步（`crates/speclink-core/assets`、repo 技能實例、render golden）於乾淨樹重生 golden。驗證：remote 技能/marker 內容審視無讀路徑指示、golden diff 僅預期變動

## 6. Store 公開整合面文件

- [ ] 6.1 撰寫 `docs/integration.md`（新增）並補 `docs/sdk-node.md` 連結（design D5: Store 縫公開整合面文件化）：明載 host 實作 `Store` 的完整方法契約與以 `createEngine` 建自家 server 的路徑。驗證：內容審視確認方法契約齊備、範例可循
