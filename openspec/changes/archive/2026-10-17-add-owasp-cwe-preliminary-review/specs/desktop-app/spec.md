## Purpose

補充桌面專案圖示可辨識性的裝置偏好。

## ADDED Requirements

### Requirement: 裝置層專案圖示底色

Desktop SHALL 在既有專案設定提供預設及六個固定色票，Local／Remote SHALL 依 locatorKey 儲存在裝置偏好，不修改團隊文件。設定顏色的專案 SHALL 在選取時顯示深底白字，未選取時底色混入50%白色並搭配同色系深色字，並取代未設定時的灰底／主色樣式；主視窗與系統匣 SHALL 一致。重開 SHALL 保留；失敗 SHALL 顯示錯誤且不變更已採用值。Remote reader SHALL 能改自己的裝置偏好。

#### Scenario: 同首字母的專案以色彩區分

- **WHEN** Spec A 設紫色且被選取，Spec B 設粉紅色且未選取
- **THEN** 兩者皆顯示S，A為紫色深底、B為淡化50%的粉紅底，切換選取後A淡化50%、B恢復深底；關閉重開仍保留，清回預設只影響該專案

#### Scenario: 儲存失敗不改舊值

- **WHEN** 藍色專案改紫色但裝置儲存拒絕
- **THEN** 顯示失敗原因，圖示維持藍色；Remote政策沒有寫入
