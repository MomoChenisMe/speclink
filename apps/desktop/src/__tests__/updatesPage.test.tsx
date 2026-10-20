// 更新頁（desktop-settings-mode design D6；spec「桌面自動更新」「更新日誌彈窗」「更新日誌依介面語言呈現」）。
import { describe, expect, it, vi } from "vitest";
import { fireEvent, render as rtlRender, screen, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider, type UiLocale } from "@speclink/ui";

import { UpdatesPage } from "../views/settings/UpdatesPage";
import type { AppSettingsUpdaterProps } from "../views/AppSettingsView";
import type { UpdaterState } from "../core/updater";
import { APP_MESSAGES } from "../i18n/messages";
import { RELEASE_NOTES } from "../release-notes/release-notes";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);
const render = (ui: ReactElement) => rtlRender(ui, { wrapper: zhWrapper });
const inLocale = (locale: UiLocale, ui: ReactElement) => (
  <I18nProvider locale={locale} messages={APP_MESSAGES}>
    {ui}
  </I18nProvider>
);
const KIND_LABELS = {
  "zh-TW": { features: "新功能", fixes: "修正", improvements: "改善" },
  en: { features: "New features", fixes: "Fixes", improvements: "Improvements" },
} as const;

function updaterProps(state: UpdaterState, over: Partial<AppSettingsUpdaterProps> = {}): AppSettingsUpdaterProps {
  return {
    state,
    currentVersion: "0.1.0",
    onCheck: vi.fn(),
    onShowReleaseNotes: vi.fn(),
    onAccept: vi.fn(),
    onCancel: vi.fn(),
    onRelaunch: vi.fn(),
    autoCheck: true,
    onAutoCheckChange: vi.fn(),
    ...over,
  };
}

const row = (label: string) => screen.getByRole("group", { name: label });
const installCard = () => screen.queryByRole("region", { name: "下載與安裝" });

describe("UpdatesPage", () => {
  it.each(["zh-TW", "en"] as const)("%s：沒有 updater 仍有目前版本與內附日誌，只有網路動作停用", (locale) => {
    rtlRender(inLocale(locale, <UpdatesPage currentVersion="1.50.1" />));
    const m = APP_MESSAGES[locale];
    expect(screen.getByText("1.50.1")).toBeTruthy();
    expect(screen.getByRole("region", { name: m["releaseNotes.open"] }).textContent).toContain(RELEASE_NOTES[0].sections[0].items[0][locale]);
    expect((screen.getByRole("button", { name: m["updater.check"] }) as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByText(m["updater.page.unavailable"])).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: m["updater.page.allNotes"] }));
    expect(screen.getByTestId("release-notes-dialog")).toBeTruthy();
  });

  it("背景失敗保留最後結果、重試與診斷入口，不冒稱已最新", () => {
    const onDiagnostics = vi.fn();
    const props = updaterProps({ phase: "idle" }, {
      lastCheck: { checkedAt: 1, status: "failed", errorCode: "proxy-auth", operationId: "op-one" }, onDiagnostics,
    });
    render(<UpdatesPage updater={props} />);
    expect(screen.getByRole("status", { name: "最後檢查" }).textContent).toContain("proxy-auth");
    expect(screen.queryByText("已是最新")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "重試" }));
    fireEvent.click(screen.getByRole("button", { name: "網路與診斷" }));
    expect(props.onCheck).toHaveBeenCalledOnce();
    expect(onDiagnostics).toHaveBeenCalledOnce();
  });
  it("頁標題「更新」右側「檢查更新」回呼 onCheck、檢查中停用；版本卡有目前版本列（等寬）", () => {
    const onCheck = vi.fn();
    const { rerender } = render(<UpdatesPage updater={updaterProps({ phase: "idle" }, { onCheck })} />);
    expect(screen.getByRole("heading", { level: 2, name: "更新" })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "檢查更新" }));
    expect(onCheck).toHaveBeenCalledTimes(1);
    expect(within(row("目前版本")).getByText("0.1.0").className).toContain("font-mono");
    rerender(<UpdatesPage updater={updaterProps({ phase: "checking", manual: true })} />);
    expect((screen.getByRole("button", { name: "檢查更新" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it.each<[string, UpdaterState, string, string]>([
    ["閒置", { phase: "idle" }, "—", "尚未檢查"],
    ["檢查中", { phase: "checking", manual: true }, "", "檢查中"],
    ["已是最新", { phase: "upToDate" }, "0.1.0", "已是最新"],
    ["檢查失敗", { phase: "checkFailed" }, "", "無法檢查"],
    ["待同意", { phase: "available", version: "0.2.0" }, "0.2.0", "有新版本"],
    ["下載中", { phase: "downloading", version: "0.2.0" }, "0.2.0", "有新版本"],
    ["待重啟", { phase: "restartPending", version: "0.2.0" }, "0.2.0", "有新版本"],
  ])("最新版本列：%s", (_name, state, version, pill) => {
    render(<UpdatesPage updater={updaterProps(state)} />);
    const latest = row("最新版本");
    expect(within(latest).getByText(pill)).toBeTruthy();
    if (version) expect(within(latest).getByText(version).className).toContain("font-mono");
  });

  it("待同意時出現「下載與安裝」卡：標題 Speclink 目標版本與「下載」鈕；閒置時缺席", () => {
    const updater = updaterProps({ phase: "available", version: "0.2.0" });
    const { rerender } = render(<UpdatesPage updater={updater} />);
    const card = installCard()!;
    expect(within(card).getByRole("group", { name: "Speclink 0.2.0" })).toBeTruthy();
    expect(card.textContent).toContain("安裝後會重新啟動");
    fireEvent.click(within(card).getByRole("button", { name: "下載" }));
    expect(updater.onAccept).toHaveBeenCalledTimes(1);
    rerender(<UpdatesPage updater={updaterProps({ phase: "idle" })} />);
    expect(installCard()).toBeNull();
  });

  it("下載中、待重啟與失敗：卡內承載對應控制", () => {
    for (const [state, label] of [
      [{ phase: "downloading", version: "0.2.0" }, "取消"],
      [{ phase: "restartPending", version: "0.2.0" }, "安裝並重新啟動"],
      [{ phase: "error", message: "invalid signature" }, "重試"],
    ] as const) {
      const { unmount } = render(<UpdatesPage updater={updaterProps(state)} />);
      expect(within(installCard()!).getByRole("button", { name: label })).toBeTruthy();
      unmount();
    }
  });

  it("更新日誌卡：最新版本為第一列並展開分組列點；較舊版本每列「檢視」與底列「全部更新日誌」開瀏覽模式", () => {
    const onShowReleaseNotes = vi.fn();
    render(<UpdatesPage updater={updaterProps({ phase: "idle" }, { onShowReleaseNotes })} />);
    const card = screen.getByRole("region", { name: "更新日誌" });
    const [latest, ...older] = RELEASE_NOTES;
    const first = within(card).getAllByRole("group")[0];
    expect(first.textContent).toContain(latest.version);
    expect(first.textContent).toContain(latest.date);
    for (const section of latest.sections) {
      expect(within(first).getByText(KIND_LABELS["zh-TW"][section.kind])).toBeTruthy();
      expect(first.textContent).toContain(section.items[0]["zh-TW"]);
    }
    // 舊版本列的摘要是該版首條的繁中文字。
    expect(within(card).getAllByRole("group")[1].textContent).toContain(older[0].sections[0].items[0]["zh-TW"]);
    const views = within(card).getAllByRole("button", { name: "檢視" });
    expect(views).toHaveLength(older.length);
    // 舊版本列的「檢視」帶該列版號（開啟後捲到該版）；「全部更新日誌」不帶版號（自頂端呈現）。
    fireEvent.click(views[0]);
    fireEvent.click(within(card).getByRole("button", { name: "全部更新日誌" }));
    expect(onShowReleaseNotes.mock.calls).toEqual([[older[0].version], []]);
  });

  it("英文介面的更新日誌卡：最新版本展開英文分組標題與英文條目，舊版本列摘要為英文首條", () => {
    rtlRender(inLocale("en", <UpdatesPage updater={updaterProps({ phase: "idle" })} />));
    const card = screen.getByRole("region", { name: "Changelog" });
    const [latest, ...older] = RELEASE_NOTES;
    const [first, second] = within(card).getAllByRole("group");
    for (const section of latest.sections) {
      expect(within(first).getByText(KIND_LABELS.en[section.kind])).toBeTruthy();
      for (const item of section.items) {
        expect(first.textContent).toContain(item.en);
        expect(first.textContent).not.toContain(item["zh-TW"]);
      }
    }
    expect(second.textContent).toContain(older[0].sections[0].items[0].en);
    expect(second.textContent).not.toContain(older[0].sections[0].items[0]["zh-TW"]);
  });

  it("切換介面語言後，更新日誌卡改以新語言呈現", () => {
    const page = <UpdatesPage updater={updaterProps({ phase: "idle" })} />;
    const { rerender } = rtlRender(inLocale("zh-TW", page));
    const top = RELEASE_NOTES[0].sections[0];
    expect(screen.getByRole("region", { name: "更新日誌" }).textContent).toContain(top.items[0]["zh-TW"]);
    rerender(inLocale("en", page));
    const card = screen.getByRole("region", { name: "Changelog" });
    expect(card.textContent).toContain(top.items[0].en);
    expect(within(card).getAllByText(KIND_LABELS.en[top.kind]).length).toBeGreaterThan(0);
    expect(card.textContent).not.toContain(top.items[0]["zh-TW"]);
  });

  it("自動檢查卡：開關反映偏好、切換回呼 onAutoCheckChange", () => {
    const onAutoCheckChange = vi.fn();
    const { rerender } = render(<UpdatesPage updater={updaterProps({ phase: "idle" }, { onAutoCheckChange })} />);
    const sw = screen.getByRole("switch", { name: "啟動時在背景檢查新版本" });
    expect(sw.getAttribute("aria-checked")).toBe("true");
    expect(row("啟動時在背景檢查新版本").textContent).toContain("只檢查，不會自動下載");
    fireEvent.click(sw);
    expect(onAutoCheckChange).toHaveBeenCalledWith(false);
    rerender(<UpdatesPage updater={updaterProps({ phase: "idle" }, { autoCheck: false, onAutoCheckChange })} />);
    expect(screen.getByRole("switch", { name: "啟動時在背景檢查新版本" }).getAttribute("aria-checked")).toBe("false");
  });

  it("focus 為 true 時捲到「下載與安裝」卡一次並回呼；無該卡時捲到版本卡", () => {
    const scrollIntoView = vi.fn();
    Element.prototype.scrollIntoView = scrollIntoView;
    const onFocusHandled = vi.fn();
    const available = { phase: "available", version: "0.2.0" } as const;
    const { rerender, unmount } = render(<UpdatesPage updater={updaterProps(available, { focus: false, onFocusHandled })} />);
    expect(scrollIntoView).not.toHaveBeenCalled();
    rerender(<UpdatesPage updater={updaterProps(available, { focus: true, onFocusHandled })} />);
    expect(scrollIntoView).toHaveBeenCalledTimes(1);
    expect((scrollIntoView.mock.instances[0] as unknown as HTMLElement).contains(installCard())).toBe(true);
    expect(onFocusHandled).toHaveBeenCalledTimes(1);
    unmount();

    scrollIntoView.mockClear();
    render(<UpdatesPage updater={updaterProps({ phase: "idle" }, { focus: true, onFocusHandled })} />);
    expect(scrollIntoView).toHaveBeenCalledTimes(1);
    const target = scrollIntoView.mock.instances[0] as unknown as HTMLElement;
    expect(target.contains(screen.getByRole("region", { name: "版本" }))).toBe(true);
  });
});
