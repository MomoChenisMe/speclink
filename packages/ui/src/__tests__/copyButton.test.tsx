// 共用複製鈕（spec desktop-app「共用元件唯一來源」Scenario「系統匣面板的複製鈕來自
// 共用元件庫」；design D8）：預設寫 navigator.clipboard、onCopy 可覆寫，成功後主色勾號
// 1.5 秒，失敗不拋錯也不呈勾號。
import { describe, it, expect, vi, afterEach } from "vitest";
import { act, cleanup, fireEvent, render as rtlRender, screen } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { CopyButton } from "../components/CopyButton";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const writeText = vi.fn<(v: string) => Promise<void>>();

afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.unstubAllGlobals();
  writeText.mockReset();
});

const btn = () => screen.getByRole("button", { name: "複製名稱" });
const hasCheck = () => btn().querySelector("svg.lucide-check") !== null;
/** 讓 promise 的 then 跑完（假計時器下也適用）。 */
const flush = () => act(async () => {});

describe("CopyButton", () => {
  it("預設寫入 navigator.clipboard，成功後呈勾號並以 status 宣告已複製", async () => {
    writeText.mockResolvedValue();
    vi.stubGlobal("navigator", { clipboard: { writeText } });
    render(<CopyButton value="demo" label="複製名稱" />);
    expect(hasCheck()).toBe(false);
    fireEvent.click(btn());
    await flush();
    expect(writeText).toHaveBeenCalledWith("demo");
    expect(hasCheck()).toBe(true);
    expect(screen.getByRole("status").textContent).toBe("已複製");
  });

  it("onCopy 覆寫剪貼簿路徑：以 value 呼叫、不碰 navigator.clipboard", async () => {
    vi.stubGlobal("navigator", { clipboard: { writeText } });
    const onCopy = vi.fn();
    render(<CopyButton value="demo" label="複製名稱" onCopy={onCopy} />);
    fireEvent.click(btn());
    await flush();
    expect(onCopy).toHaveBeenCalledWith("demo");
    expect(writeText).not.toHaveBeenCalled();
    expect(hasCheck()).toBe(true);
  });

  it("成功後勾號維持 1.5 秒內、之後復原", () => {
    vi.useFakeTimers();
    render(<CopyButton value="demo" label="複製名稱" onCopy={() => {}} />);
    fireEvent.click(btn());
    expect(hasCheck()).toBe(true);
    act(() => {
      vi.advanceTimersByTime(1400);
    });
    expect(hasCheck()).toBe(true);
    act(() => {
      vi.advanceTimersByTime(200);
    });
    expect(hasCheck()).toBe(false);
  });

  it("失敗不拋錯、不呈勾號：onCopy reject", async () => {
    render(
      <CopyButton value="demo" label="複製名稱" onCopy={() => Promise.reject(new Error("denied"))} />,
    );
    fireEvent.click(btn());
    await flush();
    expect(hasCheck()).toBe(false);
  });

  it("失敗不拋錯、不呈勾號：沒有 clipboard API", async () => {
    vi.stubGlobal("navigator", {});
    render(<CopyButton value="demo" label="複製名稱" />);
    expect(() => fireEvent.click(btn())).not.toThrow();
    await flush();
    expect(hasCheck()).toBe(false);
  });

  it("寫入完成前元件已卸載：完成後不再開計時器", async () => {
    vi.useFakeTimers();
    let resolve!: () => void;
    const pending = () =>
      new Promise<void>((r) => {
        resolve = r;
      });
    const { unmount } = render(<CopyButton value="demo" label="複製名稱" onCopy={pending} />);
    fireEvent.click(btn());
    unmount();
    await act(async () => resolve());
    expect(vi.getTimerCount()).toBe(0);
  });

  it("點擊不冒泡到外層（列本體的開啟不被觸發）", () => {
    const onRow = vi.fn();
    render(
      <div onClick={onRow}>
        <CopyButton value="demo" label="複製名稱" onCopy={() => {}} />
      </div>,
    );
    fireEvent.click(btn());
    expect(onRow).not.toHaveBeenCalled();
  });

  it("size=sm 呈外框小鈕與「複製／已複製」文字", async () => {
    render(<CopyButton value="demo" label="複製名稱" size="sm" onCopy={() => {}} />);
    expect(btn().textContent).toContain("複製");
    expect(btn().className).toContain("border");
    fireEvent.click(btn());
    await flush();
    expect(btn().textContent).toContain("已複製");
  });

  it("inverted：放在 hover 整列主色反白的列裡，hover 時圖示改用主色前景色；預設不帶", () => {
    const { rerender } = render(<CopyButton value="demo" label="複製名稱" onCopy={() => {}} />);
    expect(btn().className).not.toContain("group-hover:text-primary-foreground");
    rerender(<CopyButton value="demo" label="複製名稱" onCopy={() => {}} inverted />);
    expect(btn().className).toContain("group-hover:text-primary-foreground");
  });

  it("成功後標上 data-copied，呼叫端可據此讓 hover 才顯現的鈕保持可見", () => {
    render(<CopyButton value="demo" label="複製名稱" onCopy={() => {}} />);
    expect(btn().hasAttribute("data-copied")).toBe(false);
    fireEvent.click(btn());
    expect(btn().hasAttribute("data-copied")).toBe(true);
  });
});
