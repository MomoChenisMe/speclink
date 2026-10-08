// 共用確認框（spec desktop-app「共用元件唯一來源」；design D7）：兩鍵式確認框的
// 標題、說明、文案、危險變體與 busy／disabled 由呼叫端以屬性提供，外觀只此一份。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { ConfirmDialog } from "../components/ui/confirm-dialog";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const base = {
  open: true,
  onOpenChange: () => {},
  title: "封存變更",
  description: "確定要封存 demo？",
  confirmLabel: "封存",
  onConfirm: () => {},
};

const button = (name: string) => screen.getByRole("button", { name }) as HTMLButtonElement;

describe("ConfirmDialog", () => {
  it("渲染標題、說明與兩鍵文案；取消預設取 i18n common.cancel", () => {
    render(<ConfirmDialog {...base} />);
    const dialog = screen.getByRole("alertdialog");
    expect(dialog.textContent).toContain("封存變更");
    expect(dialog.textContent).toContain("確定要封存 demo？");
    expect(button("封存")).toBeTruthy();
    expect(button("取消")).toBeTruthy();
  });

  it("cancelLabel 覆寫取消文案", () => {
    render(<ConfirmDialog {...base} cancelLabel="先不要" />);
    expect(button("先不要")).toBeTruthy();
  });

  it("children 渲染在說明之後、按鈕之前", () => {
    render(
      <ConfirmDialog {...base}>
        <p>額外內容</p>
      </ConfirmDialog>,
    );
    const extra = screen.getByText("額外內容");
    const desc = screen.getByText("確定要封存 demo？");
    const confirm = button("封存");
    expect(desc.compareDocumentPosition(extra) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(extra.compareDocumentPosition(confirm) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });

  it("destructive 變體的確認鈕為實心紅，一般為主色", () => {
    const { unmount } = render(<ConfirmDialog {...base} destructive />);
    expect(button("封存").className).toContain("bg-destructive");
    expect(button("封存").className).not.toContain("bg-primary");
    unmount();
    render(<ConfirmDialog {...base} />);
    expect(button("封存").className).toContain("bg-primary");
    expect(button("封存").className).not.toContain("bg-destructive");
  });

  it("busy 時兩鍵皆 disabled", () => {
    render(<ConfirmDialog {...base} busy />);
    expect(button("封存").disabled).toBe(true);
    expect(button("取消").disabled).toBe(true);
  });

  it("confirmDisabled 只停用確認鈕，取消照常可按", () => {
    render(<ConfirmDialog {...base} confirmDisabled />);
    expect(button("封存").disabled).toBe(true);
    expect(button("取消").disabled).toBe(false);
  });

  it("按確認觸發 onConfirm 並關閉；按取消只關閉", () => {
    const onConfirm = vi.fn();
    const onOpenChange = vi.fn();
    render(<ConfirmDialog {...base} onConfirm={onConfirm} onOpenChange={onOpenChange} />);
    fireEvent.click(button("封存"));
    expect(onConfirm).toHaveBeenCalledTimes(1);
    expect(onOpenChange).toHaveBeenLastCalledWith(false);
    onOpenChange.mockClear();
    fireEvent.click(button("取消"));
    expect(onConfirm).toHaveBeenCalledTimes(1);
    expect(onOpenChange).toHaveBeenLastCalledWith(false);
  });
});
