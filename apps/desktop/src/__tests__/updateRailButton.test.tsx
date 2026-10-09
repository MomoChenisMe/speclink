// 「有新版本」圖示鈕（desktop-app spec「桌面自動更新」；desktop-notice-relocation design D2）：
// 圖示列齒輪上方的狀態鈕——待同意＝下載箭頭＋琥珀點、下載中＝轉圈、待重啟＝重啟箭頭＋琥珀點、
// 失敗＝紅色警示；其他狀態不出現。提示文字帶目標版本與「到設定 › 軟體更新」；點擊進設定頁。
import { describe, expect, it, vi } from "vitest";
import { fireEvent, render as rtlRender, screen } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider, TooltipProvider } from "@speclink/ui";

import { UpdateRailButton } from "../components/UpdateRailButton";
import type { UpdaterState } from "../core/updater";
import { APP_MESSAGES } from "../i18n/messages";

const wrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    <TooltipProvider>{children}</TooltipProvider>
  </I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper });
}

const button = () => screen.getByTestId("update-rail-button") as HTMLButtonElement;
const dot = () => button().querySelector("[data-update-dot]");
const icon = () => button().querySelector("svg") as SVGElement;

describe("UpdateRailButton", () => {
  it("待同意：下載箭頭＋右上琥珀點；aria-label 含版號與「到設定 › 軟體更新」", () => {
    render(<UpdateRailButton state={{ phase: "available", version: "0.5.1" }} onClick={vi.fn()} />);
    expect(icon().classList.contains("lucide-download")).toBe(true);
    expect(dot()?.className).toContain("bg-status-warning");
    expect(button().getAttribute("aria-label")).toBe("有新版本 0.5.1 — 到設定 › 軟體更新");
  });

  it("下載中：轉圈、無點", () => {
    render(<UpdateRailButton state={{ phase: "downloading", version: "0.5.1" }} onClick={vi.fn()} />);
    expect(icon().classList.contains("lucide-loader-circle")).toBe(true);
    expect(icon().classList.contains("animate-spin")).toBe(true);
    expect(dot()).toBeNull();
    expect(button().getAttribute("aria-label")).toBe("正在下載 0.5.1");
  });

  it("待重啟：重啟箭頭＋琥珀點", () => {
    render(<UpdateRailButton state={{ phase: "restartPending", version: "0.5.1" }} onClick={vi.fn()} />);
    expect(icon().classList.contains("lucide-rotate-cw")).toBe(true);
    expect(dot()?.className).toContain("bg-status-warning");
    expect(button().getAttribute("aria-label")).toBe("0.5.1 已就緒，到設定安裝並重新啟動");
  });

  it("失敗：紅色警示、無點（spec「錯誤態以紅呈現」）", () => {
    render(<UpdateRailButton state={{ phase: "error", message: "invalid signature" }} onClick={vi.fn()} />);
    expect(icon().classList.contains("lucide-triangle-alert")).toBe(true);
    expect(icon().getAttribute("class")).toContain("text-destructive");
    expect(dot()).toBeNull();
    expect(button().getAttribute("aria-label")).toBe("更新失敗 — 到設定查看");
  });

  it.each<[string, UpdaterState]>([
    ["閒置", { phase: "idle" }],
    ["檢查中", { phase: "checking", manual: true }],
    ["已是最新", { phase: "upToDate" }],
    ["檢查失敗", { phase: "checkFailed" }],
  ])("%s：不渲染", (_label, state) => {
    render(<UpdateRailButton state={state} onClick={vi.fn()} />);
    expect(screen.queryByTestId("update-rail-button")).toBeNull();
  });

  it("點擊呼叫 onClick", () => {
    const onClick = vi.fn();
    render(<UpdateRailButton state={{ phase: "available", version: "0.5.1" }} onClick={onClick} />);
    fireEvent.click(button());
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("聚焦時顯示 tooltip，文案帶版號", async () => {
    render(<UpdateRailButton state={{ phase: "available", version: "0.5.1" }} onClick={vi.fn()} />);
    fireEvent.focus(button());
    const tip = await screen.findByRole("tooltip");
    expect(tip.textContent).toContain("0.5.1");
  });
});
