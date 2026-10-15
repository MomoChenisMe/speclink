// 觸控板手勢事件的測試替身（desktop-swipe-back-finger-lift design D4）：SwipeBack 元件面
// （swipeBack.test.tsx）與 App 面（App.test.tsx）共用。測試檔的 `@tauri-apps/api/event` mock
// 把 trackpad-gesture 的訂閱交給 subscribeGesture，再以 gesture() 模擬原生層的手指按下與離開。
import { act } from "@testing-library/react";
import type { TrackpadGesture } from "../../components/SwipeBack";

export type GestureHandler = (e: { payload: TrackpadGesture }) => void;

/** 目前訂閱中的 trackpad-gesture handler；測試結束時清空。 */
export const gestureHandlers: GestureHandler[] = [];

/** listen mock 的 trackpad-gesture 分支：登記 handler，回傳的取消函式移除它。 */
export function subscribeGesture(handler: GestureHandler) {
  gestureHandlers.push(handler);
  return Promise.resolve(() => {
    const i = gestureHandlers.indexOf(handler);
    if (i >= 0) gestureHandlers.splice(i, 1);
  });
}

/** 原生層的手指按下／離開；ended 的 swipeEnabled 恆為 false（Rust 端如此送）。 */
export function gesture(phase: TrackpadGesture["phase"], swipeEnabled = phase === "began") {
  act(() => {
    for (const h of [...gestureHandlers]) h({ payload: { phase, swipeEnabled } });
  });
}
