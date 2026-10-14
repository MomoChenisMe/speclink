// ProjectSettingsView（spec 需求「設定頁圖形化讀寫兩層設定」「設定頁編輯專案說明與產出規則」
// 的前端面；desktop-settings-mode design D8）：依專案設定分類渲染四頁（專案說明、產出規則、
// 產出流程、.speclink.yaml），各頁獨立編輯態、解析失敗頁頂說明。
import { describe, it, expect, vi } from "vitest";
import { act, render as rtlRender, screen, fireEvent, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState, type ReactElement, type ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { ProjectSettingsView } from "../views/ProjectSettingsView";
import { APP_MESSAGES } from "../i18n/messages";
import type { ReviewRules, SchemaEntry, SettingsSnapshot } from "../adapter/workspace";
import type { WorkspaceSettingsProvider } from "../session";
import type { ProjectSettingsSection } from "../settingsCatalog";
import { editorText, editorView, pressSave, typeInEditor } from "./helpers/codemirror";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

/** 引擎提供的 12 個 smell 名稱（快照 reviewBaselineSmells 的測試值）。 */
const SMELLS = ["Mysterious Name", "Duplicated Code", "Feature Envy", "Data Clumps", "Primitive Obsession", "Repeated Switches", "Shotgun Surgery", "Divergent Change", "Speculative Generality", "Message Chains", "Middle Man", "Refused Bequest"];

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
      review: { baseline: "extend", critical: [], warning: [], suggestion: [] },
      reviewBaselineSmells: SMELLS,
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
    writeWorkflowReview: vi.fn().mockResolvedValue(undefined),
    readSchemas: vi.fn().mockResolvedValue([]),
    writeWorkflowSchema: vi.fn().mockResolvedValue(undefined),
    forkSchema: vi.fn().mockResolvedValue("spec-driven-custom"),
    createSchema: vi.fn().mockResolvedValue(undefined),
    revealSchema: vi.fn().mockResolvedValue(undefined),
    deleteSchema: vi.fn().mockResolvedValue(undefined),
  } as unknown as WorkspaceSettingsProvider;
}

/** 分類欄的替身：所在分類放在 state，goto 切頁（狀態與草稿跨頁保留）。 */
const nav = { goto: (_section: ProjectSettingsSection) => {} };
function Harness({
  settings,
  initial = "context",
  onParseErrors,
  onDirtyChange,
}: {
  settings: WorkspaceSettingsProvider;
  initial?: ProjectSettingsSection;
  onParseErrors?: (errors: { config: boolean; speclink: boolean }) => void;
  onDirtyChange?: (dirty: boolean) => void;
}) {
  const [section, setSection] = useState<ProjectSettingsSection>(initial);
  nav.goto = setSection;
  return (
    <ProjectSettingsView
      settings={settings}
      section={section}
      onParseErrors={onParseErrors}
      onDirtyChange={onDirtyChange}
    />
  );
}
function goto(section: ProjectSettingsSection) {
  act(() => nav.goto(section));
}

function renderView(snap: SettingsSnapshot, initial: ProjectSettingsSection = "context") {
  const ws = fakeSettings(snap);
  render(<Harness settings={ws} initial={initial} />);
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

describe("ProjectSettingsView 載入", () => {
  it("呈現兩檔現值：產出流程頁的政策欄位、.speclink.yaml 頁的 tools 開關；未設定欄位呈預設狀態", async () => {
    renderView(snapshot(), "workflow");
    // 下拉是 Radix Select（非原生 select），現值讀 trigger 上顯示的文字。
    expect((await screen.findByLabelText("locale")).textContent).toContain("tw");
    // 未設定的 spec_locale 呈預設值狀態（空字串＝未設定）。
    expect(screen.getByLabelText("spec_locale").textContent).toContain("未設定");
    // tdd／audit 為列右側開關（role="switch"）。
    expect(screen.getByRole("switch", { name: "tdd" }).getAttribute("aria-checked")).toBe("true");
    expect(screen.getByRole("switch", { name: "audit" }).getAttribute("aria-checked")).toBe("false");
    goto("speclink");
    expect(screen.getByLabelText("claude").getAttribute("aria-checked")).toBe("true");
    expect(screen.getByLabelText("codex").getAttribute("aria-checked")).toBe("false");
    expect(screen.getByLabelText("copilot").getAttribute("aria-checked")).toBe("false");
  });

  it("自訂工具描述子呈現為不可編輯項", async () => {
    renderView(snapshot({ app: { tools: ["claude"], customTools: ["wad-harness"], parseError: null } }), "speclink");
    const custom = await screen.findByText("wad-harness");
    // 不可編輯：非表單輸入（無對應開關）。
    expect(screen.queryByRole("switch", { name: "wad-harness" })).toBeNull();
    expect(custom).toBeTruthy();
  });
});

describe("政策下拉未知值顯性呈現", () => {
  // spec 需求「設定頁政策下拉的未知值顯性呈現」（workflow-config-locale-validation）：
  // 選項集外的儲存值不得靜默空白，也不得於讀取時被改寫。
  it("儲存值不在選項集 → 下拉顯示原始值與無效標註、欄位下方浮出引導提示", async () => {
    const ws = renderView(projectSnap({ locale: "繁體中文", specLocale: "zh-Hant" }), "workflow");
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
    renderView(projectSnap({ locale: "tw", specLocale: null }), "workflow");
    expect((await screen.findByLabelText("locale")).textContent).not.toContain("無效");
    expect(screen.getByLabelText("spec_locale").textContent).not.toContain("無效");
    expect(screen.queryByTestId("locale-invalid-hint")).toBeNull();
    expect(screen.queryByTestId("spec-locale-invalid-hint")).toBeNull();
  });
});

describe("ProjectSettingsView 寫入", () => {
  it("tools 加選 codex 後儲存 → writeAppTools 收到完整選集", async () => {
    const ws = renderView(snapshot(), "speclink");
    fireEvent.click(await screen.findByLabelText("codex"));
    fireEvent.click(screen.getByTestId("save-app"));
    await waitFor(() => expect(ws.writeAppTools).toHaveBeenCalledWith(["claude", "codex"]));
  });

  it("tools 加選 copilot 後儲存 → writeAppTools 收到完整選集", async () => {
    // spec「設定頁圖形化讀寫兩層設定」：AI 工具卡多選含 copilot。
    const ws = renderView(snapshot(), "speclink");
    fireEvent.click(await screen.findByLabelText("copilot"));
    fireEvent.click(screen.getByTestId("save-app"));
    await waitFor(() => expect(ws.writeAppTools).toHaveBeenCalledWith(["claude", "copilot"]));
  });

  it("audit 切開後儲存 → writeWorkflowConfig 收到完整目標狀態（含讀入現值）", async () => {
    const ws = renderView(snapshot(), "workflow");
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
    const ws = renderView(snapshot(), "workflow");
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
    renderView(snapshot({ workflow: { worktree: true } } as Partial<SettingsSnapshot>), "workflow");
    const box = (await screen.findByLabelText("worktree")) as HTMLElement;
    await waitFor(() => expect(box.getAttribute("data-state")).toBe("checked"));
  });

  it("關閉遇活躍 worktree → 擋下訊息浮出且開關回復開啟", async () => {
    const ws = renderView(snapshot({ workflow: { worktree: true } } as Partial<SettingsSnapshot>), "workflow");
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
    const ws = renderView(snapshot(), "workflow");
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
    render(<Harness settings={ws} initial="workflow" />);

    fireEvent.click(await screen.findByLabelText("audit"));
    fireEvent.click(screen.getByTestId("save-workflow"));

    expect(await screen.findByText(/server unreachable/)).toBeTruthy();
    expect(ws.readSettings).toHaveBeenCalledTimes(1);
  });

  it("locale 下拉改值後儲存 → 新值進完整目標狀態；設回未設定送 null", async () => {
    const ws = renderView(snapshot(), "workflow");
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
    const ws = renderView(snapshot(), "workflow");
    (ws.writeWorkflowConfig as ReturnType<typeof vi.fn>).mockRejectedValue(
      "openspec/config.yaml: write failed: denied",
    );
    fireEvent.click(await screen.findByLabelText("tdd"));
    fireEvent.click(screen.getByTestId("save-workflow"));
    expect(await screen.findByText(/write failed/)).toBeTruthy();
  });
});

// ---- 分類頁組織（spec 需求「設定頁圖形化讀寫兩層設定」；desktop-settings-mode design D8）----

describe("專案設定分類頁（spec Scenario「兩種設定模式與預設分類」）", () => {
  it("專案說明頁標題下等寬註記檔案與鍵；產出流程頁依序為產出政策卡與產出流程卡；.speclink.yaml 頁有檔案列與 AI 工具卡", async () => {
    renderView(projectSnap());
    const page = await screen.findByTestId("context-page");
    expect(within(page).getByRole("heading", { level: 2, name: "專案說明" })).toBeTruthy();
    const note = screen.getByTestId("file-note-config");
    expect(note.textContent).toBe("openspec/config.yaml · context");
    expect(note.className).toContain("font-mono");
    // 其他頁的內容不在畫面上。
    expect(screen.queryByTestId("policy-card")).toBeNull();
    expect(screen.queryByLabelText("claude")).toBeNull();

    goto("rules");
    expect(screen.getByTestId("file-note-config").textContent).toBe("openspec/config.yaml · rules");

    goto("workflow");
    const regions = screen.getAllByRole("region").map((r) => r.getAttribute("aria-label"));
    expect(regions).toEqual(["產出政策", "產出流程"]);
    expect(screen.getByRole("heading", { level: 2, name: "產出流程" })).toBeTruthy();
    // 專案說明頁與產出規則頁不含產出流程內容（spec「產出流程頁」）。
    expect(screen.queryByTestId("context-page")).toBeNull();

    goto("speclink");
    expect(screen.getByRole("region", { name: "AI 工具" })).toBeTruthy();
    expect(screen.getByLabelText("claude").getAttribute("aria-checked")).toBe("true");
    expect(screen.getByTestId("file-note-speclink").className).toContain("font-mono");
  });

  it(".speclink.yaml 頁的檔案列為等寬唯讀路徑", async () => {
    renderView(projectSnap(), "speclink");
    const row = await screen.findByRole("group", { name: "檔案" });
    const path = within(row).getByTestId("speclink-file-path");
    expect(path.textContent).toBe(".speclink.yaml");
    expect(path.className).toContain("font-mono");
    expect(within(row).queryByRole("textbox")).toBeNull();
  });

  it("tdd 以開關呈現，切換後存檔寫回 true", async () => {
    const ws = renderView(snapshot({ workflow: { tdd: false } } as Partial<SettingsSnapshot>), "workflow");
    const tdd = await screen.findByRole("switch", { name: "tdd" });
    expect(tdd.getAttribute("aria-checked")).toBe("false");
    fireEvent.click(tdd);
    fireEvent.click(screen.getByTestId("save-workflow"));
    await waitFor(() =>
      expect(ws.writeWorkflowConfig).toHaveBeenCalledWith(expect.objectContaining({ tdd: true })),
    );
  });

  it("config.yaml 解析失敗：回報分類欄警示、三頁浮出解析失敗說明（紅）、表單與編輯鈕停用", async () => {
    const onParseErrors = vi.fn();
    const ws = fakeSettings(
      projectSnap({ context: null, rules: {}, schemaArtifacts: [], parseError: "invalid yaml at line 3" }),
    );
    render(<Harness settings={ws} onParseErrors={onParseErrors} />);
    await waitFor(() => expect(onParseErrors).toHaveBeenLastCalledWith({ config: true, speclink: false }));
    // spec「錯誤態以紅呈現」：解析失敗是錯誤（表單因此停用），不是琥珀提醒。
    const banner = (await screen.findByText(/invalid yaml at line 3/)).closest("p") as HTMLElement;
    expect(banner.className).toContain("destructive");
    expect(banner.className).not.toContain("status-warning");
    expect((screen.getByTestId("context-edit") as HTMLButtonElement).disabled).toBe(true);
    goto("rules");
    expect(screen.getByText(/invalid yaml at line 3/)).toBeTruthy();
    expect((screen.getByTestId("rules-edit") as HTMLButtonElement).disabled).toBe(true);
    goto("workflow");
    expect(screen.getByText(/invalid yaml at line 3/)).toBeTruthy();
    expect((screen.getByTestId("save-workflow") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByLabelText("tdd") as HTMLButtonElement).disabled).toBe(true);
  });

  it("卸載時清掉回報：分類欄不沿用上一個專案的警示點，返回也不再問未存修改", async () => {
    const onParseErrors = vi.fn();
    const onDirtyChange = vi.fn();
    const ws = fakeSettings(projectSnap({ parseError: "invalid yaml at line 3" }));
    const { unmount } = render(<Harness settings={ws} onParseErrors={onParseErrors} onDirtyChange={onDirtyChange} />);
    await waitFor(() => expect(onParseErrors).toHaveBeenLastCalledWith({ config: true, speclink: false }));
    unmount();
    expect(onParseErrors).toHaveBeenLastCalledWith({ config: false, speclink: false });
    expect(onDirtyChange).toHaveBeenLastCalledWith(false);
  });

  it(".speclink.yaml 解析失敗：回報分類欄警示、AI 工具表單停用；產出流程頁表單照常可用", async () => {
    const onParseErrors = vi.fn();
    const ws = fakeSettings(snapshot({ app: { tools: [], customTools: [], parseError: "bad tools yaml" } }));
    render(<Harness settings={ws} initial="workflow" onParseErrors={onParseErrors} />);
    await waitFor(() => expect(onParseErrors).toHaveBeenLastCalledWith({ config: false, speclink: true }));
    expect((screen.getByTestId("save-workflow") as HTMLButtonElement).disabled).toBe(false);
    goto("speclink");
    expect(await screen.findByText(/bad tools yaml/)).toBeTruthy();
    expect((screen.getByTestId("save-app") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByLabelText("claude") as HTMLButtonElement).disabled).toBe(true);
  });
});

// ---- 專案說明卡／產出規則卡（spec 需求「設定頁編輯專案說明與產出規則」；design D2）----

// 未存修改回報（App 的返回動作據此先問要不要捨棄）：表單值對照檔案現值，
// 文字卡只看編輯中的草稿；儲存成功即回到 false。
describe("未存修改回報", () => {
  const dirtyHarness = (initial: ProjectSettingsSection) => {
    const onDirtyChange = vi.fn();
    const ws = fakeSettings(projectSnap());
    render(<Harness settings={ws} initial={initial} onDirtyChange={onDirtyChange} />);
    return onDirtyChange;
  };

  it("改政策開關即 true，儲存成功後回 false", async () => {
    const onDirtyChange = dirtyHarness("workflow");
    await screen.findByTestId("save-workflow");
    expect(onDirtyChange).toHaveBeenLastCalledWith(false);
    fireEvent.click(screen.getByLabelText("tdd"));
    await waitFor(() => expect(onDirtyChange).toHaveBeenLastCalledWith(true));
    fireEvent.click(screen.getByTestId("save-workflow"));
    await waitFor(() => expect(onDirtyChange).toHaveBeenLastCalledWith(false));
  });

  it("勾選 AI 工具即 true，儲存成功後回 false", async () => {
    const onDirtyChange = dirtyHarness("speclink");
    fireEvent.click(await screen.findByLabelText("codex"));
    await waitFor(() => expect(onDirtyChange).toHaveBeenLastCalledWith(true));
    fireEvent.click(screen.getByTestId("save-app"));
    await waitFor(() => expect(onDirtyChange).toHaveBeenLastCalledWith(false));
  });

  it("專案說明只開編輯為 false，改了草稿為 true，取消後回 false", async () => {
    const onDirtyChange = dirtyHarness("context");
    fireEvent.click(await screen.findByTestId("context-edit"));
    expect(onDirtyChange).toHaveBeenLastCalledWith(false);
    typeInEditor(screen.getByTestId("context-input"), "改過");
    await waitFor(() => expect(onDirtyChange).toHaveBeenLastCalledWith(true));
    fireEvent.click(screen.getByTestId("context-cancel"));
    await waitFor(() => expect(onDirtyChange).toHaveBeenLastCalledWith(false));
  });

  it("產出規則改了草稿為 true", async () => {
    const onDirtyChange = dirtyHarness("rules");
    fireEvent.click(await screen.findByTestId("rules-edit"));
    expect(onDirtyChange).toHaveBeenLastCalledWith(false);
    typeInEditor(screen.getByTestId("rules-input-tasks"), "只留一條");
    await waitFor(() => expect(onDirtyChange).toHaveBeenLastCalledWith(true));
  });
});

describe("兩卡唯讀優先（spec Scenario「唯讀優先與各卡就地編輯」）", () => {
  it("開啟專案說明頁：唯讀 markdown 渲染、無文字區、頁標題右側僅編輯", async () => {
    renderView(projectSnap());
    const card = await screen.findByTestId("context-page");
    expect(within(card).getByText("專案說明")).toBeTruthy();
    // markdown 渲染：# 標題成為 heading，而非 raw 文字區。
    expect(within(card).getByRole("heading", { name: "專案簡介" })).toBeTruthy();
    expect(within(card).queryAllByRole("textbox")).toHaveLength(0);
    // 唯讀態按鈕列：僅編輯，無取消/儲存。
    expect(within(card).getByTestId("context-edit")).toBeTruthy();
    expect(within(card).queryByTestId("context-save")).toBeNull();
    expect(within(card).queryByTestId("context-cancel")).toBeNull();
  });

  it("超長專案說明呈現全文、無顯示更多（專案說明自成一頁，不收合）", async () => {
    const longContext = Array.from({ length: 20 }, (_, i) => `第 ${i + 1} 段說明`).join("\n\n");
    renderView(projectSnap({ context: longContext }));
    const card = await screen.findByTestId("context-page");
    expect(within(card).getByText("第 20 段說明")).toBeTruthy();
    expect(within(card).queryByText("顯示更多")).toBeNull();
    expect(card.querySelector(".overflow-hidden")).toBeNull();
  });

  it("未設定專案說明顯示空狀態提示", async () => {
    renderView(projectSnap({ context: null }));
    const card = await screen.findByTestId("context-page");
    expect(within(card).getByText(/尚未設定專案說明/)).toBeTruthy();
  });

  it("產出規則卡唯讀僅列有條目鍵，鍵名為小節標題、條目為清單", async () => {
    renderView(projectSnap(), "rules");
    const card = await screen.findByTestId("rules-page");
    expect(within(card).getByRole("heading", { level: 2, name: "產出規則" })).toBeTruthy();
    expect(within(card).getByTestId("rules-readonly-proposal")).toBeTruthy();
    // 鍵名等寬，旁附條數。
    expect(within(card).getByTestId("rules-readonly-tasks").textContent).toContain("2 條");
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
  it("點編輯就地切換：按鈕列變「編輯／預覽」切換＋取消＋儲存 ⌘S，內容卡換成 markdown 編輯器（spec Scenario「唯讀優先與各頁獨立編輯」）", async () => {
    renderView(projectSnap());
    const card = await screen.findByTestId("context-page");
    fireEvent.click(within(card).getByTestId("context-edit"));
    expect(within(card).queryByTestId("context-edit")).toBeNull();
    expect(within(card).getByTestId("context-cancel")).toBeTruthy();
    expect(within(within(card).getByTestId("context-save")).getByText("Ctrl+S")).toBeTruthy();
    expect(within(card).getByRole("tab", { name: "編輯" }).getAttribute("aria-selected")).toBe("true");
    expect(within(card).getByRole("tab", { name: "預覽" })).toBeTruthy();
    const input = within(card).getByTestId("context-input");
    expect(input.querySelector(".cm-editor")).not.toBeNull();
    expect(input.querySelector(".cm-lineNumbers")).not.toBeNull();
    expect(within(card).getByText("markdown · 自動換行")).toBeTruthy();
    expect(editorText(input)).toBe("# 專案簡介\n\n這是一段說明");
  });

  it("編輯與預覽切換保留草稿與游標，期間不寫入（spec Scenario「編輯與預覽切換保留草稿」）", async () => {
    const ws = renderView(projectSnap());
    const card = await screen.findByTestId("context-page");
    fireEvent.click(within(card).getByTestId("context-edit"));
    const input = within(card).getByTestId("context-input");
    typeInEditor(input, "# 草稿標題\n- 第一項\n- 第二項");
    const view = editorView(input);
    act(() => view.dispatch({ selection: { anchor: 4 } }));
    const user = userEvent.setup();
    await user.click(within(card).getByRole("tab", { name: "預覽" }));
    expect(input.hidden).toBe(true);
    const preview = within(card).getByTestId("context-preview");
    expect(within(preview).getByRole("heading", { name: "草稿標題" })).toBeTruthy();
    expect(within(preview).getByText("第二項")).toBeTruthy();
    await user.click(within(card).getByRole("tab", { name: "編輯" }));
    expect(input.hidden).toBe(false);
    expect(editorText(input)).toBe("# 草稿標題\n- 第一項\n- 第二項");
    expect(view.state.selection.main.head).toBe(4);
    expect(ws.writeWorkflowContext).not.toHaveBeenCalled();
  });

  it("⌘S 儲存寫入原字串：兩層縮排與檔尾換行保留，只寫一次（spec Scenario「編輯專案說明並儲存」）", async () => {
    const ws = renderView(projectSnap());
    const card = await screen.findByTestId("context-page");
    fireEvent.click(within(card).getByTestId("context-edit"));
    const draft = "# 規範\n\n- 第一層\n  - 第二層\n    - 第三層\n";
    typeInEditor(within(card).getByTestId("context-input"), draft);
    pressSave(within(card).getByTestId("context-input"));
    await waitFor(() => expect(ws.writeWorkflowContext).toHaveBeenCalledWith(draft));
    expect(ws.writeWorkflowContext).toHaveBeenCalledTimes(1);
    await waitFor(() => expect(within(card).getByTestId("context-edit")).toBeTruthy());
  });

  it("焦點在編輯器外（預覽中）按 ⌘S 也觸發儲存", async () => {
    const ws = renderView(projectSnap());
    const card = await screen.findByTestId("context-page");
    fireEvent.click(within(card).getByTestId("context-edit"));
    typeInEditor(within(card).getByTestId("context-input"), "預覽中儲存");
    await userEvent.setup().click(within(card).getByRole("tab", { name: "預覽" }));
    fireEvent.keyDown(document.body, { key: "s", ctrlKey: true });
    await waitFor(() => expect(ws.writeWorkflowContext).toHaveBeenCalledWith("預覽中儲存"));
    expect(ws.writeWorkflowContext).toHaveBeenCalledTimes(1);
  });

  it("儲存進行中儲存鈕停用，再按 ⌘S 不重送", async () => {
    const ws = renderView(projectSnap());
    (ws.writeWorkflowContext as ReturnType<typeof vi.fn>).mockReturnValue(new Promise(() => {}));
    const card = await screen.findByTestId("context-page");
    fireEvent.click(within(card).getByTestId("context-edit"));
    fireEvent.click(within(card).getByTestId("context-save"));
    await waitFor(() =>
      expect((within(card).getByTestId("context-save") as HTMLButtonElement).disabled).toBe(true),
    );
    expect(editorView(within(card).getByTestId("context-input")).state.readOnly).toBe(true);
    pressSave(within(card).getByTestId("context-input"));
    expect(ws.writeWorkflowContext).toHaveBeenCalledTimes(1);
  });

  it("儲存中切到別頁再切回，⌘S 與儲存鈕都不重送（寫入中狀態跨頁保留）", async () => {
    const ws = renderView(projectSnap());
    (ws.writeWorkflowContext as ReturnType<typeof vi.fn>).mockReturnValue(new Promise(() => {}));
    const card = await screen.findByTestId("context-page");
    fireEvent.click(within(card).getByTestId("context-edit"));
    fireEvent.click(within(card).getByTestId("context-save"));
    goto("rules");
    goto("context");
    pressSave(screen.getByTestId("context-input"));
    fireEvent.click(screen.getByTestId("context-save"));
    expect(ws.writeWorkflowContext).toHaveBeenCalledTimes(1);
  });

  it("產出規則儲存中編輯器唯讀、儲存鈕停用，再按 ⌘S 不重送", async () => {
    const ws = renderView(projectSnap(), "rules");
    (ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mockReturnValue(new Promise(() => {}));
    const card = await screen.findByTestId("rules-page");
    fireEvent.click(within(card).getByTestId("rules-edit"));
    pressSave(within(card).getByTestId("rules-input-tasks"));
    await waitFor(() =>
      expect((within(card).getByTestId("rules-save") as HTMLButtonElement).disabled).toBe(true),
    );
    expect(editorView(within(card).getByTestId("rules-input-tasks")).state.readOnly).toBe(true);
    pressSave(within(card).getByTestId("rules-input-tasks"));
    expect(ws.writeWorkflowRules).toHaveBeenCalledTimes(1);
  });

  // spec「表單控制項與按鈕以主題化元件呈現」Scenario「設定頁編輯器主題化」
  it("設定頁編輯器外殼取用設計 token（邊框、底色、focus 環）", async () => {
    renderView(projectSnap());
    const ctxCard = await screen.findByTestId("context-page");
    fireEvent.click(within(ctxCard).getByTestId("context-edit"));
    const ctxShell = within(ctxCard).getByTestId("context-input").querySelector('[data-slot="code-editor"]')!;
    expect(ctxShell.className).toContain("border-border");
    expect(ctxShell.className).toContain("bg-card");
    expect(ctxShell.className).toContain("focus-within:ring");
    goto("rules");
    const rulesCard = screen.getByTestId("rules-page");
    fireEvent.click(within(rulesCard).getByTestId("rules-edit"));
    const rulesInput = within(rulesCard).getByTestId("rules-input-tasks");
    const rulesShell = rulesInput.querySelector('[data-slot="code-editor"]')!;
    expect(rulesShell.className).toContain("border-border");
    expect(rulesShell.className).toContain("bg-card");
    expect(rulesShell.className).toContain("focus-within:ring");
    expect(within(rulesInput).getByText("純文字 · 自動換行")).toBeTruthy();
  });

  it("一頁編輯中另一頁唯讀可用；各頁取消僅還原本頁且草稿重設（spec Scenario「唯讀優先與各頁獨立編輯」）", async () => {
    const ws = renderView(projectSnap());
    const ctxPage = () => screen.getByTestId("context-page");
    const rulesPage = () => screen.getByTestId("rules-page");
    await screen.findByTestId("context-page");
    fireEvent.click(within(ctxPage()).getByTestId("context-edit"));
    typeInEditor(within(ctxPage()).getByTestId("context-input"), "編輯中的說明");
    // 切到產出規則頁：維持唯讀且編輯鈕仍可用。
    goto("rules");
    expect(within(rulesPage()).queryAllByRole("textbox")).toHaveLength(0);
    const rulesEdit = within(rulesPage()).getByTestId("rules-edit") as HTMLButtonElement;
    expect(rulesEdit.disabled).toBe(false);
    // 兩頁可同時各持編輯態。
    fireEvent.click(rulesEdit);
    typeInEditor(within(rulesPage()).getByTestId("rules-input-tasks"), "被放棄");
    // 取消產出規則：僅該頁還原，專案說明頁編輯態與草稿不受影響。
    fireEvent.click(within(rulesPage()).getByTestId("rules-cancel"));
    expect(within(rulesPage()).queryAllByRole("textbox")).toHaveLength(0);
    goto("context");
    expect(editorText(within(ctxPage()).getByTestId("context-input"))).toBe("編輯中的說明");
    expect(ws.writeWorkflowContext).not.toHaveBeenCalled();
    expect(ws.writeWorkflowRules).not.toHaveBeenCalled();
    // 重進產出規則編輯：草稿為讀入值，非被放棄的修改。
    goto("rules");
    fireEvent.click(within(rulesPage()).getByTestId("rules-edit"));
    expect(editorText(within(rulesPage()).getByTestId("rules-input-tasks"))).toBe(
      "先寫失敗測試\n更新文件",
    );
  });

  it("編輯態產出規則文字區恰為活躍 schema 固定鍵各一、值為條目換行串接、無自由鍵輸入", async () => {
    // spec Scenario「固定鍵分節不可自由輸入」。
    renderView(projectSnap(), "rules");
    const card = await screen.findByTestId("rules-page");
    fireEvent.click(within(card).getByTestId("rules-edit"));
    for (const id of ["proposal", "design", "specs", "tasks"]) {
      expect(within(card).getByTestId(`rules-input-${id}`)).toBeTruthy();
    }
    expect(within(card).getAllByRole("textbox")).toHaveLength(4);
    expect(editorText(within(card).getByTestId("rules-input-tasks"))).toBe(
      "先寫失敗測試\n更新文件",
    );
    expect(editorText(within(card).getByTestId("rules-input-design"))).toBe("");
  });

  it("編輯態每個固定鍵一張小卡：等寬鍵名＋「N 條」計數＋純文字編輯器，儲存鈕標示 ⌘S", async () => {
    renderView(projectSnap(), "rules");
    const card = await screen.findByTestId("rules-page");
    fireEvent.click(within(card).getByTestId("rules-edit"));
    const expected = { proposal: "1 條", design: "0 條", specs: "0 條", tasks: "2 條" };
    for (const [id, count] of Object.entries(expected)) {
      const section = within(card).getByTestId(`rules-input-${id}`);
      expect(within(section).getByText(id).className).toContain("font-mono");
      expect(within(section).getByTestId(`rules-count-${id}`).textContent).toBe(count);
      expect(section.querySelector(".cm-editor")).not.toBeNull();
    }
    expect(within(within(card).getByTestId("rules-save")).getByText("Ctrl+S")).toBeTruthy();
  });

  it("產出規則條數即時計數，空行不計；⌘S 寫回恰為三條（spec Scenario「產出規則條數即時計數」）", async () => {
    const ws = renderView(projectSnap(), "rules");
    const card = await screen.findByTestId("rules-page");
    fireEvent.click(within(card).getByTestId("rules-edit"));
    const tasks = within(card).getByTestId("rules-input-tasks");
    typeInEditor(tasks, "先寫失敗測試\n\n更新文件");
    expect(within(card).getByTestId("rules-count-tasks").textContent).toBe("2 條");
    typeInEditor(tasks, "先寫失敗測試\n\n更新文件\n補上文件連結");
    expect(within(card).getByTestId("rules-count-tasks").textContent).toBe("3 條");
    pressSave(tasks);
    await waitFor(() => expect(ws.writeWorkflowRules).toHaveBeenCalled());
    expect(ws.writeWorkflowRules).toHaveBeenCalledTimes(1);
    expect((ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mock.calls[0][0]).toContainEqual([
      "tasks",
      ["先寫失敗測試", "更新文件", "補上文件連結"],
    ]);
  });

  it("產出規則卡儲存：逐行 trim、空行滌除、行序即順序（spec Example 行對調）；僅寫 rules、context 逐字元不變", async () => {
    // spec Scenario「各卡儲存僅寫對應鍵」——僅存產出規則時不觸碰 context。
    const ws = renderView(projectSnap(), "rules");
    const card = await screen.findByTestId("rules-page");
    fireEvent.click(within(card).getByTestId("rules-edit"));
    typeInEditor(within(card).getByTestId("rules-input-tasks"), "  更新文件  \n\n先寫失敗測試\n");
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
    const ws = renderView(projectSnap(), "rules");
    const card = await screen.findByTestId("rules-page");
    fireEvent.click(within(card).getByTestId("rules-edit"));
    typeInEditor(within(card).getByTestId("rules-input-tasks"), "先寫失敗測試\n更新文件\n@完成後執行全部測試");
    fireEvent.click(within(card).getByTestId("rules-save"));
    await waitFor(() => expect(ws.writeWorkflowRules).toHaveBeenCalled());
    const payload = (ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mock.calls[0][0];
    expect(payload).toContainEqual(["tasks", ["先寫失敗測試", "更新文件", "@完成後執行全部測試"]]);
    expect(payload).toContainEqual(["proposal", ["提案必須列出影響的 crates"]]);
  });

  it("清空某鍵文字區＝該鍵送空清單、其餘鍵保留（spec Example 鍵移除第二列）", async () => {
    const ws = renderView(projectSnap(), "rules");
    const card = await screen.findByTestId("rules-page");
    fireEvent.click(within(card).getByTestId("rules-edit"));
    typeInEditor(within(card).getByTestId("rules-input-tasks"), "");
    fireEvent.click(within(card).getByTestId("rules-save"));
    await waitFor(() => expect(ws.writeWorkflowRules).toHaveBeenCalled());
    const payload = (ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mock.calls[0][0];
    expect(payload).toContainEqual(["tasks", []]);
    expect(payload).toContainEqual(["proposal", ["提案必須列出影響的 crates"]]);
  });

  it("全部鍵清空＝全空 payload（觸發後端移除 rules 鍵；spec Example 鍵移除第三列）", async () => {
    const ws = renderView(projectSnap(), "rules");
    const card = await screen.findByTestId("rules-page");
    fireEvent.click(within(card).getByTestId("rules-edit"));
    typeInEditor(within(card).getByTestId("rules-input-tasks"), "");
    typeInEditor(within(card).getByTestId("rules-input-proposal"), "  \n ");
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
    const card = await screen.findByTestId("context-page");
    fireEvent.click(within(card).getByTestId("context-edit"));
    typeInEditor(within(card).getByTestId("context-input"), "");
    fireEvent.click(within(card).getByTestId("context-save"));
    await waitFor(() => expect(ws.writeWorkflowContext).toHaveBeenCalledWith(""));
    expect(ws.writeWorkflowRules).not.toHaveBeenCalled();
  });

  it("編輯專案說明並儲存：回唯讀渲染新內容；僅寫 context（spec Scenario 編輯專案說明並儲存）", async () => {
    const ws = renderView(projectSnap());
    const card = await screen.findByTestId("context-page");
    fireEvent.click(within(card).getByTestId("context-edit"));
    typeInEditor(within(card).getByTestId("context-input"), "# 新標題\n\n新的說明");
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
    const card = await screen.findByTestId("context-page");
    fireEvent.click(within(card).getByTestId("context-edit"));
    typeInEditor(within(card).getByTestId("context-input"), "被放棄的修改");
    fireEvent.click(within(card).getByTestId("context-cancel"));
    // 還原唯讀呈現、零寫入。
    expect(within(card).getByTestId("context-edit")).toBeTruthy();
    expect(ws.writeWorkflowContext).not.toHaveBeenCalled();
    expect(ws.writeWorkflowRules).not.toHaveBeenCalled();
    // 產出規則頁全程唯讀不受影響。
    goto("rules");
    expect(within(screen.getByTestId("rules-page")).queryAllByRole("textbox")).toHaveLength(0);
    // 重進編輯：草稿為讀入值，非被放棄的修改。
    goto("context");
    const page = screen.getByTestId("context-page");
    fireEvent.click(within(page).getByTestId("context-edit"));
    expect(editorText(within(page).getByTestId("context-input"))).toBe(
      "# 專案簡介\n\n這是一段說明",
    );
  });

  it("寫入失敗：顯示單行錯誤並維持編輯態不遺失輸入（design 契約失敗模式）", async () => {
    const ws = renderView(projectSnap());
    (ws.writeWorkflowContext as ReturnType<typeof vi.fn>).mockRejectedValue(
      "openspec/config.yaml: write failed: denied",
    );
    const card = await screen.findByTestId("context-page");
    fireEvent.click(within(card).getByTestId("context-edit"));
    typeInEditor(within(card).getByTestId("context-input"), "改到一半的內容");
    fireEvent.click(within(card).getByTestId("context-save"));
    expect(await within(card).findByText(/write failed/)).toBeTruthy();
    // 仍在編輯態、輸入未遺失。
    expect(within(card).getByTestId("context-save")).toBeTruthy();
    expect(editorText(within(card).getByTestId("context-input"))).toBe(
      "改到一半的內容",
    );
  });

  it("頁名採正典詞；編輯態說明載明一行一條規則、頭尾空白不保留，以及產出規則區塊內的註解不保留", async () => {
    renderView(projectSnap());
    const ctxCard = await screen.findByTestId("context-page");
    expect(within(ctxCard).getByRole("heading", { level: 2, name: "專案說明" })).toBeTruthy();
    fireEvent.click(within(ctxCard).getByTestId("context-edit"));
    expect(within(ctxCard).getByText(/清空儲存即移除該鍵/)).toBeTruthy();
    goto("rules");
    const rulesCard = screen.getByTestId("rules-page");
    expect(within(rulesCard).getByRole("heading", { level: 2, name: "產出規則" })).toBeTruthy();
    fireEvent.click(within(rulesCard).getByTestId("rules-edit"));
    expect(within(rulesCard).getByText(/一行一條規則（頭尾空白不保留）/)).toBeTruthy();
    expect(within(rulesCard).getByText(/儲存只改動產出規則：產出規則區塊內的註解不會保留，config.yaml 的其他內容與註解會保留/)).toBeTruthy();
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

  it("editor 於產出流程頁見 mono revision，政策可存且成功後 revision 前進（spec Scenario「遠端專案的專案設定分類」）", async () => {
    const snap = projectSnap({ revision: 41 } as Partial<SettingsSnapshot["workflow"]>);
    const ws = remoteSettings(snap, true);
    render(<Harness settings={ws} initial="workflow" />);

    const revision = await screen.findByTestId("policy-revision");
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
    render(<Harness settings={ws} initial="workflow" />);

    await screen.findByTestId("policy-card");
    expect(screen.queryByLabelText("worktree")).toBeNull();
    expect(screen.getByLabelText("tdd")).toBeTruthy();
    expect(screen.getByLabelText("audit")).toBeTruthy();
  });

  it("reader 看得到四頁現值但全唯讀，存檔停用並附繁中角色說明", async () => {
    const ws = remoteSettings(
      projectSnap({ revision: 9 } as Partial<SettingsSnapshot["workflow"]>),
      false,
    );
    render(<Harness settings={ws} />);

    await screen.findByTestId("context-page");
    expect(screen.getByTestId("policy-reader-note").textContent).toContain("你的角色為檢視者");
    expect((screen.getByTestId("context-edit") as HTMLButtonElement).disabled).toBe(true);
    goto("rules");
    expect(screen.getByTestId("policy-reader-note")).toBeTruthy();
    expect((screen.getByTestId("rules-edit") as HTMLButtonElement).disabled).toBe(true);
    goto("review");
    expect(screen.getByTestId("policy-reader-note")).toBeTruthy();
    expect((screen.getByTestId("review-edit") as HTMLButtonElement).disabled).toBe(true);
    goto("workflow");
    expect(screen.getByTestId("policy-reader-note")).toBeTruthy();
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
    render(<Harness settings={ws} />);

    const contextCard = await screen.findByTestId("context-page");
    fireEvent.click(within(contextCard).getByTestId("context-edit"));
    typeInEditor(within(contextCard).getByTestId("context-input"), "我的原始輸入\n保留換行");
    fireEvent.click(within(contextCard).getByTestId("context-save"));

    const panel = await screen.findByTestId("policy-conflict-panel");
    // 政策衝突是警示（要人決定，不是壞掉）：琥珀邊框，不用主色（主色＝可點的東西）。
    expect(panel.className).toContain("status-warning");
    expect(panel.className).not.toContain("border-primary");
    expect(editorText(within(contextCard).getByTestId("context-input"))).toBe(
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

  it("衝突對照卡開著時 ⌘S 不寫入、編輯器轉唯讀（不繞過對照卡蓋掉 server 版）", async () => {
    const initial = projectSnap({ revision: 41 } as Partial<SettingsSnapshot["workflow"]>);
    const latest = projectSnap({ revision: 42, context: "server 較新的說明" } as Partial<SettingsSnapshot["workflow"]>);
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
    render(<Harness settings={ws} />);
    const page = await screen.findByTestId("context-page");
    fireEvent.click(within(page).getByTestId("context-edit"));
    const input = within(page).getByTestId("context-input");
    typeInEditor(input, "我的輸入");
    pressSave(input);
    await screen.findByTestId("policy-conflict-panel");
    expect(editorView(input).state.readOnly).toBe(true);
    pressSave(input);
    fireEvent.keyDown(document.body, { key: "s", ctrlKey: true });
    await act(async () => {});
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
    render(<Harness settings={ws} initial="workflow" />);

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

// ---- 產出流程頁（desktop-schema-panel design D4／D5；desktop-settings-mode design D8）----

describe("產出流程頁（spec「產出流程頁」）", () => {
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

  /** 載入後切到產出流程頁。 */
  async function openSchemasTab() {
    await screen.findByTestId("context-page");
    goto("workflow");
  }

  it("產出流程自成一頁：專案說明頁與產出規則頁無此內容（spec Scenario「產出流程自成一頁」）", async () => {
    const ws = schemaSettings(projectSnap());
    render(<Harness settings={ws} />);
    await screen.findByTestId("context-page");
    expect(screen.queryByTestId("schema-card")).toBeNull();
    goto("rules");
    expect(screen.queryByTestId("schema-card")).toBeNull();
    goto("workflow");
    expect(await screen.findByTestId("schema-card")).toBeTruthy();
    expect(screen.getByTestId("policy-card")).toBeTruthy();
  });

  it("清單列出可解析項（名稱、來源層級、artifact 圖），點入唯讀詳情含全文且無編輯入口", async () => {
    // spec Scenario「清單列出可解析的 schema」＋「詳情唯讀呈現內容」＋
    // Example「清單一列的形狀」。
    const ws = schemaSettings(projectSnap());
    render(<Harness settings={ws} />);
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
    render(<Harness settings={ws} />);
    await openSchemasTab();
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    const select = await screen.findByLabelText("使用中的產出流程");
    expect(select.textContent).toContain("spec-driven");
    await user.click(select);
    await user.click(await screen.findByRole("option", { name: /my-flow/ }));
    await waitFor(() => expect(ws.writeWorkflowSchema).toHaveBeenCalledWith("my-flow"));
    // 固定鍵隨新 schema 更新：切到產出規則頁，編輯分節只剩 plan。
    await waitFor(() => expect(screen.getByLabelText("使用中的產出流程").textContent).toContain("my-flow"));
    goto("rules");
    fireEvent.click(await screen.findByTestId("rules-edit"));
    await waitFor(() => expect(screen.getByTestId("rules-input-plan")).toBeTruthy());
    expect(screen.queryByTestId("rules-input-proposal")).toBeNull();
  });

  it("切換寫入失敗：錯誤浮出於表單，不靜默", async () => {
    const ws = schemaSettings(projectSnap());
    (ws.writeWorkflowSchema as ReturnType<typeof vi.fn>).mockRejectedValue(
      "openspec/config.yaml: write failed: denied",
    );
    render(<Harness settings={ws} />);
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
    render(<Harness settings={ws} />);
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
    render(<Harness settings={ws} initial="rules" />);
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
    render(<Harness settings={ws} />);
    // 專案說明頁：開編輯並輸入未存草稿。
    fireEvent.click(await screen.findByTestId("context-edit"));
    typeInEditor(screen.getByTestId("context-input"), "還沒存的草稿");
    // 切到產出流程頁做切換。
    goto("workflow");
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(await screen.findByLabelText("使用中的產出流程"));
    await user.click(await screen.findByRole("option", { name: /my-flow/ }));
    await waitFor(() => expect(ws.writeWorkflowSchema).toHaveBeenCalledWith("my-flow"));
    // 切回專案說明頁：編輯態仍開、草稿仍在。
    goto("context");
    expect(editorText(await screen.findByTestId("context-input"))).toBe("還沒存的草稿");
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
    render(<Harness settings={ws} />);
    await openSchemasTab();
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(await screen.findByLabelText("使用中的產出流程"));
    await user.click(await screen.findByRole("option", { name: /my-flow/ }));
    expect(await screen.findByTestId("policy-conflict-panel")).toBeTruthy();
    expect(screen.getByTestId("conflict-revision").textContent).toContain("42");
  });

  it("切換進行中停用政策表單與各頁編輯，完成後恢復（等待期間的改動不會被重讀蓋回；review-rules-config 品質關卡第 2 輪）", async () => {
    const snap = projectSnap({ revision: 41 } as Partial<SettingsSnapshot["workflow"]>);
    let finish: (revision: number) => void = () => {};
    const ws = schemaSettings(snap, [BUILTIN_ENTRY, CUSTOM_ENTRY], {
      kind: "remote",
      writeWorkflowSchema: vi.fn().mockReturnValue(new Promise<number>((resolve) => (finish = resolve))),
    });
    render(<Harness settings={ws} />);
    await openSchemasTab();
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(await screen.findByLabelText("使用中的產出流程"));
    await user.click(await screen.findByRole("option", { name: /my-flow/ }));
    await waitFor(() => expect((screen.getByLabelText("tdd") as HTMLButtonElement).disabled).toBe(true));
    goto("review");
    expect((screen.getByTestId("review-edit") as HTMLButtonElement).disabled).toBe(true);
    await act(async () => finish(42));
    await waitFor(() => expect((screen.getByTestId("review-edit") as HTMLButtonElement).disabled).toBe(false));
    goto("workflow");
    expect((screen.getByLabelText("tdd") as HTMLButtonElement).disabled).toBe(false);
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
    render(<Harness settings={ws} />);
    await openSchemasTab();
    const trigger = await screen.findByLabelText("使用中的產出流程");
    expect(trigger.textContent).toContain("their-flow");
  });

  it("readSchemas 失敗：錯誤浮出而非靜默空白（review R7）", async () => {
    const ws = schemaSettings(projectSnap());
    (ws.readSchemas as ReturnType<typeof vi.fn>).mockRejectedValue(
      "not a speclink project: /gone",
    );
    render(<Harness settings={ws} />);
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
    render(<Harness settings={ws} />);
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
    render(<Harness settings={ws} />);
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
    render(<Harness settings={ws} />);
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
    render(<Harness settings={ws} />);
    await openSchemasTab();
    await screen.findByTestId("schema-item-my-flow");
    expect(screen.queryByTestId("schema-delete-spec-driven")).toBeNull();
    fireEvent.click(screen.getByTestId("schema-delete-my-flow"));
    fireEvent.click(within(await screen.findByRole("alertdialog")).getByRole("button", { name: "刪除" }));
    await waitFor(() => expect(ws.deleteSchema).toHaveBeenCalledWith("my-flow"));
    await waitFor(() => expect(screen.queryByTestId("schema-item-my-flow")).toBeNull());
  });

  it("刪除確認對話框取消：零呼叫零變動", async () => {
    // spec Scenario「取消確認零變動」。
    const ws = schemaSettings(projectSnap());
    render(<Harness settings={ws} />);
    await openSchemasTab();
    await screen.findByTestId("schema-item-my-flow");
    fireEvent.click(screen.getByTestId("schema-delete-my-flow"));
    fireEvent.click(within(await screen.findByRole("alertdialog")).getByRole("button", { name: "取消" }));
    expect(ws.deleteSchema).not.toHaveBeenCalled();
    expect(screen.getByTestId("schema-item-my-flow")).toBeTruthy();
  });

  it("刪除失敗（使用中的 schema 拒刪）：錯誤浮出於表單", async () => {
    const ws = schemaSettings(projectSnap());
    (ws.deleteSchema as ReturnType<typeof vi.fn>).mockRejectedValue(
      "'my-flow' 是使用中的產出流程（config.yaml 的 schema 鍵正指著它）——請先切換到其他項目再刪除",
    );
    render(<Harness settings={ws} />);
    await openSchemasTab();
    await screen.findByTestId("schema-item-my-flow");
    fireEvent.click(screen.getByTestId("schema-delete-my-flow"));
    fireEvent.click(within(await screen.findByRole("alertdialog")).getByRole("button", { name: "刪除" }));
    expect(await screen.findByText(/使用中的產出流程/)).toBeTruthy();
    expect(screen.getByTestId("schema-item-my-flow")).toBeTruthy();
  });

  it("fork／建立／刪除不重設產出規則的編輯態與草稿（review N1）", async () => {
    const ws = schemaSettings(projectSnap());
    render(<Harness settings={ws} initial="rules" />);
    // 產出規則頁：開編輯並輸入未存草稿。
    fireEvent.click(await screen.findByTestId("rules-edit"));
    typeInEditor(screen.getByTestId("rules-input-proposal"), "還沒存的規則草稿");
    // 產出流程頁做 fork。
    goto("workflow");
    await screen.findByTestId("schema-item-spec-driven");
    fireEvent.click(screen.getByTestId("schema-fork-spec-driven"));
    await waitFor(() => expect(ws.forkSchema).toHaveBeenCalled());
    // 切回：編輯態仍開、草稿仍在。
    goto("rules");
    expect(editorText(await screen.findByTestId("rules-input-proposal"))).toBe("還沒存的規則草稿");
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
    render(<Harness settings={ws} initial="rules" />);
    // 開產出規則編輯並改草稿。
    fireEvent.click(await screen.findByTestId("rules-edit"));
    typeInEditor(screen.getByTestId("rules-input-proposal"), "編輯中的提案規則");
    // 產出流程頁 fork → refreshSchemaFacts 換集（schemaArtifacts 變 ["plan"]）。
    goto("workflow");
    await screen.findByTestId("schema-item-spec-driven");
    fireEvent.click(screen.getByTestId("schema-fork-spec-driven"));
    await waitFor(() => expect(ws.forkSchema).toHaveBeenCalled());
    // 切回：編輯面仍是開編輯當下的分節與草稿（不是新鍵的空白 textarea）。
    goto("rules");
    expect(editorText(await screen.findByTestId("rules-input-proposal"))).toBe("編輯中的提案規則");
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
    render(<Harness settings={ws} />);
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
    render(<Harness settings={ws} />);
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
    render(<Harness settings={ws} />);
    await openSchemasTab();
    fireEvent.change(await screen.findByTestId("schema-create-name"), {
      target: { value: "My Flow" },
    });
    fireEvent.click(screen.getByTestId("schema-create"));
    expect(await screen.findByText(/kebab-case/)).toBeTruthy();
  });

  it("remote：無 .speclink.yaml 頁、無 fork 與建立入口、下拉僅內建、非內建顯示遠端自訂尚不支援且不猜固定鍵", async () => {
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
    render(<Harness settings={ws} />);
    await screen.findByTestId("context-page");
    // 不猜固定鍵：產出規則頁無任何分節內容。
    goto("rules");
    expect(screen.queryByTestId("rules-readonly-proposal")).toBeNull();

    goto("workflow");
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

// ---- 審查規則頁（spec 需求「設定頁編輯審查規則」；review-rules-config）----

describe("審查規則頁（spec 需求「設定頁編輯審查規則」）", () => {
  const RULES: ReviewRules = {
    baseline: "extend",
    critical: ["所有 SQL 一律用參數化查詢，不得用字串拼接", "不得把金鑰或密碼寫死在程式碼裡"],
    warning: ["新增的公開 API 要有對應的測試"],
    suggestion: ["函式超過 50 行時宜拆分"],
  };
  const reviewSnap = (review = RULES) => projectSnap({ review } as Partial<SettingsSnapshot["workflow"]>);
  /** 從預設清單下拉選一個模式（Radix Select，與產出流程頁 locale 同一元件）。 */
  async function chooseBaseline(page: HTMLElement, mode: string) {
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(within(page).getByLabelText("12 條 code smell"));
    await user.click(await screen.findByRole("option", { name: mode }));
  }

  it("唯讀呈現：預設清單卡顯示模式，專案規則卡依三節列出規則、條數與效果說明（spec Scenario「唯讀呈現」）", async () => {
    renderView(reviewSnap(), "review");
    const page = await screen.findByTestId("review-page");
    expect(within(page).getByRole("heading", { level: 2, name: "審查規則" })).toBeTruthy();
    expect(screen.getByTestId("file-note-config").textContent).toBe("openspec/config.yaml · review");
    expect(within(page).getByTestId("review-baseline-mode").textContent).toBe("沿用並追加");
    const expected = {
      critical: ["CRITICAL", "2 條", "擋蓋章；要接受放行須逐條再確認"],
      warning: ["WARNING", "1 條", "擋蓋章；偶有正當例外可接受放行"],
      suggestion: ["SUGGESTION", "1 條", "只記在工單，不擋蓋章"],
    };
    for (const [list, texts] of Object.entries(expected)) {
      const section = within(page).getByTestId(`review-section-${list}`);
      for (const text of texts) expect(within(section).getByText(text)).toBeTruthy();
    }
    expect(within(page).getByTestId("review-section-critical").textContent).toContain(
      "不得把金鑰或密碼寫死在程式碼裡",
    );
    // 唯讀：沒有編輯器、頁標題右側只有編輯鈕。
    expect(page.querySelector(".cm-editor")).toBeNull();
    expect(within(page).getByTestId("review-edit")).toBeTruthy();
  });

  it("三節皆空時專案規則卡顯示空狀態；整份換掉的模式名稱照實呈現", async () => {
    renderView(reviewSnap({ baseline: "replace", critical: [], warning: [], suggestion: [] }), "review");
    const page = await screen.findByTestId("review-page");
    expect(within(page).getByTestId("review-baseline-mode").textContent).toBe("整份換掉");
    expect(within(page).getByTestId("review-empty")).toBeTruthy();
    expect(within(page).queryByTestId("review-section-critical")).toBeNull();
  });

  it("查看 12 條展開引擎提供的名稱（spec Scenario「展開 12 條名稱」）；遠端專案同樣可展開", async () => {
    renderView(reviewSnap(), "review");
    const page = await screen.findByTestId("review-page");
    expect(within(page).queryByTestId("review-smells")).toBeNull();
    const toggle = within(page).getByRole("button", { name: "查看 12 條" });
    expect(toggle.getAttribute("aria-expanded")).toBe("false");
    fireEvent.click(toggle);
    expect(toggle.getAttribute("aria-expanded")).toBe("true");
    // 與產出流程頁的「檢視內容／收合」同一種展開鈕：展開後改為收合。
    expect(toggle.textContent).toBe("收合");
    const names = within(within(page).getByTestId("review-smells")).getAllByRole("listitem");
    expect(names.map((n) => n.textContent)).toEqual(SMELLS);
  });

  it("遠端專案的審查規則頁同樣可展開 12 條，名稱與本機一致", async () => {
    const ws = { ...fakeSettings(reviewSnap()), kind: "remote" } as WorkspaceSettingsProvider;
    render(<Harness settings={ws} initial="review" />);
    const page = await screen.findByTestId("review-page");
    fireEvent.click(within(page).getByRole("button", { name: "查看 12 條" }));
    const names = within(within(page).getByTestId("review-smells")).getAllByRole("listitem");
    expect(names.map((n) => n.textContent)).toEqual(SMELLS);
  });

  it("編輯態：預設清單為列右側的兩選項下拉、專案規則恰為三個固定節（嚴重度籤＋即時條數），無新增節的輸入（spec Scenario「固定三節不可自由新增」）", async () => {
    renderView(reviewSnap(), "review");
    const page = await screen.findByTestId("review-page");
    fireEvent.click(within(page).getByTestId("review-edit"));
    // 與產出流程頁 locale 同一個共用下拉；列說明寫所選模式的效果。
    expect(within(page).queryAllByRole("radio")).toHaveLength(0);
    const baseline = within(page).getByLabelText("12 條 code smell");
    expect(baseline.textContent).toContain("沿用並追加");
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(baseline);
    expect((await screen.findAllByRole("option")).map((o) => o.textContent)).toEqual(["沿用並追加", "整份換掉"]);
    await user.keyboard("{Escape}");
    expect(within(page).getAllByRole("textbox")).toHaveLength(3);
    for (const [list, count] of [["critical", "2 條"], ["warning", "1 條"], ["suggestion", "1 條"]]) {
      const section = within(page).getByTestId(`review-input-${list}`);
      expect(within(section).getByText(list.toUpperCase())).toBeTruthy();
      expect(within(section).getByTestId(`review-count-${list}`).textContent).toBe(count);
      expect(section.querySelector(".cm-editor")).not.toBeNull();
    }
    expect(editorText(within(page).getByTestId("review-input-critical"))).toBe(RULES.critical.join("\n"));
    expect(within(within(page).getByTestId("review-save")).getByText("Ctrl+S")).toBeTruthy();
  });

  it("編輯並以 ⌘S 儲存只寫 review：逐行修剪、空行不寫；回唯讀並顯示新內容（spec Scenario「編輯並儲存僅寫 review」）", async () => {
    const ws = renderView(reviewSnap(), "review");
    const page = await screen.findByTestId("review-page");
    fireEvent.click(within(page).getByTestId("review-edit"));
    const warning = within(page).getByTestId("review-input-warning");
    typeInEditor(warning, "新增的公開 API 要有對應的測試\n\n  錯誤訊息要指出出錯的檔案  ");
    expect(within(page).getByTestId("review-count-warning").textContent).toBe("2 條");
    pressSave(warning);
    await waitFor(() => expect(ws.writeWorkflowReview).toHaveBeenCalledTimes(1));
    expect(ws.writeWorkflowReview).toHaveBeenCalledWith({
      ...RULES,
      warning: ["新增的公開 API 要有對應的測試", "錯誤訊息要指出出錯的檔案"],
    });
    expect(ws.writeWorkflowContext).not.toHaveBeenCalled();
    expect(ws.writeWorkflowRules).not.toHaveBeenCalled();
    expect(ws.writeWorkflowConfig).not.toHaveBeenCalled();
    await waitFor(() => expect(page.querySelector(".cm-editor")).toBeNull());
    expect(within(page).getByTestId("review-section-warning").textContent).toContain("錯誤訊息要指出出錯的檔案");
  });

  it("整份換掉且三節皆空：顯示提示但可儲存（spec Scenario「整份換掉且無規則時提示但可儲存」）", async () => {
    const ws = renderView(reviewSnap(), "review");
    const page = await screen.findByTestId("review-page");
    fireEvent.click(within(page).getByTestId("review-edit"));
    await chooseBaseline(page, "整份換掉");
    expect(within(page).queryByTestId("review-replace-empty")).toBeNull();
    for (const list of ["critical", "warning", "suggestion"]) {
      typeInEditor(within(page).getByTestId(`review-input-${list}`), "");
    }
    expect(within(page).getByTestId("review-replace-empty").textContent).toContain(
      "Standards 軸只依 repo 文件審查",
    );
    const save = within(page).getByTestId("review-save") as HTMLButtonElement;
    expect(save.disabled).toBe(false);
    fireEvent.click(save);
    await waitFor(() =>
      expect(ws.writeWorkflowReview).toHaveBeenCalledWith({
        baseline: "replace",
        critical: [],
        warning: [],
        suggestion: [],
      }),
    );
  });

  it("切回沿用並追加、清空某節：payload 帶 extend 與空清單（移除語意由引擎落實；spec Scenario「清空即移除鍵」）", async () => {
    const ws = renderView(reviewSnap({ ...RULES, baseline: "replace" }), "review");
    const page = await screen.findByTestId("review-page");
    fireEvent.click(within(page).getByTestId("review-edit"));
    await chooseBaseline(page, "沿用並追加");
    typeInEditor(within(page).getByTestId("review-input-suggestion"), "");
    fireEvent.click(within(page).getByTestId("review-save"));
    await waitFor(() =>
      expect(ws.writeWorkflowReview).toHaveBeenCalledWith({ ...RULES, baseline: "extend", suggestion: [] }),
    );
  });

  it("取消：還原唯讀且不寫入；重進編輯草稿已重設；未存修改回報隨草稿起落", async () => {
    const onDirtyChange = vi.fn();
    const ws = fakeSettings(reviewSnap());
    render(<Harness settings={ws} initial="review" onDirtyChange={onDirtyChange} />);
    const page = await screen.findByTestId("review-page");
    fireEvent.click(within(page).getByTestId("review-edit"));
    expect(onDirtyChange).toHaveBeenLastCalledWith(false);
    typeInEditor(within(page).getByTestId("review-input-critical"), "改掉");
    await waitFor(() => expect(onDirtyChange).toHaveBeenLastCalledWith(true));
    fireEvent.click(within(page).getByTestId("review-cancel"));
    await waitFor(() => expect(onDirtyChange).toHaveBeenLastCalledWith(false));
    expect(ws.writeWorkflowReview).not.toHaveBeenCalled();
    fireEvent.click(within(page).getByTestId("review-edit"));
    expect(editorText(within(page).getByTestId("review-input-critical"))).toBe(RULES.critical.join("\n"));
  });

  it("審查規則頁編輯中，產出規則頁照常唯讀可用（一頁進入編輯不影響其他頁）", async () => {
    renderView(reviewSnap(), "review");
    const page = await screen.findByTestId("review-page");
    fireEvent.click(within(page).getByTestId("review-edit"));
    goto("rules");
    const rulesPage = screen.getByTestId("rules-page");
    expect(rulesPage.querySelector(".cm-editor")).toBeNull();
    expect((within(rulesPage).getByTestId("rules-edit") as HTMLButtonElement).disabled).toBe(false);
  });

  it("config.yaml 解析失敗：頁內浮出解析原因、編輯鈕停用（spec Scenario「解析失敗時停用編輯」）", async () => {
    const ws = fakeSettings(
      projectSnap({ context: null, rules: {}, schemaArtifacts: [], parseError: "review: unknown field `critcal`" }),
    );
    render(<Harness settings={ws} initial="review" />);
    const page = await screen.findByTestId("review-page");
    expect(within(page).getByText(/critcal/)).toBeTruthy();
    expect((within(page).getByTestId("review-edit") as HTMLButtonElement).disabled).toBe(true);
  });

  it("遠端專案：儲存帶 expected revision 的寫入、成功後 revision 前進；409 時輸入保留並浮出 review 對照（spec Scenario「遠端專案儲存帶 revision」）", async () => {
    const initial = projectSnap({ revision: 41, review: RULES } as Partial<SettingsSnapshot["workflow"]>);
    const latest = projectSnap({
      revision: 42,
      review: { ...RULES, critical: ["server 端的規則"] },
    } as Partial<SettingsSnapshot["workflow"]>);
    const ws = {
      ...fakeSettings(initial),
      kind: "remote",
      writeWorkflowReview: vi
        .fn()
        .mockRejectedValueOnce({ message: "conflict", reason: "revision_conflict", status: 409 })
        .mockResolvedValueOnce(43),
    } as unknown as WorkspaceSettingsProvider;
    (ws.readSettings as ReturnType<typeof vi.fn>)
      .mockReset()
      .mockResolvedValueOnce(initial)
      .mockResolvedValueOnce(latest);
    render(<Harness settings={ws} initial="review" />);
    const page = await screen.findByTestId("review-page");
    expect(screen.getByTestId("policy-revision").textContent).toContain("41");
    fireEvent.click(within(page).getByTestId("review-edit"));
    typeInEditor(within(page).getByTestId("review-input-critical"), "我新增的 CRITICAL 規則");
    fireEvent.click(within(page).getByTestId("review-save"));

    const panel = await screen.findByTestId("policy-conflict-panel");
    const row = within(panel).getByTestId("conflict-row-review-critical");
    expect(within(row).getByTestId("conflict-server").textContent).toContain("server 端的規則");
    expect(within(row).getByTestId("conflict-mine").textContent).toContain("我新增的 CRITICAL 規則");
    expect(editorText(within(page).getByTestId("review-input-critical"))).toBe("我新增的 CRITICAL 規則");

    fireEvent.click(within(panel).getByRole("button", { name: "檢視後以最新 revision 重新提交" }));
    await waitFor(() => expect(screen.queryByTestId("policy-conflict-panel")).toBeNull());
    expect(ws.writeWorkflowReview).toHaveBeenCalledTimes(2);
    expect(screen.getByTestId("policy-revision").textContent).toContain("43");
  });

  it("只改模式時，沒編輯的多行規則原樣寫回不拆開；進入編輯未改動不算未存修改（review R3／R7）", async () => {
    const multi: ReviewRules = { ...RULES, critical: ["第一行\n第二行", ...RULES.critical] };
    const onDirtyChange = vi.fn();
    const ws = fakeSettings(reviewSnap(multi));
    render(<Harness settings={ws} initial="review" onDirtyChange={onDirtyChange} />);
    const page = await screen.findByTestId("review-page");
    fireEvent.click(within(page).getByTestId("review-edit"));
    expect(onDirtyChange).toHaveBeenLastCalledWith(false);
    await chooseBaseline(page, "整份換掉");
    await waitFor(() => expect(onDirtyChange).toHaveBeenLastCalledWith(true));
    fireEvent.click(within(page).getByTestId("review-save"));
    await waitFor(() => expect(ws.writeWorkflowReview).toHaveBeenCalledWith({ ...multi, baseline: "replace" }));
  });

  it("產出規則只改一節時，其他節的多行規則原樣寫回不拆開（review R3 同類位置）", async () => {
    const ws = renderView(
      projectSnap({ rules: { proposal: ["提案必須列出影響的 crates"], tasks: ["第一行\n第二行"] } }),
      "rules",
    );
    const page = await screen.findByTestId("rules-page");
    fireEvent.click(within(page).getByTestId("rules-edit"));
    typeInEditor(within(page).getByTestId("rules-input-proposal"), "新的提案規則");
    fireEvent.click(within(page).getByTestId("rules-save"));
    await waitFor(() => expect(ws.writeWorkflowRules).toHaveBeenCalledTimes(1));
    const payload = Object.fromEntries((ws.writeWorkflowRules as ReturnType<typeof vi.fn>).mock.calls[0][0]);
    expect(payload.proposal).toEqual(["新的提案規則"]);
    expect(payload.tasks).toEqual(["第一行\n第二行"]);
  });

  it("別區的衝突重送後，審查規則採納 server 最新值，之後儲存不蓋掉別人的修改（review R8）", async () => {
    const initial = projectSnap({ revision: 41, review: RULES } as Partial<SettingsSnapshot["workflow"]>);
    const serverReview: ReviewRules = { ...RULES, warning: ["別人剛加的 WARNING"] };
    const latest = projectSnap({
      revision: 42,
      context: "server 較新的說明",
      review: serverReview,
    } as Partial<SettingsSnapshot["workflow"]>);
    const ws = {
      ...fakeSettings(initial),
      kind: "remote",
      writeWorkflowContext: vi
        .fn()
        .mockRejectedValueOnce({ message: "conflict", reason: "revision_conflict", status: 409 })
        .mockResolvedValueOnce(43),
      writeWorkflowReview: vi.fn().mockResolvedValue(44),
    } as unknown as WorkspaceSettingsProvider;
    (ws.readSettings as ReturnType<typeof vi.fn>)
      .mockReset()
      .mockResolvedValueOnce(initial)
      .mockResolvedValueOnce(latest);
    render(<Harness settings={ws} initial="context" />);
    const contextPage = await screen.findByTestId("context-page");
    fireEvent.click(within(contextPage).getByTestId("context-edit"));
    typeInEditor(within(contextPage).getByTestId("context-input"), "我的說明");
    fireEvent.click(within(contextPage).getByTestId("context-save"));
    const panel = await screen.findByTestId("policy-conflict-panel");
    fireEvent.click(within(panel).getByRole("button", { name: "檢視後以最新 revision 重新提交" }));
    await waitFor(() => expect(screen.queryByTestId("policy-conflict-panel")).toBeNull());

    goto("review");
    const page = screen.getByTestId("review-page");
    expect(within(page).getByTestId("review-section-warning").textContent).toContain("別人剛加的 WARNING");
    fireEvent.click(within(page).getByTestId("review-edit"));
    typeInEditor(within(page).getByTestId("review-input-suggestion"), "我的 SUGGESTION");
    fireEvent.click(within(page).getByTestId("review-save"));
    await waitFor(() =>
      expect(ws.writeWorkflowReview).toHaveBeenCalledWith({ ...serverReview, suggestion: ["我的 SUGGESTION"] }),
    );
  });
});
