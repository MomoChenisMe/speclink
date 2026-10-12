// 更新日誌對話框（desktop-app spec「更新日誌彈窗」）：whatsNew 模式標題「X.Y.Z 更新內容」、
// 主鈕「知道了」；browse 模式標題「更新日誌」、主鈕「關閉」、列出全部條目；空清單顯示提示。
import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, within } from "@testing-library/react";
import type { ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { ReleaseNotesDialog } from "../components/ReleaseNotesDialog";
import { APP_MESSAGES } from "../i18n/messages";
import type { ReleaseNotesEntry } from "../release-notes/release-notes";

const ENTRIES: ReleaseNotesEntry[] = [
  {
    version: "0.3.1",
    date: "2026-09-20",
    sections: [
      { title: "新功能", items: ["甲功能"] },
      { title: "修正", items: ["乙修正"] },
    ],
  },
  {
    version: "0.3.0",
    date: "2026-09-10",
    sections: [{ title: "改善", items: ["丙改善"] }],
  },
];

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);

function renderDialog(props: { mode: "whatsNew" | "browse"; entries: ReleaseNotesEntry[]; focusVersion?: string }) {
  const onOpenChange = vi.fn();
  render(<ReleaseNotesDialog open {...props} onOpenChange={onOpenChange} />, {
    wrapper: zhWrapper,
  });
  return { onOpenChange, dialog: screen.getByTestId("release-notes-dialog") };
}

describe("ReleaseNotesDialog", () => {
  it("whatsNew 模式：標題「0.3.1 更新內容」，主鈕「知道了」按下呼叫 onOpenChange(false)", () => {
    const { onOpenChange, dialog } = renderDialog({ mode: "whatsNew", entries: ENTRIES });
    expect(within(dialog).getByText("0.3.1 更新內容")).toBeTruthy();
    expect(within(dialog).queryByRole("button", { name: "關閉" })).toBeNull();
    fireEvent.click(within(dialog).getByRole("button", { name: "知道了" }));
    expect(onOpenChange).toHaveBeenCalledTimes(1);
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it("browse 模式：標題「更新日誌」、主鈕「關閉」、列出全部條目的「版號（日期）」小標與三類分組", () => {
    const { onOpenChange, dialog } = renderDialog({ mode: "browse", entries: ENTRIES });
    expect(within(dialog).getByText("更新日誌")).toBeTruthy();
    expect(within(dialog).queryByRole("button", { name: "知道了" })).toBeNull();
    const text = dialog.textContent ?? "";
    expect(text.indexOf("0.3.1（2026-09-20）")).toBeGreaterThanOrEqual(0);
    expect(text.indexOf("0.3.0（2026-09-10）")).toBeGreaterThan(text.indexOf("0.3.1（2026-09-20）"));
    for (const heading of ["新功能", "修正", "改善"]) {
      expect(within(dialog).getByText(heading)).toBeTruthy();
    }
    for (const item of ["甲功能", "乙修正", "丙改善"]) {
      expect(within(dialog).getByText(item)).toBeTruthy();
    }
    fireEvent.click(within(dialog).getByRole("button", { name: "關閉" }));
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it("browse 帶 focusVersion：開啟後把該版小標所在的條目捲到頂端一次（desktop-about-page design D8）", () => {
    const scroll = vi.spyOn(Element.prototype, "scrollIntoView");
    try {
      const { dialog } = renderDialog({ mode: "browse", entries: ENTRIES, focusVersion: "0.3.0" });
      expect(scroll).toHaveBeenCalledTimes(1);
      expect(scroll).toHaveBeenCalledWith({ block: "start" });
      const target = scroll.mock.contexts[0] as HTMLElement;
      expect(dialog.contains(target)).toBe(true);
      expect(target.getAttribute("data-version")).toBe("0.3.0");
    } finally {
      scroll.mockRestore();
    }
  });

  it.each<[string, string | undefined]>([
    ["不帶 focusVersion", undefined],
    ["focusVersion 不在清單內", "0.0.1"],
  ])("browse %s 時不捲動", (_name, focusVersion) => {
    const scroll = vi.spyOn(Element.prototype, "scrollIntoView");
    try {
      renderDialog({ mode: "browse", entries: ENTRIES, focusVersion });
      expect(scroll).not.toHaveBeenCalled();
    } finally {
      scroll.mockRestore();
    }
  });

  it("entries 為空時顯示 releaseNotes.empty 文案", () => {
    const { dialog } = renderDialog({ mode: "browse", entries: [] });
    expect(within(dialog).getByText(APP_MESSAGES["zh-TW"]["releaseNotes.empty"])).toBeTruthy();
  });
});
