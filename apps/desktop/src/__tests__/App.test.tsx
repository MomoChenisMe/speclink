import { describe, it, expect, vi } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";

import { App } from "../App";
import type { SpeclinkDataSource, StatusReport } from "@speclink/ui";

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
