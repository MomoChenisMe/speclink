// ProjectTabs 分頁列元件（design D10）：active teal 標示、✕ 僅 active 顯示、
// 「＋」入口、徽章＋tooltip、失效分頁錯誤態。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { ProjectTabs } from "../components/ProjectTabs";
import { APP_MESSAGES } from "../i18n/messages";
import type { ProjectTab } from "../tabs";
import type { RemoteWorkspaceRecoveryState, WorkspaceLocator } from "../session";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const local = (root: string): WorkspaceLocator => ({ kind: "local", root });

const tabs: ProjectTab[] = [
  { locator: local("C:\\proj\\alpha"), name: "alpha", badge: 3 },
  { locator: local("C:\\proj\\beta"), name: "beta", badge: 2 },
];

describe("ProjectTabs", () => {
  it("renders one tab per project and marks the active one", () => {
    render(<ProjectTabs tabs={tabs} activeKey="local:C:\proj\alpha" tabErrors={{}} />);
    const alpha = screen.getByText("alpha").closest("[data-tab]") as HTMLElement;
    const beta = screen.getByText("beta").closest("[data-tab]") as HTMLElement;
    expect(alpha.getAttribute("data-active")).toBe("true");
    expect(beta.getAttribute("data-active")).toBe("false");
  });

  it("shows the in-progress badge on each tab", () => {
    render(<ProjectTabs tabs={tabs} activeKey="local:C:\proj\alpha" tabErrors={{}} />);
    const alpha = screen.getByText("alpha").closest("[data-tab]") as HTMLElement;
    expect(within(alpha).getByText("3")).toBeTruthy();
    const beta = screen.getByText("beta").closest("[data-tab]") as HTMLElement;
    expect(within(beta).getByText("2")).toBeTruthy();
  });

  it("clicking a background tab fires onActivate with its root", () => {
    const onActivate = vi.fn();
    render(
      <ProjectTabs tabs={tabs} activeKey="local:C:\proj\alpha" tabErrors={{}} onActivate={onActivate} />,
    );
    fireEvent.click(screen.getByText("beta"));
    expect(onActivate).toHaveBeenCalledWith("local:C:\\proj\\beta");
  });

  it("close button appears on the active tab and fires onClose without activating", () => {
    const onClose = vi.fn();
    const onActivate = vi.fn();
    render(
      <ProjectTabs
        tabs={tabs}
        activeKey="local:C:\proj\alpha"
        tabErrors={{}}
        onClose={onClose}
        onActivate={onActivate}
      />,
    );
    const alpha = screen.getByText("alpha").closest("[data-tab]") as HTMLElement;
    fireEvent.click(within(alpha).getByLabelText("關閉分頁"));
    expect(onClose).toHaveBeenCalledWith("local:C:\\proj\\alpha");
    expect(onActivate).not.toHaveBeenCalled();
  });

  it("the + button fires onOpen (dialog entry)", () => {
    const onOpen = vi.fn();
    render(<ProjectTabs tabs={tabs} activeKey="local:C:\proj\alpha" tabErrors={{}} onOpen={onOpen} />);
    fireEvent.click(screen.getByLabelText("新增 Workspace"));
    expect(onOpen).toHaveBeenCalled();
  });

  it("a tab with an error renders the error state and offers removal", () => {
    const onClose = vi.fn();
    render(
      <ProjectTabs
        tabs={tabs}
        activeKey="local:C:\proj\alpha"
        tabErrors={{ "local:C:\\proj\\beta": "cannot open" }}
        onClose={onClose}
      />,
    );
    const beta = screen.getByText("beta").closest("[data-tab]") as HTMLElement;
    expect(beta.getAttribute("data-error")).toBe("true");
    fireEvent.click(within(beta).getByLabelText("自分頁移除"));
    expect(onClose).toHaveBeenCalledWith("local:C:\\proj\\beta");
  });

  it("remote tabs carry a cloud marker and the Project/Repo name（remote-data-source §10.5）", () => {
    const remoteTabs: ProjectTab[] = [
      ...tabs,
      {
        locator: {
          kind: "remote",
          connectionId: "c1",
          projectId: "demo",
          repoId: "backend",
          checkoutRoot: "/work/backend",
        },
        name: "Demo/backend",
        badge: null,
      },
    ];
    render(<ProjectTabs tabs={remoteTabs} activeKey="remote:c1/demo/backend" tabErrors={{}} />);
    const remote = screen.getByText("Demo/backend").closest("[data-tab]") as HTMLElement;
    const cloud = remote.querySelector("[data-cloud]") as HTMLElement;
    expect(cloud).toBeTruthy();
    // cloud 以主色加深——與 local 的 folder 圖示形成視覺區分。
    expect(cloud.getAttribute("class")).toContain("text-primary");
    expect(remote.getAttribute("title")).toBe("已連接 checkout：/work/backend");
    expect(remote.querySelector("[data-folder]")).toBeNull();
    // local 分頁長 folder 圖示、不長 cloud。
    const alpha = screen.getByText("alpha").closest("[data-tab]") as HTMLElement;
    expect(alpha.querySelector("[data-cloud]")).toBeNull();
    expect(alpha.querySelector("[data-folder]")).toBeTruthy();
  });

  it("remote error tab remains a selectable active destination with one concise status", () => {
    const remoteTabs: ProjectTab[] = [
      {
        locator: { kind: "remote", connectionId: "c1", projectId: "demo", repoId: "backend" },
        name: "Demo/backend",
        badge: null,
      },
    ];
    const recoveryStates: Record<string, RemoteWorkspaceRecoveryState> = {
      "remote:c1/demo/backend": {
        status: "error",
        failure: {
          kind: "unreachable",
          message: "server unreachable — technical detail only",
          reason: null,
          status: null,
        },
      },
    };
    render(
      <ProjectTabs
        tabs={remoteTabs}
        activeKey="remote:c1/demo/backend"
        tabErrors={{}}
        recoveryStates={recoveryStates}
      />,
    );

    const tab = screen.getByRole("tab", { name: /Demo\/backend/ });
    expect(tab.getAttribute("aria-selected")).toBe("true");
    expect(tab.getAttribute("aria-disabled")).not.toBe("true");
    expect(tab.getAttribute("data-status")).toBe("error");
    expect(tab.getAttribute("title")).toBe("無法連線");
    expect(tab.getAttribute("title")).not.toContain("technical detail");
    expect(tab.querySelectorAll("[data-tab-status]")).toHaveLength(1);
    expect(tab.getAttribute("class")).not.toContain("opacity-60");
  });

  it("background recovery tab supports mouse and keyboard activation", () => {
    const onActivate = vi.fn();
    const remoteTabs: ProjectTab[] = [
      {
        locator: { kind: "remote", connectionId: "c1", projectId: "demo", repoId: "backend" },
        name: "Demo/backend",
        badge: null,
      },
    ];
    render(
      <ProjectTabs
        tabs={remoteTabs}
        activeKey={null}
        tabErrors={{}}
        recoveryStates={{
          "remote:c1/demo/backend": { status: "restoring", failure: null },
        }}
        onActivate={onActivate}
      />,
    );
    const tab = screen.getByRole("tab", { name: /Demo\/backend/ });
    fireEvent.keyDown(tab, { key: "Enter" });
    fireEvent.click(tab);
    expect(onActivate).toHaveBeenCalledTimes(2);
    expect(tab.getAttribute("data-status")).toBe("restoring");
    expect(tab.querySelectorAll("[data-tab-status]")).toHaveLength(1);
  });
});
