import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { WorkspaceChooser } from "../components/WorkspaceChooser";
import { RemoteConflictDialog } from "../components/RemoteConflictDialog";
import { APP_MESSAGES } from "../i18n/messages";
import type { ConnectionsAdapter, ConnectionView } from "../adapter/connections";
import type { WorkspaceAdapter } from "../adapter/workspace";

const CONNECTION: ConnectionView = {
  id: "conn_1",
  origin: "https://spec.example.test",
  name: "團隊 Server",
  lastActorDisplay: "Momo",
  loggedIn: true,
};

const SCOPES = {
  projects: [
    {
      id: "prj_1",
      key: "speclink",
      name: "Speclink",
      repos: [
        { id: "repo_1", key: "desktop", name: "Desktop" },
        { id: "repo_2", key: "server", name: "Server" },
      ],
    },
  ],
};

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);

function fakeConnections(over: Partial<ConnectionsAdapter> = {}): ConnectionsAdapter {
  return {
    list: vi.fn().mockResolvedValue([CONNECTION]),
    add: vi.fn().mockResolvedValue(CONNECTION),
    remove: vi.fn().mockResolvedValue(undefined),
    deviceLogin: vi.fn().mockResolvedValue({ status: "loggedIn", display: "Momo" }),
    patLogin: vi.fn().mockResolvedValue({ status: "loggedIn", display: "Momo" }),
    logout: vi.fn().mockResolvedValue({ revokedOnServer: true, patNotice: false }),
    scopes: vi.fn().mockResolvedValue(SCOPES),
    bindCheckout: vi.fn().mockImplementation(async (path: string) => path),
    ...over,
  };
}

function fakeWorkspace(over: Partial<WorkspaceAdapter> = {}): WorkspaceAdapter {
  return {
    openProject: vi.fn(),
    initProject: vi.fn(),
    startupDir: vi.fn(),
    projectStats: vi.fn(),
    watchWorkspace: vi.fn(),
    pickFolder: vi.fn().mockResolvedValue(null),
    ...over,
  };
}

function renderChooser({
  adapter = fakeConnections(),
  workspace = fakeWorkspace(),
  connections = [CONNECTION],
  onOpenRemote = vi.fn().mockResolvedValue(undefined),
  onOpenLocal = vi.fn().mockResolvedValue(undefined),
  onRequestMigration = vi.fn().mockResolvedValue(undefined),
  onAddServer = vi.fn().mockResolvedValue(undefined),
}: {
  adapter?: ConnectionsAdapter;
  workspace?: WorkspaceAdapter;
  connections?: ConnectionView[];
  onOpenRemote?: ReturnType<typeof vi.fn>;
  onOpenLocal?: ReturnType<typeof vi.fn>;
  onRequestMigration?: ReturnType<typeof vi.fn>;
  onAddServer?: ReturnType<typeof vi.fn>;
} = {}) {
  const onOpenChange = vi.fn();
  render(
    <WorkspaceChooser
      open
      onOpenChange={onOpenChange}
      connections={connections}
      connectionAdapter={adapter}
      workspace={workspace}
      onOpenLocal={onOpenLocal}
      onRequestMigration={onRequestMigration}
      onAddServer={onAddServer}
      onRefreshConnections={vi.fn().mockResolvedValue(undefined)}
      onOpenRemote={onOpenRemote}
    />,
    { wrapper: zhWrapper },
  );
  return {
    adapter,
    workspace,
    onOpenRemote,
    onOpenLocal,
    onRequestMigration,
    onAddServer,
    onOpenChange,
  };
}

async function chooseDesktopRepo() {
  fireEvent.click(screen.getByRole("button", { name: /Speclink Server/ }));
  fireEvent.click(screen.getByRole("button", { name: /團隊 Server/ }));
  await waitFor(() => expect(screen.getByRole("radio", { name: /Desktop/ })).toBeTruthy());
  fireEvent.click(screen.getByRole("radio", { name: /Desktop/ }));
  fireEvent.click(screen.getByRole("button", { name: "下一步" }));
}

describe("WorkspaceChooser", () => {
  it("來源→已登入 server→scopes 分組單選→略過 checkout 開啟 spec-only 分頁", async () => {
    const open = vi.fn().mockResolvedValue(undefined);
    const { adapter, onOpenChange } = renderChooser({ onOpenRemote: open });

    await chooseDesktopRepo();
    fireEvent.click(screen.getByRole("button", { name: /略過（規格模式）/ }));
    fireEvent.click(screen.getByRole("button", { name: "開啟 Workspace" }));

    await waitFor(() =>
      expect(open).toHaveBeenCalledWith("conn_1", "speclink/desktop", undefined),
    );
    expect(adapter.scopes).toHaveBeenCalledWith("conn_1");
    expect(screen.queryByPlaceholderText("project 或 project/repo")).toBeNull();
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it("選擇 checkout 會走 marker 驗證，拒絕訊息指出 marker 指向且不開分頁", async () => {
    const bind = vi.fn().mockRejectedValue(
      new Error(
        "此資料夾的 remote marker 指向 https://other.example.test / api，與所選不一致",
      ),
    );
    const adapter = fakeConnections({ bindCheckout: bind });
    const workspace = fakeWorkspace({ pickFolder: vi.fn().mockResolvedValue("/work/desktop") });
    const open = vi.fn().mockResolvedValue(undefined);
    renderChooser({ adapter, workspace, onOpenRemote: open });

    await chooseDesktopRepo();
    fireEvent.click(screen.getByRole("button", { name: /選擇本機資料夾/ }));

    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain(
        "https://other.example.test / api",
      ),
    );
    expect(bind).toHaveBeenCalledWith(
      "/work/desktop",
      "https://spec.example.test",
      "speclink",
      "desktop",
    );
    expect(open).not.toHaveBeenCalled();
  });

  it("scopes 沒有 membership 時顯示繁中空清單說明", async () => {
    const adapter = fakeConnections({ scopes: vi.fn().mockResolvedValue({ projects: [] }) });
    renderChooser({ adapter });

    fireEvent.click(screen.getByRole("button", { name: /Speclink Server/ }));
    fireEvent.click(screen.getByRole("button", { name: /團隊 Server/ }));

    expect((await screen.findByTestId("scopes-empty")).textContent).toContain(
      "此帳號目前沒有任何 Project／Repo membership",
    );
    expect((screen.getByRole("button", { name: "下一步" }) as HTMLButtonElement).disabled).toBe(
      true,
    );
  });

  it("server 清單可就地新增並登入，完成後回到清單步驟", async () => {
    const add = vi.fn().mockResolvedValue(undefined);
    renderChooser({ connections: [], onAddServer: add });

    fireEvent.click(screen.getByRole("button", { name: /Speclink Server/ }));
    expect(screen.getByText(/目前沒有已登入的 server/)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "新增 server" }));
    fireEvent.change(screen.getByLabelText("伺服器位址（http://…）"), {
      target: { value: "https://new.example.test" },
    });
    fireEvent.change(screen.getByLabelText("顯示名"), { target: { value: "新 Server" } });
    fireEvent.click(screen.getByRole("button", { name: "新增並登入" }));

    await waitFor(() =>
      expect(add).toHaveBeenCalledWith("https://new.example.test", "新 Server"),
    );
    // add 被呼叫≠完成：status 訊息在 promise resolve 後才渲染，須非同步等。
    expect((await screen.findByRole("status")).textContent).toContain("連線已新增");
  });

  it("本機資料夾分流沿用既有開啟流程", async () => {
    const local = vi.fn().mockResolvedValue(undefined);
    const workspace = fakeWorkspace({
      pickFolder: vi.fn().mockResolvedValue("/work/local"),
      openProject: vi.fn().mockResolvedValue({
        status: "project",
        root: "/work/local",
        name: "Local",
      }),
    });
    const { onOpenChange } = renderChooser({ workspace, onOpenLocal: local });

    fireEvent.click(screen.getByRole("button", { name: /本機資料夾/ }));
    fireEvent.click(await screen.findByRole("button", { name: "開啟本機" }));

    await waitFor(() => expect(local).toHaveBeenCalledWith("/work/local"));
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it("選到含 openspec 的本機專案後提供遷移到 server 次要動作", async () => {
    const migrate = vi.fn().mockResolvedValue(undefined);
    const workspace = fakeWorkspace({
      pickFolder: vi.fn().mockResolvedValue("/work/local"),
      openProject: vi.fn().mockResolvedValue({
        status: "project",
        root: "/work/local",
        name: "Local",
      }),
    });
    const { onOpenChange } = renderChooser({
      workspace,
      onRequestMigration: migrate,
    });

    fireEvent.click(screen.getByRole("button", { name: /本機資料夾/ }));
    fireEvent.click(await screen.findByRole("button", { name: "遷移到 Server…" }));

    await waitFor(() => expect(migrate).toHaveBeenCalledWith("/work/local"));
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });
});

describe("remote marker 與本機 openspec 並存", () => {
  it("強制呈現繼續本機、以 server 為準、遷移本機內容三個出口", async () => {
    const continueLocal = vi.fn().mockResolvedValue(undefined);
    const useServer = vi.fn().mockResolvedValue(undefined);
    const migrateLocal = vi.fn();
    render(
      <RemoteConflictDialog
        conflict={{
          path: "/work/coexists",
          url: "https://spec.example.test/api/speclink/v1/projects/demo",
          repo: "backend",
        }}
        onContinueLocal={continueLocal}
        onUseServer={useServer}
        onMigrateLocal={migrateLocal}
      />,
      { wrapper: zhWrapper },
    );

    expect(screen.getByText(/備份後棄用本機內容.*不會上傳或合併/)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: /^繼續本機/ }));
    fireEvent.click(screen.getByRole("button", { name: /^以 Server 為準/ }));
    await waitFor(() => expect(useServer).toHaveBeenCalledTimes(1));
    await waitFor(() =>
      expect(
        (screen.getByRole("button", { name: /^遷移本機內容/ }) as HTMLButtonElement).disabled,
      ).toBe(false),
    );
    fireEvent.click(screen.getByRole("button", { name: /^遷移本機內容/ }));

    await waitFor(() => expect(continueLocal).toHaveBeenCalledTimes(1));
    expect(migrateLocal).toHaveBeenCalledTimes(1);
    expect(screen.queryByRole("button", { name: "取消" })).toBeNull();
  });
});
