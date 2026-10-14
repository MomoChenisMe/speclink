// 嚴重度色章（審查／驗證工單、分析面板、desktop 審查規則頁共用；review-rules-config 品質關卡 R4）。
import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";

import { SeverityTag } from "../components/SeverityTag";

const tagOf = (ui: Parameters<typeof render>[0]) => render(ui).container.firstElementChild!;

describe("SeverityTag", () => {
  it("三級同語意同色、大小寫不拘；未知嚴重度走中性", () => {
    expect(tagOf(<SeverityTag severity="CRITICAL" />).className).toContain("text-destructive");
    expect(tagOf(<SeverityTag severity="Critical" />).className).toContain("text-destructive");
    expect(tagOf(<SeverityTag severity="Warning" />).className).toContain("status-warning");
    expect(tagOf(<SeverityTag severity="SUGGESTION" />).className).toContain("text-muted-foreground");
    expect(tagOf(<SeverityTag severity="INFO" />).className).toContain("text-muted-foreground");
  });

  it("預設顯示嚴重度名稱；可換成自訂內容並帶 data 屬性", () => {
    expect(tagOf(<SeverityTag severity="CRITICAL" />).textContent).toBe("CRITICAL");
    const tag = tagOf(
      <SeverityTag severity="WARNING" data-ticket-count="WARNING">
        WARNING 3
      </SeverityTag>,
    );
    expect(tag.textContent).toBe("WARNING 3");
    expect(tag.getAttribute("data-ticket-count")).toBe("WARNING");
  });
});
