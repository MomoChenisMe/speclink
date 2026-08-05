// 更新通知列（desktop-app spec「桌面自動更新」，design D6）：available 徵詢
// （顯示版本、同意／稍後）、restartPending 重啟提示、error 呈現錯誤；其餘狀態
// （閒置／檢查中／已最新／無法檢查）不佔畫面——檢查結果的行內呈現歸設定頁。
import { describe, expect, it, vi } from "vitest";
import { fireEvent, render as rtlRender, screen } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider, SEMANTIC_TONE } from "@speclink/ui";

import { UpdateBanner } from "../components/UpdateBanner";
import { APP_MESSAGES } from "../i18n/messages";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);

function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

function handlers() {
  return { onAccept: vi.fn(), onDismiss: vi.fn(), onRelaunch: vi.fn() };
}

describe("UpdateBanner", () => {
  it("發現新版：顯示目標版本並提供更新／稍後；同意才回呼下載", () => {
    const h = handlers();
    render(<UpdateBanner state={{ phase: "available", version: "0.2.0" }} {...h} />);

    expect(screen.getByTestId("update-banner").textContent).toContain("0.2.0");

    fireEvent.click(screen.getByRole("button", { name: "更新" }));
    expect(h.onAccept).toHaveBeenCalledTimes(1);
    expect(h.onRelaunch).not.toHaveBeenCalled();
  });

  it("稍後：回呼 dismiss、不觸發下載", () => {
    const h = handlers();
    render(<UpdateBanner state={{ phase: "available", version: "0.2.0" }} {...h} />);

    fireEvent.click(screen.getByRole("button", { name: "稍後" }));
    expect(h.onDismiss).toHaveBeenCalledTimes(1);
    expect(h.onAccept).not.toHaveBeenCalled();
  });

  it("待重啟：提示重啟並可立即重啟", () => {
    const h = handlers();
    render(<UpdateBanner state={{ phase: "restartPending", version: "0.2.0" }} {...h} />);

    fireEvent.click(screen.getByRole("button", { name: "立即重啟" }));
    expect(h.onRelaunch).toHaveBeenCalledTimes(1);
  });

  it("錯誤態：呈現錯誤訊息並可關閉", () => {
    const h = handlers();
    render(<UpdateBanner state={{ phase: "error", message: "invalid signature" }} {...h} />);

    expect(screen.getByTestId("update-banner").textContent).toContain("invalid signature");
    fireEvent.click(screen.getByRole("button", { name: "關閉" }));
    expect(h.onDismiss).toHaveBeenCalledTimes(1);
  });

  it("底色中性、狀態由圖示語意色承載：更新失敗為紅、有新版為藍", () => {
    // spec「錯誤態以紅呈現」：更新失敗是錯誤，不能穿琥珀；整片底色上色會壓過
    // 分頁內容的層級，改為中性底＋語意色圖示。
    const h = handlers();
    const { unmount } = render(
      <UpdateBanner state={{ phase: "error", message: "invalid signature" }} {...h} />,
    );
    const errorBanner = screen.getByTestId("update-banner");
    expect(errorBanner.className).not.toContain("amber");
    expect(errorBanner.className).not.toContain("bg-primary");
    expect(errorBanner.querySelector("svg")?.getAttribute("class")).toContain("destructive");
    unmount();

    render(<UpdateBanner state={{ phase: "available", version: "0.2.0" }} {...h} />);
    const banner = screen.getByTestId("update-banner");
    expect(banner.className).not.toContain("bg-primary");
    expect(banner.querySelector("svg")?.getAttribute("class")).toContain(SEMANTIC_TONE.inProgress);
  });

  it("閒置與檢查結果狀態不渲染任何內容", () => {
    const h = handlers();
    for (const state of [
      { phase: "idle" } as const,
      { phase: "checking", manual: true } as const,
      { phase: "upToDate" } as const,
      { phase: "checkFailed" } as const,
    ]) {
      const { unmount } = render(<UpdateBanner state={state} {...h} />);
      expect(screen.queryByTestId("update-banner")).toBeNull();
      unmount();
    }
  });
});
