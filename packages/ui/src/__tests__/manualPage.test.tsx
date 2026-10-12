// spec desktop-manual-page（design「側欄樹、搜尋與上下頁在前端由索引推導」）：
// 「手冊頁的側欄樹與閱讀序」「手冊頁的搜尋列」「內頁渲染與出處跳規格」
// 「可能過期與未入冊的標示」「無手冊與 remote 模式的空狀態」。索引由 props 注入
// （已依閱讀序排好），元件只做分組、過濾、上下頁與內文載入。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, waitFor, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { ManualPage } from "../components/ManualPage";
import { TREE_NODE_CLS } from "../components/ui/tabs";
import { countHeadings } from "../manualDoc";
import { SEMANTIC_SURFACE, SEMANTIC_TONE } from "../tone";
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
      // 預設 spec 目錄：core 解析出的相對路徑與契約預設版面同字。
      locateDocument={async (doc) => ("manual" in doc ? { relative: `openspec/manual/${doc.manual}.md` } : null)}
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
const docHeader = () => content().querySelector("[data-doc-header]") as HTMLElement;
const docProvenance = () => content().querySelector("[data-doc-provenance]") as HTMLElement | null;
const tiles = () => Array.from(content().querySelectorAll<HTMLElement>("[data-doc-tile]")).map((el) => el.textContent);
const staleCallout = () => content().querySelector('[data-callout="stale"]') as HTMLElement | null;
const prevButton = () => screen.queryByRole("button", { name: "上一頁" });
const nextButton = () => screen.queryByRole("button", { name: "下一頁" });
/** 上一頁／下一頁按鈕標的 slug（data hook）；缺席＝無該方向的頁。 */
const prevTarget = () => document.querySelector("[data-manual-prev]")?.getAttribute("data-manual-prev") ?? null;
const nextTarget = () => document.querySelector("[data-manual-next]")?.getAttribute("data-manual-next") ?? null;

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

  it("spec Example「排序與分組」四頁逐列：側欄位置、上一頁、下一頁", async () => {
    // spec Scenario「依 order 排序並依 section 分組」Example 表，逐列以同值斷言。
    const example: ManualIndex = {
      ...INDEX,
      pages: [
        page({ slug: "index", title: "手冊", section: "開始使用", order: 10 }),
        page({ slug: "first-login", title: "第一次登入", section: "開始使用", order: 20 }),
        page({ slug: "editor", title: "編輯器", section: "文件協作", order: 30 }),
        page({ slug: "about", title: "本手冊的來源", section: "附錄", order: 40 }),
      ],
      malformed: [],
    };
    const table: Array<[string, string, number, string | null, string | null]> = [
      ["index", "開始使用", 0, null, "first-login"],
      ["first-login", "開始使用", 1, "index", "editor"],
      ["editor", "文件協作", 0, "first-login", "about"],
      ["about", "附錄", 0, "editor", null],
    ];
    renderManual({ index: example }, loader({}));
    await waitFor(() => expect(row("index").getAttribute("aria-current")).toBe("page"));
    expect(sections()).toEqual(["開始使用", "文件協作", "附錄"]);
    for (const [slug, section, at, prev, next] of table) {
      fireEvent.click(row(slug));
      expect(row(slug).getAttribute("aria-current")).toBe("page");
      const rowsInSection = Array.from(
        tree().querySelectorAll(`[data-manual-section="${section}"] [data-manual-page]`),
      ).map((r) => r.getAttribute("data-manual-page"));
      expect(rowsInSection[at]).toBe(slug);
      expect(prevTarget()).toBe(prev);
      expect(nextTarget()).toBe(next);
      if (prev) expect(prevButton()!.textContent).toContain(example.pages.find((p) => p.slug === prev)!.title);
      else expect(prevButton()).toBeNull();
      if (next) expect(nextButton()!.textContent).toContain(example.pages.find((p) => p.slug === next)!.title);
      else expect(nextButton()).toBeNull();
    }
    // 沿下一頁逐頁走完整個閱讀序。
    fireEvent.click(row("index"));
    for (const expected of ["first-login", "editor", "about"]) {
      fireEvent.click(nextButton()!);
      expect(row(expected).getAttribute("aria-current")).toBe("page");
    }
    expect(nextButton()).toBeNull();
  });

  it("外部新增一頁（order 落於既有兩頁之間）後側欄於對應位置出現，其餘順序不變", async () => {
    // spec Scenario「外部新增頁後側欄出現」：索引換新（已由 core 依 order 排好）→ 側欄跟著變。
    const { rerender, loadPage } = renderManual();
    await screen.findByText("歡迎。");
    const inserted: ManualIndex = {
      ...INDEX,
      pages: [
        ...INDEX.pages.slice(0, 2),
        page({ slug: "shortcuts", title: "快捷鍵", section: "開始使用", order: 25 }),
        ...INDEX.pages.slice(2),
      ],
    };
    rerender(
      <ManualPage index={inserted} loadPage={loadPage} onOpenSpec={vi.fn()} capabilities={[]} />,
    );
    expect(rows()).toEqual(["index", "first-login", "shortcuts", "editor", "about", "orphan", "broken"]);
    expect(row("index").getAttribute("aria-current")).toBe("page");
    expect(screen.getByText("歡迎。")).toBeTruthy();
  });

  it("索引清空（切換 workspace）時丟棄舊內文與選頁，新索引到達前只顯示骨架", async () => {
    // review R1：手冊頁開著時切換分頁，index 先變 null；新 workspace 的首頁 slug 同名
    // （契約規定都叫 index）時，不得把舊 workspace 的內文當成新頁顯示。
    const gate = deferred<string | null>();
    const calls: string[] = [];
    const loadPage = vi.fn(async (slug: string) => {
      calls.push(slug);
      return calls.length <= 2 ? BODIES[slug] : gate.promise;
    });
    const { rerender } = renderManual({}, loadPage);
    await screen.findByText("歡迎。");
    fireEvent.click(row("editor"));
    await screen.findByText("提示內容");
    rerender(<ManualPage index={null} loadPage={loadPage} onOpenSpec={vi.fn()} capabilities={[]} />);
    expect(screen.queryByText("提示內容")).toBeNull();
    const other: ManualIndex = {
      ...INDEX,
      pages: [
        page({ slug: "index", title: "另一份手冊", section: "開始使用", order: 10 }),
        page({ slug: "editor", title: "另一個編輯器", section: "文件協作", order: 20 }),
      ],
      malformed: [],
    };
    rerender(<ManualPage index={other} loadPage={loadPage} onOpenSpec={vi.fn()} capabilities={[]} />);
    // 新 workspace 從首頁開始（不沿用舊選頁），且舊內文不得出現。
    expect(row("index").getAttribute("aria-current")).toBe("page");
    expect(calls[calls.length - 1]).toBe("index");
    expect(screen.queryByText("提示內容")).toBeNull();
    expect(screen.queryByText("歡迎。")).toBeNull();
    expect(content().querySelector('[aria-busy="true"]')).toBeTruthy();
    gate.resolve("# 另一份手冊\n\n新 workspace 內文。");
    await screen.findByText("新 workspace 內文。");
  });

  it("索引清空時，在途的舊載入回應不得寫回（切分頁瞬間的競態）", async () => {
    // review R1 第 2 輪殘留：切分頁當下若 loadPage 在途，回應落地時索引已是 null 或
    // 已換成新 workspace（首頁同名 index），舊內文不得寫回。
    const old = deferred<string | null>();
    const fresh = deferred<string | null>();
    const loadPage = vi.fn().mockReturnValueOnce(old.promise).mockReturnValueOnce(fresh.promise);
    const { rerender } = renderManual({}, loadPage);
    expect(loadPage).toHaveBeenCalledTimes(1);
    rerender(<ManualPage index={null} loadPage={loadPage} onOpenSpec={vi.fn()} capabilities={[]} />);
    // 舊回應落在索引為 null 的窗口內。
    old.resolve("# 舊 workspace\n\n舊 workspace 內文。");
    await new Promise((r) => setTimeout(r, 0));
    const other: ManualIndex = {
      ...INDEX,
      pages: [page({ slug: "index", title: "另一份手冊", section: "開始使用", order: 10 })],
      malformed: [],
    };
    rerender(<ManualPage index={other} loadPage={loadPage} onOpenSpec={vi.fn()} capabilities={[]} />);
    expect(loadPage).toHaveBeenCalledTimes(2);
    expect(screen.queryByText("舊 workspace 內文。")).toBeNull();
    expect(content().querySelector('[aria-busy="true"]')).toBeTruthy();
    fresh.resolve("# 另一份手冊\n\n新 workspace 內文。");
    await screen.findByText("新 workspace 內文。");
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
  it("stale 頁列尾為警示色面的「過期」籤，其他頁無", async () => {
    // spec Scenario「來源更新後標示可能過期」。
    renderManual();
    await screen.findByText("歡迎。");
    const marker = row("first-login").querySelector("[data-manual-stale]") as HTMLElement;
    expect(marker.textContent).toBe("過期");
    expect(marker.className).toContain("rounded-full");
    expect(marker.className).toContain(SEMANTIC_SURFACE.warning);
    expect(marker.className).toContain(SEMANTIC_TONE.warning);
    expect(row("index").querySelector("[data-manual-stale]")).toBeNull();
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

  it("帶 Requirement 錨定的 sources：出處列只列井號前的 capability、去重且可點", async () => {
    // manual-pages「frontmatter 六欄」Scenario「帶 Requirement 錨定的 sources」：
    // `["desktop-app#看板與任務", "desktop-app#系統匣選單", policy-config]` → 出處只列 desktop-app 與 policy-config 各一次。
    const index: ManualIndex = {
      ...INDEX,
      pages: [
        ...INDEX.pages,
        page({
          slug: "board",
          title: "看板與任務",
          section: "桌面 app",
          order: 35,
          // 第三項刻意帶空白：Rust 讀取端接受並回原字串，前端切完 `#` 也要 trim 才不會多一顆 chip。
          sources: ["desktop-app#看板與任務", "desktop-app#系統匣選單", " desktop-app # 專案分頁 ", "policy-config"],
          generated: "2026-09-05",
        }),
      ],
    };
    const { onOpenSpec } = renderManual(
      { index, capabilities: ["github-oauth", "editor", "desktop-app"] },
      loader({ board: async () => "# 看板與任務\n\n看板。" }),
    );
    await screen.findByText("歡迎。");
    fireEvent.click(row("board"));
    await screen.findByText("看板。");
    const sources = document.querySelector("[data-manual-sources]") as HTMLElement;
    expect(sources.textContent).not.toContain("#");
    expect(within(sources).getAllByText("desktop-app")).toHaveLength(1);
    fireEvent.click(within(sources).getByRole("button", { name: "desktop-app" }));
    expect(onOpenSpec).toHaveBeenCalledWith("desktop-app");
    expect(within(sources).queryByRole("button", { name: "policy-config" })).toBeNull();
    expect(sources.textContent).toContain("policy-config");
    // 閱讀卡出處行同樣只有一顆可點的 desktop-app、policy-config 為純文字；數字磚「出處規格」為 2。
    const provenance = docProvenance()!;
    expect(within(provenance).getAllByText("desktop-app")).toHaveLength(1);
    expect(within(provenance).getByRole("button", { name: "desktop-app" })).toBeTruthy();
    expect(within(provenance).queryByRole("button", { name: "policy-config" })).toBeNull();
    expect(provenance.textContent).toContain("policy-config");
    expect(tiles()[3]).toBe("2出處規格");
  });

  it("出處列以索引 sources 為準：內文沒有出處行時照樣列出", async () => {
    const { onOpenSpec } = renderManual({}, loader({ editor: async () => "# 認識畫面\n\n沒有出處行。" }));
    await screen.findByText("歡迎。");
    fireEvent.click(row("editor"));
    await screen.findByText("沒有出處行。");
    const sources = document.querySelector("[data-manual-sources]") as HTMLElement;
    fireEvent.click(within(sources).getByRole("button", { name: "editor" }));
    expect(onOpenSpec).toHaveBeenCalledWith("editor");
  });

  it("GitHub Alert 在手冊內文呈現為提示框、內文在 768px 置中閱讀卡內渲染", async () => {
    renderManual();
    await screen.findByText("歡迎。");
    fireEvent.click(row("editor"));
    await screen.findByText("提示內容");
    expect(content().querySelector(".markdown-alert-note")).toBeTruthy();
    const body = content().querySelector("[data-manual-body]") as HTMLElement;
    expect(body.querySelector(".max-w-\\[768px\\]")).toBeTruthy();
    // 內文包在白底閱讀卡內（design D5）。
    const readingCard = body.querySelector("[data-manual-card]") as HTMLElement;
    expect(readingCard.className).toContain("bg-card");
    expect(readingCard.contains(body.querySelector(".markdown")!)).toBe(true);
  });

  it("標題在閱讀卡標題區、捲動區外無頁首、底列固定、內文不重複 H1、無 H1 時退回索引 title", async () => {
    renderManual();
    await screen.findByText("歡迎。");
    const body = content().querySelector("[data-manual-body]") as HTMLElement;
    const card = body.querySelector("[data-manual-card]") as HTMLElement;
    const footer = content().querySelector("[data-manual-footer]") as HTMLElement;
    expect(card.contains(docHeader())).toBe(true);
    expect(within(docHeader()).getByRole("heading", { level: 2 }).textContent).toBe("手冊");
    expect(content().querySelector("h1")).toBeNull();
    // 捲動區外只剩固定底列：沒有頁首。
    expect(Array.from(body.parentElement!.children)).toEqual([body, footer]);
    expect(body.contains(footer)).toBe(false);
    expect(footer.contains(nextButton()!)).toBe(true);
    fireEvent.click(row("first-login"));
    await screen.findByText("用 GitHub 登入。");
    expect(footer.contains(document.querySelector("[data-manual-sources]")!)).toBe(true);
    fireEvent.click(row("orphan"));
    await screen.findByText("沒有欄位的頁");
    expect(within(docHeader()).getByRole("heading", { level: 2 }).textContent).toBe("orphan");
  });

  it("右側錨點列列出內文 h2／h3、首項為目前段；點擊捲至該標題；無標題時錨點列缺席", async () => {
    // spyOn＋finally 還原：全域 prototype 覆寫不得外溢到同檔後續測試。
    const scrollSpy = vi.spyOn(Element.prototype, "scrollIntoView").mockImplementation(() => {});
    try {
      renderManual(
        {},
        loader({
          editor: async () => "# 認識畫面\n\n## 看板\n\n段一。\n\n### 卡片\n\n段二。\n\n## 抽屜\n\n段三。",
        }),
      );
      await screen.findByText("歡迎。");
      expect(document.querySelector("[data-manual-toc]")).toBeNull();
      fireEvent.click(row("editor"));
      await screen.findByText("段一。");
      const toc = await waitFor(() => {
        const el = document.querySelector("[data-manual-toc]") as HTMLElement;
        expect(el).toBeTruthy();
        return el;
      });
      expect(toc.className).toContain("w-[200px]");
      const anchors = Array.from(toc.querySelectorAll("[data-manual-anchor]"));
      expect(anchors.map((a) => a.textContent)).toEqual(["看板", "卡片", "抽屜"]);
      expect(anchors[0].getAttribute("aria-current")).toBe("location");
      expect(anchors[2].getAttribute("aria-current")).toBeNull();
      fireEvent.click(anchors[2]);
      expect(scrollSpy).toHaveBeenCalledTimes(1);
      expect((scrollSpy.mock.contexts[0] as HTMLElement).textContent).toBe("抽屜");
      // 切到無 h2／h3 的頁：錨點列消失。
      fireEvent.click(row("about"));
      await screen.findByText("取材範圍。");
      await waitFor(() => expect(document.querySelector("[data-manual-toc]")).toBeNull());
    } finally {
      scrollSpy.mockRestore();
    }
  });

  it("內文的相對檔名連結在手冊內切頁；連到不存在的頁只擋下導航", async () => {
    // manual-pages 契約「內文慣例」跨頁連結為相對檔名；WebView 直接導航會整頁離開 app。
    renderManual({}, loader({ index: async () => "# 手冊\n\n看[認識畫面](editor.md)與[失聯](nope.md)。" }));
    const good = await screen.findByRole("link", { name: "認識畫面" });
    // fireEvent 回傳 false＝preventDefault 已呼叫（導航被擋下）。
    expect(fireEvent.click(good)).toBe(false);
    await screen.findByText("提示內容");
    expect(row("editor").getAttribute("aria-current")).toBe("page");
    fireEvent.click(row("index"));
    const bad = await screen.findByRole("link", { name: "失聯" });
    expect(fireEvent.click(bad)).toBe(false);
    expect(row("index").getAttribute("aria-current")).toBe("page");
    expect(screen.getByRole("link", { name: "認識畫面" })).toBeTruthy();
  });

  it("內文載入失敗顯示失敗文案，側欄照常、其他頁仍可開", async () => {
    // spec Scenario「內文載入失敗」。
    renderManual({}, loader({ editor: () => Promise.reject(new Error("io")) }));
    await screen.findByText("歡迎。");
    fireEvent.click(row("editor"));
    await screen.findByText("內文載入失敗");
    expect(content().querySelector("[data-manual-load-failed]")).toBeTruthy();
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

describe("ManualPage React key", () => {
  it("明寫 section「其他」與缺 section 的組不相鄰、索引 sources 重複列名時，不產生 React key 警告", async () => {
    // review R4：分區以位置為 key、出處名去重（出處列以索引 sources 為單一真相）。
    const errors = vi.spyOn(console, "error").mockImplementation(() => {});
    const index: ManualIndex = {
      ...INDEX,
      pages: [
        page({ slug: "misc", title: "雜項", section: "其他", order: 10, sources: ["github-oauth", "github-oauth"] }),
        page({ slug: "index", title: "手冊", section: "開始使用", order: 20 }),
        page({ slug: "orphan" }),
      ],
      malformed: [],
    };
    renderManual(
      { index },
      loader({ misc: async () => "# 雜項\n\n內文。\n\n**出處**：`github-oauth`、`github-oauth`" }),
    );
    await screen.findByText("內文。");
    expect(sections()).toEqual(["其他", "開始使用", "其他"]);
    const sources = document.querySelector("[data-manual-sources]") as HTMLElement;
    expect(within(sources).getAllByRole("button", { name: "github-oauth" })).toHaveLength(1);
    expect(errors.mock.calls.filter((c) => String(c[0]).includes("key"))).toEqual([]);
    errors.mockRestore();
  });
});

describe("ManualPage 空狀態", () => {
  it("索引載入中顯示骨架", () => {
    renderManual({ index: null });
    expect(document.querySelector('[aria-busy="true"]')).toBeTruthy();
    expect(document.querySelector("[data-manual-tree]")).toBeNull();
  });

  it("present false 顯示尚無手冊的空狀態（可用手冊技能生成）", () => {
    // spec Scenario「無手冊目錄」。
    const { loadPage } = renderManual({
      index: { present: false, reason: null, pages: [], uncoveredNew: [], malformed: [] },
    });
    expect(screen.getByText("尚無手冊")).toBeTruthy();
    expect(screen.getByText(/手冊技能/)).toBeTruthy();
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

// spec desktop-manual-page「內頁渲染與出處跳規格」（desktop-list-pages-reskin design D5）：
// 三欄寬度（240／768 置中／200）、頁首 24px 標題與產生時間行、白底上緣細線的固定底列
// 對齊 768px、出處籤與帶目標頁標題的上一頁／下一頁框線鈕。
describe("ManualPage 三欄版面與固定底列", () => {
  it("目錄樹容器為 240px 淡灰底；頁列與文件樹節點同款（30px、圓角、hover 淡底、作用中主色淡底與主色字）", async () => {
    renderManual();
    await screen.findByText("歡迎。");
    const aside = tree().closest("aside") as HTMLElement;
    expect(aside.className).toContain("w-60");
    expect(aside.className).toContain("bg-sidebar");
    // 同款＝直接共用文件樹節點的 class；作用中與否只看 data-state。
    for (const slug of ["index", "first-login"]) expect(row(slug).className).toBe(TREE_NODE_CLS);
    expect(TREE_NODE_CLS).toContain("h-[30px]");
    expect(TREE_NODE_CLS).toContain("hover:bg-foreground/5");
    expect(TREE_NODE_CLS).toContain("data-[state=active]:bg-primary/12");
    expect(row("index").getAttribute("data-state")).toBe("active");
    expect(row("index").getAttribute("aria-current")).toBe("page");
    expect(row("first-login").getAttribute("data-state")).toBe("inactive");
    // 樹的搜尋框為全圓。
    expect(screen.getByPlaceholderText("搜尋手冊…").className).toContain("rounded-full");
  });

  it("閱讀卡標題為 24px 一般字重；出處行為灰字路徑與「產生於 {generated}」；generated 缺席時該段缺席", async () => {
    renderManual();
    await screen.findByText("歡迎。");
    const title = within(docHeader()).getByRole("heading", { level: 2 });
    expect(title.className).toContain("text-2xl");
    expect(title.className).toContain("font-normal");
    expect(docProvenance()!.className).toContain("text-muted-foreground");
    expect(docProvenance()!.textContent).toBe("openspec/manual/index.md·產生於 2026-09-01");
    expect(docProvenance()!.querySelector("[data-manual-generated]")).toBeTruthy();
    fireEvent.click(row("about"));
    await screen.findByText("取材範圍。");
    expect(docProvenance()!.textContent).toBe("openspec/manual/about.md");
    expect(content().querySelector("[data-manual-generated]")).toBeNull();
  });

  it("底列固定：白底、上緣細線、內容對齊 768px；左出處籤為框線籤、右為帶小字標籤與目標頁標題的框線鈕", async () => {
    renderManual();
    await screen.findByText("歡迎。");
    fireEvent.click(row("first-login"));
    await screen.findByText("用 GitHub 登入。");
    const footer = content().querySelector("[data-manual-footer]") as HTMLElement;
    expect(footer.className).toContain("bg-card");
    expect(footer.className).toContain("border-t");
    expect(footer.className).toContain("shrink-0");
    expect(footer.querySelector(".max-w-\\[768px\\]")).toBeTruthy();
    // 出處籤：存在者為可點 button 的框線籤（等寬字）、不存在者純文字。
    const sources = footer.querySelector("[data-manual-sources]") as HTMLElement;
    const chip = within(sources).getByRole("button", { name: "github-oauth" });
    expect(chip.className).toContain("font-mono");
    expect(chip.className).toContain("border");
    expect(within(sources).queryByRole("button", { name: "user-pending-blocked-pages" })).toBeNull();
    // 上一頁／下一頁：框線鈕、小字標籤＋目標頁標題；無障礙名稱維持「上一頁」「下一頁」。
    const prev = prevButton()!;
    const next = nextButton()!;
    expect(footer.contains(prev)).toBe(true);
    expect(prev.className).toContain("border");
    expect(prev.textContent).toContain("上一頁");
    expect(prev.textContent).toContain("手冊");
    expect(next.textContent).toContain("下一頁");
    expect(next.textContent).toContain("認識畫面");
    expect(prevTarget()).toBe("index");
    expect(nextTarget()).toBe("editor");
  });
});

describe("countHeadings", () => {
  it("只數圍欄外的 ATX h2／h3", () => {
    const md = [
      "# 頁標題",
      "## 原生選單",
      "### 變更列",
      "#### 更深一層",
      "  ### 縮排三格內仍是標題",
      "##沒有空白不是標題",
      "```md",
      "## 圍欄內",
      "### 圍欄內",
      "```",
      "Setext 標題",
      "---",
      "## 分區太長時 ##",
      "###",
    ].join("\n");
    expect(countHeadings(md)).toEqual({ h2: 2, h3: 3 });
  });

  it("空內文為 0", () => {
    expect(countHeadings("")).toEqual({ h2: 0, h3: 0 });
  });
});

/** 〈系統匣選單〉（Scenario「閱讀卡標題區、出處行與數字磚」）：3 個 h2、11 個 h3、7 個關鍵字、1 個出處。 */
const TRAY_PAGE = page({
  slug: "tray",
  title: "系統匣選單",
  section: "桌面 app",
  order: 360,
  keywords: ["系統匣", "選單列", "面板", "進度", "複製名稱", "複製 slug", "保留在途"],
  sources: ["tray-status-menu"],
  generated: "2026-09-17T16:05:41+08:00",
});
const TRAY_BODY = [
  "# 系統匣選單",
  "開頭。",
  "## 原生選單（Windows、Linux）",
  ...["變更列", "討論列", "專案切換", "動作區"].map((h) => `### ${h}`),
  "## 面板（macOS）",
  ...["專案分頁條", "內容區", "品質關卡章", "remote 專案", "載入中", "滑鼠操作", "建立失敗"].map((h) => `### ${h}`),
  "```md",
  "## 圍欄內不算",
  "```",
  "## 分區太長時",
  "**出處**：`tray-status-menu`",
].join("\n\n");

/** 規格 x：最近的 @trace 為 2026-09-05（change-x），另一條較舊。 */
const X_SPEC = [
  "# x Specification",
  "## Purpose",
  "x。",
  "## Requirements",
  "### Requirement: 甲",
  "甲 SHALL 做事。",
  "<!-- @trace\nsource: change-old\nupdated: 2026-08-01\n-->",
  "### Requirement: 乙",
  "乙 SHALL 做事。",
  "<!-- @trace\nsource: change-x\nupdated: 2026-09-05T10:00:00+08:00\n-->",
].join("\n\n");

const STALE_INDEX: ManualIndex = {
  present: true,
  reason: null,
  pages: [
    page({ slug: "stale-page", title: "過期頁", section: "開始使用", order: 10, sources: ["x"], generated: "2026-09-01", stale: true }),
    page({ slug: "fresh", title: "新鮮頁", section: "開始使用", order: 20, sources: ["y"], generated: "2026-09-01" }),
  ],
  uncoveredNew: [],
  malformed: [],
};
const STALE_BODIES = { "stale-page": async () => "# 過期頁\n\n過期內文。", fresh: async () => "# 新鮮頁\n\n新鮮內文。" };

describe("ManualPage 閱讀卡標題區、出處行、數字磚與提示框", () => {
  it("〈系統匣選單〉資料：分區籤、出處行三段、四格磚 3／11／7／1", async () => {
    const { onOpenSpec } = renderManual(
      { index: { ...INDEX, pages: [TRAY_PAGE] }, capabilities: ["tray-status-menu"] },
      loader({ tray: async () => TRAY_BODY }),
    );
    await screen.findByText("開頭。");
    expect(within(docHeader()).getByRole("heading", { level: 2 }).textContent).toBe("系統匣選單");
    expect(docHeader().querySelector("[data-doc-badges]")!.textContent).toBe("桌面 app");
    expect(content().querySelector("[data-manual-stale-pill]")).toBeNull();
    expect(docProvenance()!.textContent).toBe("openspec/manual/tray.md·產生於 2026-09-17T16:05:41+08:00·出處tray-status-menu");
    fireEvent.click(within(docProvenance()!).getByRole("button", { name: "tray-status-menu" }));
    expect(onOpenSpec).toHaveBeenCalledWith("tray-status-menu");
    expect(tiles()).toEqual(["3章節", "11小節", "7關鍵字", "1出處規格"]);
    expect(staleCallout()).toBeNull();
  });

  it("出處行的路徑取 core 解析的實際位置：改過 spec 目錄名時照實顯示；沒有可定位的檔案時不列路徑", async () => {
    const locateDocument = vi.fn(async () => ({ relative: "docs/specs/manual/tray.md" }));
    const view = renderManual(
      { index: { ...INDEX, pages: [TRAY_PAGE] }, capabilities: ["tray-status-menu"], locateDocument },
      loader({ tray: async () => TRAY_BODY }),
    );
    await screen.findByText("開頭。");
    await waitFor(() =>
      expect(docProvenance()!.textContent).toBe("docs/specs/manual/tray.md·產生於 2026-09-17T16:05:41+08:00·出處tray-status-menu"),
    );
    expect(locateDocument).toHaveBeenCalledWith({ manual: "tray" });
    view.unmount();
    renderManual(
      { index: { ...INDEX, pages: [TRAY_PAGE] }, capabilities: ["tray-status-menu"], locateDocument: undefined },
      loader({ tray: async () => TRAY_BODY }),
    );
    await screen.findByText("開頭。");
    expect(docProvenance()!.textContent).toBe("產生於 2026-09-17T16:05:41+08:00·出處tray-status-menu");
  });

  it("切換專案後同名頁的出處行不沿用前一個專案的路徑：新位置查到前該段缺席", async () => {
    const gate = deferred<{ relative: string } | null>();
    let call = 0;
    const locateDocument = vi.fn(() => (call++ === 0 ? Promise.resolve({ relative: "a/manual/index.md" }) : gate.promise));
    const props = { onOpenSpec: vi.fn(), capabilities: [], loadPage: loader(), locateDocument };
    const { rerender } = render(<ManualPage index={INDEX} {...props} />);
    await screen.findByText("歡迎。");
    await waitFor(() => expect(docProvenance()!.textContent).toContain("a/manual/index.md"));
    rerender(<ManualPage index={null} {...props} />);
    rerender(<ManualPage index={INDEX} {...props} />);
    await screen.findByText("歡迎。");
    await waitFor(() => expect(locateDocument).toHaveBeenCalledTimes(2));
    expect(docProvenance()!.textContent).not.toContain("a/manual");
    gate.resolve({ relative: "b/manual/index.md" });
    await waitFor(() => expect(docProvenance()!.textContent).toContain("b/manual/index.md"));
  });

  it("缺 section 的頁分區籤為「其他」", async () => {
    renderManual();
    await screen.findByText("歡迎。");
    fireEvent.click(row("orphan"));
    await screen.findByText("沒有欄位的頁");
    expect(docHeader().querySelector("[data-doc-badges]")!.textContent).toBe("其他");
  });

  it("stale 頁：籤槽「可能過期」籤、提示框列出出處的最近 @trace 與「本頁產生於」；非 stale 頁無籤無提示框", async () => {
    const loadSpec = vi.fn(async (cap: string) => (cap === "x" ? X_SPEC : null));
    renderManual({ index: STALE_INDEX, capabilities: ["x", "y"], loadSpec }, loader(STALE_BODIES));
    await screen.findByText("過期內文。");
    expect(docHeader().querySelector("[data-manual-stale-pill]")!.textContent).toBe("可能過期");
    await waitFor(() => expect(staleCallout()!.textContent).toContain("出處 x 於 2026-09-05 由變更 change-x 更新"));
    expect(loadSpec).toHaveBeenCalledWith("x");
    expect(staleCallout()!.textContent).toContain("本頁產生於 2026-09-01");
    expect(staleCallout()!.textContent).toContain("/speclink-manual");
    // 提示框在數字磚與內文之間。
    const slot = content().querySelector("[data-doc-callout]") as HTMLElement;
    expect(slot.contains(staleCallout())).toBe(true);
    const order = [content().querySelector("[data-doc-tiles]")!, slot, content().querySelector("[data-doc-body]")!];
    expect(order[0].compareDocumentPosition(order[1]) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(order[1].compareDocumentPosition(order[2]) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    fireEvent.click(row("fresh"));
    await screen.findByText("新鮮內文。");
    expect(content().querySelector("[data-manual-stale-pill]")).toBeNull();
    expect(staleCallout()).toBeNull();
    expect(loadSpec).not.toHaveBeenCalledWith("y");
  });

  it("loadSpec reject 或規格無 @trace 時該行為「在本頁產生後更新過」，籤照常", async () => {
    const noTrace = "# x Specification\n\n## Requirements\n\n### Requirement: 甲\n\n甲 SHALL 做事。";
    const cases: [string, () => Promise<string | null>][] = [
      ["reject", () => Promise.reject(new Error("io"))],
      ["缺件", async () => null],
      ["無 @trace", async () => noTrace],
    ];
    for (const [, read] of cases) {
      const view = renderManual({ index: STALE_INDEX, capabilities: ["x"], loadSpec: vi.fn(read) }, loader(STALE_BODIES));
      await screen.findByText("過期內文。");
      await waitFor(() => expect(staleCallout()!.textContent).toContain("出處 x 在本頁產生後更新過"));
      expect(staleCallout()!.textContent).not.toContain("由變更");
      expect(docHeader().querySelector("[data-manual-stale-pill]")).toBeTruthy();
      view.unmount();
    }
  });

  it("切換專案後同名的過期頁不沿用前一個專案的出處日期：新規格讀完前提示框不出現", async () => {
    const gate = deferred<string | null>();
    let call = 0;
    const loadSpec = vi.fn(() => (call++ === 0 ? Promise.resolve(X_SPEC) : gate.promise));
    const props = { onOpenSpec: vi.fn(), capabilities: ["x"], loadSpec, loadPage: loader(STALE_BODIES) };
    const { rerender } = render(<ManualPage index={STALE_INDEX} {...props} />);
    await waitFor(() => expect(staleCallout()!.textContent).toContain("由變更 change-x 更新"));
    rerender(<ManualPage index={null} {...props} />);
    rerender(<ManualPage index={STALE_INDEX} {...props} />);
    await screen.findByText("過期內文。");
    await waitFor(() => expect(loadSpec).toHaveBeenCalledTimes(2));
    expect(staleCallout()).toBeNull();
    gate.resolve(X_SPEC.replace("source: change-x", "source: change-b"));
    await waitFor(() => expect(staleCallout()!.textContent).toContain("由變更 change-b 更新"));
    expect(staleCallout()!.textContent).not.toContain("change-x");
  });

  it("換頁作廢在途的規格讀取：晚到的舊回應不覆蓋最新一次", async () => {
    const reads = [deferred<string | null>(), deferred<string | null>()];
    let call = 0;
    const loadSpec = vi.fn(() => reads[call++].promise);
    renderManual({ index: STALE_INDEX, capabilities: ["x", "y"], loadSpec }, loader(STALE_BODIES));
    await screen.findByText("過期內文。");
    // 過期頁 → 新鮮頁 → 過期頁：第一次讀取在途時就換頁。
    fireEvent.click(row("fresh"));
    await screen.findByText("新鮮內文。");
    fireEvent.click(row("stale-page"));
    await screen.findByText("過期內文。");
    await waitFor(() => expect(loadSpec).toHaveBeenCalledTimes(2));
    reads[1].resolve(X_SPEC);
    await waitFor(() => expect(staleCallout()!.textContent).toContain("由變更 change-x 更新"));
    reads[0].resolve(X_SPEC.replace("source: change-x", "source: change-late").replace("2026-09-05T10:00:00+08:00", "2026-12-01"));
    await new Promise((r) => setTimeout(r, 0));
    expect(staleCallout()!.textContent).toContain("由變更 change-x 更新");
    expect(staleCallout()!.textContent).not.toContain("change-late");
  });

  it("renderDocumentActions 收到 { manual: slug } 與已載入原文並渲染在標題列；未傳時無動作", async () => {
    const renderDocumentActions = vi.fn(() => <button type="button">開啟 ▾</button>);
    const view = renderManual({ renderDocumentActions });
    await screen.findByText("歡迎。");
    expect(renderDocumentActions).toHaveBeenLastCalledWith({ doc: { manual: "index" }, content: BODIES.index });
    expect(within(docHeader()).getByRole("button", { name: "開啟 ▾" })).toBeTruthy();
    view.unmount();
    renderManual();
    await screen.findByText("歡迎。");
    expect(within(docHeader()).queryByRole("button", { name: "開啟 ▾" })).toBeNull();
  });

  it("載入失敗時無磚無提示框，籤照常", async () => {
    const loadSpec = vi.fn(async () => X_SPEC);
    renderManual(
      { index: STALE_INDEX, capabilities: ["x"], loadSpec },
      loader({ "stale-page": () => Promise.reject(new Error("io")) }),
    );
    await screen.findByText("內文載入失敗");
    expect(content().querySelector("[data-doc-tiles]")).toBeNull();
    expect(staleCallout()).toBeNull();
    expect(docHeader().querySelector("[data-manual-stale-pill]")).toBeTruthy();
  });

  it("DocBody 容器 class 含 15px 與 h2／h3 覆蓋", async () => {
    renderManual();
    await screen.findByText("歡迎。");
    const body = content().querySelector("[data-manual-card] [data-doc-body]") as HTMLElement;
    expect(body.className).toContain("text-[15px]");
    expect(body.className).toContain("leading-[1.75]");
    const wrapper = body.parentElement as HTMLElement;
    expect(wrapper.className).toContain("[&_.markdown_h2]:text-base");
    expect(wrapper.className).toContain("[&_.markdown_h2]:font-semibold");
    expect(wrapper.className).toContain("[&_.markdown_h3]:text-sm");
    expect(wrapper.className).toContain("[&_.markdown_h3]:font-semibold");
    expect(body.textContent).toContain("歡迎。");
  });
});
