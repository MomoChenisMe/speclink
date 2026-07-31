// spec「進行中變更可自看板退回提案中」的守門對話框:渲染引擎證據(已勾任務數
// 與 touched 檔案清單)與出路說明;不提供任何清理或強制退回的機械出路。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { RevertBlockedDialog } from "../components/RevertBlockedDialog";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const info = {
  change: "oops-started",
  checkedTasks: 3,
  touchedFiles: ["src/a.rs", "src/b.ts"],
};

describe("RevertBlockedDialog", () => {
  it("renders the engine evidence and the ways out (uncheck-and-retry / ask the agent)", () => {
    render(<RevertBlockedDialog info={info} onClose={vi.fn()} />);
    const dialog = screen.getByRole("alertdialog");
    expect(dialog.textContent).toContain("oops-started");
    expect(dialog.textContent).toContain("3");
    expect(dialog.textContent).toContain("src/a.rs");
    expect(dialog.textContent).toContain("src/b.ts");
    // 出路說明:已勾任務可於任務分頁取消後重試;touched 需請 agent 判斷。
    expect(dialog.textContent).toContain("取消勾選");
    expect(dialog.textContent).toContain("agent");
  });

  it("offers no cleanup or force-revert button — the single button just closes", () => {
    const onClose = vi.fn();
    render(<RevertBlockedDialog info={info} onClose={onClose} />);
    const dialog = screen.getByRole("alertdialog");
    const buttons = within(dialog).getAllByRole("button");
    expect(buttons).toHaveLength(1);
    fireEvent.click(buttons[0]);
    expect(onClose).toHaveBeenCalled();
  });

  it("renders each evidence block only when present (checked-only case hides the file list)", () => {
    render(
      <RevertBlockedDialog
        info={{ change: "only-tasks", checkedTasks: 2, touchedFiles: [] }}
        onClose={vi.fn()}
      />,
    );
    const dialog = screen.getByRole("alertdialog");
    expect(dialog.textContent).toContain("2");
    expect(dialog.textContent).toContain("取消勾選");
    expect(dialog.textContent).not.toContain("agent");
  });

  it("null info renders nothing", () => {
    render(<RevertBlockedDialog info={null} onClose={vi.fn()} />);
    expect(screen.queryByRole("alertdialog")).toBeNull();
  });
});
