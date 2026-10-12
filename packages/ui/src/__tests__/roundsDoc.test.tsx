// spec「討論輪以卡片呈現」的討論過程渲染器（desktop-discussion-doc-views design D3）。
import { describe, expect, it } from "vitest";
import { fireEvent, render as rtlRender } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { RoundsDoc } from "../components/RoundsDoc";

const wrapper = ({ children }: { children: ReactNode }) => <I18nProvider locale="zh-TW">{children}</I18nProvider>;
const render = (ui: ReactElement) => rtlRender(ui, { wrapper });

const ROUNDS = `
<!-- \`### Round N — <mode> (<date>)\` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-12)

**Focus**: 現況的痛點是哪一種
**Position**: 使用者裁定：只改 UI。
- 決策空間甲
- 決策空間乙
- 決策空間丙
**Ruled out**: A——理由甲；B——理由乙
**Open**: 點綴色留 teal 還是換紫；切刀

### Round 2 — interview (2026-10-12)

**Focus**: 第二輪焦點
**Position**: 直答
**Open**: 無

### Round 3 — explore (2026-10-13)

**Focus**: 第三輪焦點
**Position**: 三

### Round 4 — interview (2026-10-14)

**Focus**: 收尾
**Position**: 定案
**Ruled out**: 甲
**Open**: 無——寫結論
`;

const cards = (container: HTMLElement) => Array.from(container.querySelectorAll<HTMLElement>("[data-round]"));
const toggle = (card: HTMLElement) => card.querySelector("button[aria-expanded]") as HTMLButtonElement;

describe("RoundsDoc", () => {
  it("四輪：第 1 與第 4 輪 aria-expanded 為 true、第 2 與第 3 輪 false，卡頭含 Round N、焦點、mode、日期", () => {
    const { container } = render(<RoundsDoc text={ROUNDS} />);
    expect(container.querySelector("[data-round-timeline]")).toBeTruthy();
    const all = cards(container);
    expect(all.map((c) => c.getAttribute("data-round"))).toEqual(["1", "2", "3", "4"]);
    expect(all.map((c) => toggle(c).getAttribute("aria-expanded"))).toEqual(["true", "false", "false", "true"]);
    const head = toggle(all[0]).textContent;
    for (const part of ["Round 1", "現況的痛點是哪一種", "assumptions", "2026-10-12"]) expect(head).toContain(part);
    expect(all[0].querySelector("[data-doc-heading]")!.textContent).toBe("Round 1");
    expect(all[0].querySelector("h3")!.textContent).toContain("現況的痛點是哪一種");
  });

  it("立場定論句與 3 條細線列、粗體前綴原文不出現", () => {
    const { container } = render(<RoundsDoc text={ROUNDS} />);
    const first = cards(container)[0];
    expect(first.querySelector("[data-position-verdict]")!.textContent).toBe("使用者裁定：只改 UI。");
    expect(first.querySelectorAll('[data-round-field="position"] [data-row-list] > li').length).toBe(3);
    for (const prefix of ["Focus", "Position", "Ruled out", "**Open**"]) expect(container.textContent).not.toContain(prefix);
  });

  it("否決兩列帶 × 圖示、待決兩顆籤、末輪「無——寫結論」為 data-open-none", () => {
    const { container } = render(<RoundsDoc text={ROUNDS} />);
    const [first, , , last] = cards(container);
    const ruledOut = first.querySelectorAll("[data-ruled-out]");
    expect(Array.from(ruledOut).map((r) => r.textContent)).toEqual(["A——理由甲", "B——理由乙"]);
    for (const row of ruledOut) expect(row.querySelector("svg.lucide-x")).toBeTruthy();
    expect(Array.from(first.querySelectorAll("[data-open-item]")).map((o) => o.textContent)).toEqual([
      "點綴色留 teal 還是換紫",
      "切刀",
    ]);
    const none = last.querySelector("[data-open-none]")!;
    expect(none.textContent).toBe("無——寫結論");
    expect(none.querySelector("svg.lucide-check")).toBeTruthy();
    expect(last.querySelector("[data-open-item]")).toBeNull();
  });

  it("點第 2 輪卡頭展開、再點收合、第 1 輪不受影響", () => {
    const { container } = render(<RoundsDoc text={ROUNDS} />);
    const [first, second] = cards(container);
    expect(second.querySelector("[data-position-verdict]")).toBeNull();
    fireEvent.click(toggle(second));
    expect(toggle(second).getAttribute("aria-expanded")).toBe("true");
    expect(second.querySelector("[data-position-verdict]")!.textContent).toBe("直答");
    fireEvent.click(toggle(second));
    expect(toggle(second).getAttribute("aria-expanded")).toBe("false");
    expect(second.querySelector("[data-position-verdict]")).toBeNull();
    expect(toggle(first).getAttribute("aria-expanded")).toBe("true");
  });

  it("缺 Ruled out 時無否決欄位", () => {
    const { container } = render(<RoundsDoc text={ROUNDS} />);
    const third = cards(container)[2];
    fireEvent.click(toggle(third));
    expect(third.querySelector('[data-round-field="position"]')).toBeTruthy();
    expect(third.querySelector('[data-round-field="ruledOut"]')).toBeNull();
    expect(third.querySelector('[data-round-field="open"]')).toBeNull();
  });

  it("非標準輪標題整篇 Markdown、無時間軸", () => {
    const bad = ROUNDS.replace("### Round 2 — interview (2026-10-12)", "### 插入的手寫標題");
    const { container } = render(<RoundsDoc text={bad} />);
    expect(container.querySelector("[data-round-timeline]")).toBeNull();
    expect(container.textContent).toContain("插入的手寫標題");
  });

  it("零輪時為空態文案", () => {
    const { container } = render(<RoundsDoc text={"\n<!-- scaffold -->\n"} empty="（尚無討論過程）" />);
    expect(container.querySelector("[data-round-timeline]")).toBeNull();
    expect(container.textContent).toBe("（尚無討論過程）");
  });

  it("色點：assumptions 帶 bg-primary、interview 帶 bg-stage-discussion、其他 mode 中性", () => {
    const { container } = render(<RoundsDoc text={ROUNDS} />);
    const dots = cards(container).map((c) => c.querySelector("[data-round-dot]")!.className);
    expect(dots[0]).toContain("bg-primary");
    expect(dots[1]).toContain("bg-stage-discussion");
    expect(dots[2]).toContain("bg-muted-foreground");
    expect(dots[2]).not.toMatch(/bg-primary|bg-stage-discussion/);
  });
});
