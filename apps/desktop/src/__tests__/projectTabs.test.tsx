// ProjectTabs 分頁列元件（design D10）：active teal 標示、✕ 僅 active 顯示、
// 「＋」入口、徽章＋tooltip、失效分頁錯誤態。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { ProjectTabs } from "../components/ProjectTabs";
import { APP_MESSAGES } from "../i18n/messages";
import type { ProjectTab } from "../tabs";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const tabs: ProjectTab[] = [
  { root: "C:\\proj\\alpha", name: "alpha", badge: 3 },
  { root: "C:\\proj\\beta", name: "beta", badge: 2 },
];

describe("ProjectTabs", () => {
  it("renders one tab per project and marks the active one", () => {
    render(<ProjectTabs tabs={tabs} activeRoot="C:\proj\alpha" tabErrors={{}} />);
    const alpha = screen.getByText("alpha").closest("[data-tab]") as HTMLElement;
    const beta = screen.getByText("beta").closest("[data-tab]") as HTMLElement;
    expect(alpha.getAttribute("data-active")).toBe("true");
    expect(beta.getAttribute("data-active")).toBe("false");
  });

  it("shows the in-progress badge on each tab", () => {
    render(<ProjectTabs tabs={tabs} activeRoot="C:\proj\alpha" tabErrors={{}} />);
    const alpha = screen.getByText("alpha").closest("[data-tab]") as HTMLElement;
    expect(within(alpha).getByText("3")).toBeTruthy();
    const beta = screen.getByText("beta").closest("[data-tab]") as HTMLElement;
    expect(within(beta).getByText("2")).toBeTruthy();
  });

  it("clicking a background tab fires onActivate with its root", () => {
    const onActivate = vi.fn();
    render(
      <ProjectTabs tabs={tabs} activeRoot="C:\proj\alpha" tabErrors={{}} onActivate={onActivate} />,
    );
    fireEvent.click(screen.getByText("beta"));
    expect(onActivate).toHaveBeenCalledWith("C:\\proj\\beta");
  });

  it("close button appears on the active tab and fires onClose without activating", () => {
    const onClose = vi.fn();
    const onActivate = vi.fn();
    render(
      <ProjectTabs
        tabs={tabs}
        activeRoot="C:\proj\alpha"
        tabErrors={{}}
        onClose={onClose}
        onActivate={onActivate}
      />,
    );
    const alpha = screen.getByText("alpha").closest("[data-tab]") as HTMLElement;
    fireEvent.click(within(alpha).getByLabelText("關閉分頁"));
    expect(onClose).toHaveBeenCalledWith("C:\\proj\\alpha");
    expect(onActivate).not.toHaveBeenCalled();
  });

  it("the + button fires onOpen (dialog entry)", () => {
    const onOpen = vi.fn();
    render(<ProjectTabs tabs={tabs} activeRoot="C:\proj\alpha" tabErrors={{}} onOpen={onOpen} />);
    fireEvent.click(screen.getByLabelText("開啟專案"));
    expect(onOpen).toHaveBeenCalled();
  });

  it("a tab with an error renders the error state and offers removal", () => {
    const onClose = vi.fn();
    render(
      <ProjectTabs
        tabs={tabs}
        activeRoot="C:\proj\alpha"
        tabErrors={{ "C:\\proj\\beta": "cannot open" }}
        onClose={onClose}
      />,
    );
    const beta = screen.getByText("beta").closest("[data-tab]") as HTMLElement;
    expect(beta.getAttribute("data-error")).toBe("true");
    fireEvent.click(within(beta).getByLabelText("自分頁移除"));
    expect(onClose).toHaveBeenCalledWith("C:\\proj\\beta");
  });
});
