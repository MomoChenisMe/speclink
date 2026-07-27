import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { Sheet, SheetContent, SheetHeader, SheetTitle } from "../components/ui/sheet";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../components/ui/select";

// 抽屜內的下拉是管理面的常見組合（邀請抽屜挑專案、細節抽屜改角色）。Sheet 是 modal
// Dialog，它的 FocusScope 會把焦點鎖在 content 內；Select 的選單卻 portal 到 body，
// 落在 content 外。兩者對同一次 focus 事件各自反應就會互推到爆堆疊——這支測試釘住
// 「抽屜內的下拉可以正常開啟與選取」。
function Fixture() {
  return (
    <Sheet open>
      <SheetContent>
        <SheetHeader>
          <SheetTitle>抽屜</SheetTitle>
        </SheetHeader>
        <Select defaultValue="a">
          <SelectTrigger aria-label="選項">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="a">甲</SelectItem>
            <SelectItem value="b">乙</SelectItem>
          </SelectContent>
        </Select>
      </SheetContent>
    </Sheet>
  );
}

describe("抽屜內的 Select", () => {
  it("可以開啟選單並選取，不與 Sheet 的 focus trap 互推", async () => {
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    render(<Fixture />);
    await user.click(screen.getByRole("combobox", { name: "選項" }));
    await user.click(await screen.findByRole("option", { name: "乙" }));
    expect(screen.getByRole("combobox", { name: "選項" }).textContent).toContain("乙");
  });
});
