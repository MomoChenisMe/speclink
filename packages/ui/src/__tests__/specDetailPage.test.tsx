// 規格詳情頁（spec「桌面 app 呈現 change 與 spec 的清單與內容」；desktop-spec-manual-pages
// design D1–D5）：三欄骨架——左欄身分區、「文件」節點規格／溯源、「狀態」三列；閱讀卡標題區
// （需求數籤、搜尋需求、「開啟 ▾」、出處行）、數字磚、提示框、Purpose 引言段、需求段（第一條
// 展開、其餘收合）；右欄本頁目錄；溯源文件的出處行、數字磚與列。缺件空態、世代重載不清空、
// latest-wins、換目標清空重載與 skeleton 三態沿用既有行為。
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

/** 三段需求，各帶兩個 Scenario 與 @trace（spec Scenario「選定 spec 以詳情頁顯示需求分段」）。 */
const THREE_MD = `# demo Specification

## Purpose

示範規格。

## Requirements

### Requirement: 登入
使用者 SHALL 能登入。

#### Scenario: 正確密碼
- **WHEN** 輸入正確密碼
- **THEN** 進入首頁

#### Scenario: 錯誤密碼
- **WHEN** 輸入錯誤密碼
- **THEN** 顯示錯誤

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

#### Scenario: 逾時登出
- **WHEN** 閒置逾時
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

#### Scenario: 已勾選
- **WHEN** 勾選記住我
- **THEN** 下次帶入帳號

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
    // 預設 spec 目錄：core 解析出的相對路徑與契約預設版面同字。
    locateDocument: vi.fn(async (doc: { spec?: string }) => ({ relative: `openspec/specs/${doc.spec}/spec.md` })),
    ...over,
  };
}

/** Scenario「規格文件的出處行、數字磚與狀態列」：15 條需求、74 個 scenario、9 個 `@trace` 來源變更、
 * 最近更新 2026-10-12 來自 tray-panel-restyle。`untraced` 給的標題取代最後幾條需求、且不帶 @trace。 */
const TRAY_SOURCES = [
  "tray-status-menu-init",
  "tray-quick-actions",
  "tray-project-switch",
  "tray-discussion-rows",
  "tray-remote-panel",
  "tray-quality-stamps",
  "tray-load-states",
  "tray-overflow-fold",
  "tray-panel-restyle",
];
const TRAY_UPDATED = [
  "2026-09-10",
  "2026-09-11",
  "2026-09-12",
  "2026-09-13",
  "2026-09-14",
  "2026-09-15",
  "2026-09-16",
  "2026-09-17",
  "2026-10-12T16:18:33+08:00",
];
function trayMd({ untraced = [] as string[], purpose = "系統匣選單的行為。" } = {}): string {
  const reqs = Array.from({ length: 15 }, (_, i) => {
    const bare = i >= 15 - untraced.length;
    const title = bare ? untraced[i - (15 - untraced.length)] : `需求 ${i + 1}`;
    // 14 條各 5 個、最後一條 4 個：加總 74。
    const scenarios = Array.from(
      { length: i === 14 ? 4 : 5 },
      (_, j) => `#### Scenario: 情境 ${i + 1}-${j + 1}\n- **WHEN** 甲\n- **THEN** 乙\n`,
    ).join("\n");
    const k = i % 9;
    const trace = bare ? "" : `<!-- @trace\nsource: ${TRAY_SOURCES[k]}\nupdated: ${TRAY_UPDATED[k]}\n-->\n`;
    return `### Requirement: ${title}\n\n系統 SHALL 做事 ${i + 1}。\n\n${scenarios}\n${trace}`;
  });
  return `# tray-status-menu Specification\n\n## Purpose\n\n${purpose}\n\n## Requirements\n\n${reqs.join("\n---\n")}`;
}

const identityRow = () => document.querySelector("[data-identity-provenance]") as HTMLElement | null;
const docProvenance = () => document.querySelector("[data-doc-provenance]") as HTMLElement | null;
const tiles = () => Array.from(document.querySelectorAll<HTMLElement>("[data-doc-tile]")).map((el) => el.textContent);
const callout = (kind: string) => document.querySelector(`[data-callout="${kind}"]`) as HTMLElement | null;
const tab = (name: RegExp) => screen.getByRole("tab", { name });
const sections = () => Array.from(document.querySelectorAll<HTMLElement>("section[data-requirement]"));
const toc = () => document.querySelector("[data-detail-toc] [data-manual-toc]") as HTMLElement | null;
const tocAnchors = () => Array.from(toc()?.querySelectorAll("[data-manual-anchor]") ?? []).map((a) => a.textContent);
/** 需求段標題列的收合鈕（有 collapse 時）。 */
const toggleOf = (s: HTMLElement) => s.querySelector("button[aria-expanded]") as HTMLElement;
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

describe("SpecDetailPage（左欄與閱讀卡標題）", () => {
  it("開啟載入正典全文：三欄骨架、身分區標題與「正式規格」籤、文件節點「規格 2」「溯源 2」、閱讀卡標題「規格」", async () => {
    const props = makeProps();
    render(<SpecDetailPage {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    expect(props.loadDocument).toHaveBeenCalledWith("desktop-app");
    expect(document.querySelector("[data-spec-detail][data-detail-shell]")).toBeTruthy();
    const identity = document.querySelector("[data-detail-identity]") as HTMLElement;
    expect(within(identity).getByRole("heading", { level: 1, name: "desktop-app" })).toBeTruthy();
    expect(within(identity).getByText("正式規格")).toBeTruthy();
    // 第二條需求預設收合：標題在、內文不出。
    expect(screen.getByRole("heading", { level: 2, name: "其他" })).toBeTruthy();
    expect(screen.queryByText("其他內文。")).toBeNull();
    expect(tab(/規格/).textContent).toContain("2");
    expect(tab(/溯源/).textContent).toContain("2");
    expect(tab(/規格/).getAttribute("aria-selected")).toBe("true");
    const header = document.querySelector("[data-doc-header]") as HTMLElement;
    expect(within(header).getByRole("heading", { level: 2 }).textContent).toBe("規格");
    expect(within(header).getByText("2")).toBeTruthy();
    expect(within(header).getByPlaceholderText("搜尋需求…")).toBeTruthy();
  });

  it("身分區出身行、溯源關聯列與狀態三列：N 條需求 · M 個 scenario · 最近更新 日期", async () => {
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => THREE_MD) }) as never)} />);
    await waitFor(() => expect(sections()).toHaveLength(3));
    expect(identityRow()!.textContent).toBe("3 條需求·6 個 scenario·最近更新 2026-08-01");
    const relation = document.querySelector("[data-relation-trace]") as HTMLElement;
    expect(relation.textContent).toBe("溯源add-login+1");
    expect(document.querySelector("[data-status-requirements]")!.textContent).toBe("需求3");
    expect(document.querySelector("[data-status-sources]")!.textContent).toBe("溯源2");
    const updated = document.querySelector("[data-status-updated]") as HTMLElement;
    expect(updated.textContent).toContain("2026-08-01");
    expect(updated.textContent).toContain("add-logout");
    expect(docProvenance()!.textContent).toBe(
      "openspec/specs/desktop-app/spec.md·正式規格·最近更新 2026-08-01（add-logout）",
    );
  });

  it("無 @trace 時身分區無最近更新段與溯源關聯列、狀態列無最近更新、需求段無錨點、溯源徽章 0 並顯示空態", async () => {
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => NO_TRACE_MD) }) as never)} />);
    await waitFor(() => expect(screen.getByText("唯一內文。")).toBeTruthy());
    expect(identityRow()!.textContent).toBe("1 條需求·0 個 scenario");
    expect(document.querySelector("[data-relation-trace]")).toBeNull();
    expect(document.querySelector("[data-status-updated]")).toBeNull();
    expect(document.querySelector("[data-trace-anchor]")).toBeNull();
    expect(tab(/溯源/).textContent).toContain("0");
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

  it("「開啟 ▾」收到正式規格的文件參照與已載入原文，渲染在閱讀卡標題列", async () => {
    const renderDocumentActions = vi.fn(() => <button type="button">開啟 ▾</button>);
    render(<SpecDetailPage {...(makeProps({ renderDocumentActions }) as never)} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    expect(renderDocumentActions).toHaveBeenLastCalledWith({ doc: { spec: "desktop-app" }, content: SPEC_MD });
    const header = document.querySelector("[data-doc-header]") as HTMLElement;
    expect(within(header).getByRole("button", { name: "開啟 ▾" })).toBeTruthy();
  });

  it("文件缺席顯示空狀態而非錯誤，身分區出身行缺席", async () => {
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => null) }) as never)} />);
    await waitFor(() => expect(screen.getByText("（無內容）")).toBeTruthy());
    expect(identityRow()).toBeNull();
  });

  it("出處行、四格磚與狀態列三列（Scenario「規格文件的出處行、數字磚與狀態列」資料）", async () => {
    render(
      <SpecDetailPage
        {...(makeProps({ capability: "tray-status-menu", loadDocument: vi.fn(async () => trayMd()) }) as never)}
      />,
    );
    await waitFor(() => expect(sections()).toHaveLength(15));
    expect(docProvenance()!.textContent).toBe(
      "openspec/specs/tray-status-menu/spec.md·正式規格·最近更新 2026-10-12（tray-panel-restyle）",
    );
    expect(tiles()).toEqual(["15需求", "74Scenario", "9溯源變更", "2026-10-12最近更新 · tray-panel-restyle"]);
    expect(document.querySelector("[data-status-requirements]")!.textContent).toBe("需求15");
    expect(document.querySelector("[data-status-sources]")!.textContent).toBe("溯源9");
    expect(document.querySelector("[data-status-updated]")!.textContent).toContain("2026-10-12");
    expect(document.querySelector("[data-doc-callout]")).toBeNull();
  });

  it("點溯源關聯列後溯源節點作用中、溯源文件的出處行與四格磚", async () => {
    render(
      <SpecDetailPage
        {...(makeProps({ capability: "tray-status-menu", loadDocument: vi.fn(async () => trayMd()) }) as never)}
      />,
    );
    await waitFor(() => expect(sections()).toHaveLength(15));
    const relation = document.querySelector("[data-relation-trace]") as HTMLElement;
    // 「+N」籤本身是灰底：滑過換主色淡底，不是更淡的灰。
    expect(within(relation).getByRole("button", { name: "+8" }).className).toContain("hover:bg-accent");
    fireEvent.click(within(relation).getByRole("button", { name: "+8" }));
    await waitFor(() => expect(tab(/溯源/).getAttribute("aria-selected")).toBe("true"));
    const header = document.querySelector("[data-doc-header]") as HTMLElement;
    expect(within(header).getByRole("heading", { level: 2 }).textContent).toBe("溯源");
    expect(docProvenance()!.textContent).toBe(
      "自全文的 @trace 註解推導，不是檔案·封存清單命中者在前、依封存日期升冪",
    );
    expect(tiles()).toEqual([
      "9來源變更",
      "2026-09-10最早 · tray-status-menu-init",
      "2026-10-12最近 · tray-panel-restyle",
      "15 ／ 15有來源的需求",
    ]);
    expect(traceRows()).toHaveLength(9);
    // 首顆變更籤同樣切到溯源文件。
    fireEvent.mouseDown(tab(/規格/));
    await waitFor(() => expect(tab(/規格/).getAttribute("aria-selected")).toBe("true"));
    fireEvent.click(within(document.querySelector("[data-relation-trace]") as HTMLElement).getByRole("button", { name: "tray-status-menu-init" }));
    await waitFor(() => expect(tab(/溯源/).getAttribute("aria-selected")).toBe("true"));
  });

  it("Purpose 以 TBD 前綴起頭時 warning 提示框含路徑；兩條需求無 @trace 時 note 提示框列出標題且溯源磚為 13 ／ 15", async () => {
    const md = trayMd({
      untraced: ["選單內容與看板同源", "分區溢出摺疊"],
      purpose: "TBD - created by archiving change tray-status-menu. Update Purpose after archive.",
    });
    render(
      <SpecDetailPage {...(makeProps({ capability: "tray-status-menu", loadDocument: vi.fn(async () => md) }) as never)} />,
    );
    await waitFor(() => expect(sections()).toHaveLength(15));
    const tbd = callout("purpose-tbd")!;
    const untraced = callout("untraced")!;
    expect(tbd.textContent).toContain("Purpose 待補");
    expect(tbd.textContent).toContain("openspec/specs/tray-status-menu/spec.md");
    expect(untraced.textContent).toContain("2 條需求沒有來源變更");
    expect(untraced.textContent).toContain("選單內容與看板同源、分區溢出摺疊");
    // 兩者都在提示框槽內、Purpose 待補在前；Purpose 引言段仍呈現佔位原文。
    const slot = document.querySelector("[data-doc-callout]") as HTMLElement;
    expect(slot.contains(tbd) && slot.contains(untraced)).toBe(true);
    expect(tbd.compareDocumentPosition(untraced) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(document.querySelector("[data-purpose]")!.textContent).toContain("TBD - created by archiving");
    const [, , , , , , , , , , , , , bare1, bare2] = sections();
    expect(bare1.querySelector("[data-trace-anchor]")).toBeNull();
    expect(bare2.querySelector("[data-trace-anchor]")).toBeNull();
    await openTraceTab();
    await waitFor(() => expect(tiles()[3]).toBe("13 ／ 15有來源的需求"));
  });

  it("專案改過 spec 目錄名：出處行與 Purpose 待補提示框都用 core 解析出的實際相對路徑", async () => {
    const locateDocument = vi.fn(async () => ({ relative: "docs/specs/specs/tray-status-menu/spec.md" }));
    const md = trayMd({ purpose: "TBD - created by archiving change tray-status-menu. Update Purpose after archive." });
    render(
      <SpecDetailPage
        {...(makeProps({ capability: "tray-status-menu", loadDocument: vi.fn(async () => md), locateDocument }) as never)}
      />,
    );
    await waitFor(() => expect(sections()).toHaveLength(15));
    await waitFor(() =>
      expect(docProvenance()!.textContent).toBe(
        "docs/specs/specs/tray-status-menu/spec.md·正式規格·最近更新 2026-10-12（tray-panel-restyle）",
      ),
    );
    expect(locateDocument).toHaveBeenCalledWith({ spec: "tray-status-menu" });
    expect(callout("purpose-tbd")!.textContent).toContain("docs/specs/specs/tray-status-menu/spec.md");
    expect(callout("purpose-tbd")!.textContent).not.toContain("openspec/specs");
  });

  it("沒有可定位的檔案（remote 無 checkout）：出處行不列路徑、Purpose 待補提示框改用不帶路徑的說法", async () => {
    const md = trayMd({ purpose: "TBD - created by archiving change tray-status-menu. Update Purpose after archive." });
    render(
      <SpecDetailPage
        {...(makeProps({
          capability: "tray-status-menu",
          loadDocument: vi.fn(async () => md),
          locateDocument: undefined,
        }) as never)}
      />,
    );
    await waitFor(() => expect(sections()).toHaveLength(15));
    expect(docProvenance()!.textContent).toBe("正式規格·最近更新 2026-10-12（tray-panel-restyle）");
    const tbd = callout("purpose-tbd")!;
    expect(tbd.textContent).toContain("直接編輯這份規格檔的 Purpose 段即可");
    expect(tbd.textContent).not.toContain("spec.md");
  });

  it("無 @trace 時無最近更新段、無溯源關聯列、note 提示框列出全部需求、溯源文件無磚", async () => {
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => NO_TRACE_MD) }) as never)} />);
    await waitFor(() => expect(sections()).toHaveLength(1));
    expect(docProvenance()!.textContent).toBe("openspec/specs/desktop-app/spec.md·正式規格");
    expect(tiles()).toEqual(["1需求", "0Scenario", "0溯源變更"]);
    expect(callout("untraced")!.textContent).toContain("1 條需求沒有來源變更");
    expect(callout("untraced")!.textContent).toContain("唯一");
    expect(callout("purpose-tbd")).toBeNull();
    await openTraceTab();
    await waitFor(() => expect(screen.getByText("此規格沒有來源變更")).toBeTruthy());
    expect(document.querySelector("[data-doc-tiles]")).toBeNull();
  });
});

describe("SpecDetailPage（需求分段）", () => {
  it("三段需求各有 Requirement 小標、[data-doc-heading] 標題、來源錨點、SHALL 灰底與 Scenario 縮排區塊", async () => {
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => THREE_MD) }) as never)} />);
    await waitFor(() => expect(sections()).toHaveLength(3));
    const [login, logout, remember] = sections();
    // 第二、三段預設收合：展開後再看內文。
    fireEvent.click(toggleOf(logout));
    fireEvent.click(toggleOf(remember));
    for (const s of sections()) {
      expect(s.querySelector("[data-requirement-label]")!.textContent).toBe("Requirement");
      expect(s.querySelector("[data-scenario]")!.className).toContain("border-l-2");
      expect(s.querySelector("[data-doc-heading]")!.tagName).toBe("H2");
      expect(s.querySelector("[data-doc-heading]")!.className).toContain("text-base font-semibold");
    }
    expect(login.querySelector("[data-doc-heading]")!.textContent).toBe("登入");
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
    expect(document.querySelector("[data-doc-body] hr")).toBeNull();
  });

  it("第一段展開、第二第三段收合為一列帶「2 個 scenario」籤；點列切換；點收合列上的錨點只開封存變更", async () => {
    const props = makeProps({
      loadDocument: vi.fn(async () => THREE_MD),
      archivedChanges: [{ datedName: "2026-08-01-add-logout", date: "2026-08-01", name: "add-logout" }],
    });
    render(<SpecDetailPage {...(props as never)} />);
    await waitFor(() => expect(sections()).toHaveLength(3));
    const [login, logout, remember] = sections();
    expect(toggleOf(login).getAttribute("aria-expanded")).toBe("true");
    expect(login.querySelectorAll("[data-scenario]")).toHaveLength(2);
    expect(login.textContent).not.toContain("2 個 scenario");
    for (const s of [logout, remember]) {
      expect(toggleOf(s).getAttribute("aria-expanded")).toBe("false");
      expect(s.querySelector("[data-scenario]")).toBeNull();
      expect(s.textContent).toContain("2 個 scenario");
      expect(s.querySelector("[data-doc-heading]")).toBeTruthy();
    }
    // 收合列看得出可點：共用游標變數（網頁手指、桌面箭頭），滑過整列（含錨點）有淡底。
    expect(toggleOf(logout).className).toContain("cursor-(--cursor-interaction)");
    expect(toggleOf(logout).parentElement!.className).toContain("hover:bg-foreground/5");
    // 收合列上的可點錨點：開該封存變更，不切換收合。
    fireEvent.click(within(logout).getByRole("button", { name: /add-logout/ }));
    expect(props.onOpenArchivedChange).toHaveBeenCalledWith("2026-08-01-add-logout");
    expect(toggleOf(logout).getAttribute("aria-expanded")).toBe("false");
    // 點列展開、第一段維持展開；再點收合。
    fireEvent.click(toggleOf(logout));
    expect(toggleOf(logout).getAttribute("aria-expanded")).toBe("true");
    expect(logout.querySelectorAll("[data-scenario]")).toHaveLength(2);
    expect(toggleOf(login).getAttribute("aria-expanded")).toBe("true");
    fireEvent.click(toggleOf(logout));
    expect(toggleOf(logout).getAttribute("aria-expanded")).toBe("false");
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

  it("右欄本頁目錄列出 Purpose 與全部需求，且只在主區寬 ≥1200px 的容器下顯示", async () => {
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => THREE_MD) }) as never)} />);
    await waitFor(() => expect(sections()).toHaveLength(3));
    await waitFor(() => expect(toc()).toBeTruthy());
    expect(tocAnchors()).toEqual(["Purpose", "登入", "登出", "記住帳號"]);
    // jsdom 不跑容器查詢：只能斷言顯示條件的 class，版面在真機驗收。
    const aside = document.querySelector("[data-detail-toc]") as HTMLElement;
    expect(aside.className).toContain("hidden");
    expect(aside.className).toContain("@min-[1200px]:flex");
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
    const heads = Array.from(document.querySelectorAll<HTMLElement>("[data-doc-heading]"));
    heads.forEach((h, i) => (h.getBoundingClientRect = rect(i < 3 ? 50 : 400)));
    fireEvent.scroll(screen.getByTestId("elsewhere"));
    expect(anchor("Purpose").getAttribute("aria-current")).toBe("location");
    fireEvent.scroll(scroller);
    expect(anchor("登出").getAttribute("aria-current")).toBe("location");
    expect(anchor("Purpose").getAttribute("aria-current")).toBeNull();
  });

  it("展開或收合需求段觸發目錄重掃時，高亮留在原本的目前段；搜尋換掉標題時才回到第一項", async () => {
    render(
      <div data-testid="scroller">
        <SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => THREE_MD) }) as never)} />
      </div>,
    );
    await waitFor(() => expect(sections()).toHaveLength(3));
    await waitFor(() => expect(toc()).toBeTruthy());
    const anchor = (text: string) =>
      Array.from(toc()!.querySelectorAll("[data-manual-anchor]")).find((a) => a.textContent === text)!;
    // 捲到「登出」為目前段（同上一題的量測替身）。
    const scroller = screen.getByTestId("scroller");
    const rect = (top: number) => () => ({ top }) as DOMRect;
    scroller.getBoundingClientRect = rect(100);
    const heads = Array.from(document.querySelectorAll<HTMLElement>("[data-doc-heading]"));
    heads.forEach((h, i) => (h.getBoundingClientRect = rect(i < 3 ? 50 : 400)));
    fireEvent.scroll(scroller);
    expect(anchor("登出").getAttribute("aria-current")).toBe("location");
    // 點開「登出」：展開不捲動，高亮不得跳回 Purpose。
    fireEvent.click(toggleOf(sections()[1]));
    await waitFor(() => expect(toggleOf(sections()[1]).getAttribute("aria-expanded")).toBe("true"));
    expect(anchor("登出").getAttribute("aria-current")).toBe("location");
    expect(anchor("Purpose").getAttribute("aria-current")).toBeNull();
    fireEvent.click(toggleOf(sections()[1]));
    await waitFor(() => expect(toggleOf(sections()[1]).getAttribute("aria-expanded")).toBe("false"));
    expect(anchor("登出").getAttribute("aria-current")).toBe("location");
    // 搜尋只留「記住帳號」：原本的目前段不在了，回到第一項。
    fireEvent.change(screen.getByPlaceholderText("搜尋需求…"), { target: { value: "記住" } });
    await waitFor(() => expect(tocAnchors()).toEqual(["Purpose", "記住帳號"]));
    expect(anchor("Purpose").getAttribute("aria-current")).toBe("location");
  });

  it("搜尋需求只留標題命中的段，目錄隨之縮減；無命中顯示空態；清空還原", async () => {
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => THREE_MD) }) as never)} />);
    await waitFor(() => expect(sections()).toHaveLength(3));
    const search = screen.getByPlaceholderText("搜尋需求…");
    fireEvent.change(search, { target: { value: "登出" } });
    expect(sections().map((s) => s.querySelector("h2")!.textContent)).toEqual(["登出"]);
    expect(screen.getByRole("heading", { level: 2, name: "Purpose" })).toBeTruthy();
    await waitFor(() => expect(tocAnchors()).toEqual(["Purpose", "登出"]));
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

  it("無需求標題時整篇 Markdown、無磚、無提示框、無錨點", async () => {
    const md = "# x\n\n## Purpose\n\n只有目的。\n\n## 其他\n\n雜項。\n";
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => md) }) as never)} />);
    await waitFor(() => expect(screen.getByText("雜項。")).toBeTruthy());
    expect(sections()).toHaveLength(0);
    expect(document.querySelector("[data-doc-tiles]")).toBeNull();
    expect(document.querySelector("[data-doc-callout]")).toBeNull();
    // 整篇的 `## ` 標題也不成錨點。
    await new Promise((r) => setTimeout(r, 0));
    expect(toc()).toBeNull();
  });

  it("搜尋後只剩命中段且該段展開、右欄錨點只剩 Purpose 與命中標題", async () => {
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => THREE_MD) }) as never)} />);
    await waitFor(() => expect(sections()).toHaveLength(3));
    fireEvent.change(screen.getByPlaceholderText("搜尋需求…"), { target: { value: "記住" } });
    expect(sections()).toHaveLength(1);
    expect(sections()[0].querySelector("[data-doc-heading]")!.textContent).toBe("記住帳號");
    expect(toggleOf(sections()[0]).getAttribute("aria-expanded")).toBe("true");
    expect(sections()[0].querySelectorAll("[data-scenario]")).toHaveLength(2);
    await waitFor(() => expect(tocAnchors()).toEqual(["Purpose", "記住帳號"]));
    // 清空還原：第一條展開、其餘收合。
    fireEvent.change(screen.getByPlaceholderText("搜尋需求…"), { target: { value: "" } });
    expect(sections().map((s) => toggleOf(s).getAttribute("aria-expanded"))).toEqual(["true", "false", "false"]);
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
    // 「附錄」列入本頁目錄。
    await waitFor(() => expect(tocAnchors()).toContain("附錄"));
  });

  it("收合的需求段展開後，內文的子標題補進本頁目錄；再收合即移除", async () => {
    const md = [
      "# x Specification",
      "## Purpose",
      "目的。",
      "## Requirements",
      "### Requirement: A",
      "A 內文。",
      "### Requirement: B",
      "B 內文。",
      "### 補充細節",
      "細節內文。",
    ].join("\n\n");
    render(<SpecDetailPage {...(makeProps({ loadDocument: vi.fn(async () => md) }) as never)} />);
    await waitFor(() => expect(sections()).toHaveLength(2));
    await waitFor(() => expect(tocAnchors()).toEqual(["Purpose", "A", "B"]));
    fireEvent.click(toggleOf(sections()[1]));
    await waitFor(() => expect(tocAnchors()).toEqual(["Purpose", "A", "B", "補充細節"]));
    fireEvent.click(toggleOf(sections()[1]));
    await waitFor(() => expect(tocAnchors()).toEqual(["Purpose", "A", "B"]));
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
      renderDocumentActions: () => <button type="button">開啟 ▾</button>,
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
    // 閱讀卡標題「溯源」帶籤 3；推導文件不對應檔案、無「開啟 ▾」。
    const header = document.querySelector("[data-doc-header]") as HTMLElement;
    expect(within(header).getByRole("heading", { level: 2 }).textContent).toBe("溯源");
    expect(within(header).getByText("3")).toBeTruthy();
    expect(within(header).queryByRole("button", { name: "開啟 ▾" })).toBeNull();
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

  it("內文在閱讀卡的 DocBody 內：15px、行寬以卡欄為上限", async () => {
    render(<SpecDetailPage {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    const body = document.querySelector("[data-doc-body]") as HTMLElement;
    expect(body.className).toContain("text-[15px]");
    expect(body.className).toContain("[&_.markdown]:max-w-none");
    expect(body.textContent).toContain("清單內文。");
    expect((document.querySelector("[data-detail-reading] > div") as HTMLElement).className).toContain("max-w-[768px]");
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
