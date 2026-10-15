// 更新日誌對話框（desktop-app spec「更新日誌彈窗」「更新日誌依介面語言呈現」）：whatsNew 模式標題
// 「X.Y.Z 更新內容」、主鈕「知道了」；browse 模式標題「更新日誌」、主鈕「關閉」、列出全部條目；
// 空清單顯示提示。條目、分組標題與版號小標依介面語言呈現，切換語言後已開啟的對話框跟著換。
import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, within } from "@testing-library/react";
import { I18nProvider, type UiLocale } from "@speclink/ui";

import { ReleaseNotesDialog } from "../components/ReleaseNotesDialog";
import { APP_MESSAGES } from "../i18n/messages";
import { RELEASE_NOTES, type ReleaseNotesEntry } from "../release-notes/release-notes";

const ENTRIES: ReleaseNotesEntry[] = [
  {
    version: "0.3.1",
    date: "2026-09-20",
    sections: [
      { kind: "features", items: [{ en: "Alpha feature", "zh-TW": "甲功能" }] },
      { kind: "fixes", items: [{ en: "Beta fix", "zh-TW": "乙修正" }] },
    ],
  },
  {
    version: "0.3.0",
    date: "2026-09-10",
    sections: [{ kind: "improvements", items: [{ en: "Gamma improvement", "zh-TW": "丙改善" }] }],
  },
];

type DialogProps = { mode: "whatsNew" | "browse"; entries: ReleaseNotesEntry[]; focusVersion?: string };

function dialogIn(locale: UiLocale, props: DialogProps, onOpenChange: (open: boolean) => void) {
  return (
    <I18nProvider locale={locale} messages={APP_MESSAGES}>
      <ReleaseNotesDialog open {...props} onOpenChange={onOpenChange} />
    </I18nProvider>
  );
}

function renderDialog(props: DialogProps, locale: UiLocale = "zh-TW") {
  const onOpenChange = vi.fn();
  const { rerender } = render(dialogIn(locale, props, onOpenChange));
  return {
    onOpenChange,
    dialog: screen.getByTestId("release-notes-dialog"),
    switchTo: (next: UiLocale) => rerender(dialogIn(next, props, onOpenChange)),
  };
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
    expect(text).not.toContain("Alpha feature");
    fireEvent.click(within(dialog).getByRole("button", { name: "關閉" }));
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it("英文介面 whatsNew：標題「What's new in 0.3.1」、英文條目與英文分組標題、主鈕「Got it」", () => {
    const { dialog } = renderDialog({ mode: "whatsNew", entries: ENTRIES.slice(0, 1) }, "en");
    expect(within(dialog).getByText("What's new in 0.3.1")).toBeTruthy();
    for (const heading of ["New features", "Fixes"]) {
      expect(within(dialog).getByText(heading)).toBeTruthy();
    }
    for (const item of ["Alpha feature", "Beta fix"]) {
      expect(within(dialog).getByText(item)).toBeTruthy();
    }
    expect(dialog.textContent).not.toMatch(/甲功能|乙修正|新功能|修正/);
    expect(within(dialog).getByRole("button", { name: "Got it" })).toBeTruthy();
  });

  it("英文介面 browse：版號小標「0.3.1 (2026-09-20)」、分組 New features／Fixes／Improvements、全部英文條目", () => {
    const { dialog } = renderDialog({ mode: "browse", entries: ENTRIES }, "en");
    const text = dialog.textContent ?? "";
    expect(text.indexOf("0.3.1 (2026-09-20)")).toBeGreaterThanOrEqual(0);
    expect(text.indexOf("0.3.0 (2026-09-10)")).toBeGreaterThan(text.indexOf("0.3.1 (2026-09-20)"));
    expect(text).not.toContain("（");
    for (const heading of ["New features", "Fixes", "Improvements"]) {
      expect(within(dialog).getByText(heading)).toBeTruthy();
    }
    for (const item of ["Alpha feature", "Beta fix", "Gamma improvement"]) {
      expect(within(dialog).getByText(item)).toBeTruthy();
    }
    expect(text).not.toMatch(/甲功能|乙修正|丙改善/);
  });

  it("切換介面語言後，已開啟的對話框改以新語言呈現", () => {
    const { dialog, switchTo } = renderDialog({ mode: "browse", entries: ENTRIES });
    expect(within(dialog).getByText("甲功能")).toBeTruthy();
    switchTo("en");
    const reopened = screen.getByTestId("release-notes-dialog");
    expect(within(reopened).getByText("Alpha feature")).toBeTruthy();
    expect(within(reopened).getByText("Improvements")).toBeTruthy();
    expect(reopened.textContent).not.toContain("甲功能");
    switchTo("zh-TW");
    expect(within(screen.getByTestId("release-notes-dialog")).getByText("甲功能")).toBeTruthy();
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

// release-notes spec「更新日誌 JSON 為唯一真相」Scenario「條目形狀」：打包進 app 的真 JSON。
describe("release-notes.json 形狀", () => {
  const KINDS = ["features", "fixes", "improvements"];

  it("每筆 version 為 X.Y.Z、date 為 YYYY-MM-DD；kind 為三值之一且依序不重複；每個條目的 en 與 zh-TW 都是非空字串", () => {
    expect(RELEASE_NOTES.length).toBeGreaterThan(0);
    for (const entry of RELEASE_NOTES) {
      expect(entry.version).toMatch(/^\d+\.\d+\.\d+$/);
      expect(entry.date).toMatch(/^\d{4}-\d{2}-\d{2}$/);
      const order = entry.sections.map((section) => KINDS.indexOf(section.kind));
      expect(order.every((index) => index >= 0), `${entry.version} 的 kind`).toBe(true);
      expect([...order].sort((a, b) => a - b), `${entry.version} 的分組順序`).toEqual([...new Set(order)]);
      for (const section of entry.sections) {
        expect(section.items.length, `${entry.version} ${section.kind}`).toBeGreaterThan(0);
        for (const item of section.items) {
          expect(typeof item.en === "string" && item.en.trim().length > 0, `${entry.version} 的 en`).toBe(true);
          expect(typeof item["zh-TW"] === "string" && item["zh-TW"].trim().length > 0, `${entry.version} 的 zh-TW`).toBe(
            true,
          );
        }
      }
    }
  });
});
