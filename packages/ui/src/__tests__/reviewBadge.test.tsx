// 審查標示（spec desktop-app「看板卡片的審查標示」「詳情抽屜的審查資訊列」
// 「已封存側的審查標示」「封存入口的未結工單三選項」；design D6）：卡片行內
// 小章四態、抽屜資訊列、封存側結局標示、三選項對話框未選擇不封存。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";

// 既有中文斷言包 I18nProvider locale zh-TW 後照舊斷言（i18n 抽 key 的回歸保護）。
const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

// 攔截 TaskList（jsdom 不需要真任務清單；沿 richDrawer.test 慣例）。
vi.mock("../components/TaskList", () => ({
  TaskList: () => <div data-testid="tasklist-stub" />,
}));

import { ChangeCard } from "../components/ChangeCard";
import { RichDetailDrawer } from "../components/RichDetailDrawer";
import { ArchivedList } from "../components/ArchivedList";
import { ArchivedDrawer } from "../components/ArchivedDrawer";
import { ReviewArchiveDialog } from "../components/ReviewArchiveDialog";
import type { ArchivedItem, ChangeItem } from "../adapter";

const base: ChangeItem = { name: "c", status: "in-progress", totalTasks: 2, completedTasks: 1 };

const REVIEW_LABELS = ["審查中", "已審查", "已審查·其後有變動"];

/** RichDetailDrawer 的最小 props（審查資訊列以外的相依一律給空替身）。 */
function drawerProps(change: ChangeItem) {
  return {
    open: true,
    onOpenChange: vi.fn(),
    change,
    loadDocument: vi.fn(async () => "# doc"),
    loadCapabilities: vi.fn(async () => [] as string[]),
    loadMeta: vi.fn(async () => null),
    onRunVerb: vi.fn(),
    onDelete: vi.fn(),
  };
}

describe("ChangeCard 審查章（四態）", () => {
  it("reviewed 顯示「已審查」章", () => {
    render(
      <ChangeCard
        change={{ ...base, reviewStatus: "reviewed", reviewedAt: "2026-08-01", reviewedBy: "Rev <r@example.com>" }}
      />,
    );
    expect(screen.getByLabelText("已審查")).toBeTruthy();
  });

  it("reviewedStale 顯示降級章「已審查·其後有變動」", () => {
    render(
      <ChangeCard
        change={{ ...base, reviewStatus: "reviewedStale", reviewedAt: "2026-08-01", reviewedBy: "Rev <r@example.com>" }}
      />,
    );
    expect(screen.getByLabelText("已審查·其後有變動")).toBeTruthy();
  });

  it("inReview 顯示「審查中」章", () => {
    render(<ChangeCard change={{ ...base, reviewStatus: "inReview" }} />);
    expect(screen.getByLabelText("審查中")).toBeTruthy();
  });

  it("none（含缺席）無任何審查相關元素", () => {
    // spec Scenario「無審查痕跡」：卡片維持極簡，不因欄位存在而長出空殼。
    for (const change of [{ ...base, reviewStatus: "none" as const }, base]) {
      const { unmount } = render(<ChangeCard change={change} />);
      for (const label of REVIEW_LABELS) {
        expect(screen.queryByLabelText(label)).toBeNull();
      }
      unmount();
    }
  });
});

describe("RichDetailDrawer 審查資訊列", () => {
  it("reviewed 顯示狀態詞、蓋章時間與審查者", () => {
    render(
      <RichDetailDrawer
        {...(drawerProps({
          ...base,
          reviewStatus: "reviewed",
          reviewedAt: "2026-08-01",
          reviewedBy: "Rev <r@example.com>",
        }) as never)}
      />,
    );
    const row = document.querySelector("[data-review-row]") as HTMLElement;
    expect(row).toBeTruthy();
    expect(within(row).getByText(/已審查/)).toBeTruthy();
    expect(row.textContent).toContain("2026-08-01");
    expect(row.textContent).toContain("Rev <r@example.com>");
  });

  it("inReview 僅顯示狀態詞，無時間與審查者", () => {
    render(<RichDetailDrawer {...(drawerProps({ ...base, reviewStatus: "inReview" }) as never)} />);
    const row = document.querySelector("[data-review-row]") as HTMLElement;
    expect(row).toBeTruthy();
    expect(within(row).getByText(/審查中/)).toBeTruthy();
    expect(row.textContent).not.toContain("2026-");
  });

  it("none 不渲染資訊列", () => {
    render(<RichDetailDrawer {...(drawerProps({ ...base, reviewStatus: "none" }) as never)} />);
    expect(document.querySelector("[data-review-row]")).toBeNull();
  });
});

describe("已封存側審查結局標示", () => {
  const plain: ArchivedItem = { datedName: "2026-07-04-plain", date: "2026-07-04", name: "plain" };
  const passed: ArchivedItem = {
    datedName: "2026-07-05-passed",
    date: "2026-07-05",
    name: "passed",
    reviewStatus: "reviewed",
  };
  const carried: ArchivedItem = {
    datedName: "2026-07-06-carried",
    date: "2026-07-06",
    name: "carried",
    reviewStatus: "reviewedNotPassed",
  };

  function renderList() {
    return render(
      <ArchivedList
        archived={[plain, passed, carried]}
        query=""
        onQuery={() => {}}
        archivedDiscussions={[]}
        onOpen={() => {}}
      />,
    );
  }

  it("清單卡依結局標示：帶章＝已審查、化石工單＝曾審查未通過、皆無＝無標示", () => {
    renderList();
    const card = (name: string) =>
      document.querySelector(`[data-archived="${name}"]`) as HTMLElement;
    expect(within(card("2026-07-05-passed")).getByLabelText("已審查")).toBeTruthy();
    expect(within(card("2026-07-06-carried")).getByLabelText("曾審查未通過")).toBeTruthy();
    for (const label of ["已審查", "曾審查未通過"]) {
      expect(within(card("2026-07-04-plain")).queryByLabelText(label)).toBeNull();
    }
  });

  it("抽屜顯示「曾審查未通過」標示", () => {
    render(
      <ArchivedDrawer
        open
        onOpenChange={() => {}}
        target={{ kind: "change", datedName: "2026-07-06-carried" }}
        reviewStatus="reviewedNotPassed"
        loadDocument={async () => null}
        loadCapabilities={async () => []}
        loadDiscussionDocument={async () => null}
      />,
    );
    expect(screen.getByText(/曾審查未通過/)).toBeTruthy();
  });
});

describe("審查標示配色（四態各自可辨識，不落回灰階）", () => {
  const archivedItems: ArchivedItem[] = [
    { datedName: "2026-07-05-passed", date: "2026-07-05", name: "passed", reviewStatus: "reviewed" },
    {
      datedName: "2026-07-06-carried",
      date: "2026-07-06",
      name: "carried",
      reviewStatus: "reviewedNotPassed",
    },
  ];

  it("卡片章：審查中＝藍、已審查＝主色、其後有變動＝琥珀", () => {
    const tones: Array<[ChangeItem["reviewStatus"], string, string]> = [
      ["inReview", "審查中", "text-sky-600"],
      ["reviewed", "已審查", "text-primary"],
      ["reviewedStale", "已審查·其後有變動", "text-amber-600"],
    ];
    for (const [reviewStatus, label, cls] of tones) {
      const { unmount } = render(<ChangeCard change={{ ...base, reviewStatus }} />);
      const stamp = screen.getByLabelText(label);
      expect(stamp.className).toContain(cls);
      expect(stamp.className).not.toContain("text-muted-foreground");
      unmount();
    }
  });

  it("抽屜資訊列：狀態詞依狀態上色", () => {
    const tones: Array<[ChangeItem["reviewStatus"], string]> = [
      ["inReview", "text-sky-600"],
      ["reviewed", "text-primary"],
      ["reviewedStale", "text-amber-600"],
    ];
    for (const [reviewStatus, cls] of tones) {
      const { unmount } = render(
        <RichDetailDrawer {...(drawerProps({ ...base, reviewStatus }) as never)} />,
      );
      const tone = document.querySelector("[data-review-row] [data-review-tone]") as HTMLElement;
      expect(tone).toBeTruthy();
      expect(tone.className).toContain(cls);
      unmount();
    }
  });

  it("已封存清單：已審查＝主色、曾審查未通過＝紅", () => {
    render(
      <ArchivedList
        archived={archivedItems}
        query=""
        onQuery={() => {}}
        archivedDiscussions={[]}
        onOpen={() => {}}
      />,
    );
    expect(screen.getByLabelText("已審查").className).toContain("text-primary");
    expect(screen.getByLabelText("曾審查未通過").className).toContain("text-rose-600");
  });

  it("已封存抽屜：已審查＝主色、曾審查未通過＝紅", () => {
    for (const [reviewStatus, cls] of [
      ["reviewed", "text-primary"],
      ["reviewedNotPassed", "text-rose-600"],
    ] as const) {
      const { unmount } = render(
        <ArchivedDrawer
          open
          onOpenChange={() => {}}
          target={{ kind: "change", datedName: "2026-07-06-x" }}
          reviewStatus={reviewStatus}
          loadDocument={async () => null}
          loadCapabilities={async () => []}
          loadDiscussionDocument={async () => null}
        />,
      );
      const outcome = document.querySelector("[data-review-outcome]") as HTMLElement;
      expect(outcome.className).toContain(cls);
      expect(outcome.className).not.toContain("text-muted-foreground");
      unmount();
    }
  });

  it("「照樣帶走」按鈕用未通過的紅（與其永久標示同語意）", () => {
    render(
      <ReviewArchiveDialog
        open
        change="demo"
        onOpenChange={vi.fn()}
        onGoStamp={vi.fn()}
        onDiscardReview={vi.fn()}
        onCarryReview={vi.fn()}
      />,
    );
    expect(screen.getByText("照樣帶走").className).toContain("text-rose-600");
  });
});

describe("封存入口的未結工單三選項（ReviewArchiveDialog）", () => {
  function dialogProps() {
    return {
      open: true,
      change: "demo",
      onOpenChange: vi.fn(),
      onGoStamp: vi.fn(),
      onDiscardReview: vi.fn(),
      onCarryReview: vi.fn(),
    };
  }

  it("彈出三選項且未選擇前不觸發任何處置", () => {
    // spec Scenario「封存審查中的 change」：未選擇前不執行封存。
    const p = dialogProps();
    render(<ReviewArchiveDialog {...p} />);
    expect(screen.getByText("前往完成蓋章")).toBeTruthy();
    expect(screen.getByText("放棄審查")).toBeTruthy();
    expect(screen.getByText("照樣帶走")).toBeTruthy();
    expect(p.onGoStamp).not.toHaveBeenCalled();
    expect(p.onDiscardReview).not.toHaveBeenCalled();
    expect(p.onCarryReview).not.toHaveBeenCalled();
    // 照樣帶走的永久標示警語（spec：說明將永久顯示「曾審查未通過」）。
    expect(screen.getByText(/曾審查未通過/)).toBeTruthy();
  });

  it("各選項觸發對應處置", () => {
    const p = dialogProps();
    render(<ReviewArchiveDialog {...p} />);
    fireEvent.click(screen.getByText("前往完成蓋章"));
    expect(p.onGoStamp).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByText("放棄審查"));
    expect(p.onDiscardReview).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByText("照樣帶走"));
    expect(p.onCarryReview).toHaveBeenCalledTimes(1);
  });
});
