## Context

core 已有 `openspec/config.yaml` 的 text→text 改寫 seam（政策四欄完整目標態、context 三態、rules 整份代換、未知鍵保留），remote 已有 config 讀寫端點（讀回內容與版本、寫回帶 CAS）；desktop 設定頁是這兩者目前唯一的消費組合。本變更為 CLI 補上第二個消費者（`speclink workflow-config` 動詞），並在其上出貨內嵌技能 speclink-config。

範圍內：CLI 動詞（fs＋remote 兩模式）、內嵌技能資產與渲染、docs/configuration 兩語版更新。範圍外：Node SDK dispatch、desktop 改動、Host 新抽象層、`openspec/config.yaml` 本檔的內容（已完成）。

## Goals / Non-Goals

- Goal: local 與 remote 對等的 config.yaml 管理動詞；技能可在兩種模式下走同一介面。
- Goal: 寫入路徑與 desktop 共用 core seam——兩個客戶端的改寫語意永不分岔。
- Non-Goal: 不做政策的有效值解析（show 只回正典值）；不做設定的互動式 TUI。

## Decisions

### 決策一：動詞落點與命令執行層的關係

`speclink workflow-config` 歸 speclink-cli 的周邊設定動詞（與 config、init、update 同類），不進命令執行層、不擴 Node SDK dispatch。流程邏輯歸屬：改寫語意在 speclink-core 的 config seam（既有，不動）；fs 讀寫與 remote 編排在 speclink-cli（fs 分支經既有 workspace 定位讀寫檔案，remote 分支在 remote_commands.rs 沿既有 remote 動詞模式呼叫 speclink-remote 的 config 讀寫）。替代方案：進命令執行層供 dispatch 共用——被否：Node SDK 無此需求、命令層覆蓋表是凍結契約，擴表茲事體大；desktop 也未經命令層。此擺法朝 storage 解耦靠攏：CLI 只組裝輸入與呈現，改寫語意單一落點在 core。

### 決策二：remote 的版本處理——單動詞內讀-改-寫、版本不進介面

remote 寫入固定編排：讀 server config（得內容與版本）→ core seam 改寫 → 寫回帶讀得的版本。版本識別不出現在旗標或輸出；CAS 失敗＝他人並行改寫，以非零 exit code 提示重跑。替代方案：暴露 --expected-revision 旗標——被否：使用者無從得知版本值，暴露只會製造照抄的儀式；desktop 的畫面持有版本是因為有長駐 UI 狀態，CLI 單發指令沒有。風險：讀與寫之間的窗口內他人寫入——CAS 正確拒絕，重跑即可，無資料遺失。

### 決策三：--dry-run 由 CLI 以同一改寫路徑產 diff

dry-run 與實寫共用完全相同的「現況文字 → seam → 目標文字」計算，僅在最後一步改為輸出 unified diff 而不落檔。技能永不自行計算 diff。替代方案：技能以文字比對自算——被否：技能算的 diff 與實寫結果可能不一致（YAML 序列化細節），預覽即謊言。

### 決策四：set 的單鍵語意映射到 seam 的完整目標態

seam 的政策寫入是四欄完整目標態（false／未設＝移除鍵）。CLI 的 set <key> <value> 先讀現況組出四欄現值、改目標鍵、送 seam——單鍵編輯語意由 CLI 組裝，seam 介面不動。序列化向後相容由 seam 既有行為保證：未知鍵原樣保留、既有值不動；模板註解喪失是 seam 已載明的取捨（desktop 已走過），規格明述。替代方案：為單鍵編輯在 core 加 patch 介面——被否：兩客戶端就有兩種寫入語意，違反單一落點。

### 決策五：技能為內嵌資產，走 commit-skill 同機制

新資產 crates/speclink-core/assets/skills/config.md，於 skills 渲染註冊表登記，init／update 渲染至 claude（.claude/skills/speclink-config/）與 codex（.agents/skills/speclink-config/）。三處必須同步：core assets、repo 兩個技能實例、render golden——golden 於乾淨樹以 UPDATE_GOLDEN=1 再生並審視 diff。技能內容四要素：固定輸入來源、四判準（payload 反證、降 rules、不寫會過時、引用存在性）、dry-run 先行＋政策四欄逐項詢問、收斂驗收（連跑兩次第二次零 diff）。替代方案：技能只放本 repo 不入 assets——被否：這是產品能力，使用者的專案也要能 init 出來。

### 決策六：git 互動與跨平台

本動詞不呼叫 git。diff 輸出自行以文字生成 unified 格式，不依賴系統 diff 工具（Windows 無保證）。換行：seam 輸出以 LF 為準，與 .gitattributes 的 LF checkout 一致；Windows 上讀入 CRLF 檔時由 YAML 解析吸收。路徑：經既有 workspace 定位，不手組分隔符。

## Risks / Trade-offs

- golden 再生於 dirty 樹烙進未提交狀態（曾發生）：緩解——tasks 明定乾淨樹前置檢查。
- 與 spectra-legacy-cleanup 變更同批動 golden：緩解——兩變更依序 apply／archive（cleanup 先行），各自於乾淨樹再生。
- read-modify-write 喪失模板註解：已於規格明述為既知取捨；首次使用者可先跑 --dry-run 看 diff 再決定。
- remote 併行寫入窗口：CAS 拒絕＋重跑提示，無靜默覆蓋。

## Migration Plan

純新增，無遷移。既有 workspace 跑 speclink update 後獲得 speclink-config 技能；不跑 update 也不受影響。

## Open Questions

（無）
