import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { MigrationDialog } from "../components/MigrationDialog";
import type { MigrationAdapter, MigrationResult } from "../adapter/migration";
import type { ConnectionView, ConnectionsAdapter } from "../adapter/connections";
import { APP_MESSAGES } from "../i18n/messages";

const CONNECTION: ConnectionView = {
  id: "conn_1",
  origin: "https://spec.example.test",
  name: "團隊 Server",
  loggedIn: true,
};

const SCOPES = {
  projects: [
    {
      id: "prj_1",
      key: "speclink",
      name: "Speclink",
      repos: [{ id: "repo_1", key: "desktop", name: "Desktop" }],
    },
  ],
};

const RESULT: MigrationResult = {
  report: {
    projectRevision: 8,
    documents: Array.from({ length: 14 }, (_, index) => ({ index })),
  },
  backupPath: "/work/local/openspec.migrated-2026-07-21",
  checkoutRoot: "/work/local",
};

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);

function renderDialog(migrate: MigrationAdapter["migrate"]) {
  const onMigrated = vi.fn().mockResolvedValue(undefined);
  const connectionAdapter: Pick<ConnectionsAdapter, "scopes"> = {
    scopes: vi.fn().mockResolvedValue(SCOPES),
  };
  render(
    <MigrationDialog
      open
      root="/work/local"
      connections={[CONNECTION]}
      connectionAdapter={connectionAdapter}
      migration={{ migrate, adoptRemote: vi.fn() }}
      onOpenChange={vi.fn()}
      onMigrated={onMigrated}
    />,
    { wrapper: zhWrapper },
  );
  return { connectionAdapter, onMigrated };
}

async function reachConfirmation() {
  fireEvent.click(screen.getByRole("button", { name: /團隊 Server/ }));
  const repo = await screen.findByRole("radio", { name: /Desktop/ });
  fireEvent.click(repo);
  fireEvent.click(screen.getByRole("button", { name: "檢查遷移" }));
}

describe("MigrationDialog", () => {
  it("選空 scope、確認目標與備份、顯示進度，成功後原地轉成 checkout remote 分頁", async () => {
    let resolveMigration: (result: MigrationResult) => void = () => {};
    const migrate = vi.fn(
      () =>
        new Promise<MigrationResult>((resolve) => {
          resolveMigration = resolve;
        }),
    );
    const { connectionAdapter, onMigrated } = renderDialog(migrate);

    await reachConfirmation();

    expect(screen.getAllByText("speclink / desktop")).toHaveLength(2);
    expect(screen.getByText(/openspec\/.*改名備份/)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "開始遷移" }));

    expect(screen.getByRole("status").textContent).toContain("正在建立 Bundle 並上傳");
    resolveMigration(RESULT);

    await waitFor(() =>
      expect(migrate).toHaveBeenCalledWith(
        "/work/local",
        "conn_1",
        "speclink",
        "desktop",
      ),
    );
    await waitFor(() =>
      expect(onMigrated).toHaveBeenCalledWith(
        "conn_1",
        "speclink/desktop",
        "/work/local",
      ),
    );
    expect(connectionAdapter.scopes).toHaveBeenCalledWith("conn_1");
    expect(screen.getByText(/已遷移 14 份文件/)).toBeTruthy();
    expect(screen.getByText(RESULT.backupPath)).toBeTruthy();
  });

  it("server 拒絕時原樣呈現錯誤且不轉換分頁", async () => {
    const migrate = vi
      .fn()
      .mockRejectedValue(new Error("409 create-new import requires an empty target scope"));
    const { onMigrated } = renderDialog(migrate);

    await reachConfirmation();
    fireEvent.click(screen.getByRole("button", { name: "開始遷移" }));

    expect((await screen.findByRole("alert")).textContent).toContain(
      "409 create-new import requires an empty target scope",
    );
    expect(onMigrated).not.toHaveBeenCalled();
  });
});
