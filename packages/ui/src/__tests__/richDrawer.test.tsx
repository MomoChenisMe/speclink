import { describe, it, expect, vi } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";

import { RichDetailDrawer } from "../components/RichDetailDrawer";
import type { ChangeItem } from "../adapter";

const change: ChangeItem = {
  name: "desktop-shell-and-browser",
  status: "in-progress",
  totalTasks: 30,
  completedTasks: 30,
};

const SPEC_MD = "## ADDED Requirements\n\n### Requirement: a\nb\n\n### Requirement: c\nd\n";

function makeProps(over: Record<string, unknown> = {}) {
  return {
    open: true,
    onOpenChange: vi.fn(),
    change,
    loadDocument: vi.fn(async (_c: string, artifact: string) =>
      artifact.startsWith("specs/") ? SPEC_MD : `# doc for ${artifact}`,
    ),
    loadCapabilities: vi.fn(async () => ["desktop-app"]),
    loadMeta: vi.fn(async () => ({ created: "2026-07-05", createdBy: "MomoChen", createdWith: "claude" })),
    onRunVerb: vi.fn(),
    onDelete: vi.fn(),
    ...over,
  };
}

describe("RichDetailDrawer", () => {
  it("renders metadata row (author, agent, task count) and progress", async () => {
    render(<RichDetailDrawer {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("MomoChen")).toBeTruthy());
    expect(screen.getByText(/claude/)).toBeTruthy();
    expect(screen.getAllByText(/30\/30/).length).toBeGreaterThan(0);
  });

  it("shows spec delta counts on the specs tab", async () => {
    render(<RichDetailDrawer {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText(/\+2/)).toBeTruthy());
  });

  it("fires onDelete when the delete action is clicked", async () => {
    const props = makeProps();
    render(<RichDetailDrawer {...(props as never)} />);
    fireEvent.click(screen.getByRole("button", { name: /刪除/ }));
    expect(props.onDelete).toHaveBeenCalledWith("desktop-shell-and-browser");
  });

  it("fires onRunVerb for analyze / archive actions", async () => {
    const props = makeProps();
    render(<RichDetailDrawer {...(props as never)} />);
    fireEvent.click(screen.getByRole("button", { name: /分析/ }));
    expect(props.onRunVerb).toHaveBeenCalledWith("analyze", "desktop-shell-and-browser");
    fireEvent.click(screen.getByRole("button", { name: /封存/ }));
    expect(props.onRunVerb).toHaveBeenCalledWith("archive", "desktop-shell-and-browser");
  });
});
