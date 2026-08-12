import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";

import { Skeleton } from "../components/ui/skeleton";

// spec「看板首訪以 skeleton 佔位」「抽屜文件載入以 skeleton 呈現」的視覺基元
// （design D4）：圓角灰塊＋pulse 動畫，prefers-reduced-motion 下停用動畫。
describe("Skeleton 基元", () => {
  it("以圓角灰塊呈現並帶 pulse 動畫", () => {
    const { container } = render(<Skeleton />);
    const el = container.firstElementChild as HTMLElement;
    expect(el.className).toContain("animate-pulse");
    expect(el.className).toContain("rounded-md");
    expect(el.className).toContain("bg-muted");
  });

  // 逐處加 motion-reduce: 是本 repo 既有慣例（ProjectTabs、TrayPanel 等）；
  // jsdom 不求值 media query，故以 class 存在為斷言面。
  it("帶 motion-reduce 變體，減少動態效果時不動畫", () => {
    const { container } = render(<Skeleton />);
    const el = container.firstElementChild as HTMLElement;
    expect(el.className).toContain("motion-reduce:animate-none");
  });

  it("接受外部 className 且與基底 class 合併", () => {
    const { container } = render(<Skeleton className="h-4 w-24" />);
    const el = container.firstElementChild as HTMLElement;
    expect(el.className).toContain("h-4");
    expect(el.className).toContain("w-24");
    expect(el.className).toContain("animate-pulse");
  });

  it("透傳 HTML 屬性（如 data-testid）", () => {
    const { getByTestId } = render(<Skeleton data-testid="bar" />);
    expect(getByTestId("bar")).toBeTruthy();
  });
});
