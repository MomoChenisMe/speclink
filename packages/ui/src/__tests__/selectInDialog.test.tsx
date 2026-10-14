import { describe, it, expect } from "vitest";
import { createRef, useState, type Ref } from "react";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogTitle,
} from "../components/ui/alert-dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../components/ui/select";

// 對話框內的下拉（server-web-detail-pages design D4：邀請對話框挑專案）。AlertDialog 同樣
// 是 modal，選單若 portal 到 body 會落在它的 focus trap 外面——這支測試釘住「選單掛進
// 對話框自己、可開啟可選取、焦點不離開對話框」（同 selectInSheet.test.tsx 的抽屜版）。
function Fixture() {
  const [open, setOpen] = useState(true);
  return (
    <AlertDialog open={open} onOpenChange={setOpen}>
      <AlertDialogContent>
        <AlertDialogTitle>對話框</AlertDialogTitle>
        <AlertDialogDescription>挑一個選項。</AlertDialogDescription>
        <Select defaultValue="a">
          <SelectTrigger aria-label="選項">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="a">甲</SelectItem>
            <SelectItem value="b">乙</SelectItem>
          </SelectContent>
        </Select>
      </AlertDialogContent>
    </AlertDialog>
  );
}

describe("對話框內的 Select", () => {
  it("選單掛在對話框內，可以開啟並選取，焦點留在對話框", async () => {
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    render(<Fixture />);
    const dialog = screen.getByRole("alertdialog");
    await user.click(screen.getByRole("combobox", { name: "選項" }));
    const listbox = await screen.findByRole("listbox");
    expect(dialog.contains(listbox)).toBe(true);
    expect(dialog.contains(document.activeElement)).toBe(true);
    await user.click(screen.getByRole("option", { name: "乙" }));
    expect(screen.getByRole("combobox", { name: "選項" }).textContent).toContain("乙");
    expect(dialog.contains(document.activeElement)).toBe(true);
  });

  // Select 與 AlertDialog 各自依賴不同版本的 Radix dismissable-layer，兩邊都以為自己在最上層：
  // 不擋的話一次 Esc 會同時關掉選單與對話框，且 body 的 pointer-events 停在 none（整頁點不動）。
  it("選單開著時按 Esc 只收起選單、對話框保留；再按 Esc 才關對話框，頁面恢復可點", async () => {
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    render(<Fixture />);
    await user.click(screen.getByRole("combobox", { name: "選項" }));
    await screen.findByRole("listbox");
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("listbox")).toBeNull();
    expect(screen.getByRole("alertdialog")).toBeTruthy();
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(document.body.style.pointerEvents).toBe("");
  });
});

// 掛載點改以 callback ref 取 content 節點後，呼叫端傳進來的 ref 仍要收到同一個節點
// （server-web-detail-pages 任務 1.3 的 sharp-edges 檢查：函式 ref 與物件 ref 兩種形式）。
describe("AlertDialogContent 的外部 ref", () => {
  function Dialog({ contentRef }: { contentRef: Ref<HTMLDivElement> }) {
    return (
      <AlertDialog open>
        <AlertDialogContent ref={contentRef} aria-describedby={undefined}>
          <AlertDialogTitle>對話框</AlertDialogTitle>
        </AlertDialogContent>
      </AlertDialog>
    );
  }

  it("物件 ref 收到對話框節點", () => {
    const ref = createRef<HTMLDivElement>();
    render(<Dialog contentRef={ref} />);
    expect(ref.current).toBe(screen.getByRole("alertdialog"));
  });

  it("函式 ref 收到對話框節點，卸載時收到 null", () => {
    const calls: (HTMLDivElement | null)[] = [];
    const { unmount } = render(<Dialog contentRef={(node) => void calls.push(node)} />);
    expect(calls.at(-1)).toBe(screen.getByRole("alertdialog"));
    unmount();
    expect(calls.at(-1)).toBeNull();
  });
});
