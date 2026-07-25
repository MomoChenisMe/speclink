// jsdom 未實作 matchMedia，Radix Sheet 與 responsive／reduced-motion 判斷都依賴它。
// 預設所有 media query 不匹配；個別測試可覆寫（例如窄螢幕或 prefers-reduced-motion）。
// node 環境的測試（如 build.test.ts）沒有 window，跳過。
if (typeof window !== "undefined") {
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
