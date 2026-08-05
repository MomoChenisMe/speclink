import { describe, expect, it, vi } from "vitest";
import { fireEvent, render as rtlRender, screen, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider, SEMANTIC_TONE } from "@speclink/ui";

import { AppSettingsView } from "../views/AppSettingsView";
import { APP_MESSAGES } from "../i18n/messages";
import type { CliInstallView } from "../store";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);

function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const servers = {
  connections: [],
  phases: {},
  onAdd: vi.fn().mockResolvedValue(undefined),
  onLogin: vi.fn(),
  onSubmitPat: vi.fn(),
  onLogout: vi.fn(),
  onRemove: vi.fn(),
  onRefresh: vi.fn(),
};

describe("AppSettingsView 資訊架構", () => {
  it("頁簽依序為本機設定、伺服器且預設本機設定，內容含介面語言卡與裝置本機註記", () => {
    render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={vi.fn()}
        servers={servers}
      />,
    );

    const tabs = screen.getAllByRole("tab");
    expect(tabs.map((tab) => tab.textContent)).toEqual(["本機設定", "伺服器"]);
    expect(tabs[0].getAttribute("data-state")).toBe("active");
    expect(screen.getByTestId("ui-locale-card")).toBeTruthy();
    expect(screen.getByTestId("local-note").textContent).toContain("僅存於此裝置");

    fireEvent.mouseDown(within(document.body).getByRole("tab", { name: "伺服器" }));
    expect(screen.getByTestId("servers-card")).toBeTruthy();
  });

  it("切換 UI 語言即回呼本機偏好，且不出現已拆除的系統匣樣式卡", () => {
    const onLocalePrefChange = vi.fn();
    render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={onLocalePrefChange}
      />,
    );

    const group = screen.getByTestId("ui-locale");
    fireEvent.click(within(group).getByText("English"));
    expect(onLocalePrefChange).toHaveBeenCalledWith("en");
    fireEvent.click(within(group).getByText(/跟隨系統/));
    expect(onLocalePrefChange).toHaveBeenCalledWith(null);
    expect(screen.queryByTestId("tray-style-card")).toBeNull();
    expect(screen.queryByText("系統匣樣式")).toBeNull();
  });

  it("面板建立失敗時，本機設定簽以獨立警示行浮出錯誤", () => {
    render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={vi.fn()}
        trayPanelError="tray panel window creation failed: boom"
      />,
    );

    const alert = screen.getByRole("alert");
    expect(alert.textContent).toContain("tray panel window creation failed: boom");
    expect(screen.queryByTestId("tray-style-card")).toBeNull();
  });
});

// --- 軟體更新卡（desktop-app「桌面自動更新」手動檢查入口） ---

describe("AppSettingsView 軟體更新卡", () => {
  it("未注入 updater 面時不出現更新卡", () => {
    render(<AppSettingsView localePref={null} onLocalePrefChange={vi.fn()} />);
    expect(screen.queryByTestId("updater-card")).toBeNull();
  });

  it("更新卡常駐顯示目前版本號", () => {
    render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={{ state: { phase: "idle" }, currentVersion: "0.1.0", onCheck: vi.fn() }}
      />,
    );
    expect(screen.getByTestId("updater-card").textContent).toContain("目前版本 0.1.0");
  });

  it("檢查更新按鈕回呼 onCheck；檢查中按鈕停用", () => {
    const onCheck = vi.fn();
    const { unmount } = render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={{ state: { phase: "idle" }, onCheck }}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "檢查更新" }));
    expect(onCheck).toHaveBeenCalledTimes(1);
    unmount();

    render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={{ state: { phase: "checking", manual: true }, onCheck: vi.fn() }}
      />,
    );
    expect(
      (screen.getByRole("button", { name: "檢查更新" }) as HTMLButtonElement).disabled,
    ).toBe(true);
  });

  it("手動檢查已最新顯示已是最新；檢查失敗顯示無法檢查更新", () => {
    const { unmount } = render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={{ state: { phase: "upToDate" }, onCheck: vi.fn() }}
      />,
    );
    expect(screen.getByTestId("updater-card").textContent).toContain("已是最新版本");
    unmount();

    render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={{ state: { phase: "checkFailed" }, onCheck: vi.fn() }}
      />,
    );
    expect(screen.getByTestId("updater-card").textContent).toContain("無法檢查更新");
  });

  it("發現新版時更新卡顯示目標版本", () => {
    render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={{ state: { phase: "available", version: "0.2.0" }, onCheck: vi.fn() }}
      />,
    );
    expect(screen.getByTestId("updater-card").textContent).toContain("0.2.0");
  });

  it("更新狀態語意色：檢查失敗與錯誤為紅、有新版為藍", () => {
    // spec「錯誤態以紅呈現」：更新檢查失敗是錯誤，不是待辦提醒。
    const { unmount } = render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={{ state: { phase: "checkFailed" }, onCheck: vi.fn() }}
      />,
    );
    expect(screen.getByText("無法檢查更新").className).toContain("destructive");
    unmount();

    render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={{ state: { phase: "available", version: "0.2.0" }, onCheck: vi.fn() }}
      />,
    );
    expect(screen.getByText(/有新版本/).className).toContain(SEMANTIC_TONE.inProgress);
  });
});

// --- CLI 指令卡（desktop-app「安裝 CLI 指令到 PATH」） ---

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

describe("AppSettingsView CLI 指令卡", () => {
  it("未注入 cliInstall 面時不出現 CLI 卡", () => {
    render(<AppSettingsView localePref={null} onLocalePrefChange={vi.fn()} />);
    expect(screen.queryByTestId("cli-install-card")).toBeNull();
  });

  it("未安裝且可佈署：顯示未安裝與安裝按鈕，點擊回呼 onInstall", () => {
    const onInstall = vi.fn();
    render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={vi.fn()}
        cliInstall={{ view: cliView(), onInstall }}
      />,
    );
    expect(screen.getByTestId("cli-install-card").textContent).toContain("未安裝");
    fireEvent.click(screen.getByRole("button", { name: "安裝 CLI 指令" }));
    expect(onInstall).toHaveBeenCalledTimes(1);
  });

  it("已安裝同版：顯示已安裝與版本、不出現安裝按鈕", () => {
    render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={vi.fn()}
        cliInstall={{
          view: cliView({ status: { kind: "installed", version: "0.2.0" } }),
          onInstall: vi.fn(),
        }}
      />,
    );
    const card = screen.getByTestId("cli-install-card");
    expect(card.textContent).toContain("已安裝");
    expect(card.textContent).toContain("0.2.0");
    expect(screen.queryByRole("button", { name: "安裝 CLI 指令" })).toBeNull();
  });

  it("版本不符且可佈署：顯示版本不符與重新安裝按鈕", () => {
    const onInstall = vi.fn();
    render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={vi.fn()}
        cliInstall={{
          view: cliView({ status: { kind: "version-mismatch", version: "0.1.0" } }),
          onInstall,
        }}
      />,
    );
    expect(screen.getByTestId("cli-install-card").textContent).toContain("版本不符");
    fireEvent.click(screen.getByRole("button", { name: "重新安裝" }));
    expect(onInstall).toHaveBeenCalledTimes(1);
  });

  it("佈署目錄不在 PATH：提示加入方式", () => {
    render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={vi.fn()}
        cliInstall={{
          view: cliView({
            status: { kind: "installed", version: "0.2.0" },
            pathHint: true,
          }),
          onInstall: vi.fn(),
        }}
      />,
    );
    expect(screen.getByTestId("cli-path-hint").textContent).toContain("/Users/u/.local/bin");
    expect(screen.getByTestId("cli-path-hint").textContent).toContain("PATH");
  });

  it("Windows 僅回報狀態：無安裝按鈕、顯示安裝器管理說明", () => {
    render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={vi.fn()}
        cliInstall={{
          view: cliView({
            platform: "windows",
            canDeploy: false,
            deployDir: null,
            status: { kind: "installed", version: "0.2.0" },
          }),
          onInstall: vi.fn(),
        }}
      />,
    );
    const card = screen.getByTestId("cli-install-card");
    expect(card.textContent).toContain("安裝器");
    expect(screen.queryByRole("button", { name: "安裝 CLI 指令" })).toBeNull();
  });

  it("佈署失敗錯誤浮出於卡內", () => {
    render(
      <AppSettingsView
        localePref={null}
        onLocalePrefChange={vi.fn()}
        cliInstall={{
          view: cliView({ error: "permission denied" }),
          onInstall: vi.fn(),
        }}
      />,
    );
    expect(screen.getByTestId("cli-install-card").textContent).toContain("permission denied");
  });
});
