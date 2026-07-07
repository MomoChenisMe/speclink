import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";
import { render, screen, waitFor, fireEvent, within } from "@testing-library/react";

import { App } from "../App";
import type { SpeclinkDataSource, StatusReport } from "@speclink/ui";

// 模擬 Tauri 事件層：捕捉 workspace-changed 的訂閱 handler，測試可手動觸發。
const { workspaceHandlers } = vi.hoisted(() => ({
  workspaceHandlers: [] as Array<() => void>,
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((event: string, handler: () => void) => {
    if (event === "workspace-changed") workspaceHandlers.push(handler);
    return Promise.resolve(() => {
      const i = workspaceHandlers.indexOf(handler);
      if (i >= 0) workspaceHandlers.splice(i, 1);
    });
  }),
}));

// 兩個抽屜的 pass-through spy：捕捉 props（驗證刷新世代下發）後照常渲染原元件。
const { drawerSpy } = vi.hoisted(() => ({
  drawerSpy: { rich: [] as Array<Record<string, unknown>>, disc: [] as Array<Record<string, unknown>> },
}));
vi.mock("@speclink/ui", async (importOriginal) => {
  const mod = await importOriginal<typeof import("@speclink/ui")>();
  return {
    ...mod,
    RichDetailDrawer: (props: never) => {
      drawerSpy.rich.push(props);
      return <mod.RichDetailDrawer {...(props as object) as Parameters<typeof mod.RichDetailDrawer>[0]} />;
    },
    DiscussionDrawer: (props: never) => {
      drawerSpy.disc.push(props);
      return <mod.DiscussionDrawer {...(props as object) as Parameters<typeof mod.DiscussionDrawer>[0]} />;
    },
  };
});

beforeEach(() => {
  workspaceHandlers.length = 0;
  drawerSpy.rich.length = 0;
  drawerSpy.disc.length = 0;
});

const STATUS: StatusReport = {
  changeName: "desktop-shell-and-browser",
  schemaName: "spec-driven",
  isComplete: false,
  applyRequires: ["tasks"],
  artifacts: [],
};

function fakeDataSource(over: Partial<SpeclinkDataSource> = {}): SpeclinkDataSource {
  return {
    listChanges: vi.fn().mockResolvedValue([
      { name: "desktop-shell-and-browser", status: "in-progress", totalTasks: 30, completedTasks: 30 },
    ]),
    listSpecs: vi.fn().mockResolvedValue([{ id: "desktop-app" }]),
    listArchived: vi.fn().mockResolvedValue([]),
    status: vi.fn().mockResolvedValue(STATUS),
    getDocument: vi.fn().mockResolvedValue("## Why\nhello body"),
    getSpecDocument: vi.fn().mockResolvedValue("# spec"),
    changeCapabilities: vi.fn().mockResolvedValue(["desktop-app"]),
    changeMeta: vi.fn().mockResolvedValue({ created: "2026-07-05", createdBy: "MomoChen", createdWith: "claude" }),
    deleteChange: vi.fn().mockResolvedValue(undefined),
    setTaskDone: vi.fn().mockResolvedValue(undefined),
    moveTask: vi.fn().mockResolvedValue(undefined),
    runVerb: vi.fn().mockResolvedValue({ valid: true }),
    getArchivedDocument: vi.fn().mockResolvedValue(null),
    archivedCapabilities: vi.fn().mockResolvedValue([]),
    listDiscussions: vi.fn().mockResolvedValue({ active: [], archived: [] }),
    getDiscussionDocument: vi.fn().mockResolvedValue(null),
    promoteDiscussion: vi.fn().mockResolvedValue({ change: "promoted-change" }),
    archiveDiscussion: vi.fn().mockResolvedValue(undefined),
    ...over,
  };
}

describe("App (kanban primary + rich detail)", () => {
  it("renders the kanban board by default with change cards", async () => {
    render(<App dataSource={fakeDataSource()} />);
    await waitFor(() => expect(screen.getByText("desktop-shell-and-browser")).toBeTruthy());
    // 看板欄位存在
    expect(document.querySelector('[data-column="ready"]')).toBeTruthy();
  });

  it("opens the rich detail drawer with metadata when a card is clicked", async () => {
    const ds = fakeDataSource();
    render(<App dataSource={ds} />);
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    fireEvent.click(screen.getByText("desktop-shell-and-browser"));
    await waitFor(() => expect(screen.getByText("MomoChen")).toBeTruthy());
    expect(ds.changeMeta).toHaveBeenCalledWith("desktop-shell-and-browser");
  });

  it("delete flow: drawer delete → confirm dialog → deleteChange called", async () => {
    const ds = fakeDataSource();
    render(<App dataSource={ds} />);
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    fireEvent.click(screen.getByText("desktop-shell-and-browser"));
    await waitFor(() => screen.getByRole("button", { name: /刪除/ }));
    fireEvent.click(screen.getByRole("button", { name: /刪除/ }));
    // 確認對話框
    await waitFor(() => screen.getByText("刪除變更？"));
    fireEvent.click(screen.getByRole("button", { name: "刪除" }));
    await waitFor(() => expect(ds.deleteChange).toHaveBeenCalledWith("desktop-shell-and-browser"));
  });

  it("passes an increasing refreshGen generation to both drawers", async () => {
    // design D1：世代自 store 經 props 下發，內容元件據此重載（重載行為由 packages/ui 測試承載）。
    render(<App dataSource={fakeDataSource()} />);
    await waitFor(() => expect(workspaceHandlers.length).toBeGreaterThan(0));
    await waitFor(() => expect(drawerSpy.rich.length).toBeGreaterThan(0));
    await waitFor(() => expect(drawerSpy.disc.length).toBeGreaterThan(0));
    const before = drawerSpy.rich[drawerSpy.rich.length - 1].refreshGen;
    expect(typeof before).toBe("number");
    workspaceHandlers.forEach((h) => h());
    await waitFor(() => {
      expect(drawerSpy.rich[drawerSpy.rich.length - 1].refreshGen as number).toBeGreaterThan(before as number);
      expect(drawerSpy.disc[drawerSpy.disc.length - 1].refreshGen as number).toBeGreaterThan(before as number);
    });
  });

  it("workspace-changed event triggers a full refresh (external writers reflected)", async () => {
    const ds = fakeDataSource();
    render(<App dataSource={ds} />);
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    await waitFor(() => expect(workspaceHandlers.length).toBeGreaterThan(0));
    const before = (ds.listChanges as Mock).mock.calls.length;
    // 模擬檔案監看發出的 Tauri 事件（外部 CLI/agent 寫入後）。
    workspaceHandlers.forEach((h) => h());
    await waitFor(() =>
      expect((ds.listChanges as Mock).mock.calls.length).toBeGreaterThan(before)
    );
    expect((ds.listArchived as Mock).mock.calls.length).toBeGreaterThan(1);
  });

  it("promote flow: 討論卡「轉為變更」→ 確認對話框（使用者語言）→ promoteDiscussion ＋整批 refresh", async () => {
    const ds = fakeDataSource({
      listDiscussions: vi.fn().mockResolvedValue({
        active: [
          { slug: "settled", topic: "Settled topic", status: "concluded", rounds: 2, created: "2026-07-01", promotedTo: [] },
        ],
        archived: [],
      }),
    });
    render(<App dataSource={ds} />);
    await waitFor(() => screen.getByText("Settled topic"));
    fireEvent.click(screen.getByRole("button", { name: /轉為變更/ }));
    await waitFor(() => screen.getByText("轉為變更？"));
    const dialog = screen.getByRole("alertdialog");
    // 文案說「會發生什麼」，不暴露工程詞（design D4）。
    expect(dialog.textContent).toContain("提案中");
    expect(dialog.textContent).toContain("結論");
    expect(dialog.textContent).not.toMatch(/from_discussion|kebab-case|proposal|meta/);
    fireEvent.click(within(dialog).getByRole("button", { name: "轉為變更" }));
    await waitFor(() => expect(ds.promoteDiscussion).toHaveBeenCalledWith("settled", "settled"));
    await waitFor(() =>
      expect((ds.listDiscussions as Mock).mock.calls.length).toBeGreaterThan(1),
    );
  });

  it("轉為變更確認框可自訂變更名稱（再轉出的入口），說明為使用者語言", async () => {
    const ds = fakeDataSource({
      listDiscussions: vi.fn().mockResolvedValue({
        active: [
          { slug: "settled", topic: "Settled topic", status: "concluded", rounds: 2, created: "2026-07-01", promotedTo: [] },
        ],
        archived: [],
      }),
    });
    render(<App dataSource={ds} />);
    await waitFor(() => screen.getByText("Settled topic"));
    fireEvent.click(screen.getByRole("button", { name: /轉為變更/ }));
    await waitFor(() => screen.getByText("轉為變更？"));
    // 名稱輸入 label「變更名稱」、說明「英文小寫，字間用 -」；預設由 slug 衍生。
    const input = screen.getByLabelText("變更名稱") as HTMLInputElement;
    expect(input.value).toBe("settled");
    expect(screen.getByText(/英文小寫，字間用 -/)).toBeTruthy();
    fireEvent.change(input, { target: { value: "second-cut" } });
    const dialog = screen.getByRole("alertdialog");
    fireEvent.click(within(dialog).getByRole("button", { name: "轉為變更" }));
    await waitFor(() => expect(ds.promoteDiscussion).toHaveBeenCalledWith("settled", "second-cut"));
  });

  it("archive-discussion flow: 討論卡「封存」→ 確認（使用者語言）→ archiveDiscussion called", async () => {
    const ds = fakeDataSource({
      listDiscussions: vi.fn().mockResolvedValue({
        active: [
          { slug: "settled", topic: "Settled topic", status: "concluded", rounds: 2, created: "2026-07-01", promotedTo: [] },
        ],
        archived: [],
      }),
    });
    render(<App dataSource={ds} />);
    await waitFor(() => screen.getByText("Settled topic"));
    const card = screen.getByText("Settled topic").closest("[data-discussion]") as HTMLElement;
    fireEvent.click(within(card).getByRole("button", { name: /^封存$/ }));
    await waitFor(() => screen.getByText("封存討論？"));
    const dialog = screen.getByRole("alertdialog");
    expect(dialog.textContent).toContain("已封存頁");
    expect(dialog.textContent).not.toContain("discussions/archive");
    fireEvent.click(within(dialog).getByRole("button", { name: "封存" }));
    await waitFor(() => expect(ds.archiveDiscussion).toHaveBeenCalledWith("settled"));
  });

  it("archived entry in the top bar jumps to the archived list", async () => {
    const ds = fakeDataSource({
      listArchived: vi.fn().mockResolvedValue([
        { datedName: "2026-07-04-old-change", date: "2026-07-04", name: "old-change" },
      ]),
    });
    render(<App dataSource={ds} />);
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    fireEvent.click(screen.getByLabelText("已封存"));
    await waitFor(() => expect(screen.getByText("已封存的變更")).toBeTruthy());
    expect(screen.getByText("old-change")).toBeTruthy();
    expect(screen.getByText("2026-07-04")).toBeTruthy();
  });
});
