// 規格詳情頁（spec「桌面 app 呈現 change 與 spec 的清單與內容」；desktop-readonly-detail-pages
// design D2）：標題列＋meta 列＋搜尋需求、「規格 N」「溯源 M」分頁、需求分段（Requirement 小標、
// h2 標題、來源錨點、SHALL 灰底、Scenario 縮排）、需求目錄、溯源列；缺件空態、世代重載不清空、
// latest-wins、換目標清空重載與 skeleton 三態沿用規格抽屜的既有行為。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, waitFor, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { SpecDetailPage } from "../components/SpecDetailPage";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const SPEC_MD = `# desktop-app Specification

## Purpose

桌面 app 的行為。

## Requirements

### Requirement: 顯示清單

清單內文。

<!-- @trace
source: change-one
updated: 2026-07-01
code:
  - a.rs
-->

### Requirement: 其他

其他內文。

<!-- @trace
source: change-two
updated: 2026-07-02
code:
  - b.rs
-->

<!-- @trace
source: change-one
updated: 2026-07-03
code:
  - c.rs
-->
`;

/** 三段需求，各帶 Scenario 與 @trace（spec Scenario「選定 spec 以詳情頁顯示需求分段」）。 */
const THREE_MD = `# demo Specification

## Purpose

示範規格。

## Requirements

### Requirement: 登入
使用者 SHALL 能登入。

#### Scenario: 正確密碼
- **WHEN** 輸入正確密碼
- **THEN** 進入首頁

<!-- @trace
source: add-login
updated: 2026-07-05
-->

---
### Requirement: 登出
使用者 MUST 能登出。

#### Scenario: 點登出
- **WHEN** 點登出
- **THEN** 回登入頁

<!-- @trace
source: add-logout
updated: 2026-08-01T10:00:00+08:00
-->

---
### Requirement: 記住帳號
系統 SHALL NOT 在未勾選時記住帳號。

#### Scenario: 未勾選
- **WHEN** 未勾選記住我
- **THEN** 下次需重新輸入

<!-- @trace
source: add-login
updated: 2026-07-20
-->
`;

const NO_TRACE_MD = `# plain Specification

## Purpose

無溯源內文。

## Requirements

### Requirement: 唯一

唯一內文。
`;

/** 封存清單 fixture：change-two 較晚封存、change-one 較早——與文件出現序相反，驗證排序取日期。 */
const ARCHIVED = [
  { datedName: "2026-07-02-change-two", date: "2026-07-02", name: "change-two" },
  { datedName: "2026-07-01-change-one", date: "2026-07-01", name: "change-one" },
];

function makeProps(over: Record<string, unknown> = {}) {
  return {
    capability: "desktop-app",
    loadDocument: vi.fn(async () => SPEC_MD),
    archivedChanges: ARCHIVED,
    onOpenArchivedChange: vi.fn(),
    ...over,
  };
}

const metaRow = () => document.querySelector("[data-provenance-row]") as HTMLElement | null;
const sections = () => Array.from(document.querySelectorAll<HTMLElement>("section[data-requirement]"));
const toc = () => document.querySelector("[data-manual-toc]") as HTMLElement | null;
const traceRows = () => Array.from(document.querySelectorAll<HTMLElement>("[data-trace-row]"));
const openTraceTab = async () => {
  await waitFor(() => screen.getByRole("tab", { name: /溯源/ }));
  fireEvent.mouseDown(screen.getByRole("tab", { name: /溯源/ }));
};
const clipboard = () => {
  const writeText = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
  return writeText;
};

describe("SpecDetailPage（標頭與分頁）", () => {
  it("開啟載入正典全文：標題為 capability、分頁「規格 2」「溯源 2」、內文呈現", async () => {
    const props = makeProps();
    render(<SpecDetailPage {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    expect(props.loadDocument).toHaveBeenCalledWith("desktop-app");
    expect(screen.getByRole("heading", { level: 1, name: "desktop-app" })).toBeTruthy();
    expect(screen.getByText("其他內文。")).toBeTruthy();
    expect(screen.getByRole("tab", { name: /規格/ }).textContent).toContain("2");
    expect(screen.getByRole("tab", { name: /溯源/ }).textContent).toContain("2");
  });

  it("meta 列：N 條需求 · 來自 M 個已封存變更 · 最近更新 日期（變更）", async () => {
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => THREE_MD) }) as never)} />);
    await waitFor(() => expect(sections()).toHaveLength(3));
    expect(metaRow()!.textContent).toBe(
      "3 條需求·來自 2 個已封存變更·最近更新 2026-08-01（add-logout）",
    );
  });

  it("無 @trace 時 meta 無「來自」「最近更新」段、需求段無錨點、溯源徽章 0 並顯示空態", async () => {
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => NO_TRACE_MD) }) as never)} />);
    await waitFor(() => expect(screen.getByText("唯一內文。")).toBeTruthy());
    expect(metaRow()!.textContent).toBe("1 條需求");
    expect(document.querySelector("[data-trace-anchor]")).toBeNull();
    expect(screen.getByRole("tab", { name: /溯源/ }).textContent).toContain("0");
    await openTraceTab();
    await waitFor(() => expect(screen.getByText("此規格沒有來源變更")).toBeTruthy());
    expect(traceRows()).toHaveLength(0);
  });

  it("複製名稱鈕寫入 capability 名並顯示已複製回饋", async () => {
    const writeText = clipboard();
    render(<SpecDetailPage {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    fireEvent.click(screen.getByLabelText("複製名稱"));
    expect(writeText).toHaveBeenCalledWith("desktop-app");
    await waitFor(() => expect(screen.getByRole("status").textContent).toBe("已複製"));
  });

  it("「開啟 ▾」收到正式規格的文件參照與已載入原文", async () => {
    const renderDocumentActions = vi.fn(() => null);
    render(<SpecDetailPage {...(makeProps({ renderDocumentActions }) as never)} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    expect(renderDocumentActions).toHaveBeenLastCalledWith({ doc: { spec: "desktop-app" }, content: SPEC_MD });
  });

  it("文件缺席顯示空狀態而非錯誤，meta 列缺席", async () => {
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => null) }) as never)} />);
    await waitFor(() => expect(screen.getByText("（無內容）")).toBeTruthy());
    expect(metaRow()).toBeNull();
  });
});

describe("SpecDetailPage（需求分段）", () => {
  it("三段需求各有 Requirement 小標、h2 標題、來源錨點、SHALL 灰底與 Scenario 縮排區塊", async () => {
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => THREE_MD) }) as never)} />);
    await waitFor(() => expect(sections()).toHaveLength(3));
    const [login, logout, remember] = sections();
    for (const s of sections()) {
      expect(s.querySelector("[data-requirement-label]")!.textContent).toBe("Requirement");
      expect(s.querySelector("[data-scenario]")!.className).toContain("border-l-2");
    }
    expect(login.querySelector("h2")!.textContent).toBe("登入");
    expect(login.querySelector("[data-trace-anchor]")!.textContent).toBe("來自 add-login · 2026-07-05");
    expect(logout.querySelector("[data-trace-anchor]")!.textContent).toBe("來自 add-logout · 2026-08-01");
    expect(login.querySelector("code.spec-kw")!.textContent).toBe("SHALL");
    expect(logout.querySelector("code.spec-kw")!.textContent).toBe("MUST");
    expect(remember.querySelector("code.spec-kw")!.textContent).toBe("SHALL NOT");
    expect(login.querySelector("[data-scenario]")!.textContent).toContain("Scenario：正確密碼");
    // Purpose 段在需求段之前、為 h2。
    expect(screen.getByRole("heading", { level: 2, name: "Purpose" })).toBeTruthy();
    expect(screen.getByText("示範規格。")).toBeTruthy();
    // 分隔線與 @trace 註解不入內文。
    expect(document.querySelector("[data-spec-body] hr")).toBeNull();
  });

  it("錨點在封存清單命中時可點，點擊以 datedName 開啟；未命中時為純文字", async () => {
    const props = makeProps({
      loadDocument: vi.fn(async () => THREE_MD),
      archivedChanges: [{ datedName: "2026-07-05-add-login", date: "2026-07-05", name: "add-login" }],
    });
    render(<SpecDetailPage {...(props as never)} />);
    await waitFor(() => expect(sections()).toHaveLength(3));
    const [login, logout] = sections();
    fireEvent.click(within(login).getByRole("button", { name: /add-login/ }));
    expect(props.onOpenArchivedChange).toHaveBeenCalledWith("2026-07-05-add-login");
    expect(within(logout).queryByRole("button", { name: /add-logout/ })).toBeNull();
  });

  it("需求目錄列出 Purpose 與全部需求，且只在頁寬 ≥1200px 的容器下顯示", async () => {
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => THREE_MD) }) as never)} />);
    await waitFor(() => expect(sections()).toHaveLength(3));
    await waitFor(() => expect(toc()).toBeTruthy());
    const anchors = Array.from(toc()!.querySelectorAll("[data-manual-anchor]")).map((a) => a.textContent);
    expect(anchors).toEqual(["Purpose", "登入", "登出", "記住帳號"]);
    // jsdom 不跑容器查詢：只能斷言顯示條件的 class，版面在真機驗收。
    const gutter = document.querySelector("[data-spec-toc]") as HTMLElement;
    expect(gutter.className).toContain("hidden");
    expect(gutter.className).toContain("@min-[1200px]:block");
    expect(document.querySelector("[data-spec-detail]")!.className).toContain("@container");
  });

  it("目錄高亮跟著包住內文的祖先捲動容器移動，其他元素的捲動不理會", async () => {
    render(
      <div>
        <div data-testid="scroller">
          <SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => THREE_MD) }) as never)} />
        </div>
        <div data-testid="elsewhere" />
      </div>,
    );
    await waitFor(() => expect(toc()).toBeTruthy());
    const anchor = (text: string) =>
      Array.from(toc()!.querySelectorAll("[data-manual-anchor]")).find((a) => a.textContent === text)!;
    expect(anchor("Purpose").getAttribute("aria-current")).toBe("location");
    // 頁層頂緣 100px：Purpose、登入、登出已捲過頂緣下 12px 線，記住帳號還在下方。
    const scroller = screen.getByTestId("scroller");
    const rect = (top: number) => () => ({ top }) as DOMRect;
    scroller.getBoundingClientRect = rect(100);
    const heads = Array.from(document.querySelectorAll<HTMLElement>("[data-spec-heading]"));
    heads.forEach((h, i) => (h.getBoundingClientRect = rect(i < 3 ? 50 : 400)));
    fireEvent.scroll(screen.getByTestId("elsewhere"));
    expect(anchor("Purpose").getAttribute("aria-current")).toBe("location");
    fireEvent.scroll(scroller);
    expect(anchor("登出").getAttribute("aria-current")).toBe("location");
    expect(anchor("Purpose").getAttribute("aria-current")).toBeNull();
  });

  it("搜尋需求只留標題命中的段，目錄隨之縮減；無命中顯示空態；清空還原", async () => {
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => THREE_MD) }) as never)} />);
    await waitFor(() => expect(sections()).toHaveLength(3));
    const search = screen.getByPlaceholderText("搜尋需求…");
    fireEvent.change(search, { target: { value: "登出" } });
    expect(sections().map((s) => s.querySelector("h2")!.textContent)).toEqual(["登出"]);
    expect(screen.getByRole("heading", { level: 2, name: "Purpose" })).toBeTruthy();
    await waitFor(() =>
      expect(Array.from(toc()!.querySelectorAll("[data-manual-anchor]")).map((a) => a.textContent)).toEqual([
        "Purpose",
        "登出",
      ]),
    );
    fireEvent.change(search, { target: { value: "zzz" } });
    expect(sections()).toHaveLength(0);
    expect(screen.getByText("沒有符合的需求")).toBeTruthy();
    fireEvent.change(search, { target: { value: "" } });
    expect(sections()).toHaveLength(3);
  });

  it("搜尋大小寫不敏感", async () => {
    const md = "## Purpose\n\np\n\n### Requirement: Login Flow\nA.\n\n### Requirement: Other\nB.\n";
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => md) }) as never)} />);
    await waitFor(() => expect(sections()).toHaveLength(2));
    fireEvent.change(screen.getByPlaceholderText("搜尋需求…"), { target: { value: "login" } });
    expect(sections().map((s) => s.querySelector("h2")!.textContent)).toEqual(["Login Flow"]);
  });

  it("無需求標題的規格整篇單一 markdown、無目錄、無錯誤", async () => {
    const md = "# x\n\n## Purpose\n\n只有目的。\n\n## 其他\n\n雜項。\n";
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => md) }) as never)} />);
    await waitFor(() => expect(screen.getByText("雜項。")).toBeTruthy());
    expect(sections()).toHaveLength(0);
    expect(toc()).toBeNull();
    expect(document.querySelector("[data-spec-toc]")).toBeNull();
  });

  it("需求段之外的內容照樣呈現：引言在 Purpose 之後、其他 `## ` 區段在需求之後", async () => {
    const md = [
      "# x Specification",
      "## Purpose",
      "目的。",
      "## Requirements",
      "這些需求適用於所有平台。",
      "### Requirement: A",
      "A 內文。",
      "## 附錄",
      "補充說明。",
    ].join("\n\n");
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => md) }) as never)} />);
    await waitFor(() => expect(sections()).toHaveLength(1));
    const intro = document.querySelector("[data-spec-intro]") as HTMLElement;
    const appendix = document.querySelector("[data-spec-appendix]") as HTMLElement;
    expect(intro.textContent).toContain("這些需求適用於所有平台。");
    expect(appendix.textContent).toContain("補充說明。");
    // 文件順序：Purpose → 引言 → 需求 → 附錄。
    const order = [document.querySelector("[data-purpose]")!, intro, sections()[0], appendix];
    for (let i = 1; i < order.length; i++) {
      expect(order[i - 1].compareDocumentPosition(order[i]) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    }
  });
});

describe("SpecDetailPage（溯源分頁）", () => {
  it("溯源列依封存日期升冪、副標為封存日期（spec Example「三個來源變更的排序」）", async () => {
    const doc = [
      "# x\n\n## Requirements\n\n### Requirement: a\n\n內文 a。\n",
      "<!-- @trace\nsource: drawer-polish\nupdated: 2026-08-04\n-->\n",
      "### Requirement: b\n\n內文 b。\n",
      "<!-- @trace\nsource: spec-archive-drawer\nupdated: 2026-07-11\n-->\n",
      "### Requirement: c\n\n內文 c。\n",
      "<!-- @trace\nsource: desktop-archived-parity\nupdated: 2026-08-11\n-->\n",
    ].join("\n");
    const props = makeProps({
      loadDocument: vi.fn(async () => doc),
      archivedChanges: [
        { datedName: "2026-08-04-drawer-polish", date: "2026-08-04", name: "drawer-polish" },
        { datedName: "2026-07-11-spec-archive-drawer", date: "2026-07-11", name: "spec-archive-drawer" },
        { datedName: "2026-08-11-desktop-archived-parity", date: "2026-08-11", name: "desktop-archived-parity" },
      ],
    });
    render(<SpecDetailPage {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("內文 a。")).toBeTruthy());
    expect(screen.getByRole("tab", { name: /溯源/ }).textContent).toContain("3");
    await openTraceTab();
    await waitFor(() => expect(traceRows()).toHaveLength(3));
    expect(traceRows().map((r) => r.textContent)).toEqual([
      expect.stringContaining("spec-archive-drawer"),
      expect.stringContaining("drawer-polish"),
      expect.stringContaining("desktop-archived-parity"),
    ]);
    expect(traceRows()[0].textContent).toContain("2026-07-11");
    expect(traceRows()[1].textContent).toContain("2026-08-04");
    expect(traceRows()[2].textContent).toContain("2026-08-11");
  });

  it("點命中列以 datedName 呼叫 onOpenArchivedChange", async () => {
    const props = makeProps();
    render(<SpecDetailPage {...(props as never)} />);
    await openTraceTab();
    await waitFor(() => expect(traceRows()).toHaveLength(2));
    fireEvent.click(traceRows()[0]);
    expect(props.onOpenArchivedChange).toHaveBeenCalledWith("2026-07-01-change-one");
    fireEvent.click(traceRows()[1]);
    expect(props.onOpenArchivedChange).toHaveBeenCalledWith("2026-07-02-change-two");
  });

  it("無封存記錄的來源排在後、副標「無封存記錄」、aria-disabled 且點擊不呼叫", async () => {
    const props = makeProps({ archivedChanges: [ARCHIVED[1]] }); // 只有 change-one 有封存記錄
    render(<SpecDetailPage {...(props as never)} />);
    await openTraceTab();
    await waitFor(() => expect(traceRows()).toHaveLength(2));
    const [hit, ghost] = traceRows();
    expect(hit.textContent).toContain("change-one");
    expect(ghost.textContent).toContain("change-two");
    expect(ghost.textContent).toContain("無封存記錄");
    expect(ghost.getAttribute("aria-disabled")).toBe("true");
    fireEvent.click(ghost);
    expect(props.onOpenArchivedChange).not.toHaveBeenCalled();
  });
});

describe("SpecDetailPage（載入）", () => {
  it("refreshGen 世代重載不清空，回應到達後單次替換", async () => {
    const pending: Array<(v: string | null) => void> = [];
    let hang = false;
    const loadDocument = vi.fn((_cap: string) =>
      hang ? new Promise<string | null>((r) => pending.push(r)) : Promise.resolve<string | null>(SPEC_MD),
    );
    const props = makeProps({ loadDocument });
    const { rerender } = render(<SpecDetailPage {...(props as never)} refreshGen={0} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    hang = true;
    rerender(<SpecDetailPage {...(props as never)} refreshGen={1} />);
    await waitFor(() => expect(pending.length).toBe(1));
    expect(screen.getByText("清單內文。")).toBeTruthy();
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
    pending[0]("# 新版\n\n更新後內文。\n");
    await waitFor(() => expect(screen.getByText("更新後內文。")).toBeTruthy());
    expect(screen.queryByText("清單內文。")).toBeNull();
  });

  it("latest-wins：舊世代回應晚到不覆蓋新世代內容", async () => {
    const pending: Array<(v: string | null) => void> = [];
    const loadDocument = vi.fn(() => new Promise<string | null>((r) => pending.push(r)));
    const props = makeProps({ loadDocument });
    const { rerender } = render(<SpecDetailPage {...(props as never)} refreshGen={0} />);
    await waitFor(() => expect(pending.length).toBe(1));
    rerender(<SpecDetailPage {...(props as never)} refreshGen={1} />);
    await waitFor(() => expect(pending.length).toBe(2));
    pending[1]("# v2\n\nv2 內文。\n");
    await waitFor(() => expect(screen.getByText("v2 內文。")).toBeTruthy());
    pending[0]("# v1\n\nv1-stale 內文。\n");
    await new Promise((r) => setTimeout(r, 25));
    expect(screen.queryByText("v1-stale 內文。")).toBeNull();
    expect(screen.getByText("v2 內文。")).toBeTruthy();
  });

  it("換目標時清空並全量重載，搜尋字串歸零", async () => {
    const loadDocument = vi.fn(async (cap: string) =>
      cap === "desktop-app" ? SPEC_MD : "# 另一份\n\n另一份內文。\n",
    );
    const props = makeProps({ loadDocument });
    const { rerender } = render(<SpecDetailPage {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    fireEvent.change(screen.getByPlaceholderText("搜尋需求…"), { target: { value: "清單" } });
    rerender(<SpecDetailPage {...(props as never)} capability="desktop-config" />);
    await waitFor(() => expect(screen.getByText("另一份內文。")).toBeTruthy());
    expect(loadDocument).toHaveBeenCalledWith("desktop-config");
    expect(screen.queryByText("清單內文。")).toBeNull();
    expect((screen.getByPlaceholderText("搜尋需求…") as HTMLInputElement).value).toBe("");
  });

  it("內文在共用置中閱讀欄內", async () => {
    render(<SpecDetailPage {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    const col = document.querySelector("[data-reading-column]") as HTMLElement;
    expect(col.className).toContain("max-w-[96ch]");
    expect(col.textContent).toContain("清單內文。");
  });
});

// spec「詳情頁文件載入以 skeleton 呈現」：三態分流。
describe("規格詳情頁文件三態", () => {
  it("載入中 → 文件骨架，不出空態文案", async () => {
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(() => new Promise<never>(() => {})) }) as never)} />);
    await waitFor(() => expect(document.querySelector('[aria-busy="true"]')).toBeTruthy());
    expect(screen.queryByText("（無內容）")).toBeNull();
  });

  it("載入完成且不存在 → 空態文案，無骨架", async () => {
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => null) }) as never)} />);
    await waitFor(() => expect(screen.getByText("（無內容）")).toBeTruthy());
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });

  it("首載失敗 → 落到空態文案，不停在骨架", async () => {
    const props = makeProps({ loadDocument: vi.fn().mockRejectedValue(new Error("offline")) });
    render(<SpecDetailPage {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("（無內容）")).toBeTruthy());
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });
});
