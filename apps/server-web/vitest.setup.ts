// jsdom 未實作 matchMedia，Radix Sheet 與 responsive／reduced-motion 判斷都依賴它。
// 預設所有 media query 不匹配；個別測試可覆寫（例如窄螢幕或 prefers-reduced-motion）。
// node 環境的測試（如 build.test.ts）沒有 window，跳過。
if (typeof window !== "undefined") {
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
