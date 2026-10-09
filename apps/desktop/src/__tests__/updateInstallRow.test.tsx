// 軟體更新卡的「下載與安裝」列（desktop-app spec「桌面自動更新」；desktop-notice-relocation
// design D3）：待同意＝目標版本＋「下載」、下載中＝不確定進度列＋「取消」、待重啟＝「更新已就緒」
// ＋「安裝並重新啟動」、失敗＝錯誤＋「重試」；其他狀態不出現。原 updateBanner.test.tsx 的案例搬到這裡。
import { describe, expect, it, vi } from "vitest";
import { fireEvent, render as rtlRender, screen } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider, SEMANTIC_TONE } from "@speclink/ui";

import { UpdateInstallRow, type UpdateInstallRowProps } from "../components/UpdateInstallRow";
import type { UpdaterState } from "../core/updater";
import { APP_MESSAGES } from "../i18n/messages";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

function props(state: UpdaterState): UpdateInstallRowProps {
  return { state, onAccept: vi.fn(), onCancel: vi.fn(), onRelaunch: vi.fn(), onRetry: vi.fn() };
}
const row = () => screen.getByTestId("update-install-row");
const button = (name: string) => screen.getByRole("button", { name }) as HTMLButtonElement;

describe("UpdateInstallRow", () => {
  it("待同意：顯示目標版本與「下載」；同意才回呼下載、不觸發重啟", () => {
    const p = props({ phase: "available", version: "0.2.0" });
    render(<UpdateInstallRow {...p} />);
    expect(row().textContent).toContain("0.2.0");
    expect(screen.getByText(/有新版本/).className).toContain(SEMANTIC_TONE.inProgress);
    expect(screen.queryByRole("progressbar")).toBeNull();
    fireEvent.click(button("下載"));
    expect(p.onAccept).toHaveBeenCalledTimes(1);
    expect(p.onRelaunch).not.toHaveBeenCalled();
    expect(p.onCancel).not.toHaveBeenCalled();
  });

  it("下載中：不確定進度列（無 aria-valuenow）與「取消」；取消回呼 onCancel", () => {
    const p = props({ phase: "downloading", version: "0.2.0" });
    render(<UpdateInstallRow {...p} />);
    expect(row().textContent).toContain("0.2.0");
    const bar = screen.getByRole("progressbar");
    expect(bar.getAttribute("aria-valuenow")).toBeNull();
    fireEvent.click(button("取消"));
    expect(p.onCancel).toHaveBeenCalledTimes(1);
    expect(p.onAccept).not.toHaveBeenCalled();
  });

  it("待重啟：「更新已就緒」與「安裝並重新啟動」", () => {
    const p = props({ phase: "restartPending", version: "0.2.0" });
    render(<UpdateInstallRow {...p} />);
    expect(row().textContent).toContain("更新已就緒");
    expect(screen.queryByRole("progressbar")).toBeNull();
    fireEvent.click(button("安裝並重新啟動"));
    expect(p.onRelaunch).toHaveBeenCalledTimes(1);
  });

  it("失敗：紅字錯誤訊息與「重試」；重試回呼 onRetry（重新檢查）", () => {
    const p = props({ phase: "error", message: "invalid signature" });
    render(<UpdateInstallRow {...p} />);
    const message = screen.getByText(/invalid signature/);
    expect(message.className).toContain("destructive");
    fireEvent.click(button("重試"));
    expect(p.onRetry).toHaveBeenCalledTimes(1);
    expect(p.onAccept).not.toHaveBeenCalled();
  });

  it.each<[string, UpdaterState]>([
    ["閒置", { phase: "idle" }],
    ["檢查中", { phase: "checking", manual: true }],
    ["已是最新", { phase: "upToDate" }],
    ["檢查失敗", { phase: "checkFailed" }],
  ])("%s：該列不出現", (_label, state) => {
    render(<UpdateInstallRow {...props(state)} />);
    expect(screen.queryByTestId("update-install-row")).toBeNull();
  });
});
