// 應用程式設定（desktop-settings-mode design D5；desktop-about-page design D7）：依所在分類渲染
// 一頁——一般、CLI 於此檔；更新頁、伺服器頁與關於頁各有自己的測試檔。
import { describe, expect, it, vi } from "vitest";
import { act, fireEvent, render as rtlRender, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ComponentProps, ReactElement, ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { AppSettingsView, type AppSettingsUpdaterProps } from "../views/AppSettingsView";
import type { UpdaterState } from "../core/updater";
import { APP_MESSAGES } from "../i18n/messages";
import type { CliInstallView } from "../store";
vi.mock("../adapter/security",()=>({listSecurityData:vi.fn().mockResolvedValue({standards:[]}),watchSecurityProgress:vi.fn().mockResolvedValue(()=>{})}));

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);

function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

/** 軟體更新面的最小注入：狀態＋回呼。 */
function updaterProps(state: UpdaterState, over: Partial<AppSettingsUpdaterProps> = {}): AppSettingsUpdaterProps {
  return {
    state,
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

function cliView(over: Partial<CliInstallView> = {}): CliInstallView {
  return {
    platform: "macos",
    status: { kind: "not-installed" },
    canDeploy: true,
    pathHint: false,
    deployDir: "/Users/u/.local/bin",
    busy: false,
    error: null,
    ...over,
  };
}

type ViewProps = ComponentProps<typeof AppSettingsView>;
function view(over: Partial<ViewProps> = {}) {
  return (
    <AppSettingsView
      section="general"
      platform="macos"
      localePref={null}
      onLocalePrefChange={vi.fn()}
      themePref={null}
      onThemePrefChange={vi.fn()}
      {...over}
    />
  );
}

const cardTitles = () => screen.getAllByRole("region").map((r) => r.getAttribute("aria-label"));
const row = (label: string) => screen.getByRole("group", { name: label });

it("沒有專案仍可從應用程式設定進入安全標準資料", async () => {
  await act(async()=>render(view({section:"security-data"})));
  expect(screen.getByRole("heading",{name:"安全標準資料"})).toBeTruthy();
});

describe("一般頁", () => {
  it("外觀保存失敗保留選中值並顯示錯誤，重試後清除", async () => {
    const onThemePrefChange = vi.fn().mockImplementationOnce(() => { throw new Error("quota denied"); });
    const user = userEvent.setup();
    render(view({ themePref: "light", onThemePrefChange }));
    await user.click(within(row("外觀")).getByRole("combobox"));
    await user.click(await screen.findByRole("option", { name: "深色" }));
    expect(screen.getByRole("alert").textContent).toContain("quota denied");
    expect(within(row("外觀")).getByRole("combobox").textContent).toContain("淺色");
    await user.click(within(row("外觀")).getByRole("combobox"));
    await user.click(await screen.findByRole("option", { name: "深色" }));
    expect(onThemePrefChange).toHaveBeenCalledTimes(2);
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("頁標題「一般」、「僅存於此裝置」註記；只有語言與外觀、鍵盤快捷鍵兩卡，語言與外觀卡兩列依序為顯示語言與外觀", () => {
    render(view());
    expect(screen.getByRole("heading", { level: 2, name: "一般" })).toBeTruthy();
    expect(screen.getByText("此頁內容僅存於此裝置，不寫入版本庫")).toBeTruthy();
    expect(cardTitles()).toEqual(["語言與外觀", "鍵盤快捷鍵"]);
    const card = screen.getByRole("region", { name: "語言與外觀" });
    expect(within(card).getAllByRole("group").map((g) => g.querySelector("span, label")?.textContent)).toEqual([
      "顯示語言",
      "外觀",
    ]);
  });

  it("注入更新與 CLI 面時一般頁也不重複其他分類頁的列（目前版本、檢查更新、更新日誌、speclink CLI）", () => {
    render(view({ updater: updaterProps({ phase: "upToDate" }), cliInstall: { view: cliView(), onInstall: vi.fn() } }));
    expect(cardTitles()).toEqual(["語言與外觀", "鍵盤快捷鍵"]);
    for (const label of ["目前版本", "檢查更新", "更新日誌", "speclink CLI"]) {
      expect(screen.queryByRole("group", { name: label })).toBeNull();
    }
  });

  it("顯示語言列三選回呼本機偏好（跟隨系統＝null）", async () => {
    const onLocalePrefChange = vi.fn();
    const user = userEvent.setup();
    render(view({ onLocalePrefChange }));
    const trigger = within(row("顯示語言")).getByRole("combobox", { name: "顯示語言" });
    expect(trigger.textContent).toContain("跟隨系統");
    await user.click(trigger);
    await user.click(await screen.findByRole("option", { name: "English" }));
    expect(onLocalePrefChange).toHaveBeenCalledWith("en");
  });

  it("外觀列三選回呼 onThemePrefChange（跟隨系統／淺色／深色）", async () => {
    const onThemePrefChange = vi.fn();
    const user = userEvent.setup();
    const { rerender } = render(view({ themePref: "dark", onThemePrefChange }));
    const trigger = within(row("外觀")).getByRole("combobox", { name: "外觀" });
    expect(trigger.textContent).toContain("深色");
    await user.click(trigger);
    await user.click(await screen.findByRole("option", { name: "淺色" }));
    expect(onThemePrefChange).toHaveBeenLastCalledWith("light");
    rerender(view({ themePref: "light", onThemePrefChange }));
    await user.click(within(row("外觀")).getByRole("combobox", { name: "外觀" }));
    await user.click(await screen.findByRole("option", { name: "跟隨系統" }));
    expect(onThemePrefChange).toHaveBeenLastCalledWith(null);
  });

  it("面板建立失敗時，頁底以獨立警示行浮出錯誤", () => {
    render(view({ trayPanelError: "tray panel window creation failed: boom" }));
    expect(screen.getByRole("alert").textContent).toContain("tray panel window creation failed: boom");
  });
});

describe("CLI 頁的 CLI 指令卡", () => {
  const card = () => screen.getByRole("region", { name: "CLI 指令" });
  const withCli = (over: Partial<CliInstallView>, onInstall = vi.fn(), platform: ViewProps["platform"] = "macos") =>
    view({ section: "cli", platform, cliInstall: { view: cliView(over), onInstall } });

  it("未安裝且可佈署：列「speclink CLI」顯示未安裝與安裝鈕，點擊回呼 onInstall", () => {
    const onInstall = vi.fn();
    render(withCli({}, onInstall));
    expect(within(row("speclink CLI")).getByText("未安裝")).toBeTruthy();
    fireEvent.click(within(card()).getByRole("button", { name: "安裝 CLI 指令" }));
    expect(onInstall).toHaveBeenCalledTimes(1);
  });

  it("已安裝同版：顯示已安裝與版本、不出現安裝鈕", () => {
    render(withCli({ status: { kind: "installed", version: "0.2.0" } }));
    expect(card().textContent).toContain("已安裝");
    expect(card().textContent).toContain("0.2.0");
    expect(screen.queryByRole("button", { name: "安裝 CLI 指令" })).toBeNull();
  });

  it("版本不符且可佈署：顯示版本不符與重新安裝鈕", () => {
    const onInstall = vi.fn();
    render(withCli({ status: { kind: "version-mismatch", version: "0.1.0" } }, onInstall));
    expect(card().textContent).toContain("版本不符");
    fireEvent.click(screen.getByRole("button", { name: "重新安裝" }));
    expect(onInstall).toHaveBeenCalledTimes(1);
  });

  it("佈署目錄不在 PATH：提示加入方式", () => {
    render(withCli({ status: { kind: "installed", version: "0.2.0" }, pathHint: true }));
    const hint = screen.getByTestId("cli-path-hint");
    expect(hint.textContent).toContain("/Users/u/.local/bin");
    expect(hint.textContent).toContain("PATH");
  });

  it("Windows 僅回報狀態：無安裝鈕、說明由安裝器管理", () => {
    render(
      withCli(
        { platform: "windows", canDeploy: false, deployDir: null, status: { kind: "installed", version: "0.2.0" } },
        vi.fn(),
        "windows",
      ),
    );
    expect(card().textContent).toContain("安裝器");
    expect(screen.queryByRole("button", { name: "安裝 CLI 指令" })).toBeNull();
  });

  it("Linux 非 AppImage 執行僅回報狀態：無安裝鈕、說明沒有可佈署的 CLI", () => {
    render(withCli({ platform: "linux-unpackaged", canDeploy: false, deployDir: null }, vi.fn(), "linux"));
    expect(card().textContent).toContain("沒有可佈署的 CLI");
    expect(screen.queryByRole("button", { name: "安裝 CLI 指令" })).toBeNull();
  });

  it("佈署失敗錯誤浮出於卡內", () => {
    render(withCli({ error: "permission denied" }));
    expect(card().textContent).toContain("permission denied");
  });
});

describe("一般頁的鍵盤快捷鍵卡（desktop-native-menu design D5）", () => {
  const card = () => screen.getByRole("region", { name: "鍵盤快捷鍵" });

  it("macOS：最後一張卡為「鍵盤快捷鍵」，11 列（動作名左、按鍵右）含 ⌘, 與 ⌃1–9", () => {
    render(view({ updater: updaterProps({ phase: "idle" }), cliInstall: { view: cliView(), onInstall: vi.fn() } }));
    const titles = cardTitles();
    expect(titles[titles.length - 1]).toBe("鍵盤快捷鍵");
    expect(within(card()).getAllByRole("group")).toHaveLength(11);
    expect(within(row("設定…")).getByText("⌘,")).toBeTruthy();
    expect(card().textContent).toContain("⌃1–9");
  });

  it("en 介面：卡片標題為 Keyboard Shortcuts", () => {
    rtlRender(view({ localePref: "en" }), {
      wrapper: ({ children }: { children: ReactNode }) => (
        <I18nProvider locale="en" messages={APP_MESSAGES}>
          {children}
        </I18nProvider>
      ),
    });
    expect(screen.getByRole("region", { name: "Keyboard Shortcuts" })).toBeTruthy();
  });

  it("Windows：5 列，整張卡沒有 ⌘", () => {
    render(view({ platform: "windows" }));
    expect(within(card()).getAllByRole("group")).toHaveLength(5);
    expect(card().textContent).not.toContain("⌘");
  });
});

describe("CLI 頁", () => {
  it("speclink CLI 列與等寬佈署位置列", () => {
    render(view({ section: "cli", cliInstall: { view: cliView({ status: { kind: "installed", version: "0.2.0" } }), onInstall: vi.fn() } }));
    expect(screen.getByRole("heading", { level: 2, name: "CLI" })).toBeTruthy();
    expect(within(row("speclink CLI")).getByText(/已安裝/)).toBeTruthy();
    const dir = within(row("佈署位置")).getByText("/Users/u/.local/bin");
    expect(dir.className).toContain("font-mono");
  });

  it("佈署位置未知（null）時該列缺席", () => {
    render(view({ section: "cli", cliInstall: { view: cliView({ platform: "windows", canDeploy: false, deployDir: null }), onInstall: vi.fn() } }));
    expect(screen.queryByRole("group", { name: "佈署位置" })).toBeNull();
  });
});
