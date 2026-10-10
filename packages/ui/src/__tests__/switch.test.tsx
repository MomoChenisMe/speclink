// Switch 原語（desktop-settings-mode design D3）：Radix switch——role=switch、
// aria-checked 隨切換、停用不觸發、空白鍵可切換；開為主色軌、關為 input 色軌。
import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { Switch } from "../components/ui/switch";

const classes = (el: Element) => (el.getAttribute("class") ?? "").split(/\s+/);

describe("Switch", () => {
  it("以 role=switch 呈現，點擊切換 aria-checked 並回呼", async () => {
    const onCheckedChange = vi.fn();
    const user = userEvent.setup();
    render(<Switch aria-label="tdd" onCheckedChange={onCheckedChange} />);
    const sw = screen.getByRole("switch", { name: "tdd" });
    expect(sw.getAttribute("aria-checked")).toBe("false");
    await user.click(sw);
    expect(sw.getAttribute("aria-checked")).toBe("true");
    expect(onCheckedChange).toHaveBeenCalledWith(true);
  });

  it("停用時點擊不觸發", async () => {
    const onCheckedChange = vi.fn();
    const user = userEvent.setup();
    render(<Switch aria-label="audit" disabled onCheckedChange={onCheckedChange} />);
    await user.click(screen.getByRole("switch", { name: "audit" }));
    expect(onCheckedChange).not.toHaveBeenCalled();
  });

  it("空白鍵切換", async () => {
    const onCheckedChange = vi.fn();
    const user = userEvent.setup();
    render(<Switch aria-label="worktree" defaultChecked onCheckedChange={onCheckedChange} />);
    screen.getByRole("switch", { name: "worktree" }).focus();
    await user.keyboard(" ");
    expect(onCheckedChange).toHaveBeenCalledWith(false);
  });

  it("軌 h-5 w-9、開主色關 input 色；thumb 16px；focus 環沿 Button", () => {
    render(<Switch aria-label="x" checked onCheckedChange={() => {}} />);
    const sw = screen.getByRole("switch", { name: "x" });
    expect(classes(sw)).toEqual(
      expect.arrayContaining([
        "h-5",
        "w-9",
        "rounded-full",
        "data-[state=checked]:bg-primary",
        "data-[state=unchecked]:bg-input",
        "focus-visible:ring-2",
        "focus-visible:ring-ring",
      ]),
    );
    const thumb = sw.querySelector("span") as HTMLElement;
    expect(classes(thumb)).toEqual(expect.arrayContaining(["h-4", "w-4", "rounded-full"]));
  });
});
