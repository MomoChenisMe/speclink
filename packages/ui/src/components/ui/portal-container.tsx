import * as React from "react";

// 浮層（Select 選單之類）預設 portal 到 document.body。但在 modal 對話框／抽屜內部，
// body 是 focus trap 的外面：Dialog 的 FocusScope 會把焦點拉回 content，Radix Select
// 又把焦點送去它 portal 出去的選單，兩邊對同一次 focus 事件互推，直到爆堆疊。
//
// 因此 modal 容器把自己的 content 節點透過這個 context 提供出去，浮層改 portal 進去，
// 焦點就一直待在同一個 scope 裡。容器外（一般頁面）沒有 provider，維持 portal 到 body。
const PortalContainerContext = React.createContext<HTMLElement | null>(null);

export const PortalContainerProvider = PortalContainerContext.Provider;

/** 目前應該 portal 進去的節點；不在 modal 容器內時為 null（＝portal 到 body）。 */
export function usePortalContainer(): HTMLElement | null {
  return React.useContext(PortalContainerContext);
}
