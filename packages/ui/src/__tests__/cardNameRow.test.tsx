// 卡片名稱列（spec「看板卡片統一解剖學」，取代 design D4 的折行版本）：名稱恆
// 單行不換行，複製鈕與名稱同一列尾隨不被擠到次行；名稱過長截斷時尾端漸層淡出
// （取代硬切）。變更卡與討論卡共用同一列，兩者行為必須一致。
import { describe, it, expect, afterEach, vi } from "vitest";
import { render as rtlRender, screen, cleanup } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { ChangeCard } from "../components/ChangeCard";
import { DiscussionColumn } from "../components/DiscussionColumn";
import type { ChangeItem, DiscussionItem } from "../adapter";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const base: ChangeItem = {
  name: "worktree-parallel-apply",
  status: "in-progress",
  totalTasks: 14,
  completedTasks: 0,
};

/** jsdom 的 scrollWidth/clientWidth 恆為 0：以 getter stub 假造「名稱溢出與否」。 */
function stubWidths(scroll: number, client: number) {
  vi.spyOn(HTMLElement.prototype, "scrollWidth", "get").mockReturnValue(scroll);
  vi.spyOn(HTMLElement.prototype, "clientWidth", "get").mockReturnValue(client);
}

afterEach(() => {
  vi.restoreAllMocks();
  cleanup();
});

describe("ChangeCard 名稱列", () => {
  it("名稱不換行，複製鈕與名稱在同一列容器內", () => {
    render(<ChangeCard change={base} />);
    const name = document.querySelector("[data-name]") as HTMLElement;
    const copy = screen.getByLabelText("複製名稱");
    expect(name.className).toContain("whitespace-nowrap");
    expect(name.parentElement).toBe(copy.parentElement);
    // 名稱被壓縮、複製鈕保持原寬（不被擠到次行的必要條件）。
    expect(name.className).toContain("min-w-0");
    expect(copy.className).toContain("shrink-0");
  });

  it("名稱溢出時尾端套漸層淡出遮罩", () => {
    stubWidths(320, 160);
    render(<ChangeCard change={base} />);
    const name = document.querySelector("[data-name]") as HTMLElement;
    expect(name.dataset.fade).toBe("true");
    expect(name.style.maskImage).toContain("linear-gradient");
  });

  it("名稱未溢出時不套遮罩（不誤淡末尾字元）", () => {
    stubWidths(120, 160);
    render(<ChangeCard change={{ ...base, name: "short" }} />);
    const name = document.querySelector("[data-name]") as HTMLElement;
    expect(name.dataset.fade).toBeUndefined();
    expect(name.style.maskImage).toBe("");
    // 短名稱路徑也釘住同列關係（長名稱由第一條驗）。
    expect(name.parentElement).toBe(screen.getByLabelText("複製名稱").parentElement);
  });

  it("長名稱擠壓時 meta icons 不被擠出卡外", () => {
    stubWidths(320, 160);
    render(
      <ChangeCard
        change={{ ...base, createdBy: "Momo Chen <m@example.com>", fromDiscussions: ["some-topic"] }}
      />,
    );
    const name = document.querySelector("[data-name]") as HTMLElement;
    // jsdom 量不到版面，class 是唯一可釘的層次：只有名稱列吸收擠壓，
    // meta icons 各自固定寬度。任一邊被拿掉都會讓 icons 被擠出卡外。
    expect(name.parentElement?.className).toContain("flex-1");
    expect(name.parentElement?.className).toContain("min-w-0");
    expect(screen.getByLabelText("Momo Chen <m@example.com>").className).toContain("shrink-0");
    expect(screen.getByLabelText("來自討論").className).toContain("shrink-0");
  });
});

const discussion: DiscussionItem = {
  slug: "cross-station-staleness",
  topic: "跨站陳舊",
  status: "open",
  rounds: 3,
  created: "2026-07-01",
  promotedTo: [],
};

describe("DiscussionColumn 全卡的 slug 列（與變更卡同骨架）", () => {
  it("slug 不換行，複製鈕與 slug 在同一列容器內", () => {
    render(<DiscussionColumn discussions={[discussion]} changes={[]} archived={[]} />);
    const name = document.querySelector("[data-name]") as HTMLElement;
    const copy = screen.getByLabelText("複製 slug");
    expect(name.className).toContain("whitespace-nowrap");
    // slug 過長不再 break-all 折行——單行截斷才留得住尾隨的複製鈕。
    expect(name.className).not.toContain("break-all");
    expect(name.parentElement).toBe(copy.parentElement);
  });

  it("slug 溢出時尾端套漸層淡出遮罩", () => {
    stubWidths(320, 160);
    render(<DiscussionColumn discussions={[discussion]} changes={[]} archived={[]} />);
    const name = document.querySelector("[data-name]") as HTMLElement;
    expect(name.dataset.fade).toBe("true");
    expect(name.style.maskImage).toContain("linear-gradient");
  });
});
