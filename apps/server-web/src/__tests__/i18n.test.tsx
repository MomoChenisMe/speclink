import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { App } from "../App";
import { APP_MESSAGES } from "../i18n/messages";
import { makeAdminClient, setViewport } from "./helpers/adminHarness";

// spec 需求「介面語言支援中文與英文」：未設定偏好時跟隨瀏覽器語言（zh 開頭為中文，
// 其餘為英文）、header 提供三選切換、切換即時生效並在重新載入後維持、兩語言訊息集合
// 完全對應。語言選擇不影響 artifacts 產出語言、CLI 或伺服器回傳的字串。

const LANGUAGE_KEY = "speclink.uiLocale";

function setBrowserLanguage(language: string) {
  Object.defineProperty(navigator, "language", { value: language, configurable: true });
}

beforeEach(() => {
  setViewport(false);
  localStorage.clear();
  localStorage.setItem("speclink.tourSeen", "1");
});

afterEach(() => setBrowserLanguage("zh-TW"));

function renderAdmin() {
  return render(<App client={makeAdminClient() as never} initialEntries={["/admin"]} />);
}

describe("未設定偏好時跟隨瀏覽器語言", () => {
  it("瀏覽器為 en-US 時管理面為英文", async () => {
    setBrowserLanguage("en-US");
    renderAdmin();
    expect(await screen.findByRole("heading", { level: 1, name: "Overview" })).toBeTruthy();
    expect(screen.getByRole("link", { name: "Users" })).toBeTruthy();
    expect(screen.queryByText("總覽")).toBeNull();
  });

  it("瀏覽器為 zh-TW 時管理面為中文", async () => {
    setBrowserLanguage("zh-TW");
    renderAdmin();
    expect(await screen.findByRole("heading", { level: 1, name: "總覽" })).toBeTruthy();
    expect(screen.queryByText("Overview")).toBeNull();
  });

  it("瀏覽器語言未知時退回英文", async () => {
    setBrowserLanguage("de-DE");
    renderAdmin();
    expect(await screen.findByRole("heading", { level: 1, name: "Overview" })).toBeTruthy();
  });
});

describe("header 的語言切換", () => {
  it("切為 English 後管理面改為英文，重新整理後維持", async () => {
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    const { unmount } = renderAdmin();
    await screen.findByRole("heading", { level: 1, name: "總覽" });

    await user.click(screen.getByRole("combobox", { name: /介面語言/ }));
    await user.click(await screen.findByRole("option", { name: "English" }));
    expect(await screen.findByRole("heading", { level: 1, name: "Overview" })).toBeTruthy();
    expect(localStorage.getItem(LANGUAGE_KEY)).toBe("en");

    // 重新整理：偏好留在瀏覽器，介面仍是英文（瀏覽器語言仍為 zh-TW，明示偏好優先）。
    unmount();
    renderAdmin();
    expect(await screen.findByRole("heading", { level: 1, name: "Overview" })).toBeTruthy();
  });

  it("切回跟隨系統後清除偏好並回到瀏覽器語言", async () => {
    localStorage.setItem(LANGUAGE_KEY, "en");
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    renderAdmin();
    await screen.findByRole("heading", { level: 1, name: "Overview" });

    await user.click(screen.getByRole("combobox", { name: /Interface language/ }));
    await user.click(await screen.findByRole("option", { name: "Follow system" }));
    expect(await screen.findByRole("heading", { level: 1, name: "總覽" })).toBeTruthy();
    expect(localStorage.getItem(LANGUAGE_KEY)).toBeNull();
  });
});

describe("兩種語言的訊息集合對應", () => {
  it("zh-TW 與 en 的 key 集合完全相等", () => {
    const zh = Object.keys(APP_MESSAGES["zh-TW"]).sort();
    const en = Object.keys(APP_MESSAGES.en).sort();
    expect(en).toEqual(zh);
  });

  it("每個 key 在兩種語言都有非空字串", () => {
    for (const locale of ["zh-TW", "en"] as const) {
      for (const [key, value] of Object.entries(APP_MESSAGES[locale])) {
        expect(typeof value, `${locale}/${key} 應為字串`).toBe("string");
        expect(value.trim().length, `${locale}/${key} 不應為空`).toBeGreaterThan(0);
      }
    }
  });
});
