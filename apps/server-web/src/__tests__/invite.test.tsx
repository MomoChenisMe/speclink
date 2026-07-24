import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { App } from "../App";
import { WebApiError } from "../api/client";

// 邀請接受流程（server-identity「邀請一次性且到期失效」, D2／D3）。token 由 URL path
// 帶入；有效邀請顯示設定密碼表單，提交後由 typed client 建帳號並自動登入，SPA 依
// Server 回傳的 destination 導向；無效邀請顯示不可區分的「邀請無效」且無表單。

const UNAUTH = { authenticated: false, user: null, home: "/login" };

function makeClient(overrides: Record<string, unknown> = {}) {
  return {
    getSession: vi.fn(async () => UNAUTH),
    login: vi.fn(),
    logout: vi.fn(),
    getAdminOverview: vi.fn(),
    getInvitation: vi.fn(async () => ({ email: "invitee@example.com", display: "Invitee", admin: false })),
    acceptInvitation: vi.fn(async () => ({ destination: "/account" })),
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

describe("邀請接受流程", () => {
  it("有效邀請顯示 email 與設定密碼表單", async () => {
    renderAt("/invite/tok123", makeClient());
    expect(await screen.findByText(/invitee@example.com/)).toBeTruthy();
    expect(screen.getByLabelText(/密碼|password/i)).toBeTruthy();
    expect(screen.getByRole("button", { name: /建立帳號|接受邀請|設定密碼/ })).toBeTruthy();
  });

  it("提交密碼呼叫 acceptInvitation 並帶 token 與密碼", async () => {
    const acceptInvitation = vi.fn(async () => ({ destination: "/account" }));
    const client = makeClient({ acceptInvitation });
    renderAt("/invite/tok123", client);
    const user = userEvent.setup();
    await user.type(await screen.findByLabelText(/密碼|password/i), "hunter2password");
    await user.click(screen.getByRole("button", { name: /建立帳號|接受邀請|設定密碼/ }));
    await waitFor(() => expect(acceptInvitation).toHaveBeenCalledOnce());
    expect(acceptInvitation).toHaveBeenCalledWith(
      "tok123",
      expect.objectContaining({ password: "hunter2password" }),
    );
  });

  it("無效邀請顯示邀請無效，且無設定密碼表單", async () => {
    const client = makeClient({
      getInvitation: vi.fn(async () => {
        throw new WebApiError(404, "invalid_invitation", "邀請無效");
      }),
    });
    renderAt("/invite/bad", client);
    expect(await screen.findByText(/邀請無效/)).toBeTruthy();
    expect(screen.queryByLabelText(/密碼|password/i)).toBeNull();
  });

  it("接受失敗的欄位錯誤以 role=alert 宣告", async () => {
    const client = makeClient({
      acceptInvitation: vi.fn(async () => {
        throw new WebApiError(400, "validation_error", "密碼太短", { password: "密碼至少 8 碼" });
      }),
    });
    renderAt("/invite/tok123", client);
    const user = userEvent.setup();
    await user.type(await screen.findByLabelText(/密碼|password/i), "x");
    await user.click(screen.getByRole("button", { name: /建立帳號|接受邀請|設定密碼/ }));
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("密碼至少 8 碼");
  });
});
