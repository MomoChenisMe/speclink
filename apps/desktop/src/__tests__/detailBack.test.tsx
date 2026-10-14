// 詳情頁的返回鈕與返回快捷鍵（spec desktop-app「詳情頁的導覽與返回」；desktop-readonly-detail-pages design D8）。
// ‹ n／m › 的案例隨元件搬到 packages/ui 的 detailNav.test.tsx（server-web-detail-pages design D3）。
import { describe, expect, it, vi, afterEach } from "vitest";
import { cleanup, fireEvent, render as rtlRender, screen } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { DetailBack, isBackShortcut } from "../components/DetailBack";
import { APP_MESSAGES } from "../i18n/messages";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

afterEach(cleanup);

describe("DetailBack", () => {
  it("名稱為「返回<頁名>」、提示標示 Esc 與返回快捷鍵，點擊呼叫 onBack", () => {
    const onBack = vi.fn();
    render(<DetailBack page="規格" keys="⌘[" onBack={onBack} />);
    const btn = screen.getByRole("button", { name: "返回規格" });
    expect(btn.getAttribute("title")).toBe("返回規格（Esc 或 ⌘[）");
    expect(btn.querySelector(".lucide-arrow-left")).toBeTruthy();
    fireEvent.click(btn);
    expect(onBack).toHaveBeenCalledTimes(1);
  });
});

describe("isBackShortcut", () => {
  const key = (over: Partial<KeyboardEvent>) =>
    ({ key: "", code: "", metaKey: false, altKey: false, ctrlKey: false, shiftKey: false, ...over }) as KeyboardEvent;

  it.each([
    ["macOS ⌘[", "macos", { metaKey: true, key: "[", code: "BracketLeft" }, true],
    ["macOS ⌘[（中文輸入法：e.key 不是「[」）", "macos", { metaKey: true, key: "「", code: "BracketLeft" }, true],
    ["macOS ⌘←", "macos", { metaKey: true, key: "ArrowLeft", code: "ArrowLeft" }, true],
    ["macOS ⌥←（文字跳字，不是返回）", "macos", { altKey: true, key: "ArrowLeft", code: "ArrowLeft" }, false],
    ["macOS ⇧⌘[（切分頁慣例，不是返回）", "macos", { metaKey: true, shiftKey: true, key: "{", code: "BracketLeft" }, false],
    ["Windows Alt+←", "windows", { altKey: true, key: "ArrowLeft", code: "ArrowLeft" }, true],
    ["Windows ⊞+[", "windows", { metaKey: true, key: "[", code: "BracketLeft" }, false],
    ["Linux Ctrl+Alt+←", "linux", { altKey: true, ctrlKey: true, key: "ArrowLeft", code: "ArrowLeft" }, false],
  ] as const)("%s", (_name, platform, over, expected) => {
    expect(isBackShortcut(key(over), platform)).toBe(expected);
  });
});
