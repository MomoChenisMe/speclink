// 詳情頁「狀態」列的標籤欄（fix-english-status-labels-and-plurals D1、D5；desktop-app「詳情頁狀態列的
// 標籤欄依最長標籤對齊」）：各列以 subgrid 共用同一個標籤欄，欄寬取最長標籤（下限 56px、上限 112px），
// 英文單字標籤不再溢出到數值上。jsdom 量不到版面，這裡釘住結構；畫面由實機驗收。
import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";

import { DetailStatus } from "../components/DetailShell";
import { StatusRow } from "../components/DetailHeader";

function renderStatus() {
  return render(
    <DetailStatus>
      <StatusRow mark="requirements" label="Requirements">
        2
      </StatusRow>
      <StatusRow mark="sources" label="Provenance">
        1
      </StatusRow>
    </DetailStatus>,
  );
}

describe("詳情頁狀態列的標籤欄", () => {
  it("容器是兩欄格線：標籤欄依內容寬、數值欄吃剩下的寬", () => {
    const { container } = renderStatus();
    const section = container.querySelector("[data-status-card]") as HTMLElement;
    expect(section.className).toContain("grid");
    expect(section.className).toContain("grid-cols-[max-content_minmax(0,1fr)]");
  });

  it("每一列跨兩欄並以 subgrid 共用容器的欄", () => {
    const { container } = renderStatus();
    for (const mark of ["requirements", "sources"]) {
      const row = container.querySelector(`[data-status-${mark}]`) as HTMLElement;
      expect(row.className).toContain("col-span-2");
      expect(row.className).toContain("grid-cols-subgrid");
    }
  });

  it("標籤寬度下限 56px、上限 112px、可在字間折行，不再寫死 w-14", () => {
    const { container } = renderStatus();
    const label = container.querySelector("[data-status-requirements] > span") as HTMLElement;
    expect(label.textContent).toBe("Requirements");
    const classes = label.className.split(/\s+/);
    expect(classes).toContain("min-w-14");
    expect(classes).toContain("max-w-28");
    expect(classes).toContain("break-words");
    expect(classes).not.toContain("w-14");
  });
});
