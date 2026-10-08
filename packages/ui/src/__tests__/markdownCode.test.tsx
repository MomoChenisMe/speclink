// 程式碼區塊上色與複製鈕（desktop-design-foundation D10）：Streamdown 的 code 外掛以
// Shiki 上色（淺色 github-light、深色 github-dark），區塊右上有複製鈕，按下後寫入原文並
// 宣告「已複製」；語言未載入時以無上色的等寬文字呈現、不拋錯。
import { describe, it, expect, vi, afterEach } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { Markdown } from "../components/Markdown";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
const CODE = "const answer: number = 42;\nconsole.log(answer);";

afterEach(cleanup);

describe("Markdown 程式碼區塊", () => {
  it("複製鈕寫入原文，按下後宣告「已複製」並換成勾號", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
    render(<Markdown content={"```ts\n" + CODE + "\n```"} />, { wrapper: zhWrapper });
    const button = screen.getByRole("button", { name: "複製程式碼" });
    fireEvent.click(button);
    await waitFor(() => expect(writeText).toHaveBeenCalledTimes(1));
    // Streamdown 複製的是 fence 內原文（尾端帶 fence 前的換行）。
    expect((writeText.mock.calls[0][0] as string).replace(/\n$/, "")).toBe(CODE);
    expect(await screen.findByText("已複製")).toBeTruthy();
    expect(button.querySelector("svg.lucide-check")).toBeTruthy();
  });

  it("以 Shiki 上色：關鍵字帶 github-light 與 github-dark 兩套色", async () => {
    const { container } = render(<Markdown content={"```ts\n" + CODE + "\n```"} />, {
      wrapper: zhWrapper,
    });
    const pre = container.querySelector("pre") as HTMLElement;
    await waitFor(
      () => {
        const keyword = [...pre.querySelectorAll("span[style]")].find(
          (s) => s.textContent === "const",
        ) as HTMLElement | undefined;
        expect(keyword).toBeTruthy();
        const style = keyword!.getAttribute("style")!.toUpperCase();
        expect(style).toContain("#D73A49"); // github-light keyword
        expect(style).toContain("#F97583"); // github-dark keyword
      },
      { timeout: 5000 },
    );
  });

  it("未知語言以等寬原文呈現、不拋錯", async () => {
    const { container } = render(<Markdown content={"```nosuchlang\nplain <b>text</b>\n```"} />, {
      wrapper: zhWrapper,
    });
    const pre = container.querySelector("pre") as HTMLElement;
    expect(pre.textContent).toContain("plain <b>text</b>");
    expect(screen.getByRole("button", { name: "複製程式碼" })).toBeTruthy();
  });
});
