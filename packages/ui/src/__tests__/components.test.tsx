import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";

import { ChangeBoard } from "../components/ChangeBoard";
import { DocumentTree } from "../components/DocumentTree";
import { DocumentViewer } from "../components/DocumentViewer";
import type { ChangeItem, SpecItem } from "../adapter";

const changes: ChangeItem[] = [
  { name: "desktop-shell-and-browser", status: "in-progress", totalTasks: 17, completedTasks: 11 },
  { name: "web-server-postgres", status: "pending", totalTasks: 0, completedTasks: 0 },
];
const specs: SpecItem[] = [{ id: "verb-contract" }, { id: "desktop-app" }];

describe("ChangeBoard", () => {
  it("renders each change with its name and task progress", () => {
    render(<ChangeBoard changes={changes} />);
    expect(screen.getByText("desktop-shell-and-browser")).toBeTruthy();
    expect(screen.getByText(/11\s*\/\s*17/)).toBeTruthy();
  });

  it("invokes onRunVerb with the verb and change name when a verb button is clicked", () => {
    const onRunVerb = vi.fn();
    render(<ChangeBoard changes={changes} onRunVerb={onRunVerb} />);
    fireEvent.click(screen.getAllByRole("button", { name: /validate/i })[0]);
    expect(onRunVerb).toHaveBeenCalledWith("validate", "desktop-shell-and-browser");
  });
});

describe("DocumentTree", () => {
  it("renders changes and specs, and calls onSelect on click", () => {
    const onSelect = vi.fn();
    render(<DocumentTree changes={changes} specs={specs} onSelect={onSelect} />);
    expect(screen.getByText("verb-contract")).toBeTruthy();
    fireEvent.click(screen.getByText("web-server-postgres"));
    expect(onSelect).toHaveBeenCalledWith({ kind: "change", id: "web-server-postgres" });
  });
});

describe("DocumentViewer", () => {
  it("renders the given document content", () => {
    render(<DocumentViewer content={"## Why\nbecause"} />);
    expect(screen.getByText(/because/)).toBeTruthy();
  });

  it("shows an empty state when content is null", () => {
    render(<DocumentViewer content={null} />);
    expect(screen.getByText(/選擇|no document|select/i)).toBeTruthy();
  });
});
