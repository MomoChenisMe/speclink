import { describe, it, expect, beforeEach } from "vitest";
import { screen, within } from "@testing-library/react";
import { makeAdminClient, renderAt, setViewport } from "./helpers/adminHarness";

// spec 需求「共用設計系統維持高密度可存取體驗」的 Scenario「工具列控件高度一致」：
// 同一組工具列內的文字輸入、下拉與日期輸入高度相同，且下拉是套用 theme 的 Select
// 原語而非原生 <select>。高度以 class 上的 h-* 比對——參差的成因就是兩套原語的預設
// 高度不同（Input h-9、原生 select h-8），所以直接釘住「三者相同」而非釘住某個數值。

beforeEach(() => setViewport(false));

// Radix Select 在 <form> 內會額外渲染一個 aria-hidden 的原生 select 供表單送出使用。
// 那不是使用者看得到或操作得到的控件，所以「不得有原生 select」指的是可見的那一種。
const VISIBLE_NATIVE_SELECT = "select:not([aria-hidden='true'])";

/** 該元素 class 上的 tailwind 高度 utility（例如 h-9）。 */
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
  it("稽核頁的搜尋輸入、下拉篩選與日期輸入高度相同", async () => {
    renderAt("/admin/audit", makeAdminClient());
    const search = await screen.findByLabelText("搜尋");
    const action = screen.getByRole("combobox", { name: /動作/ });
    const from = screen.getByLabelText("起");

    const h = heightClass(search);
    expect(h, "搜尋輸入應帶明確的高度 class").toBeTruthy();
    expect(heightClass(action)).toBe(h);
    expect(heightClass(from)).toBe(h);
  });

  it("使用者頁的搜尋輸入與狀態下拉高度相同", async () => {
    renderAt("/admin/users", makeAdminClient());
    const search = await screen.findByLabelText("搜尋");
    const status = screen.getByRole("combobox", { name: /狀態/ });
    expect(heightClass(status)).toBe(heightClass(search));
  });
});

describe("使用者抽屜的成員資格表單使用 Select 原語", () => {
  it("加入專案的專案與角色選擇皆為 combobox", async () => {
    const { default: userEvent } = await import("@testing-library/user-event");
    const user = userEvent.setup();
    renderAt("/admin/users", makeAdminClient());

    const table = await screen.findByRole("table");
    await user.click(within(table).getByRole("row", { name: /member@example\.com/ }));
    const drawer = await screen.findByRole("dialog", { name: /Member/ });
    await user.click(within(drawer).getByRole("tab", { name: "成員資格" }));
    await user.click(within(drawer).getByRole("button", { name: /加入專案/ }));

    expect(within(drawer).getByRole("combobox", { name: /專案/ })).toBeTruthy();
    expect(within(drawer).getByRole("combobox", { name: /角色/ })).toBeTruthy();
    expect(document.querySelector(VISIBLE_NATIVE_SELECT)).toBeNull();
  });
});
