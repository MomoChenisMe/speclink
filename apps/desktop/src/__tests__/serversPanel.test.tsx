// ServersPanel（規格「伺服器管理最小面」＋「device login 預設與 PAT fallback」
// 的前端面；client-browser-login D7、D9）：假 adapter 驅動真 store——新增→清單
// 即現、需要存取金鑰→PAT 輸入現身、登入成功→顯示身分、登出→回未登入、拒絕→
// 可讀狀態、等待瀏覽器登入或裝置核准→就地的等待面。等待的迴圈在 Rust；假
// adapter 以 onPhase 送出階段、以 loginCancel 結束等待。credential 全程不出現在
// 任何斷言面（TS 只見狀態與顯示名）。
import { describe, it, expect, vi } from "vitest";
import { cleanup, render as rtlRender, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { ServersPanel } from "../components/ServersPanel";
import { createAppStore, type AppState } from "../store";
import { APP_MESSAGES } from "../i18n/messages";
import type {
  ConnectionsAdapter,
  ConnectionView,
  LoginPhase,
  LoginResult,
} from "../adapter/connections";
import type { UseBoundStore, StoreApi } from "zustand";
import type { WorkspaceSession } from "../session";

const DISPLAY = "Dev <dev@example.com>";

/** 等待裝置核准的階段（Rust 送來的樣子）。 */
const APPROVAL: LoginPhase = {
  kind: "awaitingApproval",
  userCode: "ABCD-EFGH",
  verificationUri: "http://localhost:8080/activate",
  expiresAt: Date.now() + 900_000,
};

/** 等待瀏覽器登入的階段：授權網址只帶 state 與 challenge。 */
const BROWSER: LoginPhase = {
  kind: "awaitingBrowser",
  url: "https://id.example.com/authorize?response_type=code&client_id=speclink&state=s1&code_challenge=c1&code_challenge_method=S256",
  expiresAt: Date.now() + 300_000,
};

/** 停在等待面的登入：送出階段後一直等，直到 loginCancel 才以 cancelled 結束；
 * `finish` 讓測試代替使用者完成登入。 */
function waitingLogin(phase: LoginPhase) {
  let resolve: (result: LoginResult) => void = () => {};
  const loginStart = vi.fn(
    (_origin: string, onPhase: (p: LoginPhase) => void) =>
      new Promise<LoginResult>((r) => {
        resolve = r;
        onPhase(phase);
      }),
  );
  const loginCancel = vi.fn(async () => resolve({ status: "cancelled" }));
  return { loginStart, loginCancel, finish: (result: LoginResult) => resolve(result) };
}

const { writeText } = vi.hoisted(() => ({ writeText: vi.fn(async () => {}) }));
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({ writeText }));

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);

/** 假 adapter：in-memory registry＋可覆寫的登入行為。 */
function fakeAdapter(over: Partial<ConnectionsAdapter> = {}) {
  const entries: Array<Omit<ConnectionView, "loggedIn">> = [];
  const loggedIn = new Set<string>();
  const adapter: ConnectionsAdapter = {
    list: async () =>
      entries.map((e) => ({ ...e, loggedIn: loggedIn.has(e.origin) })),
    add: async (baseUrl, name) => {
      const origin = baseUrl.trim().replace(/\/+$/, "");
      const existing = entries.find((e) => e.origin === origin);
      if (existing) {
        existing.name = name;
        return { ...existing, loggedIn: loggedIn.has(origin) };
      }
      const entry = { id: `conn_${entries.length + 1}`, origin, name };
      entries.push(entry);
      return { ...entry, loggedIn: false };
    },
    remove: async (id) => {
      const idx = entries.findIndex((e) => e.id === id);
      if (idx >= 0) {
        loggedIn.delete(entries[idx].origin);
        entries.splice(idx, 1);
      }
    },
    loginStart: async (origin): Promise<LoginResult> => {
      loggedIn.add(origin);
      const entry = entries.find((e) => e.origin === origin);
      if (entry) entry.lastActorDisplay = DISPLAY;
      return { status: "loggedIn", display: DISPLAY };
    },
    loginCancel: async () => {},
    patLogin: async (origin, pat) => {
      if (pat !== "spk_pat_good") throw new Error("PAT 無效或已被撤銷");
      loggedIn.add(origin);
      const entry = entries.find((e) => e.origin === origin);
      if (entry) entry.lastActorDisplay = DISPLAY;
      return { status: "loggedIn", display: DISPLAY };
    },
    logout: async (origin) => {
      loggedIn.delete(origin);
      const entry = entries.find((e) => e.origin === origin);
      if (entry) entry.lastActorDisplay = null;
      return { revokedOnServer: true, patNotice: false };
    },
    scopes: async () => ({ projects: [] }),
    inspectCheckout: async (path) => ({ root: path, tools: [] }),
    bindCheckout: async (path) => path,
    ...over,
  };
  return adapter;
}

/** 真 store＋面板：面板 props 全數接線 store 的 connections 分片。 */
function Harness({
  useStore,
  onOpenWorkspace,
}: {
  useStore: UseBoundStore<StoreApi<AppState>>;
  onOpenWorkspace?: (id: string) => void;
}) {
  const s = useStore();
  return (
    <ServersPanel
      connections={s.connections}
      phases={s.connectionPhases}
      onAdd={s.addConnection}
      onLogin={s.loginConnection}
      onCancelLogin={s.cancelLogin}
      onSubmitPat={s.submitPat}
      onLogout={s.logoutConnection}
      onRemove={s.removeConnection}
      onRefresh={s.refreshConnections}
      onOpenWorkspace={onOpenWorkspace}
      // 伺服器頁的新增卡由頁標題「新增連線」展開；此處直接展開以走新增表單。
      addOpen
    />
  );
}

/** 連線列：狀態籤「已登入」與身分顯示名（desktop-settings-mode design D7）。 */
async function expectLoggedInAs(display: string) {
  await waitFor(() => expect(screen.getByText(display)).toBeTruthy());
  expect(screen.getByText("已登入")).toBeTruthy();
}

/** 連線列的「⋯」選單動作。 */
async function rowMenu(item: string) {
  const user = userEvent.setup({ pointerEventsCheck: 0 });
  await user.click(screen.getByRole("button", { name: "更多動作" }));
  await user.click(await screen.findByRole("menuitem", { name: item }));
}

function renderPanel(adapter: ConnectionsAdapter) {
  const useStore = createAppStore({
    createSession: () => ({}) as WorkspaceSession,
    connections: adapter,
  });
  rtlRender(<Harness useStore={useStore} />, { wrapper: zhWrapper });
  return useStore;
}

/** 走新增表單：填 URL＋顯示名、按新增。 */
function addServer(url: string, name: string) {
  fireEvent.change(screen.getByPlaceholderText("伺服器位址（http://…）"), {
    target: { value: url },
  });
  fireEvent.change(screen.getByPlaceholderText("顯示名"), { target: { value: name } });
  fireEvent.click(screen.getByRole("button", { name: "新增" }));
}

describe("ServersPanel", () => {
  it("新增後清單即時反映並進入登入流程，登入成功顯示身分", async () => {
    renderPanel(fakeAdapter());
    addServer("http://localhost:8080/", "本地");

    // 清單即現條目（顯示名＋正規化 origin）。
    await waitFor(() => expect(screen.getByText("本地")).toBeTruthy());
    expect(screen.getByText("http://localhost:8080")).toBeTruthy();

    // 隨即登入成功：呈現已登入與身分顯示名。
    await expectLoggedInAs(DISPLAY);
  });

  it("server 只收存取金鑰（或沒有 device flow）時就地現 PAT 輸入，有效 PAT 完成登入", async () => {
    const adapter = fakeAdapter();
    renderPanel({ ...adapter, loginStart: async () => ({ status: "patRequired" }) });
    addServer("http://legacy.example", "舊機");

    // 需要存取金鑰 → PAT 輸入直接現身，沒有任何等待面。
    const patInput = await screen.findByPlaceholderText("spk_pat_…");
    expect(screen.queryByTestId("awaiting-approval-http://legacy.example")).toBeNull();
    expect(screen.queryByTestId("awaiting-browser-http://legacy.example")).toBeNull();
    fireEvent.change(patInput, { target: { value: "spk_pat_good" } });
    fireEvent.click(screen.getByRole("button", { name: "以 PAT 登入" }));

    await expectLoggedInAs(DISPLAY);
  });

  it("無效 PAT 拒絕並留在輸入面", async () => {
    const adapter = fakeAdapter();
    renderPanel({
      ...adapter,
      loginStart: async () => ({ status: "patRequired" }),
    });
    addServer("http://legacy.example", "舊機");

    const patInput = await screen.findByPlaceholderText("spk_pat_…");
    fireEvent.change(patInput, { target: { value: "spk_pat_bad" } });
    fireEvent.click(screen.getByRole("button", { name: "以 PAT 登入" }));

    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain("PAT 無效或已被撤銷"),
    );
    // 輸入面仍在，可重試。
    expect(screen.getByPlaceholderText("spk_pat_…")).toBeTruthy();
  });

  it("登出後回未登入", async () => {
    renderPanel(fakeAdapter());
    addServer("http://localhost:8080", "本地");
    await expectLoggedInAs(DISPLAY);

    fireEvent.click(screen.getByRole("button", { name: "登出" }));
    await waitFor(() => expect(screen.getByText("未登入")).toBeTruthy());
    expect(screen.queryByText(DISPLAY)).toBeNull();
  });

  it("瀏覽器拒絕授權呈現可讀狀態", async () => {
    const adapter = fakeAdapter();
    renderPanel({
      ...adapter,
      loginStart: async (_origin, onPhase) => {
        onPhase(APPROVAL);
        return { status: "denied", method: "device" };
      },
    });
    addServer("http://localhost:8080", "本地");

    await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("已在瀏覽器拒絕授權"));
    // 可讀狀態取代狀態行；絕不呈現已登入。
    expect(screen.queryByText(/已登入/)).toBeNull();
  });

  it("探索失敗顯示連線錯誤，不出現 PAT 輸入", async () => {
    const adapter = fakeAdapter();
    renderPanel({
      ...adapter,
      loginStart: async () => {
        throw new Error("server unavailable — check the connection url");
      },
    });
    addServer("http://localhost:8080", "本地");

    await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("server unavailable"));
    expect(screen.queryByPlaceholderText("spk_pat_…")).toBeNull();
  });

  // --- 等待授權面（規格「device login 預設與 PAT fallback」的等待授權要求） ---

  /** 停在等待面的面板：登入送出 `phase` 後一直等，直到取消或測試代為完成。 */
  function renderAwaiting(phase: LoginPhase = APPROVAL) {
    const adapter = fakeAdapter();
    const login = waitingLogin(phase);
    const useStore = createAppStore({
      createSession: () => ({}) as WorkspaceSession,
      connections: { ...adapter, loginStart: login.loginStart, loginCancel: login.loginCancel },
    });
    rtlRender(<Harness useStore={useStore} />, { wrapper: zhWrapper });
    addServer("http://localhost:8080", "本地");
    return login;
  }

  it("等待授權時顯示裝置碼、驗證網址、倒數與取消", async () => {
    renderAwaiting();
    const waiting = await screen.findByTestId("awaiting-approval-http://localhost:8080");
    expect(waiting.textContent).toContain("ABCD-EFGH");
    expect(waiting.textContent).toContain("http://localhost:8080/activate");
    // 倒數（分:秒）與取消操作就在同一面。
    expect(waiting.textContent).toMatch(/\d+:\d{2}/);
    expect(screen.getByRole("button", { name: "取消登入" })).toBeTruthy();
  });

  it("裝置碼與驗證網址各附複製，複製後剪貼簿即為該內容", async () => {
    writeText.mockClear();
    renderAwaiting();
    await screen.findByTestId("awaiting-approval-http://localhost:8080");

    fireEvent.click(screen.getByRole("button", { name: "複製裝置碼" }));
    await waitFor(() => expect(writeText).toHaveBeenCalledWith("ABCD-EFGH"));
    fireEvent.click(screen.getByRole("button", { name: "複製驗證網址" }));
    await waitFor(() => expect(writeText).toHaveBeenCalledWith("http://localhost:8080/activate"));
  });

  it("取消等待即呼叫取消並回未登入", async () => {
    const login = renderAwaiting();
    await screen.findByTestId("awaiting-approval-http://localhost:8080");

    fireEvent.click(screen.getByRole("button", { name: "取消登入" }));
    await waitFor(() => expect(screen.getByText("未登入")).toBeTruthy());
    expect(login.loginCancel).toHaveBeenCalledWith("http://localhost:8080");
    expect(screen.queryByTestId("awaiting-approval-http://localhost:8080")).toBeNull();
  });

  // --- 等待瀏覽器登入面（client-browser-login D9） ---

  it("等待瀏覽器登入時就地顯示說明、可複製的授權網址、倒數與取消，沒有裝置碼", async () => {
    writeText.mockClear();
    renderAwaiting(BROWSER);
    const waiting = await screen.findByTestId("awaiting-browser-http://localhost:8080");
    expect(waiting.textContent).toContain("等待瀏覽器登入");
    expect(waiting.textContent).toContain(BROWSER.kind === "awaitingBrowser" ? BROWSER.url : "");
    expect(waiting.textContent).toMatch(/\d+:\d{2}/);
    expect(waiting.textContent).not.toContain("裝置碼");
    expect(screen.queryByTestId("awaiting-approval-http://localhost:8080")).toBeNull();
    // 狀態籤是等待授權。
    expect(screen.getByText("等待授權")).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: "複製登入網址" }));
    await waitFor(() =>
      expect(writeText).toHaveBeenCalledWith(BROWSER.kind === "awaitingBrowser" ? BROWSER.url : ""),
    );
  });

  it("等待瀏覽器登入時取消即呼叫取消並回未登入", async () => {
    const login = renderAwaiting(BROWSER);
    await screen.findByTestId("awaiting-browser-http://localhost:8080");

    fireEvent.click(screen.getByRole("button", { name: "取消登入" }));
    await waitFor(() => expect(screen.getByText("未登入")).toBeTruthy());
    expect(login.loginCancel).toHaveBeenCalledWith("http://localhost:8080");
    expect(screen.queryByTestId("awaiting-browser-http://localhost:8080")).toBeNull();
  });

  it("瀏覽器登入被拒絕與逾時各有可讀狀態", async () => {
    for (const [status, message] of [
      ["denied", "已在登入服務拒絕登入"],
      ["expired", "瀏覽器登入逾時"],
    ] as const) {
      const login = renderAwaiting(BROWSER);
      await screen.findByTestId("awaiting-browser-http://localhost:8080");
      login.finish({ status, method: "browser" });
      await waitFor(() => expect(screen.getByRole("alert").textContent).toContain(message));
      expect(screen.queryByTestId("awaiting-browser-http://localhost:8080")).toBeNull();
      cleanup();
    }
  });

  // --- 登入成功的行動呼籲（規格「伺服器管理最小面」：聚焦開啟工作區入口） ---

  it("登入成功後開啟工作區入口取得鍵盤焦點，且不自動開啟工作區選擇器——三種方式都是", async () => {
    for (const method of ["browser", "device", "pat"] as const) {
      const open = vi.fn();
      const adapter = fakeAdapter();
      const loginStart: ConnectionsAdapter["loginStart"] =
        method === "pat"
          ? async () => ({ status: "patRequired" })
          : async (origin, onPhase) => {
              onPhase(method === "browser" ? BROWSER : APPROVAL);
              const done = await adapter.loginStart(origin, onPhase);
              return done;
            };
      const useStore = createAppStore({
        createSession: () => ({}) as WorkspaceSession,
        connections: { ...adapter, loginStart },
      });
      rtlRender(<Harness useStore={useStore} onOpenWorkspace={open} />, { wrapper: zhWrapper });
      addServer("http://localhost:8080", "本地");
      if (method === "pat") {
        fireEvent.change(await screen.findByPlaceholderText("spk_pat_…"), { target: { value: "spk_pat_good" } });
        fireEvent.click(screen.getByRole("button", { name: "以 PAT 登入" }));
      }

      const entry = await screen.findByTestId("open-workspace-http://localhost:8080");
      await waitFor(() => expect(document.activeElement).toBe(entry), { timeout: 2000 });
      expect(open).not.toHaveBeenCalled();
      cleanup();
    }
  });

  it("移除連線後清單消失", async () => {
    renderPanel(fakeAdapter());
    addServer("http://localhost:8080", "本地");
    await waitFor(() => expect(screen.getByText("本地")).toBeTruthy());

    await rowMenu("移除");
    await waitFor(() => expect(screen.queryByText("本地")).toBeNull());
    expect(screen.getByTestId("servers-empty")).toBeTruthy();
  });

  // --- 開啟 workspace：退役文字輸入，匯流至統一 chooser ---

  function renderLoggedInPanel(onOpenWorkspace: (id: string) => void) {
    rtlRender(
      <ServersPanel
        connections={[
          {
            id: "conn_1",
            origin: "http://localhost:8080",
            name: "本地",
            lastActorDisplay: DISPLAY,
            loggedIn: true,
          },
        ]}
        phases={{}}
        onAdd={async () => {}}
        onLogin={() => {}}
        onSubmitPat={() => {}}
        onLogout={() => {}}
        onRemove={() => {}}
        onOpenWorkspace={onOpenWorkspace}
      />,
      { wrapper: zhWrapper },
    );
  }

  // 既有已登入條目的開啟專案在「⋯」選單；行動呼籲按鈕只給剛完成的登入。
  const openWorkspaceChooser = () => rowMenu("開啟專案");

  it("已登入條目開啟統一 chooser 並預選該 connection", async () => {
    const open = vi.fn();
    renderLoggedInPanel(open);

    await openWorkspaceChooser();
    expect(open).toHaveBeenCalledWith("conn_1");
    expect(screen.queryByPlaceholderText("project 或 project/repo")).toBeNull();
  });

  it("不再呈現臨時 repo 文字輸入表單", async () => {
    const open = vi.fn();
    renderLoggedInPanel(open);

    await openWorkspaceChooser();
    expect(screen.queryByTestId("workspace-form-http://localhost:8080")).toBeNull();
    expect(screen.queryByPlaceholderText("project 或 project/repo")).toBeNull();
  });
});
