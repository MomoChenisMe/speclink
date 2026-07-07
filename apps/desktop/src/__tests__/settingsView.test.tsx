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
    // 警告於政策、專案設定兩張卡各自呈現。
    expect(await screen.findAllByText(/invalid yaml at line 3/)).toHaveLength(2);
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

// ---- 專案設定卡（spec 需求「設定頁編輯專案說明與產出規則」；design D1/D2/D3）----

const ORIGINAL_RULES_PAYLOAD = [
  ["proposal", ["提案必須列出影響的 crates"]],
  ["design", []],
  ["specs", []],
  ["tasks", ["先寫失敗測試", "更新文件"]],
];

function projectSnap(over: Partial<SettingsSnapshot["workflow"]> = {}) {
  return snapshot({
    workflow: {
      locale: "tw",
      specLocale: null,
      tdd: true,
      audit: false,
      context: "# 專案簡介\n\n這是一段說明",
      rules: { proposal: ["提案必須列出影響的 crates"], tasks: ["先寫失敗測試", "更新文件"] },
      schemaArtifacts: ["proposal", "design", "specs", "tasks"],
      parseError: null,
      ...over,
    },
  });
}

async function findProjectCard() {
  return await screen.findByTestId("project-settings-card");
}

/** Radix TabsTrigger 以 mousedown 觸發。 */
function switchToRulesTab() {
  fireEvent.mouseDown(screen.getByRole("tab", { name: "產出規則" }));
}

describe("專案設定卡唯讀優先（spec Scenario「唯讀優先與就地編輯切換」；design D1/D3）", () => {
  it("開啟設定頁：頂部卡片唯讀——專案說明 markdown 渲染、標註 config.yaml、無文字區；短文不收合", async () => {
    renderView(projectSnap());
    const card = await findProjectCard();
    expect(within(card).getByText("專案設定")).toBeTruthy();
    expect(within(card).getByText(/config\.yaml/)).toBeTruthy();
    // markdown 渲染：# 標題成為 heading，而非 raw 文字區。
    expect(within(card).getByRole("heading", { name: "專案簡介" })).toBeTruthy();
    expect(within(card).queryAllByRole("textbox")).toHaveLength(0);
    // 唯讀態按鈕列：僅編輯，無取消/儲存。
    expect(within(card).getByTestId("project-edit")).toBeTruthy();
    expect(within(card).queryByTestId("project-save")).toBeNull();
    expect(within(card).queryByTestId("project-cancel")).toBeNull();
    // 短文不收合：無顯示更多。
    expect(within(card).queryByTestId("context-show-more")).toBeNull();
  });

  it("超長專案說明收合並提供顯示更多，點擊展開", async () => {
    const longContext = Array.from({ length: 20 }, (_, i) => `第 ${i + 1} 行說明`).join("\n");
    renderView(projectSnap({ context: longContext }));
    const card = await findProjectCard();
    const readonly = within(card).getByTestId("context-readonly");
    expect(readonly.className).toContain("overflow-hidden");
    const more = within(card).getByTestId("context-show-more");
    expect(more.textContent).toContain("顯示更多");
    fireEvent.click(more);
    expect(within(card).queryByTestId("context-show-more")).toBeNull();
    expect(within(card).getByTestId("context-readonly").className).not.toContain("overflow-hidden");
  });

  it("未設定專案說明顯示空狀態提示", async () => {
    renderView(projectSnap({ context: null }));
    const card = await findProjectCard();
    expect(within(card).getByText(/尚未設定專案說明/)).toBeTruthy();
    expect(within(card).queryByTestId("context-show-more")).toBeNull();
  });

  it("產出規則唯讀僅列有條目鍵，鍵名為小節標題、條目為清單", async () => {
    renderView(projectSnap());
    const card = await findProjectCard();
    switchToRulesTab();
    expect(within(card).getByTestId("rules-readonly-proposal")).toBeTruthy();
    const tasks = within(card).getByTestId("rules-readonly-tasks");
    expect(within(tasks).getByText("先寫失敗測試")).toBeTruthy();
    expect(within(tasks).getByText("更新文件")).toBeTruthy();
    // 無條目的鍵不列出；唯讀態無任何輸入框。
    expect(within(card).queryByTestId("rules-readonly-design")).toBeNull();
    expect(within(card).queryByTestId("rules-readonly-specs")).toBeNull();
    expect(within(card).queryAllByRole("textbox")).toHaveLength(0);
  });
});

describe("專案設定卡就地編輯（spec 需求「設定頁編輯專案說明與產出規則」；design D1/D2）", () => {
  it("點編輯就地切換：按鈕列變取消/儲存，專案說明為 raw markdown 文字區", async () => {
    renderView(projectSnap());
    const card = await findProjectCard();
    fireEvent.click(within(card).getByTestId("project-edit"));
    expect(within(card).queryByTestId("project-edit")).toBeNull();
    expect(within(card).getByTestId("project-cancel")).toBeTruthy();
    expect(within(card).getByTestId("project-save")).toBeTruthy();
    const input = within(card).getByTestId("context-input") as HTMLTextAreaElement;
    expect(input.value).toBe("# 專案簡介\n\n這是一段說明");
  });

  it("編輯態產出規則文字區恰為活躍 schema 固定鍵各一、值為條目換行串接、無自由鍵輸入", async () => {
    // spec Scenario「固定鍵分節不可自由輸入」。
    renderView(projectSnap());
    const card = await findProjectCard();
    fireEvent.click(within(card).getByTestId("project-edit"));
    switchToRulesTab();
    for (const id of ["proposal", "design", "specs", "tasks"]) {
      expect(within(card).getByTestId(`rules-input-${id}`)).toBeTruthy();
    }
    expect(within(card).getAllByRole("textbox")).toHaveLength(4);
    expect((within(card).getByTestId("rules-input-tasks") as HTMLTextAreaElement).value).toBe(
      "先寫失敗測試\n更新文件",
    );
    expect((within(card).getByTestId("rules-input-design") as HTMLTextAreaElement).value).toBe("");
  });

  it("儲存：逐行 trim、空行滌除、行序即順序（spec Example 行對調）；未動的專案說明原樣寫回", async () => {
    const ws = renderView(projectSnap());
    const card = await findProjectCard();
    fireEvent.click(within(card).getByTestId("project-edit"));
    switchToRulesTab();
    fireEvent.change(within(card).getByTestId("rules-input-tasks"), {
      target: { value: "  更新文件  \n\n先寫失敗測試\n" },
    });
    fireEvent.click(within(card).getByTestId("project-save"));
    await waitFor(() => expect(ws.writeWorkflowRules).toHaveBeenCalled());
    expect((ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mock.calls[0][0]).toEqual([
      ["proposal", ["提案必須列出影響的 crates"]],
      ["design", []],
      ["specs", []],
      ["tasks", ["更新文件", "先寫失敗測試"]],
    ]);
    expect(ws.writeWorkflowContext).toHaveBeenCalledWith("# 專案簡介\n\n這是一段說明");
  });

  it("以保留字元開頭的條目原文進 payload（spec Example 保留字元條目自動加引號的前端面）", async () => {
    const ws = renderView(projectSnap());
    const card = await findProjectCard();
    fireEvent.click(within(card).getByTestId("project-edit"));
    switchToRulesTab();
    fireEvent.change(within(card).getByTestId("rules-input-tasks"), {
      target: { value: "先寫失敗測試\n更新文件\n@完成後執行全部測試" },
    });
    fireEvent.click(within(card).getByTestId("project-save"));
    await waitFor(() => expect(ws.writeWorkflowRules).toHaveBeenCalled());
    const payload = (ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mock.calls[0][0];
    expect(payload).toContainEqual(["tasks", ["先寫失敗測試", "更新文件", "@完成後執行全部測試"]]);
    expect(payload).toContainEqual(["proposal", ["提案必須列出影響的 crates"]]);
  });

  it("清空某鍵文字區＝該鍵送空清單、其餘鍵保留（spec Example 鍵移除第二列）", async () => {
    const ws = renderView(projectSnap());
    const card = await findProjectCard();
    fireEvent.click(within(card).getByTestId("project-edit"));
    switchToRulesTab();
    fireEvent.change(within(card).getByTestId("rules-input-tasks"), { target: { value: "" } });
    fireEvent.click(within(card).getByTestId("project-save"));
    await waitFor(() => expect(ws.writeWorkflowRules).toHaveBeenCalled());
    const payload = (ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mock.calls[0][0];
    expect(payload).toContainEqual(["tasks", []]);
    expect(payload).toContainEqual(["proposal", ["提案必須列出影響的 crates"]]);
  });

  it("全部鍵清空＝全空 payload（觸發後端移除 rules 鍵；spec Example 鍵移除第三列）", async () => {
    const ws = renderView(projectSnap());
    const card = await findProjectCard();
    fireEvent.click(within(card).getByTestId("project-edit"));
    switchToRulesTab();
    fireEvent.change(within(card).getByTestId("rules-input-tasks"), { target: { value: "" } });
    fireEvent.change(within(card).getByTestId("rules-input-proposal"), { target: { value: "  \n " } });
    fireEvent.click(within(card).getByTestId("project-save"));
    await waitFor(() => expect(ws.writeWorkflowRules).toHaveBeenCalled());
    expect((ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mock.calls[0][0]).toEqual([
      ["proposal", []],
      ["design", []],
      ["specs", []],
      ["tasks", []],
    ]);
  });

  it("清空專案說明儲存 → 送空字串（鍵移除），未動的產出規則原樣寫回（spec Example 鍵移除第一列）", async () => {
    const ws = renderView(projectSnap());
    const card = await findProjectCard();
    fireEvent.click(within(card).getByTestId("project-edit"));
    fireEvent.change(within(card).getByTestId("context-input"), { target: { value: "" } });
    fireEvent.click(within(card).getByTestId("project-save"));
    await waitFor(() => expect(ws.writeWorkflowContext).toHaveBeenCalledWith(""));
    expect((ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mock.calls[0][0]).toEqual(
      ORIGINAL_RULES_PAYLOAD,
    );
  });

  it("編輯專案說明並儲存：回唯讀渲染新內容，未動的產出規則原樣寫回（spec Scenario 編輯專案說明並儲存）", async () => {
    const ws = renderView(projectSnap());
    const card = await findProjectCard();
    fireEvent.click(within(card).getByTestId("project-edit"));
    fireEvent.change(within(card).getByTestId("context-input"), {
      target: { value: "# 新標題\n\n新的說明" },
    });
    fireEvent.click(within(card).getByTestId("project-save"));
    await waitFor(() => expect(ws.writeWorkflowContext).toHaveBeenCalledWith("# 新標題\n\n新的說明"));
    expect((ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mock.calls[0][0]).toEqual(
      ORIGINAL_RULES_PAYLOAD,
    );
    // 卡片回唯讀並渲染新內容。
    await waitFor(() => expect(within(card).getByTestId("project-edit")).toBeTruthy());
    expect(within(card).getByRole("heading", { name: "新標題" })).toBeTruthy();
    expect(within(card).queryAllByRole("textbox")).toHaveLength(0);
  });

  it("取消放棄編輯：還原唯讀且不觸發寫入，重進編輯草稿已重設（spec Scenario 取消放棄編輯）", async () => {
    const ws = renderView(projectSnap());
    const card = await findProjectCard();
    fireEvent.click(within(card).getByTestId("project-edit"));
    fireEvent.change(within(card).getByTestId("context-input"), { target: { value: "被放棄的修改" } });
    switchToRulesTab();
    fireEvent.change(within(card).getByTestId("rules-input-tasks"), { target: { value: "也被放棄" } });
    fireEvent.click(within(card).getByTestId("project-cancel"));
    // 還原唯讀呈現、零寫入。
    expect(within(card).getByTestId("project-edit")).toBeTruthy();
    expect(ws.writeWorkflowContext).not.toHaveBeenCalled();
    expect(ws.writeWorkflowRules).not.toHaveBeenCalled();
    // 重進編輯：草稿為讀入值，非被放棄的修改。
    fireEvent.click(within(card).getByTestId("project-edit"));
    expect((within(card).getByTestId("rules-input-tasks") as HTMLTextAreaElement).value).toBe(
      "先寫失敗測試\n更新文件",
    );
  });

  it("寫入失敗：顯示單行錯誤並維持編輯態不遺失輸入（design 契約失敗模式）", async () => {
    const ws = renderView(projectSnap());
    (ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mockRejectedValue(
      "openspec/config.yaml: write failed: denied",
    );
    const card = await findProjectCard();
    fireEvent.click(within(card).getByTestId("project-edit"));
    fireEvent.change(within(card).getByTestId("context-input"), { target: { value: "改到一半的內容" } });
    fireEvent.click(within(card).getByTestId("project-save"));
    expect(await within(card).findByText(/write failed/)).toBeTruthy();
    // 仍在編輯態、輸入未遺失。
    expect(within(card).getByTestId("project-save")).toBeTruthy();
    expect((within(card).getByTestId("context-input") as HTMLTextAreaElement).value).toBe(
      "改到一半的內容",
    );
  });

  it("分頁名採正典詞；編輯態說明載明一行一條規則、頭尾空白不保留與註解不保留（design D4／風險緩解）", async () => {
    renderView(projectSnap());
    const card = await findProjectCard();
    expect(within(card).getByRole("tab", { name: "專案說明" })).toBeTruthy();
    expect(within(card).getByRole("tab", { name: "產出規則" })).toBeTruthy();
    fireEvent.click(within(card).getByTestId("project-edit"));
    expect(within(card).getByText(/清空儲存即移除該鍵/)).toBeTruthy();
    switchToRulesTab();
    expect(within(card).getByText(/一行一條規則（頭尾空白不保留）/)).toBeTruthy();
    expect(within(card).getByText(/檔內註解不會保留/)).toBeTruthy();
  });

  it("解析失敗停用編輯：卡片浮出說明、編輯鈕停用（spec Scenario 解析失敗停用編輯）", async () => {
    renderView(
      projectSnap({
        context: null,
        rules: {},
        schemaArtifacts: [],
        parseError: "invalid yaml at line 3",
      }),
    );
    const card = await findProjectCard();
    expect(within(card).getByText(/invalid yaml at line 3/)).toBeTruthy();
    expect((within(card).getByTestId("project-edit") as HTMLButtonElement).disabled).toBe(true);
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
