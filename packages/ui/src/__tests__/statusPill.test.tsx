// 狀態籤原語（design D5）：圓角全圓、語意色淡底深字。章籤、討論狀態籤與衍生
// 變更階段籤共用；色只走 theme token，不寫原生色階。
import { describe, it, expect, afterEach } from "vitest";
import { render, screen, cleanup } from "@testing-library/react";

import { StatusPill, type StatusPillTone } from "../components/ui/status-pill";
import { REVIEW_PILL_TONE } from "../components/reviewStyle";
import { VERIFY_PILL_TONE } from "../components/verifyStyle";

afterEach(cleanup);

const TONE_CLASSES: [StatusPillTone, string[]][] = [
  ["neutral", ["bg-muted", "text-muted-foreground"]],
  ["primary", ["bg-primary/12", "text-primary"]],
  ["progress", ["bg-status-progress/12", "text-status-progress"]],
  ["success", ["bg-status-success/12", "text-status-success"]],
  ["warning", ["bg-status-warning/12", "text-status-warning"]],
  ["destructive", ["bg-destructive/12", "text-destructive"]],
  ["stamp", ["bg-stamp/12", "text-stamp"]],
  ["improve", ["bg-improve/12", "text-improve"]],
];

describe("StatusPill", () => {
  it.each(TONE_CLASSES)("tone %s 渲染對應 token 淡底深字", (tone, classes) => {
    render(<StatusPill tone={tone}>籤</StatusPill>);
    const pill = screen.getByText("籤");
    for (const cls of classes) expect(pill.className.split(/\s+/)).toContain(cls);
    expect(pill.className).toContain("rounded-full");
    expect(pill.className).toContain("h-5");
    expect(pill.className).toContain("px-2");
    expect(pill.className).toContain("text-[11px]");
  });

  it("任何 tone 都不出現原生色階字面", () => {
    for (const [tone] of TONE_CLASSES) {
      const { container, unmount } = render(<StatusPill tone={tone}>x</StatusPill>);
      expect(container.innerHTML).not.toMatch(/-(?:red|blue|green|amber|violet|purple|gray|slate|zinc)-\d{2,3}\b/);
      unmount();
    }
  });

  it("icon 槽渲染於文字之前", () => {
    render(
      <StatusPill tone="stamp" icon={<svg data-testid="pill-icon" />}>
        已審查
      </StatusPill>,
    );
    const pill = screen.getByText("已審查");
    const icon = screen.getByTestId("pill-icon");
    expect(pill.contains(icon)).toBe(true);
    expect(pill.firstElementChild).toBe(icon);
  });

  it("className 併入而不取代基底", () => {
    render(
      <StatusPill tone="neutral" className="line-through">
        已刪除
      </StatusPill>,
    );
    const pill = screen.getByText("已刪除");
    expect(pill.className).toContain("line-through");
    expect(pill.className).toContain("rounded-full");
  });
});

describe("章籤 tone 對照", () => {
  it("審查四態：已審查 stamp、進行中 progress、過期 warning、未通過 destructive", () => {
    expect(REVIEW_PILL_TONE).toEqual({
      inReview: "progress",
      reviewed: "stamp",
      reviewedStale: "warning",
      reviewedNotPassed: "destructive",
    });
  });

  it("驗證四態與審查同一語意", () => {
    expect(VERIFY_PILL_TONE).toEqual({
      inVerify: "progress",
      verified: "stamp",
      verifiedStale: "warning",
      verifiedNotPassed: "destructive",
    });
  });
});
