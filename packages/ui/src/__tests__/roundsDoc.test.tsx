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

### Round 1 — assumptions (2026-07-08)

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

// 記錄檔往尾端追加第 5 輪（引擎 discuss add-round 的寫法）。
const FIVE_ROUNDS = `${ROUNDS}
### Round 5 — assumptions (2026-10-15)

**Focus**: 結論後再補一輪
**Position**: 補充
`;

// spec Example「九輪的排列」：Rounds 區段依序為 Round 1 到 Round 9，Round 9 的 Focus 為「結論後補充」。
const NINE_ROUNDS = Array.from(
  { length: 9 },
  (_, i) =>
    `### Round ${i + 1} — assumptions (2026-10-0${i + 1})\n\n**Focus**: ${i === 8 ? "結論後補充" : `第 ${i + 1} 輪焦點`}\n**Position**: 定論\n`,
).join("\n");

/** 畫面上由上而下的輪卡。 */
const cards = (container: HTMLElement) => Array.from(container.querySelectorAll<HTMLElement>("[data-round]"));
/** 依輪次取卡（不以畫面位置代表輪次）。 */
const card = (container: HTMLElement, round: number) =>
  container.querySelector<HTMLElement>(`[data-round="${round}"]`) as HTMLElement;
const toggle = (el: HTMLElement) => el.querySelector("button[aria-expanded]") as HTMLButtonElement;

describe("RoundsDoc", () => {
  // spec「標準記錄逐輪成卡」：由新到舊，第 4 與第 1 輪（最上與最下）展開。Round 1 即 spec Example「輪標題解析」
  // 的輸入：Round 1、assumptions、2026-07-08、焦點「現況的痛點是哪一種」。
  it("四輪：由上而下 data-round 依序為 4、3、2、1，aria-expanded 依序為 true、false、false、true，Round 1 卡頭含 Round 1、焦點、mode、日期", () => {
    const { container } = render(<RoundsDoc text={ROUNDS} />);
    expect(container.querySelector("[data-round-timeline]")).toBeTruthy();
    const all = cards(container);
    expect(all.map((c) => c.getAttribute("data-round"))).toEqual(["4", "3", "2", "1"]);
    expect(all.map((c) => toggle(c).getAttribute("aria-expanded"))).toEqual(["true", "false", "false", "true"]);
    const first = card(container, 1);
    const head = toggle(first).textContent;
    for (const part of ["Round 1", "現況的痛點是哪一種", "assumptions", "2026-07-08"]) expect(head).toContain(part);
    expect(first.querySelector("[data-doc-heading]")!.textContent).toBe("Round 1");
    expect(first.querySelector("h3")!.textContent).toContain("現況的痛點是哪一種");
  });

  // spec「新的輪排在最上」的 Example「九輪的排列」。
  it("九輪：第一張卡為 Round 9 且焦點為「結論後補充」、最後一張為 Round 1，兩者展開、其間七張收合", () => {
    const { container } = render(<RoundsDoc text={NINE_ROUNDS} />);
    const all = cards(container);
    expect(all.map((c) => c.getAttribute("data-round"))).toEqual(["9", "8", "7", "6", "5", "4", "3", "2", "1"]);
    expect(all[0].querySelector("[data-doc-heading]")!.textContent).toBe("Round 9");
    expect(all[0].querySelector("[data-round-focus]")!.textContent).toBe("結論後補充");
    expect(toggle(all[0]).getAttribute("aria-expanded")).toBe("true");
    expect(all[8].querySelector("[data-doc-heading]")!.textContent).toBe("Round 1");
    expect(toggle(all[8]).getAttribute("aria-expanded")).toBe("true");
    expect(all.slice(1, 8).map((c) => toggle(c).getAttribute("aria-expanded"))).toEqual(Array(7).fill("false"));
  });

  // 記錄檔只會往尾端追加輪，倒序後新輪插在畫面最上；點過的展開狀態跟著輪次走、不錯位。
  it("新輪加入不錯位：點開 Round 2 後以五輪記錄重新渲染，Round 2 仍展開、Round 5 在最上且展開、Round 4 收合", () => {
    const { container, rerender } = render(<RoundsDoc text={ROUNDS} />);
    fireEvent.click(toggle(card(container, 2)));
    expect(toggle(card(container, 2)).getAttribute("aria-expanded")).toBe("true");
    rerender(<RoundsDoc text={FIVE_ROUNDS} />);
    expect(cards(container)[0].getAttribute("data-round")).toBe("5");
    expect(toggle(card(container, 5)).getAttribute("aria-expanded")).toBe("true");
    expect(toggle(card(container, 4)).getAttribute("aria-expanded")).toBe("false");
    expect(toggle(card(container, 2)).getAttribute("aria-expanded")).toBe("true");
    expect(toggle(card(container, 1)).getAttribute("aria-expanded")).toBe("true");
  });

  it("立場定論句與 3 條細線列、粗體前綴原文不出現", () => {
    const { container } = render(<RoundsDoc text={ROUNDS} />);
    const first = card(container, 1);
    expect(first.querySelector("[data-position-verdict]")!.textContent).toBe("使用者裁定：只改 UI。");
    expect(first.querySelectorAll('[data-round-field="position"] [data-row-list] > li').length).toBe(3);
    for (const prefix of ["Focus", "Position", "Ruled out", "**Open**"]) expect(container.textContent).not.toContain(prefix);
  });

  it("否決兩列帶 × 圖示、待決兩顆籤、末輪「無——寫結論」為 data-open-none", () => {
    const { container } = render(<RoundsDoc text={ROUNDS} />);
    const first = card(container, 1);
    const last = card(container, 4);
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
    const first = card(container, 1);
    const second = card(container, 2);
    expect(second.querySelector("[data-position-verdict]")).toBeNull();
    // 卡頭可點：共用游標變數與滑過淡底（與工單輪卡頭、需求收合列同款）。
    expect(toggle(second).className).toContain("cursor-(--cursor-interaction)");
    expect(toggle(second).className).toContain("hover:bg-foreground/5");
    fireEvent.click(toggle(second));
    expect(toggle(second).getAttribute("aria-expanded")).toBe("true");
    expect(second.querySelector("[data-position-verdict]")!.textContent).toBe("直答");
    fireEvent.click(toggle(second));
    expect(toggle(second).getAttribute("aria-expanded")).toBe("false");
    expect(second.querySelector("[data-position-verdict]")).toBeNull();
    expect(toggle(first).getAttribute("aria-expanded")).toBe("true");
  });

  it("焦點在收合卡頭一行截斷、展開後折行完整顯示", () => {
    const long = "第二輪的焦點寫得很長，".repeat(8).trim();
    const { container } = render(<RoundsDoc text={ROUNDS.replace("第二輪焦點", long)} />);
    const second = card(container, 2);
    const focus = () => second.querySelector("[data-round-focus]") as HTMLElement;
    expect(focus().className).toContain("truncate");
    fireEvent.click(toggle(second));
    expect(focus().className).not.toContain("truncate");
    expect(focus().className).toContain("break-words");
    expect(focus().textContent).toBe(long);
  });

  it("缺 Ruled out 時無否決欄位", () => {
    const { container } = render(<RoundsDoc text={ROUNDS} />);
    const third = card(container, 3);
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
    const dot = (round: number) => card(container, round).querySelector("[data-round-dot]")!.className;
    expect(dot(1)).toContain("bg-primary");
    expect(dot(2)).toContain("bg-stage-discussion");
    expect(dot(3)).toContain("bg-muted-foreground");
    expect(dot(3)).not.toMatch(/bg-primary|bg-stage-discussion/);
  });
});
