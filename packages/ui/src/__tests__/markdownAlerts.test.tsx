// spec desktop-manual-page「Markdown 的 GitHub Alert 提示框」（design：共用 Markdown
// 元件的內建 remark 轉換、不新增依賴）：四型 blockquote 呈現為帶類型 class 與類型
// 標籤的提示框、標記文字消失、其餘內容保留；首段不以標記開頭的 blockquote 與
// 純 react-markdown 管線輸出逐位元一致；配色取介面狀態語意色 token。
import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import type { ReactNode } from "react";
import ReactMarkdown from "react-markdown";
import remarkBreaks from "remark-breaks";
import remarkGfm from "remark-gfm";

import { I18nProvider, type UiLocale } from "../i18n";
import { Markdown } from "../components/Markdown";
import { SEMANTIC_SURFACE, SEMANTIC_TONE } from "../tone";

function renderMd(content: string, locale: UiLocale = "zh-TW") {
  const wrapper = ({ children }: { children: ReactNode }) => (
    <I18nProvider locale={locale}>{children}</I18nProvider>
  );
  return render(<Markdown content={content} />, { wrapper });
}

const alertBox = (container: HTMLElement, type: string) =>
  container.querySelector(`.markdown-alert.markdown-alert-${type}`) as HTMLElement | null;

describe("Markdown GitHub Alert 提示框", () => {
  it.each([
    ["NOTE", "note", "注意", "inProgress"],
    ["TIP", "tip", "提示", "success"],
    ["WARNING", "warning", "警告", "warning"],
    ["CAUTION", "caution", "小心", "danger"],
  ] as const)("`> [!%s]` 呈現為 %s 提示框：類型標籤、標記消失、其餘內容保留、語意色分層", (marker, type, label, tone) => {
    // spec Scenario「四型提示框」。
    const { container } = renderMd(`> [!${marker}]\n> 第一行內容\n> 第二行 **粗體**\n\n一般段落。`);
    const box = alertBox(container, type);
    expect(box).toBeTruthy();
    expect(box!.tagName).not.toBe("BLOCKQUOTE");
    // 類型標籤（LANGUAGE 用語）＋語意色（狀態色 token，不佔主色）。
    const title = box!.querySelector(".markdown-alert-title") as HTMLElement;
    expect(title.textContent).toBe(label);
    for (const cls of SEMANTIC_TONE[tone].split(" ")) expect(title.className).toContain(cls);
    for (const cls of SEMANTIC_SURFACE[tone].split(" ")) expect(box!.className).toContain(cls);
    // 標記文字不出現，blockquote 內其餘文字完整顯示（含行內格式）。
    expect(container.textContent).not.toContain(`[!${marker}]`);
    expect(box!.textContent).toContain("第一行內容");
    expect(box!.textContent).toContain("第二行");
    expect(box!.querySelector("strong")?.textContent).toBe("粗體");
    // 提示框外的內容照常。
    expect(container.textContent).toContain("一般段落。");
    expect(container.querySelector("blockquote")).toBeNull();
  });

  it("類型標籤跟隨介面語言（en）", () => {
    const { container } = renderMd("> [!WARNING]\n> 內容", "en");
    expect(alertBox(container, "warning")!.querySelector(".markdown-alert-title")!.textContent).toBe(
      "Warning",
    );
  });

  it("小寫標記同樣轉換（GitHub 也接受）", () => {
    const { container } = renderMd("> [!note]\n> 小寫內容");
    const box = alertBox(container, "note")!;
    expect(box.querySelector(".markdown-alert-title")!.textContent).toBe("注意");
    expect(box.textContent).toContain("小寫內容");
    expect(container.textContent).not.toContain("[!note]");
  });

  it("標記行以硬換行結尾（行尾兩空格）時，提示框內文不以 <br> 起頭", () => {
    const { container } = renderMd("> [!TIP]  \n> 第一行\n> 第二行");
    const box = alertBox(container, "tip")!;
    const first = box.querySelector("p") as HTMLElement;
    expect(first.innerHTML.startsWith("<br")).toBe(false);
    expect(first.textContent).toContain("第一行");
    expect(first.textContent).toContain("第二行");
  });

  it("標記之後的多段內容全部保留在提示框內", () => {
    const { container } = renderMd("> [!TIP]\n> 第一段\n>\n> 第二段\n>\n> - 清單項");
    const box = alertBox(container, "tip")!;
    expect(box.querySelectorAll("p").length).toBe(2);
    expect(box.querySelector("li")?.textContent).toBe("清單項");
    expect(box.textContent).toContain("第一段");
    expect(box.textContent).toContain("第二段");
  });

  it("首段不以四種標記開頭的 blockquote：輸出與純 react-markdown 管線逐位元一致", () => {
    // spec Scenario「一般引言不受影響」：一般引言、標記不在首段行首、標記與文字
    // 同行（GitHub 不視為 alert）、未知類型、非 blockquote 的標記文字。
    const md = [
      "> 一般引言\n> 第二行",
      "> 前導文字\n> [!NOTE]",
      "> [!NOTE] 同行文字",
      "> [!IMPORTANT]\n> 未支援的類型",
      "> **粗體** [!TIP]",
      "[!NOTE] 不在 blockquote 內",
      "> 第一段\n>\n> [!WARNING]\n> 第二段才有標記",
    ].join("\n\n");
    const { container: ours } = renderMd(md);
    const { container: reference } = render(
      <ReactMarkdown remarkPlugins={[remarkGfm, remarkBreaks]} skipHtml>
        {md}
      </ReactMarkdown>,
    );
    expect(ours.querySelector(".markdown")!.innerHTML).toBe(reference.innerHTML);
    expect(ours.querySelectorAll("blockquote").length).toBe(6);
    expect(ours.querySelector(".markdown-alert")).toBeNull();
  });

  it("無 blockquote 的一般文件輸出逐位元不變", () => {
    const md = "# 標題\n\n段落一\n換行\n\n| a | b |\n| - | - |\n| 1 | 2 |\n\n- [ ] 任務";
    const { container: ours } = renderMd(md);
    const { container: reference } = render(
      <ReactMarkdown remarkPlugins={[remarkGfm, remarkBreaks]} skipHtml>
        {md}
      </ReactMarkdown>,
    );
    expect(ours.querySelector(".markdown")!.innerHTML).toBe(reference.innerHTML);
  });
});
