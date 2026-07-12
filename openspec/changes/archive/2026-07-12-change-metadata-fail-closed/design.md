## Context

engine-typed-core 之後，`.speclink.yaml` 與 openspec/config.yaml 已 fail closed，命令層（crates/speclink-core/src/command/mod.rs）是 CLI 與 Node dispatch 的唯一執行路徑，錯誤以封閉碼註冊表分類。剩下的 fail-open 入口是 change metadata：`ChangeMeta::from_text`（crates/speclink-core/src/model.rs）對「存在但解析失敗」的 `.openspec.yaml` 以 unwrap_or_default 靜默退回全預設。呼叫點分佈在三層：Store 實作建構 Change 時（crates/speclink-fs/src/lib.rs、crates/speclink-core/src/teststore.rs）、core 流程函式（crates/speclink-core/src/inprogress.rs、crates/speclink-core/src/discuss.rs、crates/speclink-core/src/discard.rs 經 change.meta）、桌面 core 直呼（apps/desktop/core/src/cache.rs、apps/desktop/core/src/manage.rs）。此外 `set_board_rank` 與 in-progress 的文字手術寫回不經解析即可在壞檔上疊寫。桌面 app 直呼 core 函式、不經命令層（Phase 3 才收編），因此守門位置必須讓兩條路徑都受保護。

## Goals / Non-Goals

**Goals:**

- `.openspec.yaml`「存在但解析失敗」時：讀取查詢將該 change 標為 invalid 並附診斷；一切會寫入該 change 的動詞以 typed error 拒絕。
- 「檔案不存在」與「欄位缺席」維持既有預設行為（change-lifecycle「meta 新欄位向後相容」不變）。
- CLI 與 Node dispatch 對壞 metadata 回一致的 invalid_config 錯誤分類；桌面直呼路徑受同一層守門保護。
- 有效 metadata 的 workspace：所有動詞人眼與 --json 輸出逐位元不變。

**Non-Goals:**

- 不動 Store trait 簽名、revision、CAS 與 Unit of Work（teamstore-contract-v2 刀）。
- 不提供壞檔修復工具、不做 metadata schema 驗證（僅 YAML 可解析性與既有欄位型別）。
- 不擴增錯誤碼註冊表；不改桌面看板視覺設計與拖曳互動。
- 不遷移桌面直呼路徑到命令層（Phase 3）。

## Decisions

### 決策一：from_text 改回 Result，Change 帶 meta_error 診斷欄位

`ChangeMeta::from_text` 簽名改為回 `Result<ChangeMeta, String>`（錯誤字串為 serde_yaml 的解析原因）：缺檔（None）與空文件仍回 `Ok(default)`，「存在但解析失敗」回 `Err`。`Change` 結構新增 `meta_error: Option<String>` 欄位：Store 實作建構 Change 時，解析失敗以 `meta: ChangeMeta::default() + meta_error: Some(原因)` 承載，讓 list 類查詢照常列出全部 change。換簽名（而非另加寬鬆變體）是刻意的：所有殘留呼叫點會編譯失敗，型別系統完成 fail-open 路徑的窮舉盤點。

替代方案：(a) `Change.meta` 改 `Result<ChangeMeta, MetaError>`——所有 `.meta.` 消費者全數改寫，churn 遠超本刀範圍；(b) 保留寬鬆 from_text 另加 try 變體——fail-open 路徑仍在，違反本刀目的。取捨：`meta_error` 是編譯期不強制檢查的旁位欄位，守門依賴決策二的語意層集中防護；此弱點以「守門下沉 core 流程函式」封閉。此設計不新增儲存假設，Store 仍以文字讀寫 metadata，朝 storage 解耦方向中性。

### 決策二：守門下沉 core 流程函式，命令層只做錯誤碼映射

fail-closed 檢查放在讀寫該 change 語意的 core 函式內（流程邏輯歸 core）：開工標記寫入與開工判定（inprogress）、discuss link／seal 的 change 側、discard 的動工痕跡判定、archive 的 schema 解析、new artifact／status／instructions／validate／analyze／drift／artifact cat 的 schema 解析點、`set_board_rank` 的文字手術前。遇 `meta_error` 一律回帶「workspace 相對路徑＋解析原因」的錯誤（沿用 engine-typed-core 對壞設定檔的訊息形式），命令層將此類錯誤映射為 invalid_config——為此 core 引入可辨識的錯誤型別（如 `MetaError`，PascalCase），避免以字串比對分類。

替代方案：守門集中在命令層的 change 解析守門點——桌面直呼 core 函式的路徑（cache、manage、verbs）完全漏防，壞檔仍會被桌面疊寫，被拒。取捨：core 函式各自守門有重複樣板，但每個動詞恰好一個檢查點，且桌面「免費」受保護。

### 決策三：錯誤碼沿用 invalid_config，不擴碼

註冊表語意「設定檔存在但無法解析」與 `.openspec.yaml` 情境完全同構；封閉集合維持五碼是 command-runtime 正典的既定立場。錯誤碼對應表新增一行「`.openspec.yaml` 存在但 YAML 解析失敗 → invalid_config」。

替代方案：新增 invalid_meta 碼——語意上與 invalid_config 無可觀測差異（都是「檔案存在但無法解析，修檔即癒」），擴碼徒增下游 match 分支，被拒。

### 決策四：list 對 invalid change 的診斷呈現

list --json 的 change 項目新增選填欄位 `metaError`（camelCase，經 serde rename；僅 metadata 損壞時出現，值為解析原因），其餘欄位以預設 meta 衍生值呈現（與今日行為一致，維持欄位形狀）。人眼 list 在該 change 行尾附 invalid 標記（形如「(invalid .openspec.yaml)」，確切文字實作時定於渲染層並凍結為測試斷言）。`sort_changes` 的 created 排序已將 invalid-metadata 變更排在有效變更之後（既有行為），不動。桌面 board payload（cache）同樣帶 invalid 標記欄位，卡片照 stage 顯示但標記 invalid；UI 僅做最小視覺標記，操作被 core 錯誤拒絕即可。

替代方案：invalid change 從 list 隱藏——使用者失去發現與修復壞檔的入口，且與「一份壞檔不得讓 UI 失效」的目的相反，被拒。

### 決策五：看板補章與排序寫入的防護

欄內補章（manage 的批次 board_rank 寫入）計算時排除 `meta_error` 卡片：invalid 卡不列入「缺 rank」清單、不觸發寫入，其餘卡片補章照常。`set_board_rank` 在文字手術前先解析，壞檔回 MetaError。這封閉「壞 metadata 被當缺 rank 而觸發自動寫入」的路徑——補章是唯一不經使用者手勢就寫 metadata 的動詞。

替代方案：補章遇 invalid 卡整欄中止——一份壞檔癱瘓整欄排序功能，與 list 不失效的原則矛盾，被拒。

### 決策六：輸出凍結與壞檔情境的測試邊界

有效 metadata workspace 的輸出凍結由既有 parity（31 項）／color（16 項）／twin（8 情境）對照保證——本刀不需重建 baseline exe（壞檔是新情境、有效路徑無行為變更，回歸對照本身即是凍結證據）。壞檔情境以新測試固定：core 單元測試（解析、各守門點）、CLI 整合測試（list 標記與 --json 欄位、寫入動詞非零 exit 與 stderr 訊息）、Node vitest（dispatch 錯誤碼 invalid_config）、桌面 core 測試（cache 標記、補章排除）。

## Implementation Contract

- **行為**：workspace 內某 change 的 `.openspec.yaml` 為壞 YAML 時——`speclink list`（人眼）該行帶 invalid 標記、`--json` 該項帶 `metaError` 欄位且清單其餘 change 不受影響；`status`／`instructions`／`validate`／`analyze`／`drift`／`artifact cat` 指向該 change 時非零 exit、stderr 指出 workspace 相對路徑與解析原因；`new artifact`／`task done`／`task undone`／`claim`／`in-progress add`／`archive`／`discard`（含帶 --force）／`discuss link`／`discuss seal` 及看板排序寫入一律拒絕且不寫任何檔案；欄內補章不對該卡寫入。缺 `.openspec.yaml` 或欄位缺席的 change 行為與現行完全相同。
- **介面／資料形狀**：`ChangeMeta::from_text(Option<&str>) -> Result<ChangeMeta, String>`；`Change` 新增 `meta_error: Option<String>`；core 新增 MetaError 錯誤型別（帶 change 名與解析原因），命令層映射為錯誤碼 invalid_config；list --json 新增選填 camelCase 欄位 `metaError`。
- **失敗模式**：壞 metadata 的錯誤一律顯性（CLI 非零 exit＋stderr、dispatch reject code invalid_config、桌面顯示 core 錯誤）；唯一刻意沉默是 list／board 查詢不因單一壞檔失敗（以標記代替錯誤）。
- **驗收**：cargo test -p speclink-core、-p speclink-cli 全綠且含上述壞檔新測試；crates/speclink-node 的 npm test 全綠含 dispatch invalid_config 案例；npm test -w apps/desktop 全綠含補章排除案例；parity／color／twin 對照全綠（root 單一指令依 delivery-baseline 執行全量驗證）。
- **範圍邊界**：in scope——from_text 簽名、Change.meta_error、core 各守門點、CLI 渲染標記、desktop core 跟進與補章排除、node bridge 編譯跟進；out of scope——Store trait 簽名、錯誤碼註冊表擴增、壞檔修復指令、桌面 UI 重設計、archive 目錄下歷史 change 的 metadata 語意（僅活躍 change）。

## Risks / Trade-offs

- [遺漏 fail-open 呼叫點] → from_text 換簽名使殘留呼叫編譯失敗；以 git grep from_text 盤點作雙重確認。
- [list --json 新欄位破壞下游消費者] → 欄位選填且僅壞檔情境出現；parity 基線 fixture 無壞檔，逐位元對照不受影響。
- [meta_error 旁位欄位被新代碼忽略（編譯期不強制）] → 守門集中在 core 流程函式的單一檢查點；spec 場景逐動詞斷言拒絕行為，回歸網補足型別系統缺口。
- [錯誤訊息路徑跨平台漂移（Windows 反斜線）] → 沿用 engine-typed-core 壞設定檔訊息的 workspace 相對路徑機制與其既有跨平台測試慣例。
- [桌面補章排除改變欄內既有卡片的 rank 結果] → 補章只排除 invalid 卡，有效卡的 rank 計算輸入不變；桌面 core 測試以混欄 fixture 斷言。

## Migration Plan

單刀交付，無資料遷移：壞檔在舊版被靜默讀為預設，新版報錯——使用者依訊息修正 YAML 即恢復，無檔案格式變更。回滾即還原 commit，無持久狀態殘留。

## Open Questions

（無）
