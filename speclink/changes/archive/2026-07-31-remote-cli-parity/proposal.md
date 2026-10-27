## Why

Remote（team）模式的 CLI 動詞覆蓋有七處缺口：四個 discuss 動詞明文拒絕（--slug、discard、link、seal），show 與 in-progress add 靜默讀寫本機 store 給出錯誤結果，demo 靜默在本機建立資料。已造成實際事故：agent 在 remote 模式被拒 --slug 後 fallback 產出全中文 slug 的 0 輪空記錄，且 remote 無 discard 可清。引擎層對全部動詞早已支援，缺口只在 remote 傳輸鏈（protocol → server 路由 → remote client → CLI），同一條管線一次接通成本最低。

目標使用者：以 AI 代理跑 SDD 的開發者於 remote（team）模式操作 speclink CLI；對應 discuss（記錄與清理）、apply（開工標記）、show（檢視）等 workflow 階段與技能。

## What Changes

- remote 模式 speclink discuss new 支援 --slug 覆寫：與 fs 模式共用引擎驗證（純 ASCII kebab-case；非法值非零 exit code、stderr 說明原因、不建立記錄），合法值由 server 端以該 slug 建檔。影響 speclink-protocol（建立請求增列選填 slug 欄位）、speclink-server、speclink-remote、speclink-cli。
- remote 模式新增 speclink discuss discard <slug>，選配 --force：引擎 guard 於 server 端生效——有輪拒刪（需 --force），0 輪直接刪；exit code 與 fs 模式一致。
- remote 模式新增 speclink discuss link <slug> <change> 與 speclink discuss seal（與 fs 模式同語意），使 ingest 收尾與討論連結流程可在 remote 完成。
- remote 模式 speclink show 改為 server 讀取組合（以既有讀取 API 取 change meta、artifact 與規格文件），不再靜默讀本機 store；人眼與 --json 輸出形狀與 fs 模式一致。為此 change 讀取回應（GET /changes/{name}）增列選填 created、fromDiscussions、deltaCapabilities（camelCase、缺席即省略；舊 client 不受影響）。
- remote 模式 speclink in-progress add 路由至 server：started_at 與 started_by 落 server 端 change meta，started_by 取 server 認證身分；stdout、stderr 與 exit code 維持 parity 凍結形狀（含對不存在 change 的靜默成功）。
- wire 的變更清單摘要新增選填 startedAt 欄位（camelCase，缺席時預設，舊 client 不受影響）；desktop remote 看板與系統匣的欄位推導補回「已開工即進行中」判定，「完成數 > 0」的 fallback 保留（涵蓋繞過工具的寫入路徑）。
- **BREAKING** remote 模式 speclink demo 改為明確拒絕：非零 exit code、stderr 說明 demo 僅限本機模式，不再靜默於本機建立 demo change。僅影響誤用情境。
- docs/verb-contract.md 正典契約文件同步補新端點與 payload 形狀。

相容性影響：

- fs 模式所有指令的人眼與 --json 輸出逐位元不變（迴歸對照不受影響——改動僅及 remote 分支、server 與 protocol）。
- remote 模式四個 discuss 動詞從報錯變成功，輸出形狀依 verb-contract 既有要求與 fs 模式一致；既有使用者無需遷移。
- remote 模式 show 從「讀本機、回錯誤結果」變「讀 server」，輸出形狀不變、內容由錯轉對。
- remote 模式 in-progress add 可見行為不變（靜默、exit 0），副作用從「無」變「server meta 蓋章」。
- remote 模式 demo 從靜默成功變非零 exit——唯一破壞性行為變更。
- wire DTO 僅新增選填欄位，新舊 client 與 server 互通不受影響。

## Non-Goals

- 不啟用 wire 保留槽 lifecycle 與 claimed_by 的 server 端狀態機或 durable 認領——需先有權限與管理基建，屬另一功能（討論 Round 5 排除）。
- 不改引擎 slugify 對非 ASCII topic 的 fallback 行為（保留 CJK 維持現狀）。
- 不修 remote 模式 bulk archive 的拒絕（有逐一 archive 替代路徑）。
- 不動刻意設計的拒絕：status 與 instructions 的 --schema 拒絕（server 政策決定 schema）、claim 在 fs 模式的拒絕。
- 不含 wad-old-web 既有空記錄的清理動作本身（本變更落地後以 discuss discard 執行）。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `verb-contract`: 動詞契約涵蓋面擴充——discuss 的 slug 覆寫、discard、link、seal，in-progress add，show 讀取組合，demo 於 remote 的明確拒絕
- `discussion-docs`: --slug 與 discard、link、seal 在 remote 模式的可用性（與 fs 同語意）
- `change-lifecycle`: started 站經 remote 通道寫入（server 認證身分歸屬）與 startedAt 上 wire
- `desktop-app`: remote payload 的看板欄位推導補回已開工判定
- `server-verb-api`: 新增討論 discard、link、seal 與 change in-progress 端點；討論建立收選填 slug

## Impact

- Affected specs: verb-contract、discussion-docs、change-lifecycle、desktop-app、server-verb-api
- Affected code:
  - New: (無——全部落於既有檔案)
  - Modified: crates/speclink-protocol/src/command.rs、crates/speclink-protocol/src/query.rs、crates/speclink-server/src/app.rs、crates/speclink-server/src/routes.rs、crates/speclink-server/tests/discussion_routes.rs、crates/speclink-server/tests/verb_api.rs、crates/speclink-server/tests/query_routes.rs、crates/speclink-server/tests/backup_e2e.rs、crates/speclink-server/tests/e2e_cli.rs、crates/speclink-remote/src/client.rs、crates/speclink-remote/tests/typed_client.rs、crates/speclink-cli/src/remote_commands.rs、crates/speclink-cli/src/commands.rs、crates/speclink-cli/tests/discuss_slug.rs、crates/speclink-cli/tests/remote_write_path.rs、crates/speclink-cli/tests/remote_verb_parity.rs、apps/desktop/src-tauri/src/remote.rs、apps/desktop/src-tauri/tests/remote_data.rs、apps/desktop/src/__tests__/remoteDataSource.test.ts、docs/verb-contract.md、docs/verb-contract.zh-TW.md（前端 remoteDataSource.ts 為 wire 直通、零改動——startedAt 隨 DTO 自然上行，僅以測試釘住）
  - Removed: (無)
