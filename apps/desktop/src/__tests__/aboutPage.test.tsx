// 關於頁（desktop-about-page design D2；spec「關於頁」）。
import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render as rtlRender, screen, waitFor, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";
import { openUrl } from "@tauri-apps/plugin-opener";

import { AboutPage, type AboutPageProps } from "../views/settings/AboutPage";
import type { UpdaterState } from "../core/updater";
import { APP_MESSAGES } from "../i18n/messages";

const { toastError } = vi.hoisted(() => ({ toastError: vi.fn() }));
vi.mock("sonner", () => ({ toast: Object.assign(vi.fn(), { error: toastError }) }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn().mockResolvedValue(undefined) }));

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);
const render = (ui: ReactElement) => rtlRender(ui, { wrapper: zhWrapper });

const INFO = {
  appVersion: "0.8.0",
  engineVersion: "v1.44.0",
  platform: "macOS",
  arch: "arm64",
};
const THREE_LINES = "Speclink 0.8.0\nEngine v1.44.0\nPlatform macOS arm64";

function aboutProps(over: Partial<AboutPageProps> = {}): AboutPageProps {
  return {
    appInfo: INFO,
    onLoadAppInfo: vi.fn(),
    updaterState: { phase: "idle" },
    onOpenUpdates: vi.fn(),
    thirdPartyLicenses: null,
    ...over,
  };
}

const row = (label: string) => screen.getByRole("group", { name: label });
const brandCard = () => screen.getByTestId("about-brand");

beforeEach(() => {
  vi.mocked(openUrl).mockClear().mockResolvedValue(undefined);
  toastError.mockClear();
});

describe("AboutPage", () => {
  it("頁標題與說明；進頁即要求載入版本資訊", () => {
    const onLoadAppInfo = vi.fn();
    render(<AboutPage {...aboutProps({ onLoadAppInfo })} />);
    expect(screen.getByRole("heading", { level: 2, name: "關於 Speclink" })).toBeTruthy();
    expect(screen.getByText("版本、授權與連結。")).toBeTruthy();
    expect(onLoadAppInfo).toHaveBeenCalledTimes(1);
  });

  it("品牌卡：鎖版、版本列「0.8.0 · 引擎 v1.44.0 · macOS arm64」與狀態籤、產品描述，無裸標記", () => {
    render(<AboutPage {...aboutProps({ updaterState: { phase: "upToDate" } })} />);
    const card = brandCard();
    expect(within(card).getAllByRole("img", { name: "Speclink" })[0].className).toContain("h-10");
    expect(within(card).getByText("0.8.0 · 引擎 v1.44.0 · macOS arm64")).toBeTruthy();
    expect(within(card).getByText("已是最新")).toBeTruthy();
    expect(within(card).getByText("一套 SDD Engine，支援 Local Repo 與 Remote Store")).toBeTruthy();
    expect(card.querySelector("[role=img] svg")).toBeNull();
  });

  it.each<[string, UpdaterState, string]>([
    ["未檢查", { phase: "idle" }, "尚未檢查"],
    ["檢查中", { phase: "checking", manual: true }, "檢查中"],
    ["有新版本", { phase: "available", version: "0.9.0" }, "有新版本"],
    ["無法檢查", { phase: "checkFailed" }, "無法檢查"],
  ])("版本列狀態籤：%s", (_name, state, pill) => {
    render(<AboutPage {...aboutProps({ updaterState: state })} />);
    expect(within(brandCard()).getByText(pill)).toBeTruthy();
  });

  it("狀態籤是按鈕：點擊進更新頁（呼叫 onOpenUpdates 一次）；未注入更新狀態時不顯示", () => {
    const onOpenUpdates = vi.fn();
    const { rerender } = render(<AboutPage {...aboutProps({ updaterState: { phase: "upToDate" }, onOpenUpdates })} />);
    fireEvent.click(within(brandCard()).getByRole("button", { name: "更新狀態：已是最新" }));
    expect(onOpenUpdates).toHaveBeenCalledTimes(1);
    rerender(<AboutPage {...aboutProps({ updaterState: undefined })} />);
    expect(within(brandCard()).queryByRole("button", { name: /更新狀態/ })).toBeNull();
  });

  it("頁內沒有版本卡、檢查更新列與更新日誌列", () => {
    render(<AboutPage {...aboutProps()} />);
    expect(screen.queryByRole("region", { name: "版本" })).toBeNull();
    expect(screen.queryByRole("group", { name: "檢查更新" })).toBeNull();
    expect(screen.queryByRole("group", { name: "更新日誌" })).toBeNull();
    expect(screen.queryByRole("button", { name: "檢查更新" })).toBeNull();
  });

  it("版本資訊尚未取得時版本列為「—」、複製鈕缺席", () => {
    render(<AboutPage {...aboutProps({ appInfo: null })} />);
    expect(within(brandCard()).getByText("—")).toBeTruthy();
    expect(within(brandCard()).queryByRole("button", { name: "複製版本資訊" })).toBeNull();
  });

  it("Scenario「複製版本資訊」：品牌卡版本列旁的鈕寫入三行到剪貼簿並短暫顯示已複製", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
    render(<AboutPage {...aboutProps()} />);
    const copy = within(brandCard()).getByRole("button", { name: "複製版本資訊" });
    expect(copy.textContent).toBe("複製");
    fireEvent.click(copy);
    expect(writeText).toHaveBeenCalledWith(THREE_LINES);
    await waitFor(() => expect(copy.textContent).toBe("已複製"));
  });

  it("連結卡三列整列可點：原始碼、說明文件、回報問題（網址含版本三行）", () => {
    render(<AboutPage {...aboutProps()} />);
    const links = screen.getByRole("region", { name: "連結" });
    const source = within(links).getByRole("button", { name: /原始碼/ });
    expect(within(source).getByText("github.com/mosutw/speclink").className).toContain("font-mono");
    fireEvent.click(source);
    fireEvent.click(within(links).getByRole("button", { name: /說明文件/ }));
    fireEvent.click(within(links).getByRole("button", { name: /回報問題/ }));
    expect(vi.mocked(openUrl).mock.calls).toEqual([
      ["https://github.com/mosutw/speclink"],
      ["https://github.com/mosutw/speclink/tree/main/docs"],
      [`https://github.com/mosutw/speclink/issues/new?body=${encodeURIComponent(THREE_LINES)}`],
    ]);
    fireEvent.click(within(links).getByRole("button", { name: /原作者專案/ }));
    expect(openUrl).toHaveBeenLastCalledWith("https://github.com/MomoChenisMe/speclink");
  });

  it("Scenario「連結外開」：開啟失敗以失敗 toast 呈現錯誤", async () => {
    vi.mocked(openUrl).mockRejectedValueOnce("not allowed by scope");
    render(<AboutPage {...aboutProps()} />);
    fireEvent.click(screen.getByRole("button", { name: /回報問題/ }));
    await waitFor(() => expect(toastError).toHaveBeenCalledWith("not allowed by scope"));
  });

  it("Scenario「授權對話框」：授權條款顯示 LICENSE 全文；關閉後第三方授權分 npm 與 cargo 兩節", async () => {
    const thirdPartyLicenses = {
      npm: [
        { name: "react", version: "19.2.7", license: "MIT" },
        { name: "zustand", version: "5.0.14", license: null },
      ],
      cargo: [
        { name: "anyhow", version: "1.0.80", license: "MIT OR Apache-2.0" },
        { name: "serde", version: "1.0.200", license: "MIT OR Apache-2.0" },
      ],
    };
    render(<AboutPage {...aboutProps({ thirdPartyLicenses })} />);
    expect(within(row("授權條款")).getByText("MIT License · © 2026 MomoChen")).toBeTruthy();
    fireEvent.click(within(row("授權條款")).getByRole("button", { name: "檢視" }));
    const license = await screen.findByRole("alertdialog", { name: "授權條款" });
    expect(license.textContent).toContain("Permission is hereby granted, free of charge");
    fireEvent.click(within(license).getByRole("button", { name: "關閉" }));
    await waitFor(() => expect(screen.queryByRole("alertdialog")).toBeNull());

    fireEvent.click(within(row("第三方授權")).getByRole("button", { name: "檢視" }));
    const dialog = await screen.findByRole("alertdialog", { name: "第三方授權" });
    const sections = within(dialog).getAllByRole("region");
    expect(sections.map((section) => section.getAttribute("aria-label"))).toEqual(["npm", "cargo"]);
    expect(within(sections[0]).getAllByRole("listitem").map((item) => item.textContent)).toEqual([
      "react 19.2.7 · MIT",
      "zustand 5.0.14 · 未標示",
    ]);
    expect(within(sections[1]).getAllByRole("listitem").map((item) => item.textContent)).toEqual([
      "anyhow 1.0.80 · MIT OR Apache-2.0",
      "serde 1.0.200 · MIT OR Apache-2.0",
    ]);
  });

  it("第三方清單檔缺席時對話框顯示「清單尚未產生」而非錯誤；cargo 節為空時該節同樣標示", async () => {
    const { unmount } = render(<AboutPage {...aboutProps({ thirdPartyLicenses: null })} />);
    fireEvent.click(within(row("第三方授權")).getByRole("button", { name: "檢視" }));
    const dialog = await screen.findByRole("alertdialog", { name: "第三方授權" });
    expect(within(dialog).getByText("清單尚未產生")).toBeTruthy();
    expect(within(dialog).queryAllByRole("listitem")).toHaveLength(0);
    unmount();

    render(
      <AboutPage
        {...aboutProps({ thirdPartyLicenses: { npm: [{ name: "react", version: "19.2.7", license: "MIT" }], cargo: [] } })}
      />,
    );
    fireEvent.click(within(row("第三方授權")).getByRole("button", { name: "檢視" }));
    const partial = await screen.findByRole("alertdialog", { name: "第三方授權" });
    const cargo = within(partial).getByRole("region", { name: "cargo" });
    expect(within(cargo).getByText("清單尚未產生")).toBeTruthy();
  });
});
