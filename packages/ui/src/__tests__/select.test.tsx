import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../components/ui/select";
import { Input } from "../components/ui/input";

// spec 需求「共用設計系統維持高密度可存取體驗」新增的控件契約：介面上的下拉選單一律
// 使用這個 Select 原語，不再有未套用 theme 的原生 <select>。原生控件展開後的清單由作業
// 系統繪製，focus ring、選中態與圓角都跟不上 theme——被換掉的正是那一層。
// 這裡釘住的是「鍵盤路徑不因換掉原生控件而退化」：開啟、移動、選取三步都要可鍵盤完成。

function Fixture({ onValueChange }: { onValueChange?: (value: string) => void }) {
  return (
    <Select defaultValue="active" onValueChange={onValueChange}>
      <SelectTrigger aria-label="狀態">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="active">有效</SelectItem>
        <SelectItem value="suspended">已停權</SelectItem>
      </SelectContent>
    </Select>
  );
}

describe("Select 原語", () => {
  it("以 role=combobox 呈現且顯示當前選中值", () => {
    render(<Fixture />);
    const trigger = screen.getByRole("combobox", { name: "狀態" });
    expect(trigger.textContent).toContain("有效");
    // 原生 select 不得殘留：整份畫面上沒有 <select> 標籤。
    expect(document.querySelector("select")).toBeNull();
  });

  it("Enter 開啟選單、方向鍵移動、Enter 選取後回報選中值", async () => {
    const onValueChange = vi.fn();
    const user = userEvent.setup();
    render(<Fixture onValueChange={onValueChange} />);

    const trigger = screen.getByRole("combobox", { name: "狀態" });
    trigger.focus();
    await user.keyboard("{Enter}");
    expect(await screen.findByRole("listbox")).toBeTruthy();

    await user.keyboard("{ArrowDown}");
    await user.keyboard("{Enter}");

    expect(onValueChange).toHaveBeenCalledWith("suspended");
    expect(screen.getByRole("combobox", { name: "狀態" }).textContent).toContain("已停權");
  });

  it("Space 亦可開啟選單", async () => {
    const user = userEvent.setup();
    render(<Fixture />);
    screen.getByRole("combobox", { name: "狀態" }).focus();
    await user.keyboard("{ }");
    expect(await screen.findByRole("listbox")).toBeTruthy();
  });
});

describe("Select 與 Input 的視覺一致", () => {
  it("trigger 與 Input 共用高度、內距、圓角、無陰影與聚焦樣式（design D5）", () => {
    render(
      <>
        <Input />
        <Fixture />
      </>,
    );
    const input = screen.getByRole("textbox");
    const trigger = screen.getByRole("combobox", { name: "狀態" });
    // 並排時看得出差異的幾項：高度、水平內距、圓角、聚焦框；兩者都不帶陰影。
    for (const cls of [
      "h-8",
      "px-3",
      "rounded-lg",
      "focus-visible:border-ring",
      "focus-visible:ring-2",
      "focus-visible:ring-ring/30",
    ]) {
      expect(input.className.split(/\s+/), `Input 應有 ${cls}`).toContain(cls);
      expect(trigger.className.split(/\s+/), `SelectTrigger 應有 ${cls}`).toContain(cls);
    }
    expect(input.className).not.toContain("shadow");
    expect(trigger.className).not.toContain("shadow");
  });

  it("選單面板 12px 圓角帶淡陰影、選項 8px 圓角 hover 灰底、選取只呈勾號不填主色", async () => {
    const user = userEvent.setup();
    render(<Fixture />);
    await user.click(screen.getByRole("combobox", { name: "狀態" }));
    // Radix 的 SelectContent 本身就是 role=listbox 的元素。
    const contentCls = (await screen.findByRole("listbox")).className.split(/\s+/);
    for (const cls of ["min-w-56", "rounded-xl", "p-1.5", "shadow-lg", "shadow-black/5"]) {
      expect(contentCls, `SelectContent 應有 ${cls}`).toContain(cls);
    }
    const option = screen.getAllByRole("option")[0];
    const optionCls = option.className.split(/\s+/);
    for (const cls of ["rounded-lg", "px-2.5", "py-1.5", "text-[13px]", "focus:bg-muted"]) {
      expect(optionCls, `SelectItem 應有 ${cls}`).toContain(cls);
    }
    expect(option.className).not.toContain("bg-primary");
    expect(option.className).not.toContain("bg-accent");
  });
});

describe("@speclink/ui 匯出面", () => {
  it("匯出 shadcn 命名的 Select 家族", async () => {
    const ui = await import("../index");
    for (const name of ["Select", "SelectTrigger", "SelectValue", "SelectContent", "SelectItem"]) {
      expect(name in ui, `@speclink/ui 應匯出 ${name}`).toBe(true);
    }
  });

  it("不再匯出 NativeSelect", async () => {
    const ui = await import("../index");
    expect("NativeSelect" in ui).toBe(false);
  });
});
