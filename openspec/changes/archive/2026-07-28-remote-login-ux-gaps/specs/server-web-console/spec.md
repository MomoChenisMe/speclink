## ADDED Requirements

### Requirement: 帳號頁呈現我的專案

`/account` SHALL 呈現我的專案區塊，列出目前使用者隸屬的每個專案（顯示名與角色；顯示名缺席時以專案 key 呈現），資料 SHALL 來自 account summary、SHALL NOT 另打管理端點。admin 與一般成員 SHALL 看到同一區塊與同一形狀（admin 不因管理身分而多列非隸屬專案）。無任何隸屬時 SHALL 呈現引導性空狀態（說明由管理員授予隸屬），SHALL NOT 隱藏整個區塊。本區塊 SHALL 為唯讀，SHALL NOT 提供任何隸屬變更操作。

#### Scenario: 成員看到自己的專案與角色

- **WHEN** 隸屬兩個專案的一般成員開啟 /account
- **THEN** 我的專案區塊列出兩個專案的顯示名與各自角色，無任何編輯操作

#### Scenario: admin 看到的是自己的隸屬而非全部專案

- **WHEN** 具 admin 旗標、僅隸屬一個專案的使用者開啟 /account
- **THEN** 我的專案區塊僅列該一個專案；其餘專案不出現（全部專案屬 /admin 的治理視角）

#### Scenario: 無隸屬時的空狀態

- **WHEN** 無任何專案隸屬的使用者開啟 /account
- **THEN** 我的專案區塊呈現空狀態文字，說明隸屬由管理員授予；區塊本身仍可見
