import { describe, expect, it, vi } from "vitest";
import { fireEvent, render as rtlRender, screen, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider, SEMANTIC_TONE } from "@speclink/ui";

import { AppSettingsView, type AppSettingsUpdaterProps } from "../views/AppSettingsView";
import type { UpdaterState } from "../core/updater";
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


/** 軟體更新面的最小注入：狀態＋五個回呼（下載與安裝列的動作）。 */
function updaterProps(state: UpdaterState, over: Partial<AppSettingsUpdaterProps> = {}): AppSettingsUpdaterProps {
  return {
    state,
    onCheck: vi.fn(),
    onShowReleaseNotes: vi.fn(),
    onAccept: vi.fn(),
    onCancel: vi.fn(),
    onRelaunch: vi.fn(),
    ...over,
  };
}

describe("AppSettingsView 資訊架構", () => {
  it("頁簽依序為本機設定、伺服器且預設本機設定，內容含介面語言卡與裝置本機註記", () => {
    render(
      <AppSettingsView
        platform="macos"
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
        platform="macos"
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
        platform="macos"
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
    render(<AppSettingsView platform="macos" localePref={null} onLocalePrefChange={vi.fn()} />);
    expect(screen.queryByTestId("updater-card")).toBeNull();
  });

  it("更新卡常駐顯示目前版本號", () => {
    render(
      <AppSettingsView
        platform="macos"
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={updaterProps({ phase: "idle" }, { currentVersion: "0.1.0", onCheck: vi.fn(), onShowReleaseNotes: vi.fn() })}
      />,
    );
    expect(screen.getByTestId("updater-card").textContent).toContain("目前版本 0.1.0");
  });

  it("檢查更新按鈕回呼 onCheck；檢查中按鈕停用", () => {
    const onCheck = vi.fn();
    const { unmount } = render(
      <AppSettingsView
        platform="macos"
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={updaterProps({ phase: "idle" }, { onCheck, onShowReleaseNotes: vi.fn() })}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "檢查更新" }));
    expect(onCheck).toHaveBeenCalledTimes(1);
    unmount();

    render(
      <AppSettingsView
        platform="macos"
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={updaterProps({ phase: "checking", manual: true }, { onCheck: vi.fn(), onShowReleaseNotes: vi.fn() })}
      />,
    );
    expect(
      (screen.getByRole("button", { name: "檢查更新" }) as HTMLButtonElement).disabled,
    ).toBe(true);
  });

  it("手動檢查已最新顯示已是最新；檢查失敗顯示無法檢查更新", () => {
    const { unmount } = render(
      <AppSettingsView
        platform="macos"
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={updaterProps({ phase: "upToDate" }, { onCheck: vi.fn(), onShowReleaseNotes: vi.fn() })}
      />,
    );
    expect(screen.getByTestId("updater-card").textContent).toContain("已是最新版本");
    unmount();

    render(
      <AppSettingsView
        platform="macos"
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={updaterProps({ phase: "checkFailed" }, { onCheck: vi.fn(), onShowReleaseNotes: vi.fn() })}
      />,
    );
    expect(screen.getByTestId("updater-card").textContent).toContain("無法檢查更新");
  });

  it("發現新版時更新卡顯示目標版本", () => {
    render(
      <AppSettingsView
        platform="macos"
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={updaterProps({ phase: "available", version: "0.2.0" }, { onCheck: vi.fn(), onShowReleaseNotes: vi.fn() })}
      />,
    );
    expect(screen.getByTestId("updater-card").textContent).toContain("0.2.0");
  });

  it("更新狀態語意色：檢查失敗與錯誤為紅、有新版為藍", () => {
    // spec「錯誤態以紅呈現」：更新檢查失敗是錯誤，不是待辦提醒。
    const { unmount } = render(
      <AppSettingsView
        platform="macos"
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={updaterProps({ phase: "checkFailed" }, { onCheck: vi.fn(), onShowReleaseNotes: vi.fn() })}
      />,
    );
    expect(screen.getByText("無法檢查更新").className).toContain("destructive");
    unmount();

    render(
      <AppSettingsView
        platform="macos"
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={updaterProps({ phase: "available", version: "0.2.0" }, { onCheck: vi.fn(), onShowReleaseNotes: vi.fn() })}
      />,
    );
    expect(screen.getByText(/有新版本/).className).toContain(SEMANTIC_TONE.inProgress);
  });

  // --- 「下載與安裝」列（desktop-notice-relocation design D3）與聚焦捲動 ---

  it("發現新版時卡內有「下載與安裝」列與「下載」鈕；行內狀態不重複顯示；按下回呼 onAccept", () => {
    const updater = updaterProps({ phase: "available", version: "0.2.0" });
    render(<AppSettingsView platform="macos" localePref={null} onLocalePrefChange={vi.fn()} updater={updater} />);
    const card = screen.getByTestId("updater-card");
    expect(within(card).getByTestId("update-install-row")).toBeTruthy();
    expect(within(card).getAllByText(/有新版本/)).toHaveLength(1);
    fireEvent.click(within(card).getByRole("button", { name: "下載" }));
    expect(updater.onAccept).toHaveBeenCalledTimes(1);
  });

  it("下載中、待重啟與失敗：列承載控制，行內狀態不重複；閒置無列", () => {
    for (const [state, label] of [
      [{ phase: "downloading", version: "0.2.0" }, "取消"],
      [{ phase: "restartPending", version: "0.2.0" }, "安裝並重新啟動"],
      [{ phase: "error", message: "invalid signature" }, "重試"],
    ] as const) {
      const { unmount } = render(
        <AppSettingsView platform="macos" localePref={null} onLocalePrefChange={vi.fn()} updater={updaterProps(state)} />,
      );
      const card = screen.getByTestId("updater-card");
      expect(within(card).getByRole("button", { name: label })).toBeTruthy();
      expect(within(card).getAllByTestId("update-install-row")).toHaveLength(1);
      unmount();
    }
    render(<AppSettingsView platform="macos" localePref={null} onLocalePrefChange={vi.fn()} updater={updaterProps({ phase: "idle" })} />);
    expect(screen.queryByTestId("update-install-row")).toBeNull();
  });

  it("focus 為 true 時捲到軟體更新卡一次並回呼 onFocusHandled；false 時不捲", () => {
    const scrollIntoView = vi.fn();
    Element.prototype.scrollIntoView = scrollIntoView;
    const onFocusHandled = vi.fn();
    const { rerender } = render(
      <AppSettingsView
        platform="macos"
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={updaterProps({ phase: "available", version: "0.2.0" }, { focus: false, onFocusHandled })}
      />,
    );
    expect(scrollIntoView).not.toHaveBeenCalled();
    rerender(
      <AppSettingsView
        platform="macos"
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={updaterProps({ phase: "available", version: "0.2.0" }, { focus: true, onFocusHandled })}
      />,
    );
    expect(scrollIntoView).toHaveBeenCalledTimes(1);
    expect(scrollIntoView.mock.instances[0]).toBe(screen.getByTestId("updater-card"));
    expect(onFocusHandled).toHaveBeenCalledTimes(1);
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
    render(<AppSettingsView platform="macos" localePref={null} onLocalePrefChange={vi.fn()} />);
    expect(screen.queryByTestId("cli-install-card")).toBeNull();
  });

  it("未安裝且可佈署：顯示未安裝與安裝按鈕，點擊回呼 onInstall", () => {
    const onInstall = vi.fn();
    render(
      <AppSettingsView
        platform="macos"
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
        platform="macos"
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
        platform="macos"
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
        platform="macos"
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
        platform="windows"
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

  it("Linux 非 AppImage 執行僅回報狀態：無安裝按鈕、說明沒有可佈署的 CLI", () => {
    render(
      <AppSettingsView
        platform="linux"
        localePref={null}
        onLocalePrefChange={vi.fn()}
        cliInstall={{
          view: cliView({
            platform: "linux-unpackaged",
            canDeploy: false,
            deployDir: null,
            status: { kind: "not-installed" },
          }),
          onInstall: vi.fn(),
        }}
      />,
    );
    const card = screen.getByTestId("cli-install-card");
    expect(card.textContent).toContain("沒有可佈署的 CLI");
    expect(screen.queryByRole("button", { name: "安裝 CLI 指令" })).toBeNull();
  });

  it("佈署失敗錯誤浮出於卡內", () => {
    render(
      <AppSettingsView
        platform="macos"
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

describe("AppSettingsView 更新日誌入口（desktop-app「更新日誌彈窗」設定頁瀏覽）", () => {
  it("更新卡有「更新日誌」按鈕，按下回呼 onShowReleaseNotes", () => {
    const onShowReleaseNotes = vi.fn();
    render(
      <AppSettingsView
        platform="macos"
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={updaterProps({ phase: "idle" }, { onCheck: vi.fn(), onShowReleaseNotes })}
      />,
    );
    fireEvent.click(within(screen.getByTestId("updater-card")).getByRole("button", { name: "更新日誌" }));
    expect(onShowReleaseNotes).toHaveBeenCalledTimes(1);
  });

  it("updater 未注入時「更新日誌」按鈕不存在", () => {
    render(<AppSettingsView platform="macos" localePref={null} onLocalePrefChange={vi.fn()} />);
    expect(screen.queryByRole("button", { name: "更新日誌" })).toBeNull();
  });
});

describe("AppSettingsView 鍵盤快捷鍵卡（desktop-native-menu design D5）", () => {
  const localCards = () =>
    Array.from(screen.getByRole("tabpanel").children)
      .map((el) => el.getAttribute("data-testid") ?? "")
      .filter((id) => id.endsWith("-card"));

  it("macOS：本機設定簽最後一張卡為「鍵盤快捷鍵」，列出 11 列且含 ⌘, 與 ⌃1–9", () => {
    render(
      <AppSettingsView
        platform="macos"
        localePref={null}
        onLocalePrefChange={vi.fn()}
        updater={updaterProps({ phase: "idle" })}
        cliInstall={{ view: cliView(), onInstall: vi.fn() }}
      />,
    );
    const cards = localCards();
    expect(cards[cards.length - 1]).toBe("shortcuts-card");
    const card = screen.getByTestId("shortcuts-card");
    expect(within(card).getByText("鍵盤快捷鍵")).toBeTruthy();
    expect(within(card).getAllByRole("listitem")).toHaveLength(11);
    expect(card.textContent).toContain("⌘,");
    expect(card.textContent).toContain("⌃1–9");
  });

  it("en 介面：卡片標題為 Keyboard Shortcuts", () => {
    rtlRender(<AppSettingsView platform="macos" localePref="en" onLocalePrefChange={vi.fn()} />, {
      wrapper: ({ children }: { children: ReactNode }) => (
        <I18nProvider locale="en" messages={APP_MESSAGES}>
          {children}
        </I18nProvider>
      ),
    });
    expect(within(screen.getByTestId("shortcuts-card")).getByText("Keyboard Shortcuts")).toBeTruthy();
  });

  it("Windows：列出 5 列，整張卡沒有 ⌘", () => {
    render(<AppSettingsView platform="windows" localePref={null} onLocalePrefChange={vi.fn()} />);
    const card = screen.getByTestId("shortcuts-card");
    expect(within(card).getAllByRole("listitem")).toHaveLength(5);
    expect(card.textContent).not.toContain("⌘");
  });
});

describe("AppSettingsView 手動檢查更新時顯示軟體更新卡（desktop-native-menu「檢查更新…」）", () => {
  const view = (
    state: UpdaterState,
    focusConnectionId: string | null = null,
    over: Partial<AppSettingsUpdaterProps> = {},
  ) => (
    <AppSettingsView
      platform="macos"
      localePref={null}
      onLocalePrefChange={vi.fn()}
      servers={servers}
      focusConnectionId={focusConnectionId}
      updater={updaterProps(state, over)}
    />
  );
  const activeTab = () =>
    screen.getAllByRole("tab").find((tab) => tab.getAttribute("data-state") === "active")?.textContent;

  it("停在伺服器簽時開始手動檢查，切回本機設定簽", () => {
    const { rerender } = render(view({ phase: "idle" }));
    fireEvent.mouseDown(screen.getByRole("tab", { name: "伺服器" }));
    expect(activeTab()).toBe("伺服器");
    rerender(view({ phase: "checking", manual: true }));
    expect(activeTab()).toBe("本機設定");
    expect(screen.getByTestId("updater-card")).toBeTruthy();
  });

  it("需要重新登入的導向（focusConnectionId）切到伺服器簽", () => {
    const { rerender } = render(view({ phase: "idle" }));
    expect(activeTab()).toBe("本機設定");
    rerender(view({ phase: "idle" }, "conn-1"));
    expect(activeTab()).toBe("伺服器");
  });

  it("背景自動檢查不切簽", () => {
    const { rerender } = render(view({ phase: "idle" }));
    fireEvent.mouseDown(screen.getByRole("tab", { name: "伺服器" }));
    rerender(view({ phase: "checking", manual: false }));
    expect(activeTab()).toBe("伺服器");
  });

  it("停在伺服器簽時聚焦軟體更新卡（圖示列更新鈕、toast「查看」）：切回本機設定簽並捲到卡一次", () => {
    // desktop-notice-relocation × desktop-native-menu：聚焦與手動檢查同理，卡在本機設定簽，
    // 停在伺服器簽時要先切簽、等卡掛上才捲。
    const scrollIntoView = vi.fn();
    Element.prototype.scrollIntoView = scrollIntoView;
    const onFocusHandled = vi.fn();
    const available = { phase: "available", version: "0.2.0" } as const;
    const { rerender } = render(view(available, null, { focus: false, onFocusHandled }));
    fireEvent.mouseDown(screen.getByRole("tab", { name: "伺服器" }));
    expect(activeTab()).toBe("伺服器");
    rerender(view(available, null, { focus: true, onFocusHandled }));
    expect(activeTab()).toBe("本機設定");
    expect(scrollIntoView).toHaveBeenCalledTimes(1);
    expect(scrollIntoView.mock.instances[0]).toBe(screen.getByTestId("updater-card"));
    expect(onFocusHandled).toHaveBeenCalledTimes(1);
  });
});
