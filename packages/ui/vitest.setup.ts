// jsdom 未實作 Pointer Capture 與 scrollIntoView，Radix Select 在開關與捲到選中項時
// 會直接呼叫這三支。缺一即在測試中拋 TypeError——這是環境缺口，不是元件行為，
// 所以補 no-op stub 而非改元件。
if (typeof window !== "undefined") {
  const proto = window.Element.prototype as unknown as Record<string, unknown>;
  if (!proto.hasPointerCapture) proto.hasPointerCapture = () => false;
  if (!proto.setPointerCapture) proto.setPointerCapture = () => {};
  if (!proto.releasePointerCapture) proto.releasePointerCapture = () => {};
  if (!proto.scrollIntoView) proto.scrollIntoView = () => {};

  // Radix Presence 的 animationend 處理以 CSS.escape 比對動畫名；jsdom 沒有 CSS 全域，
  // 缺了關閉動畫結束事件會在 handler 內拋錯、面板永不卸載。
  if (!window.CSS) {
    window.CSS = { escape: (value: string) => value } as unknown as typeof CSS;
  }

  // Radix Select 的 Content 以 ResizeObserver 量測可用高度。
  if (!window.ResizeObserver) {
    window.ResizeObserver = class {
      observe() {}
      unobserve() {}
      disconnect() {}
    } as unknown as typeof ResizeObserver;
  }
}
