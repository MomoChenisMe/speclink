// ProjectSettingsView（spec 需求「設定頁圖形化讀寫兩層設定」「設定頁編輯專案說明與產出規則」
// 的前端面）：兩頁簽組織（config.yaml／.speclink.yaml）、
// 專案說明與產出規則拆為獨立卡各持編輯態、解析失敗簽級警示。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, waitFor, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { ProjectSettingsView } from "../views/ProjectSettingsView";
import { APP_MESSAGES } from "../i18n/messages";
import type { SettingsSnapshot } from "../adapter/workspace";
import type { WorkspaceSettingsProvider } from "../session";

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

/** 活躍 session 的設定面 mock（workspace-session 決策 3：root 已綁定）。 */
function fakeSettings(snap: SettingsSnapshot): WorkspaceSettingsProvider {
  return {
    kind: "local",
    policyWrite: true,
    readSettings: vi.fn().mockResolvedValue(snap),
    writeAppTools: vi.fn().mockResolvedValue(undefined),
    writeWorkflowConfig: vi.fn().mockResolvedValue(undefined),
    writeWorkflowContext: vi.fn().mockResolvedValue(undefined),
    writeWorkflowRules: vi.fn().mockResolvedValue(undefined),
  } as unknown as WorkspaceSettingsProvider;
}

function renderView(snap: SettingsSnapshot) {
  const ws = fakeSettings(snap);
  render(
    <ProjectSettingsView
      settings={ws}
    />,
  );
  return ws;
}

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

/** 頂層頁簽切換（Radix TabsTrigger 以 mousedown 觸發）。 */
function switchToTab(name: string) {
  fireEvent.mouseDown(screen.getByRole("tab", { name }));
}

describe("ProjectSettingsView 載入", () => {
  it("呈現兩檔現值：config.yaml 簽的政策欄位、.speclink.yaml 簽的 tools 勾選；未設定欄位呈預設狀態", async () => {
    renderView(snapshot());
    const locale = (await screen.findByLabelText("locale")) as HTMLSelectElement;
    expect(locale.value).toBe("tw");
    // 未設定的 spec_locale 呈預設值狀態（空字串＝未設定）。
    expect((screen.getByLabelText("spec_locale") as HTMLSelectElement).value).toBe("");
    expect(screen.getByLabelText("tdd").getAttribute("aria-checked")).toBe("true");
    expect(screen.getByLabelText("audit").getAttribute("aria-checked")).toBe("false");
    switchToTab(".speclink.yaml");
    expect(screen.getByLabelText("claude").getAttribute("aria-checked")).toBe("true");
    expect(screen.getByLabelText("codex").getAttribute("aria-checked")).toBe("false");
  });

  it("自訂工具描述子呈現為不可編輯項", async () => {
    renderView(snapshot({ app: { tools: ["claude"], customTools: ["wad-harness"], parseError: null } }));
    await screen.findByRole("tab", { name: ".speclink.yaml" });
    switchToTab(".speclink.yaml");
    const custom = await screen.findByText("wad-harness");
    // 不可編輯：非表單輸入（無對應 checkbox）。
    expect(screen.queryByLabelText("wad-harness")).toBeNull();
    expect(custom).toBeTruthy();
  });
});

describe("ProjectSettingsView 寫入", () => {
  it("tools 加選 codex 後儲存 → writeAppTools 收到完整選集", async () => {
    const ws = renderView(snapshot());
    await screen.findByRole("tab", { name: ".speclink.yaml" });
    switchToTab(".speclink.yaml");
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

// ---- 兩頁簽組織（spec 需求「設定頁圖形化讀寫兩層設定」）----

describe("專案設定頁兩頁簽組織（spec Scenario「兩頁分工與預設簽」）", () => {
  it("頁簽依序 config.yaml／.speclink.yaml，預設落在 config.yaml 簽（三卡＋mono 路徑註記）", async () => {
    renderView(projectSnap());
    await screen.findByTestId("context-card");
    const tabs = screen.getAllByRole("tab").map((t) => t.textContent);
    expect(tabs).toEqual(["config.yaml", ".speclink.yaml"]);
    expect(screen.getByRole("tab", { name: "config.yaml" }).getAttribute("aria-selected")).toBe("true");
    // config.yaml 簽卡片歸屬：專案說明、產出規則、產出政策。
    expect(screen.getByTestId("context-card")).toBeTruthy();
    expect(screen.getByTestId("rules-card")).toBeTruthy();
    expect(screen.getByTestId("policy-card")).toBeTruthy();
    // 簽首等寬字檔案路徑註記。
    const note = screen.getByTestId("file-note-config");
    expect(note.textContent).toBe("openspec/config.yaml");
    expect(note.className).toContain("font-mono");
    // 其他簽的卡片不在畫面上。
    expect(screen.queryByLabelText("claude")).toBeNull();
  });

  it("切至 .speclink.yaml 簽見 AI 工具卡與等寬字檔案路徑註記", async () => {
    renderView(projectSnap());
    await screen.findByTestId("context-card");
    switchToTab(".speclink.yaml");
    expect(screen.getByText("AI 工具")).toBeTruthy();
    expect(screen.getByLabelText("claude").getAttribute("aria-checked")).toBe("true");
    const note = screen.getByTestId("file-note-speclink");
    expect(note.textContent).toBe(".speclink.yaml");
    expect(note.className).toContain("font-mono");
    expect(screen.queryByTestId("context-card")).toBeNull();
  });

  it("config.yaml 解析失敗：頁簽帶警示點、簽首橫幅、簽內表單與編輯鈕停用", async () => {
    renderView(
      projectSnap({ context: null, rules: {}, schemaArtifacts: [], parseError: "invalid yaml at line 3" }),
    );
    const configTab = await screen.findByRole("tab", { name: "config.yaml" });
    expect(within(configTab).getByTestId("tab-warning")).toBeTruthy();
    // config.yaml 簽：橫幅浮出、政策表單與兩卡編輯鈕停用。
    expect(await screen.findByText(/invalid yaml at line 3/)).toBeTruthy();
    expect((screen.getByTestId("save-workflow") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByLabelText("tdd") as HTMLInputElement).disabled).toBe(true);
    expect((screen.getByTestId("context-edit") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByTestId("rules-edit") as HTMLButtonElement).disabled).toBe(true);
    // .speclink.yaml 簽無警示點。
    expect(
      within(screen.getByRole("tab", { name: ".speclink.yaml" })).queryByTestId("tab-warning"),
    ).toBeNull();
  });

  it(".speclink.yaml 解析失敗：其頁簽帶警示點、AI 工具表單停用；config.yaml 簽表單照常可用", async () => {
    renderView(snapshot({ app: { tools: [], customTools: [], parseError: "bad tools yaml" } }));
    const spTab = await screen.findByRole("tab", { name: ".speclink.yaml" });
    expect(within(spTab).getByTestId("tab-warning")).toBeTruthy();
    // 預設簽（config.yaml）表單照常可用。
    expect((screen.getByTestId("save-workflow") as HTMLButtonElement).disabled).toBe(false);
    switchToTab(".speclink.yaml");
    expect(await screen.findByText(/bad tools yaml/)).toBeTruthy();
    expect((screen.getByTestId("save-app") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByLabelText("claude") as HTMLInputElement).disabled).toBe(true);
  });
});

// ---- 專案說明卡／產出規則卡（spec 需求「設定頁編輯專案說明與產出規則」；design D2）----

describe("兩卡唯讀優先（spec Scenario「唯讀優先與各卡就地編輯」）", () => {
  it("開啟 config.yaml 簽：專案說明卡唯讀 markdown 渲染、無文字區、按鈕列僅編輯；短文不收合", async () => {
    renderView(projectSnap());
    const card = await screen.findByTestId("context-card");
    expect(within(card).getByText("專案說明")).toBeTruthy();
    // markdown 渲染：# 標題成為 heading，而非 raw 文字區。
    expect(within(card).getByRole("heading", { name: "專案簡介" })).toBeTruthy();
    expect(within(card).queryAllByRole("textbox")).toHaveLength(0);
    // 唯讀態按鈕列：僅編輯，無取消/儲存。
    expect(within(card).getByTestId("context-edit")).toBeTruthy();
    expect(within(card).queryByTestId("context-save")).toBeNull();
    expect(within(card).queryByTestId("context-cancel")).toBeNull();
    // 短文不收合：無顯示更多。
    expect(within(card).queryByTestId("context-show-more")).toBeNull();
  });

  it("超長專案說明收合並提供顯示更多，點擊展開", async () => {
    const longContext = Array.from({ length: 20 }, (_, i) => `第 ${i + 1} 行說明`).join("\n");
    renderView(projectSnap({ context: longContext }));
    const card = await screen.findByTestId("context-card");
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
    const card = await screen.findByTestId("context-card");
    expect(within(card).getByText(/尚未設定專案說明/)).toBeTruthy();
    expect(within(card).queryByTestId("context-show-more")).toBeNull();
  });

  it("產出規則卡唯讀僅列有條目鍵，鍵名為小節標題、條目為清單", async () => {
    renderView(projectSnap());
    const card = await screen.findByTestId("rules-card");
    expect(within(card).getByText("產出規則")).toBeTruthy();
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

describe("拆卡獨立編輯態（spec 需求「設定頁編輯專案說明與產出規則」；design D2）", () => {
  it("點編輯就地切換：按鈕列變取消/儲存，專案說明為 raw markdown 文字區", async () => {
    renderView(projectSnap());
    const card = await screen.findByTestId("context-card");
    fireEvent.click(within(card).getByTestId("context-edit"));
    expect(within(card).queryByTestId("context-edit")).toBeNull();
    expect(within(card).getByTestId("context-cancel")).toBeTruthy();
    expect(within(card).getByTestId("context-save")).toBeTruthy();
    const input = within(card).getByTestId("context-input") as HTMLTextAreaElement;
    expect(input.value).toBe("# 專案簡介\n\n這是一段說明");
  });

  // spec「表單控制項與按鈕以主題化元件呈現」Scenario「設定頁多行輸入主題化」
  it("專案說明與產出規則的多行輸入為主題化 Textarea 原語（data-slot 標記＋token 樣式）", async () => {
    renderView(projectSnap());
    const ctxCard = await screen.findByTestId("context-card");
    fireEvent.click(within(ctxCard).getByTestId("context-edit"));
    const ctxInput = within(ctxCard).getByTestId("context-input");
    expect(ctxInput.getAttribute("data-slot")).toBe("textarea");
    expect(ctxInput.className).toContain("border-input");
    expect(ctxInput.className).toContain("focus-visible:ring");
    const rulesCard = screen.getByTestId("rules-card");
    fireEvent.click(within(rulesCard).getByTestId("rules-edit"));
    const rulesInput = within(rulesCard).getByTestId("rules-input-tasks");
    expect(rulesInput.getAttribute("data-slot")).toBe("textarea");
    expect(rulesInput.className).toContain("border-input");
    expect(rulesInput.className).toContain("focus-visible:ring");
  });

  it("一卡編輯中另一卡唯讀可用；各卡取消僅還原本卡且草稿重設", async () => {
    const ws = renderView(projectSnap());
    const ctxCard = await screen.findByTestId("context-card");
    const rulesCard = screen.getByTestId("rules-card");
    fireEvent.click(within(ctxCard).getByTestId("context-edit"));
    // 另一卡維持唯讀且編輯鈕仍可用。
    expect(within(rulesCard).queryAllByRole("textbox")).toHaveLength(0);
    const rulesEdit = within(rulesCard).getByTestId("rules-edit") as HTMLButtonElement;
    expect(rulesEdit.disabled).toBe(false);
    // 兩卡可同時各持編輯態。
    fireEvent.click(rulesEdit);
    expect(within(ctxCard).getByTestId("context-input")).toBeTruthy();
    expect(within(rulesCard).getByTestId("rules-input-tasks")).toBeTruthy();
    // 於兩卡各改草稿後取消產出規則卡：僅該卡還原，專案說明卡編輯態與草稿不受影響。
    fireEvent.change(within(ctxCard).getByTestId("context-input"), { target: { value: "編輯中的說明" } });
    fireEvent.change(within(rulesCard).getByTestId("rules-input-tasks"), { target: { value: "被放棄" } });
    fireEvent.click(within(rulesCard).getByTestId("rules-cancel"));
    expect(within(rulesCard).queryAllByRole("textbox")).toHaveLength(0);
    expect((within(ctxCard).getByTestId("context-input") as HTMLTextAreaElement).value).toBe("編輯中的說明");
    expect(ws.writeWorkflowContext).not.toHaveBeenCalled();
    expect(ws.writeWorkflowRules).not.toHaveBeenCalled();
    // 重進產出規則卡編輯：草稿為讀入值，非被放棄的修改。
    fireEvent.click(within(rulesCard).getByTestId("rules-edit"));
    expect((within(rulesCard).getByTestId("rules-input-tasks") as HTMLTextAreaElement).value).toBe(
      "先寫失敗測試\n更新文件",
    );
  });

  it("編輯態產出規則文字區恰為活躍 schema 固定鍵各一、值為條目換行串接、無自由鍵輸入", async () => {
    // spec Scenario「固定鍵分節不可自由輸入」。
    renderView(projectSnap());
    const card = await screen.findByTestId("rules-card");
    fireEvent.click(within(card).getByTestId("rules-edit"));
    for (const id of ["proposal", "design", "specs", "tasks"]) {
      expect(within(card).getByTestId(`rules-input-${id}`)).toBeTruthy();
    }
    expect(within(card).getAllByRole("textbox")).toHaveLength(4);
    expect((within(card).getByTestId("rules-input-tasks") as HTMLTextAreaElement).value).toBe(
      "先寫失敗測試\n更新文件",
    );
    expect((within(card).getByTestId("rules-input-design") as HTMLTextAreaElement).value).toBe("");
  });

  it("產出規則卡儲存：逐行 trim、空行滌除、行序即順序（spec Example 行對調）；僅寫 rules、context 逐字元不變", async () => {
    // spec Scenario「各卡儲存僅寫對應鍵」——僅存產出規則時不觸碰 context。
    const ws = renderView(projectSnap());
    const card = await screen.findByTestId("rules-card");
    fireEvent.click(within(card).getByTestId("rules-edit"));
    fireEvent.change(within(card).getByTestId("rules-input-tasks"), {
      target: { value: "  更新文件  \n\n先寫失敗測試\n" },
    });
    fireEvent.click(within(card).getByTestId("rules-save"));
    await waitFor(() => expect(ws.writeWorkflowRules).toHaveBeenCalled());
    expect((ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mock.calls[0][0]).toEqual([
      ["proposal", ["提案必須列出影響的 crates"]],
      ["design", []],
      ["specs", []],
      ["tasks", ["更新文件", "先寫失敗測試"]],
    ]);
    expect(ws.writeWorkflowContext).not.toHaveBeenCalled();
  });

  it("以保留字元開頭的條目原文進 payload（spec Example 保留字元條目自動加引號的前端面）", async () => {
    const ws = renderView(projectSnap());
    const card = await screen.findByTestId("rules-card");
    fireEvent.click(within(card).getByTestId("rules-edit"));
    fireEvent.change(within(card).getByTestId("rules-input-tasks"), {
      target: { value: "先寫失敗測試\n更新文件\n@完成後執行全部測試" },
    });
    fireEvent.click(within(card).getByTestId("rules-save"));
    await waitFor(() => expect(ws.writeWorkflowRules).toHaveBeenCalled());
    const payload = (ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mock.calls[0][0];
    expect(payload).toContainEqual(["tasks", ["先寫失敗測試", "更新文件", "@完成後執行全部測試"]]);
    expect(payload).toContainEqual(["proposal", ["提案必須列出影響的 crates"]]);
  });

  it("清空某鍵文字區＝該鍵送空清單、其餘鍵保留（spec Example 鍵移除第二列）", async () => {
    const ws = renderView(projectSnap());
    const card = await screen.findByTestId("rules-card");
    fireEvent.click(within(card).getByTestId("rules-edit"));
    fireEvent.change(within(card).getByTestId("rules-input-tasks"), { target: { value: "" } });
    fireEvent.click(within(card).getByTestId("rules-save"));
    await waitFor(() => expect(ws.writeWorkflowRules).toHaveBeenCalled());
    const payload = (ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mock.calls[0][0];
    expect(payload).toContainEqual(["tasks", []]);
    expect(payload).toContainEqual(["proposal", ["提案必須列出影響的 crates"]]);
  });

  it("全部鍵清空＝全空 payload（觸發後端移除 rules 鍵；spec Example 鍵移除第三列）", async () => {
    const ws = renderView(projectSnap());
    const card = await screen.findByTestId("rules-card");
    fireEvent.click(within(card).getByTestId("rules-edit"));
    fireEvent.change(within(card).getByTestId("rules-input-tasks"), { target: { value: "" } });
    fireEvent.change(within(card).getByTestId("rules-input-proposal"), { target: { value: "  \n " } });
    fireEvent.click(within(card).getByTestId("rules-save"));
    await waitFor(() => expect(ws.writeWorkflowRules).toHaveBeenCalled());
    expect((ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mock.calls[0][0]).toEqual([
      ["proposal", []],
      ["design", []],
      ["specs", []],
      ["tasks", []],
    ]);
  });

  it("清空專案說明儲存 → 送空字串（鍵移除）；僅寫 context、產出規則不被觸碰（spec Example 鍵移除第一列）", async () => {
    const ws = renderView(projectSnap());
    const card = await screen.findByTestId("context-card");
    fireEvent.click(within(card).getByTestId("context-edit"));
    fireEvent.change(within(card).getByTestId("context-input"), { target: { value: "" } });
    fireEvent.click(within(card).getByTestId("context-save"));
    await waitFor(() => expect(ws.writeWorkflowContext).toHaveBeenCalledWith(""));
    expect(ws.writeWorkflowRules).not.toHaveBeenCalled();
  });

  it("編輯專案說明並儲存：回唯讀渲染新內容；僅寫 context（spec Scenario 編輯專案說明並儲存）", async () => {
    const ws = renderView(projectSnap());
    const card = await screen.findByTestId("context-card");
    fireEvent.click(within(card).getByTestId("context-edit"));
    fireEvent.change(within(card).getByTestId("context-input"), {
      target: { value: "# 新標題\n\n新的說明" },
    });
    fireEvent.click(within(card).getByTestId("context-save"));
    await waitFor(() => expect(ws.writeWorkflowContext).toHaveBeenCalledWith("# 新標題\n\n新的說明"));
    expect(ws.writeWorkflowRules).not.toHaveBeenCalled();
    // 卡片回唯讀並渲染新內容。
    await waitFor(() => expect(within(card).getByTestId("context-edit")).toBeTruthy());
    expect(within(card).getByRole("heading", { name: "新標題" })).toBeTruthy();
    expect(within(card).queryAllByRole("textbox")).toHaveLength(0);
  });

  it("取消放棄編輯：還原唯讀且不觸發寫入，重進編輯草稿已重設；產出規則卡全程不受影響（spec Scenario 取消放棄編輯）", async () => {
    const ws = renderView(projectSnap());
    const card = await screen.findByTestId("context-card");
    fireEvent.click(within(card).getByTestId("context-edit"));
    fireEvent.change(within(card).getByTestId("context-input"), { target: { value: "被放棄的修改" } });
    fireEvent.click(within(card).getByTestId("context-cancel"));
    // 還原唯讀呈現、零寫入。
    expect(within(card).getByTestId("context-edit")).toBeTruthy();
    expect(ws.writeWorkflowContext).not.toHaveBeenCalled();
    expect(ws.writeWorkflowRules).not.toHaveBeenCalled();
    // 產出規則卡全程唯讀不受影響。
    expect(within(screen.getByTestId("rules-card")).queryAllByRole("textbox")).toHaveLength(0);
    // 重進編輯：草稿為讀入值，非被放棄的修改。
    fireEvent.click(within(card).getByTestId("context-edit"));
    expect((within(card).getByTestId("context-input") as HTMLTextAreaElement).value).toBe(
      "# 專案簡介\n\n這是一段說明",
    );
  });

  it("寫入失敗：顯示單行錯誤並維持編輯態不遺失輸入（design 契約失敗模式）", async () => {
    const ws = renderView(projectSnap());
    (ws.writeWorkflowContext as ReturnType<typeof vi.fn>).mockRejectedValue(
      "openspec/config.yaml: write failed: denied",
    );
    const card = await screen.findByTestId("context-card");
    fireEvent.click(within(card).getByTestId("context-edit"));
    fireEvent.change(within(card).getByTestId("context-input"), { target: { value: "改到一半的內容" } });
    fireEvent.click(within(card).getByTestId("context-save"));
    expect(await within(card).findByText(/write failed/)).toBeTruthy();
    // 仍在編輯態、輸入未遺失。
    expect(within(card).getByTestId("context-save")).toBeTruthy();
    expect((within(card).getByTestId("context-input") as HTMLTextAreaElement).value).toBe(
      "改到一半的內容",
    );
  });

  it("卡名採正典詞；編輯態說明載明一行一條規則、頭尾空白不保留與註解不保留（design 風險緩解）", async () => {
    renderView(projectSnap());
    const ctxCard = await screen.findByTestId("context-card");
    const rulesCard = screen.getByTestId("rules-card");
    expect(within(ctxCard).getByText("專案說明")).toBeTruthy();
    expect(within(rulesCard).getByText("產出規則")).toBeTruthy();
    fireEvent.click(within(ctxCard).getByTestId("context-edit"));
    expect(within(ctxCard).getByText(/清空儲存即移除該鍵/)).toBeTruthy();
    fireEvent.click(within(rulesCard).getByTestId("rules-edit"));
    expect(within(rulesCard).getByText(/一行一條規則（頭尾空白不保留）/)).toBeTruthy();
    expect(within(rulesCard).getByText(/檔內註解不會保留/)).toBeTruthy();
  });
});

describe("remote Workflow 設定（remote-workflow-policy 決策 5/6）", () => {
  function remoteSettings(
    snap: SettingsSnapshot,
    policyWrite: boolean,
    nextRevision = 42,
  ): WorkspaceSettingsProvider {
    return {
      kind: "remote",
      policyWrite,
      readSettings: vi.fn().mockResolvedValue(snap),
      writeAppTools: vi.fn().mockRejectedValue(new Error("remote 無 tools 設定")),
      writeWorkflowConfig: vi.fn().mockResolvedValue(nextRevision),
      writeWorkflowContext: vi.fn().mockResolvedValue(nextRevision),
      writeWorkflowRules: vi.fn().mockResolvedValue(nextRevision),
    } as unknown as WorkspaceSettingsProvider;
  }

  it("editor 僅見 Workflow 簽、mono revision，政策可存且成功後 revision 前進", async () => {
    const snap = projectSnap({ revision: 41 } as Partial<SettingsSnapshot["workflow"]>);
    const ws = remoteSettings(snap, true);
    render(<ProjectSettingsView settings={ws} />);

    const tab = await screen.findByRole("tab", { name: "Workflow" });
    expect(screen.getAllByRole("tab")).toHaveLength(1);
    expect(tab.getAttribute("aria-selected")).toBe("true");
    expect(screen.queryByRole("tab", { name: ".speclink.yaml" })).toBeNull();
    const revision = screen.getByTestId("policy-revision");
    expect(revision.textContent).toContain("41");
    expect(revision.className).toContain("font-mono");

    fireEvent.click(screen.getByLabelText("audit"));
    fireEvent.click(screen.getByTestId("save-workflow"));
    await waitFor(() =>
      expect(ws.writeWorkflowConfig).toHaveBeenCalledWith({
        locale: "tw",
        specLocale: null,
        tdd: true,
        audit: true,
      }),
    );
    await waitFor(() => expect(screen.getByTestId("policy-revision").textContent).toContain("42"));
  });

  it("reader 看得到三卡現值但全唯讀，存檔停用並附繁中角色說明", async () => {
    const ws = remoteSettings(
      projectSnap({ revision: 9 } as Partial<SettingsSnapshot["workflow"]>),
      false,
    );
    render(<ProjectSettingsView settings={ws} />);

    await screen.findByRole("tab", { name: "Workflow" });
    expect(screen.getByTestId("context-card")).toBeTruthy();
    expect(screen.getByTestId("rules-card")).toBeTruthy();
    expect(screen.getByTestId("policy-card")).toBeTruthy();
    expect(screen.getByTestId("policy-reader-note").textContent).toContain("你的角色為檢視者");
    expect((screen.getByTestId("context-edit") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByTestId("rules-edit") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByLabelText("locale") as HTMLSelectElement).disabled).toBe(true);
    expect((screen.getByLabelText("tdd") as HTMLInputElement).disabled).toBe(true);
    expect((screen.getByTestId("save-workflow") as HTMLButtonElement).disabled).toBe(true);
    expect(ws.writeWorkflowConfig).not.toHaveBeenCalled();
  });

  it("409 保留原始輸入、重讀最新 revision 逐欄對照，且只提供兩個知情出口", async () => {
    const initial = projectSnap({ revision: 41 } as Partial<SettingsSnapshot["workflow"]>);
    const latest = projectSnap({
      revision: 42,
      context: "server 較新的說明",
      rules: { tasks: ["server rule"] },
    } as Partial<SettingsSnapshot["workflow"]>);
    const ws = remoteSettings(initial, true);
    (ws.readSettings as ReturnType<typeof vi.fn>)
      .mockReset()
      .mockResolvedValueOnce(initial)
      .mockResolvedValueOnce(latest);
    (ws.writeWorkflowContext as ReturnType<typeof vi.fn>).mockRejectedValueOnce({
      message: "policy revision conflict",
      reason: "revision_conflict",
      status: 409,
    });
    render(<ProjectSettingsView settings={ws} />);

    const contextCard = await screen.findByTestId("context-card");
    fireEvent.click(within(contextCard).getByTestId("context-edit"));
    fireEvent.change(within(contextCard).getByTestId("context-input"), {
      target: { value: "我的原始輸入\n保留換行" },
    });
    fireEvent.click(within(contextCard).getByTestId("context-save"));

    const panel = await screen.findByTestId("policy-conflict-panel");
    expect((within(contextCard).getByTestId("context-input") as HTMLTextAreaElement).value).toBe(
      "我的原始輸入\n保留換行",
    );
    expect(ws.readSettings).toHaveBeenCalledTimes(2);
    expect(within(panel).getByTestId("conflict-revision").textContent).toContain("42");
    const contextRow = within(panel).getByTestId("conflict-row-context");
    expect(within(contextRow).getByTestId("conflict-server").textContent).toContain(
      "server 較新的說明",
    );
    expect(within(contextRow).getByTestId("conflict-mine").textContent).toContain(
      "我的原始輸入",
    );
    expect(within(panel).getByTestId("conflict-row-rules-tasks")).toBeTruthy();
    expect(within(panel).getByTestId("conflict-row-audit")).toBeTruthy();
    expect(within(panel).getAllByRole("button").map((button) => button.textContent)).toEqual([
      "以 server 版重載",
      "檢視後以最新 revision 重新提交",
    ]);
    expect(screen.queryByText(/強制覆寫/)).toBeNull();

    fireEvent.click(within(panel).getByRole("button", { name: "以 server 版重載" }));
    await waitFor(() => expect(screen.queryByTestId("policy-conflict-panel")).toBeNull());
    expect(screen.getByTestId("context-readonly").textContent).toContain("server 較新的說明");
    expect(screen.getByTestId("policy-revision").textContent).toContain("42");
    expect(ws.writeWorkflowContext).toHaveBeenCalledTimes(1);
  });

  it("以最新 revision 重送仍遇 409 時遞迴重讀對照，輸入不變且無 force 路徑", async () => {
    const initial = projectSnap({ revision: 41 } as Partial<SettingsSnapshot["workflow"]>);
    const latest42 = projectSnap({
      revision: 42,
      locale: "ja",
    } as Partial<SettingsSnapshot["workflow"]>);
    const latest43 = projectSnap({
      revision: 43,
      locale: "tw",
    } as Partial<SettingsSnapshot["workflow"]>);
    const ws = remoteSettings(initial, true, 44);
    (ws.readSettings as ReturnType<typeof vi.fn>)
      .mockReset()
      .mockResolvedValueOnce(initial)
      .mockResolvedValueOnce(latest42)
      .mockResolvedValueOnce(latest43);
    (ws.writeWorkflowConfig as ReturnType<typeof vi.fn>)
      .mockReset()
      .mockRejectedValueOnce({ reason: "revision_conflict", status: 409, message: "conflict 41" })
      .mockRejectedValueOnce({ reason: "revision_conflict", status: 409, message: "conflict 42" })
      .mockResolvedValueOnce(44);
    render(<ProjectSettingsView settings={ws} />);

    const locale = (await screen.findByLabelText("locale")) as HTMLSelectElement;
    fireEvent.change(locale, { target: { value: "en" } });
    fireEvent.click(screen.getByTestId("save-workflow"));
    let panel = await screen.findByTestId("policy-conflict-panel");
    let localeRow = within(panel).getByTestId("conflict-row-locale");
    expect(within(localeRow).getByTestId("conflict-server").textContent).toContain("ja");
    expect(within(localeRow).getByTestId("conflict-mine").textContent).toContain("en");

    fireEvent.click(
      within(panel).getByRole("button", { name: "檢視後以最新 revision 重新提交" }),
    );
    await waitFor(() =>
      expect(screen.getByTestId("conflict-revision").textContent).toContain("43"),
    );
    panel = screen.getByTestId("policy-conflict-panel");
    localeRow = within(panel).getByTestId("conflict-row-locale");
    expect(within(localeRow).getByTestId("conflict-server").textContent).toContain("tw");
    expect(within(localeRow).getByTestId("conflict-mine").textContent).toContain("en");
    expect(locale.value).toBe("en");
    expect(ws.readSettings).toHaveBeenCalledTimes(3);
    expect(ws.writeWorkflowConfig).toHaveBeenCalledTimes(2);

    fireEvent.click(
      within(panel).getByRole("button", { name: "檢視後以最新 revision 重新提交" }),
    );
    await waitFor(() => expect(screen.queryByTestId("policy-conflict-panel")).toBeNull());
    expect(screen.getByTestId("policy-revision").textContent).toContain("44");
    expect(locale.value).toBe("en");
    expect(ws.writeWorkflowConfig).toHaveBeenCalledTimes(3);
    for (const call of (ws.writeWorkflowConfig as ReturnType<typeof vi.fn>).mock.calls) {
      expect(call[0]).toMatchObject({ locale: "en" });
    }
    expect(screen.queryByText(/強制覆寫/)).toBeNull();
  });
});
