// design D7 自製輕量 i18n：I18nProvider（locale＋可選 app 層 messages 合併）與
// useI18n() 的 t(key)。缺 key 回傳 key 本身（開發期可見的失敗，而非靜默錯語言）。
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";

import { I18nProvider, useI18n, MESSAGES } from "../i18n";

function Probe({ id }: { id: string }) {
  const { t } = useI18n();
  return <span data-testid="probe">{t(id)}</span>;
}

function probeText(): string {
  return screen.getByTestId("probe").textContent ?? "";
}

describe("I18nProvider / useI18n", () => {
  it("resolves built-in keys per locale", () => {
    const { unmount } = render(
      <I18nProvider locale="zh-TW">
        <Probe id="stage.proposed" />
      </I18nProvider>,
    );
    expect(probeText()).toBe("提案中");
    unmount();
    render(
      <I18nProvider locale="en">
        <Probe id="stage.proposed" />
      </I18nProvider>,
    );
    expect(probeText()).toBe("Proposed");
  });

  it("merges app-layer messages over the built-in dictionary", () => {
    const messages = {
      "zh-TW": { "app.hello": "你好" },
      en: { "app.hello": "Hello" },
    };
    const { unmount } = render(
      <I18nProvider locale="en" messages={messages}>
        <Probe id="app.hello" />
      </I18nProvider>,
    );
    expect(probeText()).toBe("Hello");
    unmount();
    // 合併後內建 key 仍查得到。
    render(
      <I18nProvider locale="en" messages={messages}>
        <Probe id="stage.proposed" />
      </I18nProvider>,
    );
    expect(probeText()).toBe("Proposed");
  });

  it("returns the key itself when the key is missing", () => {
    render(
      <I18nProvider locale="en">
        <Probe id="no.such.key" />
      </I18nProvider>,
    );
    expect(probeText()).toBe("no.such.key");
  });

  it("defaults to zh-TW outside any provider", () => {
    render(<Probe id="stage.proposed" />);
    expect(probeText()).toBe("提案中");
  });

  it("keeps the zh-TW and en dictionaries key-equal", () => {
    const zh = Object.keys(MESSAGES["zh-TW"]).sort();
    const en = Object.keys(MESSAGES.en).sort();
    expect(zh).toEqual(en);
    expect(zh.length).toBeGreaterThan(0);
  });

  it("exposes the active locale", () => {
    function LocaleProbe() {
      const { locale } = useI18n();
      return <span data-testid="probe">{locale}</span>;
    }
    render(
      <I18nProvider locale="en">
        <LocaleProbe />
      </I18nProvider>,
    );
    expect(probeText()).toBe("en");
  });
});
