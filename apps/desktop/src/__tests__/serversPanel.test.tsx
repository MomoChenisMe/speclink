// ServersPanel（規格「伺服器管理最小面」＋「device login 預設與 PAT fallback」
// 的前端面；design 決策 7）：假 adapter 驅動真 store——新增→清單即現、探測
// 不支援→PAT 輸入現身、登入成功→顯示身分、登出→回未登入、device 拒絕→可讀
// 狀態。credential 全程不出現在任何斷言面（TS 只見狀態與顯示名）。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { ServersPanel } from "../components/ServersPanel";
import { createAppStore, type AppState } from "../store";
import { APP_MESSAGES } from "../i18n/messages";
import type {
  ConnectionsAdapter,
  ConnectionView,
  DeviceLoginResult,
} from "../adapter/connections";
import type { UseBoundStore, StoreApi } from "zustand";
import type { WorkspaceSession } from "../session";

const DISPLAY = "Dev <dev@example.com>";

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
    deviceLogin: async (origin): Promise<DeviceLoginResult> => {
      loggedIn.add(origin);
      const entry = entries.find((e) => e.origin === origin);
      if (entry) entry.lastActorDisplay = DISPLAY;
      return { status: "loggedIn", display: DISPLAY };
    },
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
    bindCheckout: async (path) => path,
    ...over,
  };
  return adapter;
}

/** 真 store＋面板：面板 props 全數接線 store 的 connections 分片。 */
function Harness({ useStore }: { useStore: UseBoundStore<StoreApi<AppState>> }) {
  const s = useStore();
  return (
    <ServersPanel
      connections={s.connections}
      phases={s.connectionPhases}
      onAdd={s.addConnection}
      onLogin={s.loginConnection}
      onSubmitPat={s.submitPat}
      onLogout={s.logoutConnection}
      onRemove={s.removeConnection}
      onRefresh={s.refreshConnections}
    />
  );
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
    await waitFor(() => expect(screen.getByText(`已登入 · ${DISPLAY}`)).toBeTruthy());
  });

  it("探測不支援時就地現 PAT 輸入，有效 PAT 完成登入", async () => {
    const adapter = fakeAdapter();
    const patFallbackOnce = vi
      .fn<() => Promise<DeviceLoginResult>>()
      .mockResolvedValue({ status: "unsupported" });
    renderPanel({ ...adapter, deviceLogin: () => patFallbackOnce() });
    addServer("http://legacy.example", "舊機");

    // 明確不支援 → PAT 輸入現身（fallback 只給這個訊號，不是錯誤）。
    const patInput = await screen.findByPlaceholderText("spk_pat_…");
    fireEvent.change(patInput, { target: { value: "spk_pat_good" } });
    fireEvent.click(screen.getByRole("button", { name: "以 PAT 登入" }));

    await waitFor(() => expect(screen.getByText(`已登入 · ${DISPLAY}`)).toBeTruthy());
  });

  it("無效 PAT 拒絕並留在輸入面", async () => {
    const adapter = fakeAdapter();
    renderPanel({
      ...adapter,
      deviceLogin: async () => ({ status: "unsupported" }),
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
    await waitFor(() => expect(screen.getByText(`已登入 · ${DISPLAY}`)).toBeTruthy());

    fireEvent.click(screen.getByRole("button", { name: "登出" }));
    await waitFor(() => expect(screen.getByText("未登入")).toBeTruthy());
    expect(screen.queryByText(`已登入 · ${DISPLAY}`)).toBeNull();
  });

  it("瀏覽器拒絕授權呈現可讀狀態", async () => {
    const adapter = fakeAdapter();
    renderPanel({ ...adapter, deviceLogin: async () => ({ status: "denied" }) });
    addServer("http://localhost:8080", "本地");

    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain("已在瀏覽器拒絕授權"),
    );
    // 可讀狀態取代狀態行；絕不呈現已登入。
    expect(screen.queryByText(/已登入/)).toBeNull();
  });

  it("移除連線後清單消失", async () => {
    renderPanel(fakeAdapter());
    addServer("http://localhost:8080", "本地");
    await waitFor(() => expect(screen.getByText("本地")).toBeTruthy());

    fireEvent.click(screen.getByRole("button", { name: "移除" }));
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

  function openWorkspaceChooser() {
    fireEvent.click(screen.getByRole("button", { name: "開啟 workspace" }));
  }

  it("已登入條目開啟統一 chooser 並預選該 connection", () => {
    const open = vi.fn();
    renderLoggedInPanel(open);

    openWorkspaceChooser();
    expect(open).toHaveBeenCalledWith("conn_1");
    expect(screen.queryByPlaceholderText("project 或 project/repo")).toBeNull();
  });

  it("不再呈現臨時 repo 文字輸入表單", () => {
    const open = vi.fn();
    renderLoggedInPanel(open);

    openWorkspaceChooser();
    expect(screen.queryByTestId("workspace-form-http://localhost:8080")).toBeNull();
    expect(screen.queryByPlaceholderText("project 或 project/repo")).toBeNull();
  });
});
