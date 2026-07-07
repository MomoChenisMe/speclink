import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, waitFor } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";

// 既有中文斷言包 I18nProvider locale zh-TW 後照舊斷言（i18n 抽 key 的回歸保護）。
const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

import { ChangeListItem } from "../components/ChangeListItem";
import type { ChangeItem } from "../adapter";

// design D3：清單展開元件的內容失效契約——不做一次性快取、掛刷新世代。

const change: ChangeItem = {
  name: "demo-change",
  status: "in-progress",
  totalTasks: 5,
  completedTasks: 2,
};

function makeProps(over: Record<string, unknown> = {}) {
  return {
    change,
    expanded: true,
    onToggle: vi.fn(),
    loadDocument: vi.fn(async (artifact: string) => `# doc for ${artifact}`),
    loadCapabilities: vi.fn(async () => [] as string[]),
    ...over,
  };
}

const docCalls = (props: ReturnType<typeof makeProps>) =>
  (props.loadDocument as ReturnType<typeof vi.fn>).mock.calls.length;

describe("ChangeListItem 內容失效契約", () => {
  it("收合後再展開重新抓取文件（一次性快取移除，檔案為真相）", async () => {
    const props = makeProps();
    const { rerender } = render(<ChangeListItem {...(props as never)} />);
    await waitFor(() => expect(docCalls(props)).toBeGreaterThan(0));
    const c0 = docCalls(props);
    rerender(<ChangeListItem {...(props as never)} expanded={false} />);
    rerender(<ChangeListItem {...(props as never)} expanded={true} />);
    await waitFor(() => expect(docCalls(props)).toBeGreaterThan(c0));
  });

  it("展開中 refreshGen 遞增即重載已載入的文件", async () => {
    const props = makeProps();
    const { rerender } = render(<ChangeListItem {...(props as never)} refreshGen={0} />);
    await waitFor(() => expect(docCalls(props)).toBeGreaterThan(0));
    const c0 = docCalls(props);
    rerender(<ChangeListItem {...(props as never)} refreshGen={1} />);
    await waitFor(() => expect(docCalls(props)).toBeGreaterThan(c0));
  });

  it("收合狀態下不載入，世代遞增亦不觸發抓取", async () => {
    const props = makeProps({ expanded: false });
    const { rerender } = render(<ChangeListItem {...(props as never)} refreshGen={0} />);
    await new Promise((r) => setTimeout(r, 20));
    expect(docCalls(props)).toBe(0);
    rerender(<ChangeListItem {...(props as never)} refreshGen={1} />);
    await new Promise((r) => setTimeout(r, 20));
    expect(docCalls(props)).toBe(0);
  });
});
