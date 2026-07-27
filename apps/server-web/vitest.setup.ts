import { JSDOM } from "jsdom";

// vitest 3.2 的 jsdom 環境未把 Web Storage 掛到 window 或全域（與 apps/desktop 同一個
// 問題）。導覽的「看過了」狀態存 localStorage，沒有它連測試的初始狀態都無從設定。
const { window: storageWindow } = new JSDOM("", { url: "http://localhost:3000/" });
for (const key of ["localStorage", "sessionStorage"] as const) {
  const storage = storageWindow[key];
  storage.clear();
  Object.defineProperty(globalThis, key, { configurable: true, value: storage });
}

// jsdom 的 navigator.language 預設是 en-US，而既有測試斷言的是中文介面。把測試環境的
// 瀏覽器語言釘成 zh-TW——這是環境預設值，不是 app 狀態；要驗語言偵測的測試自己覆寫它。
if (typeof navigator !== "undefined") {
  Object.defineProperty(navigator, "language", { value: "zh-TW", configurable: true });
}

// jsdom 未實作 matchMedia，Radix Sheet 與 responsive／reduced-motion 判斷都依賴它。
// 預設所有 media query 不匹配；個別測試可覆寫（例如窄螢幕或 prefers-reduced-motion）。
// node 環境的測試（如 build.test.ts）沒有 window，跳過。
if (typeof window !== "undefined") {
  // jsdom 未實作 Pointer Capture 與 scrollIntoView，Radix Select 開關與捲到選中項時
  // 會直接呼叫。缺一即拋 TypeError——環境缺口，補 no-op stub。
  const proto = window.Element.prototype as unknown as Record<string, unknown>;
  if (!proto.hasPointerCapture) proto.hasPointerCapture = () => false;
  if (!proto.setPointerCapture) proto.setPointerCapture = () => {};
  if (!proto.releasePointerCapture) proto.releasePointerCapture = () => {};
  if (!proto.scrollIntoView) proto.scrollIntoView = () => {};

  // jsdom 未實作 ResizeObserver；Radix Checkbox 等原語內部依賴它。no-op stub 即可
  //（測試不斷言尺寸觀察行為）。
  if (!window.ResizeObserver) {
    window.ResizeObserver = class {
      observe() {}
      unobserve() {}
      disconnect() {}
    } as unknown as typeof ResizeObserver;
  }
  if (!window.matchMedia) {
    window.matchMedia = (query: string): MediaQueryList =>
      ({
        matches: false,
        media: query,
        onchange: null,
        addEventListener: () => {},
        removeEventListener: () => {},
        addListener: () => {},
        removeListener: () => {},
        dispatchEvent: () => false,
      }) as unknown as MediaQueryList;
  }
}
