// 品牌標記與字標（spec desktop-app「共用元件唯一來源」Scenario「品牌資產只有一處」
// 「品牌標記隨主題取主色」；design D8）。
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";

/** 外層 span 是無障礙圖片；內嵌 SVG 原稿自帶 role=img，為 role=img 的子節點、瀏覽器視為裝飾。 */
const markEl = (container: HTMLElement) => container.firstElementChild as HTMLElement;

import { BrandMark, Wordmark } from "../components/Brand";

describe("BrandMark", () => {
  it("以 currentColor 內嵌 SVG 標記，顏色跟 text-primary 走", () => {
    const { container } = render(<BrandMark />);
    const mark = markEl(container);
    expect(mark.tagName).toBe("SPAN");
    expect(mark.getAttribute("role")).toBe("img");
    expect(mark.getAttribute("aria-label")).toBe("Speclink");
    expect(mark.className).toContain("text-primary");
    const svg = mark.querySelector("svg");
    expect(svg).toBeTruthy();
    expect(svg!.innerHTML).toContain('fill="currentColor"');
    expect(svg!.outerHTML).not.toContain("#167873");
  });

  it("高度預設 26px，size 可覆寫", () => {
    const first = render(<BrandMark />);
    expect(markEl(first.container).style.height).toBe("26px");
    first.unmount();
    const second = render(<BrandMark size={20} />);
    expect(markEl(second.container).style.height).toBe("20px");
  });
});

describe("Wordmark", () => {
  it("橫式鎖版 img（alt=Speclink）＋深色偏好的 source，className 套到 img", () => {
    const { container } = render(<Wordmark className="h-5" />);
    const img = screen.getByAltText("Speclink") as HTMLImageElement;
    expect(img.tagName).toBe("IMG");
    expect(img.className).toContain("h-5");
    expect(img.getAttribute("src")).toContain("logo-horizontal");
    const source = container.querySelector("picture > source") as HTMLSourceElement;
    expect(source.getAttribute("media")).toBe("(prefers-color-scheme: dark)");
    expect(source.getAttribute("srcset")).toContain("logo-horizontal-dark");
  });
});
