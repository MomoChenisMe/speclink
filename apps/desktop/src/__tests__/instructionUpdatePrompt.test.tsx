// 指令檔提示橫幅（desktop-app spec「指令檔過期提示」，決策 7）：分頁內容頂部
// 非阻斷呈現，主動作依探測態分文案（過期→更新、缺失→安裝）＋保留現狀；
// 更新失敗錯誤於原位呈現且可重試。
import { describe, expect, it, vi } from "vitest";
import { fireEvent, render as rtlRender, screen } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { InstructionUpdatePrompt } from "../components/InstructionUpdatePrompt";
import { APP_MESSAGES } from "../i18n/messages";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);

function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

function handlers() {
  return { onApply: vi.fn(), onDismiss: vi.fn() };
}

const STALE = { kind: "stale" as const, fileCount: 3, version: "v1.3.0" };
const MISSING = { kind: "missing" as const, fileCount: 12, version: "v1.3.0" };

describe("InstructionUpdatePrompt", () => {
  it("過期態：顯示將被改寫的檔案數，主動作為「更新」", () => {
    const h = handlers();
    render(<InstructionUpdatePrompt prompt={STALE} error={null} busy={false} {...h} />);

    const banner = screen.getByTestId("instruction-prompt");
    expect(banner.textContent).toContain("3");
    fireEvent.click(screen.getByRole("button", { name: "更新" }));
    expect(h.onApply).toHaveBeenCalledTimes(1);
  });

  it("缺失態：主動作為「安裝」（從未安裝的專案不以更新稱之）", () => {
    const h = handlers();
    render(<InstructionUpdatePrompt prompt={MISSING} error={null} busy={false} {...h} />);

    expect(screen.queryByRole("button", { name: "更新" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "安裝" }));
    expect(h.onApply).toHaveBeenCalledTimes(1);
  });

  it("保留現狀：回呼 dismiss、不觸發再生", () => {
    const h = handlers();
    render(<InstructionUpdatePrompt prompt={STALE} error={null} busy={false} {...h} />);

    fireEvent.click(screen.getByRole("button", { name: "保留現狀" }));
    expect(h.onDismiss).toHaveBeenCalledTimes(1);
    expect(h.onApply).not.toHaveBeenCalled();
  });

  it("失敗：錯誤於提示原位呈現，主動作仍可重試", () => {
    const h = handlers();
    render(
      <InstructionUpdatePrompt
        prompt={STALE}
        error="CLAUDE.md: permission denied"
        busy={false}
        {...h}
      />,
    );

    const banner = screen.getByTestId("instruction-prompt");
    expect(banner.textContent).toContain("permission denied");
    fireEvent.click(screen.getByRole("button", { name: "更新" }));
    expect(h.onApply).toHaveBeenCalledTimes(1);
  });

  it("進行中：主動作停用，避免重複觸發", () => {
    const h = handlers();
    render(<InstructionUpdatePrompt prompt={STALE} error={null} busy {...h} />);

    const apply = screen.getByRole("button", { name: "更新" }) as HTMLButtonElement;
    expect(apply.disabled).toBe(true);
    fireEvent.click(apply);
    expect(h.onApply).not.toHaveBeenCalled();
  });

  it("非阻斷：無 dialog 語意、不遮蔽分頁內容", () => {
    const h = handlers();
    render(<InstructionUpdatePrompt prompt={STALE} error={null} busy={false} {...h} />);

    expect(screen.queryByRole("dialog")).toBeNull();
    expect(screen.getByTestId("instruction-prompt").getAttribute("role")).toBe("status");
  });

  it("無提示時不渲染任何內容", () => {
    const h = handlers();
    render(<InstructionUpdatePrompt prompt={null} error={null} busy={false} {...h} />);
    expect(screen.queryByTestId("instruction-prompt")).toBeNull();
  });
});
