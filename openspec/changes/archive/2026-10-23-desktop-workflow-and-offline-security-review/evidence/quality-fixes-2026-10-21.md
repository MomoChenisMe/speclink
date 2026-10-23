# 品質修正與 Round 2 複核（2026-10-21）

原 Round 1 的 13 項已依使用者「全部修正」指示處理。兩站都由 CLI 寫入 Round 2，findings 為空；沒有接受保留、沒有蓋章、提交或封存。原始工單備份仍在 [備份 manifest](quality-before-restart-2026-10-21/manifest.json)。原發現與凍結基準見 [Round 1 完整報告](quality-restart-2026-10-21.md)。

## 凍結與紀錄

- Base：faefda7dabde01b8d90974aea63f4f7a8bdd5c24；HEAD：f5f5179c805590e94fdf08083905874a6bb0507b，加本輪未提交修正。
- Review Round 2：validation，sha256:d0e4a1f19b869f3fff1817d68f0c7e3e1961362c31adf15488cd42ca163bfb42，36 個檔案，0 findings。
- Verify Round 2：validation，sha256:7f8caa4519c16f2666d67906c51dd100f91af3d219088668dd057c2f90fa67df，36 個檔案，0 findings。
- 審查規則：extend（12 項基本規則）＋專案 CRITICAL 1、WARNING 4、SUGGESTION 0。
- Standards、Correctness 與 Verify 三個唯讀軸平行定向複核；三者均回報原項解決、未發現修正直接引入的回歸。未重新探索原變更的全部範圍。

## 逐項修正

| 編號 | 修正及證據 |
|---|---|
| R1 | CLI Show 沿用 Host review_view；Local／Remote 共用 print_security_review。 |
| R2 | 發佈報告先取索引比較快照，再解析索引；一次性 TestStore hook 測試確認交錯寫入回 revision_conflict，已完成審查與 metadata 不遺失。新增測試先重現舊版錯誤成功提交，再驗證修正。 |
| R3 | ArchiveChecksPlan 保存 expected_meta，於 FS 共用寫入鎖內核對；規劃後修改 metadata 的測試先重現舊版錯誤覆蓋，再確認拒絕且保留最新資料。 |
| R4 | Node 切 args[2..] 前要求存在子命令；缺少 security review 子命令回輸入錯誤，非 panic。 |
| R5 | 手動更新備援先比較已安裝版本；沿用 compareVersions 並支援第四段，同版／較舊候選不提示更新。三個新增版本案例先重現失敗，再轉綠。 |
| R6 | 僅正規化 macOS 系統 /var、/tmp、/etc 實際指向 /private 的別名；Host／FS 共用。系統別名可用、使用者 symlink 仍拒絕，原路徑相關 FS／Host／CLI 失敗排除。 |
| V1 | 封存索引保留 outcome=omitted；Local／FS／Remote／UI 皆辨識。UI 顯示政策省略且不讀不存在的文件；八種保留組合、封存交易與 Remote 摘要測試更新。 |
| V2 | KB 人眼輸出補 projectKey／policyDigest／standardId／version／dataSha256；兩端審查摘要共用完整欄位與決策輸出，CLI 用實際 JSON 值比對。 |
| V3 | DeepEvidence.endLine／function 改必填，拒缺欄／null／空白／錯範圍；提交測試先重現缺 endLine 仍成功，再轉綠；FS fixture 改有效欄位。 |
| V4 | 未知 reason 保留 server message、reason 及 HTTP 狀態；typed_client／client_errors 兩組契約測試同步。無法解析的 envelope 仍沿原 fallback。 |
| V5 | 本機測試完成 CONNECT→驗證 TLS→取得 manifest→updater::check 回傳候選。僅測試 client 信任 fixture CA，不關閉憑證或主機名驗證，不連外。 |
| V6 | 同一測試使用 4 個完成寫碼任務、1 個未完成人工任務、Round 1 findings／Round 2 clean，斷言 reviewed_tasks_total=5。 |
| V7 | 同一測試使用 required=true、零 finding、不完整及缺必要 check，斷言 blocked、incomplete_security_review 與 necessary_checks_not_verified。 |

## 實際測試

- cargo build -q -p speclink-cli：通過；Desktop 正式 build 與 TypeScript：通過。
- Core lib：1,077 通過、1 ignored；render_golden：63 通過。
- FS：52 通過、1 ignored；另 integration 1 與 21 通過。
- Host lib：163 通過（序列執行）；Remote security history：1 通過；其他 Host integration 5 通過。
- Desktop core：358 通過（含真實 loopback CONNECT／TLS）。
- Remote client：20 單元及 104 integration 通過。
- CLI security：20 通過，原 11 個路徑失敗已排除。
- Node：先重建 addon，再 56 通過。
- Desktop：1,309 通過；UI：1,343 通過；省略 UI 最終定向重跑 40 通過。
- scripts：386 通過；git diff --check 通過。

## 限制與後續擴充

- 擴大跑 Core integration 時，既有 skill_authoring::every_skill_source_follows_the_authoring_rules 失敗：archive 強調字 5 超過上限 4；security-review 上限未登記（2 個）；verify 為 22 超過上限 20。原稿及守門檔非本輪 13 項修正內容，未以修改上限掩蓋，不能宣稱整個 CI 通過。
- Host 平行測試曾在 reader lease drop 後短暫遇 busy；全組改序列執行後 163 項通過。未宣稱這個平行執行現象已修正。
- 沒有跑完整 test:all；未完成範圍仍由 CI 守門。Windows／UNC／NAS 未實機執行；公司 Proxy 12.3 依使用者指示待打包後驗收，不代勾。
- Round 2 完成後，使用者追加 Antigravity／目錄標籤，已經 ingest 擴充 proposal／design／specs／tasks。依 quality 的 scope changed 規則，本次品質流程到此結束；上述清零只適用這 13 項修正，不能作為新增工具範圍的審查通過證明。新擴充見 [Antigravity 實作紀錄](antigravity-tools-2026-10-21.md)。
