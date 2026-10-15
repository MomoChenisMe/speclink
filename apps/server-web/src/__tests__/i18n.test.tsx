import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MESSAGES, dictionaryProblems } from "@speclink/ui";
import { App } from "../App";
import { APP_MESSAGES } from "../i18n/messages";
import { makeAdminClient, setViewport } from "./helpers/adminHarness";

// spec 需求「介面語言支援中文與英文」：未設定偏好時跟隨瀏覽器語言（zh 開頭為中文，
// 其餘為英文）、側欄底部帳號選單提供三選切換、切換即時生效並在重新載入後維持、兩語言
// 訊息集合完全對應。語言選擇不影響 artifacts 產出語言、CLI 或伺服器回傳的字串。

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

describe("帳號選單的語言切換", () => {
  it("切為 English 後管理面改為英文，重新整理後維持", async () => {
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    const { unmount } = renderAdmin();
    await screen.findByRole("heading", { level: 1, name: "總覽" });

    await user.click(screen.getByRole("button", { name: /帳號選單/ }));
    await user.click(await screen.findByRole("menuitemradio", { name: "English" }));
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

    await user.click(screen.getByRole("button", { name: /Account menu/ }));
    await user.click(await screen.findByRole("menuitemradio", { name: "Follow system" }));
    expect(await screen.findByRole("heading", { level: 1, name: "總覽" })).toBeTruthy();
    expect(localStorage.getItem(LANGUAGE_KEY)).toBeNull();
  });
});

describe("兩種語言的訊息集合對應", () => {
  // 兩語言鍵集合（.one 除外）、.one 只在英文、計數文案用 {n} 並有 .one——與 @speclink/ui 共用同一個檢查。
  it("通過共用的字典檢查", () => {
    expect(dictionaryProblems(APP_MESSAGES)).toEqual([]);
  });

  // app 字典與 @speclink/ui 內建字典合併：同 key 同值的條目只是重複定義，換字時兩邊要一起改。
  it("不重複定義與 @speclink/ui 內建字典同 key 同值的條目", () => {
    for (const locale of ["zh-TW", "en"] as const) {
      const shadowed = Object.entries(APP_MESSAGES[locale])
        .filter(([key, value]) => MESSAGES[locale][key] === value)
        .map(([key]) => key);
      expect(shadowed, locale).toEqual([]);
    }
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
