import { describe, it, expect, beforeEach } from "vitest";
import { screen, within } from "@testing-library/react";
import { makeAdminClient, renderAt, setViewport } from "./helpers/adminHarness";

// spec 需求「共用設計系統維持高密度可存取體驗」的 Scenario「工具列控件高度一致」：
// 同一組工具列內的搜尋框、下拉與日期輸入高度相同，下拉是套用 theme 的 Select 原語而非
// 原生 <select>，且每個控件都有無障礙名稱（工具列在頁標題區，沒有可見標籤）。高度以 class
// 上的 h-* 比對——參差的成因就是兩套原語的預設高度不同，所以直接釘住「彼此相同」而非某個數值。

beforeEach(() => setViewport(false));

// Radix Select 在 <form> 內會額外渲染一個 aria-hidden 的原生 select 供表單送出使用。
// 那不是使用者看得到或操作得到的控件，所以「不得有原生 select」指的是可見的那一種。
const VISIBLE_NATIVE_SELECT = "select:not([aria-hidden='true'])";

/** 該元素 class 上的 tailwind 高度 utility（例如 h-8）。 */
function heightClass(el: Element): string | undefined {
  return el.className.toString().split(/\s+/).find((c) => /^h-\d/.test(c));
}

describe("列表工具列的下拉是 Select 原語", () => {
  it("使用者頁的狀態篩選是 combobox 而非原生 select", async () => {
    renderAt("/admin/users", makeAdminClient());
    const toolbar = await screen.findByRole("combobox", { name: /狀態/ });
    expect(toolbar.tagName).not.toBe("SELECT");
    expect(document.querySelector(VISIBLE_NATIVE_SELECT)).toBeNull();
  });

  it("憑證頁的狀態篩選是 combobox 而非原生 select", async () => {
    renderAt("/admin/credentials", makeAdminClient());
    const trigger = await screen.findByRole("combobox", { name: /狀態/ });
    expect(trigger.tagName).not.toBe("SELECT");
    expect(document.querySelector(VISIBLE_NATIVE_SELECT)).toBeNull();
  });
});

describe("工具列控件高度一致", () => {
  it("稽核頁的搜尋框、兩個下拉與兩個日期輸入高度相同，各有無障礙名稱，下拉不是原生 select", async () => {
    renderAt("/admin/audit", makeAdminClient());
    const search = await screen.findByRole("textbox", { name: "搜尋" });
    const action = screen.getByRole("combobox", { name: "動作" });
    const source = screen.getByRole("combobox", { name: "來源" });
    const from = screen.getByLabelText("起始日期");
    const to = screen.getByLabelText("結束日期");

    const h = heightClass(search);
    expect(h, "搜尋框應帶明確的高度 class").toBeTruthy();
    for (const control of [action, source, from, to]) {
      expect(heightClass(control)).toBe(h);
    }
    expect(search.className).toContain("rounded-full");
    expect(action.tagName).not.toBe("SELECT");
    expect(source.tagName).not.toBe("SELECT");
    expect(document.querySelector(VISIBLE_NATIVE_SELECT)).toBeNull();
  });

  it("使用者頁的搜尋框與狀態下拉高度相同", async () => {
    renderAt("/admin/users", makeAdminClient());
    const search = await screen.findByRole("textbox", { name: "搜尋" });
    const status = screen.getByRole("combobox", { name: /狀態/ });
    expect(heightClass(status)).toBe(heightClass(search));
  });
});

describe("使用者詳情頁的成員資格列使用 Select 原語", () => {
  it("每列的角色與加入專案的專案、角色選擇皆為 combobox", async () => {
    const { default: userEvent } = await import("@testing-library/user-event");
    const user = userEvent.setup();
    renderAt("/admin/users/m1", makeAdminClient());

    await screen.findByRole("heading", { level: 1, name: "Member" });
    await user.click(screen.getByRole("tab", { name: /^成員資格/ }));
    const panel = screen.getByRole("tabpanel");

    expect(within(panel).getByRole("combobox", { name: "demo 的角色" })).toBeTruthy();
    expect(within(panel).getByRole("combobox", { name: "要加入的專案" })).toBeTruthy();
    expect(within(panel).getByRole("combobox", { name: "加入時的角色" })).toBeTruthy();
    expect(document.querySelector(VISIBLE_NATIVE_SELECT)).toBeNull();
  });
});
