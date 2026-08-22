// ProjectSettingsView（spec 需求「設定頁圖形化讀寫兩層設定」「設定頁編輯專案說明與產出規則」
// 的前端面）：兩頁簽組織（config.yaml／.speclink.yaml）、
// 專案說明與產出規則拆為獨立卡各持編輯態、解析失敗簽級警示。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { ProjectSettingsView } from "../views/ProjectSettingsView";
import { APP_MESSAGES } from "../i18n/messages";
import type { SchemaEntry, SettingsSnapshot } from "../adapter/workspace";
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
      worktree: false,
      context: null,
      rules: {},
      schemaArtifacts: ["proposal", "design", "specs", "tasks"],
      schemaName: "spec-driven",
      schemaKnown: true,
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
    readSchemas: vi.fn().mockResolvedValue([]),
    writeWorkflowSchema: vi.fn().mockResolvedValue(undefined),
    forkSchema: vi.fn().mockResolvedValue("spec-driven-custom"),
    createSchema: vi.fn().mockResolvedValue(undefined),
    revealSchema: vi.fn().mockResolvedValue(undefined),
    deleteSchema: vi.fn().mockResolvedValue(undefined),
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
      worktree: false,
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
    // 下拉是 Radix Select（非原生 select），現值讀 trigger 上顯示的文字。
    expect((await screen.findByLabelText("locale")).textContent).toContain("tw");
    // 未設定的 spec_locale 呈預設值狀態（空字串＝未設定）。
    expect(screen.getByLabelText("spec_locale").textContent).toContain("未設定");
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

describe("政策下拉未知值顯性呈現", () => {
  // spec 需求「設定頁政策下拉的未知值顯性呈現」（workflow-config-locale-validation）：
  // 選項集外的儲存值不得靜默空白，也不得於讀取時被改寫。
  it("儲存值不在選項集 → 下拉顯示原始值與無效標註、欄位下方浮出引導提示", async () => {
    const ws = renderView(projectSnap({ locale: "繁體中文", specLocale: "zh-Hant" }));
    const locale = await screen.findByLabelText("locale");
    expect(locale.textContent).toContain("繁體中文");
    expect(locale.textContent).toContain("無效");
    const specLocale = screen.getByLabelText("spec_locale");
    expect(specLocale.textContent).toContain("zh-Hant");
    expect(specLocale.textContent).toContain("無效");
    const localeHint = screen.getByTestId("locale-invalid-hint");
    for (const code of ["tw", "ja", "en"]) {
      expect(localeHint.textContent).toContain(code);
    }
    expect(screen.getByTestId("spec-locale-invalid-hint").textContent).toContain("auto");
    // 讀取不改寫：未按儲存前不得有任何寫入。
    expect(ws.writeWorkflowConfig).not.toHaveBeenCalled();
  });

  it("合法值與未設定 → 無任何無效標註或提示", async () => {
    renderView(projectSnap({ locale: "tw", specLocale: null }));
    expect((await screen.findByLabelText("locale")).textContent).not.toContain("無效");
    expect(screen.getByLabelText("spec_locale").textContent).not.toContain("無效");
    expect(screen.queryByTestId("locale-invalid-hint")).toBeNull();
    expect(screen.queryByTestId("spec-locale-invalid-hint")).toBeNull();
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
        worktree: false,
      }),
    );
  });

  it("worktree 開關切開後儲存 → writeWorkflowConfig 收到實值（不再恆為 false）", async () => {
    const ws = renderView(snapshot());
    fireEvent.click(await screen.findByLabelText("worktree"));
    fireEvent.click(screen.getByTestId("save-workflow"));
    await waitFor(() =>
      expect(ws.writeWorkflowConfig).toHaveBeenCalledWith({
        locale: "tw",
        specLocale: null,
        tdd: true,
        audit: false,
        worktree: true,
      }),
    );
  });

  it("worktree 開關載入時反映 config 現值", async () => {
    renderView(snapshot({ workflow: { worktree: true } } as Partial<SettingsSnapshot>));
    const box = (await screen.findByLabelText("worktree")) as HTMLElement;
    await waitFor(() => expect(box.getAttribute("data-state")).toBe("checked"));
  });

  it("關閉遇活躍 worktree → 擋下訊息浮出且開關回復開啟", async () => {
    const ws = renderView(snapshot({ workflow: { worktree: true } } as Partial<SettingsSnapshot>));
    const blocked =
      "add-auth 正在 worktree（speclink/add-auth）中進行，請先執行 speclink-worktree-merge 收尾再操作。";
    (ws.writeWorkflowConfig as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error(blocked));

    const box = (await screen.findByLabelText("worktree")) as HTMLElement;
    fireEvent.click(box);
    fireEvent.click(screen.getByTestId("save-workflow"));

    expect(await screen.findByText(new RegExp("speclink-worktree-merge"))).toBeTruthy();
    await waitFor(() => expect(box.getAttribute("data-state")).toBe("checked"));
  });

  it("技能同步失敗（config 已寫入）→ 開關維持新值，再存不回寫舊值", async () => {
    // 同步失敗的半套狀態：config 為正典（新值已落檔）、技能足跡過期。畫面
    // 不得退回舊快照——否則下次儲存會靜默把政策寫回去（連技能一起收走）。
    const ws = renderView(snapshot());
    (ws.writeWorkflowConfig as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
      new Error("workflow config written, but the skill footprint did not sync — run `speclink update` to rebuild"),
    );
    (ws.readSettings as ReturnType<typeof vi.fn>).mockResolvedValue(
      snapshot({ workflow: { worktree: true } } as Partial<SettingsSnapshot>),
    );

    const box = (await screen.findByLabelText("worktree")) as HTMLElement;
    fireEvent.click(box);
    fireEvent.click(screen.getByTestId("save-workflow"));

    expect(await screen.findByText(/did not sync/)).toBeTruthy();
    await waitFor(() => expect(box.getAttribute("data-state")).toBe("checked"));

    fireEvent.click(screen.getByTestId("save-workflow"));
    await waitFor(() =>
      expect(ws.writeWorkflowConfig).toHaveBeenLastCalledWith(
        expect.objectContaining({ worktree: true }),
      ),
    );
  });

  it("remote 存檔失敗（非 conflict）→ 不重讀設定，revision 維持舊值", async () => {
    // remote adapter 的 readSettings 會靜默採納最新 revision；失敗後重讀等於
    // 讓下一次儲存帶最新 revision 提交過期欄位值，繞過 409 衝突對話框靜默
    // 覆蓋他人併發修改。失敗重讀是 local 的 worktree 半套語意，remote 不適用。
    const snap = snapshot();
    const ws = {
      kind: "remote",
      policyWrite: true,
      readSettings: vi.fn().mockResolvedValue(snap),
      writeAppTools: vi.fn().mockResolvedValue(undefined),
      writeWorkflowConfig: vi.fn().mockRejectedValueOnce(new Error("server unreachable")),
      writeWorkflowContext: vi.fn().mockResolvedValue(undefined),
      writeWorkflowRules: vi.fn().mockResolvedValue(undefined),
      readSchemas: vi.fn().mockResolvedValue([]),
      writeWorkflowSchema: vi.fn().mockResolvedValue(undefined),
      forkSchema: vi.fn().mockRejectedValue(new Error("遠端工作區尚不支援 fork 產出流程")),
      createSchema: vi.fn().mockRejectedValue(new Error("遠端工作區尚不支援建立產出流程")),
      revealSchema: vi.fn().mockRejectedValue(new Error("遠端工作區沒有本機檔案可顯示")),
      deleteSchema: vi.fn().mockRejectedValue(new Error("遠端工作區尚不支援刪除產出流程")),
    } as unknown as WorkspaceSettingsProvider;
    render(<ProjectSettingsView settings={ws} />);

    fireEvent.click(await screen.findByLabelText("audit"));
    fireEvent.click(screen.getByTestId("save-workflow"));

    expect(await screen.findByText(/server unreachable/)).toBeTruthy();
    expect(ws.readSettings).toHaveBeenCalledTimes(1);
  });

  it("locale 下拉改值後儲存 → 新值進完整目標狀態；設回未設定送 null", async () => {
    const ws = renderView(snapshot());
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(await screen.findByLabelText("locale"));
    await user.click(await screen.findByRole("option", { name: /未設定/ }));
    fireEvent.click(screen.getByTestId("save-workflow"));
    await waitFor(() =>
      expect(ws.writeWorkflowConfig).toHaveBeenCalledWith({
        locale: null,
        specLocale: null,
        tdd: true,
        audit: false,
        worktree: false,
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
    // desktop-schema-panel D4 改版：產出流程為第二簽（spec「產出流程自成頁籤」）。
    expect(tabs).toEqual(["config.yaml", "Schema", ".speclink.yaml"]);
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
    // spec「錯誤態以紅呈現」：解析失敗是錯誤（表單因此停用），不是琥珀提醒；
    // 頁簽警示點仍是琥珀——它只說「這裡有事」，嚴重度由簽內橫幅承載。
    const banner = (await screen.findByText(/invalid yaml at line 3/)).closest("p") as HTMLElement;
    expect(banner.className).toContain("destructive");
    expect(banner.className).not.toContain("amber");
    expect(within(configTab).getByTestId("tab-warning").className).toContain("amber");
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
      readSchemas: vi.fn().mockResolvedValue([]),
      writeWorkflowSchema: vi.fn().mockResolvedValue(nextRevision),
      forkSchema: vi.fn().mockRejectedValue(new Error("遠端工作區尚不支援 fork 產出流程")),
      createSchema: vi.fn().mockRejectedValue(new Error("遠端工作區尚不支援建立產出流程")),
      revealSchema: vi.fn().mockRejectedValue(new Error("遠端工作區沒有本機檔案可顯示")),
      deleteSchema: vi.fn().mockRejectedValue(new Error("遠端工作區尚不支援刪除產出流程")),
    } as unknown as WorkspaceSettingsProvider;
  }

  it("editor 僅見 Workflow 簽、mono revision，政策可存且成功後 revision 前進", async () => {
    const snap = projectSnap({ revision: 41 } as Partial<SettingsSnapshot["workflow"]>);
    const ws = remoteSettings(snap, true);
    render(<ProjectSettingsView settings={ws} />);

    const tab = await screen.findByRole("tab", { name: "Workflow" });
    // remote 兩簽：Workflow＋產出流程（D4 改版）；.speclink.yaml 仍不出現。
    expect(screen.getAllByRole("tab").map((t) => t.textContent)).toEqual([
      "Workflow",
      "Schema",
    ]);
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
        worktree: false,
      }),
    );
    await waitFor(() => expect(screen.getByTestId("policy-revision").textContent).toContain("42"));
  });

  it("remote 工作區不顯示 worktree 開關（worktree facts 是 host-local 觀察）", async () => {
    // spec desktop-config「產出政策的 worktree 開關」Scenario「remote 工作區
    // 不顯示開關」：其餘政策欄位照常。
    const ws = remoteSettings(projectSnap({ revision: 7 } as Partial<SettingsSnapshot["workflow"]>));
    render(<ProjectSettingsView settings={ws} />);

    await screen.findByRole("tab", { name: "Workflow" });
    expect(screen.queryByLabelText("worktree")).toBeNull();
    expect(screen.getByLabelText("tdd")).toBeTruthy();
    expect(screen.getByLabelText("audit")).toBeTruthy();
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
    expect((screen.getByLabelText("locale") as HTMLButtonElement).disabled).toBe(true);
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
    // 政策衝突是警示（要人決定，不是壞掉）：琥珀邊框，不用主色（主色＝可點的東西）。
    expect(panel.className).toContain("amber");
    expect(panel.className).not.toContain("border-primary");
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

    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(await screen.findByLabelText("locale"));
    await user.click(await screen.findByRole("option", { name: /^en/ }));
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
    // 輸入不變：衝突面板來回後，下拉仍停在使用者選的值。
    expect(screen.getByLabelText("locale").textContent).toContain("en");
    expect(ws.readSettings).toHaveBeenCalledTimes(3);
    expect(ws.writeWorkflowConfig).toHaveBeenCalledTimes(2);

    fireEvent.click(
      within(panel).getByRole("button", { name: "檢視後以最新 revision 重新提交" }),
    );
    await waitFor(() => expect(screen.queryByTestId("policy-conflict-panel")).toBeNull());
    expect(screen.getByTestId("policy-revision").textContent).toContain("44");
    expect(screen.getByLabelText("locale").textContent).toContain("en");
    expect(ws.writeWorkflowConfig).toHaveBeenCalledTimes(3);
    for (const call of (ws.writeWorkflowConfig as ReturnType<typeof vi.fn>).mock.calls) {
      expect(call[0]).toMatchObject({ locale: "en" });
    }
    expect(screen.queryByText(/強制覆寫/)).toBeNull();
  });
});

// ---- 產出流程頁籤（desktop-schema-panel design D4 改版＋D5）----

describe("產出流程頁籤（spec「設定頁的產出流程頁籤」）", () => {
  const BUILTIN_ENTRY: SchemaEntry = {
    name: "spec-driven",
    source: "package",
    artifactIds: ["proposal", "design", "specs", "tasks"],
    artifacts: [
      { id: "proposal", description: "提案文件", instruction: "寫提案", template: "# Proposal template" },
      { id: "design", description: "設計文件", instruction: "寫設計", template: "# Design template" },
      { id: "specs", description: "規格文件", instruction: "寫規格", template: "# Spec template" },
      { id: "tasks", description: "任務清單", instruction: "寫任務", template: "# Tasks template" },
    ],
    path: null,
    error: null,
  };
  const CUSTOM_ENTRY: SchemaEntry = {
    name: "my-flow",
    source: "project",
    artifactIds: ["plan"],
    artifacts: [
      { id: "plan", description: "計畫文件", instruction: "先寫計畫", template: "# 計畫模板" },
    ],
    path: "/proj/openspec/schemas/my-flow",
    error: null,
  };

  function schemaSettings(
    snap: SettingsSnapshot,
    schemas: SchemaEntry[] = [BUILTIN_ENTRY, CUSTOM_ENTRY],
    over: Record<string, unknown> = {},
  ): WorkspaceSettingsProvider {
    return {
      kind: "local",
      policyWrite: true,
      readSettings: vi.fn().mockResolvedValue(snap),
      writeAppTools: vi.fn().mockResolvedValue(undefined),
      writeWorkflowConfig: vi.fn().mockResolvedValue(undefined),
      writeWorkflowContext: vi.fn().mockResolvedValue(undefined),
      writeWorkflowRules: vi.fn().mockResolvedValue(undefined),
      readSchemas: vi.fn().mockResolvedValue(schemas),
      writeWorkflowSchema: vi.fn().mockResolvedValue(undefined),
      forkSchema: vi.fn().mockResolvedValue("spec-driven-custom"),
      createSchema: vi.fn().mockResolvedValue(undefined),
      revealSchema: vi.fn().mockResolvedValue(undefined),
      deleteSchema: vi.fn().mockResolvedValue(undefined),
      ...over,
    } as unknown as WorkspaceSettingsProvider;
  }

  /** 切到產出流程頁籤（D4 改版：獨立頁籤，內容不再掛在 config.yaml 簽）。 */
  async function openSchemasTab() {
    await screen.findByRole("tab", { name: "Schema" });
    switchToTab("Schema");
  }

  it("local 頁簽依序 config.yaml／產出流程／.speclink.yaml，且 config.yaml 簽內無此節", async () => {
    // spec Scenario「產出流程自成頁籤」。
    const ws = schemaSettings(projectSnap());
    render(<ProjectSettingsView settings={ws} />);
    await screen.findByTestId("context-card");
    expect(screen.getAllByRole("tab").map((t) => t.textContent)).toEqual([
      "config.yaml",
      "Schema",
      ".speclink.yaml",
    ]);
    // 預設簽（config.yaml）內不得再有產出流程內容。
    expect(screen.queryByTestId("schema-card")).toBeNull();
    switchToTab("Schema");
    expect(await screen.findByTestId("schema-card")).toBeTruthy();
  });

  it("清單列出可解析項（名稱、來源層級、artifact 圖），點入唯讀詳情含全文且無編輯入口", async () => {
    // spec Scenario「清單列出可解析的 schema」＋「詳情唯讀呈現內容」＋
    // Example「清單一列的形狀」。
    const ws = schemaSettings(projectSnap());
    render(<ProjectSettingsView settings={ws} />);
    await openSchemasTab();
    const card = await screen.findByTestId("schema-card");
    expect(card.textContent).toContain("產出流程");
    const item = await screen.findByTestId("schema-item-spec-driven");
    expect(item.textContent).toContain("spec-driven");
    expect(item.textContent).toContain("內建");
    // spec Example「清單一列的形狀」：artifact 圖整串釘死（引擎顯示序）。
    expect(item.textContent).toContain("proposal → design → specs → tasks");
    const custom = screen.getByTestId("schema-item-my-flow");
    expect(custom.textContent).toContain("專案");
    expect(custom.textContent).toContain("plan");

    fireEvent.click(screen.getByTestId("schema-toggle-spec-driven"));
    const detail = await screen.findByTestId("schema-detail-spec-driven");
    for (const text of ["提案文件", "寫提案", "# Proposal template", "任務清單", "# Tasks template"]) {
      expect(detail.textContent).toContain(text);
    }
    expect(within(detail).queryByRole("textbox")).toBeNull();
  });

  it("下拉切換觸發寫入，成功後重讀快照且產出規則分節固定鍵更新", async () => {
    // spec Scenario「切換寫入且其餘內容保留」的前端面。
    const first = projectSnap();
    const after = projectSnap({ schemaName: "my-flow", schemaArtifacts: ["plan"], rules: {} });
    const ws = schemaSettings(first);
    (ws.readSettings as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce(first)
      .mockResolvedValue(after);
    render(<ProjectSettingsView settings={ws} />);
    await openSchemasTab();
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    const select = await screen.findByLabelText("使用中的產出流程");
    expect(select.textContent).toContain("spec-driven");
    await user.click(select);
    await user.click(await screen.findByRole("option", { name: /my-flow/ }));
    await waitFor(() => expect(ws.writeWorkflowSchema).toHaveBeenCalledWith("my-flow"));
    // 固定鍵隨新 schema 更新：切回 config.yaml 簽，產出規則編輯分節只剩 plan。
    await waitFor(() => expect(screen.getByLabelText("使用中的產出流程").textContent).toContain("my-flow"));
    switchToTab("config.yaml");
    fireEvent.click(await screen.findByTestId("rules-edit"));
    await waitFor(() => expect(screen.getByTestId("rules-input-plan")).toBeTruthy());
    expect(screen.queryByTestId("rules-input-proposal")).toBeNull();
  });

  it("切換寫入失敗：錯誤浮出於表單，不靜默", async () => {
    const ws = schemaSettings(projectSnap());
    (ws.writeWorkflowSchema as ReturnType<typeof vi.fn>).mockRejectedValue(
      "openspec/config.yaml: write failed: denied",
    );
    render(<ProjectSettingsView settings={ws} />);
    await openSchemasTab();
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(await screen.findByLabelText("使用中的產出流程"));
    await user.click(await screen.findByRole("option", { name: /my-flow/ }));
    expect(await screen.findByText(/write failed/)).toBeTruthy();
  });

  it("fork 僅 local 渲染，按下後清單反映新專案層項目", async () => {
    // spec Scenario「fork 產出專案層複本」的前端面。
    const forked: SchemaEntry = { ...BUILTIN_ENTRY, name: "spec-driven-custom", source: "project" };
    const ws = schemaSettings(projectSnap());
    (ws.readSchemas as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce([BUILTIN_ENTRY, CUSTOM_ENTRY])
      .mockResolvedValue([BUILTIN_ENTRY, forked, CUSTOM_ENTRY]);
    render(<ProjectSettingsView settings={ws} />);
    await openSchemasTab();
    await screen.findByTestId("schema-item-spec-driven");
    fireEvent.click(screen.getByTestId("schema-fork-spec-driven"));
    await waitFor(() => expect(ws.forkSchema).toHaveBeenCalledWith("spec-driven"));
    await screen.findByTestId("schema-item-spec-driven-custom");
  });

  it("存產出規則保留非固定鍵的既有分節（review R3：切換後舊 schema 分節不被靜默刪除）", async () => {
    const snap = projectSnap({
      schemaName: "my-flow",
      schemaArtifacts: ["plan"],
      rules: { plan: ["p1"], proposal: ["舊 schema 的規則"] },
    });
    const ws = schemaSettings(snap);
    render(<ProjectSettingsView settings={ws} />);
    fireEvent.click(await screen.findByTestId("rules-edit"));
    await screen.findByTestId("rules-input-plan");
    fireEvent.click(screen.getByTestId("rules-save"));
    await waitFor(() =>
      expect(ws.writeWorkflowRules).toHaveBeenCalledWith([
        ["plan", ["p1"]],
        ["proposal", ["舊 schema 的規則"]],
      ]),
    );
  });

  it("切換 schema 不重設另一卡的編輯態與草稿（review R4）", async () => {
    const first = projectSnap();
    const after = projectSnap({ schemaName: "my-flow", schemaArtifacts: ["plan"], rules: {} });
    const ws = schemaSettings(first);
    (ws.readSettings as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce(first)
      .mockResolvedValue(after);
    render(<ProjectSettingsView settings={ws} />);
    // config.yaml 簽：開專案說明編輯並輸入未存草稿。
    fireEvent.click(await screen.findByTestId("context-edit"));
    fireEvent.change(screen.getByTestId("context-input"), {
      target: { value: "還沒存的草稿" },
    });
    // 切到 Schema 簽做切換。
    switchToTab("Schema");
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(await screen.findByLabelText("使用中的產出流程"));
    await user.click(await screen.findByRole("option", { name: /my-flow/ }));
    await waitFor(() => expect(ws.writeWorkflowSchema).toHaveBeenCalledWith("my-flow"));
    // 切回 config.yaml 簽：編輯態仍開、草稿仍在。
    switchToTab("config.yaml");
    const input = (await screen.findByTestId("context-input")) as HTMLTextAreaElement;
    expect(input.value).toBe("還沒存的草稿");
  });

  it("remote 切換撞 revision_conflict → 開對照對話框（review R5）", async () => {
    const initial = projectSnap({ revision: 41 } as Partial<SettingsSnapshot["workflow"]>);
    const latest = projectSnap({ revision: 42 } as Partial<SettingsSnapshot["workflow"]>);
    const ws = schemaSettings(initial, [BUILTIN_ENTRY, CUSTOM_ENTRY], { kind: "remote" });
    (ws.readSettings as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce(initial)
      .mockResolvedValue(latest);
    (ws.writeWorkflowSchema as ReturnType<typeof vi.fn>).mockRejectedValue({
      reason: "revision_conflict",
      message: "policy revision conflict",
    });
    render(<ProjectSettingsView settings={ws} />);
    await openSchemasTab();
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(await screen.findByLabelText("使用中的產出流程"));
    await user.click(await screen.findByRole("option", { name: /my-flow/ }));
    expect(await screen.findByTestId("policy-conflict-panel")).toBeTruthy();
    expect(screen.getByTestId("conflict-revision").textContent).toContain("42");
  });

  it("下拉現值不在選項集時仍顯示名稱（review R6：remote 非內建）", async () => {
    const snap = projectSnap({
      revision: 7,
      schemaName: "their-flow",
      schemaKnown: false,
      schemaArtifacts: [],
      rules: {},
    } as Partial<SettingsSnapshot["workflow"]>);
    const ws = schemaSettings(snap, [BUILTIN_ENTRY], { kind: "remote" });
    render(<ProjectSettingsView settings={ws} />);
    await openSchemasTab();
    const trigger = await screen.findByLabelText("使用中的產出流程");
    expect(trigger.textContent).toContain("their-flow");
  });

  it("readSchemas 失敗：錯誤浮出而非靜默空白（review R7）", async () => {
    const ws = schemaSettings(projectSnap());
    (ws.readSchemas as ReturnType<typeof vi.fn>).mockRejectedValue(
      "not a speclink project: /gone",
    );
    render(<ProjectSettingsView settings={ws} />);
    await openSchemasTab();
    expect(await screen.findByText(/not a speclink project/)).toBeTruthy();
  });

  it("同名跨層項目只有解析命中的那層有 fork 按鈕（review R8）", async () => {
    // 引擎 fork 以 project→user 第一命中解析：被 shadow 的 user 層項若給 fork，
    // 複製到的會是專案層同名內容——不給入口。
    const dupProject: SchemaEntry = { ...CUSTOM_ENTRY, name: "dup", path: "/proj/openspec/schemas/dup" };
    const dupUser: SchemaEntry = {
      ...CUSTOM_ENTRY,
      name: "dup",
      source: "user",
      path: "/home/userdir/schemas/dup",
    };
    const ws = schemaSettings(projectSnap(), [BUILTIN_ENTRY, dupProject, dupUser]);
    render(<ProjectSettingsView settings={ws} />);
    await openSchemasTab();
    await waitFor(() => expect(screen.getAllByTestId("schema-item-dup")).toHaveLength(2));
    expect(screen.getAllByTestId("schema-fork-dup")).toHaveLength(1);
  });

  it("建立表單僅 local 渲染：輸入名稱送出後 createSchema 被呼叫且清單反映新項", async () => {
    // spec Scenario「建立產出專案層骨架」＋ Example「建立輸入與結果」row 1。
    const created: SchemaEntry = {
      name: "my-new-flow",
      source: "project",
      artifactIds: ["plan", "tasks"],
      artifacts: [],
      path: "/proj/openspec/schemas/my-new-flow",
      error: null,
    };
    const ws = schemaSettings(projectSnap());
    (ws.readSchemas as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce([BUILTIN_ENTRY, CUSTOM_ENTRY])
      .mockResolvedValue([BUILTIN_ENTRY, CUSTOM_ENTRY, created]);
    render(<ProjectSettingsView settings={ws} />);
    await openSchemasTab();
    await screen.findByTestId("schema-item-spec-driven");
    fireEvent.change(screen.getByTestId("schema-create-name"), {
      target: { value: "my-new-flow" },
    });
    fireEvent.click(screen.getByTestId("schema-create"));
    await waitFor(() => expect(ws.createSchema).toHaveBeenCalledWith("my-new-flow"));
    await screen.findByTestId("schema-item-my-new-flow");
  });

  it("有磁碟路徑的項目帶開啟所在資料夾按鈕，內建項無（產出流程的編輯入口）", async () => {
    // spec Scenario「專案層項目開啟所在資料夾」＋「內建項無編輯入口」。
    const ws = schemaSettings(projectSnap());
    render(<ProjectSettingsView settings={ws} />);
    await openSchemasTab();
    await screen.findByTestId("schema-item-my-flow");
    expect(screen.queryByTestId("schema-reveal-spec-driven")).toBeNull();
    fireEvent.click(screen.getByTestId("schema-reveal-my-flow"));
    await waitFor(() =>
      expect(ws.revealSchema).toHaveBeenCalledWith("/proj/openspec/schemas/my-flow"),
    );
  });

  it("專案層項目帶刪除按鈕：確認後呼叫且清單反映，內建項無此按鈕", async () => {
    // spec Scenario「刪除經確認後移除專案層目錄」。
    const ws = schemaSettings(projectSnap());
    (ws.readSchemas as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce([BUILTIN_ENTRY, CUSTOM_ENTRY])
      .mockResolvedValue([BUILTIN_ENTRY]);
    render(<ProjectSettingsView settings={ws} />);
    await openSchemasTab();
    await screen.findByTestId("schema-item-my-flow");
    expect(screen.queryByTestId("schema-delete-spec-driven")).toBeNull();
    fireEvent.click(screen.getByTestId("schema-delete-my-flow"));
    fireEvent.click(await screen.findByTestId("schema-delete-confirm"));
    await waitFor(() => expect(ws.deleteSchema).toHaveBeenCalledWith("my-flow"));
    await waitFor(() => expect(screen.queryByTestId("schema-item-my-flow")).toBeNull());
  });

  it("刪除確認對話框取消：零呼叫零變動", async () => {
    // spec Scenario「取消確認零變動」。
    const ws = schemaSettings(projectSnap());
    render(<ProjectSettingsView settings={ws} />);
    await openSchemasTab();
    await screen.findByTestId("schema-item-my-flow");
    fireEvent.click(screen.getByTestId("schema-delete-my-flow"));
    fireEvent.click(await screen.findByTestId("schema-delete-cancel"));
    expect(ws.deleteSchema).not.toHaveBeenCalled();
    expect(screen.getByTestId("schema-item-my-flow")).toBeTruthy();
  });

  it("刪除失敗（使用中的 schema 拒刪）：錯誤浮出於表單", async () => {
    const ws = schemaSettings(projectSnap());
    (ws.deleteSchema as ReturnType<typeof vi.fn>).mockRejectedValue(
      "'my-flow' 是使用中的產出流程（config.yaml 的 schema 鍵正指著它）——請先切換到其他項目再刪除",
    );
    render(<ProjectSettingsView settings={ws} />);
    await openSchemasTab();
    await screen.findByTestId("schema-item-my-flow");
    fireEvent.click(screen.getByTestId("schema-delete-my-flow"));
    fireEvent.click(await screen.findByTestId("schema-delete-confirm"));
    expect(await screen.findByText(/使用中的產出流程/)).toBeTruthy();
    expect(screen.getByTestId("schema-item-my-flow")).toBeTruthy();
  });

  it("fork／建立／刪除不重設產出規則的編輯態與草稿（review N1）", async () => {
    const ws = schemaSettings(projectSnap());
    render(<ProjectSettingsView settings={ws} />);
    // config.yaml 簽：開產出規則編輯並輸入未存草稿。
    fireEvent.click(await screen.findByTestId("rules-edit"));
    fireEvent.change(screen.getByTestId("rules-input-proposal"), {
      target: { value: "還沒存的規則草稿" },
    });
    // Schema 簽做 fork。
    switchToTab("Schema");
    await screen.findByTestId("schema-item-spec-driven");
    fireEvent.click(screen.getByTestId("schema-fork-spec-driven"));
    await waitFor(() => expect(ws.forkSchema).toHaveBeenCalled());
    // 切回：編輯態仍開、草稿仍在。
    switchToTab("config.yaml");
    const input = (await screen.findByTestId("rules-input-proposal")) as HTMLTextAreaElement;
    expect(input.value).toBe("還沒存的規則草稿");
  });

  it("編輯中換固定鍵集：編輯面凍結在開編輯當下的分節，儲存不清掉新 schema 的既有規則（review N4）", async () => {
    const first = projectSnap(); // 固定鍵 proposal/design/specs/tasks
    const after = projectSnap({
      schemaName: "my-flow",
      schemaArtifacts: ["plan"],
      rules: { plan: ["新 schema 的既有規則"], proposal: ["提案必須列出影響的 crates"] },
    });
    const ws = schemaSettings(first);
    (ws.readSettings as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce(first)
      .mockResolvedValue(after);
    render(<ProjectSettingsView settings={ws} />);
    // 開產出規則編輯並改草稿。
    fireEvent.click(await screen.findByTestId("rules-edit"));
    fireEvent.change(screen.getByTestId("rules-input-proposal"), {
      target: { value: "編輯中的提案規則" },
    });
    // Schema 簽 fork → refreshSchemaFacts 換集（schemaArtifacts 變 ["plan"]）。
    switchToTab("Schema");
    await screen.findByTestId("schema-item-spec-driven");
    fireEvent.click(screen.getByTestId("schema-fork-spec-driven"));
    await waitFor(() => expect(ws.forkSchema).toHaveBeenCalled());
    // 切回：編輯面仍是開編輯當下的分節與草稿（不是新鍵的空白 textarea）。
    switchToTab("config.yaml");
    const input = (await screen.findByTestId("rules-input-proposal")) as HTMLTextAreaElement;
    expect(input.value).toBe("編輯中的提案規則");
    expect(screen.queryByTestId("rules-input-plan")).toBeNull();
    // 儲存：payload 以凍結鍵集送出草稿，新固定鍵 plan 的既有規則以兜底原樣保留。
    fireEvent.click(screen.getByTestId("rules-save"));
    await waitFor(() => expect(ws.writeWorkflowRules).toHaveBeenCalled());
    const payload = (ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mock
      .calls[0][0] as Array<[string, string[]]>;
    expect(payload).toContainEqual(["proposal", ["編輯中的提案規則"]]);
    expect(payload).toContainEqual(["plan", ["新 schema 的既有規則"]]);
  });

  it("切換失敗後下拉退回現值，不謊報活躍 schema（review N2）", async () => {
    const ws = schemaSettings(projectSnap());
    (ws.writeWorkflowSchema as ReturnType<typeof vi.fn>).mockRejectedValue(
      "openspec/config.yaml: write failed: denied",
    );
    render(<ProjectSettingsView settings={ws} />);
    await openSchemasTab();
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(await screen.findByLabelText("使用中的產出流程"));
    await user.click(await screen.findByRole("option", { name: /my-flow/ }));
    await screen.findByText(/write failed/);
    expect(screen.getByLabelText("使用中的產出流程").textContent).toContain("spec-driven");
  });

  it("被壞檔前層 shadow 的項也不給 fork 入口（review R8 殘留）", async () => {
    // 引擎 sources 只看 schema.yaml 檔案存在——前層壞檔仍是解析命中層。
    const brokenProject: SchemaEntry = {
      name: "dup",
      source: "project",
      artifactIds: [],
      artifacts: [],
      path: "/proj/openspec/schemas/dup",
      error: "Schema parse error: bad yaml",
    };
    const dupUser: SchemaEntry = {
      ...CUSTOM_ENTRY,
      name: "dup",
      source: "user",
      path: "/home/userdir/schemas/dup",
    };
    const ws = schemaSettings(projectSnap(), [BUILTIN_ENTRY, brokenProject, dupUser]);
    render(<ProjectSettingsView settings={ws} />);
    await openSchemasTab();
    await waitFor(() => expect(screen.getAllByTestId("schema-item-dup")).toHaveLength(2));
    expect(screen.queryAllByTestId("schema-fork-dup")).toHaveLength(0);
  });

  it("建立失敗（不合法名稱／已存在）：引擎錯誤浮出於表單", async () => {
    // spec Scenario「不合法名稱顯性失敗」＋ Example「建立輸入與結果」row 2/3。
    const ws = schemaSettings(projectSnap());
    (ws.createSchema as ReturnType<typeof vi.fn>).mockRejectedValue(
      "Invalid schema name 'My Flow': must be lowercase kebab-case (e.g. my-flow)",
    );
    render(<ProjectSettingsView settings={ws} />);
    await openSchemasTab();
    fireEvent.change(await screen.findByTestId("schema-create-name"), {
      target: { value: "My Flow" },
    });
    fireEvent.click(screen.getByTestId("schema-create"));
    expect(await screen.findByText(/kebab-case/)).toBeTruthy();
  });

  it("remote：頁簽序 Workflow／產出流程、無 fork 與建立入口、下拉僅內建、非內建顯示遠端自訂尚不支援且不猜固定鍵", async () => {
    // spec Scenario「remote 模式無 fork 入口」「remote 模式無建立入口」「remote
    // 下拉僅內建」「remote 快照不讀本機 user 層」的前端面。
    const snap = projectSnap({
      revision: 7,
      schemaName: "their-flow",
      schemaKnown: false,
      schemaArtifacts: [],
      rules: {},
    } as Partial<SettingsSnapshot["workflow"]>);
    const ws = schemaSettings(snap, [BUILTIN_ENTRY], { kind: "remote" });
    render(<ProjectSettingsView settings={ws} />);
    await screen.findByTestId("context-card");
    expect(screen.getAllByRole("tab").map((t) => t.textContent)).toEqual([
      "Workflow",
      "Schema",
    ]);
    // 不猜固定鍵：Workflow 簽的產出規則卡無任何分節內容。
    expect(screen.queryByTestId("rules-readonly-proposal")).toBeNull();

    switchToTab("Schema");
    await screen.findByTestId("schema-card");
    expect(screen.queryByTestId("schema-fork-spec-driven")).toBeNull();
    expect(screen.queryByTestId("schema-create")).toBeNull();
    expect(screen.queryByTestId("schema-reveal-spec-driven")).toBeNull();
    expect(screen.queryByTestId("schema-delete-spec-driven")).toBeNull();
    expect(screen.getByTestId("schema-unknown-note").textContent).toContain("遠端自訂尚不支援");

    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(screen.getByLabelText("使用中的產出流程"));
    const options = await screen.findAllByRole("option");
    // 沿 locale 未知值模式：現值以停用項顯示（不可選），可選的切換目標僅內建。
    expect(
      options.map((o) => [o.textContent, o.getAttribute("aria-disabled") === "true"]),
    ).toEqual([
      ["their-flow", true],
      ["spec-driven", false],
    ]);
  });
});
