// design D7 自製輕量 i18n：I18nProvider（locale＋可選 app 層 messages 合併）與
// useI18n() 的 t(key)。缺 key 回傳 key 本身（開發期可見的失敗，而非靜默錯語言）。
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";

import { I18nProvider, useI18n, MESSAGES, dictionaryProblems, translatorFor } from "../i18n";

function Probe({ id }: { id: string }) {
  const { t } = useI18n();
  return <span data-testid="probe">{t(id)}</span>;
}

function CountProbe({ id, n }: { id: string; n: number }) {
  const { tn } = useI18n();
  return <span data-testid="probe">{tn(id, n)}</span>;
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

  // fix-english-status-labels-and-plurals D4：兩語言鍵集合（.one 除外）、.one 只在英文且有基礎鍵、
  // 計數文案（數字佔位符之後接複數名詞）一律用 {n} 並有 .one——三份字典共用同一個檢查。
  it("passes the shared dictionary check", () => {
    expect(dictionaryProblems(MESSAGES)).toEqual([]);
    expect(Object.keys(MESSAGES.en).length).toBeGreaterThan(0);
  });

  it("dictionaryProblems names each kind of problem", () => {
    const problems = dictionaryProblems({
      "zh-TW": {
        "a.count": "{n} 個",
        "b.only": "只有中文",
        "c.rules": "{count} 條規則",
        "d.x.one": "x",
        "f.ok": "第 {n} 輪",
        "g.items": "{count} 個項目",
      },
      en: {
        "a.count": "{n} widgets",
        "c.rules": "{count} rules",
        "e.orphan.one": "{n} thing",
        "f.ok": "Round {n}",
        "g.items": "{n} items",
        "g.items.one": "{n} item",
        "h.new": "New",
      },
    });
    expect(problems).toEqual([
      "key only in zh-TW: b.only",
      ".one outside en: zh-TW d.x.one",
      "key only in en: h.new",
      ".one without a base key: e.orphan.one",
      "count message without .one: a.count",
      "count message must use {n}: c.rules",
      "zh-TW count message must use {n} too: g.items",
    ]);
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

// fix-english-status-labels-and-plurals D2：tn(key, n) 依數量選單複數。只有英文字典寫 `.one`。
describe("tn(key, n)", () => {
  const messages = {
    "zh-TW": { "app.count": "{n} 個元件", "app.plain": "{n} 個東西" },
    en: { "app.count": "{n} widgets", "app.count.one": "{n} widget", "app.plain": "{n} things" },
  };

  function countText(locale: "zh-TW" | "en", id: string, n: number): string {
    const { unmount } = render(
      <I18nProvider locale={locale} messages={messages}>
        <CountProbe id={id} n={n} />
      </I18nProvider>,
    );
    const text = probeText();
    unmount();
    return text;
  }

  it("uses the .one form in English when n is 1", () => {
    expect(countText("en", "app.count", 1)).toBe("1 widget");
  });

  it("uses the plural form in English when n is 0 or 2", () => {
    expect(countText("en", "app.count", 0)).toBe("0 widgets");
    expect(countText("en", "app.count", 2)).toBe("2 widgets");
  });

  it("falls back to the base key when no .one form exists", () => {
    expect(countText("en", "app.plain", 1)).toBe("1 things");
  });

  it("always uses the base key in zh-TW", () => {
    expect(countText("zh-TW", "app.count", 1)).toBe("1 個元件");
    expect(countText("zh-TW", "app.count", 3)).toBe("3 個元件");
  });

  it("returns the key itself when the key is missing", () => {
    expect(countText("en", "no.such.key", 1)).toBe("no.such.key");
  });

  it("works outside any provider with the zh-TW dictionary", () => {
    render(<CountProbe id="doc.scenarioCount" n={1} />);
    expect(probeText()).toBe("1 個 scenario");
  });
});

// 真實字典的計數文案（desktop-config「英文計數文案依數量使用單數或複數」的情境值）。
describe("英文計數文案（真實鍵）", () => {
  const en = translatorFor("en");

  it.each([
    ["common.tasksCount", 0, "0 tasks"],
    ["common.rounds", 1, "1 round"],
    ["doc.scenarioCount", 1, "1 scenario"],
    ["analyze.issues", 1, "1 issue"],
    ["analyze.issues", 2, "2 issues"],
    ["analyze.validateFail", 1, "Structural validation: 1 error"],
    ["revert.blockedChecked", 1, "1 checked task"],
    ["ticket.rules.projectCount", 1, "1 project rule"],
    ["doc.capabilityRemovedScenarios", 1, ", 1 scenario removed with REMOVED-SCENARIO"],
  ] as const)("%s n=%i → %s", (key, n, expected) => {
    expect(en.tn(key, n)).toBe(expected);
  });

  it("繁中與英文不帶數字的同一句都用 {n}", () => {
    expect(translatorFor("zh-TW").tn("ticket.rules.replaceWithRules", 12)).toBe("只用專案規則（沒有套用預設 12 條）");
  });
});
