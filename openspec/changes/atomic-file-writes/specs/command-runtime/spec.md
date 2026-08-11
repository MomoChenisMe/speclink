## ADDED Requirements

### Requirement: 本地檔案寫入原子落盤

引擎對 workspace 共享真相檔案（openspec/ 樹、.speclink.yaml、openspec/config.yaml）的寫入 SHALL 經單一寫檔入口以原子方式落盤：先寫同目錄暫存檔、再 rename 至目的路徑，使並行讀者於任一時點讀到的都是舊全文或新全文，SHALL NOT 觀察到空檔或部分內容。rename 因平台限制失敗（如 Windows sharing violation）時 SHALL 退回直接寫入並清理暫存檔——行為不劣於原子化前；成功路徑 SHALL NOT 於目的目錄殘留暫存檔。原子保證於 unix SHALL 全額成立，Windows 為 best-effort（退回路徑存在即可）。

#### Scenario: 並行讀者不見半份內容

- **WHEN** 一個 process 正經引擎寫入 workspace 檔案，另一 process（或執行緒）同時反覆讀取同一路徑
- **THEN** 每次讀取得到的都是舊全文或新全文之一，絕不出現空檔、截斷或新舊混合內容（unix 全額保證）

#### Scenario: 寫入完成不殘留暫存檔

- **WHEN** 引擎寫檔成功完成
- **THEN** 目的目錄中不存在該次寫入使用的暫存檔

#### Scenario: 設定寫入走同一原子入口

- **WHEN** CLI 的設定編輯動詞或 desktop 設定頁寫入 openspec/config.yaml
- **THEN** 寫入經同一原子入口落盤，觀察面與引擎其他寫入一致（無暫存殘留、內容為完整全文）
