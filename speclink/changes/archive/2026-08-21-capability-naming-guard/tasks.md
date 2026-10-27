## 1. 近似名排序工具

- [x] 1.1 以 TDD 先寫紅燈測試再實作 `crates/speclink-core/src/capname.rs`：純函式接受候選名與帶來源標注的既有名集合（名稱＋來源＋Purpose 首行），回傳至多三筆排序建議；排序行為＝token 完全包含優先、kebab 字段交集數次之、編輯距離再次，無近似時回空清單。單元測試（`#[cfg(test)]`）涵蓋：`auth` 對 `authentication` 的包含關係排首位、交集數勝過編輯距離、上限三筆截斷、毫無交集回空。對應需求「近似名單的來源與排序」的排序半部。驗證：`cargo test -p speclink-core capname` 綠燈。 <!-- speclink-task:tsk_01M0F5PRH6Z28PMXPY57DJDBD6 -->

## 2. 建立點主閘

- [x] 2.1 先寫紅燈測試：`crates/speclink-core/src/newcmd.rs` 的閘門行為——未收錄名稱未帶確認參數時拒絕且不落盤（delta 目錄不存在）、命中正典名稱時行為與輸出維持現狀、帶確認參數時照現行流程建立、確認參數不豁免 delta 格式驗證、change 不存在時維持既有錯誤。驗證：`cargo test -p speclink-core newcmd` 先紅。 <!-- speclink-task:tsk_01M0F5PRH74P9M4P2MD9C5EPBM -->
- [x] 2.2 實作主閘於 `crates/speclink-core/src/newcmd.rs`：路徑解析後、任何寫入前檢查正典是否收錄該名稱；未收錄且未確認即以 refused 語意錯誤拒絕，錯誤訊息組裝 capname 建議清單（建議池＝正典 capabilities 附 Purpose 首行＋其他未封存 change 的 delta capabilities 附來源 change 名與 Purpose 首行，無 Purpose 略去該行）與兩條指引文字。交付需求「建立點主閘——未收錄名稱預設拒絕」與「近似名單的來源與排序」的建議池半部。驗證：2.1 全綠，並含「進行中 change 的 delta 出現在建議清單」案例。 <!-- speclink-task:tsk_01M0F5PRH741MPBMEMZC8FWRW1 -->
- [x] 2.3 CLI 旗標與命令層貫通，交付需求「--new 旗標顯性宣告新 capability」：`crates/speclink-cli/src/verbs/new.rs` 新增 `--new` 布林旗標、`crates/speclink-core/src/command/mod.rs` 的 dispatch 傳遞確認參數並讓拒絕歸類 refused。整合測試（`crates/speclink-cli/tests/`）斷言：未收錄名稱未帶 `--new` 時 exit code 非零、stderr 含建議與兩條指引；帶 `--new` 成功建立；命中正典名稱輸出位元級不變；`--json` 成功路徑 payload 欄位存在且 camelCase 形狀不變（artifact、change、path、status、validated、warnings），主閘拒絕時 stdout 無成功 payload。驗證：`cargo test -p speclink-cli --test it new_artifact` 綠燈。 <!-- speclink-task:tsk_01M0F5PRH7QH1JJNSNK1YG1Y1P -->

## 3. validate 第二網

- [x] 3.1 先寫紅燈測試：`crates/speclink-core/src/validate.rs` 的近似名 warning——正典有 `auth` 而 delta 目錄為 `authentication` 時報 warning 且驗證結果仍通過、delta 目錄與正典同名不報、毫無交集的新名不報且既有 Purpose 早檢查照常。驗證：`cargo test -p speclink-core validate` 先紅。 <!-- speclink-task:tsk_01M0F5PRH7YYAEBP6H8AK4MDVV -->
- [x] 3.2 實作 warning lint 於 `crates/speclink-core/src/validate.rs`，交付需求「新開 capability 的近似名 warning」：對每個正典無同名的 delta capability 呼叫 capname 同一建議池，非空即產出 warning 級發現，訊息含近似名與「沿用既有名／確為新 capability 可忽略」指引；不改變 valid 布林與 exit code。驗證：3.1 全綠。 <!-- speclink-task:tsk_01M0F5PRH7V5RW8S937H83SDZS -->

## 4. 技能資產與衍生物

- [x] 4.1 增補技能資產指令：`crates/speclink-core/assets/skills/propose.md` 加三項（既有規格掃描結果留痕於 proposal、New Capabilities 每項附一句「為何既有規格不涵蓋」、`--new` 旗標語意與使用時機——先跑 speclink new artifact spec 不帶旗標，被拒且確認建議清單無同義項後才帶 `--new` 重跑）；`crates/speclink-core/assets/skills/ingest.md` 加一項（新增 delta capability 前先對照正典與進行中 change 的既有名）。驗證：內容審閱對照 specs/capability-naming-guard 的「技能資產的命名守門指令」需求逐項命中。 <!-- speclink-task:tsk_01M0F5PRH79TA5B4ARRC3M039H -->
- [x] 4.2 資產三連動：推進 `crates/speclink-core/src/init.rs` 的 MARKER_VERSION、再生 `crates/speclink-core/tests/golden/` 的 snapshots 與 assets.lock。驗證：`cargo test -p speclink-core --test it` 中 golden 與資產一致性測試綠燈，`cargo test -p speclink-cli --test it engine_version` 綠燈。 <!-- speclink-task:tsk_01M0F5PRH751XVPW8GJNQZ5V36 -->
- [x] 4.3 於本 repo 執行 speclink update 再生 `.claude/skills/` 衍生技能檔並以 git status 盤點衍生物（衍生 SKILL.md 不進 evidence、收尾 commit 時一併帶上）。驗證：`.claude/skills/speclink-propose/SKILL.md` 與 `.claude/skills/speclink-ingest/SKILL.md` 含 4.1 的新指引文字。 <!-- speclink-task:tsk_01M0F5PRH7ZTB6ZQ4Y1KT38FKJ -->

## 5. 遷移與收尾

- [x] 5.1 盤點既有測試與 fixture 中「以正典未收錄名稱建 delta spec」的呼叫路徑（`crates/speclink-core`、`crates/speclink-cli` 兩處的測試），補上 `--new` 或確認參數使其符合新契約。驗證：`cargo test -p speclink-core` 與 `cargo test -p speclink-cli --test it` 全綠。 <!-- speclink-task:tsk_01M0F5PRH76DWRHKATGBX1EMX0 -->
- [x] 5.2 收尾回歸：依序跑 `cargo test -p speclink-core`、`cargo test -p speclink-cli --test it`，確認位元級輸出承諾（命中正典名稱與帶 `--new` 的路徑輸出不變）由既有 golden 與整合測試背書；git status 盤點所有改動檔（含衍生物）無遺漏。驗證：兩個測試面全綠、盤點清單與 proposal Impact 一致。 <!-- speclink-task:tsk_01M0F5PRH79ENRJ0CJHXJ14WF4 -->
