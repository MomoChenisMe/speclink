// spec「討論詳情頁檢視與轉出變更」背景段的渲染器（desktop-discussion-doc-views design D4）。
import { describe, expect, it, vi } from "vitest";
import { fireEvent, render as rtlRender } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { ContextDoc, type ContextDocProps } from "../components/ContextDoc";
import { TooltipProvider } from "../components/ui/tooltip";
import { contextModel, type ContextNames } from "../discussionModel";

const wrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">
    <TooltipProvider>{children}</TooltipProvider>
  </I18nProvider>
);
const render = (ui: ReactElement) => rtlRender(ui, { wrapper });

// spec Scenario「背景文件的引言、關聯與記錄列」。
const CONTEXT = [
  "使用者要重做卡片，偵察命中 desktop-app 與 tray-status-menu。",
  "",
  "在途變更 add-copilot-tool 也碰到；已封存的 tray-panel-card-design 是前例。",
  "",
  "Related specs: desktop-app（79 條）、tray-status-menu",
  "Prior discussions: card-drawer-header-colors, admin-console-redesign",
  "Source doc: none",
].join("\n");

const NAMES: ContextNames = {
  specs: ["desktop-app", "tray-status-menu", "server-api"],
  active: ["add-copilot-tool"],
  archived: ["tray-panel-card-design"],
  discussions: ["card-redesign", "card-drawer-header-colors", "admin-console-redesign"],
  self: "card-redesign",
};

function props({ text = CONTEXT, ...over }: Partial<ContextDocProps> & { text?: string } = {}): ContextDocProps {
  return {
    model: contextModel(text, NAMES),
    created: "2026-10-08",
    createdBy: "MomoChen <momo@example.com>",
    kind: null,
    recordPath: "openspec/discussions/card-redesign.md",
    onOpenSpec: vi.fn(),
    onOpenChange: vi.fn(),
    onOpenArchivedChange: vi.fn(),
    onOpenDiscussion: vi.fn(),
    ...over,
  };
}

const section = (container: HTMLElement, key: string) =>
  container.querySelector(`[data-section="${key}"]`) as HTMLElement | null;
const rowValues = (container: HTMLElement) =>
  Array.from(container.querySelectorAll<HTMLElement>("[data-context-row]")).map((row) => [
    row.getAttribute("data-context-row"),
    row.querySelector("dd")!.textContent,
  ]);

describe("ContextDoc", () => {
  it("引言為灰底段、內文四顆名稱籤、收尾四行不在內文；關聯五列、記錄三列；點籤分派到對應回呼", () => {
    const p = props();
    const { container } = render(<ContextDoc {...p} />);
    const intro = section(container, "context.intro")!;
    expect(intro.querySelector("[data-context-lead]")!.textContent).toBe(
      "使用者要重做卡片，偵察命中 desktop-app 與 tray-status-menu。",
    );
    expect(Array.from(intro.querySelectorAll("[data-name-chip]")).map((c) => c.getAttribute("data-name-chip"))).toEqual([
      "spec:desktop-app",
      "spec:tray-status-menu",
      "change:add-copilot-tool",
      "archived:tray-panel-card-design",
    ]);
    for (const line of ["Related specs:", "Prior discussions:", "Source doc:"]) expect(intro.textContent).not.toContain(line);

    expect(rowValues(section(container, "context.relations")!)).toEqual([
      ["specs", "desktop-apptray-status-menu"],
      ["changes", "add-copilot-tool"],
      ["archived", "tray-panel-card-design"],
      ["prior", "card-drawer-header-colorsadmin-console-redesign"],
      ["source", "無"],
    ]);
    expect(rowValues(section(container, "context.record")!)).toEqual([
      ["created", "2026-10-08 · MomoChen"],
      ["kind", "一般討論"],
      ["file", "openspec/discussions/card-redesign.md"],
    ]);
    expect(section(container, "context.record")!.querySelector('[data-context-row="file"] button')).toBeTruthy();

    fireEvent.click(intro.querySelector('[data-name-chip="spec:desktop-app"]')!);
    expect(p.onOpenSpec).toHaveBeenCalledWith("desktop-app");
    fireEvent.click(intro.querySelector('[data-name-chip="change:add-copilot-tool"]')!);
    expect(p.onOpenChange).toHaveBeenCalledWith("add-copilot-tool");
    fireEvent.click(intro.querySelector('[data-name-chip="archived:tray-panel-card-design"]')!);
    expect(p.onOpenArchivedChange).toHaveBeenCalledWith("tray-panel-card-design");
    // 關聯列的籤同樣可點。
    fireEvent.click(section(container, "context.relations")!.querySelector('[data-context-row="prior"] button')!);
    expect(p.onOpenDiscussion).toHaveBeenCalledWith("card-drawer-header-colors");
    expect(Array.from(container.querySelectorAll("h2[data-doc-heading]")).map((h) => h.textContent)).toEqual([
      "起因與範圍",
      "關聯",
      "記錄",
    ]);
  });

  it("Enter／空白鍵只在名稱籤上攔下並開啟；內文連結上的按鍵照瀏覽器預設", () => {
    const p = props({ text: "提到 desktop-app，詳見 [說明](https://example.com/doc)。" });
    const { container } = render(<ContextDoc {...p} />);
    const intro = section(container, "context.intro")!;
    const link = intro.querySelector("a")!;
    expect(fireEvent.keyDown(link, { key: "Enter" })).toBe(true);
    expect(fireEvent.keyDown(link, { key: " " })).toBe(true);
    expect(p.onOpenSpec).not.toHaveBeenCalled();
    const chip = intro.querySelector('[data-name-chip="spec:desktop-app"]')!;
    expect(fireEvent.keyDown(chip, { key: "Enter" })).toBe(false);
    expect(p.onOpenSpec).toHaveBeenCalledWith("desktop-app");
  });

  it("無收尾行且零命中時無關聯段、無籤", () => {
    const { container } = render(<ContextDoc {...props({ text: "第一段散文。\n\n第二段散文。" })} />);
    expect(section(container, "context.relations")).toBeNull();
    expect(container.querySelector("[data-name-chip]")).toBeNull();
    expect(section(container, "context.intro")).toBeTruthy();
    expect(section(container, "context.record")).toBeTruthy();
  });

  it("kind improve 時種類為改進討論；無 recordPath 時無檔案列；沒有回呼的種類不成籤", () => {
    const { container } = render(
      <ContextDoc {...props({ kind: "improve", recordPath: null, onOpenArchivedChange: undefined })} />,
    );
    expect(rowValues(section(container, "context.record")!)).toEqual([
      ["created", "2026-10-08 · MomoChen"],
      ["kind", "改進討論"],
    ]);
    expect(container.querySelector('[data-name-chip^="archived:"]')).toBeNull();
    expect(container.querySelector('[data-name-chip="spec:desktop-app"]')).toBeTruthy();
  });

  it("空內文顯示空態文案", () => {
    const { container } = render(<ContextDoc {...props({ text: "\n\n" })} empty="（無背景）" />);
    expect(container.textContent).toBe("（無背景）");
    expect(container.querySelector("[data-section]")).toBeNull();
  });
});
