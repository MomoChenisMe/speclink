## Context

discuss 流程的記錄機件(frontmatter metadata、rounds、conclude、promote/link、封存)已完整,且 add-round 的 mode 為自由字串,新流程可全數複用。缺口有三:模型播種段沒有技能承載;討論記錄無型別標記,GUI 無法區分一般討論與模型發起的改進討論;workflow 文件未提及新入口。技能檔在本專案是引擎生成物——事實來源 crates/speclink-core/assets/skills/,經 init/update 渲染至 claude 與 codex 技能目錄,由 render golden 測試保護。源頭討論 architecture-improve-flow 已定案六步骨架與全部取捨;in-flight changes code-review-stage 與 verify-station-parity 的卡片小章樣式是 desktop 標示的既定範本。

## Goals / Non-Goals

**Goals**
- /speclink-improve 技能模板落地,六步骨架,Matt 原文精髓段(scope-before-you-scan、五條 friction 訊號、deletion test)逐字保留
- 討論記錄具機器可讀的 kind 標記,從 CLI 到 desktop 單一契約流通
- 卡片與抽屜標示,樣式與審查小章一致
- CLAUDE.md/AGENTS.md 注入區塊與 README 兩份的 workflow 同步

**Non-Goals**
- 不動 rounds/conclude/promote/link/archive 機件;不做 HTML 報告;不加系統匣標示;不新增設定欄位;不做行為正確性審查(與品質站分工);不擴充 LANGUAGE.md 承載架構詞彙

## Decisions

### kind 落在討論 frontmatter,discuss new 增 --kind 白名單旗標

kind 寫入既有 frontmatter(topic/slug/status/created/created_by 之側),由 speclink-core 的 new_discussion 承載;CLI 只傳字串,白名單驗證(僅 improve)在 core 的系統邊界執行,非法值非零 exit、stderr 說明、不落檔——鏡像 --slug 的驗證模式(discussion-docs 既有規格)。缺省無 kind 行即一般討論,舊記錄零遷移。frontmatter 由 core 讀寫、store 只存文件文字,fs 與 remote 兩條儲存路徑同管道,不新增 storage 耦合。
替代方案:slug 前綴或 round mode 字串推斷(stringly、無契約,討論已否決);獨立記錄型別(動 CLI/GUI/生命週期,成本大無收益,已否決)。

### DiscussionInfo 增選填 kind,protocol 單一正典流至 desktop

crates/speclink-protocol 的 DiscussionInfo 增 Option 欄位,--json 以 camelCase 曝露(kind 單字即合規),無值省略序列化——舊 payload 逐位元不變,既有 client 與回歸對照不受影響;server 與 typed client 共用同一 struct,remote 路徑免費繼承。desktop 經 packages/ui 的 adapter 型別增列 kind,元件不自行解析 frontmatter。
替代方案:desktop 直讀記錄檔 frontmatter(繞過 protocol 契約、remote 模式下無檔可讀,不取)。

### improve.md 模板自包含六步骨架,精髓段逐字保留

新模板 crates/speclink-core/assets/skills/improve.md,渲染為 claude 與 codex 兩份 /speclink-improve。六步:載入詞彙(speclink language show)→ 防重提檢查(discuss list 含 --archived 讀 Ruled out 與結論、speclink list 避開 in-flight 重疊區)→ 範圍收斂(使用者方向優先;否則 git log 熱點推斷加權近期變更,輔以 openspec/changes/archive 的 touched 記錄;分散則放寬網)→ 掃描(五條 friction 訊號與 deletion test 逐字保留 Matt 原文語意;inline 為預設,未指定方向或跨 crate 才派 Explore subagent,硬上限 2)→ 建記錄(discuss new --kind improve --slug improve-<範圍>,Round 1 以 mode 標籤 scan 記 candidates:Files/Problem/Solution/Wins/建議強度三級＋首選建議,問使用者挑哪個)→ grilling 收斂(自包含精簡版 interview 紀律:一次一題、提案帶證據、depth check 對每個被挑 candidate 無條件執行;conclude → promote/link 扇出;全數否決也 conclude 後 archive,不 discard)。frontmatter 標示僅使用者可發起。
替代方案:執行期引用 discuss 技能(載入鏈不確定,已否決);濃縮精髓段(丟操作性,Round 3 契約禁止)。

### desktop 小章鏡像審查章樣式

DiscussionColumn 卡片行內小章:lucide 既有 icon 家族＋Tooltip,不加文字列維持卡片極簡;DiscussionDrawer 同步顯示標示。狀態模型單純二值:kind 為 improve 顯示、否則無元素(含已轉出與已封存側,標示隨 kind 恆定,不隨生命週期變化)。i18n 詞條 tw「改進討論」(入 LANGUAGE.md 正典詞)、en 對應詞條,落 packages/ui/src/i18n.tsx 與 apps/desktop/src/i18n/messages.ts。
替代方案:卡片加文字列(違反極簡慣例,與審查章樣式不一致,不取)。

### 文件同步經 speclink update 落地,README 手改

CLAUDE.md/AGENTS.md 注入區塊的 workflow 段(discuss?/improve? → propose)改於模板事實來源,乾淨樹 golden 再生後,本 repo 執行 speclink update 落地——不手改生成檔。README.md 與 README.en.md 的 workflow 圖與 improve 一節手改,鏡像 code-review-stage 的安排。
替代方案:手改 CLAUDE.md/AGENTS.md(會被下次 update 蓋掉,討論已否決)。

## Implementation Contract

**CLI(speclink-cli ＋ speclink-core)**
- speclink discuss new <topic> --slug <s> --kind improve:建檔且 frontmatter 含 kind: improve;--json payload 增 kind 欄位;人眼輸出沿用既有建立訊息格式不新增行
- --kind 帶非白名單值:非零 exit code、stderr 說明僅接受 improve、openspec/discussions/ 不新增任何檔案
- 未帶 --kind:人眼輸出與 --json 逐位元不變(回歸保護對象)
- discuss list --json 與 discuss show --json:kind 有值時曝露、無值省略

**協定與 desktop(speclink-protocol ＋ packages/ui ＋ apps/desktop)**
- DiscussionInfo 的 kind 為 Option,無值省略序列化;既有 payload shape 不變
- kind 為 improve 的討論:看板卡片名稱旁出現小章,tooltip 為「改進討論」(tw)/對應英文(en);抽屜顯示同標示;一般討論無任何新增元素

**技能與文件(speclink-core assets ＋ golden)**
- speclink init/update 產出 claude 與 codex 兩份 speclink-improve 技能檔,內容含:六步骨架全段、Matt 精髓段(方向優先/熱點推斷/放寬網、五條 friction 訊號、deletion test 准入)、Explore subagent 硬上限 2、candidates 卡片五欄位與三級建議強度、全數否決走 conclude+archive
- cargo test -p speclink-core --test it render_golden:: 全綠(快照更新屬本變更的刻意變更)
- CLAUDE.md/AGENTS.md 注入區塊含 improve 入口;README.md 與 README.en.md workflow 圖含 improve

**範圍邊界**:in scope——上列四組;out of scope——系統匣、promote/link/archive 行為、config 欄位、既有 discuss 技能內文(不動)

## Risks / Trade-offs

- [golden 快照大面積變動與其他 in-flight change 衝突] → 乾淨樹再生、僅納本變更檔集;commit 前重盤 git status,與 code-review-stage/verify-station-parity 的並行工作以檔集分離
- [--kind 白名單未來擴充時散落多處] → 白名單常數單點定義於 core,CLI 與訊息引用同一來源
- [UI 詞條漏配導致 en 介面出現 tw 文案] → i18n 測試涵蓋兩詞條;鏡像 code-review-stage 任務 4.4 的驗證方式
- [remote 模式討論建檔繞過 kind] → kind 由 core 寫入文件文字,store 管道無關驅動;以既有 remote 寫路徑測試模式驗證一次

## Migration Plan

無資料遷移:舊記錄無 kind 欄位即一般討論。部署即生效;回滾即還原程式與模板,已寫入的 kind 行對舊版本是未知 frontmatter 鍵,讀取不受影響。

## Open Questions

- 卡片 icon 選型(lucide 家族內)留給實作,與審查章區辨即可
