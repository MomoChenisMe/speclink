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
      context: null,
      rules: {},
      schemaArtifacts: ["proposal", "design", "specs", "tasks"],
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
    writeWorkflowContext: vi.fn().mockResolvedValue(undefined),
    writeWorkflowRules: vi.fn().mockResolvedValue(undefined),
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
        workflow: {
          locale: null,
          specLocale: null,
          tdd: false,
          audit: false,
          context: null,
          rules: {},
          schemaArtifacts: [],
          parseError: "invalid yaml at line 3",
        },
      }),
    );
    // 警告於政策、專案說明、產出規則三張卡各自呈現。
    expect(await screen.findAllByText(/invalid yaml at line 3/)).toHaveLength(3);
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

describe("SettingsView 專案說明與產出規則（spec 需求「設定頁編輯專案說明與產出規則」）", () => {
  const contentSnap = () =>
    snapshot({
      workflow: {
        locale: "tw",
        specLocale: null,
        tdd: true,
        audit: false,
        context: "舊的專案說明",
        rules: { proposal: ["提案必須列出影響的 crates"], tasks: ["先寫失敗測試", "更新文件"] },
        schemaArtifacts: ["proposal", "design", "specs", "tasks"],
        parseError: null,
      },
    });

  it("專案說明呈現現值，改寫後儲存 → writeWorkflowContext 收到新文字", async () => {
    const ws = renderView(contentSnap());
    const input = (await screen.findByTestId("context-input")) as HTMLTextAreaElement;
    expect(input.value).toBe("舊的專案說明");
    fireEvent.change(input, { target: { value: "新的專案說明\n跨兩行" } });
    fireEvent.click(screen.getByTestId("save-context"));
    await waitFor(() =>
      expect(ws.writeWorkflowContext).toHaveBeenCalledWith("新的專案說明\n跨兩行"),
    );
  });

  it("清空專案說明儲存 → 送出空字串（鍵移除語意在後端）", async () => {
    const ws = renderView(contentSnap());
    const input = (await screen.findByTestId("context-input")) as HTMLTextAreaElement;
    fireEvent.change(input, { target: { value: "" } });
    fireEvent.click(screen.getByTestId("save-context"));
    await waitFor(() => expect(ws.writeWorkflowContext).toHaveBeenCalledWith(""));
  });

  it("產出規則恰以 schemaArtifacts 固定鍵分節且無自由鍵輸入", async () => {
    // spec Scenario「固定鍵分節不可自由輸入」。
    renderView(contentSnap());
    for (const id of ["proposal", "design", "specs", "tasks"]) {
      expect(await screen.findByTestId(`rules-section-${id}`)).toBeTruthy();
    }
    // 無第五節、無新增分節鍵的輸入介面。
    expect(document.querySelectorAll("[data-testid^='rules-section-']")).toHaveLength(4);
    expect(screen.queryByTestId("add-section")).toBeNull();
  });

  it("條目上移後儲存 → payload 依新順序（spec Example 條目對調）", async () => {
    const ws = renderView(contentSnap());
    const tasks = await screen.findByTestId("rules-section-tasks");
    // tasks 節依序「先寫失敗測試」「更新文件」——將第二條上移一位。
    fireEvent.click(within(tasks).getAllByLabelText("上移")[1]);
    fireEvent.click(screen.getByTestId("save-rules"));
    await waitFor(() => expect(ws.writeWorkflowRules).toHaveBeenCalled());
    const payload = (ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mock.calls[0][0];
    expect(payload).toEqual([
      ["proposal", ["提案必須列出影響的 crates"]],
      ["design", []],
      ["specs", []],
      ["tasks", ["更新文件", "先寫失敗測試"]],
    ]);
  });

  it("新增與編輯條目後儲存 → payload 含新條目", async () => {
    const ws = renderView(contentSnap());
    const design = await screen.findByTestId("rules-section-design");
    fireEvent.click(within(design).getByText("新增條目"));
    const inputs = within(design).getAllByRole("textbox");
    fireEvent.change(inputs[inputs.length - 1], { target: { value: "設計必須列出替代方案" } });
    fireEvent.click(screen.getByTestId("save-rules"));
    await waitFor(() => expect(ws.writeWorkflowRules).toHaveBeenCalled());
    const payload = (ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mock.calls[0][0];
    expect(payload).toContainEqual(["design", ["設計必須列出替代方案"]]);
  });

  it("刪除某節全部條目後儲存 → 該節送空清單（清空觸發鍵移除語意）", async () => {
    const ws = renderView(contentSnap());
    const tasks = await screen.findByTestId("rules-section-tasks");
    fireEvent.click(within(tasks).getAllByLabelText("刪除")[0]);
    fireEvent.click(within(tasks).getAllByLabelText("刪除")[0]);
    expect(within(tasks).queryAllByRole("textbox")).toHaveLength(0);
    fireEvent.click(screen.getByTestId("save-rules"));
    await waitFor(() => expect(ws.writeWorkflowRules).toHaveBeenCalled());
    const payload = (ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mock.calls[0][0];
    expect(payload).toContainEqual(["tasks", []]);
  });

  it("config.yaml 解析失敗時兩區段停用且不可儲存", async () => {
    renderView(
      snapshot({
        workflow: {
          locale: null,
          specLocale: null,
          tdd: false,
          audit: false,
          context: null,
          rules: {},
          schemaArtifacts: [],
          parseError: "invalid yaml at line 3",
        },
      }),
    );
    const input = (await screen.findByTestId("context-input")) as HTMLTextAreaElement;
    expect(input.disabled).toBe(true);
    expect((screen.getByTestId("save-context") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByTestId("save-rules") as HTMLButtonElement).disabled).toBe(true);
  });

  it("區段名採正典詞且產出規則附註解遺失說明（i18n 字典）", async () => {
    // design D5：zh-TW 正典詞「專案說明」「產出規則」；en 對應鍵由
    // messages.test.ts 的 key 集合相等測試保護。
    renderView(contentSnap());
    expect(await screen.findByText("專案說明")).toBeTruthy();
    expect(screen.getByText("產出規則")).toBeTruthy();
    expect(screen.getByText(/檔內註解不會保留/)).toBeTruthy();
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
