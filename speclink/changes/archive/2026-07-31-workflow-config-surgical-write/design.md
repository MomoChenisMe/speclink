## Context

openspec/config.yaml 的所有寫入（CLI workflow-config set／context／rules、desktop 設定頁存檔、remote 模式寫回）匯流於 speclink-core config 模組的單一改寫函式 update_workflow_config_text：把原文 parse 成 serde_yaml Mapping、insert 目標鍵、整份重新序列化。兩個後果：(1) mapping insert 對不存在的鍵一律附加尾端——政策四鍵跑到 rules 之後，違反 init 範本正典序（schema → 政策鍵註解示例 → context → rules）；(2) 重新序列化抹掉全檔註解與空行——規格現以「read-modify-write 取捨」記載。來源討論 desktop-workspace-auto-init 定案改文字層手術；使用者明確要求：缺鍵插 schema 之下、與 context 之間空一行、schema 底下使用者自加內容不刪不動。

## Goals / Non-Goals

**Goals:**

- 缺鍵按範本正典序（locale、spec_locale、tdd、audit）成連續區塊插於 schema 鍵行之後，與下一內容行之間恰一空行；schema 缺席插檔案最頂端
- 既有鍵原位改值或移除、不搬家；非目標行（註解、空行、未知頂層鍵、schema 底下使用者內容）逐位元保留
- 手術結果落檔前以解析等值驗證防呆，失敗 fail-closed
- CLI 三動詞、desktop 設定頁、remote 寫回自動繼承（同一接縫、消費端零改動）

**Non-Goals:**

- 不引入 YAML 註解保留（round-trip）第三方庫——config.yaml 的頂層結構受限（已知鍵＋未知頂層鍵），行級分段足夠，通用庫是過度配備
- 不重排既有鍵至正典序——搬動使用者檔案佈局也是「動到」；只有新增的缺鍵按正典位置插入
- 不改寫入動詞的介面（旗標、stdin、exit code、錯誤訊息語意）與 remote 版本衝突機制
- 不動 .speclink.yaml 的 tools 改寫函式（update_app_config_tools_text）——不在使用者回報範圍，其註解喪失另案再議

## Decisions

### 決策 1：行級分段的文字層手術，不用註解保留庫

改寫函式重寫為：將原文按頂層鍵行（行首非空白且含冒號的行）分段為「鍵區塊」序列，區塊間的註解行與空行歸屬其後方區塊之前的獨立區段、手術一律不刪。政策四鍵是頂層單行 scalar——原位改值＝替換該行；移除＝刪該行（不刪其上方註解行）；缺鍵＝於 schema 鍵行之後插入正典序連續區塊（前後空行規則見決策 2）。context／rules 的整塊替換＝以該鍵行起、至下一頂層鍵行前止的區塊代換為新序列化內容。YAML 的縮排規則保證 block scalar（context: |）的內容行必縮排，行首非空白即新頂層鍵——分段判定天然安全。
替代方案：serde_yaml Mapping 重排序＋後處理塞空行（討論的方案 A）——位置能修但註解仍全滅，違反「不動使用者內容」，討論已否決；引入 yaml round-trip 保留庫——對受限結構是重依賴，且行為黑箱難以測試釘死，否決。

### 決策 2：缺鍵插入位置與空行規則

缺鍵區塊插於 schema 鍵行（及其緊接的同區塊行）之後：先一空行、再按正典序列出缺鍵各一行、再一空行接原有後續內容；原有後續內容已以空行開頭時不重複補（恰一空行語意）。schema 鍵不存在時插於檔案最頂端（區塊後接一空行）。schema 底下已有使用者自加內容時，插入點仍為 schema 鍵行之後——使用者內容被往下推但逐位元不變（討論確認「不刪到不動到即可」）。同次寫入多個缺鍵成一個連續區塊，不分散。
替代方案：插於 context 鍵行之前——context 缺席時退化為尾端附加、回到原病灶，否決；插於使用者內容之後——「schema 底下」的錨點語意變成浮動、無法測試釘死，否決。

### 決策 3：寫後解析等值驗證 fail-closed

手術產出落檔前：重新以既有 parse 路徑解析，與「目標狀態」（原解析結果套用本次變更後的語意值）逐鍵比對——政策四鍵、schema、context、rules 與未知頂層鍵全部等值才寫入；不等值即以單行錯誤拒絕（訊息指明內部改寫驗證失敗）。此驗證是手術 bug 的最後防線：文字拼接錯誤最壞情況是「拒絕寫入」，永不落壞檔——與既有「壞檔 fail-closed 拒寫」同一設計哲學。
替代方案：信任手術直接落檔——文字層編輯的邊界情況（結尾無換行、重複鍵、CRLF）出 bug 即毀使用者檔案，代價不對稱，否決。

### 決策 4：測試斷言隨輸出更新的邊界

CLI dry-run diff 與 desktop settings 的既有測試斷言以「重新序列化後的全檔」為預期值者，隨新輸出更新為「僅目標行變動」的預期；更新僅限斷言值，呼叫端程式與測試結構不動。這批斷言翻新即是本變更的行為驗收之一：diff 收窄為目標行是使用者可見的改善（dry-run 不再顯示整檔重排）。

## Implementation Contract

**行為**：

- 於含 schema、context、rules 且無政策鍵的 config.yaml 執行 set locale tw → locale 行出現在 schema 鍵之後、與下一內容間恰一空行；檔內其他行（含全部註解與空行）逐位元不變。
- desktop 設定頁存檔（寫入完整目標狀態、含原先未設定的鍵）→ 新鍵成連續區塊插於 schema 之後正典序排列；既有鍵原位改值。
- 對曾被舊版附加在檔尾的政策鍵改值 → 該鍵留在檔尾原位改值，不搬家。
- context 整塊替換 → 僅 context 區塊內容變動，前後註解行與其他區段逐位元不變。
- 手術結果解析不等值（防呆觸發）→ 拒絕寫入、單行錯誤、原檔逐位元不變。

**介面／資料形狀**：update_workflow_config_text 的函式簽名、參數語意（WorkflowPolicyFields 完整目標狀態、ContextEdit 三態、rules 整份代換）、錯誤型別不變——僅輸出文字的產生方式改變。CLI 與 desktop 呼叫端零改動。

**失敗模式**：壞檔寫前 fail-closed（既有）；寫後等值驗證失敗 fail-closed（新增），兩者皆單行錯誤且原檔不動。

**驗收準則**：

- speclink-core 單元測試（config.rs）：規格「workflow-config set 政策欄位寫入」修改後全部場景——缺鍵插 schema 之下含空行規則、註解與空行逐位元保留、既有鍵原位改值不搬家、schema 缺席插頂端、schema 底下使用者內容原樣後移、context 整塊替換不動其他區段、多缺鍵一次寫入成連續區塊、寫後等值驗證防呆路徑。
- 既有測試回歸：crates/speclink-cli（set／context／rules／dry-run diff）與 apps/desktop/core settings 測試隨輸出更新斷言後全綠。
- 全套 cargo test 綠燈（remote 寫回同函式，serverfs／sqlite 相關套件測試不得受影響）。

**範圍邊界**：in scope＝update_workflow_config_text 重寫、其單元測試、既有斷言翻新、workflow-config spec 更新；out of scope＝update_app_config_tools_text（.speclink.yaml）、寫入動詞介面、remote 版本衝突機制、desktop 設定頁 UI。

## Risks / Trade-offs

- [文字層手術在邊界情況產生壞 YAML] → 決策 3 的寫後解析等值驗證兜底：最壞情況拒寫，永不毀檔；邊界情況（結尾無換行、CRLF、重複鍵）入單元測試。
- [既有測試大量斷言重新序列化輸出] → 決策 4 限定斷言值翻新；翻新本身是行為驗收（diff 收窄）。
- [「恰一空行」規則與各種原檔空行狀態的組合爆炸] → 規則收斂為兩條（插入區塊前後各一空行、已有者不重複），單元測試以代表性原檔覆蓋。
- [未知頂層鍵含多行值的分段誤判] → YAML 縮排規則保證延續行必縮排；以含多行 block scalar 未知鍵的測試釘死。

## Migration Plan

純行為改善、無資料遷移：合入後首次寫入即按新位置插鍵；先前被附加在檔尾的鍵不搬家（原位改值），使用者可自行手動整理一次、之後不再劣化。回退＝revert commit，寫入回到附加尾端與註解喪失的舊行為，已寫出的檔案無需處理。

## Open Questions

（無——插入位置、空行規則、保留範圍均由來源討論與使用者原話定案。）
