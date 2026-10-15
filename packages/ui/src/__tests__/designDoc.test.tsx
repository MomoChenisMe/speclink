// spec「提案與設計章節以中文標籤呈現」設計側（desktop-change-doc-views design D4）。
import { describe, expect, it, vi } from "vitest";
import { fireEvent, render as rtlRender, screen } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { DesignDoc } from "../components/DesignDoc";

const wrapper = ({ children }: { children: ReactNode }) => <I18nProvider locale="zh-TW">{children}</I18nProvider>;
const render = (ui: ReactElement) => rtlRender(ui, { wrapper });

const DECISION_TITLES = ["面板底與卡片層", "卡片間距", "標題列", "狀態色", "捲動", "深色模式"];

const DESIGN_MD = [
  "## Context",
  "",
  "- 現況一",
  "- 現況二",
  "",
  "## Goals / Non-Goals",
  "",
  "**Goals:**",
  "",
  ...["一", "二", "三", "四", "五"].map((n) => `- 目標${n}`),
  "",
  "**Non-Goals:**",
  "",
  ...["一", "二", "三"].map((n) => `- 非目標${n}`),
  "",
  "## Decisions",
  "",
  ...DECISION_TITLES.flatMap((title, i) => [`### D${i + 1} ${title}`, "", `- 決策 ${i + 1} 的第一點`, `- 決策 ${i + 1} 的第二點`, ""]),
  "## Implementation Contract",
  "",
  "- **Behavior**：開頁看到卡",
  "- **Failure modes**：降級",
  "",
  "## Risks / Trade-offs",
  "",
  "- [風險一] → 對策一",
  "- [風險二] → 對策二",
  "- [風險三] → 對策三",
  "- 只有風險",
  "",
  "## Open Questions",
  "",
  "無。",
].join("\n");

const decisionCard = (container: HTMLElement, id: string) =>
  container.querySelector(`[data-decision="${id}"]`)!.querySelector("button[aria-expanded]")!;

describe("DesignDoc", () => {
  it("決策段每個三級標題一張卡：預設收合、D 籤與列點數，標題留在本頁目錄", () => {
    const { container } = render(<DesignDoc content={DESIGN_MD} jump={null} onJumped={() => {}} />);
    const cards = container.querySelectorAll("[data-decision]");
    expect(cards).toHaveLength(6);
    for (const card of Array.from(cards)) {
      expect(card.querySelector("button")!.getAttribute("aria-expanded")).toBe("false");
    }
    const tocTitles = Array.from(container.querySelectorAll("[data-doc-heading]")).map((h) => h.textContent);
    for (const title of DECISION_TITLES) expect(tocTitles).toContain(title);
    const head = decisionCard(container, "D1");
    expect(head.textContent).toContain("D1");
    expect(head.textContent).toContain("2 點");
    expect(screen.queryByText("決策 1 的第一點")).toBeNull();
    expect(head.className).toContain("cursor-(--cursor-interaction)");
    expect(head.className).toContain("hover:bg-foreground/5");
    fireEvent.click(head);
    expect(head.getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByText("決策 1 的第一點")).toBeTruthy();
  });

  it("jump 命中的決策卡展開並回報完成", () => {
    const onJumped = vi.fn();
    const { container } = render(<DesignDoc content={DESIGN_MD} jump={{ id: "D3", seq: 1 }} onJumped={onJumped} />);
    expect(decisionCard(container, "D3").getAttribute("aria-expanded")).toBe("true");
    expect(decisionCard(container, "D1").getAttribute("aria-expanded")).toBe("false");
    expect(onJumped).toHaveBeenCalledTimes(1);
  });

  it("jump 沒命中只回報完成", () => {
    const onJumped = vi.fn();
    const { container } = render(<DesignDoc content={DESIGN_MD} jump={{ id: "D9", seq: 1 }} onJumped={onJumped} />);
    expect(onJumped).toHaveBeenCalledTimes(1);
    expect(container.querySelectorAll('[data-decision] button[aria-expanded="true"]')).toHaveLength(0);
  });

  it("風險與取捨逐條成 warning 提示框（「→」切標題與內文）", () => {
    const { container } = render(<DesignDoc content={DESIGN_MD} jump={null} onJumped={() => {}} />);
    const risks = container.querySelectorAll("[data-risk]");
    expect(risks).toHaveLength(4);
    expect(risks[0].className).toContain("border-l-4");
    expect(risks[0].textContent).toBe("風險一對策一");
    expect(risks[3].textContent).toBe("只有風險");
  });

  it("風險標題與決策標題的行內程式碼成等寬籤、粗體不直出星號；風險的巢狀子項接在內文", () => {
    const md = [
      "## Decisions",
      "",
      "### D7 `RequirementBlock` 共用",
      "",
      "- 點",
      "",
      "## Risks / Trade-offs",
      "",
      "- [`parseSpecSections` 認 **二級標題** 為終點] → 先切 op 區段",
      "  - 子項說明",
    ].join("\n");
    const { container } = render(<DesignDoc content={md} jump={null} onJumped={() => {}} />);
    const h3 = container.querySelector("[data-decision] h3")!;
    expect(h3.querySelector("code")!.textContent).toBe("RequirementBlock");
    expect(h3.textContent).not.toContain("`");
    const risk = container.querySelector("[data-risk]")!;
    const title = risk.firstElementChild!;
    expect(title.querySelector("code")!.textContent).toBe("parseSpecSections");
    expect(title.querySelector("strong")!.textContent).toBe("二級標題");
    expect(title.textContent).toBe("parseSpecSections 認 二級標題 為終點");
    expect(risk.textContent).toContain("先切 op 區段");
    expect(risk.textContent).toContain("子項說明");
  });

  it("契約列的值為空時不畫「無內容」占位", () => {
    const { container } = render(
      <DesignDoc content={"## Implementation Contract\n\n- **Behavior**：\n- **Failure modes**：降級\n"} jump={null} onJumped={() => {}} />,
    );
    expect(Array.from(container.querySelectorAll("dt")).map((d) => d.textContent)).toEqual(["Behavior", "Failure modes"]);
    expect(container.textContent).not.toContain("（無內容）");
  });

  it("目標與非目標兩欄各自列點數，背景為細線列、契約為鍵值列", () => {
    const { container } = render(<DesignDoc content={DESIGN_MD} jump={null} onJumped={() => {}} />);
    const cols = container.querySelectorAll("[data-goals-column]");
    expect(Array.from(cols).map((c) => c.firstElementChild!.textContent)).toEqual(["目標 5", "非目標 3"]);
    expect(cols[0].querySelectorAll("li")).toHaveLength(5);
    expect(cols[1].querySelectorAll("li")).toHaveLength(3);
    expect(container.querySelectorAll('[data-section="sections.context"] [data-row-list] li')).toHaveLength(2);
    expect(Array.from(container.querySelectorAll('[data-section="sections.contract"] dt')).map((d) => d.textContent)).toEqual([
      "Behavior",
      "Failure modes",
    ]);
  });

  it("切不到 Goals／Non-Goals 標籤時照 Markdown", () => {
    const { container } = render(<DesignDoc content={"## Goals / Non-Goals\n\n- 只寫列點\n"} jump={null} onJumped={() => {}} />);
    expect(container.querySelector("[data-goals-column]")).toBeNull();
    expect(screen.getByText("只寫列點")).toBeTruthy();
  });
});
