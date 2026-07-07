import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";

// 既有中文斷言包 I18nProvider locale zh-TW 後照舊斷言（i18n 抽 key 的回歸保護）。
const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

import { Button } from "../components/ui/button";
import { Badge } from "../components/ui/badge";
import { cn } from "../lib/utils";

describe("shadcn 設計系統原語", () => {
  it("Button renders children and fires onClick", () => {
    const onClick = vi.fn();
    render(<Button onClick={onClick}>validate</Button>);
    const btn = screen.getByRole("button", { name: "validate" });
    expect(btn).toBeTruthy();
    fireEvent.click(btn);
    expect(onClick).toHaveBeenCalled();
  });

  it("Button applies variant/size classes via cva", () => {
    render(<Button variant="outline" size="sm">x</Button>);
    const btn = screen.getByRole("button", { name: "x" });
    expect(btn.className).toContain("border");
  });

  it("Badge renders its label", () => {
    render(<Badge>in-progress</Badge>);
    expect(screen.getByText("in-progress")).toBeTruthy();
  });

  it("cn merges and de-duplicates tailwind classes", () => {
    expect(cn("p-2", "p-4")).toBe("p-4");
  });
});
