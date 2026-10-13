// 外觀偏好的開機套用（desktop-settings-mode design D4）：主視窗在 index.html 的 inline script
// 先套 data-theme（module script 之前，避免先閃一次系統主題）；系統匣面板另把原生視窗主題
// 設成同一偏好——vibrancy 材質跟原生視窗外觀走，不同步就會淺色字疊深色玻璃。
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { THEME_STORAGE_KEY } from "@speclink/ui";

import indexHtml from "../../index.html?raw";
import trayPanelCapsRaw from "../../src-tauri/capabilities/tray-panel.json?raw";

const tauri = vi.hoisted(() => ({
  setTheme: vi.fn(() => Promise.resolve()),
  handlers: new Map<string, (event: { payload: unknown }) => void>(),
}));
vi.mock("@tauri-apps/api/event", () => ({
  emit: vi.fn(() => Promise.resolve()),
  listen: vi.fn(async (name: string, handler: (event: { payload: unknown }) => void) => {
    tauri.handlers.set(name, handler);
    return () => {};
  }),
}));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ setTheme: tauri.setTheme, setSize: vi.fn(() => Promise.resolve()) }),
  LogicalSize: class {},
}));
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({ writeText: vi.fn() }));

beforeEach(() => {
  localStorage.clear();
  document.documentElement.removeAttribute("data-theme");
});
afterEach(() => {
  localStorage.clear();
  document.documentElement.removeAttribute("data-theme");
});

describe("主視窗開機腳本", () => {
  const head = indexHtml.slice(0, indexHtml.indexOf("</head>"));
  const boot = head.match(/<script>([\s\S]*?)<\/script>/)?.[1] ?? "";
  const run = () => new Function(boot)();

  it("在 head 的 inline script 讀同一個偏好鍵", () => {
    expect(boot).toContain(`"${THEME_STORAGE_KEY}"`);
  });

  it.each(["light", "dark"])("偏好 %s 套成 data-theme", (pref) => {
    localStorage.setItem(THEME_STORAGE_KEY, pref);
    run();
    expect(document.documentElement.getAttribute("data-theme")).toBe(pref);
  });

  it("缺鍵或壞值跟隨系統（不設屬性）", () => {
    run();
    expect(document.documentElement.hasAttribute("data-theme")).toBe(false);
    localStorage.setItem(THEME_STORAGE_KEY, "system");
    run();
    expect(document.documentElement.hasAttribute("data-theme")).toBe(false);
  });
});

describe("系統匣面板", () => {
  it("啟動與主視窗切換時，網頁與原生視窗主題都套同一偏好", async () => {
    localStorage.setItem(THEME_STORAGE_KEY, "light");
    await import("../panel/main");
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
    expect(tauri.setTheme).toHaveBeenLastCalledWith("light");

    tauri.handlers.get("speclink:theme-changed")!({ payload: "dark" });
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
    expect(tauri.setTheme).toHaveBeenLastCalledWith("dark");

    // 跟隨系統：原生視窗也交回系統外觀。
    tauri.handlers.get("speclink:theme-changed")!({ payload: null });
    expect(document.documentElement.hasAttribute("data-theme")).toBe(false);
    expect(tauri.setTheme).toHaveBeenLastCalledWith(null);
  });

  it("面板能力檔給面板視窗 set-theme 權限", () => {
    const caps = JSON.parse(trayPanelCapsRaw) as { windows: string[]; permissions: string[] };
    expect(caps.windows).toContain("tray-panel");
    expect(caps.permissions).toContain("core:window:allow-set-theme");
  });
});
