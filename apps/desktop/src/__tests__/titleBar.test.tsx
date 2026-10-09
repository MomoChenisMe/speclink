// 三平台標題列（desktop-app「隱藏原生標題列與三平台視窗殼」，design D1／D6）。
import { beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render as rtlRender, screen } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { LeftTitleBar, MainTitleBar } from "../components/TitleBar";
import { APP_MESSAGES } from "../i18n/messages";
import defaultCapsRaw from "../../src-tauri/capabilities/default.json?raw";
import baseConfRaw from "../../src-tauri/tauri.conf.json?raw";
import macConfRaw from "../../src-tauri/tauri.macos.conf.json?raw";
import winConfRaw from "../../src-tauri/tauri.windows.conf.json?raw";

vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: vi.fn() }));

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);

function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

function mockWindow(maximized = false) {
  let onResized: (() => void) | undefined;
  const win = {
    minimize: vi.fn().mockResolvedValue(undefined),
    toggleMaximize: vi.fn().mockResolvedValue(undefined),
    close: vi.fn().mockResolvedValue(undefined),
    isMaximized: vi.fn().mockResolvedValue(maximized),
    onResized: vi.fn(async (handler: () => void) => {
      onResized = handler;
      return () => {};
    }),
  };
  vi.mocked(getCurrentWindow).mockReturnValue(win as unknown as ReturnType<typeof getCurrentWindow>);
  return { win, resize: () => onResized?.() };
}

beforeEach(() => {
  vi.mocked(getCurrentWindow).mockReset();
});

describe("LeftTitleBar", () => {
  it("專案名左緣對齊下方專案欄導覽文字（98px）、可拖曳、純文字標題 hover 顯示路徑", () => {
    render(<LeftTitleBar title="speclink" path="/Users/a/speclink" menu={<button>⋯</button>} />);
    const bar = screen.getByTestId("left-titlebar");
    // 圖示列 56＋專案欄內距 8＋導覽項內距 10＋圖示 16＋間距 8＝導覽文字左緣。
    expect(bar.className).toContain("pl-[98px]");
    expect(bar.getAttribute("data-tauri-drag-region")).toBe("deep");
    expect(screen.getByText("speclink").getAttribute("title")).toBe("/Users/a/speclink");
    expect(screen.getByRole("button", { name: "⋯" })).toBeTruthy();
  });

  it("下緣與主區頂列同一條細線，頂列橫貫整個視窗", () => {
    render(<LeftTitleBar />);
    const left = screen.getByTestId("left-titlebar");
    expect(left.className).toContain("border-b");
    expect(left.className).toContain("border-border");
    expect(left.className).toContain("h-12");
  });
});

describe("MainTitleBar", () => {
  it("麵包屑三段渲染、第一段可點、最後一段深字", () => {
    const onProject = vi.fn();
    render(
      <MainTitleBar
        platform="macos"
        crumbs={[{ label: "speclink", onClick: onProject }, { label: "規格" }, { label: "desktop-app" }]}
      />,
    );
    const bar = screen.getByTestId("main-titlebar");
    expect(bar.getAttribute("data-tauri-drag-region")).toBe("deep");
    fireEvent.click(screen.getByRole("button", { name: "speclink" }));
    expect(onProject).toHaveBeenCalledTimes(1);
    expect(screen.getByText("規格").tagName).not.toBe("BUTTON");
    expect(screen.getByText("desktop-app").className).toContain("text-foreground");
    expect(screen.getByText("規格").className).not.toContain("text-foreground");
  });

  it("macOS 與 Linux 無自繪視窗鈕", () => {
    const { unmount } = render(<MainTitleBar platform="macos" crumbs={[{ label: "A" }]} />);
    expect(screen.queryByRole("button", { name: "縮到最小" })).toBeNull();
    unmount();
    render(<MainTitleBar platform="linux" crumbs={[]} />);
    expect(screen.queryByRole("button", { name: "關閉視窗" })).toBeNull();
    expect(getCurrentWindow).not.toHaveBeenCalled();
  });

  it("Windows：三顆視窗鈕各呼叫對應的 window API", async () => {
    const { win } = mockWindow(false);
    render(<MainTitleBar platform="windows" crumbs={[{ label: "A" }]} />);
    await act(async () => {});

    fireEvent.click(screen.getByRole("button", { name: "縮到最小" }));
    expect(win.minimize).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("button", { name: "最大化" }));
    expect(win.toggleMaximize).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("button", { name: "關閉視窗" }));
    expect(win.close).toHaveBeenCalledTimes(1);
  });

  it("Windows：最大化後鈕換成「還原」（依 isMaximized 與 onResized）", async () => {
    const { win, resize } = mockWindow(false);
    render(<MainTitleBar platform="windows" crumbs={[]} />);
    await act(async () => {});
    expect(screen.getByRole("button", { name: "最大化" })).toBeTruthy();

    win.isMaximized.mockResolvedValue(true);
    await act(async () => {
      resize();
    });
    expect(screen.getByRole("button", { name: "還原" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "最大化" })).toBeNull();
  });

  it("Windows：API 被拒時按鈕不拋錯、記 console.error", async () => {
    const { win } = mockWindow(false);
    win.minimize.mockRejectedValue(new Error("not allowed"));
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});
    render(<MainTitleBar platform="windows" crumbs={[]} />);
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "縮到最小" }));
    });
    expect(spy).toHaveBeenCalled();
    spy.mockRestore();
  });
});

// 設定檔守門：視窗鈕與拖曳所需權限、平台覆蓋檔不得因合併取代掉主視窗尺寸。
describe("Tauri 視窗設定", () => {

  it("capabilities/default.json 含拖曳與三顆視窗鈕所需的五條權限", () => {
    const { permissions } = JSON.parse(defaultCapsRaw);
    for (const p of [
      "core:window:allow-start-dragging",
      "core:window:allow-minimize",
      "core:window:allow-toggle-maximize",
      "core:window:allow-close",
      "core:window:allow-is-maximized",
    ]) {
      expect(permissions).toContain(p);
    }
  });

  it("平台覆蓋檔的 main 視窗保留基礎設定的標題與尺寸（陣列整個取代）", () => {
    const base = JSON.parse(baseConfRaw).app.windows[0];
    const mac = JSON.parse(macConfRaw).app.windows[0];
    const win = JSON.parse(winConfRaw).app.windows[0];
    for (const override of [mac, win]) {
      expect(override).toMatchObject(base);
    }
    expect(mac).toMatchObject({
      titleBarStyle: "Overlay",
      hiddenTitle: true,
      trafficLightPosition: { x: 16, y: 26 },
    });
    expect(mac.decorations).toBeUndefined();
    expect(win).toMatchObject({ decorations: false, shadow: true });
  });
});
