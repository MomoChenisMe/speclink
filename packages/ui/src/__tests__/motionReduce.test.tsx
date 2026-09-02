// desktop-app「抽屜與浮層的開關動畫」Scenario「減少動態偏好下無動畫」：jsdom 沒有版面
// 與媒體查詢，比照 skeleton.test 以 motion-reduce: 變體 class 斷言（repo 既有慣例），
// 並確認進出兩向的動畫 class 都在（出場 class 缺席＝關閉時直接消失）。
import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";

import { Sheet, SheetContent } from "../components/ui/sheet";
import { Popover, PopoverContent, PopoverTrigger } from "../components/ui/popover";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "../components/ui/tooltip";

const REQUIRED = ["motion-reduce:animate-none", "data-[state=open]:animate-in", "data-[state=closed]:animate-out"];

describe("抽屜與浮層的減少動態變體", () => {
  it("Sheet 的遮罩與面板都帶進出動畫與 motion-reduce 變體", () => {
    render(
      <Sheet open>
        <SheetContent>面板</SheetContent>
      </Sheet>,
    );
    const layers = Array.from(document.querySelectorAll<HTMLElement>('[data-state="open"]'));
    expect(layers.length).toBeGreaterThanOrEqual(2);
    for (const el of layers) for (const cls of REQUIRED) expect(el.className).toContain(cls);
  });

  it("Popover 與 Tooltip 的內容層同樣帶進出動畫與 motion-reduce 變體", () => {
    render(
      <>
        <Popover open>
          <PopoverTrigger>p</PopoverTrigger>
          <PopoverContent>浮層</PopoverContent>
        </Popover>
        <TooltipProvider>
          <Tooltip open>
            <TooltipTrigger>t</TooltipTrigger>
            <TooltipContent>提示</TooltipContent>
          </Tooltip>
        </TooltipProvider>
      </>,
    );
    // 兩種內容層都由 Radix 標上 data-side；trigger 沒有。Popover 的開啟態是 open，
    // Tooltip 的是 delayed-open／instant-open——進場變體必須掛在實際會出現的值上。
    const layers = Array.from(document.querySelectorAll<HTMLElement>("[data-side]"));
    expect(layers.length).toBe(2);
    const popover = layers.find((el) => el.getAttribute("role") === "dialog") as HTMLElement;
    const tooltip = layers.find((el) => el !== popover) as HTMLElement;
    expect(popover.getAttribute("data-state")).toBe("open");
    for (const cls of REQUIRED) expect(popover.className).toContain(cls);
    const tooltipState = tooltip.getAttribute("data-state") ?? "";
    expect(["delayed-open", "instant-open"]).toContain(tooltipState);
    for (const cls of [
      "motion-reduce:animate-none",
      "data-[state=closed]:animate-out",
      `data-[state=${tooltipState}]:animate-in`,
      "data-[state=delayed-open]:animate-in",
      "data-[state=instant-open]:animate-in",
    ]) {
      expect(tooltip.className).toContain(cls);
    }
    expect(tooltip.className).not.toContain("data-[state=open]:");
  });
});
