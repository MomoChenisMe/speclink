// design D2：文件節點＝tabs 原語的直向 tree 變體——保留 tab／tabpanel 語意，作用中以主色淡底標示。
import { describe, it, expect } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";

import { Tabs, TabsContent, TabsList, TabsTrigger } from "../components/ui/tabs";

function renderTree() {
  return render(
    <Tabs defaultValue="a" orientation="vertical">
      <TabsList variant="tree">
        <TabsTrigger value="a">提案</TabsTrigger>
        <TabsTrigger value="b">設計</TabsTrigger>
      </TabsList>
      <TabsContent value="a">內文 A</TabsContent>
      <TabsContent value="b">內文 B</TabsContent>
    </Tabs>,
  );
}

describe("Tabs tree 變體", () => {
  it("觸發鈕保留 tab 語意，作用中帶主色淡底、非作用中不帶", () => {
    renderTree();
    const a = screen.getByRole("tab", { name: "提案" });
    const b = screen.getByRole("tab", { name: "設計" });
    // jsdom 不算 CSS：底色掛在 data-state 變體上，作用中與否由 data-state 決定；非作用中不得有常駐底色。
    expect(a.getAttribute("data-state")).toBe("active");
    expect(a.className).toContain("data-[state=active]:bg-primary/12");
    expect(b.getAttribute("data-state")).toBe("inactive");
    expect(b.className).not.toMatch(/(^|\s)bg-primary\/12/);
    expect(screen.getByRole("tablist").getAttribute("aria-orientation")).toBe("vertical");
  });

  it("點節點切換作用中與 tabpanel 內容", () => {
    renderTree();
    const b = screen.getByRole("tab", { name: "設計" });
    fireEvent.mouseDown(b, { button: 0, ctrlKey: false });
    expect(b.getAttribute("data-state")).toBe("active");
    expect(screen.getByRole("tabpanel").textContent).toBe("內文 B");
  });

  it("清單直向排列", () => {
    renderTree();
    expect(screen.getByRole("tablist").className).toContain("flex-col");
  });
});

// server-web-list-pages design D1：標籤後的計數籤（已封存頁與 server 使用者頁、憑證頁共用）。
describe("TabsTrigger count", () => {
  it("count 渲染在標籤之後的計數籤；未給時沒有計數籤", () => {
    render(
      <Tabs defaultValue="users">
        <TabsList variant="card">
          <TabsTrigger value="users" count={12}>
            使用者
          </TabsTrigger>
          <TabsTrigger value="pending">邀請中</TabsTrigger>
        </TabsList>
      </Tabs>,
    );
    const users = screen.getByRole("tab", { name: /使用者/ });
    const pill = users.querySelector("[data-tab-count]") as HTMLElement;
    expect(pill.textContent).toBe("12");
    expect(pill.className).toContain("rounded-full");
    expect(users.textContent).toBe("使用者12");
    expect(screen.getByRole("tab", { name: "邀請中" }).querySelector("[data-tab-count]")).toBeNull();
  });

  it("未給 count 時子節點原樣交給 Radix：asChild 搭單一子元素照常渲染", () => {
    render(
      <Tabs defaultValue="a">
        <TabsList variant="card">
          <TabsTrigger value="a" asChild>
            <a href="#a">連結分頁</a>
          </TabsTrigger>
        </TabsList>
      </Tabs>,
    );
    const tab = screen.getByRole("tab", { name: "連結分頁" });
    expect(tab.tagName).toBe("A");
  });

  it("count 為 0 時仍渲染計數籤", () => {
    render(
      <Tabs defaultValue="a">
        <TabsList variant="card">
          <TabsTrigger value="a" count={0}>
            邀請中
          </TabsTrigger>
        </TabsList>
      </Tabs>,
    );
    expect(screen.getByRole("tab").querySelector("[data-tab-count]")!.textContent).toBe("0");
  });
});
