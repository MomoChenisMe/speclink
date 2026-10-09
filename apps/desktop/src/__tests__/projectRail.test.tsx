// 圖示列（design D3；desktop-config「專案分頁列存於 app 本機」、desktop-app「分頁切換中
// 即時回饋」）：品牌標記、每專案一個首字母方塊、「＋」、底部齒輪。原 projectTabs.test.tsx
// 的案例逐條搬過來（方塊取代分頁），再補 tooltip、角落狀態點、錯誤選單與齒輪。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider, SEMANTIC_TONE } from "@speclink/ui";

import { ProjectRail, type ProjectRailProps } from "../components/ProjectRail";
import type { ProjectActions } from "../components/ProjectActionsMenu";
import { APP_MESSAGES } from "../i18n/messages";
import type { ProjectTab } from "../tabs";
import type { RemoteWorkspaceRecoveryState, WorkspaceLocator } from "../session";

const zh = APP_MESSAGES["zh-TW"];
const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const local = (root: string): WorkspaceLocator => ({ kind: "local", root });
const KEY_ALPHA = "local:C:\\proj\\alpha";
const KEY_BETA = "local:C:\\proj\\beta";

const tabs: ProjectTab[] = [
  { locator: local("C:\\proj\\alpha"), name: "alpha" },
  { locator: local("C:\\proj\\beta"), name: "beta" },
];

function fakeActions(): ProjectActions {
  return {
    reveal: vi.fn(),
    openTerminal: vi.fn(),
    openEditor: vi.fn(),
    copyPath: vi.fn(),
    refresh: vi.fn(),
    close: vi.fn(),
  };
}

function renderRail(over: Partial<ProjectRailProps> = {}) {
  const props: ProjectRailProps = {
    tabs,
    activeKey: KEY_ALPHA,
    tabErrors: {},
    platform: "macos",
    actions: fakeActions(),
    ...over,
  };
  render(<ProjectRail {...props} />);
  return props;
}

const square = (key: string) =>
  Array.from(document.querySelectorAll<HTMLElement>("[data-tab]")).find(
    (el) => el.getAttribute("data-tab") === key,
  ) as HTMLElement;

describe("ProjectRail", () => {
  it("品牌標記在頂、方塊依序一個專案一個、作用中為主色底", () => {
    renderRail();
    const rail = document.querySelector("[data-project-rail]") as HTMLElement;
    expect(rail.firstElementChild?.getAttribute("aria-label")).toBe("Speclink");
    const list = screen.getByRole("tablist", { name: zh["app.projectRail"] });
    const squares = within(list).getAllByRole("tab");
    expect(squares.map((s) => s.textContent)).toEqual(["A", "B"]);
    expect(squares[0].getAttribute("aria-selected")).toBe("true");
    expect(squares[0].className).toContain("bg-primary");
    expect(squares[1].getAttribute("aria-selected")).toBe("false");
    expect(squares[1].className).not.toContain("bg-primary");
    expect(squares[1].className).toContain("bg-muted");
  });

  it("方塊無任何計數徽章（spec「分頁不顯示計數徽章」）", () => {
    renderRail();
    expect(document.querySelector("[data-badge]")).toBeNull();
    for (const tab of screen.getAllByRole("tab")) expect(tab.textContent).toMatch(/^[A-Z]$/);
  });

  it("CJK 顯示名取第一個字", () => {
    renderRail({ tabs: [{ locator: local("/w/規格"), name: "規格中心" }], activeKey: null });
    expect(screen.getByRole("tab").textContent).toBe("規");
  });

  it("點背景方塊以其 locator key 呼叫 onActivate；點作用中方塊不動作", () => {
    const onActivate = vi.fn();
    renderRail({ onActivate });
    fireEvent.click(square(KEY_BETA));
    fireEvent.click(square(KEY_ALPHA));
    expect(onActivate.mock.calls).toEqual([[KEY_BETA]]);
  });

  it("右鍵方塊彈出專案動作選單；「關閉專案」以該 key 呼叫 close、不切換", () => {
    const onActivate = vi.fn();
    const { actions } = renderRail({ onActivate });
    fireEvent.contextMenu(square(KEY_BETA));
    expect(screen.getAllByRole("menuitem")).toHaveLength(6);
    fireEvent.click(screen.getByRole("menuitem", { name: /^關閉專案/ }));
    expect(actions.close).toHaveBeenCalledWith(KEY_BETA);
    expect(onActivate).not.toHaveBeenCalled();
  });

  it("「＋」虛線方塊呼叫 onOpen", () => {
    const onOpen = vi.fn();
    renderRail({ onOpen });
    const plus = screen.getByRole("button", { name: zh["app.newProject"] });
    expect(plus.className).toContain("border-dashed");
    fireEvent.click(plus);
    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it("底部齒輪呼叫 onOpenSettings，settingsActive 時為主色底", () => {
    const onOpenSettings = vi.fn();
    renderRail({ onOpenSettings, settingsActive: true });
    const gear = screen.getByRole("button", { name: zh["app.navSettings"] });
    expect(gear.className).toContain("bg-primary");
    fireEvent.click(gear);
    expect(onOpenSettings).toHaveBeenCalledTimes(1);
  });

  it("齒輪非作用中時無主色底", () => {
    renderRail({ settingsActive: false });
    expect(screen.getByRole("button", { name: zh["app.navSettings"] }).className).not.toContain(
      "bg-primary",
    );
  });

  it("路徑失效的本機方塊：角落紅點、點擊開錯誤選單（訊息＋自專案列移除）而不切換", async () => {
    const user = userEvent.setup();
    const onActivate = vi.fn();
    const { actions } = renderRail({ onActivate, tabErrors: { [KEY_BETA]: "目錄不存在" } });
    const beta = square(KEY_BETA);
    expect(beta.getAttribute("data-error")).toBe("true");
    expect(beta.querySelector("[data-tab-dot]")?.className).toContain("bg-destructive");
    await user.click(beta);
    expect(onActivate).not.toHaveBeenCalled();
    expect(screen.getByText("目錄不存在")).toBeTruthy();
    expect(screen.getAllByRole("menuitem").map((i) => i.textContent)).toEqual([zh["app.removeTab"]]);
    await user.click(screen.getByRole("menuitem", { name: zh["app.removeTab"] }));
    expect(actions.close).toHaveBeenCalledWith(KEY_BETA);
  });

  it("路徑失效的本機方塊按右鍵：同樣彈出錯誤變體（訊息＋自專案列移除）", () => {
    const { actions } = renderRail({ tabErrors: { [KEY_BETA]: "目錄不存在" } });
    fireEvent.contextMenu(square(KEY_BETA));
    expect(screen.getByText("目錄不存在")).toBeTruthy();
    expect(screen.getAllByRole("menuitem").map((i) => i.textContent)).toEqual([zh["app.removeTab"]]);
    fireEvent.click(screen.getByRole("menuitem", { name: zh["app.removeTab"] }));
    expect(actions.close).toHaveBeenCalledWith(KEY_BETA);
  });

  it("本機方塊 tooltip 顯示顯示名與完整路徑", async () => {
    renderRail();
    fireEvent.focus(square(KEY_BETA));
    const tip = await screen.findByRole("tooltip");
    expect(tip.textContent).toContain("beta");
    expect(tip.textContent).toContain("C:\\proj\\beta");
  });

  it("remote 有 checkout：tooltip 明示連接的 checkout 路徑", async () => {
    renderRail({
      tabs: [
        {
          locator: {
            kind: "remote",
            connectionId: "c1",
            projectId: "demo",
            repoId: "backend",
            checkoutRoot: "/work/backend",
          },
          name: "Demo/backend",
        },
      ],
      activeKey: "remote:c1/demo/backend",
    });
    fireEvent.focus(screen.getByRole("tab"));
    const tip = await screen.findByRole("tooltip");
    expect(tip.textContent).toContain("Demo/backend");
    expect(tip.textContent).toContain("已連接 checkout：/work/backend");
  });

  it("remote 無 checkout：tooltip 顯示連線名稱加 Project/Repo", async () => {
    renderRail({
      tabs: [
        {
          locator: { kind: "remote", connectionId: "c1", projectId: "demo", repoId: "api" },
          name: "Demo/api",
        },
      ],
      activeKey: null,
      connectionNames: { c1: "公司 Server" },
    });
    fireEvent.focus(screen.getByRole("tab"));
    const tip = await screen.findByRole("tooltip");
    expect(tip.textContent).toContain("公司 Server · demo/api");
  });

  it("remote 復原錯誤仍是可選的目的地：紅點、tooltip 只給精簡狀態不露技術細節", async () => {
    const onActivate = vi.fn();
    const recoveryStates: Record<string, RemoteWorkspaceRecoveryState> = {
      "remote:c1/demo/backend": {
        status: "error",
        failure: {
          kind: "unreachable",
          message: "server unreachable — technical detail only",
          reason: null,
          status: null,
        },
      },
    };
    renderRail({
      tabs: [
        {
          locator: { kind: "remote", connectionId: "c1", projectId: "demo", repoId: "backend" },
          name: "Demo/backend",
        },
      ],
      activeKey: null,
      tabErrors: { "remote:c1/demo/backend": "server unreachable — technical detail only" },
      recoveryStates,
      onActivate,
    });
    const tab = screen.getByRole("tab", { name: /Demo\/backend/ });
    expect(tab.getAttribute("data-status")).toBe("error");
    // spec「錯誤態以紅呈現」：連不上是錯誤，不能與 offline／需重新登入的琥珀同色。
    const dot = tab.querySelector("[data-tab-dot]") as HTMLElement;
    expect(dot.className).toContain("bg-destructive");
    expect(dot.className).not.toContain("status-warning");
    fireEvent.click(tab);
    expect(onActivate).toHaveBeenCalledWith("remote:c1/demo/backend");
    fireEvent.focus(tab);
    const tip = await screen.findByRole("tooltip");
    expect(tip.textContent).toContain("無法連線");
    expect(tip.textContent).not.toContain("technical detail");
  });

  it("remote 離線或需重新登入：角落琥珀點", () => {
    renderRail({
      tabs: [
        {
          locator: { kind: "remote", connectionId: "c1", projectId: "demo", repoId: "api" },
          name: "Demo/api",
        },
        {
          locator: { kind: "remote", connectionId: "c1", projectId: "demo", repoId: "web" },
          name: "Demo/web",
        },
      ],
      activeKey: null,
      connectionStates: {
        "remote:c1/demo/api": { connectionId: "c1", state: "offline", message: null },
        "remote:c1/demo/web": { connectionId: "c1", state: "needs-reauth", message: null },
      },
    });
    for (const key of ["remote:c1/demo/api", "remote:c1/demo/web"]) {
      expect(square(key).querySelector("[data-tab-dot]")?.className).toContain("bg-status-warning");
    }
  });

  it("背景復原中的方塊：滑鼠與鍵盤皆可啟用，還原 spinner 為進行中色", () => {
    const onActivate = vi.fn();
    renderRail({
      tabs: [
        {
          locator: { kind: "remote", connectionId: "c1", projectId: "demo", repoId: "backend" },
          name: "Demo/backend",
        },
      ],
      activeKey: null,
      recoveryStates: { "remote:c1/demo/backend": { status: "restoring", failure: null } },
      onActivate,
    });
    const tab = screen.getByRole("tab", { name: /Demo\/backend/ });
    fireEvent.keyDown(tab, { key: "Enter" });
    fireEvent.click(tab);
    expect(onActivate).toHaveBeenCalledTimes(2);
    expect(tab.getAttribute("data-status")).toBe("restoring");
    expect(tab.querySelector('[data-tab-status="restoring"]')?.getAttribute("class")).toContain(
      SEMANTIC_TONE.inProgress,
    );
  });
});

// spec「分頁切換中即時回饋」（design D1）：探測擋在翻頁前，目標方塊先出 spinner。
describe("切換中方塊 spinner", () => {
  it("pendingKey 指向的方塊以 spinner 取代首字母，aria-label 含「正在切換」", () => {
    renderRail({ pendingKey: KEY_BETA });
    const beta = square(KEY_BETA);
    expect(beta.textContent).toBe("");
    expect(beta.querySelector('[data-tab-status="pending"]')?.getAttribute("class")).toContain(
      "animate-spin",
    );
    expect(beta.getAttribute("aria-label")).toContain(zh["app.tabSwitching"]);
  });

  it("spinner 帶切換中語意的無障礙標籤", () => {
    renderRail({ pendingKey: KEY_BETA });
    expect(screen.getByLabelText(zh["app.tabSwitching"])).toBeTruthy();
  });

  it("非 pending 的方塊不出 spinner", () => {
    renderRail({ pendingKey: KEY_BETA });
    expect(square(KEY_ALPHA).querySelector('[data-tab-status="pending"]')).toBeNull();
    expect(square(KEY_ALPHA).textContent).toBe("A");
  });

  it("未給 pendingKey → 全部方塊照舊，無 spinner", () => {
    renderRail();
    expect(document.querySelector('[data-tab-status="pending"]')).toBeNull();
  });

  // 錯誤方塊經快捷鍵重探：仍在錯誤態但正在重探，切換中回饋優先，紅點照舊。
  it("重探錯誤方塊時 spinner 蓋過首字母，錯誤紅點照舊", () => {
    renderRail({ tabErrors: { [KEY_BETA]: "目錄不存在" }, pendingKey: KEY_BETA });
    const beta = square(KEY_BETA);
    expect(beta.querySelectorAll("[data-tab-status]")).toHaveLength(1);
    expect(beta.querySelector('[data-tab-status="pending"]')).toBeTruthy();
    expect(beta.getAttribute("data-error")).toBe("true");
    expect(beta.querySelector("[data-tab-dot]")?.className).toContain("bg-destructive");
  });
});
