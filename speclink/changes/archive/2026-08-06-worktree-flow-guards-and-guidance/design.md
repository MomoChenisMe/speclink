## Context

討論 worktree-archive-merge-order 定案 worktree 流程的正典順序：worktree 內 apply →（review? ∥ verify?，建議於 worktree 內完成）→ commit → worktree-merge → 封存一律在主 checkout。現況三個落差：(1) 封存引擎對執行環境零判斷，linked worktree 內封存靜默成功但解封存備份寫進 worktree 的 gitignored `.speclink/snapshots/`，隨 worktree 移除蒸發（資料遺失級）；(2) worktree-merge 技能的交棒文案指向「merge 後跑品質站」，與品質站機制的實際落點（Apply baseline 只在 worktree 的 `.speclink/review-scopes/` 存在）相反；(3) 生成指令檔沒有 worktree 流程線，代理無從得知順序。約束：兩站在 workflow 正典中是可選的，引導不強制——唯一硬擋保留給資料遺失級的封存；決策權在使用者。

## Goals / Non-Goals

**Goals:**

- 封存動詞在 linked worktree 內 fail-closed 拒絕，零檔案效果，訊息指路 worktree-merge
- worktree 政策開啟時，生成指令檔載明正典順序（流程線＋指引 bullet），政策關閉時輸出不含任何新內容
- 三個技能 asset（worktree-merge、apply-worktree-post、archive）的交棒／提示文字與正典順序一致
- 全部生成物經三連動（MARKER_VERSION／golden／assets.lock）落地，不手改生成物

**Non-Goals:**

- worktree-merge 技能的蓋章 preflight（兩站可選，硬擋違反 workflow 語意——討論已否決）
- 品質站動詞（review／verify 蓋章）的 worktree 防呆（無資料遺失後果——討論已否決）
- 章跨 merge 的指紋語意變更（「其後有變動」是正確警示，保留）
- worktree overlay 映射條件與 teardown_blockers 行為變更
- quality 編排技能的內容（歸 quality-skill-canonicalization；本 change 的技能文字只以站名稱呼品質站，不點名 quality 技能）
- 主 checkout 補跑品質站時 baseline 缺席警告的指路強化（討論 Deferred）
- 解封存（unarchive）動詞本身

## Decisions

**D1 防呆落點＝speclink-core 的 archive() 入口。** desktop（apps/desktop/core/src/verbs.rs）與 command runtime（crates/speclink-core/src/command/mod.rs，CLI 經此）都收斂到 core `archive::archive()` 一個咽喉點；守門放在此函式最前段（與 require_valid_meta 同層、任何檔案效果之前），單筆與 bulk（逐筆呼叫同函式）自然同受保護。替代案「依 host-runtime 的『本地 git 事實歸 Host』慣例放 speclink-host」被否決：那條慣例服務的是 fail-open 的觀測面（list 的 overlay），本守門是寫入側的 fail-closed 閘，且 git helper（util::git）本就位於 core——在每個呼叫端各放一道會invite分歧。

**D2 判定條件與 fail-open 邊界。** 兩條件同時成立才拒絕：(a) workspace root 的 `.git` 是檔案（linked worktree 特徵；純 fs 判定，主 checkout 在此即短路、零 git 開銷）；(b) `git branch --show-current` 輸出具 `speclink/` 前綴。git 不可用、指令失敗或輸出空（detached HEAD）→ 放行（fail-open，沿 worktree discovery 慣例：無 git 的環境不得因此永遠無法封存）。分支前綴條件保護「使用者自己習慣在非 speclink worktree 內開發」的情境不被誤傷。拒絕走既有 Refusal 通道（非零 exit code、stderr 說明），訊息含 worktree 事實與「先 worktree-merge 合回主分支再封存」的指路。

**D3 引導內容與政策閘。** init.rs `instructions_body` 已收 `worktree: bool`；新增內容進同一閘：(a) Workflow 段主流程線之下加一條 worktree 流程線——claude／neutral 目標為 `worktree: apply-with-worktree ⇄ ingest → (review? ∥ verify?) → worktree-merge → archive (main checkout)`，codex 目標沿既有分支慣例以 `review?` 取代並列站（codex 無 verify 技能）；(b) Workflow 段 bullet 清單加一條「品質站建議於 worktree 內完成（Apply baseline 在 worktree）；封存僅在主 checkout，worktree 內封存會被引擎拒絕」。技能清單段的既有兩行 worktree 指引不動。政策關閉時輸出與現行位元一致（僅 MARKER_VERSION 異動）。

**D4 技能文案改向的邊界。** 只動交棒／提示文字，不動任何流程步驟與守則結構：worktree-merge asset 的合併成功交棒改為「品質站建議已於 worktree 內完成——已完成（或使用者略過）則下一步封存；未完成仍可於主 checkout 補跑（降級：主 checkout 無 Apply baseline）」；apply-worktree-post asset 的收尾交棒自僅點名 worktree-merge 擴充為「建議先於 worktree 內跑 review ∥ verify（蓋章後補提交）再 worktree-merge」（不合併、不移除 worktree 的停點不變）；archive asset 補一句主 checkout 限定與引擎拒絕的提示。品質站一律以站名稱呼。

**D5 三連動與平行協調。** MARKER_VERSION 提升一個 minor 版位；golden 再生涵蓋 claude／claude-worktree／codex／neutral 各快照與 assets.lock；本 repo 的 CLAUDE.md／AGENTS.md 與三個技能檔以 `speclink update` 刷新。平行進行中的 quality-skill-canonicalization 與 verify-station-parity 觸及同一 init.rs workflow 行與 golden——本 change 不依賴兩者、可獨立落地；後落地者以先落地的正典為基準重整（討論結論 Deferred 明記）。

## Implementation Contract

**行為（防呆）**：於 linked worktree（`.git` 為檔案）且分支 `speclink/<任意名>` 內執行 `speclink archive <change>`（或 bulk 封存）→ 非零 exit code；stderr 一句話說明「封存不得於 linked worktree 內執行」並指路 worktree-merge；change 目錄、正典規格、snapshots 目錄零變動。同環境但分支無 `speclink/` 前綴 → 行為與現行完全相同。主 checkout（`.git` 為目錄）→ 行為與現行完全相同且不 spawn git。git 不可用 → 放行（與現行相同）。

**介面**：無新旗標、無新子命令、無 `--json` shape 變更；拒絕訊息為既有 Refusal 通道的英文文字（確切措辭於 apply 時定稿，測試斷言關鍵詞 worktree 與 worktree-merge）。

**生成物**：`worktree: true` 專案的 CLAUDE.md marker 區塊含 worktree 流程線與指引 bullet（golden claude-worktree.snapshot.md 釘死位元）；`worktree: false` 專案的輸出除 MARKER_VERSION 外與現行位元一致（golden claude.snapshot.md 等釘死）。三個技能檔的新文字經 golden 與技能同步測試保護。

**驗收**：TDD——先寫 crates/speclink-cli/tests/it/archive_readiness_gate.rs 的整合測試（worktree 內拒絕＋零檔案效果、非 speclink 分支放行、無 git 放行沿用既有 fail-open 測試慣例），再實作守門；render_golden 測試以再生後的 golden 通過；既有全部測試綠燈。手動驗收：於本 repo 掛起的任一 speclink worktree 內執行封存應被拒絕。

**範圍邊界**：in scope＝archive() 守門、init.rs 範本、三個 asset 文字、golden／assets.lock／MARKER_VERSION、本 repo 生成物刷新；out of scope＝Non-Goals 全部項目、desktop UI、任何 CLI 輸出 shape。

## Risks / Trade-offs

- **git spawn 開銷**：僅在 `.git` 為檔案時才 spawn（主 checkout 純 fs 短路），單次封存增加毫秒級延遲——可接受。
- **fail-open 縫隙**：無 git 環境中在 worktree 內封存仍會成功並踩坑——接受此縫隙以保「無 git 不得封鎖封存」的既有慣例；speclink worktree 本身由 git 建立，實務上無 git 即無此環境。
- **golden 機械衝突**：與兩個平行 change 撞 init.rs 與 golden——純機械衝突，落地順序協調已明記於提案與討論 Deferred。
- **文案漂移**：三個 asset 的新文字若與引擎拒絕訊息措辭不一致會造成困惑——apply 時以同一份措辭定稿並在 review 檢查一致性。
