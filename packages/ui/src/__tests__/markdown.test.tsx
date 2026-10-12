// 共用 Markdown 的名稱籤外掛（desktop-discussion-doc-views design D10）：`nameChips` 有值時，文字節點中與名單
// 完全相同的字詞（D1 matchNames 同一條邊界）換成 `span.name-chip[data-name-chip="<kind>:<name>"]`；
// 行內 code、程式碼區塊與連結內不處理；缺席時管線與輸出不變。
import { describe, expect, it } from "vitest";
import { render } from "@testing-library/react";
import type { ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { Markdown } from "../components/Markdown";

const wrapper = ({ children }: { children: ReactNode }) => <I18nProvider locale="zh-TW">{children}</I18nProvider>;

const chips = (container: HTMLElement) =>
  Array.from(container.querySelectorAll("[data-name-chip]")).map((el) => [
    el.getAttribute("data-name-chip"),
    el.textContent,
    el.className,
  ]);

describe("Markdown 名稱籤", () => {
  it("內文提到 desktop-app 與 add-copilot-tool，傳入兩顆籤後各成一個 [data-name-chip] 且文字不變", () => {
    const content = "偵察命中 desktop-app，在途變更 add-copilot-tool 也碰到 desktop-app。";
    const { container } = render(
      <Markdown
        content={content}
        nameChips={[
          { kind: "spec", name: "desktop-app" },
          { kind: "change", name: "add-copilot-tool" },
        ]}
      />,
      { wrapper },
    );
    expect(chips(container)).toEqual([
      ["spec:desktop-app", "desktop-app", "name-chip"],
      ["change:add-copilot-tool", "add-copilot-tool", "name-chip"],
      ["spec:desktop-app", "desktop-app", "name-chip"],
    ]);
    expect(container.querySelector(".markdown")!.textContent).toBe(content);
  });

  it("反引號內、程式碼區塊內、連結內與 desktop-app-x 這類前綴不命中", () => {
    const { container } = render(
      <Markdown
        content={"`desktop-app` 與 desktop-app-x、[desktop-app](https://example.com)\n\n```\ndesktop-app\n```"}
        nameChips={[{ kind: "spec", name: "desktop-app" }]}
      />,
      { wrapper },
    );
    expect(chips(container)).toEqual([]);
  });

  it("不傳 nameChips 時輸出 HTML 與傳空陣列相同", () => {
    const content = "提到 desktop-app 與 **粗體**。\n\n- 列點 SHALL 照常";
    const a = render(<Markdown content={content} />, { wrapper }).container.innerHTML;
    const b = render(<Markdown content={content} nameChips={[]} />, { wrapper }).container.innerHTML;
    expect(b).toBe(a);
    expect(a).not.toContain("data-name-chip");
  });
});
