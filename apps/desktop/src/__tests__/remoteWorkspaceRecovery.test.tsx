import { cleanup, fireEvent, render as rtlRender, screen } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import { I18nProvider } from "@speclink/ui";

import { RemoteWorkspaceRecovery } from "../components/RemoteWorkspaceRecovery";
import { APP_MESSAGES } from "../i18n/messages";
import type { ConnectionView } from "../adapter/connections";
import type { RemoteWorkspaceRecoveryState } from "../session";
import type { ProjectTab } from "../tabs";

const tab: ProjectTab = {
  locator: {
    kind: "remote",
    connectionId: "c1",
    projectId: "demo",
    repoId: "backend",
  },
  name: "Demo/backend",
  badge: null,
};

const connection: ConnectionView = {
  id: "c1",
  origin: "https://spec.example.test",
  name: "Team Server",
  loggedIn: true,
};

const wrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);

function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper });
}

function errorRecovery(kind: "unreachable" | "needs-reauth" | "access-denied" | "not-found"):
  RemoteWorkspaceRecoveryState {
  return {
    status: "error",
    failure: {
      kind,
      message: "server unreachable — check internal transport diagnostics",
      reason: kind === "needs-reauth" ? "permission_denied" : null,
      status: kind === "needs-reauth" ? 401 : null,
    },
  };
}

describe("RemoteWorkspaceRecovery", () => {
  it("shows a localized recovery destination and progressively discloses technical detail", () => {
    const onRetry = vi.fn();
    const onOpenSettings = vi.fn();
    const onRemove = vi.fn();
    render(
      <RemoteWorkspaceRecovery
        tab={tab}
        recovery={errorRecovery("unreachable")}
        connection={connection}
        onRetry={onRetry}
        onOpenSettings={onOpenSettings}
        onReauthenticate={vi.fn()}
        onRemove={onRemove}
      />,
    );

    expect(screen.getByRole("alert").textContent).toContain("無法連線到伺服器");
    expect(screen.getByText("Demo/backend")).toBeTruthy();
    expect(screen.getByText(/Team Server/).textContent).toContain("spec.example.test");
    expect(screen.getByRole("button", { name: "重新連線" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "伺服器設定" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "自分頁移除" })).toBeTruthy();

    const disclosure = screen.getByRole("button", { name: "技術細節" });
    expect(disclosure.getAttribute("aria-expanded")).toBe("false");
    expect(screen.queryByText(/internal transport diagnostics/)).toBeNull();
    fireEvent.click(disclosure);
    expect(disclosure.getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByText(/internal transport diagnostics/)).not.toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "重新連線" }));
    fireEvent.click(screen.getByRole("button", { name: "伺服器設定" }));
    fireEvent.click(screen.getByRole("button", { name: "自分頁移除" }));
    expect(onRetry).toHaveBeenCalledTimes(1);
    expect(onOpenSettings).toHaveBeenCalledTimes(1);
    expect(onRemove).toHaveBeenCalledTimes(1);
  });

  it("uses reauthentication as the primary action for needs-reauth", () => {
    const onReauthenticate = vi.fn();
    render(
      <RemoteWorkspaceRecovery
        tab={tab}
        recovery={errorRecovery("needs-reauth")}
        connection={connection}
        onRetry={vi.fn()}
        onOpenSettings={vi.fn()}
        onReauthenticate={onReauthenticate}
        onRemove={vi.fn()}
      />,
    );
    expect(screen.getByRole("alert").textContent).toContain("需要重新登入");
    fireEvent.click(screen.getByRole("button", { name: "重新登入" }));
    expect(onReauthenticate).toHaveBeenCalledTimes(1);
  });

  it("announces restoring, disables duplicate retry, and never renders workspace data", () => {
    render(
      <RemoteWorkspaceRecovery
        tab={tab}
        recovery={{ status: "restoring", failure: null }}
        connection={connection}
        onRetry={vi.fn()}
        onOpenSettings={vi.fn()}
        onReauthenticate={vi.fn()}
        onRemove={vi.fn()}
      />,
    );
    expect(screen.getByRole("status").textContent).toContain("正在連線");
    expect(screen.queryByRole("button", { name: "重新連線" })).toBeNull();
    expect(screen.queryByText("previous-workspace-change")).toBeNull();
  });

  it("狀態語意色：還原中為藍、存取遭拒為紅、需重新登入維持琥珀", () => {
    // spec「進行中以藍呈現」「錯誤態以紅呈現」：舊版一律塗琥珀，「等一下就好」
    // 與「這個工作區你進不去」看起來同一級。
    const { unmount } = render(
      <RemoteWorkspaceRecovery
        tab={tab}
        recovery={{ status: "restoring", failure: null }}
        connection={connection}
        onRetry={vi.fn()}
        onOpenSettings={vi.fn()}
        onReauthenticate={vi.fn()}
        onRemove={vi.fn()}
      />,
    );
    const spinner = screen.getByRole("status").querySelector("div") as HTMLElement;
    expect(spinner.className).toContain("sky");
    expect(spinner.className).not.toContain("primary");
    unmount();

    const renderFailure = (kind: "access-denied" | "needs-reauth") => {
      render(
        <RemoteWorkspaceRecovery
          tab={tab}
          recovery={errorRecovery(kind)}
          connection={connection}
          onRetry={vi.fn()}
          onOpenSettings={vi.fn()}
          onReauthenticate={vi.fn()}
          onRemove={vi.fn()}
        />,
      );
      return screen.getByRole("alert").querySelector("div") as HTMLElement;
    };

    expect(renderFailure("access-denied").className).toContain("destructive");
    cleanup();
    expect(renderFailure("needs-reauth").className).toContain("amber");
  });
});
