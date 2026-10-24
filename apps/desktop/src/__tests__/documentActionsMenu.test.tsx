// 詳情頁內容卡標頭的「開啟 ▾」（spec desktop-app「詳情頁的文件動作」；design D4）。
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render as rtlRender, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider, type DocumentTarget } from "@speclink/ui";

import { DocumentActionsMenu } from "../components/DocumentActionsMenu";
import type { DocumentLocation } from "../adapter/fsActions";
import { APP_MESSAGES } from "../i18n/messages";
import type { Platform } from "../platform";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

afterEach(cleanup);

const DOC = { change: "add-auth", artifact: "proposal.md" };
const LOC: DocumentLocation = {
  path: "/work/app/speclink/changes/add-auth/proposal.md",
  relative: "speclink/changes/add-auth/proposal.md",
};

function setup({
  target = { doc: DOC, content: "# 提案" } as DocumentTarget | null,
  local = true,
  platform = "macos" as Platform,
  locate = vi.fn().mockResolvedValue(LOC),
} = {}) {
  const callbacks = {
    locate,
    onOpenDefault: vi.fn(),
    onReveal: vi.fn(),
    onCopyPath: vi.fn(),
    onCopyContent: vi.fn(),
  };
  const view = render(<DocumentActionsMenu target={target} local={local} platform={platform} {...callbacks} />);
  return { ...callbacks, view };
}

const user = () => userEvent.setup();
const openMenu = async (u: ReturnType<typeof userEvent.setup>) =>
  u.click(screen.getByRole("button", { name: /開啟/ }));
const itemNames = () => screen.getAllByRole("menuitem").map((item) => item.textContent);

describe("DocumentActionsMenu", () => {
  it("本機：展開時向宿主查位置，首列為等寬灰字相對路徑，其後三項（macOS 標籤為 Finder）", async () => {
    const u = user();
    const cb = setup();
    await openMenu(u);
    const label = await screen.findByText(LOC.relative);
    expect(cb.locate).toHaveBeenCalledWith(DOC);
    expect(label.className).toContain("font-mono");
    expect(label.className).toContain("text-muted-foreground");
    expect(itemNames()).toEqual(["以預設 App 開啟", "在 Finder 顯示", "複製路徑"]);
  });

  it("首列與動作採宿主回報的實際位置（已封存討論的日期檔名）", async () => {
    const u = user();
    const archived = {
      path: "/work/app/speclink/discussions/archive/2026-01-03-old-topic.md",
      relative: "speclink/discussions/archive/2026-01-03-old-topic.md",
    };
    const cb = setup({
      target: { doc: { discussion: "old-topic" }, content: "# 記錄" },
      locate: vi.fn().mockResolvedValue(archived),
    });
    await openMenu(u);
    await screen.findByText(archived.relative);
    await u.click(screen.getByRole("menuitem", { name: "複製路徑" }));
    expect(cb.locate).toHaveBeenCalledWith({ discussion: "old-topic" });
    expect(cb.onCopyPath).toHaveBeenCalledWith(archived);
  });

  it("位置查詢中或查不到時三項停用、無首列", async () => {
    const u = user();
    setup({ locate: vi.fn().mockResolvedValue(null) });
    await openMenu(u);
    const item = await screen.findByRole("menuitem", { name: "複製路徑" });
    expect(item.getAttribute("aria-disabled")).toBe("true");
    expect(screen.queryByText(LOC.relative)).toBeNull();
  });

  it("Windows 標籤為檔案總管", async () => {
    const u = user();
    setup({ platform: "windows" });
    await openMenu(u);
    expect(itemNames()).toContain("在檔案總管顯示");
  });

  it("各項以實際位置呼叫對應 callback", async () => {
    const u = user();
    const cb = setup();
    await openMenu(u);
    await screen.findByText(LOC.relative);
    await u.click(screen.getByRole("menuitem", { name: "以預設 App 開啟" }));
    await openMenu(u);
    await screen.findByText(LOC.relative);
    await u.click(screen.getByRole("menuitem", { name: "在 Finder 顯示" }));
    await openMenu(u);
    await screen.findByText(LOC.relative);
    await u.click(screen.getByRole("menuitem", { name: "複製路徑" }));
    expect(cb.onOpenDefault).toHaveBeenCalledWith(LOC);
    expect(cb.onReveal).toHaveBeenCalledWith(LOC);
    expect(cb.onCopyPath).toHaveBeenCalledWith(LOC);
    expect(cb.onCopyContent).not.toHaveBeenCalled();
  });

  it("remote 無本機 checkout：只有「複製內容」且無路徑首列、不查位置，選後寫入原文", async () => {
    const u = user();
    const cb = setup({ local: false, target: { doc: { discussion: "alpha" }, content: "# 討論原文" } });
    await openMenu(u);
    expect(itemNames()).toEqual(["複製內容"]);
    expect(cb.locate).not.toHaveBeenCalled();
    await u.click(screen.getByRole("menuitem", { name: "複製內容" }));
    expect(cb.onCopyContent).toHaveBeenCalledWith("# 討論原文");
  });

  it("remote 且原文未載完：「複製內容」停用", async () => {
    const u = user();
    setup({ local: false, target: { doc: DOC, content: null } });
    await openMenu(u);
    const item = screen.getByRole("menuitem", { name: "複製內容" });
    expect(item.getAttribute("aria-disabled")).toBe("true");
  });

  it("remote 且分頁不載原文（工單）時不渲染", () => {
    const { view } = setup({ local: false, target: { doc: { change: "add-auth", artifact: "review.md" } } });
    expect(view.container.innerHTML).toBe("");
  });

  it("本機但目標不帶檔案（已封存討論）：只有「複製內容」、不查位置", async () => {
    const u = user();
    const cb = setup({ target: { doc: null, content: "# 封存討論原文" } });
    await openMenu(u);
    expect(itemNames()).toEqual(["複製內容"]);
    expect(cb.locate).not.toHaveBeenCalled();
    await u.click(screen.getByRole("menuitem", { name: "複製內容" }));
    expect(cb.onCopyContent).toHaveBeenCalledWith("# 封存討論原文");
  });

  it("正式規格與封存變更的文件參照原樣交給宿主查位置", async () => {
    const u = user();
    const spec = { path: "/work/app/speclink/specs/auth/spec.md", relative: "speclink/specs/auth/spec.md" };
    const cb = setup({ target: { doc: { spec: "auth" }, content: "# spec" }, locate: vi.fn().mockResolvedValue(spec) });
    await openMenu(u);
    await screen.findByText(spec.relative);
    expect(cb.locate).toHaveBeenCalledWith({ spec: "auth" });
    cb.view.unmount();
    const archived = setup({ target: { doc: { archived: "2026-07-04-old", artifact: "design.md" }, content: "# d" } });
    await openMenu(u);
    expect(archived.locate).toHaveBeenCalledWith({ archived: "2026-07-04-old", artifact: "design.md" });
  });

  it("分頁沒有對應檔案（target 為 null）時不渲染", () => {
    const { view } = setup({ target: null });
    expect(view.container.innerHTML).toBe("");
  });
});
