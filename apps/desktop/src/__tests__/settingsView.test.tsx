// SettingsView（spec 需求「設定頁圖形化讀寫兩層設定」「設定寫入具解析驗證且
// 失敗浮出」的前端面＋design D5/D8/D9）。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, waitFor, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { SettingsView } from "../views/SettingsView";
import { APP_MESSAGES } from "../i18n/messages";
import type { SettingsSnapshot, WorkspaceAdapter } from "../adapter/workspace";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

function snapshot(over: Partial<SettingsSnapshot> = {}): SettingsSnapshot {
  return {
    app: { tools: ["claude"], customTools: [], parseError: null, ...(over.app ?? {}) },
    workflow: {
      locale: "tw",
      specLocale: null,
      tdd: true,
      audit: false,
      parseError: null,
      ...(over.workflow ?? {}),
    },
  };
}

function fakeWorkspace(snap: SettingsSnapshot): WorkspaceAdapter {
  return {
    openProject: vi.fn(),
    initProject: vi.fn(),
    currentProject: vi.fn(),
    projectStats: vi.fn(),
    pickFolder: vi.fn(),
    readSettings: vi.fn().mockResolvedValue(snap),
    writeAppTools: vi.fn().mockResolvedValue(undefined),
    writeWorkflowConfig: vi.fn().mockResolvedValue(undefined),
  } as unknown as WorkspaceAdapter;
}

function renderView(
  snap: SettingsSnapshot,
  over: { onLocalePrefChange?: (p: "zh-TW" | "en" | null) => void } = {},
) {
  const ws = fakeWorkspace(snap);
  render(
    <SettingsView
      workspace={ws}
      localePref={null}
      onLocalePrefChange={over.onLocalePrefChange ?? vi.fn()}
    />,
  );
  return ws;
}

describe("SettingsView 載入", () => {
  it("呈現兩檔現值：tools 勾選、locale 現值、tdd 開關；未設定欄位呈預設狀態", async () => {
    renderView(snapshot());
    const claude = (await screen.findByLabelText("claude")) as HTMLInputElement;
    expect(claude.checked).toBe(true);
    expect((screen.getByLabelText("codex") as HTMLInputElement).checked).toBe(false);
    const locale = screen.getByLabelText("locale") as HTMLSelectElement;
    expect(locale.value).toBe("tw");
    // 未設定的 spec_locale 呈預設值狀態（空字串＝未設定）。
    expect((screen.getByLabelText("spec_locale") as HTMLSelectElement).value).toBe("");
    expect((screen.getByLabelText("tdd") as HTMLInputElement).checked).toBe(true);
    expect((screen.getByLabelText("audit") as HTMLInputElement).checked).toBe(false);
  });

  it("自訂工具描述子呈現為不可編輯項", async () => {
    renderView(snapshot({ app: { tools: ["claude"], customTools: ["wad-harness"], parseError: null } }));
    const custom = await screen.findByText("wad-harness");
    // 不可編輯：非表單輸入（無對應 checkbox）。
    expect(screen.queryByLabelText("wad-harness")).toBeNull();
    expect(custom).toBeTruthy();
  });
});

describe("SettingsView 寫入", () => {
  it("tools 加選 codex 後儲存 → writeAppTools 收到完整選集", async () => {
    const ws = renderView(snapshot());
    fireEvent.click(await screen.findByLabelText("codex"));
    fireEvent.click(screen.getByTestId("save-app"));
    await waitFor(() => expect(ws.writeAppTools).toHaveBeenCalledWith(["claude", "codex"]));
  });

  it("audit 切開後儲存 → writeWorkflowConfig 收到完整目標狀態（含讀入現值）", async () => {
    const ws = renderView(snapshot());
    fireEvent.click(await screen.findByLabelText("audit"));
    fireEvent.click(screen.getByTestId("save-workflow"));
    await waitFor(() =>
      expect(ws.writeWorkflowConfig).toHaveBeenCalledWith({
        locale: "tw",
        specLocale: null,
        tdd: true,
        audit: true,
      }),
    );
  });

  it("locale 下拉改值後儲存 → 新值進完整目標狀態；設回未設定送 null", async () => {
    const ws = renderView(snapshot());
    const locale = (await screen.findByLabelText("locale")) as HTMLSelectElement;
    fireEvent.change(locale, { target: { value: "" } });
    fireEvent.click(screen.getByTestId("save-workflow"));
    await waitFor(() =>
      expect(ws.writeWorkflowConfig).toHaveBeenCalledWith({
        locale: null,
        specLocale: null,
        tdd: true,
        audit: false,
      }),
    );
  });

  it("寫入失敗：顯示單行錯誤訊息", async () => {
    const ws = renderView(snapshot());
    (ws.writeWorkflowConfig as ReturnType<typeof vi.fn>).mockRejectedValue(
      "openspec/config.yaml: write failed: denied",
    );
    fireEvent.click(await screen.findByLabelText("tdd"));
    fireEvent.click(screen.getByTestId("save-workflow"));
    expect(await screen.findByText(/write failed/)).toBeTruthy();
  });
});

describe("SettingsView parseError（spec Scenario「解析失敗的檔案拒絕寫入」）", () => {
  it("config.yaml 解析失敗：顯示警告、該檔表單與儲存停用；另一檔不受影響", async () => {
    renderView(
      snapshot({
        workflow: { locale: null, specLocale: null, tdd: false, audit: false, parseError: "invalid yaml at line 3" },
      }),
    );
    expect(await screen.findByText(/invalid yaml at line 3/)).toBeTruthy();
    expect((screen.getByTestId("save-workflow") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByLabelText("tdd") as HTMLInputElement).disabled).toBe(true);
    // .speclink.yaml 表單照常可用。
    expect((screen.getByTestId("save-app") as HTMLButtonElement).disabled).toBe(false);
  });

  it(".speclink.yaml 解析失敗：tools 表單停用", async () => {
    renderView(
      snapshot({ app: { tools: [], customTools: [], parseError: "bad tools yaml" } }),
    );
    expect(await screen.findByText(/bad tools yaml/)).toBeTruthy();
    expect((screen.getByTestId("save-app") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByLabelText("claude") as HTMLInputElement).disabled).toBe(true);
  });
});

describe("SettingsView UI 語言三選（design D8）", () => {
  it("切換 UI 語言即回呼偏好、不觸碰 config.yaml 寫入", async () => {
    const onLocalePrefChange = vi.fn();
    const snap = snapshot();
    const ws = fakeWorkspace(snap);
    render(
      <SettingsView workspace={ws} localePref={null} onLocalePrefChange={onLocalePrefChange} />,
    );
    const group = await screen.findByTestId("ui-locale");
    fireEvent.click(within(group).getByText("English"));
    expect(onLocalePrefChange).toHaveBeenCalledWith("en");
    fireEvent.click(within(group).getByText(/跟隨系統/));
    expect(onLocalePrefChange).toHaveBeenCalledWith(null);
    expect(ws.writeWorkflowConfig).not.toHaveBeenCalled();
  });
});
