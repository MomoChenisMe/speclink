# Review / Verify 檢查覆蓋紀錄

- 變更：`desktop-workflow-and-offline-security-review`
- 基準提交：`faefda7dabde01b8d90974aea63f4f7a8bdd5c24`
- 凍結程式碼：`sha256:2d1997aac1a9124a56397b62ebed064d0fbd0534239b0250ea041e1e328aa29b`
- 檔案：242；變更段：884；實際閱讀：674；生成檔／二進位依路徑核對：210；未完成：0。
- 本輪補完 Correctness；Standards 與 Verify 沿用同一凍結範圍的前輪檢查。未修程式、未重跑測試、未蓋章或封存。
- 生成檔／二進位的 path-only 不表示內容已逐字檢查；每項排除理由列在下方及 JSON。
- 原始逐段資料與發現見同目錄 `quality-coverage-20261023.json`。

## 發現與決策狀態

- Correctness：本輪三份分工報告均無確認缺陷。
- Review：沒有 CRITICAL／WARNING；沿用一項不阻擋蓋章的 SUGGESTION，未擅自修正。
- [SUGGESTION] packages/ui/src/components/CheckRecords.tsx — Standards: possible Duplicated Code: `startedAt` and `startedOrder` repeat the same path split, filename extraction, and 14-digit timestamp regex/slicing; extract one timestamp parser so display and sorting cannot drift.
- Verify：前輪逐項對照未發現明確的規格落差；任務 66/66（寫碼 65/65、人工 1/1）完成。

## Correctness 逐段紀錄

### `.github/skills/speclink-apply-with-worktree/SKILL.md`

- `65849a490607cbcb34ca90185d8a0d2d7e3096f66cd1f3f3a56820391d323b82` L5–11 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `.github/skills/speclink-apply/SKILL.md`

- `9ab4fb35d68fc8961d0c80b12afdd0094c902b16e00386b3d8a1dcf440f91fd0` L5–11 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `.github/skills/speclink-archive/SKILL.md`

- `2e9475b4b01819c6d80ae297b8771a2171fc47c132caf4e3df0718ea3b2166a9` L5–11 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `39b2bd3226ee986c58022c6e564f2daa28d86c86bede5e53b12c3c7ff4fdfd03` L13–24 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `0f8d84954891a1a3d7ddc7fb711ae402357487bb7358b02c6b2324879165102a` L38–44 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `.github/skills/speclink-audit/SKILL.md`

- `69e6bc554c24324fa868d655196767e466c46941e87b57db20638d1a3cc2238b` L5–11 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `.github/skills/speclink-baseline/SKILL.md`

- `173e39d85f0d3939b7562f494bd7bffcbfef4d1d3880ae9fa09f6e2f4f7c88e6` L5–11 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `.github/skills/speclink-commit/SKILL.md`

- `59cc838d064de3b12bb541726df750adf13ff396b651fd6e86b32bc94851b227` L5–11 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `.github/skills/speclink-config/SKILL.md`

- `0ae7280203f081f84205911acd77b0baefcb6b3628cc2446fcdc67d3fcb04ba0` L5–11 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `f497db4a4199d0229bbc821739f9303bb5b77ac0fafcf332b7d520274c7063c3` L248–255 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `.github/skills/speclink-discuss/SKILL.md`

- `55c9fa3deb1b042be660b2e36c6958d6368db28ffdc15b6ad993c25325330d66` L5–11 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `.github/skills/speclink-drift/SKILL.md`

- `ef03ca016129fa605b7ad2c1fb3a1707df18ee1d9c09803fe8e80bba7664d9e4` L5–11 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `.github/skills/speclink-improve/SKILL.md`

- `8e76e7af9bb0317668fbcf134d9eb1462907f63a3eb3b70c6331cb47c13a7fea` L5–11 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `.github/skills/speclink-ingest/SKILL.md`

- `44a554775861ff5d967eb4cc6928b780e0912dab033b7a80d9d2f0521569a176` L5–11 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `.github/skills/speclink-manual/SKILL.md`

- `015089ff2f7d117636af83d0a056335b1a15b0fe9910f12da8ec1b24b81a9365` L5–11 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `.github/skills/speclink-propose/SKILL.md`

- `c5904af224c9a47e0ca34a0317075bea9dc55284afea401b1f46170868dcadad` L5–11 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `2e7c4e60e57b60b8d93d53adfdc9a8c1881bf2a8c3cb742670daf1bb6d5b4896` L169–175 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `a54b21a93fa227ab284c2149a5e9de5d0374811c1af5db88ed835a25a6d00644` L462–469 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `.github/skills/speclink-quality/SKILL.md`

- `a27e93967b4b010e9b3cfba65651a0f4594892474f8ce1c6ee99bdfed027fae3` L5–11 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `.github/skills/speclink-review/SKILL.md`

- `f9e47f28b5f5846a5c1cba7ce8eb506983549a272bacf02b6e10e06454ba3793` L5–11 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `e86bbe0b5362aaa7635299a530365fa236c04e9f92d47fc1fafc0117ce3c2e5c` L265–271 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `.github/skills/speclink-security-review/SKILL.md`

- `0f22d8c10c3d8c14d15073b0ff6a95227b39ff03c31675241c62d821065afa05` L1–84 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `.github/skills/speclink-trace/SKILL.md`

- `d1d851245e69db20b1ef111f549b9c426cd6ce75e46b7f281caebfd7bcd1a91f` L5–11 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `.github/skills/speclink-verify/SKILL.md`

- `a172628713628f16b26e05a882d966d79afa1ebfcbd9cab94d31965be02244ef` L5–11 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `065654f455e6314c72fafc04a2fc7a0b949e17413a9c082f68cf3e37f35caad3` L54–60 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `97200fe8cbf93ac88bb6ea5a3dd00a0fd64ca4705452b7e3e330c326626108c3` L100–106 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `d9f8f5cf4e583564cbbf9bcea5fdf317fe30dc29c10a630762f481f17badd676` L325–338 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `.github/skills/speclink-worktree-merge/SKILL.md`

- `d902ff09bc347c47a72fb1b0b19df79e91ea17a6d041705d5b8c3520d6259e48` L5–11 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `CONTRIBUTING.md`

- `8ce76cf054bb83bd593b93ef3be4ca94e8549d273d09a6625268bdb3fda9c1ca` L55–64 — **reviewed**：Hunk 55: Read contributing doc hunk: contributor/account attribution and coauthor instructions do not alter authorship automatically; source links supplied as references.

### `Cargo.lock`

- `69aaeadd7b2fa5e9f9f8bfa6b50d779aabc2368996a625d034e78530c8f8d861` L4508–4514 — **path-only**：Generated dependency lockfile; excluded from source correctness inspection per brief discipline; path-only, not claimed reviewed.
- `a5d31f8294ac084f949d4552e21b044ceb6b2dd28a0e0e321065562a4b39045c` L4524–4535 — **path-only**：Generated dependency lockfile; excluded from source correctness inspection per brief discipline; path-only, not claimed reviewed.
- `1f5d7151ec27d059f691408489efc95c36b48020f5af12b1030a5c4da70ae6c5` L4544–4550 — **path-only**：Generated dependency lockfile; excluded from source correctness inspection per brief discipline; path-only, not claimed reviewed.
- `455d7f29ad5397ed4552d4e18fd1bdd0d372cc23105620d396f0607c1c93059a` L4580–4600 — **path-only**：Generated dependency lockfile; excluded from source correctness inspection per brief discipline; path-only, not claimed reviewed.
- `bdcc044b3fbc85dde8be8e1efe22d2936c2ef744f163eef7a5cf9d2b40a02b74` L4604–4610 — **path-only**：Generated dependency lockfile; excluded from source correctness inspection per brief discipline; path-only, not claimed reviewed.
- `58786406e9045bc49f54eb02a233a721f86d13adc7cdcace94c4a30724301ac6` L4621–4627 — **path-only**：Generated dependency lockfile; excluded from source correctness inspection per brief discipline; path-only, not claimed reviewed.
- `27fd9607a17b4d30d5d974afe304130e18514eff5aba133fa0dbdc3ce65e5a8b` L4637–4653 — **path-only**：Generated dependency lockfile; excluded from source correctness inspection per brief discipline; path-only, not claimed reviewed.
- `653ab2d56183ab30fe43fb9873dafc80dc98dde74be998bd5b917a206621c3bb` L4662–4668 — **path-only**：Generated dependency lockfile; excluded from source correctness inspection per brief discipline; path-only, not claimed reviewed.
- `ba252b91a4ab8109216f001f85e893929ad87dac73eca1b3c6670c130770af42` L4671–4677 — **path-only**：Generated dependency lockfile; excluded from source correctness inspection per brief discipline; path-only, not claimed reviewed.
- `185e017c5cb06d995423eea1deff35087607370234fb11040f24f22753121fcb` L4690–4701 — **path-only**：Generated dependency lockfile; excluded from source correctness inspection per brief discipline; path-only, not claimed reviewed.
- `ae20bd6715cc570dd25e02ee29f032e9595d8ef64c44361993046a1e263177aa` L4728–4734 — **path-only**：Generated dependency lockfile; excluded from source correctness inspection per brief discipline; path-only, not claimed reviewed.
- `9dff6f82525be8a94c655233b3d4350666a1268e28870bc63e30203f6dcef0e4` L4737–4743 — **path-only**：Generated dependency lockfile; excluded from source correctness inspection per brief discipline; path-only, not claimed reviewed.
- `9cdc2dbae10bb7f86e182f6c3e84efe7306a2a3394bdc71656e71ebbee2c4ec4` L4750–4756 — **path-only**：Generated dependency lockfile; excluded from source correctness inspection per brief discipline; path-only, not claimed reviewed.
- `2fd3ab49dcf542abac07e7e9a39e7c67ddcb160105adf5bf63fc3f21f73b35c8` L4762–4768 — **path-only**：Generated dependency lockfile; excluded from source correctness inspection per brief discipline; path-only, not claimed reviewed.

### `README.md`

- `0e6d7d193889ab00732d5760b7c53b4fc093e4554fff2eeca71dfaf55cca5549` L32–38 — **reviewed**：支援工具文件新增 Antigravity。
- `0b2761d385f8cc10f601b0d5f66272cd8cd5d94e18bd78f0bcdb23ab347f35b3` L52–60 — **reviewed**：init/tools 及 invocation 說明與測試一致。
- `1cedebdf8add449e6e925a9365b7a337c1bce0f716174afda88d804e06b64c20` L78–84 — **reviewed**：features 說明工具 invocation。
- `44fe9004c56b10bd2bd3f2d493aaa561101a2aae2480f1d907036b777f688038` L223–229 — **reviewed**：產品狀態列新增支援工具。
- `e74015685d6ddc23225208ae1735c49d13198bae5e14e8376206f63d2954a64c` L246–254 — **reviewed**：acknowledgments 清楚區分 fork/original，連結格式有效。

### `README.zh-TW.md`

- `a7fef43900c25665024ef632da40f336f5bba1f5d5a78ae30f5b82f61670c043` L32–38 — **reviewed**：Hunk 32: Read README hunks: four-tool matrix and command prefix descriptions match renderer; contributor/fork attribution links are explicit; no executable contract error found.
- `c7a4d4ac59dc1f31d7f5cace2bda75136cce388fc73acaf260a058aab3519b4a` L52–60 — **reviewed**：Hunk 52: Read README hunks: four-tool matrix and command prefix descriptions match renderer; contributor/fork attribution links are explicit; no executable contract error found.
- `67889f97907da79f280f2f44a892c35398e00933a454670fc5e50011b4b54d6e` L78–84 — **reviewed**：Hunk 78: Read README hunks: four-tool matrix and command prefix descriptions match renderer; contributor/fork attribution links are explicit; no executable contract error found.
- `a8e0d01ef5ff98d3a911e2561c6f7d1593b95989586f414edf52257233b62cc2` L223–229 — **reviewed**：Hunk 223: Read README hunks: four-tool matrix and command prefix descriptions match renderer; contributor/fork attribution links are explicit; no executable contract error found.
- `d602c174e94e3e64e0fd8ec6caa7277896f1534f7ecb40048fde0ab6777a4bcc` L246–254 — **reviewed**：Hunk 246: Read README hunks: four-tool matrix and command prefix descriptions match renderer; contributor/fork attribution links are explicit; no executable contract error found.

### `apps/desktop/core/Cargo.toml`

- `03c2835dde9414c77612025f4e6d0a931a4a5abd4e403665f675d5171cbce1a5` L9–15 — **reviewed**：core 新增 protocol path dependency 與使用 wire types 相符。
- `8d2c0fdf4652b119a0337b478650a66a2142147b9d8fdb021517e058456d79db` L24–30 — **reviewed**：測試 tokio features 支援新增 updater tests。

### `apps/desktop/core/src/lib.rs`

- `7acc868776ee78af1ccc0265b749a8a4e7aeb172d4dc91e5fb3d5fcb2d7daf93` L177–199 — **reviewed**：Hunk 177: Read diagnostic test hunk: finish retains operation id and proxy-auth error without inventing HTTP status.

### `apps/desktop/core/src/network.rs`

- `bd6dda5fa2dee19c07c6105f7f6aeb08ac75a975cc2ccbf2db1b5e4d89ac966d` L216–302 — **reviewed**：Hunk 216: Read complete added network test hunks: CONNECT proxy fixture performs trusted-CA TLS check and verified GET; malformed/disconnected tunnel cases require error and no HTTP status; no production behavior modified here.

### `apps/desktop/core/src/project.rs`

- `1672ef38abdcff674390818f10132c3addaabecf3be55ebc1c95532fc7463d57` L121–127 — **reviewed**：Hunk 121: Read project init comment hunk: tool parser remains called before any mutation, comment now includes Antigravity.

### `apps/desktop/core/src/query.rs`

- `716023a63c678e7d38f35d602f26249733375f6f31b8d6f152416094202c728f` L1976–1989 — **reviewed**：Hunk 1976: Read query hunks: allowed check kinds are explicitly parsed, security record ids go through report API, active/archived context resolved before indexed read; absent report returns unknown_report.

### `apps/desktop/core/src/security.rs`

- `30d2762a7515364c693001c7e8b0d5b546c20be4cb56127b8ec983ac9b45a32f` L41–48 — **reviewed**：錯誤碼白名單與階段映射。
- `604f6f018e6c377391a5909a65a17047c2c5b9581fae9dc6d00178b669c8e1ea` L88–96 — **reviewed**：標準資料錯誤碼白名單。
- `4c05d8b1b5796a3e4b76c8a8135a2fad24c55de46b05ddd30cdd09856589c158` L138–145 — **reviewed**：新增僅測試鎖，避免環境競爭。
- `e41a61f207541ada6e8045797d232d161137e965bb2154a7daa9f5758e19b1df` L152–162 — **reviewed**：路徑寫入持 job 鎖，pending outcome 時拒絕。
- `b2acb0c001d2e89389471877abe473a8b06b60f7add399c78112f4c3a8b3cbd6` L305–327 — **reviewed**：repository 持有 path lease，路徑與政策操作綁定同一位置。
- `d26b882dcbcc369a4e17a5e0a2d07bf0bf3c4d00019f76c2e188877bfc1dcb92` L340–346 — **reviewed**：run 資料工作改從 lease repository 取得，先檢查取消。
- `6f6c093ca155e04a1cd28d71711027ed921ba13f515c4d895817095b07954337` L433–439 — **reviewed**：inspection deviceLocal 欄位供前端。
- `c7a50bfb9d5581fc35a48938331272acedf47c672143027d65dd4bdedb5318b5` L458–467 — **reviewed**：本機 change 對應 workspace 根目錄，remote checkout 仍顯式選取。
- `563160d8d42c3c9de0c46e31d17c369f1f1ac1d6aa28d3ab3d6350c6bb4d846a` L493–499 — **reviewed**：remote ReviewTarget 初始化 proposal key 為空，待 restore。
- `127f19b7d79e4ce86428fcb22248d83d3d4a777c1757116a0dfc1fc1dc09b615` L510–529 — **reviewed**：從 prepared policy 還原遠端 proposal key，inspect 前呼叫。
- `e015ff42426ff5992d83eee3dda70ad88e7f448a4bf079acf89b3753c845b45b` L564–576 — **reviewed**：本機提案權威採 ProposalReports，不回退裝置草稿。
- `74b2dac35d9eb67ea5b1b75db58414a6d69f79e89037356f71f14607cf02e64f` L603–624 — **reviewed**：review job 本機發布提案、remote 維持待發布，routing key 維持 checkout binding。
- `a8e7ad970385dc96521830f1584928bb949dfaeb7d223b0ae0d9b98ae89696f6` L703–716 — **reviewed**：測試 pending job 禁止改路徑。
- `75d142f5eceebf3c5ea3f878fc285498e2a90005a0cdb4ff8a7469edb6d2caab` L798–804 — **reviewed**：測試鎖限制同時修改全域設定。
- `0a961d9cf06dbd67a58c382f15e190308f4f4003d848df5df6b3e9e85cc0cb5c` L840–963 — **reviewed**：逐段讀取 target report/export/reveal、knowledge job、remote attestation、review view 及歷史整合測試。

### `apps/desktop/core/src/settings.rs`

- `0b119225be35009f963dab1e26cc7c3db2259560489cbd5b5f79805a239f141f` L45–51 — **reviewed**：settings payload 增加 check_records。
- `a20ec6f1fe4304ac0c02cb0526f001354be46ae3864e6fbd4d0ec03399b1c0d4` L137–143 — **reviewed**：讀有效 config 值。
- `a1903ef181fcca19d0e222ad9023811f81f290fa6c9a37279325877b02eea9b3` L155–161 — **reviewed**：parse error/default payload 仍有保留預設。
- `081d7928d5638b1b6c480fa2fcd077e7f6a4ebc768e8eaa005dd6449f0b7614f` L390–401 — **reviewed**：兩種 patch 先全部成功再進寫入。
- `ae7badaaadd1f05a89cd13627431c352ceeff7f414ff2fd588c2c3af141de037` L433–446 — **reviewed**：舊 write wrapper 委派新版本，寫入採 combined text。
- `94c2c933a8fb06fd744fc5500f86957ea6cdc75f753bc7a708e7148cac646b05` L1733–1750 — **reviewed**：測試未知欄位保留、invalid records patch 失敗前未落檔。

### `apps/desktop/core/src/testdata/README.md`

- `208a75bf23ca6f4bd2d79893a9a881d73f3f2ebab20437b00819880840e556b8` L1–6 — **reviewed**：Hunk 1: Read TLS fixture README hunk: keys clearly test-only and trust restricted to loopback test client; actual network test uses reserved updates.test and local fixture CA.

### `apps/desktop/core/src/testdata/proxy-ca.der`

- path-only：generated skill/snapshot or binary fixture; inspect source/consumer tests


### `apps/desktop/core/src/testdata/proxy-server-key.der`

- path-only：generated skill/snapshot or binary fixture; inspect source/consumer tests


### `apps/desktop/core/src/testdata/proxy-server.der`

- path-only：generated skill/snapshot or binary fixture; inspect source/consumer tests


### `apps/desktop/core/src/updater.rs`

- `c8d74c2df3d0277b5c6dda4f3e4fd5a0ba7d419988ffb621e6eb31876ef71076` L1–672 — **reviewed**：已逐段讀取 1–672：檢查 release 標籤與來源過濾、代理診斷結算、job 下載後安裝條件；既有單元測試涵蓋錯誤碼及拒絕上游來源。未執行測試。

### `apps/desktop/core/src/verbs.rs`

- `bb3503f616eba68b3fefeb7adeeded9d96a5863ec92140902b9149d746125ee1` L76–91 — **reviewed**：Hunk 76: Read archive verb hunks: acceptance policy parsed before archive, authorized local root and correct spec dir bind fresh proof only when enabled/required; archive outcome exposes record plan.
- `3649fa4c784caf9774a6f77364e3a84d46b509fb8e7f7c8eb98ac2c15af7ff0a` L95–101 — **reviewed**：Hunk 95: Read archive verb hunks: acceptance policy parsed before archive, authorized local root and correct spec dir bind fresh proof only when enabled/required; archive outcome exposes record plan.

### `apps/desktop/package.json`

- `aaf879847845da12ba1682b49d7760bdb3ed8cc18e0cb24dc3afac161e320d9a` L17–25 — **reviewed**：desktop 新增 dnd-kit sortable/utilities 與既有 core 同世代 dependency。

### `apps/desktop/src-tauri/capabilities/default.json`

- `280c5c6574a2dcf5a3b2da4bf4f0a8d9152cce9761e94f0fa7976549df56f154` L25–34 — **reviewed**：Read newStart 25–34 with surrounding capabilities: opener URL permission allowlist includes fixed Speclink GitHub, OWASP GitHub/raw and MITRE data/license hosts; permission-only wiring, no wildcard arbitrary-host addition. Hunk at new line 25.

### `apps/desktop/src-tauri/src/connections.rs`

- `9a49daede7fffdaccdf93526cd61d2be412c9a3f9f243ddd364d4f8fc6267066` L149–155 — **reviewed**：Read all five checkout hunks: tool picker/help derives from core names; independent Codex/Antigravity generation and empty selection no-write assertions. Hunk at new line 149.
- `cde9c1941f4d942f5bf090f118272d541694cda344ac86516249d48d701b9238` L198–204 — **reviewed**：Read all five checkout hunks: tool picker/help derives from core names; independent Codex/Antigravity generation and empty selection no-write assertions. Hunk at new line 198.
- `b61924601d37746736567224b9ab8a4e33bfe08c73856ce5eaf618d726a8bda2` L246–252 — **reviewed**：Read all five checkout hunks: tool picker/help derives from core names; independent Codex/Antigravity generation and empty selection no-write assertions. Hunk at new line 246.
- `a45e2eb77e2683991be934b5da3b112c04132133b4b24b11cb0a2aa942a468ce` L696–711 — **reviewed**：Read all five checkout hunks: tool picker/help derives from core names; independent Codex/Antigravity generation and empty selection no-write assertions. Hunk at new line 696.
- `d1d3649478e72e969d1f516f32de53473964ff635a2f81e0d5714eaa0e8b965c` L763–769 — **reviewed**：Read all five checkout hunks: tool picker/help derives from core names; independent Codex/Antigravity generation and empty selection no-write assertions. Hunk at new line 763.

### `apps/desktop/src-tauri/src/lib.rs`

- `502071dbbeef8cb7e6c36295724a701310db85d75fb4c57cc826e09483199c47` L741–747 — **reviewed**：Hunk 741: Read all Tauri command hunks: check-record policy patch propagates to local/remote settings, history/report/review commands use scoped calls and background execution, 401 maps to reauthentication, archive forwards checkout root, all new command handlers registered.
- `1f92b67905e519b83770314a63f42f93e24e1af44358f0a7fac90e14397e1af7` L751–761 — **reviewed**：Hunk 751: Read all Tauri command hunks: check-record policy patch propagates to local/remote settings, history/report/review commands use scoped calls and background execution, 401 maps to reauthentication, archive forwards checkout root, all new command handlers registered.
- `943cb6e216c245a9efd9753ac536f046ada3e38bafef602d8606960e38476961` L1279–1285 — **reviewed**：Hunk 1279: Read all Tauri command hunks: check-record policy patch propagates to local/remote settings, history/report/review commands use scoped calls and background execution, 401 maps to reauthentication, archive forwards checkout root, all new command handlers registered.
- `14e249fdad055ad72ba09bb3d05f8a1a334908c57073e87937045d1e4e5eca51` L1290–1300 — **reviewed**：Hunk 1290: Read all Tauri command hunks: check-record policy patch propagates to local/remote settings, history/report/review commands use scoped calls and background execution, 401 maps to reauthentication, archive forwards checkout root, all new command handlers registered.
- `e5f34859460ff5ff8c367f229bf2c3981b4f971ad2af561a312623de0c605930` L1558–1576 — **reviewed**：Hunk 1558: Read all Tauri command hunks: check-record policy patch propagates to local/remote settings, history/report/review commands use scoped calls and background execution, 401 maps to reauthentication, archive forwards checkout root, all new command handlers registered.
- `c83a4c4d25d4049b95525691b536dd6cc1b173380a48f653283690c844e59cf8` L1668–1677 — **reviewed**：Hunk 1668: Read all Tauri command hunks: check-record policy patch propagates to local/remote settings, history/report/review commands use scoped calls and background execution, 401 maps to reauthentication, archive forwards checkout root, all new command handlers registered.
- `ae6a37d474ae3107919138f0acb30af5ad9a797fa1d37f6833b5e9e15b0175cd` L2092–2109 — **reviewed**：Hunk 2092: Read all Tauri command hunks: check-record policy patch propagates to local/remote settings, history/report/review commands use scoped calls and background execution, 401 maps to reauthentication, archive forwards checkout root, all new command handlers registered.
- `36b53a3121eed6bbeda4e4410a9ee4acb30f4c65eeff43261a8f0364c76dd805` L2111–2118 — **reviewed**：Hunk 2111: Read all Tauri command hunks: check-record policy patch propagates to local/remote settings, history/report/review commands use scoped calls and background execution, 401 maps to reauthentication, archive forwards checkout root, all new command handlers registered.

### `apps/desktop/src-tauri/src/remote.rs`

- `b59e98dd13d7c2ca59843b25c2ffe3d86f1f1d1416cf8d23f0b9f06718e87359` L1134–1141 — **reviewed**：Read all assigned remote capability/preparation/settings/history/report/archive hunks: canonical key comes from handshake, required policy uses attestation and index revision, ordinary archive remains reference-mode path. Hunk at new line 1134.
- `72a006e3788e82658ecc826ad30a2881e5f38df3d51b7a1551c59b1896566550` L1182–1189 — **reviewed**：Read all assigned remote capability/preparation/settings/history/report/archive hunks: canonical key comes from handshake, required policy uses attestation and index revision, ordinary archive remains reference-mode path. Hunk at new line 1182.
- `77693f0af72a429212c273a0a55171ad691e45f9fb7d8a8e29e5abf8cabba59f` L1351–1357 — **reviewed**：Read all assigned remote capability/preparation/settings/history/report/archive hunks: canonical key comes from handshake, required policy uses attestation and index revision, ordinary archive remains reference-mode path. Hunk at new line 1351.
- `855dc12bfc7561a33962945fd8bf07c944d10e3d10ec03abbfe457a76b86d0e9` L1363–1376 — **reviewed**：Read all assigned remote capability/preparation/settings/history/report/archive hunks: canonical key comes from handshake, required policy uses attestation and index revision, ordinary archive remains reference-mode path. Hunk at new line 1363.
- `06349f381aa21912a6f86ae08d046dde189fb21b6d53227504616e9d3b9a5753` L1379–1386 — **reviewed**：Read all assigned remote capability/preparation/settings/history/report/archive hunks: canonical key comes from handshake, required policy uses attestation and index revision, ordinary archive remains reference-mode path. Hunk at new line 1379.
- `f9ab228ed33d993e2d8d772ab4a7eac31afcfa5b70bca424476c3d9830612dbb` L1493–1505 — **reviewed**：Read all assigned remote capability/preparation/settings/history/report/archive hunks: canonical key comes from handshake, required policy uses attestation and index revision, ordinary archive remains reference-mode path. Hunk at new line 1493.
- `4d0e66941f971b69d56ca3061dcdb6a556ad531fdb8c84081c30f4eb43cc43c2` L1507–1517 — **reviewed**：Read all assigned remote capability/preparation/settings/history/report/archive hunks: canonical key comes from handshake, required policy uses attestation and index revision, ordinary archive remains reference-mode path. Hunk at new line 1507.
- `8aaaa0d42c31ab70849d60c08b177ac6d00f449317828f688f217d3b26e94235` L1626–1639 — **reviewed**：Read all assigned remote capability/preparation/settings/history/report/archive hunks: canonical key comes from handshake, required policy uses attestation and index revision, ordinary archive remains reference-mode path. Hunk at new line 1626.
- `0d4fd0102a53437a6662d95c68eddcbbcec2db0268d3d7857ebb632eeebb5991` L1758–1773 — **reviewed**：Read all assigned remote capability/preparation/settings/history/report/archive hunks: canonical key comes from handshake, required policy uses attestation and index revision, ordinary archive remains reference-mode path. Hunk at new line 1758.
- `36d4613d1a737c7b7f2040041ecf7a0d18dfdd726dda097c8a24f7fa3dbdfe22` L1941–1947 — **reviewed**：Read all assigned remote capability/preparation/settings/history/report/archive hunks: canonical key comes from handshake, required policy uses attestation and index revision, ordinary archive remains reference-mode path. Hunk at new line 1941.

### `apps/desktop/src-tauri/src/security_commands.rs`

- `70975d06e2414dd1b9e4160a842805046c95e2b95536b4369939a3fc324b1c6d` L42–55 — **reviewed**：Tauri 路徑 read/write 使用 blocking worker 與 jobs coordinator。
- `98cae637022db4269060240c4db9800889b19e8e986589c5618be6982f6926f3` L140–165 — **reviewed**：report/summary/export 依 target/change 選權威；knowledge job 發 completed 事件。
- `b39c5e231b2531c7ce344e547834c104041139e9bcb0e3e0a75f47f4bdaee148` L167–173 — **reviewed**：reveal 帶 change 進目標範圍驗證。

### `apps/desktop/src-tauri/src/updater_commands.rs`

- `b1fae7fd02a2cd08137e2ab3b0d1fedcd4e22129c01d8b9b918dd83890e5fed4` L1–126 — **reviewed**：逐段讀取 1–126：Tauri 僅包裝 core lifecycle，plugin error typed 分類、immutable proxy client、blocking 安裝 worker；測試拒絕從錯誤字串猜 HTTP 狀態。

### `apps/desktop/src/App.tsx`

- `0172500ff91763bc8fa8ee90d41632820d3aee21171dc0facda052a1f18c183a` L90–108 — **reviewed**：Hunk 90: Read all App hunks: checkRecords passed into active/archive detail, stable project reorder action wired, updater last-check diagnostics connected, tool selection shares builtins, archive confirmations display preview and block pending/error states.
- `2115ffdb08d3eb6c100d88a3d2c3a66e7a5327615b31b79943f1595a4a72baee` L919–925 — **reviewed**：Hunk 919: Read all App hunks: checkRecords passed into active/archive detail, stable project reorder action wired, updater last-check diagnostics connected, tool selection shares builtins, archive confirmations display preview and block pending/error states.
- `f88d926689f97538b8ec1d077a6dfb793d77487b0dc9d0fbe227af0cfeaa2b58` L1043–1049 — **reviewed**：Hunk 1043: Read all App hunks: checkRecords passed into active/archive detail, stable project reorder action wired, updater last-check diagnostics connected, tool selection shares builtins, archive confirmations display preview and block pending/error states.
- `6c4bce10ee1465902e77d6f2e854bb7cdbe3ddfe4e863790fad666210d2454bb` L1131–1137 — **reviewed**：Hunk 1131: Read all App hunks: checkRecords passed into active/archive detail, stable project reorder action wired, updater last-check diagnostics connected, tool selection shares builtins, archive confirmations display preview and block pending/error states.
- `ee2ae2e57a09e449d4a603ee0ebe6eda9c2dcb37ed40f1eedf7e22cc7c9ec4a4` L1262–1272 — **reviewed**：Hunk 1262: Read all App hunks: checkRecords passed into active/archive detail, stable project reorder action wired, updater last-check diagnostics connected, tool selection shares builtins, archive confirmations display preview and block pending/error states.
- `de9d2e54cfa7090359f38fc9d32ee0f9761e1b15cd654581e96598f06dea9a19` L1529–1535 — **reviewed**：Hunk 1529: Read all App hunks: checkRecords passed into active/archive detail, stable project reorder action wired, updater last-check diagnostics connected, tool selection shares builtins, archive confirmations display preview and block pending/error states.
- `e93cb60da8db6751930ad7367dcbd50b00ed9743cb7681fcffc6b9cb50814a1a` L1540–1546 — **reviewed**：Hunk 1540: Read all App hunks: checkRecords passed into active/archive detail, stable project reorder action wired, updater last-check diagnostics connected, tool selection shares builtins, archive confirmations display preview and block pending/error states.
- `0af42146b2062151605a1a76047175fdbb9d3aca6e60950cbb3a50c949b4a560` L1555–1561 — **reviewed**：Hunk 1555: Read all App hunks: checkRecords passed into active/archive detail, stable project reorder action wired, updater last-check diagnostics connected, tool selection shares builtins, archive confirmations display preview and block pending/error states.
- `8923d6627813201aec348bd8213f8857f1ffa86fdde07b2721581fba02bbfb12` L1566–1572 — **reviewed**：Hunk 1566: Read all App hunks: checkRecords passed into active/archive detail, stable project reorder action wired, updater last-check diagnostics connected, tool selection shares builtins, archive confirmations display preview and block pending/error states.
- `1b1d80d4710425e554c6a9d55606f4d35ae6c4c0a02416d90760afb5b03f0ae7` L1592–1598 — **reviewed**：Hunk 1592: Read all App hunks: checkRecords passed into active/archive detail, stable project reorder action wired, updater last-check diagnostics connected, tool selection shares builtins, archive confirmations display preview and block pending/error states.
- `21930dd580caef87778e581d8d320c52b59016ba2a6f19b8aca04e75fd5a0b71` L1603–1609 — **reviewed**：Hunk 1603: Read all App hunks: checkRecords passed into active/archive detail, stable project reorder action wired, updater last-check diagnostics connected, tool selection shares builtins, archive confirmations display preview and block pending/error states.
- `2b4e7ca7af7c6d126652ccd5e465b9504918388c26b75c0455d60772d603206f` L1613–1619 — **reviewed**：Hunk 1613: Read all App hunks: checkRecords passed into active/archive detail, stable project reorder action wired, updater last-check diagnostics connected, tool selection shares builtins, archive confirmations display preview and block pending/error states.
- `451e777d75bb8927b969beef46c75ab91fec39819a8c30ac37d20f6204b071c3` L1622–1628 — **reviewed**：Hunk 1622: Read all App hunks: checkRecords passed into active/archive detail, stable project reorder action wired, updater last-check diagnostics connected, tool selection shares builtins, archive confirmations display preview and block pending/error states.

### `apps/desktop/src/__tests__/App.test.tsx`

- `2a4c6a60d606a2ba8f5f666e349483e460368009fb621a6407e03e5734f9fe0d` L705–717 — **reviewed**：Hunk 705: Read all App test hunks: tool accessible names reflect shared directory labels, hidden Dnd status excluded narrowly, fork links updated, localized archive count preview asserts cancel makes no write.
- `5f651d5bcfe7a8fdf9d27a637dcc828379caa06d8b98da617a5b2ecc4aef3dd6` L743–760 — **reviewed**：Hunk 743: Read all App test hunks: tool accessible names reflect shared directory labels, hidden Dnd status excluded narrowly, fork links updated, localized archive count preview asserts cancel makes no write.
- `02c830d8a0c6a30babca57b825e74d8a60304925e8db235bd91632cf1f664a1d` L773–782 — **reviewed**：Hunk 773: Read all App test hunks: tool accessible names reflect shared directory labels, hidden Dnd status excluded narrowly, fork links updated, localized archive count preview asserts cancel makes no write.
- `10c46d03afca6408bb5a5fcb9c3ad9b616c8f7262e9becddc276d2c71807b7cd` L1072–1079 — **reviewed**：Hunk 1072: Read all App test hunks: tool accessible names reflect shared directory labels, hidden Dnd status excluded narrowly, fork links updated, localized archive count preview asserts cancel makes no write.
- `dd2ac6bfe99b795c456e7f8b1fc2c64f7632f59686cc8dab093067139d51ce88` L2858–2865 — **reviewed**：Hunk 2858: Read all App test hunks: tool accessible names reflect shared directory labels, hidden Dnd status excluded narrowly, fork links updated, localized archive count preview asserts cancel makes no write.
- `5491e7b386f2766e1d63d1b2b9401ad1af84ce0c44e38e6f6d8609c8a118ae9a` L3107–3132 — **reviewed**：Hunk 3107: Read all App test hunks: tool accessible names reflect shared directory labels, hidden Dnd status excluded narrowly, fork links updated, localized archive count preview asserts cancel makes no write.

### `apps/desktop/src/__tests__/aboutPage.test.tsx`

- `2b4d6815db2a5c250a00a402a0bd79c706a4a5e14f2f2175b4b8b1a85475731b` L114–130 — **reviewed**：Hunk 114: Read about-page test hunk: fork source/docs/issues URLs and independent upstream link asserted with issue text URL-encoded.

### `apps/desktop/src/__tests__/appSettingsView.test.tsx`

- `7e6e7108847c90e306149a914abb1fd60db31a0cac5dc4d05e0b40e0cdfe031d` L10–16 — **reviewed**：Read appSettingsView test mock hunk: new security path settings API mocked read-only, type fixture describes default effective path. Hunk at new line 10.

### `apps/desktop/src/__tests__/checkRecords.test.ts`

- `4e5af4ea452197cf7e59222edf478a0baf5e21f7336f1f47dc2b177c3178ae84` L1–25 — **reviewed**：Read all 25 test lines: history uses root/change/index IDs instead of display paths, remote errors do not fall back local, archived acceptance receipt bypasses report-ID endpoint. Hunk at new line 1.

### `apps/desktop/src/__tests__/projectRail.test.tsx`

- `9cf8f47d62aa0777b67da4872900fb908b0f0e81c65e927cec49e6404eac912c` L62–114 — **reviewed**：Read 62–114 with context: bounded context-menu moves and pointer Escape/outside/drop tests use cleanup and assert project is not activated. Hunk at new line 62.

### `apps/desktop/src/__tests__/projectSettingsView.test.tsx`

- `9be8154c79ef152e1b419114af742e172fa7b20f82d8ab0621454ac2ae108af4` L225–233 — **reviewed**：工具標籤加入技能資料夾，斷言維持 aria-checked。
- `abc1c04f51953168279b9dd85df39a4b808f3a73b010735cfb529cf0d03f9cfa` L271–277 — **reviewed**：codex 標籤更新且儲存純 ID。
- `300917d76328771132f47954fce0453b58c27ca8b866dcdbbf62d7e8e0ae35cc` L279–285 — **reviewed**：copilot 標籤更新且儲存純 ID。
- `566618a31446635c0dfbd80fb98cc8f50d9b10afd4d6b24851f3b1f7ef19d14d` L429–435 — **reviewed**：未進入頁面的 label 斷言同步更新。
- `43a2d84e7a1bf44791190d5538723b89167388acefa516fe373a2611edcfac3d` L443–449 — **reviewed**：speclink 頁面工具可讀標籤同步。
- `58ae932dfaa57af270c9f589710b6019219032cc985191b88f1202d78ff9d6b2` L508–514 — **reviewed**：parse error 鎖定設定寫入斷言。
- `7124eb350f901d746fb5e2aad17699931b7a41c80a5a918ba9e52c8c6274db0d` L536–542 — **reviewed**：dirty 狀態測試更新 label。
- `c744fa3b78ecbf0499fb20137ea7d1e26d8a416be8ddf1417fad2ed8e8eff315` L1713–1719 — **reviewed**：smell 說明按鈕斷言從文字改 role。
- `c45894f5af02a65f58ace588acb20a4887f512e9b78856f5ceb648c741fd5d92` L1976–2025 — **reviewed**：逐段讀取驗收/保留預設與取消、disabled reader、CAS 保留草稿與 Antigravity 工具 ID 測試。

### `apps/desktop/src/__tests__/securityDataPage.test.tsx`

- `66d2c9fbe9a41432102462bdb07173f7fd5630823e8a773b6f1951484c36fb11` L11–22 — **reviewed**：Hunk 11: Read changed security-data UI tests: failed save retains draft, read-only disables mutation while permitting export, reset writes null only, chooser/cancel do not write, stale inventory response cannot replace saved new-root result.
- `24d0f38e6012d24110eb337601aa00d7cc65922ee54dcb2f98801e06824514f5` L25–80 — **reviewed**：Hunk 25: Read changed security-data UI tests: failed save retains draft, read-only disables mutation while permitting export, reset writes null only, chooser/cancel do not write, stale inventory response cannot replace saved new-root result.

### `apps/desktop/src/__tests__/securityReview.test.tsx`

- `d298b6f95fd3b2715dc7949745b513f1cffcca25e73e374f7ed0065f3edf801b` L17–29 — **reviewed**：Hunk 17: Read security adapter test hunk: different project adapter closures carry distinct root/change/id across report reads, preventing switched-page scope capture.

### `apps/desktop/src/__tests__/store.test.ts`

- `b9f1b8fd8035da03dfa9cea65983a63822b4097c403fa898b732ec78c56aa80f` L3183–3225 — **reviewed**：Read 3183–3225 with context: archive count preview, cancellation stale response rejection and fail-closed offline preview prevent archive/discard/carry effects. Hunk at new line 3183.

### `apps/desktop/src/__tests__/tabs.test.ts`

- `d5a049bdc44ec4cda94ada4b0cce941503265271f55cc9c69bd27dbefcb2b979` L11–17 — **reviewed**：Hunk 11: Read tabs test hunks: reorder is validated for complete distinct locator set, leaves input untouched and persists current active locator across local/remote ordering.
- `4769e9ce7c6a0b85c3141351417614626fbbc4e5ad7b5b44b7035d355ff0864d` L43–65 — **reviewed**：Hunk 43: Read tabs test hunks: reorder is validated for complete distinct locator set, leaves input untouched and persists current active locator across local/remote ordering.

### `apps/desktop/src/__tests__/updater.test.ts`

- `bd4eb1ad4b7f47aaeb89897140dba0db4eead7ba21c46daf50f817c123882070` L30–54 — **reviewed**：Hunk 30: Read changed updater tests: manual candidates never download, failed background checks retain sanitized code/id then clear on success, only descriptor errors fall back, Rust candidate id reused, four-part/build version ordering cases covered.
- `d915ad34418aaf02f53f31beeda9ca19ffd9bbf2270eda0f8a6f65a88c0a69ea` L491–537 — **reviewed**：Hunk 491: Read changed updater tests: manual candidates never download, failed background checks retain sanitized code/id then clear on success, only descriptor errors fall back, Rust candidate id reused, four-part/build version ordering cases covered.

### `apps/desktop/src/__tests__/updatesPage.test.tsx`

- `5f533f2fdbb7297bac021de6e50529865ef05aa09d2abd108c0ddef12e6236d1` L1–6 — **reviewed**：測試新增非同步查詢工具 import。
- `ea6fd58af00633031b69a5ac8eab0657b487930f1caa759b4e80413f3204cdd5` L45–132 — **reviewed**：逐段讀取更新頁測試：fork/upstream 狀態與成功時間分離、無簽章可手動更新、無 adapter 仍有版本/日誌、失敗不冒稱最新。

### `apps/desktop/src/__tests__/workspace.test.ts`

- `06f664d4a19acd823f6e8bb8991673914c50e2bd618a7365d7d76a7ffed63ddd` L101–135 — **reviewed**：排序測試驗證重新讀持久化順序、session/activeKey 不變、storage error 或重複 keys 不改狀態。

### `apps/desktop/src/__tests__/workspaceChooser.test.tsx`

- `1922b1f92f1b46e1b32578b51344481f6eed33a5e8d3d87b54c6bdc7d1be6bae` L284–292 — **reviewed**：chooser 工具 checkbox 測試加 Antigravity 標籤，codex 原選擇斷言保留。

### `apps/desktop/src/adapter/checkRecords.ts`

- `1468727b4fd25b679b5a94f686cee3e958c336bc0c48c1ed2d184ccc78202163` L1–25 — **reviewed**：逐段讀取 1–25：adapter scope closure 隔離專案，三類並行讀取後更新 receipt cache，安全報告專用 endpoint、review/verify history 共用查詢。

### `apps/desktop/src/adapter/connections.ts`

- `4ab1a2e903d963c33dfd40465d3f9a578a8c99b15dd316c7fbca80c9d1ba0179` L73–79 — **reviewed**：Hunk 73: Read connection inspection comment hunk: returned tools remain builtin string keys and no credential/state introduced.

### `apps/desktop/src/adapter/remoteDataSource.ts`

- `5dd2b7ad43e2c579044f690031333045a8693495834476d62771c107ffc272a4` L1–4 — **reviewed**：Hunk 1: Read remote datasource hunks: history adapter added only with negotiated capability, security adapter with old supported flag, remote archive carries optional checkoutRoot for gate evidence.
- `158f011ba07c4238b3c71b359658b6efac00b54671d8630f101c561b333e9e0f` L65–71 — **reviewed**：Hunk 65: Read remote datasource hunks: history adapter added only with negotiated capability, security adapter with old supported flag, remote archive carries optional checkoutRoot for gate evidence.
- `64e9b087956449fb0e0653e46e347f561cd1ee007fd614d6ee2dc948d7b20334` L81–87 — **reviewed**：Hunk 81: Read remote datasource hunks: history adapter added only with negotiated capability, security adapter with old supported flag, remote archive carries optional checkoutRoot for gate evidence.
- `ae41fbf9877f4707cb237273819d89522b4e346fccaa35c426f61ec783ee1def` L174–180 — **reviewed**：Hunk 174: Read remote datasource hunks: history adapter added only with negotiated capability, security adapter with old supported flag, remote archive carries optional checkoutRoot for gate evidence.

### `apps/desktop/src/adapter/security.ts`

- `cebb8fc008cd7275db5a7a804b4c4e951b1a6805f61ad5823976e757d829c35d` L2–16 — **reviewed**：Hunk 2: Read adapter hunks: path settings invoke correct camelCase arguments, policy default gate normalized false before diff, knowledge preparation and change-aware report/reveal/export propagate selected target, check-record patch contains only changed leaves.
- `753e9a0e7457168e33b420b046b35777fb66733c651ddbd4569f21138a20af81` L24–30 — **reviewed**：Hunk 24: Read adapter hunks: path settings invoke correct camelCase arguments, policy default gate normalized false before diff, knowledge preparation and change-aware report/reveal/export propagate selected target, check-record patch contains only changed leaves.
- `f2ced116ef577eb181854015ffa814a3aef51ffdde1ed7639d9fbf7ae1504f5c` L48–54 — **reviewed**：Hunk 48: Read adapter hunks: path settings invoke correct camelCase arguments, policy default gate normalized false before diff, knowledge preparation and change-aware report/reveal/export propagate selected target, check-record patch contains only changed leaves.
- `8c3c252a2dbd2f924209bcc1fba37321ba8db79ae67ab4b7c08ad40897efdd22` L91–122 — **reviewed**：Hunk 91: Read adapter hunks: path settings invoke correct camelCase arguments, policy default gate normalized false before diff, knowledge preparation and change-aware report/reveal/export propagate selected target, check-record patch contains only changed leaves.

### `apps/desktop/src/adapter/tauriDataSource.ts`

- `7f9259575aa662822eaeac7505a835e84e0528d40d42cc8c87e66399bd10be48` L17–23 — **reviewed**：Read both tauriDataSource hunks: adapter combines review/verify/security history by original root/change; receipt returned content shortcut; record reads route by ID and kind. Hunk at new line 17.
- `a74e90ec83680b866381112d5dc8d35df08cdd6aa27058ef075341533c94aef4` L28–34 — **reviewed**：Read both tauriDataSource hunks: adapter combines review/verify/security history by original root/change; receipt returned content shortcut; record reads route by ID and kind. Hunk at new line 28.

### `apps/desktop/src/adapter/updater.ts`

- `9892e566e4ed2634bc9aab6e8fcfa38d0895fdb627be54dca6c91f2633fb75b5` L2–8 — **reviewed**：新增版本比較供 manual fallback。
- `b05fb3e5bfea5b89009528f4ab84bc66dd99bfbfe971a205ac0f4ec27a8b1131` L17–30 — **reviewed**：adapter 合約加入 ManualUpdate 及獨立 release lookup。
- `ac56fe456800df264235cd2d04eb17b18607317be550c7160d51b0eb6f537a19` L38–54 — **reviewed**：僅 404/unavailable/parse fallback 到可信 fork release，compareVersions 確認 newer，其他 TLS/簽章錯誤維持失敗。

### `apps/desktop/src/adapter/workspace.ts`

- `c7c9aa6ad7927c3fc01925665ce47d68521e126adb5afffb3fcc43348ed7c116` L57–63 — **reviewed**：snapshot workflow type 新增可選 checkRecords。
- `d667de7088e2378cdcf2ff79753ef851809dbaf2995b44c40e6bbd899b4f3018` L88–94 — **reviewed**：partial fields 型別新增 patch。
- `1184857710d129b703d5f3708e2fbc37625b63210ce9c23d5465982f2e502b81` L165–171 — **reviewed**：本機 invoke 只傳明確 patch。
- `af49d61bc23e3f32e1df5a326c6ed2102ec15576629d09968b9699df00a362ba` L191–198 — **reviewed**：remote provider 接收 v2/history capabilities。
- `bc6d04f6ba9949f41f53d71e43630a50bdd94385a64501d475d8e4feddf1db35` L211–217 — **reviewed**：能力公開給表單使用。
- `3f246248253680c25491cc13f3df1907fb3c6ee1e22db4194bfd98597b16e64e` L222–228 — **reviewed**：遠端寫入依能力拒絕不支援政策。
- `515af85ca24fb4bc5b6d356c0910441a791c551373fb7441187aa5bedb9c45c3` L232–238 — **reviewed**：remote invoke 傳 exact patch 與 revision。

### `apps/desktop/src/builtinTools.ts`

- `e4c2adee014ca2d4c1e4055e0c30371b4aadebc5afb2e5e18fa59989923ab21c` L1–10 — **reviewed**：Hunk 1: Read builtinTools entire hunk: four literal tool keys map to correct managed directories; picker array and labels derive from same mapping.

### `apps/desktop/src/components/ProjectRail.tsx`

- `4150cd44fe0f069f6ca599c8309abc6a00982b0287362d5772cc9e3e763856b8` L3–21 — **reviewed**：Hunk 3: Read all rail hunks: sortable wrapper receives stable locator keys, drag target indexes are checked, menu reordering disables endpoints, pointer activation threshold avoids immediate click drag; wrappers preserve menu/tooltip composition.
- `5761449a17e266113d605fe13edb2f15a5a4fc896bacd6a1b92dec63d64edfa0` L56–62 — **reviewed**：Hunk 56: Read all rail hunks: sortable wrapper receives stable locator keys, drag target indexes are checked, menu reordering disables endpoints, pointer activation threshold avoids immediate click drag; wrappers preserve menu/tooltip composition.
- `109efe9bbcf55945c95a142987577b732c848f195bed2d8e0a4ee4b9406a5333` L65–92 — **reviewed**：Hunk 65: Read all rail hunks: sortable wrapper receives stable locator keys, drag target indexes are checked, menu reordering disables endpoints, pointer activation threshold avoids immediate click drag; wrappers preserve menu/tooltip composition.
- `89bd8c7349bcb462ecc65040786ef6e966237d52e6fbb7e07c749715f042352a` L113–127 — **reviewed**：Hunk 113: Read all rail hunks: sortable wrapper receives stable locator keys, drag target indexes are checked, menu reordering disables endpoints, pointer activation threshold avoids immediate click drag; wrappers preserve menu/tooltip composition.
- `3f6ca833e3361e2968428c1eba642c9ed97dd2a5009d6b52c994d87673d48028` L129–142 — **reviewed**：Hunk 129: Read all rail hunks: sortable wrapper receives stable locator keys, drag target indexes are checked, menu reordering disables endpoints, pointer activation threshold avoids immediate click drag; wrappers preserve menu/tooltip composition.
- `3f0d4617b89ed5d44893d81db7b86f602ce891752fdcf9e6a17698deea201ada` L240–247 — **reviewed**：Hunk 240: Read all rail hunks: sortable wrapper receives stable locator keys, drag target indexes are checked, menu reordering disables endpoints, pointer activation threshold avoids immediate click drag; wrappers preserve menu/tooltip composition.
- `6acdc70faae21a1924aa07c8b0db28347d7453ed245fcafe06df9ce2674668b0` L249–255 — **reviewed**：Hunk 249: Read all rail hunks: sortable wrapper receives stable locator keys, drag target indexes are checked, menu reordering disables endpoints, pointer activation threshold avoids immediate click drag; wrappers preserve menu/tooltip composition.
- `cc7519f7ffd539bf19f214a2549680cc713e70e4993b614a1a330c3c8d182cb2` L263–269 — **reviewed**：Hunk 263: Read all rail hunks: sortable wrapper receives stable locator keys, drag target indexes are checked, menu reordering disables endpoints, pointer activation threshold avoids immediate click drag; wrappers preserve menu/tooltip composition.
- `86daf09a1f6f5bd06f2b5fe90319c989b93917e6d5ca6097ed0b59411b63d8d8` L280–291 — **reviewed**：Hunk 280: Read all rail hunks: sortable wrapper receives stable locator keys, drag target indexes are checked, menu reordering disables endpoints, pointer activation threshold avoids immediate click drag; wrappers preserve menu/tooltip composition.

### `apps/desktop/src/components/WorkspaceChooser.tsx`

- `88c3e9f85f61ca3e7ee53c97407f8aa7161dfa3b70d7456cd512359ccbde8abb` L28–34 — **reviewed**：Hunk 28: Read chooser hunks: shared builtins and directory labels feed checkbox identity/selection values without changing selected tool keys.
- `7ee05c6b1a049b90aab325697cff9887f33d0e1eae92e4462de327a80b2baf41` L316–322 — **reviewed**：Hunk 316: Read chooser hunks: shared builtins and directory labels feed checkbox identity/selection values without changing selected tool keys.
- `5ca28af5a4a772395bc97438bca81dc54af3492fc4943ff8261eabeb71ad4297` L718–738 — **reviewed**：Hunk 718: Read chooser hunks: shared builtins and directory labels feed checkbox identity/selection values without changing selected tool keys.

### `apps/desktop/src/core/updater.ts`

- `0b0cad2c3811f4f9e20e702486aa85ff984059737c407f634f3bd54fb810831c` L8–14 — **reviewed**：Hunk 8: Read state-machine hunks: manual result requires checking phase, manual can foreground recheck, failure metadata accepts only closed codes/bounded operation id; no downloaded candidate exposed for manual phase.
- `84577da0d1acbbc65b7cd50a279cac1871866d1c267601a490ecda286a567f4f` L16–22 — **reviewed**：Hunk 16: Read state-machine hunks: manual result requires checking phase, manual can foreground recheck, failure metadata accepts only closed codes/bounded operation id; no downloaded candidate exposed for manual phase.
- `d7dc4d304283909d5a0e4d9f46627d5ac4337e90e54e507616c7fb38f45e7abb` L25–48 — **reviewed**：Hunk 25: Read state-machine hunks: manual result requires checking phase, manual can foreground recheck, failure metadata accepts only closed codes/bounded operation id; no downloaded candidate exposed for manual phase.
- `3d6d2bb06ca3d5e6f7a03eefa40267952e90cb79d4a21b75477a785dd647ae1a` L60–66 — **reviewed**：Hunk 60: Read state-machine hunks: manual result requires checking phase, manual can foreground recheck, failure metadata accepts only closed codes/bounded operation id; no downloaded candidate exposed for manual phase.
- `2a9b85db3c58bdf3c53a3e56921e0bba879e97c284459cfdc882561866a07af0` L80–87 — **reviewed**：Hunk 80: Read state-machine hunks: manual result requires checking phase, manual can foreground recheck, failure metadata accepts only closed codes/bounded operation id; no downloaded candidate exposed for manual phase.

### `apps/desktop/src/core/whatsNew.ts`

- `7a6f3509f8a67f396ade065724862614812cc822dd681775fbb32194e344a390` L26–53 — **reviewed**：逐段讀取 SemVer 比較：四段版號沿打包規則正規化 build prerelease、core 數字順序、正式版優於 pre、pre 數字與字串順序。

### `apps/desktop/src/i18n/messages.ts`

- `706487ba9931355594e75561524f90ddda7862892af1937d9054171edc905227` L4–30 — **reviewed**：中英文路徑與版本來源訊息；placeholder 無錯配。
- `608998d165b11bf3b046cc10893d500fa6465a1e20bb6f2e8475b62ed4178151` L65–71 — **reviewed**：資料 help 配合可切換共用路徑。
- `778db6efb82dac3c4803161a0b1b0d562b2b6228cb4a441fc99568c5a0ef7b90` L101–114 — **reviewed**：安全驗收及保留政策說明符合 checkbox 語意。
- `361190fef308b7c55447ea202b7133e45193aa785006f802441b3a51c146551b` L316–326 — **reviewed**：最後檢查狀態文字 key。
- `875e016a7fca55266c826baf2a2c695000764c52c90c7b5352a7557846f0f45f` L473–487 — **reviewed**：封存保留數量 placeholder 三者完整。
- `4e9b452b38137905f5faea0e19473643322878e88bc85e051d4e58036928ff91` L663–670 — **reviewed**：專案排序文字 key。
- `8a3c54822f72acbc492d813f024492297bcd8b2828e63d29db91ac5aa3ecb434` L676–702 — **reviewed**：英語路徑/版本來源訊息，對應中文 keys。
- `5d804b17930765a717ea1f3ef72db91479831f33e6c37c399ead79ae65dbca41` L737–743 — **reviewed**：英語標準資料 help。
- `f7300b7178e39ca8352d895fb8ce0071cec893f4cbbed8cf6bfe94c3480a6874` L773–786 — **reviewed**：英語驗收及保留政策 keys 對齊。
- `a2c828713010d5303975ce24b3dc8fdcbf3fd800c7308d35fe33f08fe7c5782d` L990–1000 — **reviewed**：英語 updater 最後檢查狀態。
- `c06f0646c14977d78d92e0c8b2074abc9a670dec91ede55cac0af5d45e413a2a` L1157–1171 — **reviewed**：英語封存保留數量 placeholders 對齊。
- `4366ecb1f16e811c490a5c8477a6078cb1cb87e1722ecf5ac4b536ba80d50f49` L1347–1354 — **reviewed**：英語專案排序 keys。

### `apps/desktop/src/session.ts`

- `82cabbe3de3c82f71f6b46e4739e3582844b78fa1ae0d09473fdbb1516b7b40f` L49–56 — **reviewed**：Hunk 49: Read session hunks: new optional capability booleans preserve older payloads, remote check-history/securityV2 flags passed explicitly true-only and checkout root kept scoped to remote datasource.
- `b7709660871209e850bf34518775e122c8331b51bb3cfaca46f6f0bd61f5190c` L124–131 — **reviewed**：Hunk 124: Read session hunks: new optional capability booleans preserve older payloads, remote check-history/securityV2 flags passed explicitly true-only and checkout root kept scoped to remote datasource.
- `bd66d0c755832698c9b42e941d0c023b63d342749b2b4e098bbab454d4923884` L420–426 — **reviewed**：Hunk 420: Read session hunks: new optional capability booleans preserve older payloads, remote check-history/securityV2 flags passed explicitly true-only and checkout root kept scoped to remote datasource.
- `2ab6dde2ddd79d3c28c570cb2c9da4c4ec4de42c596f2c89926b251e25d04b01` L428–435 — **reviewed**：Hunk 428: Read session hunks: new optional capability booleans preserve older payloads, remote check-history/securityV2 flags passed explicitly true-only and checkout root kept scoped to remote datasource.

### `apps/desktop/src/store.ts`

- `0b1848fad0f0cb50cc142fdf341058549ec300df318610f6aac7c322b62feb61` L43–49 — **reviewed**：新增 reorderTabs import。
- `1159367e0cc3cde67dc04131a3c3ab476856506fe67307de184c4c3af28b5d78` L68–75 — **reviewed**：新增 UpdateCheckResult/import 跟 updater state 使用相符。
- `90afd960ed8eb93206f29aa5e1a37616cedef3f54c5fabc439ee301cfdbf6b5e` L298–304 — **reviewed**：archivePreview 初始狀態型別。
- `799386e958377f3df442836cc052c0bbf8ec1ed7ad40bf2f689cac6f5e56ab5c` L442–448 — **reviewed**：公開 reorderProject action 型別。
- `505e607d61ae60bb39b993f85c5c8ea9046030020f0f6e2bd33af62540ebca0b` L492–498 — **reviewed**：最後檢查結果 state 型別。
- `b4aad8f35789847b1f9db547d81f687673abae7c4aadbde2ac0d6101021e81f6` L646–658 — **reviewed**：idle gate 同步清空 preview，counts 包含三類與 receipt。
- `866eb0279b3cddf2d2d643ab7256cbb30912ccac1dd1e1405654280412a902b3` L907–918 — **reviewed**：封存後依 counts 顯示 retained/omitted；只在成功後清 detail。
- `11338cfeffc5fdfc82a6668981522e76180f8e8769e1308061999d815eb76543` L1580–1618 — **reviewed**：requestArchive 並行讀紀錄與設定，identity guard 防舊請求覆蓋；loading/error 阻擋確認。
- `d17ddc92b775760fe3db732f5541271e83f5360ee241afda6da1e48345472012` L1644–1650 — **reviewed**：carry ticket 跟一般確認共用 preview gate。
- `a5d9753cf6167df3ca86e98003e865a4fe298f5ddf89d42f20d52ebfef262ef5` L1978–1984 — **reviewed**：updaterLastCheck 預設 null。
- `08b768fe36fa2ce0f41587882afeef3483792e5be3f965f2804c68acc1c739e9` L2001–2026 — **reviewed**：檢查結果區分 manual/available/up-to-date，pendingUpdate 只接受可安裝結果；錯誤保存診斷。
- `6c6008669b38ef2200bd76775a8db35a81509b48c1d385c55dde8a2cd3d3d798` L2526–2542 — **reviewed**：排序 action 先持久化成功才更新 state。

### `apps/desktop/src/tabs.ts`

- `7f7254682cb09ec5fd76e0df36a4e52ddd08ac84682757b79d4e167de90d28fa` L43–57 — **reviewed**：完整排列驗證包含 length/unique/existence；keys map 回原 tab 物件，不重建狀態。

### `apps/desktop/src/views/AppSettingsView.tsx`

- `d9baca43657a0666006c9c28ceaf1fa8901a4850f3a39a06ef8c0355b154bb83` L14–22 — **reviewed**：Hunk 14: Read AppSettingsUpdaterProps hunk: lastCheck/onDiagnostics optional, older adapters remain assignable and UpdatesPage consumes same types.

### `apps/desktop/src/views/project-settings/ReviewRulesPage.tsx`

- `77ef145a3e02dfeafa07591b4324838780e5673d074c39ea3d16b692e490f3ec` L90–96 — **reviewed**：Read newStart 90–96 with surrounding review-rules UI: baseline smell expansion button/list and enclosing row/card structure preserve existing state and visibility; changed JSX layout has no concrete behavior defect. Hunk at new line 90.

### `apps/desktop/src/views/project-settings/SpeclinkPage.tsx`

- `22f8c700d26ad1fa1f81d77cca9a6c853b8a688b2f40458a9500af47556c35c7` L10–16 — **reviewed**：Hunk 10: Read project-settings tool hunks: shared builtin selection values and visible root labels stay aligned with Switch htmlFor keys.
- `31c6a183e0b8095b72ebb5ec511864d6a1b22b7abeed81dba3539465d0210626` L42–49 — **reviewed**：Hunk 42: Read project-settings tool hunks: shared builtin selection values and visible root labels stay aligned with Switch htmlFor keys.

### `apps/desktop/src/views/project-settings/WorkflowPage.tsx`

- `946469b1c925d56eb29c1c8708ab8f16a0a409aaf467ddb55820e54d01c83707` L5–11 — **reviewed**：Read WorkflowPage hunks with control context: security required inactive/unsupported states and archive retention controls are explicit policy edits, disabled for unsupported remote capabilities. Hunk at new line 5.
- `a12248172fdabfc2df16dfb1d1043b28b5108d037eadaaa50dca6c8eb9f85d8c` L161–170 — **reviewed**：Read WorkflowPage hunks with control context: security required inactive/unsupported states and archive retention controls are explicit policy edits, disabled for unsupported remote capabilities. Hunk at new line 161.
- `071995c05cfa87fb0df67ebecd572dcd378b9da4f2c56995c8be7a3e680950e7` L189–201 — **reviewed**：Read WorkflowPage hunks with control context: security required inactive/unsupported states and archive retention controls are explicit policy edits, disabled for unsupported remote capabilities. Hunk at new line 189.

### `apps/desktop/src/views/project-settings/useProjectSettings.ts`

- `f44747ffa97bea4c5df49c85dbe784f414f00c3e3c4a98ab0626d9add9d70d0b` L10–16 — **reviewed**：引入 checkRecords patch 型別/預設/套用工具。
- `38b5784242a051785227385db563caca76d6477da621ba82e08c2dd83fb8ef6e` L104–110 — **reviewed**：新增保留設定 draft state。
- `28c99fc57d39d024994935bb4ee7f13941c29eb3bc15b48358e9ff50a92cd842` L161–167 — **reviewed**：新快照更新 checkRecords。
- `be7fae2537427904eca0d670b5a2078f20f7ff09ba1e24c76940674204d40425` L223–230 — **reviewed**：policy dirty 包含保留 patch。
- `cdced51ac73654603b1c9c3bae94e0b529489ec47e16180d4f914ba661697ccd` L238–244 — **reviewed**：非編輯區域採最新 checkRecords。
- `5f85f8b1b380bd841f75de5638e66bcc4c2f203e5b183f64ac76912d5c19cdd4` L263–269 — **reviewed**：成功寫入在本機 snapshot 套精確 patch。
- `fd86fdfbaca4f8c189e2e7792bcb2ff7f43ac7a859ae771fde1fad986f2c7972` L359–366 — **reviewed**：CAS 面板遞迴列出 check_records 衝突。
- `05037e49b504748c9e04c0809aa11f96b7983cb9d4a39d03745428f3d421be1d` L416–422 — **reviewed**：save 前比較 draft 產生 recordsPatch。
- `ada7486e21b27c3b522fd7bfe34bc259a0c3c9c689a5a3f9f600bf5b5bb43cbd` L424–430 — **reviewed**：只傳有修改的 checkRecordsPatch。
- `375a191e76a94f154aba1390f68657deb3a9a8408ef9c581c11c3e10f6b65605` L684–690 — **reviewed**：衝突重送後以最新快照套 records patch。
- `3bea7254da3c2560e84b4905884e23222ad9ebcd6607dc992cc9f31d21d22323` L744–751 — **reviewed**：公開保留欄位並 cancel 時恢復 snap。

### `apps/desktop/src/views/settings/AboutPage.tsx`

- `6fba02c8acc01b656d0e21db819b31eba5c458aa632996f61831388b8195ee0a` L28–34 — **reviewed**：About 主要 repo/issue/docs 換 fork。
- `34d36d016cfa8356d153952a3216732dd924d2c9b31b616bdb5ec5c03d81b562` L110–116 — **reviewed**：另保留原作者來源列，開固定可信 URL。

### `apps/desktop/src/views/settings/SecurityDataPage.tsx`

- `ed8a5b3388c927dbbe4b0bca8619f3e390d393f951501a153601f6cd05ce3062` L1–6 — **reviewed**：Read lines 1–120 covering all assigned hunks: path settings busy/read/write/default guards, request generation invalidation, inventory updates and action disabling preserve path ownership. Hunk at new line 1.
- `863098f64a2a81ca121f58cc575c8ca52587bcb2d5ed929d60e5b5d9bff47011` L14–33 — **reviewed**：Read lines 1–120 covering all assigned hunks: path settings busy/read/write/default guards, request generation invalidation, inventory updates and action disabling preserve path ownership. Hunk at new line 14.
- `1b8257cc8f3c52e67f47884f077b34ee091a9a96d3a9013886440ff0faf3bbb8` L47–69 — **reviewed**：Read lines 1–120 covering all assigned hunks: path settings busy/read/write/default guards, request generation invalidation, inventory updates and action disabling preserve path ownership. Hunk at new line 47.
- `058c73b9adeff8b317af86a16cdcccdefd743ae31958be8e9176e1418b22cb17` L77–100 — **reviewed**：Read lines 1–120 covering all assigned hunks: path settings busy/read/write/default guards, request generation invalidation, inventory updates and action disabling preserve path ownership. Hunk at new line 77.
- `97cb5087aef02a48350c873183cf3c0f2b5decf2bb7e840030c7368868f147ae` L111–119 — **reviewed**：Read lines 1–120 covering all assigned hunks: path settings busy/read/write/default guards, request generation invalidation, inventory updates and action disabling preserve path ownership. Hunk at new line 111.

### `apps/desktop/src/views/settings/UpdatesPage.tsx`

- `47b82601620724ad4c3a3e0aba78ed5a1b09aa984fea22b9f1e74de4480b1bcd` L1–7 — **reviewed**：Hunk 1: Read every UpdatesPage hunk: manual phase is displayed, independent source lookup updates busy/error/checkedAt state, header triggers both own-release lookup and updater, download links use desktop release data, last-check diagnostics and optional release dialog are connected.
- `00acf20ef1ef3c599db666edd1a5a17c5827d310dbdb1456701394d5e095a88d` L14–25 — **reviewed**：Hunk 14: Read every UpdatesPage hunk: manual phase is displayed, independent source lookup updates busy/error/checkedAt state, header triggers both own-release lookup and updater, download links use desktop release data, last-check diagnostics and optional release dialog are connected.
- `88a94fac8670c20fb623026ca41e3d684482548471acd6e24cbeac3707a25a1e` L27–34 — **reviewed**：Hunk 27: Read every UpdatesPage hunk: manual phase is displayed, independent source lookup updates busy/error/checkedAt state, header triggers both own-release lookup and updater, download links use desktop release data, last-check diagnostics and optional release dialog are connected.
- `c00c7c363e63a3317eab8bade4c0b6f280122366cfbd385366808154fb3153fa` L51–79 — **reviewed**：Hunk 51: Read every UpdatesPage hunk: manual phase is displayed, independent source lookup updates busy/error/checkedAt state, header triggers both own-release lookup and updater, download links use desktop release data, last-check diagnostics and optional release dialog are connected.
- `031ba88fc489821cee1505e4685aa90cfbe5ed204fab6c5b19aae6ddedd49f7a` L86–92 — **reviewed**：Hunk 86: Read every UpdatesPage hunk: manual phase is displayed, independent source lookup updates busy/error/checkedAt state, header triggers both own-release lookup and updater, download links use desktop release data, last-check diagnostics and optional release dialog are connected.
- `e98f964ac04accb31d986acd7350fc220145855c93ff24f83435a479ebd8763b` L99–115 — **reviewed**：Hunk 99: Read every UpdatesPage hunk: manual phase is displayed, independent source lookup updates busy/error/checkedAt state, header triggers both own-release lookup and updater, download links use desktop release data, last-check diagnostics and optional release dialog are connected.
- `640aa665af9dffb13e3088330dd85fe2d56d88cc45cae3728ae563c1ebb10cf8` L121–157 — **reviewed**：Hunk 121: Read every UpdatesPage hunk: manual phase is displayed, independent source lookup updates busy/error/checkedAt state, header triggers both own-release lookup and updater, download links use desktop release data, last-check diagnostics and optional release dialog are connected.
- `98f8ed31b41879437c8fd18a39c5797ca6299c971fd26216f489f19247787c5c` L191–204 — **reviewed**：Hunk 191: Read every UpdatesPage hunk: manual phase is displayed, independent source lookup updates busy/error/checkedAt state, header triggers both own-release lookup and updater, download links use desktop release data, last-check diagnostics and optional release dialog are connected.
- `8fd240a8c1e0b72e0938c5c1eb756595e148b61fe052bdbd081d7b324807701b` L210–223 — **reviewed**：Hunk 210: Read every UpdatesPage hunk: manual phase is displayed, independent source lookup updates busy/error/checkedAt state, header triggers both own-release lookup and updater, download links use desktop release data, last-check diagnostics and optional release dialog are connected.

### `crates/adapters/speclink-cli/Cargo.toml`

- `a601b172cae0ec703a5206fc0e4f9b21eb0497c97b3f7a9ea5e77bcfd512501b` L24–30 — **reviewed**：CLI dev zip dependency 支援 offline fixture 構造。

### `crates/adapters/speclink-cli/src/common.rs`

- `ee260ca050ca7a0d072c180bd9b73301dd588a93e42b825c106d0273586117eb` L45–58 — **reviewed**：Hunk 45: Read common.rs hunks: proof computed only for acceptance-bound verify/archive, optional inferred sole change, configured shared-root lease retained, policy disabled returns before knowledge access and authorized checkout binding is constructed fresh.
- `b74ccfc7650d995fa2dd2f7c8b30b3c0b85d315236c4d80784b7c121d45325ff` L63–86 — **reviewed**：Hunk 63: Read common.rs hunks: proof computed only for acceptance-bound verify/archive, optional inferred sole change, configured shared-root lease retained, policy disabled returns before knowledge access and authorized checkout binding is constructed fresh.

### `crates/adapters/speclink-cli/src/verbs/config.rs`

- `4f2267fa9c77982af8873c221278d4e530e95017622bfc37aeebdb30b453e60e` L90–102 — **reviewed**：CLI 新增 check-records stdin/dry-run。
- `e92aa4cef55f067d2222e28f60abdf36c832c5ce95924e0a0ae44ca3a0f0b69b` L323–329 — **reviewed**：show 輸出 check_records。
- `9655cb8a4658b3cd4a1b7949087eb3127ea3437a866eabab8e49bfbb1f7a1d79` L334–340 — **reviewed**：write enum 區分 CheckRecords。
- `625b058ff97999123b12b8516bff47b32fbd0e9943c0c1de99ce8dab51dde0bc` L460–470 — **reviewed**：共用 stdin 限制但依 cmd 分流。
- `31b6ec50a636db6f68b4a01f1ca04594d45ab8564ea6e5446178205ddccd498a` L472–478 — **reviewed**：64 KiB 限制，建立正確 write 變體。
- `a28ea9a5a961f140b998b22ccdf882658af8620f9e2c9fdd898ab09c2804197a` L532–538 — **reviewed**：JSON show 從 cfg 取預設保留。
- `2f95732da26f251c149720c6def9ebc0b5044ca98b699f19e0d92f7419d56725` L639–650 — **reviewed**：CheckRecords 呼叫同一純 patch 函式。
- `e210ef5b0258d53ca6ebbe395f8207e26a7b9b82bc490182045dbe31b68d2342` L883–890 — **reviewed**：遠端接受/保留政策先驗 v2+history capability。

### `crates/adapters/speclink-cli/src/verbs/init.rs`

- `314082ddd182c6b10996e1256241b2dc5a00e688814f5a8143a829f6d1cf864e` L16–22 — **reviewed**：Read all assigned tool-selection/help/tests hunks: Antigravity joins four-option interactive selection; answer fixtures updated to four prompts and EOF/empty safeguards preserved. Hunk at new line 16.
- `9541b1cee50af1c1716e19f91ca16b8b1a33b72075034e0ecddf87f26f9521df` L87–93 — **reviewed**：Read all assigned tool-selection/help/tests hunks: Antigravity joins four-option interactive selection; answer fixtures updated to four prompts and EOF/empty safeguards preserved. Hunk at new line 87.
- `744f02b43a325719be4279656c5c6fa0b82036f9398e391eff4aff938c4f3c90` L111–117 — **reviewed**：Read all assigned tool-selection/help/tests hunks: Antigravity joins four-option interactive selection; answer fixtures updated to four prompts and EOF/empty safeguards preserved. Hunk at new line 111.
- `7b1f60c3dea34acf69b3f2452a73ef4396ee909de3d3726ec7d7a8d637788f83` L125–131 — **reviewed**：Read all assigned tool-selection/help/tests hunks: Antigravity joins four-option interactive selection; answer fixtures updated to four prompts and EOF/empty safeguards preserved. Hunk at new line 125.
- `57cf5df35e9ce2149809cac73b570da13f34962dc72bb5b2e6099996b9dd98c5` L136–142 — **reviewed**：Read all assigned tool-selection/help/tests hunks: Antigravity joins four-option interactive selection; answer fixtures updated to four prompts and EOF/empty safeguards preserved. Hunk at new line 136.
- `cbfbfcbdf329d005dc14e9947090194d582cdde13de2ba1ca799cb4dbbbb78d3` L269–275 — **reviewed**：Read all assigned tool-selection/help/tests hunks: Antigravity joins four-option interactive selection; answer fixtures updated to four prompts and EOF/empty safeguards preserved. Hunk at new line 269.
- `57b3de7606b046663ad7e226578b0f09e596bd6da3a63ebe49db708b6cb4a88c` L315–321 — **reviewed**：Read all assigned tool-selection/help/tests hunks: Antigravity joins four-option interactive selection; answer fixtures updated to four prompts and EOF/empty safeguards preserved. Hunk at new line 315.
- `48a90d4adcd25db45e8b57b458144cfd226ff684874b81203fe1871cf594bd84` L323–330 — **reviewed**：Read all assigned tool-selection/help/tests hunks: Antigravity joins four-option interactive selection; answer fixtures updated to four prompts and EOF/empty safeguards preserved. Hunk at new line 323.
- `17bb253d50e862698f05d3f94d26817c7df9e85d6abecda2b0c4d1e6ec7efc1a` L332–385 — **reviewed**：Read all assigned tool-selection/help/tests hunks: Antigravity joins four-option interactive selection; answer fixtures updated to four prompts and EOF/empty safeguards preserved. Hunk at new line 332.

### `crates/adapters/speclink-cli/src/verbs/lifecycle.rs`

- `e10bbc0bdd066bcb35632c00cb4cabdb6d1a07a64461addf5a8237aace6517f4` L46–52 — **reviewed**：ArchiveOptions 初始化無預先 proof，後續 Host 注入。
- `b91232ed7b049f03fa88d56ad0479d4f4f7ee7c1460d3e4e1a74aebf4020706f` L95–101 — **reviewed**：成功封存輸出各類 retained/omitted 及 receipt。
- `7821f4ba97fae5472cfaf336146fcca8b666f9db8acacb175c31a33c38ebab7e` L264–274 — **reviewed**：Remote archive 讀有效 security policy，必要時送 attestation gate。
- `a51255a37dc5436536d9c62ca295c3ea49066a2aca48d55f3d6fb2177ebf14cd` L321–327 — **reviewed**：wire counts 轉 core 同形欄位，舊 server 缺值回預設。

### `crates/adapters/speclink-cli/src/verbs/security.rs`

- `29d0ab62b341dd8b5b1ac0fb8aeae70e3f06446de070582e22debcd3536b46a0` L23–38 — **reviewed**：CLI Review/Kb variants route explicit commands; existing Run/Prepare retained.
- `a1fbb1ae66b9ac2f4d9d04bb2ea22a8b39d7fd013d215dbae9f81936fc7d8408` L45–52 — **reviewed**：Optional --change selects proposal report repository; device report remains default.
- `3a3074e652f9cb9bd3e9caf5f00e96a0eb6572a23c34c4f1ecc7f9040f3561cc` L56–102 — **reviewed**：Review stdin is required; strict legacy and deep input payloads disambiguate; kb args carry root/change.
- `38800f54f89495c128ea4ee03ddbf367bdaf9531499d2480b3ca3dc0fd3414d8` L210–241 — **reviewed**：Report --change resolves correct local/remote storage and validates versioned export without data dependency.
- `f6adc432d4874cf6cd772514ea22251543a6bc0581a09bfa53c90792c17f53cb` L244–295 — **reviewed**：Data repository is constructed only for commands requiring it; local proposal Run saves via ProposalReports; remote Run remains device draft.
- `198893661e963dd275aedc22c4b0f42a6b68744a83aa2b782f5df86cdfe3a57a` L298–304 — **reviewed**：Remote draft warning occurs only for remote change runs.
- `11f376b91fae87406b5fe2d496817e44e575900405bcff34568ffbaf9f19e73f` L338–346 — **reviewed**：Data commands acquire shared configured repository at their boundary.
- `d797e150918ca7441bb9347bcc866a4ec47d4710d407b8a93829983721f07d8f` L530–536 — **reviewed**：Match branches remain exhaustive after Data nesting.
- `10b8d2c57b754128afa8513e722c81ebde0e066b86aeb9f8b15bb490bbb277d6` L560–646 — **reviewed**：Reviewed local Show/AddRound/Accept: bounded stdin, change checks, identity, freshness context and explicit legacy import; human JSON output fields match protocol.
- `dbd3550744b7da01f8de156714df72428e8d76e625a9a17abb3dc82e94a1a635` L690–696 — **reviewed**：Remote target starts with no canonical proposal key until server preparation supplies it.
- `e9c54beec58a08371dedd729f96b6b3885b2ae540c0c3760188e270d376ccd94` L710–719 — **reviewed**：Prepared header restores canonical key while keeping checkout cache key.
- `213af91cde57e399386419b6e583b345285dc73e5c0c0f174fba1eecda42481b` L812–818 — **reviewed**：Fetched binding carries optional canonical security project key into preparation.
- `bb84b89c3cada1b039167d6b014c012a5c4e62f020a61b2e0bf224d8951b57b8` L828–836 — **reviewed**：Server canonical key is retained in remote target before building preparation.
- `08a191e96c5b2f43e28b69b64e241f506f9fe10a4036551e019809aa6225727d` L857–945 — **reviewed**：Reviewed remote show/check, builtin retrieval, client redaction, CAS submit, legacy binding and acceptance frame creation; gates use fresh local attestation.

### `crates/adapters/speclink-cli/src/verbs/station.rs`

- `88dcf951f37e5a84fc5e0ac7187044cea1a02914ba8502e715281f945bb11ece` L21–28 — **reviewed**：Read all history command mappings, local/remote query and verify stamp branches; bounded reread recovered the one aggregate-truncated line at 825–834. Required verify calls attested gate, reference mode uses ordinary stamp. Hunk at new line 21.
- `681ddfc4c2f701d690b749e8460587b06ec99542882e48aab0c8f73d835561c5` L83–90 — **reviewed**：Read all history command mappings, local/remote query and verify stamp branches; bounded reread recovered the one aggregate-truncated line at 825–834. Required verify calls attested gate, reference mode uses ordinary stamp. Hunk at new line 83.
- `cd9c69ba33913cf61f698793281f4f59d13f445a55fe90c907ac9a4db90b36f1` L130–136 — **reviewed**：Read all history command mappings, local/remote query and verify stamp branches; bounded reread recovered the one aggregate-truncated line at 825–834. Required verify calls attested gate, reference mode uses ordinary stamp. Hunk at new line 130.
- `10f947e01655c3b35fd0f811fb4d9ebf18c695912d6c4d8c87d440cd473ce532` L169–189 — **reviewed**：Read all history command mappings, local/remote query and verify stamp branches; bounded reread recovered the one aggregate-truncated line at 825–834. Required verify calls attested gate, reference mode uses ordinary stamp. Hunk at new line 169.
- `a81c92835b585919e20abc9c16c9d614aa9199618fc452b356fba4a7713c1996` L203–209 — **reviewed**：Read all history command mappings, local/remote query and verify stamp branches; bounded reread recovered the one aggregate-truncated line at 825–834. Required verify calls attested gate, reference mode uses ordinary stamp. Hunk at new line 203.
- `685de2ba54c14daf0896a3e6a2a276e5dd68d6fc4ab07f628e7dcee85138d798` L245–254 — **reviewed**：Read all history command mappings, local/remote query and verify stamp branches; bounded reread recovered the one aggregate-truncated line at 825–834. Required verify calls attested gate, reference mode uses ordinary stamp. Hunk at new line 245.
- `1e039c9ee2611f4048fced88d2c366d1316ec94f39c038e92580ee39c1627663` L310–317 — **reviewed**：Read all history command mappings, local/remote query and verify stamp branches; bounded reread recovered the one aggregate-truncated line at 825–834. Required verify calls attested gate, reference mode uses ordinary stamp. Hunk at new line 310.
- `f0b99d4f3c6c4913677d8bd14a497506d1802084fd7712a1d61ba983bac8b382` L320–326 — **reviewed**：Read all history command mappings, local/remote query and verify stamp branches; bounded reread recovered the one aggregate-truncated line at 825–834. Required verify calls attested gate, reference mode uses ordinary stamp. Hunk at new line 320.
- `50604abae86abdccb5438b1e046a80aba035eed3e8c6971418ab1cbbe1328f40` L738–748 — **reviewed**：Read all history command mappings, local/remote query and verify stamp branches; bounded reread recovered the one aggregate-truncated line at 825–834. Required verify calls attested gate, reference mode uses ordinary stamp. Hunk at new line 738.
- `40fa7c672e2f0ca5e47b74f0f340138bfdfb5fb7a8593da392b8cea5a6300278` L825–834 — **reviewed**：Read all history command mappings, local/remote query and verify stamp branches; bounded reread recovered the one aggregate-truncated line at 825–834. Required verify calls attested gate, reference mode uses ordinary stamp. Hunk at new line 825.

### `crates/adapters/speclink-cli/tests/it/archive_readiness_gate.rs`

- `e509104d3ce468f19a9f70e3ee1a35faf78d6774916c7b3b1905387a83fb77f8` L18–41 — **reviewed**：Read both archive gate test hunks, reread 18–41 and 433–441 after aggregate truncation: required security blocks even skip-specs/no-validate/mark/carry flags; outcome now checks history counts. Hunk at new line 18.
- `0143c4bf89bb9de53d61204a90b1258751ab03e78cc624642f2f0943882cc42e` L433–441 — **reviewed**：Read both archive gate test hunks, reread 18–41 and 433–441 after aggregate truncation: required security blocks even skip-specs/no-validate/mark/carry flags; outcome now checks history counts. Hunk at new line 433.

### `crates/adapters/speclink-cli/tests/it/init_tools.rs`

- `c52eb7a1de5c79d2f7d9cc668e9163b4bf8d4bb67a072326115b0c32732a91c1` L353–369 — **reviewed**：Read init_tools test hunk: unknown standalone vscode rejected with zero writes, supported tool list assertion remains. Hunk at new line 353.

### `crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs`

- `2796ee46547490da57d0b768daa8a1758ee5438940791f083ffb9367e36724f0` L28–37 — **reviewed**：Read remote parity test list hunk: adds explicit verify/review history invocation to CLI surface checks; fixed valid flags. Hunk at new line 28.

### `crates/adapters/speclink-cli/tests/it/review_verbs.rs`

- `b64081c48e6d44b0f60a27d48170632cd86b423419bcda198345449c418a1963` L142–176 — **reviewed**：Read review history test hunk: empty/list/record metadata, immutable full original ticket, invalid/missing ID and archived dated query cover new CLI path. Hunk at new line 142.

### `crates/adapters/speclink-cli/tests/it/security.rs`

- `954874e1efa7574e1481a22e2c2b8ab1dd9e9f82ea1bb0f5c29e8791bf47df37` L17–123 — **reviewed**：Fixture isolates user/data directories; tests cover configured standards root, unavailable mount, private report read and strict workflow stdin contract.
- `b53ac285f3ef771a3d4978559738236294167c713f211949db66a5cfb33b4fd7` L125–146 — **reviewed**：Missing knowledge preparation must fail atomically for root and change targets; invalid stdin flag rejected.
- `2bd2835a9d512654597c13661445eda8655b972e84826542f3c1cca9d0fce9aa` L153–160 — **reviewed**：Inventory checks stdout JSON and stderr separately.
- `4b7694d3ead66ab336c9b877f84901ef20c76df15e15d0868ae8e794f43f7e5a` L530–544 — **reviewed**：Offline fixture adds the authorized local guidance archive before installation.
- `1f79017633f6a0f1a22dfbd6fcf446f1d4a9d9223680429e3f1135e85dfb6200` L562–585 — **reviewed**：KB preparation tests selected installed standard only, receipt identities, no ANSI, and human fields.
- `6f742ecb73dd0e11142fdfb9c64cb165f78855d29fd004a3208e0667b1a5b6ca` L601–608 — **reviewed**：Offline run checks both output streams and network trap.
- `bc73a6320616aa4b5ddba4a174841e682cb0ee37be645fa09346ec7b5f6e5479` L622–709 — **reviewed**：Integration covers current builtin/deep linkage, secret redaction, payload bounds, invalid targets/version/digest, duplicate checks, explicit legacy import and accepted-risk identity/receipts.
- `60ea8afac52e987ff54d02a200b9ce57d724ade54697dc34789071480691cfac` L713–722 — **reviewed**：Archived report and review reads are checked using actual archive change name.

### `crates/adapters/speclink-cli/tests/it/verify_verbs.rs`

- `217bc45a1380db07f7ee6fcf6b3833dec8a5904a7a38722dbc3a2989fe211026` L18–58 — **reviewed**：Read 18–58 with context: ordinary --accept cannot bypass required security; completed verify history metadata and dated archive queries, invalid/missing IDs tested. Hunk at new line 18.

### `crates/adapters/speclink-node/__test__/dispatch-contract.spec.ts`

- `cfdf94eba2bbdb996713dab4c59bd512c641069cf1222167dad7dc876d156375` L107–116 — **reviewed**：Node dispatch contract 回歸測試 security review 缺 subcommand 為 invalid_argument，非 panic。

### `crates/adapters/speclink-node/__test__/helpers.ts`

- `d906ffb773e776ccf0cd0d1b22524e3d3917a679e337702d806a75a82909e94d` L1–4 — **reviewed**：Read all helpers hunks: in-memory JS store atomically compares expected SHA-256 before replacing next artifact map, archived artifact query reads saved archive snapshot. Hunk at new line 1.
- `8aa60a0655e58e04994a26f5fcfab48b375646738440b5674ef5173423e2defe` L175–191 — **reviewed**：Read all helpers hunks: in-memory JS store atomically compares expected SHA-256 before replacing next artifact map, archived artifact query reads saved archive snapshot. Hunk at new line 175.
- `15b6f9a086751b8d3a6fec0089de05fd0711fda999bbab4c99a29e0a444c6660` L226–235 — **reviewed**：Read all helpers hunks: in-memory JS store atomically compares expected SHA-256 before replacing next artifact map, archived artifact query reads saved archive snapshot. Hunk at new line 226.

### `crates/adapters/speclink-node/__test__/render.spec.ts`

- `fa2054a3db3eafad1f25846ab3b6897ce5a6c1a2728edf352e5091ed6466cf56` L43–70 — **reviewed**：Read all three render test hunks: neutral options/invalid invocation, target choices and Antigravity independent .agent generation compared to CLI, cleanup in finally. Hunk at new line 43.
- `5ab124b74f8f0c69bc66ac6d851553396dd1e18c355027878daa455a8c8627e0` L120–126 — **reviewed**：Read all three render test hunks: neutral options/invalid invocation, target choices and Antigravity independent .agent generation compared to CLI, cleanup in finally. Hunk at new line 120.
- `c0493c586fae69356240cb675347f0c2c2283cb9f87bcfe43b7f3d1dab9a1d9c` L134–149 — **reviewed**：Read all three render test hunks: neutral options/invalid invocation, target choices and Antigravity independent .agent generation compared to CLI, cleanup in finally. Hunk at new line 134.

### `crates/adapters/speclink-node/__test__/review-verify-verbs.spec.ts`

- `791fa56e63670a3362d53c6f3e4888810edb5f5b828f3c09b8432c032b666177` L176–217 — **reviewed**：Hunk 176: Read Node tests: completed ticket history survives new round/archive, missing commitChecks refuses before metadata/index writes; JS-only security returns unverified rather than borrowing filesystem checkout.

### `crates/adapters/speclink-node/__test__/types.spec.ts`

- `64f512b3160ab7db61944aa6a9d9d073046018a555cc4a8df62e52319632f762` L1–15 — **reviewed**：Read full 15-line new types test: invokes installed TypeScript compiler with consumer tsconfig, asserts process error/status. Hunk at new line 1.

### `crates/adapters/speclink-node/__test__/types/render-api.ts`

- `de561397938a09f00f44a7936de3132a2fe6337e43dd78f2944c8aa8bb23ef98` L1–22 — **reviewed**：公開型別測試從 package types entry 導入，五種 target、有用選項，以及不支援 invocation/store/worktree/舊 instructions 匯出需 TS 拒絕。

### `crates/adapters/speclink-node/__test__/types/tsconfig.json`

- `717719799a21a8c1119e171a7294f05bc4c521a3b69733bf22dd179361666e12` L1–11 — **reviewed**：TS 型別測試 Node16 resolution、strict/noEmit、只編 render-api，符合 package entry resolution。

### `crates/adapters/speclink-node/index.d.ts`

- `45d4af7063fa903fe81b1fb2c0261b89ad420d90bf517d564d709056a86a0d04` L52–64 — **reviewed**：Hunk 52: Read type hunks: atomic writes/deletes/expected map matches core ChecksCommit, optional commitChecks/readArchivedArtifact preserve compatibility with unsupported hosts, Antigravity added to render union.
- `12f07e5f222b875cb3f2b10d94aa56647d835e34e284180333e2cc069e651e36` L97–106 — **reviewed**：Hunk 97: Read type hunks: atomic writes/deletes/expected map matches core ChecksCommit, optional commitChecks/readArchivedArtifact preserve compatibility with unsupported hosts, Antigravity added to render union.
- `25d899c998b1722d30ad56daf2bf99037e3ce270f8e93316cdca5b27d495be2e` L134–140 — **reviewed**：Hunk 134: Read type hunks: atomic writes/deletes/expected map matches core ChecksCommit, optional commitChecks/readArchivedArtifact preserve compatibility with unsupported hosts, Antigravity added to render union.
- `04b40f77110c3c6b1f8b3ea806dae3df886d87b3f3ac4314f22402c77078155f` L234–245 — **reviewed**：Hunk 234: Read type hunks: atomic writes/deletes/expected map matches core ChecksCommit, optional commitChecks/readArchivedArtifact preserve compatibility with unsupported hosts, Antigravity added to render union.

### `crates/adapters/speclink-node/package-lock.json`

- `a2983c4a873d82e62ea29ab9a59421cdcf97ec3e9f0539310bab9ddfee8fe2bf` L10–16 — **path-only**：Generated dependency lockfile; excluded from source correctness inspection per brief discipline; path-only, not claimed reviewed.
- `666fec8cb5d78c2054a38ac3eaee6d1dcc534c0f48d80663d58fbd56fc85f9e2` L1416–1435 — **path-only**：Generated dependency lockfile; excluded from source correctness inspection per brief discipline; path-only, not claimed reviewed.

### `crates/adapters/speclink-node/package.json`

- `51813860c693db616137a01bf30923cd6c6cc7511e4579ea95a53bfa3081a3f4` L34–40 — **reviewed**：Hunk 34: Read Node package hunk: TypeScript dev dependency present for declared public type checks, runtime package interfaces unaffected.

### `crates/adapters/speclink-node/src/lib.rs`

- `054b7ea2e80619abdcc015e6eb6c0acd9c3735ea4bbc6381116a754ae25fc687` L10–20 — **reviewed**：Read all assigned hunks: standard path lease, VerifyStamp acceptance proof injection, history dispatch and security show/add-round/accept/report boundary with input limits; host-store writes correctly require checkout. Hunk at new line 10.
- `ec363abc2329780d94c780dc39d7de47a1eded6344e86d65f4a634637ad9397b` L92–111 — **reviewed**：Read all assigned hunks: standard path lease, VerifyStamp acceptance proof injection, history dispatch and security show/add-round/accept/report boundary with input limits; host-store writes correctly require checkout. Hunk at new line 92.
- `6e0b2d8d81ada00832a7c80ea4f817be48b734c50372437c49b46734054c6542` L339–345 — **reviewed**：Read all assigned hunks: standard path lease, VerifyStamp acceptance proof injection, history dispatch and security show/add-round/accept/report boundary with input limits; host-store writes correctly require checkout. Hunk at new line 339.
- `54e227064d3f4a5048a67d73bfdc18d792c0e7012657120cad07cbdb76cbe6b2` L595–601 — **reviewed**：Read all assigned hunks: standard path lease, VerifyStamp acceptance proof injection, history dispatch and security show/add-round/accept/report boundary with input limits; host-store writes correctly require checkout. Hunk at new line 595.
- `25174e1771de70f3b32fa4a54ed283e9f45ac6bb70d7da2329c6e423bbb90cdf` L604–617 — **reviewed**：Read all assigned hunks: standard path lease, VerifyStamp acceptance proof injection, history dispatch and security show/add-round/accept/report boundary with input limits; host-store writes correctly require checkout. Hunk at new line 604.
- `e0f8dfb0499077bc32b0d96ae5a860542a1cb7b4a33fb5779137422a7c5ef9f8` L720–773 — **reviewed**：Read all assigned hunks: standard path lease, VerifyStamp acceptance proof injection, history dispatch and security show/add-round/accept/report boundary with input limits; host-store writes correctly require checkout. Hunk at new line 720.

### `crates/adapters/speclink-node/src/render.rs`

- `aff642f5de3607d02c60af1caf87d54bc6de960fc239a8e4b336f7765af39ffb` L1–5 — **reviewed**：Read all three render.rs hunks: Rust pass-through advertises Antigravity, parses through Tool::parse, and preserves neutral invocation matrix. Hunk at new line 1.
- `2e908bef9a813aed8980b04114fb120645f68438c4256472316779196ba58122` L15–21 — **reviewed**：Read all three render.rs hunks: Rust pass-through advertises Antigravity, parses through Tool::parse, and preserves neutral invocation matrix. Hunk at new line 15.
- `dc319874ee4ccf05c8badd75b0640bf667c48d8b24e3ae354e1893d03606990c` L60–66 — **reviewed**：Read all three render.rs hunks: Rust pass-through advertises Antigravity, parses through Tool::parse, and preserves neutral invocation matrix. Hunk at new line 60.

### `crates/adapters/speclink-node/src/store_bridge.rs`

- `b7fedf681ff01308dea9a85f71634931b4c26be3f8db2c080b12b2c38f2e115e` L109–115 — **reviewed**：JS stamp capability 預檢新增 commitChecks，避免部分 destructive 寫入。
- `a08d5825488628ebf08172f6b0dd99327364a91d406062d1ab9487b0f22c4ae8` L433–444 — **reviewed**：JS commitChecks 先驗 plan，再跨邊界呼叫 adapter 原子實作。
- `bfed70534cf7681a721206d2ef26663c27ade605975d40b00d895e996a094d53` L540–549 — **reviewed**：readArchivedArtifact 為 optional，只有 missing method 視為 None，其他錯誤保留。

### `crates/engine/speclink-core/assets/schema/spec-driven/proposal.template.md`

- `745f4a125dea82135dd3f7779224449b54555b0b37d2b3231c7fa455aa0f071e` L1–12 — **reviewed**：模板只改 Why/What Changes 註解與逐點預設，既有 section structure 保留。

### `crates/engine/speclink-core/assets/security/spec_registry.json`

- `58d00fd4914aa74d85641ff01856b72faece2420b766b3f4a68090b8010851d0` L1–118 — **reviewed**：逐段讀取 1–118：registry 固定 sourceRevision，檔名/URL/sha256 一對一，包含原始授權檔，資料無動態下載執行。未連網核對外部內容。

### `crates/engine/speclink-core/assets/skills/archive.md`

- `14d50c940d38c684dee9911945368f3a9e0a3a3b48098186ce59dfecb57a1158` L1–11 — **reviewed**：archive skill 原始碼新增 security 必要/參考模式與保留結果流程，沿用 core gate，禁止 Agent 代接受風險。
- `d2d5e19062309574cd8412aa9e8416a108c3e30295e888ec3b40a6309beaa8e1` L25–31 — **reviewed**：調整不可自行選可封存候選的指引，無影響命令正確性。

### `crates/engine/speclink-core/assets/skills/config.md`

- `afcf589c87aacb9cec72738a48802f028f991a7a1c7455c379f9b86565df0ec4` L235–242 — **reviewed**：config skill 原始碼交代 required 默認 false、保留政策僅封存時生效、dry-run/approved patch 流程，不自動刪歷史或抓資料。

### `crates/engine/speclink-core/assets/skills/propose.md`

- `c3fd557d1ace986759902e72291aeae2034088234405e52bbf2c7831337414c2` L156–162 — **reviewed**：proposal 模板註解改成逐點動機。
- `4dc9d3eb38d7b912ce0c4ba7fbf33c3528c779e651e255d4dbcf8a86e1c32db3` L449–456 — **reviewed**：skill 原始碼與 template 同步，既有 proposals 不批改，selected security 邊界保留。

### `crates/engine/speclink-core/assets/skills/review.md`

- `97c36729c3514e2cd6228b474796d487bcf018c3c6cb90d20fa56ef0fda9df94` L219–225 — **reviewed**：Read review skill source hunk: completed history instructions query immutable record rather than reconstructing/editing tickets; errors not claimed success. Hunk at new line 219.

### `crates/engine/speclink-core/assets/skills/security-review.md`

- `d8292ba571d0ca942edd4cc7c1e74e69e1cb84de0efb16e18d9a61e96c0b350d` L1–71 — **reviewed**：Read all 71 source skill-template lines, including rereading lines 25–50 after aggregate truncation. Offline resource boundary, explicit acceptance, selected controls, completeness and 4MiB submission handoff are internally coherent. Hunk at new line 1.

### `crates/engine/speclink-core/assets/skills/verify.md`

- `342eed0490fff94ee43dd10ae2276ac46971d793a17c3b69c1c851cae699aafd` L8–14 — **reviewed**：Hunk 8: Read source verify skill hunks: preliminary information uses proposal-bound report id, optional acceptance honors shared verdict and ordinary --accept cannot bypass; completed history queried only after successful persistence.
- `7363e161d0a122c4667aaae20ba716911e66d773c20ac7542740314f8ebc9ce1` L54–60 — **reviewed**：Hunk 54: Read source verify skill hunks: preliminary information uses proposal-bound report id, optional acceptance honors shared verdict and ordinary --accept cannot bypass; completed history queried only after successful persistence.
- `fb3efba81074388e542a132cab3f6cce3fd74d51e3a3d836a74dd682f8db755d` L279–292 — **reviewed**：Hunk 279: Read source verify skill hunks: preliminary information uses proposal-bound report id, optional acceptance honors shared verdict and ordinary --accept cannot bypass; completed history queried only after successful persistence.

### `crates/engine/speclink-core/src/command/mod.rs`

- `b9ba8c1b0519701e8bf951046317aa5d4fa0f85b50e45d504ba91e1849e95f9e` L124–132 — **reviewed**：Read all command-context/error/dispatch hunks: GateFailure maps refused, request Host proof passed into archive and VerifyStamp, no proof inferred from command flags. Hunk at new line 124.
- `8ba8e1796393dc43fe10ac427bf3b1b99317e10100b546ddeacc4c74d5f43c40` L735–742 — **reviewed**：Read all command-context/error/dispatch hunks: GateFailure maps refused, request Host proof passed into archive and VerifyStamp, no proof inferred from command flags. Hunk at new line 735.
- `5ccafeb19199472af5b388f26861d16f9f00e29fe89f9f1bc2cb3f564085ca91` L840–846 — **reviewed**：Read all command-context/error/dispatch hunks: GateFailure maps refused, request Host proof passed into archive and VerifyStamp, no proof inferred from command flags. Hunk at new line 840.
- `8717583c5422ea63f56829f687dd44f00b50f04669f240ccb046bda57e61df49` L927–933 — **reviewed**：Read all command-context/error/dispatch hunks: GateFailure maps refused, request Host proof passed into archive and VerifyStamp, no proof inferred from command flags. Hunk at new line 927.
- `863408ef1915d3303f66009ba7243e820c20634109448d8548f7dc7473685a27` L936–942 — **reviewed**：Read all command-context/error/dispatch hunks: GateFailure maps refused, request Host proof passed into archive and VerifyStamp, no proof inferred from command flags. Hunk at new line 936.

### `crates/engine/speclink-core/src/lib.rs`

- `170de9941dd582c860695fd3fb8481226f3f559ce1a953862246f41725b64352` L30–34 — **reviewed**：core 公開 history reexport 對齊各 caller imports。

### `crates/engine/speclink-core/src/lifecycle/archive.rs`

- `2a5e78f7c15270b590eeba960f4af01f3ed5688954c84977d0d0447f9be7fea7` L30–41 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 30.
- `a603e43a72b9102a9995ef5f1e0ea02679036b58b902f76edcb83d701d9fbb9d` L85–95 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 85.
- `528c593c1c9170d99582c9c1f69a0afaf259a3fe048833bde68ae2d3ea0ba28d` L208–218 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 208.
- `a8c0e844647ab26ad3ec2a1e8acdf72de06276ca2defb26744cb153557d3cced` L335–341 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 335.
- `48382eafcaaa3b831d54a180b8c1613646752b47003dec0c9bbd65d92bfd2e1a` L387–397 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 387.
- `3726935a8ea71f68d830efa036520b1644b7edaf3b4fb0e5bcaf74b554d235a0` L421–430 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 421.
- `16b8ead97fe7c91037b98cde7474e1e31aa37392cf9cc5c08ba7390db2c60522` L448–457 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 448.
- `3622e647d491cea7b0424452a4c4223cc72bf74571367b5a973e7800279ad2c2` L479–485 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 479.
- `eb9cb75e1095ce488dab24e5ef010b5bb2ea54c00028858bb0dff9f9dab2c454` L624–634 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 624.
- `b43357f74cdb6535d27e98a3b779a25e290413b8f61e9884c870cf2c70866c21` L637–653 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 637.
- `c90f230517357e3891cc4139e084a8f440f9fedb2c4fe08ac55fc27321cc89cd` L698–742 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 698.
- `19cc87ebb98ca07b8b6e2e20c81f6f1c6dd4061766970a38a00fa54def7825da` L762–770 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 762.
- `484744630b2cf0aa688ac22fccce2a55ad1dd1c36adab1d33cf459bba46e5d50` L775–783 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 775.
- `22bf8eaef0d7ce54bebe0d7a13207af177d13655c8be11a4245e6238f3a89b30` L802–833 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 802.
- `14af1231b80242552d32ca4ccdb72d85b80301379df379a168cf369df1e64c9e` L838–847 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 838.
- `9c0d02dce17cb80ae385b7130efa1d640cd070a573697bf25a27a6b33ba456c8` L849–859 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 849.
- `8bebcd550c0abe6c3dac357063e8f6cd0d57fc5eb33525f7bf51dab99e8a7d9f` L863–879 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 863.
- `2bea6e99939a971d3d5b7a62bb7757e91defb3c13f56e80f91bd76951227b2fd` L886–891 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 886.
- `a2f7c94a14cc3379ae4be853361c32dd9fd68343e638f7e20f2270db44a8a79c` L916–922 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 916.
- `ab65792cfadddf6c3110d11f656897e583d0183d5233e7ce476f6e17c17218ac` L1025–1035 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 1025.
- `03419f81ecff9f3dbd9c9f452017bcde9956171d315350ab38f705e4f568f66a` L1120–1129 — **reviewed**：Inspected every assigned range plus archive() gate/commit context: security proof gating and reviewed task mutation refusal occur before effects; snapshot backups now precede locked backend canonical writes/archive. Formatting-only parser hunks preserve operations. Removed direct spec writes and post-move metadata stamp were inspected in frozen patch; moved behavior remains equivalent. Hunk at new line 1120.

### `crates/engine/speclink-core/src/lifecycle/archive/tests.rs`

- `446aec295e773420c918b68b59b59ad857d62f71f17788f9e9c0bfa7bb442393` L29–35 — **reviewed**：Hunk 29: Read changed archive tests: new options use no security proof for disabled policies; injected index conflict asserts no canonical writes; all 8 retention masks assert record omission and carried live tickets.
- `ad03e5c33c2c8e4e13c45611529c3633b8d9442a97a523665bce996eeef876ce` L215–221 — **reviewed**：Hunk 215: Read changed archive tests: new options use no security proof for disabled policies; injected index conflict asserts no canonical writes; all 8 retention masks assert record omission and carried live tickets.
- `4b4eda1b902a54553e84f603c0ae7ee9f94f0cfb152871afcb12ebaed1674711` L1238–1263 — **reviewed**：Hunk 1238: Read changed archive tests: new options use no security proof for disabled policies; injected index conflict asserts no canonical writes; all 8 retention masks assert record omission and carried live tickets.
- `cb1f925baa7fbd8106e3d3236c80855aeb7ffdd82c992e023b11e47fc5b3a297` L1588–1702 — **reviewed**：Hunk 1588: Read changed archive tests: new options use no security proof for disabled policies; injected index conflict asserts no canonical writes; all 8 retention masks assert record omission and carried live tickets.

### `crates/engine/speclink-core/src/quality/history.rs`

- `8c933f3d7c16746fab211022eeaa98236fbb0ba372b03a1c4ff0e2e96c9621ba` L1–328 — **reviewed**：逐段讀取 1–328：檢查 CheckKind 路徑、UTC 起始時間、ID/digest 驗證、原子提交 compare 前置、完整性與歷史讀取；測試覆蓋時區及舊路徑。

### `crates/engine/speclink-core/src/quality/history/archive.rs`

- `11b712234872e84a6a3bcf5ff7a7481d1c937db1d8ee61b3b12c572b6255aaa8` L1–314 — **reviewed**：Inspected lines 1–314: retention path selection, expected index/metadata digests, receipt report/decision validation, immutable content hashes, metadata stamping. Existing actor YAML formatting and canonical partial-write semantics predate the frozen change (confirmed lifecycle removal hunk). Hunk at new line 1.

### `crates/engine/speclink-core/src/quality/mod.rs`

- `4480dc417c1fca2b36025875a62e695fd668ebd4ef770efed701be86754fec8b` L1–5 — **reviewed**：quality 宣告 history module 對齊 public reexport。

### `crates/engine/speclink-core/src/quality/station.rs`

- `ba816f0451aa4c65d66548c48dc461317a7ac34dd9f1875ba8b0ab1161da2bbd` L247–253 — **reviewed**：Read every changed range plus add_round/stamp callers: recovery before writes, journal/read checks, acceptance proof on verify only, completed ticket/index/metadata CAS and first-round time parsing. Atomic backend paths checked in checks.rs. Hunk at new line 247.
- `2bf3cffc708584dbdd68402c7b112f6c8490e61acf9e6f89d831c01a214addf1` L296–306 — **reviewed**：Read every changed range plus add_round/stamp callers: recovery before writes, journal/read checks, acceptance proof on verify only, completed ticket/index/metadata CAS and first-round time parsing. Atomic backend paths checked in checks.rs. Hunk at new line 296.
- `665f8c18180cd6f746baa3e9071c47cb0edc0b36fe43f926ec850a5857a5380f` L328–334 — **reviewed**：Read every changed range plus add_round/stamp callers: recovery before writes, journal/read checks, acceptance proof on verify only, completed ticket/index/metadata CAS and first-round time parsing. Atomic backend paths checked in checks.rs. Hunk at new line 328.
- `a3ef42f336fa4befcbed3af867ddb5d4ea8d5d3b088e4e75d57af6a48b8d7b25` L343–349 — **reviewed**：Read every changed range plus add_round/stamp callers: recovery before writes, journal/read checks, acceptance proof on verify only, completed ticket/index/metadata CAS and first-round time parsing. Atomic backend paths checked in checks.rs. Hunk at new line 343.
- `2efd2e54a86286a4e6eff87319b38ec454626ec29aefee767433e317bca1583b` L365–376 — **reviewed**：Read every changed range plus add_round/stamp callers: recovery before writes, journal/read checks, acceptance proof on verify only, completed ticket/index/metadata CAS and first-round time parsing. Atomic backend paths checked in checks.rs. Hunk at new line 365.
- `36df8483b14fe432929c40c1ddd72aa95aea935086ba591dbab7debc0de82df1` L427–433 — **reviewed**：Read every changed range plus add_round/stamp callers: recovery before writes, journal/read checks, acceptance proof on verify only, completed ticket/index/metadata CAS and first-round time parsing. Atomic backend paths checked in checks.rs. Hunk at new line 427.
- `f67c123d073fc291193d2ace3126503fa0ac3661672f478e8e782da6dc41f89d` L447–464 — **reviewed**：Read every changed range plus add_round/stamp callers: recovery before writes, journal/read checks, acceptance proof on verify only, completed ticket/index/metadata CAS and first-round time parsing. Atomic backend paths checked in checks.rs. Hunk at new line 447.
- `e1214ec206b7e1df8c72b8001b8594b4a93cc63abebc9d156ba70a0b2d64e61a` L599–609 — **reviewed**：Read every changed range plus add_round/stamp callers: recovery before writes, journal/read checks, acceptance proof on verify only, completed ticket/index/metadata CAS and first-round time parsing. Atomic backend paths checked in checks.rs. Hunk at new line 599.
- `87d3968fb46b03eeebb7866c30c08da6039a10fafddc243c450abf1f1d156380` L611–620 — **reviewed**：Read every changed range plus add_round/stamp callers: recovery before writes, journal/read checks, acceptance proof on verify only, completed ticket/index/metadata CAS and first-round time parsing. Atomic backend paths checked in checks.rs. Hunk at new line 611.
- `ca7f9d256325052f8d62f176f6feb181a7cf138c37f0c0216388cf5b7f52dd59` L650–656 — **reviewed**：Read every changed range plus add_round/stamp callers: recovery before writes, journal/read checks, acceptance proof on verify only, completed ticket/index/metadata CAS and first-round time parsing. Atomic backend paths checked in checks.rs. Hunk at new line 650.
- `caea7df6e6a9f904a8d1f66ac9b0536297b239638731862ee8c1e7f052bac983` L661–667 — **reviewed**：Read every changed range plus add_round/stamp callers: recovery before writes, journal/read checks, acceptance proof on verify only, completed ticket/index/metadata CAS and first-round time parsing. Atomic backend paths checked in checks.rs. Hunk at new line 661.
- `7e57666c5ad53477b3399115affc0e540e1bf7466e0d263a548bb10b33e5ce02` L670–676 — **reviewed**：Read every changed range plus add_round/stamp callers: recovery before writes, journal/read checks, acceptance proof on verify only, completed ticket/index/metadata CAS and first-round time parsing. Atomic backend paths checked in checks.rs. Hunk at new line 670.
- `e12610789ce698e0b738cef9d7d6ae74ae26fc6b621f13cfe997d9b5838f478c` L696–739 — **reviewed**：Read every changed range plus add_round/stamp callers: recovery before writes, journal/read checks, acceptance proof on verify only, completed ticket/index/metadata CAS and first-round time parsing. Atomic backend paths checked in checks.rs. Hunk at new line 696.

### `crates/engine/speclink-core/src/quality/station/tests.rs`

- `73b5f718692761a9b7b15b02d51953aadbc4a788437c79b28ea096f5165110c1` L6–77 — **reviewed**：Read both test hunks: immutable completed review/verify records across discard/new rounds, redaction query, UTC first-round filename prefix, manual-only stamp remains valid. Hunk at new line 6.
- `19dbefb352052c6796527fcc76c3f93ce19411c8cac910e363afb42b7bc66a40` L582–589 — **reviewed**：Read both test hunks: immutable completed review/verify records across discard/new rounds, redaction query, UTC first-round filename prefix, manual-only stamp remains valid. Hunk at new line 582.

### `crates/engine/speclink-core/src/security/acceptance.rs`

- `267dfb08ff53ca349bd9d35c86fb1dbc969e9e9266c5df6189185b2cbf691a73` L1–646 — **reviewed**：逐段讀取 1–646：檢查 policy/report/source 綁定、required check 補證、風險決策摘要綁定及驗收矩陣測試。待追查 proof reasons 如何供應。

### `crates/engine/speclink-core/src/security/data.rs`

- `f0f2774750f920883c3cfd2fd2f5c4a293eb36998a76f88b5320f4b9f01abfa4` L31–37 — **reviewed**：Hunk 31: Read all data hunks: pinned guidance registry contributes allowed package entries and derives knowledge status without requiring ready for legacy data; metadata backward default missing; tests reject floating revisions, absent license/hash and modified guidance.
- `9e0b45aa14c533b3a13f33ff110a65211f18fe5f67c8d6f6a7b25ca8f9ea837e` L73–81 — **reviewed**：Hunk 73: Read all data hunks: pinned guidance registry contributes allowed package entries and derives knowledge status without requiring ready for legacy data; metadata backward default missing; tests reject floating revisions, absent license/hash and modified guidance.
- `488e2a3bae255e144d7f0fc004d11293e751447fd9e210b74ebf0c3c90e3509e` L94–100 — **reviewed**：Hunk 94: Read all data hunks: pinned guidance registry contributes allowed package entries and derives knowledge status without requiring ready for legacy data; metadata backward default missing; tests reject floating revisions, absent license/hash and modified guidance.
- `0fdfdf11e7d8c19e23ca0badae85757323ff113a513b01b2e18338902ff29d69` L107–114 — **reviewed**：Hunk 107: Read all data hunks: pinned guidance registry contributes allowed package entries and derives knowledge status without requiring ready for legacy data; metadata backward default missing; tests reject floating revisions, absent license/hash and modified guidance.
- `79a4e4405571eb3f6265ddfa54c375b4b483d5d8040ce904d47797984f619ca0` L138–144 — **reviewed**：Hunk 138: Read all data hunks: pinned guidance registry contributes allowed package entries and derives knowledge status without requiring ready for legacy data; metadata backward default missing; tests reject floating revisions, absent license/hash and modified guidance.
- `fae3631ef3ef03aa97a868691d27f9d90fd6bb7da01f77f98dda0eda49ce3ea4` L149–156 — **reviewed**：Hunk 149: Read all data hunks: pinned guidance registry contributes allowed package entries and derives knowledge status without requiring ready for legacy data; metadata backward default missing; tests reject floating revisions, absent license/hash and modified guidance.
- `d34daaf333f46820358b43d53d00361bf8be4f3573eff9af55ec80bebefabe7f` L244–250 — **reviewed**：Hunk 244: Read all data hunks: pinned guidance registry contributes allowed package entries and derives knowledge status without requiring ready for legacy data; metadata backward default missing; tests reject floating revisions, absent license/hash and modified guidance.
- `f472d8fbc41c478cdb42a2699d5bbbcaa7858b1fb7705593c6f04ca23ab838c2` L327–376 — **reviewed**：Hunk 327: Read all data hunks: pinned guidance registry contributes allowed package entries and derives knowledge status without requiring ready for legacy data; metadata backward default missing; tests reject floating revisions, absent license/hash and modified guidance.

### `crates/engine/speclink-core/src/security/input.rs`

- `6359bd4238a54267c418d3fe68601783a4096a6a207d08aff2e78616d0e20024` L81–94 — **reviewed**：Hunk 81: Read input hunks: tool-owned KB/check transaction files and change checks excluded from evidence to prevent self-invalidating report hashes; Deep is separate from Code/Api/Spec, raw bounded byte method added at ProjectInput boundary.
- `fe53479250f5237f2838c0ef8e77b3aec68ec270ca7c331873c6299b5f128fd9` L104–121 — **reviewed**：Hunk 104: Read input hunks: tool-owned KB/check transaction files and change checks excluded from evidence to prevent self-invalidating report hashes; Deep is separate from Code/Api/Spec, raw bounded byte method added at ProjectInput boundary.

### `crates/engine/speclink-core/src/security/knowledge.rs`

- `4fa5397bf60510eb60407bdddc291725d92629d3cc6609d8cde4bcff1c5c2f68` L1–205 — **reviewed**：Inspected lines 1–205: deterministic generation-independent knowledge identity, guidance relocation, CWE map filtering, generation allowlist, pinned registry URLs/licenses/digests and missing-versus-ready logic. No concrete introduced defect identified. Hunk at new line 1.

### `crates/engine/speclink-core/src/security/mod.rs`

- `ce0e00b8ae0d55e291316e76603ec81afc43fa0e243844b492a52aac0ae98d39` L1–10 — **reviewed**：security module 暴露 acceptance/knowledge。
- `614cba0af7ca0b663bb55c4920ee385fe6f8f905fe45976bd2602b4d9719d87c` L21–42 — **reviewed**：require_acceptance 先判主開關與 required、proof change/key/index 綁定，再檢查 passed+verified freshness；一般 accept 不介入。

### `crates/engine/speclink-core/src/security/policy.rs`

- `acda2ed809542ac8d10d4014431a96fc97529c9f16412fac44e95c9e888f531d` L122–128 — **reviewed**：Hunk 122: Read all policy hunks: new acceptance flag defaults false and accepts YAML snake_case; review digest clears only gate flag; shared leaf patch validation preserves unrelated mapping and validates reconstructed WorkflowConfig.
- `1e3ed669c3056e43bbd6e545ac7eae618a6a89f9d9806388f53cde3c22dec791` L134–141 — **reviewed**：Hunk 134: Read all policy hunks: new acceptance flag defaults false and accepts YAML snake_case; review digest clears only gate flag; shared leaf patch validation preserves unrelated mapping and validates reconstructed WorkflowConfig.
- `92da4248f62e2b4ea5b6c9a6b7ff144edb5f78c28a8998c8ef0ac5f405c343b9` L150–156 — **reviewed**：Hunk 150: Read all policy hunks: new acceptance flag defaults false and accepts YAML snake_case; review digest clears only gate flag; shared leaf patch validation preserves unrelated mapping and validates reconstructed WorkflowConfig.
- `b0bba87d376e945334e6e7b11bf6a54b439611153dd1ab8c7ff67ebf82bfffa4` L163–174 — **reviewed**：Hunk 163: Read all policy hunks: new acceptance flag defaults false and accepts YAML snake_case; review digest clears only gate flag; shared leaf patch validation preserves unrelated mapping and validates reconstructed WorkflowConfig.
- `400d36df9cb18dd58390b378bf6646b43418a3b2345a02619d5f20f9944e90fe` L263–294 — **reviewed**：Hunk 263: Read all policy hunks: new acceptance flag defaults false and accepts YAML snake_case; review digest clears only gate flag; shared leaf patch validation preserves unrelated mapping and validates reconstructed WorkflowConfig.
- `2882ba45b2bab416c598db6d9671a476fdf37ac2c1f8dd4b1b988fc252796ec7` L303–309 — **reviewed**：Hunk 303: Read all policy hunks: new acceptance flag defaults false and accepts YAML snake_case; review digest clears only gate flag; shared leaf patch validation preserves unrelated mapping and validates reconstructed WorkflowConfig.
- `ceb65cad1d4ee7b04191f80028dbbfc307c945209599c6921fd43257234a3dbd` L317–329 — **reviewed**：Hunk 317: Read all policy hunks: new acceptance flag defaults false and accepts YAML snake_case; review digest clears only gate flag; shared leaf patch validation preserves unrelated mapping and validates reconstructed WorkflowConfig.
- `8fd61f8767788cd5ee61fda87d15ddd8ac62b5d4d76ac1111790cd6e1f319808` L335–342 — **reviewed**：Hunk 335: Read all policy hunks: new acceptance flag defaults false and accepts YAML snake_case; review digest clears only gate flag; shared leaf patch validation preserves unrelated mapping and validates reconstructed WorkflowConfig.
- `c7d2c4ad2a10c08dd4b415b1205306d28bf30ceb08f382b269231a288fecf8f6` L346–356 — **reviewed**：Hunk 346: Read all policy hunks: new acceptance flag defaults false and accepts YAML snake_case; review digest clears only gate flag; shared leaf patch validation preserves unrelated mapping and validates reconstructed WorkflowConfig.
- `7a28ed701c5183f5751fd8371a6ec3b6fdb1715c8bbcbc00359ececa78ed9743` L360–366 — **reviewed**：Hunk 360: Read all policy hunks: new acceptance flag defaults false and accepts YAML snake_case; review digest clears only gate flag; shared leaf patch validation preserves unrelated mapping and validates reconstructed WorkflowConfig.

### `crates/engine/speclink-core/src/security/prepared.rs`

- `e0a053de81dd3af62b56736600a4093851f5b86c08ea201b3018b218e3176806` L66–73 — **reviewed**：Hunk 66: Read prepared snapshot hunks plus header/body validation: optional canonical proposal key backward defaults None and is digest-validated when present; checkout/cache binding remains separately validated.
- `692636695a877f89cef84cf244f8a5d332daf49fa9bfa133f536d44f256613e6` L114–120 — **reviewed**：Hunk 114: Read prepared snapshot hunks plus header/body validation: optional canonical proposal key backward defaults None and is digest-validated when present; checkout/cache binding remains separately validated.

### `crates/engine/speclink-core/src/security/report.rs`

- `f0d32f430ceee8c9ccff85cc49f0666c343a2e22588ed5f73e3c30e91fa2f5de` L8–50 — **reviewed**：redact_text 的鍵值、PEM、JWT 遮蔽與重複處理。
- `41d55071e943643a9fbe42480ee1f1aaad9811bcadfea35e2bc37192ba826bd6` L161–181 — **reviewed**：新增唯讀 sources/policy/checks/coverage 存取器，無改變擁有權。
- `11d6c63c2e74c417776df8d7b11511ec2ae98ca8123043ffcad623f5f5c42ed9` L440–446 — **reviewed**：projectName 新增 prose 遮蔽，保留 changeName 驗證。
- `2705b21e52c7482ea6978ad772719b656426307f6797ec45ee6876be43da2e39` L475–483 — **reviewed**：finding 的 title/description/remediation 走遮蔽，evidence 延續規則遮蔽。
- `a32b20be49860b3bb82cff0cc4afef2b7bc17b3e7f2ffc47527d00b901889ead` L520–528 — **reviewed**：新增 policy_digest 存取器供新鮮度比較。
- `4dbd881257d7fc8e6c284e4f307536553d83b0315d067429bc310ebbbf8b42a1` L789–1159 — **reviewed**：逐段讀完 789–1159 測試：CAS、深度報告必填欄位與 redaction、多種來源摘要、風險接受理由驗證。

### `crates/engine/speclink-core/src/security/report/deep.rs`

- `88e9df944f6775e51d3d0f42f7aa8bec47cbe450273c93afc59c9450aa176a2b` L1–1054 — **reviewed**：Inspected lines 1–1054 in bounded ranges: input/context digests, authorized targets/evidence/control references, completeness/count recomputation, redaction and markdown binding, immutable report writes, pair validation, and risk-decision CAS expectations. No concrete introduced correctness defect identified in these paths. No commands/tests run. Narrow follow-up inspected rules.rs redact_evidence at 531–570: final snippet is capped at 400 chars after redaction, so redaction expansion does not create a persisted over-limit snippet.

### `crates/engine/speclink-core/src/store.rs`

- `2b80ae9e853e6c0ad1861cf38329b32ef1fcaf79aaf42dede60231ae9cc46a40` L94–104 — **reviewed**：Read both Store trait hunks: default unsupported transaction fails loudly, archive plan default rejects nontrivial check selection before canonical/archive writes. Hunk at new line 94.
- `3f524a066e1e1a343cf291ee2fa58d1d1d94fef0befce1120173a691830be533` L142–155 — **reviewed**：Read both Store trait hunks: default unsupported transaction fails loudly, archive plan default rejects nontrivial check selection before canonical/archive writes. Hunk at new line 142.

### `crates/engine/speclink-core/src/teststore.rs`

- `7145c546728c29ed797bfa9151ec52d4b466b4f1105e2a607a4b28459383d094` L56–64 — **reviewed**：Read all TestStore hunks with method context: one-shot hooks, expected digests before mutation, canonical archive plan order and retained record/index/receipt metadata copies match production contracts. Hunk at new line 56.
- `52f39aa10b09c2191e7180536a292c145e1fb5fc20c33852368705ef06f0027f` L167–176 — **reviewed**：Read all TestStore hunks with method context: one-shot hooks, expected digests before mutation, canonical archive plan order and retained record/index/receipt metadata copies match production contracts. Hunk at new line 167.
- `13551b9d30f76ae66bce4144decde66a2b2292cd7f19977bcf44ad20222328ba` L190–208 — **reviewed**：Read all TestStore hunks with method context: one-shot hooks, expected digests before mutation, canonical archive plan order and retained record/index/receipt metadata copies match production contracts. Hunk at new line 190.
- `ad68f20992bd13eac4075057732748b2c72c8cc49914578a4cb1291e1250e65b` L276–291 — **reviewed**：Read all TestStore hunks with method context: one-shot hooks, expected digests before mutation, canonical archive plan order and retained record/index/receipt metadata copies match production contracts. Hunk at new line 276.

### `crates/engine/speclink-core/src/util.rs`

- `30da8fff163b7ad8589e151215fa5a82a76bf3c5b05bba2d54772e51064e56b5` L3–23 — **reviewed**：macOS 僅允許 /var /tmp /etc 對 /private 的 OS alias；其餘 symlink 不解析。
- `7b0613933607f820e34e0e34091de1d643b90acb6afbead6c94f39c7e6b5552a` L324–339 — **reviewed**：測試三個 OS alias 與使用者 link 保持原路徑。

### `crates/engine/speclink-core/src/workspace/config.rs`

- `7b4b47630f37f1db938b0ddc5f64f8da2f8d500c59b2b76cead8ef661d84cb60` L88–94 — **reviewed**：文件註解新增 Antigravity。
- `753d9d7955b154d87384d0218035786736c3b1e6c8169b2d2fbbdea91a05a89b` L155–161 — **reviewed**：descriptor 名稱衝突提供正確遷移說明。
- `50db7987df4adc86e18bed088210cae49f487134d802192115007fc358525775` L359–386 — **reviewed**：保留政策 serde(default)，三類預設 true。
- `4518be99c246e846c25e68d392707538ae59ca32508a93f5178f1f13ad23efc2` L649–658 — **reviewed**：accessor 缺欄位回預設。

### `crates/engine/speclink-core/src/workspace/config/tests.rs`

- `a1ba664aaca137e516a4b993f8f5d13a9cf2303e2c7f0396cac78082661c41a2` L12–61 — **reviewed**：Hunk 12: Read config tests: gate defaults/disabled preservation, wrong booleans rejected, check-record default retention, CRLF/comment/unrelated content preserved and exact 64KiB patch bound exercised.

### `crates/engine/speclink-core/src/workspace/init.rs`

- `51025ac9c10ddc7f34a67b525d1bbfe2db2e169b4893dd311d11443b3dbd2880` L9–15 — **reviewed**：Read all four init hunks: embedded asset version v1.50.5, stable Antigravity target order and no instruction-file cleanup for new target; behavior delegated to existing core parser. Hunk at new line 9.
- `749d2f5964549370f30428adf6e142768c6e05294af59fd324c74eb1d9202f3a` L304–310 — **reviewed**：Read all four init hunks: embedded asset version v1.50.5, stable Antigravity target order and no instruction-file cleanup for new target; behavior delegated to existing core parser. Hunk at new line 304.
- `41c9874d449447efd58fff8d8e5ea955955babee90e3bb0ed0c5f23e5ae2d2d9` L411–417 — **reviewed**：Read all four init hunks: embedded asset version v1.50.5, stable Antigravity target order and no instruction-file cleanup for new target; behavior delegated to existing core parser. Hunk at new line 411.
- `aa03b7bdb7f01e348f549336288a571236ee007f8ee605f7154ad1279afe330e` L1079–1090 — **reviewed**：Read all four init hunks: embedded asset version v1.50.5, stable Antigravity target order and no instruction-file cleanup for new target; behavior delegated to existing core parser. Hunk at new line 1079.

### `crates/engine/speclink-core/src/workspace/init/tests.rs`

- `b6b2cb875b5512896e451e9598cdd5f6f5eb511b69d675709550c8296ed2b655` L125–131 — **reviewed**：Antigravity 不產生指令檔的 match 分支。
- `c6f8bd32ca27bc975a9bf85dba9f848a273830d8b5ad311320e2e2ea220d12d4` L1489–1495 — **reviewed**：未知工具訊息包含四個支援工具。
- `36792460c9f5ee03fb7adb0e031888999d2ce02d73348560c868ca369a31ef07` L1592–1598 — **reviewed**：只選 codex 的 deselection 清單納入 Antigravity。
- `df4538717a9a6b994cfe0283690d22808350ad94bf4b60daafd827d6aefb49b1` L1620–1626 — **reviewed**：自訂工具測試 deselection 清單同步。
- `705fe76954e25da79ec2ab93b7c2efa8756d49ca5f42fbeabcb00af929645e10` L1981–2046 — **reviewed**：Antigravity 共存/取消選取只刪 owned skills、descriptor 衝突、symlink 不清除、remote init 無本機 spec tree 測試。

### `crates/engine/speclink-core/src/workspace/skills.rs`

- `078029da1da12745aa15679a6916b0092d79da04f12a19984545afed2e885240` L3–9 — **reviewed**：Hunk 3: Read all skills.rs hunks: Antigravity is included in parser/name/directory/slash matches and ALL; security-review body included/registered read-only; generated baseline tests iterate the same ALL.
- `bf4beeec8142714de3faf900dc33307032f5e19982375538bfb378dd22218cdd` L13–25 — **reviewed**：Hunk 13: Read all skills.rs hunks: Antigravity is included in parser/name/directory/slash matches and ALL; security-review body included/registered read-only; generated baseline tests iterate the same ALL.
- `6f0a0131480a2433952438419c01ce98ebbecc986254216c2106e482cb1f1665` L27–35 — **reviewed**：Hunk 27: Read all skills.rs hunks: Antigravity is included in parser/name/directory/slash matches and ALL; security-review body included/registered read-only; generated baseline tests iterate the same ALL.
- `7421da06b9bdbddf7d28aa016a6c7914f14fb05ea1d123af501109469b1e18cb` L39–45 — **reviewed**：Hunk 39: Read all skills.rs hunks: Antigravity is included in parser/name/directory/slash matches and ALL; security-review body included/registered read-only; generated baseline tests iterate the same ALL.
- `c9641b38b875b974e058f3c92692e9d5b6b2c999bf8c8681b162177f8de6cb44` L48–54 — **reviewed**：Hunk 48: Read all skills.rs hunks: Antigravity is included in parser/name/directory/slash matches and ALL; security-review body included/registered read-only; generated baseline tests iterate the same ALL.
- `c6a5ca2b47f23cbc4144d152d76391cb9b39025a123a8a8ce032061c588251ff` L57–76 — **reviewed**：Hunk 57: Read all skills.rs hunks: Antigravity is included in parser/name/directory/slash matches and ALL; security-review body included/registered read-only; generated baseline tests iterate the same ALL.
- `54defb7136a2ec4eb206893e7fbc601b994ca044f8da57d406c03f93f48fd9e2` L108–114 — **reviewed**：Hunk 108: Read all skills.rs hunks: Antigravity is included in parser/name/directory/slash matches and ALL; security-review body included/registered read-only; generated baseline tests iterate the same ALL.
- `ec325eff90ecd31d2836fab9c9afe865ab010df14ab0e1db8c99cfc0841c2006` L158–164 — **reviewed**：Hunk 158: Read all skills.rs hunks: Antigravity is included in parser/name/directory/slash matches and ALL; security-review body included/registered read-only; generated baseline tests iterate the same ALL.
- `1868a6d07eb489b478cd6b8e9b17909591fac1f72d9db12b43b8270d5a916f2b` L418–424 — **reviewed**：Hunk 418: Read all skills.rs hunks: Antigravity is included in parser/name/directory/slash matches and ALL; security-review body included/registered read-only; generated baseline tests iterate the same ALL.
- `53f04c16e80508dfa47f1d25841c5b1d9c133d9346ec205c6d99cb63113def02` L516–522 — **reviewed**：Hunk 516: Read all skills.rs hunks: Antigravity is included in parser/name/directory/slash matches and ALL; security-review body included/registered read-only; generated baseline tests iterate the same ALL.
- `1141d4d89b4e03ddd5df23b0e660f208bb2a357fca33a402a07fca593146cdee` L535–541 — **reviewed**：Hunk 535: Read all skills.rs hunks: Antigravity is included in parser/name/directory/slash matches and ALL; security-review body included/registered read-only; generated baseline tests iterate the same ALL.

### `crates/engine/speclink-core/tests/golden/assets.lock`

- `0f3e599d4e1b9fa5bd92ea5c0062bb4fc40e7c6e8d5f8c1b2edee1813a1ac90c` L1–2 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md`

- `3c224c62a8ebcf7ca75927a24427197e6bfc0d9616ee2c2f9689490924d3223a` L1–90 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `015daa2969cdfd6e266f98e0d735eff33ed38ebc65443594c71bc11068d44365` L96–102 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `ed4a4b0f4660657f6e7135c0877ff2db3e82233d8b239a35e285ea3afb121f27` L189–195 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `dd50c72aef3d2ecf9de355e3a9e300428567af14b4b9e118ba99f6e47fed5ac8` L532–538 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `a0e8c766ecc81c1b72989d8656d788396548b533cbe46aabfa63bfacba172b03` L1155–1161 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `030e639abb8532ee7bb73dd62fafed502c584defb5453230d7e1c15feb5bc7cc` L1163–1174 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `492d775e51d8a4ba087c6eacbc71e033d87d154734ea4e11610116876b3a0402` L1188–1194 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `959192e7b4eab2c873c3dbc8677b953e713b51a9029b023dcbbe3112ac7ebeb3` L1505–1511 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `afb1cdbcea9776c35948754b579150ec3155578d4efa07e45100f16262828beb` L1762–1768 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `7e7ba0c0eff529a40aa96ff954be73a61f3a972f92aea0dc4f46d6d8cd6c5506` L1891–1897 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `2fd2a3831bb23422eb5f46f351cfff4d5d36a0b49a077d7284e7e3c940fe7d6c` L2203–2209 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `2dd139fe2d99fc1d45d86eeaa797ae6d8eec569f8fb155f80fac6df8a01a36b3` L2446–2455 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `01b4864733ead452fed5c0b220aa3737fe2d5c27008563d4cc811b1e48278f00` L2461–2467 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `e687d9786b0923bccfc36a30eb79ed1f193e23bba6f24b57cd9dce57b1048eca` L2950–2956 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `b619496ef71596fad64b8417c78583dfa022fe9fa3184099a8ed5b3f337ef887` L3093–3099 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `8b36520534e7fe494791a93e7dd83e54833ad2f6241e426d4e9a1d738e950581` L3290–3296 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `9e207f05a53276ea333562836042cbbdabe1633e4e06fdc85eacc67b1c7f4d22` L3604–3610 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `be782f44a12a69d08d1dfbe484ac15698fc71c2115b68503e4c453fe0192b5b5` L3819–3825 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `b161fb4c8359db333c50f03df91fec8d77ed1f1dad9d5f6ea860674d6aae65eb` L3983–3989 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `493632e2b7bb88a6a0757ecec1bf3634571525b44fe2502a921b2b7541ccb2b2` L4276–4283 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `8b95e503059ef8dedf18ca69fe0ffbf136649d7ac5b25d65dda8f8a4ca2f7f6e` L4292–4298 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `3054699c7a1a2282645f63f9ee4bc8ff27c0a7cf9b2e991a13733263b20bc5e7` L4416–4422 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `65f666268b000acc6bd86ac9d9b9efd202a8255ad370b9b8303e07c32554fb93` L4677–4686 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `1b9da426573bdd13cacc89308b6a039acb0eb66db7b8aee8657bfec13bc1c83f` L4692–4698 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `5b0eb6baf628b10440a671351ab6223ed4399db7043f7e8f53eee8efa5534132` L4780–4786 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `c64f776a16623bf6ea8c3c372867a61ef5a0b36f8bc8d34fd1f83f4426abc099` L4829–4835 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `506dacbbe90ab0c2a3506c9c1842937e9c5737e09b9ab149f5fb5531d355c444` L4875–4881 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `ed5dc4e596e530a29d420eec79ae76592bee752d76d4e87cc8bc98502714d656` L5100–5116 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `bf034b7d386275978e3c638b34333503286169f5df0b12124520251e07f49a95` L5121–5127 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `crates/engine/speclink-core/tests/golden/claude.snapshot.md`

- `6455497d67294abfb940d46fc8e95e23152572a1ff900f499278208ee4e42bec` L1–90 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `adb541ec96b93e314e6c4e474e667e732bd17a24a4607940e81d7c57534568c2` L96–102 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `234fea81df043ac4cf97fad1897cfa030d0ee0ce3ce1c6e38a251e0d7aa19685` L189–195 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `bbfa5a4aa6e845674bb9160e98f5228cf1ff55d6068f3ffac7cd7ac2d3c146ab` L532–538 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `ae324bb04a0a213e16de6a3b5f972a06906803f9937c351894ec4b8ce169d56f` L540–551 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `c00ddc9147bf76c099f87c3e7467d45105aec558304e621b72a0fd605eb36015` L565–571 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `c6b31b43566e4a659e6d6cddc5dc301030c09cf6d21d978dc2826645b4f9f989` L882–888 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `1ac8ec570eaa41cbe42d50cfa2e25f10df51620652cf07efb1ea533fcdc58f65` L1139–1145 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `bd439ec59ccc6dbbb6ea6e315d9b1d3db7c72c18824a407ccb2e83de2dd4df77` L1268–1274 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `34ac7ff919758dea8930406db5d6fad8c8921453555724ef330e77c1f822d0d1` L1580–1586 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `60c229004738d1a2e262b5a8d54aac8175ba98193f558054c3cb414ce2c64540` L1823–1832 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `4138d4e03b69cb0c2bb806e1fe1eba2e45498d10c9c6f059328cb74a2cc24d37` L1838–1844 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `fd4b3907ca229d193ee101c48d3cd201d2a343532d15229892ec314f422e931d` L2327–2333 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `cba0ede7014c86ad6ddd44e06ba786311f075fd7f803aa6b59756ca8a78f7761` L2470–2476 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `50e930db8c187fd3e2b3003717489b5e0208d6d49c651b106a6e587afd7ec067` L2667–2673 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `c1867188e40ad85265c5f6482b2b3c1faae041c7f9219a2db0980359c308fb52` L2981–2987 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `6087a5c566ca4ff46324bcb8dafeea565832e61bb739525d3c1d7dc30e5d81ed` L3196–3202 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `7afd860971c8df373f7cb18fa04caa911aa4a80c54d24fdf16a1d39e1e6c3be2` L3360–3366 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `758c3fb4f41c42b07e0116a48ebacbf156f7f05c9f74c3d2a8dc26a75517ff62` L3653–3660 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `a0e11173514d5b57b8e9eb854ee33a4f19a9d2c093e60d3a3c7460d4adff5197` L3669–3675 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `c4080857c0ed27faf6780679fb8b2f9b1de7f0f7618ac628dd3f49991d619526` L3793–3799 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `ebde3896d6e0ca0232d0ae483f86a24aebc02bb7bbb866431b8fe1042ec82c20` L4054–4063 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `b95d1648d5ccf1a600e7a3e178a7fdad743e97339ab5355bf780bcb15549ef6f` L4069–4075 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `250cac13c49df14f04d568b1f7a0fc0366cce71e58d2ee3432b653cb3472bfee` L4157–4163 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `a3b1a74861ef160a938ba2c5234d30c4eb62aca74f6217dd506aa8799841be8c` L4206–4212 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `12f4a71b6eccc3ebd6dd4fd7e73a2ea684e90e07df05ea010e85a9d0f48269d6` L4252–4258 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `f3f2e309cdcc1f48223e025a99213cb38a55c3940590535e9435594624ac28a3` L4477–4490 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `crates/engine/speclink-core/tests/golden/codex.snapshot.md`

- `a2c744a892b98178cf1761883cc90427e08eff7e89dfed7ebd9f808557c542ba` L1–89 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `eeb71188758c7b585fbf0734e49a857fd2ab27012bd9aaae3dcff8b4b22b03bf` L92–98 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `650a8852a1659e8791fcb06d2c8851d7b40e4ed4edd0aa6f8c3932793d888fdb` L435–441 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `8a33a6abb56fc91c78fad91d0d9e566cb8f9a66d28661ca5e9e92ba2d83af5a6` L443–454 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `0efefce0641ac0c3e217b2570d4c0913c651bd3f060ee8a01f2f235dc371c4b2` L468–474 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `f1f0989b50092b9c6c9786a9ace8e762599c2d374e913c3d602d02785eda6ce0` L784–790 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `35da12029ce04f98663eb789821fec3717b19e1dc5000f736680bcd6c53b69f4` L1041–1047 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `b44e15cf634223f144ecc33b0141deab9390406159e957e0c5c2d00f434c80bf` L1170–1176 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `3a89c5f39dcb52f8d52ac6a45622418be42d0df2a532bbfb842f5fd6be2ecbc4` L1481–1487 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `04b1367be463b69e20108ba657b6a119451ac8ad4c5c0e06a061fdcedc2373a4` L1724–1733 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `6038f8db93752748b4bcba45e38e647d020ad0158194728e5006dd6b67e91d76` L1738–1744 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `8a23dafe350126d3e9c9e35ca20e363c3f8c54394144cf4452f4db44c51c0a51` L2224–2230 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `85fbfb164db0a6c160a3ee7577e4629588f434d71086caba1485955d2efd0575` L2358–2364 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `42d8733be81b537a74e117698112221fca3bcad57a451b3b430755f61142bbe6` L2555–2561 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `832ea0dd8431223039872ee27488b096b4f06e56fcbb51621b04021515dc2bd3` L2869–2875 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `57a86faca8bf2b2d24473f5207b0be4fd9f42359d83f3cef93ac7b5485c39cf2` L3084–3090 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `80736520cad0e48d8cf05f745d688db40cea56ef4fd2a020f031a70d7ba8fc45` L3248–3254 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `f46b78a46ef12a42fc7ef3e8b36789a57d2f3d70de2b520ae40cb9153975f4d0` L3541–3548 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `5d34c732e79c78c1245c2391ded4e1b5e42a7bb1e297fc88a3a32dad9eb19fa8` L3557–3563 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `87abe94c27b992ef20fd54296db4e149d0696354a1980a96ec87dc1bc41f954a` L3681–3687 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `24451b8354e687dd41b82c3038f88cda1a65c8ee32163445ea35dd4e8cd97b88` L3942–3951 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `9ff5c3cbbb7a64a299e7dbd69d0f520d3045788fb5156d9e53c29f0b83c38fad` L3954–3960 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `6ebc4d1ed4313dc1299c7de5bed882e305ec4120c19faa5acc46593a2c64cd20` L4042–4048 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `57121ac55e0c1617f3a8cc0296703cb5f3236c6505919b6c481eb4af30579309` L4091–4097 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `ce597a12fc2711de6fb5bdfa5cfa9c452f03b49433850ba7a0640ce5a595c9e3` L4137–4143 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `c926858bde0a31c94a5fc4348d66f1ab0b0685905d7a9fa5895fe7fe918ad39a` L4362–4375 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `crates/engine/speclink-core/tests/golden/copilot.snapshot.md`

- `891a90a2de9f393daea905bfd21d2fca4bb2719f22c90919685159d79167876c` L1–89 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `4468f32c2e652d115d3c96452abdb9ec40bd236260b9ee4cbf067bdd75dada63` L92–98 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `d145870371c49da24fd63e25bc30837d3df64b32ba1b88db4b79e63e6ad18796` L435–441 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `0857b254e36498042514aa62198cb9f7a5cb6f822efca7aebae2c6907f890dd9` L443–454 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `976fc7fe4527feccf05303b20478bd03ee39ecaf373af830e702ac384112bd97` L468–474 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `1de3a93bff4afad087775f5fb466095e617402aab6ce5ee998a24c5de657ba2a` L784–790 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `2d50b58ce0faabe524e1e83d1f4a2a32ed05c89b87e3639e3c9c2d5c9d2e168b` L1041–1047 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `6b03b609c6e8811aef2fe8ec5057772e5bbe204256798ec7cc04496d46681c11` L1170–1176 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `6147a2dcc70b858be871af72035fed053ad682bff862674884e82f9858587094` L1481–1487 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `35dbb23e78103e5fed8fa01ba9d5c350482c52f1de474f7d7d0c701bda9bad1f` L1724–1733 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `62a9509256719c14d52d4b3876d381dd763ece58b901a48704eb991aa9828453` L1738–1744 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `10568c743638e2734d31aed63cc7505825cdc433f8b9bd612565eb83a879d9c8` L2224–2230 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `12ba1ef4222fdccf69e5a82e5bafe10747b09bd2ad0e4018c14860b3b5662d6b` L2358–2364 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `b24c77d1a1f53608fcc2da8eae9258eac9924313678a77a5c42e680593b5d949` L2555–2561 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `c7042d9c824835eb56fb4f2f2e9d0b23b1ea89cecd37ecf020dfb1c995f79d2e` L2869–2875 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `c26144fa1ebfcb95f0c95cb8fc82f808d72f8c1659a4483fa7ad106fd3425ab3` L3084–3090 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `db58ac2092a46e1d50425b779cbc5527f0db6a602110b56baea4b440644568b9` L3248–3254 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `870c3cf3cf025bb643fa3ca63336933c6a835d9d2659d42334a2bc3773ddd772` L3541–3548 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `c7c30333ebe772d154ab8a8b5a02385df98b59cbb6a0b4820eb1842524173efe` L3557–3563 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `60c0ad3f5a92dec4dba667a1a2f468e7b26f2f3a505aa27e8414e7255c0ae2b8` L3681–3687 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `ea04126fbcd4a5f48e374c052d44a0602d3637f6a1f977424e7e9e6e3bdd140e` L3942–3951 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `b817e314ea01416f2e86b6630a1599d56f21fcef387ed43f55752558b5c337ae` L3954–3960 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `466cf0d78cfef89436ef4718ba913aeeb88f8724f87672eb9614d8653da3098a` L4042–4048 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `6bc0f5238e1cc5fbfa011bb98f36cac82b9047387d29fdc2f746a132db919631` L4091–4097 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `95174f851fd46d6674c3328050390f018096a4a48d7a180f1ad62aa5877a60a5` L4137–4143 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `9c6750d55174e4fe6a1043afa5379a439bb60cd08c0daf9c10d2cee0efbf5152` L4362–4375 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md`

- `4b72d1580fec8e9047b5a4baa813a8e2d976e570d0314abd1370c695d14c31b3` L1–95 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `52cf76621379500ec36701a00b24d1d53e99111a92b60ab4ee77605b535f1995` L98–104 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `eba9a2190503ffd5bae2879894465c3f3129f7b2c2dba14249687cfef1a594ff` L447–453 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `2c58245014aed8b9a6919851f6de414220caf64cffed629f260be4efd9369bae` L461–472 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `81b5b81adfe311c6e3e7b34a2df1569693786ac7ad98a45bea3fc699488cf6e0` L486–492 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `5738f18bcb83c02000780acacc2af7541590ce3ed1ccc3c39c9fbb6415f7988b` L802–808 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `9516849199368f020c173356e89db2a797d3985a8841d1b0954777a6fa61d03c` L1065–1071 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `4869903b37c4b3aed98bfeca1b091a3462901c03fd078759ab0a2dc40ed65b9f` L1200–1206 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `2aa5c78a89cc31fde7f34b66a9c380e6fc43292363e40ce28d7a2b191b571162` L1517–1523 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `7118334e7e78ce997cef284f6c33edaeeb0a7c6f0935ae98deb57bb1c6948e3a` L1766–1775 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `2353723df38cd0ef54947d4593325180732007186ad409403de4b90d1f491203` L1780–1786 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `f9ea2e0283513e3c75fd68d84d50dc9b02ce0c9d1c37733c506e279803ce2144` L2272–2278 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `92ba31cd3d65805301f2e8c8781d93d35f35cbcd56dffa3890057ffe48a50310` L2412–2418 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `197299913d20c3af49dd65e02739b7a0a893bc92f208f1549f3ff64abe9b9076` L2615–2621 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `921fe065ebd55fc5eadae65eb178be7cdb712726dcc3dc8664afb548fe9e3e28` L2934–2940 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `073fd4ac4a7bae8d41ca27d502b52b03fe3480944dc26f12e8a502414f29e827` L3155–3161 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `aed9459504a53e22a05429f3900fb2fab021088fb871c4d3d0905da5b0cdfd9b` L3324–3330 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `4b9f9988ffc14c667f7acfa593ba06bb3912783a5d6972395b57797290f660b9` L3616–3623 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `69bc85bedf7cd67a8dd17b38e1bd9d7652b8a975f2e21bd1bac74d5d0cef1f4f` L3632–3638 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `95069eb06281104db0af73ad66bc59a3b7f24bfd8452dbe18a892bcb4a4c4c45` L3762–3768 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `b0e54707e21e34b07eff5df48c4105c4df76e44fa6bfe859e0b1fe37e7f59522` L4029–4038 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `e4ebf2e798df0faa5b4286e35f1a3a1c8f26bebc6db1db06b3f2982cd43340dc` L4041–4047 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `7e1c61b392728ea12eb9313f64d8f52156e755689e2fae53589bdd34a3d2759d` L4135–4141 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `ca5165676f7e10ef6081f6d93eee546ba59ae045997db0a07509fdc1fb4f1ded` L4190–4196 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `c19a1fdfe2e23a5ea7f83c68853474df0fcafb53e57d31c008c6ce8d4e022a76` L4236–4242 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `dedc53cbbb4fa7f295c2386601d37beede935d6e14675a8a86225baf62d21900` L4461–4474 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md`

- `30cd4e3cb111a0eb111b74bf2201996c6f8c6aeb4b739ed8b4a91ca1565fed9b` L1–95 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `6b544e01702ae214b0a5c541fbb919cb852785e20bb94f95338746878781f3e0` L98–104 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `6c09e60943d09acc9809ba60a4a1d526a4ff26e63aa42d9e92dd2447c82bf00a` L447–453 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `e9ae16fd6880562deb8285c8fb355c486fdb1f8b414bcc04071419e027dcd5c6` L461–472 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `c86d5dc84515f87a94cafabe0434bf20697da02f85a9b75702b77ca9b41848ce` L486–492 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `7928f3143bb95b150afb2de4269406d7a0fe25bfc430c526e633401d00a5dea4` L802–808 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `7f0a4a9c720e034d47da2254975841a41d5bf493ce36031b874f2cdb1b9b898a` L1065–1071 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `6a24cd402053402cddfef2243411347a9663a19d024be3f1a3b01301dbb066da` L1200–1206 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `e9216ae7b7f625b125330bc7adc81d515f17ba7a4fc2e062a531dea625f60827` L1517–1523 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `0720753c6f6a31f4c29a570bccd87a17a834f42710b34f026a41b0c7f5885292` L1766–1775 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `4191a071d68cbe38a6003c15efa1999b0811beebcca694fad12a979e26d7734d` L1780–1786 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `f9345eea7f0da47ac3e810e19f1dc044276042f70ef7f33e4ea92a0e2ecbdd8b` L2272–2278 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `98b9173469d94a6b9400b81113c5c206b3dd59204e0cfa32085129b05b575201` L2412–2418 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `ff8b59f924058d1a1748d609233ee020df3c6bc57610cb9d0d67bb320c068639` L2615–2621 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `e4a04adfe1e7a101781fce9827336587cd7942c1e944986094e8d8c8239ceff2` L2934–2940 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `2e16969baf7662e66d935fe948569a57d0912674007c6dfb56d4e18576e66d2a` L3155–3161 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `05bdaa1171858825961a60c9a85e877b9bb324c962818b73826ad79d37dcf2f3` L3324–3330 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `8c78af374b331d6190d717f34fdc89e74111648ab8e9715ec2fc1dbceecd7715` L3616–3623 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `7b6347bb36a2a20f3de28e75160820b95e72f95b8d9ce69769732f4097c2d1c4` L3632–3638 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `78a4c17edaa075ba274edf2e7491d528291b3c2c1b747a2a1761f36c99a07aa2` L3762–3768 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `4de37119bcf3324ead5dc5695d260b27ce163a3d33751b1600ea3243896c4aac` L4029–4038 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `ca3cb850fd9515bbced6c89958eddc56aa5b590abe620591317443b5f37d2bde` L4041–4047 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `ed135905d19983397b8cd1f711247dfd3b1fdc7efc18167a1c87e13074e04d39` L4135–4141 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `5197dcd109cccdba25342b0f8e895b16a906bb9adda6e86d33d16e99af152e4a` L4190–4196 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `ddbf878f0ac6dbbf395000808de35b7656194465ef5ea5e4fa3e93fbfa7a4787` L4236–4242 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests
- `dc293140886c78cd994c09ade92272c37d539951ceb23bb06567f667d5ee8cb7` L4461–4474 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `crates/engine/speclink-core/tests/golden/security-review-antigravity.snapshot.md`

- `17f5e28bc30d7312c63b4b15f27b61ff5a4dd08b2fd70b55fd51b650ee520f5b` L1–84 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `crates/engine/speclink-core/tests/golden/security-review-claude.snapshot.md`

- `ab0a6213900e8f6d5c0067064332d56680d63da7073fcfe416c556c802c29e71` L1–85 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `crates/engine/speclink-core/tests/golden/security-review-codex.snapshot.md`

- `1ea16a292c15a55a960130f8d4674003665d5c8bf1597c1611153999c935c2e9` L1–84 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `crates/engine/speclink-core/tests/golden/security-review-copilot.snapshot.md`

- `5b2a620c1b417775e576c063c0a9e18d63197cf413489e35d8dbce668d3639b6` L1–84 — **path-only**：generated skill/snapshot or binary fixture; inspect source/consumer tests

### `crates/engine/speclink-core/tests/it/render_golden.rs`

- `ec7e56583f85ed5c73a09cfc7b7af21073b3d1ab9dbf903f62ec0d212dd10a78` L87–136 — **reviewed**：Read 87–136 with context: proposal bullets, registered offline security skill rendering for all tools/goldens and history/Host acceptance handoff assertions. Hunk at new line 87.

### `crates/engine/speclink-core/tests/it/skill_authoring.rs`

- `379c63fc4e9a96760ee82ae611c13cd553d82c4d2197270aec18033d6d8efed8` L32–38 — **reviewed**：authoring test 認可新 security-review 原始碼，計數項無語法問題。

### `crates/engine/speclink-fs/src/archive_checks.rs`

- `6c1eca0ef74182aab3a81448b886477afb0916001f2a44cc0feb4addb07e75c6` L1–463 — **reviewed**：Inspected lines 1–463: archive identity, recovery phases and visibility, bounded symlink-rejecting copy, base validation and final publication under checks lock, rollback/metadata conflict tests. Canonical-spec ordering requires lifecycle context checked separately.

### `crates/engine/speclink-fs/src/checks.rs`

- `ec1f8c99049e57a0c69addba28419b655d237a350204515e2d93a83262c087dc` L1–602 — **reviewed**：Inspected lines 1–602: external/legacy lock exclusion, expected digests, bounded staging/backup journal, ordered publication, rollback and interrupted-recovery tests; no concrete introduced defect found.

### `crates/engine/speclink-fs/src/layout.rs`

- `0deacdf25a0bf36b8db586b8c815274be3148d56a4e0c500e525793bf563d749` L70–79 — **reviewed**：Hunk 70: Read layout hunk: legacy evidence path stays root/.speclink/touched/<change>.json, comment clarifies migration-only source; no path behavior changed.

### `crates/engine/speclink-fs/src/lib.rs`

- `75a4341b3decaac57468a237c509450571f73944192fb61b085213331abe5eb7` L6–13 — **reviewed**：Inspected all 12 assigned hunks with method context: check journal masks metadata, writes/deletes share checks lock, check reads bounded, archive visibility consults transaction journal and archive delegates plan. Discussion hunks are formatting-only, no behavior change. Hunk at new line 6.
- `139cf95a6df4c89c1d07e230a90d2984983d7b445401023ea3283ed56f749f33` L42–61 — **reviewed**：Inspected all 12 assigned hunks with method context: check journal masks metadata, writes/deletes share checks lock, check reads bounded, archive visibility consults transaction journal and archive delegates plan. Discussion hunks are formatting-only, no behavior change. Hunk at new line 42.
- `c85c0e369fe56948cd0dc108437223e70d42089ea4a031af4930084e9b68a887` L103–114 — **reviewed**：Inspected all 12 assigned hunks with method context: check journal masks metadata, writes/deletes share checks lock, check reads bounded, archive visibility consults transaction journal and archive delegates plan. Discussion hunks are formatting-only, no behavior change. Hunk at new line 103.
- `a0ca12912abfdae2c41e73e7925721141544299dac449b38e050f9c9bf41335b` L170–182 — **reviewed**：Inspected all 12 assigned hunks with method context: check journal masks metadata, writes/deletes share checks lock, check reads bounded, archive visibility consults transaction journal and archive delegates plan. Discussion hunks are formatting-only, no behavior change. Hunk at new line 170.
- `513787f074f4178009ece10d4e924ad55c5612ec078f0b83ff6df79aed69c734` L184–203 — **reviewed**：Inspected all 12 assigned hunks with method context: check journal masks metadata, writes/deletes share checks lock, check reads bounded, archive visibility consults transaction journal and archive delegates plan. Discussion hunks are formatting-only, no behavior change. Hunk at new line 184.
- `e47f3811b37fd183d4915c528ab04eb210c209d1767b40360d76b4cbd0968880` L205–211 — **reviewed**：Inspected all 12 assigned hunks with method context: check journal masks metadata, writes/deletes share checks lock, check reads bounded, archive visibility consults transaction journal and archive delegates plan. Discussion hunks are formatting-only, no behavior change. Hunk at new line 205.
- `bf380d54cd06a9e9f611fa47189fbc4bfd229a466294f49cd91e1461858db489` L216–260 — **reviewed**：Inspected all 12 assigned hunks with method context: check journal masks metadata, writes/deletes share checks lock, check reads bounded, archive visibility consults transaction journal and archive delegates plan. Discussion hunks are formatting-only, no behavior change. Hunk at new line 216.
- `a774b622a82aaf68a36fa9833ab31dcdccb4284da953279167ea94c2c8794636` L342–351 — **reviewed**：Inspected all 12 assigned hunks with method context: check journal masks metadata, writes/deletes share checks lock, check reads bounded, archive visibility consults transaction journal and archive delegates plan. Discussion hunks are formatting-only, no behavior change. Hunk at new line 342.
- `ab6a4d81836718d8a775166be7fd42c4ceaf8f6556dc62ccbfe8df6cb2c0bdb4` L353–403 — **reviewed**：Inspected all 12 assigned hunks with method context: check journal masks metadata, writes/deletes share checks lock, check reads bounded, archive visibility consults transaction journal and archive delegates plan. Discussion hunks are formatting-only, no behavior change. Hunk at new line 353.
- `b5382649fa4d212651329b50310fb8fd8569bf91ace126affb8e2f3cae9ef983` L422–431 — **reviewed**：Inspected all 12 assigned hunks with method context: check journal masks metadata, writes/deletes share checks lock, check reads bounded, archive visibility consults transaction journal and archive delegates plan. Discussion hunks are formatting-only, no behavior change. Hunk at new line 422.
- `971289c6821b03f16422aadb44594365faa934bbdf1374c0de687f8e6f57c06a` L488–499 — **reviewed**：Inspected all 12 assigned hunks with method context: check journal masks metadata, writes/deletes share checks lock, check reads bounded, archive visibility consults transaction journal and archive delegates plan. Discussion hunks are formatting-only, no behavior change. Hunk at new line 488.
- `e05fe6ace6c3dc288b4137d5d6846b7e0615c859a0e02f2cb5a88dc8c877698d` L514–526 — **reviewed**：Inspected all 12 assigned hunks with method context: check journal masks metadata, writes/deletes share checks lock, check reads bounded, archive visibility consults transaction journal and archive delegates plan. Discussion hunks are formatting-only, no behavior change. Hunk at new line 514.

### `crates/engine/speclink-fs/src/security.rs`

- `3411685b44ca693f880e0ab30240193074c4cd11a39e210e20d6a1a06484c1dd` L1–5 — **reviewed**：Hunk 1: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.
- `9a02efd40fc64ac5444d557618e225a954975ae08c298f656728369b3534ad3d` L25–32 — **reviewed**：Hunk 25: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.
- `2693a78a2f0c34285b84111c378baa54fe2a07236e076d821bbe50113515f69e` L52–86 — **reviewed**：Hunk 52: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.
- `b1dc187d3a264295b61fe7f93b592fd5f2118982e68e8095c2245264f6faa865` L96–102 — **reviewed**：Hunk 96: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.
- `fb20e3cfbdd6a77982e3618f11b1b50bf3254768060c92796a11a762058b5f18` L138–144 — **reviewed**：Hunk 138: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.
- `d7d94bd9acd8be2a2996acc721c045f4cb6c4c91687c4e2f1a3dce59f1b49f23` L153–161 — **reviewed**：Hunk 153: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.
- `4d31dd7934c34c63e7bdf9cd631d0cb9782c83c27652cb2f1e6e6748c8ded115` L233–242 — **reviewed**：Hunk 233: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.
- `6f7323cf15493854030d9521e573a7b966c9f0585b7c7d949a805610f22bb67f` L245–251 — **reviewed**：Hunk 245: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.
- `19c232a0aa62f679e4720c591313dd3762f5a2ef84a0f178bb0c21308bb8e1e6` L327–334 — **reviewed**：Hunk 327: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.
- `867a1e63e56045a33d5eb8ffbeec0dab9defaaf987acef98fef7e5b30c921872` L386–392 — **reviewed**：Hunk 386: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.
- `dcdbbe4f9833d4b5f67dc108d5447da929db17b64fc270dfdbbe8ffccfb5ac0c` L410–416 — **reviewed**：Hunk 410: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.
- `b320815be242b9dd3fdb5813373e84cf946891ee7ba90de48e4b35d206a2963a` L507–513 — **reviewed**：Hunk 507: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.
- `7d01d57a02a6d970ab6ec8becf053aefc9bb395089f27ff974f898ab5388de2a` L535–541 — **reviewed**：Hunk 535: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.
- `af33572c5eb057ed00f130b629b53202d818da70eb2ab84c352cafbc973777e3` L552–558 — **reviewed**：Hunk 552: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.
- `f32d4c59b3644bed3ea6f750a0b60026098baa670cec9ee56c93c338ec75b987` L637–647 — **reviewed**：Hunk 637: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.
- `190336b0d5d232e3b120da17bf8570bce5ea5a9ea0858738bd0e2ab9268209a5` L655–661 — **reviewed**：Hunk 655: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.
- `2cb23bdb9cf24ff244881b22a8b18470de76163623cdb211a170e6b27423eba6` L665–671 — **reviewed**：Hunk 665: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.
- `1ca8c3bac30fc430dd9b323d47ca8c2e9697d8ac6ec12b5aed32760031192268` L677–683 — **reviewed**：Hunk 677: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.
- `3f4a711d369de6b783ff1dd915dc8a6fe9654e60334e8b4599685fff1f481218` L857–871 — **reviewed**：Hunk 857: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.
- `fb11457eb166fa59d9b6066c1c533ecb0231eb6006e337df9d260ca85f57274d` L1015–1045 — **reviewed**：Hunk 1015: Read all modified FS security hunks and owned_directory context. Custom root is absolute, exists, checked without symlinks; version operations use selected boundary, share locks/pins, retain private references and deny custom-root deletion. Registry knowledge status is derived on read; raw ZIP headers checked. Added tests cover sharing and disconnection.

### `crates/engine/speclink-fs/src/security/knowledge.rs`

- `fe63e04276dcb8297f6c5ec05214e75d6c8be8a57ca3d9d72dec9b74207f4817` L1–399 — **reviewed**：Inspected lines 1–399: pinned repository generations, guidance hashes, deterministic snapshot assembly, prior snapshot validation, stage/backup publication and ignore rollback; read local completeness, cross-device digest and symlink/cancellation tests. Hunk at new line 1.

### `crates/engine/speclink-fs/src/security/prepared.rs`

- `53b3f640196bd39fcffe5aa365bfdd9cb4c54df0236842957188414794c3f23b` L271–277 — **reviewed**：Hunk 271: Read prepared fixture hunk: optional canonical proposal key initialized None for existing local/cache-binding fixture.

### `crates/engine/speclink-fs/src/security/reports.rs`

- `4b82ad7bf4c3cf06119ee7ccb806c645dc8406218ef5d21a957b7b0521d1cf61` L50–86 — **reviewed**：record_publication 校驗 report/change/id，僅現存裝置草稿寫 publication receipt，原子替換 metadata。
- `5b6f2019f39ac91fa272c8cedb14eeed7af7478392d65bd36780f436640cf447` L286–354 — **reviewed**：測試 v1/v2 同一提案 index、深度報告遮蔽、report JSON 摘要竄改拒絕、封存後讀取與不存在 ID。

### `crates/engine/speclink-fs/src/security/scan.rs`

- `3d8baf1d474fce1f594697781125cd725dc564a8e69734623ee4616ecf95e98d` L92–118 — **reviewed**：Hunk 92: Read every modified scan hunk plus scan/read loop context. Binary reads preserve bytes through authorized open_relative and enforce limits/cancellation; Deep adds listed languages/config files without widening basic scan. Unsupported/generated filters remain applied; tests cover bounds, missing/link paths, cancellation and language counts.
- `ee9a9db66b2e86f78b0ff7d6bef6994ea262343b13e2b6b31c3d4b407c42aecc` L288–330 — **reviewed**：Hunk 288: Read every modified scan hunk plus scan/read loop context. Binary reads preserve bytes through authorized open_relative and enforce limits/cancellation; Deep adds listed languages/config files without widening basic scan. Unsupported/generated filters remain applied; tests cover bounds, missing/link paths, cancellation and language counts.
- `2db3238ad139a1be57a5b7c6aef530e965aaf380ca496ae58a042b9e1b923a25` L374–382 — **reviewed**：Hunk 374: Read every modified scan hunk plus scan/read loop context. Binary reads preserve bytes through authorized open_relative and enforce limits/cancellation; Deep adds listed languages/config files without widening basic scan. Unsupported/generated filters remain applied; tests cover bounds, missing/link paths, cancellation and language counts.
- `a5c3722179c129734ecc2588a0ee7dfc5d926bedae66c7cb407aa2e6bf2b3a52` L406–414 — **reviewed**：Hunk 406: Read every modified scan hunk plus scan/read loop context. Binary reads preserve bytes through authorized open_relative and enforce limits/cancellation; Deep adds listed languages/config files without widening basic scan. Unsupported/generated filters remain applied; tests cover bounds, missing/link paths, cancellation and language counts.
- `5fe033f79579c0ded8dff030dc49900f61fffc1b70a5c668f0648c48e2af1490` L513–534 — **reviewed**：Hunk 513: Read every modified scan hunk plus scan/read loop context. Binary reads preserve bytes through authorized open_relative and enforce limits/cancellation; Deep adds listed languages/config files without widening basic scan. Unsupported/generated filters remain applied; tests cover bounds, missing/link paths, cancellation and language counts.
- `d582219115520c4564454aadddf9b1cd6a2088d0572a1a31381b3b3f15dfcc73` L549–560 — **reviewed**：Hunk 549: Read every modified scan hunk plus scan/read loop context. Binary reads preserve bytes through authorized open_relative and enforce limits/cancellation; Deep adds listed languages/config files without widening basic scan. Unsupported/generated filters remain applied; tests cover bounds, missing/link paths, cancellation and language counts.
- `8fe39cbf525f32e4e08410560e936ccb807c9274c9c7f2a9e262f403d7678320` L676–731 — **reviewed**：Hunk 676: Read every modified scan hunk plus scan/read loop context. Binary reads preserve bytes through authorized open_relative and enforce limits/cancellation; Deep adds listed languages/config files without widening basic scan. Unsupported/generated filters remain applied; tests cover bounds, missing/link paths, cancellation and language counts.

### `crates/engine/speclink-fs/src/security_packages.rs`

- `4e5adc6079b1df056c89e60502393af5131ffad2442f2ed856c2ac923f36f5b7` L472–529 — **reviewed**：逐段讀取離線資料包測試：舊包仍供 builtin，完整 guidance roundtrip、hash 篡改與未允許檔名拒絕。

### `crates/engine/speclink-fs/tests/fixtures/security-guidance.zip`

- path-only：generated skill/snapshot or binary fixture; inspect source/consumer tests


### `crates/engine/speclink-fs/tests/store_fs.rs`

- `e04cba8735c77b76e37092cd9b207f777649c6ba8af56e8b821987206168e44d` L540–569 — **reviewed**：測試 checks 活動/封存讀取拒絕 file/parent symlink，避免讀出外部秘密。

### `crates/host/speclink-host/Cargo.toml`

- `b5de0a1f8eb293f59ec8740986010bcc315f613c5b0e9104692000956cee626f` L13–19 — **reviewed**：Hunk 13: Read host Cargo hunks: tempfile available for host runtime work and zip deflate confined to tests; dependency directions unchanged by these entries.
- `cf5ef191bb5d5bc3de526ef446dce63ae232515702204149674cd322e72415ce` L25–31 — **reviewed**：Hunk 25: Read host Cargo hunks: tempfile available for host runtime work and zip deflate confined to tests; dependency directions unchanged by these entries.

### `crates/host/speclink-host/src/bridge.rs`

- `f974b47c8dfc86874496cdb43fcf9e682a865caf4927d173d83528de69fb95e2` L63–77 — **reviewed**：來源路徑映射為守衛文件，僅提案或 canonical spec。
- `ffd4a379be7eda340abf282ed48679f7a2bba2339fc40a1a3ef21dd1811584d9` L81–114 — **reviewed**：execute_with_security 驗證 index revision、生成 proof，守衛 policy 與 attested files。
- `082d5dd0addae1cb0c10df63e83b2873c8904a531a368da079ede2c46f5f6013` L122–128 — **reviewed**：把讀取 guards 帶入同一 unit-of-work commit。
- `cee4f382183c34ad5e95ea30c2f90f00a8b0e8ed41983712a3dd6850bfd7fff4` L173–179 — **reviewed**：BridgeStore 新增 guards 容器。
- `343ee7fed1682f17bb2c874c705589306e63789646f22bdfa2956ec6f9adef28` L202–227 — **reviewed**：revision/guard 取自 materialize base，避免後續讀漂移。
- `106c608b5df453bb0addc686bc6e9cf5b7bc28cf62a413e7e8985da810d333d3` L417–435 — **reviewed**：commit_checks 驗證 compare 後統一 stage writes/deletes。
- `f671b2bdb4c74fa4d5515ca5a8bae2314941e9064765bd8ff70bac66c4c7487f` L562–577 — **reviewed**：封存 checks 套計畫並刪除未保留文檔，最後寫 index/receipt/meta。
- `71b93bf0726886ded12eee45476da78a17846c48a109b5dfe9145bf455d1204f` L722–728 — **reviewed**：測試引入 attested_document。
- `0cd9c6ebc59227261763a2511d2a518f265ba9cefeb33056de04ba6355b32569` L742–827 — **reviewed**：測試 CAS 競爭不留下部分寫入，policy/spec guards、封存衝突保留 active records。

### `crates/host/speclink-host/src/context.rs`

- `2f742db449f32d4b153e0d5e01b8937f2014c1f81f2fccbf258c89939fc474bc` L154–194 — **reviewed**：測試路徑設定本機隔離與原子失敗、未知版本拒絕、reader lease 阻擋改位置及 Windows UNC 路徑。

### `crates/host/speclink-host/src/history.rs`

- `814cf6cd77b3b2d90be469a519a2145ea5e8779c01408260334c63a45ed1c014` L1–84 — **reviewed**：逐段讀取 1–84：history query 依 kind 限定 record，list security 可附 receipt；remote 從 scoped BridgeStore materialize，NotFound 映射保留。

### `crates/host/speclink-host/src/lib.rs`

- `b4f23a1003d197201a25305d0743f77ec841ad14f6f4725b3793197f6f53297a` L16–28 — **reviewed**：Read host lib module exports hunk: history/security_paths exported alongside existing security/network modules; module-only wiring. Hunk at new line 16.

### `crates/host/speclink-host/src/network.rs`

- `8e744dbe9501312f51c93d8e9d370728c568bc003ea991929df06669d5c9f19b` L1–57 — **reviewed**：Read bounded release metadata fetch, fixed repo allowlist, no redirects, status/size/parse errors, request_error mapping and malformed-body test. Network calls remain explicit through supplied proxy policy. Hunk at new line 1.
- `200644a5a2674a4ce4a9941780bc474a0d9116c9db84c8b24e298d9aecb0fed4` L265–289 — **reviewed**：Read bounded release metadata fetch, fixed repo allowlist, no redirects, status/size/parse errors, request_error mapping and malformed-body test. Network calls remain explicit through supplied proxy policy. Hunk at new line 265.
- `c640b61eabeb7bfb312286fcdd01fd31339edf49dbf4c881c1ca7232c84f304d` L356–395 — **reviewed**：Read bounded release metadata fetch, fixed repo allowlist, no redirects, status/size/parse errors, request_error mapping and malformed-body test. Network calls remain explicit through supplied proxy policy. Hunk at new line 356.

### `crates/host/speclink-host/src/security.rs`

- `2a43238e7bf7e360d8019848fb4a0dbb3c021f3b1ccdad74e3c311a8ed541c23` L21–29 — **reviewed**：匯出 deep/remote 模組。
- `b10a02de1b26ae697a48e17dfb80eb39a40629839251ad08fffd22a2ff27a742` L60–66 — **reviewed**：proposal key 與 device binding 分離。
- `a20e156e472f433088f37815c1a066f25ad88a2b385ed8a16d1114ee377c4814` L73–79 — **reviewed**：本機 target 無 proposal override。
- `ec5149f0cfa66c0a44b239c3624cf42c44b20e5a805adc5d9ba0f53fdb0bfe68` L92–98 — **reviewed**：proposal key 走 valid_project_key。
- `3c62a38e6fc3448c7726724abdd9713ca2aba560f43d89e553c9501486217dcf` L101–112 — **reviewed**：提案/知識 key 選擇及 preparation input 欄位。
- `0d4ab50cbc530fb244f9b9ca2a10c4dd8a314405c4a9cf30f20aad4d003b23a9` L132–138 — **reviewed**：prepared snapshot 保存 proposal override。
- `0825bb916641f659a388905e807fda91725e549c38a35324eb7af86d010293be` L211–217 — **reviewed**：本機 preparation 無 override。
- `c4199c7efeeb361b36bb0fb12f083bb41df39174bc12fd607764e26523375073` L287–293 — **reviewed**：remote preparation 保留 override。
- `10e9e836d77d74f0c6d3e5de566159bd9d679590da6f12358d9c59048fe98296` L443–453 — **reviewed**：report key 與 cache key 分開，避免裝置/伺服器混淆。
- `533efcf249dbb8cbebd813cad08497229b2f409f49ae7bae0d059eb534e27840` L496–502 — **reviewed**：進度事件仍使用 cache routing key。
- `1eb7318ee4fe2d27978d0288b45dc0b1997371e0a22534f6fa7cd22e9ea346de` L658–664 — **reviewed**：讀 prepared snapshot 使用 cache key。
- `c260f27366eeae3b4d87d6f9356c85f3222d2b16d74c337bf2c27f47f630c62d` L975–1008 — **reviewed**：ProposalReports 直接從提案 index 保存/讀取並按 key 隔離；測試鎖避免全域環境競爭。
- `60415e0a7b0530ed35e65f46fec6183369628d52d587963ae334d2637f17507d` L1044–1054 — **reviewed**：disabled policy 測試新增 bytes 讀取 panic，確保不讀知識。
- `5650285c66d39b7023dd62d587e4be82119faef09ec95eb39d56d5b41f594522` L1139–1145 — **reviewed**：測試 remote target 初始化 override。
- `f011249b5aaeb6f29fd854195ff9a55573d29a6ef9fa5967a0712da5caef4e20` L1152–1158 — **reviewed**：測試 remote preparation 初始化 override。

### `crates/host/speclink-host/src/security/deep.rs`

- `aba544c9975cfc95f529e206371b62406d6a7c4e3f711147b7b9e490953c55b7` L1–1234 — **reviewed**：Read deep.rs 1–1234 in bounded chunks. Checked local binding/policy/index freshness, latest deep-to-builtin pairing, accepted-risk revision guard, knowledge manifest hashes and selected control references, bounded source scan/evidence mapping, submission save and redacted review view. Read added regression test covering added/deleted/changed sources, knowledge tampering, risk identity and archive proof invalidation. No definite introduced defect found.

### `crates/host/speclink-host/src/security/remote.rs`

- `e08ee959e0dd3d4025b846a875109fb17ff4e6828f1fd50e73096179a8429def` L1–563 — **reviewed**：Read remote.rs 1–563. Checked scoped BridgeStore materialization, expected index revision CAS, builtin/deep atomic staging and duplicate identity, legacy binding, accepted-risk index advance, timestamp/knowledge/source attestation validation and SDD source-set equality. No definite introduced defect found.

### `crates/host/speclink-host/src/security_data.rs`

- `a63ef337bc7918c0d0f22e62e264710a457ed5fd85239bcc201d9eb1fffd65ab` L488–498 — **reviewed**：Hunk 488: Read all security_data hunks: approved pinned guidance URLs join source allowlist, install downloads all registered files then requires ready before commit, inventory exposes derived knowledge status; fixture serves real guidance bytes and checks total progress.
- `616b217312904cd09e895599e08ff0be93446ff5762f5208600eaff954e2cf42` L753–761 — **reviewed**：Hunk 753: Read all security_data hunks: approved pinned guidance URLs join source allowlist, install downloads all registered files then requires ready before commit, inventory exposes derived knowledge status; fixture serves real guidance bytes and checks total progress.
- `fcedfa8dc13c4605334a40c5e08cbf08d8532c7df47529e65701c78d11eda7d8` L774–781 — **reviewed**：Hunk 774: Read all security_data hunks: approved pinned guidance URLs join source allowlist, install downloads all registered files then requires ready before commit, inventory exposes derived knowledge status; fixture serves real guidance bytes and checks total progress.
- `6b8ff4d139e0fd656a38a6b09f9ac64cab5ab34b7a94ac77203f4498df2b6aa7` L798–804 — **reviewed**：Hunk 798: Read all security_data hunks: approved pinned guidance URLs join source allowlist, install downloads all registered files then requires ready before commit, inventory exposes derived knowledge status; fixture serves real guidance bytes and checks total progress.
- `1fdd60ca35a07fe5fe055b281f698336f0410eb10a8f4f5b984496421700564f` L813–827 — **reviewed**：Hunk 813: Read all security_data hunks: approved pinned guidance URLs join source allowlist, install downloads all registered files then requires ready before commit, inventory exposes derived knowledge status; fixture serves real guidance bytes and checks total progress.
- `1ade88a87777972c26c5b90d60d927130f501c1ce187b4016b0b50b619097ca0` L842–848 — **reviewed**：Hunk 842: Read all security_data hunks: approved pinned guidance URLs join source allowlist, install downloads all registered files then requires ready before commit, inventory exposes derived knowledge status; fixture serves real guidance bytes and checks total progress.
- `00dd84d9c7452e14334ebcdc6e199e4d7aef05ac5ad6fa1851258bb54e102923` L1272–1283 — **reviewed**：Hunk 1272: Read all security_data hunks: approved pinned guidance URLs join source allowlist, install downloads all registered files then requires ready before commit, inventory exposes derived knowledge status; fixture serves real guidance bytes and checks total progress.
- `7512c0bbe58f69658d9c54b8f3a1aa5d52eca1434b28852098471f5f860a87d5` L1310–1317 — **reviewed**：Hunk 1310: Read all security_data hunks: approved pinned guidance URLs join source allowlist, install downloads all registered files then requires ready before commit, inventory exposes derived knowledge status; fixture serves real guidance bytes and checks total progress.

### `crates/host/speclink-host/src/security_paths.rs`

- `604dbfbb685066cf7234033035187e736c4eec9a97ad623ea80c2730a62a8aee` L1–246 — **reviewed**：逐段讀取 1–246：檢查絕對路徑與 symlink、唯讀位置快照、共享/獨佔 lease、偏好原子寫入；macOS alias 測試與 resolve_system_path 配合。

### `crates/host/speclink-host/src/worktree.rs`

- `7fc8ece06d33577f78ca374348ea2bc296b55cb293203822530ffeb6f21f5d69` L279–287 — **reviewed**：Hunk 279: Read worktree hunks plus WorktreeOverlay contract/context: checks readability follows mapped change read store, recovery/commit/archive writes delegate main store consistent with explicitly read-only overlay contract.
- `c1ec03fba6960b0480246aee653603773f2ff71348909f13968a4b0d04dbfef1` L315–322 — **reviewed**：Hunk 315: Read worktree hunks plus WorktreeOverlay contract/context: checks readability follows mapped change read store, recovery/commit/archive writes delegate main store consistent with explicitly read-only overlay contract.

### `crates/host/speclink-host/tests/security_remote.rs`

- `67b1db3cf1c32e20d04e6cdf9f3439a1e91f71feb290e81b7f72b6ac6483962c` L1–476 — **reviewed**：逐段讀取 1–476：遠端整合測試覆蓋成對原子發布、CAS、來源憑證過期、接受決策遮蔽及封存 receipt。斷言與各流程前置狀態相符。

### `crates/host/speclink-server/Cargo.toml`

- `068c80eee02f2fab327e3a63b4f7201d1da86baf5878f157348058a925b47899` L11–17 — **reviewed**：Read newStart 11–17 with package/dependency context: server now directly depends on speclink-fs alongside core/host/protocol; relative path resolves to crates/engine/speclink-fs. No generated lockfile correctness claim. Hunk at new line 11.

### `crates/host/speclink-server/src/api.rs`

- `3caeee6539b5af71202b619bccb07dfa4afeab4c06fd1ad90417a39ef0fa6ef4` L8–12 — **reviewed**：Hunk 8: Read API module hunk: security module is exported for referenced handlers/helpers.

### `crates/host/speclink-server/src/api/error.rs`

- `c376a30626b12e6a2f03ebc2960e7eeaa69ee57d9f7a14e54eacba814257b573` L1–10 — **reviewed**：Hunk 1: Read error mapper hunks: typed StoreError CAS/forbidden/unavailable retained; security codes map to dedicated reasons, malformed input to 400, absent targets to 404 and messages redacted; CommandError gate sources routed through same mapper.
- `87d97d12cc516474fdde3410a5fa02b3e75e1995176ab3de04e8d2d52778976a` L36–68 — **reviewed**：Hunk 36: Read error mapper hunks: typed StoreError CAS/forbidden/unavailable retained; security codes map to dedicated reasons, malformed input to 400, absent targets to 404 and messages redacted; CommandError gate sources routed through same mapper.
- `7a7f6afc281051856b712a209ec9702772f96940bf96e8a0a3a4e2beb3a904d7` L163–169 — **reviewed**：Hunk 163: Read error mapper hunks: typed StoreError CAS/forbidden/unavailable retained; security codes map to dedicated reasons, malformed input to 400, absent targets to 404 and messages redacted; CommandError gate sources routed through same mapper.
- `fd051ebecfff792b38c9d22f556ec4b177cd7eef2b5e2234e1989fa2e9d719de` L180–186 — **reviewed**：Hunk 180: Read error mapper hunks: typed StoreError CAS/forbidden/unavailable retained; security codes map to dedicated reasons, malformed input to 400, absent targets to 404 and messages redacted; CommandError gate sources routed through same mapper.

### `crates/host/speclink-server/src/api/routes.rs`

- `a833c496b8e33c2732d724698f5f9bc6cd3be3def1527d25d1545eeecf4dd300` L1194–1221 — **reviewed**：新增三類 history handler；security record 使用專用 endpoint；query result 仍回 scoped etag。
- `fe559130969d3b5b4e38f7087052370dbe104eaa069cc6e3891c4af588e296da` L1346–1356 — **reviewed**：verify stamp 透過 run_with_security 帶 request gate。
- `aef4994beaf53ed8a088ff24a5d754725d4abc9ebbe3bdad777535cd4c1125fd` L1475–1483 — **reviewed**：archive 可接受 JSON gate body。
- `7cac5d42fefa03e137bea50cbf03971cedf440a9ac533409ef111e3f2f1c649e` L1488–1494 — **reviewed**：缺 archive gate body 採 default，core 決定是否必要。
- `4acb58764a86a537a100fe7097362bbb36c35b815fb5473d36a8f302641ec87f` L1496–1502 — **reviewed**：ArchiveResponse conditionally 回 counts，serde 類型對齊。

### `crates/host/speclink-server/src/api/security.rs`

- `8a46eb004a5016a70bfabc9a6211762c42866fcf321c128b74f3d977fd280198` L1–113 — **reviewed**：逐段讀取 1–113：HTTP Handler 身分由 Binding 決定，Host 操作用 spawn_blocking，source key 綁 public URL/project/repo；標準 repository 持 path lease。

### `crates/host/speclink-server/src/api/verb.rs`

- `f20fa803b27b44c339eb64b47ed7097d154fe871dc14263c41caa734576d0374` L54–72 — **reviewed**：Hunk 54: Read server verb hunk: no-attestation uses existing command gate, attested path computes canonical project key and executes proof with same bridge snapshot on blocking worker; events only notified on success.

### `crates/host/speclink-server/src/app.rs`

- `d666b83cc9ad9ea01e75a677c327ebe7b84ba3d5541d7a22d0a9cb636ae2de34` L71–89 — **reviewed**：三類歷史/安全端點掛到 project router，深度提交 body 限制 8 MiB。
- `ca96810a2b96761fce4c06c3587b57fbe614eb2cc1cbfcdb9cedc24d72ba3af7` L154–161 — **reviewed**：只讀 security/check POST 放在 editor middleware 以外，仍位於 authenticated project router。
- `447c399b43f1b604d780cf4b1587f79a138524780e946282f9d18f58d6b5dce2` L255–260 — **reviewed**：binding response 使用 server public URL 計算 security key，錯誤正確傳遞。

### `crates/host/speclink-server/src/identity/auth.rs`

- `0c46b5e3360f6fd0fd4650b46c27c36dca1c95abf24c3e00adb5d9d79904f70a` L77–86 — **reviewed**：binding public URL/project/repo 衍生 security project key。
- `5316f357de5da1f1bc162bacc366b5a4ed605577f339f82b2270fa4b16e45982` L98–105 — **reviewed**：server 宣告 v2/history 能力，對應新增 endpoints。
- `55ce7da3b166a3021a2b9121fa2e87286ea17cc7f2cffbd8563d18f88a111160` L122–128 — **reviewed**：to_response 回 Result 與 caller ? 對齊。

### `crates/host/speclink-server/tests/it/api/review_api.rs`

- `51bed6f007e521cd992ad197f043b13fd486b5276cd19ed92af25333efee8677` L38–79 — **reviewed**：Read all 103 assigned lines with test context: unauthenticated/reader security writes leave no index/revision, binding and revision conflicts, reader history and disconnected standard-root report lookup. Hunk at new line 38.
- `1b98db41010960a40143cb0e2cb6c02c9558155d632b9fc84ba81372fbf093d3` L201–223 — **reviewed**：Read all 103 assigned lines with test context: unauthenticated/reader security writes leave no index/revision, binding and revision conflicts, reader history and disconnected standard-root report lookup. Hunk at new line 201.
- `e327f134e90edf3521cd4a8aa462336790d4613ddd05ef294e7f0c4dd646d49d` L479–516 — **reviewed**：Read all 103 assigned lines with test context: unauthenticated/reader security writes leave no index/revision, binding and revision conflicts, reader history and disconnected standard-root report lookup. Hunk at new line 479.

### `crates/host/speclink-server/tests/it/api/write_guard.rs`

- `c42164862c70294af801c0e5a02340ddbba8f1e5d3bdeda0d145ddce5af7eb4e` L32–39 — **reviewed**：Hunk 32: Read write-guard test hunk: both new mutating security routes are included in write authorization matrix.

### `crates/protocol/speclink-protocol/src/binding.rs`

- `a7c85aa7d52ea55d317d6d271a66a0cd62e282b7423c1585e9f0fabd13e4d448` L17–24 — **reviewed**：binding optional key 對舊 wire 預設 None。
- `b108790b82657f5cafd8c76d4038b3f384facbc02369adc9fcaca05798cf6fcd` L46–55 — **reviewed**：新增 capability 對舊 server 預設 false。

### `crates/protocol/speclink-protocol/src/command.rs`

- `4e23300f4f217d8608b04dfd4c0c9f43baedf7a7f487b08397a61aa77cb202b2` L136–143 — **reviewed**：ArchiveResponse 新增 optional counts，舊回應可讀。
- `065d4cd0b034f728a771e03877cb43ece6bce72c6b04599fbc4cb96ec46ee584` L150–162 — **reviewed**：counts wire 欄位 camelCase 與 core shape 相符。
- `72f3958d1c3d8f8dc8891603dca46f1b52f3f7e832959e7d70b3b6b0a550f84c` L461–468 — **reviewed**：StampReviewRequest flatten gate request。
- `d673e7191823fc0c2be576ef1da2eb719df73b5e70c00eaf90190b248ef5221a` L715–721 — **reviewed**：序列化測試初始化 optional counts None。
- `47b7ed2ba9643858fa09c5cc6078d11ef11257d80d179db92c5f4e92ea643050` L908–914 — **reviewed**：request roundtrip 測試初始化 default security gate。

### `crates/protocol/speclink-protocol/src/error.rs`

- `6847514ef9ae069539da1ab4f6eae0014568cc3a91b524b13f2926f9faec0bac` L41–54 — **reviewed**：Read all five error hunks and nearby serde/test cases: new error variants map to stable snake_case wire strings with Unknown fallback, exhaustive round-trip registry tests updated. Hunk at new line 41.
- `6ac2fa36a1b55cd469181163dbba2739a7c1d71427157161afdcec85c6ad9207` L67–80 — **reviewed**：Read all five error hunks and nearby serde/test cases: new error variants map to stable snake_case wire strings with Unknown fallback, exhaustive round-trip registry tests updated. Hunk at new line 67.
- `0866688174ef8a8826649a99dbd841c9369ca99b1a13cfe16320a5b327632fa6` L117–131 — **reviewed**：Read all five error hunks and nearby serde/test cases: new error variants map to stable snake_case wire strings with Unknown fallback, exhaustive round-trip registry tests updated. Hunk at new line 117.
- `4fb978e788e9ab089961b0470bcbc5db327e65dfc978a68cd334eda4745de53c` L134–140 — **reviewed**：Read all five error hunks and nearby serde/test cases: new error variants map to stable snake_case wire strings with Unknown fallback, exhaustive round-trip registry tests updated. Hunk at new line 134.
- `002e7e17f645dd7a060a682f855ce2efad3ce3b583889ae69e007a9772908860` L145–160 — **reviewed**：Read all five error hunks and nearby serde/test cases: new error variants map to stable snake_case wire strings with Unknown fallback, exhaustive round-trip registry tests updated. Hunk at new line 145.

### `crates/protocol/speclink-protocol/src/lib.rs`

- `5b68865c73c2f17af48e92b829d06c698402f6b534599e0e8e3cc8a6a59910fe` L15–21 — **reviewed**：protocol 暴露 security 模組，其他新增 wire imports 可用。

### `crates/protocol/speclink-protocol/src/query.rs`

- `c600f9514b8dcb61dd27faa10fa942fb13a6e540db859f46afa5529cc916d242` L12–64 — **reviewed**：Hunk 12: Read new history wire records and every changed test hunk: camelCase/optional receipt metadata and untagged response shapes agree with serialized values; remaining hunks preserve existing test assertions while formatting them. No behavioral defect found.
- `c2e6a17b28520a019e5ba367e4b8d9611a3fa3c2a418fb3dfb5e6cd4f8e1f2c7` L899–908 — **reviewed**：Hunk 899: Read new history wire records and every changed test hunk: camelCase/optional receipt metadata and untagged response shapes agree with serialized values; remaining hunks preserve existing test assertions while formatting them. No behavioral defect found.
- `33837b7946a95f0937769521845eee655f24ba3d7158958070ac9e1eae07f460` L956–971 — **reviewed**：Hunk 956: Read new history wire records and every changed test hunk: camelCase/optional receipt metadata and untagged response shapes agree with serialized values; remaining hunks preserve existing test assertions while formatting them. No behavioral defect found.
- `e338dab3f773b026357bee33966a9bfe3b284783d9d1f6991b31d3079600fc36` L1042–1051 — **reviewed**：Hunk 1042: Read new history wire records and every changed test hunk: camelCase/optional receipt metadata and untagged response shapes agree with serialized values; remaining hunks preserve existing test assertions while formatting them. No behavioral defect found.
- `13f745a01aa1a4f727f153c323e22c0b931cbc0b2d507306ac3ffd6c5a3d5353` L1055–1064 — **reviewed**：Hunk 1055: Read new history wire records and every changed test hunk: camelCase/optional receipt metadata and untagged response shapes agree with serialized values; remaining hunks preserve existing test assertions while formatting them. No behavioral defect found.
- `b723c1df1a051b41d4dc014a8c1299aa821f7f3a8b9584950aa9a59311667dc7` L1134–1149 — **reviewed**：Hunk 1134: Read new history wire records and every changed test hunk: camelCase/optional receipt metadata and untagged response shapes agree with serialized values; remaining hunks preserve existing test assertions while formatting them. No behavioral defect found.
- `ab021516569ec8016297cb2eef752fe428e809f1ac3611026b17dc43cf0fc280` L1283–1293 — **reviewed**：Hunk 1283: Read new history wire records and every changed test hunk: camelCase/optional receipt metadata and untagged response shapes agree with serialized values; remaining hunks preserve existing test assertions while formatting them. No behavioral defect found.
- `f853397ae8566c80752e54cf3af16db193c04116547970032e7e2db88ae28573` L1307–1316 — **reviewed**：Hunk 1307: Read new history wire records and every changed test hunk: camelCase/optional receipt metadata and untagged response shapes agree with serialized values; remaining hunks preserve existing test assertions while formatting them. No behavioral defect found.
- `71b62bfb6497b91051322bd195e286dc439ea9242cdc45dd50fb707d675bbcb8` L1354–1363 — **reviewed**：Hunk 1354: Read new history wire records and every changed test hunk: camelCase/optional receipt metadata and untagged response shapes agree with serialized values; remaining hunks preserve existing test assertions while formatting them. No behavioral defect found.
- `6b231513a81f36223580be4fc44d8a6bbb0b17d921fc84b17df1a9cf7af1d26d` L1376–1385 — **reviewed**：Hunk 1376: Read new history wire records and every changed test hunk: camelCase/optional receipt metadata and untagged response shapes agree with serialized values; remaining hunks preserve existing test assertions while formatting them. No behavioral defect found.
- `f08b6732bc4d116e5f250a509549d6e46025da6ad8726d008ae2a68eedd4812b` L1399–1408 — **reviewed**：Hunk 1399: Read new history wire records and every changed test hunk: camelCase/optional receipt metadata and untagged response shapes agree with serialized values; remaining hunks preserve existing test assertions while formatting them. No behavioral defect found.
- `fb1d6f0867b9a346efc139858f8760106ae5032c9748c3970ce7716f2d9477a2` L1478–1496 — **reviewed**：Hunk 1478: Read new history wire records and every changed test hunk: camelCase/optional receipt metadata and untagged response shapes agree with serialized values; remaining hunks preserve existing test assertions while formatting them. No behavioral defect found.
- `4224a99e07fba90a2d480ab638d9e97113901e3b357489d4c72f2757074e02e9` L1504–1513 — **reviewed**：Hunk 1504: Read new history wire records and every changed test hunk: camelCase/optional receipt metadata and untagged response shapes agree with serialized values; remaining hunks preserve existing test assertions while formatting them. No behavioral defect found.
- `3283f53ff4c8ffe705229cf1c3ecc553babceaac6c177bec6a0288218e44eda1` L1533–1552 — **reviewed**：Hunk 1533: Read new history wire records and every changed test hunk: camelCase/optional receipt metadata and untagged response shapes agree with serialized values; remaining hunks preserve existing test assertions while formatting them. No behavioral defect found.
- `82ab5fde2bbf3d862e4cfc8255e98c40f4e34c047f706f3fcb1e67bec6961750` L1570–1579 — **reviewed**：Hunk 1570: Read new history wire records and every changed test hunk: camelCase/optional receipt metadata and untagged response shapes agree with serialized values; remaining hunks preserve existing test assertions while formatting them. No behavioral defect found.

### `crates/protocol/speclink-protocol/src/security.rs`

- `54f94dc2cd3302c869acd98a7eaa4b45e208b0fe41bf7cd1456a67b0429c4060` L1–430 — **reviewed**：Read protocol security.rs 1–430. Checked serde camelCase/status wire encodings, strict deep/attestation inputs, legacy/deep untagged payload distinction, report/decision/request field compatibility and defaults. No definite introduced defect found.

### `crates/protocol/speclink-remote/Cargo.toml`

- `f4ed9caef372aadf3fa7a6db4f386d49febb984e8415020e0562d9493f5f93b7` L5–11 — **reviewed**：Hunk 5: Read Remote Cargo hunk: new Host dependency matches security proof helper use without visible reverse Remote dependency in inspected host manifest.

### `crates/protocol/speclink-remote/src/client.rs`

- `89bf2b448b7e5233a1d648f0dd4dd93411d2bafc20826ebaa1ce3da2ba701f9a` L188–221 — **reviewed**：Read all changed remote client methods: handshake feature checks, validation of path segments and query encoding, versioned review methods, attested verify/archive requests. Hunk at new line 188.
- `45867edd0928b54e610cf001d9724bee6a152af6c96fed5a844ed5a8b057c33f` L253–267 — **reviewed**：Read all changed remote client methods: handshake feature checks, validation of path segments and query encoding, versioned review methods, attested verify/archive requests. Hunk at new line 253.
- `609482017a6f4b8bcbfc862e83c953d8502e127d645b1f5805d39d9170e5a29f` L579–589 — **reviewed**：Read all changed remote client methods: handshake feature checks, validation of path segments and query encoding, versioned review methods, attested verify/archive requests. Hunk at new line 579.
- `44a256fbd58b11695fe67f409af85cdbd3b2f85ff6063fb593b0deb6b64fea61` L748–754 — **reviewed**：Read all changed remote client methods: handshake feature checks, validation of path segments and query encoding, versioned review methods, attested verify/archive requests. Hunk at new line 748.
- `09a601e3921d19676573beed4e5d1b3f37909103eda33f1d300d558f6d6f123d` L757–767 — **reviewed**：Read all changed remote client methods: handshake feature checks, validation of path segments and query encoding, versioned review methods, attested verify/archive requests. Hunk at new line 757.
- `183e0abf1952cbc6e07b10f5fb761383e264da1cbf2f905590919245ce993b5d` L888–897 — **reviewed**：Read all changed remote client methods: handshake feature checks, validation of path segments and query encoding, versioned review methods, attested verify/archive requests. Hunk at new line 888.

### `crates/protocol/speclink-remote/src/lib.rs`

- `c1b4c33b27422ed6b25be6d017ecf5b5ec4968ba54343605ac6cfc03b1c7e19c` L70–77 — **reviewed**：Hunk 70: Read Remote error translation hunks: known new security refusal reasons forward server message; unknown reason preserves message plus status as revised test asserts.
- `3f6a4d7693def4a2def932c6f48cdbb3b04c57c2f5a4b7faa0e8efc3e9c25848` L120–136 — **reviewed**：Hunk 120: Read Remote error translation hunks: known new security refusal reasons forward server message; unknown reason preserves message plus status as revised test asserts.

### `crates/protocol/speclink-remote/tests/it/client_errors.rs`

- `36313f3c17f5a021ddc66b78da4e747fd657b4ef444807d1f2e0957fdaa35c57` L215–224 — **reviewed**：Hunk 215: Read client-error test hunks: unknown reason preserves server diagnostic, reason and HTTP status; no change to transport semantics.
- `c9d896f99206a469a36e00e88dfed8f8ca2c46fe9c4a4c4395aee99d59042fa3` L226–237 — **reviewed**：Hunk 226: Read client-error test hunks: unknown reason preserves server diagnostic, reason and HTTP status; no change to transport semantics.

### `crates/protocol/speclink-remote/tests/it/typed_client.rs`

- `ece8d602a079f00e90c5dbabcafe962feedafdf7cc0f626b90f5a5c6a0bcd64e` L15–28 — **reviewed**：Read both typed-client test hunks: older server handshake stops v2 before POST, existing refusal/unknown reason expectations preserved. Hunk at new line 15.
- `530418b54d320e53b4ccf6bc009ea36746ed8caadaa5c1da35c3e7877a7e3e54` L779–789 — **reviewed**：Read both typed-client test hunks: older server handshake stops v2 before POST, existing refusal/unknown reason expectations preserved. Hunk at new line 779.

### `crates/store/speclink-store-fs/src/lib.rs`

- `5c132b22e61a8d751f7bcb32c9332c858c0a46abc2b25b9c5f9c2c3d5182f1b1` L654–667 — **reviewed**：Read FsStore guard hunk in existing locked commit: compare doc revision to expected before writing staged docs; errors carry actual/expected values. Hunk at new line 654.

### `crates/store/speclink-store-postgres/src/lib.rs`

- `b826f20a536edf5d40e29f1c0c9219780dd45405566d6739ab3a923de6de04e2` L439–452 — **reviewed**：逐段讀取 guard loop 與 370–440 交易上下文：先取得 scope advisory tx lock、檢查 guards，才改文件；CAS 失敗整筆 rollback。

### `crates/store/speclink-store-sqlite/src/lib.rs`

- `8888dd215a5407f6a59572d810c8415e7267655bede8da45076c7b9ddf4fc5c2` L289–307 — **reviewed**：Read SQLite guard hunk with commit context: revision checks run inside existing transaction after operation preconditions and before document writes, returns RevisionConflict on mismatch. Hunk at new line 289.

### `crates/store/speclink-store/src/conformance/mod.rs`

- `58332cf5682a481b4e561015b632ca7b8d7bc21fa7fa225e3f7f94ac870d0710` L468–489 — **reviewed**：新增 conformance CAS read guard 測試：競爭 writer 更新後 guarded commit 必須拒絕且 billing 未部分寫入。

### `crates/store/speclink-store/src/memory.rs`

- `8bfaf76e5e27535ccded61e4a7ba67731d630b8b9b7c9df1c3bc63f7120db447` L424–436 — **reviewed**：Hunk 424: Read memory store guard hunk: all read guards checked against scoped document revisions before journal/application, mismatch returns typed RevisionConflict with expected/actual.

### `crates/store/speclink-store/src/types.rs`

- `f970a0a530cab99d1fe68b503647ed0c638bdb0a08e461b5d934ec02847ea091` L124–138 — **reviewed**：Read ExpectedRevision::matches hunk: Absent maps None, At maps exact revision, matching backend guard use. Hunk at new line 124.

### `crates/store/speclink-store/src/uow.rs`

- `657899efc5cdc6073ad00aed0fcdfbacfa0ba6029ad92451e2269718f6aa0d80` L3–9 — **reviewed**：Hunk 3: Read UnitOfWork hunks: guards initialized on every transaction and exposed separately from write operations, keyed by DocumentId with observed expected revision.
- `6dbf02213b88b58e53a74dbef31ceee7041dad9b28410dd51ccdde9097742a14` L50–56 — **reviewed**：Hunk 50: Read UnitOfWork hunks: guards initialized on every transaction and exposed separately from write operations, keyed by DocumentId with observed expected revision.
- `7649012d31d51351799c47203d13d3fcb250a31391fe65e0bc2177985c57cba1` L59–65 — **reviewed**：Hunk 59: Read UnitOfWork hunks: guards initialized on every transaction and exposed separately from write operations, keyed by DocumentId with observed expected revision.
- `53ea9da010ae4dca3fc9be34665aeda12e26ef21da0a8c296a3f039c8226ae5f` L98–113 — **reviewed**：Hunk 98: Read UnitOfWork hunks: guards initialized on every transaction and exposed separately from write operations, keyed by DocumentId with observed expected revision.

### `docs/configuration.md`

- `2d872db6b92c672099a31a884a720808ba2a9a8da22e4e2238c51f6c533284c0` L165–175 — **reviewed**：Hunk 165: Read configuration doc hunks: builtin tool matrix, directory/name descriptions and custom-descriptor conflict warning match renderer; paths kept distinct for Codex and Antigravity. No runtime defect found in changed content.
- `1e7b533d791a95a9ed7cb6e2474190927e361a45ffa1488db3d588075ba2dbd6` L188–199 — **reviewed**：Hunk 188: Read configuration doc hunks: builtin tool matrix, directory/name descriptions and custom-descriptor conflict warning match renderer; paths kept distinct for Codex and Antigravity. No runtime defect found in changed content.
- `c025ecde5b5303e1711e7ea47450f56b4b401e1fd9cb09731f240032fd442420` L232–238 — **reviewed**：Hunk 232: Read configuration doc hunks: builtin tool matrix, directory/name descriptions and custom-descriptor conflict warning match renderer; paths kept distinct for Codex and Antigravity. No runtime defect found in changed content.

### `docs/configuration.zh-TW.md`

- `a944b76c6d98a1055bf81d98b6f3dc2a35b1582358f4fc77b4bb54d975a0655d` L165–175 — **reviewed**：Read all configuration doc hunks: builtin names/paths and collision guidance match core use; no executable behavior regression identified. External claims not independently browsed in this read-only source review. Hunk at new line 165.
- `4232faf76ed885343fc2dea6f5c12d6bfd19963af38aac09a11da2ae0aeb314c` L188–199 — **reviewed**：Read all configuration doc hunks: builtin names/paths and collision guidance match core use; no executable behavior regression identified. External claims not independently browsed in this read-only source review. Hunk at new line 188.
- `e32472e61cb23cbefbefcfccb1ab254bca59a67085842aa8aa85c7fc5dc7e729` L232–238 — **reviewed**：Read all configuration doc hunks: builtin names/paths and collision guidance match core use; no executable behavior regression identified. External claims not independently browsed in this read-only source review. Hunk at new line 232.

### `docs/development.md`

- `b7d1d1c387b771037cbd51f247a95cf554c310cee17336d18b226c2a78a4fc4a` L156–175 — **reviewed**：Hunk 156: Read development doc hunk: release workflow and shared-root operations distinguish explicit service/client settings, read-only locks and local snapshots; old-writer upgrade limitation documented.

### `docs/development.zh-TW.md`

- `c3db8958cf785998cfd24c816c793c014995b25e7e5ad302e9f70ad12d7787d4` L156–175 — **reviewed**：發版操作文件交代 main commit 打包、release 篩選、共用根與本機資料邊界、舊 writer 停機升級及尚未完成 NAS/Proxy 驗收。

### `docs/getting-started.md`

- `d3794ea293c08b9b16e5743bf79c6da0413ce8bac765773e14c7bff210d2445b` L89–100 — **reviewed**：Hunk 89: Read getting-started hunks: four supported tool names and directories, slash-prefix examples match inspected renderer; generated command examples retain change-name arguments.
- `c1340c984ff5d9f938c320d330c025676e00fbc55c17d3e26592f030ee6245c3` L127–133 — **reviewed**：Hunk 127: Read getting-started hunks: four supported tool names and directories, slash-prefix examples match inspected renderer; generated command examples retain change-name arguments.
- `c375eca75fd34b624badcf33e4de0ea47045a7a7c7f0f6bac5ad1173453a4cc1` L198–204 — **reviewed**：Hunk 198: Read getting-started hunks: four supported tool names and directories, slash-prefix examples match inspected renderer; generated command examples retain change-name arguments.
- `0bbc01098b1e2b2604db78104af7b080854256aad5decd08b304b14967da9855` L265–271 — **reviewed**：Hunk 265: Read getting-started hunks: four supported tool names and directories, slash-prefix examples match inspected renderer; generated command examples retain change-name arguments.

### `docs/getting-started.zh-TW.md`

- `9fd5d61d4fbae02049e031c48563f65756f324599690a280e105beb21f2083e1` L89–100 — **reviewed**：Read all four getting-started additions: four tool values and directories, Antigravity slash-command examples match inspected CLI/core rendering changes. Hunk at new line 89.
- `7898c80a4f7c38ca7950718bc3d3befa194e5275536502ca0adeaf8d6a65ff7a` L127–133 — **reviewed**：Read all four getting-started additions: four tool values and directories, Antigravity slash-command examples match inspected CLI/core rendering changes. Hunk at new line 127.
- `765c590f0090be06b67a50e1aa38e6952ca8fe1d34eb3c740c51e0b33dcc4e14` L198–204 — **reviewed**：Read all four getting-started additions: four tool values and directories, Antigravity slash-command examples match inspected CLI/core rendering changes. Hunk at new line 198.
- `f339561663c8278826001cdfaffe8b16c54f8fb5c54321cfa9d428d73d9fa638` L265–271 — **reviewed**：Read all four getting-started additions: four tool values and directories, Antigravity slash-command examples match inspected CLI/core rendering changes. Hunk at new line 265.

### `docs/product-status.md`

- `17e6787550b504397e81bd6a9d206977b8e50de99bd7fec9e98cf14e81253ed2` L50–56 — **reviewed**：工具能力表新增 Antigravity invocation 與數量。
- `374b4fbce9520ac185d1364ba5a56f85e2c96c3d3684b89f9ca04015ef877a3d` L95–121 — **reviewed**：逐段讀取新工作流程說明：手動更新、共用路徑限制、必要驗收、保留政策、知識 registry、遠端 client-attested；人工驗收仍標 pending。歷史 UI 入口另以相關測試為準。

### `docs/product-status.zh-TW.md`

- `c9ec54575660af91c0917acdef3add88631b8a2f258fc2868ed4593bc9692692` L50–56 — **reviewed**：Hunk 50: Read status doc hunks: added workflow accurately distinguishes device drafts/publication, client-attested source freshness, optional gate, retained checks and pending real corporate/NAS acceptance; tool matrix reflects implementation.
- `36261521ea4e0f67500aced655805917074351c2ede8bc821aa4efbf981d69b0` L95–121 — **reviewed**：Hunk 95: Read status doc hunks: added workflow accurately distinguishes device drafts/publication, client-attested source freshness, optional gate, retained checks and pending real corporate/NAS acceptance; tool matrix reflects implementation.

### `docs/sdk-node.md`

- `fe9d502460215d8d22a8cfbfae49775b08ed35eb7fd232407a9694e92b7fc9d6` L84–90 — **reviewed**：Hunk 84: Read documentation hunks: supported verbs/store atomicity, rendering Antigravity target, archived history optional hook and strict offline security input sizes agree with inspected adapter contracts.
- `d8653f142967a89728c0b1eb819f6163570dc7236e0648cbb815d559e1aa08e3` L119–125 — **reviewed**：Hunk 119: Read documentation hunks: supported verbs/store atomicity, rendering Antigravity target, archived history optional hook and strict offline security input sizes agree with inspected adapter contracts.
- `b5b047bad3c663d6f0b6a179020e249bd5009274fb29f295f7ed80c4ef3fb673` L195–201 — **reviewed**：Hunk 195: Read documentation hunks: supported verbs/store atomicity, rendering Antigravity target, archived history optional hook and strict offline security input sizes agree with inspected adapter contracts.
- `2c06578f16b5f1e9964eec2ad282a8d128957ef90322fdc0b06ce338dc1d1ba9` L210–221 — **reviewed**：Hunk 210: Read documentation hunks: supported verbs/store atomicity, rendering Antigravity target, archived history optional hook and strict offline security input sizes agree with inspected adapter contracts.
- `c5a5e83dc659fd1322a141a3d235395e497cc0695695c1e6f06dd044d56fe9c3` L272–285 — **reviewed**：Hunk 272: Read documentation hunks: supported verbs/store atomicity, rendering Antigravity target, archived history optional hook and strict offline security input sizes agree with inspected adapter contracts.

### `docs/sdk-node.zh-TW.md`

- `ae026fd620d30342b1dab6d343d8a9fecad788b94eab724b0ebc1570d9cf277e` L119–125 — **reviewed**：Read all four documentation hunks against inspected SDK dispatch: commitChecks atomically expects/writes/deletes, target list includes Antigravity, checkout-only security writes and input bounds match implementation. Hunk at new line 119.
- `b9388eb166983cf34e628ba001a719a82d8d76da1bb18825b8bd3155e7d075bc` L195–201 — **reviewed**：Read all four documentation hunks against inspected SDK dispatch: commitChecks atomically expects/writes/deletes, target list includes Antigravity, checkout-only security writes and input bounds match implementation. Hunk at new line 195.
- `97bef16cd38ab993d43bc70f2e558ad9f2caca1eb4134d7cfc30d8ac6c1c4b44` L210–221 — **reviewed**：Read all four documentation hunks against inspected SDK dispatch: commitChecks atomically expects/writes/deletes, target list includes Antigravity, checkout-only security writes and input bounds match implementation. Hunk at new line 210.
- `131f9ba3e4e08ccce5022acc7e88feaac645b23d230f102ecabbd0907f7d698c` L272–285 — **reviewed**：Read all four documentation hunks against inspected SDK dispatch: commitChecks atomically expects/writes/deletes, target list includes Antigravity, checkout-only security writes and input bounds match implementation. Hunk at new line 272.

### `docs/workflow.md`

- `310ee8a309cef462762b4890d919067ecc03c1b75017cc3b70107b10b81e8375` L236–243 — **reviewed**：Read workflow doc hunk: completed record query paths and Host required-security gates described consistently with checked CLI/history. Hunk at new line 236.

### `docs/workflow.zh-TW.md`

- `776709335ccdbff5abf7b5905197be9fb79cbd8281199bd4638f4be192e5a85c` L236–243 — **reviewed**：工作流程文件說明已審查/已驗證展開入口與 UTC 起始檔名，對應新增 UI 測試。

### `integrations/claude-code/skill-evals/evals/security-review-missing-local-knowledge/case.yaml`

- `c44da33e81df2d661252cf0451580611ab2676a3f0e5133db29dac8bf885f6b2` L1–3 — **reviewed**：Read full evaluation YAML: skill is security-review, fixture description and Boolean grader all refer to same missing-local-knowledge case. Hunk at new line 1.

### `integrations/claude-code/skill-evals/evals/security-review-missing-local-knowledge/graders/no-resource-fetch.md`

- `0969e77ab04ee108c4615665281b6b2ef22fc207e6175830bf364704ed940b96` L1–5 — **reviewed**：Read full evaluation grader: transcript forbids resource network/dependency actions, scope/policy tampering and successful report on absent knowledge. Hunk at new line 1.

### `integrations/claude-code/skill-evals/evals/security-review-missing-local-knowledge/graders/reports-missing-knowledge.md`

- `705c259957011cdfce352be5040c095af2eb80b2eeb81ea1b1d937b3e6906b7e` L1–5 — **reviewed**：missing-knowledge grader 只要求停下深度審查，不把缺本機知識誤作 PASS 或接受風險。

### `integrations/claude-code/skill-evals/evals/security-review-missing-local-knowledge/prompt.md`

- `91c9ba397a51869cd0cd66a3c45980dfc254d4acb2e2caf30e2ead49ad08750d` L1–10 — **reviewed**：Read full missing-knowledge prompt: explicitly offline review, no reference download/acceptance weakening, no manufactured findings. Hunk at new line 1.

### `integrations/claude-code/skill-evals/evals/security-review-missing-local-knowledge/scaffold.sh`

- `0cdaadaf50b33d587974f497d4d15917920bd2c45f1b637f566ccc890a7997b6` L1–13 — **reviewed**：Read full 13-line scaffold: shared greeting fixture initialized, required OWASP Top 10 2021 policy appended with spec/code enabled and API boundary disabled, then policy committed; no download/install action appears. Hunk at new line 1.

### `integrations/claude-code/speclink-skills/hooks/groups.ts`

- `9a76361e5b89e2bda72e7a74c89e88b5d76ec9cb44b599006b1ccccb55da6d43` L8–14 — **reviewed**：Read hooks group hunk: security-review added to quality group, existing workflow requirements unchanged. Hunk at new line 8.

### `package-lock.json`

- `1950802a558d5badf823a8fa251f628e7d5f90a07d0d1fdc0e6d5f09c43ddd26` L26–34 — **path-only**：Generated dependency lockfile; excluded from source correctness inspection per brief discipline; path-only, not claimed reviewed.

### `packages/ui/src/__tests__/archivedDetailPage.test.tsx`

- `f3c0fbde4abd35eeed5ec009176d861ac955ad7bdeb197efbfd55d99badd7fae` L687–702 — **reviewed**：封存 UI 測試確認 history 不列一般文件樹、沒有修改/蓋章操作，也不列 omitted placeholder。

### `packages/ui/src/__tests__/baselineSmellHelp.test.tsx`

- `9bdb1e2956f5cc35ed326aa34d9fdd6d254e91da5986aca3e757a32b37ddc3ae` L1–46 — **reviewed**：逐段讀取 1–46：十二名稱從引擎常數取出，中英文說明/例子、鍵盤 Enter/Space/Escape 焦點還原及未知名稱/replace 測試。

### `packages/ui/src/__tests__/changeDetailPage.test.tsx`

- `9af55e2179047300a1550cf45a00e152f22ad006a8170f22b063e0a6e46fbe22` L2238–2321 — **reviewed**：逐段讀取歷史 UI 測試：按需 list/read、HTML 不執行、切換提案丟棄舊回應、按起始時間排序、兩種品質站分開、重試及 refresh 失敗保留已讀內容。

### `packages/ui/src/__tests__/proposalDoc.test.tsx`

- `4f74c7135a4a0c482335ca6fc4d1d1a031091b29dd90cff2601c438b4ce4d509` L115–139 — **reviewed**：逐段讀取 proposal UI 測試：既有散文/清單顯示編號、不改 links/nested list、code/custom section 保留、空章節無空號。

### `packages/ui/src/__tests__/securityReviewPanel.test.tsx`

- `bd6769bcb017455984e664dd3fcea647e30b8d96ac45e1d014ab5502199bd4d2` L10–20 — **reviewed**：Read both SecurityReviewPanel test hunks: export/copy calls carry change, knowledge preparation uses separate action without starting scan; existing progress/project isolation unchanged. Hunk at new line 10.
- `7500947fabcb40dca9d3cf6613bba80cbaf79354f6a73400a0069d9604340b42` L30–40 — **reviewed**：Read both SecurityReviewPanel test hunks: export/copy calls carry change, knowledge preparation uses separate action without starting scan; existing progress/project isolation unchanged. Hunk at new line 30.

### `packages/ui/src/adapter.ts`

- `ed23249380ee98bf37d15fd3f7cd9052535b83a5040da014dd192850ca50e347` L397–403 — **reviewed**：DataSource 新增 optional checkRecords，舊 provider 維持相容。

### `packages/ui/src/checkRecords.ts`

- `1a88d55841726352fc4da931b7e7c44285043504e00d849b125eea79c1947953` L1–6 — **reviewed**：Read complete 6-line check record interface: kinds and adapter list/read receipt-content shapes match inspected hook and tauri data source. Hunk at new line 1.

### `packages/ui/src/components/BaselineSmellList.tsx`

- `39292a0faf5dc35589759c57b10dde14784d43fc1676f734dedc0b58fe93ff33` L1–46 — **reviewed**：Hunk 1: Read BaselineSmellList hunk: known names use own-property dictionary lookup, help button is accessible, translated purpose/example/fallback, replace explanation and baseline notice rendered consistently.

### `packages/ui/src/components/ChangeDetailPage.tsx`

- `0f00c2b6c41c9f5bb0381767ffdd8773b1e7bcd8bad3d194b8e7535724e4df9d` L93–99 — **reviewed**：Read all assigned ChangeDetailPage ranges: adapter prop, quality accordion state scoped to change, XOR toggle leaves opposite station state unchanged, record tabs/document target/body integration. Hunk at new line 93.
- `3d08318a2e1e752cf019527d0e284b6e40256df5fe84379869092f9d5bb02dfd` L106–112 — **reviewed**：Read all assigned ChangeDetailPage ranges: adapter prop, quality accordion state scoped to change, XOR toggle leaves opposite station state unchanged, record tabs/document target/body integration. Hunk at new line 106.
- `ce07175ae56a01a7f4babaf9f4b4edbb11d049838b32e61f2d0470edd4dd7800` L195–212 — **reviewed**：Read all assigned ChangeDetailPage ranges: adapter prop, quality accordion state scoped to change, XOR toggle leaves opposite station state unchanged, record tabs/document target/body integration. Hunk at new line 195.
- `eb01de160a8dc67a95bef182e383015caf56bbbe934057b2cfb24f0605fe844a` L274–280 — **reviewed**：Read all assigned ChangeDetailPage ranges: adapter prop, quality accordion state scoped to change, XOR toggle leaves opposite station state unchanged, record tabs/document target/body integration. Hunk at new line 274.
- `119b48c373eb8ce48c826101be9aca53d91b3e0fa122dbdefd7e73a993f5ca4f` L322–332 — **reviewed**：Read all assigned ChangeDetailPage ranges: adapter prop, quality accordion state scoped to change, XOR toggle leaves opposite station state unchanged, record tabs/document target/body integration. Hunk at new line 322.
- `b2d716781be3cfeb60491ad960f8527a923544e4a3d2cb387113db2819eafe42` L520–526 — **reviewed**：Read all assigned ChangeDetailPage ranges: adapter prop, quality accordion state scoped to change, XOR toggle leaves opposite station state unchanged, record tabs/document target/body integration. Hunk at new line 520.
- `cde07beb1b293dd2508fadb977d3957a6b49f9d91a92da906d4c88c1ba6629df` L773–791 — **reviewed**：Read all assigned ChangeDetailPage ranges: adapter prop, quality accordion state scoped to change, XOR toggle leaves opposite station state unchanged, record tabs/document target/body integration. Hunk at new line 773.
- `a0e51294352d036bf59f9b8938849cbe9c0b3f22d2ccb70d7c17587d12319c2c` L839–845 — **reviewed**：Read all assigned ChangeDetailPage ranges: adapter prop, quality accordion state scoped to change, XOR toggle leaves opposite station state unchanged, record tabs/document target/body integration. Hunk at new line 839.
- `4eb5b5f6b230d1632b19cf1cb31287979cac59aa6e48095d516a0929b043ade4` L849–855 — **reviewed**：Read all assigned ChangeDetailPage ranges: adapter prop, quality accordion state scoped to change, XOR toggle leaves opposite station state unchanged, record tabs/document target/body integration. Hunk at new line 849.

### `packages/ui/src/components/CheckRecords.tsx`

- `6141598f3ff4c8ba3136a27c1bb8751222cd3f3fbeb1ba3713c7116055f77c08` L1–62 — **reviewed**：Read full 62-line hook/component hunk: effect cleanup binds responses to adapter/change, lazy list/record loading, retry and omission handling, first-round UTC ordering and markup. No confirmed direct defect. Hunk at new line 1.

### `packages/ui/src/components/ProposalDoc.tsx`

- `ecb2eea0cf16609dd3fdf9794d698ca670480ac2501a194e28695b73892e79a7` L5–11 — **reviewed**：Read both ProposalDoc hunks with renderer context: itemMarkdown preserves item children for row lists, step lists and alternatives instead of stripping to top text. Hunk at new line 5.
- `db27c2587c82fe16112fa63e7b32fcaa40f7ce76b79c48439dd32ecc9a8cbfaf` L110–125 — **reviewed**：Read both ProposalDoc hunks with renderer context: itemMarkdown preserves item children for row lists, step lists and alternatives instead of stripping to top text. Hunk at new line 110.

### `packages/ui/src/components/SecurityReviewPanel.tsx`

- `b6fb1ae3cf836f0cee4c926dc20a9fffb6aa017bd58f964397d0a721869b40ca` L15–29 — **reviewed**：Read all changed panel lines with whole 71-line component context: prepareKnowledge explicit action, project-generation guard and job matching, report/export calls include proposal change. Hunk at new line 15.
- `bda0615bc72d5651442a0dfb2fd8b9b7225a4730e59ba33c2c66b716469df53c` L35–46 — **reviewed**：Read all changed panel lines with whole 71-line component context: prepareKnowledge explicit action, project-generation guard and job matching, report/export calls include proposal change. Hunk at new line 35.
- `74e0f2fe8ad912aa95776056c9610054acd2cacd178a075657ea5f4d72c94ebf` L52–61 — **reviewed**：Read all changed panel lines with whole 71-line component context: prepareKnowledge explicit action, project-generation guard and job matching, report/export calls include proposal change. Hunk at new line 52.

### `packages/ui/src/i18n.tsx`

- `3c772eb6091766b062884f540bfb4fe0b1ee1344dbcb229266867c4141e2144b` L14–47 — **reviewed**：Hunk 14: Read every new Chinese/English dictionary entry: baseline explanation, check history outcomes, reauthentication and device-draft wording use matching keys and interpolation variables.
- `c440d3198df417237b1a154d01af160d78bc8e56ba584f67719ff5ddceb62236` L56–84 — **reviewed**：Hunk 56: Read every new Chinese/English dictionary entry: baseline explanation, check history outcomes, reauthentication and device-draft wording use matching keys and interpolation variables.
- `d32b6bfbdac116aefed333c3e9364e92cb2ceace6b2289af3caaff828304188c` L561–594 — **reviewed**：Hunk 561: Read every new Chinese/English dictionary entry: baseline explanation, check history outcomes, reauthentication and device-draft wording use matching keys and interpolation variables.
- `22b2b1e14a5af070be029e598056ee71a26f33d152989e954112d3015e9316c8` L603–631 — **reviewed**：Hunk 603: Read every new Chinese/English dictionary entry: baseline explanation, check history outcomes, reauthentication and device-draft wording use matching keys and interpolation variables.

### `packages/ui/src/index.ts`

- `5c097d728d0ce2c53cf5c788f055df4a1a92b102298e8cd16ebbd048ef1035a8` L331–335 — **reviewed**：Hunk 331: Read UI export hunk: check-record adapter types exported to desktop through shared package boundary.

### `packages/ui/src/security.ts`

- `8fd1e3b64ebb3761748b1a5931faaf9d0c47f9ba69776daf235b2019ae6ebcaf` L13–29 — **reviewed**：Read security adapter interface hunk: optional prepareKnowledge and optional change argument flow matches panel callers; pendingPublication response optional. Hunk at new line 13.

## Verify 逐項對照

前輪已核對 17 份規格的 48 個需求（含 1 個移除需求）、141 個情境、7 個範例。下列保留每一項名稱、原文行號與該能力使用的實作／測試群組；本輪不把這些對照當作新的測試執行結果。

外部限制：真實 NAS／UNC 多機互斥、模型對五語言專案的審查品質及 GitHub 即時服務，均不由本機程式閱讀證明。12.3 依使用者確認已完成；公司 Proxy 的版本／環境／operationId 等詳細資訊未齊全，本輪未獨立重現。

### change-lifecycle

- 規格：`openspec/changes/desktop-workflow-and-offline-security-review/specs/change-lifecycle/spec.md`
- 實作群組：crates/engine/speclink-core/src/lifecycle/archive.rs; crates/engine/speclink-fs/src/archive_checks.rs
- 測試群組：crates/engine/speclink-core/src/lifecycle/archive/tests.rs; crates/adapters/speclink-cli/tests/it/archive_readiness_gate.rs

- L3 `requirement`：必要安全驗收的封存判定 — 前輪已對照
- L7 `scenario`：沒跑驗證也不能略過必要安全檢查 — 前輪已對照
- L12 `scenario`：參考模式與明確接受 — 前輪已對照
- L17 `requirement`：封存時按類別保留完成紀錄 — 前輪已對照
- L23 `scenario`：三類保留的八種組合 — 前輪已對照
- L28 `scenario`：staging或提交失敗 — 前輪已對照

### client-protocol

- 規格：`openspec/changes/desktop-workflow-and-offline-security-review/specs/client-protocol/spec.md`
- 實作群組：crates/protocol/speclink-protocol/src/{command,query,error,security}.rs; crates/protocol/speclink-remote/src/client.rs
- 測試群組：crates/protocol/speclink-remote/tests/it/{typed_client,client_errors}.rs; crates/host/speclink-server/tests/it/api/{review_api,write_guard}.rs

- L3 `requirement`：檢查歷史與安全驗收的共用契約 — 前輪已對照
- L11 `scenario`：Local與Remote型別一致 — 前輪已對照
- L16 `scenario`：舊server與衝突 — 前輪已對照
- L23 `requirement`：標準 error reason registry — 前輪已對照
- L27 `scenario`：reason 對映沿用現行訊息 — 前輪已對照
- L32 `scenario`：未知 reason 不崩潰 — 前輪已對照

### desktop-app

- 規格：`openspec/changes/desktop-workflow-and-offline-security-review/specs/desktop-app/spec.md`
- 實作群組：apps/desktop/src/{store,tabs,App}.tsx/ts; apps/desktop/src/components/ProjectRail.tsx; apps/desktop/src/views/settings/UpdatesPage.tsx; packages/ui/src/components/{ChangeDetailPage,CheckRecords,ProposalDoc}.tsx
- 測試群組：apps/desktop/src/__tests__/{projectRail,tabs,store,updatesPage,updater,aboutPage}.test.ts/tsx; packages/ui/src/__tests__/{changeDetailPage,archivedDetailPage,proposalDoc}.test.tsx

- L3 `requirement`：本分支更新與原作者版本 — 前輪已對照
- L11 `scenario`：本分支有新版而原作者版號更高 — 前輪已對照
- L16 `scenario`：只有未簽章安裝檔 — 前輪已對照
- L21 `scenario`：原作者查詢失敗且本分支正常 — 前輪已對照
- L26 `requirement`：專案貢獻與原作署名 — 前輪已對照
- L30 `scenario`：查看本分支貢獻說明 — 前輪已對照
- L35 `requirement`：左側專案排序與裝置持久化 — 前輪已對照
- L39 `scenario`：本機與遠端混排後重開 — 前輪已對照
- L44 `scenario`：取消與保存失敗 — 前輪已對照
- L49 `requirement`：更新頁內附資訊不依賴網路成功 — 前輪已對照
- L53 `scenario`：更新元件缺席 — 前輪已對照
- L58 `scenario`：Proxy拒絕與背景失敗 — 前輪已對照
- L63 `requirement`：文件區只顯示進行中的審查與驗證 — 前輪已對照
- L67 `scenario`：文件區只顯示進行中的工單 — 前輪已對照
- L72 `scenario`：蓋章後從品質關卡查詢歷史 — 前輪已對照
- L79 `requirement`：提案與設計章節以中文標籤呈現 — 前輪已對照
- L89 `scenario`：提案模板章節成中文標籤 — 前輪已對照
- L94 `example`：章節對照與畫法 — 前輪已對照
- L111 `scenario`：章節標題為 16px 粗體 — 前輪已對照
- L116 `scenario`：白名單外章節照排 — 前輪已對照
- L121 `scenario`：無二級標題整篇退回 — 前輪已對照
- L126 `scenario`：替代方案與影響段的逐項降級 — 前輪已對照
- L131 `example`：影響段不符形狀時當一般列 — 前輪已對照
- L137 `scenario`：移除組的路徑籤沒有顯示鈕 — 前輪已對照
- L142 `scenario`：能力籤跳到規格與改寫提示框 — 前輪已對照
- L147 `scenario`：提案的籤槽與數字磚 — 前輪已對照
- L152 `scenario`：設計的決策卡與數字磚 — 前輪已對照
- L164 `scenario`：不同專案的散文提案預設列式 — 前輪已對照
- L171 `requirement`：品質關卡展開審查與驗證紀錄 — 前輪已對照
- L175 `scenario`：展開並查看審查紀錄 — 前輪已對照
- L180 `scenario`：展開並查看驗證紀錄 — 前輪已對照
- L185 `scenario`：空清單、失敗與切換變更 — 前輪已對照

### desktop-config

- 規格：`openspec/changes/desktop-workflow-and-offline-security-review/specs/desktop-config/spec.md`
- 實作群組：apps/desktop/src/views/settings/SecurityDataPage.tsx; apps/desktop/src/views/project-settings/{WorkflowPage,ReviewRulesPage,SpeclinkPage,useProjectSettings}.tsx/ts; apps/desktop/src/builtinTools.ts
- 測試群組：apps/desktop/src/__tests__/{securityDataPage,projectSettingsView,workspaceChooser}.test.tsx; packages/ui/src/__tests__/baselineSmellHelp.test.tsx

- L3 `requirement`：安全標準資料存放路徑設定 — 前輪已對照
- L9 `scenario`：本機預設切換共用目錄 — 前輪已對照
- L14 `scenario`：共用位置唯讀或離線 — 前輪已對照
- L19 `scenario`：儲存失敗或工作未完成 — 前輪已對照
- L24 `requirement`：預設十二條的問號說明 — 前輪已對照
- L28 `scenario`：十二條逐一查看 — 前輪已對照
- L33 `scenario`：整份換掉仍能理解預設清單 — 前輪已對照
- L38 `requirement`：納入安全驗收與封存紀錄的設定 — 前輪已對照
- L44 `scenario`：參考模式改驗收模式 — 前輪已對照
- L49 `scenario`：封存保留設定與遠端衝突 — 前輪已對照
- L55 `requirement`：AI 工具名稱顯示技能目錄 — 前輪已對照
- L59 `scenario`：所有內建工具顯示目錄 — 前輪已對照
- L64 `scenario`：儲存新增與取消 Antigravity — 前輪已對照
- L69 `scenario`：初始化與 checkout 選項一致 — 前輪已對照

### desktop-network-diagnostics

- 規格：`openspec/changes/desktop-workflow-and-offline-security-review/specs/desktop-network-diagnostics/spec.md`
- 實作群組：apps/desktop/core/src/{updater,network}.rs; crates/host/speclink-host/src/network.rs; apps/desktop/src-tauri/src/updater_commands.rs
- 測試群組：apps/desktop/src/__tests__/{updater,updatesPage}.test.ts/tsx; apps/desktop/core/src/testdata/README.md; evidence/company-proxy.md

- L3 `requirement`：公司 Proxy 更新失敗的定位與呈現 — 前輪已對照
- L7 `scenario`：模擬 CONNECT 與407 — 前輪已對照
- L12 `scenario`：無可得HTTP碼的TLS失敗 — 前輪已對照
- L17 `requirement`：公司 Proxy 打包後驗收 — 前輪已對照
- L23 `scenario`：開發環境無公司安裝條件 — 前輪已對照
- L28 `scenario`：打包後進入公司測試 — 前輪已對照

### node-sdk

- 規格：`openspec/changes/desktop-workflow-and-offline-security-review/specs/node-sdk/spec.md`
- 實作群組：crates/adapters/speclink-node/src/render.rs; crates/adapters/speclink-node/index.d.ts
- 測試群組：crates/adapters/speclink-node/__test__/{render,types,dispatch-contract}.spec.ts; crates/adapters/speclink-node/__test__/types/render-api.ts

- L3 `requirement`：渲染 API — 前輪已對照
- L7 `scenario`：中性 tool-call 渲染 — 前輪已對照
- L11 `scenario`：與 CLI 生成一致 — 前輪已對照
- L15 `scenario`：copilot target 與 CLI 生成一致 — 前輪已對照
- L19 `scenario`：未知 target 被拒 — 前輪已對照
- L23 `scenario`：Copilot 原生渲染與 CLI 同源 — 前輪已對照
- L28 `scenario`：Antigravity 原生渲染與 CLI 同源 — 前輪已對照
- L33 `scenario`：公開 TypeScript 宣告可供使用者編譯 — 前輪已對照
- L38 `scenario`：選填參數與預設值 — 前輪已對照
- L43 `scenario`：未知 invocation 被拒 — 前輪已對照
- L50 `requirement`：渲染 API 的 worktree 軸 — 依規格移除

### propose-skill

- 規格：`openspec/changes/desktop-workflow-and-offline-security-review/specs/propose-skill/spec.md`
- 實作群組：crates/engine/speclink-core/assets/schema/spec-driven/proposal.template.md; crates/engine/speclink-core/assets/skills/propose.md
- 測試群組：crates/engine/speclink-core/tests/it/render_golden.rs; packages/ui/src/__tests__/proposalDoc.test.tsx

- L3 `requirement`：提案內容預設以條列起草 — 前輪已對照
- L7 `scenario`：新專案採用相同條列預設 — 前輪已對照

### review-skill

- 規格：`openspec/changes/desktop-workflow-and-offline-security-review/specs/review-skill/spec.md`
- 實作群組：crates/engine/speclink-core/assets/skills/review.md; crates/engine/speclink-core/src/quality/{history,station}.rs
- 測試群組：crates/engine/speclink-core/tests/it/{render_golden,skill_authoring}.rs; crates/adapters/speclink-cli/tests/it/review_verbs.rs

- L3 `requirement`：審查完成紀錄的技能交棒 — 前輪已對照
- L7 `scenario`：蓋章後提供可查詢紀錄 — 前輪已對照
- L12 `scenario`：保存失敗 — 前輪已對照

### review-station

- 規格：`openspec/changes/desktop-workflow-and-offline-security-review/specs/review-station/spec.md`
- 實作群組：crates/engine/speclink-core/src/quality/{history,station}.rs; crates/engine/speclink-fs/src/{checks,archive_checks}.rs; crates/host/speclink-host/src/history.rs
- 測試群組：crates/engine/speclink-core/src/quality/station/tests.rs; crates/adapters/speclink-cli/tests/it/review_verbs.rs; crates/adapters/speclink-node/__test__/review-verify-verbs.spec.ts

- L3 `requirement`：蓋章守門與蓋章效果 — 前輪已對照
- L8 `scenario`：寫碼任務未全完成即拒絕 — 前輪已對照
- L13 `scenario`：僅餘手動任務可蓋章 — 前輪已對照
- L18 `scenario`：末輪有未解 findings 且未帶 --accept — 前輪已對照
- L23 `scenario`：僅 SUGGESTION 的末輪乾淨蓋章 — 前輪已對照
- L28 `scenario`：帶保留蓋章 — 前輪已對照
- L33 `scenario`：乾淨蓋章 — 前輪已對照
- L38 `example`：蓋章寫入的任務錨 — 前輪已對照
- L44 `scenario`：歷史寫入失敗保全工單 — 前輪已對照
- L49 `scenario`：審查紀錄檔名保留開始時間 — 前輪已對照
- L57 `requirement`：檢查紀錄鎖與封存搬移 — 前輪已對照
- L61 `scenario`：Windows 封存與競爭寫入 — 前輪已對照
- L66 `requirement`：審查完成紀錄查詢 — 前輪已對照
- L70 `scenario`：查詢已蓋章紀錄的人眼與JSON — 前輪已對照
- L75 `scenario`：缺歷史與非法ID — 前輪已對照

### security-review

- 規格：`openspec/changes/desktop-workflow-and-offline-security-review/specs/security-review/spec.md`
- 實作群組：crates/engine/speclink-core/assets/skills/security-review.md; crates/engine/speclink-core/src/security/acceptance.rs; crates/host/speclink-host/src/security/{deep,remote}.rs; crates/adapters/speclink-cli/src/verbs/security.rs
- 測試群組：crates/adapters/speclink-cli/tests/it/security.rs; crates/host/speclink-host/tests/security_remote.rs; integrations/claude-code/skill-evals/evals/security-review-missing-local-knowledge/

- L3 `requirement`：保留基本掃描並提供深度安全審查技能 — 前輪已對照
- L9 `scenario`：只選2025且保留基本掃描 — 前輪已對照
- L14 `scenario`：停用或缺件 — 前輪已對照
- L19 `requirement`：五種語言的架構與資料流深度審查 — 前輪已對照
- L23 `scenario`：跨帳戶ID的授權追查 — 前輪已對照
- L28 `scenario`：無法完成的路徑 — 前輪已對照
- L33 `requirement`：深度審查提交與查詢命令 — 前輪已對照
- L41 `scenario`：提交及查詢人眼與JSON — 前輪已對照
- L46 `scenario`：越界引用與Remote衝突 — 前輪已對照
- L51 `requirement`：安全驗收的等級完整性與新鮮度判定 — 前輪已對照
- L57 `scenario`：風險等級與必要資料矩陣 — 前輪已對照
- L62 `scenario`：程式碼或政策變更 — 前輪已對照
- L67 `example`：納入驗收的具體矩陣 — 前輪已對照
- L79 `requirement`：安全風險明確接受命令 — 前輪已對照
- L83 `scenario`：有理由的接受與空理由 — 前輪已對照

### security-review-report

- 規格：`openspec/changes/desktop-workflow-and-offline-security-review/specs/security-review-report/spec.md`
- 實作群組：crates/engine/speclink-core/src/security/report.rs; crates/engine/speclink-core/src/security/report/deep.rs; crates/engine/speclink-fs/src/security/reports.rs; crates/host/speclink-host/src/security/deep.rs
- 測試群組：crates/adapters/speclink-cli/tests/it/security.rs; crates/host/speclink-host/tests/security_remote.rs; packages/ui/src/__tests__/securityReviewPanel.test.tsx

- L3 `requirement`：本機報告保存與查詢 — 前輪已對照
- L8 `scenario`：保存中斷與錯誤 ID — 前輪已對照
- L13 `scenario`：封存後報告仍可查 — 前輪已對照
- L18 `scenario`：明確目標的提案內報告 — 前輪已對照
- L24 `requirement`：版本化 JSON 報告模型 — 前輪已對照
- L29 `scenario`：有 dirty 檔與無 Git — 前輪已對照
- L34 `requirement`：SDD 提示與 AI 邊界 — 前輪已對照
- L39 `scenario`：沒有 AI 的離線旅程 — 前輪已對照
- L44 `scenario`：外部資料帶指令 — 前輪已對照
- L49 `requirement`：報告輸出命令與語言 — 前輪已對照
- L54 `scenario`：兩種格式一致 — 前輪已對照
- L59 `scenario`：介面與 Agent 語言 — 前輪已對照
- L66 `requirement`：深度報告九項內容與風險接受紀錄 — 前輪已對照
- L72 `scenario`：九項證據與機密遮罩 — 前輪已對照
- L77 `scenario`：接受理由可回查且不改原報告 — 前輪已對照

### security-standard-data

- 規格：`openspec/changes/desktop-workflow-and-offline-security-review/specs/security-standard-data/spec.md`
- 實作群組：crates/engine/speclink-core/assets/security/spec_registry.json; crates/engine/speclink-fs/src/security/{knowledge,prepared}.rs; crates/host/speclink-host/src/{security_data,security_paths}.rs
- 測試群組：crates/engine/speclink-fs/src/security_packages.rs; crates/engine/speclink-fs/tests/store_fs.rs; crates/adapters/speclink-cli/tests/it/security.rs; apps/desktop/src/__tests__/securityDataPage.test.tsx

- L3 `requirement`：應用程式資料目錄與離線版本清單 — 前輪已對照
- L10 `scenario`：Desktop 安裝後 CLI 離線可用 — 前輪已對照
- L15 `scenario`：缺裝置路徑與缺資料 — 前輪已對照
- L20 `scenario`：兩台電腦共用同一資料 — 前輪已對照
- L25 `scenario`：共用寫入競爭與失敗保舊 — 前輪已對照
- L30 `scenario`：共用位置不可用或唯讀 — 前輪已對照
- L37 `requirement`：深度審查完整離線資料包 — 前輪已對照
- L43 `scenario`：下載後斷網與交換完整包 — 前輪已對照
- L48 `scenario`：浮動來源與舊包缺件 — 前輪已對照
- L53 `requirement`：本機知識庫準備命令 — 前輪已對照
- L59 `scenario`：本機準備及人眼JSON — 前輪已對照
- L64 `scenario`：同名目錄或必要檔被改 — 前輪已對照

### ui-copy-vocabulary

- 規格：`openspec/changes/desktop-workflow-and-offline-security-review/specs/ui-copy-vocabulary/spec.md`
- 實作群組：openspec/LANGUAGE.md; packages/ui/src/i18n.tsx; apps/desktop/src/i18n/messages.ts
- 測試群組：apps/desktop/src/__tests__/projectSettingsView.test.tsx; packages/ui/src/__tests__/securityReviewPanel.test.tsx

- L3 `requirement`：安全驗收與檢查紀錄的用詞 — 前輪已對照
- L7 `scenario`：參考與驗收模式 — 前輪已對照
- L12 `scenario`：關閉封存完整紀錄 — 前輪已對照

### verify-skill

- 規格：`openspec/changes/desktop-workflow-and-offline-security-review/specs/verify-skill/spec.md`
- 實作群組：crates/engine/speclink-core/assets/skills/verify.md; crates/engine/speclink-core/src/security/acceptance.rs
- 測試群組：crates/engine/speclink-core/tests/it/{render_golden,skill_authoring}.rs; crates/adapters/speclink-cli/tests/it/{verify_verbs,security,archive_readiness_gate}.rs

- L3 `requirement`：Verify 報告中的獨立初步安全資訊 — 前輪已對照
- L10 `scenario`：初步疑似風險不改驗證工單 — 前輪已對照
- L15 `scenario`：缺資料或停用政策 — 前輪已對照
- L20 `scenario`：既有守門仍有效 — 前輪已對照
- L25 `scenario`：必要安全條件未滿 — 前輪已對照
- L32 `requirement`：驗證完成紀錄的技能交棒 — 前輪已對照
- L36 `scenario`：成功與保存失敗的回報 — 前輪已對照

### verify-station

- 規格：`openspec/changes/desktop-workflow-and-offline-security-review/specs/verify-station/spec.md`
- 實作群組：crates/engine/speclink-core/src/quality/{history,station}.rs; crates/adapters/speclink-cli/src/verbs/station.rs
- 測試群組：crates/engine/speclink-core/src/quality/station/tests.rs; crates/adapters/speclink-cli/tests/it/{verify_verbs,security}.rs

- L3 `requirement`：驗證蓋章守門與蓋章效果 — 前輪已對照
- L8 `scenario`：寫碼任務未完成即拒絕蓋章 — 前輪已對照
- L13 `scenario`：僅餘手動任務可蓋章 — 前輪已對照
- L18 `scenario`：末輪有未解 findings 且未帶 --accept — 前輪已對照
- L23 `scenario`：僅 SUGGESTION 的末輪乾淨蓋章 — 前輪已對照
- L28 `scenario`：乾淨蓋章 — 前輪已對照
- L33 `example`：蓋章寫入的任務錨 — 前輪已對照
- L39 `scenario`：必修驗收不能用一般接受略過 — 前輪已對照
- L45 `scenario`：歷史寫入失敗保全工單 — 前輪已對照
- L50 `scenario`：驗證紀錄檔名保留開始時間 — 前輪已對照
- L58 `requirement`：驗證完成紀錄查詢 — 前輪已對照
- L62 `scenario`：查詢已蓋章紀錄的人眼與JSON — 前輪已對照
- L67 `scenario`：缺歷史與非法ID — 前輪已對照

### workflow-config

- 規格：`openspec/changes/desktop-workflow-and-offline-security-review/specs/workflow-config/spec.md`
- 實作群組：crates/engine/speclink-core/src/security/policy.rs; crates/engine/speclink-core/src/workspace/config.rs; crates/adapters/speclink-cli/src/verbs/config.rs
- 測試群組：crates/engine/speclink-core/src/workspace/config/tests.rs; crates/adapters/speclink-cli/tests/it/security.rs; apps/desktop/src/__tests__/projectSettingsView.test.tsx

- L3 `requirement`：安全政策區塊預設與獨立選項 — 前輪已對照
- L8 `scenario`：各標準互不影響 — 前輪已對照
- L13 `scenario`：非法安全政策 — 前輪已對照
- L18 `scenario`：舊政策與停用後保值 — 前輪已對照
- L24 `requirement`：安全政策部分更新與保值 — 前輪已對照
- L29 `scenario`：修改一個葉節點保留未知內容 — 前輪已對照
- L34 `scenario`：dry-run 與失敗原檔不變 — 前輪已對照
- L39 `scenario`：Remote 並行修改 — 前輪已對照
- L46 `requirement`：檢查紀錄封存政策 — 前輪已對照
- L50 `scenario`：只修改安全紀錄的封存選項 — 前輪已對照
- L55 `scenario`：dry-run與非法值 — 前輪已對照

### workspace-tools

- 規格：`openspec/changes/desktop-workflow-and-offline-security-review/specs/workspace-tools/spec.md`
- 實作群組：crates/engine/speclink-core/src/workspace/{config,init,skills}.rs; crates/adapters/speclink-cli/src/verbs/init.rs; apps/desktop/src/builtinTools.ts
- 測試群組：crates/engine/speclink-core/src/workspace/init/tests.rs; crates/adapters/speclink-cli/tests/it/{init_tools,remote_verb_parity}.rs; crates/adapters/speclink-node/__test__/render.spec.ts

- L3 `requirement`：tools 自訂描述子的接受與驗證 — 前輪已對照
- L7 `scenario`：合法描述子生成對應工具檔 — 前輪已對照
- L12 `scenario`：名稱與內建工具衝突被拒 — 前輪已對照
- L17 `scenario`：路徑逸出專案根被拒 — 前輪已對照
- L22 `scenario`：殘留 instructions_file 欄位得棄用提示 — 前輪已對照
- L27 `scenario`：結尾分隔符於驗證時削去 — 前輪已對照
- L32 `scenario`：削去後等同專案根的 skills_dir 被拒 — 前輪已對照
- L37 `scenario`：與內建工具 skills 目錄相撞被拒 — 前輪已對照
- L42 `example`：skills_dir 的驗證結果 — 前輪已對照
- L59 `scenario`：名為 copilot 或指向 Copilot 目錄的既有描述子升級後被拒 — 前輪已對照
- L66 `requirement`：init 內建 Agent 工具選擇 — 前輪已對照
- L70 `scenario`：filesystem init 顯式選擇 Codex — 前輪已對照
- L75 `scenario`：Remote init 顯式選擇兩個工具 — 前輪已對照
- L80 `scenario`：互動終端選擇 Claude 與 Codex — 前輪已對照
- L85 `scenario`：互動終端不得提交空選集 — 前輪已對照
- L90 `scenario`：非互動 init 缺少 tools 零寫入失敗 — 前輪已對照
- L95 `scenario`：空或未知的顯式 tools 被拒 — 前輪已對照
- L100 `scenario`：no-color 不改變工具選擇語意 — 前輪已對照
- L105 `scenario`：filesystem init 顯式選擇 Copilot — 前輪已對照
- L110 `scenario`：顯式選擇三個工具 — 前輪已對照
- L115 `example`：--tools 值的解析 — 前輪已對照
- L128 `requirement`：Antigravity 技能生成與獨立清理 — 前輪已對照
- L132 `scenario`：同時使用 Codex 與 Antigravity — 前輪已對照
- L137 `scenario`：取消 Antigravity 保留使用者檔案 — 前輪已對照
- L142 `scenario`：舊自訂描述子與新內建工具衝突 — 前輪已對照
