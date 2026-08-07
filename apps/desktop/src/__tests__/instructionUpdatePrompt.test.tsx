// 指令檔提示橫幅（desktop-app spec「指令檔過期提示」，決策 7）：分頁內容頂部
// 非阻斷呈現，主動作依探測態分文案（過期→更新、缺失→安裝）＋保留現狀；
// 更新失敗錯誤於原位呈現且可重試。
import { describe, expect, it, vi } from "vitest";
import { fireEvent, render as rtlRender, screen } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider, SEMANTIC_TONE } from "@speclink/ui";

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
const NEWER = { kind: "newer" as const, fileCount: 2, version: "v1.3.0" };

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

  it("較新態：以「app 是舊版」語意呈現，只留保留現狀、無任何改寫動作", () => {
    const h = handlers();
    render(<InstructionUpdatePrompt prompt={NEWER} error={null} busy={false} {...h} />);

    const banner = screen.getByTestId("instruction-prompt");
    expect(banner.textContent).toContain("比你的 Speclink 新");
    expect(screen.queryByRole("button", { name: "更新" })).toBeNull();
    expect(screen.queryByRole("button", { name: "安裝" })).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "保留現狀" }));
    expect(h.onDismiss).toHaveBeenCalledTimes(1);
    expect(h.onApply).not.toHaveBeenCalled();
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

  it("捲動釘選：黏在可視區頂部、底不透明、層級高於捲過的內容", () => {
    // spec「指令檔過期提示捲動釘選」：一捲動就消失等於沒提示；釘住的同時底必須
    // 不透明，否則下層內容會透出來疊字。
    const h = handlers();
    render(<InstructionUpdatePrompt prompt={STALE} error={null} busy={false} {...h} />);

    const banner = screen.getByTestId("instruction-prompt");
    expect(banner.className).toContain("sticky");
    expect(banner.className).toContain("top-0");
    expect(banner.className).toMatch(/\bz-\d+\b/);
    // 不透明底：半透明（bg-muted/40 之類的 /透明度 後綴）會透字。
    expect(banner.className).toContain("bg-muted");
    expect(banner.className).not.toMatch(/bg-muted\//);
  });

  it("底色中性、狀態由圖示語意色承載：過期為琥珀警示、套用失敗為紅", () => {
    const h = handlers();
    const { unmount } = render(
      <InstructionUpdatePrompt prompt={STALE} error={null} busy={false} {...h} />,
    );
    const stale = screen.getByTestId("instruction-prompt");
    expect(stale.className).not.toContain("bg-primary");
    expect(stale.querySelector("svg")?.getAttribute("class")).toContain(SEMANTIC_TONE.warning);
    unmount();

    render(<InstructionUpdatePrompt prompt={STALE} error={"寫入失敗"} busy={false} {...h} />);
    // 套用失敗是錯誤，不是警示——與過期提示分色才看得出嚴重度差別。
    const failed = screen.getByTestId("instruction-prompt");
    expect(failed.querySelector("svg")?.getAttribute("class")).toContain("destructive");
    expect(screen.getByText(/寫入失敗/).className).toContain("destructive");
  });

  it("無提示時不渲染任何內容", () => {
    const h = handlers();
    render(<InstructionUpdatePrompt prompt={null} error={null} busy={false} {...h} />);
    expect(screen.queryByTestId("instruction-prompt")).toBeNull();
  });
});
