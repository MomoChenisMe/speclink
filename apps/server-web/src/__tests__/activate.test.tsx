import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { App } from "../App";

// 裝置核准頁（server-device-auth「核准頁 session 保護且明確確認」, D3）。需已登入；
// GET 不查授權狀態，只預填 URL 帶入的裝置碼。使用者提交後才得到明確的核准／拒絕確認
// 步驟；未登入導向登入頁並保留裝置碼。

const MEMBER = {
  authenticated: true,
  user: { id: "u1", email: "member@example.com", display: "Member", admin: false },
  home: "/account",
};
const UNAUTH = { authenticated: false, user: null, home: "/login" };

function makeClient(overrides: Record<string, unknown> = {}) {
  return {
    getSession: vi.fn(async () => MEMBER),
    login: vi.fn(),
    logout: vi.fn(),
    getAdminOverview: vi.fn(),
    checkActivation: vi.fn(async () => ({ status: "pending" })),
    decideActivation: vi.fn(async () => ({ status: "approved" })),
    ...overrides,
  };
}

function renderAt(route: string, client: ReturnType<typeof makeClient>) {
  return render(<App client={client as never} initialEntries={[route]} />);
}

beforeEach(() => {
  window.matchMedia = ((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: () => {},
    removeEventListener: () => {},
    addListener: () => {},
    removeListener: () => {},
    dispatchEvent: () => false,
  })) as unknown as typeof window.matchMedia;
});

describe("裝置核准頁", () => {
  it("預填 URL 帶入的裝置碼", async () => {
    renderAt("/activate?user_code=ABCD-EFGH", makeClient());
    expect(await screen.findByDisplayValue("ABCD-EFGH")).toBeTruthy();
  });

  it("提交後才顯示明確的核准／拒絕選項", async () => {
    const client = makeClient();
    renderAt("/activate?user_code=ABCD-EFGH", client);
    const user = userEvent.setup();
    // 核准／拒絕在確認步驟前不顯示。
    await screen.findByDisplayValue("ABCD-EFGH");
    expect(screen.queryByRole("button", { name: /核准/ })).toBeNull();
    // 提交下一步：檢查該碼（不改狀態），顯示核准／拒絕。
    await user.click(screen.getByRole("button", { name: /下一步|確認/ }));
    await waitFor(() => expect(client.checkActivation).toHaveBeenCalledWith("ABCD-EFGH"));
    expect(await screen.findByRole("button", { name: /核准/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /拒絕/ })).toBeTruthy();
  });

  it("核准呼叫 decideActivation", async () => {
    const client = makeClient();
    renderAt("/activate?user_code=ABCD-EFGH", client);
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: /下一步|確認/ }));
    await user.click(await screen.findByRole("button", { name: /核准/ }));
    await waitFor(() => expect(client.decideActivation).toHaveBeenCalledWith("ABCD-EFGH", "approve"));
  });

  // 結果頁指引（server-device-auth「核准頁 session 保護且明確確認」）：核准與拒絕
  // 兩種結果都要告知可返回 Speclink app 繼續，不止於單行結果。
  it.each([
    ["approve" as const, "approved" as const, /核准/],
    ["deny" as const, "denied" as const, /拒絕/],
  ])("%s 的結果頁指引返回 app 繼續", async (action, status, label) => {
    const client = makeClient({ decideActivation: vi.fn(async () => ({ status })) });
    renderAt("/activate?user_code=ABCD-EFGH", client);
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: /下一步|確認/ }));
    await user.click(await screen.findByRole("button", { name: label }));
    await waitFor(() => expect(client.decideActivation).toHaveBeenCalledWith("ABCD-EFGH", action));
    expect(await screen.findByText(/回到 Speclink/)).toBeTruthy();
  });

  it("未登入導向登入頁", async () => {
    renderAt("/activate?user_code=ABCD-EFGH", makeClient({ getSession: vi.fn(async () => UNAUTH) }));
    expect(await screen.findByRole("button", { name: /登入/ })).toBeTruthy();
  });
});
