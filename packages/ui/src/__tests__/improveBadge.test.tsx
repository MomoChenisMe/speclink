// 改進標示（spec desktop-app「看板討論卡片的改進標示」「討論抽屜的改進標示」；
// change add-improve-flow）：kind 為 improve 的討論在卡片、已封存側與抽屜各顯示
// 同一枚小章，一般討論不長出任何新元素；標示隨 kind 恆定，不隨生命週期變化。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider, MESSAGES } from "../i18n";

const wrapWith = (locale: "zh-TW" | "en") =>
  function Wrapper({ children }: { children: ReactNode }) {
    return <I18nProvider locale={locale}>{children}</I18nProvider>;
  };
function render(ui: ReactElement, locale: "zh-TW" | "en" = "zh-TW") {
  return rtlRender(ui, { wrapper: wrapWith(locale) });
}

import { DiscussionCard } from "../components/DiscussionColumn";
import { DiscussionDrawer } from "../components/DiscussionDrawer";
import { ArchivedList } from "../components/ArchivedList";
import type { DiscussionItem } from "../adapter";

const IMPROVE_TW = "改進討論";
const IMPROVE_EN = MESSAGES.en["discussion.kindImprove"];

const plain: DiscussionItem = {
  slug: "board-search-bar",
  topic: "看板搜尋列",
  status: "open",
  rounds: 2,
  created: "2026-08-01",
  promotedTo: [],
};
const improve: DiscussionItem = {
  slug: "improve-store-layer",
  topic: "store 層結構改進",
  status: "open",
  rounds: 1,
  created: "2026-08-07",
  kind: "improve",
  promotedTo: [],
};

/** DiscussionDrawer 的最小 props（標示以外的相依一律給空替身）。 */
function drawerProps(discussion: DiscussionItem) {
  return {
    open: true,
    onOpenChange: vi.fn(),
    discussion,
    changes: [],
    archivedChanges: [],
    loadDocument: vi.fn(async () => "## Rounds\n"),
  };
}

describe("看板討論卡片的改進標示", () => {
  it("kind 為 improve 的卡片出現小章，tooltip 為「改進討論」", () => {
    render(<DiscussionCard d={improve} />);
    expect(screen.getByLabelText(IMPROVE_TW)).toBeTruthy();
  });

  it("一般討論（無 kind）不出現任何新增元素", () => {
    render(<DiscussionCard d={plain} />);
    expect(screen.queryByLabelText(IMPROVE_TW)).toBeNull();
  });

  it("已轉出的改進討論維持標示（標示隨 kind 恆定，不隨生命週期變化）", () => {
    render(<DiscussionCard d={{ ...improve, status: "promoted", promotedTo: ["cut-a"] }} />);
    expect(screen.getByLabelText(IMPROVE_TW)).toBeTruthy();
  });

  it("en 介面使用對應英文詞條，不落回中文", () => {
    render(<DiscussionCard d={improve} />, "en");
    expect(screen.getByLabelText(IMPROVE_EN)).toBeTruthy();
    expect(screen.queryByLabelText(IMPROVE_TW)).toBeNull();
  });
});

describe("已封存側的改進標示", () => {
  it("已封存的改進討論仍顯示小章，一般討論不顯示", () => {
    render(
      <ArchivedList
        archived={[]}
        query=""
        onQuery={() => {}}
        archivedDiscussions={[improve, plain]}
        onOpen={() => {}}
      />,
    );
    // 已封存頁預設停在「變更」子頁籤；討論節要切過去才渲染。
    fireEvent.mouseDown(screen.getByRole("tab", { name: /已封存的討論/ }));
    const card = (slug: string) =>
      document.querySelector(`[data-archived-discussion="${slug}"]`) as HTMLElement;
    expect(within(card("improve-store-layer")).getByLabelText(IMPROVE_TW)).toBeTruthy();
    expect(within(card("board-search-bar")).queryByLabelText(IMPROVE_TW)).toBeNull();
  });
});

describe("討論抽屜的改進標示", () => {
  it("kind 為 improve 時顯示標示", () => {
    render(<DiscussionDrawer {...(drawerProps(improve) as never)} />);
    const badge = document.querySelector("[data-discussion-kind]") as HTMLElement;
    expect(badge).toBeTruthy();
    expect(badge.textContent).toContain(IMPROVE_TW);
  });

  it("一般討論的抽屜不顯示標示", () => {
    render(<DiscussionDrawer {...(drawerProps(plain) as never)} />);
    expect(document.querySelector("[data-discussion-kind]")).toBeNull();
  });

  it("en 介面的抽屜標示使用英文詞條", () => {
    render(<DiscussionDrawer {...(drawerProps(improve) as never)} />, "en");
    const badge = document.querySelector("[data-discussion-kind]") as HTMLElement;
    expect(badge.textContent).toContain(IMPROVE_EN);
  });
});
