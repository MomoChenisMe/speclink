import { readFileSync } from "node:fs";
import { describe, it, expect } from "vitest";
import { act, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { BaselineSmellList } from "../components/BaselineSmellList";
import { I18nProvider, MESSAGES } from "../i18n";

const source = readFileSync("../../crates/engine/speclink-core/src/workspace/skills.rs", "utf8");
const names = [...source.match(/REVIEW_BASELINE_SMELLS:.*?= \[([\s\S]*?)\];/)![1].matchAll(/"([^"]+)"/g)].map(match => match[1]);

describe("預設十二條的問號說明", () => {
  it.each(["zh-TW", "en"] as const)("%s 逐名涵蓋引擎的十二條用途與例子", async (locale) => {
    expect(names).toHaveLength(12);
    render(<I18nProvider locale={locale}><BaselineSmellList names={names} /></I18nProvider>);
    const user = userEvent.setup();
    for (const name of names) {
      const button = screen.getByRole("button", { name: MESSAGES[locale]["reviewSmell.help"].replace("{name}", name) });
      await user.click(button);
      const dialog = screen.getByRole("dialog");
      expect(within(dialog).getByText(MESSAGES[locale][`reviewSmell.${name}.purpose`])).toBeTruthy();
      expect(within(dialog).getByText(MESSAGES[locale][`reviewSmell.${name}.example`])).toBeTruthy();
      await user.keyboard("{Escape}");
      expect(document.activeElement).toBe(button);
    }
    expect(screen.getByText(MESSAGES[locale]["reviewSmell.baselineHelp"])).toBeTruthy();
  });

  it("Enter／Space 開啟，Escape 返回問號；未知名稱保留且顯示版本說明", async () => {
    render(<I18nProvider locale="zh-TW"><BaselineSmellList names={["Future Smell"]} /></I18nProvider>);
    const user = userEvent.setup();
    const button = screen.getByRole("button", { name: "Future Smell 說明" });
    act(() => button.focus());
    for (const key of ["{Enter}", " "]) {
      await user.keyboard(key);
      expect(screen.getByRole("dialog").textContent).toContain("此版本未提供說明");
      await user.keyboard("{Escape}");
      expect(document.activeElement).toBe(button);
    }
    expect(screen.getByText("Future Smell")).toBeTruthy();
  });

  it("replace 模式清楚標示預設未套用", () => {
    render(<I18nProvider locale="zh-TW"><BaselineSmellList names={names} baseline="replace" /></I18nProvider>);
    expect(screen.getByText("目前為 replace：這份預設清單未套用。")).toBeTruthy();
  });
});
