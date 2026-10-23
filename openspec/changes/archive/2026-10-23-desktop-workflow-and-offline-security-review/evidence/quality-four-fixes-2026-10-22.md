# 四項品質問題修正 — 2026-10-22

使用者已授權全部修正並重跑複核。本輪沿用原兩站工單，修正程式以符合既有規格，未改 proposal、design、specs 或 tasks，未新增或重開任務。未蓋章、未提交、未合併、未打包、未封存。

## 修正及 RED→GREEN

1. 更新標籤：新測試先對原作者 v0.10.0 失敗，再按來源解析 Fork desktop-v／舊 desktop-...-build.N 與 Upstream v；舊 fork 版本轉四段顯示。保留固定 repo URL、桌面資產、draft／prerelease、最多100筆與 CLI-only 排除。
2. 版本比較：實際 build.3 測試先有10項失敗，之後將四段版號沿打包規則轉為 -build.N，再按 SemVer core／預發行識別比較；正式版高於相同core的預發行版，忽略 build metadata。共用 whatsNew 比較器，不另建一份 adapter 比較器。新增 .2／.3 不更新、.4／0.10.1 可更新、beta 數字排序與反向比較案例。
3. CWE ZIP：既有跨 Local checkout／Remote 發布整合測試加入官方下載形狀的 ZIP＋XML 後，submission_context 原先失敗 binary。ProjectInput 新增受控原始位元組讀取，Host 以原始位元組驗 SHA，保留 ZIP／XML 與全部指紋；檔案遭竄改仍拒絕。讀取仍沿 AuthorizedProject 的 handle、拒連結／路徑穿越與大小／取消限制；原文字掃描仍拒絕 binary／unsupported encoding。Spy 補相同 trait 接點並保持 disabled 時不可讀知識庫。
4. 缺 metadata：FS 真實 Store 的兩站測試先因 revision_conflict 失敗。StampGate 保留 Option，區分缺檔 None 與存在但空檔 Some；文字編輯才取空字串，CAS 仍比較原狀態。測試兩站缺檔／空檔成功保存完整紀錄；gate後並行建立metadata仍拒絕，保留對方內容與原工單、不留索引。

## 同類排查

rg 查兩個來源的 tag 解析、compareVersions 全部 caller、security_kb 的文字讀取與 station 原 meta 預期值。版本比較由更新備援與日誌判定共用，本輪同修；manifest.json 仍為文字，KB manifest 列出的檔案一律以 bytes 驗證。兩種 stamp API 共用同一 StampGate/write_stamp；未另做其他站別實作。原有無關 unused import 不刪。

## 驗證

- cargo build --offline -q -p speclink-cli 通過。
- cargo test --offline -q -p speclink-core -p speclink-fs -p speclink-host -- --test-threads=1 通過：Core 1081＋94（1 ignored）、FS 54＋1＋21（1 ignored）、Host 163＋5＋1。
- cargo test --offline -q -p speclink-desktop-core：359 通過。
- CLI integration filters：review_verbs 49、verify_verbs 35、archive_readiness_gate 10、security 20，全部通過。
- npm test -w @speclink/desktop：1322；npm test -w @speclink/ui：1343，全部通過。
- npm run build -w @speclink/desktop 通過，保留既有 chunk size 警告。
- npm --prefix crates/adapters/speclink-node run build 後 test：57 通過。
- node --test scripts/*.test.mjs scripts/*/*.test.mjs：386；validate 與 git diff --check 通過。
- CLI 部分測試先在沙箱因 mock listener／安全資料路徑權限失敗，依實際權限需求升級執行後全數通過，沒有放寬產品控制或略過失敗。

沒有跑完整 test:all（由 CI 守門）、公司 Proxy／NAS／Windows／Antigravity 實機。人工 12.3 仍未完成；security policy disabled，未另啟 OWASP 深度審查。

## 複核範圍

兩站為 validation，11 paths。Review patch sha256:7ab42a618f7c07ad55b4463c294a1cb20219d48caed3454e22744ba51a870419；Verify patch sha256:5d412f892ce6a89452f7db827d3ff3f59938100f163c71477f563f0dc1b89999。工具的 new attribution 在少數檔案亦帶入已於 discovery 捕獲的舊內容；以原 frozen snapshot 的 afterText／afterHash 明確辨認既有內容，補 exact movement 對照，只判修正與直接回歸，不把舊內容當本輪新工作。
## Round 2 複核結果

三路唯讀複核已完成。Standards：原四項修正均可歸屬，0 findings；Correctness：4 resolved、0 unresolved、0 新回歸；Verify：2 resolved、0 unresolved、0 新回歸。全部限定複核項目已完成，沒有新增建議或將舊內容重新列項。

已由 CLI 依 review→verify 順序 add-round 寫入兩站 Round 2，再以 show --json 回讀，均為 phase=validation、findings=[]，兩站各保留完整 Round 1＋Round 2。未使用 --accept，沒有接受或略過任何原 finding。

兩站使用的 patchHash 與本記錄前段一致。複核後再比對11個檔案與frozen afterHash，內容均未移動。可以進入兩站的 closing stamp call，但依使用者要求與 quality 的 clean-round pause，本次不蓋章。人工 12.3 保持未勾；它不阻擋品質章，正式封存仍等打包後人工完成。

檢查限制：代理未自行重跑測試；主線已執行前段列出的 build 與受影響測試。完整 CI、Windows／UNC／NAS、Antigravity 與公司 Proxy 實機未執行，不把自動測試通過等同公司驗收。
