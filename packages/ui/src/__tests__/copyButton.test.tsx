// 共用複製鈕（spec desktop-app「共用元件唯一來源」Scenario「系統匣面板的複製鈕來自
// 共用元件庫」；design D8）：預設寫 navigator.clipboard、onCopy 可覆寫，成功後主色勾號
// 1.5 秒，失敗不拋錯也不呈勾號。
import { describe, it, expect, vi, afterEach } from "vitest";
import { act, cleanup, fireEvent, render as rtlRender, screen } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { CopyButton, REVEAL_ON_HOVER } from "../components/CopyButton";
import { REVEAL_ON_GROUP_HOVER } from "../reveal";

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

  it("成功後標上 data-copied，呼叫端可據此讓 hover 才顯現的鈕保持可見", () => {
    render(<CopyButton value="demo" label="複製名稱" onCopy={() => {}} />);
    expect(btn().hasAttribute("data-copied")).toBe(false);
    fireEvent.click(btn());
    expect(btn().hasAttribute("data-copied")).toBe(true);
  });

  // 滑過才出現的共用基底（複製鈕、server 撤銷鈕）：所在列滑過或鍵盤聚焦時出現，觸控裝置恆常顯示。
  it("REVEAL_ON_HOVER 以共用基底組成：列滑過、鍵盤聚焦、觸控恆顯，複製後保持可見", () => {
    const base = REVEAL_ON_GROUP_HOVER.split(" ");
    expect(base).toEqual(
      expect.arrayContaining([
        "opacity-0",
        "group-hover:opacity-100",
        "focus-visible:opacity-100",
        "[@media(hover:none)]:opacity-100",
      ]),
    );
    const copy = REVEAL_ON_HOVER.split(" ");
    expect(copy).toEqual(expect.arrayContaining([...base, "data-[copied]:opacity-100"]));
  });

  // spec desktop-app「變更與討論詳情頁的宿主為看板」Scenario「宿主頁不推動詳情頁外框」：
  // absolute 的朗讀區不釘位置會停在捲動區外的深處，撐高 desktop 的 main。
  // 這是 class 字面守門（jsdom 不算版面），版面本身由該 Scenario 的 Mac 實機手動任務確認。
  it("隱藏朗讀區釘在定位容器左上角（sr-only top-0 left-0）", () => {
    render(<CopyButton value="demo" label="複製名稱" />);
    const live = screen.getByRole("status");
    expect(live.classList).toContain("sr-only");
    expect(live.classList).toContain("top-0");
    expect(live.classList).toContain("left-0");
  });
});
