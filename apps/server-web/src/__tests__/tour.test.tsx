import { describe, it, expect, beforeEach, vi } from "vitest";
import { render } from "@testing-library/react";
import { screen, within, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { App } from "../App";
import { makeAdminClient, setViewport } from "./helpers/adminHarness";

// spec 需求「首次進入提供可略過的分步導覽」：首次開啟管理面自動啟動、每一步指向畫面上
// 實際存在的元素並附一句說明、可上一步／下一步／略過、看過後不再自動啟動、系統頁保留
// 重新啟動入口、目標元素缺席時跳過該步。
//
// 這裡不用 adminHarness 的 renderAt——它刻意把「看過了」預先寫進去，好讓其他頁的測試
// 不被導覽蓋住。導覽自己的測試要從「沒看過」開始。

const TOUR_KEY = "speclink.tourSeen";

beforeEach(() => {
  setViewport(false);
  localStorage.clear();
});

function renderFresh(route: string) {
  return render(<App client={makeAdminClient() as never} initialEntries={[route]} />);
}

/** 導覽疊層；未啟動時為 null。 */
function tour(): HTMLElement | null {
  return screen.queryByRole("region", { name: "導覽" });
}

describe("首次進入自動啟動", () => {
  it("尚未看過時開啟 /admin 自動啟動，第一步指向總覽並提供下一步與略過", async () => {
    renderFresh("/admin");
    const panel = await screen.findByRole("region", { name: "導覽" });
    expect(within(panel).getByText("總覽")).toBeTruthy();
    expect(within(panel).getByRole("button", { name: "下一步" })).toBeTruthy();
    expect(within(panel).getByRole("button", { name: "略過" })).toBeTruthy();
    // 第一步沒有上一步可回。
    expect(
      (within(panel).getByRole("button", { name: "上一步" }) as HTMLButtonElement).disabled,
    ).toBe(true);
  });

  it("已看過時不自動啟動", async () => {
    localStorage.setItem(TOUR_KEY, "1");
    renderFresh("/admin");
    await screen.findByRole("heading", { level: 1, name: "總覽" });
    expect(tour()).toBeNull();
  });

  it("略過後重新整理不再自動啟動", async () => {
    const user = userEvent.setup();
    const { unmount } = renderFresh("/admin");
    await user.click(within(await screen.findByRole("region", { name: "導覽" })).getByRole("button", { name: "略過" }));
    await waitFor(() => expect(tour()).toBeNull());
    unmount();

    renderFresh("/admin");
    await screen.findByRole("heading", { level: 1, name: "總覽" });
    expect(tour()).toBeNull();
  });

  it("走完最後一步後重新整理不再自動啟動", async () => {
    const user = userEvent.setup();
    const { unmount } = renderFresh("/admin");
    let panel = await screen.findByRole("region", { name: "導覽" });
    while (within(panel).queryByRole("button", { name: "下一步" })) {
      await user.click(within(panel).getByRole("button", { name: "下一步" }));
      panel = screen.getByRole("region", { name: "導覽" });
    }
    await user.click(within(panel).getByRole("button", { name: "完成" }));
    await waitFor(() => expect(tour()).toBeNull());
    unmount();

    renderFresh("/admin");
    await screen.findByRole("heading", { level: 1, name: "總覽" });
    expect(tour()).toBeNull();
  });
});

describe("重新啟動入口", () => {
  it("系統頁的重看導覽可再次啟動", async () => {
    localStorage.setItem(TOUR_KEY, "1");
    const user = userEvent.setup();
    renderFresh("/admin/system");
    await screen.findByRole("heading", { level: 1, name: "系統" });
    expect(tour()).toBeNull();
    await user.click(screen.getByRole("button", { name: "重看導覽" }));
    expect(await screen.findByRole("region", { name: "導覽" })).toBeTruthy();
  });
});

describe("目標元素缺席時跳過該步", () => {
  it("總覽頁沒有列表 primary action，該步不出現且導覽不中斷", async () => {
    const user = userEvent.setup();
    renderFresh("/admin");
    let panel = await screen.findByRole("region", { name: "導覽" });
    const seen: string[] = [];
    while (within(panel).queryByRole("button", { name: "下一步" })) {
      seen.push(panel.getAttribute("data-tour-step") ?? "");
      await user.click(within(panel).getByRole("button", { name: "下一步" }));
      panel = screen.getByRole("region", { name: "導覽" });
    }
    seen.push(panel.getAttribute("data-tour-step") ?? "");
    // 六個側欄目的地都走到，指向列表 primary action 的那一步被跳過。
    expect(seen).toEqual([
      "nav-overview",
      "nav-users",
      "nav-registry",
      "nav-credentials",
      "nav-system",
      "nav-audit",
    ]);
    expect(within(panel).getByRole("button", { name: "完成" })).toBeTruthy();
  });

  it("使用者頁存在該元素時該步出現在最後", async () => {
    const user = userEvent.setup();
    renderFresh("/admin/users");
    let panel = await screen.findByRole("region", { name: "導覽" });
    while (within(panel).queryByRole("button", { name: "下一步" })) {
      await user.click(within(panel).getByRole("button", { name: "下一步" }));
      panel = screen.getByRole("region", { name: "導覽" });
    }
    expect(panel.getAttribute("data-tour-step")).toBe("list-primary");
  });
});

describe("疊層貼著它正在指的元素", () => {
  it("卡片以目標的座標定位，而不是固定在畫面底部", async () => {
    renderFresh("/admin");
    const panel = await screen.findByRole("region", { name: "導覽" });
    // 定位由目標的 rect 決定：卡片帶 inline 座標，而非寫死的 bottom-4 left-1/2。
    expect(panel.style.top, "卡片應有依目標算出的 top").not.toBe("");
    expect(panel.style.left, "卡片應有依目標算出的 left").not.toBe("");
    expect(panel.className).not.toContain("bottom-4");
  });
});

describe("鍵盤與降級", () => {
  it("導覽進行中按 Escape 離開", async () => {
    const user = userEvent.setup();
    renderFresh("/admin");
    await screen.findByRole("region", { name: "導覽" });
    await user.keyboard("{Escape}");
    await waitFor(() => expect(tour()).toBeNull());
  });

  it("上一步回到前一步", async () => {
    const user = userEvent.setup();
    renderFresh("/admin");
    let panel = await screen.findByRole("region", { name: "導覽" });
    await user.click(within(panel).getByRole("button", { name: "下一步" }));
    panel = screen.getByRole("region", { name: "導覽" });
    expect(panel.getAttribute("data-tour-step")).toBe("nav-users");
    await user.click(within(panel).getByRole("button", { name: "上一步" }));
    expect(screen.getByRole("region", { name: "導覽" }).getAttribute("data-tour-step")).toBe(
      "nav-overview",
    );
  });

  it("localStorage 讀寫丟例外時導覽照常運作，只是不持久化", async () => {
    const getItem = vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("localStorage disabled");
    });
    const setItem = vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("localStorage disabled");
    });
    try {
      const user = userEvent.setup();
      renderFresh("/admin");
      // 讀取失敗＝視為未看過，導覽照常啟動。
      const panel = await screen.findByRole("region", { name: "導覽" });
      // 寫入失敗不得讓略過丟例外。
      await user.click(within(panel).getByRole("button", { name: "略過" }));
      await waitFor(() => expect(tour()).toBeNull());
    } finally {
      getItem.mockRestore();
      setItem.mockRestore();
    }
  });
});
