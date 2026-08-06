// 驗證標示（spec desktop-app「看板卡片的驗證標示」「詳情抽屜的驗證資訊列」
// 「已封存側的驗證標示」「封存入口三選項擴及驗證工單」；design D5）：卡片第二
// 顆行內小章與審查章並排、抽屜驗證資訊列、封存側結局標示、站別化三選項對話框。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

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
const VERIFY_LABELS = ["驗證中", "已驗證", "已驗證·其後有變動"];

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

describe("ChangeCard 驗證章（四態）", () => {
  it("verified 顯示「已驗證」章", () => {
    render(
      <ChangeCard
        change={{
          ...base,
          verifyStatus: "verified",
          verifiedAt: "2026-08-02",
          verifiedBy: "Ver <v@example.com>",
        }}
      />,
    );
    expect(screen.getByLabelText("已驗證")).toBeTruthy();
  });

  it("verifiedStale 顯示降級章「已驗證·其後有變動」", () => {
    render(<ChangeCard change={{ ...base, verifyStatus: "verifiedStale" }} />);
    expect(screen.getByLabelText("已驗證·其後有變動")).toBeTruthy();
  });

  it("inVerify 顯示「驗證中」章", () => {
    render(<ChangeCard change={{ ...base, verifyStatus: "inVerify" }} />);
    expect(screen.getByLabelText("驗證中")).toBeTruthy();
  });

  it("none（含缺席）無任何驗證相關元素", () => {
    for (const change of [{ ...base, verifyStatus: "none" as const }, base]) {
      const { unmount } = render(<ChangeCard change={change} />);
      for (const label of VERIFY_LABELS) {
        expect(screen.queryByLabelText(label)).toBeNull();
      }
      unmount();
    }
  });

  it("兩章並排且順序固定（審查章在前、驗證章在後）", () => {
    // spec Scenario「兩章並排」：兩站互不遮蔽，順序固定才不會每次刷新換位。
    render(
      <ChangeCard change={{ ...base, reviewStatus: "reviewed", verifyStatus: "verified" }} />,
    );
    const review = screen.getByLabelText("已審查");
    const verify = screen.getByLabelText("已驗證");
    expect(
      review.compareDocumentPosition(verify) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
  });

  it("僅驗證章時不長出審查元素", () => {
    // spec Scenario「僅驗證章」。
    render(
      <ChangeCard change={{ ...base, reviewStatus: "none", verifyStatus: "verifiedStale" }} />,
    );
    expect(screen.getByLabelText("已驗證·其後有變動")).toBeTruthy();
    for (const label of ["審查中", "已審查", "已審查·其後有變動"]) {
      expect(screen.queryByLabelText(label)).toBeNull();
    }
  });
});

describe("兩站同狀態同色、異形可辨（design D5）", () => {
  it("同為蓋章態時配色同值，圖示分屬徽章形與盾牌形", () => {
    // spec Scenario「兩站同狀態同色、異形可辨」：色承載狀態、形承載站別。
    render(
      <ChangeCard change={{ ...base, reviewStatus: "reviewed", verifyStatus: "verified" }} />,
    );
    const review = screen.getByLabelText("已審查");
    const verify = screen.getByLabelText("已驗證");
    expect(verify.className).toBe(review.className);
    // lucide 以 class 標記圖示身分：徽章形 vs 盾牌形必須不同。
    const iconClass = (el: HTMLElement) => el.querySelector("svg")?.getAttribute("class") ?? "";
    expect(iconClass(verify)).toContain("shield");
    expect(iconClass(review)).not.toContain("shield");
  });

  it("三個 active 態的驗證章配色與審查章對應態同值", () => {
    const pairs: Array<[ChangeItem["reviewStatus"], string, ChangeItem["verifyStatus"], string]> = [
      ["inReview", "審查中", "inVerify", "驗證中"],
      ["reviewed", "已審查", "verified", "已驗證"],
      ["reviewedStale", "已審查·其後有變動", "verifiedStale", "已驗證·其後有變動"],
    ];
    for (const [reviewStatus, reviewLabel, verifyStatus, verifyLabel] of pairs) {
      const { unmount } = render(
        <ChangeCard change={{ ...base, reviewStatus, verifyStatus }} />,
      );
      expect(screen.getByLabelText(verifyLabel).className).toBe(
        screen.getByLabelText(reviewLabel).className,
      );
      expect(screen.getByLabelText(verifyLabel).className).not.toContain("text-muted-foreground");
      unmount();
    }
  });
});

describe("RichDetailDrawer 驗證資訊列", () => {
  it("verified 顯示狀態詞、蓋章時間與驗證者", () => {
    // spec Scenario「已驗證抽屜」。
    render(
      <RichDetailDrawer
        {...(drawerProps({
          ...base,
          verifyStatus: "verified",
          verifiedAt: "2026-08-02",
          verifiedBy: "Ver <v@example.com>",
        }) as never)}
      />,
    );
    const row = document.querySelector("[data-verify-row]") as HTMLElement;
    expect(row).toBeTruthy();
    expect(within(row).getByText(/已驗證/)).toBeTruthy();
    expect(row.textContent).toContain("2026-08-02");
    expect(row.textContent).toContain("Ver <v@example.com>");
  });

  it("inVerify 僅顯示狀態詞，無時間與驗證者", () => {
    render(<RichDetailDrawer {...(drawerProps({ ...base, verifyStatus: "inVerify" }) as never)} />);
    const row = document.querySelector("[data-verify-row]") as HTMLElement;
    expect(row).toBeTruthy();
    expect(within(row).getByText(/驗證中/)).toBeTruthy();
    expect(row.textContent).not.toContain("2026-");
  });

  it("none 不渲染資訊列", () => {
    render(<RichDetailDrawer {...(drawerProps({ ...base, verifyStatus: "none" }) as never)} />);
    expect(document.querySelector("[data-verify-row]")).toBeNull();
  });

  it("兩站資訊列同構並列", () => {
    render(
      <RichDetailDrawer
        {...(drawerProps({
          ...base,
          reviewStatus: "reviewed",
          reviewedAt: "2026-08-01",
          verifyStatus: "verified",
          verifiedAt: "2026-08-02",
        }) as never)}
      />,
    );
    const review = document.querySelector("[data-review-row]") as HTMLElement;
    const verify = document.querySelector("[data-verify-row]") as HTMLElement;
    expect(review && verify).toBeTruthy();
    expect(verify.className).toBe(review.className);
    expect(
      review.compareDocumentPosition(verify) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
  });
});

describe("已封存側驗證結局標示", () => {
  const plain: ArchivedItem = { datedName: "2026-08-04-plain", date: "2026-08-04", name: "plain" };
  const passed: ArchivedItem = {
    datedName: "2026-08-05-passed",
    date: "2026-08-05",
    name: "passed",
    verifyStatus: "verified",
  };
  const carried: ArchivedItem = {
    datedName: "2026-08-06-carried",
    date: "2026-08-06",
    name: "carried",
    reviewStatus: "reviewed",
    verifyStatus: "verifiedNotPassed",
  };

  it("清單卡依結局標示，且與審查結局並存", () => {
    // spec Scenario「曾驗證未通過」＋「與審查結局標示可並存」。
    render(
      <ArchivedList
        archived={[plain, passed, carried]}
        query=""
        onQuery={() => {}}
        archivedDiscussions={[]}
        onOpen={() => {}}
      />,
    );
    const card = (name: string) =>
      document.querySelector(`[data-archived="${name}"]`) as HTMLElement;
    expect(within(card("2026-08-05-passed")).getByLabelText("已驗證")).toBeTruthy();
    const both = card("2026-08-06-carried");
    expect(within(both).getByLabelText("曾驗證未通過")).toBeTruthy();
    expect(within(both).getByLabelText("已審查")).toBeTruthy();
    for (const label of ["已驗證", "曾驗證未通過"]) {
      expect(within(card("2026-08-04-plain")).queryByLabelText(label)).toBeNull();
    }
    expect(screen.getByLabelText("曾驗證未通過").className).toContain("text-rose-600");
  });

  it("抽屜顯示「曾驗證未通過」標示", () => {
    render(
      <ArchivedDrawer
        open
        onOpenChange={() => {}}
        target={{ kind: "change", datedName: "2026-08-06-carried" }}
        verifyStatus="verifiedNotPassed"
        loadDocument={async () => null}
        loadCapabilities={async () => []}
        loadDiscussionDocument={async () => null}
      />,
    );
    const outcome = document.querySelector("[data-verify-outcome]") as HTMLElement;
    expect(outcome).toBeTruthy();
    expect(outcome.textContent).toContain("曾驗證未通過");
    expect(outcome.className).toContain("text-rose-600");
  });
});

describe("封存入口三選項擴及驗證工單（ReviewArchiveDialog station=verify）", () => {
  function dialogProps() {
    return {
      open: true,
      change: "demo",
      station: "verify" as const,
      onOpenChange: vi.fn(),
      onGoStamp: vi.fn(),
      onDiscardReview: vi.fn(),
      onCarryReview: vi.fn(),
    };
  }

  it("彈出驗證站文案的三選項且未選擇前不觸發任何處置", () => {
    // spec Scenario「封存驗證中的 change」：未選擇前不執行封存。
    const p = dialogProps();
    render(<ReviewArchiveDialog {...p} />);
    expect(screen.getByText("前往完成驗證蓋章")).toBeTruthy();
    expect(screen.getByText("放棄驗證")).toBeTruthy();
    expect(screen.getByText("照樣帶走")).toBeTruthy();
    expect(screen.getByText(/曾驗證未通過/)).toBeTruthy();
    expect(p.onGoStamp).not.toHaveBeenCalled();
    expect(p.onDiscardReview).not.toHaveBeenCalled();
    expect(p.onCarryReview).not.toHaveBeenCalled();
  });

  it("各選項觸發對應處置", () => {
    const p = dialogProps();
    render(<ReviewArchiveDialog {...p} />);
    fireEvent.click(screen.getByText("前往完成驗證蓋章"));
    expect(p.onGoStamp).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByText("放棄驗證"));
    expect(p.onDiscardReview).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByText("照樣帶走"));
    expect(p.onCarryReview).toHaveBeenCalledTimes(1);
  });

  it("station 預設為審查站（既有呼叫端行為不變）", () => {
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
    expect(screen.getByText("前往完成蓋章")).toBeTruthy();
    expect(screen.getByText("放棄審查")).toBeTruthy();
  });
});
