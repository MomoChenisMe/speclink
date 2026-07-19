import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import { toast, type ToasterProps } from "sonner";

import { I18nProvider } from "../i18n";
import { Toaster } from "../components/ui/sonner";

const TOAST_ID = "desktop-failure";

beforeAll(() => {
  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    value: vi.fn((query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      addListener: vi.fn(),
      removeListener: vi.fn(),
      dispatchEvent: vi.fn(() => false),
    })),
  });
});

function renderToaster(props: ToasterProps = {}) {
  return render(
    <I18nProvider locale="zh-TW">
      <Toaster {...props} />
    </I18nProvider>,
  );
}

afterEach(() => {
  act(() => toast.dismiss());
  vi.useRealTimers();
});

describe("Toaster", () => {
  it("沿用主介面 tokens，並以 destructive 局部標示 error toast", async () => {
    renderToaster();

    act(() => {
      toast.error("設計 token 失敗", { id: TOAST_ID });
    });

    const toastNode = (await screen.findByText("設計 token 失敗")).closest("[data-sonner-toast]");
    const toaster = toastNode?.closest<HTMLElement>("[data-sonner-toaster]");
    expect(toaster).toBeTruthy();
    expect(toaster?.style.getPropertyValue("--normal-bg")).toBe("var(--card)");
    expect(toaster?.style.getPropertyValue("--normal-text")).toBe("var(--card-foreground)");
    expect(toaster?.style.getPropertyValue("--normal-border")).toBe("var(--border)");
    expect(toaster?.style.getPropertyValue("--border-radius")).toBe("var(--radius)");
    expect(toaster?.style.fontFamily).toBe("inherit");

    expect(toastNode?.getAttribute("data-rich-colors")).toBeNull();
    expect(toastNode?.className).toContain("!shadow-lg");
    expect(toastNode?.className).toContain("!border-destructive/40");
    expect(toastNode?.className).toContain("[&_[data-icon]]:text-destructive");
    expect(toastNode?.className).not.toContain("bg-destructive");

    const closeButton = screen.getByRole("button", { name: "關閉通知" });
    expect(closeButton.className).toContain("!bg-card");
    expect(closeButton.className).toContain("!border-border");
    expect(closeButton.className).toContain("!text-muted-foreground");
  });

  it("合併呼叫端 root style、toast style 與逐型別 classNames", async () => {
    renderToaster({
      style: { letterSpacing: "0.123em" },
      toastOptions: {
        style: { opacity: 0.75 },
        classNames: {
          toast: "consumer-toast",
          error: "consumer-error",
        },
      },
    });

    act(() => {
      toast.error("自訂樣式失敗", { id: TOAST_ID });
    });

    const toastNode = (await screen.findByText("自訂樣式失敗")).closest<HTMLElement>(
      "[data-sonner-toast]",
    );
    const toaster = toastNode?.closest<HTMLElement>("[data-sonner-toaster]");
    expect(toaster?.style.letterSpacing).toBe("0.123em");
    expect(toaster?.style.getPropertyValue("--normal-bg")).toBe("var(--card)");
    expect(toastNode?.style.opacity).toBe("0.75");
    expect(toastNode?.className).toContain("!shadow-lg");
    expect(toastNode?.className).toContain("consumer-toast");
    expect(toastNode?.className).toContain("!border-destructive/40");
    expect(toastNode?.className).toContain("consumer-error");
  });

  it("顯示訊息與關閉鈕，且同 id 的新訊息取代舊訊息", async () => {
    renderToaster();

    act(() => {
      toast.error("第一則失敗", { id: TOAST_ID });
    });
    expect(await screen.findByText("第一則失敗")).toBeTruthy();
    expect(screen.getByRole("button", { name: "關閉通知" })).toBeTruthy();

    act(() => {
      toast.error("第二則失敗", { id: TOAST_ID });
    });
    expect(await screen.findByText("第二則失敗")).toBeTruthy();
    expect(screen.queryByText("第一則失敗")).toBeNull();
  });

  it("6000 毫秒後自動消失", async () => {
    vi.useFakeTimers();
    renderToaster();

    act(() => {
      toast.error("逾時失敗", { id: TOAST_ID });
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    const toastNode = screen.getByText("逾時失敗").closest("[data-sonner-toast]");
    expect(toastNode).toBeTruthy();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(6000);
    });
    expect(toastNode?.getAttribute("data-removed")).toBe("true");

    await act(async () => {
      await vi.advanceTimersByTimeAsync(200);
    });
    expect(screen.queryByText("逾時失敗")).toBeNull();
  });

  it("點按關閉鈕立即消失", async () => {
    renderToaster();

    act(() => {
      toast.error("手動關閉失敗", { id: TOAST_ID });
    });
    fireEvent.click(await screen.findByRole("button", { name: "關閉通知" }));

    await waitFor(() => {
      expect(screen.queryByText("手動關閉失敗")).toBeNull();
    });
  });

  it("HTML-like 訊息以純文字呈現，不建立可執行元素", async () => {
    renderToaster();
    const htmlLikeMessage = '<img src="x" onerror="globalThis.pwned=true">';

    act(() => {
      toast.error(htmlLikeMessage, { id: TOAST_ID });
    });

    expect(await screen.findByText(htmlLikeMessage)).toBeTruthy();
    expect(document.querySelector("img")).toBeNull();
  });
});
