## ADDED Requirements

### Requirement: ingest 參數可指定要更新的變更

技能檔（事實來源 crates/engine/speclink-core/assets/skills/ingest.md，經 init 與 update 渲染至 claude 與 codex 的技能目錄）SHALL 規定 ingest 參數的解析順序：參數含 `/` 或以 `.md` 結尾時 SHALL 當成計畫檔；否則代理人 SHALL 先以 `speclink list --json` 取得作用中變更名，參數等於其中一個時 SHALL 把該變更當成要更新的變更、以目前的對話內容（與該變更連結的討論結論）為需求來源，SHALL NOT 再當成計畫檔尋找，SHALL NOT 再詢問要更新哪個變更；參數不等於任何作用中變更名時 SHALL 當成計畫檔名稱，找不到計畫檔時 SHALL 回報錯誤並停止，錯誤訊息 SHALL 同時說明「不是作用中的變更名稱」與「找不到同名計畫檔」。不帶參數時 SHALL 維持既有行為。

#### Scenario: 參數是作用中變更名

- **WHEN** 作用中變更有 `add-auth` 與 `add-billing`，使用者執行 `/speclink-ingest add-auth`
- **THEN** 技能檔指示代理人更新 `add-auth`、以對話內容為來源，不尋找 `add-auth.md`、不詢問要更新哪個變更

#### Scenario: 參數是計畫檔路徑

- **WHEN** 使用者執行 `/speclink-ingest <計畫目錄>agile-discovering-rocket.md`
- **THEN** 技能檔指示代理人照既有流程讀取該計畫檔，再詢問或確認要更新的變更

#### Scenario: 參數是計畫檔名稱

- **WHEN** 作用中變更沒有名為 `agile-discovering-rocket` 者，使用者執行 `/speclink-ingest agile-discovering-rocket`，且計畫目錄有 `agile-discovering-rocket.md`
- **THEN** 技能檔指示代理人照既有流程使用該計畫檔

#### Scenario: 兩種都找不到

- **WHEN** 使用者執行 `/speclink-ingest add-ath`（拼錯），作用中變更沒有 `add-ath`，計畫目錄也沒有 `add-ath.md`
- **THEN** 技能檔指示代理人回報錯誤並停止，訊息說明 `add-ath` 不是作用中的變更名稱、也找不到同名計畫檔，不修改任何 artifact

#### Scenario: 技能檔含參數解析指示

- **WHEN** 技能再生後讀取 claude 與 codex 兩份 speclink-ingest 的 SKILL.md
- **THEN** 兩份技能檔都含「參數等於作用中變更名時當成要更新的變更」的指示，且該指示位於選擇變更的步驟之前
