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
