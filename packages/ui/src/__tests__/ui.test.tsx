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
import { Card } from "../components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../components/ui/tabs";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "../components/ui/tooltip";
import { Textarea } from "../components/ui/textarea";
import { Checkbox } from "../components/ui/checkbox";
import { Popover, PopoverContent, PopoverTrigger } from "../components/ui/popover";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogTitle,
} from "../components/ui/alert-dialog";
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

  it("Tooltip 氣泡為白底細框帶陰影（design D5），不借主色", () => {
    // spec「介面狀態語意色分層」：主色實心氣泡與「已就緒」徽章撞色；改為與浮層
    // 同一套白底細框，全域 tooltip 一次生效。
    render(
      <TooltipProvider>
        <Tooltip defaultOpen>
          <TooltipTrigger asChild>
            <button>觸發</button>
          </TooltipTrigger>
          <TooltipContent>提示內容</TooltipContent>
        </Tooltip>
      </TooltipProvider>,
    );
    const bubble = document.querySelector("[data-side]") as HTMLElement;
    expect(bubble.textContent).toContain("提示內容");
    const cls = bubble.className.split(/\s+/);
    for (const c of ["bg-card", "text-foreground", "border", "shadow-md", "text-xs", "rounded-md"]) {
      expect(cls).toContain(c);
    }
    expect(bubble.className).not.toContain("bg-primary");
    expect(bubble.className).not.toContain("bg-foreground");
  });
});

/** className 拆成 token；不帶變體前綴（hover:、data-[…]:）的才是靜態樣式。 */
const classes = (el: Element) => (el.getAttribute("class") ?? "").split(/\s+/);
const staticClasses = (el: Element) => classes(el).filter((c) => !c.includes(":"));

// 原語扁平化（desktop-design-foundation design D5）。
describe("原語扁平化：button／card／badge／tabs", () => {
  it("Button 圓角 rounded-md、焦點為 2px ring", () => {
    render(<Button>go</Button>);
    const btn = screen.getByRole("button", { name: "go" });
    expect(classes(btn)).toContain("rounded-md");
    expect(classes(btn)).toContain("focus-visible:ring-2");
  });

  it("secondary 變體為 bg-primary/10 text-primary", () => {
    render(<Button variant="secondary">s</Button>);
    const cls = classes(screen.getByRole("button", { name: "s" }));
    expect(cls).toContain("bg-primary/10");
    expect(cls).toContain("text-primary");
  });

  it("subtle 變體靜態無底、hover 有底", () => {
    render(<Button variant="subtle" size="icon-sm" aria-label="more" />);
    const btn = screen.getByRole("button", { name: "more" });
    expect(staticClasses(btn).some((c) => c.startsWith("bg-"))).toBe(false);
    expect(classes(btn)).toContain("hover:bg-foreground/5");
  });

  it("destructive 為實心紅、danger 為淡紅底紅字", () => {
    render(
      <>
        <Button variant="destructive">del</Button>
        <Button variant="danger">warn</Button>
      </>,
    );
    expect(classes(screen.getByRole("button", { name: "del" }))).toContain("bg-destructive");
    const danger = classes(screen.getByRole("button", { name: "warn" }));
    expect(danger).toContain("bg-destructive/10");
    expect(danger).toContain("text-destructive");
  });

  it.each([
    ["default", "h-9"],
    ["sm", "h-7"],
    ["toolbar", "h-8"],
    ["icon", "h-8 w-8"],
    ["icon-sm", "h-7 w-7"],
    ["icon-xs", "h-6 w-6"],
  ] as const)("Button size=%s 為 %s", (size, expected) => {
    render(<Button size={size}>{size}</Button>);
    const cls = classes(screen.getByRole("button", { name: size }));
    for (const c of expected.split(" ")) expect(cls).toContain(c);
  });

  it("Card 為 rounded-2xl 細框白底、無陰影", () => {
    render(<Card data-testid="card" />);
    const cls = classes(screen.getByTestId("card"));
    expect(cls).toEqual(expect.arrayContaining(["rounded-2xl", "border", "bg-card"]));
    expect(cls.some((c) => c.startsWith("shadow"))).toBe(false);
  });

  it("card nested 變體為 rounded-xl", () => {
    render(<Card size="nested" data-testid="card" />);
    const cls = classes(screen.getByTestId("card"));
    expect(cls).toContain("rounded-xl");
    expect(cls).not.toContain("rounded-2xl");
  });

  it("card interactive 變體 hover 框變深、底微灰", () => {
    render(<Card interactive data-testid="card" />);
    const hover = classes(screen.getByTestId("card")).filter((c) => c.startsWith("hover:"));
    expect(hover.some((c) => c.startsWith("hover:border-"))).toBe(true);
    expect(hover.some((c) => c.startsWith("hover:bg-"))).toBe(true);
  });

  it("Badge 全圓、11px 字", () => {
    render(<Badge>b</Badge>);
    const cls = classes(screen.getByText("b"));
    expect(cls).toContain("rounded-full");
    expect(cls).toContain("text-[11px]");
  });

  const renderTabs = (variant?: "underline" | "card") =>
    render(
      <Tabs defaultValue="a">
        <TabsList variant={variant} actions={<button>開啟</button>}>
          <TabsTrigger value="a">甲</TabsTrigger>
          <TabsTrigger value="b">乙</TabsTrigger>
        </TabsList>
        <TabsContent value="a">內容</TabsContent>
      </Tabs>,
    );

  it("分頁作用中帶 border-primary 底線且無底色、List 的 actions slot 渲染在右端", () => {
    renderTabs();
    const active = screen.getByRole("tab", { name: "甲" });
    expect(active.getAttribute("data-state")).toBe("active");
    const cls = classes(active);
    expect(cls).toContain("data-[state=active]:border-primary");
    expect(cls).toContain("data-[state=active]:text-primary");
    expect(cls).toContain("border-b-2");
    expect(cls.some((c) => c.startsWith("data-[state=active]:bg-"))).toBe(false);
    expect(cls.some((c) => c.startsWith("data-[state=active]:shadow"))).toBe(false);
    // actions 不進 tablist（tablist 只容納分頁），放在同一列的右端。
    const tablist = screen.getByRole("tablist");
    const action = screen.getByRole("button", { name: "開啟" });
    expect(tablist.contains(action)).toBe(false);
    const bar = tablist.parentElement as HTMLElement;
    expect(bar.lastElementChild!.contains(action)).toBe(true);
    expect(classes(bar.lastElementChild!)).toContain("ml-auto");
  });

  it("分頁列預設為整列底線：只有底邊框，沒有外框、圓角與底色", () => {
    // 卡片標頭式要有內容卡接在下方才成立；各頁版面尚未接上前，預設不畫外框。
    renderTabs();
    const bar = classes(screen.getByRole("tablist").parentElement!);
    expect(bar).toEqual(expect.arrayContaining(["w-full", "border-b", "border-border"]));
    expect(bar).not.toContain("border");
    expect(bar).not.toContain("bg-card");
    expect(bar.some((c) => c.startsWith("rounded"))).toBe(false);
  });

  it("variant=card 為內容卡的標頭：外框、上方 16px 圓角、卡片底色", () => {
    renderTabs("card");
    const bar = classes(screen.getByRole("tablist").parentElement!);
    expect(bar).toEqual(expect.arrayContaining(["rounded-t-2xl", "bg-card", "border", "px-2"]));
  });
});

describe("原語扁平化：表單與浮層（design D5）", () => {
  it("Textarea 8px 圓角、無陰影、聚焦為 ring 框", () => {
    render(<Textarea aria-label="t" />);
    const cls = classes(screen.getByRole("textbox", { name: "t" }));
    for (const c of ["rounded-lg", "focus-visible:border-ring", "focus-visible:ring-2", "focus-visible:ring-ring/30"]) {
      expect(cls).toContain(c);
    }
    expect(cls.some((c) => c.includes("shadow"))).toBe(false);
  });

  it("Checkbox 為 4px 圓角", () => {
    render(<Checkbox aria-label="c" />);
    expect(classes(screen.getByRole("checkbox", { name: "c" }))).toContain("rounded-[4px]");
  });

  it("Popover 12px 圓角、p-3、shadow-md", () => {
    render(
      <Popover defaultOpen>
        <PopoverTrigger>開</PopoverTrigger>
        <PopoverContent>浮層內容</PopoverContent>
      </Popover>,
    );
    const cls = classes(screen.getByText("浮層內容"));
    for (const c of ["rounded-xl", "p-3", "shadow-md"]) expect(cls).toContain(c);
  });

  it("確認框：30% 中性遮罩、28rem 寬 12px 圓角、標題 15px、說明 13px、取消為 ghost", () => {
    render(
      <AlertDialog open>
        <AlertDialogContent>
          <AlertDialogTitle>標題</AlertDialogTitle>
          <AlertDialogDescription>說明</AlertDialogDescription>
          <AlertDialogCancel>取消</AlertDialogCancel>
          <AlertDialogAction>確認</AlertDialogAction>
        </AlertDialogContent>
      </AlertDialog>,
    );
    const content = screen.getByRole("alertdialog");
    for (const c of ["w-[28rem]", "rounded-xl", "p-5", "shadow-lg"]) {
      expect(classes(content)).toContain(c);
    }
    const overlay = content.previousElementSibling as HTMLElement;
    expect(classes(overlay)).toContain("bg-foreground/30");
    expect(classes(screen.getByText("標題"))).toEqual(
      expect.arrayContaining(["text-[15px]", "font-medium"]),
    );
    expect(classes(screen.getByText("說明"))).toEqual(
      expect.arrayContaining(["text-[13px]", "text-muted-foreground"]),
    );
    const cancel = classes(screen.getByRole("button", { name: "取消" }));
    expect(cancel).toContain("hover:bg-accent");
    expect(cancel).not.toContain("border");
  });
});
