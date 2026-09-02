// spec desktop-manual-page（design「側欄樹、搜尋與上下頁在前端由索引推導」）：
// 「手冊頁的側欄樹與閱讀序」「手冊頁的搜尋列」「內頁渲染與出處跳規格」
// 「可能過期與未入冊的標示」「無手冊與 remote 模式的空狀態」。索引由 props 注入
// （已依閱讀序排好），元件只做分組、過濾、上下頁與內文載入。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, waitFor, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { ManualPage } from "../components/ManualPage";
import type { ManualIndex, ManualPageItem } from "../adapter";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

function page(over: Partial<ManualPageItem> & { slug: string }): ManualPageItem {
  return {
    title: over.slug,
    section: null,
    order: null,
    keywords: [],
    sources: [],
    generated: null,
    stale: false,
    ...over,
  };
}

/** 索引 fixture：四個分區（開始使用×2、文件協作、附錄）＋缺欄頁（orphan）＋壞頁
 * （broken）——後兩者 section 為 null、order 為 null、已由 core 置於閱讀序末端。 */
const INDEX: ManualIndex = {
  present: true,
  reason: null,
  pages: [
    page({ slug: "index", title: "手冊", section: "開始使用", order: 10, generated: "2026-09-01" }),
    page({
      slug: "first-login",
      title: "第一次登入",
      section: "開始使用",
      order: 20,
      keywords: ["登入", "github", "審核"],
      sources: ["github-oauth", "user-pending-blocked-pages"],
      generated: "2026-09-01",
      stale: true,
    }),
    page({
      slug: "editor",
      title: "認識畫面",
      section: "文件協作",
      order: 30,
      sources: ["editor"],
      generated: "2026-09-01",
    }),
    page({ slug: "about", title: "本手冊的來源", section: "附錄", order: 40 }),
    page({ slug: "orphan" }),
    page({ slug: "broken" }),
  ],
  uncoveredNew: [],
  malformed: ["broken"],
};

const BODIES: Record<string, string> = {
  index: "# 手冊\n\n歡迎。",
  "first-login": "# 第一次登入\n\n用 GitHub 登入。\n\n**出處**：`github-oauth`、`user-pending-blocked-pages`\n",
  editor: "# 認識畫面\n\n> [!NOTE]\n> 提示內容\n\n**出處**：`editor`",
  about: "# 本手冊的來源\n\n取材範圍。",
  orphan: "沒有欄位的頁",
  broken: "---\ntitle: [unclosed\n---\n壞頁全文",
};

function loader(over: Record<string, () => Promise<string | null>> = {}) {
  return vi.fn(async (slug: string) => (slug in over ? over[slug]() : (BODIES[slug] ?? null)));
}

function renderManual(
  props: Partial<Parameters<typeof ManualPage>[0]> = {},
  loadPage = loader(),
) {
  const onOpenSpec = vi.fn();
  const view = render(
    <ManualPage
      index={INDEX}
      loadPage={loadPage}
      onOpenSpec={onOpenSpec}
      capabilities={["github-oauth", "editor"]}
      {...props}
    />,
  );
  return { ...view, onOpenSpec, loadPage };
}

const tree = () => document.querySelector("[data-manual-tree]") as HTMLElement;
const sections = () =>
  Array.from(tree().querySelectorAll("[data-manual-section]")).map((s) =>
    s.getAttribute("data-manual-section"),
  );
const rows = () =>
  Array.from(tree().querySelectorAll("[data-manual-page]")).map((r) => r.getAttribute("data-manual-page"));
const row = (slug: string) => tree().querySelector(`[data-manual-page="${slug}"]`) as HTMLElement;
const content = () => document.querySelector("[data-manual-content]") as HTMLElement;
const prevButton = () => screen.queryByRole("button", { name: "上一頁" });
const nextButton = () => screen.queryByRole("button", { name: "下一頁" });

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

describe("ManualPage 側欄樹與閱讀序", () => {
  it("依閱讀序分區與列序；缺 section 的頁歸「其他」；壞頁以檔名列出", async () => {
    // spec Scenario「依 order 排序並依 section 分組」Example＋「缺欄位的頁寬容降級」
    // ＋「frontmatter 壞掉的頁仍可開」。
    renderManual();
    await screen.findByText("歡迎。");
    expect(sections()).toEqual(["開始使用", "文件協作", "附錄", "其他"]);
    expect(rows()).toEqual(["index", "first-login", "editor", "about", "orphan", "broken"]);
    expect(row("first-login").textContent).toContain("第一次登入");
    expect(row("orphan").textContent).toContain("orphan");
    expect(row("broken").textContent).toContain("broken");
    expect(document.querySelector("[data-manual-no-results]")).toBeNull();
  });

  it("預設開啟首頁；上一頁／下一頁沿閱讀序，首頁無上一頁、末頁無下一頁", async () => {
    const { loadPage } = renderManual();
    await screen.findByText("歡迎。");
    expect(loadPage).toHaveBeenCalledWith("index");
    expect(row("index").getAttribute("aria-current")).toBe("page");
    expect(prevButton()).toBeNull();
    expect(nextButton()).toBeTruthy();

    fireEvent.click(nextButton()!);
    await screen.findByText("用 GitHub 登入。");
    expect(row("first-login").getAttribute("aria-current")).toBe("page");
    expect(row("index").getAttribute("aria-current")).toBeNull();
    expect(prevButton()).toBeTruthy();
    fireEvent.click(prevButton()!);
    await screen.findByText("歡迎。");

    fireEvent.click(row("broken"));
    await screen.findByText("壞頁全文");
    expect(nextButton()).toBeNull();
    expect(prevButton()).toBeTruthy();
  });

  it("壞頁與缺欄頁點擊可開，畫面無錯誤提示", async () => {
    renderManual();
    await screen.findByText("歡迎。");
    fireEvent.click(row("orphan"));
    await screen.findByText("沒有欄位的頁");
    fireEvent.click(row("broken"));
    await screen.findByText("壞頁全文");
    expect(screen.queryByText("內文載入失敗")).toBeNull();
  });

  it("索引重載後目前頁仍在則維持；消失則回首頁", async () => {
    const { rerender, loadPage } = renderManual();
    await screen.findByText("歡迎。");
    fireEvent.click(row("editor"));
    await screen.findByText("提示內容");
    const kept = { ...INDEX, pages: INDEX.pages.slice(0, 4) };
    rerender(
      <ManualPage index={kept} loadPage={loadPage} onOpenSpec={vi.fn()} capabilities={[]} />,
    );
    expect(row("editor").getAttribute("aria-current")).toBe("page");
    expect(screen.getByText("提示內容")).toBeTruthy();
    const withoutEditor = { ...INDEX, pages: INDEX.pages.filter((p) => p.slug !== "editor") };
    rerender(
      <ManualPage index={withoutEditor} loadPage={loadPage} onOpenSpec={vi.fn()} capabilities={[]} />,
    );
    await screen.findByText("歡迎。");
    expect(row("index").getAttribute("aria-current")).toBe("page");
  });
});

describe("ManualPage 搜尋列", () => {
  it("大小寫不敏感比對 title 與 keywords，只留命中頁及其分區；清空還原", async () => {
    // spec Scenario「以標題或關鍵字過濾」。
    renderManual();
    await screen.findByText("歡迎。");
    const input = screen.getByPlaceholderText("搜尋手冊…") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "GitHub" } });
    expect(rows()).toEqual(["first-login"]);
    expect(sections()).toEqual(["開始使用"]);
    fireEvent.change(input, { target: { value: "認識" } });
    expect(rows()).toEqual(["editor"]);
    expect(sections()).toEqual(["文件協作"]);
    fireEvent.change(input, { target: { value: "" } });
    expect(rows()).toEqual(["index", "first-login", "editor", "about", "orphan", "broken"]);
  });

  it("無命中顯示無結果文案，內容區維持目前頁；搜尋不比對內文", async () => {
    // spec Scenario「無命中顯示無結果」。
    renderManual();
    await screen.findByText("歡迎。");
    const input = screen.getByPlaceholderText("搜尋手冊…") as HTMLInputElement;
    // 「歡迎」只出現在內文，不在任何 title／keywords。
    fireEvent.change(input, { target: { value: "歡迎" } });
    expect(rows()).toEqual([]);
    expect(document.querySelector("[data-manual-no-results]")?.textContent).toContain("沒有符合的頁面");
    expect(screen.getByText("歡迎。")).toBeTruthy();
  });
});

describe("ManualPage 過期與未入冊標示", () => {
  it("stale 頁列帶「可能過期」標記，其他頁無", async () => {
    // spec Scenario「來源更新後標示可能過期」。
    renderManual();
    await screen.findByText("歡迎。");
    expect(within(row("first-login")).getByText("可能過期")).toBeTruthy();
    expect(within(row("index")).queryByText("可能過期")).toBeNull();
  });

  it("uncoveredNew 非空時側欄底部顯示計數提示；空時提示缺席", async () => {
    // spec Scenario「生成後新增的規格計入未入冊」。
    const { rerender, loadPage } = renderManual();
    await screen.findByText("歡迎。");
    expect(document.querySelector("[data-manual-uncovered]")).toBeNull();
    rerender(
      <ManualPage
        index={{ ...INDEX, uncoveredNew: ["billing", "tray-menu"] }}
        loadPage={loadPage}
        onOpenSpec={vi.fn()}
        capabilities={[]}
      />,
    );
    const hint = document.querySelector("[data-manual-uncovered]") as HTMLElement;
    expect(hint.textContent).toContain("2");
    expect(hint.textContent).toContain("未入冊");
  });
});

describe("ManualPage 內頁渲染與出處", () => {
  it("出處 capability：正典存在者可點並觸發 onOpenSpec，不存在者為純文字；出處行不重複進內文", async () => {
    // spec Scenario「點出處跳規格頁展開」＋「不存在的出處不可點」。
    const { onOpenSpec } = renderManual();
    await screen.findByText("歡迎。");
    fireEvent.click(row("first-login"));
    await screen.findByText("用 GitHub 登入。");
    const sources = document.querySelector("[data-manual-sources]") as HTMLElement;
    const link = within(sources).getByRole("button", { name: "github-oauth" });
    fireEvent.click(link);
    expect(onOpenSpec).toHaveBeenCalledWith("github-oauth");
    expect(within(sources).queryByRole("button", { name: "user-pending-blocked-pages" })).toBeNull();
    expect(sources.textContent).toContain("user-pending-blocked-pages");
    // 內文的 `**出處**：` 行已抽成頁尾出處列，不在 Markdown 內重複呈現。
    expect(content().querySelector(".markdown code")).toBeNull();
    expect(onOpenSpec).toHaveBeenCalledTimes(1);
  });

  it("內文沒有出處行時，頁尾出處列退回 frontmatter 的 sources", async () => {
    const { onOpenSpec } = renderManual({}, loader({ editor: async () => "# 認識畫面\n\n沒有出處行。" }));
    await screen.findByText("歡迎。");
    fireEvent.click(row("editor"));
    await screen.findByText("沒有出處行。");
    const sources = document.querySelector("[data-manual-sources]") as HTMLElement;
    fireEvent.click(within(sources).getByRole("button", { name: "editor" }));
    expect(onOpenSpec).toHaveBeenCalledWith("editor");
  });

  it("GitHub Alert 在手冊內文呈現為提示框、以共用閱讀欄渲染", async () => {
    renderManual();
    await screen.findByText("歡迎。");
    fireEvent.click(row("editor"));
    await screen.findByText("提示內容");
    expect(content().querySelector(".markdown-alert-note")).toBeTruthy();
    expect(content().querySelector(".max-w-\\[96ch\\]")).toBeTruthy();
  });

  it("內文載入失敗顯示失敗文案，側欄照常、其他頁仍可開", async () => {
    // spec Scenario「內文載入失敗」。
    renderManual({}, loader({ editor: () => Promise.reject(new Error("io")) }));
    await screen.findByText("歡迎。");
    fireEvent.click(row("editor"));
    await screen.findByText("內文載入失敗");
    expect(rows().length).toBe(6);
    fireEvent.click(row("about"));
    await screen.findByText("取材範圍。");
    expect(screen.queryByText("內文載入失敗")).toBeNull();
  });

  it("載入中以 skeleton 佔位", async () => {
    const gate = deferred<string | null>();
    renderManual({}, loader({ index: () => gate.promise }));
    expect(content().querySelector('[aria-busy="true"]')).toBeTruthy();
    gate.resolve("# 手冊\n\n歡迎。");
    await screen.findByText("歡迎。");
    expect(content().querySelector('[aria-busy="true"]')).toBeNull();
  });

  it("refreshGen 變動重載目前頁；交錯回應以最新為準", async () => {
    // spec「手冊頁隨外部變更即時更新」的元件半邊：外部重生一頁後內容更新。
    const first = deferred<string | null>();
    const second = deferred<string | null>();
    let call = 0;
    const loadPage = vi.fn(async () => (call++ === 0 ? first.promise : second.promise));
    const { rerender } = renderManual({ refreshGen: 1 }, loadPage);
    expect(loadPage).toHaveBeenCalledTimes(1);
    rerender(
      <ManualPage index={INDEX} loadPage={loadPage} onOpenSpec={vi.fn()} capabilities={[]} refreshGen={2} />,
    );
    await waitFor(() => expect(loadPage).toHaveBeenCalledTimes(2));
    second.resolve("最新內文");
    await screen.findByText("最新內文");
    first.resolve("過時內文");
    await new Promise((r) => setTimeout(r, 0));
    expect(screen.queryByText("過時內文")).toBeNull();
    expect(screen.getByText("最新內文")).toBeTruthy();
  });
});

describe("ManualPage 空狀態", () => {
  it("索引載入中顯示骨架", () => {
    renderManual({ index: null });
    expect(document.querySelector('[aria-busy="true"]')).toBeTruthy();
    expect(document.querySelector("[data-manual-tree]")).toBeNull();
  });

  it("present false 顯示尚無手冊的空狀態（可用 manual 技能生成）", () => {
    // spec Scenario「無手冊目錄」。
    const { loadPage } = renderManual({
      index: { present: false, reason: null, pages: [], uncoveredNew: [], malformed: [] },
    });
    expect(screen.getByText("尚無手冊")).toBeTruthy();
    expect(screen.getByText(/manual 技能/)).toBeTruthy();
    expect(document.querySelector("[data-manual-empty]")?.getAttribute("data-manual-empty")).toBe("none");
    expect(document.querySelector("[data-manual-tree]")).toBeNull();
    expect(loadPage).not.toHaveBeenCalled();
  });

  it("reason remote 顯示 remote 模式尚不支援手冊的空狀態", () => {
    // spec Scenario「remote 分頁」。
    const { loadPage } = renderManual({
      index: { present: false, reason: "remote", pages: [], uncoveredNew: [], malformed: [] },
    });
    expect(screen.getByText("remote 模式尚不支援手冊")).toBeTruthy();
    expect(document.querySelector("[data-manual-empty]")?.getAttribute("data-manual-empty")).toBe("remote");
    expect(loadPage).not.toHaveBeenCalled();
  });
});
