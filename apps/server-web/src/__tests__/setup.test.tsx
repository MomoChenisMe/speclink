import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { App } from "../App";
import { WebApiError } from "../api/client";

// Setup 開箱流程（server-setup「setup 流程完成開箱四要素」, D2／D3／D8 第三階段）。
// token 由 URL query 帶入；兩個提交節點（建立管理員、建立 Project／Repo）由 typed
// client 完成，最後節點回 `/admin?welcome=1` 與 connection，SPA 站內導向。

const UNAUTH = { authenticated: false, user: null, home: "/login" };

const STORE = {
  driver: "memory",
  contractVersion: 1,
  level: "full",
  capabilities: ["read", "write"],
  healthy: true,
  identitySchemaVersion: 1,
};

function makeClient(overrides: Record<string, unknown> = {}) {
  return {
    getSession: vi.fn(async () => UNAUTH),
    login: vi.fn(),
    logout: vi.fn(),
    getAdminOverview: vi.fn(),
    getSetupState: vi.fn(async () => ({ step: "admin", store: STORE })),
    submitSetupAdmin: vi.fn(async () => ({ step: "registry", store: STORE })),
    submitSetupRegistry: vi.fn(async () => ({
      destination: "/admin?welcome=1",
      connection: { publicUrl: "http://127.0.0.1", projectKey: "demo", repoKey: "backend" },
    })),
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

describe("setup 開箱流程", () => {
  it("token 有效時顯示建立管理員表單", async () => {
    renderAt("/setup?token=boot", makeClient());
    expect(await screen.findByRole("heading", { name: /初始設定|setup/i })).toBeTruthy();
    expect(screen.getByLabelText(/email|電子郵件/i)).toBeTruthy();
    expect(screen.getByLabelText(/顯示名稱|display/i)).toBeTruthy();
    expect(screen.getByLabelText(/密碼|password/i)).toBeTruthy();
    expect(screen.getByRole("button", { name: /建立管理員|下一步/ })).toBeTruthy();
  });

  it("建立管理員後進入 registry 步驟", async () => {
    const client = makeClient();
    renderAt("/setup?token=boot", client);
    const user = userEvent.setup();
    await user.type(await screen.findByLabelText(/email|電子郵件/i), "root@example.com");
    await user.type(screen.getByLabelText(/顯示名稱|display/i), "Root");
    await user.type(screen.getByLabelText(/密碼|password/i), "hunter2password");
    await user.click(screen.getByRole("button", { name: /建立管理員|下一步/ }));
    await waitFor(() => expect(client.submitSetupAdmin).toHaveBeenCalledOnce());
    expect(client.submitSetupAdmin).toHaveBeenCalledWith(
      "boot",
      expect.objectContaining({ email: "root@example.com", display: "Root", password: "hunter2password" }),
    );
    // The registry step's project/repo fields appear.
    expect(await screen.findByLabelText(/專案代號/)).toBeTruthy();
    expect(screen.getByLabelText(/儲存庫代號/)).toBeTruthy();
  });

  it("完成 registry 呼叫 submitSetupRegistry 並帶正確參數", async () => {
    const client = makeClient({ getSetupState: vi.fn(async () => ({ step: "registry", store: STORE })) });
    renderAt("/setup?token=boot", client);
    const user = userEvent.setup();
    await user.type(await screen.findByLabelText(/專案代號/), "demo");
    await user.type(screen.getByLabelText(/儲存庫代號/), "backend");
    await user.click(screen.getByRole("button", { name: /建立|完成/ }));
    await waitFor(() => expect(client.submitSetupRegistry).toHaveBeenCalledOnce());
    expect(client.submitSetupRegistry).toHaveBeenCalledWith(
      "boot",
      expect.objectContaining({ projectKey: "demo", repoKey: "backend" }),
    );
  });

  it("欄位驗證失敗以 role=alert 宣告且保留非祕密輸入", async () => {
    const client = makeClient({
      submitSetupAdmin: vi.fn(async () => {
        throw new WebApiError(400, "validation_error", "欄位有誤", { email: "email 格式不正確" });
      }),
    });
    renderAt("/setup?token=boot", client);
    const user = userEvent.setup();
    const email = await screen.findByLabelText(/email|電子郵件/i);
    await user.type(email, "bad");
    await user.type(screen.getByLabelText(/顯示名稱|display/i), "Root");
    await user.type(screen.getByLabelText(/密碼|password/i), "pw");
    await user.click(screen.getByRole("button", { name: /建立管理員|下一步/ }));
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("email 格式不正確");
    expect((email as HTMLInputElement).value).toBe("bad");
  });

  it("無效 token 顯示設定連結無效，不顯示表單", async () => {
    const client = makeClient({
      getSetupState: vi.fn(async () => {
        throw new WebApiError(401, "invalid_setup_token", "設定連結無效");
      }),
    });
    renderAt("/setup?token=bad", client);
    expect(await screen.findByText(/設定連結無效|無效/)).toBeTruthy();
    expect(screen.queryByLabelText(/顯示名稱|display/i)).toBeNull();
  });
});
