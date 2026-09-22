## MODIFIED Requirements

### Requirement: 討論記錄的樹慣例與格式不變

技能檔 SHALL 規定記錄內容慣例：首輪 Position 攤開初始決策空間（得含 ASCII 樹），之後每輪聚焦解掉一個節點，中途發現的新分支記入該輪 Open。討論文件的骨架（Context／Rounds／Conclusion）、輪模板欄位（Focus／Position／Ruled out／Open）與 append-only 規則 SHALL 維持不變，既有討論記錄 SHALL NOT 需要遷移。

技能檔的文件規則 SHALL 另含一條與「Position bullets over prose」對稱的結論條列規則：Conclusion 的 Decision、Rejected alternatives、Deferred 三個欄位超過一句時 SHALL 條列，以 `- ` 一點一行——Decision 以一句定論起頭、其後條列；Rejected alternatives 與 Deferred 的欄位標頭後 SHALL NOT 接文字，直接一點一行；Rationale、Capture to、Next 三個欄位 SHALL 維持單段。Decision SHALL 保留全部定案細節、SHALL NOT 為了縮短而刪減，其各點（含縮排子項）SHALL NOT 回指輪次；結論規劃多刀轉出時，Decision SHALL 每刀一個 bullet，開頭為 `**cut N \`change-name\`**：一句範圍`，該刀細節 SHALL 為縮排子項、一子項一件事。Rejected alternatives SHALL 每項一行，寫「方案——落敗理由」；Deferred SHALL 每項一行，寫「問題——為何現在不解」，無擱置項時單寫 none。技能檔的 conclude 指令範例與「Capture decisions」的 Conclusion 摘要格式 SHALL 與此規則同形（範例本身即為條列形）。此規則 SHALL 只約束新寫入的結論；既有記錄的結論 SHALL NOT 需要改寫。

#### Scenario: 首輪攤樹且每輪一節點

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的記錄規則
- **THEN** 技能檔 SHALL 規定首輪 Position 含初始決策空間、後續每輪解一個節點、新分支記入該輪 Open

#### Scenario: 既有記錄格式沿用

- **WHEN** 以更新後的技能進行討論並經 speclink discuss 動詞寫入記錄
- **THEN** 產出的討論文件 SHALL 維持 Context／Rounds／Conclusion 骨架與 Focus／Position／Ruled out／Open 欄位，與既有記錄格式一致，無需任何遷移

#### Scenario: 技能檔載有結論條列規則且三種渲染目標同源

- **WHEN** 檢視 claude、codex、neutral 三種渲染目標產出的 speclink-discuss 技能檔的 Document rules
- **THEN** 三份技能檔 SHALL 各含同一條結論條列規則：Decision／Rejected alternatives／Deferred 超過一句條列、Rationale／Capture to／Next 單段、多刀每刀一個 bullet；且 conclude 指令範例與 Conclusion 摘要格式為條列形

#### Scenario: 多刀結論的 Decision 逐刀條列

- **WHEN** agent 依更新後的技能為規劃三刀轉出的討論寫入結論
- **THEN** Decision 第一行為一句總結，其後每刀一個 bullet 以 `**cut N \`change-name\`**：` 起頭、細節為縮排子項；Rejected alternatives 每項一行「方案——落敗理由」；Rationale 為單段；桌面結論分頁依既有「討論結論以欄位標籤呈現」渲染為標籤區塊內的清單

##### Example: 多刀結論的形狀

```
**Decision**: 新版 Server 與團隊管理分三刀轉出。
- **cut 0 `redesign-settings-page`**：設定頁與帳號選單
  - 側欄底部單一帳號列：頭像＋名字＋方案
  - 選單項目：使用情況／設定／登出
- **cut 1 `add-team-server-foundation`**：身分、專案、成員與管理畫面
  - Server：Fastify 5＋PostgreSQL＋Drizzle
  - 登入頁版型：白底、置中 340 px 直欄、間距 32
- **cut 2 `add-shared-byok-gateway`**：共用模型閘道
  - 路由 /api/model-gateway/azure：驗 Bearer→查權限→換真 key→串流轉發
**Rationale**: 登入是所有團隊功能的前提，所以 Server 第一刀只做身分與專案。
**Rejected alternatives**:
- 大場景 SVG 插圖——舊版做法，與現行系統風格不合
- 第二顆實心按鈕——次要動作一律底線文字鈕
**Deferred**: none
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion team-server-and-admin
```

#### Scenario: 單刀結論一句即止時不強制條列

- **WHEN** agent 寫入的結論 Decision 只有一句、Rejected alternatives 只有一項
- **THEN** 兩欄位 SHALL 得維持單行；規則只在欄位超過一句時要求條列

#### Scenario: 既有結論不回改

- **WHEN** 更新後的技能落地後，開啟一份規則落地前寫入、Decision 為整段長句的討論記錄
- **THEN** 記錄檔內容 SHALL 逐位元不變；桌面結論分頁依既有「討論結論以欄位標籤呈現」渲染，不報錯
