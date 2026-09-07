## Context

討論記錄的自動封存有兩個觸發點，都在 speclink-core：
- 變更封存時的隨行封存：archive 的來源討論過濾器，對變更 meta 的 from_discussion 清單逐一判斷「無其他在途變更引用」且「Conclusion 已寫入內文」，兩者成立即呼叫 archive_discussion。
- conclude 的閉環：結論寫入後，若 promoted_to 非空且無在途變更引用，順手封存記錄。

三條轉出路徑（discuss promote、new change --from-discussion、discuss seal）都經 crates/speclink-core/src/discuss.rs 的 mark_promoted 累加 promoted_to；link 依規格不改討論記錄，由 seal 補標。

分期立案（先立刀 A、封存後再回同一討論轉出刀 B）的意圖只存在於結論的 Deferred 散文。刀 A 封存時兩個條件都成立，討論被收進封存區；promote 對封存記錄拒絕，只能手動搬檔。來源討論 discussion-planned-spinout-hold 已否決：結構化計數、獨立 hold／release 動詞、先立刀 B 骨架、unarchive 救援動詞、只改技能文字。

## Goals / Non-Goals

**Goals:**

- 討論結論可帶一個機器可讀的「保留在途」訊號；兩個自動封存點都守它。
- 訊號由下一次轉出自動清除，之後生命週期與今天完全相同。
- 不帶旗標時，CLI 人眼與 --json 輸出、既有記錄的封存判斷逐位元不變。
- local 與 remote 路徑同語意，實作落點唯一（引擎）。
- 技能文字明說何時帶旗標。

**Non-Goals:**

- desktop 看板不新增「保留中」標示，資料鏈（DiscussionInfo、server 列表回應、ui adapter）不加欄位——討論留在途本身就是看板可觀察的結果（它留在「已轉出」收合列而非消失）。
- 不提供 unarchive 動詞；本案 improve-workspace-sync 的救援是手動搬檔。
- 不改 link 的「討論記錄逐位元不變」承諾；link 不清旗標。
- 不偵測舊 server 忽略 hold 欄位的情況（見 Risks）。

## Decisions

**D1. 訊號落在討論記錄 frontmatter，鍵名 `hold`，值 `true`，寫成獨立一行。**
與 promoted_to、kind、board_rank 同一層——都是 frontmatter_value 讀取的純量。不進 DiscussionInfo（沿 remote-read-parity design D2：discuss list --json 逐位元不變）。替代方案「寫在 Conclusion 內文的結構化行」被否決：結論內文是給人讀的，且 conclude 的內容跳脫機制會動到它。

**D2. 唯一寫入點是 conclude；唯一清除點是 mark_promoted。**
- conclude 帶 hold 時，結論內文與 hold 行在同一次 write_live_discussion 落盤（先組好文字再寫），不留半套。hold 行插在 frontmatter 最後一行（結尾 `---` 之前）；記錄已有 hold 行時不重複。不帶 hold 的 conclude 移除既有 hold 行（結論改寫即重述意圖）。
- mark_promoted 累加 promoted_to 時一併移除 hold 行——promote、new change --from-discussion、seal 三條路徑都經此函式，一處覆蓋。替代方案「在三個呼叫端各清一次」是重複；「link 也清」違反 link 不改記錄的既有規格。
- 移除以「刪掉 `hold: true` 這一行」實作，不依賴它在 frontmatter 的位置。

**D3. 兩個自動封存點的判準各加一條「記錄未帶 hold」。**
引擎提供 held_in(text) 與 discussion_held(store, slug)（與 concluded_in／discussion_concluded 同形）。隨行封存過濾器：still_referenced 或未結論或 held 任一成立即跳過。conclude 閉環：閉環條件多一條「本次寫入後記錄未帶 hold」——用寫入後的文字判斷，帶 --hold 的 conclude 必然不閉環。手動 speclink discuss archive 不看 hold：明示動詞是放棄後續刀的出口。

**D4. 命令與 outcome。**
Command::DiscussConclude 增 `hold: bool`；DiscussConcludeOutcome 增 `held: bool`（本次寫入後記錄是否帶 hold）。CLI 人眼輸出在既有行之後多一行 `  Held live (a later spin-out is planned)`；--json 僅 held 為 true 時增 `held: true` 鍵。held 為 false 時兩條輸出逐位元不變。

**D5. wire 與 server。**
ConcludeDiscussionRequest 增 `hold: bool`（serde default，false 時不序列化）；ConcludeDiscussionResponse 增 `held: bool`（serde default，false 時不序列化）——與 auto_archived 同一模式。server 結論端點把 req.hold 轉傳引擎命令、回填 held。speclink-remote 的 discussion_conclude 增 hold 參數；CLI remote 分支轉傳旗標並沿用同一 render 函式。實作落點唯一：判準、寫入、清除全在引擎，CLI 與 server 不重複。

**D6. 技能文字。**
- discuss.md 中途轉出段第 1 步：在「the discussion is archived automatically when the last of its changes is archived and its conclusion is written」之後補一句——結論規劃之後回同一記錄再轉出時，conclude 帶 --hold，記錄留在途直到下一次轉出清掉旗標；否則後續刀走新討論。conclude 指令範例區塊上方補一行說明 --hold 的用途。
- improve.md 扇出段同一句。
- ASSET_VERSION v1.30.0 → v1.31.0；依序再生 golden 快照、assets.lock，再以 speclink update 再生 .claude/skills/ 與 .agents/skills/ 全部受管 SKILL.md。

## Implementation Contract

**行為**
- `speclink discuss conclude <slug> --hold --stdin`：結論寫入、記錄 frontmatter 出現 `hold: true` 行；status 轉換規則不變（open → concluded；promoted 保持）。restale 蓋章照舊。閉環封存不觸發。人眼 stdout 於既有行後多一行；--json 含 `held: true`。exit code 0。
- 不帶 --hold 的 conclude：既有 hold 行被移除；其餘行為、人眼與 --json 輸出與改動前逐位元一致。
- 帶 hold 的討論，其唯一轉出變更封存時：變更照常封存（exit code 0），討論維持於 openspec/discussions/，封存輸出的隨行封存清單不含它。
- discuss promote／new change --from-discussion／discuss seal 對帶 hold 的討論成功時：promoted_to 累加、hold 行消失。之後該變更封存時討論隨行封存（既有行為）。
- speclink discuss archive 對帶 hold 的討論：照常封存。
- remote 模式 `--hold` 經 wire 生效，可觀察行為與本機一致。

**介面／資料形狀**
- CLI 旗標：`--hold`（布林，僅 conclude 子指令）。
- frontmatter：`hold: true`（獨立一行）。
- 引擎：Command::DiscussConclude { slug, content, hold }；DiscussConcludeOutcome { restale_flagged, auto_archived, closing_error, held }；discuss::held_in(&str) -> bool；discuss::discussion_held(&dyn Store, &str) -> bool。
- --json：`{ "slug", "status": "concluded", "restaleFlagged"?, "autoArchived"?, "held"? }`，held 僅 true 時出現。
- wire：ConcludeDiscussionRequest { content, hold }；ConcludeDiscussionResponse { restaleFlagged, autoArchived?, held? }。

**失敗模式**
- conclude 的寫入失敗：與今天相同，結論與 hold 都未落盤。
- 閉環封存步失敗：不可能在帶 hold 時發生（閉環不觸發）；不帶 hold 時沿既有 closing_error 語意。
- 新 client 帶 hold 打舊 server：欄位被忽略，回應無 held 鍵，CLI 不印保留行——使用者從輸出可看出旗標未生效。刻意接受，不加偵測。

**驗收**
- speclink-core 單元測試：conclude 帶 hold 寫入 hold 行且不閉環；不帶 hold 清除既有 hold 行；mark_promoted 清除 hold 行；archive 的隨行封存過濾器對帶 hold 記錄跳過；discuss archive 無視 hold。
- speclink-cli 整合測試：--hold 的人眼與 --json 輸出；不帶 --hold 的輸出與基準逐位元一致；帶 hold 討論的唯一變更封存後記錄留在途；remote 分支 POST body 含 `"hold":true`。
- speclink-protocol 單元測試：request 缺 hold 反序列化為 false、hold 為 false 時不序列化；response held 同形。
- speclink-server 整合測試：結論端點帶 hold 時回應含 held: true 且討論留在 live 清單。
- golden：不帶 UPDATE_GOLDEN 重跑綠燈；`speclink update` 後 git status 無新增變動。

**範圍邊界**
- In scope：speclink-core（discuss.rs、archive.rs、command/mod.rs、init.rs 的版號、assets/skills/discuss.md 與 improve.md、tests/golden）、speclink-cli、speclink-protocol、speclink-remote、speclink-server、.claude/skills/ 與 .agents/skills/ 的受管 SKILL.md。
- Out of scope：desktop（core、src-tauri、ui）、speclink-node、speclink-host、docs/。

## Risks / Trade-offs

- **回歸對照**：golden 快照五份與 assets.lock 必須在乾淨樹再生，且版號、golden、lock 三者同批；漏跑 speclink update 會讓 repo 安裝的 SKILL.md 停在舊版號（golden 測試抓不到）。tasks 以「speclink update 後 git status 無變動」為驗收。
- **不帶 --hold 的輸出逐位元不變**是既有 CLI 測試與 remote_write_path 基準的保護對象；held 欄位以「false 不出鍵」實作即可維持。
- **舊 server 靜默忽略 hold**：與其他選填請求欄位的既有取捨相同；人眼輸出缺保留行即為訊號。
- **frontmatter 文字手術**：hold 行的插入與移除是純字串操作，值固定為 `true`，不涉及 yaml_scalar 跳脫；移除以整行比對，不受位置影響。
- **跨平台**：只動 frontmatter 文字與 CLI 輸出，無路徑或 git 互動；換行沿記錄既有的 `\n`。
