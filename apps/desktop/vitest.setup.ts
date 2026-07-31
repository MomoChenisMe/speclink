import { JSDOM } from "jsdom";

// vitest 3.2 的 jsdom 測試環境未把 Web Storage 掛到 window 或全域，依賴本機儲存
// 持久化的 desktop 測試因此在存取 localStorage／sessionStorage 時拋
// 「Cannot read properties of undefined」。以一個真實 jsdom Window 的 Storage
// 補上這兩個全域，並於每個測試檔起始清空，確保測試檔之間狀態不殘留。
const { window: storageWindow } = new JSDOM("", { url: "http://localhost:3000/" });

for (const key of ["localStorage", "sessionStorage"] as const) {
  const storage = storageWindow[key];
  storage.clear();
  Object.defineProperty(globalThis, key, { configurable: true, value: storage });
}

// jsdom 未實作 Pointer Capture 與 scrollIntoView，Radix Select 開關與捲到選中項時
// 會直接呼叫。缺一即拋 TypeError——環境缺口，補 no-op stub。
if (typeof window !== "undefined") {
  const proto = window.Element.prototype as unknown as Record<string, unknown>;
  if (!proto.hasPointerCapture) proto.hasPointerCapture = () => false;
  if (!proto.setPointerCapture) proto.setPointerCapture = () => {};
  if (!proto.releasePointerCapture) proto.releasePointerCapture = () => {};
  if (!proto.scrollIntoView) proto.scrollIntoView = () => {};
}

// jsdom 未實作 ResizeObserver，面板捲動指示條與高度自適應會直接 new——
// 同屬環境缺口，補 no-op stub（jsdom 無版面計算，觀察本身無從觸發）。
if (typeof globalThis.ResizeObserver === "undefined") {
  globalThis.ResizeObserver = class {
    observe() {}
    unobserve() {}
    disconnect() {}
  } as unknown as typeof ResizeObserver;
}
