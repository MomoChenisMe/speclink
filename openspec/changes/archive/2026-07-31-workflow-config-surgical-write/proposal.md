## Summary

openspec/config.yaml 的唯一寫入接縫（引擎的 workflow-config 改寫函式）由「整份 YAML 重新序列化」改為文字層手術：缺鍵按範本正典序插在 schema 鍵之下並以空行區隔、既有鍵原位改值不搬家、檔內註解／空行／未知頂層鍵逐位元保留。

## Motivation

使用者於 desktop 設定頁存檔後，四個政策鍵（locale、spec_locale、tdd、audit）被附加到檔案最尾（rules 之後），與 init 範本的正典序（schema → 政策鍵 → context → rules）相悖；且現行 read-modify-write 把整份文件 parse 成 mapping 再重新序列化，檔內所有註解與空行喪失——規格目前把註解喪失記載為取捨，使用者自加的說明內容實際上每存一次檔就被清洗一次。討論 desktop-workspace-auto-init 定案改文字層手術（方案 B）：位置、空行與「不動使用者內容」三個要求只有文字層做得到。

## Proposed Solution

- **文字層手術改寫**：引擎的 workflow-config 改寫函式（CLI workflow-config set／context／rules 與 desktop 設定頁共用的唯一接縫）重寫——
  - 政策四鍵為頂層單行 scalar：既有鍵逐行原位改值或移除（不搬家，曾被附加在檔尾的鍵留在原地）；缺鍵按範本正典序以連續區塊插在 schema 鍵行之後，區塊與下一內容行之間恰一空行；schema 缺席時插檔案最頂端。
  - context／rules 維持整塊替換語意：以頂層鍵邊界分段、僅代換該鍵區塊，其餘區段（含註解行與空行）原樣拼接；手術一律不刪註解行。
  - 防呆：寫前解析驗證照舊（壞檔 fail-closed）；落檔前將手術結果重新解析、與目標狀態等值比對，不等值即拒絕寫入——文字拼接的正確性以解析對照釘死。
- **消費端零改動**：CLI 三個寫入動詞、desktop 設定頁、remote 模式的讀-改-寫走同一函式，行為自動繼承；--dry-run 的 unified diff 隨之只含目標行的變動。
- **規格更新**：workflow-config 的 set 需求刪除「原檔的模板註解於重寫後喪失（read-modify-write 取捨）」，改為逐位元保留保證與缺鍵插入位置規範。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workflow-config`: 修改需求——set 政策欄位寫入的保留語意由「僅鍵值保留、註解喪失」升級為「非目標行逐位元保留」，新增缺鍵正典序插入位置與寫後解析等值驗證；context 與 rules 寫入經既有「與 set 一致」措辭自動繼承，不另動文字。

## Impact

- Affected specs: `workflow-config`（修改）
- Affected code:
  - Modified:
    - crates/speclink-core/src/config.rs
    - crates/speclink-cli/tests/workflow_config.rs
    - apps/desktop/core/src/settings.rs
  - New: (none)
  - Removed: (none)
- crates/speclink-cli 與 apps/desktop/core 僅測試斷言隨輸出更新（呼叫端程式不動）；server 端不動（remote 寫回內容由同一函式產生）。與同討論轉出的 desktop-enable-speclink-prompt 及進行中的 desktop-instruction-staleness-prompt 零檔案重疊，可獨立先行落地。

## Non-Goals

（範圍排除與否決方案詳見 design.md 的 Goals / Non-Goals。）
