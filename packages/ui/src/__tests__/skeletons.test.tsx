import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";

import { CardSkeleton, DocSkeleton, RowSkeleton } from "../components/skeletons";

// spec「看板首訪以 skeleton 佔位」「面板分區首訪 skeleton」「抽屜文件載入以
// skeleton 呈現」的三種佔位組件（design D4）。aria-busy 標記載入中區塊。
describe("CardSkeleton（看板佔位卡）", () => {
  it("標記 aria-busy 並以卡形呈現名稱條與摘要條", () => {
    const { container } = render(<CardSkeleton />);
    const el = container.firstElementChild as HTMLElement;
    expect(el.getAttribute("aria-busy")).toBe("true");
    // 名稱條＋摘要條：卡內至少兩條佔位灰塊。
    expect(container.querySelectorAll(".animate-pulse").length).toBeGreaterThanOrEqual(2);
  });

  it("不含任何文字內容（純佔位，不冒充空態文案）", () => {
    const { container } = render(<CardSkeleton />);
    expect(container.textContent).toBe("");
  });

  // 佔位卡是「還沒到的那張真卡片」，外框自刻就會與真實卡片分歧：裸寫 border 在
  // Tailwind v4 落到 currentColor（近黑描邊），正是這樣飄掉的。鎖的是 Card 的三個
  // 表面值——Card 換了樣式，這裡就該紅一次讓人回來看骨架還像不像真卡片。
  it("外框帶 Card 的表面樣式，不自刻描邊", () => {
    const { container } = render(<CardSkeleton />);
    const el = container.firstElementChild as HTMLElement;
    expect(el.className).toContain("border-border");
    expect(el.className).toContain("rounded-lg");
    expect(el.className).toContain("shadow-sm");
    expect(el.className).toContain("bg-card");
  });
});

describe("RowSkeleton（面板佔位列）", () => {
  it("標記 aria-busy 並以單行列形呈現", () => {
    const { container } = render(<RowSkeleton />);
    const el = container.firstElementChild as HTMLElement;
    expect(el.getAttribute("aria-busy")).toBe("true");
    expect(container.querySelectorAll(".animate-pulse").length).toBeGreaterThanOrEqual(1);
  });

  it("不含任何文字內容", () => {
    const { container } = render(<RowSkeleton />);
    expect(container.textContent).toBe("");
  });
});

describe("DocSkeleton（文件佔位）", () => {
  it("標記 aria-busy 並以標題條＋數行內文條呈現", () => {
    const { container } = render(<DocSkeleton />);
    const el = container.firstElementChild as HTMLElement;
    expect(el.getAttribute("aria-busy")).toBe("true");
    // 標題條與內文條合計多於單行，才看得出是「一份文件」而非一條進度。
    expect(container.querySelectorAll(".animate-pulse").length).toBeGreaterThan(2);
  });

  it("不含任何文字內容（載入中不冒充「沒有文件」空態）", () => {
    const { container } = render(<DocSkeleton />);
    expect(container.textContent).toBe("");
  });
});
