import { describe, expect, it, vi } from "vitest";
import { fireEvent, render as rtlRender, screen, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { AppSettingsView } from "../views/AppSettingsView";
import { APP_MESSAGES } from "../i18n/messages";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);

function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const servers = {
  connections: [],
  phases: {},
  onAdd: vi.fn().mockResolvedValue(undefined),
  onLogin: vi.fn(),
  onSubmitPat: vi.fn(),
  onLogout: vi.fn(),
  onRemove: vi.fn(),
  onRefresh: vi.fn(),
};

describe("AppSettingsView 資訊架構", () => {
  it("頁簽依序為本機設定、伺服器且預設本機設定，內容含介面語言卡與裝置本機註記", () => {
    render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={vi.fn()}
        servers={servers}
      />,
    );

    const tabs = screen.getAllByRole("tab");
    expect(tabs.map((tab) => tab.textContent)).toEqual(["本機設定", "伺服器"]);
    expect(tabs[0].getAttribute("data-state")).toBe("active");
    expect(screen.getByTestId("ui-locale-card")).toBeTruthy();
    expect(screen.getByTestId("local-note").textContent).toContain("僅存於此裝置");

    fireEvent.mouseDown(within(document.body).getByRole("tab", { name: "伺服器" }));
    expect(screen.getByTestId("servers-card")).toBeTruthy();
  });

  it("切換 UI 語言即回呼本機偏好，且不出現已拆除的系統匣樣式卡", () => {
    const onLocalePrefChange = vi.fn();
    render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={onLocalePrefChange}
      />,
    );

    const group = screen.getByTestId("ui-locale");
    fireEvent.click(within(group).getByText("English"));
    expect(onLocalePrefChange).toHaveBeenCalledWith("en");
    fireEvent.click(within(group).getByText(/跟隨系統/));
    expect(onLocalePrefChange).toHaveBeenCalledWith(null);
    expect(screen.queryByTestId("tray-style-card")).toBeNull();
    expect(screen.queryByText("系統匣樣式")).toBeNull();
  });

  it("面板建立失敗時，本機設定簽以獨立警示行浮出錯誤", () => {
    render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={vi.fn()}
        trayPanelError="tray panel window creation failed: boom"
      />,
    );

    const alert = screen.getByRole("alert");
    expect(alert.textContent).toContain("tray panel window creation failed: boom");
    expect(screen.queryByTestId("tray-style-card")).toBeNull();
  });
});
